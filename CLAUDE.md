# Instructions for a coding agent in a Builder project

`AGENTS.md` holds the same rules for agents that read that file instead.

1. Read `BRIEF.md` and `BUILDER-SPEC.md` before changing code.
2. The core (`crates/core`) owns every application decision and performs no input or output.
3. The shim (`crates/shim`) translates values and decides no application behaviour.
4. The page (`web/`) draws what the core returns and does not reproduce a core rule.
5. Run `cargo test` after each meaningful core change.
6. For each new behaviour test, break the behaviour it covers once, watch the test fail, then restore it. Note the failure message in `SECOND-ORDER.md`.
7. Keep `SECOND-ORDER.md` while you work: one row per assumption, surprise, divergence from the plan, or item the person must do.
8. Do not deploy. Deploying is the person's decision (stage 8).
9. Before handoff, run `./scripts/check.sh` and record the result in `HANDOFF.md`.

Write `PLAN.md` from the brief and stop until the person has read it, unless the brief says the plan is approved in advance. When a rule in `BUILDER-SPEC.md` seems to block the work, stop and say so in a worklist row; do not loosen a check to make it pass.
