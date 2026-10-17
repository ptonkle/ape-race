# Making a map

The build checklist. Every map agent reads this file only. The detail (per-map lessons, AI tuning, art notes, history, SECOND-ORDER row ids) is in [`notes/maps-archive.md`](notes/maps-archive.md): open it only when a step below sends you there. When a map teaches something new, add one line here if it is a common gotcha and the full lesson to the archive.

## What a map is

A `Map` entry in `map.rs` (with `key()`, `course()` from a fixed seed, strings `map.<key>`), one `Course` (`course.rs`: centreline, vines, bouncers, platforms, crates, bananas, props, starts, plus an optional map-specific layout), its drawing (`scene::scenery` hands off to the map's module; `scene::look` gives sky, fog and light) and the per-map app state (preview, flyover, bests, medals, ghost, replays). Archive: "What a map is", "How the City was built".

## Build steps

Keep `notes/<key>-progress.md` with one row per step, status **Done**, **In progress**, **Not started**, **Waiting on you** or **Blocked**, and show that table in every progress report (`/progress-report`). Under it keep a **features table**: one row per concept from the brief or new mid-build (with its source), stages Model, Gameplay (or n/a), Placed in course, Tested. Work that applies to every map becomes a new step here.

1. **Brief and answers.** `briefs/<key>.md` with the person's words kept exactly, time of day and weather decided, questions answered; a `notes/<key>-pitfalls.md` saying what the gotchas below mean for this map.
2. **Skeleton and contract.** The `Map` entry, a stand-in course, the contract types the course fills and the scene draws; held off the grid with `Map::held`. Committed before anyone fans out.
3. **Ape costumes.** The map's gear on every ape kind and its Floor is Lava cat.
4. **Models.** Standalone mesh builders and gallery entries for the props, creatures and landmarks, in the map's own distinct palette.
5. **Sky and lighting.** Time of day, sun, sky, fog, ambient written at the top of the map's module; a palette no other map uses (archive "Art lessons").
6. **Course layout.** The real route from spacings the AI already climbs, its own `Dice`, every level finishing every layout (archive "Getting the AI round a new map", "Layout lessons").
7. **Map rules.** Mechanics, solids, hazards, `Course::floor`, camera keep-inside, each tested alone on a tiny course first.
8. **Art placed on the course.** Models on the real route, drawn from the same numbers as the solids; vines stand out above everything; inside the shape budget.
9. **Lobby spot and flyover.** A lobby spot chosen on purpose with the set piece behind and room for the lineup (`scene::preview`, `menu_dock.rs preview_spot`); host and joiner checked; a flyover (archive "Checklist for a new map").
10. **AI validation.** Sloth, Ape and King Kong finish every layout inside 150 s and King Kong beats the Sloth on average, in a map AI test. No spacing targets.
11. **Medal times.** Measured with `ai::time_trial` (King Kong alone, Speed Boosts only, every kind); author just under the best kind, then steps of 7, 10 and 15 s. Retune only when the person asks.
12. **Performance test.** Worst tick and shapes per frame measured with `examples/frame_cost.rs -- <map>` and `scenery_cost.rs`, sky included; inside 5,500 shapes per frame.
13. **Colourblind and JOJO MODE.** Vines, hazards and the item marker read in both; screenshots with `blind=on` / `jojo=on`.
14. **Map card.** `.venv/bin/python scripts/map_cards.py --only <key>` makes `data/maps/<key>.webp`; `tests/map_pictures.rs` passes.
15. **Screenshots at 1920×1080.** From the most interesting vista, published on the design journal.
16. **Other maps unchanged.** `tests/maps_unchanged.rs` and `jungle_unchanged.rs` pass.
17. **Docs.** README (How to play, Layout), TRIAGE rows, SECOND-ORDER rows, a gotcha line here if new, the lesson in the archive, HANDOFF.
18. **Full checks.** The full check passes, or each failing test is recorded with the person's say-so (`notes/test-failures.md`).
19. **Deploy.** Only when the person says; patch notes entry first; release the map from `Map::held`.

## Common gotchas

Most frequent first. Full text with row ids: archive "Common gotchas, full text".

1. **Only clear obstacles are solid, and they are solid from the first commit.** Walls, arches, roofs, landmarks and large rocks are solid, tops included, with the solids reaching past the drawing; small dressing and thin things stay non-solid. The ski lift has no collision, by design. Test by flying an ape at each solid kind (`tower/audit.rs`).
2. **Drawing and solids from the same numbers.** Leave openings empty, stop inner blocks short of outer faces (flush faces z-fight), and check every opening from the chase camera.
3. **Every top surface is a real floor.** Overlap boxes, never just touch them; test that an ape can land, stand, walk and jump on every ledge, deck and log.
4. **Keep vines at least 4 m clear of any wall top**, or the feet swing into the stone.
5. **The camera never inside a solid.** Solids at least 16 m behind `path[0]`; every tunnel, cave and arch gets a keep-inside volume (`Temple::kept_in`); test it.
6. **No camera jiggle.** Don't dodge ropes, vines or bridges; rely on the see-through hole (`see.rs`); measure with `examples/camera_jiggle.rs`.
7. **Enclosed parts get their own light** and visible light sources; push the fog out; tops lit from below need a rim light.
8. **Every hazard reads before you touch it**, in colourblind and JOJO MODE too, and nothing traps an ape for more than about 3 s.
9. **Route from proven spacings, AI before art.** A station every 22–28 m, a low vine at every station or every other one, always more than one way on; no tight turns; the start level with the first hangers. Fix the layout, not the shared AI levels.
10. **Fixed route vines.** Random vines make times a lottery; vary only decoration.
11. **The lobby is planned**: not looking at the ground, the ape not sinking; check host and joiner at three window sizes.
12. **Shared moving things are a pure function of the tick** (`*_at(i, tick)` for physics, AI and drawing); anything reset or tripped goes in the save points.
13. **Other maps must not change**: new placement gets its own `Dice`; anything new is gated on the map's layout.
14. **The AI's copies pay for everything per tick.** Index solids by place, reject with a cheap box first, count the sky's shapes, measure as soon as moving things land.
15. **Token burn.** Route first, art last, small time-boxed agents, ranged reads.
16. **Don't hand off red.** Fix a failing test or record it with the person's say-so.
