#!/usr/bin/env bash
# about: RTIC v2 on the Pico (rp2040-hal, thumbv6 backend): TIMER monotonic, blink + periodic report tasks
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + PRINT the flash plan (never flashes); test = build + static ELF checks.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=rtic_pico_monotonic
ELF="$OUT/$BIN.elf"
case "$act" in
  build) fw_build "$BIN" ;;
  run) fw_build "$BIN"; fw_plan pico "$ELF" ;;
  test)
    fw_build "$BIN"
    fw_check "$ELF" 0x10000100 0x20000000
    "$LAUNCH" uf2 "$ELF" -o "$OUT/$BIN.uf2"
    [[ -s "$OUT/$BIN.uf2" ]] || { echo "FAIL: no uf2"; exit 1; }
    echo "PASS rtic_pico_monotonic" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
