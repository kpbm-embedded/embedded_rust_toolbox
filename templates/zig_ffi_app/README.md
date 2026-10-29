# @NAME@

Created by `rust-box new zig_ffi_app @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

Rust firmware + a Zig static library (`zig/dsp.zig`), calls in both directions, on QEMU lm3s6965evb. Zig comes
from 32_24_microzig (not installed by rust-box):

```sh
microzig-box zig build-lib -target thumb-freestanding-eabi -mcpu cortex_m3 -O ReleaseSmall -fno-unwind-tables \
  -femit-bin=$PWD/@NAME@/zig/libzigdsp.a $PWD/@NAME@/zig/dsp.zig
rust-box run bash -c 'cd @NAME@ && ZIG_LIB_DIR=$PWD/zig cargo run --release'
```
