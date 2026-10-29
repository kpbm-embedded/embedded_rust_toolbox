//! One package, two linker layouts: each binary gets its own memory.x through a per-binary search path
//! (cortex-m-rt's link.x does `INCLUDE memory.x`).
fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    for bin in ["boot", "app"] {
        println!("cargo:rustc-link-arg-bin={bin}=-L{dir}/layout/{bin}");
        println!("cargo:rerun-if-changed=layout/{bin}/memory.x");
    }
}
