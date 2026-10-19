#!/usr/bin/env python3
"""Write the site's patch-notes page from data/patch-notes.json (the same
file the game's What's New panel reads). Every version is listed, newest
first; a minor one (x.y.z, z not 0) is drawn smaller.

    scripts/patch_notes_html.py OUT.html
"""

import html
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent

STYLE = """
    :root { --soil: #2b2118; --panel: #3a2c20; --line: #6b553c; --ink: #f4ead8;
            --muted: #c9b89c; --accent: #f0c24b; --leaf: #4f8a3a; }
    * { box-sizing: border-box; }
    body { margin: 0; min-height: 100vh; color: var(--ink); font: 16px/1.5 system-ui, sans-serif;
           background: radial-gradient(ellipse at 20% 10%, #4a3a2a 0, transparent 60%),
                       radial-gradient(ellipse at 80% 90%, #3f2f20 0, transparent 55%), var(--soil);
           padding: 2rem 1rem; }
    main { max-width: 40rem; margin: 0 auto; }
    h1 { margin: 0 0 0.3rem; font-size: 2.2rem; color: var(--accent); letter-spacing: 0.04em; }
    .links { margin: 0 0 1.5rem; color: var(--muted); }
    a { color: var(--accent); }
    section { background: var(--panel); border: 1px solid var(--line); border-radius: 12px;
              padding: 0.9rem 1.2rem; margin: 0 0 0.8rem; }
    h2 { margin: 0; font-size: 1.3rem; color: var(--accent); }
    h2 time { margin-left: 0.6rem; font-size: 0.85rem; font-weight: 400; color: var(--muted); }
    ul { margin: 0.4rem 0 0; padding-left: 1.2rem; }
    li { margin: 0.15rem 0; }
    section.minor { padding: 0.5rem 1.2rem; margin-left: 1.5rem; background: transparent; }
    section.minor h2 { font-size: 1rem; }
    section.minor ul { font-size: 0.9rem; color: var(--muted); }
"""


def minor(version):
    return version.split(".")[-1] != "0"


def page(entries):
    out = [
        "<!doctype html>",
        '<html lang="en">',
        "<head>",
        '  <meta charset="utf-8">',
        '  <meta name="viewport" content="width=device-width, initial-scale=1">',
        "  <title>Ape Race patch notes</title>",
        "  <style>" + STYLE + "  </style>",
        "</head>",
        "<body>",
        "  <main>",
        "    <h1>Ape Race patch notes</h1>",
        '    <p class="links"><a href="./">Play Ape Race</a> · <a href="tests.html">Test times</a> · <a href="../">ptonk</a></p>',
    ]
    for e in entries:
        v = html.escape(e["version"])
        d = html.escape(e["date"])
        cls = ' class="minor"' if minor(e["version"]) else ""
        out.append(f"    <section{cls}>")
        out.append(f'      <h2>v{v} <time datetime="{d}">{d}</time></h2>')
        out.append("      <ul>")
        out.extend(f"        <li>{html.escape(n)}</li>" for n in e["notes"])
        out.append("      </ul>")
        out.append("    </section>")
    out += ["  </main>", "</body>", "</html>", ""]
    return "\n".join(out)


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    entries = json.loads((ROOT / "data" / "patch-notes.json").read_text())
    pathlib.Path(sys.argv[1]).write_text(page(entries))


if __name__ == "__main__":
    main()
