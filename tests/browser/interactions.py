"""The game's own browser interactions, run by check.py in every engine.

check.py has already loaded the page, waited for it to report ready, and let
it idle. This drives the game the way a person would and checks what the
page shows. Expected values are worked out by hand from the brief (C4),
never copied from the game's own output.
"""

import json
import pathlib

STRINGS = json.loads((pathlib.Path(__file__).resolve().parents[2] / "data" / "strings.json").read_text())


def text(page, sel):
    return page.inner_text(sel).strip()


def canvas_drawn(page):
    """True when the 3D view holds more than one colour (WebGL drew)."""
    return page.evaluate(
        """() => {
        const c = document.getElementById('view');
        const probe = document.createElement('canvas');
        probe.width = 16; probe.height = 16;
        const g = probe.getContext('2d');
        g.drawImage(c, 0, 0, 16, 16);
        const d = g.getImageData(0, 0, 16, 16).data;
        const seen = new Set();
        for (let i = 0; i < d.length; i += 4) seen.add(d[i] + ',' + d[i + 1] + ',' + d[i + 2]);
        return seen.size > 4;
    }"""
    )


def chosen_map(page):
    """The keys of the map cards marked chosen on the setup's grid."""
    return page.eval_on_selector_all("[id='setup.map'] .tile.chosen", "ts => ts.map(t => t.dataset.value)")


