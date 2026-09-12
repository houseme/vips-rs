# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html) as implemented by Cargo.

---

## [Unreleased]

### Changed

- Point `package.documentation` and README docs links at <https://docs.rs/vips/> (drop GitHub Pages URLs).

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
