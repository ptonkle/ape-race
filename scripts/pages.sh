#!/usr/bin/env bash
# Copy the built game into the GitHub Pages site folder (stage 8, deploy),
# with a thumbnail of the six apes for the site's home page. It only
# assembles files; publishing (git commit and push in the site folder) is
# a separate step.
#
#   ./scripts/pages.sh                    → ../ptonkle.github.io/ape-race/
#   ./scripts/pages.sh /path/to/site      → /path/to/site/ape-race/
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
echo "Assembled $SITE/ape-race/ and $SITE/img/ape-race/lineup.png:"
find "$SITE/ape-race" "$SITE/img/ape-race" -type f | sort
