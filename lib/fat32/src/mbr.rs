use core::fmt;
use core::fmt::Display;
use shim::const_assert_size;
use shim::io;
use shim::newioerr;

use crate::traits::BlockDevice;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct CHS {
    head: u8,
    sector_and_cylinder_start: u16,
}

impl CHS {
    fn start_sector(&self) -> u8 {
        (self.sector_and_cylinder_start & 0x3F) as u8
    }
    fn start_cylinder(&self) -> u16 {
        self.sector_and_cylinder_start >> 6
    }
}
impl fmt::Debug for CHS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CHS")
            .field("sector", &format!("{}", self.start_sector()))
            .finish()
    }
}

const_assert_size!(CHS, 3);

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct PartitionEntry {
    boot_indicator: u8,
    start_chs: CHS,
    partition_type: u8,
    end_chs: CHS,
    sector_offset: u32,
    num_sectors: u32,
}

impl PartitionEntry {
    const BOOTABLE: u8 = 0x80;

    pub fn starting_sector(&self) -> u32 {
        self.sector_offset
    }
}

const_assert_size!(PartitionEntry, 16);

/// The master boot record (MBR).
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MasterBootRecord {
    bootstrap: [u8; 436],
    disk_id: [u8; 10],
    partions: [PartitionEntry; 4],
    magic: [u8; 2],
}

const_assert_size!(MasterBootRecord, 512);

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(io) => write!(f, "{}", io),
            Error::UnknownBootIndicator(val) => write!(f, "UnknownBootIndicator[{}]", val),
            Error::BadSignature => write!(f, "fat32 BadSignature"),
        }
    }
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut dev: T) -> Result<MasterBootRecord, Error> {
        let mut buf: [u8; 512] = [0; 512];

        match dev.read_sector(0, &mut buf) {
            Err(e) => Err(Error::Io(e)),
            Ok(n) => {
                if n != buf.len() {
                    return Err(Error::Io(newioerr!(
                        UnexpectedEof,
                        "Insufficient bytes from sector"
                    )));
                }

                let mbr = unsafe { &*(buf.as_ptr() as *const MasterBootRecord) };
                if mbr.magic != [0x55, 0xaa] {
                    return Err(Error::BadSignature);
                }

                for i in 0..4usize {
                    let indicator = mbr.partions[i].boot_indicator;
                    if indicator != 0x0 && indicator != PartitionEntry::BOOTABLE {
                        return Err(Error::UnknownBootIndicator(i as u8));
                    }
                }

                Ok(*mbr)
            }
        }
    }

    pub fn fat32_partition(&self) -> Option<&PartitionEntry> {
        self.partions
            .iter()
            .find(|x| x.partition_type == 0xB || x.partition_type == 0xC)
    }
}

#[cfg(test)]
mod mbr_tests {

    use crate::mbr::MasterBootRecord;
    use std::fs::File;
    use std::io;
    use std::io::prelude::*;
    use std::io::Cursor;

    #[test]
    fn mbr_from_test() {
        let mut f = File::open("test_data/mbr.img").unwrap();
        let mut buf: [u8; 512] = [0; 512];
        assert_eq!(f.read(&mut buf).unwrap(), 512);

        let cursor: Cursor<&mut [u8]> = Cursor::new(&mut buf);
        let mbr = MasterBootRecord::from(cursor);
        assert_eq!(mbr.is_ok(), true);
        let mbr = mbr.unwrap();
        let fats = mbr.fat32_partition();
        assert_eq!(fats.is_some(), true);
    }
}
