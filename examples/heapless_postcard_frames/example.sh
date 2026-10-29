#!/usr/bin/env bash
# about: no-alloc wire protocol: serde + postcard + COBS frames, heapless Vec/String/SPSC queue; host tests + cross-build
# Usage (through ./rust-box examples ...): example.sh build | run | test
# build = cross-compile the no_std driver ; test = host `cargo test` (5 tests) + build.
set -euo pipefail
act=${1:-test}
OUT=${EX_OUT:?run through ./rust-box examples}
HOST=$(rustc -vV | sed -n 's/^host: //p')
NAME=heapless_postcard_frames
build() {
  local t
  for t in thumbv7em-none-eabihf thumbv8m.main-none-eabihf riscv32imac-unknown-none-elf; do
    cargo build --release --lib --target "$t" 
    cp "$CARGO_TARGET_DIR/$t/release/lib$NAME.rlib" "$OUT/lib$NAME-$t.rlib"
  done
  echo "protocol built for thumbv7em-hf (BlackPill/WeAct/nRF52840), thumbv8m.main-hf (Pico 2, H562), riscv32imac (Pico 2 RISC-V)"
  echo "(spsc::Queue needs atomic CAS: thumbv6m/riscv32imc would need heapless's portable-atomic feature)"
}
case "$act" in
  build) build ;;
  run|test)
    cargo test --target "$HOST" 2>&1 | tee "$OUT/test.log"
    grep -q 'test result: ok\. 5 passed' "$OUT/test.log" || { echo "FAIL: host tests"; exit 1; }
    build
    echo "PASS $NAME" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
