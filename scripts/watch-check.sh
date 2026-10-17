#!/usr/bin/env bash
# Runs a check where a person can watch its progress bars, and keeps a plain
# log to grep.
#
#   ./scripts/watch-check.sh notes/run.log -- ./scripts/timed-checks.sh quick
#   ./scripts/watch-check.sh /tmp/full.log --name "full check" -- ./scripts/check.sh
#
# The check runs detached (setsid), on a terminal of its own that `script`
# records to <log>.raw; a Windows Terminal window only shows that recording
# (`tail -f`), so closing the window never stops the check: its title says
# "view only, safe to close anytime". When the check ends, <log> gets the
# plain copy (no colour, bars as their last frame; scripts/checkrun.py plain)
# with a last line `EXIT <n>`, and the window shows a PASSED/FAILED banner.
# Without wt.exe (not WSL), or with WATCH_NO_WINDOW=1, it opens no window:
# it prints where the logs are and returns. `tail -f <log>.raw` in any
# terminal shows the run too. WATCH_COLS sets the check's width (80).
set -euo pipefail
cd "$(dirname "$0")/.."
[ $# -ge 3 ] || { sed -n '2,16p' "$0"; exit 2; }
LOG=$(realpath -m "$1"); shift
NAME=""
if [ "$1" = --name ]; then NAME=$2; shift 2; fi
[ "$1" = -- ] || { echo "watch-check: expected -- before the command" >&2; exit 2; }
shift
NAME=${NAME:-$*}
RAW="$LOG.raw"
COLS=${WATCH_COLS:-80}
mkdir -p "$(dirname "$LOG")"
: > "$RAW"; rm -f "$LOG"
q() { printf '%q' "$1"; }

# The check, on its own terminal (COLS wide) that `script` records; then the
# plain log and the banner. Written to files so the command's quoting holds.
INNER="$RAW.cmd"; RUNNER="$RAW.run"
{ echo "stty cols $COLS rows 50 2>/dev/null"; printf '%q ' "$@"; echo; } > "$INNER"
{
  echo "cd $(q "$(pwd)")"
  echo "script -q -f -e -c $(q "bash $INNER") $(q "$RAW") </dev/null >/dev/null"
  echo 'code=$?'
  echo "python3 scripts/checkrun.py plain $COLS < $(q "$RAW") | grep -v '^Script \(started\|done\) on ' > $(q "$LOG")"
  echo "echo \"EXIT \$code\" >> $(q "$LOG")"
  echo 'if [ $code = 0 ]; then banner="\033[1;32m✓ PASSED"; else banner="\033[1;31m✗ FAILED (exit $code)"; fi'
  echo "printf '\\n%b — finished, safe to close this window\\033[0m\\nplain log: %s\\n' \"\$banner\" $(q "$LOG") >> $(q "$RAW")"
  echo "rm -f $(q "$INNER") $(q "$RUNNER")"
} > "$RUNNER"
setsid nohup bash "$RUNNER" >/dev/null 2>&1 < /dev/null &
PID=$!
echo "watch-check: '$NAME' running detached (pid $PID)"
echo "  live view: tail -f $RAW"
echo "  plain log: $LOG (ends with EXIT <n> when done)"

# The window: a view of the recording only. tail stops when the check does,
# after the banner (the recording's last lines).
TITLE="$NAME — view only, safe to close anytime"
# The window's commands go in a file: Windows Terminal splits its command
# line at every ';' (a new tab each), so none may reach wt.exe.
VIEW="$RAW.view"
{
  echo "printf '\\033]0;%s\\007' $(q "$TITLE")"
  echo "tail -n +1 -f --pid=$PID $(q "$RAW") 2>/dev/null"
  echo "printf '\\033]0;%s\\007' $(q "$NAME — finished, safe to close")"
  echo "read -r _"
  echo "rm -f $(q "$VIEW")"
} > "$VIEW"
if [ -z "${WATCH_NO_WINDOW:-}" ] && command -v wt.exe >/dev/null && [ -n "${WSL_DISTRO_NAME:-}" ]; then
  wt.exe -w new --size "$((COLS + 2)),52" --title "$TITLE" wsl.exe -d "$WSL_DISTRO_NAME" -- bash "$VIEW" >/dev/null 2>&1 &
  echo "  window: opened ($TITLE)"
fi
