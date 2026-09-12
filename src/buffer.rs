use crate::ffi;
use crate::{Error, Result, VipsImage, take_vips_error};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::{null, null_mut};

/// Extension trait for thumbnailing from a byte buffer.
pub trait VipsBuffer {
    /// Decode `self` as an image and force a `width`×`height` thumbnail.
    ///
    /// # Errors
    /// Returns an error if the buffer is not a valid image or thumbnail fails.
    fn thumbnail(&self, width: u32, height: u32) -> Result<VipsImage<'_>>;
}

impl VipsBuffer for &[u8] {
    fn thumbnail(&self, width: u32, height: u32) -> Result<VipsImage<'_>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: buffer pointer/len come from a valid Rust slice; out_ptr is
        // written by libvips on success. Do not pre-allocate an image (leak).
        let ret: c_int = unsafe {
            vips_sys::vips_thumbnail_buffer(
                self.as_ptr() as *mut c_void,
                self.len(),
                &mut out_ptr,
                width as c_int,
                c"height".as_ptr(),
                height as c_int,
                c"size".as_ptr(),
                vips_sys::VipsSize::VIPS_SIZE_FORCE,
                null() as *const c_char,
            )
        };
        if ret == 0 && !out_ptr.is_null() {
            // SAFETY: ownership transferred from libvips.
            Ok(unsafe { VipsImage::from_raw(out_ptr) })
        } else {
            if !out_ptr.is_null() {
                // SAFETY: partial result must be unref'd on failure.
                unsafe { ffi::unref(out_ptr.cast()) };
            }
            Err(Error::Vips(take_vips_error().unwrap_or_else(|| {
                "Unknown error from libvips".to_string()
            })))
        }
    }
}
