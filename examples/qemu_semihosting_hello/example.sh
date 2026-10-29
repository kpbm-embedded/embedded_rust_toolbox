#!/usr/bin/env bash
# about: Cortex-M3 firmware run on qemu lm3s6965evb (no board): semihosting print, CPUID, exit code
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + run on the pinned qemu-system-arm; test = run + check the output and the exit code.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=qemu_semihosting_hello
ELF="$OUT/$BIN.elf"
case "$act" in
  build) fw_build "$BIN" ;;
  run) fw_build "$BIN"; qemu_arm lm3s6965evb "$ELF" "$OUT/qemu.log" ;;
  test)
    fw_build "$BIN"
    elf_vt "$ELF" 0x0
    qemu_arm lm3s6965evb "$ELF" "$OUT/qemu.log" || { echo "FAIL: qemu exit $?"; exit 1; }
    expect "$OUT/qemu.log" "hello from Rust on qemu lm3s6965evb" "partno 0xc23" "fib(30) = 832040" "result: PASS"
    echo "PASS $BIN" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
