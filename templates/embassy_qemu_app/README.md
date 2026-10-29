# @NAME@

Created by `rust-box new embassy_qemu_app @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

Embassy on QEMU mps2-an385 without a HAL: an own embassy-time driver on SysTick, the thread executor, tasks,
Channel, Ticker, Timer and select(). Useful to test async logic in CI; on a board the HAL's time driver replaces
`SysTickDriver`.

```sh
rust-box run bash -c 'cd @NAME@ && cargo run --release'
```
