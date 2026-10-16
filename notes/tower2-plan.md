# Mape's Tower, second design: the plan (2026-10-16)

From `briefs/mapes-tower-v2.md`. Sizes are relative to the first design (the keep: 137 m tall, 41 m radius; a spiral round it at 55 m; a Monkey AI's climb about 40 s, the whole race 92 s). Branch `tower2`; the live map stays until the new one passes its checks.

## The course, leg by leg

1. **Start:** a deck at the foot of tower 1, as now (16 m up), facing the lower bridge.
2. **Under the lower bridge (about 10 s):** the bridge deck low, about a third of the way up tower 2; its vines hang from the underside, one station every 22 m as now, so the swing runs level just under the deck. Length set so a Monkey AI takes about 10 s (about 250 m, two current spirals' worth).
3. **Up the outside of tower 2:** tower 2 is 60% of tower 1, so about 1.2× the old keep. One and a half turns round it on balcony mages' and dragons' beams, with the climb lift as now, from the lower bridge's height to the upper bridge's. The clouds sit round its top.
4. **Under the upper bridge (about 10 s):** the same as the lower one, mirrored, from tower 2's top back to tower 1, ending at tower 1's doorway about half way up tower 1.
5. **Inside tower 1:** tower 1 is twice the old keep (about 275 m) and wider: an outer radius 1.25× the keep's, hollow, so the inside is about 90 m across. From the doorway half way up, vines are cast from mages at inner balconies and windows all round the wall and from a few floating conjured anchors in the middle, scattered with no helix: anchors every 15–25 m of height, each within one swing of two or three others, so there is always more than one way up. The course's centreline (what "progress" measures) is a loose zigzag up the middle; the vines do not follow it. Windows let the sky in. The lift on climb vines stays; resets land on ledges inside the wall with an upward boost.
6. **Out through the portal:** the top of tower 1 is open; the wizard and his tear stand beside the top on its far side, and the last beams throw you out sideways through it. The tear is no longer a flat lozenge: it turns and breathes like Sector Ape's black hole (`sectorscape`: the turning bands, the pull, the sparkles drawn in), simpler than that one, with the void and the eldritch banana seen through it.

## The portal world

7. **The tube:** a vertical tube, Dropper-style, about 300 m, its inside about 2/3 of a current spiral's width (roughly 35 m across), in the void's colours with the zip running down one side and the creatures looking in through rings. **20–30 bounce pads** across it at intervals, each covering part of the width (a half, a ring with a hole, two quarters), their gaps lined up so you weave: left, right, middle. Touching a pad bounces you back up (the current mushroom bounce), so it costs you a second or two; the fastest line never touches one. No vines. The tube walls are solid.
8. **Air control in the tube:** on the current rules (8–12 m/s² sideways, a fall at 50 m/s) you shift under a metre between pads 12 m apart, which is not dodging. So, map-only and measured first: the void's air is thicker (a lower terminal speed in the tube, about 35–40 m/s, so the fall takes about 8 s) and air control is about 3× (the void pushes back). The skeleton agent measures a human-like weave and sets both numbers so a pad's gap can be reached from the previous gap by a real player; the AI falls the tube with the same steering.
9. **The bottom row:** a line of vines hung from the top edge of the tunnel mouth, 10° below horizontal, reaching out into the tube like the start row. Grabbed at falling speed they swing you under the anchor and you let go flying level into the tunnel.
10. **The tunnel:** horizontal, about the length of a bridge, the finish line part way along; the sling's flight reaches it without landing. Past the finish, the banana sea and the dancing, as now.

## Rules kept, rules gone

- Kept: climb lift, resets to the last pad with a boost (pads on every leg; in the tube a reset puts you back at the portal), solid stone, the vine audit, crates in rows, wizard gear, the skies, the flame cat.
- Gone: the trampoline room, the sub-tower branches, the mirrored zipper, trampolines on the climb.
- New: the tube's air rule (step 8), bounce pads as obstacles, the sling row, the moving portal.

## Work split

Three agents at once, each on its own worktree off `tower2`, about an hour each:

- **Skeleton and contract:** the new `TowerLayout` (towers, bridges, doorway, interior, tube, pads, tunnel), the centreline and progress, the tube rules (air, pads, walls), the sling; a stand-in course finished by every AI level; the Dropper measurement (step 8).
- **Models:** hollow tower 1 with its doorway, inner balconies and windows; tower 2; the bridges with vine hangers under their decks; the tube's rings and pads; the tunnel mouth and tunnel; reusing the mages, dragons, wizard, tear, creatures, zip and banana sea.
- **Art and lobby:** the placement module, the lobby spot and the flyover and map card, once the contract lands (this one starts after the skeleton's contract is in).

Then: the course agent (layout and AI) after the contract; validation, medals, costs, colourblind, shots, docs, full checks; deploy as 1.14.0 replacing the live tower.

## Open points (decided unless you say otherwise)

- The bridges' decks can be walked on but the race never goes over them; falling off a bridge resets you to a pad under it.
- Tower 1's interior climb starts half way up (where the upper bridge arrives), so each half of tower 1 is one segment: the outside of tower 2 covers the lower half's height, the inside covers the upper half.
- A bounce in the tube is the only penalty; no ape is knocked down the tube by another.
