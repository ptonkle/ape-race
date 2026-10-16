#!/usr/bin/env python3
"""Test tags: which tests a change needs (tests/tags.toml).

    scripts/tags.py guard                 fail if a test, browser step, online
                                          step or tracked file has no tags
    scripts/tags.py quick [base] [opts]   the quick check (scripts/quick-check.sh)
    scripts/tags.py failures LOG          the failing tests named in a check's output

quick options:
    --range A..B      take the changed files from A..B only (not the working
                      tree); the tests run are today's
    --dry-run         print the plan, run nothing
    --all-browsers    browser and online checks in Chromium, Firefox and
                      WebKit (also ALL_BROWSERS=1); Chromium only otherwise

The base defaults to the commit of the newest passing `check` row in
notes/check-times.md, so a quick check only ever skips relative to a full pass.
"""
import json
import os
import pathlib
import re
import subprocess
import sys
import time
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
TAGS = ROOT / "tests" / "tags.toml"
ALL = "all"


def load():
    return tomllib.loads(TAGS.read_text())


def glob_re(glob):
    out = ""
    i = 0
    while i < len(glob):
        if glob.startswith("**/", i):
            out += "(?:.*/)?"
            i += 3
        elif glob.startswith("**", i):
            out += ".*"
            i += 2
        elif glob[i] == "*":
            out += "[^/]*"
            i += 1
        elif glob[i] == "?":
            out += "[^/]"
            i += 1
        else:
            out += re.escape(glob[i])
            i += 1
    return re.compile(out + r"\Z")


def file_tags(cfg, path):
    """The tags of one changed file, or None when no glob matches it."""
    hit = None
    for glob, tags in cfg["sources"].items():
        if glob_re(glob).match(path):
            hit = (hit or set()) | set(tags)
    return hit


def sh(cmd, **kw):
    return subprocess.run(cmd, cwd=ROOT, text=True, check=True, capture_output=True, **kw).stdout


# --------------------------------------------------------------- cargo tests


def cargo_tests():
    """Every cargo test as an id: `lib:<path>` for the core's own tests,
    `shim:<path>` for the shim's, `test:<file>:<name>` for integration tests."""
    out = subprocess.run(
        ["cargo", "test", "--workspace", "--no-run", "--message-format=json", "--quiet"],
        cwd=ROOT, text=True, check=True, stdout=subprocess.PIPE,
    ).stdout
    ids = []
    for line in out.splitlines():
        if not line.startswith("{"):
            continue
        m = json.loads(line)
        if m.get("reason") != "compiler-artifact" or not m.get("executable") or not m["profile"]["test"]:
            continue
        t = m["target"]
        if "test" in t["kind"]:
            prefix = f"test:{t['name']}:"
        elif t["name"] == "tool_core":
            prefix = "lib:"
        else:
            prefix = "shim:"
        listed = subprocess.run(
            [m["executable"], "--list", "--format", "terse"], cwd=ROOT, text=True, check=True, capture_output=True
        ).stdout
        for row in listed.splitlines():
            if row.endswith(": test"):
                ids.append(prefix + row[: -len(": test")])
    return ids


def key_prefix(key):
    # `test:ai` covers the file's tests, not `test:ai_more`
    return key + ":" if key.startswith("test:") and key.count(":") == 1 else key


def test_tags(cfg, tid):
    best = None
    for key, tags in cfg["tests"].items():
        p = key_prefix(key)
        if tid.startswith(p) and (best is None or len(p) > len(best[0])):
            best = (p, tags)
    return None if best is None else set(best[1])


def wanted(tags, hit):
    return ALL in hit or bool(tags & hit)


# --------------------------------------------------------------------- steps


def browser_steps():
    sys.path.insert(0, str(ROOT / "tests" / "browser"))
    import interactions

    return [name for name, _ in interactions.STEPS]


ONLINE_STEPS = {"transport": "tests/browser/online.py", "race": "tests/browser/online_race.py"}


def step_hit(tags, hit):
    return ALL in hit or "page" in hit or bool(set(tags) & hit)


# --------------------------------------------------------------------- guard


