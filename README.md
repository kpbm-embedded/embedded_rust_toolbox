# 32_17_embedded_rust - Embedded Rust (Embassy, defmt, RTIC v2)

A pinned Rust toolchain for your boards (Raspberry Pi Pico and Pico 2, BlackPill F411, WeAct G431/H562/H743,
nice!nano nRF52840) plus the current embedded Rust stack:
Embassy (async executor + HALs), defmt (deferred-format logging) and RTIC v2 (interrupt-driven, priority-ceiling
concurrency), with probe-rs and flip-link. rustup's `RUSTUP_HOME` and `CARGO_HOME` live inside this folder:
`~/.rustup` and `~/.cargo` are never read or written (the host `~/.cargo/bin` is removed from `PATH` inside the
environment). The launcher is named `rust-box` so it never shadows `cargo`/`rustup`. No sudo, no system packages.

Status: **ready** (doctor 30 ok / 1 warn; examples 26/26 plus 26/26 offline rebuild; templates 17/17 built or run;
smoke 24/24; dry run 37/37; 2026-10-29). The warning is the probe udev rules (see "Limitations / needs you"). Nothing has
run on a real board: board examples are built and statically checked, the qemu examples run in QEMU. Launcher: `./rust-box`.

## What's included

| Component | Version | Notes |
|---|---|---|
| Rust (rustc, cargo, rust-std, llvm-tools, clippy, rustfmt) | 1.99.1 stable, profile minimal | `rust-toolchain.toml` |
| Targets | `thumbv6m-none-eabi` (Pico RP2040), `thumbv7em-none-eabihf` (BlackPill F411, WeAct G431/H743, nRF52840), `thumbv7m-none-eabi` (QEMU lm3s6965evb / mps2-an385), `thumbv8m.main-none-eabihf` (Pico 2, WeAct H562), `riscv32imac-unknown-none-elf` (Pico 2 Hazard3, QEMU virt), `riscv32imc-unknown-none-elf` (ESP32-C3, libraries only) | `rust-toolchain.toml`, installed by `bootstrap` |
| xPack QEMU (qemu-system-arm, qemu-system-riscv32) | 9.2.4-1 | sha256 pinned in `config/pins.tsv`; runs the qemu examples and templates |
| rustup-init | 1.29.2 | sha256 from `rustup-init.sha256` |
| probe-rs-tools (probe-rs, cargo-flash, cargo-embed) | 0.32.0 | sha256 from the release |
| flip-link | 0.1.12 | sha256.sum of the release |
| defmt-print | 1.2.0 | `cargo install --locked`, `config/cargo-tools.tsv` |
| Embassy | executor 0.10.0, time 0.5.1, rp 0.10.0, stm32 0.6.0, sync 0.8.0 | crates.io |
| defmt | defmt 1.2.0, defmt-rtt 1.3.1, panic-probe 1.0.0, defmt-test 0.5.1 | crates.io |
| RTIC | rtic 2.4.0, rtic-monotonics 3.0.0, rtic-sync 1.6.0 | crates.io |
| HALs for RTIC | stm32f4xx-hal 0.23.0, rp2040-hal 0.12.0, rp2040-boot2 0.3.0 | crates.io |
| More HALs / crates (2026-10-29) | embassy-nrf 0.11.0, embassy-usb 0.6.0, embassy-time-driver 0.2.2, cortex-m-semihosting 0.6.0, defmt-semihosting 0.4.0, lm3s6965 0.2.0, riscv 0.16.1, riscv-rt 0.18.0, embedded-hal 1.0.0, embedded-hal-async 1.0.0, embedded-hal-mock 0.11.1, heapless 0.9.3, postcard 1.1.3 | crates.io, `Cargo.lock` per example |
| pixi env | python 3.12, conda-forge `c-compiler` (gcc 14.4, host linker), make | `pixi.lock` |
| pixi | 0.81.0 | sha256 pinned in the launcher |

Every example has its own `Cargo.lock`; crates come from crates.io into `cache/cargo/registry`, so after the first build
everything rebuilds offline (`offline-check`). Optional cargo tools (`cargo-generate`, `elf2uf2-rs`) are installed only on
request: `./rust-box cargo-tool cargo-generate`. Why these three frameworks (stars, releases, sources):
[docs/why_these_picks.md](docs/why_these_picks.md).

## Quick start

