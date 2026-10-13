# Making a map

Notes for building the next map, from the Jungle (the first course) and the City (New York at dusk, 2026-10-12). Read this before adding a map. Update it when a map teaches something new. SECOND-ORDER rows C4–C8 and on (City course), Y1–Y14 (City art), U1–U10 (map selector), I3 and I6 (integration), and V1–V12 (the jaguar mode, which runs on every map) have the details.

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
   - **Map selector and UI:** only needed for the second map. Every later map is just an entry in `Map::ALL` plus its strings.
3. **Hold the selector until the real map lands**: a selectable stand-in that looks like another map confuses players.
4. **Pre-merge art and UI on a staging worktree** while the course agent finishes. The slow part is always the course/AI agent.

## Layout lessons

- **Keep the old maps byte-identical.** The City added a `jungle_unchanged` test (the Jungle's layout fingerprint and a replay of a Jungle run). New map code must not change the old maps' dice draws: give new placement its own `Dice` (e.g. `seed ^ 0x…`).
- **Sharp corners** (the City): the centreline still bends smoothly, with a radius just over the half width (32 m for 90°, 16 m for 45°), so `nearest`, `progress` and the minimap stay smooth. The **walls** are the union of straight street boxes, mitred: square outside a 90° corner, a sharp inside corner (`Course.streets`, `Street::span`).
- **The 90° corner swing needs no physics change.** A short cable (about 13 m) high over the crossing (27–28 m), caught near its top at speed, whirls the ape round a quarter turn in 0.4–1.0 s and keeps 80–95% of its speed. A long cable caught low swings out like a pendulum instead. Put a station just before and just after each 90° corner (C7; `examples/corner_swing.rs`).
- **Vine density:** the Jungle has stations every 22–28 m (two or three vines across lanes) with a low vine at every other station, so an ape on the ground can climb back. The City is sparser (stations every 24–30 m), and every map still needs low vines regularly and always a way on.
- **Bouncers:** anything that bounces you is a `Mushroom` entry (top, radius) with the same reward, whatever is drawn: a car or boat carries the index of its bouncer, and the scene never draws a mushroom on that map. Moving bouncers (driving cars) are deterministic from the tick: one shared function (`Course::car_at(i, tick)`) is used by both the physics and the drawing.
- **Water** (the City's river): swimming slowly to a bank or ladder is that map's "ground". Keep it forgiving (boats to bounce off, low vines nearby): falling in must not lose you the race. Water counts as the floor for Floor is Lava (`Course::floor`).
- **`Course::floor`** is what the jaguar mode treats as the floor, and what the jaguar runs on. A map with a different floor (water, a bridge deck) changes only that function.
- **The camera's start view:** the chase camera sits about 10 m behind the ape (`camera.rs` `BACK_REST`, for 2× apes). Keep any solid face at least 16 m behind `path[0]`, or the race opens inside a wall (City Y13).
- **Watch the step cost**, not just the drawing: the City's first layout had a 6–7 ms stepping spike once a second (Y10). Measure with `examples/frame_cost.rs -- <map>`. The cause was the AI: each plan flies copies of the ape hundreds of ticks ahead, so anything per ape per tick is paid thousands of times. The City's bouncer search walked every car for every moving bouncer. Fix: skip it when the feet are above the tallest bouncer (`city::BOUNCER_TALLEST`, 5 m), and look moving cars up directly. Worst step 1.7 ms after.
- **The start camera fix** (Y13): the City's start street runs on 18 m behind the start (`START_BACK` in `city/layout.rs`) to the office closing it; the start's back rule still stops apes at 6 m.

## Getting the AI round a new map

- **Find where it stalls before tuning anything.** `examples/city_trace.rs` prints one AI ape every half second (progress, mode, height, speed, grabs, wall hits); stalls show up as 3 s with under 25 m gained. `examples/city_report.rs -- 8` gives every difficulty over eight seeds; `examples/city_map.rs` draws the layout from above as an SVG.
- **Fix the layout, not the AI:** the difficulties are shared by every map, and retuning them changes the Jungle. On the City every fix was in the layout:
  - a station 12–16 m short of each 90° corner (without it the last vines could be 35 m short of the corner cables, and the AI swung back and forth with no way on);
  - a station 5 m into the new street after each 90° corner, so a quarter-circle release always has something close;
  - stations every 24–30 m (26–32 m was past Sloth's 22 m ideal hop: it stalled on straights and failed to finish 1 race in 6);
  - a low vine at every other station (every third left a fallen ape running 10 s).
- **New ground needs an AI rule:** in the City's river the AI presses no keys, so the auto-swim takes it to the nearest bank, ladder or boat (`ai::think`). Check it never swims at a vine it cannot reach.
- **Then add an ordering test for the map** (`tests/city_ai.rs`: eight layouts, each level 6 s slower than the next, every race inside 150 s; every kind at Sloth and King Kong; four AIs with items).
- **Medal times:** `city_time_trial_report` (King Kong alone, Speed Boosts only, every kind). Author just under the best kind, then the Jungle's steps of 7, 10 and 15 s (City: Gibbon 67.2 s, so 1:07 / 1:14 / 1:24 / 1:39). Medals are per map: `trial::medals(map)`, `target_on`, `earned_on`, `author_shown_on`, `next_target_on`.

## Art lessons

- **One style:** faceted low-poly from the unit meshes (`mesh.rs`); no new meshes were needed for the whole City. Apes are drawn at 2×, so props have to read at that scale.
- **The swingable lines must stand out above everything else.** The City's first bridge had so many suspenders the wires got lost; the fix was far fewer decorative cables, thin and dim haze-coloured, and none within 4 m of a vine. Check this in colourblind mode too (vines vivid, world muted) and with JOJO MODE on.
- **Each map's vine look** (`scene::vine_look_on(map, …)`): the City's are glowing cyan cables. Keep in-reach (yellow, white and thick in colourblind mode), catch-up (coral) and the item marker distinct: Jungle flowers are violet, City item wires get violet sparks, and the marker is hidden when items are off.
- **Give the map a set piece the person can see from the course:** the City's sunset sits 22° off the course heading over the river, in view while crossing the bridge, and is also where the light comes from. Glow above 1 in `scene.frag` counts as sky (no fog) so the sun stays visible.
- **A finish set piece** past the arch (the Jungle: a podium clearing; the City: a gantry and a plaza), since a finished ape coasts on (`course::OUTRO`).
- **Budget:** about 20k scenery shapes and 6–7k drawn at once (Jungle 19,997 / 6,209; City 17,935 / 7,399; `examples/scenery_cost.rs`).

## Checklist for a new map

- [ ] `Map` entry, its `key`, and strings `map.<key>`
- [ ] Course builder with a fixed seed, its own dice for new placement, and the old maps' fingerprint test still passing
- [ ] Starts and platform, finish and outro, item vines, bananas, bouncers, low vines
- [ ] `Course::floor` for the map; Floor is Lava works there (the jaguar's run-in stays inside the walls)
- [ ] Every AI difficulty finishes it reliably and stays in order (an ordering test for the map); medal times measured with `ai::time_trial`
- [ ] Look (sky, fog, light), drawing, vine look, item marker, preview spot, flyover
- [ ] Colourblind mode and JOJO MODE checked on it; the start camera not inside anything; step and frame cost measured
- [ ] Bests, medals, ghost and replays per map; online start carries the map
- [ ] README (How to play, Layout), TRIAGE row(s), SECOND-ORDER rows

## Ideas waiting (TRIAGE)

T50 (downhill), T51 (uphill, as a perturbed helix: a helix overlaps itself in plan view, so `nearest`/`progress`/`contain` and the floor must tell coils apart by height), T38 (climbing from rising lava), T39 (co-op checkpoints), T41 (swing for distance). Sloped maps need the centreline, floor, walls, ceiling (`rules::CEILING`) and vine anchors to follow height. Today all of them assume a level course.
