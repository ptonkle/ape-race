#!/usr/bin/env bash
# Build the release WebAssembly module and assemble the static site in dist/.
# Needs: Rust with the wasm32-unknown-unknown target, and the wasm-bindgen CLI
# at the version pinned in crates/shim/Cargo.toml.
set -euo pipefail
cd "$(dirname "$0")/.."

PIN=$(sed -n 's/^wasm-bindgen = "=\(.*\)"/\1/p' crates/shim/Cargo.toml)
HAVE=$(wasm-bindgen --version 2>/dev/null | awk '{print $2}')
if [ "$HAVE" != "$PIN" ]; then
  echo "wasm-bindgen CLI is '${HAVE:-missing}', the shim pins $PIN." >&2
  echo "Install it with: cargo install wasm-bindgen-cli --version $PIN --locked" >&2
  exit 1
fi

cargo build --release --target wasm32-unknown-unknown -p tool_shim

rm -rf dist
mkdir -p dist/pkg dist/data
wasm-bindgen --target web --no-typescript --out-dir dist/pkg \
  target/wasm32-unknown-unknown/release/tool_shim.wasm
cp web/* dist/
cp data/* dist/data/

echo "Built dist/:"
find dist -type f | sort
