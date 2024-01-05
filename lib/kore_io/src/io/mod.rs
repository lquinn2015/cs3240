use core::cmp;
use core::fmt;
use core::iter::Iterator;
use core::option::Option::{self, None, Some};
use core::result;
use core::result::Result::{Err, Ok};

pub use self::cursor::Cursor;
pub use self::error::{Error, ErrorKind, Result};

mod cursor;
mod error;
mod impls;
pub mod prelude;

#[allow(dead_code)]
const DEFAULT_BUF_SIZE: usize = 64 * 1024;

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;

    fn rewind(&mut self) -> Result<()> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    fn stream_len(&mut self) -> Result<u64> {
        let old_pos = self.stream_position()?;
        let len = self.seek(SeekFrom::End(0))?;

        if old_pos != len {
            self.seek(SeekFrom::Start(old_pos))?;
        }

        Ok(len)
    }

    fn stream_position(&mut self) -> Result<u64> {
        self.seek(SeekFrom::Current(0))
    }
}

pub enum SeekFrom {
    Start(u64),
    End(u64),
    Current(u64),
}

pub trait Write {
    /// Required methods
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn flush(&mut self) -> Result<()>;

    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(Error::new(
                        ErrorKind::WriteZero,
                        "Write all failed to write",
                    ));
                }
                Ok(n) => buf = &buf[n..],
                Err(ref e) if e.is_interrupted() => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

pub trait Read {
    /// Required to impl
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// this is trimmed down from the rust STD
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        default_read_exact(self, buf)
    }
}

pub(crate) fn default_read_exact<R: Read + ?Sized>(this: &mut R, mut buf: &mut [u8]) -> Result<()> {
    while !buf.is_empty() {
        match this.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                let tmp = buf;
                buf = &mut tmp[n..];
            }
            Err(ref e) if e.is_interrupted() => {}
            Err(e) => return Err(e),
        }
    }
    if !buf.is_empty() {
        Err(Error::new(
            ErrorKind::UnexpectedEof,
            "UnexpectedEof in read_all",
        ))
    } else {
        Ok(())
    }
}
