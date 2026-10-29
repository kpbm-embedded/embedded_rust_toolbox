#!/usr/bin/env bash
# about: Embassy on the Pico 2 (RP2350 Cortex-M33): IMAGE_DEF block, picotool binary info, PWM task; ELF checks + UF2
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + PRINT the flash plan; test = vector table, flip-link, the IMAGE_DEF marker in the first 4 KB,
# Armv8-M attributes and a UF2 with the rp2350-arm family. Booting needs the board (not tested here).
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=embassy_pico2
ELF="$OUT/$BIN.elf"
case "$act" in
  build) fw_build "$BIN" ;;
  run) fw_build "$BIN"; fw_plan pico2 "$ELF" ;;
  test)
    fw_build "$BIN"
    fw_check "$ELF" 0x10000000 0x20000000
    sb=$(sec_addr "$ELF" '\.start_block')
    [[ -n "$sb" ]] && [[ "$((16#$sb))" -lt $((0x10001000)) ]] || { echo "FAIL: .start_block at 0x${sb:-?}, must be in the first 4 KB"; exit 1; }
    marker=$(llvm-objdump -s -j .start_block "$ELF" | grep -m1 -E '^ +[0-9a-f]+ ' | awk '{print $2}')
    [[ "$marker" = d3deffff ]] || { echo "FAIL: IMAGE_DEF start marker $marker, expected 0xffffded3"; exit 1; }
    echo "IMAGE_DEF block at 0x$sb (marker 0xffffded3)"
    grep -q 'v8-M' <<<"$(llvm-readelf -A "$ELF")" || { echo "FAIL: not an Armv8-M image"; exit 1; }
    grep -qaF "Box embedded-rust: Pico 2" "$ELF" || { echo "FAIL: binary info missing"; exit 1; }
    "$LAUNCH" uf2 "$ELF" -o "$OUT/$BIN.uf2" --family rp2350-arm
    echo "PASS $BIN" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
