#!/usr/bin/env bash
# about: panic and HardFault handling on qemu mps2-an385: fault frame + HFSR/CFSR, panic location, exit codes
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + run both programs on qemu; test = check the fault PC is the udf instruction and the exit codes.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
build() { fw_build hardfault; fw_build panic; }
case "$act" in
  build) build ;;
  run) build; qemu_arm mps2-an385 "$OUT/hardfault.elf" "$OUT/hardfault.log" || true
       qemu_arm mps2-an385 "$OUT/panic.elf" "$OUT/panic.log" || true ;;
  test)
    build
    elf_vt "$OUT/hardfault.elf" 0x0
    qemu_arm mps2-an385 "$OUT/hardfault.elf" "$OUT/hardfault.log" || { echo "FAIL: hardfault exit code"; exit 1; }
    expect "$OUT/hardfault.log" "HardFault: pc=0x" "FORCED=1" "UNDEFINSTR=1"
    pc=$(sed -n 's/.*HardFault: pc=0x\([0-9a-f]*\).*/\1/p' "$OUT/hardfault.log")
    insn=$(llvm-objdump -d --start-address="0x$pc" --stop-address="$((16#$pc + 2))" "$OUT/hardfault.elf" | tail -1)
    grep -q 'udf' <<<"$insn" || { echo "FAIL: instruction at 0x$pc is not udf: $insn"; exit 1; }
    echo "fault PC 0x$pc disassembles to: ${insn//$'\t'/ }"
    rc=0; qemu_arm mps2-an385 "$OUT/panic.elf" "$OUT/panic.log" || rc=$?
    [[ "$rc" = 1 ]] || { echo "FAIL: panic exit code $rc, expected 1"; exit 1; }
    expect "$OUT/panic.log" "reading 1234 -> 1234" "PANIC at src/bin/panic.rs:" "sensor value out of range: 4242"
    echo "PASS qemu_hardfault" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
