"""The game's own browser interactions, run by check.py in every engine.

check.py has already loaded the page, waited for it to report ready, and let
it idle. This drives the game the way a person would and checks what the
page shows. Expected values are worked out by hand from the brief (C4),
never copied from the game's own output.
"""

import json
import os
import pathlib
import time
import urllib.parse

STRINGS = json.loads((pathlib.Path(__file__).resolve().parents[2] / "data" / "strings.json").read_text())


def text(page, sel):
    return page.inner_text(sel).strip()


def canvas_drawn(page):
    """True when the 3D view holds more than one colour (WebGL drew).

    Sampled inside an animation frame, after the game's own frame callback
    has drawn: on a GPU the drawing buffer is cleared once the frame is
    shown, so a copy taken between frames is blank (it only read back under
    software drawing)."""
    return page.evaluate(
        """() => new Promise(done => requestAnimationFrame(() => {
        const c = document.getElementById('view');
        const probe = document.createElement('canvas');
        probe.width = 16; probe.height = 16;
        const g = probe.getContext('2d');
        g.drawImage(c, 0, 0, 16, 16);
        const d = g.getImageData(0, 0, 16, 16).data;
        const seen = new Set();
        for (let i = 0; i < d.length; i += 4) seen.add(d[i] + ',' + d[i + 1] + ',' + d[i + 2]);
        done(seen.size > 4);
    }))"""
    )


def fast(page):
    """Whether the page runs in the core's fast mode (`?fast`, check.py):
    a race's start countdown is run through at once."""
    return "fast" in urllib.parse.parse_qs(urllib.parse.urlparse(page.url).query, keep_blank_values=True)


def seconds(clock):
    """The HUD's race clock, "m:ss.cc", in seconds."""
    m, s = clock.strip().split(":")
    return int(m) * 60 + float(s)


def chosen_map(page):
    """The keys of the map cards marked chosen on the setup's grid."""
    return page.eval_on_selector_all("[id='setup.map'] .tile.chosen, [id='setup.map'] .card.chosen", "ts => ts.map(t => t.dataset.value)")


MAPS = [STRINGS["map." + k] for k in ["jungle", "city", "ski", "sector", "ruins", "bay", "tower", "ravine", "grocery", "spindle", "random"]]


def step_boot(page, check):
    check(page.title() == STRINGS["title"], "the tab is titled Ape Race")
    check(page.evaluate("document.body.dataset.screen") == "title", "the title screen is up")
    check(STRINGS["title"].upper() in text(page, "#menus").upper(), "the title is drawn")
    check(text(page, "#message") == "", "no error message is shown")
    check(canvas_drawn(page), "the jungle is drawn behind the title (WebGL 2)")


def step_settings(page, check):
    # Settings: change the volume, see it shown, come back
    page.click("[id='menu.settings']")
    check(page.evaluate("document.body.dataset.screen") == "settings", "Settings opens")
    page.fill("[id='settings.volume']", "35")
    page.dispatch_event("[id='settings.volume']", "change")
    page.wait_for_timeout(200)
    check("35%" in text(page, "#menus"), "the volume slider shows 35%")
    page.click("[id='menu.back']")
    # Settings were saved: reload and look
    page.reload()
    page.wait_for_selector("body[data-ready='true']", timeout=20000)
    page.click("[id='menu.settings']")
    check("35%" in text(page, "#menus"), "the volume is kept after a reload")
    page.click("[id='menu.back']")


def step_race(page, check):
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
    # the Hook, the Speed Boost and the Vine are offered (1.17.0)
    check(boxes == [True] * 3, f"three item checkboxes, all ticked {boxes}")
    page.click("[id='setup.item.vine']")
    page.wait_for_timeout(100)
    check(not page.is_checked("[id='setup.item.vine']"), "the Vine box unticks")
    page.click("[id='setup.item.vine']")
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


def step_secret(page, check):
    page.click("[id='menu.settings']")
    check(STRINGS["settings.speed_colours"] not in text(page, "#menus"), "JOJO MODE (speed colours) is a secret at first")
    page.click("[id='menu.back']")
    # The secret (TRIAGE row 26): ten clicks on the title unlock JOJO MODE (the speed colours, T36)
    for _ in range(10):
        page.click("[id='title.click']")
    page.click("[id='menu.settings']")
    check(STRINGS["settings.speed_colours"] in text(page, "#menus"), "ten clicks on the title unlock JOJO MODE in Settings")
    page.click("[id='menu.back']")


