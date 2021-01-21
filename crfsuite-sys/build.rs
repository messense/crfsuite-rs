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
    let mut cfg = cmake::Config::new("");
    cfg.register_dep("cqdb").register_dep("lbfgs");
    if cfg!(target_os = "macos") {
        let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
        if target_arch == "x86_64" {
            cfg.define("CMAKE_OSX_ARCHITECTURES", "x86_64");
        }
    }
    let dst = cfg.build();
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=cqdb");
    println!("cargo:rustc-link-lib=static=lbfgs");
    println!("cargo:rustc-link-lib=static=crfsuite");
    println!("cargo:root={}", dst.to_str().unwrap());
    println!("cargo:include={}/include", dst.to_str().unwrap());
}

fn main() {
    fail_on_empty_directory("crfsuite");
    build_crfsuite();
}
