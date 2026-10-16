# Test consolidation: which tests have earned their time

2026-10-15, analysis only (no test, script or game code changed). Asked: "for tests that many many many versions in a row don't fail, consider if they can be consolidated or something".

## Summary

**Time now (per deploy, from `notes/check-times.md`):** `check.sh` 280–400 s (core tests 60–90 s, browser check 220–330 s in three engines) plus `check-online.sh` 90–155 s: **about 6–9 minutes**.

**Estimated after the five recommendations below:** core 35–50 s, browser 90–120 s, online 50–70 s: **about 3–4 minutes**, with no behaviour left unchecked.

The main finding is that the "never fails" tests are not where the time goes. 465 of the 597 core tests never failed in the record, but together they take under 30 s of CPU and run 16 at a time, so merging or deleting them would save almost nothing. The time is in about 25 tests, mostly full AI races, and in how they are scheduled. The integration-test files run one after another. The browser engines also run one after another. And one test rebuilds the City thousands of times.

**Top 5 recommendations**

| # | Change | Saves (normal load / under heavy agent load) | What is lost |
| --- | --- | --- | --- |
| 1 | `city::tests::moving_cars_drive_their_street_the_same_every_time` rebuilds the whole City (`city()`) inside its loop for every car every 30 ticks: build the second copy once before the loop. It alone is 58 s of the 61 s the lib-test binary takes. | 15–20 s / ~40 s | Nothing (same assertions). |
| 2 | Put the 13 integration-test files into **one test binary** (`tests/all/main.rs` with `mod ai; mod city_ai; …`). Today cargo runs them one after another (≈69 s in a row here). In one binary they run side by side, so the wall time becomes the slowest test (≈16 s). It also links 1 binary instead of 13. | 20–30 s / ~40 s | Nothing. |
| 3 | Browser check: run the three engines **in parallel** (three processes) instead of in turn, and cut the 10 s idle per engine to ~2 s. The closing "no off-origin request at any point" check already covers the whole ~70 s run. Do the same in `online_race.py`. | 120–200 s (check) + 30–60 s (online) | Nothing; the idle check is still made, just shorter. |
| 4 | Per map, **run each AI race once and assert several things** from it. Merge `ruins_difficulties_are_ordered_and_spaced` into `every_level_finishes_sixteen_ruins_layouts`, since the same Chimpanzee races on layouts 0–7 are run twice. Fold `four_ai_apes_with_items_finish_<map>` into that map's `*_unchanged` hash race, as one race that gives both the hash and the finish check. On Sector, `sector_finish` and `sector_unchanged` each run 4-AI races on the same course and can share them. Also add the missing Ruins and Bay hashes (gap, below). | ~25 s CPU; 5–10 s wall after #2 | Nothing if the assertions are kept together. |
| 5 | `app::ski_camera_tests::on_the_ski_slope_the_chase_camera_keeps_out_of_the_conifers_smoothly` runs four whole Ski races one after another (2 seeds × trees on/off). Run them on threads, or split it into one test per seed. Once #1 is done it is the next long pole of the lib binary (19 s). | ~8 s / ~10 s | Nothing. |

Not recommended: deleting never-failed tests. The cheap ones cost nothing. The expensive never-failed ones (the AI "every level finishes", the camera-in-trees and camera-in-hulls tests, `sector_finish`) each guard a bug the person actually saw once: RT10, SW9, BR6, SF7.

## 1. Which tests have ever failed

### Sources and method

