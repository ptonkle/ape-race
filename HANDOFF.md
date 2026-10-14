# Handoff

Stage 9, written by the agent. `C5 Consolidate in your own words`.

A fresh session starts from this file, not from memory.

## Start here (end of session, 2026-10-13)

- **Live:** https://ptonkle.github.io/ape-race/ is main (a946b85); everything on main is deployed. Last checks before the last deploy: `check.sh` and `check-online.sh` passed in Chromium, Firefox and WebKit. Main is clean, no agents are running, and the only other branch is `todo/time-trial` (old, superseded; safe to delete).
- **Two maps:** the Jungle and the City (New York at dusk), picked on Race Setup, Time Trial and the lobby (`map.rs`). The City has sharp 90°/45° corners swung on short high cables, driving cars and boats as bouncers, a river you swim out of, its own AI handicaps and medals (Author 1:04, Gold 1:11, Silver 1:21, Bronze 1:36), an orange sunset with a sun path on the river, glowing cyan wires and a muted backdrop. Read **MAPS.md** before making another map.
- **Also built this session:** TRIAGE T22–T37, T40, T42–T48 (spectating, online replays, item checkboxes, hype call-outs, 2× apes, catch-up vines, banana arcs, rename and screech in the lobby, menus docked to the side, JOJO MODE, Floor is Lava with a jaguar). The jaguar is a surprise: the setting says only "Floor is Lava". The hype screen flash is off (`hype::SCREEN_FLASH`, I2).
- **Open todos** (TRIAGE "Future ideas", parked: build only when the person asks): T38 climb from rising lava, T39 co-op checkpoints / Harambe Mode, T41 swing for distance, T49 ape colour palettes, T50 downhill map, T51 uphill map (a perturbed helix; setting still open), T52 online vines held by opponents (as a `/try`), T53 a celebratory dance per ape at the finish (no sad slumps), T54 a map grid instead of the dropdown.
- **Small follow-ups noticed, not done:** the home-page card text (in `../ptonkle.github.io/index.html`) still says "3D jungle vine-swinging race"; the river's sun sheen stays orange in colourblind mode (on purpose; could go grey); the person should check the yellow in-reach highlight against the City's orange in play; places can flicker near the inside of 90° corners (C17).
- **Waiting on the person:** their screech sound file (L4; a synthesised placeholder plays), a listen to the hype call-outs (E7) and the sounds (S6), the room-code path by hand (N11).
- **How the person works** (also in ~/.claude/CLAUDE.md and memory): `/todo` = append a TRIAGE row and commit, don't build; a follow-up adding detail to a parked todo amends that row, it is not a go-ahead; `/try` = build on a `try/` branch and deploy so one step reverts it; anything else = do it now and **deploy at once** (this overrides CLAUDE.md rule 8 for this person). Ship now and hand blocking follow-ups to a background agent in a worktree. Desktop only: no phone-layout work. They play live, so keep the feedback-change-deploy loop fast and report in plain words.
- **Deploy:** `./scripts/check.sh` and `./scripts/check-online.sh` (log to a file, check the exit codes), then `TILES=0 ./scripts/pages.sh` (without `TILES=0` it remakes the 21 home-page shots, a few minutes), then commit in `../ptonkle.github.io` as `git -c user.name=ptonk -c user.email=65458712+ptonkle@users.noreply.github.com commit` and push.
- **Feel is the person's:** TRIAGE rows 1–27 are their play feedback and override PLAN.md P4–P6. Current physics: hold left click to catch (reach 3.6–5.2 m), Space jumps, W drives along the swing (fades with speed), a clean grab keeps all speed, gravity 17, ropes slide out to 18 m.
- **After any physics or course change:** `cargo test --release -p tool_core --test ai --test city_ai` (AI levels ordered and finishing on both maps), `--test jungle_unchanged` (the Jungle stays byte-identical), and `examples/frame_cost.rs -- <map>` for step cost.

## Deployed (stage 8, 2026-10-11, by the person's decision)

- **Live:** https://ptonkle.github.io/ape-race/, with a card (the six-ape lineup) on https://ptonkle.github.io/.
- **How:** `./scripts/pages.sh` builds the game into `~/slop/ptonkle.github.io/ape-race/` and screenshots the lineup into `img/ape-race/lineup.png`. Then commit and push in that folder. Site commits use author `ptonk <65458712+ptonkle@users.noreply.github.com>`. Only the built game is published; the source stays local.
- **Privacy:** `build.sh` maps the home folder to `~` in paths baked into the wasm; the published files hold no personal names.
- **Updates since:** the person's play feedback is TRIAGE rows 1–12 (denser vines in lanes, left-click hold-to-catch with double reach, W drives the swing with no pumping, equal restarts, 1–2 s swings on long vines, bending 8-link vines, a banana speedometer, a forward-facing camera, easy first vines). Each was redeployed. The AI landed (70–74 s a race) and three AI opponents are the default again.

