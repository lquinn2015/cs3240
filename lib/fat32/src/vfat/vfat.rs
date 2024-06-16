use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::vec::Vec;

use shim::io;
use shim::ioerr;
use shim::newioerr;
use shim::path;
use shim::path::Path;

use crate::mbr::MasterBootRecord;
use crate::traits::{BlockDevice, FileSystem};
use crate::util::SliceExt;
use crate::vfat::{BiosParameterBlock, CachedPartition, Partition};
use crate::vfat::{Cluster, Dir, Entry, Error, FatEntry, File, Status};

/// A generic trait that handles a critical section as a closure
pub trait VFatHandle: Clone + Debug + Send + Sync {
    fn new(val: VFat<Self>) -> Self;
    fn lock<R>(&self, f: impl FnOnce(&mut VFat<Self>) -> R) -> R;
}

#[derive(Debug)]
pub struct VFat<HANDLE: VFatHandle> {
    phantom: PhantomData<HANDLE>,
    device: CachedPartition,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    rootdir_cluster: Cluster,
}

impl<HANDLE: VFatHandle> VFat<HANDLE> {
    pub fn from<T>(mut device: T) -> Result<HANDLE, Error>
    where
        T: BlockDevice + 'static,
    {
        let mbr = MasterBootRecord::from(device).map_err(|e| Error::Mbr(e))?;
        let partition = mbr.fat32_partition().ok_or(Error::NotFound)?;

        let bpb = BiosParameterBlock::from(&mut device, partition.starting_sector() as u64)?;

        Ok(HANDLE::new(VFat {
            phantom: PhantomData {},
            device: CachedPartition::new(
                device,
                Partition {
                    start: partition.starting_sector() as u64,
                    num_sectors: (partition.num_sectors as u64)
                        / ((bpb.bytes_per_sector as u64) / (device.sector_size() as u64)),
                    sector_size: bpb.bytes_per_sector as u64,
                },
            ),
            bytes_per_sector: bpb.bytes_per_sector,
            sectors_per_cluster: bpb.sectors_per_cluster,
            sectors_per_fat: bpb.sectors_per_fat,
            fat_start_sector: bpb.reserved_sectors as u64,
            data_start_sector: (bpb.reserved_sectors as u64)
                + (bpb.num_fats as u64 * bpb.sectors_per_fat as u64),
            rootdir_cluster: Cluster::from(bpb.root_cluster),
        }))
    }

    fn start_sector(&mut self, curr: Cluster) -> io::Result<u64> {
        if curr.num() < 2 {
            Err(newioerr!(InvalidData, "Unexpected Cluster num"))
        } else {
            Ok(self.data_start_sector as u64
                + (curr.num() - 2) as u64 * self.sectors_per_cluster as u64)
        }
    }

    fn cluster_byte_size(&mut self) -> usize {
        (self.sectors_per_cluster as u64 * self.bytes_per_sector as u64) as usize
    }

    /*
        [-------][------][---------]
        |             .
        |            / \
        |   offset ---|
        |
        cluster

        start_addr =   cluster_addr + offset
        start_sector  = start_addr / bps


    */

    //  * A method to read from an offset of a cluster into a buffer.
    fn read_cluster(
        &mut self,
        cluster: Cluster,
        offset: usize,
        buf: &mut [u8],
    ) -> io::Result<usize> {
        // if offset is larger than this this sector
        if offset >= self.cluster_byte_size() {
            return Ok(0);
        }

        let start_sector = self.start_sector(cluster)?;
        let sec_size = self.bytes_per_sector as u64;
        let end_sector = self.sectors_per_cluster as u64 + start_sector;
        let start_addr = start_sector * sec_size + offset as u64;
        let first_sector = start_addr / sec_size;
        let mut n = 0;

        // reach each sector that has
        for c_sector in first_sector..=end_sector {
            let bytes = self.device.get(c_sector)?;
            if bytes.len() != sec_size as usize {
                return Err(newioerr!(UnexpectedEof, "Missing bytes in sector read"));
            }
            let c_sector_addr = c_sector * sec_size;
            let offset = if c_sector_addr > start_addr {
                (c_sector_addr - start_addr) as usize
            } else {
                0
            };
            let amt2cpy = core::cmp::min(sec_size as usize - offset, buf.len() - n);
            buf[n..n + amt2cpy].copy_from_slice(&bytes[offset..offset + amt2cpy]);

            n += amt2cpy;
            if n >= buf.len() {
                break;
            }
        }

        Ok(n)
    }

    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    fn read_chain(&mut self, start: Cluster, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut n = 0;
        let mut cluster = start;
        loop {
            let start = buf.len();
            buf.resize(start + self.bytes_per_cluster(), 0);
            let bytes = self.read_cluster(cluster, 0, &mut buf.as_mut_slice()[start..])?;
            buf.truncate(start + bytes); // eliminate over read
            n += bytes;

            match self.next_cluster(cluster)? {
                Some(next_cluster) => cluster = next_cluster,
                None => break,
            };
        }
        Ok(n)
    }

    #[inline]
    pub fn bytes_per_cluster(&mut self) -> usize {
        (self.bytes_per_sector as u64 * self.sectors_per_cluster as u64) as usize
    }

    // Will resolve the Option for the next cluster
    fn next_cluster(&mut self, curr: Cluster) -> io::Result<Option<Cluster>> {
        match self.fat_entry(curr)?.status() {
            Status::Data(next_cluster) => Ok(Some(next_cluster)),
            Status::Eoc(_) => Ok(None),
            _ => Err(newioerr!(InvalidData, "Unexpected Cluster type")),
        }
    }

    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        let entry_offset = cluster.num() as u64 * 4;
        let logical_sector = self.fat_start_sector + entry_offset / self.bytes_per_sector as u64;

        // access sector contained entry
        let sector = self.device.get(logical_sector)?;

        // access entry in this sector
        let index = (entry_offset % self.bytes_per_sector as u64) / 4;
        // Safety sector contains a series of FatEntry each is u32  i.e int mult of u8
        let fat_entries: &[FatEntry] = unsafe { sector.cast() };
        Ok(&fat_entries[index as usize])
    }
}

impl<'a, HANDLE: VFatHandle> FileSystem for &'a HANDLE {
    type File = crate::traits::Dummy;
    type Dir = crate::traits::Dummy;
    type Entry = crate::traits::Dummy;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        unimplemented!("FileSystem::open()")
    }
}
