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
| 8 | Art placed on the course | Done | Three polish passes: camera out of hulls/yards, huge chest, cream sails, a 51-ship background battle, foam waterlines, soft splashes and smoke (BA, BP, BQ, BR rows) |
| 9 | Lobby spot and flyover | Done | Ape at the pier's seaward end, the first galleon framed between panels and ape at 3 sizes, host and joiner; flyover along the pirate side (BL1–BL3) |
| 10 | AI validation | Done | Own Bay handicaps (BT1): 16 layouts Sloth 76.8, Monkey 67.1, Ape 59.7, King Kong 49.7 s; ordering test over 16 layouts |
| 11 | Medal times | Done | Author 0:35, Gold 0:42, Silver 0:52, Bronze 1:07 (best kind 35.06 s, BT2) |
| 12 | Step and frame cost | Done | Worst step 1.9–3.3 ms (like the Ruins); browser frames 203–288 ms vs Ruins 173–359 ms in software rendering (BL5) |
| 13 | Colourblind and JOJO MODE | Done | Sea, sky and clouds follow both modes; flags, barrels and cannon fire muted in colourblind (BP5, BQ5) |
| 14 | Map card | Done | `data/maps/bay.webp` from the flyover: navy frigate, pirate line, a hull hit |
| 15 | Screenshots at 1920×1080 | Done | Models, outfits, sky, three rounds of in-game shots, lobby and flyover on the journal |
| 16 | Other maps unchanged | Done | `jungle_unchanged`, `maps_unchanged`, `sector_unchanged` and the Ruins tests pass on final main |
| 17 | Docs | Done | README, MAPS.md (what the bay taught), TRIAGE T87, HANDOFF; Settings weather select fixed (BW1) |
| 18 | Full checks | Done | Final main: check.sh 307.6 s and check-online.sh 94.0 s, passed in Chromium, Firefox, WebKit |
| 19 | Deploy | Done | Live as v1.12.0 on 2026-10-15 (with T84, T88, T89), the T82 try on top as v1.12.1 |

## This map's features

One row per concept from the brief, plus any added later. Stages: **Model**, **Gameplay** (n/a if only scenery), **Placed in course**, **Tested**.

| Feature | Source | Model | Gameplay | Placed in course | Tested |
| --- | --- | --- | --- | --- | --- |
| 3 pm Caribbean sky and light | Brief | Done | n/a | Done | Done |
| Pirate outfit (tricorn/bandana, eyepatch, cutlass, bandolier, peg leg, parrot…) | Brief | Done | n/a | Done | Done |
| Still ships, 3+ kinds (galleon, frigate, sloop), ~20 | Brief | Done | n/a | Done | Done |
| Pirate and nonspecific navy flags | Brief | Done | n/a | Done | Done |
| Ropes from masts and yards; sails missing where you swing | Brief | Done | Done | Done | Done |
| Decks you land and run on | Brief | n/a | Done | Done | Done |
| Moving textured water (up and down swell) | Brief | Done | n/a | Done | Done |
| Water fall: back on the last ship you were over | Brief | n/a | Done | Done | Done |
| Flagged boundary barrels (soft wall) | Brief, your answer | Done | Done | Done | Done |
| Bouncy barrels in the water | Your addition | Done | Done | Done | Done |
| Cannon fire between ships (scenery) | Brief, your answer | Done | n/a | Done | Done |
| Low-detail monkey pirates fighting on decks | Brief | Done | n/a | Done | Done |
| Doubloons on item vines | Brief | Done | n/a | Done | Done |
| Finish island with a chest of golden bananas | Brief | Done | n/a | Done | Done |
| Squall weather (rain, dark sky, rough swell, lightning) | Your answer | Done | n/a | Done | Done |
| Background battle beyond the barrels (51 ships, some burning) | Integrator, from the shots | Done | n/a | Done | Done |
| Cream canvas; rolled furled sails; ragged torn sails | Integrator, from the shots | Done | n/a | Done | Done |
| Camera keeps out of hulls, decks and yards | Integrator, from the shots | n/a | Done | Done | Done |
