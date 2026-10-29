# @NAME@

Created by `rust-box new qemu_app @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

Bare-metal Cortex-M3 app that runs on QEMU lm3s6965evb (no board): cortex-m-rt, semihosting output, exit code.

```sh
rust-box run bash -c 'cd @NAME@ && cargo run --release'   # builds, then runs it in the pinned qemu-system-arm
```
Change `memory.x` and the target in `.cargo/config.toml` for a real chip; the runner line is QEMU, not a flasher.
