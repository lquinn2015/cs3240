/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    assert!(!is_power_of_two(align));
    addr & !(addr - 1)
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2
/// or aligning up overflows the address.
pub fn align_up(addr: usize, align: usize) -> usize {
    assert!(!is_power_of_two(align));
    addr.checked_add(align - 1).unwrap() & !(align - 1)
}

fn is_power_of_two(val: usize) -> bool {
    (val & (val - 1)) == 0
}
