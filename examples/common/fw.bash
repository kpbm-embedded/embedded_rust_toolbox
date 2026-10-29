# shellcheck shell=bash
# Shared helpers for the firmware examples (sourced by examples/<ex>/example.sh, which runs
# inside ./rust-box examples ... with the pinned toolchain active). Nothing here touches a device:
# `run` builds and PRINTS the flash plan.
OUT=${EX_OUT:?run through ./rust-box examples}
LAUNCH="${BOXT_ROOT:?}/rust-box"

# fw_build <bin-name> [cargo args...]: release build, copy the ELF to $OUT/<bin>.elf
fw_build() {
  local bin=$1; shift
  cargo build --release --bin "$bin" "$@"
  local target
  target=$(sed -n 's/^target *= *"\(.*\)".*/\1/p' .cargo/config.toml | head -1)
  cp "$CARGO_TARGET_DIR/$target/release/$bin" "$OUT/$bin.elf"
  echo "built $OUT/$bin.elf"
}

# fw_check <elf> <flash-origin-hex> <ram-origin-hex>: static checks of a firmware ELF (no device):
# vector table at the flash origin, defmt table present, flip-link moved the stack below .bss/.data.
sec_addr() { llvm-readelf -S "$1" | sed -n "s/.*\] $2 *[A-Z_]* *\([0-9a-f]*\) .*/\1/p" | head -1; }
fw_check() {
  local elf=$1 flash=$2 ram=$3 vt sz
  [[ -s "$elf" ]] || { echo "FAIL: $elf missing"; return 1; }
  vt=$(sec_addr "$elf" '\.vector_table')
  [[ "$((16#${vt:-0}))" = "$((flash))" ]] || { echo "FAIL: .vector_table at 0x${vt:-?}, expected $flash"; return 1; }
  grep -q ' \.defmt ' <<<"$(llvm-readelf -S "$elf")" || { echo "FAIL: no .defmt section"; return 1; }
  # flip-link: the initial stack pointer (first vector word) sits below .data/.bss, not at the RAM end
  local sp data
  sp=$(llvm-objdump -s -j .vector_table "$elf" | grep -m1 -E '^ +[0-9a-f]+ [0-9a-f]{8} ' | awk '{print $2}')
  sp=$(printf '%s' "$sp" | sed 's/\(..\)\(..\)\(..\)\(..\)/\4\3\2\1/')
  data=$(sec_addr "$elf" '\.data')
  if [[ -n "$data" ]] && [[ "$((16#$sp))" -le "$((16#$data))" ]] && [[ "$((16#$sp))" -ge "$((ram))" ]]; then
    echo "flip-link: initial SP 0x$sp <= .data 0x$data (stack overflow faults instead of corrupting RAM)"
  else echo "FAIL: flip-link layout (SP 0x$sp, .data 0x${data:-?})"; return 1; fi
  sz=$(llvm-size "$elf" | tail -1)
  echo "size: $sz"
  echo "check ok: $(basename "$elf")"
}

# fw_plan <board> <elf>: print (never run) how to flash it
fw_plan() { "$LAUNCH" plan "$1" "$2"; }

# elf_vt <elf> <origin-hex>: .vector_table at the flash origin and a reset vector inside it (no defmt or
# flip-link requirement; used by the qemu and multi-board examples). Prints the size line.
elf_vt() {
  local elf=$1 origin=$2 vt reset
  [[ -s "$elf" ]] || { echo "FAIL: $elf missing"; return 1; }
  vt=$(sec_addr "$elf" '\.vector_table')
  [[ "$((16#${vt:-0}))" = "$((origin))" ]] || { echo "FAIL: .vector_table at 0x${vt:-?}, expected $origin"; return 1; }
  reset=$(llvm-objdump -s -j .vector_table "$elf" | grep -m1 -E '^ +[0-9a-f]+ ' | awk '{print $3}' | sed 's/\(..\)\(..\)\(..\)\(..\)/\4\3\2\1/')
  [[ "$((16#$reset & ~1))" -ge "$((origin))" ]] || { echo "FAIL: reset vector 0x$reset below $origin"; return 1; }
  echo "vector table at $origin, reset 0x$reset; size: $(llvm-size "$elf" | tail -1)"
}

# qemu_arm <machine> <elf> <log> [seconds]: run a Cortex-M ELF on qemu-system-arm (pinned xPack QEMU, in
# install/qemu-arm) with semihosting; output to <log> and stdout. Returns qemu's exit code (semihosting exit).
qemu_arm() {
  local m=$1 elf=$2 log=$3 t=${4:-30} cpu=() rc=0
  [[ "$m" = lm3s6965evb ]] && cpu=(-cpu cortex-m3)
  echo "+ qemu-system-arm ${cpu[*]} -machine $m -nographic -semihosting-config enable=on,target=native -kernel $(basename "$elf")"
  timeout "$t" qemu-system-arm "${cpu[@]}" -machine "$m" -nographic -monitor none \
    -semihosting-config enable=on,target=native -kernel "$elf" >"$log" 2>&1 || rc=$?
  cat "$log"
  return "$rc"
}

# expect <log> <fixed-string>...: every string must appear in the log
expect() {
  local log=$1 s; shift
  for s in "$@"; do grep -qF -- "$s" "$log" || { echo "FAIL: '$s' not in $(basename "$log")"; return 1; }; done
}
