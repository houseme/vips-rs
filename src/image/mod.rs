use crate::{Error, Result, VipsInterpolate, take_vips_error};
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::ptr::{null, null_mut};
use vips_sys::{VipsBandFormat, VipsCombineMode, VipsDirection, VipsKernel, VipsSize};

/// Convert a filesystem path to a C string without intermediate UTF-8 `Vec` allocations.
fn path_to_cstring(path: &Path) -> Result<CString> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        CString::new(path.as_os_str().as_bytes())
            .map_err(|e| Error::InitFailed(format!("invalid path: {e}")))
    }
    #[cfg(not(unix))]
    {
        CString::new(path.to_string_lossy().as_bytes())
            .map_err(|e| Error::InitFailed(format!("invalid path: {e}")))
    }
}

/// Representation of a libvips image.
/// This struct wraps a raw pointer to a `VipsImage` from the libvips C library.
/// It provides methods for creating, manipulating, and destroying images.
/// # Lifetimes
/// The `'a` lifetime parameter ensures that the `VipsImage` does not outlive any data it references.
/// This is particularly important for images created from memory buffers, where the buffer must remain valid
/// for the lifetime of the `VipsImage`.
/// # Memory Management
/// The `VipsImage` struct implements the `Drop` trait to automatically unreference the underlying
/// libvips image when the `VipsImage` instance goes out of scope. This helps prevent memory leaks
/// when working with images in Rust.
///
/// # Thread Safety
/// The `VipsImage` struct is not inherently thread-safe. Users must ensure that instances are not
/// accessed concurrently from multiple threads unless proper synchronization is implemented.
/// # Error Handling
/// Many methods on `VipsImage` return a `Result` type to handle errors that may occur during
/// image operations. Users should handle these errors appropriately in their code.
///
/// # Safety Note
/// The `VipsImage` struct contains a raw pointer to a libvips image.
/// The user must ensure that the pointer is valid and that the image is properly managed.
/// The `Drop` implementation will unreference the image when the `VipsImage` instance is dropped.
///
/// # Example
/// ```no_run
/// use vips::*;
///
/// fn main() -> Result<()> {
///     let _instance = VipsInstance::new("app_test", true)?;
///     let img = VipsImage::from_file("input.jpg")?;
///     let thumb = img.thumbnail(100, 100, VipsSize::VIPS_SIZE_BOTH)?;
///     thumb.write_to_file("thumb.jpg")?;
///     Ok(())
/// }
/// ```
pub struct VipsImage<'a> {
    pub c: *mut vips_sys::VipsImage,
    marker: PhantomData<&'a ()>,
}

impl<'a> Drop for VipsImage<'a> {
    fn drop(&mut self) {
        if !self.c.is_null() {
            unsafe {
                vips_sys::g_object_unref(self.c as *mut c_void);
            }
        }
    }
}

/// Callback function to free memory after a VipsImage created from memory is closed.
///
/// # Safety
/// This function is called by libvips when the image is closed.
/// The `user_data` pointer must be a valid pointer to a `Box<Box<[u8]>>`.
///
/// # Arguments
/// * `_ptr` - Pointer to the VipsImage (unused)
/// * `user_data` - Pointer to the user data (Boxed buffer)
///
pub unsafe extern "C" fn image_postclose(_ptr: *mut vips_sys::VipsImage, user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    // Edition 2024: unsafe fn bodies are safe by default.
    let b: Box<Box<[u8]>> = unsafe { Box::from_raw(user_data as *mut Box<[u8]>) };
    drop(b);
}

