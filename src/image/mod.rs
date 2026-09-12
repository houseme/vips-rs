use crate::ffi;
use crate::{Error, Result, VipsInterpolate, take_vips_error};
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::ptr::{null, null_mut};
use vips_sys::{
    VipsBandFormat, VipsCombineMode, VipsCompassDirection, VipsDirection, VipsExtend,
    VipsInteresting, VipsKernel, VipsSize,
};

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

/// Safe RAII wrapper around a libvips `VipsImage*`.
///
/// The raw pointer is private. Use the safe methods, or [`VipsImage::as_ptr`]
/// when you must pass the handle into a lower-level FFI call.
///
/// # Lifetimes
/// `'a` ties images created from borrowed buffers to that buffer's lifetime.
/// Owned constructors (`from_file`, `from_memory`) use an unbound lifetime.
///
/// # Thread safety
/// Not `Send`/`Sync`. Do not share an image across threads without external
/// synchronization. libvips pipelines are otherwise process-global.
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
    /// Unique ownership of a `VipsImage*`; always non-null.
    c: std::ptr::NonNull<vips_sys::VipsImage>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Drop for VipsImage<'a> {
    fn drop(&mut self) {
        // SAFETY: exclusive refcount ownership; NonNull is never null.
        unsafe { ffi::unref(self.c.as_ptr().cast()) };
    }
}

/// Free the boxed pixel buffer after libvips closes a memory image.
///
/// # Safety
/// Called by GObject with `user_data` from `g_signal_connect_data`; must be a
/// `Box<Box<[u8]>>` allocated by `from_memory`.
unsafe extern "C" fn image_postclose(_ptr: *mut vips_sys::VipsImage, user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    // SAFETY: reconstructed exactly as boxed in `from_memory`.
    let b: Box<Box<[u8]>> = unsafe { Box::from_raw(user_data as *mut Box<[u8]>) };
    drop(b);
}

impl<'a> VipsImage<'a> {
    /// Borrow the underlying libvips pointer for FFI.
    ///
    /// The pointer is valid while `self` is alive. Do not unref it.
    #[inline]
    pub fn as_ptr(&self) -> *mut vips_sys::VipsImage {
        self.c.as_ptr()
    }