```sh
cd "$BOX_EMBEDDED_RUST"                      # or: cd $BOX/30_39_tools/32_embedded/32_17_embedded_rust
./rust-box install && ./rust-box doctor      # toolchain + tools (already installed here), then health checks
./rust-box examples test                     # all 26 examples: qemu runs, static ELF checks, host tests, UF2
./rust-box new embassy_app my_fw --dir ~/src # a Pico/Embassy project from a template
./rust-box plan pico fw.elf                  # PRINTS probe-rs / UF2 / DFU commands; never touches a device
```

With the Box shell set up once (`box shell install --apply`), the `rust-box` shim works from any folder,
and `box test|doctor|status embedded_rust` run this launcher (`box clean embedded_rust` is a dry run until `--apply`).
Standalone (no Box, or `BOX_INTEGRATION=0`), `./rust-box` keeps its output in this folder.

## Commands

Global option: `--dry-run` / `-n` prints what a command would do. Exit codes: 0 ok, 1 failure, 2 usage error (unknown command or option, or arguments given to `where`, `status`, `doctor`, `bootstrap` or `relocate`), 3 not applicable. `help` ends with this tool's own examples and variables (`tool examples`, `tool environment`).

| Command | What it does | Example |
|---|---|---|
| `install [--no-demos]` | bootstrap + rustup-init (sha256), toolchain 1.99.1 + targets, probe-rs, flip-link, defmt-print, example build | `./rust-box install` |
| `bootstrap` | pinned pixi + env + core downloads (sha-checked) | `./rust-box bootstrap` |
| `update [--check\|--env]` | `--check` (default): pins vs newest upstream; `--env`: re-solve the pixi env (rewrites `pixi.lock`) | `./rust-box update --check` |
| `clean [--all] [--yes]` | move build/log output (`--all`: + install/, .pixi, native/, downloads) to the Trash; dry run unless `--yes` | `./rust-box --dry-run clean --all` |
| `status`, `where`, `doctor` | versions and sizes, paths, health checks (exit 1 on failure) | `./rust-box doctor` |
| `shell [-- cmd ...]` | bash (or a command) with the tools on PATH | `./rust-box shell` |
| `run <cmd> [args]` | one command inside the environment | `./rust-box run rustc --version` |
| `cargo [args]` | the pinned cargo (`RUSTUP_HOME`/`CARGO_HOME` in this ID, target dir on idisk03) | `./rust-box cargo build --release` |
| `rustup [args]` | the in-ID rustup (toolchains under `install/rustup`) | `./rust-box rustup show` |
| `boards` | your boards: target triple, probe-rs chip, UF2 family (`config/boards.tsv`) | `./rust-box boards` |
| `plan <board> <fw.elf>` | PRINT flash/run commands (probe-rs, UF2 copy, DFU), runs nothing | `./rust-box plan pico fw.elf` |
| `uf2 <fw.elf> [-o F]` | ELF to UF2 for RP2040 BOOTSEL drag-and-drop (writes a file only) | `./rust-box uf2 fw.elf -o fw.uf2` |
| `probe-rs`, `flip-link`, `defmt-print [args]` | the pinned tools (pass-through: runs what you type) | `./rust-box defmt-print -e fw.elf` |
| `cargo-tool <name>` | `cargo install --locked` an optional tool from `config/cargo-tools.tsv` | `./rust-box cargo-tool cargo-generate` |
| `offline-check` | every example's `build` with `CARGO_NET_OFFLINE=true` (registry cache in `cache/cargo`) | `./rust-box offline-check` |
| `new <kind> <name> [--dir D] [--force]` | project from `templates/` (17 kinds, see below) | `./rust-box new rtic_app my_fw --dir ~/src` |
| `test [--all]` | `tests/test_smoke.sh`; `--all` adds the dry-run test and examples test | `./rust-box test` |
| `examples list \| run <ex> [args] \| build [ex...] \| test [ex...]` | list, run, build or test the examples | `./rust-box examples test` |
| `native [--print]` | write `native/{native.zsh,native.bash,bin/}` | `./rust-box native` |
| `relocate` | after moving the folder: re-create the pixi env, rewrite `native/` | `./rust-box relocate` |
| `pack [--with-cache] [--out F]` | tarball of this folder (minus envs/build) for another machine | `./rust-box pack --out /tmp/p.tgz` |
| `help`, `-h`, `--help` | the command list | `./rust-box help` |

