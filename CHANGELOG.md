# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html) as implemented by Cargo.

---

## [Unreleased]

### Added

- Tag-driven `Release` workflow (`release.yml`): version/CHANGELOG gates, quality checks, crates.io publish, GitHub Release notes.
- CI/Release verify on Linux, macOS, and Windows (Windows typecheck/clippy without link tests).
- Local development now depends on sibling `vips-sys` via `path = "../vips-sys"` (still versioned for crates.io).
- Bumped `vips-sys` dependency to `0.2.0`.
- `VipsImage::resize_reduce` — box pre-shrink (`vips_shrink`) + resize for large sources.
- `VipsImage::shrink_box` — integer box shrink pre-pass.
- `VipsImage::write_to_memory` and `VipsImage::write_jpeg` as public APIs.
- `VipsImage::bands` accessor.
- Documentation overhaul in `README.md` (EN) and `README_CN.md` (ZH).
- Clear guidance on owned vs borrowed constructors and lifetime pitfalls.
- Notes on initialization via `VipsInstance` and `OnceLock`-based global init/auto-shutdown.

### Changed

- CI `Build` workflow no longer publishes on tags; publishing is owned by `Release`.
- `from_file` / `write_to_file` accept `impl AsRef<Path>` and convert once to `CString` (no intermediate `Vec<u8>` / UTF-8 round-trip).
- `version_string()` returns a cached `&'static str` instead of allocating a `String` on every call.
- Mosaic defaults aligned with libvips (`hwindow=5`, `harea=2`, `bandno=-1`) instead of the previous incorrect `1/1/0`.
- Hot accessors (`width`/`height`/`size`/`bands`/`concurrency`) marked `#[inline]`.
- Dropped the `rust-version` field from `Cargo.toml`.
- Consistent lifetime annotations:
    - Return `VipsImage<'a>` for constructors tied to input slices.
    - Return `VipsImage<'_>` for methods borrowing `&self`.
- Replaced manual NUL-terminated strings with `c"..."` literals.
- Removed unused lifetime parameter from `impl Drop for VipsInterpolate`.
- Locally allowed `clippy::too_many_arguments` for functions that require many parameters.
- Tests: initialize `compiletest_rs::Config` via struct literal instead of field reassignments.

### Fixed

- `VipsBuffer::thumbnail` no longer pre-allocates a `VipsImage` that was immediately overwritten (memory leak).
- `VipsImage::thumbnail` now checks the libvips return code, not only a null pointer.
- `VipsImage::from_memory` returns early without installing the postclose free hook when creation fails.
- `VipsImage::Drop` / `image_postclose` null-pointer guards.
- Addressed "hiding/elided lifetime is confusing" warnings.
- Prevented "does not live long enough" by keeping owners and derived images in the same scope in examples.

## [0.1.0-alpha.3]

- Initial pre-release with core image IO, resize/thumbnail, drawing, and stitching helpers.