def guard():
    cfg = load()
    known = set(cfg["tags"])
    problems = []

    def check_names(where, tags):
        for t in tags:
            if t not in known:
                problems.append(f"{where}: unknown tag {t!r}")

    for section in ("sources", "tests", "browser", "online"):
        for k, tags in cfg[section].items():
            check_names(f"[{section}] {k}", tags)
    check_names("[always]", cfg["always"]["tags"])

    ids = cargo_tests()
    used = set()
    for tid in ids:
        tags = test_tags(cfg, tid)
        if not tags:
            problems.append(f"test {tid} has no tags in [tests]")
        for key in cfg["tests"]:
            if tid.startswith(key_prefix(key)):
                used.add(key)
    for key in cfg["tests"]:
        if key not in used:
            problems.append(f"[tests] {key} matches no test (stale)")

    steps = browser_steps()
    for s in steps:
        if not cfg["browser"].get(s):
            problems.append(f"browser step {s} has no tags in [browser]")
    for s in cfg["browser"]:
        if s not in steps:
            problems.append(f"[browser] {s} is not a step in interactions.py (stale)")
    for s in ONLINE_STEPS:
        if not cfg["online"].get(s):
            problems.append(f"online step {s} has no tags in [online]")

    files = sh(["git", "ls-files"]).split()
    unmatched = [f for f in files if file_tags(cfg, f) is None]
    for f in unmatched:
        problems.append(f"tracked file {f} matches no glob in [sources]")

    print(f"tags: {len(ids)} cargo tests, {len(steps)} browser steps, {len(ONLINE_STEPS)} online steps, {len(files)} tracked files")
    if problems:
        print("TAG GUARD FAILED (classify them in tests/tags.toml):")
        for p in problems:
            print("  - " + p)
        return 1
    print("every test, step and file is classified")
    return 0


# --------------------------------------------------------------------- quick


def default_base():
    log = ROOT / "notes" / "check-times.md"
    for row in reversed(log.read_text().splitlines()):
        cells = [c.strip() for c in row.split("|")]
        if len(cells) > 8 and cells[5].split(" ")[0] == "check" and cells[8] == "pass":
            return cells[3].rstrip("+")
    sys.exit("no passing check row in notes/check-times.md: run the full check")


def changed_files(base, rng):
    if rng:
        return sorted(set(sh(["git", "diff", "--name-only", rng]).split()))
    files = set(sh(["git", "diff", "--name-only", base, "HEAD"]).split())
    files |= set(sh(["git", "diff", "--name-only", "HEAD"]).split())
    files |= set(sh(["git", "ls-files", "--others", "--exclude-standard"]).split())
    return sorted(files)


def plan(cfg, files):
    hit = set(cfg["always"]["tags"])
    rows = []
    for f in files:
        tags = file_tags(cfg, f)
        if tags is None:
            print(f"  {f} matches no glob in tests/tags.toml: running everything")
            tags = {ALL}
        rows.append((f, tags))
        hit |= tags
    return rows, hit


