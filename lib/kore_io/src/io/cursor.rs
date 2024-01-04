use crate::const_io_error;
use crate::io;
use crate::io::{Error, ErrorKind, SeekFrom};

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Cursor<T> {
    inner: T,
    pos: u64,
}

impl<T> Cursor<T> {
    pub const fn new(inner: T) -> Cursor<T> {
        Cursor { pos: 0, inner }
    }
    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn get_ref(&self) -> &T {
        &self.inner
    }
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn position(&self) -> u64 {
        self.pos
    }

    pub fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}

impl<T> Cursor<T>
where
    T: AsRef<[u8]>,
{
    pub fn remaining_slice(&self) -> &[u8] {
        let len = self.pos.min(self.inner.as_ref().len() as u64);
        &self.inner.as_ref()[(len as usize)..]
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.inner.as_ref().len() as u64
    }
}

impl<T> Clone for Cursor<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Cursor {
            inner: self.inner.clone(),
            pos: self.pos,
        }
    }

    #[inline]
    fn clone_from(&mut self, other: &Self) {
        self.inner.clone_from(&other.inner);
        self.pos = other.pos;
    }
}

impl<T> io::Seek for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn seek(&mut self, style: SeekFrom) -> io::Result<u64> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(n);
            }
            SeekFrom::End(n) => (self.inner.as_ref().len() as u64, n),
            SeekFrom::Current(n) => (self.pos, n),
        };
        match base_pos.checked_add_signed(offset as i64) {
            Some(n) => {
                self.pos = n;
                Ok(self.pos)
            }
            None => Err(const_io_error!(
                ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )),
        }
    }

    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(self.inner.as_ref().len() as u64)
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.pos)
    }
}

impl<T> io::Read for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = io::Read::read(&mut self.remaining_slice(), buf)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let n = buf.len();
        io::Read::read_exact(&mut self.remaining_slice(), buf)?;
        self.pos += n as u64;
        Ok(())
    }
}

#[inline]
fn slice_write(pos_mut: &mut u64, slice: &mut [u8], buf: &[u8]) -> io::Result<usize> {
    let pos = core::cmp::min(*pos_mut, slice.len() as u64);
    let amt = (&mut slice[(pos as usize)..]).write(buf)?;
    *pos_mut += amt as u64;
    Ok(amt)
}

impl io::Write for Cursor<&mut [u8]> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        slice_write(&mut self.pos, self.inner, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use core::alloc::Allocator;

use crate::io::Write;
#[cfg(feature = "alloc")]
use alloc::vec::*;

#[cfg(feature = "alloc")]
fn reserve_and_pad<A: Allocator>(
    pos_mut: &mut u64,
    vec: &mut Vec<u8, A>,
    buf_len: usize,
) -> io::Result<usize> {
    let pos: usize = (*pos_mut).try_into().map_err(|_| {
        const_io_error!(
            ErrorKind::InvalidInput,
            "cursor position exceeds maximum possible vector length",
        )
    })?;

    // For safety reasons, we don't want these numbers to overflow
    // otherwise our allocation won't be enough
    let desired_cap = pos.saturating_add(buf_len);
    if desired_cap > vec.capacity() {
        // We want our vec's total capacity
        // to have room for (pos+buf_len) bytes. Reserve allocates
        // based on additional elements from the length, so we need to
        // reserve the difference
        vec.reserve(desired_cap - vec.len());
    }
    // Pad if pos is above the current len.
    if pos > vec.len() {
        let diff = pos - vec.len();
        // Unfortunately, `resize()` would suffice but the optimiser does not
        // realise the `reserve` it does can be eliminated. So we do it manually
        // to eliminate that extra branch
        let spare = vec.spare_capacity_mut();
        debug_assert!(spare.len() >= diff);
        // Safety: we have allocated enough capacity for this.
        // And we are only writing, not reading
        unsafe {
            spare
                .get_unchecked_mut(..diff)
                .fill(core::mem::MaybeUninit::new(0));
            vec.set_len(pos);
        }
    }

    Ok(pos)
}

#[cfg(feature = "alloc")]
unsafe fn vec_write_unchecked<A>(pos: usize, vec: &mut Vec<u8, A>, buf: &[u8]) -> usize
where
    A: Allocator,
{
    debug_assert!(vec.capacity() >= pos + buf.len());
    vec.as_mut_ptr().add(pos).copy_from(buf.as_ptr(), buf.len());
    pos + buf.len()
}

#[cfg(feature = "alloc")]
fn vec_write<A>(pos_mut: &mut u64, vec: &mut Vec<u8, A>, buf: &[u8]) -> io::Result<usize>
where
    A: Allocator,
{
    let buf_len = buf.len();
    let mut pos = reserve_and_pad(pos_mut, vec, buf_len)?;

    // Write the buf then progress the vec forward if necessary
    // Safety: we have ensured that the capacity is available
    // and that all bytes get written up to pos
    unsafe {
        pos = vec_write_unchecked(pos, vec, buf);
        if pos > vec.len() {
            vec.set_len(pos);
        }
    };

    // Bump us forward
    *pos_mut += buf_len as u64;
    Ok(buf_len)
}

#[cfg(feature = "alloc")]
impl<A> Write for Cursor<Vec<u8, A>>
where
    A: Allocator,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        vec_write(&mut self.pos, &mut self.inner, buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<A> Write for Cursor<Box<[u8], A>>
where
    A: Allocator,
{
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        slice_write(&mut self.pos, &mut self.inner, buf)
    }
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
