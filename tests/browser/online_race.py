#!/usr/bin/env python3
"""A race between two tabs through the game page (PLAN P13).

Step 3 of scripts/check-online.sh. Two tabs, each in its own browser
context, load index.html with `?codes&ice=none`, so they connect by pasted
codes on host candidates alone: no relay, no STUN, and every off-origin
request is aborted and counted as a failure. For each engine it:

- A opens Online and hosts by code; B joins with A's invite code; A accepts
  B's reply code;
- B renames itself in the lobby and A sees it; B screeches and both tabs
  play it;
- B presses Ready Up; A's Start comes on and A presses it;
- both tabs reach the race screen;
- with W held on both, each tab sees a remote ape leave its start
  (`body[data-remote-moved="true"]`): A sees B's ape, B sees A's and the
  host's AI apes;
- B's Esc menu does not pause the online race;
- fails on any off-origin request, WebSocket, CSP violation, console error
  or uncaught page error.

WebKit's tabs connect only with loopback candidates (see online.py,
LOOPBACK_ONLY): the pasted codes are rewritten for it.

    python3 tests/browser/online_race.py
"""
import pathlib
import sys

from playwright.sync_api import sync_playwright

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import online as transport  # noqa: E402  (the helpers of the transport check)

WAIT_MS = transport.WAIT_MS
check = transport.check


def by_id(id_):
    return f'[id="{id_}"]'


class Tab:
    def __init__(self, browser, origin, name, seen):
        self.name = name
        self.context = transport.watched_context(browser, origin, seen)
        self.page = self.context.new_page()
        tag = f"[{name}] "
        self.page.on("console", lambda m: seen["console"].append(tag + m.text) if m.type == "error" else None)
        self.page.on("pageerror", lambda e: seen["errors"].append(tag + str(e)))
        self.page.on("websocket", lambda ws: seen["sockets"].append(ws.url))
        self.page.goto(origin + "/index.html?codes&ice=none")
        self.page.wait_for_selector("body[data-ready='true']", timeout=WAIT_MS)

    def click(self, id_):
        self.page.click(by_id(id_), timeout=WAIT_MS)

    def code(self, id_):
        sel = by_id(id_)
        self.page.wait_for_function(
            "sel => { const e = document.querySelector(sel); return e && e.value; }",
            arg=sel,
            timeout=WAIT_MS,
        )
        return self.page.input_value(sel)

    def fill(self, id_, text):
        self.page.fill(by_id(id_), text)

    def screen(self, name):
        try:
            self.page.wait_for_selector(f"body[data-screen='{name}']", timeout=WAIT_MS)
            return True
        except Exception:
            return False

    def enabled(self, id_):
        try:
            self.page.wait_for_selector(by_id(id_) + ":not([disabled])", timeout=WAIT_MS)
            return True
        except Exception:
            return False

    def csp(self):
        return self.page.evaluate("() => window.__builderCsp || []")


def run_engine(p, engine, origin):
    print(f"\n[{engine}] {origin}/index.html?codes&ice=none")
    seen = {"foreign": [], "console": [], "errors": [], "sockets": []}
    browser = getattr(p, engine).launch()
    paste = transport.loopback if engine in transport.LOOPBACK_ONLY else (lambda c: c)
    a = Tab(browser, origin, "A", seen)
    b = Tab(browser, origin, "B", seen)
    tabs = [a, b]

    def finish():
        csp = [x for t in tabs for x in t.csp()]
        check(not csp, f"no Content-Security-Policy violation {csp}")
        check(not seen["foreign"], f"no off-origin request at any point {seen['foreign']}")
        check(not seen["sockets"], f"no WebSocket {seen['sockets']}")
        check(not seen["errors"], f"no uncaught page error {seen['errors']}")
        check(not seen["console"], f"no console error {seen['console']}")
        browser.close()

    for t in tabs:
        t.click("menu.online")
    check(a.screen("online") and b.screen("online"), "both tabs open the Online screen")

    a.click("online.host_code")
    check(a.screen("lobby"), "A hosts and is in the Lobby")
    invite = a.code("online.invite")
    check(invite.startswith("O"), "A shows an invite code")
    b.fill("online.invite", paste(invite))
    b.click("online.join_code")
    reply = b.code("online.reply")
    check(reply.startswith("A"), "B joins and shows a reply code")
    a.fill("online.reply", paste(reply))
    a.click("online.connect")

    ok = b.enabled("lobby.ready")
    check(ok, "B is seated in A's lobby")
    if not ok:
        return finish()
    seats = a.page.locator(".panel.setup table.cells tr").count() - 1
    check(seats >= 2, f"A's lobby lists both players (and its AI apes): {seats} seats")
    # TRIAGE T35: B renames itself in the lobby, and A's seats show it
    b.fill("lobby.name", "Banana Bob")
    try:
        a.page.wait_for_function(
            "() => [...document.querySelectorAll('table.cells td')].some(td => td.textContent === 'Banana Bob')",
            timeout=WAIT_MS,
        )
        renamed = True
    except Exception:
        renamed = False
    check(renamed, "B's new name reaches A's lobby")
    # TRIAGE T44: B's screech plays on A (and on B)
    b.click("lobby.screech")
    try:
        for t in tabs:
            t.page.wait_for_selector("body[data-sound='screech']", timeout=WAIT_MS)
        screeched = True
    except Exception:
        screeched = False
    check(screeched, "B's screech plays on both tabs")
    b.click("lobby.ready")
    ok = a.enabled("lobby.start")
    check(ok, "A's Start comes on once B is Ready")
    if not ok:
        return finish()
    a.click("lobby.start")
    ok = a.screen("race") and b.screen("race")
    check(ok, "both tabs reach the race")
    if not ok:
        return finish()

    for t in tabs:
        t.page.evaluate("() => document.activeElement && document.activeElement.blur()")
        t.page.keyboard.down("w")
    for t, other in ((a, "B's ape"), (b, "A's apes")):
        try:
            t.page.wait_for_selector("body[data-remote-moved='true']", timeout=WAIT_MS)
            moved = True
        except Exception:
            moved = False
        check(moved, f"{t.name} sees {other} move")

    # Esc on B opens the menu; the online race does not pause.
    b.page.keyboard.press("Escape")
    b.page.wait_for_selector(by_id("menu.resume"), timeout=WAIT_MS)
    before = b.page.text_content("#hud-time .v")
    b.page.wait_for_timeout(500)
    after = b.page.text_content("#hud-time .v")
    check(before != after, f"the online race runs on behind B's menu ({before} -> {after})")
    for t in tabs:
        t.page.keyboard.up("w")
    finish()


def main():
    if not (transport.DIST / "index.html").exists():
        sys.exit("dist/index.html is missing: run scripts/build.sh first")
    server = transport.serve()
    host, port = server.server_address[:2]
    origin = f"http://{host}:{port}"
    with sync_playwright() as p:
        for engine in transport.ENGINES:
            run_engine(p, engine, origin)
    server.shutdown()
    if transport.failures:
        print(f"\nONLINE RACE CHECK FAILED: {len(transport.failures)} problem(s)")
        for f in transport.failures:
            print("  - " + f)
        sys.exit(1)
    print(f"\nONLINE RACE CHECK PASSED in {', '.join(transport.ENGINES)}")


if __name__ == "__main__":
    main()
