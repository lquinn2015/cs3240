pub trait BlockDevice: Send {
    fn sector_size(&self) -> u64 {
        512
    }

    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize>;
    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize>;
}

impl<'a, T: BlockDevice> BlockDevice for &'a mut T {
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        (*self).read_sector(n, buf)
    }

    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize> {
        (*self).write_sector(n, buf)
    }
}

macro_rules! impl_for_read_write_seek {
    ($(<$($gen:tt),*>)* $T:path) => {
        impl $(<$($gen),*>)* BlockDevice for $T {
            fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
                let sector_size = self.sector_size();
                let to_read = ::core::cmp::min(sector_size as usize, buf.len());
                self.seek(io::SeekFrom::Start(n * sector_size))?;
                self.read_exact(&mut buf[..to_read])?;
                Ok(to_read)
            }

            fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize> {
                let sector_size = self.sector_size();
                let to_write = ::core::cmp::min(sector_size as usize, buf.len());
                self.seek(io::SeekFrom::Start(n * sector_size))?;
                self.write_all(&buf[..to_write])?;
                Ok(to_write)
            }
        }
    };
}

use shim::io;
use shim::io::{Read, Seek, Write};

impl_for_read_write_seek!(<'a> shim::io::Cursor<&'a mut [u8]>);
impl_for_read_write_seek!(shim::io::Cursor<Vec<u8>>);
impl_for_read_write_seek!(shim::io::Cursor<Box<[u8]>>);
#[cfg(test)]
impl_for_read_write_seek!(::std::fs::File);
