# Rattlesnake Ravine: pitfalls to avoid

The general list is MAPS.md "Common gotchas". This is what it means for this map; every agent's brief points here.

- **Only clear obstacles are solid** (the person, 2026-10-17; the ski lift is right to have none). Small dressing stays non-solid: vines, ropes, torches, mushrooms, fireflies, mining props, wall-inlay bones and fossils, tents and the campfire.
- **Canyon walls, cave roofs, archways, the mine tunnel, the wall holes the tracks come out of, the T-rex skull and the sphinx are solid from the first commit**, roofs included, with the solids reaching well past what's drawn. One layout function gives both the solids and the drawing. A test flies an ape at each kind.
- **The T-rex mouth:** the gap between the teeth is 2× the tallest ape. Solids and drawing come from the same tooth list. Brushing a tooth resets the ape to the floor below the jaw (a kill in Floor is Lava). The resettable state goes in the save points. The skull has a keep-inside volume so the camera doesn't sit in the bone.
- **Tunnel, caves, arches:** a keep-inside volume for the camera each (`Temple::kept_in` style), their own lighting (torches, mushrooms, lanterns), no filler block in any mouth.
- **Bridges and logs:** real decks you can land, stand and walk on. Vines hang from them at least 4 m clear of any top. The camera doesn't dodge ropes.
- **Snake vines:** placed from a fixed list (no lottery), readable in colourblind and JOJO MODE. The snap is shared and deterministic, and goes in the save points if it lasts.
- **Lizards and mine carts:** one `lizard_at(i, tick)` / `cart_at(i, tick)` each, shared by the physics, the AI and the drawing, like `turtle_at` / `car_at`.
- **Route:** spacings the AI already climbs (a station every 22–28 m, low vines, more than one way on). The upper passage's sparse vines must still be climbable. The shortcut is optional and is not needed for the AI to finish.
- **Start:** no solid within 16 m behind `path[0]`. **Lobby:** planned on purpose, checked at three window sizes.
- **Cost:** new placement uses its own `Dice`, so the other maps stay unchanged. Index the wall solids by place. Measure `frame_cost` and stay in the 5,500-shape budget.
