# @NAME@

Created by `rust-box new bootloader_app @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

Bootloader + application in one package with two linker layouts (`layout/boot/memory.x` at 0x0,
`layout/app/memory.x` at 0x8000, picked per binary in `build.rs`). The bootloader validates the app's vector table,
sets VTOR and jumps. Runs on QEMU lm3s6965evb:

```sh
rust-box run bash -c 'cd @NAME@ && cargo build --release && T=$CARGO_TARGET_DIR/thumbv7m-none-eabi/release &&
  llvm-objcopy -O binary $T/app $T/app.bin && qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic \
  -semihosting-config enable=on,target=native -kernel $T/boot -device loader,file=$T/app.bin,addr=0x8000'
```
