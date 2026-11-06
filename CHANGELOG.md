# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html) as implemented by Cargo.

---

## [Unreleased]

## [0.3.0] - 2026-11-07

Prepared in Git for coordinated testing. Package publication is deferred.

### Added

- Opt-in `v8_19` feature for libvips 8.19.0-rc1 and later, including `v8_18`.
- QOI file/buffer loading and saving, with native-owned encoded output.
- `cicp2scrgb`, configurable `scrgb2cicp`, and public CICP enum families.
- `percent_lum` with finite percentile/histogram-bound validation.
- `loop_count` and zero-copy `blob_value`, whose slice borrows the image.
- `smartcrop_at`, `thumbnail_file_at` and `thumbnail_buffer_at` for an explicit
  point of interest. Thumbnail constructors retain native shrink-on-load.
- Native image/codec tests, QOI-input and blob-image lifetime regressions,
  pinned 8.19 CI and a functionality/performance review.

### Changed

- Use a fixed Git revision of `vips-sys` 0.4.0 while publication is deferred.
  Default and `v8_18` functionality is verified with native 8.18.7.
- CI selects native-compatible features and runs all features in the pinned
  8.19 job. Future package publication requires the registry dependency switch.

### Known upstream limitations

- QOI thumbnails in 8.19.0-rc1 can fail with `bad QOI magic`; a native C
  reproducer confirms the source-loader issue. QOI load/save tests pass.
- HDR reference-white changes merged after RC1 remain part of the next upstream
  validation cycle. See `docs/review-2026-11-07.md`.

## [0.2.0] - 2026-10-22

### Safety and compatibility

- **Breaking:** `VipsRegion` retains the backing buffer lifetime. Lazy multi-image
  operations constrain every input buffer, including secondary images and masks.
- **Breaking:** `VipsInterpolateMethod::call` is unsafe because prepared-region,
  output-size/alignment, and method/interpolator invariants cannot be inferred.
- `VipsInstance` and `init` share `vips-sys`'s initialization lock and success state.
  Failures do not consume the singleton slot. Dropping a `VipsInstance` leaves the
  process-wide library alive for outstanding images and workers.
- Drawing detaches borrowed/shared memory before its first modification, preserving
  immutable Rust inputs. Memory constructors validate size, dimensions and alignment.
- Invalid `linear` arrays and overflowing/invalid `median` sizes return errors.

### Performance

- `VipsOwnedBuffer`, `write_to_memory_owned` and `write_to_buffer_owned` avoid the
  additional full-output copy and allocation into a Rust `Vec`; existing Vec APIs
  delegate to the same RAII owner so all returned native allocations are freed.
- `from_memory` retains the original Vec allocation, including spare capacity,
  rather than shrinking it through `into_boxed_slice`.
- File/buffer loading supports explicit sequential/random `VipsAccess` hints.
- Add a release-mode ABBA raw-export benchmark. Existing `perf_smoke` timings now
  force pixel evaluation instead of timing lazy pipeline construction alone.

### Added in feature `v8_18`

- `tile_size`, `gainmap`, and `interpretation_bands`.
- `dcrawload_file` / `dcrawload_buffer` for camera RAW.
- `uhdrload_file` / `uhdrload_buffer`, `write_uhdr` / `write_uhdr_buffer`, `uhdr2scrgb`.
- `xyz2oklab`, `oklab2xyz`, `oklab2oklch`, `oklch2oklab`.
- These wrap stable libvips 8.18 APIs; RAW/HDR support depends on optional native codecs.

### Validation and delivery

- Add borrowed-secondary/conditional/region lifetime regressions, initialization
  contention and retry checks, native output ownership/drawing/shape tests, and
  8.18 colour/metadata checks.
- Resolve published `vips-sys` 0.3.0 from crates.io for local builds and CI,
  including its Windows/vcpkg discovery fix and native build improvements.
  The lockfile records the registry source and package checksum; CI no longer
  needs a sibling checkout. Local development can still use `[patch.crates-io]`.
- CI exercises 8.18 APIs on macOS and default APIs with Linux distro libraries;
  CodeQL also tracks the default `main` branch.

### Added

