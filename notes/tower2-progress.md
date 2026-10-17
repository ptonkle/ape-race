# Mape's Tower, second design: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Updated with every report. Branch `tower2` (worktree `~/slop/ape-race-tower`); the first design stays live until this one is checked.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done | `briefs/mapes-tower-v2.md`, drawing `briefs/mapes-tower-v2.png` |
| 2 | Skeleton and contract | Done (contract); stand-in course In progress | `tower.rs` contract merged (TW2-1..8); the tube air measured (TW2-5); merged `tower2-ai`: a straight shaft up tower 1's axis; Monkey, Ape and King Kong finish the stand-in (135.7, 130.6, 90.0 s), the Sloth runs out of time in the tube (TW2-9..13) |
| 3 | Ape costumes | Done (v1) | Unchanged: wizard and witch mixes |
| 4 | Models | Done (unseen in the browser) | `towermodels/towers.rs`, `tube.rs`, `portal.rs` (TM2 rows); gallery keys `towers`, `tower1`, `tower2`, `longbridge`, `portal`, `tube`, `voidpads`, `tunnelmouth`; the art agent shoots them first |
| 5 | Sky and lighting | Done (v1) | Unchanged: Clear, Twilight, Dawn; the void past the portal |
| 6 | Course layout | In progress | merged `tower2-course`: portal lane beams longer, lower and lifting; the rest still the stand-in (a straight shaft); the hand-over agent measured the hand-over at 4 s to never over layouts 0–3 and the inside climb at 30–88 s, and found the cause is the AI planner, not the beams (TC2-7); three layouts tried and reverted, kept as `notes/tower2-handover-attempt.patch` (TC2-11) |
| 7 | Map rules | In progress | Tube air (terminal 38 m/s, 3× control), pads, walls, resets per leg in `tower/rules.rs`, `stone.rs`; the sling untested until an AI finishes |
| 8 | Art placed on the course | Done (on the stand-in) | merged `tower2-art`: every v2 model placed from the contract (TA2-1); the inside's sharing/caps can go once the course thins the rings (TA2-3); stipple bands on the walls and the void fog for the person to judge (TA2-8) |
| 9 | Lobby spot and flyover | Done | lobby mid lower bridge looking to tower 2 (both towers can't fit one view: 380 m apart, fog at 330 m); flyover a closed stadium lap, no jump (TA2-5, TA2-6) |
| 10 | AI validation | In progress | planner agent (`tower2-planner`): on the tower the planner steers for all 10 candidates, and a beam 5 m higher than the one held counts as ahead though the course places it behind (TC2-12, TC2-13). King Kong and the Ape now finish layouts 0–3 (King Kong 85–100 s, hand-over 3–14.5 s, inside 24.5–33 s); the Monkey is out of time on layout 1 and the Sloth on 1 and 2; the Sloth's and the Monkey's inside climb is 39–72 s and on layout 0 the Sloth (123.4 s) beats the Monkey (137.0 s). Two candidate orders for the inside climb were worse and reverted (TC2-14); next is layout (b) with TC2-8 (TC2-15) |
| 11 | Medal times | Done (provisional) | Author 94.5, Gold 101.5, Silver 111.5, Bronze 126.5 (v1 61/68/78/93); re-measure after the hand-over (TC2-4) |
| 12 | Step and frame cost | In progress | worst frame 5,355 shapes (v1 4,896; budget 5,500), driven by the stand-in's inside density (TA2-2); the budget test passes after the merges; step cost not measured |
| 13 | Colourblind and JOJO MODE | Not started | |
| 14 | Map card | Not started | New card from the new flyover |
| 15 | Screenshots at 1920×1080 | Done (first pass) | `notes/tower2-shots/` 21 shots: gallery, play, lobby, flyover; on the journal; retake after the course lands |
| 16 | Other maps unchanged | Done so far | fingerprint and replay-hash tests pass unchanged after every merge |
| 17 | Docs | Not started | |
| 18 | Full checks | In progress | debug workspace tests on `tower2-planner`: fmt and clippy clean; the other maps unchanged; failing: the vine audit (120 rows: 32 entry-beam, 88 caster lines, TC2-10), the three tower AI tests (the Sloth and Monkey out of time on some layouts, order wrong on layout 0) and now the medal test (best Time Trial 87.7 s under Author 94.5; re-measure once every layout finishes), TC2-16 |
| 19 | Deploy | Not started | Replaces the live tower |

## This map's features

| Feature | Source | Model | Gameplay | Placed in course | Tested |
| --- | --- | --- | --- | --- | --- |
| Tower 1: twice the old keep, hollow, a doorway, climbed inside on free vines | Brief | Done | Done (stone, the doorway's hole, resets) | Stand-in only (a straight shaft) | Through the AI runs; the climb is a 30–88 s lottery |
| Tower 2: 60% of tower 1, climbed outside | Brief | Done | Done (stone) | Not started | Not started |
| Two straight bridges, vines hung under them, ~10 s each | Brief | Done | Done (stone, resets) | Not started | Not started |
| Start at the foot of tower 1 | Brief | n/a | Done | Done | Done |
| The portal out of tower 1's top, moving like Sector Ape's black hole, simpler | Brief | Done | Beams and the fall pad Done; the movement Not started | Done | Failing: the hand-over (TC2-7) |
| The tube: ~300 m Dropper fall, 20–30 bounce pads to dodge, no vines | Brief | Done | Done (air, terminal speed, pad hits) | Done | Done |
| Air control in the tube tuned so a human can weave | Plan | n/a | Done (measured, TW2-5) | n/a | Not played by the person yet |
| The bottom row: near-horizontal vines from the tunnel mouth's top, a sling into the tunnel | Brief | Done (anchor bar) | Done (hangers, resets) | Done | Done: four levels finish layout 0 |
| The horizontal tunnel with the finish line; reached without landing | Brief | Done | Done | Done | Done |
| The void look kept: colours, creatures, zip | Brief | Done (v1) | n/a | Done | Done |
| The trampoline room removed | Brief | n/a | Done | Done | Done |
| Resets with a boost on every leg and in the tube | v1, kept | n/a | Done | Done | Done |
| Solid stone and the vine audit on the new layout | v1, kept | n/a | Done | Done | Failing: 32 entry-beam swings, 88 caster lines (TC2-10) |
