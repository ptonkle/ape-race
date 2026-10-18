---
name: progress-report
description: Report what the background agents are doing and have done, in a consistent format that adapts to the work. Use when the user types /progress-report or asks for a progress report, status, "where are we", "what are the agents doing", or an ETA. For a map build it gives the full map progress report (build steps and features tables); for other work it gives the same shape adapted to that work.
---

# /progress-report: what the agents are doing and have done

Give the person one report, always in the same order and table formats, built from **evidence**, not memory. Never report a running agent's results before its notification arrives: say it's still running and what its last commit shows.

## 1. Gather the facts (in parallel)

- **Agents this session:** every agent launched, with its one-line job, and whether it has reported back. For unfinished ones, look at their branches: `git branch --format='%(refname:short)' | grep worktree` then `git log --format='%ar %s' main..<branch> | head -4` (last commit and how long ago). Note the current time (`date`).
- **Main:** `git log --oneline -10` since the work began; anything merged since the last report.
- **Tests on main:** if anything merged since the last report, run the project's quick tests (in Ape Race: `cargo test --release -q -p tool_core --lib --test maps_unchanged --test sector_unchanged --test jungle_unchanged`) and note failures by name.
- **Tracker file:** if the work has one (a map build has `notes/<key>-progress.md`; other work may have a TRIAGE row, a `notes/<topic>-*.md`, or a plan), read it and **update it first** from the facts above, commit it, then report from it.
- **Notebook upkeep:** if the project keeps a second-order notebook with an archive script (Ape Race: `./scripts/second-order-archive.py`), run it on main and on each live branch so closed rows move out as they close; commit with the tracker. Rows marked waiting on the person go in "Waiting on you".

## 2. Pick the shape

- **A map build** (a new map or a big pass on one): use the project's standard steps (Ape Race: MAPS.md "Build steps", 19 rows) and the map's **features table** (one row per concept, stages Model / Gameplay / Placed in course / Tested; add a row for any new concept that came up since the last report, with its source). Print both tables in full.
- **Anything else** (todos, bug fixes, menus, online, tooling, a /try): make a **work items table** instead, one row per item the person asked for or that came out of the work (their todos by number, e.g. T88; follow-ups found by agents). Columns: `Item | Source | Status | Agent / where | Notes`. If an item has natural stages (e.g. build → test → shots → merged → deployed), add them as columns the way the features table does.
- **Mixed:** one map table set plus one work items table for the rest.

Statuses are always one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Notes are short and concrete (numbers, names, what's left), in plain words, no jargon the person didn't use.

## 3. The report, in this order

1. **Headline:** one or two sentences: overall state and what just changed.
2. **The table(s)** from step 2, in full (not just changes), unless the person asked for changes only.
3. **Running now:** one line per live agent: what it's doing, last commit and how long ago, and when it should land against its time box. Flag any agent past its time box.
4. **Problems:** failing tests (by name), things that look wrong in shots, risks, anything blocked, each with who's handling it.
5. **Waiting on you:** decisions or checks only the person can make. Leave the section out if there are none.
6. **ETA:** to the next milestone (e.g. "ready to play on main", "deployed"), with the remaining steps and rough minutes each.

Keep it scannable: tables and short lines, no restating the whole history. If there are screenshots the person hasn't seen, publish them on their screenshot page first (see the project's memory) and give the link in the headline.
