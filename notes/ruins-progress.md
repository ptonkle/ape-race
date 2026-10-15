# Sunset Ruins: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Updated with every report.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done | `briefs/sunset-ruins.md` |
| 2 | Skeleton and contract | Done | `Map::Ruins`, `ruins.rs`, shared trapdoor state, PROTOCOL 10 |
| 3 | Ape costumes | Done | Explorer outfit on all six apes, pith helmet on the jaguar |
| 4 | Models | Done | Two passes; pile packed solid, temple face in relief |
| 5 | Sky and lighting | Done | Sunset dome, haze ring; inside and skylit looks |
| 6 | Course layout | Done | 1.75 km: river, bridge, corridor, 300° chamber |
| 7 | Map rules | Done | Current, ravine reset, bridge boost, spikes, trapdoors |
| 8 | Art placed on the course | Done | Placed and polished (1.10.1): lit entrance, cliffs, random pile, glyphs in step; panel glyph sockets and one jog panel still off |
| 9 | Lobby spot and flyover | In progress | Lobby fixed in 1.11.2; the online lobby's framing still hides the mouth |
| 10 | AI validation | Done | Ordered over 32 layouts: Sloth 88.1, Monkey 78.2, Ape 71.9, King Kong 64.7 s; river, bridge and turn-in stalls fixed |
| 11 | Medal times | Done | Author 0:56, Gold 1:03, Silver 1:13, Bronze 1:28 (re-measured) |
| 12 | Step and frame cost | Done | Worst step 2.75 ms over 2,000 frames (was 7.1); race results unchanged by the speed-ups |
| 13 | Colourblind and JOJO MODE | Done | Colourblind and JOJO MODE both recolour the whole map (1.10.1) |
| 14 | Map card | Done | Its own card from the flyover: bridge, mouth, sunset (live in 1.10.1) |
| 15 | Screenshots at 1920×1080 | In progress | Apes, contrast and 14 in-game shots on the journal; finish shot and v2 coming |
| 16 | Other maps unchanged | Done | All tests pass on main, release and debug (a debug-only overflow found and fixed) |
| 17 | Docs | In progress | SECOND-ORDER rows written; README, MAPS.md, HANDOFF to do |
| 18 | Full checks | Done | 1.10.1 passed in Chromium, Firefox and WebKit: check 8.3 min, online 3.8 min; rerun before each deploy where time allows |
| 19 | Deploy | Done | 1.10.0 23:20 … 1.10.4 00:50, 1.11.0 (patch notes), 1.11.1 (solid face); fixes follow |

## This map's features

One row per concept from the brief, plus any added later (your feedback, a new idea, a new mechanic): add a row the moment it comes up. Each is tracked through the same stages: **Model** (drawn), **Gameplay** (how it plays: physics, rules; "n/a" if only scenery), **Placed in course** (placed in the real layout), **Tested** (seen working in an in-game shot or proved by a test). Each cell is Done, In progress, Not started or n/a.

| Feature | Source | Model | Gameplay | Placed in course | Tested |
| --- | --- | --- | --- | --- | --- |
| Sunset sky (yellow → crimson) | Brief | Done | n/a | Done | Done |
| Explorer outfit | Brief | Done | n/a | Done | Done |
| Outfit stands apart from fur in the sunset | 1080p shots | Done | n/a | Done | Done |
| Jaguar pith helmet | Your feedback | Done | n/a | Done | Done |
| Dense jungle | Brief | Done | n/a | Done | Done |
| Banana-bunch and moai ape ruins | Brief | Done | n/a | Done | Done |
| River with rapids and current | Brief | Done | Done | Done | Done |
| The current carries you downstream only, never sideways | Your feedback (1.10.0) | n/a | Done | Done | Done |
| Snapping turtles (bouncers) | Brief | Done | Done | Done | In progress |
| Turtles stay in the water, clear of the statues | Your feedback (1.10.0) | Done | n/a | Done | Done |
| Waterfall and ravine reset | Brief | Done | Done | Done | Done |
| Rope bridge, three gorillas wide | Brief | Done | n/a | Done | Done |
| Boost vines launch into the mouth | Brief | Done | Done | Done | Done |
| Weaker AIs cross the bridge and mouth without stalling | AI tuning | n/a | Done | Done | Done |
| Temple mouth in Aztec relief, with depth | Brief, your feedback | Done | Done | Done | Done |
| Temple face is solid; only the mouth lets you in | Your feedback (1.10.2) | n/a | Done | Done | Done |
| No black box in the mouth | Your feedback (1.10.2) | Done | n/a | Done | Done |
| Waterfall from the mouth | Brief | Done | n/a | Done | Done |
| Banana-glyph stonework, pulsing glow | Brief | Done | n/a | Done | Done |
| Darker temple lighting | Brief | Done | n/a | Done | Done |
| You can see into the temple from the bridge and through the mouth | Your feedback (1.10.0, again on 1.10.1) | Done | n/a | Done | Done |
| Wall panels sit flush along every wall, past the bends | Your feedback (1.10.0, again on 1.10.2) | Done | n/a | Done | Done |
| Banana glyphs on the wall panels can be seen and glow | Your feedback (1.10.2) | Done | n/a | Done | Done |
| Floor spike traps | Brief | Done | Done | Done | Done |
| Trip wires → trapdoors after 0.75 s | Brief | Done | Done | Done | Done |
| Glowing banana-spike pits | Brief | Done | Done | Done | In progress |
| Out of a spike pit within a few seconds, never stuck | Your feedback (1.10.0) | n/a | Done | Done | Done |
| Pit vines drop when doors open | Brief | Done | Done | Done | In progress |
| Pit vines stay visible while they can be grabbed | Your feedback (1.10.0) | Done | Done | Done | Done |
| Counter-clockwise round chamber, invisible inner wall | Brief | Done | Done | Done | Done |
| Banana pile finish, packed solid | Brief, your feedback | Done | Done | Done | In progress |
| Skylight with foliage and soft spotlight | Brief | Done | n/a | Done | In progress |
| Sun low and to the side, raking across the temple face | Models review, your choice | n/a | n/a | Done | Done |
| Ravine sides read as rocky cliffs | In-game shots | Done | n/a | Done | Done |
| Banana pile heaped at random, not in rings | Your feedback | Done | n/a | Done | Done |
| Lobby camera shows your ape with the temple mouth behind | Your feedback (1.10.1) | Done | n/a | Done | Done |
| Lobby ape stands on the surface, not sunk in | Your feedback (1.10.2) | Done | n/a | Done | Done |
| The Sloth finishes every layout | AI report (1.10.3) | n/a | In progress | n/a | Not started |
| AI levels stay 6 s apart with the solid face | Ordering test (1.11.1) | n/a | In progress | n/a | Not started |
| The camera stays out of the floor and walls just inside the mouth | Mouth shots (1.10.4) | n/a | In progress | In progress | Not started |
