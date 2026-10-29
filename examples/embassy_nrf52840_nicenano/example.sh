#!/usr/bin/env bash
# about: Embassy on the nice!nano nRF52840: app linked at 0x26000 (SoftDevice + UF2 bootloader kept), async TEMP; UF2
# Usage (through ./rust-box examples ...): example.sh build | run | test
# run = build + PRINT the flash plan; test = vector table at 0x26000, nothing below it or in the bootloader,
# flip-link, and a UF2 (family 0xada52840) whose blocks all lie in the app region. Flashing needs the board.
set -euo pipefail
act=${1:-run}
# shellcheck source-path=SCRIPTDIR source=../common/fw.bash
. ../common/fw.bash
BIN=embassy_nrf52840_nicenano
ELF="$OUT/$BIN.elf"
case "$act" in
  build) fw_build "$BIN" ;;
  run) fw_build "$BIN"; fw_plan nicenano "$ELF" ;;
  test)
    fw_build "$BIN"
    fw_check "$ELF" 0x26000 0x20000008
    "$LAUNCH" uf2 "$ELF" -o "$OUT/$BIN.uf2" --family nrf52840
    python3 - "$OUT/$BIN.uf2" <<'PY'
import struct, sys
d = open(sys.argv[1], "rb").read()
addrs = [struct.unpack_from("<I", d, i + 12)[0] for i in range(0, len(d), 512)]
fams = {struct.unpack_from("<I", d, i + 28)[0] for i in range(0, len(d), 512)}
assert fams == {0xADA52840}, fams
assert min(addrs) == 0x26000 and max(addrs) + 256 <= 0xF4000, (hex(min(addrs)), hex(max(addrs)))
print(f"uf2: {len(addrs)} blocks in 0x{min(addrs):x}..0x{max(addrs) + 256:x}, family 0xada52840")
PY
    echo "PASS $BIN" ;;
  *) echo "usage: example.sh build|run|test" >&2; exit 2 ;;
esac
