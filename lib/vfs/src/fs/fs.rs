use super::defs::*;

use crate::traits::block_device::BlockDevice;
use std::io::{self, Read, Seek, Write};

#[derive(Debug)]
pub struct Ext2DevHandle {
    pub dev: std::fs::File,
    pub sb: Ext2SuperBlock,
}

impl Ext2DevHandle {
    pub fn mount(p: String) -> std::io::Result<Ext2DevHandle> {
        let mut dev = std::fs::File::open(p)?;

        let mut buf = [0u8; 1024];

        dev.seek(std::io::SeekFrom::Start(1024))?;
        dev.read(&mut buf)?;

        let sb: Ext2SuperBlock = unsafe { core::mem::transmute(buf) };

        Ok(Ext2DevHandle { dev, sb })
    }

    pub fn block_size(&mut self) -> u64 {
        1024 << self.sb.log_block_size
    }

    pub fn read_struct<C: Copy>(&mut self, offset: u64) -> io::Result<C> {
        let mut obj = core::mem::MaybeUninit::<C>::uninit();
        let mut sbuf = core::mem::MaybeUninit::<[u8; 512]>::uninit();

        let (mut dst, mut src) = unsafe {
            (
                core::slice::from_raw_parts_mut(
                    obj.as_mut_ptr() as *mut u8,
                    core::mem::size_of::<C>(),
                ),
                core::slice::from_raw_parts_mut(sbuf.as_mut_ptr() as *mut u8, 512),
            )
        };

        let start_sector = offset / self.sector_size();
        let mut t_start = (offset - (self.sector_size() * start_sector)) as usize;

        let mut iter = 0;
        while !dst.is_empty() {
            self.read_sector(start_sector + iter, &mut src).unwrap();

            iter += 1;
            let to_read = usize::min(dst.len(), src[t_start..].len());

            let tslice = &mut src[t_start..t_start + to_read];
            t_start = 0;

            let (to_fill, rdst) = dst.split_at_mut(tslice.len());
            to_fill.copy_from_slice(&tslice);
            dst = rdst;
        }

        unsafe { Ok(obj.assume_init()) }
    }

    pub fn get_superblock(&mut self) -> &Ext2SuperBlock {
        &self.sb
    }
}

impl BlockDevice for Ext2DevHandle {
    fn sector_size(&self) -> u64 {
        512
    }
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> std::io::Result<usize> {
        println!("reading sector {} into len {} buf", n, buf.len());
        self.dev
            .seek(std::io::SeekFrom::Start(n * self.sector_size()))
            .unwrap();
        self.dev.read(buf)
    }
    fn write_sector(&mut self, n: u64, buf: &[u8]) -> std::io::Result<usize> {
        self.dev
            .seek(std::io::SeekFrom::Start(n * self.sector_size()))
            .unwrap();
        self.dev.write(buf)
    }
}
