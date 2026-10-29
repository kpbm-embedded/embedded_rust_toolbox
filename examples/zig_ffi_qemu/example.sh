#!/usr/bin/env bash
# about: Rust + Zig on one Cortex-M3: a Zig static lib (32_24_microzig's Zig) linked into Rust, calls both ways; qemu
# Usage (through ./rust-box examples ...): example.sh build | run | test
# Needs 32_24_microzig (BOX_MICROZIG or the sibling folder): reached through its launcher, nothing copied;
# exits 3 (skip) without it.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=zig_ffi_qemu
ELF="$OUT/$BIN.elf"
MZ=${MICROZIG_ID:-}
[[ -x "$MZ/microzig-box" ]] || { echo "skip: 32_24_microzig not found (set BOX_MICROZIG); Zig is not installed by rust-box"; exit 3; }
build() {
  mkdir -p "$OUT/zig"
  echo "+ microzig-box zig build-lib -target thumb-freestanding-eabi -mcpu cortex_m3 -O ReleaseSmall zig/dsp.zig"
  "$MZ/microzig-box" zig build-lib -target thumb-freestanding-eabi -mcpu cortex_m3 -O ReleaseSmall \
    -fno-unwind-tables --cache-dir "$OUT/zig/cache" -femit-bin="$OUT/zig/libzigdsp.a" "$PWD/zig/dsp.zig"
  ZIG_LIB_DIR="$OUT/zig" fw_build "$BIN"
}
case "$act" in
  build) build ;;
  run) build; qemu_arm lm3s6965evb "$ELF" "$OUT/qemu.log" ;;
  test)
    build
    elf_vt "$ELF" 0x0
    syms=$(llvm-nm "$ELF")
    for s in zig_crc32 zig_fir_q15 rust_on_block; do grep -q " T $s$" <<<"$syms" || { echo "FAIL: $s not linked"; exit 1; }; done
    echo "linked: zig_crc32, zig_fir_q15 (Zig), rust_on_block (Rust)"
    qemu_arm lm3s6965evb "$ELF" "$OUT/qemu.log" || { echo "FAIL: qemu exit code"; exit 1; }
    expect "$OUT/qemu.log" "zig 0xcbf43926 rust 0xcbf43926" "zig -> rust: block 1 peak -800" "result: PASS"
    echo "PASS $BIN" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
