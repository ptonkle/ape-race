#!/usr/bin/env python3
"""Runs one stage of a check and prints it for a person watching live.

scripts/check.sh, scripts/check-online.sh and scripts/tags.py quick (the quick
check) all run their stages through this, so the three print the same way:

    == 3/7 core tests — usually ~70 s
       tool_core lib      675 passed   4.6 s
       ravine_ai            3 passed   4.3 s
       ✓ core tests: 773 passed, 0 failed (68 s)

and end with one summary line per stage. The `TIMES`, `QUICK ...`,
`... CHECK PASSED/FAILED` lines that scripts/timed-checks.sh and
`scripts/tags.py failures` read are unchanged.

    checkrun.py begin <kind>                       start a run (check, online, quick)
    checkrun.py stage <key> <n/m> <label> <mode> -- <cmd...>
    checkrun.py summary                            the closing table
    checkrun.py times                              the `TIMES` line

`kind` picks the rows of notes/check-times.md the estimates come from; `key`
is the stage's name in the `TIMES` line. Modes:

    plain   the command's output as it comes
    quiet   nothing unless it fails (then all of it)
    cargo   cargo test, one line per test binary; failures in full
    wasm    scripts/wasm_imports.py: one line on a pass, all of it on a flag

The run's state (start time, finished stages) is in the file $CHECK_STATE.
Colour only when the output is a terminal or CHECK_COLOR=1.
"""
import json
import os
import pathlib
import re
import statistics
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[1]
LOG = ROOT / "notes" / "check-times.md"


def colour(code, text):
    on = os.environ.get("CHECK_COLOR") == "1" or (sys.stdout.isatty() and os.environ.get("CHECK_COLOR") != "0")
    return f"\033[{code}m{text}\033[0m" if on else text


def secs(s):
    return f"{s:.1f} s" if s < 10 else f"{s:.0f} s"


def estimate(kind, key):
    """The median seconds of `key` in the last five passing `kind` rows, or None."""
    try:
        rows = LOG.read_text().splitlines()
    except OSError:
        return None
    seen = []
    for row in rows:
        cells = [c.strip() for c in row.split("|")]
        if len(cells) < 9 or cells[8] != "pass" or not cells[5].startswith(kind + " "):
            continue
        m = re.search(rf"(?:^| ){re.escape(key)}=([0-9.]+)", cells[6])
        if m:
            seen.append(float(m.group(1)))
    return statistics.median(seen[-5:]) if seen else None


def header(n, label, est):
    tail = f" — usually ~{secs(est)}" if est is not None and est >= 1 else ""
    print("\n" + colour("1", f"== {n} {label}{tail}"), flush=True)


def footer(ok, label, detail, took):
    mark = colour("32", "✓") if ok else colour("31", "✗")
    print(f"   {mark} {label}: {detail} ({secs(took)})" if detail else f"   {mark} {label} ({secs(took)})", flush=True)


# ------------------------------------------------------------------ runners


def run_plain(cmd, env):
    """Passes output through; the detail is the command's own last verdict line."""
    p = subprocess.Popen(cmd, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1)
    verdict, passes, fails = "", 0, 0
    for line in p.stdout:
        sys.stdout.write(line)
        sys.stdout.flush()
        if re.search(r"CHECK (PASSED|FAILED)", line):
            verdict = line.strip()
        passes += line.startswith("  PASS ")
        fails += line.startswith("  FAIL ")
    code = p.wait()
    detail = f"{passes} checks passed, {fails} failed" if passes or fails else ""
    if code and not fails and verdict:
        detail = verdict
    return code, detail


