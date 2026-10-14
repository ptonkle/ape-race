#!/usr/bin/env python3
"""The map grid's pictures (TRIAGE T54): one still per map, shot from the
built game and kept as data/maps/<key>.webp, which the build ships.

    ./scripts/build.sh && .venv/bin/python scripts/map_cards.py [--only ski]
        [--out data/maps] [--try SPEC]

The maps are read from the built game itself (the cards on Race Setup), so
a new map needs no change here: it is shot from DEFAULT until CARDS names a
better moment for it. Each shot opens `dist/index.html?tile=map=<key>,<spec>`
(crates/core/src/app/tile.rs documents the spec), is shot at SHOT and
scaled down to SIZE by the browser, which also encodes the webp. Commit the
files; `cargo test` (tests/map_pictures.rs) checks each map has one and
that it stays small.
"""
import argparse
import base64
import functools
import http.server
import pathlib
import sys
import threading

from playwright.sync_api import sync_playwright

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from shot import Handler  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[1]
SHOT = (960, 540)
SIZE = (320, 180)
QUALITY = 0.8

# A moment that shows each map at its best (picked with scripts/tiles.py
# --try); a map not listed is shot from DEFAULT.
DEFAULT = "seed=1,at=2400,focus=0,view=orbit,yaw=150,pitch=-8,dist=70"
CARDS = {
    "jungle": "seed=4,at=930,focus=2,view=orbit,yaw=20,pitch=-8,dist=70",
    "city": "seed=3,at=2100,focus=2,view=orbit,yaw=60,pitch=0,dist=60",
    "ski": "seed=2,at=3000,focus=1,view=orbit,yaw=180,pitch=10,dist=90",
}

SCALE_TO_WEBP = """async ([w, h, q]) => {
    const c = document.createElement('canvas');
    c.width = w; c.height = h;
    const ctx = c.getContext('2d');
    ctx.imageSmoothingQuality = 'high';
    const img = new Image();
    img.src = window.__shot;
    await img.decode();
    ctx.drawImage(img, 0, 0, w, h);
    return c.toDataURL('image/webp', q);
}"""


def map_keys(page, base):
    """The maps the built game offers: the cards on Race Setup."""
    page.goto(base)
    page.wait_for_selector("body[data-ready='true']", timeout=120000)
    page.click("[id='menu.race']")
    page.wait_for_selector("[id='setup.map'] .tile", timeout=10000)
    return page.eval_on_selector_all("[id='setup.map'] .tile", "ts => ts.map(t => t.dataset.value)")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(ROOT / "data" / "maps"))
    ap.add_argument("--only", default="", help="comma-separated map keys to redo")
    ap.add_argument("--try", dest="spec", default="", help="shoot --only's map from this spec to try-<key>.png")
    ap.add_argument("--wait", type=float, default=2.0)
    a = ap.parse_args()
    out = pathlib.Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    only = {k for k in a.only.split(",") if k}
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Handler, directory=str(ROOT / "dist")))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_address[1]}/index.html"
    failed = False
    with sync_playwright() as p:
        b = p.chromium.launch(args=["--use-angle=swiftshader", "--enable-unsafe-swiftshader"])
        page = b.new_page(viewport={"width": SHOT[0], "height": SHOT[1]})
        keys = map_keys(page, base)
        page.close()
        for key in keys:
            if only and key not in only:
                continue
            spec = a.spec or CARDS.get(key, DEFAULT)
            page = b.new_page(viewport={"width": SHOT[0], "height": SHOT[1]})
            errors = []
            page.on("pageerror", lambda e: errors.append(str(e)))
            page.on("console", lambda m: errors.append(m.text) if m.type == "error" else None)
            page.goto(f"{base}?tile=map={key},{spec}")
            page.wait_for_selector("body[data-ready='true']", timeout=120000)
            page.wait_for_timeout(a.wait * 1000)
            png = page.screenshot()
            if a.spec:
                target = out / f"try-{key}.png"
                target.write_bytes(png)
            else:
                page.evaluate("u => { window.__shot = u; }", "data:image/png;base64," + base64.b64encode(png).decode())
                data = page.evaluate(SCALE_TO_WEBP, [SIZE[0], SIZE[1], QUALITY])
                target = out / f"{key}.webp"
                target.write_bytes(base64.b64decode(data.split(",", 1)[1]))
            print(f"{target}  {target.stat().st_size // 1024} KB  {spec}")
            for e in errors:
                failed = True
                print("ERROR:", e)
            page.close()
        b.close()
    server.shutdown()
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
