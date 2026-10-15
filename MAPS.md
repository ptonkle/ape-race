# Making a map

Notes for building the next map, from the Jungle (the first course) and the City (New York at dusk, 2026-10-12). Read this before adding a map. Update it when a map teaches something new. SECOND-ORDER rows C4–C8 and on (City course), Y1–Y14 (City art), U1–U10 (map selector), I3 and I6 (integration), and V1–V12 (the jaguar mode, which runs on every map) have the details.

## Build steps (track these for every map)

Keep `notes/<key>-progress.md` with one row per step and a status of **Done**, **In progress**, **Not started**, **Waiting on you** or **Blocked**, and show that table in every progress report (the person, 2026-10-14; the `/progress-report` skill in ~/.claude/skills does this). The steps:

1. Brief and answers
2. Skeleton and contract
3. Ape costumes
4. Models
5. Sky and lighting
6. Course layout
7. Map rules
8. Art placed on the course
9. Lobby spot and flyover
10. AI validation
11. Medal times
12. Step and frame cost
13. Colourblind and JOJO MODE
14. Map card
15. Screenshots at 1920×1080
16. Other maps unchanged
17. Docs
18. Full checks
19. Deploy

New concepts get tracked too. Under the steps, the progress file has a **features table**: one row per concept from the brief (the river, the turtles, the trapdoors…), and a new row the moment anything new comes up (the person's feedback, a new mechanic, an idea mid-build), with its source. Each feature goes through the same stages: Model, Gameplay (or n/a), Placed in course, Tested. If a new kind of work applies to every map, add it to the step list above as well.

## What a map is

- **One entry in `map.rs`:** `Map` (`Jungle`, `City`, …), with `key()` (the course's `name`, the settings' and replay library's word for it, and `map.<key>` in `data/strings.json`), `course()` (built from a fixed seed) and `Map::of(course)`.
- **One `Course`** (`course.rs`): the centreline (`path`, `heading`, `length`, `half_width`), `vines`, `mushrooms` (every bouncer, whatever it looks like), `platforms`, `crates`/item vines, `bananas`, `props`, `starts`, plus anything map-specific on an optional field (the City's `Course.city: Option<Box<CityLayout>>` and `Course.streets`).
- **Its drawing:** `scene::scenery` hands off to a map's own module when the course has its data (the City: `cityscape.rs`, plus `vehicles.rs` for its models), and `scene::look(map)` / `look_of(course)` gives the sky, fog, light direction and colour, and ground bounce. The frame's view values carry the light, so a map can have its own sun.
- **Everything per map that the app keeps:** the menu background and ape preview (`scene::preview(course)`, `scene::flyover_camera`), bests (`Settings::best(map, trial)`), medals (`trial` / `app/medals.rs`), the ghost, and the replay library (keyed by course name, `Recording::fits` checks the layout fingerprint).

## How the City was built (it worked: copy it)

1. **The integrator writes a skeleton first** (about 15 minutes): the `Map` entry, a stand-in course (the Jungle's layout under the new name), and the **contract types**: the map-specific layout the course fills and the scene draws (the City: `CityLayout` with blocks, cars, boats, river, bridge, and a `Hanger` per vine saying what it hangs from). Commit it, then fan out.
2. **Three agents in parallel**, each in a worktree:
   - **Course:** layout, physics for anything new (water), the AI, medal times, layout tests. It should commit a first rough but real layout early and keep the contract stable after that; any contract change goes in early and into SECOND-ORDER.
   - **Art:** models and drawing from the contract. It starts on the models, then merges the course branch once there is a real layout (tell it the branch name).
   - **Map selector and UI:** only needed for the second map. Every later map is just an entry in `Map::ALL` plus its strings and its picture on the map grid (`data/maps/<key>.webp`, TRIAGE T54).
3. **Hold the selector until the real map lands**: a selectable stand-in that looks like another map confuses players.
4. **Pre-merge art and UI on a staging worktree** while the course agent finishes. The slow part is always the course/AI agent.

## Layout lessons

- **Keep the old maps byte-identical.** The City added a `jungle_unchanged` test (the Jungle's layout fingerprint and a replay of a Jungle run). New map code must not change the old maps' dice draws: give new placement its own `Dice` (e.g. `seed ^ 0x…`).
- **Sharp corners** (the City): the centreline still bends smoothly, with a radius just over the half width (32 m for 90°, 16 m for 45°), so `nearest`, `progress` and the minimap stay smooth. The **walls** are the union of straight street boxes, mitred: square outside a 90° corner, a sharp inside corner (`Course.streets`, `Street::span`).
- **The 90° corner swing needs no physics change.** A short cable (about 13 m) high over the crossing (27–28 m), caught near its top at speed, whirls the ape round a quarter turn in 0.4–1.0 s and keeps 80–95% of its speed. A long cable caught low swings out like a pendulum instead. Put a station just before and just after each 90° corner (C7; `examples/corner_swing.rs`).
- **Vine density:** the Jungle has stations every 22–28 m (two or three vines across lanes) with a low vine at every other station, so an ape on the ground can climb back. The City is sparser (stations every 24–30 m), and every map still needs low vines regularly and always a way on.
- **Bouncers:** anything that bounces you is a `Mushroom` entry (top, radius) with the same reward, whatever is drawn: a car or boat carries the index of its bouncer, and the scene never draws a mushroom on that map. Moving bouncers (driving cars) are deterministic from the tick: one shared function (`Course::car_at(i, tick)`) is used by both the physics and the drawing.
- **Water** (the City's river): swimming slowly to a bank or ladder is that map's "ground". Keep it forgiving (boats to bounce off, low vines nearby): falling in must not lose you the race. Water counts as the floor for Floor is Lava (`Course::floor`).
- **A sloped course** (the Ski Slope, 2026-10-13, SECOND-ORDER T rows): the centreline (`path`) stays at height 0 and every height comes from `Course::terrain` (`ski::Terrain`, `Copy`, so the camera holds one). Use it, never `course::ground`, for vine anchors, crates, the AI's "low vine" and "raised" checks, the camera's floor, the trackside replay camera and the jaguar. The ceiling (`terrain + CEILING`) follows it already. The hill's profile is worked out once at compile time (`ski::HILL`): the AI's plans call the terrain thousands of times a tick. Keep every leg within about ±50° of the fall line so the run never climbs; traverses are gentler than straights down the fall line. `SLOPE_DEG` (5–10) is the knob: the layout and skiing tests hold over that range; the AI's level spacing and the medal times need re-measuring after a change (T rows).
- **Forks** (`ski::Fork`, `Course::lanes`): the run splits round an island and joins again; `contain` keeps an ape in whichever lane (branch) it is nearest, so the island is a wall. Put a fork on a straight of the spine (`nearest`/`progress` stay on the spine; branch offsets of 20–45 m are far inside any bend radius), keep each branch as wide as a run (the Ski Slope: the run 27 m either side, each branch 20 m, after the person asked to widen it), and treat the fork as two lanes from its first metre: while the branches part they overlap, and a station's middle vine there sits right where the island's tip appears (apes flew into it). Take the lanes where each vine hangs (`s ± 3`), not at the station's `s`: they move. Crates, bananas and low vines go in both branches; a branch as wide as a run needs its own middle lane (a wire across it). Landmarks on an island stand at its head, where the run splits: they announce the fork, and the clearing in front of them costs no vines. Lift towers and terminals stand outside every lane.
- **Vines hung from real things** (the Ski Slope's chairlifts, conifers, wires, helicopters): each lane's vine takes the nearest hanger that reaches its spot; a chairlift is straight, so it crosses a bending run at a slant and its chairs are over the lanes only part of the way. A station with no middle vine (no chair near) leaves the weaker AIs swinging wall to wall: give the middle a wire across between two trees when nothing else reaches.
- **Ground that moves you** (skiing): an ape on the snow glides downhill (`Ape::ski`; gravity along the slope, friction, a linear drag, WASD carve, W poles, S snowploughs), and a landing keeps its speed along the slope. Keep it a fallback: skiing alone takes about 146 s down the Ski Slope against 58–89 s swinging (King Kong by kind) and 88–142 s for the Sloth.
- **`Course::floor`** is what the jaguar mode treats as the floor, and what the jaguar runs on. A map with a different floor (water, a bridge deck) changes only that function.
- **The camera's start view:** the chase camera sits about 10 m behind the ape (`camera.rs` `BACK_REST`, for 2× apes). Keep any solid face at least 16 m behind `path[0]`, or the race opens inside a wall (City Y13).
- **Watch the step cost**, not just the drawing: the City's first layout had a 6–7 ms stepping spike once a second (Y10). Measure with `examples/frame_cost.rs -- <map>`. The cause was the AI: each plan flies copies of the ape hundreds of ticks ahead, so anything per ape per tick is paid thousands of times. The City's bouncer search walked every car for every moving bouncer. Fix: skip it when the feet are above the tallest bouncer (`city::BOUNCER_TALLEST`, 5 m), and look moving cars up directly. Worst step 1.7 ms after.
- **The start camera fix** (Y13): the City's start street runs on 18 m behind the start (`START_BACK` in `city/layout.rs`) to the office closing it; the start's back rule still stops apes at 6 m.

- **Gravity per map** (Sector Ape, 2026-10-13): `Course::gravity()` (and `Course::hang(i)`, the way vine `i` hangs) return the old constants on every other map, so the Jungle, City and Ski Slope stay byte-identical (`tests/maps_unchanged.rs`, `jungle_unchanged.rs`). Use them, never `rules::GRAVITY` or "straight down", in the swing (and its energy correction), the flight, items' arcs and the AI's sums. Anything new for one map is gated on its layout (`course.sector.is_some()`), and the other maps' code paths are kept as they were, not "proven equal".
- **Low gravity changes everything about a swing.** At 9 m/s² a swing at 30 m/s loops right over its anchor, and a release flies 60–120 m. The AI then whirls for seconds when its first release window goes by, and every body near an anchor is in some swing's circle. What helped: longer reach (×1.5) and air steering (×2) on the map, flights followed for 4 s, the landing score valuing time and speed more (`SECTOR_SPEED_WORTH`, `SECTOR_WAIT_COST`), giving up a hopeless vine after 3 s, plain vines off every deck, gentle bends (an ape at 60 m/s cannot steer round a 255 m bend).
- **Odd vines** are a segment from the anchor along the hang direction: reach, the AI's distance and "low vine" tests, crates and the drawing all take the segment (`Course::vine_point`, `tip`, `vine_bottom`). The swing is the same rope round the anchor; the catch point is what changes. Up vines turn you down, level ones turn you sideways: put them where that helps.
- **No floor, a boundary** (Sector Ape): `Course::contain` does nothing (no walls, no ceiling), `on_floor` never fires, and crossing the boundary puts an ape on the nearest deck (`Race::space_rules`; with Floor is Lava it is the kill). A deterministic AI put back on the same deck makes the same mistake again: after a reset it misjudges a little more (`Brain.shaken`) and drives at its best a while.
- **Solid bodies at 85 m/s**: a swept ball against boxes (`sector::sweep`), only the bodies indexed near the ape's stretch of the course (`SectorLayout.near`); a contact that starts inside keeps the move and is pushed out from where it ends (else an ape sticks in a hull forever); one rebound boost a quarter second. Boxes touching boxes (a wall's top under a deck) trap apes: overlap them instead.
- **The AI's copies pay for everything.** A mech's anchor cost 0.9 µs until it was worked out alone (`spacecraft::mech_anchor`); the platform search, square roots in the AI's reach test and every vine looked at in `vine_in_reach` (now a window by place, `SectorLayout.by_at`) all showed up. On a map whose flights last seconds, cap the search per tick (`SECTOR_SPEND`) rather than per flight.
- **Profile before guessing** (2026-10-14, SP rows): there is no `perf` here, but `pprof` in a scratch crate outside the workspace (depending on `tool_core` by path, `debug = true`) samples a race of AIs fine; profile one bad tick by cloning the brain and thinking it 400 times. The big costs were plain arithmetic: a 128-bit square root and 128-bit divisions in every `len`/`norm`, `right_of` in every `progress`, an 81-sample scan in `nearest`. Make those cheaper *exactly* (same values; prove it with a test against the old code and with `jungle_unchanged`, `maps_unchanged`, `sector_unchanged`) rather than make the AI think less: Sector's worst step went 3.9 → 1.4 ms with every race unchanged. Things that pay: length tests by squares (`V3::longer_than`), work that does not change over a flight hoisted out of it, values that never change worked out once (`Course.right`, `MechPath.rest`, `SectorLayout.fits`), caches of pure sums keyed by tick (`ai::Anchors`), and skipping what provably cannot pass (`ai::Watch`, `nearest`).

## Getting the AI round a new map

- **Find where it stalls before tuning anything.** `examples/city_trace.rs` prints one AI ape every half second (progress, mode, height, speed, grabs, wall hits); stalls show up as 3 s with under 25 m gained. `examples/city_report.rs -- 8` gives every difficulty over eight seeds; `examples/city_map.rs` draws the layout from above as an SVG.
- **Fix the layout, not the AI:** the difficulties are shared by every map, and retuning them changes the Jungle. On the City every fix was in the layout:
  - a station 12–16 m short of each 90° corner (without it the last vines could be 35 m short of the corner cables, and the AI swung back and forth with no way on);
  - a station 5 m into the new street after each 90° corner, so a quarter-circle release always has something close;
  - stations every 24–30 m (26–32 m was past Sloth's 22 m ideal hop: it stalled on straights and failed to finish 1 race in 6);
  - a low vine at every other station (every third left a fallen ape running 10 s).
- **When the layout is right but the levels bunch up**, give the new map its own handicaps (`ai::Difficulty::tuning_on`), never the shared ones: on the City the Monkey and the Ape got quicker handicaps, the Jungle's stayed byte-identical (C12).
- **New ground needs an AI rule:** in the City's river the AI presses no keys, so the auto-swim takes it to the nearest bank, ladder or boat (`ai::think`). Check it never swims at a vine it cannot reach.
- **Downhill speed and reaction times** (the Ski Slope): swings run at 30–40 m/s downhill (tops of 48–60 m/s in a Time Trial), and the weaker levels' reaction time (up to 0.27 s before they may let go) let the good release window go by, so they whirled over the anchor with no way on (stalls everywhere, the Monkey slower than the Sloth). The fix was per map: the Ski Slope's levels react sooner (`SKI_HESITATE`), look further ahead (`SKI_LOOK` 110 m, `SKI_CANDS` 16, since a fork doubles the vines in view) and get their own arms and misjudging (`SKI_DRIVE`, `SKI_NOISE`). More drag, higher anchors and longer hops did not help. Tools: `examples/ski_trace.rs` (with `SKI_TRACE_CANDS=1` the candidates, `SKI_TRACE_JUNGLE=1` the Jungle for comparison; it counts island wall hits), `ski_report.rs` (levels × seeds in parallel, with means), `ski_vines.rs` (every vine and its hanger), `ski_speeds.rs`, `ski_map.rs` (an SVG; render it to PNG with Playwright from the repo's `.venv` to look at it).
- **Then add an ordering test for the map** (`tests/city_ai.rs`: eight layouts, each level 6 s slower than the next, every race inside 150 s; every kind at Sloth and King Kong; four AIs with items).
- **Medal times:** `city_time_trial_report` (King Kong alone, Speed Boosts only, every kind). Author just under the best kind, then the Jungle's steps of 7, 10 and 15 s (City: Orangutan 64.4 s, so 1:04 / 1:11 / 1:21 / 1:36). Medals are per map: `trial::medals(map)`, `target_on`, `earned_on`, `author_shown_on`, `next_target_on`.

## Art lessons

- **Establish the time of day and the weather first** (the person, 2026-10-13: always do this). The Jungle is morning sun, the City dusk, the Ski Slope noon and clear (snow falls only with the Weather setting). The time sets the sun's height and colour, the sky, the fog and the shadows' colour; write it at the top of the map's module.
- **Be visually distinct from the first iteration** (the person, 2026-10-13: the City's first look was not distinct enough from the Jungle; it took a second pass, O1–O8). Choose a palette no other map uses and push it: a player should know the map from one screenshot, from the sky, the ground's colour and the signature props alone.
- **A clear sky needs a dome** (the Ski Slope, B2): the clear colour is flat, so a noon sky that deepens overhead is a sphere round the eye drawn in the shader's sky-dome glow range (over 3.5): fog colour at the horizon to `uSky` × its colour overhead, so JOJO MODE and the colourblind mode still recolour it. Far mountains use the far-backdrop range (2.5–3.5): lit, not fogged by distance, hazed by a fixed amount. Draw both round the eye each frame, at a fixed distance inside the far plane, as if infinitely far.
- **Far mountains need haze under them** (B3): beyond the drawn ground there is nothing, so a range whose feet show looks as if it stands at the forest's edge. A ring of fog-coloured panels round the eye from just under the horizon down lets the range rise out of the haze. Keep the range low (2–6°) and the set piece clearly taller.
- **A forest hides landmarks** (B7, B8): clear the trees round each landmark and along a corridor from the run to it, and scale landmarks up (1.5–2×, the Ice Ape 20× life) until they read over the trees. Trees on an island in front of its landmark hide it however big it is.
- **A sloped map's terrain follows the course's height function** (`Course::terrain`) inside the corridor exactly, so feet and floor agree; everything outside (banks, islands' mounds, lumps) is drawing only. Build the ground's columns from the corridor's edges (`Course::lanes`) so the fence line always falls on a column.
- **One style:** faceted low-poly from the unit meshes (`mesh.rs`); no new meshes were needed for the whole City. Apes are drawn at 2×, so props have to read at that scale.
- **The swingable lines must stand out above everything else.** The City's first bridge had so many suspenders the wires got lost; the fix was far fewer decorative cables, thin and dim haze-coloured, and none within 4 m of a vine. Check this in colourblind mode too (vines vivid, world muted) and with JOJO MODE on. A busy map can also give its vines a soft glow (`VineLook.halo`, drawn in the `render::HALO` pass after every solid shape) and calm its backdrop (`cityscape::calm`: buildings and skyline less saturated, the map's colour coming from the sky and the light). Screenshot with `?tile=…,blind=on` / `jojo=on` (SECOND-ORDER O3, O4, O7).
- **Fine detail on the water or far away:** a few big boxes read as low-res. Use many small flat glints sized by distance from the eye (bands, each twice the size of the last), fixed to the world so they hold still as the eye moves, over a soft additive sheen (the City's sun path, O2).
- **Each map's vine look** (`scene::vine_look_on(map, …)`): the City's are glowing cyan cables. Keep in-reach (yellow, white and thick in colourblind mode), catch-up (coral) and the item marker distinct: Jungle flowers are violet, City item wires get violet sparks, and the marker is hidden when items are off.
- **Give the map a set piece the person can see from the course:** the City's sunset sits 22° off the course heading over the river, in view while crossing the bridge, and is also where the light comes from. Glow above 1 in `scene.frag` counts as sky (no fog) so the sun stays visible.
- **A finish set piece** past the arch (the Jungle: a podium clearing; the City: a gantry and a plaza), since a finished ape coasts on (`course::OUTRO`).
- **Budget:** about 20k scenery shapes and 6–7k drawn at once (Jungle 19,997 / 6,209; City 17,935 / 7,399; Ski Slope 19,722 / 10,767 since T62 (every chunk short of the fog's end, mean 8,553) and T74 (more low vines and their conifers); Sector Ape 5,114 / 2,251 static plus up to about 2,900 drawn per frame; `examples/scenery_cost.rs`).

### What Sector Ape taught (space, 2026-10-14, SECOND-ORDER SA rows)

- **A light from below needs an edge light.** With the sun under the track, every top faces away from it and reads black on black space. `Look.rim` adds a colour where a surface turns away from the eye (`scene.frag`, `uRim`; black on the other maps, so they add exactly nothing), and the ambient from above becomes a dim indigo instead of the sky's blue: bellies glow sun-orange, tops violet, outlines blue-violet.
- **No ground changes the drawing's reach.** Space is clear, so a map without fog to hide pop-in draws its chunks further (`Scenery.reach`, 420 m) and its big pieces per frame (the wreck, the set pieces, the mechs). A landmark or goal that must show from anywhere (the black hole) is drawn shrunk toward the eye beyond 300 m by the share it is further (`sectorscape::seen_from`): it looks exactly the same and stays inside the 600 m far plane; drawn at its true distance it showed as clipped arcs.
- **Glows in the sky:** halos fade with the fog, so a corona or a nebula 500 m off vanished; a halo strength of 3 or more is now a sky glow (unfogged). Make them balls, not flattened discs (a squashed sphere's halo shows its facets as hard edges). Halos write depth and are drawn grouped by mesh (Sphere before Cyl): a big halo drawn first hides later ones behind it, so push them in the order they should add.
- **Vines in any direction:** draw from the anchor along `Course::hang(i)` (`scene::vine_along`, `vinesim::Chain::hanging_along`) and keep the straight-down case's arithmetic exactly as it was, so the other maps do not change.
- **A huge set piece must not cut the course, and the camera must never end inside a solid.** The black hole's disc reaches six radii: lying flat it crossed the armada, so it stands up 60° to face the apes coming in. Finished apes float at its horizon with the camera a dozen metres off, so the black ball is smaller than the horizon (3/4 r); at r it filled the screen with black.
- **An inside lit from below:** a corridor's ceiling faces down, so the sun under the map lights it bright (there are no shadows). Line inside spaces with dark plates and their own light strips.
- **A faster map:** give it its own speedometer scale (`speedo::Dial`), widen the chase camera with its pace (`Chase.pace`), stretch the speed lines, and read the hype call-outs and JOJO MODE's sky at the same share of the dial (`Dial::standard`), so they still mean the same.
- **States worth a picture** (the door shut, the jaguar popping in) get tile keys (`door=`, `eat=`), since an AI race rarely shows them at a chosen tick.
- **A yaw grows to the left** (`geom::facing`: 90° is +x, and `right_of(+z)` is −x). The first notes put the sun on the wrong side.

## Checklist for a new map

- [ ] **The lobby zone, designed on purpose** (the person, 2026-10-13: a core thing for every map). It's where your ape stands behind Race Setup, Time Trial and the online lobby, so it's the map's first impression. Choose a spot that shows the map at its best (its time of day, its set piece behind), with room for T76's lineup of every player's ape and yours on a pedestal in front. Wire it through `scene::preview(course)` and check `app/menu_dock.rs` `preview_spot` actually uses it: the Jungle's never did; it stood the ape at the world origin on the forest floor, T77. Screenshot the setup screen and the lobby on the new map before calling the map done.
- [ ] `Map` entry, its `key`, and strings `map.<key>`
- [ ] Its card on the map grid: build, then `.venv/bin/python scripts/map_cards.py --only <key>` makes `data/maps/<key>.webp` (add a moment to `CARDS` there if the default shot is dull); `tests/map_pictures.rs` fails until it exists
- [ ] Course builder with a fixed seed, its own dice for new placement, and the old maps' fingerprint test still passing
- [ ] Starts and platform, finish and outro, item vines, bananas, bouncers, low vines
- [ ] `Course::floor` for the map; Floor is Lava works there (the jaguar's run-in stays inside the walls)
- [ ] Every AI difficulty finishes it reliably and stays in order (an ordering test for the map); medal times measured with `ai::time_trial`
- [ ] Look (sky, fog, light), drawing, vine look, item marker, preview spot, flyover
- [ ] Colourblind mode and JOJO MODE checked on it; the start camera not inside anything; step and frame cost measured
- [ ] Bests, medals, ghost and replays per map; online start carries the map
- [ ] README (How to play, Layout), TRIAGE row(s), SECOND-ORDER rows

## Ideas waiting (TRIAGE)

T50 (downhill) is the Ski Slope. T51 (uphill, as a perturbed helix: a helix overlaps itself in plan view, so `nearest`/`progress`/`contain` and the floor must tell coils apart by height), T38 (climbing from rising lava), T39 (co-op checkpoints), T41 (swing for distance). A sloped map's heights all come from `Course::terrain` now (the Ski Slope); a helix would also need `nearest`/`progress`/`contain` to tell coils apart by height.
