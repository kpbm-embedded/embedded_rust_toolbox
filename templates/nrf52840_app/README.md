# @NAME@

Created by `rust-box new nrf52840_app @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

Embassy app for the nice!nano / Pro Micro nRF52840, linked at 0x26000 so the SoftDevice and the Adafruit UF2
bootloader stay; async TEMP sensor + LED. No runner.

```sh
rust-box run bash -c 'cd @NAME@ && cargo build --release'
rust-box uf2 <elf> -o @NAME@.uf2 --family nrf52840   # double-tap RST, copy to the NICENANO drive
```
