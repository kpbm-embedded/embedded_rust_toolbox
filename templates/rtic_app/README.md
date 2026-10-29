# @NAME@

RTIC v2 app for the BlackPill STM32F411CE (stm32f4xx-hal, SysTick monotonic, 96 MHz). Created by `rust-box new rtic_app @NAME@` (32_17_embedded_rust); self-contained: own
`rust-toolchain.toml`, `.cargo/config.toml` (target + flip-link linker), `memory.x`, `build.rs`, `Cargo.lock`.

```sh
rust-box cargo build --release --manifest-path @NAME@/Cargo.toml
rust-box plan blackpill_f411 $CARGO_TARGET_DIR/<target>/release/@NAME@    # PRINTS the probe-rs / UF2 commands
```
`cargo run` is not wired to a runner on purpose (it would flash the board); `.cargo/config.toml` has
the commented runner line. ELF location: `$CARGO_TARGET_DIR` (the ID's `$OUT/build/cargo-target`).
