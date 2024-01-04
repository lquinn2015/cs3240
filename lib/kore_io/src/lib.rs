#![no_std]
#![feature(error_in_core)]
#![cfg(feature = "alloc")]
#![feature(allocator_api)]
#[cfg(feature = "alloc")]
extern crate alloc;

pub mod io;