def step_time_trial(page, check):
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
    if fast(page):
        # fast mode runs the new countdown through at once, so the clock is
        # seen going back from a second in to the start, not resting there
        page.wait_for_function(
            "() => { const [m, s] = document.querySelector('#hud-time .v').innerText.trim().split(':');"
            " return m * 60 + +s >= 1; }",
            timeout=30000,
        )
        before = seconds(text(page, "#hud-time .v"))
        page.keyboard.press("KeyR")
        page.keyboard.press("KeyR")
        page.wait_for_timeout(200)
        after = seconds(text(page, "#hud-time .v"))
        check(after < 0.5 <= before - 0.5, f"R R restarts the Time Trial (the clock {before} s -> {after} s, fast mode)")
    else:
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


def step_replays_list(page, check):
    # Replays: nothing finished yet, so the list says so
    page.click("[id='menu.replays']")
    check(STRINGS["replays.none"] in text(page, "#menus"), "the Replays list is empty before a finish")
    page.click("[id='menu.back']")


def step_whats_new(page, check):
    # What's New: marked New until opened; major versions only until the
    # minor-patches tick; the version in the corner opens it too
    notes = json.loads((pathlib.Path(__file__).resolve().parents[2] / "data" / "patch-notes.json").read_text())
    check(page.locator("[id='menu.notes'] .badge").count() == 1, "What's New is marked New before it is opened")
    page.click("[id='menu.notes']")
    page.wait_for_selector("[id='notes.list'] .note", timeout=5000)
    shown = page.eval_on_selector_all("[id='notes.list'] .note-version", "vs => vs.map(v => v.innerText.trim())")
    majors = ["v" + e["version"] for e in notes if e["version"].endswith(".0")]
    check(shown == majors, f"What's New lists the major versions, newest first {shown}")
    page.click("[id='notes.minor']")
    page.wait_for_timeout(200)
    shown = page.eval_on_selector_all("[id='notes.list'] .note-version", "vs => vs.map(v => v.innerText.trim())")
    check(shown == ["v" + e["version"] for e in notes], "the minor-patches tick lists every version")
    check(page.locator("[id='notes.list'] .note.minor").count() > 0, "minor versions are drawn as minor")
    page.click("[id='menu.back']")
    check(page.locator("[id='menu.notes'] .badge").count() == 0, "What's New is no longer New once opened")
    page.click("#version")
    page.wait_for_timeout(200)
    check(page.evaluate("document.body.dataset.screen") == "notes", "the version in the corner opens What's New")
    check(page.is_checked("[id='notes.minor']"), "the minor-patches tick is remembered")
    page.click("[id='notes.minor']")
    page.click("[id='menu.back']")


def step_city_map(page, check):
    # The City (2026-10-12): the map grid on Race Setup (TRIAGE T54), the
    # ape preview moves onto the chosen map, and the race runs there
    page.click("[id='menu.race']")
    options = page.eval_on_selector_all("[id='setup.map'] .tile, [id='setup.map'] .card", "ts => ts.map(t => t.innerText.trim())")
    maps = MAPS
    check(options == maps, f"Race Setup offers every map and Random as cards {options}")
    # the cards' pictures load after the cards appear: wait for them to finish
    # (a fast browser read them mid-load, 2026-10-17), then check each one
    try:
        page.wait_for_function(
            "() => [...document.querySelectorAll(\"[id='setup.map'] .tile-hero img, [id='setup.map'] .card img\")].every(i => i.complete)",
            timeout=5000,
        )
    except Exception:
        pass  # the check below names the ones that did not load
    pictures = page.eval_on_selector_all(
        "[id='setup.map'] .tile-hero img, [id='setup.map'] .card img", "is => is.map(i => i.complete && i.naturalWidth > 0)"
    )
    # docked beside the ape, every card shows its picture (TRIAGE T82);
    # otherwise the chosen map's picture shows large
    check(pictures and all(pictures), f"the map pictures load {pictures}")
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


