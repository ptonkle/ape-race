#!/usr/bin/env bash
# Runs check.sh and check-online.sh and adds one row per run to
# notes/check-times.md: when, the version and commit, the machine's load,
# each step's seconds and whether it passed. Run it after each update, before
# deploying (the person, 2026-10-14: "journal the time it takes to run these
# tests after each update").
#
#   ./scripts/timed-checks.sh            both checks
#   ./scripts/timed-checks.sh check      only check.sh
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
  script=scripts/check.sh; [ "$which" = online ] && script=scripts/check-online.sh
  load=$(cut -d' ' -f1 /proc/loadavg)
  when=$(date '+%Y-%m-%d %H:%M')
  out=$(mktemp)
  "$script" 2>&1 | tee "$out"
  code=${PIPESTATUS[0]}
  times=$(grep '^TIMES' "$out" | tail -n 1 | sed 's/^TIMES //')
  total=$(echo "$times" | grep -o 'total=[0-9.]*' | cut -d= -f2)
  steps=$(echo "$times" | sed 's/ *total=[0-9.]*//')
  if [ "$code" = 0 ]; then result=pass; else result="FAIL (exit $code)"; status=1; steps="${steps:-(stopped early)}"; fi
  echo "| $when | $version | $commit | $load | $which | $steps | ${total:--} | $result |" >> "$LOG"
  rm -f "$out"
done
echo; tail -n 2 "$LOG"
exit $status
