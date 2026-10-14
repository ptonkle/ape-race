# Sector Ape: questions for the person (course agent, 2026-10-14)

Decisions I made while you were asleep, that you may want to change. Each has a SECOND-ORDER row.

1. **Reset onto the nearest deck, even one ahead** (SC3). You said "resets you to the closest piece of ground". The plan said "the nearest deck not ahead", which cost 60–250 m a reset. Now it is the nearest along the course; you start again standing still, so it is never quicker than flying on. Keep it, or only decks behind you?
2. **Rebounds push you forward along the course** (SC4), not along the bounce, and never fling you back down the course or down out of the map. A swing that touches debris bounces back but you keep hold. One boost a quarter second. Is that the "speed boost and elastic collision" you wanted?
3. **Gravity 9 m/s²** (the other maps 17), reach ×1.5 and air steering ×2 on this map only. Too floaty? Too easy to blast off?
4. **About half the vines hang another way** (down 20% of station vines plus every deck row, the corridor and the over routes; up, level and slanted the rest). Up vines turn you down, level ones turn you sideways. Do they feel learnable?
5. **The wreck's hidden gap is on the main line** (straight through the bow). The AI below King Kong never aims for it; King Kong takes it about 1 race in 8. Should it be harder to find (off the line)?
6. **The switch door**: shuts 5 s when the vine just past it is swung 55° forward. The AI does not try to shut it on purpose. Want rivals to use it deliberately?
7. **Mechs are not solid**: you pass through them; only their vines matter.
8. **The finish**: a big deck under the black hole where finished apes land and dance; one that floats into the hole sits at its banana horizon.
9. **Medal times** (Author 0:56, Gold 1:03, Silver 1:13, Bronze 1:28) come from one King Kong run per ape; runs on this map vary more than elsewhere. Worth checking against your own times.
10. **Step cost**: now done (SP rows): worst frame 3.9 → 1.4 ms on a quiet machine (the Jungle 3.4 → 1.4, the City 5.0 → 2.7, the Ski Slope 4.4 → 2.0), every race exactly as before. The speed-ups to shared sums (the square root, `nearest`, `progress`) touch every map but change no value; the tests that pin the other maps still pass. Fine to leave ungated (SP3)?
11. **Finishers falling past the deck** (SF7, SP8): one in four did. Now a finisher whose feet go 2 m under the finish deck is put on the deck at the nearest place, standing: a jump, not a glide up. Want it smoother (a slow lift round the deck's edge)?
