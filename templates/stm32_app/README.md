# @NAME@

Created by `rust-box new stm32_app @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

One Embassy firmware for the WeAct STM32 boards, chip by Cargo feature (g431, h562, h743) and target
(H5 = Cortex-M33). flip-link linker, defmt over RTT. No runner (it would flash).

```sh
rust-box run bash -c 'cd @NAME@ && cargo build --release --features g431 --target thumbv7em-none-eabihf'
rust-box plan weact_g431 $CARGO_TARGET_DIR/thumbv7em-none-eabihf/release/@NAME@   # PRINTS probe-rs / DFU commands
```
