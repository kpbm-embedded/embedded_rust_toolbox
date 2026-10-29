# shellcheck shell=bash disable=SC2015,SC2034  # d_* return 0; TOOL_* are read by boxtool.bash
# rust-box: tool part (hooks for lib/boxtool.bash). A pinned Rust toolchain from rustup with
# RUSTUP_HOME and CARGO_HOME inside this ID (never ~/.rustup, ~/.cargo), the targets of the
# user's boards (rust-toolchain.toml: Cortex-M0+/M4/M33, RISC-V for Pico 2 and ESP32-C3), probe-rs,
# flip-link and xPack QEMU (pinned release binaries, qemu for the examples that run without a board),
# defmt-print (cargo install --locked, pinned), and a small pixi env (host C linker + Python for lib/uf2conv.py).
# SAFETY: nothing here flashes, erases, resets or writes a target. `plan` only prints commands,
# `uf2` only writes a .uf2 file, examples `run` builds and prints the plan.
TOOL_TITLE="32_17_embedded_rust: Rust $(sed -n 's/^channel = "\(.*\)"/\1/p' "${BOXT_ROOT:?}/rust-toolchain.toml") (rustup, in-ID) + Embassy, defmt, RTIC v2, probe-rs $(sed -n 's/^probe-rs\t\([^\t]*\)\t.*/\1/p' "$BOXT_ROOT/config/pins.tsv" | head -1)"
TOOL_INSTALL_WHAT="cargo tools (config/cargo-tools.tsv) + the example build (examples build)"
TOOL_HELP_USE="  cargo [args]            the pinned cargo (RUSTUP_HOME/CARGO_HOME in this ID, target dir on \$OUT)
  rustup [args]           the in-ID rustup (toolchains under install/rustup)
  boards                  the user's boards: target triple, probe-rs chip, UF2 family (config/boards.tsv)
  plan <board> <fw.elf>   PRINT flash/run commands (probe-rs, UF2 copy, DFU); runs nothing
  uf2 <fw.elf> [-o F]     ELF -> UF2 file for RP2040 BOOTSEL drag-and-drop (writes a file only)
  probe-rs | flip-link | defmt-print [args]   the pinned tools (pass-through: runs what you type)
  cargo-tool <name>       cargo install --locked an optional tool from config/cargo-tools.tsv
  offline-check           cargo build --offline of every example (registry cache in ./cache/cargo)"
hook_help() { # printed after boxtool's help: tool-specific examples and environment
  cat <<EOF

tool examples
  ./rust-box plan pico \$OUT/build/examples/embassy_pico_blinky/embassy_pico_blinky.elf
  ./rust-box new embassy_app my_fw --dir ~/src && ./rust-box run bash -c 'cd ~/src/my_fw && cargo build --release'
tool environment
  RUST_BOX_TARGET_DIR     cargo target dir (default \$OUT/build/cargo-target)
  BOX_MICROZIG            32_24_microzig (Zig) for the Zig interop example/template
                          (default: the sibling folders; the examples skip with exit 3 when missing)
  RUSTUP_HOME, CARGO_HOME are always set to install/rustup and cache/cargo inside this folder
EOF
}
# shellcheck source-path=SCRIPTDIR source=boxtool.bash
. "$BOXT_ROOT/lib/boxtool.bash"

