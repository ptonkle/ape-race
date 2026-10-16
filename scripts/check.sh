#!/usr/bin/env bash
# The full local pre-deploy check. CI runs this same file.
#
#   ./scripts/check.sh                 everything
#   BUILDER_ENGINES=chromium ./scripts/check.sh   one engine while iterating
#
# Each step prints its name first, so a failure names the step that failed.
set -euo pipefail
cd "$(dirname "$0")/.."

PY="${BUILDER_PYTHON:-}"
if [ -z "$PY" ]; then
  if [ -x .venv/bin/python ]; then PY=.venv/bin/python; else PY=python3; fi
fi

# Each step's time in seconds, printed at the end as one `TIMES` line
# (scripts/timed-checks.sh keeps them in notes/check-times.md).
T0=$(date +%s.%N); TLAST=$T0; TNAME=""; TIMES=""
lap() {
  local now; now=$(date +%s.%N)
  if [ -n "$TNAME" ]; then TIMES="$TIMES $TNAME=$(printf '%.1f' "$(echo "$now - $TLAST" | bc)")"; fi
  TLAST=$now
}
step() { lap; local n="${1#* }"; TNAME="${n%% *}"; printf '\n\033[1m== %s\033[0m\n' "$*"; }

step "1/7 format (cargo fmt --check)"
cargo fmt --all -- --check

step "2/7 lint (cargo clippy, warnings are errors)"
cargo clippy --workspace --all-targets --quiet -- -D warnings

step "3/7 core and boundary tests (cargo test)"
cargo test --workspace --quiet

step "4/7 test tags (scripts/tags.py guard: every test, step and file is classified)"
"$PY" scripts/tags.py guard

step "5/7 release WebAssembly build (scripts/build.sh)"
./scripts/build.sh >/dev/null
echo "dist/ assembled"

step "6/7 WebAssembly import audit (scripts/wasm_imports.py)"
"$PY" scripts/wasm_imports.py dist

step "7/7 browser check (tests/browser/check.py)"
"$PY" tests/browser/check.py

lap
echo "TIMES$TIMES total=$(printf '%.1f' "$(echo "$(date +%s.%N) - $T0" | bc)")"
printf '\n\033[1mBUILDER CHECK PASSED\033[0m\n'
