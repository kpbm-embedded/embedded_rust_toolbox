#!/usr/bin/env bash
# qemu-defmt.sh <elf>: run a Cortex-M3 ELF on qemu lm3s6965evb and decode its defmt output (semihosting
# stdout) with defmt-print. Exit code = the firmware's semihosting exit code. Needs qemu-system-arm and
# defmt-print on PATH (inside ./rust-box shell or run).
set -euo pipefail
elf=${1:?usage: qemu-defmt.sh <elf>}
qemu-system-arm -cpu cortex-m3 -machine lm3s6965evb -nographic -monitor none \
  -semihosting-config enable=on,target=native -kernel "$elf" | defmt-print -e "$elf"
