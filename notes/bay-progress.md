# Banana Bay Battle: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Updated with every report.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done | `briefs/banana-bay-battle.md` |
| 2 | Skeleton and contract | Done | `Map::Bay`, `bay.rs` BayLayout contract, stand-in course, PROTOCOL 11 (on main 85d0e5a) |
| 3 | Ape costumes | In progress | Gear agent: pirate outfit |
| 4 | Models | Done | `baymodels.rs` (models branch): 4 ship kinds × 2 sides from the contract's `Ship`, barrels, dock, island, chest, doubloons, crew, cannon fire; shots in `notes/bay-shots/models/` |
| 5 | Sky and lighting | Not started | 3 pm Caribbean; Squall weather |
| 6 | Course layout | In progress | Course agent: dock, ~20 ships, island |
| 7 | Map rules | In progress | Water reset, barrel wall, bouncy barrels |
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
| 19 | Deploy | Not started | |

## This map's features

One row per concept from the brief, plus any added later. Stages: **Model**, **Gameplay** (n/a if only scenery), **Placed in course**, **Tested**.

| Feature | Source | Model | Gameplay | Placed in course | Tested |
| --- | --- | --- | --- | --- | --- |
| 3 pm Caribbean sky and light | Brief | Not started | n/a | Not started | Not started |
| Pirate outfit (tricorn/bandana, eyepatch, cutlass, bandolier, peg leg, parrot…) | Brief | In progress | n/a | Not started | Not started |
| Still ships, 3+ kinds (galleon, frigate, sloop), ~20 | Brief | Done | n/a | Not started | Not started |
| Pirate and nonspecific navy flags | Brief | Done | n/a | Not started | Not started |
| Ropes from masts and yards; sails missing where you swing | Brief | Done | Not started | Not started | Not started |
| Decks you land and run on | Brief | n/a | Not started | Not started | Not started |
| Moving textured water (up and down swell) | Brief | Not started | n/a | Not started | Not started |
| Water fall: back on the last ship you were over | Brief | n/a | Not started | Not started | Not started |
| Flagged boundary barrels (soft wall) | Brief, your answer | Done | Not started | Not started | Not started |
| Bouncy barrels in the water | Your addition | Done | Not started | Not started | Not started |
| Cannon fire between ships (scenery) | Brief, your answer | Done | n/a | Not started | Not started |
| Low-detail monkey pirates fighting on decks | Brief | Done | n/a | Not started | Not started |
| Doubloons on item vines | Brief | Done | n/a | Not started | Not started |
| Finish island with a chest of golden bananas | Brief | Done | n/a | Not started | Not started |
| Squall weather (rain, dark sky, rough swell, lightning) | Your answer | Not started | n/a | Not started | Not started |
