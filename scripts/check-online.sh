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
set -euo pipefail
cd "$(dirname "$0")/.."

PY="${BUILDER_PYTHON:-}"
if [ -z "$PY" ]; then
  if [ -x .venv/bin/python ]; then PY=.venv/bin/python; else PY=python3; fi
fi

# Each step's time in seconds, printed at the end as one `TIMES` line
# (scripts/timed-checks.sh keeps them in notes/check-times.md).
T0=$(date +%s.%N); TLAST=$T0; TNAME=""; TIMES=""
lap() {
  local now; now=$(date +%s.%N)
  if [ -n "$TNAME" ]; then TIMES="$TIMES $TNAME=$(printf '%.1f' "$(echo "$now - $TLAST" | bc)")"; fi
  TLAST=$now
}
step() { lap; local n="${1#* }"; TNAME="${n%% *}"; printf '\n\033[1m== %s\033[0m\n' "$*"; }

step "1/3 release build (scripts/build.sh, checks web/vendor/SHA256SUMS)"
./scripts/build.sh >/dev/null
echo "dist/ assembled"

step "2/3 transport between tabs (tests/browser/online.py)"
"$PY" tests/browser/online.py

step "3/3 a race between two tabs through the game page (tests/browser/online_race.py)"
"$PY" tests/browser/online_race.py

lap
echo "TIMES$TIMES total=$(printf '%.1f' "$(echo "$(date +%s.%N) - $T0" | bc)")"
printf '\n\033[1mONLINE CHECK PASSED\033[0m\n'
