# @NAME@

Embassy app for the Raspberry Pi Pico (RP2040): LED + status task, defmt over RTT, flip-link. Created by `rust-box new embassy_app @NAME@` (32_17_embedded_rust); self-contained: own
`rust-toolchain.toml`, `.cargo/config.toml` (target + flip-link linker), `memory.x`, `build.rs`, `Cargo.lock`.

```sh
rust-box cargo build --release --manifest-path @NAME@/Cargo.toml
rust-box plan pico $CARGO_TARGET_DIR/<target>/release/@NAME@    # PRINTS the probe-rs / UF2 commands
```
`cargo run` is not wired to a runner on purpose (it would flash the board); `.cargo/config.toml` has
the commented runner line. ELF location: `$CARGO_TARGET_DIR` (the ID's `$OUT/build/cargo-target`).
