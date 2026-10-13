#!/usr/bin/env python3
"""The home page's Ape Race tiles: action shots of the built game, as
990x557 webp files, the same on every run.

    .venv/bin/python scripts/tiles.py OUT_DIR [--map jungle] [--first 1]
        [--only 3,7] [--wait 2] [--try SPEC ...]

Each shot opens `dist/index.html?tile=<spec>`: the core runs an all-AI race
from the spec's seed to its tick and holds it there, seen by the spec's
camera, with no menus (crates/core/src/app/tile.rs documents the spec).
The files are OUT_DIR/race-NN.webp, numbered from --first. Chromium encodes
the webp (the venv has no imaging library).

To shoot another map, add its list to SHOTS (pick moments with
`cargo run --release -p tool_core --example tile_moments -- "map=city,seed=3"`),
then run with --map city --first 13 and raise the count on the home page.
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
SIZE = (990, 557)
QUALITY = 0.9

# (what it shows, spec without the map). yaw/pitch in degrees, dist in dm;
# yaw 0 is from behind, along the course. Ticks are 120 a second; the
# countdown takes the first 3 s.
SHOTS = {
    "jungle": [
        ("the four apes on the start platform", "seed=1,at=300,focus=1,view=orbit,yaw=200,pitch=-22,dist=75"),
        ("leaping off the start at Go", "seed=1,at=440,focus=1,view=orbit,yaw=150,pitch=-5,dist=70"),
        ("an orangutan flying down a lane of vines", "seed=4,at=930,focus=2,view=orbit,yaw=20,pitch=-8,dist=70"),
        ("a chimpanzee and a lemur neck and neck", "seed=3,at=990,focus=2,view=orbit,yaw=-70,pitch=5,dist=60"),
        ("a mushroom bounce", "seed=1,at=1176,focus=3,view=orbit,yaw=110,pitch=0,dist=55"),
        ("a gibbon swinging through bananas", "seed=1,at=1440,focus=3,view=orbit,yaw=90,pitch=10,dist=60"),
        ("a proboscis monkey letting go, side on", "seed=2,at=1500,focus=0,view=orbit,yaw=90,pitch=-5,dist=75"),
        ("a Hook hit: the gibbon stunned, a chimp flying past", "seed=1,at=2010,focus=3,view=orbit,yaw=200,pitch=-5,dist=50"),
        ("a gorilla in flight, from below", "seed=2,at=3000,focus=1,view=orbit,yaw=-120,pitch=20,dist=70"),
        ("a banana hit by a mushroom", "seed=1,at=4490,focus=0,view=orbit,yaw=180,pitch=-8,dist=45"),
        ("the chase camera with the HUD at BANANAS! speed", "seed=3,at=5200,focus=2,view=chase,hud=on"),
        ("a gibbon and a lemur on vines side by side", "seed=3,at=6660,focus=0,view=orbit,yaw=100,pitch=0,dist=60"),
        ("Floor is Lava: the jaguar pounces", "seed=3,lava=on,at=1550,focus=2,view=orbit,yaw=110,pitch=-8,dist=65"),
        ("the leader flying head-on near the end", "seed=1,at=8178,focus=3,view=orbit,yaw=180,pitch=15,dist=80"),
        ("the winner leaping for the finish line", "seed=1,at=8165,focus=3,view=orbit,yaw=0,pitch=12,dist=60"),
    ],
    # race-16 on (run with --first 16)
    "city": [
        ("four apes on the rooftop start", "seed=1,at=300,focus=1,view=orbit,yaw=200,pitch=-22,dist=75"),
        ("a chimp swinging past Times Square's billboards", "seed=3,at=2100,focus=2,view=orbit,yaw=60,pitch=0,dist=60"),
        ("flying down Midtown through bananas", "seed=3,at=3300,view=chase"),
        ("a gorilla in flight among cranes at sunset", "seed=2,at=3100,focus=1,view=orbit,yaw=-60,pitch=5,dist=55"),
        ("a lemur leaping by the cranes and the tower", "seed=1,at=5200,focus=0,view=orbit,yaw=90,pitch=0,dist=60"),
        ("a lemur swinging on a glowing wire, helicopters behind", "seed=1,at=7600,focus=0,view=orbit,yaw=150,pitch=-8,dist=70"),
    ],
}


def to_webp(page, png):
    """The PNG bytes as webp bytes, encoded by the browser."""
    url = "data:image/png;base64," + base64.b64encode(png).decode()
    data = page.evaluate(
        """async ([url, q]) => {
            const img = new Image();
            img.src = url;
            await img.decode();
            const c = document.createElement("canvas");
            c.width = img.width; c.height = img.height;
            c.getContext("2d").drawImage(img, 0, 0);
            return c.toDataURL("image/webp", q);
        }""",
        [url, QUALITY],
    )
    return base64.b64decode(data.split(",", 1)[1])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--map", default="jungle")
    ap.add_argument("--first", type=int, default=1)
    ap.add_argument("--only", default="", help="comma-separated shot numbers (1-based) to redo")
    ap.add_argument("--wait", type=float, default=2.0)
    ap.add_argument("--png", action="store_true", help="also keep the PNG")
    ap.add_argument("--try", dest="tries", nargs="*", default=[], help="specs to try instead of the list: try-N.png")
    a = ap.parse_args()
    shots = [(f"try {spec}", spec) for spec in a.tries] or SHOTS.get(a.map)
    if not shots:
        sys.exit(f"no shots listed for map {a.map!r}; add them to SHOTS in {__file__}")
    only = {int(n) for n in a.only.split(",") if n}
    out = pathlib.Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Handler, directory=str(ROOT / "dist")))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_address[1]}/index.html"
    failed = False
    with sync_playwright() as p:
        b = p.chromium.launch(args=["--use-angle=swiftshader", "--enable-unsafe-swiftshader"])
        for k, (what, spec) in enumerate(shots, start=1):
            if only and k not in only:
                continue
            page = b.new_page(viewport={"width": SIZE[0], "height": SIZE[1]})
            errors = []
            page.on("pageerror", lambda e: errors.append(str(e)))
            page.on("console", lambda m: errors.append(m.text) if m.type == "error" else None)
            page.goto(f"{base}?tile=map={a.map},{spec}")
            page.wait_for_selector("body[data-ready='true']", timeout=120000)
            page.wait_for_timeout(a.wait * 1000)
            png = page.screenshot()
            if a.tries:
                target = out / f"try-{k}.png"
                target.write_bytes(png)
            else:
                target = out / f"race-{a.first + k - 1:02d}.webp"
                target.write_bytes(to_webp(page, png))
            if a.png and not a.tries:
                target.with_suffix(".png").write_bytes(png)
            print(f"{target}  {target.stat().st_size // 1024} KB  {what}")
            for e in errors:
                failed = True
                print("ERROR:", e)
            page.close()
        b.close()
    server.shutdown()
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
