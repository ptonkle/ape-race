#!/usr/bin/env bash
# Runs a check where a person can watch its progress bars, and keeps a plain
# log to grep.
#
#   ./scripts/watch-check.sh notes/run.log -- ./scripts/timed-checks.sh quick
#   ./scripts/watch-check.sh /tmp/full.log --name "full check" -- ./scripts/check.sh
#
# The check runs in a detached tmux session (watch-<time>), recorded by
# `script` to <log>.raw. A Windows Terminal window attaches to the session
# read-only: tmux tells the check the window's size, so moving or resizing
# the window just redraws the bars, and closing it only detaches (the check
# runs on). `tmux attach -r -t <session>` reattaches from any terminal. When
# the check ends, <log> gets the plain copy (no colour, bars as their last
# frame; scripts/checkrun.py plain) with a last line `EXIT <n>`, and the
# window shows a PASSED/FAILED banner, counts down 5 s and closes itself.
# Without wt.exe (not WSL), or with WATCH_NO_WINDOW=1, it opens no window.
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
mkdir -p "$(dirname "$LOG")"
: > "$RAW"; rm -f "$LOG"
q() { printf '%q' "$1"; }

# The check, on the session's terminal, recorded by `script`; then the plain
# log and the banner. Written to files so the command's quoting holds.
INNER="$RAW.cmd"; RUNNER="$RAW.run"
{ printf '%q ' "$@"; echo; } > "$INNER"
{
  echo "cd $(q "$(pwd)")"
  echo "printf '\\033]0;%s\\007' $(q "$NAME — running, view only, safe to close")"
  echo "script -q -f -e -c $(q "bash $INNER") $(q "$RAW")"
  echo 'code=$?'
  echo "python3 scripts/checkrun.py plain 200 < $(q "$RAW") | grep -v -e '^Script \\(started\\|done\\) on ' -e ' COLUMNS=' > $(q "$LOG")"
  echo "echo \"EXIT \$code\" >> $(q "$LOG")"
  echo 'if [ $code = 0 ]; then banner="\033[1;32m✓ PASSED"; else banner="\033[1;31m✗ FAILED (exit $code)"; fi'
  echo "printf '\\n%b — finished, safe to close this window\\033[0m\\nplain log: %s\\n' \"\$banner\" $(q "$LOG")"
  echo "printf '\\033]0;%s\\007' $(q "$NAME — finished, closing")"
  echo "rm -f $(q "$INNER") $(q "$RUNNER")"
  # the person, 2026-10-17: close the window 5 s after the last test; the
  # session ends, so the tmux client and its window exit
  echo "for s in 5 4 3 2 1; do printf '\\rclosing in %s s ' \$s; sleep 1; done"
} > "$RUNNER"

# Finished sessions from earlier runs (a dead pane, if any) go first.
for old in $(tmux ls -F '#{session_name} #{pane_dead}' 2>/dev/null | awk '$1 ~ /^watch-/ && $2 == 1 {print $1}'); do
  tmux kill-session -t "$old"
done
SESSION="watch-$(date +%H%M%S)"
tmux new-session -d -s "$SESSION" -x 100 -y 45 "sleep 0.3; bash $(q "$RUNNER")"
tmux set-option -t "$SESSION" status off >/dev/null
echo "watch-check: '$NAME' running in tmux session $SESSION"
echo "  watch it:  tmux attach -r -t $SESSION"
echo "  plain log: $LOG (ends with EXIT <n> when done)"

# The window: a read-only tmux client. No ';' may reach wt.exe (it splits
# its command line into tabs there).
TITLE="$NAME — view only, safe to close anytime"
if [ -z "${WATCH_NO_WINDOW:-}" ] && command -v wt.exe >/dev/null && [ -n "${WSL_DISTRO_NAME:-}" ]; then
  wt.exe -w new --size 102,47 --title "$TITLE" wsl.exe -d "$WSL_DISTRO_NAME" -- tmux attach -r -t "$SESSION" >/dev/null 2>&1 &
  echo "  window: opened ($TITLE)"
fi
