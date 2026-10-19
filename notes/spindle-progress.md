# The Spindle: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Branch `spindle` (skeleton `spindle-skeleton`), from upstream `main` 4ea504c (1.17.1); held until Peter releases it. Sam's requests are numbered here, `SP-T1` …; one `TRIAGE.md` row for the whole map is added just before the PR.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done (defaults) | `briefs/spindle.md`; `notes/spindle-pitfalls.md`; questions SP-Q1… below, building on the defaults |
| 2 | Skeleton and contract | Done | `Map::Spindle` held; `SpindleLayout` and `Warp` (`spindle.rs`); the tube simulated flat and drawn bent; SP1–SP3 |
| 3 | Ape costumes | Done | `Gear::Morph` (`models/morph.rs`): one suit colour per racer (seven colourways), glowing seams, the face covered; the cat in ocelot colours (`BLOCK_OCELOT`); shots `notes/spindle-shots/costumes/`; SP-G1–G9 |
| 4 | Models | Done (reused) | the blocks are cubes; space reuses Sector Ape's `spacecraft` and backdrop |
| 5 | Sky and lighting | Done (first pass) | `spindlescape::look`: an even block light from every side, a pale haze toward the open end |
| 6 | Course layout | Done (MVP) | 2,000 m, eight dropper sections; a lattice of vines round the axle; SP4 |
| 7 | Map rules | Done | wrap, axle ceiling, blocks, slime, spinning columns, the launch and space drift (`spindle/rules.rs`, tested alone) |
| 8 | Art placed on the course | Done (MVP) | wool-block wall in section dyes, lamp blocks, axle and dispensers, the blocks, space past the end; `spindlescape/` |
| 9 | Lobby spot and flyover | Done (first pass) | the floor 130 m down the tube; a rolling glide down it |
| 10 | AI validation | In progress | SP5 |
| 11 | Medal times | Done | Author 0:59, Gold 1:06, Silver 1:16, Bronze 1:31; SP7 |
| 12 | Performance test | Done (first pass) | worst frame 4,821 of 5,500 (`the_spindle_frame_stays_within_budget`); SP6 |
| 13 | Colourblind and JOJO MODE | Not started | |
| 14 | Map card | Done (stand-in) | to retake from the finished art |
| 15 | Screenshots at 1920×1080 | Done | on the design journal |
| 16 | Other maps unchanged | Not started | |
| 17 | Docs | Not started | |
| 18 | Full checks | Not started | |
| 19 | Deploy | Not started | Peter's |

## Sam's requests

| # | Request | Status |
| --- | --- | --- |
| SP-T1 | The map, from the brief (2026-10-18) | In progress |
