#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

use core::{arch::asm, fmt::Write, time::Duration};
use pi::{
    self,
    uart::{BaudRate, MiniUart},
};
use shim::io;
use xmodem::Xmodem;

/// Start address of the binary to load and of the bootloader.
const BINARY_START_ADDR: usize = 0x80000;
const BOOTLOADER_START_ADDR: usize = 0x4000000;

/// Pointer to where the loaded binary expects to be laoded.
const BINARY_START: *mut u8 = BINARY_START_ADDR as *mut u8;

/// Free space between the bootloader and the loaded binary's start address.
const MAX_BINARY_SIZE: usize = BOOTLOADER_START_ADDR - BINARY_START_ADDR;

/// Branches to the address `addr` unconditionally.
unsafe fn jump_to(addr: *mut u8) -> ! {
    asm!("br {src}", src = in(reg) addr);
    loop {
        asm!("wfe")
    }
}

fn kmain() -> ! {
    let mut uart = MiniUart::new(BaudRate::Baud115200);
    uart.set_read_timeout(Duration::from_millis(750u64));

    // Boot loader is free to use this data
    let mut binary = unsafe { core::slice::from_raw_parts_mut(BINARY_START, MAX_BINARY_SIZE) };

    uart.clear();

    loop {
        match Xmodem::receive(&mut uart, &mut binary) {
            // Safety is assumed due to checksum and binary assumptions
            Ok(_) => unsafe { jump_to(BINARY_START) },
            Err(err) => match err.kind() {
                io::ErrorKind::TimedOut => (), // try again
                _ => uart
                    .write_str("Error receving over uart crashing...\n")
                    .unwrap(),
            },
        }
    }
}