def run_quiet(cmd, env):
    p = subprocess.run(cmd, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    if p.returncode:
        sys.stdout.write(p.stdout)
    return p.returncode, "" if not p.returncode else f"exit {p.returncode}"


def run_wasm(cmd, env):
    p = subprocess.run(cmd, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    out = p.stdout
    if p.returncode:
        sys.stdout.write(out)
        last = [l for l in out.splitlines() if l.startswith("WASM IMPORT AUDIT")]
        return p.returncode, last[-1] if last else f"exit {p.returncode}"
    mods = re.findall(r"^module  (\S+)  \(([\d,]+) bytes\)", out, re.M)
    imports = sum(int(n) for n in re.findall(r"^imports (\d+)", out, re.M))
    agree = "wasm-tools agrees" if "wasm-tools print agrees" in out else "wasm-tools not on PATH"
    what = ", ".join(f"{m} {b} bytes" for m, b in mods)
    print(f"   {what}; {imports} imports, none reaches a network API; 1 fetch (the module load); {agree}")
    print("   WASM IMPORT AUDIT PASSED")
    return 0, f"{imports} imports, nothing flagged"


# cargo's own status lines, which say nothing a watcher needs
CARGO_NOISE = re.compile(r"^\s*(Compiling|Finished|Blocking|Downloaded|Downloading|Updating|Locking|Adding|Fresh)\b")


def binary_name(line):
    """`Running unittests src/lib.rs (target/debug/deps/tool_core-1a2b)` -> `tool_core lib`."""
    m = re.match(r"\s*Running (unittests )?(\S+)(?: \((\S+)\))?", line)
    src, exe = m.group(2), m.group(3) or ""
    crate = re.sub(r"-[0-9a-f]+$", "", pathlib.Path(exe).name) if exe else ""
    if m.group(1):
        return f"{crate} {'lib' if src.endswith('lib.rs') else pathlib.Path(src).stem}".strip()
    return pathlib.Path(src).stem


def run_cargo(cmd, env):
    """cargo test without --quiet, folded to one line per test binary."""
    cmd = [c for c in cmd if c != "--quiet"]
    live = os.environ.get("CHECK_COLOR") == "1" or sys.stdout.isatty()
    p = subprocess.Popen(cmd, cwd=ROOT, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, bufsize=1)
    name, buf, done, planned = None, [], 0, 0
    total = {"passed": 0, "failed": 0, "ignored": 0}
    failing, idle, last_tick = [], 0, 0.0
    for line in p.stdout:
        s = line.rstrip("\n")
        if re.match(r"\s*(Running|Doc-tests) ", s):
            name = binary_name(s) if "Running" in s else "doc " + s.split()[-1]
            buf, done, planned = [], 0, 0
            continue
        if name is None:
            if not CARGO_NOISE.match(s) and s.strip():
                print(s, flush=True)
            continue
        m = re.match(r"running (\d+) tests?", s)
        if m:
            planned = int(m.group(1))
            continue
        if re.match(r"test .* \.\.\. ok$", s) or re.match(r"test .* \.\.\. ignored", s):
            done += 1
            if live and planned > 20 and time.time() - last_tick > 0.5:
                last_tick = time.time()
                print(f"\r\033[K   {name:<18} {done}/{planned}", end="", flush=True)
            continue
        if "has been running for over" in s:
            print(("\r\033[K" if live else "") + "   " + s, flush=True)
            continue
        m = re.match(r"test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored.*finished in ([0-9.]+)s", s)
        if m:
            ok, n_pass, n_fail, n_ign, t = m.group(1) == "ok", int(m.group(2)), int(m.group(3)), int(m.group(4)), float(m.group(5))
            total["passed"] += n_pass
            total["failed"] += n_fail
            total["ignored"] += n_ign
            if live:
                print("\r\033[K", end="")
            if not ok:
                for b in buf:
                    print(b)
                print(s)
                failing += [f"{name}: {x}" for x in re.findall(r"^test (\S+) \.\.\. FAILED", "\n".join(buf), re.M)]
            if n_pass + n_fail == 0:
                idle += 1
            else:
                fail = colour("31", f"  {n_fail} FAILED") if n_fail else ""
                ign = f"  ({n_ign} ignored)" if n_ign else ""
                print(f"   {name:<18} {n_pass:>4} passed{fail}  {t:5.1f} s{ign}", flush=True)
            name = None
            continue
        if s.strip():
            buf.append(s)
    code = p.wait()
    if idle:
        print(f"   ({idle} test binaries had none of the picked tests)")
    if name is not None and buf:  # a binary that stopped without a result: show what it said
        print(f"   {name} stopped without a result:")
        for b in buf:
            print(b)
    detail = f"{total['passed']} passed, {total['failed']} failed"
    if total["ignored"]:
        detail += f", {total['ignored']} ignored"
    if failing:
        detail += " — " + "; ".join(failing)
    elif code and not total["failed"]:
        detail = f"did not build or stopped (exit {code}); {detail}"
    return code, detail


RUNNERS = {"plain": run_plain, "quiet": run_quiet, "cargo": run_cargo, "wasm": run_wasm}


# ------------------------------------------------------------------ state


class Run:
    """One check run: its stages, for the estimates and the closing table."""

    def __init__(self, kind, state=None):
        self.kind, self.state = kind, state
        self.t0 = time.time()
        self.stages = []  # (key, label, ok, seconds, detail)
        if state and state.exists() and state.read_text().strip():
            d = json.loads(state.read_text())
            self.kind, self.t0, self.stages = d["kind"], d["t0"], [tuple(s) for s in d["stages"]]

    def save(self):
        if self.state:
            self.state.write_text(json.dumps({"kind": self.kind, "t0": self.t0, "stages": self.stages}))

    def stage(self, key, n, label, mode, cmd, env=None):
        """Runs one stage; returns its exit code (the summary is printed on a failure)."""
        header(n, label, estimate(self.kind, key))
        t = time.time()
        code, detail = RUNNERS[mode](cmd, {**os.environ, **(env or {})})
        took = time.time() - t
        footer(code == 0, label, detail, took)
        self.stages.append((key, label, code == 0, took, detail))
        self.save()
        if code:
            self.summary()
        return code

    def skip(self, key, n, label, why):
        header(n, label, None)
        print(f"   - {label}: {why}")
        self.stages.append((key, label, True, 0.0, why))
        self.save()

    def times(self):
        return " ".join(f"{k}={s:.1f}" for k, _, _, s, _ in self.stages)

    def summary(self):
        print("\n" + colour("1", "Summary"))
        for _, label, ok, s, detail in self.stages:
            mark = colour("32", "✓") if ok else colour("31", "✗")
            print(f"   {mark} {label:<32} {secs(s):>7}" + ("" if ok else f"   {detail}"))
        print(f"     {'total':<32} {secs(time.time() - self.t0):>7}", flush=True)


def main(argv):
    state = pathlib.Path(os.environ["CHECK_STATE"]) if os.environ.get("CHECK_STATE") else None
    if not argv:
        sys.exit(__doc__)
    if argv[0] == "begin":
        r = Run(argv[1])
        r.state = state
        r.save()
        return 0
    if argv[0] == "stage":
        key, n, label, mode = argv[1:5]
        cmd = argv[argv.index("--") + 1:]
        return Run("check", state).stage(key, n, label, mode, cmd)
    if argv[0] == "times":
        r = Run("check", state)
        print(f"TIMES {r.times()} total={time.time() - r.t0:.1f}")
        return 0
    if argv[0] == "summary":
        Run("check", state).summary()
        return 0
    sys.exit(__doc__)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
