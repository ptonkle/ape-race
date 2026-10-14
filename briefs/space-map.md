# Brief: the space map (Star Fox–inspired)

Given by the person on 2026-10-13, to be built in a later session (after their usage resets). Their words are kept exactly below; the notes after them are the integrator's, for the session that builds it. Read `MAPS.md` first (especially "Establish the time of day and the weather first" and "Be visually distinct from the first iteration").

## The person's description (verbatim)

> Map Description: The map draws heavy inspiration from Star Fox, specifically the maps sector x, sector y, and sector z. thus the map is entirely in space, and each of the monkeys should get their own star-fox esque costume, like the ski slope  map. each monkey should have a metal limb and sunglasses, plus an extra touch to make them star fox like the coat they all have. The vines should be hanging from various large ships and debris that are floating in space along the track. The gimmick of this map should be that while in open space, there is lessened gravity, and thus you can easily blast yourself into space, which then resets you to the closest piece of ground. below you, there should be a planet or a sun far off in the distance to motivate the fact that there is gravity downwards in this patch of space. Each piece of debris / ship that has a vine has its own gravity, and so vines can be hanging in many different directions instead of from top to bottom; using the strange gravity of various platforms, I want you to create horizontal vines, and vines that are connected to the terrain at the base (i.e bottom of the view area) and hang in the opposite direction (i.e from bottom of view area to top of view area), and other atypical vine orientations. If you touch one of the debris / platforms that the vines hang from, it should give you a speed boost and send you off with an elastic collision. at one point, a great fox type ship should pass overhead, and the end of the map you need to hook through an armada of enemy mechs, that looks vaguely like monkey mechs, like sector y in star fox. make sure that as you go through the map, there are plentiful landmarks, like ships in the distance blasting each other, to a destroyed great-fox looking ship. There should be multiple diverging paths that take you THROUGH various pieces of debris, like there should  be a fast hidden route that involves sniping a small gap in a destroyed ship and flying through the wreckage. there should be a vine that acts as a switch that closes a door behind you if you swing it far enough forward.
>
> Map  Gameplay: emphasis on having to actively pathfind and improvise, and the difficulty should be getting used to swinging on the vines that are at bizzare angles and orientations. i would say 50% of the map should be normal vines, and the other 50% is atypical vines. if you hit the atypical vines well ,you should be able to go absurdly fast, faster than any other map has been able to support.
>
> Do not affect the physics for the other maps. They should play the exact same even with the new map implemented. you can verify this by saving a replay and making sure it finishes with the exact same time.
>
> The end of the mpa should be you getting sucked into a black hole, and the event horizon of teh black hole should be made up of bananas.

## Requirements, as a checklist

- [ ] Setting: entirely in space, after Star Fox's Sector X, Y and Z. A planet or a sun far below, so the downward gravity makes sense.
- [ ] Costumes on this map only, as the Ski Slope's gear (`models::Gear`): a Star Fox–style coat (flight jacket), one metal (cybernetic) limb and sunglasses on every ape kind.
- [ ] Vines hang from large ships and floating debris along the track.
- [ ] Lower gravity in open space; blasting off into space resets you to the closest piece of ground.
- [ ] Each ship or debris piece carrying a vine has its own gravity, so vines hang in many directions: horizontal vines, vines fixed at the base that hang upward, and other unusual angles.
- [ ] About 50% normal vines and 50% unusual ones. Hitting the unusual ones well gives absurd speed, faster than any other map supports.
- [ ] Touching a debris piece or platform that carries vines gives a speed boost and an elastic bounce.
- [ ] A Great Fox–type mothership passes overhead at one point.
- [ ] Plentiful landmarks: distant ships firing at each other, a destroyed Great Fox–looking ship, and more.
- [ ] Several diverging paths that go *through* debris. One is a fast hidden route: snipe a small gap in a destroyed ship and fly through the wreckage.
- [ ] A switch vine: swing it far enough forward and it closes a door behind you.
- [ ] Near the end, hook through an armada of enemy mechs that look vaguely like monkey mechs (as in Sector Y).
- [ ] The end: you're sucked into a black hole whose event horizon is made of bananas.
- [ ] Gameplay is about actively finding a path and improvising; the hard part is learning to swing on vines at strange angles.
- [ ] The Jungle, City and Ski Slope play exactly the same. Verify by replaying a saved replay and getting the identical finish time (`tests/jungle_unchanged.rs` does this for the Jungle; add the same for the City and the Ski Slope before starting).

## Integrator's notes and open questions (for the building session)

- **Lighting first** (MAPS.md rule): space has no time of day. Decide the light source before any art: the far sun or a star below, plus rim light from the planet. Make the palette distinct from the other three maps (deep black-blue space, neon wireframe and laser colours, Sector X's purple-green haze?).
- **Physics, new for this map only:**
  - Today gravity is a constant `rules::GRAVITY` pointing down.
  - This map needs a gravity field: a weak global pull toward the planet, plus each body's own pull near it.
  - A vine's rest direction then follows the local gravity, so a vine "hangs" sideways or upward. The rope and swing code (`ape.rs`, `vinesim.rs`) and the AI's flight planner (`ai.rs`, which flies copies of the ape ahead) assume gravity is down.
  - Gate all of it on the map, as the Ski Slope gated skiing (`Ape::ski` / `on_snow`), and keep it byte-identical elsewhere.
- **Speed cap:** "faster than any other map" collides with today's limits: the speedometer pins at 140 km/h, the camera's field of view stops widening at 42 m/s, and the ceiling is `rules::CEILING`. Raise these per map and check the camera, hype levels, the AI step cost and tunnelling through thin debris at high speed (more sub-steps?).
- **"Resets you to the closest piece of ground":** decide what counts as ground (any platform? the last one you touched?) and the reset rule: how far out or how long adrift before it triggers, and with what speed afterwards.
- **The switch door:** is the door each ape's own or shared? A shared door affects other racers, so it must be a deterministic function of race state, synced online and in replays. Decide whether it blocks rivals behind you (that is the point?) and when it reopens.
- **Mech armada:** are they bouncers, obstacles or vine anchors? Do they move (deterministic from the tick, like the City's cars)?
- **Black hole finish:** the finish line is the event horizon. After the finish, apes get sucked in during the outro (and the finish dances, T53: where do they dance? on the banana horizon?).
- **The hidden route:** it should be clearly faster but hard. Check the AI: Sloth shouldn't take it, King Kong maybe sometimes.
- **Build method:** follow MAPS.md (a skeleton with contract types first, then course, art and gear agents in parallel). Budget: the Ski Slope used a lot of the person's session usage, so start fresh.
- **Name:** not given. Suggestion: "Sector Ape" or "Deep Space"; ask the person.
