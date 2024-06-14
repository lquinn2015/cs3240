use core::arch::global_asm;
use core::mem::zeroed;
use core::ptr::write_volatile;

mod oom;
mod panic;

use crate::kmain;

global_asm!(include_str!("init/init.s"));

unsafe fn zeros_bss() {
    extern "C" {
        static mut __bss_beg: u64;
        static mut __bss_end: u64;
    }

    let mut iter: *mut u64 = &mut __bss_beg;
    let end: *mut u64 = &mut __bss_end;

    while iter < end {
        write_volatile(iter, zeroed());
        iter = iter.add(1);
    }
}

#[cfg(feature = "qemu")]
unsafe fn inject_atags() {
    core::ptr::write(
        pi::atags::ATAG_BASE as *mut [u32; 23],
        pi::atags::PI_ATAGS_BOOT,
    );
}

#[no_mangle]
unsafe fn kinit() -> ! {
    zeros_bss();
    #[cfg(feature = "qemu")]
    inject_atags();
    kmain();
}
