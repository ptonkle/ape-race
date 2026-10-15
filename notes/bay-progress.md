# Banana Bay Battle: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Updated with every report.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done | `briefs/banana-bay-battle.md` |
| 2 | Skeleton and contract | Done | `Map::Bay`, `bay.rs` BayLayout contract, stand-in course, PROTOCOL 11 (on main 85d0e5a) |
| 3 | Ape costumes | Done | Six pirate mixes (tricorn/bandana, eyepatch, cutlass, bandolier, peg leg, parrots, hook…), navy officer jaguar; on the journal |
| 4 | Models | Done | `baymodels.rs` (models branch): 4 ship kinds × 2 sides from the contract's `Ship`, barrels, dock, island, chest, doubloons, crew, cannon fire; shots in `notes/bay-shots/models/` |
| 5 | Sky and lighting | Done | 3 pm sun 40° up, cumulus, moving tiled sea from the physics swell; Squall (rain, slate sky, whitecaps, lightning) offered on the bay only |
| 6 | Course layout | Done | 1,458 m, 30 ships (8 galleons, 8 frigates, 8 brigs, 6 sloops), 158 vines, no set sails on the line |
| 7 | Map rules | Done | Water puts you on the last ship passed (Floor is Lava: eaten); barrel soft wall; bouncy barrels |
| 8 | Art placed on the course | In progress | Art agent |
| 9 | Lobby spot and flyover | Not started | |
| 10 | AI validation | In progress | All levels finish; Sloth 69.0, Monkey 68.5, Ape 54.4, King Kong 48.9 s: Sloth and Monkey need spacing | |
| 11 | Medal times | In progress | |
| 12 | Step and frame cost | In progress | |
| 13 | Colourblind and JOJO MODE | Not started | |
| 14 | Map card | Not started | |
| 15 | Screenshots at 1920×1080 | In progress | Models on the journal | |
| 16 | Other maps unchanged | Not started | |
| 17 | Docs | Not started | |
| 18 | Full checks | Not started | |
| 19 | Deploy | Not started | |

## This map's features

One row per concept from the brief, plus any added later. Stages: **Model**, **Gameplay** (n/a if only scenery), **Placed in course**, **Tested**.

| Feature | Source | Model | Gameplay | Placed in course | Tested |
| --- | --- | --- | --- | --- | --- |
| 3 pm Caribbean sky and light | Brief | Done | n/a | Done | Done |
| Pirate outfit (tricorn/bandana, eyepatch, cutlass, bandolier, peg leg, parrot…) | Brief | In progress | n/a | Not started | Not started |
| Still ships, 3+ kinds (galleon, frigate, sloop), ~20 | Brief | Done | n/a | Not started | Not started |
| Pirate and nonspecific navy flags | Brief | Done | n/a | Not started | Not started |
| Ropes from masts and yards; sails missing where you swing | Brief | Done | Not started | Not started | Not started |
| Decks you land and run on | Brief | n/a | Not started | Not started | Not started |
| Moving textured water (up and down swell) | Brief | Done | n/a | Done | Done |
| Water fall: back on the last ship you were over | Brief | n/a | Not started | Not started | Not started |
| Flagged boundary barrels (soft wall) | Brief, your answer | Done | Not started | Not started | Not started |
| Bouncy barrels in the water | Your addition | Done | Not started | Not started | Not started |
| Cannon fire between ships (scenery) | Brief, your answer | Done | n/a | Not started | Not started |
| Low-detail monkey pirates fighting on decks | Brief | Done | n/a | Not started | Not started |
| Doubloons on item vines | Brief | Done | n/a | Not started | Not started |
| Finish island with a chest of golden bananas | Brief | Done | n/a | Not started | Not started |
| Squall weather (rain, dark sky, rough swell, lightning) | Your answer | Done | n/a | Done | Done |
