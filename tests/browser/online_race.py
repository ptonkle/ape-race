#!/usr/bin/env python3
"""A race between two tabs through the game page (PLAN P13).

Step 3 of scripts/check-online.sh. Two tabs, each in its own browser
context, load index.html with `?codes&ice=none`, so they connect by pasted
codes on host candidates alone: no relay, no STUN, and every off-origin
request is aborted and counted as a failure. For each engine it:

- A opens Online and hosts by code; B joins with A's invite code; A accepts
  B's reply code;
- B renames itself in the lobby and A sees it; neither lobby shows the
  screech button (hidden for now, TRIAGE T57);
- B's lobby keeps its elements while the host changes a setting (TRIAGE
  T65: a rebuilt element under the mouse loses the click);
- B presses Ready Up; A's Start comes on; A picks Random and B's lobby
  shows it (T64); A picks the City, B's lobby shows it, and B stays ready
  (TRIAGE T61); A's Start and B's Ready are in view (T70);
  A presses Start;
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

`ONLINE_SHOTS=DIR` also saves each tab's lobby (DIR/lobby-<engine>-<tab>.png).
"""
import json
import os
import pathlib
import sys

from playwright.sync_api import sync_playwright

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import online as transport  # noqa: E402  (the helpers of the transport check)

WAIT_MS = transport.WAIT_MS
STRINGS = json.loads((pathlib.Path(__file__).resolve().parents[2] / "data" / "strings.json").read_text())
CITY = STRINGS["map.city"]
check = transport.check
# B's lobby shows the host's Equal Stats as this word (a list row)
EQUAL_SHOWS = (
    "([label, want]) => [...document.querySelectorAll('#menus td')]"
    ".some(td => td.textContent === label && td.nextElementSibling && td.nextElementSibling.textContent === want)"
)


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
    # TRIAGE T57: the screech button is hidden for now (T44 kept in code)
    for t in tabs:
        n = t.page.locator(by_id("lobby.screech")).count()
        check(n == 0, f"{t.name}'s lobby shows no screech button ({n})")
    # TRIAGE T65: the lobby changes under a player (the pings, a host's
    # setting); every element still there stays the same element
    marks = ["lobby.ready", "lobby.name", "lobby.ape.next", "lobby.palette"]
    b.page.evaluate("ids => ids.forEach(id => { document.getElementById(id).__mark = id; })", marks)
    for want in (STRINGS["on"], STRINGS["off"]):
        a.click("lobby.equal")
        b.page.wait_for_function(EQUAL_SHOWS, arg=[STRINGS["setup.equal"], want], timeout=WAIT_MS)
    kept = b.page.evaluate("ids => ids.map(id => document.getElementById(id).__mark === id)", marks)
    check(all(kept), f"B's lobby keeps its elements while the host changes a setting {kept}")
    # TRIAGE T65 again (the person: a dropdown still closed on every ping or
    # ape change): while B's dropdown is open its menus hold still, and they
    # catch up once it closes
    b.page.dispatch_event(by_id("lobby.palette"), "pointerdown", {"bubbles": True})
    a.click("lobby.equal")
    a.page.wait_for_timeout(300)
    b.page.wait_for_timeout(1500)
    held = b.page.evaluate(EQUAL_SHOWS, [STRINGS["setup.equal"], STRINGS["off"]])
    check(held, "B's menus hold still while its dropdown is open")
    b.page.keyboard.press("Escape")
    try:
        b.page.wait_for_function(EQUAL_SHOWS, arg=[STRINGS["setup.equal"], STRINGS["on"]], timeout=WAIT_MS)
        caught_up = True
    except Exception:
        caught_up = False
    check(caught_up, "B's menus catch up once the dropdown closes")
    a.click("lobby.equal")
    b.page.wait_for_function(EQUAL_SHOWS, arg=[STRINGS["setup.equal"], STRINGS["off"]], timeout=WAIT_MS)
    b.click("lobby.ready")
    ok = a.enabled("lobby.start")
    check(ok, "A's Start comes on once B is Ready")
    if not ok:
        return finish()
    # TRIAGE T70: Start and Ready are in view and take a click at 1280x720
    for t, id_ in ((a, "lobby.start"), (b, "lobby.ready")):
        seen_ = t.page.evaluate(
            "id => { const e = document.getElementById(id); const r = e.getBoundingClientRect();"
            " return r.top >= 0 && r.bottom <= innerHeight"
            " && document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2) === e; }",
            id_,
        )
        check(seen_, f"{t.name}'s {id_} is in view")
    # TRIAGE T64: the host's Random shows in B's lobby
    a.click("lobby.map.random")
    try:
        b.page.wait_for_selector(by_id("lobby.map") + ".read-only " + by_id("lobby.map.random") + ".chosen", timeout=WAIT_MS)
        shown = True
    except Exception:
        shown = False
    check(shown, "A's choice of Random shows in B's lobby")
    # The City (2026-10-12): the host picks the map on its grid of cards
    # (TRIAGE T54); B's lobby shows the pick live on a read-only grid.
    # TRIAGE T61: B stays ready
    check(a.enabled("lobby.map.city"), "A's map cards can be pressed")
    a.click("lobby.map.city")
    try:
        b.page.wait_for_selector(by_id("lobby.map") + ".read-only " + by_id("lobby.map.city") + ".chosen", timeout=WAIT_MS)
        b.page.wait_for_function(
            "want => { const t = document.getElementById('menus').innerText;"
            " return want.every(w => t.includes(w)); }",
            arg=[CITY],
            timeout=WAIT_MS,
        )
        shown = True
    except Exception:
        shown = False
    check(shown, "A's choice of the City shows in B's lobby")
    disabled = b.page.eval_on_selector_all(by_id("lobby.map") + " .tile, " + by_id("lobby.map") + " .card", "ts => ts.map(t => t.disabled)")
    check(disabled and all(disabled), f"B cannot press a map card {disabled}")
    if os.environ.get("ONLINE_SHOTS"):
        out = pathlib.Path(os.environ["ONLINE_SHOTS"])
        out.mkdir(parents=True, exist_ok=True)
        for t in tabs:
            t.page.screenshot(path=str(out / f"lobby-{engine}-{t.name}.png"))
    # let A's tab build the City behind its lobby before pressing Start
    # (a click during the build can stall, as in interactions.py)
    a.page.wait_for_timeout(1000)
    ok = a.enabled("lobby.start")
    check(ok, "A's Start stays on: the map change keeps B ready")
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
    for t in tabs:
        on = t.page.text_content("#hud-map .v")
        check(on == CITY, f"{t.name} races on the City ({on})")
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
