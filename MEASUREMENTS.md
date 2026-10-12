# Measurements

Stage 4, recorded by the agent before building. `B2 Require step-by-step justification`.

The starting state, measured rather than recalled, so later changes can be compared with it.

| What | Command | Result | When |
| --- | --- | --- | --- |
| Toolchain | `rustc --version; wasm-bindgen --version` | rustc 1.99.0 (b940084d7 2026-09-28); wasm-bindgen 0.2.127; targets wasm32-unknown-unknown, x86_64-unknown-linux-gnu | 2026-10-11, before any code |
| Core tests | `cargo test --workspace` | template: passes (placeholder tests only) | 2026-10-11, before any code |
| Full check | `./scripts/check.sh` | not run on the bare template; first run recorded after the first game build | 2026-10-11 |
