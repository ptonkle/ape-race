#!/usr/bin/env python3
"""Write the site's Test times page: how long the checks take per version,
what a quick check runs for a one-line change, and a slicer by tag.

    python3 scripts/tests_page.py OUT.html

Reads notes/check-times.md (one row per scripts/timed-checks.sh run),
notes/test-dashboard.json (tag data, written by scripts/test_dashboard.py at
the end of every timed run) and data/patch-notes.json (the live version).
Standard library only and no cargo, so the deploy runner can build it.
"""

import html
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
esc = html.escape

# Each file's runs on 2026-10-14, before the tag rework (fixed history).
BEFORE = {"app/weather.rs": "400 / 11", "icon.rs": "91 / 10", "app/call_tests.rs": "244 / 10",
          "app/medals.rs": "213 / 10", "vinesim.rs": "437 / 5", "ape/ski_tests.rs": "300 / 3",
          "jaguar.rs": "231 / 7", "hype.rs": "190 / 4", "models/tourist.rs": "214 / 7",
          "cityscape.rs": "141 / 6"}

STYLE = """
:root { --bg: #f2f5f9; --fg: #142033; --muted: #55627a; --panel: #fff; --line: #d3dcea;
        --accent: #2a5cc8; --vine: #d9571a; --online: #2f9e6e;
        --mono: ui-monospace, "SFMono-Regular", Menlo, Consolas, monospace;
        --display: "Arial Black", "Helvetica Neue", system-ui, sans-serif; color-scheme: light; }
@media (prefers-color-scheme: dark) {
  :root { --bg: #0d131e; --fg: #e8eef8; --muted: #9aa8c0; --panel: #151e2d; --line: #26344a;
          --accent: #82a8ff; --vine: #ff8f4d; --online: #3fbf87; color-scheme: dark; } }
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--fg); font: 15px/1.55 system-ui, -apple-system, "Segoe UI", sans-serif; }
main { max-width: 1480px; margin: 0 auto; padding: 32px 40px 64px; display: grid; gap: 28px; }
header { display: flex; flex-wrap: wrap; align-items: baseline; justify-content: space-between; gap: 8px 24px;
         border-bottom: 2px solid var(--fg); padding-bottom: 12px; }
h1 { font-family: var(--display); font-size: 36px; line-height: 1.05; margin: 0; }
header p { margin: 0; color: var(--muted); }
a { color: var(--accent); }
code { font-family: var(--mono); font-size: .9em; }
.lede { margin: 0; color: var(--muted); max-width: 120ch; }
.lede b { color: var(--fg); font-weight: 600; }
.pinned { display: grid; gap: 16px; background: var(--panel); border: 1px solid var(--line);
          border-left: 4px solid var(--vine); border-radius: 6px; padding: 20px 24px; }
.pinned-head { display: flex; flex-wrap: wrap; align-items: baseline; gap: 6px 18px; }
h2 { font-family: var(--display); font-size: 22px; line-height: 1.15; margin: 0; }
h3 { font-weight: 600; font-size: 13px; letter-spacing: .08em; text-transform: uppercase; color: var(--muted); margin: 0 0 10px; }
.updated { margin: 0; font-family: var(--mono); font-size: 13px; color: var(--muted); }
.updated b { color: var(--vine); font-weight: 600; }
.cols2 { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 20px 48px; }
.key { list-style: none; margin: 0; padding: 0; display: grid; gap: 4px; color: var(--muted); max-width: 120ch; }
.key b { color: var(--fg); font-weight: 600; }
.key i { display: inline-block; width: 12px; height: 12px; border-radius: 2px; margin-right: 8px; vertical-align: -1px; }
.bars { display: grid; gap: 8px; }
.bar-row { display: grid; grid-template-columns: 64px minmax(0, 1fr) 70px 150px; gap: 12px; align-items: center;
           font-family: var(--mono); font-size: 13px; }
.bar-row span:nth-child(3) { text-align: right; }
.bar-row span:nth-child(4) { color: var(--muted); }
.bar { display: flex; height: 18px; background: var(--line); border-radius: 3px; overflow: hidden; }
.bar i { display: block; height: 100%; flex: none; }
.k-rust { background: var(--accent); } .k-browser { background: var(--vine); }
.k-online { background: var(--online); } .k-other { background: var(--muted); }
details summary { cursor: pointer; font-weight: 600; color: var(--accent); margin-bottom: 10px; }
details .lede { margin-top: 8px; }
.tbl table { border-collapse: collapse; font-family: var(--mono); font-size: 13px; }
.tbl th, .tbl td { padding: 6px 18px 6px 0; border-bottom: 1px solid var(--line); text-align: right; }
.tbl th:nth-child(-n+3), .tbl td:nth-child(-n+3) { text-align: left; }
.tbl th { color: var(--muted); font-weight: 500; }
td.fail { color: var(--vine); }
.chips { display: flex; flex-wrap: wrap; gap: 6px; }
.chips button { font-family: var(--mono); font-size: 13px; color: var(--fg); background: var(--panel); border: 1px solid var(--line);
                border-radius: 4px; padding: 5px 9px; cursor: pointer; display: inline-flex; gap: 6px; align-items: baseline; }
.chips button span { color: var(--muted); font-variant-numeric: tabular-nums; }
.chips button:hover { border-color: var(--accent); }
.chips button[aria-pressed="true"] { background: var(--fg); color: var(--bg); border-color: var(--fg); }
.chips button[aria-pressed="true"] span { color: var(--bg); opacity: .75; }
button:focus-visible, a:focus-visible, summary:focus-visible { outline: 3px solid var(--accent); outline-offset: 2px; }
/* the tags beside what they slice, the result kept in view while scrolling the tags */
.slicer { display: grid; grid-template-columns: minmax(300px, 420px) minmax(0, 1fr); gap: 20px; align-items: start; }
.slicer .slice { position: sticky; top: 16px; max-height: calc(100vh - 32px); overflow: auto; }
.slicer .chips { align-content: start; }
.slice { display: grid; gap: 14px; background: var(--bg); border: 1px solid var(--line); border-radius: 5px; padding: 16px; min-width: 0; }
.slice h4 { margin: 0; font-family: var(--display); font-size: 20px; }
.slice p { margin: 0; color: var(--muted); max-width: 90ch; }
.nums { display: flex; flex-wrap: wrap; gap: 8px 28px; font-family: var(--mono); font-size: 13px; color: var(--muted); }
.nums b { font-family: var(--display); font-size: 22px; color: var(--fg); font-weight: 400; margin-right: 6px; }
.slice .cols { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) minmax(0, 1.6fr); gap: 24px; }
.slice .cols > div { min-width: 0; display: grid; gap: 6px; align-content: start; }
.slice h5 { margin: 0; font-size: 12px; letter-spacing: .08em; text-transform: uppercase; color: var(--muted); font-weight: 600; }
.slice ul { margin: 0; padding: 0; list-style: none; display: grid; gap: 2px; font-family: var(--mono); font-size: 13px; }
.slice li { display: flex; justify-content: space-between; gap: 12px; min-width: 0; }
.slice li code { overflow-wrap: anywhere; }
.slice li span { color: var(--muted); flex: none; }
/* a run's date and version stay on one line; its times wrap instead */
.slice .runs li code { white-space: nowrap; flex: none; }
.slice .runs li span { flex: 1 1 auto; min-width: 0; text-align: right; }
.none { color: var(--muted); font-family: var(--mono); font-size: 13px; }
.share { height: 8px; background: var(--line); border-radius: 3px; overflow: hidden; max-width: 600px; }
.share i { display: block; height: 100%; background: var(--accent); }
"""