`plan <board> <elf>` prints `probe-rs run/download/attach/gdb` (Debug Probe on the Pico, ST-LINK/DAPLink on the
BlackPill), the UF2 copy for the Pico and the `dfu-util` line for the STM32 ROM bootloader (dfu-util is not bundled;
see `32_15_flashers`).

## Examples and templates

`./rust-box examples list`, then `examples test` for all 26. Board examples: `test` = release build + static ELF checks
(vector table at the flash origin, `.defmt` present, flip-link put the initial stack pointer below `.data`, size) + UF2
where the board has a UF2 bootloader; `run` = build + the printed flash plan (never flashes). QEMU examples: `test` runs
the firmware in the pinned QEMU and checks its output and exit code. Host examples: `cargo test` on the PC + cross build.
No board example sets a cargo `runner` (that would make `cargo run` flash); the QEMU ones use QEMU as the runner.

| Example | Board | What it shows |
|---|---|---|
| `embassy_pico_blinky` | Pico | embassy-rp, LED GP25 + heartbeat task (Ticker), `spawner.spawn(unwrap!(task()))` (0.10 API) |
| `embassy_pico_uart` | Pico | BufferedUart UART0 GP0/GP1 + interrupts, reader task, Channel, upper-case echo |
| `embassy_blackpill_blinky` | BlackPill | embassy-stm32 `stm32f411ce`, LED PC13, KEY PA0 via async `ExtiInput` |
| `embassy_blackpill_tasks` | BlackPill | 96 MHz HSE/PLL, 3 tasks, Channel + Signal + `select` timeout, USART1 output |
| `defmt_blackpill_logging` | BlackPill | levels, `derive(Format)`, display hints, DWT timestamp; test proves `DEFMT_LOG` dropped `trace!` |
| `defmt_host_tests` | host + both | no_std lib (moving average, CRC-8, COBS): 6 host `cargo test`s, cross-built with `defmt::Format` |
| `defmt_test_pico` | Pico | 4 on-target `defmt-test` tests (test ELF built; run on the board with the printed plan) |
| `rtic_blackpill_blinky` | BlackPill | RTIC 2 + stm32f4xx-hal, SysTick monotonic, async task with `delay_until` |
| `rtic_blackpill_shared` | BlackPill | shared resources + `lock`, EXTI0 hardware task, 2 software priorities, rtic-sync channel |
| `rtic_pico_monotonic` | Pico | RTIC 2 thumbv6 backend + rp2040-hal, TIMER monotonic, shared counter between two priorities |
| `qemu_semihosting_hello` | QEMU lm3s6965evb | cortex-m-rt, semihosting print, CPUID, SysTick; exit code checked |
| `qemu_hardfault` | QEMU mps2-an385 | HardFault handler (exception frame, HFSR/CFSR; the fault PC disassembles to `udf`) + own `#[panic_handler]` (location, exit 1) |
| `qemu_defmt_semihosting` | QEMU lm3s6965evb | defmt frames over semihosting, decoded by defmt-print; `trace!` compiled out by `DEFMT_LOG` |
| `qemu_defmt_test` | QEMU lm3s6965evb | 5 defmt-test tests run on the target CPU: `cargo test` with a QEMU + defmt-print runner |
| `rtic_qemu_lm3s6965` | QEMU lm3s6965evb | RTIC 2: hardware task pended from software, 3 priorities, locks, rtic-sync channel, SysTick monotonic |
| `embassy_qemu_timers` | QEMU mps2-an385 | Embassy with an own SysTick embassy-time driver: tasks, Channel, Ticker, Timer, `select` timeout |
| `qemu_bootloader_jump` | QEMU lm3s6965evb | two linker layouts in one package (boot at 0x0, app at 0x8000); bootloader validates, sets VTOR, jumps |
| `riscv_qemu_virt` | QEMU riscv32 virt | riscv-rt (rv32imac as on the Pico 2 Hazard3): 16550 UART, misa/mhartid/mcycle, SiFive test exit |
| `ehal_driver_mock` | host + 3 targets | embedded-hal 1.0 I2C driver (TMP117) + ALERT pin, 7 host tests with embedded-hal-mock |
| `async_spi_flash_mock` | host + 3 targets | embedded-hal-async SPI NOR flash driver (W25Qxx, page split, erase timeout), 4 tests with the async mock |
| `heapless_postcard_frames` | host + 3 targets | serde + postcard + COBS frames, heapless Vec/String/SPSC queue, resync after noise; 5 tests |
| `embassy_stm32_family` | WeAct G431 / H562 / H743 | one Embassy firmware, chip by Cargo feature; 3 builds (H562 = Cortex-M33, H743 RAM at 0x24000000) |
| `embassy_pico_usb_cdc` | Pico | embassy-usb CDC-ACM upper-case echo; checks USB strings and the USBCTRL_IRQ handler in the image, UF2 |
| `embassy_pico2` | Pico 2 | RP2350 Cortex-M33: IMAGE_DEF block (marker checked), picotool binary info, PWM task on the FPU, UF2 rp2350-arm |
| `embassy_nrf52840_nicenano` | nice!nano | embassy-nrf, app at 0x26000 (S140 + UF2 bootloader kept), async TEMP sensor; UF2 blocks checked in the app range |
| `zig_ffi_qemu` | QEMU lm3s6965evb | Rust + Zig: a Zig static lib (Zig from 32_24_microzig) linked into the firmware, calls in both directions |

