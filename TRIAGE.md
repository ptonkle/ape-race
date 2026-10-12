# Triage

Stage 7, written by the person after using the build. `C4 Verify against hand-computed cases`.

One row per concrete change. Say what you did, what you expected, and what happened. Compare at least one result with a case you worked out by hand.

| # | What I did | Expected | What happened | Change wanted | Status |
| --- | --- | --- | --- | --- | --- |
| 1 | Played the first deploy (2026-10-11). | Several vines to choose from; a fall can be recovered. | Vines were far too sparse: about one path forward and one vine you could reach, and that jump was tight, worse after falling to the ground. | Denser vines and easier grabs. | done: stations every 13–19 m (was 24–38), two or three vines across each station in left/middle/right lanes, a low vine every other station (was every third), Reach +0.4 m on every ape, jump 10 m/s (was 9). Test `every_stretch_offers_more_than_one_way_on`. |
| 2 | Played the chimpanzee (middle Air Control). | Steer the flight onto the next vine. | Too hard to reliably reach the next vine; and the hand kept reaching for left click to grab, since Space is jump. | More air control; grab on the left mouse button. | done: Air Control 8–16 m/s² (was 4–9; the chimp 12, was 6.5). Grab is the left button (held), Space jumps from the ground only, items are E or right click. Tests `space_jumps_but_never_grabs`, `a_jump_under_a_low_vine_then_a_grab_takes_it`, `a_grab_press_on_the_ground_takes_a_vine_in_reach`; rule 6: Space made to grab too → `space_jumps_but_never_grabs` failed at tests.rs:322. |

## Future ideas (the person's todos, for a later session)

Not built in the session they were raised in. Pick them up from here.

| # | Idea | Raised | Notes | Status |
| --- | --- | --- | --- | --- |
| T1 | Classic time-trial features: save your time, target times (Bronze, Silver, Gold), and a hidden Author time revealed once Gold is earned, as in Trackmania. | 2026-10-11 | A saved personal best already exists (`Settings.best`, shown in the HUD and results). A first draft of the rest is parked on branch `todo/time-trial` (`crates/core/src/trial.rs`, a Time Trial menu entry, no items, R to retry); its boundary test fails on a banned identifier, and its target times are placeholders to set from measured runs. | open |
