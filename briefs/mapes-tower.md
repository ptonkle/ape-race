# Brief: Mape's Tower (the wizard tower map)

Given by the person on 2026-10-15. Their words are kept exactly below; the answers and notes after them are the integrator's. Read `MAPS.md` first.

## The person's description (verbatim)

> Map Description: Mape's Tower. The entire point of the level is you are swinging up and around a rangers apprentice type giant tower up into the sky. When you swing on the vines that are intended to bring you upwards, it should give a slight upwards speed bonus to accomodate making going up the tower easier. There should be offshoot branches of the tower with little sub-towers in order to add more straight away, not upwards movement  to the map, and not all upward spiral movement which could be dizzying. there should be a school of mages within the towers, on the balconies, flying on dragons outside the tower  that are firing magic spells at you, and those are the vines you are swinging on. they are casting different magic spells which control the orientation of the vine. there should be trampolines you can bounce from in order to get you to go up higher faster. at one point you should go inside a room in a tower with a bunch of trampoline surfaces and one mage in the middle with an orb of vines around him, and you need to exit through the roof. emphasis should be placed on making this map easy and not too frustrating for a human, with easy respawn points that always have some form of speed boost that gives you the correct upwards and diagonal momentum. as you go up the tower, you should go through the clouds, which also provide hook points for vines. emphasis should be placed on diverging paths such that visibility is maintained, as if there are too many vines above and in front of  you, your view of the map as you play will just be vines,  thus the vines should be relatively sparse while maintaining playability, fun, and ease. at the top of the  tower, there should be an insane wizard ripping a  tear in reality with a giant eldritch evil banana on the other side, which you fly though and then fall through a zipper like space like squidward in the spongebob episode shangai'd. at  the bottom of the zipper is infinite bananas  and then the map is over and you  get to see the monkeys dancing in the banana zone. the layout of the zipper space should be the  reflection of the mage tower, so both halves of the map are the  same  length, but in the  zipper space you are going down, with the layout being  a  little topsy turvy  and so you cannot just go from  the top to  the bottom in one jump. in the zipper, instead of mages, its weird blob type and davy jones sea monsters type creatures that emit the vines to swing from.  the mages  and  monsters in the map should be casting various kinds of magic that may explain the various platforms you land on. the bounding box for resetting you if you fall off  the tower or  get too far away  from it should be a similar implementation to sector ape, where you are snapped back to  the closest flat surface that has been optimized for easy return into the course at a  good speed.
> also the ape costumes should be  different wizard and witch  outfits, with scepters and wands and pipes potentially,  think gandalf

## The person's answers (2026-10-15)

- **Name: Mape's Tower**, exactly so; map key `tower`.
- **Light: twilight and dawn, as two weathers** (the map's weather options, like the Ski Slope's and the bay's; `snow::Weather::offered`). Default (Clear) is one of them or a neutral sky; the integrator picks and records it.
- **Length: aim for about 90 s** for a Monkey AI (longer than the other maps' 60–80 s), half climbing the tower, half falling down the zipper.
- **Floor is Lava big cat: make it look like the Balrog somehow**: a huge cat-shaped demon of shadow and flame (horns, smoke wings, embers, a fire whip/tail), inspired by, not copying, any film design.
- **The zipper space: the tower's layout mirrored and turned topsy-turvy**, eerie sea-green and violet, a giant zip running down the void, blob and Davy-Jones-style sea monsters emitting the vines.
- **Spells are scenery only**: mages, dragons and monsters fire spells; the vines are spell beams; nothing knocks or slows an ape.
- **Deploy: hold until the person says so.** Build on main, shots on the design journal as it goes.
- **Climbing vines' upward boost: tuned until climbing feels as easy as a flat map** (measure a Monkey's climb).
- **Overnight (2026-10-15, 23:00):** "yep run it over night". Make every call, record it in SECOND-ORDER, questions go in `notes/tower-morning.md`; still hold the deploy.

## Requirements, as a checklist

- [ ] A giant tower (Ranger's Apprentice feel: tall stone keep, battlements, balconies) climbed up and around, with offshoot branches to little sub-towers so part of the climb runs straight across, not all spiral (avoid dizziness).
- [ ] Climb vines give a slight upward speed bonus (map-only, tuned).
- [ ] Mages on balconies and in the towers, and mages riding dragons outside; their spell beams are the vines, and the spell type sets the vine's orientation (Sector Ape's any-direction vines).
- [ ] Trampolines (bouncers) to go up faster.
- [ ] A room inside a tower: trampoline surfaces, one mage in the middle with an orb of vines around him; exit through the roof.
- [ ] Easy for a human: forgiving resets that always give a speed boost in the right up/diagonal direction.
- [ ] Climb through the clouds; clouds also hold vine anchors.
- [ ] Sparse vines, diverging paths, so the view is never just vines.
- [ ] At the top, an insane wizard ripping a tear in reality, a giant eldritch evil banana beyond it; you fly through.
- [ ] The zipper space: the tower half mirrored and topsy-turvy, going down, same length as the climb; you cannot fall top to bottom in one go; blob and sea-monster creatures emit the vines.
- [ ] Platforms explained by the magic (conjured platforms, frozen spells…).
- [ ] At the bottom, infinite bananas, the finish, and the apes dancing in the banana zone.
- [ ] Resets like Sector Ape's: falling off or straying too far snaps you to the nearest flat surface tuned for a good return at speed.
- [ ] Gear on this map: wizard and witch outfits (Gandalf-like hats, robes, staffs/sceptres, wands, pipes), each kind its own mix.
- [ ] Weathers: Twilight and Dawn.
- [ ] Floor is Lava: the Balrog-like flame cat.
- [ ] The other maps play exactly the same.
- [ ] Lobby spot, flyover, map card, medals, AI ordering test, step and frame cost, colourblind and JOJO MODE, patch notes, README, TRIAGE, SECOND-ORDER, design journal page `#tower`.

## Risks noted at the start

- **Vertical racing is new.** Every map so far runs forward along a mostly level path. The course's progress, the chase camera, the AI's release choice and the reset all assume forward travel; climbing and then falling needs the skeleton to settle how "along the course" works on a 3D path first.
- **The camera on a climb and a fall** (looking up a tower wall, looking down a void) is the likeliest thing to feel bad; the global see-through hole (see.rs) helps with the tower wall.
