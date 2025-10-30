//! Unify error types and libvips error capture
use std::ffi::CStr;

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
pub(crate) fn code_to_result(code: i32) -> Result<()> {
    if code == 0 {
        Ok(())
    } else {
        let msg = take_vips_error().unwrap_or_else(|| "unknown vips error".to_string());
        Err(Error::Vips(msg))
    }
}
