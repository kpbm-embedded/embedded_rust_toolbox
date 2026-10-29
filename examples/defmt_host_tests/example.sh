#!/usr/bin/env bash
# about: no_std library (moving average, CRC-8, COBS) unit-tested on the host; defmt::Format on target
# Usage (through ./rust-box examples ...): example.sh build | run | test
# build = cross-compile the lib for both boards (+ defmt feature); test = host `cargo test` + build.
set -euo pipefail
act=${1:-test}
OUT=${EX_OUT:?run through ./rust-box examples}
HOST=$(rustc -vV | sed -n 's/^host: //p')
build() {
  local t
  for t in thumbv6m-none-eabi thumbv7em-none-eabihf; do
    cargo build --release --lib --target "$t" --features defmt
    cp "$CARGO_TARGET_DIR/$t/release/libdefmt_host_tests.rlib" "$OUT/libdefmt_host_tests-$t.rlib"
  done
  echo "no_std lib built for thumbv6m (Pico) and thumbv7em-hf (BlackPill) with defmt::Format"
}
case "$act" in
  build) build ;;
  run|test)
    cargo test --target "$HOST" 2>&1 | tee "$OUT/test.log"
    grep -q 'test result: ok\. 6 passed' "$OUT/test.log" || { echo "FAIL: host tests"; exit 1; }
    build
    echo "PASS defmt_host_tests" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
