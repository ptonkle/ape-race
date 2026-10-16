# Mape's Tower: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Updated with every report.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done | `briefs/mapes-tower.md` |
| 2 | Skeleton and contract | Done | `Map::Tower`, `TowerLayout` (`tower.rs`), 3D centreline with progress in 3D (TW1), mirrored zipper (TW3), reset pads with boost (TW4), climb-lift hook (TW5), Twilight/Dawn registered (TW7), PROTOCOL 13; stand-in course finished by every AI level (TW10) |
| 3 | Ape costumes | Done | Wizard and witch mixes for all six kinds; the flame-demon cat (TG rows); wired to Map::Tower |
| 4 | Models | Done | Climb half (towermodels.rs) and zipper half (zipper.rs, creatures.rs, deep.rs): keep, sub-towers, mages, dragons, beams, trampolines, room, clouds, summit, rift and eldritch banana, the zip, topsy towers, blobs, sea monsters, void platforms, banana sea |
| 5 | Sky and lighting | Done (sky branch) | Clear golden afternoon, Twilight, Dawn by height; zipper void; cloud band; follows JOJO and colourblind (TS1–TS6); shots in notes/tower-shots/sky/ |
| 6 | Course layout | Done | 674 m climb (race 1,348 m): keep circle 55 m, 4 spirals (540° round), 3 straight branches (90° round sub-towers), trampoline room, clouds, summit turns out from the keep, mirrored zipper with longer beams (TC1, TC5); any-direction beams (TC2) |
| 7 | Map rules | Done | Pads aimed at the next station; AI bounds/re-pick (TC3); summoned vines placed by their own coil; CLIMB_LIFT stays 3 m/s (the climb is now the faster half) (TC5) |
| 8 | Art placed on the course | Not started |  |
| 9 | Lobby spot and flyover | Not started |  |
| 10 | AI validation | Done | 16 layouts: Sloth 107.2 s, Monkey 92.0, Ape 77.3, King Kong 64.5 (max 137.9 s); ordering test `tower_difficulties_are_ordered_and_spaced` (TC5) |
| 11 | Medal times | Done | Lemur 58.42 s best (58.4–67.7 by kind): Author 0:58, Gold 1:05, Silver 1:15, Bronze 1:30; test `the_tower_medals_follow_its_time_trial` (TM1) |
| 12 | Step and frame cost | In progress | Step: worst 4.0 ms, median 111 µs (Ruins 2.7–3.9 ms / 56 µs, Bay 3.3 ms / 67 µs); fix suggested for `nearest_3d` (TM2). Frame (scenery) cost: art agent |
| 13 | Colourblind and JOJO MODE | Not started |  |
| 14 | Map card | Not started |  |
| 15 | Screenshots at 1920×1080 | Not started |  |
| 16 | Other maps unchanged | Not started |  |
| 17 | Docs | Not started |  |
| 18 | Full checks | Not started |  |
| 19 | Deploy | Not started |  |

## This map's features

| Feature | Source | Model | Gameplay | Placed in course | Tested |
| --- | --- | --- | --- | --- | --- |
| Giant tower climbed up and around | Brief | Not started | Not started | Not started | Not started |
| Offshoot branches to sub-towers (straight runs) | Brief | Not started | Not started | Not started | Not started |
| Climb vines give an upward boost | Brief | Not started | Hook in (placeholder 3 m/s, TW5) | Not started | Not started |
| Mages on balconies and in towers | Brief | Not started | Not started | Not started | Not started |
| Mages on dragons outside | Brief | Not started | n/a | Not started | Not started |
| Spell-beam vines, orientation by spell | Brief | Not started | Not started | Not started | Not started |
| Trampolines | Brief | Not started | Not started | Not started | Not started |
| Trampoline room with orb mage, exit through roof | Brief | Not started | Not started | Not started | Not started |
| Forgiving resets with a speed boost | Brief | Not started | Rule in, boosts placeholder (TW4) | Not started | Not started |
| Climb through clouds; cloud anchors | Brief | Not started | Not started | Not started | Not started |
| Sparse vines, diverging paths | Brief | Not started | Not started | Not started | Not started |
| Insane wizard and the tear in reality | Brief | Not started | Not started | Not started | Not started |
| Giant eldritch evil banana | Brief | Not started | n/a | Not started | Not started |
| Zipper space: mirrored, topsy-turvy, going down | Brief | Not started | Mirrored path in (TW3); topsy-turvy not yet | Not started | Not started |
| Blob and sea-monster vine emitters | Brief | Not started | Not started | Not started | Not started |
| Magic-made platforms | Brief | Not started | Not started | Not started | Not started |
| Infinite bananas finish, dancing apes | Brief | Not started | Not started | Not started | Not started |
| Wizard and witch outfits | Brief | Not started | n/a | Not started | Not started |
| Weathers: Twilight and Dawn | Brief | Not started | Registered, placeholder skies (TW7) | Not started | Not started |
| Balrog-like flame cat (Floor is Lava) | Brief | Not started | Not started | Not started | Not started |
| Climb then fall: progress along a 3D path | Skeleton (TW1) | n/a | Done | Stand-in | `tower::tests`, `tests/tower_ai.rs` |