def quick(args):
    rng = None
    dry = "--dry-run" in args
    every = "--all-browsers" in args or os.environ.get("ALL_BROWSERS") == "1"
    rest = [a for a in args if a not in ("--dry-run", "--all-browsers")]
    if "--range" in rest:
        rng = rest[rest.index("--range") + 1]
        rest = [a for a in rest if a not in ("--range", rng)]
    base = rest[0] if rest else (None if rng else default_base())
    cfg = load()
    files = changed_files(base, rng)
    rows, hit = plan(cfg, files)

    print(f"== quick check: changes {rng or base + '..HEAD + working tree'}")
    for f, tags in rows:
        print(f"  {f} -> {', '.join(sorted(tags)) or '(nothing)'}")
    print(f"tags hit: {', '.join(sorted(hit))}")

    engines = "chromium firefox webkit" if every else "chromium"
    py = os.environ.get("BUILDER_PYTHON") or (str(ROOT / ".venv/bin/python") if (ROOT / ".venv/bin/python").exists() else "python3")
    times = []
    t0 = time.time()

    def step(name, cmd, env=None):
        print(f"\n\033[1m== {name}: {' '.join(cmd) if len(cmd) < 12 else ' '.join(cmd[:10]) + ' ...'}\033[0m", flush=True)
        t = time.time()
        code = subprocess.run(cmd, cwd=ROOT, env={**os.environ, **(env or {})}).returncode
        times.append(f"{name}={time.time() - t:.1f}")
        if code:
            print(f"\nQUICK CHECK FAILED at {name} (exit {code})")
            print(f"QUICK tags={','.join(sorted(hit))} engines={engines.replace(' ', '+')}")
            print("TIMES " + " ".join(times))
            sys.exit(code)

    if not dry:
        step("format", ["cargo", "fmt", "--all", "--", "--check"])
        step("lint", ["cargo", "clippy", "--workspace", "--all-targets", "--quiet", "--", "-D", "warnings"])

    # cargo tests
    t = time.time()
    ids = cargo_tests()
    times.append(f"list={time.time() - t:.1f}")
    picked = [tid for tid in ids if wanted(test_tags(cfg, tid) or {ALL}, hit)]
    # one invocation by exact name: each test binary runs the names it has
    # (lib names carry a module path; integration names are bare)
    names = sorted({tid.split(":", 2)[-1] if tid.startswith("test:") else tid.split(":", 1)[1] for tid in picked})
    tfiles = sorted({tid.split(":")[1] for tid in picked if tid.startswith("test:")})

    bsteps = [s for s, tags in cfg["browser"].items() if step_hit(tags, hit)]
    browser_run = [s for s in bsteps if s != "boot"]
    osteps = [s for s, tags in cfg["online"].items() if ALL in hit or set(tags) & hit]

    print(f"\ncargo tests: {len(picked)} of {len(ids)} (integration files {', '.join(tfiles) or 'none'})")
    print(f"browser steps: {', '.join(['boot'] + browser_run) if browser_run else 'none (browser check skipped)'}"
          f"{' in ' + engines if browser_run else ''}")
    print(f"online steps: {', '.join(osteps) if osteps else 'none (online check skipped)'}")
    if dry:
        return 0

    if ALL in hit:
        step("core", ["cargo", "test", "--workspace", "--quiet"])
    else:
        if names:
            step("core", ["cargo", "test", "--workspace", "--lib", "--tests", "--quiet", "--", "--exact", *names])
    step("release", ["./scripts/build.sh"])
    step("WebAssembly", [py, "scripts/wasm_imports.py", "dist"])
    if browser_run:
        env = {"BUILDER_ENGINES": engines}
        if not (ALL in hit or "page" in hit):
            env["BUILDER_STEPS"] = " ".join(browser_run)
        step("browser", [py, "tests/browser/check.py"], env)
    for s in osteps:
        step(f"online-{s}", [py, ONLINE_STEPS[s]], {"BUILDER_ENGINES": engines})

    print(f"QUICK tags={','.join(sorted(hit))} engines={engines.replace(' ', '+') if browser_run or osteps else 'none'}")
    print("TIMES " + " ".join(times) + f" total={time.time() - t0:.1f}")
    print("\n\033[1mQUICK CHECK PASSED\033[0m")
    return 0


# ------------------------------------------------------------------ failures


def failures(path):
    """The failing tests a check's output names, one per line."""
    out = []
    engine = step = None
    stage = None
    for line in pathlib.Path(path).read_text(errors="replace").splitlines():
        line = re.sub(r"\x1b\[[0-9;]*m", "", line)
        if line.startswith("== "):
            stage = line[3:].split(" (")[0].split(":")[0]
        m = re.match(r"\[(\w+)\] ", line)
        if m:
            engine, step = m.group(1), None
        m = re.match(r"  -- step (\w+)", line)
        if m:
            step = m.group(1)
        m = re.match(r"test (\S+) \.\.\. FAILED", line)
        if m:
            out.append(f"cargo:{m.group(1)}")
        if line.startswith("  FAIL "):
            where = f"browser[{engine}]" + (f"/{step}" if step else "") if engine else "browser"
            if stage and "online" in stage.lower():
                where = stage
            out.append(f"{where}: {line[7:]}")
        if re.match(r"(playwright\.\S+|\w+Error): ", line):
            out.append(f"{stage or '?'}{f' [{engine}]/{step}' if engine and step else ''}: {line[:160]}")
    if not out and stage:
        out.append(f"step {stage} failed")
    seen = set()
    return [x for x in out if not (x in seen or seen.add(x))]


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    cmd = sys.argv[1]
    if cmd == "guard":
        sys.exit(guard())
    if cmd == "quick":
        sys.exit(quick(sys.argv[2:]))
    if cmd == "failures":
        print("\n".join(failures(sys.argv[2])))
        return
    sys.exit(__doc__)


if __name__ == "__main__":
    main()
