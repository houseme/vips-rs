use crate::current_error;
use crate::VipsImage;
use std::error::Error;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::null;

pub trait VipsBuffer {
    fn thumbnail(&self, width: u32, height: u32) -> Result<VipsImage<'_>, Box<dyn Error>>;
}

impl VipsBuffer for &[u8] {
    fn thumbnail(&self, width: u32, height: u32) -> Result<VipsImage<'_>, Box<dyn Error>> {
        unsafe {
            let mut out = VipsImage::new_memory()?;
            let ret: c_int = vips_sys::vips_thumbnail_buffer(
                self.as_ptr() as *mut c_void,
                self.len(),
                &mut out.c,
                width as c_int,
                c"height".as_ptr(),
                height as c_int,
                c"size".as_ptr(),
                vips_sys::VipsSize::VIPS_SIZE_FORCE,
                null() as *const c_char,
            );
            if ret == 0 {
                Ok(out)
            } else {
                Err(current_error().into())
            }
        }
    }

    // pub fn jpegload(&self) -> Result<VipsImage, Box<Error>> {
    //     let mut out = VipsImage::new_memory()?;
    //     unsafe {
    //         ffi::vips_jpegload_buffer(self.as_mut_ptr() as *mut c_void, buf.len(), &mut out.c);
    //     }
    //     Ok(out)
    // }
}
