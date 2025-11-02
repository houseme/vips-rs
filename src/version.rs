//! libvips version information encapsulation

/// Returns the libvips version as a tuple: (major, minor, micro)
///
/// # Returns
/// A tuple of three integers representing the major, minor, and micro version numbers.
///
/// # Example
/// ```no_run
/// use vips::version;
///
/// let (major, minor, micro) = version();
/// println!("libvips version: {}.{}.{}", major, minor, micro);
/// ```
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
/// # Returns
/// A string representing the full libvips version.
///
/// # Example
/// ```no_run
/// use vips::version_string;
///
/// let version_str = version_string();
/// println!("libvips version string: {}", version_str);
/// ```
///
/// # Note
/// This function involves unsafe code to interface with the underlying C library.
///
/// # Safety
/// The function assumes that the pointer returned by `vips_version_string` is valid
/// and points to a null-terminated C string.
///
/// # Panics
/// The function will panic if the C string is not valid UTF-8.
///
/// # Errors
/// This function does not return errors; it will convert invalid UTF-8 to a lossy string.
///
/// # Performance
/// The function may incur a small overhead due to the conversion from C string to Rust String.
///
/// # Thread Safety
/// The function is safe to call from multiple threads as it does not modify any shared state.
///
pub fn version_string() -> String {
    unsafe {
        let c = std::ffi::CStr::from_ptr(vips_sys::vips_version_string());
        c.to_string_lossy().into_owned()
    }
}
