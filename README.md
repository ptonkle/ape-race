# Ape Race

A 3D vine-swinging race in the browser, on seven maps: **Canopy Run** (the jungle, morning), **Concrete Jungle** (New York at dusk), **Yeti Peak** (a ski slope at noon), **Sector Ape** (open space after Star Fox, lit by a sun far below), **Sunset Ruins** (a jungle river and an ape temple at sunset), **Banana Bay Battle** (a pirate battle in a Caribbean bay at 3 pm) and **Mape's Tower** (two wizard's towers, up the inside of one and down a tube through the void). Swing from vine to vine, let go at the right moment, and carry your momentum to the finish. Race one to three AI apes, or up to four players online. Played with a mouse and keyboard (desktop only).

**Play:** https://ptonkle.github.io/ape-race/

MIT licence ([LICENSE](LICENSE), credits in [LICENSES.md](LICENSES.md)). Every push to `main` builds and deploys to GitHub Pages (`.github/workflows/deploy.yml`); the checks run locally before the push (`scripts/timed-checks.sh quick`, or the full `scripts/check.sh` for a map merge).

The design is in [`BRIEF.md`](BRIEF.md) and the build decisions in [`PLAN.md`](PLAN.md). How to make a new map is in [`MAPS.md`](MAPS.md). Future ideas wait in [`TRIAGE.md`](TRIAGE.md).

## How to play