SLICER_JS = r"""
(() => {
  let d; try { d = JSON.parse(document.getElementById('tagdata').textContent); } catch (e) { return; }
  const chips = document.getElementById('tag-chips'), out = document.getElementById('tag-slice');
  const names = Object.keys(d.tags).sort((a, b) => d.tags[b].tests - d.tags[a].tests || a.localeCompare(b));
  const esc = t => String(t).replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));
  const list = (rows, empty) => rows.length ? '<ul>' + rows.join('') + '</ul>' : '<span class="none">' + empty + '</span>';
  const li = (a, b) => '<li><code>' + a + '</code><span>' + (b || '') + '</span></li>';
  const more = (all, n, word) => all.length > n ? [li('… ' + (all.length - n) + ' more' + word)] : [];
  function pick(name) {
    [...chips.children].forEach(b => b.setAttribute('aria-pressed', b.dataset.tag === name));
    const t = d.tags[name], pct = d.total ? (100 * t.tests / d.total).toFixed(1) : '0';
    const runs = d.runs.filter(r => r.tags.includes(name) || r.tags.includes('all')).reverse();
    out.innerHTML =
      '<h4>' + esc(name) + '</h4>' + (t.desc ? '<p>' + esc(t.desc) + '</p>' : '') +
      '<div class="nums"><span><b>' + t.tests + '</b>of ' + d.total + ' cargo tests (' + pct + '%)</span>' +
      '<span><b>' + t.browser.length + '</b>of ' + d.browser_total + ' browser steps</span>' +
      '<span><b>' + t.files.length + '</b>files carry it</span></div>' +
      '<div class="share" aria-hidden="true"><i style="width:' + pct + '%"></i></div>' +
      '<div class="cols">' +
      '<div><h5>Tests, by module</h5>' + list(t.modules.slice(0, 16).map(([m, n]) => li(esc(m), n))
        .concat(more(t.modules, 16, ' modules')), 'No tests carry it') + '</div>' +
      '<div><h5>Browser steps</h5>' + list(t.browser.map(s => li(esc(s))), 'None') +
      '<h5 style="margin-top:10px">Files tagged</h5>' + list(t.files.slice(0, 14).map(f => li(esc(f)))
        .concat(more(t.files, 14, '')), 'None directly') + '</div>' +
      '<div class="runs"><h5>Quick-check runs that hit it, newest first</h5>' + list(runs.slice(0, 20).map(r => li(esc(r.when.slice(5)) + ' v' + esc(r.ver),
        'core ' + r.core.toFixed(1) + ' s' + (r.browser ? ' · browser ' + r.browser.toFixed(0) + ' s' : '') + ' · ' + esc(r.result.toLowerCase()))), 'No quick run hit it yet') + '</div>' +
      '</div>';
    try { localStorage.setItem('tag-slice', name); } catch (e) {}
  }
  names.forEach(n => {
    const b = document.createElement('button');
    b.type = 'button'; b.dataset.tag = n; b.setAttribute('aria-pressed', 'false');
    b.innerHTML = esc(n) + ' <span>' + d.tags[n].tests + '</span>';
    b.addEventListener('click', () => pick(n));
    chips.appendChild(b);
  });
  let start = d.tags.weather ? 'weather' : names[0];
  try { const s = localStorage.getItem('tag-slice'); if (s && d.tags[s]) start = s; } catch (e) {}
  if (start) pick(start);
})();
"""


