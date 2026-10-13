# Handoff

Stage 9, written by the agent. `C5 Consolidate in your own words`.

A fresh session starts from this file, not from memory.

## Start here (end of session, 2026-10-12)

- **Live:** https://ptonkle.github.io/ape-race/ is main at the last "Ape Race:" site commit; everything on main is deployed. Last checks (2026-10-12, after merging the five-agent batch L/E/G/P/K for T22–T36, T40, T42–T48): `cargo test`, `check.sh` and `check-online.sh` passed in Chromium, Firefox and WebKit.
- **Open work:** TRIAGE.md "Future ideas" rows still `open` are the four new modes the person held back: T37 (floor is lava: a jaguar eats you, then you spectate; call `focus::racing` / `Focus` from focus.rs), T38 (climb from rising lava, new map), T39 (co-op checkpoints, Harambe Mode, giant-banana revive), T41 (swing for distance). Waiting on the person: their screech sound file (SECOND-ORDER L4), a listen to the hype call-outs (E7), a try for the new Author time 1:08 (K15), real-hardware frame rate with the bigger trees (G). The hype screen flash is off at the person's request (`hype::SCREEN_FLASH`, I2).
- **How the person works** (also in ~/.claude/CLAUDE.md): `/todo` = write it down only; `/try` = experiment on a `try/` branch, keep or revert; anything else = do it now and deploy at once. A follow-up that would block a deploy (e.g. AI retuning after a physics change) goes to a background agent immediately while the change ships.
- **Feel is the person's:** TRIAGE rows 1–27 are their play feedback and override PLAN.md P4–P6. Current physics: hold left click to catch (reach 3.6–5.2 m), Space jumps, W drives along the swing (fades with speed), drag 0.00125, a clean grab keeps all speed and a mistimed one loses half the along-rope part, gravity 17, vines 23–31 m from 38–50 m, ropes slide out to 18 m, landings roll only after falls (not hops).
- **Branches:** only `main`, plus `todo/time-trial` (an old draft, superseded by the merged Time Trial; safe to delete).
- **After any physics or course change:** `cargo test --release -p tool_core --test ai` and `... -- --ignored --nocapture difficulty_report` (AI levels must stay ordered and finishing); `examples/swing_time.rs` measures swings.

## Deployed (stage 8, 2026-10-11, by the person's decision)

- **Live:** https://ptonkle.github.io/ape-race/, with a card (the six-ape lineup) on https://ptonkle.github.io/.
- **How:** `./scripts/pages.sh` builds the game into `~/slop/ptonkle.github.io/ape-race/` and screenshots the lineup into `img/ape-race/lineup.png`. Then commit and push in that folder. Site commits use author `ptonk <65458712+ptonkle@users.noreply.github.com>`. Only the built game is published; the source stays local.
- **Privacy:** `build.sh` maps the home folder to `~` in paths baked into the wasm; the published files hold no personal names.
- **Updates since:** the person's play feedback is TRIAGE rows 1–12 (denser vines in lanes, left-click hold-to-catch with double reach, W drives the swing with no pumping, equal restarts, 1–2 s swings on long vines, bending 8-link vines, a banana speedometer, a forward-facing camera, easy first vines). Each was redeployed. The AI landed (70–74 s a race) and three AI opponents are the default again.

## What passed

- T37 Floor is Lava branch (merged with main at e478e46), 2026-10-12: `cargo test --workspace` 239 core, 4 AI (5 ignored reports; `lava_report` run, SECOND-ORDER V6), 13 boundary, 16 online; `cargo fmt`, `clippy -D warnings` clean; `BUILDER_ENGINES=chromium ./scripts/check.sh` BUILDER CHECK PASSED; `BUILDER_ENGINES=chromium ./scripts/check-online.sh` ONLINE CHECK PASSED (Chromium only).
- Group K branch (T48, T45, T43, T32, T29, merged with main at 060aecc), 2026-10-12: `BUILDER_ENGINES=chromium ./scripts/check.sh` BUILDER CHECK PASSED; `BUILDER_ENGINES=chromium ./scripts/check-online.sh` ONLINE CHECK PASSED (Chromium only); `cargo test --workspace` 223 core, 8 AI (4 ignored reports run too), 13 boundary, 15 online.

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

TRIAGE Future ideas T1–T21 were built by five parallel agents (Art F, World W, Items J, Modes Q, AI D) and the integrator, merged and deployed: Time Trial with medals and a hidden Author time, R R quick restart, replays with save points and a ghost (best per map plus the 10 latest, in browser storage), four AI difficulties, faceted apes, the new jungle, mushrooms and softer landings, crates on vines, a homing banana, a longer stick with its range shown, a Vine item, item icons, a banana tab icon, speed colours (sky, ground, wood and leaves each their own colour, from smoothed horizontal speed; Settings → JOJO MODE, renamed from Speed Colours in T36), and hype levels set from a person's speeds. `check.sh` and `check-online.sh` passed in three engines before the World merge; chromium after it.

Group L (lobby, online and setup UI, 2026-10-12): T26 (1–3 AI opponents), T35 (rename in the lobby), T36 (JOJO MODE), T44 (lobby screech, placeholder sound: SECOND-ORDER L4 for the real file), T47 (menus docked beside the preview ape, core-framed). PROTOCOL is 3. `check.sh` passed in Chromium; `check-online.sh` result in the L-group commit message and report.

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
