//! libvips version information encapsulation

/// Return (major, minor, micro)
pub fn version() -> (i32, i32, i32) {
    unsafe {
        (
            vips_sys::vips_version(0),
            vips_sys::vips_version(1),
            vips_sys::vips_version(2),
        )
    }
}

/// Returns the full version string, such as "vips-8.14.2"
pub fn version_string() -> String {
    unsafe {
        let c = std::ffi::CStr::from_ptr(vips_sys::vips_version_string());
        c.to_string_lossy().into_owned()
    }
}