- **Session transcripts** (`~/.claude/projects/-home-user-slop-ape-race/**/*.jsonl`, 78 files, 2026-10-12 23:59 UTC → now). Every tool result was scanned for `test X ... FAILED`, `X --- FAILED` (terse format), `thread 'X' panicked`, and the browser check's `  FAIL <check>` lines. Repeats were collapsed to one per session per hour. A failure counts as **deliberate** when the assistant text or command just before it mentions a rule-6 break ("break once", "rule 6", "restore", "mutant"…). The rest count as **real**.
- **Not covered:** the first day (2026-10-11 → 10-12 evening, before v1.3). Those transcripts are gone. Git messages and SECOND-ORDER.md rows partly fill the gap.
- **"Real" includes development failures**, where a test fails while its own feature is being written. Those are most of the 1-count rows. The tests that caught **regressions** (a change elsewhere broke them) are listed separately below. They are the ones that answer the person's question.
- Scripts: scratchpad `scan.py`, `summ.py`, `join.py` (not committed).

### Counts

- Core tests now: **597** (525 in the lib, 72 in `crates/core/tests/`, plus 17 `#[ignore]` reports).
- **132 of the 597** had at least one real failure in the record (plus 17 failing names that have since been renamed or removed).
- **465** never failed for real. By area they are mostly cheap unit tests: `ape` 32, `items` 27, `course` 20, `models` 17 of 30, `online` (integration) 28 of 33, `sound` 9, `fx` 8, `geom` 7, `speedo` 6, `trial` 5, `rules`, `sky`, `focus`, `vinesim`, `mesh`, `render`… Their total measured time is **under 30 s CPU**, spread over 16 threads.
- Browser check: about 70 checks per engine; **9 have ever failed**. Online race check: 3 have failed.

### Regression catches (the tests that have paid for themselves)

| Test | Real failures | What it caught |
| --- | --- | --- |
| `core_source_has_no_io_and_no_nondeterminism` (boundary) | 8 | Rule-2 leaks into the core (new code reaching for I/O or nondeterminism), many sessions |
| `every_kind_finishes_the_city_at_the_easiest_and_the_best_level` | 4 | AI or physics changes made for other maps stranded a kind on the City |
| `*_difficulties_are_ordered_and_spaced` (Jungle, City, Ski, Sector, Ruins) | 1 + 2 + 2 + 2 + 3 | AI tuning drift after shared changes (e.g. RC14 in v1.11.1, the 10-14 speed-ups) |
| `every_kind_finishes_<ski/sector>…` | 2 + 2 | as above |
| `*_unchanged` hashes (`maps_unchanged` ×3, `jungle_unchanged`, `sector_unchanged`) | 1–2 each | Every shared physics/AI change that would silently move other maps' races and medal times (10-14 23:51 cluster; 10-15 22:54) |
| `sectorscape::…the_title_flyover…glides_without_shaking` | 3 | The flyover shaking on a new map's terrain (RC8, Ruins) |
| `app::menu_dock::…` (several) | 6 tests | Lobby framing broken by each new map or by menu changes (1.12.0) |
| `app::weather_tests::…` | 6 tests | Weather/setup layout after the Bay squall and costumes |
| `ski::layout::every_lane_has_a_way_on…` | 2 | T84 vine changes |
| `models::…in_every_pose`, `without_gear_nothing_is_worn` | several | Gear/costume changes leaking into other maps' apes |
| `patch_notes::…` | 2 | Patch-note entries too long / not this version (1.12.0) |
| `the_snow::pick` overflow (X6) | 1 | Only the **debug** tests caught it (release wrapped). Keep `check.sh` on debug. |
| Browser: `1280x720: Race Setup fits without a scrollbar` | **5** (+1 `1280x560 Start stays in view`) | Race Setup overflowing after each new map/setting (latest 1.12.0 costumes) |
| Browser: `Race Setup offers every map and Random as cards` | 2+ | New map added to `Map::ALL` (SA29; Bay in 1.12) |
| Browser: `the race runs on a map Random picked…` | 1 | Random picking Bay before it was ready |
| Browser: `the menus' elements are kept…` / online `B's lobby keeps its elements…` | 1 / 2 | T65 menu rebuilds |
| Browser: `the arrow and Enter keys choose the Ski Slope` | 1 | Card keyboard navigation |
| Online: `A sees B's ape move` / `B sees A's apes move` | 1 / 1 | The T71 relay bug |