BOARDS="$CONFIG/boards.tsv"
sibling_id() { # ENV_VAR FOLDER MARKER -> another tool ID: the variable's folder (or one above it) that holds
  # MARKER, else ../FOLDER (or nothing). Used only to REACH other IDs (Zig); nothing is copied.
  local d=${!1:-} i
  for i in 1 2 3; do
    [[ -n "$d" ]] && [[ -e "$d/$3" ]] && { (cd -P "$d" && pwd -P); return 0; }
    d=${d%/*}
  done
  if [[ -e "$ROOT/../$2/$3" ]]; then (cd -P "$ROOT/../$2" && pwd -P); fi
  true
}
MICROZIG_ID=$(sibling_id BOX_MICROZIG 32_24_microzig microzig-box)
CTOOLS="$CONFIG/cargo-tools.tsv"
export RUSTUP_HOME="$INST/rustup"
export CARGO_HOME="$ROOT/cache/cargo"
CT_ROOT="$INST/cargo-tools"
TOOLCHAIN=$(sed -n 's/^channel = "\(.*\)"/\1/p' "$ROOT/rust-toolchain.toml")
EXE=""; case "$PLATFORM" in win-*) EXE=.exe ;; esac

strip_home_cargo() { # drop ~/.cargo/bin (host rustup proxies) from PATH
  local out="" p IFS=:
  for p in $PATH; do
    [[ "$p" = "$HOME/.cargo/bin" ]] && continue
    out="${out:+$out:}$p"
  done
  PATH=$out
}

hook_env() {
  strip_home_cargo
  local p
  for p in "$(llvm_bin)" "$INST/flip-link" "$INST/probe-rs" "$INST/qemu-arm/bin" "$INST/qemu-riscv/bin" "$CT_ROOT/bin" "$CARGO_HOME/bin"; do
    [[ -d "$p" ]] && PATH="$p:$PATH"
  done
  export PATH RUSTUP_HOME CARGO_HOME
  export CARGO_BUILD_JOBS="$JOBS" CARGO_TERM_COLOR="${CARGO_TERM_COLOR:-auto}"
  export CARGO_TARGET_DIR="${RUST_BOX_TARGET_DIR:-$BUILD/cargo-target}"
  # host linking (cargo test, build scripts) through the env's C compiler, not the system one
  if [[ -n "${CC:-}" ]] && have "$CC"; then
    local triple
    triple=$(rustc_host)
    [[ -n "$triple" ]] && export "CARGO_TARGET_$(tr '[:lower:]-' '[:upper:]_' <<<"$triple")_LINKER=$CC"
  fi
  # probe-rs / defmt-print config and caches stay in the ID
  export XDG_CONFIG_HOME="$ROOT/cache/xdg-config" XDG_CACHE_HOME="$ROOT/cache/xdg-cache" XDG_DATA_HOME="$ROOT/cache/xdg-data"
  [[ "$DRY" = 1 ]] || mkdir -p "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME" "$XDG_DATA_HOME" 2>/dev/null || true
  export RUST_BOX_ROOT="$ROOT" RUST_BOX_LIB="$ROOT/lib" RUST_BOX_BOARDS="$BOARDS" MICROZIG_ID
}
rustc_host() { # host triple of the pinned toolchain (empty before bootstrap)
  case "$PLATFORM" in
    linux-x64) echo x86_64-unknown-linux-gnu ;; linux-arm64) echo aarch64-unknown-linux-gnu ;;
    darwin-x64) echo x86_64-apple-darwin ;; darwin-arm64) echo aarch64-apple-darwin ;;
    win-x64) echo x86_64-pc-windows-msvc ;; *) echo "" ;;
  esac
}
toolchain_targets() { # the targets listed in rust-toolchain.toml (one per line)
  sed -n '/^targets *=/,/]/p' "$ROOT/rust-toolchain.toml" | grep -o '"[^"]*"' | tr -d '"'
}
llvm_bin() { printf '%s/toolchains/%s-%s/lib/rustlib/%s/bin' "$RUSTUP_HOME" "$TOOLCHAIN" "$(rustc_host)" "$(rustc_host)"; }

# ---------- bootstrap: rustup + pinned toolchain, then cargo tools ----------
hook_bootstrap() {
  cachedir_tag "$ROOT/cache" "$CARGO_HOME"
  local ri; ri=$(pin_file rustup-init)
  if [[ ! -x "$CARGO_HOME/bin/rustup$EXE" ]]; then
    info "rustup-init (RUSTUP_HOME=${RUSTUP_HOME#"$ROOT"/}, CARGO_HOME=${CARGO_HOME#"$ROOT"/}; no PATH/profile edits)"
    run chmod +x "$ri"
    run "$ri" -y --no-modify-path --no-update-default-toolchain --profile minimal --default-toolchain none
  fi
  activate
  run rustup set auto-self-update disable
  run rustup set profile minimal
  info "toolchain $TOOLCHAIN from rust-toolchain.toml (+ targets, components)"
  (cd "$ROOT" && run timeout "$NET_TIMEOUT" rustup toolchain install)
  run rustup default "$TOOLCHAIN"
  install_cargo_tools core
}

ct_rows() { [[ -r "$CTOOLS" ]] && awk -F'\t' '!/^#/ && NF>=3' "$CTOOLS"; true; }
install_cargo_tools() { # group
  local name ver grp
  while IFS=$'\t' read -r name ver grp _; do
    [[ "$grp" = "$1" ]] || continue
    cargo_tool_install "$name" "$ver"
  done < <(ct_rows)
}
cargo_tool_install() { # name version
  local n=$1 v=$2 mark="$CT_ROOT/.$1-$2.ok"
  [[ -e "$mark" ]] && return 0
  wait_for_ram
  info "cargo install --locked $n@$v -> ${CT_ROOT#"$ROOT"/} (build dir on \$OUT)"
  run env CARGO_TARGET_DIR="$BUILD/cargo-install" timeout "$NET_TIMEOUT" cargo install --locked --root "$CT_ROOT" "$n@$v"
  [[ "$DRY" = 1 ]] || { cachedir_tag "$INST"; printf 'name=%s\nversion=%s\ndate=%s\n' "$n" "$v" "$(date -Iseconds)" >"$mark"; }
}

hook_install() { # demos-flag
  info "building the examples (examples build)"
  cmd_examples build
}

hook_status() {
  local e n=0
  for e in $(ex_list); do [[ -n "$(find "$BUILD/examples/$e" -maxdepth 1 -type f -name '*.elf' 2>/dev/null | head -1)" ]] && n=$((n+1)); done
  printf '  %-22s %s\n' toolchain "$TOOLCHAIN $( [[ -d "$RUSTUP_HOME/toolchains" ]] && human_size "$RUSTUP_HOME" || echo 'not installed')"
  printf '  %-22s %s\n' cargo-home "$(human_size "$CARGO_HOME") (registry cache)"
  printf '  %-22s %s\n' cargo-target "$(human_size "$BUILD/cargo-target") on \$OUT"
  printf '  %-22s %s of %s built in %s\n' examples "$n" "$(ex_list | wc -l)" "$BUILD/examples"
}

hook_doctor() {
  local v t
  v=$(command -v cargo) && [[ "${v#"$CARGO_HOME"/}" != "$v" ]] && d_ok "cargo is the in-ID proxy (${v#"$ROOT"/})" || d_fail "cargo on PATH is not from the ID: ${v:-none}"
  v=$(rustc --version 2>/dev/null) && grep -q " ${TOOLCHAIN} " <<<"$v " && d_ok "$v (pinned $TOOLCHAIN)" || d_fail "rustc ${v:-missing} != pinned $TOOLCHAIN (./$NAME bootstrap)"
  v=$(cargo --version 2>/dev/null) && d_ok "$v" || d_fail "cargo not runnable"
  for t in $(toolchain_targets); do
    rustup target list --installed 2>/dev/null | grep -x "$t" >/dev/null && d_ok "target $t" || d_fail "target $t missing (./$NAME bootstrap)"
  done
  [[ -x "$(llvm_bin)/llvm-objcopy$EXE" ]] && d_ok "llvm-tools (llvm-objcopy, llvm-size, llvm-nm)" || d_fail "llvm-tools component missing"
  flip-link --help 2>&1 | grep 'stack overflow protection' >/dev/null && d_ok "flip-link $(pin_get flip-link 2) (linker wrapper; rust-lld comes from the toolchain at link time)" || d_fail "flip-link not runnable"
  v=$(probe-rs --version 2>/dev/null | head -1) && d_ok "$v" || d_fail "probe-rs not runnable"
  for t in qemu-system-arm qemu-system-riscv32; do
    v=$("$t" --version 2>/dev/null | head -1) && d_ok "$t: ${v#QEMU emulator }" || d_fail "$t not runnable (./$NAME bootstrap)"
  done
  v=$(defmt-print --version 2>/dev/null | head -1) && d_ok "$v" || d_fail "defmt-print missing (./$NAME bootstrap)"
  if [[ "$PLATFORM" != win-x64 ]]; then
    [[ -n "${CC:-}" ]] && have "$CC" && d_ok "host linker $(basename "$CC") ($("$CC" -dumpversion 2>/dev/null)) from the pixi env" || d_fail "host C compiler missing in the pixi env"
  fi
  v=$(python -c 'import sys; print(sys.version.split()[0])' 2>/dev/null) && d_ok "python $v (uf2conv)" || d_fail "python missing in the pixi env"
  [[ -r "$BOARDS" ]] && d_ok "boards table: $(board_rows | wc -l) boards" || d_fail "config/boards.tsv missing"
  [[ -n "$(find "$CARGO_HOME/registry/cache" -name '*.crate' 2>/dev/null | head -1)" ]] && d_ok "cargo registry cache: $(find "$CARGO_HOME/registry/cache" -name '*.crate' | wc -l) crates (offline rebuilds)" || d_warn "cargo registry cache empty (first examples build fetches crates)"
  [[ -e "$HOME/.cargo/bin/cargo" ]] && d_ok "host ~/.cargo exists and is ignored (stripped from PATH)" || true
  local rules=/etc/udev/rules.d
  [[ -e "$rules/69-probe-rs.rules" ]] || [[ -e /usr/lib/udev/rules.d/69-probe-rs.rules ]] && d_ok "probe-rs udev rules present" || d_warn "probe-rs udev rules not installed (README 'udev': sudo commands); plan output needs them"
}

hook_native_bins() {
  echo "rust-box @"
  echo "ecargo @cargo"
  echo "flip-link install/flip-link/flip-link$EXE"
  echo "defmt-print install/cargo-tools/bin/defmt-print$EXE"
  echo "rb-probe-rs install/probe-rs/probe-rs$EXE"
}

hook_new() { # kind name dst
  local dst=$3
  [[ "$DRY" = 1 ]] && return 0
  info "toolchain pinned in $dst/rust-toolchain.toml ($TOOLCHAIN); build: ./$NAME cargo build --manifest-path $dst/Cargo.toml"
}

# ---------- boards / plan / uf2 ----------
board_rows() { [[ -r "$BOARDS" ]] && awk -F'\t' '!/^#/ && $1!="board" && NF>=5' "$BOARDS"; true; }
board_get() { board_rows | awk -F'\t' -v b="$1" '$1==b'; }
tool_cmd_boards() {
  { printf 'board\tchip\ttarget\tprobe-rs\tuf2-family\tnotes\n'; board_rows; } | column -t -s "$(printf '\t')"
}

tool_cmd_plan() {
  local b=${1:-} fw=${2:-}
  [[ -n "$b" ]] && [[ -n "$fw" ]] || usage_error "usage: ./$NAME plan <board> <firmware.elf> (boards: ./$NAME boards)"
  local row; row=$(board_get "$b")
  [[ -n "$row" ]] || usage_error "unknown board '$b' (./$NAME boards)"
  local chip target prs fam notes
  chip=$(cut -f2 <<<"$row"); target=$(cut -f3 <<<"$row"); prs=$(cut -f4 <<<"$row"); fam=$(cut -f5 <<<"$row"); notes=$(cut -f6 <<<"$row")
  echo "# flash/run plan for $b ($chip, $target): PRINTED ONLY, nothing was run."
  echo "# notes: ${notes:--}"
  [[ -e "$fw" ]] || echo "# warning: $fw does not exist yet (./$NAME examples build)"
  echo "# probe-rs over SWD (needs the udev rules, README 'udev'); defmt logs are decoded live:"
  echo "./$NAME probe-rs run --chip $prs $fw"
  echo "./$NAME probe-rs download --chip $prs $fw     # flash only"
  echo "./$NAME probe-rs attach --chip $prs $fw       # RTT/defmt of a running target"
  echo "./$NAME probe-rs gdb --chip $prs              # GDB server on :1337"
  if [[ "$fam" != - ]]; then
    local drive=RPI-RP2 how="hold BOOTSEL while plugging USB"
    case "$fam" in
      rp2350*) drive=RP2350 ;;
      nrf52840) drive=NICENANO how="double-tap RST" ;;
    esac
    echo "# no probe: UF2 drag-and-drop ($how; the $drive drive appears)"
    echo "./$NAME uf2 $fw -o ${fw%.elf}.uf2 --family $fam   # writes a file only"
    echo "cp ${fw%.elf}.uf2 /run/media/\$USER/$drive/   # this copy flashes the board"
  fi
  case "$chip" in STM32*)
    echo "# no probe: USB DFU ROM bootloader (hold BOOT0, tap NRST); dfu-util is not bundled"
    echo "./$NAME run llvm-objcopy -O binary $fw ${fw%.elf}.bin   # writes a file only"
    echo "dfu-util -a 0 -s 0x08000000:leave -D ${fw%.elf}.bin" ;;
  esac
  return 0
}

tool_cmd_uf2() {
  [[ $# -ge 1 ]] || usage_error "usage: ./$NAME uf2 <fw.elf> [-o out.uf2] [--family rp2040]"
  activate
  [[ "$DRY" = 1 ]] && { printf '+ python -I lib/uf2conv.py %s\n' "$*"; return 0; }
  exec python -I "$ROOT/lib/uf2conv.py" "$@"
}

tool_cmd_cargo() { activate; [[ "$DRY" = 1 ]] && { printf '+ cargo %s\n' "$*"; return 0; }; exec cargo "$@"; }
tool_cmd_rustup() { activate; [[ "$DRY" = 1 ]] && { printf '+ rustup %s\n' "$*"; return 0; }; exec rustup "$@"; }
tool_cmd_probe-rs() { activate; [[ "$DRY" = 1 ]] && { printf '+ probe-rs %s\n' "$*"; return 0; }; exec probe-rs "$@"; }
tool_cmd_flip-link() { activate; [[ "$DRY" = 1 ]] && { printf '+ flip-link %s\n' "$*"; return 0; }; exec flip-link "$@"; }
tool_cmd_defmt-print() { activate; [[ "$DRY" = 1 ]] && { printf '+ defmt-print %s\n' "$*"; return 0; }; exec defmt-print "$@"; }
tool_cmd_cargo-tool() {
  local n=${1:-} row
  [[ -n "$n" ]] || { echo "cargo tools (config/cargo-tools.tsv):"; ct_rows | column -t -s "$(printf '\t')"; return 0; }
  row=$(ct_rows | awk -F'\t' -v n="$n" '$1==n')
  [[ -n "$row" ]] || usage_error "no cargo tool '$n' in config/cargo-tools.tsv"
  activate
  cargo_tool_install "$n" "$(cut -f2 <<<"$row")"
}
tool_cmd_offline-check() {
  activate
  local ex rc=0 r
  export CARGO_NET_OFFLINE=true
  for ex in $(ex_list); do
    [[ -r "$ROOT/examples/$ex/Cargo.toml" ]] || continue
    # each example's own `build` (its target, features, linker; cargo reads .cargo/config.toml from the
    # example folder), with the network off: everything must come from the registry cache in cache/cargo
    if [[ "$DRY" = 1 ]]; then ex_call "$ex" build; continue; fi
    r=0; ex_call "$ex" build >/dev/null 2>&1 || r=$?
    case $r in
      0) echo "  ok    $ex (offline)" ;;
      3) echo "  skip  $ex (needs another ID)" ;;
      *) echo "  FAIL  $ex (offline)"; rc=1 ;;
    esac
  done
  return "$rc"
}

boxt_main "$@"
