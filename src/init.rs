//! Global initialization: Ensures initialization only once with `OnceLock` and automatically shuts down when the process exits.
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

/// Initialize libvips (idempotent, thread-safe). `app_name` is used for logging
/// and diagnostics; defaults to `"vips-rs"` when `None`.
///
/// Concurrent first-time callers are synchronized by `OnceLock` — only the
/// winning thread performs `vips_init`.
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
    if VIPS.get().is_some() {
        return Ok(());
    }

    let cstr = CString::new(app_name.unwrap_or("vips-rs"))
        .map_err(|e| Error::InitFailed(format!("invalid app name: {e}")))?;

    let rc = unsafe { vips_sys::vips_init(cstr.as_ptr()) };
    if rc != 0 {
        let msg = take_vips_error().unwrap_or_else(|| "vips_init() failed".to_string());
        return Err(Error::InitFailed(msg));
    }

    // Losing a race here means another thread already installed the guard;
    // libvips tolerates extra init calls, and shutdown remains once-only.
    let _ = VIPS.set(InitGuard);
    Ok(())
}

/// Is it currently initialized?
///
/// # Returns
/// `true` if libvips has been initialized, `false` otherwise.
///
/// # Example
/// ```no_run
/// use vips::is_initialized;
///
///  if is_initialized() {
///     println!("libvips is initialized");
///  } else {
///     println!("libvips is not initialized");
///  }
/// ```
#[inline]
pub fn is_initialized() -> bool {
    VIPS.get().is_some()
}
