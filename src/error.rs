//! Unify error types and libvips error capture
//! ! This module defines a unified `Error` enum to represent errors from libvips operations,
//! ! along with utility functions to capture and handle libvips error messages.
//!
//! ! # Example
//! ! ```no_run
//! ! use vips::error::{Error, Result, take_vips_error, code_to_result};
//! !
//! ! fn example() -> Result<()> {
//! !     let code = 0 // hypothetical
//! !     code_to_result(code)
//! ! }
//! ! ```
//!

use std::ffi::CStr;

/// A specialized `Result` type for libvips operations
///
/// # Example
/// ```no_run
/// use vips::{Error, Result};
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
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InitFailed(s) => write!(f, "vips init failed: {}", s),
            Error::Vips(s) => write!(f, "vips error: {}", s),
            Error::Other(s) => write!(f, "other error: {}", s),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Other(e.to_string())
    }
}

/// Read and empty the error buffer of libvips
///
/// # Returns
/// An `Option<String>` containing the error message if there was an error, or `None` if there was no error.
///
pub fn take_vips_error() -> Option<String> {
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
/// use vips::*;
///
/// fn example() -> Result<()> {
///     let code = 1; // hypothetical
///     code_to_result(code)
/// }
/// ```
pub fn code_to_result(code: i32) -> Result<()> {
    if code == 0 {
        Ok(())
    } else {
        let msg = take_vips_error().unwrap_or_else(|| "Unknown error from libvips".to_string());
        Err(Error::Vips(msg))
    }
}
