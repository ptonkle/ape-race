#!/usr/bin/env bash
# Waits for the Pages build of a pushed commit and prints one line. Run in
# the background after scripts/deploy.sh, so nothing halts on the build:
#   scripts/deploy-watch.sh [sha]      (default HEAD)
#   -> deployed <sha> in Ns     or     deploy FAILED: <url>
set -uo pipefail
cd "$(dirname "$0")/.."
head=$(git rev-parse "${1:-HEAD}")
start=$(date +%s)
id=""
for _ in $(seq 1 30); do
  id=$(gh run list --workflow deploy.yml -L 5 --json databaseId,headSha \
    --jq ".[] | select(.headSha == \"$head\") | .databaseId" 2>/dev/null | head -n 1)
  [[ -n "$id" ]] && break
  sleep 2
done
[[ -n "$id" ]] || { echo "deploy FAILED: no deploy.yml run found for ${head:0:7}"; exit 1; }
if gh run watch "$id" --exit-status >/dev/null 2>&1; then
  echo "deployed ${head:0:7} in $(($(date +%s) - start))s"
else
  echo "deploy FAILED: $(gh run view "$id" --json url --jq .url 2>/dev/null || echo "run $id")"
  exit 1
fi
