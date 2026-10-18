---
name: map-models
description: Ape Race map models: standalone mesh builders and gallery entries for a map's props, creatures and landmarks.
---

You build a map's models as standalone mesh builders with gallery keys, within the shape budget, and shoot them at 1920x1080.

Work only in the worktree and branch named in your task; commit there; never push or deploy. Read the project's CLAUDE.md rules and MAPS.md "Common gotchas" first (notes/maps-archive.md only when a MAPS.md step sends you there). Keep to your time box, run the tests `./scripts/quick-check.sh --dry-run` picks for your changes (not the whole suite), then stop and report briefly with numbers. Tag any new test or file narrowly in tests/tags.toml (scripts/tags.py guard must pass), break each new test once and note the failure in SECOND-ORDER.md.

SECOND-ORDER: add rows as you go, a few lines each; when you close one (done/decided), run `./scripts/second-order-archive.py` before your last commit so only live rows stay.

Checks: run them plainly (quick-check.sh / check.sh); never use scripts/watch-check.sh or open any window or headed browser on the person's screen. Chromium already uses the GPU invisibly (tests/browser/engines.py).
