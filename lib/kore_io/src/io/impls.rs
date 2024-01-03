use crate::const_io_error;
use crate::io;
use crate::io::*;

/// std for generic &mut RWS
impl<R: Read + ?Sized> Read for &mut R {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }
}

impl<W: Write + ?Sized> Write for &mut W {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }
}

impl<S: Seek + ?Sized> Seek for &mut S {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        (**self).stream_position()
    }
}

/// std for generic &[u8] support
impl Read for &[u8] {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = core::cmp::min(buf.len(), self.len());
        let (a, b) = self.split_at(amt);

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }

        *self = b;
        Ok(amt)
    }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if buf.len() > self.len() {
            return Err(const_io_error!(
                ErrorKind::UnexpectedEof,
                "failed to fill whole buffer"
            ));
        }
        let (a, b) = self.split_at(buf.len());

        // First check if the amount of bytes we want to read is small:
        // `copy_from_slice` will generally expand to a call to `memcpy`, and
        // for a single byte the overhead is significant.
        if buf.len() == 1 {
            buf[0] = a[0];
        } else {
            buf.copy_from_slice(a);
        }

        *self = b;
        Ok(())
    }
}

impl Write for &mut [u8] {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let amt = core::cmp::min(buf.len(), self.len());
        let (a, b) = core::mem::take(self).split_at_mut(amt);
        a.copy_from_slice(&buf[..amt]);
        *self = b;
        Ok(amt)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

#[cfg(feature = "alloc")]
/// Boxed support
impl<R: Read + ?Sized> Read for Box<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        (**self).read_exact(buf)
    }
}

#[cfg(feature = "alloc")]
impl<W: Write + ?Sized> Write for Box<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (**self).write(buf)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        (**self).write_all(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        (**self).flush()
    }
}

#[cfg(feature = "alloc")]
impl<S: Seek + ?Sized> Seek for Box<S> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (**self).seek(pos)
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        (**self).stream_position()
    }
}
