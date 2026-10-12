#!/usr/bin/env bash
# The online transport check (PLAN P13, copied from Mandible). Separate from check.sh,
# which stays strictly same-origin: this one opens WebRTC connections between
# browser tabs. It still contacts no relay and no STUN server (`?ice=none`,
# pasted codes), and it fails on any off-origin request.
#
#   ./scripts/check-online.sh
#   BUILDER_ENGINES=chromium ./scripts/check-online.sh   one engine
#
# The room-code path (public Nostr relays) is never run automatically; the
# person checks it by hand on dist/nettest.html. A race between tabs through
# the game page is for the integrator to add here once app.js wires online.js.
set -euo pipefail
cd "$(dirname "$0")/.."

PY="${BUILDER_PYTHON:-}"
if [ -z "$PY" ]; then
  if [ -x .venv/bin/python ]; then PY=.venv/bin/python; else PY=python3; fi
fi

step() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

step "1/2 release build (scripts/build.sh, checks web/vendor/SHA256SUMS)"
./scripts/build.sh >/dev/null
echo "dist/ assembled"

step "2/2 transport between tabs (tests/browser/online.py)"
"$PY" tests/browser/online.py

printf '\n\033[1mONLINE CHECK PASSED\033[0m\n'
