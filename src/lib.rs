//! vips-rs: Lightweight safety encapsulation of libvips (initialization,
//! concurrency, caching, versioning, and image IO).
//!
//! ## Safety model
//!
//! - Raw libvips pointers are **private** to each wrapper. Use safe methods, or
//!   `as_ptr()` when bridging to lower-level FFI (the pointer is valid only
//!   while the wrapper lives; do not unref it).
//! - All `unsafe` blocks are concentrated in this crate with `SAFETY:` notes.
//! - Global `init` is process-locked; RAII types unref on `Drop`.
//!
//! ```no_run
//! use vips::{init, set_concurrency};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     init(Some("my-app"))?;
//!     vips::set_concurrency(4);
//!     vips::set_max_operations(1000);
//!     vips::set_max_mem_bytes(256 * 1024 * 1024);
//!     vips::set_max_files(100);
//!     Ok(())
//! }
//! ```

mod ffi;

pub use crate::cache::*;
pub use crate::concurrency::{concurrency, set_concurrency};
pub use crate::error::{Error, Result, code_to_result, take_vips_error};
pub use crate::init::{init, is_initialized};
pub use crate::version::{version, version_string};

mod instance;
pub use instance::VipsInstance;

mod image;
pub use image::{
    VipsImage, black, find_load, find_save, gaussnoise, grey, perlin, sines, xyz, zone,
};

mod interpolate;
pub use interpolate::{VipsInterpolate, VipsInterpolateMethod};

mod region;
pub use region::VipsRegion;

mod buffer;
pub use buffer::VipsBuffer;
mod cache;
mod concurrency;
mod error;
mod init;
mod version;

pub use vips_sys::{
    VipsAccess, VipsAlign, VipsAngle, VipsAngle45, VipsArgumentFlags, VipsBandFormat,
    VipsBlendMode, VipsCoding, VipsCombine, VipsCombineMode, VipsCompassDirection, VipsDemandStyle,
    VipsDirection, VipsExtend, VipsForeignDzContainer, VipsForeignDzDepth, VipsForeignDzLayout,
    VipsForeignFlags, VipsForeignPngFilter, VipsForeignTiffCompression, VipsForeignTiffPredictor,
    VipsForeignTiffResunit, VipsForeignWebpPreset, VipsFormatFlags, VipsImageType, VipsIntent,
    VipsInteresting, VipsInterpretation, VipsKernel, VipsOperationBoolean, VipsOperationComplex,
    VipsOperationComplex2, VipsOperationComplexget, VipsOperationFlags, VipsOperationMath,
    VipsOperationMath2, VipsOperationMorphology, VipsOperationRelational, VipsOperationRound,
    VipsPCS, VipsPrecision, VipsRect, VipsSize, VipsToken,
};

// Escape hatch for advanced callers — inherently `unsafe` (raw varargs into libvips).
pub use vips_sys::vips_call;
