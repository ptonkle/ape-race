# Clean-Up on Aisle Ape: pitfalls to avoid

The general list is MAPS.md "Common gotchas". This is what it means for this map.

- **The physical store is a `RavineLayout`** (`Course.ravine`): gondolas, freezer runs, the store's walls and the checkout and service counters are its `walls`; the ceiling is one `Arch` room over the whole store; columns, fruit tables, display bins and coffin freezers are its `rocks` (posts: exact tops, which are floors); shoppers' bouncers are its `lizards`. No snakes, no skull, no tracks. The Ravine's rules, solids, AI wall checks and camera keeps then apply unchanged. Everything the Ravine's drawing would show is drawn by `groceryscape` instead: **every place the scene checks `c.ravine` must check `c.grocery` first.**
- **A wall's solid stands 10 m over its drawn top** (`WALL_ABOVE`). Gondolas and freezer runs are drawn to 42 m with their top stock so that this reaches the 52 m ceiling and seals each aisle. Never use a wall for a low thing (it would be solid 10 m above): low things are posts.
- **The Ravine's finish sphinx is always a solid** (`Finish.sphinx`, `SPHINX_SIZE` 16 × 19 × 34 m): the store draws a stacked water-case pallet display of exactly that box where it stands.
- **Only clear obstacles are solid:** shelving, freezers, walls, counters, columns, tables, bins, coffin freezers. Products, signs, banners, balloons, misters, carts and the shoppers' bodies are dressing.
- **Shoppers move as a pure function of the tick:** a shopper is a Ravine lizard (its bouncer moves with it, eased at the ends) and its balloon string a sliding vine (`Vine.slide`) whose phase matches the lizard's (`balloon_phase`), so the drawn person, the bouncer and the vine's anchor stay together.
- **Vines at least 4 m below the ceiling's underside**, never inside a gondola's solid; the balloon vines' walks stay clear of every wall.
- **Start:** no solid within 16 m behind `path[0]`; the start platform drawn where it is solid (TRIAGE T108: map_checks `Start`).
- **Finish:** the line must catch an ape anywhere across the front-end run (map_checks `Finish`).
- **Cost:** 94 product models on 26 shelf faces is far over 5,500 shapes if each facing is drawn whole; products are drawn in blocks, with detail only near the eye. Measure with `frame_cost` early.
- **Other maps unchanged:** new placement uses its own dice (`DICE`), and nothing shared changes but by addition.
