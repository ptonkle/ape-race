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
# person checks it by hand on the game's Online screen or dist/nettest.html.
# Print each browser step as it finishes, not all at the end when logged to a file.
export PYTHONUNBUFFERED=1
set -euo pipefail
cd "$(dirname "$0")/.."

PY="${BUILDER_PYTHON:-}"
if [ -z "$PY" ]; then
  if [ -x .venv/bin/python ]; then PY=.venv/bin/python; else PY=python3; fi
fi

# Each stage runs through scripts/checkrun.py (see scripts/check.sh). At the
# end one `TIMES` line (scripts/timed-checks.sh keeps it in notes/check-times.md).
export CHECK_STATE; CHECK_STATE=$(mktemp)
trap 'rm -f "$CHECK_STATE"' EXIT
"$PY" scripts/checkrun.py begin online
stage() { "$PY" scripts/checkrun.py stage "$@"; }

stage release 1/3 "release build (scripts/build.sh)" quiet -- ./scripts/build.sh
stage transport 2/3 "transport between tabs (tests/browser/online.py)" plain -- "$PY" tests/browser/online.py
# `a` is this stage's name in notes/check-times.md since the first rows
stage a 3/3 "a race between two tabs (tests/browser/online_race.py)" plain -- "$PY" tests/browser/online_race.py

"$PY" scripts/checkrun.py summary
"$PY" scripts/checkrun.py times
printf '\n\033[1mONLINE CHECK PASSED\033[0m\n'
