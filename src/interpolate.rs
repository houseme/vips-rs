use crate::current_error;
use crate::ffi;
use crate::VipsRegion;
use std::error::Error;
use std::ffi::CString;
use std::os::raw::c_void;

pub struct VipsInterpolate {
    pub c: *mut ffi::VipsInterpolate,
    is_static: bool,
}

impl Drop for VipsInterpolate {
    fn drop(&mut self) {
        if !self.is_static {
            unsafe {
                ffi::g_object_unref(self.c as *mut c_void);
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

    pub fn new(nickname: &str) -> Result<VipsInterpolate, Box<dyn Error>> {
        let nickname = CString::new(nickname)?;
        let c = unsafe { ffi::vips_interpolate_new(nickname.as_ptr()) };
        if c.is_null() {
            Err(current_error().into())
        } else {
            Ok(VipsInterpolate {
                c,
                is_static: false,
            })
        }
    }

    pub fn nearest_static() -> VipsInterpolate {
        let c = unsafe { ffi::vips_interpolate_nearest_static() };
        VipsInterpolate { c, is_static: true }
    }

    pub fn bilinear_static() -> VipsInterpolate {
        let c = unsafe { ffi::vips_interpolate_bilinear_static() };
        VipsInterpolate { c, is_static: true }
    }

    //
    // ─── PROPERTIES ─────────────────────────────────────────────────────────────────
    //

    pub fn method(&self) -> VipsInterpolateMethod {
        let c = unsafe { ffi::vips_interpolate_get_method(self.c) };
        VipsInterpolateMethod { c }
    }

    pub fn window_size(&self) -> i32 {
        unsafe { ffi::vips_interpolate_get_window_size(self.c) }
    }

    pub fn window_offset(&self) -> i32 {
        unsafe { ffi::vips_interpolate_get_window_offset(self.c) }
    }
}

pub struct VipsInterpolateMethod {
    c: ffi::VipsInterpolateMethod,
}

impl VipsInterpolateMethod {
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
