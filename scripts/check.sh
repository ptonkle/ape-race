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

step() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

step "1/6 format (cargo fmt --check)"
cargo fmt --all -- --check

step "2/6 lint (cargo clippy, warnings are errors)"
cargo clippy --workspace --all-targets --quiet -- -D warnings

step "3/6 core and boundary tests (cargo test)"
cargo test --workspace --quiet

step "4/6 release WebAssembly build (scripts/build.sh)"
./scripts/build.sh >/dev/null
echo "dist/ assembled"

step "5/6 WebAssembly import audit (scripts/wasm_imports.py)"
"$PY" scripts/wasm_imports.py dist

step "6/6 browser check (tests/browser/check.py)"
"$PY" tests/browser/check.py

printf '\n\033[1mBUILDER CHECK PASSED\033[0m\n'
