# 32_17_embedded_rust: notes

## 2026-10-25: created
- New ID in the Box tool style (launcher `rust-box`, `lib/boxtool.bash` copied unchanged from
  32_10_qemu, md5 b938d4f9...; `.bin/pixi` copied from 32_10_qemu, same pinned v0.81.0 binary).
- Rust via rustup-init 1.29.2 (sha256 from static.rust-lang.org) with `RUSTUP_HOME=install/rustup`
  and `CARGO_HOME=cache/cargo` (registry = offline cache). `~/.cargo/bin` is stripped from PATH in
  the environment. `~/.cargo`, `~/.rustup`, `~/.cache` listings compared before/after: unchanged.
- Toolchain 1.99.1 (current stable) in `rust-toolchain.toml`, profile minimal + llvm-tools,
  clippy, rustfmt (rust-src dropped: stable std/core for both thumb targets is prebuilt).
- probe-rs 0.32.0 and flip-link 0.1.12 from release tarballs (sha256 from the release checksum
  files); defmt-print 1.2.0 has no binary release, so `cargo install --locked` (build dir on $OUT).
- pixi env adds conda-forge `c-compiler` (gcc 14.4) only as the host linker (host tests, build
  scripts, cargo install) instead of relying on /usr/bin/cc; 860 MB, the largest single part.
- API changes met on the way: embassy-executor 0.10 renamed `arch-cortex-m` to `platform-cortex-m`
  and `#[task]` fns return a Result (`spawner.spawn(unwrap!(t()))`); embassy-stm32 0.6
  `ExtiInput<'d, Async>` takes a mode parameter; rtic-monotonics 3.0 (released 2026-10-21) takes
  the timer interrupt `Priority` in `start()` and its fugit 0.6 has `as_millis()` not `to_millis()`.
- Cortex-M0+ has no CAS: static_cell / RTIC's executor need `portable-atomic` with
  `critical-section` on the Pico examples (build failed without it).
- stm32f4xx-hal ships no memory.x (embassy-stm32 does with `memory-x`): the RTIC/defmt BlackPill
  examples carry their own (512K/128K).
- defmt_test_pico has no bin target, so its build.rs uses `rustc-link-arg-tests` (cargo rejects
  `-bins` there).
- Mistake: the first `offline-check` ran `cargo build --manifest-path` from the ID root, which
  ignores the example's `.cargo/config.toml` (cargo reads config from the cwd) and built for the
  host; it now runs from each example folder.
- GitHub's stargazer API no longer returns lists (REST 404, GraphQL empty edges), so the 12-month
  star counts in the README come from Wayback Machine snapshots of the repo pages.
- uf2conv.py output verified byte-for-byte against `llvm-objcopy -O binary` (embassy_pico_blinky).
- Results: doctor 22 ok / 1 warning (udev rules not installed) / 0 failed; examples test 10/10;
  offline-check 10/10; tests/test_smoke.sh 10/10; tests/test_dry_run.sh 37/37 (incl. shellcheck).
- Later the same day: `lib/boxtool.bash` replaced by the cleaned shared version from 32_06_renode
  (md5 0c5ba426..., adds `usage_error`/exit 2, help environment section). Applied the Box script
  style (00_09_meta/04_agents/script_style.md) to the ID's own scripts: `[[ ]]`, usage errors exit
  2, help lists tool examples and environment, uf2conv.py with pathlib/type hints/argparse help.
  Re-run: doctor 22/1/0, examples test 10/10, smoke 10/10, dry-run 37/37.

## 2026-10-25: readability pass 2 (script_style.md, scripts/README)
- lib/boxtool.bash (reference `00_09_meta/05_policy/templates/tool/lib/boxtool.bash`, synced to all 24 copies):
  `where`/`status`/`doctor`/`bootstrap`/`relocate` reject extra arguments (exit 2); colour only on a terminal
  and never with `NO_COLOR`; help lists `NO_COLOR`.
- examples/common/fw.bash: `llvm-readelf | grep -q` (SIGPIPE flake under pipefail) -> here-string.
- README: exit codes, `BOX_INTEGRATION` and `NO_COLOR` rows. Backups:
  `00_09_meta/02_logs/config_backups/2026_10_25_readability2_embedded_tools/`.

