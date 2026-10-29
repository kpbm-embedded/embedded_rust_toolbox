#!/usr/bin/env bash
# about: Embassy on qemu mps2-an385 with an own SysTick time driver: tasks, Channel, Ticker, Timer, select timeout
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + run on the pinned qemu-system-arm; test = run + check the output and the exit code.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=embassy_qemu_timers
ELF="$OUT/$BIN.elf"
case "$act" in
  build) fw_build "$BIN" ;;
  run) fw_build "$BIN"; qemu_arm mps2-an385 "$ELF" "$OUT/qemu.log" ;;
  test)
    fw_build "$BIN"
    elf_vt "$ELF" 0x0
    qemu_arm mps2-an385 "$ELF" "$OUT/qemu.log" || { echo "FAIL: qemu exit $?"; exit 1; }
    expect "$OUT/qemu.log" "SysTick time driver" "reading 500 at" "received 5 readings, sum 1500, timeout true" "result: PASS"
    echo "PASS $BIN" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
