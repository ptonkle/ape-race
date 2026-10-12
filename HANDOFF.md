# Handoff

Stage 9, written by the agent. `C5 Consolidate in your own words`.

A fresh session starts from this file, not from memory.

## Deployed (stage 8, 2026-10-11, by the person's decision)

- **Live:** https://ptonkle.github.io/ape-race/, with a card (the six-ape lineup) on https://ptonkle.github.io/.
- **How:** `./scripts/pages.sh` builds the game into `~/slop/ptonkle.github.io/ape-race/` and screenshots the lineup into `img/ape-race/lineup.png`. Then commit and push in that folder. Site commits use author `ptonk <65458712+ptonkle@users.noreply.github.com>`. Only the built game is published; the source stays local.
- **Privacy:** `build.sh` maps the home folder to `~` in paths baked into the wasm; the published files hold no personal names.
- **First deploy is solo-first:** the person asked to ship before the AI was done. New players start with AI Opponents 0. The option is still in the menu; until the AI lands, an AI ape only holds W and walks off the start platform (SECOND-ORDER C2).

## What passed

- `./scripts/check.sh`: BUILDER CHECK PASSED, with the browser check passing in Chromium, Firefox and WebKit (game interactions: menus, settings kept after a reload, the ape picker, Equal Stats, a race started, W moves the ape, Esc pauses).
- `./scripts/check-online.sh`: ONLINE CHECK PASSED and ONLINE RACE CHECK PASSED in three engines (two tabs by pasted codes, no relay).
- Live site: `BUILDER_ORIGIN=https://ptonkle.github.io/ape-race .venv/bin/python tests/browser/check.py` passed in three engines.
- `cargo test --workspace`: 106 core, 13 boundary and 12 online tests.

## What was built

The Builder template became a 3D vine-swinging racer. The work was split across parallel agents in git worktrees after a solo skeleton pass. SECOND-ORDER rows carry each agent's letter.

- **Physics** (`ape.rs`, `rules.rs`): a point-mass ape on an energy-conserving rope. It covers Grab Keep, Release Boost, pumping with the motion, Grab Window and Grab Cooldown, the ground, jumps and hits. Fixed 120 ticks per second, all Q16.16 integers.
- **Course** (`course.rs`): a 1.8 km jungle run with bends both ways. It has vine stations alternating sides at different heights, low vines every third station, 3 sliding vines and 2 swinging logs, and crates.
- **Rendering** (`render.rs`, `mesh.rs`, `scene.rs`, `camera.rs`, `web/gl.js`, `data/shaders/`): WebGL 2 instancing of unit meshes. The scenery is chunked by distance. The chase camera widens with speed and shows speed lines.
- **Models** (M): six apes with swing, flight, run and stun poses, a lineup and a spinning showcase.
- **Items** (I): crates weighted by place, plus Banana, Stick, Hook and Speed Boost.
- **Sound** (S): 13 synthesised cues and a seamless wind loop whose gain follows speed.
- **Online** (N): Mandible's WebRTC transport and exemptions, and a host-run lobby. There is no lockstep: each browser runs its own ape and sends its state at 20 Hz. Hits are routed to the victim's browser, and the host publishes results.
- **App:** title, Race setup, Online, lobby, settings, HUD with the minimap, results, a personal best, and pause.

## What remains

- **The AI** (P12): an agent was still building `ai.rs` and tuning the course and physics when the person chose to deploy. Its worktree branch is `worktree-agent-a85089a505766695c`. Merge it, check that an AI finishes in 60–120 s, then redeploy with `pages.sh`.
- **Future ideas** are in `TRIAGE.md` (T1: time trial medals and an Author time; a draft is parked on branch `todo/time-trial`).
- **Feel** (R5, *person*): nobody has played it yet. Every tuning number is in `rules.rs`.
- **Sounds** (S6, *person*): not yet listened to.
- **Room codes** (N11): the room-code path over the public relays is untested by hand. There is no TURN server (N2).

## What the next session must not assume

- Identifiers banned in the core (`time`, `net`, `env`, `io`, `rand`, …) are banned even as field names.
- Page JavaScript holds no number other than 0 and 1. Fractional values reach it as JSON numbers the core writes from text (`app::decimal`).
- Do not request pointer lock from a menu click. It stalls headless Chromium (R2).
- `.gitattributes` merges SECOND-ORDER.md as a union, for parallel agents.
