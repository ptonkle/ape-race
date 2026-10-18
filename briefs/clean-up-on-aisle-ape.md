# Brief: Clean-Up on Aisle Ape (the grocery store map)

Given by the person (Sam) on 2026-10-18, in a workspace that builds maps for Ape Race and offers them upstream as pull requests. Their words are kept exactly below; the answers and notes after them are the integrator's. Read `MAPS.md` first. The design journal for this map (mockups, route plan, scale, decisions) is Sam's, outside this repository.

## The person's description (verbatim)

> the map idea is monkeys going through a grocery store, swinging from people who are obstacles, alongside other obstacles like stands, aisles, etc. I've added a photo of a food lion floor plan that I'd like the map to follow, with you crossing as much of it as you can, criss crossing through some aisles but not all, so the map is still abou 90 seconds long. I've also got this photo album of flicker of the realistic inside of a food lion for reference as you build the map to use similar lighting, color pallettes, sizes, etc.  https://www.flickr.com/photos/ryanrules/52791635406/in/photostream/. I want the level of detail on this map to rival sector ape, so people moving around the aisles shopping, sprays shooting out of the fruit displays to cool them, a frozen section, each aisles should be filled with a bunch of individually made assets that represent different products that could be in that aisles, and every aisle should have 5-10 uniquely sized, colored, and textured boxes to put up for display that are repeated / ordered to mimic the real inside of a grocery store. please start with design journaling everything as described in the skill for my review and inspection like mockups and screenshots and map flow diagrams, then present me any design decisions about the map you need me to answer, then begin the map build, dont stop till its locally deployed for my testing. the apes in this map are all little food items, so one could be a barnum cracker, another a fruit snack, maybe one shaped like an oreo, you figure out the rest of them. thus the apes on this map should be rather small in comparison to the humans.

The floor plan (a Food Lion prototype: angled, bowed aisles; frozen food as two double runs in the middle; produce at the entrance corner; an arc of checkouts) and twelve reference photos of Food Lion #1624's interior were given alongside; they are references only and are not in this repository.

## Defaults built on (the person may change any; GR-Q1 … GR-Q12 in the journal)

- **Name: Clean-Up on Aisle Ape**, map key `grocery`, row prefix `GR`.
- **No brand in the game:** no store name, logo or mascot; the plan, palette and fixtures follow the photos.
- **Scale 1:12:** an ape (3.3 m drawn) is a 27 cm snack; shoppers are 20 m, shelves 23 m, the ceiling 52 m. Aisles are widened to 38 m.
- **Sealed aisles:** top stock to 42 m; a wall's solid reaches 10 m past its drawing (the Ravine's `WALL_ABOVE`), up to the ceiling's roof.
- **Swinging from people:** shoppers carry balloon bunches; the string is a vine whose anchor walks with them (a sliding vine). Shoppers' heads and shoulders are moving bouncers (as the City's cars); bodies are not solid.
- **Misters** in produce puff spray on a timer (dressing only). **Frozen aisle:** cold light and haze, glass doors, coffin freezers down the middle (solid tops).
- **Vines:** green sour-apple licorice ropes from the ceiling grid.
- **The apes are food:** Chimpanzee an animal cracker, Gorilla a sandwich cookie, Orangutan a fruit snack, Gibbon a soft pretzel, Proboscis monkey a cheese puff, Lemur a frosted circus cookie. **Floor is Lava:** a gummy jaguar.
- **Start** at the stockroom doors in the back corner by the dairy; **finish** on the front-end run past the checkouts, by the exit doors.
- **Process:** branch `grocery`; held with `Map::held` until Peter releases it; no `VERSION`, patch note or `PROTOCOL` change (named in the PR text instead).

## Requirements, as a checklist

- [ ] A grocery store following the floor plan's arrangement: dairy left, meat and seafood back, deli and bakery right, produce front right, checkouts front, five aisles, the frozen pair, six aisles, bowed as drawn.
- [ ] The route crosses as much of the store as it can, through some aisles but not all, about 90 s for a player.
- [ ] People shopping and moving round the aisles; apes swing from them; they are obstacles.
- [ ] Other obstacles: stands, aisles, display bins, end caps, columns, checkouts.
- [ ] Lighting, palette and sizes from the reference photos.
- [ ] Detail rivalling Sector Ape: misting produce displays, a frozen section, every aisle filled with individually made products, 5–10 product models per aisle with their own sizes, colours and fronts, repeated and ordered as on real shelves.
- [ ] The apes are small food items; small next to the people.
