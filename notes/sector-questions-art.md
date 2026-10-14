# Sector Ape: questions for the person (art)

The art agent's calls on Sector Ape's look that you may want to change. Each is one small edit; the SECOND-ORDER row says where. Screenshots are in `notes/sector-shots/`.

1. **The light.** The sun hangs far below, ahead and a little to the left of the way the course runs; its glow rises over the bottom of the view. Hull tops read violet-blue from an edge light, bellies sun-orange. Too strong, too blue? (SA1, SA2: `sectorscape::RIM`, `SKY`, `SUN_LIGHT`.)
2. **The vines are mint energy tethers** with a soft glow, an emitter collar where each leaves its body and pulses of light running down them. In reach they still turn yellow, catch-up vines coral, item vines carry violet crystals. Another colour? (SA5: `TETHER`.)
3. **The boundary is shown** by faint glowing corner brackets every 110 m, so you can learn where open space starts. Keep them, or make the edge truly invisible as the brief says? (SA24: `gates`, delete one call to hide them.)
4. **The minimap** shows the routes round the Hulk and the wreck as branches, but not the hidden route through the wreck's bow (it is a secret). Show it? (SA21: `route_lines`.)
5. **The black hole** is the course's landmark: always on the horizon from the start, its banana disc stood up to face you (60°), huge up close (r = 40 m: the disc spans about 500 m). Its ball is smaller than the banana horizon so the camera never ends up inside it. Too big, too bright? The disc's bands are many pastel and banana-yellow streaks after your example image (T79). (SA8, SA10, SA12, SA14, SA25.)
6. **The lobby spot.** Your ape stands near the left edge of the launch deck, looking out into open space, with the mothership riding behind it and two ape wings circling. The sun can't be in that narrow view (it is 56° down). Is this the spot? (SA15: `lobby_spot`, `LOBBY_LOOK`.)
7. **The speedometer** on this map reads 0–300 km/h (numbers every 50) and its hype levels are doubled (100 / 124 / 150 km/h), so the call-outs mean the same as on the other maps. JOJO MODE's sky starts at the same share of the dial (about 170 km/h here). (SA6.)
8. **The space jaguar** pops into being 5 m from you with a flash and sparkles, stares for the moment its run-in takes on the other maps (0.55 s), then leaps and bites on the usual ticks, and pops out again after eating. Should it pounce sooner? (SA7: `jaguar.rs` `RUN`; the leap's ticks are shared with the other maps, so a shorter stare would need its own constant.)
9. **Bouncy bodies have a faint pulsing shield glow.** Almost every piece of debris that carries a vine is bouncy, so there are many. Hitting one sends out a ripple of shield light. Keep the glow, or only the ripple? (SA17, `shield`.)
10. **Landmarks the art added** beyond the course's own set pieces: capital battles, ring stations, big dogfights and V-formations of ape wings outside the boundary every 230–300 m, and wreckage tumbling just outside it. Too busy? (SA22.)
11. **The switch vine** wears yellow-and-black hazard bands and a beacon that blinks amber, red while the door is shut; the door itself slams down from the corridor's roof in a quarter second with red lights over the corridor's mouths. (SA26.)
12. **The Hulk** (the door's ship) is a rust-red freighter with containers on top, its corridor lined dark with light strips. (SA23.)

## Added by the finishing pass (2026-10-14, branch `sector-finish`, SECOND-ORDER SF1–SF9)

13. **The black hole's disc now tips back** (35°, its near side up) instead of standing up 60° toward you. Stood up, it cut through the deck under the hole where the apes dance; tipped toward you, its near side came down into the way in. Now it arches over the armada and the last stretch, you fly in under it, and it passes well over the dance floor. Its lower jet is gone on the course (it went through the deck). You now see it from below on the way in, not from a little above as in your example. Is that all right? (SF2: `BLACK_HOLE_TILT`.)
14. **Debris is drawn the size you bounce off.** Hull chunks and asteroids are stretched to fill their collision boxes, so the chunks look thicker than before. Their thin torn spikes still poke out. (SF3.)
15. **The wreck** no longer has debris tumbling between its halves on the course (you flew through it). Its torn ribs and the big wing plates near the edge are still drawn but not solid. Make them solid, or trim them? Either changes the course a little. (SF5.)
16. **Sounds:** a warp when you are put back on a deck, a bulkhead slam and alarm when the door shuts (everyone hears it). (SF6.)
17. **For the course side:** about one finisher in four crosses the line low and falls past the finish deck forever, dancing in empty space (the pull to the hole is weaker than gravity). Fixing it is a physics change, so it was left. (SF7.)

## Screenshots (`notes/sector-shots/`)

Reshot against the final layout (2026-10-14). 01 the lobby (Race Setup), 02 the start, 03 a sideways vine, 04 a vine hanging up, 05 the door shut at the end of the Hulk's corridor, 06 the Hulk from outside, 07 the mothership's pass over the Hulk, 08 the wreck's bow with its passage open, 09 the asteroid belt and the black hole on the horizon, 10 the armada under the black hole's disc, 11 the black hole's banana horizon over the finish deck, 12 a finished ape drawn in at the horizon, 13 the space jaguar popping in, 14 its leap, 15 colourblind mode, 16 JOJO MODE at 221 km/h, 17 the black hole in the model gallery, 18 the space jaguar's helmet. Shot in Chromium with software rendering (SwiftShader), 1280×720.
