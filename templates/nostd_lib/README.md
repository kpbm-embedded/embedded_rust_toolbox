# @NAME@

no_std library with host unit tests and optional defmt::Format. Created by `rust-box new nostd_lib @NAME@` (32_17_embedded_rust); self-contained.

```sh
rust-box cargo test --target x86_64-unknown-linux-gnu      # host unit tests (use your host triple)
rust-box cargo build --release --target thumbv6m-none-eabi --features defmt      # Pico
rust-box cargo build --release --target thumbv7em-none-eabihf --features defmt   # BlackPill
```
(`rust-box` = the launcher of 32_17_embedded_rust, or `ecargo` in native mode.) Build output goes to
`$CARGO_TARGET_DIR` (the ID's `$OUT/build/cargo-target`), not into this folder.
