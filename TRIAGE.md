# Triage

Stage 7, written by the person after using the build. `C4 Verify against hand-computed cases`.

One row per concrete change. Say what you did, what you expected, and what happened. Compare at least one result with a case you worked out by hand.

| # | What I did | Expected | What happened | Change wanted | Status |
| --- | --- | --- | --- | --- | --- |
| 1 | | | | | |

## Future ideas (the person's todos, for a later session)

Not built in the session they were raised in. Pick them up from here.

| # | Idea | Raised | Notes | Status |
| --- | --- | --- | --- | --- |
| T1 | Classic time-trial features: save your time, target times (Bronze, Silver, Gold), and a hidden Author time revealed once Gold is earned, as in Trackmania. | 2026-10-11 | A saved personal best already exists (`Settings.best`, shown in the HUD and results). A first draft of the rest is parked on branch `todo/time-trial` (`crates/core/src/trial.rs`, a Time Trial menu entry, no items, R to retry); its boundary test fails on a banned identifier, and its target times are placeholders to set from measured runs. | open |