Only one failure was specific to one engine: Firefox's hover wobble (commit 318a8e4), which was a flaw in the check itself. The other two engines' failures matched Chromium's. This browser timing run also failed once by itself under load 50 (`wait_for_selector [id='menu.resume']`, 5 s timeout): the fixed 5 s waits are a flake risk while several agents run.

### Deliberate (rule-6) breaks, not counted

`map::random_picks_every_other_map_and_never_the_last`, `app::map_tests::random_picks_another_map_at_each_start…`, `app::online_play::random_online_the_host_picks…`, `skiscape::the_snow_is_the_floor_inside_the_fences…`, `baymodels::{a_set_sail_fills_its_gap…, the_fire_and_the_barrels_move…, typical_ships_fit_the_contract}`, browser `B's menus hold still while its dropdown is open`. SECOND-ORDER.md also records dozens more as break-once rows (S5, I8, M10, X8, OR7, MF6, BR6…). Those were run by hand, often with output filtered, so most do not show up in the scan.

## 2. Cost per test

Measured with `RUSTC_BOOTSTRAP=1 cargo test --workspace -- -Z unstable-options --report-time` (debug, as `check.sh` runs it). **The machine was under load 30–80 from other agents**, so absolute times are about 2× the quiet numbers. Earlier quiet runs in the transcripts show the lib binary at 22–29 s. The ranking holds.

Binary wall times (they run **one after another**): lib 60.9 s · ruins_ai 16.2 · city_ai 7.9 · maps_unchanged 7.7 · sector_ai 7.0 · ski_ai 6.9 · bay_ai 6.8 · sector_unchanged 4.8 · sector_finish 4.7 · ai 4.1 · jungle_unchanged 2.4 · online 0.1 · boundary 0.1 · map_pictures 0.0. **Sum ≈ 130 s**, of which the integration files are ≈ 69 s in a row. Compile is extra: about 44 s from clean here; incremental is smaller.

Slowest 25 (seconds, ever-failed count in brackets):

| s | Binary | Test | Failed |
| --- | --- | --- | --- |
| 58.1 | lib | `city::tests::moving_cars_drive_their_street_the_same_every_time` | 1 (dev) |
| 19.1 | lib | `app::ski_camera_tests::on_the_ski_slope_the_chase_camera_keeps_out_of_the_conifers_smoothly` | 0 |
| 16.2 | ruins_ai | `every_level_finishes_sixteen_ruins_layouts` | 0 |
| 8.1 | ruins_ai | `every_kind_finishes_the_ruins_at_the_easiest_and_the_best_level` | 0 |
| 7.9 | city_ai | `city_difficulties_are_ordered_and_spaced` | 2 |
| 7.7 | maps_unchanged | `the_city_plays_out_exactly_as_before_sector_ape` | 1 |
| 7.3 | city_ai | `every_kind_finishes_the_city_at_the_easiest_and_the_best_level` | 4 |
| 7.0 | ruins_ai | `ruins_difficulties_are_ordered_and_spaced` | 3 |
| 7.0 | sector_ai | `four_ai_apes_with_items_finish_sector_ape` | 0 |
| 6.9 | ski_ai | `ski_difficulties_are_ordered_and_spaced` | 2 |
| 6.8 | bay_ai | `bay_difficulties_are_ordered_and_spaced` | 0 |
| 6.3 | sector_ai | `sector_difficulties_are_ordered_and_spaced` | 2 |
| 5.7 | lib | `camera::hulls::on_the_bay_the_eye_keeps_out_of_the_sea_hulls_and_yards` | 0 |
| 5.5 | ski_ai | `every_kind_finishes_the_ski_slope_at_the_easiest_and_the_best_level` | 2 |
| 5.4 | lib | `app::look_tests::no_racer_wears_an_aura_and_the_minimap_dots_are_the_kinds` | 1 |
| 5.4 | lib | `ski::layout::every_lane_has_a_way_on_and_low_vines_to_climb_back_by` | 2 |
| 5.3 | sector_ai | `every_kind_finishes_sector_ape_at_the_easiest_and_the_best_level` | 2 |
| 4.9 | maps_unchanged | `the_ski_slope_plays_out_exactly_as_before_sector_ape` | 2 |
| 4.8 | sector_unchanged | `sector_ape_plays_out_exactly_as_before_the_speed_ups` | 2 |
| 4.7 | sector_finish | `every_finisher_ends_on_the_deck_or_at_the_horizon` | 0 |
| 4.2 | lib | `camera::on_sunset_ruins_the_eye_stays_in_the_temple_as_the_apes_go_in` | 0 |
| 4.1 | ai | `difficulties_are_ordered_and_spaced` | 1 |
| 4.0 | ruins_ai | `four_ai_apes_with_items_finish_the_ruins` | 0 |
| 3.9 | city_ai | `four_ai_apes_with_items_finish_the_city` | 0 |
| 3.2 | ski_ai | `four_ai_apes_with_items_finish_the_ski_slope` | 0 |

