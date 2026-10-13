#!/usr/bin/env bash
# Copy the built game into the GitHub Pages site folder (stage 8, deploy),
# with pictures for the site's home page card: the six-ape lineup (the
# card's fallback) and the action tiles race-NN.webp (scripts/tiles.py,
# about two minutes; TILES=0 skips them and keeps the ones there). It only
# assembles files; publishing (git commit and push in the site folder) is
# a separate step.
#
#   ./scripts/pages.sh                    → ../ptonkle.github.io/ape-race/
#   ./scripts/pages.sh /path/to/site      → /path/to/site/ape-race/
#   TILES=0 ./scripts/pages.sh            → the same, without new tiles
set -euo pipefail
cd "$(dirname "$0")/.."
SITE="${1:-../ptonkle.github.io}"
PY="${BUILDER_PYTHON:-}"
if [ -z "$PY" ]; then
  if [ -x .venv/bin/python ]; then PY=.venv/bin/python; else PY=python3; fi
fi
./scripts/build.sh >/dev/null
rm -rf "$SITE/ape-race"
mkdir -p "$SITE/ape-race" "$SITE/img/ape-race"
cp -r dist/. "$SITE/ape-race/"
# The transport test page is for checks only.
rm -f "$SITE/ape-race/nettest.html" "$SITE/ape-race/nettest.js"
touch "$SITE/.nojekyll"
# The thumbnail: the lineup screen (?lineup), at the home page's card size.
"$PY" scripts/shot.py "$SITE/img/ape-race/lineup.png" --query lineup --size 990x557 --wait 2 >/dev/null
# The action tiles (?tile=...): the same moments on every run. The home
# page's script picks one per visit; its count must match the files.
if [ "${TILES:-1}" != "0" ]; then
  "$PY" scripts/tiles.py "$SITE/img/ape-race" --map jungle >/dev/null
  "$PY" scripts/tiles.py "$SITE/img/ape-race" --map city --first 16 >/dev/null
fi
echo "Assembled $SITE/ape-race/ and $SITE/img/ape-race/:"
find "$SITE/ape-race" "$SITE/img/ape-race" -type f | sort
