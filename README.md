# vips-rs

[English](README.md) | [Chinese Simplified](README_CN.md)

[![Rust](https://github.com/houseme/vips-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/houseme/vips-rs/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/vips.svg)](https://crates.io/crates/vips)
[![Docs](https://img.shields.io/badge/docs-online-blue)](https://houseme.github.io/vips-rs/vips/)
[![docs.rs](https://docs.rs/vips/badge.svg)](https://docs.rs/vips/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Downloads](https://img.shields.io/crates/d/vips)](https://crates.io/crates/vips)

Rust bindings for libvips: fast, low-memory image processing with a safe, ergonomic API.

- Safe wrappers over common libvips APIs
- RAII-style initialization/shutdown management
- Practical helpers for reading, transforming, and writing images

Documentation: https://houseme.github.io/vips-rs/vips/

## Requirements

- Rust >= 1.80.0
- libvips installed on your system
    - macOS: `brew install vips`
    - Linux: `apt-get install -y libvips libvips-dev` (or your distro equivalent)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
vips = "*"
```

## Quick start

```rust
use vips::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize libvips once per process. The boolean usually controls auto-shutdown.
    let _instance = VipsInstance::new("app_example", true)?;

    // Load an image from file
    let img = VipsImage::from_file("kodim01.png")?;

    // Create a thumbnail with forced width and height
    let thumb = img.thumbnail(320, 240, VipsSize::VIPS_SIZE_FORCE)?;

    // Save the result
    thumb.write_to_file("kodim01_320x240.jpg")?;
    Ok(())
}
```

## Working with memory

- Own the pixel buffer (simple and recommended):

```rust
let pixels = vec![0u8; 256 * 256 * 3]; // RGB
let img = VipsImage::from_memory(pixels, 256, 256, 3, VipsBandFormat::VIPS_FORMAT_UCHAR) ?;
let thumb = img.thumbnail(200, 200, VipsSize::VIPS_SIZE_FORCE) ?;
thumb.write_to_file("black_200x200.png") ?;
```

- Borrow a pixel buffer (make sure the backing data outlives all derived images):

```rust
let pixels = vec![0u8; 256 * 256 * 3];
let img = VipsImage::from_memory_reference( & pixels, 256, 256, 3, VipsBandFormat::VIPS_FORMAT_UCHAR) ?; // The returned image lifetime is tied to `pixels`
let thumb = img.thumbnail(200, 200, VipsSize::VIPS_SIZE_FORCE) ?;
thumb.write_to_file("black_ref_200x200.png") ?;
```

## Lifetimes and common pitfalls

- Prefer owned constructors (`from_file`, `from_memory`) when possible.
- Borrowing constructors (`from_memory_reference`) tie the image lifetime to the borrowed slice. Do not let the slice
  drop before all derived images are fully used.
- Keep the creator image in scope while using results that reference it. Avoid creating images inside a short inner
  scope and returning derived results from it.

## API highlights

- Image IO: from file, from raw memory, from borrowed memory, save to file
- Geometry: thumbnail/resize/reduce/shrink
- Drawing: lines, circles, flood fills (in-place)
- Stitching: `merge`, `mosaic`, `match_`, `globalbalance`
- Interpolation: nearest, bilinear, or custom

The API surface is evolving; see the docs for details and more examples.

## Notes

- Initialization: the crate manages `vips_init`/`vips_shutdown` via `VipsInstance` and standard library primitives (
  `OnceLock`).
- Side effects: most operations return new images; drawing operations modify `self`.
- If a higher-level wrapper is missing, you can still access lower-level bindings in `vips-sys` or use `vips::call(...)`
  to invoke libvips operations directly.

## License

[MIT](LICENSE)

## Changelog

See [`CHANGELOG.md`](CHANGELOG.md).