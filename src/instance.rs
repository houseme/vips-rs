use crate::{Error, Result};
use std::ffi::CString;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

static IS_INSTANTIATED: AtomicBool = AtomicBool::new(false);

/// A singleton instance to manage libvips initialization and shutdown.
/// /// # Example
/// ```no_run
/// use vips::*;
///
/// fn main() -> Result<()> {
///     let _instance = VipsInstance::new("app_test", true)?;
///     // Your libvips code here
///     Ok(())
/// }
/// ```
pub struct VipsInstance {}

impl VipsInstance {
    /// Create a new VipsInstance, initializing libvips.
    /// This can only be done once per program execution.
    /// Subsequent attempts will return an error.
    /// # Arguments
    /// * `name` - Application name for libvips initialization.
    /// * `leak_test` - If true, enables leak testing in libvips.
    ///
    /// # Errors
    /// Returns an error if an instance already exists.
    ///
    /// # Example
    /// ```no_run
    /// use vips::*;
    ///
    /// fn main() -> Result<()> {
    ///     let _instance = VipsInstance::new("app_test", true)?;
    ///     // Your libvips code here
    ///     Ok(())
    /// }
    /// ```
    /// # Safety Note
    ///
    /// **Shutdown Timing:**
    /// The libvips library is automatically shut down when the `VipsInstance` is dropped (typically
    /// at the end of `main()`). Any operations referencing libvips after this point
    /// (such as in other static destructors or background threads) may result in undefined behavior.
    /// To avoid this, ensure all libvips operations are completed before the `Vips
    /// Instance` is dropped.
    ///
    pub fn new(name: &str, leak_test: bool) -> Result<VipsInstance> {
        // Try to set false -> true, allowing only once
        match IS_INSTANTIATED.compare_exchange(false, true, Relaxed, Relaxed) {
            Ok(_) => {
                let c = CString::new(name)
                    .map_err(|e| Error::InitFailed(format!("invalid name: {}", e)))?;
                unsafe {
                    vips_sys::vips_init(c.as_ptr());
                    if leak_test {
                        vips_sys::vips_leak_set(leak_test as c_int);
                    }
                }
                Ok(VipsInstance {})
            }
            Err(_) => Err(Error::Other(
                "You cannot create VipsInstance more than once.".to_string(),
            )),
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
