#!/usr/bin/env bash
# about: defmt-test unit tests that run on the target CPU in qemu lm3s6965evb (cargo test = run in QEMU)
# Usage (through ./rust-box examples ...): example.sh build | run | test
# build = test ELF only; run/test = `cargo test` (the runner qemu-defmt.sh runs it in QEMU, defmt-print decodes).
set -euo pipefail
act=${1:-test}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
case "$act" in
  build) cargo test --release --test on_qemu --no-run ;;
  run|test)
    timeout 300 cargo test --release --test on_qemu 2>&1 | tee "$OUT/test.log"
    expect "$OUT/test.log" "(1/5) running \`proportional_only_at_start\`" "(5/5) running \`hardware_float_is_absent\`" "all tests passed!"
    echo "PASS qemu_defmt_test" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
