//! memory.x on the linker path + the Zig static library (libzigdsp.a). example.sh builds it with
//! 32_24_microzig's Zig and passes the folder in ZIG_LIB_DIR.
use std::{env, fs, path::PathBuf};

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(out.join("memory.x"), include_bytes!("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=memory.x");
    let zig = env::var("ZIG_LIB_DIR").expect("ZIG_LIB_DIR: folder with libzigdsp.a (build through example.sh)");
    println!("cargo:rustc-link-search=native={zig}");
    println!("cargo:rustc-link-lib=static=zigdsp");
    println!("cargo:rerun-if-env-changed=ZIG_LIB_DIR");
    println!("cargo:rerun-if-changed={zig}/libzigdsp.a");
}
