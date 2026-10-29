#!/usr/bin/env bash
# about: RTIC v2 on qemu lm3s6965evb: hardware task, priorities, shared locks, rtic-sync channel, SysTick monotonic
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + run on the pinned qemu-system-arm; test = run + check the output and the exit code.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=rtic_qemu_lm3s6965
ELF="$OUT/$BIN.elf"
case "$act" in
  build) fw_build "$BIN" ;;
  run) fw_build "$BIN"; qemu_arm lm3s6965evb "$ELF" "$OUT/qemu.log" ;;
  test)
    fw_build "$BIN"
    elf_vt "$ELF" 0x0
    qemu_arm lm3s6965evb "$ELF" "$OUT/qemu.log" || { echo "FAIL: qemu exit $?"; exit 1; }
    expect "$OUT/qemu.log" "rtic init done" "press 1: period 5 ms" "press 3: period 20 ms" "presses 3" "result: PASS"
    echo "PASS $BIN" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