Every other test is under 3 s; 540 of them are under 0.1 s.

### Browser check

`check.py` prints one line per check, so each line was timestamped. Both runs, all three engines and then Chromium alone, **stopped at the same place under load 50–70**: `interactions.py:111` (`wait_for_selector("[id='menu.resume']", timeout=5000)` after Esc, with the locator already visible but the page too slow to settle). So this gives a partial breakdown, not a full one. The quiet totals come from `notes/check-times.md`: browser 220–330 s for three engines, about 75–110 s each.

Chromium, first ~108 s under load (seconds since the previous check):

| s | Step |
| --- | --- |
| 26.9 | holding W moves the ape after Go (countdown + a real race start) |
| 12.7 | page load to `data-ready` |
| 11.6 | Race opens the setup screen (first course build behind the menu) |
| 10.5 | the picker cycles through all six apes |
| 10.0 | the fixed idle (`BUILDER_IDLE`) |
| 8.7 | Settings opens |
| 4.3 / 4.2 / 3.2 / 3.1 | Start begins the race / volume slider / ape drawn / three apes race |

Where the time goes, per engine: about 10 s of fixed idle, about 5 s of fixed `wait_for_timeout`s (16 of them, 100–300 ms each), three page reloads (each a full WebAssembly load), and three real race starts with a countdown (Jungle, City, Random). Because the engines run one after another, three engines take three times as long. Running them in parallel is the biggest single saving in the whole check.

Online (`check-online.sh`, from check-times): transport 16–23 s, the two-tab race 74–125 s for three engines. Engines run in turn there too.

## 3. Consolidation candidates

Never-failed **and** slow, or overlapping another test.

