# @NAME@

Created by `rust-box new protocol_lib @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

no_std, no-alloc wire protocol: serde messages, postcard encoding, COBS frames, heapless Vec/String and an SPSC
queue for ISR -> main. (thumbv6m/riscv32imc lack atomic CAS: enable heapless's `portable-atomic` there.)

```sh
rust-box run bash -c 'cd @NAME@ && cargo test --target x86_64-unknown-linux-gnu'
```
