use compiletest_rs as compiletest;
use std::{env, fs, path::PathBuf, process::Command};

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

fn native_lib_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(out) = Command::new("pkg-config")
        .args(["--variable=libdir", "vips"])
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                let p = PathBuf::from(s);
                if p.exists() {
                    dirs.push(p);
                }
            }
        }
    }

    for cand in ["/opt/homebrew/lib", "/usr/local/lib", "/usr/lib"] {
        let p = PathBuf::from(cand);
        if p.exists() {
            dirs.push(p);
        }
    }
    dirs
}

fn run_mode(mode: compiletest::common::Mode, subdir: &str) {
    let mut config = compiletest::Config {
        mode,
        src_base: PathBuf::from(format!("tests/{}", subdir)),
        ..Default::default()
    };

    let deps = deps_dir();
    let lib = find_lib("vips");

    let mut flags = vec![
        format!("-L dependency={}", deps.display()),
        format!("--extern vips={}", lib.display()),
    ];
    for dir in native_lib_dirs() {
        flags.push(format!("-L native={}", dir.display()));
    }
    config.target_rustcflags = Some(flags.join(" "));
    config.edition = Some("2021".to_string());

    compiletest::run_tests(&config);
}

#[test]
fn compile_fail_tests() {
    run_mode(compiletest::common::Mode::CompileFail, "compile-fail");
}

#[test]
fn run_pass_tests() {
    run_mode(compiletest::common::Mode::RunPass, "run-pass");
}
