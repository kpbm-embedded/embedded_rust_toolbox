#!/usr/bin/env bash
# Every rust-box command in --dry-run mode (no downloads, builds, moves or device access),
# plus shellcheck of every script. Usage: tests/test_dry_run.sh
set -uo pipefail
HERE=$(cd "$(dirname "$0")/.." && pwd -P)
Q="$HERE/rust-box"
pass=0; failn=0
check() { # name expected-regex cmd...
  local name=$1 re=$2; shift 2
  local out rc
  out=$(timeout 300 "$@" 2>&1); rc=$?
  if [[ "$rc" -eq 0 ]] && grep -qE -- "$re" <<<"$out"; then
    printf 'ok    %s\n' "$name"; pass=$((pass+1))
  else
    printf 'FAIL  %s (rc=%s)\n%s\n' "$name" "$rc" "$(tail -5 <<<"$out" | sed 's/^/      /')"; failn=$((failn+1))
  fi
}
check help        'usage: \./rust-box'                         "$Q" help
check where       '^probe-rs '                                 "$Q" where
check bootstrap   'rustup toolchain install|pixi .*install'    "$Q" --dry-run bootstrap
check install     'example\.sh build'                          "$Q" --dry-run install
check update      'curl https://api.github.com/repos/probe-rs/probe-rs' "$Q" --dry-run update
check update-env  '\+ timeout .* update'                       "$Q" --dry-run update --env
check clean       'nothing to clean|dry run: re-run with --yes' "$Q" clean
check clean-all   'dry run: re-run with --yes'                 "$Q" clean --all
check status      '^  toolchain '                              "$Q" status
check doctor      'doctor: [0-9]+ ok'                          bash -c "'$Q' doctor || true"
check shell       '\+ true'                                    "$Q" --dry-run shell -- true
check run         '\+ rustc --version'                         "$Q" --dry-run run rustc --version
check test        'test_smoke\.sh'                             "$Q" --dry-run test
check new         '\+ copy .*templates/embassy_app'            "$Q" --dry-run new embassy_app demo --dir /nonexistent
check ex-list     'embassy_pico_blinky .*Embassy'              "$Q" examples list
check ex-run      'example\.sh run'                            "$Q" --dry-run examples run embassy_pico_blinky
check ex-build    'example\.sh build'                          "$Q" --dry-run examples build
check ex-test     'example\.sh test'                           "$Q" --dry-run examples test
check native      'ecargo @cargo'                              "$Q" --dry-run native
check relocate    'pixi env already matches|re-creating'       "$Q" --dry-run relocate
check pack        '\+ tar .*-cf .*rust-box_'                   "$Q" --dry-run pack
check cargo       '\+ cargo build --release'                   "$Q" --dry-run cargo build --release
check rustup      '\+ rustup show'                             "$Q" --dry-run rustup show
check boards      'blackpill_f411 .*STM32F411CE'               "$Q" boards
check plan-pico   'probe-rs run --chip RP2040 fw\.elf'         "$Q" plan pico fw.elf
check plan-uf2    'cp fw\.uf2 /run/media'                      "$Q" plan pico fw.elf
check plan-f411   'probe-rs run --chip STM32F411CE x\.elf'     "$Q" plan blackpill_f411 x.elf
check plan-dfu    'dfu-util -a 0 -s 0x08000000'                "$Q" plan blackpill_f411 x.elf
check uf2         '\+ python -I lib/uf2conv\.py fw\.elf'       "$Q" --dry-run uf2 fw.elf
check probe-rs    '\+ probe-rs list'                           "$Q" --dry-run probe-rs list
check flip-link   '\+ flip-link --help'                        "$Q" --dry-run flip-link --help
check defmt-print '\+ defmt-print --help'                      "$Q" --dry-run defmt-print --help
check cargo-tool  'cargo install --locked .*cargo-generate@'   "$Q" --dry-run cargo-tool cargo-generate
check cargo-tools 'defmt-print +1\.2\.0'                       "$Q" cargo-tool
check offline     'example.sh build'                           "$Q" --dry-run offline-check
check uf2-usage   'usage: uf2conv'                             bash -c "python3 -I '$HERE/lib/uf2conv.py' 2>&1; true"
if command -v shellcheck >/dev/null; then
  mapfile -t files < <(cd "$HERE" && { echo rust-box; ls lib/*.bash tests/*.sh examples/*/*.sh examples/common/*.bash; })
  if out=$(cd "$HERE" && shellcheck -x "${files[@]}" 2>&1); then printf 'ok    shellcheck (%s files)\n' "${#files[@]}"; pass=$((pass+1))
  else printf 'FAIL  shellcheck\n%s\n' "$(head -30 <<<"$out")"; failn=$((failn+1)); fi
else printf 'skip  shellcheck (not installed)\n'; fi
echo "dry-run: $pass ok, $failn failed"
[[ "$failn" -eq 0 ]]
