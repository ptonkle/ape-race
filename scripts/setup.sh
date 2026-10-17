#!/usr/bin/env bash
# One-time setup for the browser check: a Python virtual environment with
# Playwright, and the three browser engines it drives.
set -euo pipefail
cd "$(dirname "$0")/.."
python3 -m venv .venv
.venv/bin/pip install --quiet --upgrade pip
.venv/bin/pip install --quiet "playwright==1.63.0"
.venv/bin/pip install --quiet "rich==14.1.0"  # the browser checks' progress bars (optional)
.venv/bin/python -m playwright install ${PLAYWRIGHT_WITH_DEPS:+--with-deps} chromium firefox webkit
# cargo-nextest runs the core test files at once, with a progress bar
# (optional: without it the checks fall back to cargo test)
command -v cargo-nextest >/dev/null || cargo install cargo-nextest --locked
# Xvfb lets Chromium draw on the GPU in an invisible window (optional:
# without it Chromium runs headless in software; tests/browser/engines.py)
command -v Xvfb >/dev/null || echo "note: install xvfb (sudo apt install xvfb) for GPU drawing in the browser check"
echo "Browser check ready: .venv/bin/python tests/browser/check.py"
