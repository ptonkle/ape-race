# Quick check: run only the tests a change needs

The person, 2026-10-15: "give a go at classifying each of the unit tests and then each of the sets of changes with the same set of tags and only run tests if the feature change set included at least one change with the same tag" (then: all tests, the browser and online checks too; Chromium only unless asked).

## How it works

- `tests/tags.toml` holds the tags (22, each with a one-line meaning), then four maps:
  `[sources]` globs over repository paths → tags; `[tests]` cargo tests by `lib:<module path prefix>` or `test:<file>[:<name prefix>]` (longest key wins) → tags; `[browser]` the 11 steps of `tests/browser/interactions.py` → tags; `[online]` the two online checks → tags.
- `scripts/quick-check.sh [base]` (a wrapper for `scripts/tags.py quick`): the base is the commit of the newest passing `check` row in `notes/check-times.md`; changed files are `base..HEAD`, the working tree and untracked files. Prints files → tags, then runs: format, lint (always); the selected cargo tests in one `cargo test --workspace --lib --tests -- --exact <names>` (by name, so merging test files into one binary needs only new `[tests]` keys); release build and import audit (always); the browser check in Chromium with only the selected steps (`BUILDER_STEPS`; `boot` and the closing no-error/no-network checks always run; `page` runs every step); the online steps only on the `online` tag.
- `ALL_BROWSERS=1` (or `--all-browsers`) adds Firefox and WebKit: for a map merge or a big CSS/JS change. Evidence for Chromium only: in about 660 commits no game bug failed in just one engine (the one Firefox-only failure was the test's hover wobble, fixed in 318a8e4; the one Chromium-only failure was machine load).
- Fail safe: a changed file no glob matches prints its name and runs everything; foundations (`fx.rs`, `geom.rs`, `api.rs`, `map.rs`, `lib.rs`, the shim, `Cargo.*`, `build.sh`, `check.sh`, `index.html`, `web/vendor/`, `tags.toml`, `tags.py`) are tagged `all`. Docs, notes and briefs run nothing. `boundary` tests (the purity scanner) always run.
- The guard: `check.sh` step 4/7 runs `scripts/tags.py guard`, which fails on an untagged cargo test, browser step or online step, a stale key, an unknown tag, or a tracked file no glob matches.
- `scripts/timed-checks.sh quick` logs a row with kind `quick (<engines>; tags …)`; full rows now name their engines too. A failing run of any kind adds the failing tests and browser steps to `notes/test-failures.md` (`scripts/tags.py failures`).
- `--range A..B --dry-run` shows what a past change would select (with today's tests).

## Tests per tag (cargo, 612 tests)

physics 218, race 208, render 156, menus 103, items 89, maps-sector 73, maps-city 63, costumes 59, online 59, maps-ski 59, maps-jungle 56, ai 48, maps-bay 47, maps-ruins 41, replay 38, camera 36, sound 23, all 17 (fx, geom, map: only on a foundation change), weather 14, boundary 13, notes 7, page 2. A test can carry several tags.

## Proof on past commits

Run with `--range <c>^..<c>` on this branch, so the tests are today's. The machine was shared with other agents' browser checks all afternoon (load 36–59 on 16 cores), so every time here is 1.5–2× a quiet run; the comparison is with the full Chromium run measured in the same conditions: main's browser check alone took 417 s (load ~44) and this branch's stepped one 426 s (load ~40, the steps' own reloads add ~2%); `cargo test --workspace` 60–90 s; so a full Chromium `check.sh` is about 510 s here, plus ~90–150 s for the online check, and three engines (the old default) about 2.5× the browser part.

| Commit | Change | Tags hit | Cargo tests | Browser steps (Chromium) | Online | Quick total | Full here |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 06736bb | patch notes (`data/patch-notes.json`) | notes, boundary | 20 / 612 | boot, whats_new | skipped | **85 s** (load 47) | ~510 s + online |
| c73ac85 | race camera (`camera.rs`, `camera/hulls.rs`) | camera, boundary | 49 / 612 | boot, race | skipped | **144 s** (load 36) | ~510 s + online |
| 2580c30 | Ski Slope low vines (`ai.rs`, `ski/layout*`, `maps_unchanged.rs`) | ai, maps-ski, maps-city, maps-jungle, boundary | 175 / 612 | boot, race, time_trial, city_map, weather, random_map | skipped | **374 s** (load 48–59; cargo 78 s, browser 293 s) | ~510 s + online |
| 04196a7 | Banana Bay visuals (`baymodels`, `bayscape*`, `scene.rs`) | maps-bay, render, boundary | 193 / 612 | boot, race, secret, city_map, weather, random_map | skipped | failed at `weather` on a 5 s wait (load 57, see SECOND-ORDER QC3); ~290 s by its steps' times | ~510 s + online |
| 2b0e996 | Race Setup CSS (`web/style.css`) | page, boundary | 15 / 612 | every step (page) | skipped | ≈ full browser in Chromium; saves the other two engines and the online check | ~510 s + online |

The saving is largest for the common small changes (patch notes, one feature's menus, the camera): 85–145 s instead of ~510 s plus ~90–150 s online, in the same conditions. A change to the physics, the AI or `app.rs` still runs most of it, by design.

## Mutation check

`speedo::kmh` broken (×36 → ×35, `speedo.rs` is tagged menus only), `quick-check.sh HEAD`: tags menus and boundary, 113 of 612 cargo tests, and `speedo::tests::km_per_hour_from_metres_per_second` failed "assertion `left == right` failed left: 35 right: 36", exit 101; `scripts/tags.py failures` named it. Restored.

## Limits

- A test is only as safe as its tags. Tags are per file or name prefix; when unsure the file got the wider tag. `app.rs` carries most feature tags; edits there run much of the suite.
- Timeouts of 5 s in the browser steps fail under heavy load (SECOND-ORDER QC3); in quiet conditions they pass.
- The full `check.sh`, three engines, stays the gate before a map merge or a big change.
