#![feature(alloc_error_handler)]
#![feature(decl_macro)]
#![feature(panic_info_message)]
#![feature(raw_vec_internals)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

extern crate alloc;

pub mod allocator;
pub mod console;
//pub mod fs;
pub mod mutex;
pub mod shell;

use console::kprintln;

use allocator::Allocator;
//use fs::FileSystem;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();
//pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();

use pi::atags::Atags;

fn kmain() -> ! {
    unsafe {
        ALLOCATOR.initialize();
        //FILESYSTEM.initialize();
    }

    kprintln!("Welcome to xphosia!");

    for z in 0..20 {
        for i in 1..100 {
            let mut v = crate::alloc::vec![];
            for j in 0..(i + 10) {
                v.push(i + j);
            }
            let mut u = crate::alloc::vec![];
            for j in 0..(i + 10) {
                u.push(i + j);
            }
            kprintln!("v is {}", v.len());
        }
    }
    kprintln!("Allocator {:x?}", ALLOCATOR);

    shell::shell("> ");
}
