# Ape Race

A 3D jungle racing game in the browser. Swing from vine to vine, let go at the right moment, and carry your momentum to the finish. Race up to three AI apes, or up to four players online.

The design is in [`BRIEF.md`](BRIEF.md) and the build decisions in [`PLAN.md`](PLAN.md). Future ideas wait in [`TRIAGE.md`](TRIAGE.md).

## How to play

| Goal | How |
| --- | --- |
| Win | Reach the finish arch first. The course runs about 1.8 km from the start tree and turns both ways. |
| Swing | Press and **hold the left mouse button** near a vine to grab it. Let go to fly. Speed is kept: a grab redirects your speed along the swing, and letting go on the way up adds a little more. |
| Drive | Hold **W** on a vine to drive forward along the swing (A/D steer it). The drive is strongest when slow and fades as you speed up, so at speed your line and when you let go count. You never need to swing back and forth. |
| Grab timing | A press looks for a vine for a moment. If none comes into reach, it is a **Missed Grab** and Space cannot grab again for a short cooldown (the centre dot turns red). Mashing Space does not work. |
| The ground | Fall and you run (slowly). **Space** jumps; click to grab a low vine from the jump. Low vines hang every few stations. |
| Items | Fly through a crate to take an item (one at a time). **Right click** or **E** uses it. **Banana:** thrown where you look; a hit stuns and slows. **Stick:** whacks any ape next to you off its vine. **Hook:** pulls back the nearest ape ahead and pulls you forward. **Speed Boost:** a burst along your motion. |
| Apes | Orangutan, Gorilla, Chimpanzee, Gibbon, Proboscis Monkey, Lemur. Five stats: Swing Power, Top Speed, Reach, Air Control, Weight. Every ape's stats add up to the same total. **Equal Stats** gives every ape the average. |
| Online | **Online → Host** gives a room code to share; others **Join** with it. Everyone picks an ape and readies up; the host adds AI apes and starts. |
| Best time | Your fastest finish is kept on this device and shown in the HUD and results. |

### Controls

| Action | Key |
| --- | --- |
| Swing, steer | W A S D (or the arrow keys) |
| Grab (hold) | Left mouse button |
| Jump (from the ground) | Space |
| Use item | Right click or E |
| Look | Mouse (click the 3D view to capture the mouse) |
| Pause | Esc (online, Esc shows the menu but the race goes on) |

Settings: Volume, Mouse Sensitivity, Invert Mouse Y, Camera Follows (the camera swings back behind you when the mouse rests). They are kept in browser storage.

## Build, test and serve

```bash
./scripts/setup.sh          # once: Python venv with Playwright and three browsers
cargo test --workspace      # the core, boundary and online tests
./scripts/build.sh          # release WebAssembly build into dist/
python3 scripts/serve.py    # http://127.0.0.1:8000/
./scripts/check.sh          # everything, before a deploy
./scripts/check-online.sh   # WebRTC between tabs (pasted codes, no relay)
./scripts/pages.sh          # assemble the site copy and thumbnail in ../ptonkle.github.io
.venv/bin/python scripts/shot.py out.png [--query lineup] [--click id ...] [--hold KeyW,Space:1.5 ...]
```

Needs Rust with the `wasm32-unknown-unknown` target, `wasm-bindgen-cli` 0.2.127, and Python 3.

## Layout

| Path | Holds |
| --- | --- |
| `crates/core/src/fx.rs`, `geom.rs` | Q16.16 fixed point, integer trigonometry, vectors and matrices (the core has no floats) |
| `crates/core/src/ape.rs` | one ape's physics: swinging on a rope, flying, running, grabs, the Grab Cooldown |
| `crates/core/src/rules.rs` | every tuning number, and the six apes' stats |
| `crates/core/src/course.rs` | the jungle course: centreline, vines, logs, crates, platforms, scenery placement |
| `crates/core/src/race.rs`, `items.rs`, `ai.rs` | the race loop, items, and the AI driver |
| `crates/core/src/camera.rs`, `scene.rs`, `models.rs`, `mesh.rs`, `render.rs` | the chase camera, the drawn world, the six ape models, unit meshes, the instance-draw contract |
| `crates/core/src/sound.rs` | every sound, synthesised by the core as a WAV file |
| `crates/core/src/online.rs`, `app/online_play.rs` | online play: lobby, state sync, hits and results; its wiring into the app |
| `crates/core/src/app.rs`, `api.rs` | screens, settings, input, the view model; the one door the shim calls |
| `crates/shim` | the `wasm-bindgen` layer: one call into the core per method, Q16 → `f32` for WebGL |
| `web/` | the static page: WebGL 2 renderer (`gl.js`), menus and HUD (`app.js`), the WebRTC transport (`online.js`) |
| `data/` | on-screen words (`strings.json`) and the shaders |
| `tests/browser/` | the three-engine browser check, the game's interactions, and the online checks |
