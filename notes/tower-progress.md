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
| 8 | Art placed on the course | Done | `towerscape/art.rs`, all from the layout (TA1); 3D culling, climb and zipper drawn apart (TA2); 8,691 shapes, 4,896 at most at once (TA3); shots in notes/tower-shots/art/ |
| 9 | Lobby spot and flyover | Waiting on you | Lobby on the start deck, keep beside the ape (TL1); flyover a helix up round the keep ending over the battlements, looking at the tear (TL2); shots notes/tower-shots/lobby/. The view is plain (TL8): pick start deck or a high bridge |
| 10 | AI validation | Done | 16 layouts: Sloth 107.2 s, Monkey 92.0, Ape 77.3, King Kong 64.5 (max 137.9 s); ordering test `tower_difficulties_are_ordered_and_spaced` (TC5) |
| 11 | Medal times | Done | Lemur 58.42 s best (58.4–67.7 by kind): Author 0:58, Gold 1:05, Silver 1:15, Bronze 1:30; test `the_tower_medals_follow_its_time_trial` (TM1) |
| 12 | Step and frame cost | Done | Step: worst 4.0 ms, median 111 µs (Ruins 2.7–3.9 ms / 56 µs, Bay 3.3 ms / 67 µs); fix suggested for `nearest_3d` (TM2). Frame (scenery): 8,691 shapes in all, 4,896 at most at once (TA3) |
| 13 | Colourblind and JOJO MODE | Done | Catch-up now crimson, apart from every beam (TL3); title and lobby draw beams (TL4); shots blind.png, jojo.png (TL5) |
| 14 | Map card | Done | `data/maps/tower.webp` from the flyover at 930 m: battlements, cloud band, eldritch banana (TL7); on the Race Setup strip |
| 15 | Screenshots at 1920×1080 | Done | `notes/tower-shots/` (art, zipper, sky, lobby, models, costumes) at 1920×1080; `art/through-tear.png` a chase shot of an ape flying through the tear (TA8) |
| 16 | Other maps unchanged | Done | Full debug `cargo test --workspace` passes: every other map's tests unchanged; the tower's changes are behind `Course.tower` |
| 17 | Docs | Done | README (How to play row, Layout row, costume Wizard), MAPS.md "What Mape's Tower taught", TRIAGE T92, HANDOFF |
| 18 | Full checks | Done | check.sh passed in all three engines (332 s); check-online.sh failed on a 9 px lobby overflow in Firefox/WebKit (TA10), fixed on main (smaller seat text) and passed in all three engines |
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
