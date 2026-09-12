//! Lifetime compile tests via `trybuild` (replaces compiletest_rs).
//!
//! `compile-fail/` cases must fail to borrow-check.
//! `run-pass/` cases must compile.

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}

#[test]
fn run_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/*.rs");
}
