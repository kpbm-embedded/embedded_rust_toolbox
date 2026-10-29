#!/usr/bin/env bash
# about: USB CDC-ACM serial device on the Pico (embassy-usb): upper-case echo; build + descriptor/ELF checks, UF2
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + PRINT the flash plan (never flashes); test = static checks: vector table, flip-link, the
# USB strings and VID/PID in the image, USBCTRL_IRQ bound. Enumeration needs the board (not tested here).
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=embassy_pico_usb_cdc
ELF="$OUT/$BIN.elf"
case "$act" in
  build) fw_build "$BIN" ;;
  run) fw_build "$BIN"; fw_plan pico "$ELF" ;;
  test)
    fw_build "$BIN"
    fw_check "$ELF" 0x10000100 0x20000000
    llvm-objcopy -O binary "$ELF" "$OUT/$BIN.bin"
    for s in "Box embedded-rust" "Pico CDC echo" "BOX-0001"; do
      grep -qaF "$s" "$OUT/$BIN.bin" || { echo "FAIL: USB string '$s' not in the image"; exit 1; }
    done
    grep -q 'USBCTRL_IRQ' <<<"$(llvm-nm "$ELF")" || { echo "FAIL: USBCTRL_IRQ handler not linked"; exit 1; }
    echo "USB strings present, USBCTRL_IRQ handler linked"
    "$LAUNCH" uf2 "$ELF" -o "$OUT/$BIN.uf2"
    echo "PASS $BIN" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
