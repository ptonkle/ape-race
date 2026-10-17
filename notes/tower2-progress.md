# Mape's Tower, second design: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Updated with every report. Branch `tower2` (worktree `~/slop/ape-race-tower`); the first design stays live until this one is checked.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done | `briefs/mapes-tower-v2.md`, drawing `briefs/mapes-tower-v2.png` |
| 2 | Skeleton and contract | Done (contract); stand-in course In progress | `tower.rs` contract merged (TW2-1..8); the tube air measured (TW2-5); merged `tower2-ai`: a straight shaft up tower 1's axis; Monkey, Ape and King Kong finish the stand-in (135.7, 130.6, 90.0 s), the Sloth runs out of time in the tube (TW2-9..13) |
| 3 | Ape costumes | Done (v1) | Unchanged: wizard and witch mixes |
| 4 | Models | Done (unseen in the browser) | `towermodels/towers.rs`, `tube.rs`, `portal.rs` (TM2 rows); gallery keys `towers`, `tower1`, `tower2`, `longbridge`, `portal`, `tube`, `voidpads`, `tunnelmouth`; the art agent shoots them first |
| 5 | Sky and lighting | Done (v1) | Unchanged: Clear, Twilight, Dawn; the void past the portal |
| 6 | Course layout | In progress (course agent, branch `tower2-course`) | the portal's lane beams longer, lower and lifting (TC2-1); every level finishes layout 0 (Sloth 133.3, Monkey 139.9, Ape 125.8, King Kong 90.1 s) but the portal hand-over is a lottery: not in order, and the Sloth fails on other layouts |
| 7 | Map rules | In progress | Tube air (terminal 38 m/s, 3× control), pads, walls, resets per leg in `tower/rules.rs`, `stone.rs`; the sling untested until an AI finishes |
| 8 | Art placed on the course | In progress (agent, branch `tower2-art`) | committed 19:33: all v2 models placed from the contract (TA2-1..); shots not taken yet |
| 9 | Lobby spot and flyover | In progress (same agent) | lobby on the lower deck looking at tower 2; the flyover a closed lap over tower 1's top; tests not yet reported |
| 10 | AI validation | In progress | All four finish layout 0, out of order; `tower_difficulties_are_ordered_and_spaced` and `every_level_flies_through_the_tear` fail over 16 layouts (TC2-1: the hand-over needs a rule, not a dice roll) |
| 11 | Medal times | Done (provisional) | Gibbon 95.6 s best; Author 94.5, Gold 101.5, Silver 111.5, Bronze 126.5 (TC2-4); re-measure after TC2-1 |
| 12 | Step and frame cost | Not started | |
| 13 | Colourblind and JOJO MODE | Not started | |
| 14 | Map card | Not started | New card from the new flyover |
| 15 | Screenshots at 1920×1080 | Not started | |
| 16 | Other maps unchanged | Not started | |
| 17 | Docs | Not started | |
| 18 | Full checks | Not started | |
| 19 | Deploy | Not started | Replaces the live tower |

## This map's features

| Feature | Source | Model | Gameplay | Placed in course | Tested |
| --- | --- | --- | --- | --- | --- |
| Tower 1: twice the old keep, hollow, a doorway, climbed inside on free vines | Brief | Done | Not started | Not started | Not started |
| Tower 2: 60% of tower 1, climbed outside | Brief | Done | Not started | Not started | Not started |
| Two straight bridges, vines hung under them, ~10 s each | Brief | Done | Not started | Not started | Not started |
| Start at the foot of tower 1 | Brief | n/a | Not started | Not started | Not started |
| The portal out of tower 1's top, moving like Sector Ape's black hole, simpler | Brief | Done | Not started | Not started | Not started |
| The tube: ~300 m Dropper fall, 20–30 bounce pads to dodge, no vines | Brief | Done | Not started | Not started | Not started |
| Air control in the tube tuned so a human can weave | Plan | n/a | Not started | n/a | Not started |
| The bottom row: near-horizontal vines from the tunnel mouth's top, a sling into the tunnel | Brief | Done (anchor bar) | Not started | Not started | Not started |
| The horizontal tunnel with the finish line; reached without landing | Brief | Done | Not started | Not started | Not started |
| The void look kept: colours, creatures, zip | Brief | Done (v1) | n/a | Not started | Not started |
| The trampoline room removed | Brief | n/a | Not started | Not started | Not started |
| Resets with a boost on every leg and in the tube | v1, kept | n/a | Done (v1) | Not started | Not started |
| Solid stone and the vine audit on the new layout | v1, kept | n/a | Done (v1) | Not started | Not started |
