# Brief: Banana Bay Battle (the pirate map)

Given by the person on 2026-10-15. Their words are kept exactly below; the answers and notes after them are the integrator's. Read `MAPS.md` first.

## The person's description (verbatim)

> Pirate Map: you are swinging on ropes in a battle between pirate ships and ships of some nonspecific government, a la the golden age of pirates. The ships do not move. The water below you needs to have some texture, it would be nice if it moved but it does not necessarily need to be like moving horizontally, pure up and down movement might achieve a good enough effect. The ships do not need to move up and down. the finish line should be an island where you land and there's a chest of golden bananas similar to the sunset ruins map, jumbnled up in the chest, larger than realistic, etc. ropes hang down from the masts and other upright parts of the sails. as for how the monkeys don't hit the sails, I suppose that some ships are missing their sails, or some of their sails and that's where you swing through. side to side constraints of the path can be done with floating barrels with flags in them, I think this was done in some Zelda game. the ships should shoot cannon balls at each other, and some monkey pirates below should be fighting but they can be lower fidelity so as not to cause performance issues. time of day is 3pm. Setting is vaguely carribbean. falling into the water spawns you back on to the previous you ship you were over. items on vines are doubloons. Should try to have at least 3 different kinds of ships and sizes, galleons, frigates, etc. I would guess you swing across 20-30 in the course of the race though you can play with that. Ships should be scaled somewhat realistically, walking from end to end would take maybe 10 seconds. again, use your judgetment on that.
>
> Costumes: apes have some assortment of pirate-related costumes with elements like, cutlasses in the scabbard, eyepatch, bandana, tricorn, pegleg, banodlier, pirate on the shoulder, anything else you think of.

Added after the questions: "Also have bouncable barrels in the water besides the boundary ones."

## The person's answers (2026-10-15)

- **Name: Banana Bay Battle**, map key `bay`.
- **Cannon fire is scenery only.** Ships fire at each other (muzzle smoke, balls in flight, splashes, hits on hulls) and the deck fights are scenery; nothing touches a racer.
- **Boundary barrels are a soft wall.** The flagged barrels mark the course's edges and push a drifting ape back in, like the side limits on other maps; you can't land on them.
- **Bouncy barrels in the water** as well: floating barrels inside the course that you bounce off (bouncers, like the Ruins turtles), a way to save yourself before the water.
- **Length: about 20 ships, about 75 s** for a Monkey AI, like the other maps. Smaller ships fill the gaps between big ones.
- **Start on a dock**: a wooden pier at the edge of the bay, with the battle out ahead.
- **Floor is Lava: a Royal Navy-style jaguar** (the person: "Every map it's a jaguar, no exceptions"): the jaguar in a navy officer's kit (bicorne, blue coat, gold epaulettes), with no real nation's insignia.
- **Deploy: hold until the person says so.** Build on main, show shots on the design journal.
- **"pirate on the shoulder"** is read as a parrot on the shoulder (integrator's reading; SECOND-ORDER).

## Requirements, as a checklist

- [ ] 3 pm Caribbean light: high warm sun, bright blue sky, turquoise water.
- [ ] Gear on this map only (`models::Gear`): an assortment of pirate pieces across the six kinds: tricorn or bandana, eyepatch, cutlass in a scabbard, bandolier, peg leg, parrot on the shoulder, and more (gold earring, sash, striped shirt…). Not every ape wears every piece; each kind gets its own mix so they read apart.
- [ ] About 20 still ships of at least three kinds and sizes (galleon, frigate, sloop/brig…), scaled so walking end to end takes about 10 s for the big ones; pirate ships and a nonspecific navy (plain flags, no real nation).
- [ ] Ropes hang from masts and yards; sails missing or torn where the course swings through, so apes never pass through canvas.
- [ ] Decks you can land on and run along.
- [ ] Water with texture that moves (up and down swell is enough); falling in puts you back on the last ship you were over (Floor is Lava: the kill).
- [ ] Flagged floating barrels on both edges as a soft wall; bouncy barrels floating in the course.
- [ ] Cannon fire between ships (scenery, deterministic from the tick); low-detail monkey pirates fighting on decks.
- [ ] Item vines carry doubloons.
- [ ] Finish on an island: a big chest overflowing with oversized, jumbled golden bananas (like the Ruins pile).
- [ ] The other maps play exactly the same.
- [ ] Lobby spot, flyover, map card, medals, AI ordering test, step and frame cost, colourblind and JOJO MODE, patch notes, README, TRIAGE (closes T87), SECOND-ORDER, design journal page `#bay`.
