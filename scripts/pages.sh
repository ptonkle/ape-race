#!/usr/bin/env bash
# Refresh the pictures for the ptonkle.github.io home page's Ape Race card:
# the six-ape lineup (the card's fallback) and the action tiles
# race-NN.webp (scripts/tiles.py, about two minutes; TILES=0 skips them).
# The game itself deploys from this repo: push main and
# .github/workflows/deploy.yml builds and publishes it (T85, 2026-10-15).
# Publishing the pictures (commit and push in the site folder) is separate.
#
#   ./scripts/pages.sh                    → ../ptonkle.github.io/img/ape-race/
#   ./scripts/pages.sh /path/to/site      → /path/to/site/img/ape-race/
#   TILES=0 ./scripts/pages.sh            → only the lineup picture
set -euo pipefail
cd "$(dirname "$0")/.."
SITE="${1:-../ptonkle.github.io}"
PY="${BUILDER_PYTHON:-}"
if [ -z "$PY" ]; then
  if [ -x .venv/bin/python ]; then PY=.venv/bin/python; else PY=python3; fi
fi
./scripts/build.sh >/dev/null
mkdir -p "$SITE/img/ape-race"
# The thumbnail: the lineup screen (?lineup), at the home page's card size.
"$PY" scripts/shot.py "$SITE/img/ape-race/lineup.png" --query lineup --size 990x557 --wait 2 >/dev/null
# The action tiles (?tile=...): the same moments on every run. The home
# page's script picks one per visit; its count must match the files.
if [ "${TILES:-1}" != "0" ]; then
  "$PY" scripts/tiles.py "$SITE/img/ape-race" --map jungle >/dev/null
  "$PY" scripts/tiles.py "$SITE/img/ape-race" --map city --first 16 >/dev/null
fi
echo "Refreshed $SITE/img/ape-race/:"
find "$SITE/img/ape-race" -type f | sort
