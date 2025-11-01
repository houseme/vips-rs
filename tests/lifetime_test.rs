use compiletest_rs as compiletest;
use std::{env, fs, path::PathBuf};

fn deps_dir() -> PathBuf {
    let mut dir = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()));
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    dir.push(profile);
    dir.push("deps");
    dir
}

fn find_lib(name: &str) -> PathBuf {
    let dir = deps_dir();
    let prefix = format!("lib{}-", name);
    for entry in fs::read_dir(&dir).expect("read deps dir failed") {
        let p = entry.unwrap().path();
        let fname = p.file_name().unwrap().to_string_lossy();
        if fname.starts_with(&prefix)
            && (fname.ends_with(".rlib") || fname.ends_with(".dylib") || fname.ends_with(".so"))
        {
            return p;
        }
    }
    panic!("cannot find compiled crate {}", name);
}

fn run_mode(mode: compiletest::common::Mode, subdir: &str) {
    let mut config = compiletest::Config::default();
    config.mode = mode;
    config.src_base = PathBuf::from(format!("tests/{}", subdir));
    let deps = deps_dir();
    let lib = find_lib("vips");
    config.target_rustcflags = Some(format!(
        "-L dependency={} --extern vips={}",
        deps.display(),
        lib.display()
    ));
    config.edition = Some("2018".to_string());
    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    run_mode(compiletest::common::Mode::CompileFail, "compile-fail");
}
