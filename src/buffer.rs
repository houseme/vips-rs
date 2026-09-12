use crate::{Error, Result, VipsImage, take_vips_error};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::null_mut;

/// Extension trait for thumbnailing from a byte buffer
///
/// # Example
/// ```no_run
/// use vips::*;
///
/// fn main() -> Result<()> {
///     let _instance = VipsInstance::new("app_test", true)?;
///     let binding = std::fs::read("./examples/images/kodim01.png")?;
///     let img_data: &[u8] = binding.as_slice();
///     let thumbnail = img_data.thumbnail(100, 100)?;
///     thumbnail.write_to_file("kodim01_thumb.png")?;
///     Ok(())
/// }
/// ```
///
pub trait VipsBuffer {
    fn thumbnail(&self, width: u32, height: u32) -> Result<VipsImage<'_>>;
}

impl VipsBuffer for &[u8] {
    /// Create a thumbnail VipsImage from the byte buffer
    ///
    /// # Arguments
    /// * `width` - The desired thumbnail width
    /// * `height` - The desired thumbnail height
    ///
    /// # Errors
    /// Returns an error if the thumbnail creation fails
    ///
    /// # Safety Note
    /// The input byte slice must contain valid image data that libvips can decode.
    /// Providing invalid or corrupted data may lead to undefined behavior.
    ///
    fn thumbnail(&self, width: u32, height: u32) -> Result<VipsImage<'_>> {
        // Do not pre-allocate a memory image: thumbnail owns the output pointer.
        // Pre-creating and overwriting `out.c` would leak the first image.
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
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
                std::ptr::null() as *const c_char,
            )
        };
        if ret == 0 && !out_ptr.is_null() {
            Ok(unsafe { VipsImage::from_raw(out_ptr) })
        } else {
            if !out_ptr.is_null() {
                unsafe {
                    vips_sys::g_object_unref(out_ptr as *mut c_void);
                }
            }
            Err(Error::Vips(take_vips_error().unwrap_or_else(|| {
                "Unknown error from libvips".to_string()
            })))
        }
    }
}
