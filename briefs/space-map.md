# Brief: the space map (Star Fox–inspired)

Given by the person on 2026-10-13, to be built in a later session (after their usage resets). Their words are kept exactly below; the notes after them are the integrator's, for the session that builds it. Read `MAPS.md` first (and notes/maps-archive.md "Art lessons", especially "Establish the time of day and the weather first" and "Be visually distinct from the first iteration").

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

## The person's answers (2026-10-13)

- **Name: Sector Ape** (map key `sector`, or `sectorape`).
- **Lighting: a big sun far below.** A blazing orange-white sun below the track lights everything from beneath, with strong rim light. This is the map's "time of day".
- **Switch door: it stops rivals behind you.** One shared door: whoever swings the switch vine far enough forward shuts it for everyone behind, who must take another path. It reopens after a few seconds. It must be deterministic from race state, synced online and kept in replays.
- **Reset: past a boundary.** Crossing an invisible edge round the track resets you at once to the nearest piece of ground, like falling off a kart track.
- **Mechs: they carry vines and they move.** You hook through the armada on vines hanging from the mechs, which march or drift on fixed paths (the same every race, from the tick, like the City's cars).
- **Top speed: about double** the other maps, up to ~300 km/h. The speedometer gets a bigger scale on this map, the camera widens further, and collisions must stay safe (no passing through thin debris).
- **Gravity per debris piece: sets the vines' direction only.** Each ship sets which way its vines hang (sideways, upward, at angles). The ape always feels only the weak pull toward the sun below (lessened gravity in open space). No walking on the sides of ships.
- **Great Fox pass and distant battles: scenery only,** big moving set pieces timed to the race.
- **Floor is Lava on this map:** there is no floor, so **hitting the invisible boundary is the kill condition**: crossing it in Floor is Lava means the jaguar gets you, instead of the normal reset to the nearest ground. Without Floor is Lava, the boundary resets as above. (`Course::floor` / `on_floor` and `jaguar.rs` assume a floor today; this map's "touched the floor" becomes "crossed the boundary".)
- **The jaguar on this map wears a classic cartoon bubble space helmet** (a clear glass dome over its head), **but no spacesuit**. **The person (2026-10-13): it's funnier if the space jaguar appears out of nowhere** (pops into being right beside the ape, no run-in).

## Integrator's notes and open questions (for the building session)

- **Lobby zone:** design where your ape stands behind the menus as a showpiece (MAPS.md step 9; archive "Checklist for a new map"): e.g. on a ship's deck with the sun below and the Great Fox–type mothership behind, with room for the lineup of every player's ape (T76).
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

**Costumes built (2026-10-13):** branch `sector-gear` has `models::Gear::Space` (flight jacket with team scarf, badge and patches, headset, one metal arm, aviator sunglasses; SECOND-ORDER SG1). Switch them on for the map with one line in `models::Gear::on`: `crate::map::Map::Sector => Gear::Space,`. Preview on any map with `?tile=map=ski,gear=space,view=orbit,...`.

## The models, built ahead (2026-10-13, branch `sector-models`)

Stand-alone builders in `crates/core/src/spacecraft.rs`; no map draws them yet. Each is built in its own metres (+z forward, +y up, about its middle) and placed by a frame: `spacecraft::pose(at, yaw, pitch, roll, scale)`. Animation comes from the tick alone. Glowing parts (engines, eyes, lasers, the sun, the bananas' core) use glow 2 ("sky" in `scene.frag`: full colour, no fog), so they read far away; the planet uses the far-backdrop range (lit, unfogged). `spacecraft::look()` is a starting `Look`: black-blue sky, the light pointing toward a sun below (`light_dir.y` −0.92), its orange bounce lighting down-facing sides.

**See them:** `?gallery=<model>[,yaw=,pitch=,dist=,at=,hold=on,x=,y=,z=]` (`app/gallery.rs`): one model turning once per 40 s on a starfield, lit as above, no fog, no menus. Models: `mothership wreck grunt brute drift apewing enemy dogfight battle chunk girder asteroid debris blackhole sun planet cat armada`. Screenshot with `scripts/shot.py OUT --query "gallery=wreck,hold=on" --bare`.

| Builder | Size (as built) | Shapes (+ soft glows) | Notes |
| --- | --- | --- | --- |
| `mothership(b, f, tick)` | 242 m long (`MOTHERSHIP_LENGTH`), 140 m across the wings, ~52 m keel to beacon | 63 (+4) | Great Fox–type carrier, our own design: segmented white hull, split prow round a lit hangar mouth, drooping swept wings with blue tips and engine pods, bridge tower with blinking beacon, four pulsing engines. For the pass overhead. |
| `wreck(b, f, tick, gap)` | ~266 m long overall (halves drifted apart, ~28 m gap between torn faces), x −79…72, y −25…33 | 121 (+2) | Broken at z = −10 m: stern yawed back, bow nose-down and rolled; torn faces with decks, jagged ribs, peeled plates, embers, fire glow; scorch marks; port wing torn off below; 8 tumbling chunks between the halves; lights out, red alarm strips. |
| `wreck_passage(f, gap) -> Passage` | — | — | The hidden route: straight **across the bow half** (along its x), through the hangar deck at z = 40 m. Clear opening `gap` = (width along the hull, height), default `WRECK_GAP` 10 × 7 m, clamped to 2–17 × 2–18 m; length 35.2 m mouth to mouth (hull 34 m + 0.6 m each side). With `f` = identity its middle is at (0, −3.1, 55.6) m and its axis tilted ~9° by the bow's roll. The hole in the hull is `RECESS` (1.2 m) taller each way (ribs, pipe and lining sit in the recess). Returns `middle`, `entry`, `exit`, unit `axis`, `width`, `height`, `length` in world units (scale included) for collisions. Test: 65 sample points across the opening are inside no shape. |
| `mech(b, f, &Mech{kind, gait, phase}, tick)` | Grunt 13 m (`MECH_HEIGHT`; feet 6.6 m below the pelvis origin), Brute ×1.5 (23 m with spikes) | Grunt 37, Brute 41 | Monkey mech: hunched barrel chest, huge pauldrons, ape head (brow ridge, muzzle, round ears, red glowing eyes), long arms with fists at knee height, thruster pack. Grunt oxblood; Brute purple, horned and spiked. `Gait::March` (stride every 2 s, arms swinging) or `Gait::Drift` (legs dangling, arms reaching forward, thrusters full). `phase` puts a line out of step. |
| `mech_anchors(f, &mech, tick) -> MechAnchors` | — | — | Vine anchors this tick, world: `head` (crown; grunt at rest ≈ (0, 7.2, 2.9) m), `hands[2]` (under each fist, ≈ (±4.4, −3.6, 1.4)), `elbows[2]`, `shoulders[2]` (top of the pauldrons). Index 0 is the mech's +x side. They move with the walk, so a vine follows. |
| `ape_wing(b, f, tick)` | 11 m long, 12 m across | 15 | Our own Arwing-like dart: white, banana-yellow spine and tip fins, glass canopy, two round ape-ear intakes, twin engines. |
| `enemy_fighter(b, f, tick)` | ~9 m | 7 | Dark purple pod, magenta eye, three red claws swept forward, magenta engine. |
| `laser(b, a, z, w, c)` | — | 1 (+1) | A bolt: green (`LASER_GOOD`) or red (`LASER_BAD`). |
| `dogfight(b, f, tick)` | loops 120 m across, ±14 m | 55 (+ up to 5 bolts/flashes, + up to 4 glows) | Two chasing pairs on crossing figure-eights (14 s a loop), twin bolts every 0.3 s, every fourth volley hits. Draw 2–4× scale far off. |
| `capital_battle(b, f, tick)` | ships 240 m apart (cruiser 90 m, dreadnought 100 m) | 25 (+ up to 6) | Friendly cruiser vs enemy dreadnought broadside, bolts crossing in 0.4 s, flashes on hits. |
| `hull_chunk(b, f, tick, k)` | ~12 × 10 m | 11 | Curved plates with the mothership's blue band, ribs, scorch, torn spikes, blinking light. 3 variants. |
| `girder(b, f, len)` | `len` m along z, 2 m square | 4 + 2 per 3 m bay + 2 (24 m: 22) | Truss, hazard band, one rail bent out. |
| `asteroid(b, f, k)` | ~10 m | 8 | Lumpy rock, craters, glowing violet crystals (Sector X's haze). 4 variants. |
| `tumble(tick, k) -> Affine` | — | — | A slow spin to compose into a drifting piece's frame (not for pieces carrying vines). |
| `black_hole(b, f, r, tick)` | disc to 2.9 r (gallery: r = 20 m) | 349 (+1), 240 of them in 80 bananas | Black ball radius r; **event horizon of bananas**: a ring of 24 at 1.14 r in the disc plane and two rings of 18 at 1.22 r tilted ±62°, all turning; two spiral arms of 10 bananas falling in and shrinking; three bands of glowing streaks (white-hot, orange, violet) swirling, inner faster. The finish line is the horizon. |
| `sun(b, at, r, tick)` | flares to 1.45 r, glow to 3.8 r | 11 (+2) | White-hot ball, ten pulsing flares, orange corona. Draw round the eye each frame inside the 600 m far plane, far below. |
| `planet(b, at, r, tick)` | ring to 2.6 r | 50 (+1) | Violet gas giant with teal and lilac bands, 44-segment ring (tilted 18°, turning), a moon. Backdrop, round the eye. |
| `stars(b, eye, n)` | ball 520 m round the eye | n | Fixed starfield. |
| `models::SPACE_JAGUAR` | jaguar | jaguar + 4 (+1) | The jaguar in a cartoon bubble helmet (glass glow over the head, metal collar, two glints), no spacesuit. `BigCat.helmet`; `BigCat::on` never returns it yet (the map session adds `Map::Sector => SPACE_JAGUAR`). **The person: it appears out of nowhere** (pops in beside the ape at the boundary, no run-in): `jaguar.rs` behaviour for the map session. |

**Budget:** a typical view (wreck 121, mothership pass 63, a dogfight 60, a battle 31, an armada of 8 grunts and 2 brutes 378, 20 debris pieces ~200, sun + planet + 400 stars ~460) is about 1 300 shapes; the black hole finish adds 349. Well inside the 6–7k drawn at once. No LOD yet: the fighters and capital ships are already small; if an armada of 30+ mechs is in view, a 6-shape far mech (body, head, eyes, arms) is the obvious next step.

**For the map session:** place backdrops inside the far plane (`render::FAR` = 600 m), drawn round the eye; check the tops (dark under the sun-below light) in play and consider a rim term in `scene.frag` gated to this map (SECOND-ORDER SM4); the gallery shows the models unfogged, the map's fog will dim distant hulls (their glows stay).
