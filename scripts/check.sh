#!/usr/bin/env bash
# The full local pre-deploy check. CI runs this same file.
#
#   ./scripts/check.sh                 everything
#   BUILDER_ENGINES=chromium ./scripts/check.sh   one engine while iterating
#
# Each step prints its name first, so a failure names the step that failed.
# Print each browser step as it finishes, not all at the end when logged to a file.
export PYTHONUNBUFFERED=1
set -euo pipefail
cd "$(dirname "$0")/.."

PY="${BUILDER_PYTHON:-}"
if [ -z "$PY" ]; then
  if [ -x .venv/bin/python ]; then PY=.venv/bin/python; else PY=python3; fi
fi

# Each stage runs through scripts/checkrun.py, which prints its start (with
# the usual time from notes/check-times.md), one line per test binary, and
# its result and seconds; a failure prints the summary table. At the end one
# `TIMES` line (scripts/timed-checks.sh keeps it in notes/check-times.md).
export CHECK_STATE; CHECK_STATE=$(mktemp)
trap 'rm -f "$CHECK_STATE"' EXIT
"$PY" scripts/checkrun.py begin check
stage() { "$PY" scripts/checkrun.py stage "$@"; }

stage format 1/7 "format (cargo fmt --check)" plain -- cargo fmt --all -- --check
stage lint 2/7 "lint (cargo clippy, warnings are errors)" plain -- cargo clippy --workspace --all-targets --quiet -- -D warnings
stage core 3/7 "core and boundary tests (cargo test)" cargo -- cargo test --workspace
stage test 4/7 "test tags (scripts/tags.py guard)" plain -- "$PY" scripts/tags.py guard
stage release 5/7 "release WebAssembly build (scripts/build.sh)" quiet -- ./scripts/build.sh
stage WebAssembly 6/7 "WebAssembly import audit" wasm -- "$PY" scripts/wasm_imports.py dist
stage browser 7/7 "browser check (tests/browser/check.py)" plain -- "$PY" tests/browser/check.py

"$PY" scripts/checkrun.py summary
"$PY" scripts/checkrun.py times
printf '\n\033[1mBUILDER CHECK PASSED\033[0m\n'
