//! Lifetime UI tests via trybuild.
//!
//! Regenerate `.stderr` snapshots when rustc diagnostic formatting changes:
//! `TRYBUILD=overwrite cargo test --test lifetime_test`

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
