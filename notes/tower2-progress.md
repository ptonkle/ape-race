# Mape's Tower, second design: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Updated with every report. Branch `tower2` (worktree `~/slop/ape-race-tower`); the first design stays live until this one is checked.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done | `briefs/mapes-tower-v2.md`, drawing `briefs/mapes-tower-v2.png` |
| 2 | Skeleton and contract | In progress (agent, started 2026-10-16) | New `TowerLayout`: two towers, two bridges, the doorway, the interior, the tube, the tunnel; the tube's rules |
| 3 | Ape costumes | Done (v1) | Unchanged: wizard and witch mixes |
| 4 | Models | Done (unseen in the browser) | `towermodels/towers.rs`, `tube.rs`, `portal.rs` (TM2 rows); gallery keys `towers`, `tower1`, `tower2`, `longbridge`, `portal`, `tube`, `voidpads`, `tunnelmouth`; the art agent shoots them first |
| 5 | Sky and lighting | Done (v1) | Unchanged: Clear, Twilight, Dawn; the void past the portal |
| 6 | Course layout | Not started | After the contract |
| 7 | Map rules | Not started | The tube: no vines, bounce pads as obstacles, air control, the bottom row's sling |
| 8 | Art placed on the course | Not started | |
| 9 | Lobby spot and flyover | Not started | |
| 10 | AI validation | Not started | The AI must climb a free interior and fall the tube |
| 11 | Medal times | Not started | |
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