    /// Take ownership of a non-null `VipsImage*`.
    ///
    /// # Safety
    /// `ptr` must be a uniquely owned, non-null `VipsImage*` whose refcount is
    /// transferred to the returned value (`Drop` will unref it).
    pub(crate) unsafe fn from_raw(ptr: *mut vips_sys::VipsImage) -> VipsImage<'a> {
        let c = std::ptr::NonNull::new(ptr).expect("VipsImage::from_raw(null)");
        VipsImage {
            c,
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
        // SAFETY: libvips allocates a fresh empty image or returns null.
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
        // SAFETY: libvips allocates a fresh memory-backed image or returns null.
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

        // SAFETY: `c` is a valid image; callback frees `raw` exactly once on close.
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

        // SAFETY: unique ownership transferred to the wrapper.
        Ok(unsafe { VipsImage::from_raw(c) })
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
                self.c.as_ptr(),
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
                self.c.as_ptr(),
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
                self.c.as_ptr(),
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
            vips_sys::vips_draw_point1(self.c.as_ptr(), ink, x, y, null() as *const c_char)
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
                self.c.as_ptr(),
                img.as_ptr(),
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
                self.c.as_ptr(),
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                mask.as_ptr(),
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
                self.c.as_ptr(),
                ink,
                mask.as_ptr(),
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
                self.c.as_ptr(),
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
                self.c.as_ptr(),
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
                self.c.as_ptr(),
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
                self.c.as_ptr(),
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
                self.c.as_ptr(),
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
            vips_sys::vips_draw_flood1(self.c.as_ptr(), ink, x, y, null() as *const c_char)
        };
        result_draw(ret)
    }
    pub fn draw_smudge(&mut self, left: u32, top: u32, width: u32, height: u32) -> Result<()> {
        let ret = unsafe {
            vips_sys::vips_draw_smudge(
                self.c.as_ptr(),
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
                    self.c.as_ptr(),
                    another.as_ptr(),
                    &mut out_ptr,
                    direction,
                    dx,
                    dy,
                    c"mblend".as_ptr(),
                    mblend,
                    null() as *const c_char,
                ),
                None => vips_sys::vips_merge(
                    self.c.as_ptr(),
                    another.as_ptr(),
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
                self.c.as_ptr(),
                sec.as_ptr(),
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
                    self.c.as_ptr(),
                    sec.as_ptr(),
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
                    interpolate.as_ptr(),
                    c"mblend".as_ptr(),
                    mblend.unwrap_or(-1),
                    c"bandno".as_ptr(),
                    bandno.unwrap_or(-1),
                    null() as *const c_char,
                ),
                None => vips_sys::vips_mosaic1(
                    self.c.as_ptr(),
                    sec.as_ptr(),
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
                    self.c.as_ptr(),
                    sec.as_ptr(),
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
                    interpolate.as_ptr(),
                    null() as *const c_char,
                ),
                None => vips_sys::vips_match(
                    self.c.as_ptr(),
                    sec.as_ptr(),
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
                self.c.as_ptr(),
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
                self.c.as_ptr(),
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
        // SAFETY: `c` is a live image owned by `self`.
        unsafe { (*self.c.as_ptr()).Xsize as u32 }
    }

    /// Image height in pixels.
    #[inline]
    pub fn height(&self) -> u32 {
        // SAFETY: `c` is a live image owned by `self`.
        unsafe { (*self.c.as_ptr()).Ysize as u32 }
    }

    /// Image dimensions as `(width, height)`.
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    /// Number of bands (channels).
    #[inline]
    pub fn bands(&self) -> u32 {
        // SAFETY: `c` is a live image owned by `self`.
        unsafe { (*self.c.as_ptr()).Bands as u32 }
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
                self.c.as_ptr(),
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
                self.c.as_ptr(),
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
                self.c.as_ptr(),
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
                self.c.as_ptr(),
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
    // ─── GEOMETRY (php-vips aligned) ───────────────────────────────────────────────
    //

    /// Crop a rectangle (`left`, `top`, `width`, `height`).
    pub fn crop(&self, left: i32, top: i32, width: i32, height: i32) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image; libvips writes a new owned out image.
        let ret = unsafe {
            vips_sys::vips_crop(
                self.c.as_ptr(),
                &mut out_ptr,
                left,
                top,
                width,
                height,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Alias of [`Self::crop`] (`vips_extract_area`).
    #[inline]
    pub fn extract_area(
        &self,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
    ) -> Result<VipsImage<'a>> {
        self.crop(left, top, width, height)
    }

    /// Embed the image in a larger canvas at `(x, y)`.
    pub fn embed(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        extend: Option<VipsExtend>,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image; optional extend key defaults to black when omitted.
        let ret = unsafe {
            vips_sys::vips_embed(
                self.c.as_ptr(),
                &mut out_ptr,
                x,
                y,
                width,
                height,
                c"extend".as_ptr(),
                extend.unwrap_or(VipsExtend::VIPS_EXTEND_BLACK),
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Place the image inside a larger canvas using a compass direction.
    pub fn gravity(
        &self,
        direction: VipsCompassDirection,
        width: i32,
        height: i32,
        extend: Option<VipsExtend>,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image; optional extend key defaults to black.
        let ret = unsafe {
            vips_sys::vips_gravity(
                self.c.as_ptr(),
                &mut out_ptr,
                direction,
                width,
                height,
                c"extend".as_ptr(),
                extend.unwrap_or(VipsExtend::VIPS_EXTEND_BLACK),
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Insert `sub` into a copy of `self` at `(x, y)`.
    pub fn insert(&self, sub: &VipsImage, x: i32, y: i32) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: both images live for the call.
        let ret = unsafe {
            vips_sys::vips_insert(
                self.c.as_ptr(),
                sub.as_ptr(),
                &mut out_ptr,
                x,
                y,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Flip about a horizontal or vertical axis.
    pub fn flip(&self, direction: VipsDirection) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_flip(
                self.c.as_ptr(),
                &mut out_ptr,
                direction,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Flip horizontally.
    #[inline]
    pub fn fliphor(&self) -> Result<VipsImage<'a>> {
        self.flip(VipsDirection::VIPS_DIRECTION_HORIZONTAL)
    }

    /// Flip vertically.
    #[inline]
    pub fn flipver(&self) -> Result<VipsImage<'a>> {
        self.flip(VipsDirection::VIPS_DIRECTION_VERTICAL)
    }

    /// Rotate by a multiple of 90°.
    pub fn rot(&self, angle: vips_sys::VipsAngle) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_rot(
                self.c.as_ptr(),
                &mut out_ptr,
                angle,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Rotate 90° clockwise.
    #[inline]
    pub fn rot90(&self) -> Result<VipsImage<'a>> {
        self.rot(vips_sys::VipsAngle::VIPS_ANGLE_D90)
    }

    /// Rotate 180°.
    #[inline]
    pub fn rot180(&self) -> Result<VipsImage<'a>> {
        self.rot(vips_sys::VipsAngle::VIPS_ANGLE_D180)
    }

    /// Rotate 270° clockwise.
    #[inline]
    pub fn rot270(&self) -> Result<VipsImage<'a>> {
        self.rot(vips_sys::VipsAngle::VIPS_ANGLE_D270)
    }

    /// Rotate by an arbitrary angle in degrees (affine).
    pub fn rotate(&self, angle: f64) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_rotate(
                self.c.as_ptr(),
                &mut out_ptr,
                angle,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Rotate according to EXIF orientation tag.
    pub fn autorot(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_autorot(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Integer zoom (nearest-neighbour scale).
    pub fn zoom(&self, xfac: i32, yfac: i32) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_zoom(
                self.c.as_ptr(),
                &mut out_ptr,
                xfac,
                yfac,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Extract a single band.
    pub fn extract_band(&self, band: i32) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_extract_band(
                self.c.as_ptr(),
                &mut out_ptr,
                band,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Join with another image band-wise.
    pub fn bandjoin2(&self, other: &VipsImage) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: both images live for the call.
        let ret = unsafe {
            vips_sys::vips_bandjoin2(
                self.c.as_ptr(),
                other.as_ptr(),
                &mut out_ptr,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Append constant bands (e.g. alpha = 255).
    pub fn bandjoin_const(&self, constants: &[f64]) -> Result<VipsImage<'a>> {
        if constants.is_empty() {
            return self.copy();
        }
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: `constants` is a valid contiguous f64 buffer for `n` bands.
        let ret = unsafe {
            vips_sys::vips_bandjoin_const(
                self.c.as_ptr(),
                &mut out_ptr,
                constants.as_ptr() as *mut f64,
                constants.len() as i32,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Materialize a full memory copy (end a sequential pipeline).
    pub fn copy_memory(&self) -> Result<VipsImage<'a>> {
        // SAFETY: live image; libvips returns a new owned memory image.
        let ptr = unsafe { vips_sys::vips_image_copy_memory(self.c.as_ptr()) };
        result(ptr)
    }

    /// Copy with optional metadata overrides (format, interpretation, …).
    pub fn copy(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_copy(
                self.c.as_ptr(),
                &mut out_ptr,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    //
    // ─── ARITHMETIC / COLOR / FILTER ──────────────────────────────────────────────
    //

    /// Pixel-wise add of two images.
    pub fn add(&self, other: &VipsImage) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: both images live for the call.
        let ret = unsafe {
            vips_sys::vips_add(
                self.c.as_ptr(),
                other.as_ptr(),
                &mut out_ptr,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Pixel-wise subtract.
    pub fn subtract(&self, other: &VipsImage) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: both images live for the call.
        let ret = unsafe {
            vips_sys::vips_subtract(
                self.c.as_ptr(),
                other.as_ptr(),
                &mut out_ptr,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Pixel-wise multiply.
    pub fn multiply(&self, other: &VipsImage) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: both images live for the call.
        let ret = unsafe {
            vips_sys::vips_multiply(
                self.c.as_ptr(),
                other.as_ptr(),
                &mut out_ptr,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Pixel-wise divide.
    pub fn divide(&self, other: &VipsImage) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: both images live for the call.
        let ret = unsafe {
            vips_sys::vips_divide(
                self.c.as_ptr(),
                other.as_ptr(),
                &mut out_ptr,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Affine intensity transform: `out = a * in + b`.
    pub fn linear1(&self, a: f64, b: f64) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_linear1(
                self.c.as_ptr(),
                &mut out_ptr,
                a,
                b,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Per-band affine intensity transform (`a[i] * in + b[i]`).
    pub fn linear(&self, a: &[f64], b: &[f64]) -> Result<VipsImage<'a>> {
        assert_eq!(a.len(), b.len(), "linear a/b length mismatch");
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: equal-length contiguous buffers for n bands.
        let ret = unsafe {
            vips_sys::vips_linear(
                self.c.as_ptr(),
                &mut out_ptr,
                a.as_ptr(),
                b.as_ptr(),
                a.len() as i32,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Photometric negative (`max - in` for unsigned formats).
    pub fn invert(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_invert(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Cast to another band format.
    pub fn cast(&self, format: VipsBandFormat) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_cast(
                self.c.as_ptr(),
                &mut out_ptr,
                format,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Convert colour space (e.g. sRGB ↔ scRGB ↔ Lab).
    pub fn colourspace(
        &self,
        interpretation: vips_sys::VipsInterpretation,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_colourspace(
                self.c.as_ptr(),
                &mut out_ptr,
                interpretation,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Gaussian blur.
    pub fn gaussblur(&self, sigma: f64) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_gaussblur(
                self.c.as_ptr(),
                &mut out_ptr,
                sigma,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Unsharp-mask sharpen.
    pub fn sharpen(&self, sigma: f64) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_sharpen(
                self.c.as_ptr(),
                &mut out_ptr,
                c"sigma".as_ptr(),
                sigma,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    //
    // ─── STATISTICS ───────────────────────────────────────────────────────────────
    //

    /// Average of all pixels.
    pub fn avg(&self) -> Result<f64> {
        let mut out: f64 = 0.0;
        // SAFETY: live image; out is written by libvips.
        let ret =
            unsafe { vips_sys::vips_avg(self.c.as_ptr(), &mut out, null() as *const c_char) };
        ffi::ret_to_result(ret)?;
        Ok(out)
    }

    /// Minimum pixel value.
    pub fn min_value(&self) -> Result<f64> {
        let mut out: f64 = 0.0;
        // SAFETY: live image; out is written by libvips.
        let ret =
            unsafe { vips_sys::vips_min(self.c.as_ptr(), &mut out, null() as *const c_char) };
        ffi::ret_to_result(ret)?;
        Ok(out)
    }

    /// Maximum pixel value.
    pub fn max_value(&self) -> Result<f64> {
        let mut out: f64 = 0.0;
        // SAFETY: live image; out is written by libvips.
        let ret =
            unsafe { vips_sys::vips_max(self.c.as_ptr(), &mut out, null() as *const c_char) };
        ffi::ret_to_result(ret)?;
        Ok(out)
    }

    /// Pixel values at `(x, y)` as a `Vec<f64>` (one entry per band).
    pub fn getpoint(&self, x: i32, y: i32) -> Result<Vec<f64>> {
        let mut vector: *mut f64 = null_mut();
        let mut n: c_int = 0;
        // SAFETY: live image; libvips allocates `vector` of length `n`.
        let ret = unsafe {
            vips_sys::vips_getpoint(
                self.c.as_ptr(),
                &mut vector,
                &mut n,
                x,
                y,
                null() as *const c_char,
            )
        };
        ffi::ret_to_result(ret)?;
        if vector.is_null() || n <= 0 {
            return Ok(Vec::new());
        }
        // SAFETY: non-null buffer of `n` doubles owned by us; free with g_free.
        let values = unsafe {
            let slice = std::slice::from_raw_parts(vector as *const f64, n as usize);
            let owned = slice.to_vec();
            vips_sys::g_free(vector.cast());
            owned
        };
        Ok(values)
    }

    /// Standard deviation of all pixels.
    pub fn deviate(&self) -> Result<f64> {
        let mut out: f64 = 0.0;
        // SAFETY: live image; out is written by libvips.
        let ret =
            unsafe { vips_sys::vips_deviate(self.c.as_ptr(), &mut out, null() as *const c_char) };
        ffi::ret_to_result(ret)?;
        Ok(out)
    }

    /// Threshold at percentile `percent` of the image histogram (0–100).
    pub fn percent(&self, percent: f64) -> Result<i32> {
        let mut threshold: c_int = 0;
        // SAFETY: live image; threshold is written by libvips.
        let ret = unsafe {
            vips_sys::vips_percent(
                self.c.as_ptr(),
                percent,
                &mut threshold,
                null() as *const c_char,
            )
        };
        ffi::ret_to_result(ret)?;
        Ok(threshold)
    }

    /// Bounds of non-background pixels as `(left, top, width, height)`.
    pub fn find_trim(&self, threshold: Option<f64>, background: Option<&[f64]>) -> Result<(i32, i32, i32, i32)> {
        let mut left = 0;
        let mut top = 0;
        let mut width = 0;
        let mut height = 0;
        // SAFETY: live image; out integers written by libvips.
        // Optional keys omitted when None so libvips defaults apply.
        let ret = unsafe {
            match (threshold, background) {
                (Some(t), Some(bg)) => vips_sys::vips_find_trim(
                    self.c.as_ptr(),
                    &mut left,
                    &mut top,
                    &mut width,
                    &mut height,
                    c"threshold".as_ptr(),
                    t,
                    c"background".as_ptr(),
                    bg.as_ptr() as *mut f64,
                    bg.len() as i32,
                    null() as *const c_char,
                ),
                (Some(t), None) => vips_sys::vips_find_trim(
                    self.c.as_ptr(),
                    &mut left,
                    &mut top,
                    &mut width,
                    &mut height,
                    c"threshold".as_ptr(),
                    t,
                    null() as *const c_char,
                ),
                (None, Some(bg)) => vips_sys::vips_find_trim(
                    self.c.as_ptr(),
                    &mut left,
                    &mut top,
                    &mut width,
                    &mut height,
                    c"background".as_ptr(),
                    bg.as_ptr() as *mut f64,
                    bg.len() as i32,
                    null() as *const c_char,
                ),
                (None, None) => vips_sys::vips_find_trim(
                    self.c.as_ptr(),
                    &mut left,
                    &mut top,
                    &mut width,
                    &mut height,
                    null() as *const c_char,
                ),
            }
        };
        ffi::ret_to_result(ret)?;
        Ok((left, top, width, height))
    }

    //
    // ─── MATH / RELATIONAL / BOOLEAN (lua-vips convenience style) ─────────────────
    //

    /// Apply a math op (`sin`, `cos`, `log`, …).
    pub fn math(&self, op: vips_sys::VipsOperationMath) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_math(
                self.c.as_ptr(),
                &mut out_ptr,
                op,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Sine (degrees).
    #[inline]
    pub fn sin(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_sin(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Cosine (degrees).
    #[inline]
    pub fn cos(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_cos(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Tangent (degrees).
    #[inline]
    pub fn tan(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_tan(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Natural logarithm.
    #[inline]
    pub fn ln(&self) -> Result<VipsImage<'a>> {
        self.math(vips_sys::VipsOperationMath::VIPS_OPERATION_MATH_LOG)
    }

    /// Base-10 logarithm.
    #[inline]
    pub fn log10(&self) -> Result<VipsImage<'a>> {
        self.math(vips_sys::VipsOperationMath::VIPS_OPERATION_MATH_LOG10)
    }

    /// e^pixel.
    #[inline]
    pub fn exp(&self) -> Result<VipsImage<'a>> {
        self.math(vips_sys::VipsOperationMath::VIPS_OPERATION_MATH_EXP)
    }

    /// 10^pixel.
    #[inline]
    pub fn exp10(&self) -> Result<VipsImage<'a>> {
        self.math(vips_sys::VipsOperationMath::VIPS_OPERATION_MATH_EXP10)
    }

    /// Floor.
    pub fn floor(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_floor(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Ceiling.
    pub fn ceil(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_ceil(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Round to nearest integer.
    pub fn rint(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_rint(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Absolute value.
    pub fn abs(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_abs(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Sign of each pixel (-1, 0, +1).
    pub fn sign(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_sign(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Clamp to the band format range.
    pub fn clamp(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_clamp(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Raise each pixel to `exp`.
    pub fn pow_const(&self, exp: f64) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_pow_const1(
                self.c.as_ptr(),
                &mut out_ptr,
                exp,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Relational compare with a constant → 255/0 mask.
    pub fn relational_const(
        &self,
        op: vips_sys::VipsOperationRelational,
        c: f64,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_relational_const1(
                self.c.as_ptr(),
                &mut out_ptr,
                op,
                c,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// `self < c` mask.
    #[inline]
    pub fn less(&self, c: f64) -> Result<VipsImage<'a>> {
        self.relational_const(vips_sys::VipsOperationRelational::VIPS_OPERATION_RELATIONAL_LESS, c)
    }

    /// `self <= c` mask.
    #[inline]
    pub fn lesseq(&self, c: f64) -> Result<VipsImage<'a>> {
        self.relational_const(
            vips_sys::VipsOperationRelational::VIPS_OPERATION_RELATIONAL_LESSEQ,
            c,
        )
    }

    /// `self > c` mask.
    #[inline]
    pub fn more(&self, c: f64) -> Result<VipsImage<'a>> {
        self.relational_const(vips_sys::VipsOperationRelational::VIPS_OPERATION_RELATIONAL_MORE, c)
    }

    /// `self >= c` mask.
    #[inline]
    pub fn moreeq(&self, c: f64) -> Result<VipsImage<'a>> {
        self.relational_const(
            vips_sys::VipsOperationRelational::VIPS_OPERATION_RELATIONAL_MOREEQ,
            c,
        )
    }

    /// `self == c` mask.
    #[inline]
    pub fn equal_const(&self, c: f64) -> Result<VipsImage<'a>> {
        self.relational_const(vips_sys::VipsOperationRelational::VIPS_OPERATION_RELATIONAL_EQUAL, c)
    }

    /// `self != c` mask.
    #[inline]
    pub fn notequal_const(&self, c: f64) -> Result<VipsImage<'a>> {
        self.relational_const(
            vips_sys::VipsOperationRelational::VIPS_OPERATION_RELATIONAL_NOTEQ,
            c,
        )
    }

    /// Boolean op with a constant.
    pub fn boolean_const(
        &self,
        op: vips_sys::VipsOperationBoolean,
        c: f64,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_boolean_const1(
                self.c.as_ptr(),
                &mut out_ptr,
                op,
                c,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Bitwise AND with constant.
    #[inline]
    pub fn and_const(&self, c: f64) -> Result<VipsImage<'a>> {
        self.boolean_const(vips_sys::VipsOperationBoolean::VIPS_OPERATION_BOOLEAN_AND, c)
    }

    /// Bitwise OR with constant.
    #[inline]
    pub fn or_const(&self, c: f64) -> Result<VipsImage<'a>> {
        self.boolean_const(vips_sys::VipsOperationBoolean::VIPS_OPERATION_BOOLEAN_OR, c)
    }

    /// Bitwise XOR with constant.
    #[inline]
    pub fn eor_const(&self, c: f64) -> Result<VipsImage<'a>> {
        self.boolean_const(vips_sys::VipsOperationBoolean::VIPS_OPERATION_BOOLEAN_EOR, c)
    }

    /// Bitwise left shift by constant.
    pub fn lshift_const(&self, c: f64) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_lshift_const1(
                self.c.as_ptr(),
                &mut out_ptr,
                c,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Bitwise right shift by constant.
    pub fn rshift_const(&self, c: f64) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_rshift_const1(
                self.c.as_ptr(),
                &mut out_ptr,
                c,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Collapse all bands with a boolean op (`and`/`or`/`eor`).
    pub fn bandbool(&self, op: vips_sys::VipsOperationBoolean) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_bandbool(
                self.c.as_ptr(),
                &mut out_ptr,
                op,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Collapse bands with AND.
    #[inline]
    pub fn bandand(&self) -> Result<VipsImage<'a>> {
        self.bandbool(vips_sys::VipsOperationBoolean::VIPS_OPERATION_BOOLEAN_AND)
    }

    /// Collapse bands with OR.
    #[inline]
    pub fn bandor(&self) -> Result<VipsImage<'a>> {
        self.bandbool(vips_sys::VipsOperationBoolean::VIPS_OPERATION_BOOLEAN_OR)
    }

    /// Collapse bands with XOR.
    #[inline]
    pub fn bando(&self) -> Result<VipsImage<'a>> {
        self.bandbool(vips_sys::VipsOperationBoolean::VIPS_OPERATION_BOOLEAN_EOR)
    }

    /// Mean of each pixel across bands → 1-band image.
    pub fn bandmean(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_bandmean(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Split into one single-band image per band.
    pub fn bandsplit(&self) -> Result<Vec<VipsImage<'a>>> {
        let n = self.bands() as i32;
        let mut out = Vec::with_capacity(n as usize);
        for band in 0..n {
            out.push(self.extract_band(band)?);
        }
        Ok(out)
    }

    //
    // ─── FILTER / MORPH / EDGE ────────────────────────────────────────────────────
    //

    /// Convolve with a mask image.
    pub fn conv(&self, mask: &VipsImage) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: both images live for the call.
        let ret = unsafe {
            vips_sys::vips_conv(
                self.c.as_ptr(),
                &mut out_ptr,
                mask.as_ptr(),
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Morphological op with a structuring element.
    pub fn morph(
        &self,
        mask: &VipsImage,
        op: vips_sys::VipsOperationMorphology,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: both images live for the call.
        let ret = unsafe {
            vips_sys::vips_morph(
                self.c.as_ptr(),
                &mut out_ptr,
                mask.as_ptr(),
                op,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Erode with structuring element.
    #[inline]
    pub fn erode(&self, mask: &VipsImage) -> Result<VipsImage<'a>> {
        self.morph(mask, vips_sys::VipsOperationMorphology::VIPS_OPERATION_MORPHOLOGY_ERODE)
    }

    /// Dilate with structuring element.
    #[inline]
    pub fn dilate(&self, mask: &VipsImage) -> Result<VipsImage<'a>> {
        self.morph(mask, vips_sys::VipsOperationMorphology::VIPS_OPERATION_MORPHOLOGY_DILATE)
    }

    /// Rank filter (median when `index = w*h/2`).
    pub fn rank(&self, width: i32, height: i32, index: i32) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_rank(
                self.c.as_ptr(),
                &mut out_ptr,
                width,
                height,
                index,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// `size × size` median filter.
    #[inline]
    pub fn median(&self, size: i32) -> Result<VipsImage<'a>> {
        let index = (size * size) / 2;
        self.rank(size, size, index)
    }

    /// Sobel edge magnitude.
    pub fn sobel(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_sobel(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Canny edge detector.
    pub fn canny(&self, sigma: Option<f64>) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            match sigma {
                Some(s) => vips_sys::vips_canny(
                    self.c.as_ptr(),
                    &mut out_ptr,
                    c"sigma".as_ptr(),
                    s,
                    null() as *const c_char,
                ),
                None => vips_sys::vips_canny(
                    self.c.as_ptr(),
                    &mut out_ptr,
                    null() as *const c_char,
                ),
            }
        };
        result_with_ret(out_ptr, ret)
    }

    //
    // ─── LOGIC / CROP / COLOR extras ──────────────────────────────────────────────
    //

    /// Pick pixels from `then` or `else` using this image as a 0/non-zero mask.
    pub fn ifthenelse(
        &self,
        then: &VipsImage,
        els: &VipsImage,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: all three images live for the call.
        let ret = unsafe {
            vips_sys::vips_ifthenelse(
                self.c.as_ptr(),
                then.as_ptr(),
                els.as_ptr(),
                &mut out_ptr,
                null() as *const c_char,
            )
        };
        result_with_ret(out_ptr, ret)
    }

    /// Attention-based crop to `width`×`height`.
    pub fn smartcrop(
        &self,
        width: i32,
        height: i32,
        interesting: Option<VipsInteresting>,
    ) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image; optional interesting key omitted when None.
        let ret = unsafe {
            match interesting {
                Some(i) => vips_sys::vips_smartcrop(
                    self.c.as_ptr(),
                    &mut out_ptr,
                    width,
                    height,
                    c"interesting".as_ptr(),
                    i,
                    null() as *const c_char,
                ),
                None => vips_sys::vips_smartcrop(
                    self.c.as_ptr(),
                    &mut out_ptr,
                    width,
                    height,
                    null() as *const c_char,
                ),
            }
        };
        result_with_ret(out_ptr, ret)
    }

    /// False-colour visualisation.
    pub fn falsecolour(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_falsecolour(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
        };
        result_with_ret(out_ptr, ret)
    }

    /// Premultiply alpha.
    pub fn premultiply(&self, max_alpha: Option<f64>) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            match max_alpha {
                Some(m) => vips_sys::vips_premultiply(
                    self.c.as_ptr(),
                    &mut out_ptr,
                    c"max-alpha".as_ptr(),
                    m,
                    null() as *const c_char,
                ),
                None => vips_sys::vips_premultiply(
                    self.c.as_ptr(),
                    &mut out_ptr,
                    null() as *const c_char,
                ),
            }
        };
        result_with_ret(out_ptr, ret)
    }

    /// Undo premultiply.
    pub fn unpremultiply(&self, max_alpha: Option<f64>) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            match max_alpha {
                Some(m) => vips_sys::vips_unpremultiply(
                    self.c.as_ptr(),
                    &mut out_ptr,
                    c"max-alpha".as_ptr(),
                    m,
                    null() as *const c_char,
                ),
                None => vips_sys::vips_unpremultiply(
                    self.c.as_ptr(),
                    &mut out_ptr,
                    null() as *const c_char,
                ),
            }
        };
        result_with_ret(out_ptr, ret)
    }

    /// Sequential access hint (streaming pipelines).
    pub fn sequential(&self) -> Result<VipsImage<'a>> {
        let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
        // SAFETY: live image.
        let ret = unsafe {
            vips_sys::vips_sequential(self.c.as_ptr(), &mut out_ptr, null() as *const c_char)
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
                self.c.as_ptr(),
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
        // SAFETY: libvips allocates a buffer we own and must `g_free`.
        unsafe {
            let mut result_size: usize = 0;
            let ptr = vips_sys::vips_image_write_to_memory(
                self.c.as_ptr(),
                &mut result_size as *mut usize,
            ) as *mut u8;
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

    /// Encode to an in-memory buffer for `suffix` (e.g. `".jpg"`, `".png"`).
    ///
    /// Uses `vips_image_write_to_buffer` so the saver is chosen from the suffix.
    pub fn write_to_buffer(&self, suffix: &str) -> Result<Vec<u8>> {
        let suffix = CString::new(suffix)
            .map_err(|_| Error::InitFailed("invalid suffix: contains NUL".into()))?;
        let mut buf: *mut c_void = null_mut();
        let mut size: usize = 0;
        // SAFETY: live image; on success libvips allocates `buf` of `size` bytes.
        let ret = unsafe {
            vips_sys::vips_image_write_to_buffer(
                self.c.as_ptr(),
                suffix.as_ptr(),
                &mut buf,
                &mut size,
                null() as *const c_char,
            )
        };
        ffi::ret_to_result(ret)?;
        if buf.is_null() || size == 0 {
            return Ok(Vec::new());
        }
        // SAFETY: non-null buffer of `size` bytes; free with g_free after copy.
        let out = unsafe {
            let slice = std::slice::from_raw_parts(buf as *const u8, size);
            let owned = slice.to_vec();
            vips_sys::g_free(buf);
            owned
        };
        Ok(out)
    }

    /// JPEG-specific writer (quality path for `.jpg` destinations).
    pub fn write_jpeg(&self, path: impl AsRef<Path>, q: Option<i32>) -> Result<()> {
        let path = path_to_cstring(path.as_ref())?;
        let ret = unsafe {
            match q {
                Some(q) => vips_sys::vips_jpegsave(
                    self.c.as_ptr(),
                    path.as_ptr(),
                    c"Q".as_ptr(),
                    q,
                    null() as *const c_char,
                ),
                None => {
                    vips_sys::vips_jpegsave(self.c.as_ptr(), path.as_ptr(), null() as *const c_char)
                }
            }
        };
        result_draw(ret)
    }
}

fn result<'a>(ptr: *mut vips_sys::VipsImage) -> Result<VipsImage<'a>> {
    let owned = ffi::image_from_ptr(ptr)?;
    // SAFETY: ownership transferred from libvips into this wrapper.
    Ok(unsafe { VipsImage::from_raw(owned.into_ptr()) })
}

fn result_with_ret<'a>(ptr: *mut vips_sys::VipsImage, ret: c_int) -> Result<VipsImage<'a>> {
    let owned = ffi::image_from_ret(ptr, ret)?;
    // SAFETY: ownership transferred from libvips into this wrapper.
    Ok(unsafe { VipsImage::from_raw(owned.into_ptr()) })
}

fn result_draw(ret: c_int) -> Result<()> {
    ffi::ret_to_result(ret)
}

/// Loader nickname libvips would use for `path` (e.g. `"VipsForeignLoadJpegFile"`).
///
/// Returns `None` when no loader is registered for the file.
pub fn find_load(path: impl AsRef<Path>) -> Option<String> {
    let path = path_to_cstring(path.as_ref()).ok()?;
    // SAFETY: path is a valid C string; result is a static/owned C string.
    let ptr = unsafe { vips_sys::vips_foreign_find_load(path.as_ptr()) };
    if ptr.is_null() {
        return None;
    }
    // SAFETY: non-null NUL-terminated string from libvips.
    let s = unsafe { std::ffi::CStr::from_ptr(ptr) };
    s.to_str().ok().map(str::to_owned)
}

/// Saver nickname libvips would use for `path` (e.g. `"VipsForeignSaveJpegFile"`).
pub fn find_save(path: impl AsRef<Path>) -> Option<String> {
    let path = path_to_cstring(path.as_ref()).ok()?;
    // SAFETY: path is a valid C string; result is a static/owned C string.
    let ptr = unsafe { vips_sys::vips_foreign_find_save(path.as_ptr()) };
    if ptr.is_null() {
        return None;
    }
    // SAFETY: non-null NUL-terminated string from libvips.
    let s = unsafe { std::ffi::CStr::from_ptr(ptr) };
    s.to_str().ok().map(str::to_owned)
}

/// Create a one-band 8-bit black image (`vips_black`).
pub fn black(width: u32, height: u32) -> Result<VipsImage<'static>> {
    let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
    // SAFETY: pure constructor.
    let ret = unsafe {
        vips_sys::vips_black(
            &mut out_ptr,
            width as i32,
            height as i32,
            null() as *const c_char,
        )
    };
    result_with_ret(out_ptr, ret)
}

/// Create a two-band coordinate image (`vips_xyz`).
pub fn xyz(width: u32, height: u32) -> Result<VipsImage<'static>> {
    let mut out_ptr: *mut vips_sys::VipsImage = null_mut();
    // SAFETY: pure constructor.
    let ret = unsafe {
        vips_sys::vips_xyz(
            &mut out_ptr,
            width as i32,
            height as i32,
            null() as *const c_char,
        )
    };
    result_with_ret(out_ptr, ret)
}
