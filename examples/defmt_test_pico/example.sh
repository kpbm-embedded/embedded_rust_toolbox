#!/usr/bin/env bash
# about: defmt-test on-target unit tests for the Pico (4 tests); builds the test ELF, run is plan-only
# Usage (through ./rust-box examples ...): example.sh build | run | test
# The tests execute on the board only (probe-rs run); here: build + ELF checks + the printed plan.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
ELF="$OUT/defmt_test_pico_on_target.elf"
build() {
  local log exe
  log=$(cargo test --release --test on_target --no-run 2>&1) || { echo "$log"; return 1; }
  echo "$log"
  exe=$(sed -n 's/.*Executable .*(\(.*\))$/\1/p' <<<"$log" | tail -1)
  [[ -n "$exe" ]] || { echo "FAIL: no test executable reported"; return 1; }
  cp "$exe" "$ELF"; echo "built $ELF"
}
case "$act" in
  build) build ;;
  run) build; fw_plan pico "$ELF" ;;
  test)
    build
    fw_check "$ELF" 0x10000100 0x20000000
    syms=$(llvm-nm "$ELF")
    grep -q ' DEFMT_TEST_COUNT$' <<<"$syms" || { echo "FAIL: not a defmt-test binary"; exit 1; }
    for t in adc_full_scale temp_at_27c duty_saturates state_is_shared; do
      grep -qF "running \`$t\`" <<<"$syms" || { echo "FAIL: test $t not in the ELF"; exit 1; }
    done
    echo "defmt-test: 4 on-target tests linked; run them on the Pico with the printed plan:"
    fw_plan pico "$ELF" | grep -m1 'probe-rs run'
    echo "PASS defmt_test_pico" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
