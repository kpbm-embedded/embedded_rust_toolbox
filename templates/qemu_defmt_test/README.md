# @NAME@

Created by `rust-box new qemu_defmt_test @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

On-target unit tests with defmt-test, run in QEMU lm3s6965evb: `cargo test` builds the test ELF and the runner
(`qemu-defmt.sh`) runs it in QEMU and decodes the defmt output with defmt-print. The library is `src/lib.rs`
(imported in the tests as `firmware_lib`).

```sh
rust-box run bash -c 'cd @NAME@ && cargo test --release'
```
On a board: switch the target/memory.x and use `probe-rs run` as the runner (defmt-rtt instead of defmt-semihosting).