def vkey(v):
    return tuple(int(x) for x in re.findall(r"\d+", v))


def num(s):
    try:
        return float(s)
    except ValueError:
        return None


def read_runs(path):
    runs = []
    for row in path.read_text().splitlines():
        c = [x.strip() for x in row.split("|")]
        if len(c) < 10 or not re.match(r"\d{4}-", c[1]):
            continue
        kind = c[5].split(" ")[0]
        if kind not in ("check", "online", "quick"):
            kind = "other"
        m = re.search(r"tags ([^)]*)\)", c[5])
        tags = [t.strip() for t in m.group(1).split(",") if t.strip()] if m else []
        steps = {k: float(v) for k, v in re.findall(r"([\w-]+)=([\d.]+)", c[6])}
        runs.append(dict(when=c[1], ver=c[2], load=c[4], kind=kind, tags=tags, steps=steps,
                         total=num(c[7]), ok=c[8] == "pass", result=c[8]))
    return runs


def fastest(runs, kind):
    best = {}
    for r in runs:
        if r["kind"] == kind and r["ok"] and r["total"] is not None:
            b = best.get(r["ver"])
            if b is None or r["total"] < b["total"]:
                best[r["ver"]] = r
    return best


def bar(ver, parts, scale, total, note):
    segs = "".join(
        f'<i class="{c}" style="width:{100 * s / scale:.2f}%" title="{l} {s:.1f} s"></i>'
        for c, l, s in parts if s > 0)
    t = f"{total / 60:.1f} min" if total >= 120 else f"{total:.0f} s"
    return (f'<div class="bar-row" role="row"><span role="cell">{esc(ver)}</span>'
            f'<span class="bar" role="cell">{segs}</span><span role="cell">{t}</span>'
            f'<span role="cell">{esc(note)}</span></div>')


