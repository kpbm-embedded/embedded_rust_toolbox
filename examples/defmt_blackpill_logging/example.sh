#!/usr/bin/env bash
# about: defmt on the BlackPill (no runtime): levels, derive(Format), display hints, DWT timestamp, level filter
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + PRINT the flash plan (never flashes); test = build + ELF checks of the defmt table.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=defmt_blackpill_logging
ELF="$OUT/$BIN.elf"
case "$act" in
  build) fw_build "$BIN" ;;
  run) fw_build "$BIN"; fw_plan blackpill_f411 "$ELF" ;;
  test)
    fw_build "$BIN"
    fw_check "$ELF" 0x08000000 0x20000000
    # defmt interns format strings as symbols in .defmt; DEFMT_LOG=debug must drop trace!
    syms=$(llvm-nm "$ELF")
    for tag in defmt_info defmt_debug defmt_warn defmt_error defmt_timestamp; do
      grep -q "\"tag\":\"$tag\"" <<<"$syms" || { echo "FAIL: no $tag entry in the defmt table"; exit 1; }
    done
    if grep -q '"tag":"defmt_trace"' <<<"$syms"; then echo "FAIL: trace! not filtered by DEFMT_LOG"; exit 1; fi
    echo "defmt table: $(grep -c '"tag":"defmt_' <<<"$syms") interned strings; trace! compiled out"
    echo "PASS $BIN" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
