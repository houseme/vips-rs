use crate::current_error;
use crate::VipsInterpolate;
use ffi::{VipsBandFormat, VipsCombineMode, VipsDirection, VipsKernel, VipsSize};
use std::error::Error;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_void;
use std::ptr::null;
use std::ptr::null_mut;

pub struct VipsImage<'a> {
    pub c: *mut ffi::VipsImage,
    marker: PhantomData<&'a ()>,
}

impl<'a> Drop for VipsImage<'a> {
    fn drop(&mut self) {
        unsafe {
            ffi::g_object_unref(self.c as *mut c_void);
        }
    }
}

// callback used by gobjects
pub unsafe extern "C" fn image_postclose(ptr: *mut ffi::VipsImage, user_data: *mut c_void) {
    let b: Box<Box<[u8]>> = Box::from_raw(user_data as *mut Box<[u8]>);
    drop(b);
}

impl<'a> VipsImage<'a> {
    //
    // ─── CONSTRUCTORS ───────────────────────────────────────────────────────────────
    //

    pub fn new() -> Result<VipsImage<'a>, Box<dyn Error>> {
        let c = unsafe { ffi::vips_image_new() };
        result(c)
    }

    pub fn new_memory() -> Result<VipsImage<'a>, Box<dyn Error>> {
        let c = unsafe { ffi::vips_image_new_memory() };
        result(c)
    }

    pub fn from_file<S: Into<Vec<u8>>>(path: S) -> Result<VipsImage<'a>, Box<dyn Error>> {
        let path = CString::new(path)?;
        let c = unsafe { ffi::vips_image_new_from_file(path.as_ptr(), null() as *const c_char) };
        result(c)
    }

    pub fn from_memory(
        buf: Vec<u8>,
        width: u32,
        height: u32,
        bands: u8,
        format: VipsBandFormat,
    ) -> Result<VipsImage<'a>, Box<dyn Error>> {
        let b: Box<[_]> = buf.into_boxed_slice();
        let c = unsafe {
            ffi::vips_image_new_from_memory(
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
            ffi::g_signal_connect_data(
                c as *mut c_void,
                c"postclose".as_ptr(),
                Some(callback),
                raw,
                None,
                ffi::GConnectFlags::G_CONNECT_AFTER,
            );
        };

        result(c)
    }

    pub fn from_memory_reference(
        buf: &'a [u8],
        width: u32,
        height: u32,
        bands: u8,
        format: VipsBandFormat,
    ) -> Result<VipsImage<'a>, Box<dyn Error>> {
        let c = unsafe {
            ffi::vips_image_new_from_memory(
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

    // formatted
    pub fn from_buffer(buf: &'a [u8]) -> Result<VipsImage<'a>, Box<dyn Error>> {
        let c = unsafe {
            ffi::vips_image_new_from_buffer(
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

    pub fn draw_rect(
        &mut self,
        ink: &[f64],
        left: u32,
        top: u32,
        width: u32,
        height: u32,
    ) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_rect(
                self.c as *mut ffi::VipsImage,
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
    ) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_rect1(
                self.c as *mut ffi::VipsImage,
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
    pub fn draw_point(&mut self, ink: &[f64], x: i32, y: i32) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_point(
                self.c as *mut ffi::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                x,
                y,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_point1(&mut self, ink: f64, x: i32, y: i32) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_point1(
                self.c as *mut ffi::VipsImage,
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
    ) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_image(
                self.c as *mut ffi::VipsImage,
                img.c as *mut ffi::VipsImage,
                x,
                y,
                c"mode".as_ptr(),
                mode,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_mask(
        &mut self,
        ink: &[f64],
        mask: &VipsImage,
        x: i32,
        y: i32,
    ) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_mask(
                self.c as *mut ffi::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                mask.c as *mut ffi::VipsImage,
                x,
                y,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_mask1(
        &mut self,
        ink: f64,
        mask: &VipsImage,
        x: i32,
        y: i32,
    ) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_mask1(
                self.c as *mut ffi::VipsImage,
                ink,
                mask.c as *mut ffi::VipsImage,
                x,
                y,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_line(
        &mut self,
        ink: &[f64],
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_line(
                self.c as *mut ffi::VipsImage,
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
    pub fn draw_line1(
        &mut self,
        ink: f64,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_line1(
                self.c as *mut ffi::VipsImage,
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
    pub fn draw_circle(
        &mut self,
        ink: &[f64],
        cx: i32,
        cy: i32,
        r: i32,
        fill: bool,
    ) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_circle(
                self.c as *mut ffi::VipsImage,
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
    pub fn draw_circle1(
        &mut self,
        ink: f64,
        cx: i32,
        cy: i32,
        r: i32,
        fill: bool,
    ) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_circle1(
                self.c as *mut ffi::VipsImage,
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
    pub fn draw_flood(&mut self, ink: &[f64], x: i32, y: i32) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_flood(
                self.c as *mut ffi::VipsImage,
                ink.as_ptr() as *mut f64,
                ink.len() as i32,
                x,
                y,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_flood1(&mut self, ink: f64, x: i32, y: i32) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_flood1(
                self.c as *mut ffi::VipsImage,
                ink,
                x,
                y,
                null() as *const c_char,
            )
        };
        result_draw(ret)
    }
    pub fn draw_smudge(
        &mut self,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
    ) -> Result<(), Box<dyn Error>> {
        let ret = unsafe {
            ffi::vips_draw_smudge(
                self.c as *mut ffi::VipsImage,
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
    ) -> Result<VipsImage<'a>, Box<dyn Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_merge(
                self.c as *mut ffi::VipsImage,
                another.c as *mut ffi::VipsImage,
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
    ) -> Result<VipsImage<'_>, Box<dyn Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_mosaic(
                self.c as *mut ffi::VipsImage,
                sec.c as *mut ffi::VipsImage,
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
    ) -> Result<VipsImage<'_>, Box<dyn Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            match interpolate {
                Some(interpolate) => ffi::vips_mosaic1(
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
                None => ffi::vips_mosaic1(
                    self.c as *mut ffi::VipsImage,
                    sec.c as *mut ffi::VipsImage,
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
    ) -> Result<VipsImage<'_>, Box<dyn Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            match interpolate {
                Some(interpolate) => ffi::vips_match(
                    self.c as *mut ffi::VipsImage,
                    sec.c as *mut ffi::VipsImage,
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
                    interpolate.c as *mut ffi::VipsInterpolate,
                    null() as *const c_char,
                ),
                None => ffi::vips_match(
                    self.c as *mut ffi::VipsImage,
                    sec.c as *mut ffi::VipsImage,
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
    ) -> Result<VipsImage<'_>, Box<dyn Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_globalbalance(
                self.c as *mut ffi::VipsImage,
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

    pub fn remosaic(&self, old_str: &str, new_str: &str) -> Result<VipsImage<'_>, Box<dyn Error>> {
        let old_str = CString::new(old_str)?;
        let new_str = CString::new(new_str)?;
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_remosaic(
                self.c as *mut ffi::VipsImage,
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

    fn width(&self) -> u32 {
        unsafe { (*self.c).Xsize as u32 }
    }

    fn height(&self) -> u32 {
        unsafe { (*self.c).Ysize as u32 }
    }

    //
    // ─── RESIZE ─────────────────────────────────────────────────────────────────────
    //

    pub fn thumbnail(
        &self,
        width: u32,
        height: u32,
        size: VipsSize,
    ) -> Result<VipsImage<'_>, Box<dyn Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        unsafe {
            ffi::vips_thumbnail_image(
                self.c as *mut ffi::VipsImage,
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

    // default: block shrink + lanczos3
    fn resize(
        &self,
        scale: f64,
        vscale: Option<f64>,
        kernel: Option<VipsKernel>,
    ) -> Result<VipsImage<'_>, Box<dyn Error>> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_resize(
                self.c as *mut ffi::VipsImage,
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
    fn resize_to_size(
        &self,
        width: u32,
        height: Option<u32>,
        kernel: Option<VipsKernel>,
    ) -> Result<VipsImage<'_>, Box<dyn Error>> {
        self.resize(
            width as f64 / self.width() as f64,
            height.map(|h| h as f64 / self.height() as f64),
            kernel,
        )
    }

    // low-level
    // default: 2 * 1D lanczos3 (not recommended for shrink factor > 3)
    // or other kernels
    fn reduce(
        &self,
        hshrink: f64,
        vshrink: f64,
        kernel: Option<VipsKernel>,
        centre: Option<bool>,
    ) -> VipsImage<'_> {
        let mut out_ptr: *mut ffi::VipsImage = null_mut();
        let ret = unsafe {
            ffi::vips_reduce(
                self.c as *mut ffi::VipsImage,
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
            panic!("{}", current_error())
        }
    }

    fn shrink(&self) -> VipsImage<'_> {
        // simple average of nxn -> 1/n size
        // use a 2x2 box filter to average neighbouring pixels
        self.reduce(2.0, 2.0, Some(VipsKernel::VIPS_KERNEL_LINEAR), Some(false))
    }

    //
    // ─── IO ─────────────────────────────────────────────────────────────────────────
    //

    fn jpegsave<S: Into<Vec<u8>>>(&mut self, path: S) -> Result<(), Box<dyn Error>> {
        let path = CString::new(path)?;
        let ret = unsafe {
            ffi::vips_jpegsave(
                self.c as *mut ffi::VipsImage,
                path.as_ptr(),
                null() as *const c_char,
            )
        };
        match ret {
            0 => Ok(()),
            _ => Err(current_error().into()),
        }
    }

    pub fn write_to_file<S: Into<Vec<u8>>>(&self, path: S) -> Result<(), Box<dyn Error>> {
        let path = CString::new(path)?;
        let ret = unsafe {
            ffi::vips_image_write_to_file(
                self.c as *mut ffi::VipsImage,
                path.as_ptr(),
                null() as *const c_char,
            )
        };
        match ret {
            0 => Ok(()),
            _ => Err(current_error().into()),
        }
    }

    //
    // ─── CONVERT ────────────────────────────────────────────────────────────────────
    //

    fn to_vec(&self) -> Vec<u8> {
        unsafe {
            let mut result_size: usize = 0;
            let ptr =
                ffi::vips_image_write_to_memory(self.c, &mut result_size as *mut usize) as *mut u8;
            if ptr.is_null() {
                return Vec::new();
            }
            let slice = std::slice::from_raw_parts(ptr as *const u8, result_size);
            let vec = slice.to_vec();
            ffi::g_free(ptr as *mut c_void); // Release with GLib
            vec
        }
    }
}

fn result<'a>(ptr: *mut ffi::VipsImage) -> Result<VipsImage<'a>, Box<dyn Error>> {
    if ptr.is_null() {
        Err(current_error().into())
    } else {
        Ok(VipsImage {
            c: ptr,
            marker: PhantomData,
        })
    }
}

fn result_with_ret<'a>(
    ptr: *mut ffi::VipsImage,
    ret: c_int,
) -> Result<VipsImage<'a>, Box<dyn Error>> {
    if ret == 0 {
        Ok(VipsImage {
            c: ptr,
            marker: PhantomData,
        })
    } else {
        Err(current_error().into())
    }
}

fn result_draw(ret: c_int) -> Result<(), Box<dyn Error>> {
    match ret {
        0 => Ok(()),
        -1 => Err(current_error().into()),
        _ => Err("Unknown error from libvips".into()),
    }
}
