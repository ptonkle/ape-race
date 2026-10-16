# Mape's Tower: overnight, 2026-10-15

**State:** building on branch `tower` (worktree `~/slop/ape-race-tower`), not deployed. The live game (main) is untouched. Steps: `notes/tower-progress.md`.

## If the session stopped (token limit)

Nothing live is affected. To resume in a new session:
1. `git -C ~/slop/ape-race worktree list` and `git branch --list 'worktree-agent-*'`: each unmerged agent branch holds committed work (agents commit WIP often). Merge the ones that pass `cargo test` into `tower`.
2. Read `notes/tower-progress.md` for which step is next, and SECOND-ORDER TW/TG rows for decisions made.
3. The Concrete Jungle tourist outfits (`Gear::Tourist`) are merged into local main, **not pushed** (push = deploy). To ship: bump the version, add a patch note ("New costume: Tourist, for Concrete Jungle"), quick check, push. Shots: `~/slop/ape-race/notes/city-shots/tourist/`.

## Questions for you (most important first)

1. **Is the pace right?** A Monkey AI takes 92 s on average (Sloth 107, Ape 77, King Kong 64.5). The course is 1,348 m: a 674 m climb, then the zipper, its mirror. The climb is the faster half (about 40 s for a Monkey). (TC5)
2. **Medals: Author 0:58, Gold 1:05, Silver 1:15, Bronze 1:30.** The best ape kind's own run is 58.42 s (the Lemur), so the Author time is 0.42 s under it, the same rule as the other maps. Beatable enough? (TM1)
3. **Clear weather is a golden late afternoon**, beside Twilight and Dawn. Keep it, or make Twilight the default? (TS1)
4. **Spirals are gentler than at first.** Tight 38 m circles made every AI stall, so the keep circle is 55 m and the climb goes about 1.5 times round the keep instead of 2+, with three straight branches out to sub-towers. Enough "up and around"? (TC5)
5. **No trampolines in the zipper**: a bounce threw apes back up the way they came. The zipper has platforms and pads to break the fall instead. (TC1)
6. **The mages, dragons and creatures stand still** (drawn at tick 0). Making them move is a polish item. (TA)
7. **The step costs 4 ms at worst** (the Bay 3.3, the Ruins 3.9): fine in a frame; a known speed-up is noted in TM2.
