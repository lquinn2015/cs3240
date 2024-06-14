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

    //  * A method to read from an offset of a cluster into a buffer.
    fn read_cluster(
        &mut self,
        cluster: Cluster,
        offset: usize,
        buf: &mut [u8],
    ) -> io::Result<usize> {
    }

    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    fn read_chain(&mut self, start: Cluster, buf: &mut Vec<u8>) -> io::Result<usize> {
        todo!()
    }

    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        todo!()
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
