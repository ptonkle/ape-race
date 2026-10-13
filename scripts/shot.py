#!/usr/bin/env python3
"""Screenshots of the built game, for looking at the 3D view without a
person at the keys (and for the home page thumbnail).

    .venv/bin/python scripts/shot.py OUT.png [--query lineup] [--wait 2]
        [--click id ...] [--hold KeyW,Mouse:1.5 ...] [--size 1280x720]

--click presses the menu button with that element id, in order (`id=value`
chooses that value in a select, e.g. `setup.map=city`);
--hold presses keys (comma-separated) for some seconds, in order.
"""
import argparse
import functools
import http.server
import pathlib
import threading

from playwright.sync_api import sync_playwright

ROOT = pathlib.Path(__file__).resolve().parents[1]


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
        ".js": "text/javascript",
    }

    def log_message(self, *args):
        pass


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("--query", default="")
    ap.add_argument("--wait", type=float, default=2.0)
    ap.add_argument("--click", nargs="*", default=[])
    ap.add_argument("--hold", nargs="*", default=[])
    ap.add_argument("--size", default="1280x720")
    ap.add_argument("--frames", type=int, default=1, help="screenshots, --wait apart")
    a = ap.parse_args()
    w, h = (int(x) for x in a.size.split("x"))
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Handler, directory=str(ROOT / "dist")))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    url = f"http://127.0.0.1:{server.server_address[1]}/index.html" + (f"?{a.query}" if a.query else "")
    with sync_playwright() as p:
        b = p.chromium.launch(args=["--use-angle=swiftshader", "--enable-unsafe-swiftshader"])
        page = b.new_page(viewport={"width": w, "height": h})
        errors = []
        page.on("pageerror", lambda e: errors.append(str(e)))
        page.on("console", lambda m: errors.append(m.text) if m.type == "error" else None)
        page.goto(url)
        page.wait_for_selector("body[data-ready='true']", timeout=30000)
        for c in a.click:
            c, _, value = c.partition("=")
            page.wait_for_selector(f"[id='{c}']", timeout=10000)
            if value:
                page.select_option(f"[id='{c}']", value)
            else:
                page.click(f"[id='{c}']")
            page.wait_for_timeout(300)
        for spec in a.hold:
            keys, secs = spec.split(":")
            keys = keys.split(",")
            for k in keys:
                if k == "Mouse":
                    page.mouse.move(w / 2, h / 2)
                    page.mouse.down()
                else:
                    page.keyboard.down(k)
            page.wait_for_timeout(float(secs) * 1000)
            for k in keys:
                if k == "Mouse":
                    page.mouse.up()
                else:
                    page.keyboard.up(k)
        out = pathlib.Path(a.out)
        for i in range(a.frames):
            page.wait_for_timeout(a.wait * 1000)
            target = out if a.frames == 1 else out.with_name(f"{out.stem}-{i}{out.suffix}")
            page.screenshot(path=str(target))
            print(target)
        for e in errors:
            print("ERROR:", e)
        b.close()
    server.shutdown()


if __name__ == "__main__":
    main()
