use std::env;
use std::fs;

fn fail_on_empty_directory(name: &str) {
    if fs::read_dir(name).unwrap().count() == 0 {
        println!(
            "The `{}` directory is empty, did you forget to pull the submodules?",
            name
        );
        println!("Try `git submodule update --init --recursive`");
        panic!();
    }
}

fn build_crfsuite() {
    let dst = cmake::Config::new("").build_target("crfsuite").build();
    if cfg!(target_os = "windows") {
        let profile = match &*env::var("PROFILE").unwrap_or_else(|_| "debug".to_owned()) {
            "bench" | "release" => "Release",
            _ => "Debug",
        };
        println!(
            "cargo:rustc-link-search=native={}/build/{}",
            dst.display(),
            profile
        );
    } else {
        println!("cargo:rustc-link-search=native={}/build", dst.display());
    }
    println!("cargo:rustc-link-lib=static=cqdb");
    println!("cargo:rustc-link-lib=static=lbfgs");
    println!("cargo:rustc-link-lib=static=crfsuite");
}

fn main() {
    fail_on_empty_directory("liblbfgs");
    fail_on_empty_directory("crfsuite");
    build_crfsuite();
}