Templates (`./rust-box new <kind> <name> [--dir D]`; use a snake_case name), each self-contained (pinned toolchain,
lock file, `.cargo/config.toml` where a target is set, a README with the build/run lines). `tests/test_smoke.sh`
instantiates every kind and builds, tests or runs it offline:

| Template | From example | Smoke check |
|---|---|---|
| `embassy_app` | (Pico, Embassy, two tasks; commented `probe-rs run` runner) | `cargo build` |
| `rtic_app` | (BlackPill, RTIC v2 + stm32f4xx-hal) | `cargo build` |
| `nostd_lib` | (no_std library, optional `defmt`) | 2 host tests |
| `qemu_app` | `qemu_semihosting_hello` | `cargo run` in QEMU |
| `qemu_defmt_test` | `qemu_defmt_test` (library imported as `firmware_lib`) | `cargo test` in QEMU |
| `rtic_qemu_app` | `rtic_qemu_lm3s6965` | `cargo run` in QEMU |
| `embassy_qemu_app` | `embassy_qemu_timers` | `cargo run` in QEMU |
| `riscv_virt_app` | `riscv_qemu_virt` | `cargo run` in QEMU |
| `bootloader_app` | `qemu_bootloader_jump` | `cargo build` (both images) |
| `ehal_driver` | `ehal_driver_mock` | 7 host tests |
| `async_driver` | `async_spi_flash_mock` | 4 host tests |
| `protocol_lib` | `heapless_postcard_frames` | 5 host tests |
| `stm32_app` | `embassy_stm32_family` | `cargo build --features h562` |
| `pico2_app` | `embassy_pico2` | `cargo build` |
| `nrf52840_app` | `embassy_nrf52840_nicenano` | `cargo build` |
| `usb_cdc_app` | `embassy_pico_usb_cdc` | `cargo build` |
| `zig_ffi_app` | `zig_ffi_qemu` | Zig lib + `cargo run` in QEMU (skipped without 32_24_microzig) |

Other languages (2026-10-29):
- **Zig**: reached in 32_24_microzig, never installed here: `zig_ffi_qemu` / `zig_ffi_app` call its `microzig-box zig`
  (found through `BOX_MICROZIG` or the sibling folder; without it the example exits 3 and the smoke test skips it).
- **Bend 2**: not added. Bend emits hosted C only (pthread, mmap; no bare-metal target), so it could only check
  host-side laws of the no_std libraries, and its toolchain ID (32_31_bend) did not exist yet to verify any example.

