//! vips-rs: Lightweight security encapsulation of libvips (infrastructure such as initialization, concurrency, caching, versioning, etc.)
//!
//! Characteristic:
//! - Remove 'lazy_static' and complete the global initialization with the standard library 'OnceLock';
//! - Provide 'init()'/'is_initialized()', concurrency/cache control, version information;
//! - Unified error handling (grab 'vips_error_buffer()');
//! - Documentation examples with basic tests.
//!
//! Usage examples:
//! ```no_run
//! use vips::{init, set_concurrency, cache};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     initialization (idempotency), it is recommended to call it as early as possible
//!     init(Some("my-app"))?;
//!
//!     // Configure concurrency (default = number of CPU cores)
//!     vips::set_concurrency(4);
//!
//!     // Tuning the cache
//!     cache::set_max_operations(1000);
//!     cache::set_max_mem_bytes(256 * 1024 * 1024);
//!     cache::set_max_files(100);
//!
//!     Ok(())
//! }
//! ```

#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
extern crate vips_sys as ffi;

pub use crate::cache::*;
pub use crate::concurrency::{concurrency, set_concurrency};
pub use crate::error::{Error, Result};
pub use crate::init::{init, is_initialized};
pub use crate::version::{version, version_string};
// re-exports modules
mod common;
pub use common::*;

mod instance;
pub use instance::VipsInstance;

mod image;
pub use image::VipsImage;

mod interpolate;
pub use interpolate::{VipsInterpolate, VipsInterpolateMethod};

mod region;
pub use region::VipsRegion;

mod buffer;
mod cache;
mod concurrency;
mod error;
mod init;
mod version;

pub use buffer::VipsBuffer;
// re-exports simple structs
pub use ffi::VipsRect;

// re-exports native enums
pub use ffi::{
    VipsAccess, VipsAlign, VipsAngle, VipsAngle45, VipsArgumentFlags, VipsBBits, VipsBandFormat,
    VipsBlendMode, VipsCoding, VipsCombine, VipsCombineMode, VipsCompassDirection, VipsDemandStyle,
    VipsDirection, VipsExtend, VipsForeignDzContainer, VipsForeignDzDepth, VipsForeignDzLayout,
    VipsForeignFlags, VipsForeignPngFilter, VipsForeignTiffCompression, VipsForeignTiffPredictor,
    VipsForeignTiffResunit, VipsForeignWebpPreset, VipsFormatFlags, VipsImageType, VipsIntent,
    VipsInteresting, VipsInterpretation, VipsKernel, VipsOperationBoolean, VipsOperationComplex,
    VipsOperationComplex2, VipsOperationComplexget, VipsOperationFlags, VipsOperationMath,
    VipsOperationMath2, VipsOperationMorphology, VipsOperationRelational, VipsOperationRound,
    VipsPCS, VipsPrecision, VipsSaveable, VipsSize, VipsToken,
};

// Simply re-export, no more repeated declaration of extern "C"
pub use ffi::vips_call as call;
