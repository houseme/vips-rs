//! Concurrency control: package 'vips_concurrency_get/set'
//!
//! Reference: https://www.libvips.org/API/current/libvips-concurrency.html

/// Get the current concurrency
pub fn concurrency() -> i32 {
    unsafe { vips_sys::vips_concurrency_get() }
}

/// Set concurrency (<=0 will fall back to default)
pub fn set_concurrency(n: i32) {
    unsafe { vips_sys::vips_concurrency_set(n) }
}