## 2026-10-29: 16 examples, 14 templates, QEMU, more targets, Zig interop; git repo
- Targets added in `rust-toolchain.toml` (so `bootstrap` gives a fresh clone all of them): thumbv7m (QEMU),
  thumbv8m.main-hf (Pico 2, WeAct H562), riscv32imac (Pico 2 Hazard3, QEMU virt), riscv32imc (ESP32-C3 libs).
  doctor reads the target list from that file.
- xPack QEMU 9.2.4-1 (qemu-arm, qemu-riscv) pinned in `config/pins.tsv` (same pins/sha256 as 32_10_qemu, own copy
  in `install/`; conda-forge has only user-mode `qemu-execve-*`, no qemu-system). Added to PATH and doctor.
- New boards in `config/boards.tsv`: pico2, weact_g431, weact_h562, weact_h743, nicenano. `plan` prints the right UF2
  drive per family; `lib/uf2conv.py` gained the nrf52840 family (0xada52840, flash window 0x1000..0x100000).
- Examples (16): qemu_semihosting_hello, qemu_hardfault, qemu_defmt_semihosting, qemu_defmt_test, rtic_qemu_lm3s6965,
  embassy_qemu_timers, qemu_bootloader_jump, riscv_qemu_virt (all run in QEMU); ehal_driver_mock,
  async_spi_flash_mock, heapless_postcard_frames (host tests + cross build); embassy_stm32_family,
  embassy_pico_usb_cdc, embassy_pico2, embassy_nrf52840_nicenano (build + ELF/UF2 checks); zig_ffi_qemu.
- Templates (14): qemu_app, qemu_defmt_test, rtic_qemu_app, embassy_qemu_app, riscv_virt_app, bootloader_app,
  ehal_driver, async_driver, protocol_lib, stm32_app, pico2_app, nrf52840_app, usb_cdc_app, zig_ffi_app.
  `tests/test_smoke.sh` instantiates all 17 kinds and builds/tests/runs each offline.
- Zig: Zig 0.17 from 32_24_microzig builds a thumb static lib (`-fno-unwind-tables`, else it references
  `__aeabi_unwind_cpp_pr0`); Rust links it and Zig calls back into Rust. Zig is reached via
  `BOX_MICROZIG` or the sibling folder; exit 3 without it.
- A BEAM interop example (no_std term_to_binary codec, checked against AtomVM 0.6.6 and OTP 27) was written and
  passed, then removed the same day (user correction: the third language is Bend 2, not BEAM; moved to the Trash).
  Bend 2 not added: it emits hosted C only (no MCU target) and 32_31_bend did not exist yet.
- Problems found: cargo does not find `[target.'thumbv8m.main-none-eabihf']` (quoted key with a dot; cargo splits
  config keys at dots), so flip-link was silently not used for the H562/Pico 2: written unquoted, with a comment.
  postcard 1.1 still uses heapless 0.7 (encode into a slice, copy into heapless 0.9). heapless `spsc` needs CAS:
  not on thumbv6m/riscv32imc. `panic.rs` without any `cortex_m` use did not link the critical-section impl.
  QEMU refused `-device loader,file=app.elf` next to `-kernel boot.elf` (overlapping ELF header segment at 0):
  the app is loaded as a raw `app.bin` at 0x8000, as a flasher would write it.
- `offline-check` now runs each example's own `build` with CARGO_NET_OFFLINE=true (the old plain `cargo build`
  could not build the multi-target, feature-selected or Zig examples).
- The output disk ($BOX_V3) ran full for a while (other jobs): built with BOXT_OUT in the ID meanwhile, then trashed that output.
- Skipped: ESP32-C3/S3 examples (32_22_rust_next owns esp-hal; rustc 1.99 breaks esp-hal 1.2.2 startup per its
  NOTES), CH32V003 (riscv32ec: no stable prebuilt core), K210 (unmaintained HAL).
- Repo: `.gitignore` covers build/log/install/.pixi/.bin/native/cache (also as a symlink), Cargo `target/`, zig caches.
- Results: doctor 30 ok / 1 warn (udev rules); examples 26/26; offline-check 26/26; smoke 24/24 (17 templates);
  dry-run 37/37 (shellcheck incl. examples/*/*.sh).
