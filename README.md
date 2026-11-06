# vips-rs

[English](README.md) | [Chinese Simplified](README_CN.md)

[![Rust](https://github.com/houseme/vips-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/houseme/vips-rs/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/vips.svg)](https://crates.io/crates/vips)
[![docs.rs](https://docs.rs/vips/badge.svg)](https://docs.rs/vips/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Downloads](https://img.shields.io/crates/d/vips)](https://crates.io/crates/vips)

Rust bindings for libvips: fast, low-memory image processing with a safe, ergonomic API.

- Safe wrappers over common libvips APIs
- Process-wide initialization and RAII image ownership
- Practical helpers for reading, transforming, and writing images
- Lazy pipeline evaluation (libvips demand-driven processing)

Documentation: https://docs.rs/vips/

## Requirements

- Current stable Rust (edition 2024)
- libvips installed on your system
    - macOS: `brew install vips`
    - Linux: `apt-get install -y pkg-config libvips-dev` (or your distro equivalent)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
vips = "0.2"
```

## Quick start

```rust
use vips::*;

fn main() -> Result<()> {
    // Initialize once per process. The boolean enables leak diagnostics.
    let _instance = VipsInstance::new("app_example", true)?;

    // Create a thumbnail with forced width and height (preferred for files)
    let thumb = VipsImage::thumbnail_file("./examples/images/kodim01.png", 320, 240, VipsSize::VIPS_SIZE_FORCE)?;

    // Save the result
    thumb.write_to_file("kodim01_320x240.jpg")?;
    Ok(())
}
```

## 0.3.0 development version

Feature `v8_19` enables the APIs validated against **libvips 8.19.0-rc1**.
It includes QOI file/buffer IO, CICP/scRGB conversion, luminance percentiles,
animation loop counts, borrowed blob metadata, and point-of-interest crops and
thumbnails. The feature includes `v8_18` and requires native libvips 8.19+.
Default and `v8_18` builds are also tested with native 8.18.7.

This Git version pins an unpublished `vips-sys` 0.4.0 commit. Cargo fetches that
revision automatically. Publishing later requires publishing `vips-sys` first
and switching the dependency to the registry. The 0.2 examples below describe
the currently published package. See the [review](docs/review-2026-11-07.md)
for validation, performance measurements and the RC's QOI thumbnail limitation.

## 0.2.0 additions and migration

Version 0.2.0 uses the published `vips-sys` 0.3.0 dependency;
no sibling checkout is required.

- `write_to_memory_owned` / `write_to_buffer_owned` return `VipsOwnedBuffer`,
  which owns a GLib allocation and exposes immutable bytes without the extra
  full-buffer copy into a Rust `Vec`. Existing Vec APIs remain available.
- `from_file_with_access` / `from_buffer_with_access` accept a `VipsAccess` hint.
  Prefer sequential access for a one-pass pipeline and file/buffer thumbnail
  constructors for shrink-on-load.
- `--features v8_18` adds `tile_size`, `gainmap`, camera RAW file/buffer loaders,
  UltraHDR file/buffer IO and scRGB conversion, Oklab/Oklch conversion helpers,
  and `interpretation_bands`. Requires libvips 8.18+; RAW and HDR codecs remain
  optional build-time dependencies of the system library.
- `VipsRegion` now has a buffer lifetime; lazy multi-image operations retain
  every input buffer's lifetime. Image handles may drop before their results,
  but borrowed pixel buffers must remain alive.
- Drawing detaches borrowed/shared storage into writable memory on first use.
  Borrowed memory must be correctly aligned for its sample format.
- `VipsInterpolateMethod::call` is unsafe because output size/alignment and a
  prepared interpolation window are native caller obligations.
- `VipsInstance` uses the same initialization state as `init` and no longer
  shuts the process-wide library down when its handle is dropped.

```toml
vips = { version = "0.2", features = ["v8_18"] }
```

Run `cargo run --release --example buffer_bench` for an alternating ABBA
comparison of copied and owned raw exports. See the [review and roadmap](docs/review-2026-10-22.md).

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
- Native image references are retained by derived pipelines; creator handles may drop earlier. Every borrowed input buffer must still outlive the result.

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
- For already-decoded large images, benchmark `resize_reduce` against `resize` before choosing the extra box pre-shrink pass; it can be slower on small inputs.
- Tune `vips::set_concurrency`, `set_max_operations`, and `set_max_mem_bytes` for your workload.
- libvips pipelines are lazy: operations enqueue work until the image is written or materialized.

## Notes

- Initialization: `init` and `VipsInstance` share process-wide state. Native objects use RAII; global state remains available until process exit.
- Side effects: most operations return new images; drawing operations modify `self`.
- If a higher-level wrapper is missing, you can still access lower-level bindings in `vips-sys` or use unsafe `vips::vips_call(...)`
  to invoke libvips operations directly.

## Local development

This development version resolves the pinned `vips-sys` Git revision. To develop
both crates together, add this override at the workspace or consumer root.

```toml
[patch."https://github.com/houseme/vips-sys"]
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
