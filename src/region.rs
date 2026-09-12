use crate::VipsImage;
use crate::ffi;
use std::os::raw::c_void;
use std::ptr::NonNull;

/// Safe RAII wrapper around a libvips `VipsRegion*`.
pub struct VipsRegion {
    c: NonNull<vips_sys::VipsRegion>,
}

impl VipsRegion {
    /// Create a new region for `image`.
    pub fn new(image: &VipsImage) -> Option<VipsRegion> {
        // SAFETY: `image.as_ptr()` is a live VipsImage; region is owned by us.
        let c = unsafe { vips_sys::vips_region_new(image.as_ptr()) };
        NonNull::new(c).map(|c| VipsRegion { c })
    }

    /// Borrow the underlying pointer for FFI. Valid while `self` lives.
    #[inline]
    pub fn as_ptr(&self) -> *mut vips_sys::VipsRegion {
        self.c.as_ptr()
    }
}

impl Drop for VipsRegion {
    fn drop(&mut self) {
        // SAFETY: exclusive ownership of a live GObject region.
        unsafe { ffi::unref(self.c.as_ptr().cast::<c_void>()) };
    }
}
