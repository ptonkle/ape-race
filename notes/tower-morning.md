# Mape's Tower: overnight, 2026-10-15

**State (06:00-ish):** Mape's Tower is **built and checked** on branch `tower` (worktree `~/slop/ape-race-tower`), steps 1–18 done (`notes/tower-progress.md`), **not deployed**. All tests pass; check.sh in all three browsers; the online check passes in all three after a lobby fix.

**To deploy when you say so:** merge `tower` into main, bump to 1.13.0 with patch notes (e.g. "New map: Mape's Tower." / "New costumes: Wizard and Tourist."), quick check, push main.

**Also waiting on main, unpushed:** the Concrete Jungle **Tourist** costume, and a lobby fix (a long name with Proboscis Monkey scrolled the online lobby sideways by 9 px in Firefox and Safari; I'd only checked Chrome). They go out with the tower, or alone if you'd rather.

## If the session stopped (token limit)

Nothing live is affected. To resume in a new session:
1. `git -C ~/slop/ape-race worktree list` and `git branch --list 'worktree-agent-*'`: each unmerged agent branch holds committed work (agents commit WIP often). Merge the ones that pass `cargo test` into `tower`.
2. Read `notes/tower-progress.md` for which step is next, and SECOND-ORDER TW/TG rows for decisions made.
3. The Concrete Jungle tourist outfits (`Gear::Tourist`) are merged into local main, **not pushed** (push = deploy). To ship: bump the version, add a patch note ("New costume: Tourist, for Concrete Jungle"), quick check, push. Shots: `~/slop/ape-race/notes/city-shots/tourist/`.

## Questions for you (most important first)

1. **Is the pace right?** A Monkey AI takes 97 s on average (Sloth 111, Ape 82, King Kong 67; with the item crates and solid stone, TC7). The course is 1,348 m: a 674 m climb, then the zipper, its mirror. The climb is the faster half (about 40 s for a Monkey). (TC5)
2. **Medals: Author 1:01, Gold 1:08, Silver 1:18, Bronze 1:33** (re-measured with the item crates and the solid stone; were 0:58 / 1:05 / 1:15 / 1:30). The best ape kind's own run is 61.88 s (the Lemur), so the Author time is 0.88 s under it, the same rule as the other maps. Beatable enough? (TM1)
3. **Clear weather is a golden late afternoon**, beside Twilight and Dawn. Keep it, or make Twilight the default? (TS1)
4. **Spirals are gentler than at first.** Tight 38 m circles made every AI stall, so the keep circle is 55 m and the climb goes about 1.5 times round the keep instead of 2+, with three straight branches out to sub-towers. Enough "up and around"? (TC5)
5. **No trampolines in the zipper**: a bounce threw apes back up the way they came. The zipper has platforms and pads to break the fall instead. (TC1)
6. **The mages, dragons and creatures stand still** (drawn at tick 0). Making them move is a polish item. (TA)
7. **The step costs 4 ms at worst** (the Bay 3.3, the Ruins 3.9): fine in a frame; a known speed-up is noted in TM2.
8. **The lobby view is plain** (TL8): the ape stands on the start deck 16 m up, keep on the left, but a level camera there sees no clouds, dragons or turrets, and the grey deck fills the lower third. A high bridge would look better, but its drawn stonework sits above the walkable deck, so the camera ended up inside stone (TL1). Fix the bridge and move the lobby up, or keep it?
9. **On every map, the map-card strip covers the ape in the 1280×720 online lobby** (the lobby agent noticed). That's on the live game too; a quick fix on main once you've seen it.

