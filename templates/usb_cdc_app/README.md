# @NAME@

Created by `rust-box new usb_cdc_app @NAME@` (32_17_embedded_rust); self-contained: own `rust-toolchain.toml`,
`Cargo.lock`, `.cargo/config.toml` where a target is set. Use a snake_case name.

USB CDC-ACM serial device on the Pico (embassy-usb): upper-case echo on /dev/ttyACM*. Uses the test VID/PID
0xc0de:0xcafe: use your own for anything you ship. No runner.

```sh
rust-box run bash -c 'cd @NAME@ && cargo build --release'
rust-box plan pico <elf>
```