## What passed

- Lobby ready branch `todo/lobby-ready` (TRIAGE T56 ready after a rematch, T57 screech button hidden, T61 setting changes keep Ready; no PROTOCOL change), 2026-10-13: `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings` clean; `cargo test --release --workspace` 331 core, 4 AI (5 ignored), 13 boundary, 3 City AI (2 ignored), 2 Jungle-unchanged, 20 online, 3 Ski AI (2 ignored); `BUILDER_ENGINES=chromium ./scripts/check.sh` BUILDER CHECK PASSED; `BUILDER_ENGINES=chromium ./scripts/check-online.sh` ONLINE CHECK PASSED (Chromium only). SECOND-ORDER LR1–LR5.
- Ski Slope art branch `ski-art` (group B: noon look, sky dome and far-haze shader modes, snow, fences, forest, lifts, landmarks, pink rope vines, minimap branches; ski-course and ski-gear merged in), 2026-10-13: `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings` clean, `cargo test --workspace` (329 core) and `--test jungle_unchanged` pass, `BUILDER_ENGINES=chromium ./scripts/check.sh` BUILDER CHECK PASSED (Chromium only). SECOND-ORDER B1-B14.
- Ski Slope gear and weather branch `ski-gear` (group X: skis, helmets and goggles on the Ski Slope; Weather Clear/Snowing; PROTOCOL 6), 2026-10-13: `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings` clean; `cargo test --release --workspace` 309 core, 4 AI (5 ignored), 13 boundary, 3 City AI (2 ignored), 2 Jungle-unchanged, 19 online; `BUILDER_ENGINES=chromium ./scripts/check.sh` BUILDER CHECK PASSED; `BUILDER_ENGINES=chromium ./scripts/check-online.sh` ONLINE CHECK PASSED (Chromium only). SECOND-ORDER X1–X9.

- City course branch (group C: layout, water, moving bouncers, AI and medals per map; main merged in), 2026-10-12: `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace` (287 core, 4 Jungle AI, 3 City AI, 2 Jungle-unchanged, 13 boundary, 17 online), `./scripts/build.sh`, and `BUILDER_ENGINES=chromium ./scripts/check.sh`: BUILDER CHECK PASSED (Chromium only). City AI and medals: SECOND-ORDER C11–C13.

- Group Y branch (the City's drawing, merged with main and the City course branch at 76f2279), 2026-10-12: `BUILDER_ENGINES=chromium ./scripts/check.sh` BUILDER CHECK PASSED (Chromium only); `cargo clippy --workspace --all-targets -D warnings` clean; `cargo test --workspace` 250 core, 4 AI (5 ignored), 13 boundary, 2 jungle-unchanged, 16 online.

- Group U branch (choosing a map: Jungle/City selector, per-map courses, scenery, bests, medals, ghost, replays, PROTOCOL 5; merged with main at b51e67a), 2026-10-12: `cargo test --workspace` 250 core, 4 AI (5 ignored), 13 boundary, 17 online; `BUILDER_ENGINES=chromium ./scripts/check.sh` BUILDER CHECK PASSED; `BUILDER_ENGINES=chromium ./scripts/check-online.sh` ONLINE CHECK PASSED (Chromium only).
- Home-page tiles branch (H1–H5), 2026-10-12: `cargo test --workspace` 241 core, 4 AI (5 ignored), 13 boundary, 16 online; `cargo clippy --workspace --all-targets -D warnings` clean; `BUILDER_ENGINES=chromium ./scripts/check.sh` BUILDER CHECK PASSED (Chromium only). `scripts/tiles.py` made the 15 Jungle tiles (`img/ape-race/race-01…15.webp`, 35–73 KB); `pages.sh` now regenerates them (about two minutes; `TILES=0` skips).
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

- The open TRIAGE todos and the person's items listed in "Start here".
- **No TURN server** (N2): two players both behind strict NATs cannot connect online.
- Older rows below this point in the file are the history of earlier stages; "Start here" is the current state.

## What the next session must not assume

- Identifiers banned in the core (`time`, `net`, `env`, `io`, `rand`, …) are banned even as field names.
- Page JavaScript holds no number other than 0 and 1. Fractional values reach it as JSON numbers the core writes from text (`app::decimal`).
- Do not request pointer lock from a menu click. It stalls headless Chromium (R2).
- `.gitattributes` merges SECOND-ORDER.md as a union, for parallel agents.
- PLAN.md P4–P6 describe the first grab and pump design; the person replaced it (SECOND-ORDER C3, TRIAGE rows 2, 6, 8–10). TRIAGE.md is the current word on feel.
- `scripts/shot.py` screenshots the built game (`--hold KeyW,Mouse:1.5`); `examples/swing_time.rs` measures swing length; `cargo test --release -p tool_core --test ai -- --ignored --nocapture` reports AI finish times and speeds.
