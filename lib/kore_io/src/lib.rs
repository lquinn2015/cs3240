#![no_std]
#![feature(error_in_core)]
#![feature(allocator_api)]
#[cfg(feature = "alloc")]
extern crate alloc;

#[allow(unused_imports)]
pub mod io;
