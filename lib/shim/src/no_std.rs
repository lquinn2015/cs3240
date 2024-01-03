//#pub use core_io as io;
pub use kore_io::io;

#[cfg(feature = "alloc")]
pub mod ffi;
#[cfg(feature = "alloc")]
pub mod path;
