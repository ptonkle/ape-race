# Mape's Tower, second design: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Updated with every report. Branch `tower2` (worktree `~/slop/ape-race-tower`); the first design stays live until this one is checked.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done | `briefs/mapes-tower-v2.md`, drawing `briefs/mapes-tower-v2.png` |
| 2 | Skeleton and contract | Done (contract); stand-in course In progress | `tower.rs` contract merged (TW2-1..8); the tube air measured (TW2-5); merged `tower2-ai`: a straight shaft up tower 1's axis; Monkey, Ape and King Kong finish the stand-in (135.7, 130.6, 90.0 s), the Sloth runs out of time in the tube (TW2-9..13) |
| 3 | Ape costumes | Done (v1) | Unchanged: wizard and witch mixes |
| 4 | Models | Done (unseen in the browser) | `towermodels/towers.rs`, `tube.rs`, `portal.rs` (TM2 rows); gallery keys `towers`, `tower1`, `tower2`, `longbridge`, `portal`, `tube`, `voidpads`, `tunnelmouth`; the art agent shoots them first |
| 5 | Sky and lighting | Done (v1) | Unchanged: Clear, Twilight, Dawn; the void past the portal |
| 6 | Course layout | In progress | merged `tower2-course`: portal lane beams longer, lower and lifting; the rest still the stand-in; the hand-over from the inside rings to the portal beams is a lottery, 4–34 s (TC2-1); the two entry beams still swing into the wall (TC2-3) |
| 7 | Map rules | In progress | Tube air (terminal 38 m/s, 3× control), pads, walls, resets per leg in `tower/rules.rs`, `stone.rs`; the sling untested until an AI finishes |
| 8 | Art placed on the course | Done (on the stand-in) | merged `tower2-art`: every v2 model placed from the contract (TA2-1); the inside's sharing/caps can go once the course thins the rings (TA2-3); stipple bands on the walls and the void fog for the person to judge (TA2-8) |
| 9 | Lobby spot and flyover | Done | lobby mid lower bridge looking to tower 2 (both towers can't fit one view: 380 m apart, fog at 330 m); flyover a closed stadium lap, no jump (TA2-5, TA2-6) |
| 10 | AI validation | In progress (hand-over agent) | layout 0: Sloth 133, Monkey 140, Ape 126, King Kong 90 s: all finish, not in order; over 16 layouts the Sloth fails one and the tear test fails for most levels (TC2-1) |
| 11 | Medal times | Done (provisional) | Author 94.5, Gold 101.5, Silver 111.5, Bronze 126.5 (v1 61/68/78/93); re-measure after the hand-over (TC2-4) |
| 12 | Step and frame cost | In progress | worst frame 5,355 shapes (v1 4,896; budget 5,500), driven by the stand-in's inside density (TA2-2); step cost not measured |
| 13 | Colourblind and JOJO MODE | Not started | |
| 14 | Map card | Not started | New card from the new flyover |
| 15 | Screenshots at 1920×1080 | Done (first pass) | `notes/tower2-shots/` 21 shots: gallery, play, lobby, flyover; on the journal; retake after the course lands |
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
