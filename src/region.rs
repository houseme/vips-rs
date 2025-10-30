use crate::ffi;
use crate::VipsImage;
use std::os::raw::c_void;

pub struct VipsRegion {
    pub c: *mut ffi::VipsRegion,
}

impl VipsRegion {
    pub fn new(image: &VipsImage) -> VipsRegion {
        let c = unsafe { ffi::vips_region_new(image.c) };
        VipsRegion { c }
    }
}

impl Drop for VipsRegion {
    fn drop(&mut self) {
        unsafe {
            ffi::g_object_unref(self.c as *mut c_void);
        }
    }
}
