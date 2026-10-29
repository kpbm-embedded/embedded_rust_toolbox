//! memory.x (with the .boot2 section) on the linker path + cortex-m-rt and defmt scripts, for the
//! test binaries (this package has no bin target).
use std::{env, fs, path::PathBuf};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("memory.x"), include_bytes!("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rustc-link-arg-tests=--nmagic");
    println!("cargo:rustc-link-arg-tests=-Tlink.x");
    println!("cargo:rustc-link-arg-tests=-Tdefmt.x");
}
