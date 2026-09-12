//! Internal FFI helpers — the only place that should reason about raw
//! libvips pointers. Public API never exposes mutable ownership of `c`.

use crate::{Error, Result, take_vips_error};
use std::marker::PhantomData;
use std::os::raw::{c_int, c_void};
use std::ptr::NonNull;

/// Owned libvips image pointer transferred into `crate::VipsImage`.
pub(crate) struct OwnedRawImage<'a> {
    ptr: NonNull<vips_sys::VipsImage>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> OwnedRawImage<'a> {
    pub(crate) fn into_ptr(self) -> *mut vips_sys::VipsImage {
        self.ptr.as_ptr()
    }
}

/// Map a libvips `VipsImage*` (null = failure) into a safe wrapper.
pub(crate) fn image_from_ptr<'a>(ptr: *mut vips_sys::VipsImage) -> Result<OwnedRawImage<'a>> {
    let ptr = NonNull::new(ptr).ok_or_else(|| {
        Error::Vips(take_vips_error().unwrap_or_else(|| "Unknown error from libvips".to_string()))
    })?;
    Ok(OwnedRawImage {
        ptr,
        _lifetime: PhantomData,
    })
}

/// Map `(ret, VipsImage*)` the way most libvips operations report success.
pub(crate) fn image_from_ret<'a>(
    ptr: *mut vips_sys::VipsImage,
    ret: c_int,
) -> Result<OwnedRawImage<'a>> {
    match ret {
        0 => image_from_ptr(ptr).map_err(|_| {
            Error::Vips("libvips returned success with a null image pointer".to_string())
        }),
        -1 => {
            Err(Error::Vips(take_vips_error().unwrap_or_else(|| {
                "Unknown error from libvips".to_string()
            })))
        }
        _ => Err(Error::Vips("Unknown error from libvips".to_string())),
    }
}

/// Map a libvips status code used by in-place ops (`0` success, `-1` error).
pub(crate) fn ret_to_result(ret: c_int) -> Result<()> {
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

/// Unref a GObject if non-null.
///
/// # Safety
/// `ptr` must be a valid `GObject*` with a live ref, or null.
pub(crate) unsafe fn unref(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe { vips_sys::g_object_unref(ptr) };
    }
}
