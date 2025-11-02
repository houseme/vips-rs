//! vips-rs: Lightweight safety encapsulation of libvips (infrastructure such as initialization, concurrency, caching, versioning, etc.)
//!
//! Characteristics:
//! - Remove 'lazy_static' and complete the global initialization with the standard library 'OnceLock';
//! - Provide 'init()'/'is_initialized()', concurrency/cache control, version information;
//! - Unified error handling (grab 'vips_error_buffer()');
//! - Documentation examples with basic tests.
//!
//! Usage examples:
//! ```no_run
//! use vips::{init, set_concurrency};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // initialization (idempotent), recommended to call as early as possible
//!     init(Some("my-app"))?;
//!
//!     // Configure concurrency (default = number of CPU cores)
//!     vips::set_concurrency(4);
//!
//!     // Tuning the cache
//!     vips::set_max_operations(1000);
//!     vips::set_max_mem_bytes(256 * 1024 * 1024);
//!     vips::set_max_files(100);
//!
//!     Ok(())
//! }
//! ```

pub use crate::cache::*;
pub use crate::concurrency::{concurrency, set_concurrency};
pub use crate::error::{code_to_result, take_vips_error, Error, Result};
pub use crate::init::{init, is_initialized};
pub use crate::version::{version, version_string};

mod instance;
pub use instance::VipsInstance;

mod image;
pub use image::VipsImage;

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

// Simply re-export, no more repeated declaration of extern "C"
pub use vips_sys::vips_call as call;
