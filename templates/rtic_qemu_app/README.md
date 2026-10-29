# @NAME@

Created by `rust-box new rtic_qemu_app @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

RTIC v2 app on QEMU lm3s6965evb: hardware task (pended from software), two priorities, shared resources with
locks, an rtic-sync channel and the SysTick monotonic.

```sh
rust-box run bash -c 'cd @NAME@ && cargo run --release'
```
For a board: replace `lm3s6965` with the chip's PAC/HAL and the dispatcher interrupts.
