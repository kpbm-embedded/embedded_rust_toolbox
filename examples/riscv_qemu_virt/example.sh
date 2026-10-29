#!/usr/bin/env bash
# about: bare-metal RISC-V (riscv-rt, rv32imac as on the Pico 2's Hazard3) on qemu virt: 16550 UART, CSRs, exit device
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + boot on the pinned qemu-system-riscv32; test = entry address, output and exit code.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=riscv_qemu_virt
ELF="$OUT/$BIN.elf"
qrun() {
  local rc=0
  echo "+ qemu-system-riscv32 -machine virt -nographic -bios none -kernel $BIN.elf"
  timeout 30 qemu-system-riscv32 -machine virt -nographic -monitor none -bios none -kernel "$ELF" >"$OUT/qemu.log" 2>&1 || rc=$?
  cat "$OUT/qemu.log"; return "$rc"
}
case "$act" in
  build) fw_build "$BIN" ;;
  run) fw_build "$BIN"; qrun ;;
  test)
    fw_build "$BIN"
    entry=$(llvm-readelf -h "$ELF" | awk '/Entry point/ {print $4}')
    [[ "$((entry))" = "$((0x80000000))" ]] || { echo "FAIL: entry $entry, expected 0x80000000"; exit 1; }
    grep -q 'RISC-V' <<<"$(llvm-readelf -h "$ELF")" || { echo "FAIL: not a RISC-V ELF"; exit 1; }
    echo "entry $entry; size: $(llvm-size "$ELF" | tail -1)"
    qrun || { echo "FAIL: qemu exit code"; exit 1; }
    expect "$OUT/qemu.log" "hello from Rust on qemu riscv32 virt" "hart 0 misa extensions" "isqrt(1000000) = 1000" "result: PASS"
    echo "PASS $BIN" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
