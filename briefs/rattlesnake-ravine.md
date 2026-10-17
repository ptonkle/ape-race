# Brief: Rattlesnake Ravine (the slot canyon map)

Given by the person on 2026-10-17. Their words are kept exactly below; the answers and notes after them are the integrator's. Read `MAPS.md` first.

## The person's description (verbatim)

> Map Setting & Aesthetic: This map should be designed to look like, and is heavily inspired a sprawling ravine/ slot canyons of Utah. The ravine should have varying wooden rope bridges and  logs stretched atop, with vines hanging down in varying lengths. The ravine walls should act as barriers for the player. The lighting for this map should be soft, with various torches, groups of fireflies, and glowing mushrooms guiding the player towards the finish lane. The ravine should be covered in moss, and lichen. and have Dinosaur bones and rocks and fossils scattered in the walls. You should be able to see a beautiful starry sky above. Approximately 15% of the vines on this map should be snake vines(defined below), dispersed along the track.
> The map track should follow a line similar to the grand canyons shape, but with loosely intertwining pathways, almost like a a braid. The map should should diverge at some point into an upper passage and a lower passage.  The lower passage should enter into a tunnel that contains old mining equipment, and signs of digging for fossils and gold. The upper passage is a shorter pathway by distance, but contains a higher concentration of snake vines, and has sparse vines in general.
> This map should reward high-risk pathing. there should be multiple caves, archways, long jumps and tight corners.
> At one point there should be  an optional shortcut that goes through a large T-rex fossils mouth about halfway up the wall of the cave. The shortcut should save no more than 5-8 seconds of time. The players have to shoot the gap between the sharp teeth of the fossil. the shortcut is difficult to get through, the gap should be only 2x as tall as the largest ape model. If the player brushes against the teeth they should be reset  back to the floor. Otherwise, the players can go around the fossil shortcut.
> Costumes: The costumes from this map are heavily inspired by cowboys and the wild west.  Each monkey should have a different color pallet. some examples of accessories could include a Large leather hat, Bandanas, leather chaps, suspenders, holsters or bandoliers containing a banana, boots and spurs.
>
> Large  colorful Lizards should walk vertically on the walls, with the same properties as the turtles on the Sunset Ruins Map. Lizards should be varying shades of red, yellow, or blue.
>
> Similarly, there should be small sections of rickety wooden train tracks crossing roughly parallel along the ravine. the tracks can change elevation, or cross from one wall of the ravine to the other.These tracks should come out of Dark holes in the walls of the ravine that act as barriers to the player. At various intervals, a minecart should follow along the tracks, much likes the cars that travel in concrete jungle, and with the same properties.
>
> New Hazard: This map introduces a new hazard: the snake vine. Create a new model for snake vines. Snakes hang from the same branches and ledges that regular vines do in this map. If the player grabs a snake vine, then it should snap in half and the players momentum should plummet to the ground.
>
> The Leopard (for floor is lava mode) in this map should still be a jaguar but with 8 legs and arachnid themed.
>
> Ultimately the Ravine ends into  large barren valley Mesa setting. the finish line lies at the feet of  a large sphinx statue, except it has the head of the proboscis monkey. there should be a fall campfire set up, with bananas roasting on a stick stuck into the dirt.Some tents should be nearby.

## The person's answers (2026-10-17)

- **Name: Rattlesnake Ravine**, map key `ravine`.
- **Snake vines are visible if you look:** each has a clear snake head, scales and a slow sway, so a sharp player sees it coming. The AI avoids most of them (the Sloth the least).
- **Teeth reset:** brushing the T-rex teeth drops the ape onto the canyon floor right under the fossil with no speed; it carries on the long way round. In Floor is Lava mode, touching the teeth is the kill.
- **Process:** build on branch `ravine` (worktree `~/slop/ape-race-ravine`), in parallel agents where possible; **do not deploy** until the person says so.
- **Before building:** review feedback on earlier maps for common pitfalls (geometry with no collision, camera problems, …) and build the checklist into every agent's brief: `notes/ravine-pitfalls.md`.

## Requirements, as a checklist

- [ ] Night slot canyon (Utah): red-orange sandstone walls covered in moss and lichen, with dinosaur bones, rocks and fossils set in the walls; a starry sky visible overhead.
- [ ] The walls are barriers (collision matches what is drawn).
- [ ] Soft lighting: torches, firefly swarms and glowing mushrooms leading toward the finish.
- [ ] Wooden rope bridges and logs across the top, with vines of varying lengths hanging from them and from ledges.
- [ ] About 15% of all vines are snake vines, spread along the track; new snake-vine model. Grabbing one snaps it in half and the ape drops to the ground, losing its momentum.
- [ ] The route follows a Grand Canyon-like winding line, with loosely braided, intertwining paths.
- [ ] A split into an upper passage (shorter, sparse vines, more snake vines) and a lower passage (a mine tunnel with old mining equipment and fossil and gold digs).
- [ ] Rewards risky lines: several caves, archways, long jumps and tight corners.
- [ ] Optional T-rex fossil shortcut halfway up a cave wall: you fly through the mouth between the teeth, through a gap 2× the height of the largest ape. It saves 5–8 s at most. Touching the teeth resets the ape to the floor below. There is a way round.
- [ ] Wild West gear on every ape, each ape in its own colour palette: big leather hat, bandana, chaps, suspenders, a holster or bandolier holding a banana, boots and spurs.
- [ ] Large lizards in red, yellow and blue walking up the walls: moving bouncers, the same as the Sunset Ruins turtles.
- [ ] Short stretches of rickety mine-cart track running roughly along the ravine, changing height or crossing from wall to wall, coming out of dark holes in the walls (the holes are barriers). Mine carts run along them at intervals, the same as the City's cars.
- [ ] Floor is Lava: the jaguar becomes an eight-legged spider-jaguar.
- [ ] Finish: the canyon opens onto a large, barren mesa valley; the finish line is at the feet of a sphinx with a proboscis monkey's head; a campfire with bananas roasting on sticks pushed into the dirt; tents nearby.
