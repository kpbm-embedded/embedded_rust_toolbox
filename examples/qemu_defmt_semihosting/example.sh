#!/usr/bin/env bash
# about: defmt logging without a probe: frames over semihosting on qemu lm3s6965evb, decoded by defmt-print
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + qemu | defmt-print; test = also check the decoded lines and that trace! was compiled out.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=qemu_defmt_semihosting
ELF="$OUT/$BIN.elf"
case "$act" in
  build) fw_build "$BIN" ;;
  run) fw_build "$BIN"; timeout 60 ./qemu-defmt.sh "$ELF" ;;
  test)
    fw_build "$BIN"
    elf_vt "$ELF" 0x0
    grep -q ' \.defmt ' <<<"$(llvm-readelf -S "$ELF")" || { echo "FAIL: no .defmt section"; exit 1; }
    timeout 60 ./qemu-defmt.sh "$ELF" 2>&1 | tee "$OUT/decoded.log"
    expect "$OUT/decoded.log" "defmt over semihosting on qemu lm3s6965evb" "reading Reading { channel: 2, millivolts: 1500 }" \
      "battery 3100 mV below 3300 mV" "state Sampling { left: 2 } then Idle" "frame [de, ad, be, ef]" "done"
    ! grep -q 'compile time' "$OUT/decoded.log" || { echo "FAIL: trace! was not filtered"; exit 1; }
    echo "PASS $BIN" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