- `VipsImage::thumbnail_file` — `vips_thumbnail` constructor that combines load + resize (shrink-on-load). Preferred for file sources over `from_file` + `thumbnail`.
- `VipsImage::thumbnail_buffer` — `vips_thumbnail_buffer` constructor for encoded image bytes. Preferred over `from_buffer` + `thumbnail`.
- Geometry helpers: `crop`/`extract_area`, `embed`, `gravity`, `insert`, `flip`/`fliphor`/`flipver`, `rot`/`rot90`/`rot180`/`rot270`, `rotate`, `autorot`, `zoom`, `extract_band`, `bandjoin2`, `bandjoin_const`, `copy`, `copy_memory`.
- Arithmetic/color/filter: `add`/`subtract`/`multiply`/`divide`, `linear`/`linear1`, `invert`, `cast`, `colourspace`, `gaussblur`, `sharpen`.
- Statistics: `avg`, `min_value`, `max_value`, `getpoint`.
- IO: `write_to_buffer(suffix)`; free functions `find_load` / `find_save`.
- Math convenience helpers: `sin`/`cos`/`tan`, `ln`/`log10`/`exp`/`exp10`, `floor`/`ceil`/`rint`, `abs`/`sign`/`clamp`, `pow_const`.
- Relational masks: `less`/`lesseq`/`more`/`moreeq`/`equal_const`/`notequal_const`.
- Boolean/shift: `and_const`/`or_const`/`eor_const`, `lshift_const`/`rshift_const`, `bandbool`/`bandand`/`bandor`/`bando`, `bandmean`, `bandsplit`.
- Filter/morph/edge: `conv`, `erode`/`dilate`, `median`/`rank`, `sobel`, `canny`.
- Logic/geometry extras: `ifthenelse`, `smartcrop`, `falsecolour`, `premultiply`/`unpremultiply`, `sequential`.
- Stats: `deviate`, `percent`, `find_trim`.
- Create: free functions `black`, `xyz`.
- Additional geometry: `wrap`, `replicate`, `grid`, `copy_file`.
- Color/hist/edge: `gamma`, `hist_find`/`hist_local`/`hist_equal`/`hist_entropy`, `maplut`, `bandfold`/`bandunfold`, `labelregions`.
- Create: `grey`, `sines`, `zone`, `perlin`, `gaussnoise`.
- `examples/ops_smoke` smoke coverage for the new APIs.

### Changed

- Point `package.documentation` and README docs links at <https://docs.rs/vips/> (drop GitHub Pages URLs).
- Document that `VipsImage::thumbnail` (`vips_thumbnail_image`) is only for already-decoded pixels; file/buffer sources should use `thumbnail_file` / `thumbnail_buffer` (issue [#9](https://github.com/houseme/vips-rs/issues/9)).
- `VipsBuffer::thumbnail` now takes an explicit `VipsSize` instead of hardcoding `VIPS_SIZE_FORCE`, is implemented on `[u8]` (so `&[u8]`/`Vec<u8>` work with correct lifetimes), and delegates to `VipsImage::thumbnail_buffer`.

## [0.1.0] - 2026-09-13

First stable crates.io release. Resolves `vips-sys` 0.2.0 from crates.io.
Edition 2024 (Rust ≥ 1.85).

### Added

- Tag-driven `Release` workflow: version/CHANGELOG gates, quality checks, crates.io publish, GitHub Release notes.
- `Release` workflow also supports manual `workflow_dispatch` with a `tag` input.
- CI/Release verify on Linux, macOS, and Windows (Windows typecheck/clippy without link tests).
- `VipsImage::resize_reduce` — box pre-shrink (`vips_shrink`) + resize for large sources.
- `VipsImage::shrink_box` — integer box shrink pre-pass.
- `VipsImage::write_to_memory` and `VipsImage::write_jpeg` as public APIs.
- `VipsImage::bands` accessor.
- Owned vs borrowed constructor guidance, API table, performance tips, and Docker verify snippets in both READMEs.
- `examples/perf_smoke` for reduce/path/memory smoke checks.

### Changed

- Encapsulate raw libvips pointers: `VipsImage`/`VipsRegion`/`VipsInterpolate` store `NonNull` privately; expose only `as_ptr()` for FFI bridges.
- Internal `ffi` module centralizes null-checks, `unref`, and status-code mapping.
- Hide `image_postclose`; stop re-exporting `vips_call` under a safe-looking alias.
- `VipsRegion::new` returns `Option` (allocation can fail).
- Document the crate safety model in `lib.rs`.
- `from_file` / `write_to_file` accept `impl AsRef<Path>` and convert once to `CString`.
- `version_string()` returns a cached `&'static str` instead of allocating on every call.
- Mosaic defaults aligned with libvips (`hwindow=5`, `harea=2`, `bandno=-1`).
- Hot accessors (`width`/`height`/`size`/`bands`/`concurrency`) marked `#[inline]`.
- Operation results use `VipsImage<'a>` so intermediate images can drop safely.
- Cargo `resolver = "3"`; `vips-sys` dependency is crates.io-only (optional sibling `path` documented for local work).
- Dropped the `rust-version` field; edition remains `2024`.
- Replaced manual NUL-terminated strings with `c"..."` literals.

### Fixed

- `VipsBuffer::thumbnail` no longer pre-allocates a `VipsImage` that was immediately overwritten (memory leak).
- `VipsImage::thumbnail` now checks the libvips return code, not only a null pointer.
- `VipsImage::from_memory` returns early without installing the postclose free hook when creation fails.
- `VipsImage::Drop` / `image_postclose` null-pointer guards.
- Serialize first-time `init` under a process lock so concurrent `vips_init` cannot SIGSEGV.
- Serialize unit tests that touch libvips global state.
- CI `RustFmt` job no longer inherits the nested `working-directory` (root checkout).
- Lifetime UI tests use `trybuild` with rustc-1.98-formatted `.stderr` snapshots (avoids compiletest E0463 and version-skew mismatches).

## [0.1.0-alpha.3]

- Pre-release with core image IO, resize/thumbnail, drawing, and stitching helpers.
