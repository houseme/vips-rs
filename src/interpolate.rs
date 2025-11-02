use crate::{take_vips_error, Error, Result, VipsRegion};
use std::ffi::CString;
use std::os::raw::c_void;

/// VipsInterpolate struct wrapping libvips VipsInterpolate
///
/// # Example
/// ```no_run
/// use vips::*;
///
/// fn main() -> Result<()> {
///     let _instance = VipsInstance::new("app_test", true)?;
///     let interpolate = VipsInterpolate::bilinear_static();
///     let method = interpolate.method();
///     let img = VipsImage::from_file("examples/images/kodim01.png")?;
///     let region = VipsRegion::new(&img);
///     let mut out = vec![0u8; 3]; // assuming 3 channels
///     method.call(&interpolate, &region, &mut out, 1.5, 1.5);
///     Ok(())
/// }
/// ```
pub struct VipsInterpolate {
    pub c: *mut vips_sys::VipsInterpolate,
    is_static: bool,
}

impl Drop for VipsInterpolate {
    fn drop(&mut self) {
        if !self.is_static {
            unsafe {
                vips_sys::g_object_unref(self.c as *mut c_void);
            }
        }
    }
}

impl VipsInterpolate {
    //
    // ─── STATIC ─────────────────────────────────────────────────────────────────────
    //

    // will not implement: vips_interpolate ()

    //
    // ─── CONSTRUCTORS ───────────────────────────────────────────────────────────────
    //

    /// Create a new VipsInterpolate by nickname
    ///
    /// # Arguments
    /// * `nickname` - The nickname of the interpolation method
    ///
    /// # Errors
    /// Returns an error if the nickname is invalid
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let interpolate = VipsInterpolate::new("bilinear")?;
    ///     let method = interpolate.method();
    ///     let img = VipsImage::from_file("examples/images/kodim01.png")?;
    ///     let region = VipsRegion::new(&img);
    ///     let mut out = vec![0u8; 3]; // assuming 3 channels
    ///     method.call(&interpolate, &region, &mut out, 1.5, 1.5);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(nickname: &str) -> Result<VipsInterpolate> {
        let nickname = CString::new(nickname)
            .map_err(|_| Error::Other("Invalid nickname: contains null byte".to_string()))?;
        let c = unsafe { vips_sys::vips_interpolate_new(nickname.as_ptr()) };
        if c.is_null() {
            Err(Error::Vips(take_vips_error().unwrap_or_else(|| {
                "Unknown error from libvips".to_string()
            })))
        } else {
            Ok(VipsInterpolate {
                c,
                is_static: false,
            })
        }
    }

    /// Create a new nearest static VipsInterpolate
    ///
    /// # Returns
    /// A VipsInterpolate instance for nearest neighbor interpolation
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let interpolate = VipsInterpolate::nearest_static();
    ///     let method = interpolate.method();
    ///     let img = VipsImage::from_file("examples/images/kodim01.png")?;
    ///     let region = VipsRegion::new(&img);
    ///     let mut out = vec![0u8; 3]; // assuming 3 channels
    ///     method.call(&interpolate, &region, &mut out, 1.5, 1.5);
    ///     Ok(())
    /// }
    /// ```
    ///
    pub fn nearest_static() -> VipsInterpolate {
        let c = unsafe { vips_sys::vips_interpolate_nearest_static() };
        VipsInterpolate { c, is_static: true }
    }

    /// Create a new bilinear static VipsInterpolate
    ///
    /// # Returns
    /// A VipsInterpolate instance for bilinear interpolation
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let interpolate = VipsInterpolate::bilinear_static();
    ///     let method = interpolate.method();
    ///     let img = VipsImage::from_file("examples/images/kodim01.png")?;
    ///     let region = VipsRegion::new(&img);
    ///     let mut out = vec![0u8; 3]; // assuming 3 channels
    ///     method.call(&interpolate, &region, &mut out, 1.5, 1.5);
    ///     Ok(())
    /// }
    /// ```
    pub fn bilinear_static() -> VipsInterpolate {
        let c = unsafe { vips_sys::vips_interpolate_bilinear_static() };
        VipsInterpolate { c, is_static: true }
    }

    //
    // ─── PROPERTIES ─────────────────────────────────────────────────────────────────
    //

    /// Get the interpolation method
    ///
    /// # Returns
    /// A VipsInterpolateMethod instance representing the interpolation method
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let interpolate = VipsInterpolate::bilinear_static();
    ///     let method = interpolate.method();
    ///     let img = VipsImage::from_file("examples/images/kodim01.png")?;
    ///     let region = VipsRegion::new(&img);
    ///     let mut out = vec![0u8; 3]; // assuming 3 channels
    ///     method.call(&interpolate, &region, &mut out, 1.5, 1.5);
    ///     Ok(())
    /// }
    /// ```
    pub fn method(&self) -> VipsInterpolateMethod {
        let c = unsafe { vips_sys::vips_interpolate_get_method(self.c) };
        VipsInterpolateMethod { c }
    }

    /// Get the window size
    ///
    /// # Returns
    /// The window size used by the interpolation method
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///    let interpolate = VipsInterpolate::bilinear_static();
    ///    let window_size = interpolate.window_size();
    ///     println!("Window size: {}", window_size);
    ///     Ok(())
    /// }
    /// ```
    pub fn window_size(&self) -> i32 {
        unsafe { vips_sys::vips_interpolate_get_window_size(self.c) }
    }

    /// Get the window offset
    ///
    /// # Returns
    /// The window offset used by the interpolation method
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let interpolate = VipsInterpolate::bilinear_static();
    ///     let window_offset = interpolate.window_offset();
    ///     println!("Window offset: {}", window_offset);
    ///     Ok(())
    /// }
    /// ```
    pub fn window_offset(&self) -> i32 {
        unsafe { vips_sys::vips_interpolate_get_window_offset(self.c) }
    }
}

