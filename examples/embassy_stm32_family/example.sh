#!/usr/bin/env bash
# about: one Embassy firmware for WeAct G431 / H562 (Cortex-M33) / H743 by Cargo feature; 3 builds + ELF checks
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + PRINT the flash plans (never flashes); test = build all three + static ELF checks.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=embassy_stm32_family
# board feature target ram-origin
BOARDS_LIST="weact_g431 g431 thumbv7em-none-eabihf 0x20000000
weact_h562 h562 thumbv8m.main-none-eabihf 0x20000000
weact_h743 h743 thumbv7em-none-eabihf 0x24000000"
build() {
  local b f t r
  while read -r b f t r; do
    cargo build --release --bin "$BIN" --features "$f" --target "$t"
    cp "$CARGO_TARGET_DIR/$t/release/$BIN" "$OUT/$BIN-$f.elf"
    echo "built $OUT/$BIN-$f.elf"
  done <<<"$BOARDS_LIST"
}
case "$act" in
  build) build ;;
  run) build; while read -r b f _ _; do fw_plan "$b" "$OUT/$BIN-$f.elf"; done <<<"$BOARDS_LIST" ;;
  test)
    build
    while read -r b f t r; do
      echo "== $b ($t)"
      fw_check "$OUT/$BIN-$f.elf" 0x08000000 "$r"
      grep -q 'ARM' <<<"$(llvm-readelf -h "$OUT/$BIN-$f.elf")" || { echo "FAIL: not ARM"; exit 1; }
    done <<<"$BOARDS_LIST"
    # the H562 image must use Armv8-M instructions (Cortex-M33), the others Armv7E-M
    grep -q 'v8-M' <<<"$(llvm-readelf -A "$OUT/$BIN-h562.elf")" || { echo "FAIL: H562 build is not Armv8-M"; exit 1; }
    grep -q 'v7E-M' <<<"$(llvm-readelf -A "$OUT/$BIN-g431.elf")" || { echo "FAIL: G431 build is not Armv7E-M"; exit 1; }
    echo "PASS $BIN" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
