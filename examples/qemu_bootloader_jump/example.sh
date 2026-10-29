#!/usr/bin/env bash
# about: bootloader + application linker layouts (0x0 / 0x8000) on qemu lm3s6965evb: validate, set VTOR, jump
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = both images in one QEMU (boot.elf via -kernel, app.bin via the generic loader at 0x8000); test = layout checks + output.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
run_both() {
  local rc=0
  # the app goes in as a raw image at its flash address, as a flasher would write it
  llvm-objcopy -O binary "$OUT/app.elf" "$OUT/app.bin"
  echo "+ qemu-system-arm -machine lm3s6965evb -kernel boot.elf -device loader,file=app.bin,addr=0x8000"
  timeout 30 qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -monitor none \
    -semihosting-config enable=on,target=native -kernel "$OUT/boot.elf" -device loader,file="$OUT/app.bin",addr=0x8000 \
    >"$OUT/qemu.log" 2>&1 || rc=$?
  cat "$OUT/qemu.log"; return "$rc"
}
case "$act" in
  build) fw_build boot; fw_build app ;;
  run) fw_build boot; fw_build app; run_both ;;
  test)
    fw_build boot; fw_build app
    elf_vt "$OUT/boot.elf" 0x0
    elf_vt "$OUT/app.elf" 0x8000
    end=$(llvm-size -A -d "$OUT/boot.elf" | awk '$3 ~ /^[0-9]+$/ && $3 > 0 && $3 < 536870912 {print $2 + $3}' | sort -n | tail -1)
    [[ "$end" -le $((0x8000)) ]] || { echo "FAIL: bootloader ends at $end, overlaps the app"; exit 1; }
    echo "bootloader ends at $(printf 0x%x "$end") (< 0x8000)"
    run_both || { echo "FAIL: qemu exit code"; exit 1; }
    expect "$OUT/qemu.log" "sp=0x20010000 reset=0x0000" "boot: jumping to the application" "app: running, VTOR=0x00008000" \
      "SysTick interrupts through the relocated vector table"
    echo "PASS qemu_bootloader_jump" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
