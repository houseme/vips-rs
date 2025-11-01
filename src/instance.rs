use std::error::Error;
use std::ffi::CString;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

static IS_INSTANTIATED: AtomicBool = AtomicBool::new(false);

pub struct VipsInstance {}

impl VipsInstance {
    pub fn new(name: &str, leak_test: bool) -> Result<VipsInstance, Box<dyn Error>> {
        // Try to set false -> true, allowing only once
        match IS_INSTANTIATED.compare_exchange(false, true, Relaxed, Relaxed) {
            Ok(_) => {
                let c = CString::new(name)?;
                unsafe {
                    vips_sys::vips_init(c.as_ptr());
                    if leak_test {
                        vips_sys::vips_leak_set(leak_test as c_int);
                    }
                }
                Ok(VipsInstance {})
            }
            Err(_) => Err("You cannot create VipsInstance more than once.".into()),
        }
    }
}

impl Drop for VipsInstance {
    fn drop(&mut self) {
        unsafe {
            vips_sys::vips_shutdown();
        }
        // Note: libvips does not support re-initialization after shutdown, so IS_INSTANTIATED is not reset here.
    }
}
