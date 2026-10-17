# Measurements

Stage 4, recorded by the agent before building. `B2 Require step-by-step justification`.

The starting state, measured rather than recalled, so later changes can be compared with it.

| What | Command | Result | When |
| --- | --- | --- | --- |
| Toolchain | `rustc --version; wasm-bindgen --version` | rustc 1.99.0 (b940084d7 2026-09-28); wasm-bindgen 0.2.127; targets wasm32-unknown-unknown, x86_64-unknown-linux-gnu | 2026-10-11, before any code |
| Core tests | `cargo test --workspace` | template: passes (placeholder tests only) | 2026-10-11, before any code |
| Full check | `./scripts/check.sh` | not run on the bare template; first run recorded after the first game build | 2026-10-11 |
| Core frame cost | `cargo run --release --example frame_cost` | about 0.1 ms per frame (advance + view model + frame), 4 apes | 2026-10-11 |
| Full check | `./scripts/check.sh` | BUILDER CHECK PASSED (chromium, firefox, webkit) | 2026-10-11, first deploy |
| Live site | `BUILDER_ORIGIN=https://ptonkle.github.io/ape-race tests/browser/check.py` | BROWSER CHECK PASSED in three engines | 2026-10-11 |
| Mape's Tower v2: the tube's air (TW2-5) | `cargo test -p tool_core --lib a_weave_steered -- --nocapture` | terminal 39.8 m/s (TUBE_TERMINAL 38, settle 8/s), air ×3, lead 0.3 s: 22 pads, gaps weaving ±7 m over 12 pads, worst miss 1.8 m, 0 bounces, ~7.5 s to the sling | 2026-10-16, skeleton |
