# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html) as implemented by Cargo.

---

## [Unreleased]

### Added

- Documentation overhaul in `README.md` (EN) and `README_CN.md` (ZH).
- Clear guidance on owned vs borrowed constructors and lifetime pitfalls.
- Notes on initialization via `VipsInstance` and `OnceLock`-based global init/auto-shutdown.

### Changed

- Consistent lifetime annotations:
    - Return `VipsImage<'a>` for constructors tied to input slices.
    - Return `VipsImage<'_>` for methods borrowing `&self`.
- Replaced manual NUL-terminated strings with `c"..."` literals.
- Removed unused lifetime parameter from `impl Drop for VipsInterpolate`.
- Locally allowed `clippy::too_many_arguments` for functions that require many parameters.
- Tests: initialize `compiletest_rs::Config` via struct literal instead of field reassignments.

### Fixed

- Addressed "hiding/elided lifetime is confusing" warnings.
- Prevented "does not live long enough" by keeping owners and derived images in the same scope in examples.

## [0.1.0-alpha.3]

- Initial pre-release with core image IO, resize/thumbnail, drawing, and stitching helpers.
