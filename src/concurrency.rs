//! Concurrency control: package 'vips_concurrency_get/set'
//!
//! Reference: <https://www.libvips.org/API/current/libvips-concurrency.html>

/// Get the current concurrency
///
/// # Returns
/// The number of threads currently used by libvips
///
/// # Example
/// ```no_run
/// use vips::concurrency;
///
/// let n = concurrency();
/// println!("Current libvips concurrency: {}", n);
/// ```
pub fn concurrency() -> i32 {
    unsafe { vips_sys::vips_concurrency_get() }
}

/// Set concurrency (<=0 will fall back to default)
///
/// # Arguments
/// * `n` - The number of threads to use
///
/// # Example
/// ```no_run
/// use vips::set_concurrency;
///
/// set_concurrency(4);
/// ```
pub fn set_concurrency(n: i32) {
    unsafe { vips_sys::vips_concurrency_set(n) }
}
