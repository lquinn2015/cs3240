#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

pub mod console;
pub mod mutex;
pub mod shell;

use core::fmt::Write;

use console::kprintln;

use pi::uart::{self, MiniUart};
// FIXME: You need to add dependencies here to
// test your drivers (Phase 2). Add them as needed.

fn kmain() -> ! {
    let mut io = MiniUart::new();
    loop {
        let b = io.read_byte();
        io.write_byte(b);
        io.write_str("<-");
    }
}