def times_panel(runs):
    full, online, quick = fastest(runs, "check"), fastest(runs, "online"), fastest(runs, "quick")
    rows = []
    for v in sorted(full, key=vkey):
        f, o = full[v], online.get(v)
        core, br = f["steps"].get("core", 0), f["steps"].get("browser", 0)
        on = o["total"] if o else 0
        rows.append((v, [("k-rust", "Rust tests", core), ("k-browser", "Browser check", br),
                         ("k-online", "Online check", on),
                         ("k-other", "Format, lint, build", max(f["total"] - core - br, 0))],
                     f["total"] + on, f["when"][5:10] + ("" if o else " · no online")))
    qrows = []
    for v in sorted(quick, key=vkey):
        q = quick[v]
        s = q["steps"]
        core, br = s.get("core", 0), s.get("browser", 0)
        on = s.get("online-transport", 0) + s.get("online-race", 0)
        n = len(q["tags"])
        note = "all tags" if "all" in q["tags"] else (f"{n} tag{'s' * (n != 1)}" if n else "tags not logged")
        qrows.append((v, [("k-rust", "Rust tests", core), ("k-browser", "Browser steps", br),
                          ("k-online", "Online steps", on),
                          ("k-other", "Format, lint, list, build", max(q["total"] - core - br - on, 0))],
                      q["total"], note))

    def bars(rs):
        scale = max((r[2] for r in rs), default=1) or 1
        return "\n".join("        " + bar(v, p, scale, t, n) for v, p, t, n in rs) or '        <p class="none">No passing run yet</p>'

    trs = []
    for r in reversed(runs):
        s = r["steps"]
        cell = lambda k: f"{s[k]:.1f}" if k in s else "–"  # noqa: E731
        tot = f"{r['total']:.1f}" if r["total"] is not None else "–"
        tagn = str(len(r["tags"])) if r["kind"] == "quick" else ""
        res = "pass" if r["ok"] else esc(r["result"].replace("FAIL", "fail"))
        label = {"check": "Full", "online": "Online", "quick": "Quick"}.get(r["kind"], "Other")
        trs.append(f'          <tr><td>{esc(r["when"][5:])}</td><td>{esc(r["ver"])}</td><td>{label}</td>'
                   f'<td>{esc(r["load"])}</td><td>{cell("core")}</td><td>{cell("browser")}</td>'
                   f'<td>{tagn}</td><td>{tot}</td><td{"" if r["ok"] else " class=fail"}>{res}</td></tr>')

    summary = []
    if rows:
        first, last = min(full, key=vkey), max(full, key=vkey)
        summary.append(f"A full check runs every test in three browsers, and took <b>{rows[0][2] / 60:.1f} minutes "
                       f"at v{esc(first)}</b>; the latest full run (v{esc(last)}) took <b>{rows[-1][2] / 60:.1f} minutes</b>.")
    if quick:
        lq = max(quick, key=vkey)
        summary.append("Most deploys use a quick check, which runs only the tests whose tags match what changed, "
                       f"most recently v{esc(lq)} in <b>{quick[lq]['total']:.0f} s</b>.")
    newest = max(runs, key=lambda r: (vkey(r["ver"]), r["when"])) if runs else None
    upd = (f'last run <b>v{esc(newest["ver"])}</b>, {esc(newest["when"][:10])} · ' if newest else "")
    return f"""    <section class="pinned" id="testtimes">
      <div class="pinned-head">
        <h2>All tests per version</h2>
        <p class="updated">{upd}{len(runs)} runs from <code>notes/check-times.md</code></p>
      </div>
      <ul class="key">
        <li><i class="k-rust"></i><b>Rust tests</b>: the game's rules, physics, AI, menus and maps, checked without a browser (<code>cargo test</code>).</li>
        <li><i class="k-browser"></i><b>Browser check</b>: opens the real game page and clicks through it (Chrome, Firefox and Safari's engine for a full check; Chrome only for most quick checks).</li>
        <li><i class="k-online"></i><b>Online check</b>: two browsers host and join a race against each other.</li>
        <li><i class="k-other"></i><b>Format, lint, build</b> (and, for quick checks, listing which tests to run).</li>
      </ul>
      <div class="cols2">
        <div>
          <h3>Full check + online, fastest passing run</h3>
          <div class="bars" role="table" aria-label="Full check time per version">
{bars(rows)}
          </div>
        </div>
        <div>
          <h3>Quick check, fastest passing run</h3>
          <div class="bars" role="table" aria-label="Quick check time per version">
{bars(qrows)}
          </div>
        </div>
      </div>
      <p class="lede">Each bar is the version's fastest passing run, because other work on the machine changes a run by up to half a minute. {" ".join(summary)} The two charts use different scales.</p>
      <details>
        <summary>Every run, newest first ({len(runs)})</summary>
        <div class="tbl"><table>
          <thead><tr><th>When</th><th>Version</th><th>Check</th><th>Load</th><th>Rust tests</th><th>Browser</th><th>Tags</th><th>Total</th><th>Result</th></tr></thead>
          <tbody>
{chr(10).join(trs)}
          </tbody>
        </table></div>
        <p class="lede">Seconds. Load is the machine's 1-minute load average when the run started; a dash means the run stopped before that step.</p>
      </details>
    </section>"""


