# Tower v2 overnight, 2026-10-17: for the morning

The person said "do these but dont parallelize, I'm going to bed". One agent at a time, in this order. No deploy while they are away; 1.14.0 waits for their word.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | AI planner fix | Done (merged d1b5bf0) | tries all 10 candidates on the tower; King Kong and the Ape finish every layout 0–3; Sloth and Monkey still go out on some (TC2-12..16) |
| 2 | Real course for the first three legs | Done already | see the correction below |
| 3 | Slow AI levels reliable, in order; entry-beam audit; medals | Partly done (merged) | every level finishes all 16 layouts; medals Author 90.0, Gold 97.0, Silver 107.0, Bronze 122.0 from the trial; means King Kong 96.7, Ape 102.0, Monkey 110.5, Sloth 118.3 s; still failing: strict order on layout 0, the Ape 5.3 s ahead of King Kong (6 needed), the Monkey once late to the tear (161 s on layout 10), 32 entry-beam audit rows. Stopped at the two-tries rule; question 1 below |
| 4 | The portal moves | Done already | the model turns and breathes on the race clock every frame (`towermodels/portal.rs`, drawn by `towerscape/art.rs` `portal_frame`) |
| 5 | Model/contract mismatches | In progress | polish agent |
| 6 | Look issues | In progress | polish agent, decided below |
| 7 | Shots, map card, colourblind/JOJO, docs, patch notes | Not started | |
| 8 | Full checks, three engines | Not started | no deploy |

## A correction

Last night I told you the first three legs were a stand-in straight shaft. That was wrong. The course already runs your drawing's route: under the lower bridge, round tower 2's outside, back under the upper bridge, through the doorway, and up tower 1's inside on beams scattered by the dice with no helix. The "straight shaft" in the notes is only the centreline that progress is measured along. What was missing was the AI getting through it reliably.

## Calls I made without you (confirm or overrule)

1. Look issues, my gut: the stipple bands on the tower walls go (plain stone shading); the tube's fog moves further out and lighter so the pads read from the top; the white speed lines are switched off inside the hollow.

## Questions, most important first

1. **The inside climb is a lottery.** Even King Kong takes 20 to 60 s up tower 1's hollow, depending on how the dice scatter the beams. That swamps the gaps between levels, so the test that wants the four levels in order on one layout fails by luck. Your brief asked for random vines with no obvious path. Pick one: (a) keep it random and judge the levels on their average over 16 layouts (I may not loosen a test without you); (b) make the inside more regular, which is less "find your own way"; (c) keep it random but narrow the spread (fewer, better-spaced beams).
