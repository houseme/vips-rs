//! libvips version information encapsulation

use std::sync::OnceLock;

/// Returns the libvips version as a tuple: (major, minor, micro)
///
/// # Example
/// ```no_run
/// use vips::version;
///
/// let (major, minor, micro) = version();
/// println!("libvips version: {}.{}.{}", major, minor, micro);
/// ```
#[inline]
pub fn version() -> (i32, i32, i32) {
    unsafe {
        (
            vips_sys::vips_version(0),
            vips_sys::vips_version(1),
            vips_sys::vips_version(2),
        )
    }
}

/// Returns the full version string, such as "8.14.2"
///
/// Cached after the first call so repeated queries do not re-allocate.
///
/// # Example
/// ```no_run
/// use vips::version_string;
///
/// let version_str = version_string();
/// println!("libvips version string: {}", version_str);
/// ```
pub fn version_string() -> &'static str {
    static VERSION: OnceLock<&'static str> = OnceLock::new();
    VERSION.get_or_init(|| unsafe {
        let ptr = vips_sys::vips_version_string();
        if ptr.is_null() {
            return "unknown";
        }
        std::ffi::CStr::from_ptr(ptr).to_str().unwrap_or("unknown")
    })
}