/// Function pointer type for VipsInterpolateMethod
///
/// # Example
/// ```no_run
/// use vips::*;
///
/// fn main() -> Result<()> {
///     let _instance = VipsInstance::new("app_test", true)?;
///     let interpolate = VipsInterpolate::bilinear_static();
///     let method = interpolate.method();
///     let img = VipsImage::from_file("examples/images/kodim01.png")?;
///     let region = VipsRegion::new(&img);
///     let mut out = vec![0u8; 3]; // assuming 3 channels
///     method.call(&interpolate, &region, &mut out, 1.5, 1.5);
///     Ok(())
/// }
/// ```
pub struct VipsInterpolateMethod {
    c: vips_sys::VipsInterpolateMethod,
}

impl VipsInterpolateMethod {
    /// Call the interpolation method
    ///
    /// # Arguments
    /// * `interpolate` - The VipsInterpolate instance
    /// * `in_` - The input VipsRegion
    /// * `out` - The output buffer to write the interpolated pixel
    /// * `x` - The x coordinate to interpolate
    /// * `y` - The y coordinate to interpolate
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    /// fn main() -> Result<()> {
    ///     let interpolate = VipsInterpolate::bilinear_static();
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let img = VipsImage::from_file("examples/images/kodim01.png")?;
    ///     let region = VipsRegion::new(&img);
    ///     let mut out = vec![0u8; 3]; // assuming 3 channels
    ///     let method = interpolate.method();
    ///     method.call(&interpolate, &region, &mut out, 1.5, 1.5);
    ///     Ok(())
    /// }
    /// ```
    ///
    pub fn call(
        &self,
        interpolate: &VipsInterpolate,
        in_: &VipsRegion,
        out: &mut [u8],
        x: f64,
        y: f64,
    ) {
        unsafe { self.c.unwrap()(interpolate.c, out.as_mut_ptr() as *mut c_void, in_.c, x, y) }
    }
}
