use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt;
use hashbrown::HashMap;
use shim::{io, newioerr};

use crate::{traits::BlockDevice, util::SliceExt};

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    dirty: bool,
}

pub struct Partition {
    /// The physical sector where the partition begins.
    pub start: u64,
    /// Number of sectors
    pub num_sectors: u64,
    /// The size, in bytes, of a logical sector in the partition.
    pub sector_size: u64,
}

pub struct CachedPartition {
    device: Box<dyn BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
    partition: Partition,
    // Add dedicated line buffer
    cache_line_buffer: Vec<u32>,
}

impl CachedPartition {
    /// Creates a new `CachedPartition` that transparently caches sectors from
    /// `device` and maps physical sectors to logical sectors inside of
    /// `partition`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    ///
    /// The `partition` parameter determines the size of a logical sector and
    /// where logical sectors begin. An access to a sector `0` will be
    /// translated to physical sector `partition.start`. Virtual sectors of
    /// sector number `[0, num_sectors)` are accessible.
    ///
    /// `partition.sector_size` must be an integer multiple of
    /// `device.sector_size()`.
    ///
    /// # Panics
    ///
    /// Panics if the partition's sector size is < the device's sector size.
    pub fn new<T>(device: T, partition: Partition) -> CachedPartition
    where
        T: BlockDevice + 'static,
    {
        assert!(partition.sector_size >= device.sector_size());

        CachedPartition {
            device: Box::new(device),
            cache: HashMap::new(),
            partition: partition,
            cache_line_buffer: Vec::with_capacity(128),
        }
    }

    /// Returns the number of physical sectors that corresponds to
    /// one logical sector.
    fn factor(&self) -> u64 {
        self.partition.sector_size / self.device.sector_size()
    }

    /// Maps a user's request for a sector `virt` to the physical sector.
    /// Returns `None` if the virtual sector number is out of range.
    fn virtual_to_physical(&self, virt: u64) -> Option<u64> {
        if virt >= self.partition.num_sectors {
            return None;
        }

        let physical_offset = virt * self.factor();
        let physical_sector = self.partition.start + physical_offset;

        Some(physical_sector)
    }

    /// Loads buffer with the data available at the given sector
    ///
    /// Will throw an IO error if sector id's are bad
    ///
    /// Disk 2 cache
    fn load_sector(&mut self, buf: &mut Vec<u8>, sector: u64) -> io::Result<()> {
        buf.clear();
        buf.reserve(self.partition.sector_size as usize);

        let phy_id = self
            .virtual_to_physical(sector)
            .ok_or(io::ErrorKind::InvalidInput)?;

        // this line buffer storage is always aligned and available. Multi threading dangerous
        let mut line_buffer: Vec<u8> =
            unsafe { Vec::from_raw_parts(self.cache_line_buffer.as_mut_ptr() as *mut u8, 0, 512) };

        for i in 0..self.factor() {
            self.device.read_all_sector(phy_id + i, &mut line_buffer)?;
        }

        Ok(())
    }

    /// Grabs a Cache Entry from the cache, If non exists it loads it from
    /// the cache
    ///
    /// Cache 2 mem
    fn get_entry(&mut self, sector: u64) -> io::Result<&mut CacheEntry> {
        if let None = self.cache.get_mut(&sector) {
            let mut buf: Vec<u8> = Vec::new();
            self.load_sector(&mut buf, sector)?;

            self.cache.insert(
                sector,
                CacheEntry {
                    data: buf,
                    dirty: false,
                },
            );
        }

        // safe to unwrap because load sector would have errored
        Ok(self.cache.get_mut(&sector).unwrap())
    }

    /// Returns a mutable reference to the cached sector `sector`. If the sector
    /// is not already cached, the sector is first read from the disk.
    ///
    /// The sector is marked dirty as a result of calling this method as it is
    /// presumed that the sector will be written to. If this is not intended,
    /// use `get()` instead.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get_mut(&mut self, sector: u64) -> io::Result<&mut [u8]> {
        self.get_entry(sector).map(|entry| {
            entry.dirty = true;
            entry.data.as_mut_slice()
        })
    }

    /// Returns a reference to the cached sector `sector`. If the sector is not
    /// already cached, the sector is first read from the disk.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get(&mut self, sector: u64) -> io::Result<&[u8]> {
        self.get_entry(sector).map(|entry| entry.data.as_slice())
    }
}

// FIXME: Implement `BlockDevice` for `CacheDevice`. The `read_sector` and
// `write_sector` methods should only read/write from/to cached sectors.
impl BlockDevice for CachedPartition {
    fn sector_size(&self) -> u64 {
        self.sector_size()
    }

    fn read_sector(&mut self, sector: u64, buf: &mut [u8]) -> io::Result<usize> {
        match self.get(sector) {
            Ok(read_sector) => {
                let amt = core::cmp::min(read_sector.len(), buf.len());
                buf[..amt].copy_from_slice(&read_sector[..amt]);
                Ok(amt)
            }
            Err(e) => Err(e),
        }
    }

    fn write_sector(&mut self, sector: u64, buf: &[u8]) -> io::Result<usize> {
        match self.get_mut(sector) {
            Ok(write_sector) => {
                let amt = core::cmp::min(write_sector.len(), buf.len());
                write_sector[..amt].copy_from_slice(&buf[..amt]);
                Ok(amt)
            }
            Err(e) => Err(e),
        }
    }
}

impl fmt::Debug for CachedPartition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CachedPartition")
            .field("device", &"<block device>")
            .field("cache", &self.cache)
            .finish()
    }
}
