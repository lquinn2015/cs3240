use crate::kprintln;
use crate::ALLOCATOR;
use core::alloc::Layout;

#[alloc_error_handler]
pub fn oom(_layout: Layout) -> ! {
    kprintln!("Allocator error {:x?}", ALLOCATOR);
    panic!("OOM");
}
