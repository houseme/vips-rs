use std::ffi::CStr;

/// Retrieve the current libvips error message.
pub fn current_error() -> String {
    let msg = unsafe { CStr::from_ptr(vips_sys::vips_error_buffer()) };
    msg.to_str().unwrap().to_string()
}
