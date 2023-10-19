#![cfg_attr(feature = "no_std", no_std)]
#![feature(str_internals)]
#![feature(never_type)]
//#![feature(optin_builtin_traits)] #TODO this breaks on new rust

#[cfg(feature = "alloc")]
extern crate alloc;

cfg_if::cfg_if! {
    if #[cfg(feature = "no_std")] {
        mod no_std;
        pub use self::no_std::*;
    } else {
        mod std;
        pub use self::std::*;
    }
}

#[macro_use]
pub mod macros;

#[cfg(test)]
mod tests;
