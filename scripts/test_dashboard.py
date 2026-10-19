#!/usr/bin/env python3
"""Write notes/test-dashboard.json: the parts of the site's Test times page
(scripts/tests_page.py) that need the compiled test binaries, so the deploy
runner never compiles them.

  python3 -B scripts/test_dashboard.py [OUT]     (default notes/test-dashboard.json)

scripts/timed-checks.sh runs it after every run (the tests are compiled by
then, so it takes a few seconds). It holds:
  - "tags": per tag, how many cargo tests carry it (by module), the browser
    steps it runs and the files tagged with it (the tag slicer);
  - "linechange": what a one-line change to each of a few core files runs in
    a quick check (cargo tests / browser steps).
The runs themselves come from notes/check-times.md at page-build time.
Paths are repo-relative. The file is rewritten only when its content changes.
"""
import collections
import json
import pathlib
import sys

sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import tags as T  # noqa: E402

S = "crates/core/src/"
LINE_FILES = [  # (file under crates/core/src/, what it is)
    ("app/weather.rs", "weather picker"), ("icon.rs", "the tab icon"),
    ("app/call_tests.rs", "4 ape-call tests"), ("app/medals.rs", "medal times"),
    ("vinesim.rs", "vine bending (drawn)"), ("ape/ski_tests.rs", "ski physics tests"),
    ("jaguar.rs", "Floor is Lava jaguar"), ("hype.rs", "hype meter"),
    ("models/tourist.rs", "City outfits"), ("cityscape.rs", "City scenery"),
    ("app.rs", "the app's top level"), ("map.rs", "the list of maps"),
    ("course.rs", "Jungle course"), ("city.rs", "City course"), ("ski.rs", "Yeti Peak course"),
    ("sector.rs", "Sector Ape course"), ("ruins.rs", "Sunset Ruins course"),
    ("bay.rs", "Banana Bay course"), ("tower.rs", "tower map course"),
    ("ravine.rs", "Rattlesnake Ravine course"), ("grocery.rs", "Aisle Ape course"),
]


def module(tid):
    if tid.startswith("test:"):
        return "tests/" + tid.split(":")[1] + ".rs"
    p = tid.split(":", 1)[1].split("::")
    return "::".join(p[:-1]) or p[0]


def build():
    cfg = T.load()
    ids = sorted(T.cargo_tests())
    tt = {tid: (T.test_tags(cfg, tid) or {T.ALL}) for tid in ids}
    files = sorted(T.sh(["git", "ls-files"]).split())
    ftags = {f: (T.file_tags(cfg, f) or set()) for f in files}

    names = sorted(set(cfg["tags"]) | {t for s in tt.values() for t in s})
    tags = {}
    for t in names:
        tests = [tid for tid in ids if t in tt[tid]]
        mods = collections.Counter(module(x) for x in tests)
        tags[t] = {
            "desc": cfg["tags"].get(t, ""),
            "tests": len(tests),
            "modules": sorted(mods.items(), key=lambda kv: (-kv[1], kv[0])),
            "browser": [s for s, v in cfg["browser"].items() if t in v],
            "files": [f for f in files if t in ftags[f]],
        }

    always = set(cfg["always"]["tags"])
    base = {tid for tid in ids if T.wanted(tt[tid], T.implied(cfg, always))}
    rows = []
    for f, what in LINE_FILES:
        p = S + f
        if not (T.ROOT / p).exists():
            continue
        prefix = T.test_file_prefix(p)
        if prefix and any(t.startswith(prefix) for t in ids):
            n, b = sum(1 for t in ids if t.startswith(prefix)), 0
        else:
            hit = T.implied(cfg, always | (T.file_tags(cfg, p) or {T.ALL}))
            n = sum(1 for tid in ids if T.wanted(tt[tid], hit) and tid not in base)
            b = sum(1 for s, v in cfg["browser"].items() if T.step_hit(v, hit))
        split = p in cfg.get("functions", {}) or p in cfg.get("lines", {})
        rows.append({"file": f, "what": what, "tests": n, "browser": b, "split": split})

    return {
        "total": len(ids),
        "browser_total": len(cfg["browser"]),
        "tags": tags,
        "linechange": {"baseline": len(base), "rows": rows},
    }


def main():
    out = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else T.ROOT / "notes" / "test-dashboard.json"
    text = json.dumps(build(), indent=1, sort_keys=True) + "\n"
    if out.exists() and out.read_text() == text:
        print(f"test dashboard: {out.name} unchanged")
        return
    out.write_text(text)
    print(f"test dashboard: wrote {out.name}")


if __name__ == "__main__":
    main()
