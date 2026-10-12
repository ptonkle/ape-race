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
find web -maxdepth 1 -type f -exec cp {} dist/ \;
cp -r data/. dist/data/

# Third-party code for online play (PLAN P13): the Trystero bundle is not a
# crate and no lockfile pins it, so web/vendor/SHA256SUMS does. A file that
# was edited, replaced or added without a new pin fails the build.
mkdir -p dist/vendor
(cd web/vendor && sha256sum --check --strict --quiet SHA256SUMS) || {
  echo "web/vendor/ does not match web/vendor/SHA256SUMS; do not edit vendored files." >&2
  exit 1
}
for f in web/vendor/*; do
  name=$(basename "$f")
  if [ "$name" != SHA256SUMS ] && ! grep -q "  $name\$" web/vendor/SHA256SUMS; then
    echo "web/vendor/$name is not pinned in web/vendor/SHA256SUMS." >&2
    exit 1
  fi
done
cp web/vendor/* dist/vendor/
(cd dist/vendor && sha256sum --check --strict --quiet SHA256SUMS)

echo "Built dist/:"
find dist -type f | sort