| Goal | How |
| --- | --- |
| Win | Reach the finish arch first. The Jungle runs about 1.8 km from the start tree and turns both ways. |
| Maps | Pick **Canopy Run**, **Concrete Jungle**, **Yeti Peak**, **Sector Ape**, **Sunset Ruins**, **Banana Bay Battle** or **Mape's Tower** on Race Setup, Time Trial, or (the host) in the online lobby. Bests, medals, the ghost and replays are kept per map. On Sector Ape the vines are energy tethers hanging every way (sideways, up, at angles) from ships and debris, the speedometer reads to 300 km/h, and with Floor is Lava the space jaguar pops out of nowhere when you cross into open space. On Banana Bay Battle you swing on ropes from the masts and yards of 30 ships at anchor (galleons, frigates, brigs and sloops, pirate and navy) firing on each other, between flagged barrels, to a chest of golden bananas on an island; fall in the sea and you are put back on the last ship you passed, and item vines carry doubloons. |
| Swing | Press and **hold the left mouse button** near a vine to grab it. Let go to fly. Speed is kept: a grab redirects your speed along the swing, and letting go on the way up adds a little more. |
| Drive | Hold **W** on a vine to drive forward along the swing (A/D steer it). The drive is strongest when slow and fades as you speed up, so at speed your line and when you let go count. You never need to swing back and forth. |
| Catching | Hold the grab button and you catch the first vine that comes within reach (3.6–5.2 m, by ape). No timing needed. |
| The ground | A landing slides on and keeps some of its speed. Red **mushrooms** bounce you back up and push you forward: a bounce is a small speed boost. Fall and you run (slowly). **Space** jumps; click to grab a low vine from the jump. Low vines hang every few stations. |
| The City | New York at dusk: the vines are cables, and the streets turn sharply (90° and 45°). At a 90° corner, catch the short cable high over the crossing (a crane's or a helicopter's): it whirls you round, so let go facing down the new street. Cars and cabs (some driving), and boats on the river, bounce like mushrooms. Fall off the bridge and you swim to the nearest ladder, bank or boat and climb out (in Floor is Lava the river counts as the floor). |
| Sector Ape | Open space after Star Fox, about 2.3 km to a black hole whose event horizon of bananas is the finish. **Gravity is weak** (9 m/s², against 17), so a hard release blasts you off into space. Half the vines hang **some other way**: sideways, up from debris below, or slanted; catch one anywhere along it and you swing round its anchor. Driving an odd vine (W) keeps adding speed far longer, and letting go of one lined up with the way on is a **slingshot**: well timed, you go faster than on any other map (up to about 300 km/h). Your reach is half as long again and you steer twice as hard in the air here. **Touching debris** throws you off it with a push forward (once a touch; holding a vine, the swing bounces off it). Fly out past the invisible edge and you are put back, standing, on the **nearest deck** (with Floor is Lava, the space jaguar gets you instead). Three ways through the **Hulk**: its corridor (swing the vine just past the door far forward and the door shuts behind you for 5 s, for everyone), over its top, or round its side. At the **wreck**, snipe the 10 × 7 m gap through its bow on the line (the hidden, fastest way: miss and the hull stops you dead), or go between its broken halves or over the bow. The armada's mechs carry vines in their fists and elbows as they march. |
| Mape's Tower | Start at the foot of a tall wizard's tower, swing under a long bridge to a second, shorter tower, climb round its outside, swing back under a higher bridge and in through a doorway, then climb the tall tower's hollow inside on beams scattered with no set path. Out of the top you fly through a turning tear in reality (an insane wizard rips it open, a giant eldritch banana behind it) and fall down a tube through the void: steer in the air round 20 to 30 bounce pads, since hitting one costs your speed. At the bottom, grab the near-flat vines hung over the exit tunnel's mouth and sling yourself along it to the finish. The vines are spell beams cast by mages on balconies and dragon riders; letting go of a climbing beam while rising lifts you a little. Fall and you are put on the last reset pad with a push the right way. The apes wear wizard and witch outfits (the costume **Wizard**, wearable on any map once you beat the Author time here); the Floor is Lava cat is a flame demon. Weathers: Clear, Twilight, Dawn. |
| Bananas | Arcs of bananas hang round the bottom of some vines' swings: catch the vine near its end and swing on through them. Each one raises your top speed, up to **20 bananas for 20%** (the HUD counts them). Each racer collects their own; an item hit drops 2. |
| Catch-up vines | A vine another ape has swung on (and you have not) glows for you in its map's catch-up colour (**coral red** on Canopy Run, **magenta** in Concrete Jungle, **emerald** on Yeti Peak, **hot rose** in Sector Ape, **hot pink** in Sunset Ruins, **scarlet** on Banana Bay Battle, and **crimson** on Mape's Tower). Let go of one and you get a small push forward, once per vine: whoever is behind meets more of them. |
| Items | Catch a vine with **violet flowers** on it to take an item (one at a time; the flowers close into buds until it comes back). **Right click** or **E** uses it. **Banana:** thrown where you look; a hit stuns and slows. **Stick:** whacks any ape next to you off its vine. **Hook:** latches onto the nearest ape ahead and tows you toward it for 2 seconds. **Speed Boost:** a burst along your motion. **Race Setup → Items** has a checkbox per item: unticked items never come out (none ticked: no items at all). Online, the host's choice counts. |
| Floor is Lava | **Race Setup → Floor is Lava** (online: the host's lobby). Touch the ground and you are out: a jaguar runs in, pounces with a roar and eats you, and after a moment the camera moves on to watch the racers still going. The start platform and the mushrooms are safe. The eaten are listed after the finishers, furthest first, marked **Eaten**; the race ends when everyone is in or eaten. The AI apes are eaten too. Not in a Time Trial. |
| Apes | Orangutan, Gorilla, Chimpanzee, Gibbon, Proboscis Monkey, Lemur. Five stats: Swing Power, Top Speed, Reach, Air Control, Weight. Every ape's stats add up to the same total. **Equal Stats** gives every ape the average. |
| Online | **Online → Host** gives a room code to share; others **Join** with it. Everyone picks an ape, can change their name, and readies up; **Screech!** plays a chimp screech for everyone in the lobby. The host adds AI apes and starts. |
| Best time | Your fastest finish is kept on this device and shown in the HUD and results. |
| Hype | Swing fast enough and the speedometer gets hyped: crossing each level, an ape whoops it out (biggest at BANANAS!). |
| Time Trial | Race alone (crates give only the Speed Boost) for Bronze, Silver and Gold target times, set per map; the Author time shows once you earn Gold. Your best Time Trial races beside you as a pale-blue ghost. |
| Spectating | Once you finish, the camera pans to the leading ape still racing; the results wait in a smaller panel at the side. **A / D** (or ‹ ›) watch another racer. The full results come up when everyone is in. |
| Replays | **Watch Replay** on the results (or the pause menu) replays the race: play, pause, ½×/1×/2×, chase, orbit or trackside cameras, and a seek bar. **A / D** (or ‹ › in the panel) follow another racer. Drag the view to steer the orbit camera, and use the wheel to zoom. Online races can be replayed too, once everyone is in. Your fastest run per course and mode and your last 10 others (at most 3 online) are kept on this device under **Replays**. |

### Controls

| Action | Key |
| --- | --- |
| Swing, steer | W A S D (or the arrow keys) |
| Grab (hold) | Left mouse button |
| Jump (from the ground) | Space |
| Use item | Right click or E |
| Look | Mouse (click the 3D view to capture the mouse) |
| Pause | Esc (online, Esc shows the menu but the race goes on) |
| Quick restart | R twice quickly (offline races and Time Trials) |
| Watch another racer | A / D or ← / → (after you finish, and in replays) |
| Replay orbit camera | Drag the view to turn it, mouse wheel to zoom |

Settings: Volume, Colourblind Mode (the world goes grey, the vines stay vivid, the vine in reach turns white and thick, minimap apes get shapes, and catch-up vines take a brighter colour of their own), Mouse Sensitivity, Invert Mouse Y, Camera Follows (the camera swings back behind you when the mouse rests), Weather (Clear, Snowing or Blizzard on Yeti Peak, Clear or Squall on Banana Bay Battle; also on those maps' setups and in the online lobby), and JOJO MODE once you find the secret. They are kept in browser storage.

Colours: beside the ape picker on Race Setup, Time Trial and the online lobby, each ape has four palettes (Classic and three alternates, e.g. the Gorilla's Silverback, Mountain Brown and Snowflake). Each ape remembers its own; online, everyone sees your choice, and replays keep it. An AI ape of a kind already racing wears another palette.

## Build, test and serve

```bash
./scripts/setup.sh          # once: Python venv with Playwright and three browsers
cargo test --workspace      # the core, boundary and online tests
./scripts/build.sh          # release WebAssembly build into dist/
python3 scripts/serve.py    # http://127.0.0.1:8000/
./scripts/check.sh          # everything, before a deploy
./scripts/check-online.sh   # WebRTC between tabs (pasted codes, no relay)
./scripts/quick-check.sh    # only the tests the change needs, Chromium only (see below)
./scripts/pages.sh          # assemble the site copy, thumbnail and tiles in ../ptonkle.github.io (TILES=0: no new tiles)
.venv/bin/python scripts/tiles.py OUT_DIR [--map jungle] [--try SPEC ...]   # the home page's action shots, race-NN.webp
.venv/bin/python scripts/shot.py out.png [--query lineup] [--click id ...] [--hold KeyW,Space:1.5 ...]
```

**Quick or full.** `scripts/quick-check.sh` tags the files changed since the last passing full check (`tests/tags.toml`) and runs format, lint, the release build and the import audit, then only the cargo tests, browser steps and online steps carrying one of those tags, in Chromium only (`ALL_BROWSERS=1` adds Firefox and WebKit). A file no glob matches, or a foundation (`fx.rs`, `geom.rs`, `api.rs`, the shim, `Cargo.*`, `build.sh`), runs everything. Use it for small changes in a live session; run the full `check.sh` (three engines) before merging a map or a big change, and whenever the quick check is unsure. `scripts/timed-checks.sh quick` logs it in `notes/check-times.md`; `check.sh` fails if a new test, browser step or file has no tags. Results and limits: `notes/quick-check.md`.

./scripts/check-online.sh   # WebRTC between tabs (pasted codes, no relay)
./scripts/quick-check.sh    # only the tests the change needs, Chromium only (see below)
, `wasm-bindgen-cli` 0.2.127, and Python 3.

## Layout

| Path | Holds |
| --- | --- |
| `crates/core/src/fx.rs`, `geom.rs` | Q16.16 fixed point, integer trigonometry, vectors and matrices (the core has no floats) |
| `crates/core/src/ape.rs` | one ape's physics: swinging on a rope, flying, running, grabs, the Grab Cooldown |
| `crates/core/src/rules.rs` | every tuning number, and the six apes' stats |
| `crates/core/src/map.rs` | the list of maps (`Map::ALL`: Jungle, City, Ski Slope, Sector Ape, Ruins, Bay, Tower), each one's key and course |
| `crates/core/src/course.rs` | a course: centreline, vines, mushrooms, crates, platforms, bananas, starts; the Jungle's layout and scenery placement |
| `crates/core/src/city.rs`, `city/layout.rs` | the City: its layout contract (blocks, cars, boats, river, bridge, what each vine hangs from), the river's water, moving bouncers; the streets with sharp corners, the bridge, the vines and cars |
| `crates/core/src/cityscape.rs`, `vehicles.rs`, `sky.rs` | drawing the City (buildings, Times Square, the river and its sun path, the wire glow), its cars, cabs and boats; the skies |
| `crates/core/src/sector.rs`, `sector/layout.rs`, `sector/rules.rs`, `ape/space.rs` | Sector Ape: the layout contract (vine directions, bodies, boundary, door, mechs, wreck, black hole, set pieces, lobby, routes), the course (spine, stations, decks, the Hulk, the wreck, the armada, the finish), the race rules (the boundary's reset or the lava kill, the switch door, the pull past the finish), and the ape in open space (vines every way, the slingshot, swept hits on bodies and the door); `examples/sector_trace`, `sector_report`, `sector_stats`, `sector_think_cost`, `sector_bench` measure it |
| `crates/core/src/sectorscape.rs`, `spacecraft.rs` | drawing Sector Ape (the sun below and the edge light, the planet, stars and haze, tethers along their vines' directions, decks, the Hulk and its door, landmarks, the black hole on the horizon, the lobby, the flyover through the routes, the space jaguar's pop) and its models (the mothership, the wreck, mechs, fighters, battles, debris, the banana black hole) |
| `crates/core/src/bay.rs`, `bay/layout.rs`, `bay/rules.rs`, `bayscape.rs`, `baymodels.rs` | Banana Bay Battle: the layout contract (ships, barrels, volleys, fighters, the sea's swell, what each rope hangs from), the course (dock, 30 ships, the finish island), the rules (the sea puts you on the last ship passed, barrels as the soft wall), the 3 pm sky and the moving sea, the squall, and the ships, crews, cannon fire, barrels, dock, island and chest; `tests/bay_ai.rs` |
| `crates/core/src/tower.rs`, `tower/layout.rs`, `tower/rules.rs`, `towerscape.rs`, `towerscape/`, `towermodels.rs`, `towermodels/` | Mape's Tower: the layout contract (two towers, two bridges, doorway, balconies, windows, clouds, portal, tube and its pads, tunnel, pads, what casts each beam), the 3D centreline (progress along a path that climbs and falls), the course, the resets with a boost, the sky by height and the twilight and dawn weathers, the scenery culled by 3D cells and halves, and the towers, bridges, tube, portal, mages, dragons, eldritch banana, creatures and banana sea; `tests/tower_ai.rs`, `examples/tower_trace`, `examples/tower_legs` |
| `crates/core/src/trial.rs`, `app/medals.rs`, `replay.rs` | Time Trial medal times per map, the medals shown, replays and the ghost |
| `crates/core/src/race.rs`, `items.rs`, `ai.rs` | the race loop, items, and the AI driver |
| `crates/core/src/camera.rs`, `focus.rs`, `scene.rs`, `models.rs`, `mesh.rs`, `render.rs` | the chase, orbit and pan cameras, which racer the camera follows, the drawn world, the six ape models, unit meshes, the instance-draw contract |
| `crates/core/src/jaguar.rs` | Floor is Lava (TRIAGE T37): who is eaten, and the jaguar's run, leap, bite and trot away as a function of the race tick (its model is in `models.rs`) |
| `crates/core/src/sound.rs`, `hype.rs` | every sound, synthesised by the core as a WAV file (the hype call-outs' ape voices too); when a hype level is called out |
| `crates/core/src/online.rs`, `app/online_play.rs` | online play: lobby, state sync, hits and results; its wiring into the app |
| `crates/core/src/app.rs`, `api.rs` | screens, settings, input, the view model; the one door the shim calls |
| `crates/shim` | the `wasm-bindgen` layer: one call into the core per method, Q16 → `f32` for WebGL |
| `web/` | the static page: WebGL 2 renderer (`gl.js`), menus and HUD (`app.js`), the WebRTC transport (`online.js`) |
| `data/` | on-screen words (`strings.json`) and the shaders |
| `crates/core/examples/` | measuring tools: `frame_cost`, `scenery_cost`, `swing_time`, `corner_swing`, `city_trace`, `city_report`, `city_map` (MAPS.md says when to use each) |
| `scripts/tiles.py` | the home page's action shots (21: 15 Jungle, 6 City) |
| `tests/browser/` | the three-engine browser check, the game's interactions, and the online checks |
