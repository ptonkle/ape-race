# Rattlesnake Ravine: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Updated with every report. Branch `ravine` (worktree `~/slop/ape-race-ravine`); held until the person says deploy.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done | `briefs/rattlesnake-ravine.md`; pitfalls from earlier maps in `notes/ravine-pitfalls.md` |
| 2 | Skeleton and contract | Done | Merged: `Map::Ravine` (held from the map grid and Random, `Map::held`), the `RavineLayout` contract (`ravine.rs`), a stand-in course (the Jungle's layout, own seed), night look, PROTOCOL 15; SECOND-ORDER RV1–RV6 |
| 3 | Ape costumes | Done | `Gear::Cowboy` (models/cowboy.rs): hat, bandana, gun belt, chaps, boots with spurs, bananas in a holster or bandolier, a mix and palette per kind; `SPIDER_JAGUAR` (models/spider.rs), eight banded legs; Costume "Cowboy"; shots `notes/ravine-shots/costumes/`; RG1–RG6. Branch `ravine-gear`; Map::Ravine wiring is two lines for the skeleton (RG6) |
| 4 | Models | Done (merged) | `crates/core/src/ravinemodels.rs` + `ravinemodels/`: snake vine and snapped, wall lizards (red/yellow/blue), mine cart, track with trestles, wall hole, T-rex skull from its tooth list, proboscis sphinx, campfire with roasting bananas, tents, mining props, rope bridge, log span, fossils, mushrooms, torch, fireflies. 25 gallery keys; shots in `notes/ravine-shots/models/`; SECOND-ORDER RM-1–RM-5 |
| 5 | Sky and lighting | Not started | |
| 6 | Course layout | Done (branch `ravine-course`, not merged) | The real 1,742 m route (`ravine/layout.rs`): braids, two arches, caves, a hairpin round a fin with the T-rex skull shortcut (saves 173 m, about 7 s), four tight corners, the split (upper passage 36 m shorter, sparser, more snakes; lower through the mine tunnel), 4 mine tracks from wall holes, 8 lizards, glows thickening to the mesa finish. Snakes about 15%, fixed list. 11 layout tests (SECOND-ORDER RC1–RC6). **With the merged rules the AI does not yet finish every race** (RC4: Sloth 3 DNF of 6; King Kong stuck once at the hairpin's fin tip) |
| 7 | Map rules | Done (merged) | Snake vines snap and drop the ape (regrow after 10 s, saved, synced); teeth reset to the floor (lava: the kill); walls, roofs, cave/tunnel sides, holes, skull bone, sphinx and rocks solid, swept and indexed; decks and solid tops are floors; camera keeps inside rooms and out of rock; lizard/cart bounce tested; AI avoids most snakes (Sloth least). Tested alone on bare courses (`ravine/rules_tests.rs`); SECOND-ORDER RR1–RR8 |
| 8 | Art placed on the course | Not started | |
| 9 | Lobby spot and flyover | Not started | |
| 10 | AI validation | Not started | |
| 11 | Medal times | Not started | |
| 12 | Step and frame cost | Not started | |
| 13 | Colourblind and JOJO MODE | Not started | |
| 14 | Map card | Not started | |
| 15 | Screenshots at 1920×1080 | Not started | |
| 16 | Other maps unchanged | In progress | 10:51: the other-maps-unchanged tests pass on `ravine` with skeleton, models, costumes and rules merged |
| 17 | Docs | Not started | |
| 18 | Full checks | Not started | |
| 19 | Deploy | Waiting on you | Held until you say |

## This map's features

Stages: Model, Gameplay (or n/a), Placed in course, Tested.

| Feature | Source | Model | Gameplay | Placed | Tested |
| --- | --- | --- | --- | --- | --- |
| Slot-canyon walls as barriers (moss, lichen, fossils, bones) | brief | – | Done | – | – |
| Starry sky, soft night light | brief | – | n/a | – | – |
| Torches, fireflies, glowing mushrooms leading to the finish | brief | Done | n/a | – | – |
| Rope bridges and logs with vines of varying length | brief | Done | – | – | – |
| Snake vines (~15%; snap, drop) | brief | Done | Done | – | – |
| Braided winding route | brief | n/a | – | – | – |
| Upper passage (short, sparse vines, more snakes) | brief | n/a | – | – | – |
| Lower passage: mine tunnel, equipment, digs | brief | Done | – | – | – |
| Caves, archways, long jumps, tight corners | brief | – | – | – | – |
| T-rex mouth shortcut (teeth reset to the floor) | brief | Done | Done | – | – |
| Wild West gear, palette per ape | brief | – | n/a | n/a | – |
| Wall lizards (bouncers, like the turtles) | brief | Done | Done | – | – |
| Mine-cart tracks from wall holes; carts like the City's cars | brief | Done | Done | – | – |
| Spider-jaguar (Floor is Lava) | brief | – | – | n/a | – |
| Mesa finish: proboscis sphinx, campfire, roasting bananas, tents | brief | Done | n/a | – | – |
