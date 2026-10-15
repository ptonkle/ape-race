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
| 8 | Art placed on the course | In progress | First pass committed; shots being taken |
| 9 | Lobby spot and flyover | In progress | Lobby ledge, smooth flyover (fixes the failing test) |
| 10 | AI validation | In progress | All levels finish; Monkey and Sloth 0.2 s apart |
| 11 | Medal times | In progress | Jungle's stand-ins until measured |
| 12 | Step and frame cost | In progress | |
| 13 | Colourblind and JOJO MODE | In progress | |
| 14 | Map card | Not started | Copy of the Jungle's today |
| 15 | Screenshots at 1920×1080 | In progress | Ape photos retaking; in-game shots coming |
| 16 | Other maps unchanged | Done | `jungle_unchanged`, `maps_unchanged`, `sector_unchanged` pass |
| 17 | Docs | In progress | SECOND-ORDER rows written; README, MAPS.md, HANDOFF to do |
| 18 | Full checks | Not started | `scripts/timed-checks.sh` |
| 19 | Deploy | Waiting on you | Only on your go-ahead |

## This map's features

One row per concept from the brief, plus any added later (your feedback, a new idea, a new mechanic): add a row the moment it comes up. Each is tracked through the same stages: **Model** (drawn), **Rule** (physics or gameplay; "n/a" if only scenery), **On course** (placed in the real layout), **Checked** (seen in an in-game shot or test). Each cell is Done, In progress, Not started or n/a.

| Feature | Source | Model | Rule | On course | Checked |
| --- | --- | --- | --- | --- | --- |
| Sunset sky (yellow → crimson) | Brief | Done | n/a | In progress | Not started |
| Explorer outfit | Brief | Done | n/a | Done | Done |
| Jaguar pith helmet | Your feedback | Done | n/a | Done | Done |
| Dense jungle | Brief | Done | n/a | In progress | Not started |
| Banana-bunch and moai ape ruins | Brief | Done | n/a | In progress | Not started |
| River with rapids and current | Brief | In progress | Done | In progress | Not started |
| Snapping turtles (bouncers) | Brief | Done | Done | In progress | Not started |
| Waterfall and ravine reset | Brief | Done | Done | In progress | Not started |
| Rope bridge, three gorillas wide | Brief | Done | n/a | In progress | Not started |
| Boost vines launch into the mouth | Brief | In progress | Done | In progress | Not started |
| Temple mouth in Aztec relief, with depth | Brief, your feedback | Done | Done | In progress | Not started |
| Waterfall from the mouth | Brief | Done | n/a | In progress | Not started |
| Banana-glyph stonework, pulsing glow | Brief | Done | n/a | In progress | Not started |
| Darker temple lighting | Brief | Done | n/a | In progress | Not started |
| Floor spike traps | Brief | Done | Done | In progress | Not started |
| Trip wires → trapdoors after 0.75 s | Brief | Done | Done | In progress | Not started |
| Glowing banana-spike pits | Brief | Done | Done | In progress | Not started |
| Pit vines drop when doors open | Brief | In progress | Done | In progress | Not started |
| Counter-clockwise round chamber, invisible inner wall | Brief | In progress | Done | In progress | Not started |
| Banana pile finish, packed solid | Brief, your feedback | Done | Done | In progress | Not started |
| Skylight with foliage and soft spotlight | Brief | Done | n/a | In progress | Not started |
