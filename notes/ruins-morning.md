# Sunset Ruins: overnight, 2026-10-14/15

The person went to bed at about 00:10 after playing 1.10.0–1.10.2. Each fix below is deployed as it lands, as the next 1.10.x (the person: "you can continue to deploy and run tests at your leisure for the rest of this session"). The full table is `notes/ruins-progress.md`.

## Live when they left

- 1.10.0 (23:20): Sunset Ruins, maps renamed (Canopy Run, Concrete Jungle, Yeti Peak).
- 1.10.1 (23:45): lit temple entrance attempt, random banana pile, cliffs, JOJO MODE, its own map card.
- 1.10.2 (23:58): the river current only carries you downstream; turtles and statues apart.
- 1.10.4 (00:50, overnight): the black block in the temple mouth removed (a solid near-black box from the first model filled the opening; the 1.10.1 glow was drawn on it). From the bridge you now see the lit corridor.
- 1.10.3 (00:35, overnight): spike pits throw you out forward (out in about 1.6 s, was up to 8 s); a trapdoor never shuts on an ape in its pit, so the pit vines stay down and visible (they used to vanish the instant the door closed).

## Checks

- Full checks on 1.10.1 passed in all three browsers (check.sh 8.3 min, check-online.sh 3.8 min; `notes/check-times.md`).

## Being fixed overnight (their feedback)

| Item | Status | Version |
| --- | --- | --- |
| Black box in the temple mouth (a solid dark block from the model, `ruinsmodels.rs` "the throat") | Done | 1.10.4 |
| Just inside the mouth the chase camera ends up in a shape (half the view a brown plane); an olive slab on the floor | In progress (found overnight) | |
| The Sloth never finishes one of the 16 test layouts | In progress (found overnight) | |
| Lobby camera looks at the ground; the ape sunk into the ground | In progress | |
| Wall panels not flush in some sections; their banana glyphs can't be seen | In progress | |
| Stuck in spike pits; pit vines disappear | Done | 1.10.3 |
| You can fly straight through the temple face (no collision outside the mouth) | In progress | |

## Also overnight

| Item | Status | Version |
| --- | --- | --- |
| Patch notes: a "What's new" panel in the game (the version label opens it too, a "New" marker after an update) and a patch-notes page on the site, from one file | In progress | |

## Found overnight

- On one of the 16 test layouts the Sloth never finishes Sunset Ruins (the AI report shows its mean as infinite). It was there before the spike-pit fix. An agent is on it.
- Just past the mouth the chase camera ends up inside a shape (half the screen a flat brown plane), seen in the mouth agent's shots. An agent is on it.

## Questions for the morning

(none yet)
