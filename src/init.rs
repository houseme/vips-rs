//! Global initialization: Ensures initialization only once with 'OnceLock' and automatically shuts down when the process exits.
//!
//! # Safety Note
//!
//! **Shutdown Timing:**
//! The libvips library is automatically shut down when the process exits, during static destructor execution (after `main()` completes).
//! Any operations referencing libvips after `main()` returns (such as in other static destructors or background threads) may result in undefined behavior.
//! To avoid this, ensure all libvips operations are completed before `main()` exits.

use crate::{take_vips_error, Error, Result};
use std::ffi::CString;
use std::sync::OnceLock;

static VIPS: OnceLock<InitGuard> = OnceLock::new();

pub(crate) struct InitGuard;

impl Drop for InitGuard {
    fn drop(&mut self) {
        unsafe {
            // Make sure libvips global resources (cache, thread pool, etc.) are released on exit
            vips_sys::vips_shutdown();
        }
    }
}

/// Initialize libvips (idempotent). `app_name` can be used for logging and diagnostic display.
/// If `app_name` is `None`, a default name "vips-rs" will be used.
///
/// # Errors
/// Returns `Error::InitFailed` if initialization fails.
///
/// # Example
/// ```no_run
/// use vips::*;
///
/// fn main() -> Result<()> {
///     init(Some("my_app"))?;
///     // Your libvips code here
///     Ok(())
/// }
/// ```
pub fn init(app_name: Option<&str>) -> Result<()> {
    // If initialized, return directly.
    if VIPS.get().is_some() {
        return Ok(());
    }

    let cstr = CString::new(app_name.unwrap_or("vips-rs")) // argv0
        .map_err(|e| Error::InitFailed(format!("invalid app name: {}", e)))?;

    let rc = unsafe { vips_sys::vips_init(cstr.as_ptr()) };
    if rc != 0 {
        let msg = take_vips_error().unwrap_or_else(|| "vips_init() failed".to_string());
        return Err(Error::InitFailed(msg));
    }

    // Establish a one-time guard, which will automatically drop when the process exits -> vips_shutdown()
    let _ = VIPS.set(InitGuard);
    Ok(())
}

/// Is it currently initialized?
///
/// # Safety
/// This function is safe to call from multiple threads.
///
/// # Returns
/// `true` if libvips has been initialized, `false` otherwise.
///
/// /// # Example
/// ```no_run
/// use vips::is_initialized;
///
///  if is_initialized() {
///     println!("libvips is initialized");
///  } else {
///     println!("libvips is not initialized");
///  }
/// ```
pub fn is_initialized() -> bool {
    VIPS.get().is_some()
}
