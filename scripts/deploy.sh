#!/usr/bin/env bash
# Deploy: push main to GitHub and wait for the Pages workflow.
#   scripts/deploy.sh
# Refuses unless on main, the tree is clean, and the newest row of
# notes/check-times.md is a pass for this commit. Prints one final line:
#   pushed vX.Y.Z (<sha>); ...  (the build is not waited for; a failed
#   build is reported by the next run)
set -uo pipefail
cd "$(dirname "$0")/.."

refuse() { echo "deploy refused: $1" >&2; exit 1; }

branch=$(git symbolic-ref --short -q HEAD || true)
[[ "$branch" == main ]] || refuse "on '${branch:-detached HEAD}', not main"
[[ -z "$(git status --porcelain)" ]] || refuse "working tree is not clean"

head=$(git rev-parse HEAD)
parent=$(git rev-parse -q --verify HEAD^ || true)

# Tooling only (the person, 2026-10-18: changes to the test scripts are not
# changes to the game): when every file changed since origin/main is tagged
# [] in tests/tags.toml (scripts, notes, docs, the workflow), no game check
# is needed and no row is asked for. Anything else, or a file with no tags,
# needs the passing row below.
tooling=$(git diff --name-only origin/main..HEAD | python3 -B -c '
import sys; sys.path.insert(0, "scripts"); sys.argv = ["t"]
import tags as T
cfg = T.load()
files = [l.strip() for l in sys.stdin if l.strip()]
print("yes" if files and all(T.file_tags(cfg, f) == set() for f in files) else "no")' 2>/dev/null)

# The newest row of notes/check-times.md must be a pass for this commit.
# timed-checks.sh records the commit checked out when the checks ran, with a
# trailing "+" when the tree had uncommitted changes. The usual flow is: run
# the checks on uncommitted work (row says <parent>+), then commit it (the
# row itself goes into that commit). So accept a pass row whose commit is
# HEAD (checked after committing) or HEAD's parent with "+" (checked just
# before committing). A plain <parent> row checked the parent, not this.
if [[ "$tooling" == yes ]]; then
  echo "tooling only: no game check needed"
else
row=$(grep '^| [0-9]' notes/check-times.md | tail -n 1)
[[ -n "$row" ]] || refuse "no rows in notes/check-times.md"
IFS='|' read -r _ _when _version commit _load _check _steps _total result _ <<<"$row"
commit=$(echo "$commit" | tr -d ' ')
result=$(echo "$result" | sed 's/^ *//; s/ *$//')
[[ "$result" == pass ]] || refuse "newest check-times row is '$result', not pass"
dirty=""
[[ "$commit" == *+ ]] && dirty=1
sha=$(git rev-parse -q --verify "${commit%+}^{commit}" 2>/dev/null || true)
if [[ -n "$sha" && -z "$dirty" && "$sha" == "$head" ]]; then
  :
elif [[ -n "$sha" && -n "$dirty" && "$sha" == "$parent" ]]; then
  :
else
  refuse "newest passing check is for $commit, not HEAD ${head:0:7} or ${parent:0:7}+ (run scripts/timed-checks.sh)"
fi
fi

version=$(sed -n 's/^pub const VERSION: &str = "\(.*\)";$/\1/p' crates/core/src/app.rs)

# No waiting on the build (the person, 2026-10-18: "we don't need to wait on
# deploy.sh"). Instead, say if the previous Pages run failed, so a broken
# build shows up at the next deploy.
last=$(gh run list --workflow deploy.yml -L 1 --json conclusion,url,headSha \
  --jq '.[0] | "\(.conclusion) \(.url) \(.headSha[0:7])"' 2>/dev/null)
case "$last" in failure*|cancelled*|timed_out*) echo "note: the last Pages build ($last) did not succeed" ;; esac

git push -q origin main || refuse "git push failed"
echo "pushed v$version (${head:0:7}); GitHub builds it in about a minute (scripts/deploy-watch.sh waits for it, in the background)"