def quick_panel(dash, runs):
    lc = dash.get("linechange", {"rows": [], "baseline": 0})
    trs = "\n".join(
        f'          <tr><td>{esc(r["file"])}{"*" if r["split"] else ""}</td><td>{esc(r["what"])}</td>'
        f'<td>{BEFORE.get(r["file"], "")}</td><td>{r["tests"]} / {r["browser"]}</td></tr>'
        for r in lc["rows"])
    data = {"total": dash.get("total", 0), "browser_total": dash.get("browser_total", 0),
            "tags": dash.get("tags", {}),
            "runs": [{"when": r["when"], "ver": r["ver"], "tags": r["tags"], "core": r["steps"].get("core", 0),
                      "browser": r["steps"].get("browser", 0), "result": r["result"]}
                     for r in runs if r["kind"] == "quick"]}
    safe = json.dumps(data, separators=(",", ":")).replace("</", "<\\/")
    return f"""    <section class="pinned" id="quickcheck">
      <div class="pinned-head">
        <h2>Quick checks: what a change runs</h2>
        <p class="updated">{dash.get("total", 0)} cargo tests, {dash.get("browser_total", 0)} browser steps · tag data from <code>notes/test-dashboard.json</code></p>
      </div>
      <p class="lede">A quick check reads which files changed since what's live, collects their tags from <code>tests/tags.toml</code>, and runs only the tests and browser steps that share one. Until the 2026-10-17 rework the tags were far too broad: removing the Weather row from Settings ran <b>400 tests and all 11 browser steps</b>.</p>
      <ul class="key">
        <li><i class="k-rust"></i><b>Cargo tests</b>: small programs that check the game's own code directly, without opening the game (<code>cargo test</code>). Most take a fraction of a second; the AI races round a whole map take longer.</li>
        <li><i class="k-browser"></i><b>Browser steps</b>: scripts that open the real game page in a browser with no window and click through it like a player. Each takes 20–30 s, so they are where most of a check's time goes.</li>
        <li><i class="k-other"></i><b>Tags</b>: labels such as <code>weather</code>, <code>hud</code> or <code>maps-city</code> on every source file, cargo test and browser step.</li>
      </ul>
      <div class="cols2">
        <div>
          <h3>What a one-line change runs (cargo tests / browser steps)</h3>
          <div class="tbl"><table>
            <thead><tr><th>File changed</th><th>What it is</th><th>2026-10-14</th><th>Now</th></tr></thead>
            <tbody>
{trs}
            </tbody>
          </table></div>
          <p class="lede" style="margin-top:8px">Files in <code>crates/core/src/</code>. Not counting the {lc["baseline"]} boundary tests every check runs. A file that holds only tests runs just its own. * Tagged function by function: the number is for a line outside those functions, so most edits there run far fewer.</p>
        </div>
      </div>
      <h3>Slice by tag</h3>
      <p class="lede" style="margin-bottom:10px">Pick a tag to see how many tests carry it, which browser steps it runs, which files it is attached to, and the quick-check runs that hit it.</p>
      <div class="slicer">
        <div class="chips" id="tag-chips" role="group" aria-label="Tags"></div>
        <div class="slice" id="tag-slice" aria-live="polite"></div>
      </div>
    </section>
<script type="application/json" id="tagdata">{safe}</script>"""


def page():
    runs = read_runs(ROOT / "notes" / "check-times.md")
    dash_path = ROOT / "notes" / "test-dashboard.json"
    dash = json.loads(dash_path.read_text()) if dash_path.exists() else {}
    notes = json.loads((ROOT / "data" / "patch-notes.json").read_text())
    live = f'v{esc(notes[0]["version"])}, {esc(notes[0]["date"])}' if notes else ""
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Ape Race test times</title>
  <style>{STYLE}</style>
</head>
<body>
  <main>
    <header>
      <h1>Ape Race test times</h1>
      <p>Live: {live} · <a href="./">Play Ape Race</a> · <a href="patch-notes.html">Patch notes</a></p>
    </header>
    <p class="lede">How long the game's checks take before each deploy, and what a change makes them run. Built from the repository on every deploy.</p>
{times_panel(runs)}
{quick_panel(dash, runs)}
  </main>
<script>{SLICER_JS}</script>
</body>
</html>
"""


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    pathlib.Path(sys.argv[1]).write_text(page())


if __name__ == "__main__":
    main()
