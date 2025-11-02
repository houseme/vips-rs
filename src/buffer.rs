use crate::{take_vips_error, Error, Result, VipsImage};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::null;

/// Extension trait for thumbnailing from a byte buffer
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
    /// # Safety Note
    /// The input byte slice must contain valid image data that libvips can decode.
    /// Providing invalid or corrupted data may lead to undefined behavior.
    /// Ensure that the data is properly validated before calling this method.
    ///
    fn thumbnail(&self, width: u32, height: u32) -> Result<VipsImage<'_>> {
        unsafe {
            let mut out = VipsImage::new_memory()?;
            let ret: c_int = vips_sys::vips_thumbnail_buffer(
                self.as_ptr() as *mut c_void,
                self.len(),
                &mut out.c,
                width as c_int,
                c"height".as_ptr(),
                height as c_int,
                c"size".as_ptr(),
                vips_sys::VipsSize::VIPS_SIZE_FORCE,
                null() as *const c_char,
            );
            if ret == 0 {
                Ok(out)
            } else {
                Err(Error::Vips(take_vips_error().unwrap_or_else(|| {
                    "Unknown error from libvips".to_string()
                })))
            }
        }
    }
}
