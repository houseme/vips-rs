use crate::VipsImage;
use std::os::raw::c_void;

pub struct VipsRegion {
    pub c: *mut vips_sys::VipsRegion,
}

impl VipsRegion {
    pub fn new(image: &VipsImage) -> VipsRegion {
        let c = unsafe { vips_sys::vips_region_new(image.c) };
        VipsRegion { c }
    }
}

impl Drop for VipsRegion {
    fn drop(&mut self) {
        unsafe {
            vips_sys::g_object_unref(self.c as *mut c_void);
        }
    }
}
