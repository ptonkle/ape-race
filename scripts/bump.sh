#!/usr/bin/env bash
# Bump the version and add a patch-notes entry.
#   scripts/bump.sh <major|minor|x.y.z> "note" ["note" ...]
# major: x.y.z -> x.(y+1).0   minor: x.y.z -> x.y.(z+1)
# Edits VERSION in crates/core/src/app.rs and prepends the entry (today's
# date) to data/patch-notes.json. Prints the new version.
set -euo pipefail
cd "$(dirname "$0")/.."

usage() { echo "usage: scripts/bump.sh <major|minor|x.y.z> \"note\" [\"note\" ...]" >&2; exit 2; }
(($# >= 2)) || usage
kind="$1"
shift

app=crates/core/src/app.rs
notes=data/patch-notes.json

current=$(sed -n 's/^pub const VERSION: &str = "\([0-9]*\.[0-9]*\.[0-9]*\)";$/\1/p' "$app")
[[ -n "$current" ]] || { echo "bump: no VERSION line in $app" >&2; exit 1; }
top=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))[0]["version"])' "$notes")
[[ "$current" == "$top" ]] || { echo "bump: refused: VERSION $current in $app but top patch note is $top" >&2; exit 1; }

IFS=. read -r x y z <<<"$current"
case "$kind" in
  major) next="$x.$((y + 1)).0" ;;
  minor) next="$x.$y.$((z + 1))" ;;
  [0-9]*.[0-9]*.[0-9]*) next="$kind" ;;
  *) usage ;;
esac
[[ "$next" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || usage
[[ "$next" != "$current" ]] || { echo "bump: refused: already at $current" >&2; exit 1; }

python3 - "$notes" "$next" "$(date +%F)" "$@" <<'PY'
import json, sys
path, version, date, *lines = sys.argv[1:]
entries = json.load(open(path, encoding="utf-8"))
entries.insert(0, {"version": version, "date": date, "notes": lines})
with open(path, "w", encoding="utf-8") as f:
    f.write(json.dumps(entries, indent=2, ensure_ascii=False) + "\n")
PY
sed -i "s/^pub const VERSION: &str = \"$current\";$/pub const VERSION: \&str = \"$next\";/" "$app"
echo "$next"
