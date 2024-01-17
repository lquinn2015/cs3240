use core::alloc::Layout;
use core::ptr;

use crate::allocator::linked_list::LinkedList;
use crate::allocator::util::*;
use crate::allocator::LocalAlloc;

/// A simple allocator that allocates based on size classes.
///   bin 0 (2^3 bytes)    : handles allocations in (0, 2^3]
///   bin 1 (2^4 bytes)    : handles allocations in (2^3, 2^4]
///   ...
///   bin 29 (2^22 bytes): handles allocations in (2^31, 2^32]
///   
///   map_to_bin(size) -> k
///   

const MAX_BINS: usize = 32;

#[derive(Debug)]
pub struct Allocator {
    bins: [LinkedList; MAX_BINS],
    start: usize,
    current: usize,
    end: usize,
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        Allocator {
            bins: [LinkedList::new(); MAX_BINS],
            start,
            current: start,
            end,
        }
    }
}
fn map2bin(size: usize) -> usize {
    let mut bin = 3;
    while size > (0b1 << bin) {
        bin += 1;
    }
    bin - 3
}

fn size4bin(bin: usize) -> usize {
    0b1 << (3 + bin)
}

fn power_of_two(n: usize) -> bool {
    (n & (n - 1)) == 0
}

use crate::kprintln;

impl LocalAlloc for Allocator {
    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning null pointer (`core::ptr::null_mut`)
    /// indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's
    /// size or alignment constraints.
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        assert!(power_of_two(layout.align()));

        let bin = map2bin(core::cmp::max(layout.size(), layout.align()));
        if bin >= MAX_BINS {
            kprintln!("[MEM], map2bin end up as {} > MAX_BINS {}", bin, MAX_BINS);
            return ptr::null_mut(); // OOM to large of alloc
        }

        for node in self.bins[bin].iter_mut() {
            if node.value() as usize % layout.align() == 0 {
                // require alignment
                return node.pop() as *mut u8;
            }
        }

        let alloc_size = size4bin(bin);
        let start = align_up(self.current, layout.align());
        let start = match start.checked_add(alloc_size) {
            Some(val) => val,
            None => {
                kprintln!("[MEM] alloc had overflow on mapping new bin");
                return ptr::null_mut();
            }
        };
        if start > self.end {
            kprintln!("[MEM] bin allocator OOM");
            return ptr::null_mut();
        }

        #[cfg(DBG)]
        kprintln!(
            "alloc {} to bin {}, size {}",
            start as usize,
            bin,
            alloc_size
        );
        self.current = start + alloc_size;
        start as *mut u8
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        assert!(power_of_two(layout.align()));

        let alloc_size = core::cmp::max(layout.size(), layout.align());
        let bin = map2bin(alloc_size);
        assert!(bin < MAX_BINS);
        #[cfg(DBG)]
        kprintln!(
            "dealloc {} to bin {} of size {}",
            ptr as usize,
            bin,
            alloc_size
        );
        self.bins[bin].push(ptr as *mut usize);
    }
}
