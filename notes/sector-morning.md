# Sector Ape: read this first (morning of 2026-10-14)

## Where it stands

- **Sector Ape is built and on main as version 1.7.0, not deployed.** Last session you asked for no deploys until you say so. Say "deploy" and it goes out (this release also carries the new map picker and the snow leopard).
- **All checks pass.**
  - `check.sh` and `check-online.sh` pass in Chromium, Firefox and WebKit.
  - Every Rust test passes, including new tests that the Jungle, City and Ski Slope replay tick for tick exactly as before (plain and Floor is Lava).
- **Screenshots:** https://claude.ai/artifact/RHVqaZd4LC5CQZRHnztxWe (Sector Ape at the top). The image files are also in `notes/sector-shots/`.
- **The run:** about 2.3 km.
  1. The start deck, then a debris field.
  2. The Hulk, a rust-red freighter with three routes: its corridor with the switch door, over the top, or round the side.
  3. A bend, then the wreck of the mothership with three routes: between its halves, over the bow, or the hidden 10 × 7 m gap through the bow.
  4. The asteroid belt, then the mech armada with vines hanging from the mechs.
  5. The banana black hole as the finish. Finishers land on a deck under it and dance.
  - The mothership flies overhead from second 14 to 32. Battles and dogfights fill the distance.
- **Physics, this map only:**
  - gravity 9 (17 elsewhere);
  - about half the vines hang sideways, upward or slanted;
  - top speed 85 m/s (306 km/h);
  - debris bounces you with a boost;
  - crossing the boundary puts you back on the nearest deck. With Floor is Lava, the space jaguar pops in instead.
- **AI** over 32 layouts: Sloth 98 s, Monkey 92 s, Ape 86 s, King Kong 73 s. Every race finishes.
- **Medals:** Author 0:56, Gold 1:03, Silver 1:13, Bronze 1:28.
- **Step cost:** the worst frame is now about 1.4 ms on every map. The other three maps got faster too, with the same results.

## Questions for you (the most important first)

1. **Deploy 1.7.0?**
2. **Does it feel right?** These were my calls:
   - low gravity (9), with reach ×1.5 and air steering ×2 on this map;
   - the speed comes from odd vines (W keeps driving up to twice the usual speed), a slingshot when you let go of an odd vine pointing the right way, and the debris boost. A body's gravity only sets which way its vines hang, as you said.
3. **The reset** puts you on the nearest deck, even one ahead, standing still. Or should it be only decks behind you?
4. **Bouncing off debris** pushes you forward along the course, not along the bounce, so you're never flung backward or out of the map. Is that the "elastic collision" you meant?
5. **The hidden gap** is on the main line, so King Kong takes it about 1 race in 8. Should it be harder to find?
6. **The switch door:** 5 s shut, after a swing more than 55° forward. The AI doesn't shut it on purpose. Should it?
7. **The black hole** is tipped back 35°, so you fly in under it and see it from below. In your example it was seen from a little above. I think it looks rough from below (screenshot 11).
8. **The boundary** is hinted by faint corner brackets every 110 m. Keep them, or make the edge truly invisible?
9. **The minimap** hides the secret route. Should it show it?
10. **Busy?** The art added extra landmarks: battles, ring stations and ape-wing flights.
11. **The mechs aren't solid;** only their vines matter.

The agents' full lists, with where each setting lives, are in `notes/sector-questions-course.md` (11 items) and `notes/sector-questions-art.md` (17 items). Ignore art item 17: the finish-deck fall it describes is fixed. Finishers now land on the deck, and `tests/sector_finish.rs` checks it.