impl<'a> VipsImage<'a> {
    /// Wrap an already-owned libvips image pointer.
    ///
    /// # Safety
    /// `ptr` must be a non-null, uniquely owned `VipsImage*` whose refcount the
    /// caller is transferring; `Drop` will `g_object_unref` it.
    pub(crate) unsafe fn from_raw(ptr: *mut vips_sys::VipsImage) -> VipsImage<'a> {
        debug_assert!(!ptr.is_null());
        VipsImage {
            c: ptr,
            marker: PhantomData,
        }
    }

    //
    // ─── CONSTRUCTORS ───────────────────────────────────────────────────────────────
    //

    /// Create a new empty VipsImage.
    ///
    /// # Errors
    /// Returns an error if the image creation fails.
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let img = VipsImage::new()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new() -> Result<VipsImage<'a>> {
        let c = unsafe { vips_sys::vips_image_new() };
        result(c)
    }

    /// Create a new empty VipsImage in memory.
    ///
    /// # Errors
    /// Returns an error if the image creation fails.
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let img = VipsImage::new_memory()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new_memory() -> Result<VipsImage<'a>> {
        let c = unsafe { vips_sys::vips_image_new_memory() };
        result(c)
    }

    /// Create a VipsImage from a file.
    ///
    /// # Arguments
    /// * `path` - The file path to load the image from.
    ///
    /// # Errors
    /// Returns an error if the image loading fails.
    ///
    /// # Returns
    /// A `Result` containing the loaded `VipsImage` or an error.
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let img = VipsImage::from_file("input.jpg")?;
    ///     Ok(())
    /// }
    /// ```
    pub fn from_file(path: impl AsRef<Path>) -> Result<VipsImage<'a>> {
        let path = path_to_cstring(path.as_ref())?;
        unsafe {
            let ptr = vips_sys::vips_image_new_from_file(path.as_ptr(), null() as *const c_char);
            result(ptr)
        }
    }

    /// Create a VipsImage from a memory buffer.
    ///
    /// # Arguments
    /// * `buf` - The buffer containing the image data.
    /// * `width` - The width of the image.
    /// * `height` - The height of the image.
    /// * `bands` - The number of bands (channels) in the image.
    /// * `format` - The band format of the image.
    ///
    /// # Errors
    /// Returns an error if the image creation fails.
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let img_data: Vec<u8> = vec![/* image data */];
    ///     let img = VipsImage::from_memory(img_data, 800, 600, 3, VipsBandFormat::VIPS_FORMAT_UCHAR)?;
    ///     Ok(())
    /// }
    /// ```
    ///
    pub fn from_memory(
        buf: Vec<u8>,
        width: u32,
        height: u32,
        bands: u8,
        format: VipsBandFormat,
    ) -> Result<VipsImage<'a>> {
        let b: Box<[u8]> = buf.into_boxed_slice();
        let c = unsafe {
            vips_sys::vips_image_new_from_memory(
                b.as_ptr() as *const c_void,
                b.len(),
                width as i32,
                height as i32,
                bands as i32,
                format,
            )
        };

        if c.is_null() {
            return Err(Error::Vips(
                take_vips_error().unwrap_or_else(|| "Unknown error from libvips".to_string()),
            ));
        }

        let bb: Box<Box<[u8]>> = Box::new(b);
        let raw: *mut c_void = Box::into_raw(bb) as *mut c_void;

        unsafe {
            let callback: unsafe extern "C" fn() =
                std::mem::transmute(image_postclose as *const ());
            vips_sys::g_signal_connect_data(
                c as *mut c_void,
                c"postclose".as_ptr(),
                Some(callback),
                raw,
                None,
                vips_sys::GConnectFlags::G_CONNECT_AFTER,
            );
        };

        Ok(VipsImage {
            c,
            marker: PhantomData,
        })
    }

    /// Create a VipsImage from a memory buffer reference.
    ///
    /// # Arguments
    /// * `buf` - The buffer slice containing the image data.
    /// * `width` - The width of the image.
    /// * `height` - The height of the image.
    /// * `bands` - The number of bands (channels) in the image.
    /// * `format` - The band format of the image.
    ///
    /// # Returns
    /// A `Result` containing the created `VipsImage` or an error.
    ///
    /// # Errors
    /// Returns an error if the image creation fails.
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let img_data: &[u8] = &[/* image data */];
    ///     let img = VipsImage::from_memory_reference(img_data, 800, 600, 3, VipsBandFormat::VIPS_FORMAT_UCHAR)?;
    ///     Ok(())
    /// }
    /// ```
    ///
    pub fn from_memory_reference(
        buf: &'a [u8],
        width: u32,
        height: u32,
        bands: u8,
        format: VipsBandFormat,
    ) -> Result<VipsImage<'a>> {
        let c = unsafe {
            vips_sys::vips_image_new_from_memory(
                buf.as_ptr() as *const c_void,
                buf.len(),
                width as i32,
                height as i32,
                bands as i32,
                format,
            )
        };

        result(c)
    }

    /// Create a VipsImage from a byte buffer.
    ///
    /// # Arguments
    /// * `buf` - The buffer slice containing the image data.
    ///
    /// # Returns
    /// A `Result` containing the created `VipsImage` or an error.
    ///
    /// # Errors
    /// Returns an error if the image creation fails.
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let img_data: &[u8] = &[/* image data */];
    ///     let img = VipsImage::from_buffer(img_data)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn from_buffer(buf: &'a [u8]) -> Result<VipsImage<'a>> {
        let c = unsafe {
            vips_sys::vips_image_new_from_buffer(
                buf.as_ptr() as *const c_void,
                buf.len(),
                null(),
                null() as *const c_char,
            )
        };

        result(c)
    }

    //
    // ─── DRAW ───────────────────────────────────────────────────────────────────────
    //

    /// Draw a rectangle on the image.
    ///
    /// # Arguments
    /// * `ink` - The color to use for drawing, as a slice of f64 values.
    /// * `left` - The left coordinate of the rectangle.
    /// * `top` - The top coordinate of the rectangle.
    /// * `width` - The width of the rectangle.
    /// * `height` - The height of the rectangle.
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    ///
    /// # Errors
    /// Returns an error if the drawing operation fails.
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let mut img = VipsImage::from_file("input.jpg")?;
    ///     img.draw_rect(&[255.0, 0.0, 0.0], 10, 10, 100, 50)?;
    ///     img.write_to_file("output.jpg")?;
    ///     Ok(())
    /// }
    /// ```
    pub fn draw_rect(
        &mut self,
        ink: &[f64],
        left: u32,
        top: u32,
        width: u32,
        height: u32,
    ) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_rect(
                self.c as *mut vips_sys::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                left as i32,
                top as i32,
                width as i32,
                height as i32,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_rect1(
        &mut self,
        ink: f64,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
    ) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_rect1(
                self.c as *mut vips_sys::VipsImage,
                ink,
                left as i32,
                top as i32,
                width as i32,
                height as i32,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_point(&mut self, ink: &[f64], x: i32, y: i32) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_point(
                self.c as *mut vips_sys::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                x,
                y,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_point1(&mut self, ink: f64, x: i32, y: i32) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_point1(
                self.c as *mut vips_sys::VipsImage,
                ink,
                x,
                y,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_image(
        &mut self,
        img: &VipsImage,
        x: i32,
        y: i32,
        mode: VipsCombineMode,
    ) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_image(
                self.c as *mut vips_sys::VipsImage,
                img.c as *mut vips_sys::VipsImage,
                x,
                y,
                c"mode".as_ptr(),
                mode,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_mask(&mut self, ink: &[f64], mask: &VipsImage, x: i32, y: i32) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_mask(
                self.c as *mut vips_sys::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                mask.c as *mut vips_sys::VipsImage,
                x,
                y,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_mask1(&mut self, ink: f64, mask: &VipsImage, x: i32, y: i32) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_mask1(
                self.c as *mut vips_sys::VipsImage,
                ink,
                mask.c as *mut vips_sys::VipsImage,
                x,
                y,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_line(&mut self, ink: &[f64], x1: i32, y1: i32, x2: i32, y2: i32) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_line(
                self.c as *mut vips_sys::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                x1,
                y1,
                x2,
                y2,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_line1(&mut self, ink: f64, x1: i32, y1: i32, x2: i32, y2: i32) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_line1(
                self.c as *mut vips_sys::VipsImage,
                ink,
                x1,
                y1,
                x2,
                y2,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_circle(&mut self, ink: &[f64], cx: i32, cy: i32, r: i32, fill: bool) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_circle(
                self.c as *mut vips_sys::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                cx,
                cy,
                r,
                c"fill".as_ptr(),
                fill as i32,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_circle1(&mut self, ink: f64, cx: i32, cy: i32, r: i32, fill: bool) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_circle1(
                self.c as *mut vips_sys::VipsImage,
                ink,
                cx,
                cy,
                r,
                c"fill".as_ptr(),
                fill as i32,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_flood(&mut self, ink: &[f64], x: i32, y: i32) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_flood(
                self.c as *mut vips_sys::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                x,
                y,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_flood1(&mut self, ink: f64, x: i32, y: i32) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_flood1(
                self.c as *mut vips_sys::VipsImage,
                ink,
                x,
                y,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_smudge(&mut self, left: u32, top: u32, width: u32, height: u32) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_smudge(
                self.c as *mut vips_sys::VipsImage,
                left as i32,
                top as i32,
                width as i32,
                height as i32,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }

    //
    // ─── MOSAIC ─────────────────────────────────────────────────────────────────────
    //

    pub fn merge(
        &self,
        another: &VipsImage,
        direction: VipsDirection,
        dx: i32,
        dy: i32,
        mblend: Option<i32>,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        let ret = unsafe {
            match mblend {
                Some(mblend) => vips_sys::vips_merge(
                    self.c as *mut vips_sys::VipsImage,
                    another.c as *mut vips_sys::VipsImage,
                    &mut out_ptr,
                    direction,
                    dx,
                    dy,
                    c"mblend".as_ptr(),
                    mblend,
                    null() as *const c_char,
                ),
                None => vips_sys::vips_merge(
                    self.c as *mut vips_sys::VipsImage,
                    another.c as *mut vips_sys::VipsImage,
                    &mut out_ptr,
                    direction,
                    dx,
                    dy,
                    null() as *const c_char,
                ),
            }
        };
        result_with_ret(out_ptr, ret)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn mosaic(
        &self,
        sec: &VipsImage,
        direction: VipsDirection,
        xref: i32,
        yref: i32,
        xsec: i32,
        ysec: i32,
        bandno: Option<i32>,
        hwindow: Option<i32>,
        harea: Option<i32>,
        mblend: Option<i32>,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        let ret = unsafe {
            vips_sys::vips_mosaic(
                self.c as *mut vips_sys::VipsImage,
                sec.c as *mut vips_sys::VipsImage,
                &mut out_ptr,
                direction,
                xref,
                yref,
                xsec,
                ysec,
                c"bandno".as_ptr(),
                bandno.unwrap_or(-1),
                c"hwindow".as_ptr(),
                hwindow.unwrap_or(5),
                c"harea".as_ptr(),
                harea.unwrap_or(2),
                c"mblend".as_ptr(),
                mblend.unwrap_or(-1),
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn mosaic1(
        &self,
        sec: &VipsImage,
        direction: VipsDirection,
        xr1: i32,
        yr1: i32,
        xs1: i32,
        ys1: i32,
        xr2: i32,
        yr2: i32,
        xs2: i32,
        ys2: i32,
        search: Option<bool>,
        hwindow: Option<i32>,
        harea: Option<i32>,
        interpolate: Option<VipsInterpolate>,
        mblend: Option<i32>,
        bandno: Option<i32>,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        let ret = unsafe {
            match interpolate {
                Some(interpolate) => vips_sys::vips_mosaic1(
                    self.c,
                    sec.c,
                    &mut out_ptr,
                    direction,
                    xr1,
                    yr1,
                    xs1,
                    ys1,
                    xr2,
                    yr2,
                    xs2,
                    ys2,
                    c"search".as_ptr(),
                    search.unwrap_or(false) as i32,
                    c"hwindow".as_ptr(),
                    hwindow.unwrap_or(5),
                    c"harea".as_ptr(),
                    harea.unwrap_or(2),
                    c"interpolate".as_ptr(),
                    interpolate.c,
                    c"mblend".as_ptr(),
                    mblend.unwrap_or(-1),
                    c"bandno".as_ptr(),
                    bandno.unwrap_or(-1),
                    null() as *const c_char,
                ),
                None => vips_sys::vips_mosaic1(
                    self.c as *mut vips_sys::VipsImage,
                    sec.c as *mut vips_sys::VipsImage,
                    &mut out_ptr,
                    direction,
                    xr1,
                    yr1,
                    xs1,
                    ys1,
                    xr2,
                    yr2,
                    xs2,
                    ys2,
                    c"search".as_ptr(),
                    search.unwrap_or(false) as i32,
                    c"hwindow".as_ptr(),
                    hwindow.unwrap_or(5),
                    c"harea".as_ptr(),
                    harea.unwrap_or(2),
                    c"mblend".as_ptr(),
                    mblend.unwrap_or(-1),
                    c"bandno".as_ptr(),
                    bandno.unwrap_or(-1),
                    null() as *const c_char,
                ),
            }
        };
        result_with_ret(out_ptr, ret)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn match_(
        &self,
        sec: &VipsImage,
        xr1: i32,
        yr1: i32,
        xs1: i32,
        ys1: i32,
        xr2: i32,
        yr2: i32,
        xs2: i32,
        ys2: i32,
        search: Option<bool>,
        hwindow: Option<i32>,
        harea: Option<i32>,
        interpolate: Option<VipsInterpolate>,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        let ret = unsafe {
            match interpolate {
                Some(interpolate) => vips_sys::vips_match(
                    self.c as *mut vips_sys::VipsImage,
                    sec.c as *mut vips_sys::VipsImage,
                    &mut out_ptr,
                    xr1,
                    yr1,
                    xs1,
                    ys1,
                    xr2,
                    yr2,
                    xs2,
                    ys2,
                    c"search".as_ptr(),
                    search.unwrap_or(false) as i32,
                    c"hwindow".as_ptr(),
                    hwindow.unwrap_or(5),
                    c"harea".as_ptr(),
                    harea.unwrap_or(2),
                    c"interpolate".as_ptr(),
                    interpolate.c as *mut vips_sys::VipsInterpolate,
                    null() as *const c_char,
                ),
                None => vips_sys::vips_match(
                    self.c as *mut vips_sys::VipsImage,
                    sec.c as *mut vips_sys::VipsImage,
                    &mut out_ptr,
                    xr1,
                    yr1,
                    xs1,
                    ys1,
                    xr2,
                    yr2,
                    xs2,
                    ys2,
                    c"search".as_ptr(),
                    search.unwrap_or(false) as i32,
                    c"hwindow".as_ptr(),
                    hwindow.unwrap_or(5),
                    c"harea".as_ptr(),
                    harea.unwrap_or(2),
                    null() as *const c_char,
                ),
            }
        };
        result_with_ret(out_ptr, ret)
    }

    pub fn globalbalance(
        &self,
        gamma: Option<f64>,
        int_output: Option<bool>,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        let ret = unsafe {
            vips_sys::vips_globalbalance(
                self.c as *mut vips_sys::VipsImage,
                &mut out_ptr,
                c"gamma".as_ptr(),
                gamma.unwrap_or(1.6),
                c"int_output".as_ptr(),
                int_output.unwrap_or(false) as i32,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    pub fn remosaic(&self, old_str: &str, new_str: &str) -> Result<VipsImage<'a>> {
        let old_str = CString::new(old_str)
            .map_err(|e| Error::InitFailed(format!("invalid old_str: {e}")))?;
        let new_str = CString::new(new_str)
            .map_err(|e| Error::InitFailed(format!("invalid new_str: {e}")))?;
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        let ret = unsafe {
            vips_sys::vips_remosaic(
                self.c as *mut vips_sys::VipsImage,
                &mut out_ptr,
                old_str.as_ptr(),
                new_str.as_ptr(),
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    //
    // ─── PROPERTIES ─────────────────────────────────────────────────────────────────
    //

    /// Image width in pixels.
    #[inline]
    pub fn width(&self) -> u32 {
        unsafe { (*self.c).Xsize as u32 }
    }

    /// Image height in pixels.
    #[inline]
    pub fn height(&self) -> u32 {
        unsafe { (*self.c).Ysize as u32 }
    }

    /// Image dimensions as `(width, height)`.
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    /// Number of bands (channels).
    #[inline]
    pub fn bands(&self) -> u32 {
        unsafe { (*self.c).Bands as u32 }
    }

    //
    // ─── RESIZE ─────────────────────────────────────────────────────────────────────
    //

    /// Create a thumbnail of the image.
    ///
    /// # Arguments
    /// * `width` - The desired width of the thumbnail.
    /// * `height` - The desired height of the thumbnail.
    /// * `size` - The size mode for the thumbnail (e.g., `VIPS_SIZE_BOTH`, `VIPS_SIZE_UP`, etc.).
    ///
    /// # Returns
    /// A `Result` containing the thumbnail `VipsImage` or an error.
    ///
    /// # Errors
    /// Returns an error if the thumbnail creation fails.
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let img = VipsImage::from_file("input.jpg")?;
    ///     let thumb = img.thumbnail(100, 100, VipsSize::VIPS_SIZE_BOTH)?;
    ///     thumb.write_to_file("thumb.jpg")?;
    ///     Ok(())
    /// }
    /// ```
    pub fn thumbnail(&self, width: u32, height: u32, size: VipsSize) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        let ret = unsafe {
            vips_sys::vips_thumbnail_image(
                self.c as *mut vips_sys::VipsImage,
                &mut out_ptr,
                width as i32,
                c"height".as_ptr(),
                height as i32,
                c"size".as_ptr(),
                size,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Resize the image.
    ///
    /// # Arguments
    /// * `scale` - The scaling factor for the horizontal dimension.
    /// * `vscale` - Optional scaling factor for the vertical dimension. If not provided, it defaults to the value of `scale`.
    /// * `kernel` - Optional kernel to use for resizing. If not provided, it defaults to `VIPS_KERNEL_LANCZOS3`.
    ///
    /// # Returns
    /// A `Result` containing the resized `VipsImage` or an error.
    ///
    /// # Errors
    /// Returns an error if the resizing operation fails.
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     let img = VipsImage::from_file("input.jpg")?;
    ///     let resized_img = img.resize(0.5, None, None)?;
    ///     resized_img.write_to_file("resized.jpg")?;
    ///     Ok(())
    /// }
    /// ```
    pub fn resize(
        &self,
        scale: f64,
        vscale: Option<f64>,
        kernel: Option<VipsKernel>,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        let ret = unsafe {
            vips_sys::vips_resize(
                self.c as *mut vips_sys::VipsImage,
                &mut out_ptr,
                scale,
                c"vscale".as_ptr(),
                vscale.unwrap_or(scale),
                c"kernel".as_ptr(),
                kernel.unwrap_or(VipsKernel::VIPS_KERNEL_LANCZOS3),
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Large-downscale helper: box pre-shrink then `resize`.
    ///
    /// libvips recommends shrinking by an integer factor with a box filter
    /// before the final kernel resample. For small images the extra pass
    /// costs more than it saves, so this falls back to plain `resize`.
    pub fn resize_reduce(
        &self,
        scale: f64,
        vscale: Option<f64>,
        kernel: Option<VipsKernel>,
    ) -> Result<VipsImage<'a>> {
        let vscale = vscale.unwrap_or(scale);
        if scale >= 1.0 && vscale >= 1.0 {
            return self.resize(scale, Some(vscale), kernel);
        }

        let hscale = scale.max(f64::EPSILON);
        let vscale = vscale.max(f64::EPSILON);
        let pixels = self.width() as u64 * self.height() as u64;
        // Pre-shrink only pays off on large sources.
        if pixels < 4_000_000 {
            return self.resize(hscale, Some(vscale), kernel);
        }

        let hshrink = if hscale < 0.5 {
            (1.0 / hscale / 2.0).floor().max(1.0)
        } else {
            1.0
        };
        let vshrink = if vscale < 0.5 {
            (1.0 / vscale / 2.0).floor().max(1.0)
        } else {
            1.0
        };

        let pre = if hshrink > 1.0 || vshrink > 1.0 {
            Some(self.shrink_box(hshrink, vshrink)?)
        } else {
            None
        };
        let base = pre.as_ref().unwrap_or(self);
        base.resize(hscale * hshrink, Some(vscale * vshrink), kernel)
    }

    /// Integer box shrink (`vips_shrink`) — cheapest pre-pass for large downscale.
    pub fn shrink_box(&self, xshrink: f64, yshrink: f64) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        let ret = unsafe {
            vips_sys::vips_shrink(
                self.c as *mut vips_sys::VipsImage,
                &mut out_ptr,
                xshrink,
                yshrink,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Low-level reduce (shrink by a floating factor using a kernel).
    pub fn reduce(
        &self,
        hshrink: f64,
        vshrink: f64,
        kernel: Option<VipsKernel>,
        centre: Option<bool>,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        let ret = unsafe {
            vips_sys::vips_reduce(
                self.c as *mut vips_sys::VipsImage,
                &mut out_ptr,
                hshrink,
                vshrink,
                c"kernel".as_ptr(),
                kernel.unwrap_or(VipsKernel::VIPS_KERNEL_LANCZOS3),
                c"centre".as_ptr(),
                centre.unwrap_or(false) as i32,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    //
    // ─── IO ─────────────────────────────────────────────────────────────────────────
    //

    /// Write the image to `path`. Format is inferred from the extension.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path_to_cstring(path.as_ref())?;
        let ret = unsafe {
            vips_sys::vips_image_write_to_file(
                self.c as *mut vips_sys::VipsImage,
                path.as_ptr(),
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }

    /// Encode the image into a freshly allocated `Vec<u8>`.
    ///
    /// Prefer `write_to_file` when the destination is a filesystem path — this
    /// method always materializes a full in-memory copy.
    pub fn write_to_memory(&self) -> Result<Vec<u8>> {
        unsafe {
            let mut result_size: usize = 0;
            let ptr = vips_sys::vips_image_write_to_memory(self.c, &mut result_size as *mut usize)
                as *mut u8;
            if ptr.is_null() {
                return Err(Error::Vips(
                    take_vips_error().unwrap_or_else(|| "Unknown error from libvips".to_string()),
                ));
            }
            let slice = std::slice::from_raw_parts(ptr as *const u8, result_size);
            let vec = slice.to_vec();
            vips_sys::g_free(ptr as *mut c_void);
            Ok(vec)
        }
    }

    /// JPEG-specific writer (quality path for `.jpg` destinations).
    pub fn write_jpeg(&self, path: impl AsRef<Path>, q: Option<i32>) -> Result<()> {
        let path = path_to_cstring(path.as_ref())?;
        let ret = unsafe {
            match q {
                Some(q) => vips_sys::vips_jpegsave(
                    self.c as *mut vips_sys::VipsImage,
                    path.as_ptr(),
                    c"Q".as_ptr(),
                    q,
                    null() as *const c_char,
                ),
                None => vips_sys::vips_jpegsave(
                    self.c as *mut vips_sys::VipsImage,
                    path.as_ptr(),
                    null() as *const c_char,
                ),
            }
        };
        result_draw(ret)
    }
}

fn result<'a>(ptr: *mut vips_sys::VipsImage) -> Result<VipsImage<'a>> {
    if ptr.is_null() {
        Err(Error::Vips(take_vips_error().unwrap_or_else(|| {
            "Unknown error from libvips".to_string()
        })))
    } else {
        Ok(VipsImage {
            c: ptr,
            marker: PhantomData,
        })
    }
}

fn result_with_ret<'a>(ptr: *mut vips_sys::VipsImage, ret: c_int) -> Result<VipsImage<'a>> {
    match ret {
        0 => {
            if ptr.is_null() {
                Err(Error::Vips(
                    "libvips returned success with a null image pointer".to_string(),
                ))
            } else {
                Ok(VipsImage {
                    c: ptr,
                    marker: PhantomData,
                })
            }
        }
        -1 => {
            Err(Error::Vips(take_vips_error().unwrap_or_else(|| {
                "Unknown error from libvips".to_string()
            })))
        }
        _ => Err(Error::Vips("Unknown error from libvips".to_string())),
    }
}

fn result_draw(ret: c_int) -> Result<()> {
    match ret {
        0 => Ok(()),
        -1 => {
            Err(Error::Vips(take_vips_error().unwrap_or_else(|| {
                "Unknown error from libvips".to_string()
            })))
        }
        _ => Err(Error::Vips("Unknown error from libvips".to_string())),
    }
}
