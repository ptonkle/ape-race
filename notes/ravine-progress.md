# Rattlesnake Ravine: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Updated with every report. Branch `ravine` (worktree `~/slop/ape-race-ravine`); held until the person says deploy.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done | `briefs/rattlesnake-ravine.md`; pitfalls from earlier maps in `notes/ravine-pitfalls.md` |
| 2 | Skeleton and contract | Done | Merged: `Map::Ravine` (held from the map grid and Random, `Map::held`), the `RavineLayout` contract (`ravine.rs`), a stand-in course (the Jungle's layout, own seed), night look, PROTOCOL 15; SECOND-ORDER RV1–RV6 |
| 3 | Ape costumes | In progress | agent on `ravine-gear` (started 09:48): Wild West gear, one palette per ape; spider-jaguar |
| 4 | Models | Done (merged) | `crates/core/src/ravinemodels.rs` + `ravinemodels/`: snake vine and snapped, wall lizards (red/yellow/blue), mine cart, track with trestles, wall hole, T-rex skull from its tooth list, proboscis sphinx, campfire with roasting bananas, tents, mining props, rope bridge, log span, fossils, mushrooms, torch, fireflies. 25 gallery keys; shots in `notes/ravine-shots/models/`; SECOND-ORDER RM-1–RM-5 |
| 5 | Sky and lighting | Not started | |
| 6 | Course layout | In progress | agent on `ravine-course` (started 10:13) |
| 7 | Map rules | In progress | agent on `ravine-rules` (started 10:13): snake snap, teeth reset, solids, camera |
| 8 | Art placed on the course | Not started | |
| 9 | Lobby spot and flyover | Not started | |
| 10 | AI validation | Not started | |
| 11 | Medal times | Not started | |
| 12 | Step and frame cost | Not started | |
| 13 | Colourblind and JOJO MODE | Not started | |
| 14 | Map card | Not started | |
| 15 | Screenshots at 1920×1080 | Not started | |
| 16 | Other maps unchanged | Not started | |
| 17 | Docs | Not started | |
| 18 | Full checks | Not started | |
| 19 | Deploy | Waiting on you | Held until you say |

## This map's features

Stages: Model, Gameplay (or n/a), Placed in course, Tested.

| Feature | Source | Model | Gameplay | Placed | Tested |
| --- | --- | --- | --- | --- | --- |
| Slot-canyon walls as barriers (moss, lichen, fossils, bones) | brief | – | – | – | – |
| Starry sky, soft night light | brief | – | n/a | – | – |
| Torches, fireflies, glowing mushrooms leading to the finish | brief | Done | n/a | – | – |
| Rope bridges and logs with vines of varying length | brief | Done | – | – | – |
| Snake vines (~15%; snap, drop) | brief | Done | – | – | – |
| Braided winding route | brief | n/a | – | – | – |
| Upper passage (short, sparse vines, more snakes) | brief | n/a | – | – | – |
| Lower passage: mine tunnel, equipment, digs | brief | Done | – | – | – |
| Caves, archways, long jumps, tight corners | brief | – | – | – | – |
| T-rex mouth shortcut (teeth reset to the floor) | brief | Done | – | – | – |
| Wild West gear, palette per ape | brief | – | n/a | n/a | – |
| Wall lizards (bouncers, like the turtles) | brief | Done | – | – | – |
| Mine-cart tracks from wall holes; carts like the City's cars | brief | Done | – | – | – |
| Spider-jaguar (Floor is Lava) | brief | – | – | n/a | – |
| Mesa finish: proboscis sphinx, campfire, roasting bananas, tents | brief | Done | n/a | – | – |
