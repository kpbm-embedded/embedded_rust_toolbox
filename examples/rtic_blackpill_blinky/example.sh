#!/usr/bin/env bash
# about: RTIC v2 on the BlackPill (stm32f4xx-hal): 96 MHz, SysTick monotonic, async blink task with delay_until
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + PRINT the flash plan (never flashes); test = build + static ELF checks.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=rtic_blackpill_blinky
ELF="$OUT/$BIN.elf"
case "$act" in
  build) fw_build "$BIN" ;;
  run) fw_build "$BIN"; fw_plan blackpill_f411 "$ELF" ;;
  test)
    fw_build "$BIN"
    fw_check "$ELF" 0x08000000 0x20000000
    echo "PASS rtic_blackpill_blinky" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
