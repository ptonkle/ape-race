# Handoff

Stage 9, written by the agent. `C5 Consolidate in your own words`.

A fresh session starts from this file, not from memory.

## Deployed (stage 8, 2026-10-11, by the person's decision)

- **Live:** https://ptonkle.github.io/ape-race/, with a card (the six-ape lineup) on https://ptonkle.github.io/.
- **How:** `./scripts/pages.sh` builds the game into `~/slop/ptonkle.github.io/ape-race/` and screenshots the lineup into `img/ape-race/lineup.png`. Then commit and push in that folder. Site commits use author `ptonk <65458712+ptonkle@users.noreply.github.com>`. Only the built game is published; the source stays local.
- **Privacy:** `build.sh` maps the home folder to `~` in paths baked into the wasm; the published files hold no personal names.
- **Updates since:** the person's play feedback is TRIAGE rows 1–12 (denser vines in lanes, left-click hold-to-catch with double reach, W drives the swing with no pumping, equal restarts, 1–2 s swings on long vines, bending 8-link vines, a banana speedometer, a forward-facing camera, easy first vines). Each was redeployed. The AI landed (70–74 s a race) and three AI opponents are the default again.

## What passed

- `./scripts/check.sh`: BUILDER CHECK PASSED, with the browser check passing in Chromium, Firefox and WebKit (game interactions: menus, settings kept after a reload, the ape picker, Equal Stats, a race started, W moves the ape, Esc pauses).
- `./scripts/check-online.sh`: ONLINE CHECK PASSED and ONLINE RACE CHECK PASSED in three engines (two tabs by pasted codes, no relay).
- Live site: `BUILDER_ORIGIN=https://ptonkle.github.io/ape-race .venv/bin/python tests/browser/check.py` passed in three engines.
- `cargo test --workspace`: 106 core, 13 boundary and 12 online tests.

## What was built

The Builder template became a 3D vine-swinging racer. The work was split across parallel agents in git worktrees after a solo skeleton pass. SECOND-ORDER rows carry each agent's letter.

- **Physics** (`ape.rs`, `rules.rs`): a point-mass ape on an energy-conserving rope. It covers Grab Keep, Release Boost, pumping with the motion, Grab Window and Grab Cooldown, the ground, jumps and hits. Fixed 120 ticks per second, all Q16.16 integers.
- **Course** (`course.rs`): a 1.8 km jungle run with bends both ways: 193 vines in stations every 22–28 m across lanes, low vines to climb back up, 3 sliding vines, 18 bounce mushrooms, six tree kinds, streams, valley walls, and an outro past the finish (podium, waterfall). The logs are gone (T17).
- **Rendering** (`render.rs`, `mesh.rs`, `scene.rs`, `camera.rs`, `web/gl.js`, `data/shaders/`): WebGL 2 instancing of unit meshes. The scenery is chunked by distance. The chase camera widens with speed and shows speed lines.
- **Models** (M): six apes with swing, flight, run and stun poses, a lineup and a spinning showcase.
- **Items** (I): crates weighted by place, plus Banana, Stick, Hook and Speed Boost.
- **Sound** (S): 13 synthesised cues and a seamless wind loop whose gain follows speed.
- **Online** (N): Mandible's WebRTC transport and exemptions, and a host-run lobby. There is no lockstep: each browser runs its own ape and sends its state at 20 Hz. Hits are routed to the victim's browser, and the host publishes results.
- **App:** title, Race setup, Online, lobby, settings, HUD with the minimap, results, a personal best, and pause.

## The todo batch (2026-10-12)

TRIAGE Future ideas T1–T21 were built by five parallel agents (Art F, World W, Items J, Modes Q, AI D) and the integrator, merged and deployed: Time Trial with medals and a hidden Author time, R R quick restart, replays with save points and a ghost (best per map plus the 10 latest, in browser storage), four AI difficulties, faceted apes, the new jungle, mushrooms and softer landings, crates on vines, a homing banana, a longer stick with its range shown, a Vine item, item icons, a banana tab icon, speed colours (sky, ground, wood and leaves each their own colour, from smoothed horizontal speed; Settings → Speed Colours), and hype levels set from a person's speeds. `check.sh` and `check-online.sh` passed in three engines before the World merge; chromium after it.

## What remains

- **Sloth AI** (SECOND-ORDER D rows): on the new course the easiest level sometimes fails to finish in some seeded layouts (the jungle itself passes its test); the AI agent is fixing it. T22 (voice call-outs) is still open.

- **AI difficulty** (SECOND-ORDER A3, *person*): one difficulty; the AI is consistent and may beat new players. A handicap (a release delay) is the place to add levels.
- **Future ideas** are in `TRIAGE.md` (T1: time trial medals and an Author time; a draft is parked on branch `todo/time-trial`).
- **Feel** (R5, *person*): nobody has played it yet. Every tuning number is in `rules.rs`.
- **Sounds** (S6, *person*): not yet listened to.
- **Room codes** (N11): the room-code path over the public relays is untested by hand. There is no TURN server (N2).

## What the next session must not assume

- Identifiers banned in the core (`time`, `net`, `env`, `io`, `rand`, …) are banned even as field names.
- Page JavaScript holds no number other than 0 and 1. Fractional values reach it as JSON numbers the core writes from text (`app::decimal`).
- Do not request pointer lock from a menu click. It stalls headless Chromium (R2).
- `.gitattributes` merges SECOND-ORDER.md as a union, for parallel agents.
- PLAN.md P4–P6 describe the first grab and pump design; the person replaced it (SECOND-ORDER C3, TRIAGE rows 2, 6, 8–10). TRIAGE.md is the current word on feel.
- `scripts/shot.py` screenshots the built game (`--hold KeyW,Mouse:1.5`); `examples/swing_time.rs` measures swing length; `cargo test --release -p tool_core --test ai -- --ignored --nocapture` reports AI finish times and speeds.
