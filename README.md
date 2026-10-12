# A Builder tool

Replace this file with your tool's README once `BRIEF.md` is written.

This directory is a copy of the Builder template. Its rules are in `BUILDER-SPEC.md`, and a coding agent's instructions are in `CLAUDE.md` and `AGENTS.md`.

## Build, test and serve

```bash
./scripts/setup.sh          # once: Python venv with Playwright and three browsers
cargo test --workspace      # the core and boundary tests, in seconds
./scripts/build.sh          # release WebAssembly build into dist/
python3 scripts/serve.py    # http://127.0.0.1:8000/
./scripts/check.sh          # everything, before a deploy
```

Needs Rust with the `wasm32-unknown-unknown` target, `wasm-bindgen-cli` 0.2.127, and Python 3.

## Layout

| Path | Holds |
| --- | --- |
| `crates/core` | every application rule; no input or output |
| `crates/shim` | the `wasm-bindgen` functions the page calls; each is one call into the core |
| `web/` | the static page |
| `data/` | reader-visible text and content |
| `tests/browser/` | the three-engine browser check and this tool's interactions |
| `scripts/` | build, check, serve, import audit, mutation audit |
