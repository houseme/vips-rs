use crate::ffi;
use crate::{Error, Result, VipsRegion, take_vips_error};
use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr::NonNull;

/// Safe RAII wrapper around a libvips `VipsInterpolate*`.
pub struct VipsInterpolate {
    c: NonNull<vips_sys::VipsInterpolate>,
    is_static: bool,
}

impl Drop for VipsInterpolate {
    fn drop(&mut self) {
        if !self.is_static {
            // SAFETY: non-static interpolators are uniquely owned.
            unsafe { ffi::unref(self.c.as_ptr().cast::<c_void>()) };
        }
    }
}

impl VipsInterpolate {
    /// Borrow the underlying pointer for FFI. Valid while `self` lives.
    #[inline]
    pub fn as_ptr(&self) -> *mut vips_sys::VipsInterpolate {
        self.c.as_ptr()
    }

    /// Create a new interpolator by nickname (e.g. `"bilinear"`).
    pub fn new(nickname: &str) -> Result<VipsInterpolate> {
        let nickname = CString::new(nickname)
            .map_err(|_| Error::Other("Invalid nickname: contains null byte".to_string()))?;
        // SAFETY: libvips returns a new interpolator or null.
        let c = unsafe { vips_sys::vips_interpolate_new(nickname.as_ptr()) };
        let c = NonNull::new(c).ok_or_else(|| {
            Error::Vips(take_vips_error().unwrap_or_else(|| {
                "Unknown error from libvips".to_string()
            }))
        })?;
        Ok(VipsInterpolate {
            c,
            is_static: false,
        })
    }

    /// Shared nearest-neighbour interpolator (static lifetime inside libvips).
    pub fn nearest_static() -> VipsInterpolate {
        // SAFETY: static singleton; never freed by us (`is_static`).
        let c = unsafe { vips_sys::vips_interpolate_nearest_static() };
        let c = NonNull::new(c).expect("vips_interpolate_nearest_static returned null");
        VipsInterpolate {
            c,
            is_static: true,
        }
    }

    /// Shared bilinear interpolator (static lifetime inside libvips).
    pub fn bilinear_static() -> VipsInterpolate {
        // SAFETY: static singleton; never freed by us (`is_static`).
        let c = unsafe { vips_sys::vips_interpolate_bilinear_static() };
        let c = NonNull::new(c).expect("vips_interpolate_bilinear_static returned null");
        VipsInterpolate {
            c,
            is_static: true,
        }
    }

    /// Get the interpolation method handle.
    pub fn method(&self) -> VipsInterpolateMethod {
        // SAFETY: `self.c` is a live interpolator.
        let c = unsafe { vips_sys::vips_interpolate_get_method(self.c.as_ptr()) };
        VipsInterpolateMethod { c }
    }

    /// Window size used by the interpolator.
    pub fn window_size(&self) -> i32 {
        // SAFETY: `self.c` is a live interpolator.
        unsafe { vips_sys::vips_interpolate_get_window_size(self.c.as_ptr()) }
    }

    /// Window offset used by the interpolator.
    pub fn window_offset(&self) -> i32 {
        // SAFETY: `self.c` is a live interpolator.
        unsafe { vips_sys::vips_interpolate_get_window_offset(self.c.as_ptr()) }
    }
}

/// Function-pointer wrapper for a `VipsInterpolate` implementation.
pub struct VipsInterpolateMethod {
    c: vips_sys::VipsInterpolateMethod,
}

impl VipsInterpolateMethod {
    /// Interpolate one pixel into `out`.
    ///
    /// `out` must be large enough for the interpolator's window (use
    /// [`VipsInterpolate::window_size`] as a guide).
    pub fn call(
        &self,
        interpolate: &VipsInterpolate,
        in_: &VipsRegion,
        out: &mut [u8],
        x: f64,
        y: f64,
    ) {
        // SAFETY: method ptr is valid for the interpolator; region/buffer live
        // for the duration of the call. Caller sized `out` for the window.
        unsafe {
            if let Some(func) = self.c {
                func(
                    interpolate.as_ptr(),
                    out.as_mut_ptr() as *mut c_void,
                    in_.as_ptr(),
                    x,
                    y,
                );
            }
        }
    }
}
