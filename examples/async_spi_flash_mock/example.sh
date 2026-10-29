#!/usr/bin/env bash
# about: embedded-hal-async SPI NOR flash driver (W25Qxx) tested on the host with the async SPI mock; cross-built
# Usage (through ./rust-box examples ...): example.sh build | run | test
# build = cross-compile the no_std driver ; test = host `cargo test` (4 tests) + build.
set -euo pipefail
act=${1:-test}
OUT=${EX_OUT:?run through ./rust-box examples}
HOST=$(rustc -vV | sed -n 's/^host: //p')
NAME=async_spi_flash_mock
build() {
  local t
  for t in thumbv6m-none-eabi thumbv7em-none-eabihf riscv32imc-unknown-none-elf; do
    cargo build --release --lib --target "$t" 
    cp "$CARGO_TARGET_DIR/$t/release/lib$NAME.rlib" "$OUT/lib$NAME-$t.rlib"
  done
  echo "driver built for thumbv6m (Pico), thumbv7em-hf (BlackPill/WeAct/nRF52840), riscv32imc (ESP32-C3)"
}
case "$act" in
  build) build ;;
  run|test)
    cargo test --target "$HOST" 2>&1 | tee "$OUT/test.log"
    grep -q 'test result: ok\. 4 passed' "$OUT/test.log" || { echo "FAIL: host tests"; exit 1; }
    build
    echo "PASS $NAME" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
