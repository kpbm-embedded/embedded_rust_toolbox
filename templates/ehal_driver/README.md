# @NAME@

Created by `rust-box new ehal_driver @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

Platform-independent I2C sensor driver (TMP117) on the embedded-hal 1.0 traits, unit-tested on the PC with
embedded-hal-mock (every bus transaction is checked). Rename the registers for your chip.

```sh
rust-box run bash -c 'cd @NAME@ && cargo test --target x86_64-unknown-linux-gnu'
rust-box run bash -c 'cd @NAME@ && cargo build --release --target thumbv7em-none-eabihf --features defmt'
```
