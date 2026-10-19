# The Spindle: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Branch `spindle` (skeleton `spindle-skeleton`), from upstream `main` 4ea504c (1.17.1); held until Peter releases it. Sam's requests are numbered here, `SP-T1` …; one `TRIAGE.md` row for the whole map is added just before the PR.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done (defaults) | `briefs/spindle.md`; `notes/spindle-pitfalls.md`; questions SP-Q1… below, building on the defaults |
| 2 | Skeleton and contract | Done | `Map::Spindle` held; `SpindleLayout` and `Warp` (`spindle.rs`); the tube simulated flat and drawn bent; SP1–SP3 |
| 3 | Ape costumes | Not started | morph suits |
| 4 | Models | Not started | |
| 5 | Sky and lighting | Done (first pass) | `spindlescape::look`: an even block light from every side, a pale haze toward the open end |
| 6 | Course layout | In progress | stand-in: 2,000 m, a diamond lattice of vines round the axle |
| 7 | Map rules | Done (skeleton) | wrap, axle ceiling, blocks, slime, spinning columns (`spindle/rules.rs`, tested alone) |
| 8 | Art placed on the course | In progress | |
| 9 | Lobby spot and flyover | In progress | |
| 10 | AI validation | Not started | |
| 11 | Medal times | Not started | stand-in medals |
| 12 | Performance test | Not started | |
| 13 | Colourblind and JOJO MODE | Not started | |
| 14 | Map card | Not started | |
| 15 | Screenshots at 1920×1080 | Not started | |
| 16 | Other maps unchanged | Not started | |
| 17 | Docs | Not started | |
| 18 | Full checks | Not started | |
| 19 | Deploy | Not started | Peter's |

## Sam's requests

| # | Request | Status |
| --- | --- | --- |
| SP-T1 | The map, from the brief (2026-10-18) | In progress |
