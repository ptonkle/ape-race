# Brief: The Spindle (a spinning tube, after Iron Vector's Spindle)

Given by Sam Gilson on 2026-10-18. Their words are kept exactly below; the notes after them are the builder's. Read `MAPS.md` first.

## The person's description (verbatim)

> ok this was excellent. now ive got an idea for a simple map that is hopefully fun. look at the game iron vector by sgilson7.github.io, there is a level called spindle this is an extremely long rotating tower, where the gravity is having you land on the inside of the tube due to centrigual force. the vines hang from all points in the tube, but the tube is spinning and the gravity is distributed along the tube like in iron vector / centrifugally. so the entire is one straightaway to the end inside this tube, that I want to look like a mincraft dropper map. when you finsh the the mission, you get blasted out into space into something that looks like the sector ape level, feel free to re-use those assets. the monkeys should be dressed in morph suits that make their entire bodies a single reflective color, kinda like tron. build out the level mvp in the design log for my review like the other map. make sure we are optimizing for another set of PRs against ptonkles repo to merge this in.

## How Iron Vector does it

Iron Vector's Spindle (`sgilson7/iron-vector`, `render.rs` `Warp`, PLAN-M2 row 6): "the simulation runs on the unrolled floor with x wrapping round the circumference, and the renderer bends every box onto the inside of the cylinder, so gravity throws you outward and the floor curves up overhead." This map does the same.

## Requirements, as a checklist

- [ ] One long straightaway inside a spinning tube; the apes land on the inside wall (spin gravity, outward from the axis).
- [ ] Vines hang from all points round the tube.
- [ ] The tube spins; the gravity is spread along the tube as in Iron Vector.
- [ ] It looks like a Minecraft dropper map: blocky, bright block colours, block obstacles.
- [ ] At the finish the apes are blasted out of the tube's end into space that looks like Sector Ape (its assets may be reused).
- [ ] The apes wear morph suits: each whole body one reflective colour, with a Tron look.
- [ ] A playable MVP, shown in the design journal for Sam's review.
- [ ] Built for two pull requests to ptonkle/ape-race (skeleton, then map): shared code by addition only, other maps byte-identical.

## Defaults the build starts on (questions SP-Q1 … in `notes/spindle-progress.md`)

- Name **The Spindle**, key `spindle`, notebook prefix SP.
- Time of day: none (inside a tube in space). The tube's own light: block lamps (glowstone) on the walls and the white glare of the open end ahead; outside, Sector Ape's black space and orange sun.
- Weather: none.
