use core::fmt;
use shim::{const_assert_size, newioerr};

use crate::traits::BlockDevice;
use crate::vfat::Error;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct BiosParameterBlock {
    jump_code: [u8; 3],
    oem_ident: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub num_fats: u8,
    max_dir_entries: u16,
    logical_sectors_1: u16,
    media_descriptor: u8,
    __sectors_per_fat: u16,
    sector_per_track: u16,
    num_heads: u16,
    num_hidden_sectors: u32,

    //EBPB - offset [u8, 32]
    logical_sectors_2: u32,
    pub sectors_per_fat: u32,
    flags: u16,
    fat_version: u16,
    pub root_cluster: u32,
    fsinfo_sector: u16,
    backup_boot_sector: u16,
    _reserved: [u8; 12],
    drive_number: u8,
    nt_flags: u8,
    signature: u8,
    volumeid_serial: u32,
    volume_label: [u8; 11],
    system_identifer: [u8; 8],
    boot_code: [u8; 420],
    magic: [u8; 2],
}

const_assert_size!(BiosParameterBlock, 512);

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(mut device: T, sector: u64) -> Result<BiosParameterBlock, Error> {
        let mut buf: [u8; 512] = [0; 512];

        match device.read_sector(sector, &mut buf) {
            Err(e) => Err(e.into()),
            Ok(n) if n != buf.len() => Err(Error::Io(newioerr!(
                UnexpectedEof,
                "Insufficient bytes from sector read"
            ))),
            Ok(_) => {
                let block = unsafe { &*(buf.as_ptr() as *const BiosParameterBlock) };
                if block.magic != [0x55, 0xaa]
                    || (block.signature != 0x28 && block.signature != 0x29)
                {
                    Err(Error::BadSignature)
                } else {
                    Ok(*block)
                }
            }
        }
    }

    pub fn logical_sectors(&self) -> u32 {
        if self.logical_sectors_1 > 0 {
            self.logical_sectors_1 as u32
        } else {
            self.logical_sectors_2
        }
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BiosParamBlock")
            .field("signature", &self.signature)
            .field("root_cluster", &{ self.root_cluster })
            .field("bytes_per_sector", &{ self.bytes_per_sector })
            .field("sectors_per_cluster", &self.sectors_per_cluster)
            .field("num_fats", &self.num_fats)
            .field("sectors_per_fat", &{ self.sectors_per_fat })
            .field("reserved_sectors", &{ self.reserved_sectors })
            .field(
                "logical_sectors",
                &format_args!("{}", self.logical_sectors()),
            )
            .finish()
    }
}

#[cfg(test)]
mod ebpb_tests {

    use crate::vfat::*;
    use std::fs::File;
    use std::io;
    use std::io::prelude::*;
    use std::io::Cursor;

    #[test]
    fn ebpb_test() {
        let mut f = File::open("test_data/ebpb1.img").unwrap();
        let mut buf: [u8; 512] = [0; 512];
        assert_eq!(f.read(&mut buf).unwrap(), 512);

        let mut tbuf: [u8; 512] = [0; 512];
        tbuf[..].copy_from_slice(&buf[..]);
        let cursor: Cursor<&mut [u8]> = Cursor::new(&mut tbuf);

        let ebpb = BiosParameterBlock::from(cursor, 0);
        assert_eq!(ebpb.is_ok(), true);

        let mut tbuf: [u8; 512] = [0; 512];
        tbuf[..].copy_from_slice(&buf[..]);
        let cursor: Cursor<&mut [u8]> = Cursor::new(&mut tbuf);

        let ebpb = BiosParameterBlock::from(cursor, 1);
        assert_eq!(ebpb.is_ok(), false);
    }
}