def run(page, check, engine):
    check(page.title() == STRINGS["title"], "the tab is titled Ape Race")
    check(page.evaluate("document.body.dataset.screen") == "title", "the title screen is up")
    check(STRINGS["title"].upper() in text(page, "#menus").upper(), "the title is drawn")
    check(text(page, "#message") == "", "no error message is shown")
    check(canvas_drawn(page), "the jungle is drawn behind the title (WebGL 2)")

    # Settings: change the volume, see it shown, come back
    page.click("[id='menu.settings']")
    check(page.evaluate("document.body.dataset.screen") == "settings", "Settings opens")
    page.fill("[id='settings.volume']", "35")
    page.dispatch_event("[id='settings.volume']", "change")
    page.wait_for_timeout(200)
    check("35%" in text(page, "#menus"), "the volume slider shows 35%")
    page.click("[id='menu.back']")

    # Race setup: pick the gibbon, two opponents; the stats follow the ape
    page.click("[id='menu.race']")
    check(page.evaluate("document.body.dataset.screen") == "setup", "Race opens the setup screen")
    names = [STRINGS["ape." + k] for k in ["orangutan", "gorilla", "chimpanzee", "gibbon", "proboscis", "lemur"]]
    seen = set()
    for _ in range(6):
        seen.add(text(page, "[id='setup.ape']"))
        page.click("[id='setup.ape.next']")
    check(seen == set(names), f"the picker cycles through all six apes {sorted(seen)}")
    while text(page, "[id='setup.ape']") != STRINGS["ape.gibbon"]:
        page.click("[id='setup.ape.next']")
    # the gibbon's Top Speed is 5 of 5 (rules.rs), worked out from the brief's "fastest swinger"
    pips = page.evaluate(
        "() => [...document.querySelectorAll('.stat')].map(s => s.querySelectorAll('.pip.on').length)"
    )
    check(pips[1] == 5, f"the gibbon shows Top Speed 5 {pips}")
    page.click("[id='setup.equal']")
    page.wait_for_timeout(100)
    pips = page.evaluate(
        "() => [...document.querySelectorAll('.stat')].map(s => s.querySelectorAll('.pip.on').length)"
    )
    check(pips == [3, 3, 3, 3, 3], f"Equal Stats shows the average ape {pips}")
    page.click("[id='setup.equal']")
    # TRIAGE T42: a checkbox per item, all ticked at first; one unticks
    boxes = page.evaluate("() => [...document.querySelectorAll('.checks input')].map(b => b.checked)")
    check(boxes == [True] * 5, f"five item checkboxes, all ticked {boxes}")
    page.click("[id='setup.item.banana']")
    page.wait_for_timeout(100)
    check(not page.is_checked("[id='setup.item.banana']"), "the Banana box unticks")
    page.click("[id='setup.item.banana']")
    page.wait_for_timeout(100)
    page.select_option("[id='setup.opponents']", "2")
    check(canvas_drawn(page), "the chosen ape is drawn on the setup screen")

    # The race: countdown, the HUD, then running
    page.click("[id='menu.start']")
    check(page.evaluate("document.body.dataset.screen") == "race", "Start begins the race")
    page.wait_for_function("() => document.getElementById('hud-place').innerText.includes('/3')", timeout=10000)
    check("/3" in text(page, "#hud-place"), "three apes race: the player and two AI")
    page.wait_for_function(
        "() => document.querySelector('#hud-time .v').innerText.trim() !== '0:00.00'",
        timeout=30000,
    )
    page.keyboard.down("KeyW")
    page.wait_for_function(
        "() => document.getElementById('speedo-value').innerText.trim() !== '0'",
        timeout=20000,
    )
    page.keyboard.up("KeyW")
    check(True, "holding W moves the ape after Go")
    check(canvas_drawn(page), "the race is drawn")

    # Esc pauses; Quit goes back to the title
    page.keyboard.press("Escape")
    page.wait_for_selector("[id='menu.resume']", timeout=5000)
    check(STRINGS["menu.paused"].upper() in text(page, "#menus").upper(), "Esc pauses the race")

    # Watch the race so far (TRIAGE T15): the viewer and its seek bar, then
    # back to the paused race
    page.click("[id='menu.replay']")
    page.wait_for_selector("#replay-bar:not([hidden])", timeout=5000)
    check(page.is_visible("[id='replay.exit']"), "Watch Replay opens the replay viewer")
    check(STRINGS["replay.heading"].upper() in text(page, "#menus").upper(), "the viewer says Replay")
    page.select_option("[id='replay.speed']", "double")
    page.select_option("[id='replay.camera']", "orbit")
    page.wait_for_function(
        "() => document.querySelector('#hud-time .v').innerText.trim() !== '0:00.00'",
        timeout=20000,
    )
    check(True, "the replay plays past the countdown")
    check(canvas_drawn(page), "the replay is drawn (orbit camera)")
    page.fill("#replay-seek", "0")
    page.dispatch_event("#replay-seek", "change")
    page.wait_for_function(
        "() => document.querySelector('#hud-time .v').innerText.trim() === '0:00.00'",
        timeout=10000,
    )
    check(True, "seeking to the start shows the countdown again")
    page.click("[id='replay.exit']")
    page.wait_for_selector("[id='menu.resume']", timeout=5000)
    check(page.is_hidden("#replay-bar"), "leaving the replay goes back to the paused race")
    page.click("[id='menu.quit']")
    check(page.evaluate("document.body.dataset.screen") == "title", "Quit returns to the title")

    # Settings were saved: reload and look
    page.reload()
    page.wait_for_selector("body[data-ready='true']", timeout=20000)
    page.click("[id='menu.settings']")
    check("35%" in text(page, "#menus"), "the volume is kept after a reload")
    check(STRINGS["settings.speed_colours"] not in text(page, "#menus"), "JOJO MODE (speed colours) is a secret at first")
    page.click("[id='menu.back']")

    # The secret (TRIAGE row 26): ten clicks on the title unlock JOJO MODE (the speed colours, T36)
    for _ in range(10):
        page.click("[id='title.click']")
    page.click("[id='menu.settings']")
    check(STRINGS["settings.speed_colours"] in text(page, "#menus"), "ten clicks on the title unlock JOJO MODE in Settings")
    page.click("[id='menu.back']")

    # Time Trial (TRIAGE T1, T13): solo, the medal targets, Bronze 1:40.00
    # (105 s, set in trial.rs from the AI's 73–76 s) and the Author hidden
    page.click("[id='menu.trial']")
    check(page.evaluate("document.body.dataset.screen") == "setup", "Time Trial opens its setup")
    shown = text(page, "#menus")
    check(STRINGS["trial.heading"].upper() in shown.upper(), "the setup is headed Time Trial")
    check("1:40.00" in shown and STRINGS["medal.bronze"] in shown, "the Bronze target is 1:40.00")
    check(STRINGS["trial.hidden"] in shown, "the Author time is hidden")
    check(page.query_selector("[id='setup.opponents']") is None, "no AI opponents in a Time Trial")
    page.click("[id='menu.start']")
    page.wait_for_function("() => document.getElementById('hud-place').innerText.includes('/1')", timeout=10000)
    check("1:40.00" in text(page, "#hud-best"), "the HUD aims for Bronze first")

    # R twice quickly restarts (TRIAGE T7): the clock goes back to the countdown
    page.wait_for_function(
        "() => document.querySelector('#hud-time .v').innerText.trim() !== '0:00.00'",
        timeout=30000,
    )
    page.keyboard.press("KeyR")
    page.keyboard.press("KeyR")
    page.wait_for_timeout(200)
    check(text(page, "#hud-time .v") == "0:00.00", "R R restarts the Time Trial")
    check(page.evaluate("document.body.dataset.screen") == "race", "without leaving the race")
    page.keyboard.press("Escape")
    page.wait_for_selector("[id='menu.quit']", timeout=5000)
    page.click("[id='menu.quit']")

    # Replays: nothing finished yet, so the list says so
    page.click("[id='menu.replays']")
    check(STRINGS["replays.none"] in text(page, "#menus"), "the Replays list is empty before a finish")
    page.click("[id='menu.back']")

    # The City (2026-10-12): the map grid on Race Setup (TRIAGE T54), the
    # ape preview moves onto the chosen map, and the race runs there
    page.click("[id='menu.race']")
    options = page.eval_on_selector_all("[id='setup.map'] .tile", "ts => ts.map(t => t.innerText.trim())")
    maps = [STRINGS["map.jungle"], STRINGS["map.city"], STRINGS["map.ski"], STRINGS["map.sector"], STRINGS["map.random"]]
    check(options == maps, f"Race Setup offers the Jungle, the City, the Ski Slope, Sector Ape and Random as cards {options}")
    pictures = page.eval_on_selector_all(
        "[id='setup.map'] .tile-hero img", "is => is.map(i => i.complete && i.naturalWidth > 0)"
    )
    check(pictures == [True], f"the chosen map shows its picture large {pictures}")
    check(chosen_map(page) == ["jungle"], "the Jungle's card is marked at first")
    page.click("[id='setup.map.city']")
    page.wait_for_timeout(300)
    check(chosen_map(page) == ["city"], "the City is chosen by its card")
    # the keyboard: arrows move between cards, Enter presses one
    page.focus("[id='setup.map.city']")
    page.keyboard.press("ArrowRight")
    page.keyboard.press("Enter")
    page.wait_for_timeout(300)
    check(chosen_map(page) == ["ski"], "the arrow and Enter keys choose the Ski Slope")
    page.keyboard.press("ArrowLeft")
    page.keyboard.press("Space")
    page.wait_for_timeout(300)
    check(chosen_map(page) == ["city"], "and back to the City")
    check(canvas_drawn(page), "the ape is drawn on the City behind the setup")
    page.click("[id='menu.start']")
    page.wait_for_function("() => document.getElementById('hud-place').innerText.includes('/')", timeout=10000)
    check(STRINGS["map.city"] in text(page, "#hud-map"), "the HUD names the City")
    page.wait_for_function(
        "() => document.querySelector('#hud-time .v').innerText.trim() !== '0:00.00'",
        timeout=30000,
    )
    page.keyboard.down("KeyW")
    page.wait_for_function(
        "() => document.getElementById('speedo-value').innerText.trim() !== '0'",
        timeout=20000,
    )
    page.keyboard.up("KeyW")
    check(canvas_drawn(page), "the City race runs and is drawn")
    page.keyboard.press("Escape")
    page.wait_for_selector("[id='menu.quit']", timeout=5000)
    page.click("[id='menu.quit']")
    # the map is remembered: after a reload the Time Trial is set to the City
    page.reload()
    page.wait_for_selector("body[data-ready='true']", timeout=20000)
    check(canvas_drawn(page), "the title flies over the last chosen map")
    page.click("[id='menu.trial']")
    check(chosen_map(page) == ["city"], "the City is kept after a reload")
    page.click("[id='menu.back']")

    # The Ski Slope's weather (2026-10-13): chosen in Settings, kept after a
    # reload, and shown under the map on the Ski Slope's setup only
    page.click("[id='menu.settings']")
    page.select_option("[id='settings.weather']", "snowing")
    page.click("[id='menu.back']")
    page.reload()
    page.wait_for_selector("body[data-ready='true']", timeout=20000)
    page.click("[id='menu.race']")
    check(page.query_selector("[id='setup.weather']") is None, "no Weather on the City's setup")
    page.click("[id='setup.map.ski']")
    page.wait_for_selector("[id='setup.weather']", timeout=5000)
    check(page.input_value("[id='setup.weather']") == "snowing", "the weather is kept after a reload")
    check(canvas_drawn(page), "the Ski Slope is drawn in the snow behind the setup")
    page.click("[id='menu.back']")

    # TRIAGE T70, T81: Race Setup fits common desktop windows (the default
    # 1280x720, 1366x768, 1920x1080) with no scrollbar, and Start is in view
    # and takes the click at its middle
    page.click("[id='menu.race']")
    for w, h in [(1280, 720), (1366, 768), (1920, 1080)]:
        page.set_viewport_size({"width": w, "height": h})
        page.wait_for_timeout(300)
        fit = page.evaluate(FIT)
        check(fit["column"] and fit["panels"], f"{w}x{h}: Race Setup fits without a scrollbar {fit}")
        check(fit["start"], f"{w}x{h}: Start is in view and takes a click {fit}")
    # a window too short for the panel: it scrolls inside, Start stays in view
    page.set_viewport_size({"width": 1280, "height": 560})
    page.wait_for_timeout(300)
    fit = page.evaluate(FIT)
    check(fit["start"], f"1280x560: Start stays in view while the panel scrolls {fit}")
    page.set_viewport_size({"width": 1280, "height": 720})
    page.wait_for_timeout(300)

    # TRIAGE T65: a change to the menus keeps every element that is still
    # there (a rebuilt element under the mouse loses the click, and a
    # rebuilt select closes while open): mark some, change a setting, look
    page.evaluate(MARK, ["setup.ai_level", "setup.item.banana", "setup.ape.next", "menu.start"])
    page.click("[id='setup.item.banana']")
    page.wait_for_timeout(200)
    check(not page.is_checked("[id='setup.item.banana']"), "the Banana box unticks")
    kept = page.evaluate(MARKED, ["setup.ai_level", "setup.item.banana", "setup.ape.next", "menu.start"])
    check(all(kept), f"the menus' elements are kept, not rebuilt, when a setting changes {kept}")
    page.click("[id='setup.item.banana']")
    page.wait_for_timeout(200)
    check(page.is_checked("[id='setup.item.banana']"), "and ticks again with one click")

    # TRIAGE T64: Random, the last card; the race runs on a map it picks
    # (never the last one, the Ski Slope) and names it at the start
    page.click("[id='setup.map.random']")
    page.wait_for_timeout(300)
    check(chosen_map(page) == ["random"], "Random is chosen by its card")
    hero = page.eval_on_selector(
        "[id='setup.map'] .tile-hero", "f => [f.innerText.trim(), f.querySelector('img').complete && f.querySelector('img').naturalWidth > 0]"
    )
    check(hero == [STRINGS["map.random"], True], f"Random shows its own picture large {hero}")
    page.click("[id='menu.start']")
    page.wait_for_function("() => document.getElementById('hud-place').innerText.includes('/')", timeout=10000)
    page.wait_for_function("() => document.getElementById('hud-reward').innerText.trim() !== ''", timeout=10000)
    on = text(page, "#hud-map .v")
    check(on in maps[:-1] and on != STRINGS["map.ski"], f"the race runs on a map Random picked, not the last ({on})")
    told = text(page, "#hud-reward")
    check(told == STRINGS["hud.random_map"] + ": " + on, f"the start names the picked map ({told})")
    page.keyboard.press("Escape")
    page.wait_for_selector("[id='menu.quit']", timeout=5000)
    page.click("[id='menu.quit']")
    page.click("[id='menu.race']")
    check(chosen_map(page) == ["random"], "Random stays chosen after the race")
    page.click("[id='menu.back']")


# Whether the menus fit: the column has no scrollbar, no panel scrolls inside,
# and Start is in view with nothing over its middle.
FIT = """() => {
  const m = document.getElementById('menus');
  const panels = [...m.querySelectorAll('.panel')];
  const s = document.querySelector("[id='menu.start']");
  const r = s.getBoundingClientRect();
  const hit = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2);
  return {
    column: m.scrollHeight <= m.clientHeight && m.scrollWidth <= m.clientWidth,
    panels: panels.every(p => p.scrollHeight <= p.clientHeight),
    start: r.top >= 0 && r.bottom <= innerHeight && hit === s,
    sizes: panels.map(p => [p.scrollHeight, p.clientHeight]),
  };
}"""
MARK = "ids => ids.forEach(id => { document.getElementById(id).__mark = id; })"
MARKED = "ids => ids.map(id => document.getElementById(id).__mark === id)"
