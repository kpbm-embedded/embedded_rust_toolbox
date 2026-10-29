# @NAME@

Created by `rust-box new async_driver @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

Async SPI NOR flash driver (W25Qxx) on embedded-hal-async (`SpiDevice`, `DelayNs`), for Embassy or RTIC tasks;
host tests with embedded-hal-mock's async SPI mock and `embassy_futures::block_on`.

```sh
rust-box run bash -c 'cd @NAME@ && cargo test --target x86_64-unknown-linux-gnu'
```
