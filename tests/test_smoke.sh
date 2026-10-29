#!/usr/bin/env bash
# Smoke test of the installed rust-box: the pinned tools run from the ID (not ~/.cargo), and every
# template is instantiated with `rust-box new` and built OFFLINE from the in-ID registry cache: the
# library templates pass their host tests, the qemu templates RUN in the pinned QEMU (cargo run /
# cargo test through their runner), the board templates link (flip-link); the Pico ELF converts to UF2.
# zig_ffi_app needs 32_24_microzig (BOX_MICROZIG or the sibling folder) and is skipped without it.
# Usage: tests/test_smoke.sh
set -uo pipefail
HERE=$(cd "$(dirname "$0")/.." && pwd -P)
Q="$HERE/rust-box"
pass=0; failn=0
ok()  { printf 'ok    %s\n' "$*"; pass=$((pass+1)); }
bad() { printf 'FAIL  %s\n' "$*"; failn=$((failn+1)); }
tmp=$(mktemp -d)
cleanup() { if command -v trash-put >/dev/null; then trash-put "$tmp"; fi; }   # own temp dir; never rm
trap cleanup EXIT

want=$(sed -n 's/^channel = "\(.*\)"/\1/p' "$HERE/rust-toolchain.toml")
if v=$("$Q" run rustc --version 2>&1) && grep -q "rustc $want " <<<"$v"; then ok "$v"; else bad "rustc: $v"; fi
if v=$("$Q" run sh -c 'command -v cargo' 2>&1) && [[ "${v#"$HERE"/}" != "$v" ]]; then ok "cargo from the ID: ${v#"$HERE"/}"; else bad "cargo path: $v"; fi
for c in probe-rs defmt-print; do
  if v=$("$Q" run "$c" --version 2>&1 | head -1) && [[ -n "$v" ]]; then ok "$v"; else bad "$c: $v"; fi
done
if "$Q" run flip-link --help 2>&1 | grep 'stack overflow' >/dev/null; then ok "flip-link runs"; else bad "flip-link"; fi

host=$("$Q" run rustc -vV 2>/dev/null | sed -n 's/^host: //p')
for k in nostd_lib embassy_app rtic_app qemu_app qemu_defmt_test rtic_qemu_app embassy_qemu_app riscv_virt_app \
         bootloader_app ehal_driver async_driver protocol_lib stm32_app pico2_app nrf52840_app usb_cdc_app \
         zig_ffi_app; do
  pre=""
  case $k in
    nostd_lib) cmd="cargo test --offline --target $host" ; re='test result: ok\. 2 passed' ;;
    ehal_driver) cmd="cargo test --offline --target $host" ; re='test result: ok\. 7 passed' ;;
    async_driver) cmd="cargo test --offline --target $host" ; re='test result: ok\. 4 passed' ;;
    protocol_lib) cmd="cargo test --offline --lib --target $host" ; re='test result: ok\. 5 passed' ;;
    qemu_app|rtic_qemu_app|embassy_qemu_app|riscv_virt_app|zig_ffi_app) cmd="cargo run --offline --release" ; re='result: PASS' ;;
    qemu_defmt_test) cmd="cargo test --offline --release" ; re='all tests passed!' ;;
    stm32_app) cmd="cargo build --offline --release --features h562 --target thumbv8m.main-none-eabihf" ; re='Finished' ;;
    *) cmd="cargo build --offline --release" ; re='Finished' ;;
  esac
  if [[ $k = zig_ffi_app ]]; then
    mz=${BOX_MICROZIG:-$HERE/../32_24_microzig}
    [[ -x "$mz/microzig-box" ]] || { printf 'skip  template %s: 32_24_microzig not found (BOX_MICROZIG)\n' "$k"; continue; }
  fi
  "$Q" new "$k" "smoke_$k" --dir "$tmp" >/dev/null 2>&1 || { bad "new $k"; continue; }
  if [[ $k = zig_ffi_app ]]; then
    d="$tmp/smoke_$k/zig"
    "$mz/microzig-box" zig build-lib -target thumb-freestanding-eabi -mcpu cortex_m3 -O ReleaseSmall -fno-unwind-tables \
      --cache-dir "$d/cache" -femit-bin="$d/libzigdsp.a" "$d/dsp.zig" >/dev/null 2>&1 || { bad "template $k: zig build-lib"; continue; }
    pre="ZIG_LIB_DIR='$d'"
  fi
  if out=$("$Q" run bash -c "cd '$tmp/smoke_$k' && CARGO_NET_OFFLINE=true $pre timeout 300 $cmd" 2>&1) && grep -qE "$re" <<<"$out"; then ok "template $k: $cmd"
  else bad "template $k:"; tail -5 <<<"${out:-}" | sed 's/^/      /'; fi
done
# shellcheck disable=SC2016  # expanded inside the rust-box environment
elf=$("$Q" run sh -c 'printf %s "$CARGO_TARGET_DIR"')/thumbv6m-none-eabi/release/smoke_embassy_app
if out=$("$Q" uf2 "$elf" -o "$tmp/smoke.uf2" 2>&1) && [[ -s "$tmp/smoke.uf2" ]]; then ok "uf2: ${out##*: }"; else bad "uf2: $out"; fi
if "$Q" plan pico "$elf" | grep 'PRINTED ONLY' >/dev/null; then ok "plan prints only"; else bad "plan"; fi
echo "smoke: $pass ok, $failn failed"
[[ "$failn" -eq 0 ]]
