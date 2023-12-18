#![no_std]

#[cfg(test)]
mod tests;

use core::iter::IntoIterator;
use core::ops::{Deref, DerefMut};
use core::slice;

/// A contiguous array type backed by a slice.
///
/// `StackVec`'s functionality is similar to that of `std::Vec`. You can `push`
/// and `pop` and iterate over the vector. Unlike `Vec`, however, `StackVec`
/// requires no memory allocation as it is backed by a user-supplied slice. As a
/// result, `StackVec`'s capacity is _bounded_ by the user-supplied slice. This
/// results in `push` being fallible: if `push` is called when the vector is
/// full, an `Err` is returned.
#[derive(Debug)]
pub struct StackVec<'a, T: 'a> {
    storage: &'a mut [T],
    len: usize,
}

impl<'a, T: 'a> StackVec<'a, T> {
    /// Constructs a new, empty `StackVec<T>` using `storage` as the backing
    /// store. The returned `StackVec` will be able to hold `storage.len()`
    /// values.
    pub fn new(storage: &'a mut [T]) -> StackVec<'a, T> {
        StackVec { storage, len: 0 }
    }

    /// Constructs a new `StackVec<T>` using `storage` as the backing store. The
    /// first `len` elements of `storage` are treated as if they were `push`ed
    /// onto `self.` The returned `StackVec` will be able to hold a total of
    /// `storage.len()` values.
    ///
    /// # Panics
    ///
    /// Panics if `len > storage.len()`.
    pub fn with_len(storage: &'a mut [T], len: usize) -> StackVec<'a, T> {
        if len > storage.len() {
            panic!("Illegal stack vec size");
        }
        StackVec { storage, len }
    }

    /// Returns the number of elements this vector can hold.
    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    /// Shortens the vector, keeping the first `len` elements. If `len` is
    /// greater than the vector's current length, this has no effect. Note that
    /// this method has no effect on the capacity of the vector.
    pub fn truncate(&mut self, len: usize) {
        self.len = self.len.min(len);
    }

    /// Extracts a slice containing the entire vector, consuming `self`.
    ///
    /// Note that the returned slice's length will be the length of this vector,
    /// _not_ the length of the original backing storage.
    pub fn into_slice(self) -> &'a mut [T] {
        &mut self.storage[..self.len]
    }

    /// Extracts a slice containing the entire vector.
    pub fn as_slice(&self) -> &[T] {
        &self.storage[..self.len]
    }

    /// Extracts a mutable slice of the entire vector.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.storage[..self.len]
    }

    /// Returns the number of elements in the vector, also referred to as its
    /// 'length'.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the vector contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns true if the vector is at capacity.
    pub fn is_full(&self) -> bool {
        self.len == self.storage.len()
    }

    /// Appends `value` to the back of this vector if the vector is not full.
    ///
    /// # Error
    ///
    /// If this vector is full, an `Err` is returned. Otherwise, `Ok` is
    /// returned.
    pub fn push(&mut self, value: T) -> Result<(), ()> {
        if self.is_full() {
            Err(())
        } else {
            self.storage[self.len] = value;

            self.len += 1;
            Ok(())
        }
    }
}

impl<'a, T: Clone + 'a> StackVec<'a, T> {
    /// If this vector is not empty, removes the last element from this vector
    /// by cloning it and returns it. Otherwise returns `None`.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.len -= 1;
            Some(self.storage[self.len].clone())
        }
    }
}

impl<'a, T> Deref for StackVec<'a, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.storage.as_ptr(), Self::len(&self)) }
    }
}

impl<'a, T> DerefMut for StackVec<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.storage.as_mut_ptr(), self.len) }
    }
}

impl<'a, T: 'a> IntoIterator for StackVec<'a, T> {
    type Item = &'a mut T;
    type IntoIter = core::iter::Take<core::slice::IterMut<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.storage.into_iter().take(self.len)
    }
}

impl<'a, T: 'a> IntoIterator for &'a StackVec<'a, T> {
    type Item = &'a T;
    type IntoIter = core::iter::Take<core::slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.storage.iter().take(self.len)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn into_iter_test() {
        let mut storage = [1u8; 1024];
        let sv = StackVec::with_len(&mut storage, 2);

        for x in &sv {
            assert_eq!(*x, 1);
        }

        for x in sv {
            assert_eq!(*x, 1);
        }
    }

    #[test]
    fn basic_usage() {
        let mut storage = [0u8; 1024];
        {
            let _vec = StackVec::new(&mut storage);
            let mut storage2 = [5u8; 1];
            let mut vec2 = StackVec::with_len(&mut storage2, 1);
            assert_eq!(true, vec2.is_full());
            assert_eq!(false, vec2.is_empty());
            assert!(vec2.push(1).is_err());
            assert_eq!(vec2.pop(), Some(5));
            assert_eq!(vec2.pop(), None);
            assert!(vec2.push(1).is_ok());
        }

        let mut vec = StackVec::with_len(&mut storage, 2);
        assert_eq!(1024, vec.capacity());
        assert_eq!(2, vec.len());
        vec.truncate(4);
        assert_eq!(2, vec.len());
        vec.truncate(1);
        assert_eq!(1, vec.len());

        for i in 1..10 {
            vec.push(i * i).expect("Can push 1024");
        }

        for i in 0..10 {
            assert_eq!(vec[i as usize], i * i);
        }

        for (i, x) in vec.iter_mut().enumerate() {
            *x = i as u8;
        }

        for (i, x) in vec.iter().enumerate() {
            assert_eq!(*x as usize, i);
        }
    }
}
