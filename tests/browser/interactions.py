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
        "() => document.querySelector('#hud-speed .v').innerText.trim().split(' ')[0] !== '0'",
        timeout=20000,
    )
    page.keyboard.up("KeyW")
    check(True, "holding W moves the ape after Go")
    check(canvas_drawn(page), "the race is drawn")

    # Esc pauses; Quit goes back to the title
    page.keyboard.press("Escape")
    page.wait_for_selector("[id='menu.resume']", timeout=5000)
    check(STRINGS["menu.paused"].upper() in text(page, "#menus").upper(), "Esc pauses the race")
    page.click("[id='menu.quit']")
    check(page.evaluate("document.body.dataset.screen") == "title", "Quit returns to the title")

    # Settings were saved: reload and look
    page.reload()
    page.wait_for_selector("body[data-ready='true']", timeout=20000)
    page.click("[id='menu.settings']")
    check("35%" in text(page, "#menus"), "the volume is kept after a reload")
    page.click("[id='menu.back']")
