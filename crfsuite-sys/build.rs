extern crate cc;
extern crate bindgen;
extern crate cmake;

use std::env;
use std::fs;
use std::path::PathBuf;

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
    let dst = cmake::Config::new("")
        .build_target("crfsuite")
        .build();
    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=cqdb");
    println!("cargo:rustc-link-lib=static=lbfgs");
    println!("cargo:rustc-link-lib=static=crfsuite");

    let bindings = bindgen::Builder::default()
        .header("crfsuite/include/crfsuite.h")
        .blacklist_type("max_align_t") // https://github.com/rust-lang-nursery/rust-bindgen/issues/550
        .ctypes_prefix("libc")
        .generate()
        .expect("unable to generate crfsuite bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("unable to write crfsuite bindings");
}

fn main() {
    fail_on_empty_directory("liblbfgs");
    fail_on_empty_directory("crfsuite");
    build_crfsuite();
}
