# Brief: Sunset Ruins (the jungle temple map)

Given by the person on 2026-10-14. Their words are kept exactly below; the answers and notes after them are the integrator's. Read `MAPS.md` first.

## The person's description (verbatim)

> Time of Day: This should be vibrant sunset with hues fading from yellow to orange to red and crimson.
>
> Outfits: the monkeys should all get pith helmets, and khaki explorer jackets with a compass on their chest and a canteen hanging from their hip. They should also have on tiny round spectacles.
>
> The setting of the map is a dense jungle with scattered mossy stone ruins in the shape of banana bunches and ape heads in the style of easter island Maoi. The starting path of the map will be a downhill river with rapids and turbulent water graphics. There are snapping turtles that swim horizontally across the river placed sporadically that can be bounced off of. One third of the way through the map the river will become a waterfall landmark that descends down into a ravine that is not playable space and serves as a kill box if the player falls too deeply into it. There will be a rope bridge that is as a narrow as three gorilla models wide. This will run above where the river transitions into the waterfall feature. The rope bridge will have boosted vines dangling from them, and the players will launch over the edge of the waterfall to cross into the opening of the monkey temple. The players will fly into the gaping mouth of the monkey temple styled after Aztec stone relief artwork from the rope bridge launch. The monkey temple also has a waterfall pouring from the open mouth into the ravine. The map then turns into a stone temple inner working with Aztec stonework patterning with bananas incorporated into the design as the main motif. There will be darker lighting inside the temple, and the banana symbols in the stonework should pulse slowly in and out with a warm yellow glow to provide lighting to the temple interior. There should be sporadic spike traps on the floor of the temple, and there should also be trip wires on the ground that activate drop away floors revealing dangerous glowing banana spikes in a pit following the trapdoor. There should be a 0.75 second delay between triggering and the opening of trapdoors. When the trapdoors open, new vines should also drop from the ceiling that reach into the pit in order to provide players with a way to continue racing. This temple interior with the spike traps should comprise another third of the map. Finally the map should reveal the interior of the temple. The playable pathway will take a right turn and begin to make a counter-clockwise circle with the temple wall on the right, and an invisible wall on the left preventing players from going into the center of the map. This will continue around the interior of the temple until the circle is almost complete, at which point the path will turn towards the epicenter of the temple and players will swing towards a massive pile of glowing banana statues for the finish line. The temple interior should be enormous, and have a skylight above the pile of glowing yellow banana statues that has decorative foliage and jungle vines hanging down into it. It should cast a soft spotlight on the pile of bananas in the temple epicenter.

## The person's answers (2026-10-14)

- **Name: Sunset Ruins**, map key `ruins`.
- **Ravine fall:** reset at once onto the riverbank just before the rope bridge (behind you), no extra penalty; the lost time is the penalty. With Floor is Lava it is the kill.
- **Spikes** (floor spike traps and the pit's glowing banana spikes): fling you upward and you can't grab for about 1 s. You keep racing.
- **River water:** the current carries a swimming ape downstream at a modest speed (slower than swinging, never stuck); turtles and low vines get you back up.
- **Bridge launch:** assisted, with a wide mouth. The boosted vines fling you hard toward the temple mouth, so any decent release gets in; a bad release still drops you into the ravine.
- **Trapdoors:** shared. Whoever trips a wire opens it for everyone (deterministic from race state, synced online, kept in replays, like Sector Ape's door) 0.75 s later, and the pit vines drop. It closes and re-arms about 8 s later.
- **Process:** build now on main; **do not deploy** until the person says so.

## Requirements, as a checklist

- [ ] Vibrant sunset outside (yellow → orange → red → crimson sky); darker inside the temple, lit by pulsing banana glyphs; a soft skylight spotlight on the finish pile.
- [ ] Gear on this map only (`models::Gear`): pith helmet, khaki explorer jacket with a chest compass, a canteen on the hip, tiny round spectacles, on every ape kind.
- [ ] Dense jungle with mossy stone ruins: banana-bunch statues and moai-style ape heads.
- [ ] Third 1: a downhill river with rapids and turbulent water; snapping turtles swimming across it as moving bouncers (deterministic from the tick).
- [ ] The river becomes a waterfall into a ravine (not playable; falling deep resets you). A rope bridge three gorillas wide spans above the falls, with boosted vines that launch apes across into the temple's gaping Aztec-relief mouth, which pours its own waterfall into the ravine.
- [ ] Third 2: the temple's inner workings, Aztec stonework with a banana motif; floor spike traps; trip wires → trapdoors 0.75 s later over pits of glowing banana spikes, with vines dropping from the ceiling into the pit.
- [ ] Third 3: an enormous inner chamber. The path turns right and runs counter-clockwise round it, the temple wall on the right and an invisible wall on the left; near the full circle it turns to the centre and you swing to the finish at a huge pile of glowing banana statues under a skylight hung with foliage and vines.
- [ ] The other four maps play exactly the same (`jungle_unchanged`, `maps_unchanged`, `sector_unchanged`).
- [ ] The lobby zone designed on purpose, the map card, medals, an AI ordering test, step and frame cost, colourblind and JOJO MODE checks, README, TRIAGE, SECOND-ORDER, and a design journal page `#ruins`.