| Test(s) | Never failed? | Proposed action | Est. saved (normal load) | Risk / what could slip through |
| --- | --- | --- | --- | --- |
| `city::moving_cars_drive_their_street_the_same_every_time` | failed once in dev | Build `city()` twice before the loop and compare `car_at` on the two copies, instead of calling `city()` inside the loop | 15–20 s wall (it is the lib's long pole) | None: it is the same determinism check against a fresh build. |
| All 13 `crates/core/tests/*.rs` | mixed | One integration binary (`tests/all/`), modules unchanged | 20–30 s wall + 12 fewer links | None for coverage. The AI tests already spawn threads, so 16 cores will be busy, but the total still drops to about the slowest test. The quick-check tags being added now would filter by module path (`ai::`, `city_ai::`). |
| `ruins_difficulties_are_ordered_and_spaced` + `every_level_finishes_sixteen_ruins_layouts` | the first failed 3×, the second never | One test: run the 16-layout races once and take the ordering means from the Chimpanzee runs on layouts 0–7 | ~7 s CPU | None, if both assertions stay. **Keep the ordering assertion**: it is among the most-failing tests. |
| `four_ai_apes_with_items_finish_{city,ski,ruins,sector}` + that map's `*_unchanged` hash | four-AI: never failed; hashes: failed | Hash the four-AI race and assert every ape finishes, in one run. Note the hash races run without items: either add an items race to the hash or keep the items race separate on maps where items matter. | ~15 s CPU | Small. If the hash is re-taken carelessly after an intended change, a "doesn't finish" bug could be blessed with it; keep the finish assertion explicit. |
| `sector_finish::every_finisher_ends_on_the_deck_or_at_the_horizon` + `sector_unchanged` + `four_ai_apes_with_items_finish_sector_ape` | sector_finish never; hash 2× | All three run 4-AI Sector races: use one set of seeded races for hash + on-deck + finished | ~8 s CPU | None if assertions are kept. `sector_finish` guards SF7 (a finisher falling past the deck) and should stay. |
| `maps_unchanged` (3) + `jungle_unchanged` (2) + `sector_unchanged` (1) | all failed at least once | One `maps_unchanged` table over every map (layout fingerprint + plain + lava hash), **adding Sunset Ruins and Banana Bay Battle**, which have no hash today | ~0 s (tidier, one place to re-take hashes) | **Gap today:** a shared physics change could silently move Ruins/Bay races and medal times. |
| `app::ski_camera_tests::…conifers_smoothly` | never | One thread per seed (or two tests) | ~8 s wall | None. Guards SW9 (eye inside a conifer), which the person saw; keep it. |
| `ski::layout::every_lane_has_a_way_on…` | 2× | Work out each vine's `progress`/`nearest_anywhere` once per course instead of per sample point | ~3 s | None (same assertions). |
| `camera::hulls::on_the_bay_the_eye_keeps_out…`, `camera::on_sunset_ruins_the_eye_stays_in_the_temple…` | never | Keep as they are, or move them to the full check only once the quick check exists | 0 / ~10 s off the quick check | Guard BR6 and the Ruins temple camera; a camera regression would only show when playing. |
| `every_level_finishes_sixteen_ruins_layouts`, `every_level_finishes_four_bay_layouts`, `every_level_gets_off_every_deck_back_onto_the_line` | first two never; deck test 2× | Keep in the full check; leave them out of the quick check | 0 / ~18 s off the quick check | RT10 (Sloth Gorilla never leaving the river) would slip through a quick-only run. |
| Browser: 3 engines in turn | Firefox/WebKit never failed alone (except a test flaw) | Run the engines in parallel; quick check Chromium only, deploy check all three | 120–200 s | Running in parallel loses nothing. Chromium-only on the quick check would miss a Safari-only bug; the deploy check still catches it. |
| Browser: 10 s idle per engine | never failed | 2 s idle; the final "no off-origin request at any point" check still covers the whole run | ~24 s (sequential) / ~8 s (parallel) | A request fired only after 10 s of idling on the title screen, with no clicks, would be missed. Unlikely, since nothing in the game is on a timer that long before input. |
| ~465 cheap never-failed unit tests (`ape`, `items`, `course`, `online`, `sound`, `geom`…) | never | **Keep as they are** | ≈0 | Deleting them would save nothing measurable and remove the online protocol, items and physics guards. |

Tests that never failed but must stay whatever they cost: the medal-time and difficulty reports behind `trial::medals` (they keep medal times honest), every `online` protocol test (a break there splits players' races silently), `boundary` (rule 2), and the `*_unchanged` hashes (other maps unchanged).

## Notes for whoever does this

- Do #1 and #2 first: they change no assertion, and #2 makes the rest of the AI-race merging matter less for wall time.
- After #2, re-run `--report-time` with the machine quiet. The load here inflated every number roughly 2×.
- Fixed waits (`wait_for_selector(..., timeout=5000)`) flaked once in this measurement while about 50 processes were competing. If agents keep running checks in parallel, raise those timeouts or serialise check runs.