def step_weather(page, check):
    # The Ski Slope's weather: chosen on its setup, kept after a reload, and
    # shown under the map on the Ski Slope's setup only
    page.click("[id='menu.race']")
    check(page.query_selector("[id='setup.weather']") is None, "no Weather on another map's setup")
    page.click("[id='setup.map.ski']")
    page.wait_for_selector("[id='setup.weather']", timeout=5000)
    page.select_option("[id='setup.weather']", "snowing")
    page.reload()
    page.wait_for_selector("body[data-ready='true']", timeout=20000)
    page.click("[id='menu.race']")
    page.wait_for_selector("[id='setup.weather']", timeout=5000)
    check(page.input_value("[id='setup.weather']") == "snowing", "the weather is kept after a reload")
    check(canvas_drawn(page), "the Ski Slope is drawn in the snow behind the setup")
    page.click("[id='menu.back']")


def step_setup_layout(page, check):
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
    page.evaluate(MARK, ["setup.ai_level", "setup.item.vine", "setup.ape.next", "menu.start"])
    page.click("[id='setup.item.vine']")
    page.wait_for_timeout(200)
    check(not page.is_checked("[id='setup.item.vine']"), "the Vine box unticks")
    kept = page.evaluate(MARKED, ["setup.ai_level", "setup.item.vine", "setup.ape.next", "menu.start"])
    check(all(kept), f"the menus' elements are kept, not rebuilt, when a setting changes {kept}")
    page.click("[id='setup.item.vine']")
    page.wait_for_timeout(200)
    check(page.is_checked("[id='setup.item.vine']"), "and ticks again with one click")
    page.click("[id='menu.back']")


def step_random_map(page, check):
    page.click("[id='menu.race']")
    page.click("[id='setup.map.ski']")
    page.wait_for_timeout(300)
    # TRIAGE T64: Random, the last card; the race runs on a map it picks
    # (never the last one, the Ski Slope) and names it at the start
    page.click("[id='setup.map.random']")
    page.wait_for_timeout(300)
    check(chosen_map(page) == ["random"], "Random is chosen by its card")
    hero = page.eval_on_selector(
        "[id='setup.map'] .tile-hero, [id='setup.map'] .card.chosen", "f => [f.innerText.trim(), f.querySelector('img').complete && f.querySelector('img').naturalWidth > 0]"
    )
    check(hero == [STRINGS["map.random"], True], f"Random shows its own picture {hero}")
    page.click("[id='menu.start']")
    page.wait_for_function("() => document.getElementById('hud-place').innerText.includes('/')", timeout=10000)
    page.wait_for_function("() => document.getElementById('hud-reward').innerText.trim() !== ''", timeout=10000)
    on = text(page, "#hud-map .v")
    check(on in MAPS[:-1] and on != STRINGS["map.ski"], f"the race runs on a map Random picked, not the last ({on})")
    told = text(page, "#hud-reward")
    check(told == STRINGS["hud.random_map"] + ": " + on, f"the start names the picked map ({told})")
    page.keyboard.press("Escape")
    page.wait_for_selector("[id='menu.quit']", timeout=5000)
    page.click("[id='menu.quit']")
    page.click("[id='menu.race']")
    check(chosen_map(page) == ["random"], "Random stays chosen after the race")
    page.click("[id='menu.back']")


# The steps in order, by name. Each starts and ends on the title screen, so
# any of them can run alone after `boot`; tests/tags.toml tags each one and
# scripts/quick-check.sh picks them through BUILDER_STEPS.
STEPS = [("boot", step_boot), ("settings", step_settings), ("race", step_race), ("secret", step_secret), ("time_trial", step_time_trial), ("replays_list", step_replays_list), ("whats_new", step_whats_new), ("city_map", step_city_map), ("weather", step_weather), ("setup_layout", step_setup_layout), ("random_map", step_random_map)]


def chosen_steps():
    """The steps this run takes: all, or `boot` and those in BUILDER_STEPS."""
    wanted = os.environ.get("BUILDER_STEPS")
    chosen = None if wanted is None else set(wanted.split()) | {"boot"}
    return [(n, s) for n, s in STEPS if chosen is None or n in chosen]


def run(page, check, engine, report, first=1):
    """Runs the chosen steps; `report` (tests/browser/progress.py) shows each
    one start and end. `first` is the first step's number in the engine's
    count (the check's own load step comes before)."""
    steps = chosen_steps()
    skipped = [n for n, _ in STEPS if n not in dict(steps)]
    if skipped:
        report.note(f"  (skipping {len(skipped)} steps not picked: {', '.join(skipped)})")
    for k, (name, step) in enumerate(steps, first):
        report.step_start(k, name)
        try:
            step(page, check)
        except BaseException:
            report.step_end(error=True)
            raise
        report.step_end()


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
