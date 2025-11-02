//! Unify error types and libvips error capture
//! ! This module defines a unified `Error` enum to represent errors from libvips operations,
//! ! along with utility functions to capture and handle libvips error messages.
//!
//! ! # Example
//! ! ```no_run
//! ! use vips::error::{Error, Result, take_vips_error, code_to_result};
//! !
//! ! fn example() -> Result<()> {
//! !     let code = unsafe { vips_sys::vips_some_function() }; // hypothetical
//! !     code_to_result(code)
//! ! }
//! ! ```
//!

use std::ffi::CStr;

/// A specialized `Result` type for libvips operations
///
/// # Example
/// ```no_run
/// use vips::error::{Error, Result};
///
/// fn example() -> Result<()> {
///     // some libvips operation
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InitFailed(String),
    Vips(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InitFailed(s) => write!(f, "vips init failed: {}", s),
            Error::Vips(s) => write!(f, "vips error: {}", s),
        }
    }
}

impl std::error::Error for Error {}

/// Read and empty the error buffer of libvips
///
/// # Returns
/// An `Option<String>` containing the error message if there was an error, or `None` if there was no error.
///
pub(crate) fn take_vips_error() -> Option<String> {
    unsafe {
        let ptr = vips_sys::vips_error_buffer();
        if ptr.is_null() {
            return None;
        }
        let msg = CStr::from_ptr(ptr).to_string_lossy().into_owned();
        vips_sys::vips_error_clear();
        if msg.trim().is_empty() {
            None
        } else {
            Some(msg)
        }
    }
}

/// Convert the return code (0=OK, non-0=ERR) convention in libvips to Result
///
/// # Arguments
/// * `code` - The return code from a libvips function
///
/// # Returns
/// A `Result<()>` which is `Ok(())` if the code is 0, or `Err(Error::Vips)` with the error message otherwise.
///
/// # Example
/// ```no_run
/// use vips::error::code_to_result;
/// fn example() -> vips::error::Result<()> {
///     let code = unsafe { vips_sys::vips_some_function() }; // hypothetical
///     code_to_result(code)
/// }
/// ```
#[allow(dead_code)]
pub(crate) fn code_to_result(code: i32) -> Result<()> {
    if code == 0 {
        Ok(())
    } else {
        let msg = take_vips_error().unwrap_or_else(|| "unknown vips error".to_string());
        Err(Error::Vips(msg))
    }
}

/// Retrieve the current libvips error message.
///
/// # Returns
/// A `String` containing the current error message from libvips.
///
/// # Example
/// ```no_run
/// use vips::current_error;
///
/// fn main() {
///     let error_msg = current_error();
///     println!("Current libvips error: {}", error_msg);
/// }
/// ```
pub fn current_error() -> String {
    let msg = unsafe { CStr::from_ptr(vips_sys::vips_error_buffer()) };
    msg.to_str().unwrap().to_string()
}
