# Rattlesnake Ravine: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Updated with every report. Branch `ravine` (worktree `~/slop/ape-race-ravine`); held until the person says deploy.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done | `briefs/rattlesnake-ravine.md`; pitfalls from earlier maps in `notes/ravine-pitfalls.md` |
| 2 | Skeleton and contract | Done | Merged: `Map::Ravine` (held from the map grid and Random, `Map::held`), the `RavineLayout` contract (`ravine.rs`), a stand-in course (the Jungle's layout, own seed), night look, PROTOCOL 15; SECOND-ORDER RV1–RV6 |
| 3 | Ape costumes | Done | `Gear::Cowboy` (models/cowboy.rs): hat, bandana, gun belt, chaps, boots with spurs, bananas in a holster or bandolier, a mix and palette per kind; `SPIDER_JAGUAR` (models/spider.rs), eight banded legs; Costume "Cowboy"; shots `notes/ravine-shots/costumes/`; RG1–RG6. Merged and wired; hats redone level above the ears; spider-jaguar uses the jaguar's eyes |
| 4 | Models | Done (merged) | `crates/core/src/ravinemodels.rs` + `ravinemodels/`: snake vine and snapped, wall lizards (red/yellow/blue), mine cart, track with trestles, wall hole, T-rex skull from its tooth list, proboscis sphinx, campfire with roasting bananas, tents, mining props, rope bridge, log span, fossils, mushrooms, torch, fireflies. 25 gallery keys; shots in `notes/ravine-shots/models/`; SECOND-ORDER RM-1–RM-5 |
| 5 | Sky and lighting | Done (merged) | `ravinescape.rs`: starry sky, Milky Way (faint, to lift in the art pass), moon, night fog; shots on the journal |
| 6 | Course layout | Done (Sloth accepted; mesa opened up) | The real 1,754 m route (`ravine/layout.rs`): braids, two arches, caves, a hairpin round a fin (48 m corners, round 10 m tip) with the T-rex skull shortcut (saves 185 m, 7.7 s), four tight corners, the split (upper passage shorter, sparser, more snakes; lower through the mine tunnel), 4 mine tracks, 8 lizards, glows to the mesa finish. Snakes about 15%, each with a low vine past it. 14 layout tests (SECOND-ORDER RC1–RC9). **AI (6 layouts):** Sloth 127.4 s, King Kong 76.2 s, Ape 98.3 s, all finish (before: Sloth 3 DNF, King Kong 1). Over 12 layouts the Sloth still misses the 150 s cap 5 times (scattered snaps and swings, no single blocker, RC9) |
| 7 | Map rules | Done (merged) | Snake vines snap and drop the ape (regrow after 10 s, saved, synced); teeth reset to the floor (lava: the kill); walls, roofs, cave/tunnel sides, holes, skull bone, sphinx and rocks solid, swept and indexed; decks and solid tops are floors; camera keeps inside rooms and out of rock; lizard/cart bounce tested; AI avoids most snakes (Sloth least). Tested alone on bare courses (`ravine/rules_tests.rs`); SECOND-ORDER RR1–RR8 |
| 8 | Art placed on the course | Done (on `ravine`) | `ravinescape/course.rs`: walls (3 sandstone bands to the solids' extents, fin pieces too, moss, lichen, fossils, dinosaur bones, boulders), roofs, decks (level rope bridges, logs), junipers/pegs/knots for hangers, tracks and wall holes, rocks, skull, sphinx, tents, far buttes, tunnel gear and digs; moving: snakes (snapped from `snapped_at`, warning glow, black/white in colourblind), lizards, carts, torches, lanterns, mushrooms, fireflies, campfire. Worst frame 3,909 still + 991 moving = 4,900 (sky ~770 apart). Fixed: skull jaw plates, lemur ears under the hat, Milky Way haze (still faint). Shots `notes/ravine-shots/course/`. SECOND-ORDER RA-1–RA-8 |
| 9 | Lobby spot and flyover | Done | Lobby on a fallen log 46 m up over the start straight, looking down the canyon to the bridge and the arch (`ravine/lobby.rs`, `ravinescape/lobby.rs`); flyover down the slot under the bridges, past the skull, along the upper passage; Milky Way lifted; shots `notes/ravine-shots/lobby/`; SECOND-ORDER RL1–RL5 (RL1 asks you about the spot) |
| 10 | AI validation | In progress | 16 layouts: Sloth 130.7 s (7 timeouts, accepted), Ape 96.8 s (all finish), King Kong 76.8 s (1 timeout, layout 1 at 1115 m; agent on `ravine-kk`) |
| 11 | Medal times | Not started | |
| 12 | Performance test | In progress | Drawing: 4,900 of 5,500 shapes at the hairpin (before the Milky Way's +260); game tick time not measured yet |
| 13 | Colourblind and JOJO MODE | In progress | Snake cue done; whole-map check on `ravine-cb` (with skull light, mesa wall ends) |
| 14 | Map card | In progress | on `ravine-cb`; the card shows Canopy Run's picture now |
| 15 | Screenshots at 1920×1080 | Not started | |
| 16 | Other maps unchanged | In progress | 13:20: 1.14.4 merged in, other-maps tests pass; again after art (step 8). 10:51: the other-maps-unchanged tests pass on `ravine` with skeleton, models, costumes and rules merged |
| 17 | Docs | Not started | |
| 18 | Full checks | Not started | |
| 19 | Deploy | Waiting on you | Held until you say |

## This map's features

Stages: Model, Gameplay (or n/a), Placed in course, Tested.

| Feature | Source | Model | Gameplay | Placed | Tested |
| --- | --- | --- | --- | --- | --- |
| Slot-canyon walls as barriers (moss, lichen, fossils, bones) | brief | Done | Done | Done | Done |
| Starry sky, soft night light | brief | Done | n/a | Done | Done |
| Torches, fireflies, glowing mushrooms leading to the finish | brief | Done | n/a | Done | Done (budget) |
| Rope bridges and logs with vines of varying length | brief | Done | – | Done | Done (budget) |
| Snake vines (~15%; snap, drop) | brief | Done | Done | Done (drawn, snapped state, colourblind cue) | Done |
| Braided winding route | brief | n/a | Done | Done | Done |
| Upper passage (short, sparse vines, more snakes) | brief | n/a | Done | Done | Done |
| Lower passage: mine tunnel, equipment, digs | brief | Done | – | Done | – |
| Caves, archways, long jumps, tight corners | brief | – | Done | Done | Done |
| T-rex mouth shortcut (teeth reset to the floor) | brief | Done | Done | Done | Done |
| Wild West gear, palette per ape | brief | Done | n/a | n/a | Done |
| Wall lizards (bouncers, like the turtles) | brief | Done | Done | Done | – |
| Mine-cart tracks from wall holes; carts like the City's cars | brief | Done | Done | Done | – |
| Spider-jaguar (Floor is Lava) | brief | Done | Done | n/a | – |
| Mesa finish: proboscis sphinx, campfire, roasting bananas, tents | brief | Done | n/a | Done (open mesa, low rims, fire 2.5×, tents 2.2×, layout sticks; RA-9) | Done (sphinx frame, rims, sticks) |
