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

    let mut v = crate::alloc::vec![];
    kprintln!("Welcome to cs3210!");
    for tag in Atags::get() {
        v.push(tag.clone());
        kprintln!("Atag found: {:?}", tag);
    }
    kprintln!("vec is size {:?}", v);
    shell::shell("> ");
}
