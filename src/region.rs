use crate::VipsImage;
use std::os::raw::c_void;

/// VipsRegion struct wrapping libvips VipsRegion
///
/// # Example
/// ```no_run
/// use vips::*;
///
/// fn main() -> Result<()> {
///     let _instance = VipsInstance::new("app_test", true)?;
///     let img = VipsImage::from_file("examples/images/kodim01.png")?;
///     let region = VipsRegion::new(&img);
///     Ok(())
/// }
/// ```
pub struct VipsRegion {
    // The underlying C VipsRegion pointer
    pub c: *mut vips_sys::VipsRegion,
}

impl VipsRegion {
    /// Create a new VipsRegion for the given image
    ///
    /// # Arguments
    /// * `image` - The VipsImage to create the region for
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let img = VipsImage::from_file("examples/images/kodim01.png")?;
    ///     let region = VipsRegion::new(&img);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(image: &VipsImage) -> VipsRegion {
        let c = unsafe { vips_sys::vips_region_new(image.c) };
        VipsRegion { c }
    }
}

impl Drop for VipsRegion {
    fn drop(&mut self) {
        unsafe {
            vips_sys::g_object_unref(self.c as *mut c_void);
        }
    }
}
