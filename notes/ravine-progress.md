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
| 10 | AI validation | Done | 16 layouts: Sloth 131.9 s (5 timeouts, accepted), Ape 97.2 s and King Kong 77.1 s finish all (King Kong no longer runs at vines behind rock, RC15) |
| 11 | Medal times | In progress | agent on `ravine-medals` |
| 12 | Performance test | In progress | Drawing: 4,926 of 5,500 shapes at the hairpin; game tick time being measured on `ravine-medals` |
| 13 | Colourblind and JOJO MODE | Done | Whole map checked in both (`notes/ravine-shots/polish/colourblind.png`, `jojo.png`): vines, item flowers, snakes, bananas, carts read; wall lizards now go pale with a white glow in colourblind (RA-13). Also: skull torch-lit (4 torches, mouth glow, warm light, teeth glow) and the canyon mouth's steps dressed with talus and shoulders (RA-12). Worst frame 4,926 of 5,500 |
| 14 | Map card | Done | `data/maps/ravine.webp` from the flyover at 520 m: the torch-lit skull and the canyon beyond (was Canopy Run's picture); `map_cards.py --only ravine` shoots a held map; text "Rattlesnake Ravine"; grid shot `polish/map-grid.png` (RA-14) |
| 15 | Screenshots at 1920×1080 | Not started | |
| 16 | Other maps unchanged | In progress | Pass after every merge (last 14:35, after the colourblind merge) |
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
