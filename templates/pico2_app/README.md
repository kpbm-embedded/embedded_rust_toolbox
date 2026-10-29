# @NAME@

Created by `rust-box new pico2_app @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

Embassy app for the Raspberry Pi Pico 2 (RP2350, Arm cores): IMAGE_DEF block for the boot ROM, picotool binary
info, LED + PWM task. No runner (it would flash).

```sh
rust-box run bash -c 'cd @NAME@ && cargo build --release'
rust-box uf2 <elf> -o @NAME@.uf2 --family rp2350-arm   # then copy to the RP2350 drive (BOOTSEL)
```
