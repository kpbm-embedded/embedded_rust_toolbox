#!/usr/bin/env bash
# about: Embassy (embassy-stm32) on the BlackPill F411: LED PC13 blink, KEY PA0 via async EXTI
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + PRINT the flash plan (never flashes); test = build + static ELF checks.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=embassy_blackpill_blinky
ELF="$OUT/$BIN.elf"
case "$act" in
  build) fw_build "$BIN" ;;
  run) fw_build "$BIN"; fw_plan blackpill_f411 "$ELF" ;;
  test)
    fw_build "$BIN"
    fw_check "$ELF" 0x08000000 0x20000000
    echo "PASS embassy_blackpill_blinky" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
