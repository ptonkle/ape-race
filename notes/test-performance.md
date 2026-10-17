# Test performance: what we measured and what is left

Written 2026-10-17, after the 1.15.0 (Rattlesnake Ravine) release checks. Times are this machine (16 cores, WSL2, AMD Radeon). The check-time history is in `notes/check-times.md`; which tests a change runs is in `notes/quick-check.md`.

## Where a full check's time went (idle machine, before the GPU fix)

| Part | Time | Runs in parallel? |
| --- | --- | --- |
| Core tests (`cargo test`) | 85 s | Tests inside a test file do; the ~20 test files run one after another (the AI files take up to 27 s each) |
| Browser check, 3 browsers × 11 steps | 280 s | No: Chromium, then Firefox, then WebKit; steps one after another |
| Online checks | 128 s | No, and only after everything else |
| Format, lint, build, import audit | ~11 s | Not worth changing |

## Chromium was the slow browser (fixed, 2622b8c)

Same build and steps, release rerun at load ~8:

| Step | Chromium (before) | Firefox | WebKit |
| --- | --- | --- | --- |
| settings (has a page reload) | 10.6 s idle, 36 s loaded | 1.8 s | 3.4 s |
| race | 31.6 s idle, 89 s loaded | 8.7 s | 11.2 s |
| all 11 steps | ~93 s idle, ~340 s loaded | ~36 s | ~55 s |

- **Cause:** Playwright's headless Chromium drew WebGL in software (SwiftShader), about 6.6 frames a second. Page load was never the problem (about 1 s everywhere); every step that plays the game crawled.
- **Not the cause:** the page reload (the person opens the game fast in their own browser; reload took 1 s in the probe).
- **What didn't work:** full Chromium in new headless mode (still SwiftShader); headless with GPU flags (`--use-angle=gl`, `gl-egl`, EGL surfaceless): the WebGL context is lost and screenshots fail. A frame-rate probe on a lost context reads 50+ fps while drawing nothing: confirm drawing (readPixels colours, no `CONTEXT_LOST_WEBGL`) before trusting a speed number. Vulkan gives llvmpipe (CPU), 12 fps.
- **What works:** a windowed Chromium on an invisible Xvfb display, ANGLE on OpenGL through Mesa's d3d12 driver (`GALLIUM_DRIVER=d3d12`, passed in the launch `env` because Playwright's driver starts first) → the Radeon at 60 fps (vsync). Xvfb needs a named display (`:99`+): WSLg mounts `/tmp/.X11-unix` read-only. A window on the WSLg display shows on the person's screen: never do that. All in `tests/browser/engines.py`; falls back to headless without Xvfb or `/dev/dxg` (CI).
- **The drawn check** (`canvas_drawn`) now samples inside an animation frame: on a GPU the drawing buffer is cleared once a frame is shown.
- **Result:** browser check 4:40 → 2:03 (Chromium's steps ~93 s → ~20 s), online checks 2:08 → 1:14. Firefox (llvmpipe) and WebKit (its own software path) still draw on the CPU but were already fast.

## Load

Several agents running heavy tests at once pushed load to 30–45 and made browser steps time out (TRIAGE T106; the 1.15.0 quick check failed once that way and passed on a rerun at load 8). The person's view: the fix is fast tests, not pausing other work.

## Left to do

1. **Done (af9a7b1):** core tests run on `cargo-nextest`, all files at once, with a progress bar: 85 s → 49 s. Browser steps show `rich` bars per engine. `scripts/watch-check.sh <log> -- <check>` runs a check detached and opens a view-only window (100 columns; no `;` may reach wt.exe, it splits the command into tabs). A full quick check with everything in three browsers: ~4.5 min (was ~10.5).
2. **Run the three browsers at the same time** (cap 2–3), and the online checks alongside the browser check. With the GPU fix this matters less; measure first.
3. **Done:** readable live output (stage start with its usual time, ✓/✗ end lines, a closing summary).
4. **The fixed 10 s idle watch** at the start of every engine (`BUILDER_IDLE`) is now half of Chromium's time: worth a look.
