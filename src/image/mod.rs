use crate::{take_vips_error, Error, Result, VipsInterpolate};
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::{null, null_mut};
use vips_sys::{VipsBandFormat, VipsCombineMode, VipsDirection, VipsKernel, VipsSize};

/// Representation of a libvips image.
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
///
pub struct VipsImage<'a> {
    pub c: *mut vips_sys::VipsImage,
    marker: PhantomData<&'a ()>,
}

impl<'a> Drop for VipsImage<'a> {
    fn drop(&mut self) {
        unsafe {
            vips_sys::g_object_unref(self.c as *mut c_void);
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
    let b: Box<Box<[u8]>> = Box::from_raw(user_data as *mut Box<[u8]>);
    drop(b);
}

impl<'a> VipsImage<'a> {
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
    pub fn from_file<S: Into<Vec<u8>>>(path: S) -> Result<VipsImage<'a>> {
        let path = path.into();
        let path = std::str::from_utf8(&path)
            .map_err(|e| Error::InitFailed(format!("invalid path: {}", e)))?;
        let path =
            CString::new(path).map_err(|e| Error::InitFailed(format!("invalid path: {}", e)))?;
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
        let b: Box<[_]> = buf.into_boxed_slice();
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

        let bb: Box<Box<_>> = Box::new(b);
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

        result(c)
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
            vips_sys::vips_merge(
                self.c as *mut vips_sys::VipsImage,
                another.c as *mut vips_sys::VipsImage,
                &mut out_ptr,
                direction,
                dx,
                dy,
                c"mblend".as_ptr(),
                mblend.unwrap_or(-1),
                null() as *const c_char,
            )
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
    ) -> Result<VipsImage<'_>> {
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
                bandno.unwrap_or(0),
                c"hwindow".as_ptr(),
                hwindow.unwrap_or(1),
                c"harea".as_ptr(),
                harea.unwrap_or(1),
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
    ) -> Result<VipsImage<'_>> {
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
                    hwindow.unwrap_or(1),
                    c"harea".as_ptr(),
                    harea.unwrap_or(1),
                    c"interpolate".as_ptr(),
                    interpolate.c,
                    c"mblend".as_ptr(),
                    mblend.unwrap_or(-1),
                    c"bandno".as_ptr(),
                    bandno.unwrap_or(0),
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
                    hwindow.unwrap_or(1),
                    c"harea".as_ptr(),
                    harea.unwrap_or(1),
                    c"mblend".as_ptr(),
                    mblend.unwrap_or(-1),
                    c"bandno".as_ptr(),
                    bandno.unwrap_or(0),
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
    ) -> Result<VipsImage<'_>> {
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
                    hwindow.unwrap_or(1),
                    c"harea".as_ptr(),
                    harea.unwrap_or(1),
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
                    hwindow.unwrap_or(1),
                    c"harea".as_ptr(),
                    harea.unwrap_or(1),
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
    ) -> Result<VipsImage<'_>> {
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

    pub fn remosaic(&self, old_str: &str, new_str: &str) -> Result<VipsImage<'_>> {
        let old_str = CString::new(old_str)
            .map_err(|e| Error::InitFailed(format!("invalid old_str: {}", e)))?;
        let new_str = CString::new(new_str)
            .map_err(|e| Error::InitFailed(format!("invalid new_str: {}", e)))?;
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

    #[allow(dead_code)]
    fn width(&self) -> u32 {
        unsafe { (*self.c).Xsize as u32 }
    }

    #[allow(dead_code)]
    fn height(&self) -> u32 {
        unsafe { (*self.c).Ysize as u32 }
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
    pub fn thumbnail(&self, width: u32, height: u32, size: VipsSize) -> Result<VipsImage<'_>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        unsafe {
            vips_sys::vips_thumbnail_image(
                self.c as *mut vips_sys::VipsImage,
                &mut out_ptr,
                width as i32,
                c"height".as_ptr(),
                height as i32,
                c"size".as_ptr(),
                size,
                null() as *const c_char,
            );
        };
        result(out_ptr)
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
    #[allow(dead_code)]
    pub fn resize(
        &self,
        scale: f64,
        vscale: Option<f64>,
        kernel: Option<VipsKernel>,
    ) -> Result<VipsImage<'_>> {
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
    #[allow(dead_code)]
    fn resize_to_size(
        &self,
        width: u32,
        height: Option<u32>,
        kernel: Option<VipsKernel>,
    ) -> Result<VipsImage<'_>> {
        self.resize(
            width as f64 / self.width() as f64,
            height.map(|h| h as f64 / self.height() as f64),
            kernel,
        )
    }

    // low-level
    // default: 2 * 1D lanczos3 (not recommended for shrink factor > 3)
    // or other kernels
    #[allow(dead_code)]
    fn reduce(
        &self,
        hshrink: f64,
        vshrink: f64,
        kernel: Option<VipsKernel>,
        centre: Option<bool>,
    ) -> VipsImage<'_> {
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
        if ret == 0 {
            VipsImage {
                c: out_ptr,
                marker: PhantomData,
            }
        } else {
            panic!(
                "{}",
                take_vips_error().unwrap_or_else(|| { "Unknown error from libvips".to_string() })
            )
        }
    }

    #[allow(dead_code)]
    fn shrink(&self) -> VipsImage<'_> {
        // simple average of nxn -> 1/n size
        // use a 2x2 box filter to average neighbouring pixels
        self.reduce(2.0, 2.0, Some(VipsKernel::VIPS_KERNEL_LINEAR), Some(false))
    }

    //
    // ─── IO ─────────────────────────────────────────────────────────────────────────
    //

    #[allow(dead_code)]
    fn jpegsave<S: Into<Vec<u8>>>(&mut self, path: S) -> Result<()> {
        let path = path.into();
        let path = std::str::from_utf8(&path)
            .map_err(|e| Error::InitFailed(format!("invalid path: {}", e)))?;
        let path =
            CString::new(path).map_err(|e| Error::InitFailed(format!("invalid path: {}", e)))?;
        let ret = unsafe {
            vips_sys::vips_jpegsave(
                self.c as *mut vips_sys::VipsImage,
                path.as_ptr(),
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }

    pub fn write_to_file<S: Into<Vec<u8>>>(&self, path: S) -> Result<()> {
        let path = path.into();
        let path = std::str::from_utf8(&path)
            .map_err(|e| Error::InitFailed(format!("invalid path: {}", e)))?;
        let path =
            CString::new(path).map_err(|e| Error::InitFailed(format!("invalid path: {}", e)))?;
        let ret = unsafe {
            vips_sys::vips_image_write_to_file(
                self.c as *mut vips_sys::VipsImage,
                path.as_ptr(),
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }

    //
    // ─── CONVERT ────────────────────────────────────────────────────────────────────
    //

    #[allow(dead_code)]
    fn to_vec(&self) -> Vec<u8> {
        unsafe {
            let mut result_size: usize = 0;
            let ptr = vips_sys::vips_image_write_to_memory(self.c, &mut result_size as *mut usize)
                as *mut u8;
            if ptr.is_null() {
                return Vec::new();
            }
            let slice = std::slice::from_raw_parts(ptr as *const u8, result_size);
            let vec = slice.to_vec();
            vips_sys::g_free(ptr as *mut c_void); // Release with GLib
            vec
        }
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
        0 => Ok(VipsImage {
            c: ptr,
            marker: PhantomData,
        }),
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
