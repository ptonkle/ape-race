# Banana Bay Battle: overnight, 2026-10-15

You left the build running ("Run until the full build is done"). **It is done and on main, but not deployed**, because you asked to hold the deploy until you say so. The live site is still v1.11.4.

- Every step is in `notes/bay-progress.md`. Steps 1–18 are Done; step 19, Deploy, is waiting on you.
- The shots are on the design journal's Banana Bay Battle page: https://claude.ai/artifact/RHVqaZd4LC5CQZRHnztxWe#bay
- Final checks on main all passed in Chromium, Firefox and WebKit: check.sh took 307.6 s and check-online.sh 94.0 s.

## Questions for you (most important first)

1. **Shall I deploy it?** If yes, it goes out as **v1.12.0** with the patch notes below. Online needs PROTOCOL 11, so players on 1.11.x are asked to refresh.
2. **The camera in the rigging.** The chase camera no longer clips into hulls, decks or yards. When you swing under a ship's yards, though, the yards and furled sails still cross the top of the view (journal: "Over the first galleon", "High in the rigging"). The fix would be to keep the camera below the lowest yard or out past the yard tips, which is a bigger change to the chase camera. Do it, or play it first and see? (SECOND-ORDER BR7)
3. **30 ships, not about 20.** Ropes can only hang from masts. A rope every 25 m or so needs a ship every 45–50 m, so 20 ships made a race of about 40 s. With 30 ships (the top of your 20–30), the Monkey AI takes about 67 s. More ships or wider gaps would make it longer. (BC2)
4. **No sails are set on the race line.** Every sail the course swings past is furled (rolled on its yard), torn into strips, or missing, so no ape passes through canvas. The background battle either side does have ships with full sails. (BC, BQ1)
5. **The start is a tall pier, 12 m above the water.** From a low pier the AI swung back and forth at the start for about 8 s. (BC3)
6. **The lobby spot.** Your ape stands at the pier's seaward end, with the first galleon behind. Its rig fills the top half of the view and the planks fill the lower third. Keep it there? (BL6)
7. **The Author medal is 0:35**, only 0.06 s under the best ape kind's measured run (Proboscis). Is it beatable enough? The spread between kinds is wide on this map, 35–54 s. (BT2)
8. **The navy flag is made up**: purple with a gold chevron and a white ring, so it belongs to no real nation. (BM5)
9. **"pirate on the shoulder"**: I read this as a parrot. The orangutan wears a green parrot and the lemur a scarlet macaw.

## Patch notes, ready for the deploy (v1.12.0)

- New map: Banana Bay Battle. Swing through a pirate battle at 3 pm, from a tall pier to a treasure island.
- Pirate outfits for every ape. On this map the Floor is Lava jaguar dresses as a navy officer.
- Fall in the sea and you're put back on the last ship you passed. Flagged barrels mark the edges, and bouncy barrels can save you.
- New weather on Banana Bay Battle: Squall.
- Settings shows the weather you actually chose, Squall included.

## What was built

| Part | What | SECOND-ORDER |
| --- | --- | --- |
| Course | 1.46 km, 30 ships at anchor: 8 galleons, 8 frigates, 8 brigs and 6 sloops, pirate and navy in turn. 158 ropes from yards, masts and bowsprits. Decks you can run on. Doubloons on item ropes. | BB, BC |
| Rules | The sea puts you on the last ship you passed, or kills you in Floor is Lava. The flagged barrels are a soft wall. Bouncy barrels are bouncers. | BC |
| AI | Bay-only tuning. Over 16 layouts: Sloth 76.8, Monkey 67.1, Ape 59.7, King Kong 49.7 s. | BT |
| Medals | Author 0:35, Gold 0:42, Silver 0:52, Bronze 1:07 | BT |
| Outfits | Each kind gets its own mix of tricorn or bandana, eyepatch, cutlass, bandolier, peg leg, parrot, hook, coat. The navy jaguar wears a bicorne and epaulettes. | BG |
| Look | Sun 40° up, cumulus, and a moving sea driven by the physics swell (turquoise shallows). Squall: rain, slate sky, whitecaps, lightning. | BS |
| Art | Ships with cream canvas and crews fighting on deck. Cannon fire between neighbours. A 51-ship background battle either side, some ships burning. The pier and gantry, and the island with an oversized chest overflowing with golden bananas. | BM, BA, BP, BQ, BR |
| Camera | Keeps out of the sea, hulls, decks and yards, and doesn't lift too far or look steeply down. | BP1, BR1 |
| Menus | Lobby on the pier, a title flyover, the map card. Settings now lists each weather with its map. | BL, BW |
| Cost | Worst step 1.9–3.3 ms, like the Ruins. The worst frame draws 6,973 shapes, under the Ruins' roughly 12,000. | BT3, BR5 |

## Smaller things still open

- A rope close to the camera can read as a thick pole (BR7).
- The foam round barrels is a little busy (pale flecks) (BR7).
- In a squall, bouncy barrels ride the calm swell (the physics needs that) while their foam rides the rough sea (BR7).
- Far wreck smoke reads as a line of faint dots from a long way off (BR7).
