# @NAME@

Created by `rust-box new riscv_virt_app @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

Bare-metal RISC-V (riscv-rt, rv32imac: also the Pico 2's Hazard3 cores) on QEMU `virt`: 16550 UART, machine CSRs,
exit through the SiFive test device.

```sh
rust-box run bash -c 'cd @NAME@ && cargo run --release'   # pinned qemu-system-riscv32
```
