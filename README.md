# vips-rs

[English](README.md) | [Chinese Simplified](README_CN.md)

[![Rust](https://github.com/houseme/vips-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/houseme/vips-rs/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/vips.svg)](https://crates.io/crates/vips)
[![docs.rs](https://docs.rs/vips/badge.svg)](https://docs.rs/vips/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Downloads](https://img.shields.io/crates/d/vips)](https://crates.io/crates/vips)

Rust bindings for libvips: fast, low-memory image processing with a safe, ergonomic API.

- Safe wrappers over common libvips APIs
- RAII-style initialization/shutdown management
- Practical helpers for reading, transforming, and writing images
- Lazy pipeline evaluation (libvips demand-driven processing)

Documentation: https://docs.rs/vips/

## Requirements

- Rust >= 1.85.0 (edition 2024)
- libvips installed on your system
    - macOS: `brew install vips`
    - Linux: `apt-get install -y pkg-config libvips libvips-dev` (or your distro equivalent)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
vips = "0.1"
```

## Quick start

```rust
use vips::*;

fn main() -> Result<()> {
    // Initialize libvips once per process. The boolean usually controls auto-shutdown.
    let _instance = VipsInstance::new("app_example", true)?;

    // Create a thumbnail with forced width and height (preferred for files)
    let thumb = VipsImage::thumbnail_file("./examples/images/kodim01.png", 320, 240, VipsSize::VIPS_SIZE_FORCE)?;

    // Save the result
    thumb.write_to_file("kodim01_320x240.jpg")?;
    Ok(())
}
```

## Working with memory

- Own the pixel buffer (simple and recommended):

```rust
let pixels = vec![0u8; 256 * 256 * 3]; // RGB
let img = VipsImage::from_memory(pixels, 256, 256, 3, VipsBandFormat::VIPS_FORMAT_UCHAR)?;
let thumb = img.thumbnail(200, 200, VipsSize::VIPS_SIZE_FORCE)?;
thumb.write_to_file("black_200x200.png")?;
```

- Borrow a pixel buffer (make sure the backing data outlives all derived images):

```rust
let pixels = vec![0u8; 256 * 256 * 3];
let img = VipsImage::from_memory_reference(&pixels, 256, 256, 3, VipsBandFormat::VIPS_FORMAT_UCHAR)?;
let thumb = img.thumbnail(200, 200, VipsSize::VIPS_SIZE_FORCE)?;
thumb.write_to_file("black_ref_200x200.png")?;
```

## Lifetimes and common pitfalls

- Prefer owned constructors (`from_file`, `from_memory`) when possible.
- Borrowing constructors (`from_memory_reference`) tie the image lifetime to the borrowed slice. Do not let the slice
  drop before all derived images are fully used.
- Keep the creator image in scope while using results that reference it. Avoid creating images inside a short inner
  scope and returning derived results from it.

## API highlights

| Area | APIs |
|------|------|
| IO | `from_file`, `from_memory`, `from_memory_reference`, `from_buffer`, `write_to_file`, `write_to_memory`, `write_to_buffer`, `write_jpeg`, `find_load`/`find_save`, `black`/`xyz`/`grey`/`sines`/`zone`/`perlin`/`gaussnoise` |
| Geometry | `thumbnail_file`, `thumbnail_buffer`, `thumbnail` (in-memory), `resize`, `resize_reduce`, `reduce`, `shrink_box`, `crop`, `embed`, `flip`/`rot`/`rotate`/`autorot`, `zoom`, `insert`, `gravity`, `extract_band`, `bandjoin2`/`bandjoin_const`, `copy`/`copy_memory`, `smartcrop`, `ifthenelse` |
| Arithmetic | `add`/`subtract`/`multiply`/`divide`, `linear`/`linear1`, `invert`, `pow_const`, `abs`/`sign`/`clamp`, `floor`/`ceil`/`rint` |
| Math | `sin`/`cos`/`tan`, `ln`/`log10`/`exp`/`exp10` |
| Relational | `less`/`lesseq`/`more`/`moreeq`/`equal_const`/`notequal_const`, boolean/shift const, `bandand`/`bandor`/`bando` |
| Color | `cast`, `colourspace`, `falsecolour`, `premultiply`/`unpremultiply` |
| Filter | `gaussblur`, `sharpen`, `conv`, `erode`/`dilate`, `median`, `sobel`, `canny`, `maplut` |
| Hist | `hist_find`/`hist_local`/`hist_equal`/`hist_entropy`, `labelregions` |
| Stats | `avg`, `min_value`, `max_value`, `deviate`, `percent`, `getpoint`, `find_trim` |
| Properties | `width`, `height`, `size`, `bands` |
| Drawing | `draw_rect`, `draw_line`, `draw_circle`, `draw_flood`, … (in-place) |
| Stitching | `merge`, `mosaic`, `match_`, `globalbalance`, `remosaic` |
| Interpolation | `VipsInterpolate` nearest / bilinear / custom |

## Performance tips

- Prefer `VipsImage::thumbnail_file` / `VipsImage::thumbnail_buffer` over `from_file`/`from_buffer` + `thumbnail` for downscales. They call `vips_thumbnail` / `vips_thumbnail_buffer`, which combine load and resize so libvips can shrink-on-load (often ~3× faster and several times lower peak memory than `thumbnail_image`). Use the instance method `thumbnail` only for already-decoded raw pixels.
- Prefer `write_to_file` over `write_to_memory` when the destination is a path (avoids a full in-memory copy).
- For large downscales (factor ≳ 3 on multi-megapixel sources), prefer `resize_reduce` — it box-pre-shrinks before the final kernel.
- Tune `vips::set_concurrency`, `set_max_operations`, and `set_max_mem_bytes` for your workload.
- libvips pipelines are lazy: operations enqueue work until the image is written or materialized.

## Notes

- Initialization: the crate manages `vips_init`/`vips_shutdown` via `VipsInstance` and standard library primitives (
  `OnceLock`).
- Side effects: most operations return new images; drawing operations modify `self`.
- If a higher-level wrapper is missing, you can still access lower-level bindings in `vips-sys` or use `vips::call(...)`
  to invoke libvips operations directly.

## Local development

Published builds resolve `vips-sys` from crates.io. To develop against a sibling checkout, add a
`[patch]` at the **workspace/consumer root** (or temporarily in this crate):

```toml
[patch.crates-io]
vips-sys = { path = "../vips-sys" }
```

Docker-based check (no local libvips required):

```bash
docker run --rm -v "$PWD/..":/workspace -w /workspace/vips-rs rust:1.88-bookworm \
  bash -c 'apt-get update -qq && apt-get install -y -qq pkg-config libvips-dev && cargo test'
```

## License

[MIT](LICENSE)

## Changelog

See [`CHANGELOG.md`](CHANGELOG.md).
