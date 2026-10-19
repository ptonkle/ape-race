#!/usr/bin/env bash
# Runs check.sh and check-online.sh and adds one row per run to
# notes/check-times.md: when, the version and commit, the machine's load,
# each step's seconds and whether it passed. Run it after each update, before
# deploying (the person, 2026-10-14: "journal the time it takes to run these
# tests after each update").
#
#   ./scripts/timed-checks.sh            both checks
#   ./scripts/timed-checks.sh check      only check.sh
#   ./scripts/timed-checks.sh quick      only scripts/quick-check.sh (QUICK_BASE=<commit>
#                                        to diff from another base; ALL_BROWSERS=1)
#
# The Check cell names the engines the browser and online checks ran in, and
# for a quick check the tags it hit. A failing run also adds the names of
# the failing tests and browser steps to notes/test-failures.md.
# Print each browser step as it finishes, not all at the end when logged to a file.
export PYTHONUNBUFFERED=1
set -uo pipefail
cd "$(dirname "$0")/.."
LOG=notes/check-times.md
if [ ! -f "$LOG" ]; then
  cat > "$LOG" <<'HEAD'
# Check times

One row per run of `scripts/timed-checks.sh`, newest last. Seconds per step; `load` is the 1-minute load average on this machine (16 cores) when the run started.

| When | Version | Commit | Load | Check | Steps (s) | Total (s) | Result |
| --- | --- | --- | --- | --- | --- | --- | --- |
HEAD
fi
version=$(grep -o 'pub const VERSION: &str = "[^"]*"' crates/core/src/app.rs | cut -d'"' -f2)
commit=$(git rev-parse --short HEAD)
[ -n "$(git status --porcelain -- crates web data tests scripts)" ] && commit="$commit+"
status=0
for which in ${1:-check online}; do
  script=scripts/check.sh; args=()
  [ "$which" = online ] && script=scripts/check-online.sh
  if [ "$which" = quick ]; then script=scripts/quick-check.sh; [ -n "${QUICK_BASE:-}" ] && args=("$QUICK_BASE"); fi
  load=$(cut -d' ' -f1 /proc/loadavg)
  when=$(date '+%Y-%m-%d %H:%M')
  out=$(mktemp)
  if [ -t 1 ] && command -v script >/dev/null; then
    # watched in a terminal: the check keeps one (its progress bars), and
    # `script` records it; the recording is read as plain lines below
    script -q -f -e -c "$(printf '%q ' "$script" "${args[@]}")" "$out.raw"
    code=$?
    python3 scripts/checkrun.py plain "$(tput cols 2>/dev/null || echo 200)" < "$out.raw" | grep -v '^Script \(started\|done\) on ' > "$out"; rm -f "$out.raw"
  else
    "$script" "${args[@]}" 2>&1 | tee "$out"
    code=${PIPESTATUS[0]}
  fi
  times=$(grep '^TIMES' "$out" | tail -n 1 | sed 's/^TIMES //')
  total=$(echo "$times" | grep -o 'total=[0-9.]*' | cut -d= -f2)
  steps=$(echo "$times" | sed 's/ *total=[0-9.]*//')
  if [ "$code" = 0 ]; then result=pass; else result="FAIL (exit $code)"; status=1; steps="${steps:-(stopped early)}"; fi
  engines="${BUILDER_ENGINES:-chromium firefox webkit}"; engines="${engines// /+}"
  kind="$which ($engines)"
  if [ "$which" = quick ]; then
    q=$(grep '^QUICK ' "$out" | tail -n 1)
    kind="quick ($(echo "$q" | grep -o 'engines=[^ ]*' | cut -d= -f2); tags $(echo "$q" | grep -o 'tags=[^ ]*' | cut -d= -f2 | sed 's/,/, /g'))"
  fi
  echo "| $when | $version | $commit | $load | $kind | $steps | ${total:--} | $result |" >> "$LOG"
  if [ "$code" != 0 ]; then
    FLOG=notes/test-failures.md
    if [ ! -f "$FLOG" ]; then
      cat > "$FLOG" <<'HEAD'
# Test failures

One row per failing run of `scripts/timed-checks.sh`, newest last: the tests and browser steps that failed (from `scripts/tags.py failures`). Passing runs add nothing; notes/check-times.md has every run.

| When | Version | Commit | Check | Failing |
| --- | --- | --- | --- | --- |
HEAD
    fi
    failing=$(python3 scripts/tags.py failures "$out" | sed 's/|/\\|/g' | paste -sd ';' - | sed 's/;/; /g')
    echo "| $when | $version | $commit | $kind | ${failing:-(none named)} |" >> "$FLOG"
  fi
  rm -f "$out"
done
# The site's Test times page (scripts/tests_page.py, built on deploy) reads
# its tag data from notes/test-dashboard.json; refresh it while the tests are
# compiled. It never fails the check.
timeout 90 python3 -B scripts/test_dashboard.py \
  || echo "warning: notes/test-dashboard.json not refreshed (scripts/test_dashboard.py failed or timed out)" >&2
echo; tail -n 2 "$LOG"
exit $status
