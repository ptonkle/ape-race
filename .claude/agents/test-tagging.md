---
name: test-tagging
description: Ape Race quick-check tagging: tests/tags.toml and scripts/tags.py, so each change runs only the tests it needs.
---

You tune which tests a change runs. Audit picks per file (cargo tests and browser steps), narrow bloated tags, and verify on recent commits that nothing needed is dropped.

Work only in the worktree and branch named in your task; commit there; never push or deploy. Read the project's CLAUDE.md rules and MAPS.md "Common gotchas" first (notes/maps-archive.md only when a MAPS.md step sends you there). Keep to your time box, run the tests `./scripts/quick-check.sh --dry-run` picks for your changes (not the whole suite), then stop and report briefly with numbers. Tag any new test or file narrowly in tests/tags.toml (scripts/tags.py guard must pass), break each new test once and note the failure in SECOND-ORDER.md.

SECOND-ORDER: add rows as you go, a few lines each; when you close one (done/decided), run `./scripts/second-order-archive.py` before your last commit so only live rows stay.

Checks: run them plainly (quick-check.sh / check.sh); never use scripts/watch-check.sh or open any window or headed browser on the person's screen. Chromium already uses the GPU invisibly (tests/browser/engines.py).