Boards without an example here: **ESP32-C3/S3** live in `32_22_rust_next` (esp-hal 1.x; it pins its own toolchain
because rustc 1.99 breaks esp-hal 1.2.2's startup), the C3 target is installed only so the libraries cross-build for it;
**CH32V003** needs `riscv32ec`, which has no prebuilt std/core on stable Rust (nightly `-Zbuild-std`); **K210**
(riscv64gc) has only unmaintained HAL crates (k210-hal 0.2, 2020).

## Configuration and environment variables

| Variable | Default | Meaning |
|---|---|---|
| `BOXT_OUT` | same ID path on idisk03, else the ID folder | output root (`build/`, `log/`); also settable in `config/local.env` |
| `BOXT_JOBS` | 8 (maximum) | parallel jobs |
| `BOXT_MIN_FREE_GB` | 8 | pause builds while less RAM is free |
| `BOXT_RAM_WAIT_MAX` | 3600 | give up waiting for RAM after this many seconds |
| `BOXT_NET_TIMEOUT` | 3600 | seconds for one download |
| `BOXT_PLATFORM` | detected (`linux-x64`) | override the platform |
| `GITHUB_TOKEN` | unset | optional; lifts the GitHub API limit for `update --check` |
| `BOX_INTEGRATION` | 1 | `0` = standalone: ignore `$BOX`/`$BOX_V3`, keep output in this folder |
| `NO_COLOR` | unset | set to print plain text (colour is only used on a terminal anyway) |
| `RUST_BOX_TARGET_DIR` | `$BOXT_OUT/build/cargo-target` | cargo target dir, shared by all examples |
| `RUSTUP_HOME`, `CARGO_HOME` | `install/rustup`, `cache/cargo` | always set by `rust-box` inside this folder |
| `BOX_MICROZIG` | sibling folder `../32_24_microzig` | Zig for the interop example and template (reached, not copied) |
| `BOX_NATIVE_OFF_EMBEDDED_RUST` | unset | `1` skips native mode for one shell (old name `RUSTBOX_NATIVE_OFF` still honoured) |
| `BOX_EMBEDDED_RUST` | set by `box_env.sh` | path of this ID, used by the native snippet |

Pins: `config/pins.tsv`, boards `config/boards.tsv`, optional tools `config/cargo-tools.tsv`. `update` never edits them.

## Where files go

| What | Where |
|---|---|
| Source, launcher, examples, templates, `config/` | this ID |
| Tools | `install/rustup` (1.2 GB: toolchain, 6 targets, llvm-tools, clippy, rustfmt), `install/qemu-{arm,riscv}-9.2.4-1` (392 + 365 MB), `install/probe-rs-0.32.0` (142 MB), `install/flip-link-0.1.12`, `install/cargo-tools` (defmt-print), `.pixi/` (860 MB), `.bin/` (77 MB pixi) |
| Cache | `cache/cargo` (695 MB: rustup proxies + crates.io registry, 368 crates), `cache/pixi` (881 MB), `cache/downloads` (42 MB); `CACHEDIR.TAG` |
| Build output and logs | `$BOX_V3/30_39_tools/32_embedded/32_17_embedded_rust/{build,log}` (3.4 GB: `build/cargo-target` 3.2 GB, `build/cargo-install` 273 MB, ELFs/UF2s in `build/examples/<ex>/`) |

ID total 3.9 GB with caches (about 3.0 GB without `cache/`). `clean [--all] --yes` moves regenerable parts to the Trash
(logged in `$BOX/00_09_meta/02_logs/2026_10_24_box_trash.tsv`).

## Native mode

`./rust-box native` writes `native/native.{zsh,bash}` and relative wrappers in `native/bin`: `rust-box`, `ecargo` (the
pinned cargo through the launcher, so it never collides with `~/.cargo/bin/cargo`), `flip-link`, `defmt-print`,
`rb-probe-rs` (renamed so it does not shadow another `probe-rs`). Load it in every shell (zsh, bash, fish) with `box
native enable embedded_rust --apply`: it runs `./rust-box native` when the files are missing and rebuilds
`box_env.d/native.*` (without `--apply` it only shows the plan; `box native disable embedded_rust --apply` undoes it).
`BOX_NATIVE_OFF_EMBEDDED_RUST=1` skips it for one shell (the old `RUSTBOX_NATIVE_OFF` still works for now). Without
Box, source `native/native.zsh` or `native/native.bash` from your shell rc.

## Tests

```sh
./rust-box test --all      # smoke + dry-run + examples
tests/test_smoke.sh        # tools run from the ID + all 17 templates built/tested/run offline + UF2 + plan
tests/test_dry_run.sh      # every command in --dry-run + shellcheck
```

Last known (2026-10-29): doctor 30 ok / 1 warn / 0 fail; examples 26/26 (+26/26 offline rebuild); smoke 24/24 (17
templates); dry-run 37/37.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `cargo` in the shell is not the pinned one | use `./rust-box cargo ...` or `ecargo` (native); the host `~/.cargo/bin` is hidden only inside the environment |
| `cargo run` does not flash | by design no example sets a runner; uncomment the `probe-rs run` line in the project's `.cargo/config.toml` |
| probe-rs finds no probe / permission denied | install the udev rules (below); plugdev group on distros that use it |
| Build needs the network after `clean` | `cache/cargo` was trashed; `offline-check` rebuilds offline only while the registry cache exists |
| Host tests fail to link on Windows | the MSVC host needs the Visual Studio build tools (proprietary, not installed here) |
| Folder was moved | `./rust-box relocate`; `RUSTUP_HOME`/`CARGO_HOME` are derived from the folder each run |

## Limitations / needs you

- **Probe udev rules (sudo, run these yourself):** the file comes from <https://probe.rs/files/69-probe-rs.rules>:

```sh
sudo cp udev/69-probe-rs.rules /etc/udev/rules.d/
sudo udevadm control --reload && sudo udevadm trigger
# on distros that use it: sudo usermod -aG plugdev $USER
```

- **Serial access** for UART logs: `sudo usermod -aG uucp "$USER"`, log in again.
- **Hardware:** nothing was flashed. Not verified here: running on a real board or with a probe (board examples are
  built and statically checked; `defmt_test_pico` is only built; USB enumeration of `embassy_pico_usb_cdc`, the Pico 2
  boot ROM accepting the IMAGE_DEF block, the nice!nano bootloader accepting the UF2, the WeAct LED pins). The 8 QEMU
  examples run, but QEMU does not model clocks or peripheral timing exactly.
- **Other OSes:** Linux x64 tested. macOS, Linux arm64 and Windows (Git Bash/MSYS2) are pinned but untested.
- Related: ESP32 Rust (esp-hal, Ariel OS) is in `32_22_rust_next`; Zig in `32_24_microzig`.

## Scripts

Each script has a `[scripts.*]` descriptor in `.box.toml` (`box scripts show 32_17`; `box scripts check 32_17` must stay clean).

| File | Purpose |
|---|---|
| `rust-box` | launcher (POSIX sh): bootstraps the pinned pixi, keeps caches in this folder, then runs `lib/` |
| `lib/boxtool.bash` | shared launcher framework (boxtool 3: bootstrap, pinned downloads, pixi env, every common command); byte-identical copy of `$BOX/00_09_meta/05_policy/templates/tool/lib/boxtool.bash` (edit there, then sync) |
| `lib/rust.bash` | this tool's hooks and extra commands (cargo, rustup, plan, uf2, offline-check) |
| `lib/uf2conv.py` | ELF to UF2 converter (checked byte-for-byte against `llvm-objcopy -O binary`) |
| `examples/common/`, `examples/*/example.sh` | shared example code (`fw.bash`: build, ELF checks, qemu runner) and the 26 examples |
| `examples/*/qemu-defmt.sh` | cargo runner: QEMU piped into defmt-print (defmt examples/templates) |
| `tests/test_smoke.sh`, `tests/test_dry_run.sh` | smoke test (incl. every template), dry-run/shellcheck test |
| `native/` | written by `./rust-box native`; regenerate, do not edit |

## Links

- Rust: <https://static.rust-lang.org/dist/channel-rust-stable.toml>; rustup-init: <https://static.rust-lang.org/rustup/archive/1.29.2/>
- probe-rs: <https://github.com/probe-rs/probe-rs/releases/tag/v0.32.0>; flip-link: <https://github.com/knurling-rs/flip-link/releases/tag/v0.1.12>; defmt-print: <https://crates.io/crates/defmt-print>
- xPack QEMU: <https://github.com/xpack-dev-tools/qemu-arm-xpack/releases/tag/v9.2.4-1>, <https://github.com/xpack-dev-tools/qemu-riscv-xpack/releases/tag/v9.2.4-1>
- Embassy: <https://github.com/embassy-rs/embassy>; defmt: <https://github.com/knurling-rs/defmt>; RTIC: <https://github.com/rtic-rs/rtic>, book <https://rtic.rs/2/book/en/>
- pixi: <https://github.com/prefix-dev/pixi>
- Why these picks: [docs/why_these_picks.md](docs/why_these_picks.md). History: `NOTES.md`. Report: `$BOX/00_09_meta/02_logs/2026_10_25_embedded_tools2_report.md`.
- Sibling tools: `32_22_rust_next` (esp-hal, Ariel OS), `32_08_probe_tools`, `32_15_flashers`.
