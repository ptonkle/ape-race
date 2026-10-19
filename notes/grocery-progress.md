# Clean-Up on Aisle Ape: build progress

The standard map-build steps (MAPS.md "Build steps"), one row each. Status is one of: **Done**, **In progress**, **Not started**, **Waiting on you**, **Blocked**. Branch `grocery`, from upstream `main` d6d4106 (1.15.0); held until Peter releases it. The person's requests are numbered here, `GR-T1` …; one `TRIAGE.md` row for the whole map is added just before the PR.

| # | Step | Status | Notes |
| --- | --- | --- | --- |
| 1 | Brief and answers | Done (defaults) | `briefs/clean-up-on-aisle-ape.md`; `notes/grocery-pitfalls.md`; decisions GR-Q1–Q12 put to the person with defaults, building on the defaults |
| 2 | Skeleton and contract | Done | `Map::Grocery` held; `GroceryLayout` (`grocery.rs`), the store's solids as a Ravine layout; the models' contract and the 94-product catalogue; GR1–GR5 (branch `grocery-skeleton` marks it) |
| 3 | Ape costumes | Done (second pass: store uniforms, GR-T1) | first `Gear::Food` (`models/food.rs`): animal cracker, sandwich cookie, fruit snack, soft pretzel, cheese puff, frosted circus cookie, 4 flavours each; costume Food; `GUMMY_JAGUAR`; shots `notes/grocery-shots/costumes/`; GR-G rows |
| 4 | Models | Done | `grocerymodels/`: 12 product shapes × 8 fronts, shoppers (7 looks, walking, balloon hand, carts), balloons, ball bin, tables, bins, mister, aisle sign, banner, department signs, cart, stock cart, pallet; 32 gallery keys; shots `notes/grocery-shots/models/`; GR-M rows |
| 5 | Sky and lighting | Done | `groceryscape.rs`: strip-light white from overhead, cream bounce, a warm haze (90–250 m); the frozen aisle colder and bluer by place; every store surface carries some glow (`LIT`) |
| 6 | Course layout | Done | 1,854 m through three aisles; radius-36 turns, angled run ends, closed side aisles, the glass storefront; GR3, GR6 |
| 7 | Map rules | Done | no new rule: the Ravine's walls, roof, posts and moving bouncers; shoppers' heads bounce (`a_shopper_s_head_is_a_moving_bouncer`); balloon vines walk with their shoppers (`the_balloons_walk_with_their_shopper`); misters are dressing |
| 8 | Art placed on the course | Done (first pass) | `groceryscape/`: the store chunked by place; shelves with the planogram (bands far, products near), freezers, end caps, top stock, departments and soffits, checkouts, produce canopy, columns, signs, banners, stock carts across closed aisles; shots `notes/grocery-shots/course/` |
| 9 | Lobby spot and flyover | Done (first pass) | the frozen aisle's floor by the middle coffin freezer; the flyover glides along the route under the ceiling |
| 10 | AI validation | Done | 16 layouts: King Kong 16/16 (87.0 s), Ape 15/16 (105.7 s), Sloth 11/16 (131.8 s), as the Ravine's 5 Sloth timeouts |
| 11 | Medal times | Done | Author 1:15, Gold 1:22, Silver 1:32, Bronze 1:47 from the time trial (Orangutan 75.84 s best); GR8 |
| 12 | Performance test | Done | worst frame 4,691 still + 688 moving = 5,379 of 5,500 (`the_grocery_frame_stays_within_budget`); GR7 |
| 13 | Colourblind and JOJO MODE | Done | `notes/grocery-shots/polish/`: the vines vivid green on the muted store; no hazards on this map |
| 14 | Map card | Done | `data/maps/grocery.webp`: down the frozen aisle at 650 m |
| 15 | Screenshots at 1920×1080 | Done | on the design journal |
| 16 | Other maps unchanged | Done | the three unchanged-race tests pass in the full check |
| 17 | Docs | Done | README (How to Play row, held; code table), maps-archive lesson, MAPS gotcha 17, HANDOFF; TRIAGE row added before the PR |
| 18 | Full checks | Done (first pass); rerun for the second | GR9 |
| 19 | Deploy | Waiting on Peter | held. At release: drop `Map::Grocery` from `Map::held`, bump `VERSION`, add the patch note, the README intro and catch-up lists. Patch-note draft: "New map: Clean-Up on Aisle Ape. Race as a snack through a supermarket: down the cereal aisle, up the frozen aisle, down the snacks, round the misting produce and out past the checkouts. Swing from licorice ropes and shoppers' balloons; the apes are an animal cracker, a sandwich cookie, a fruit snack, a soft pretzel, a cheese puff and a frosted cookie." |

## This map's features

| Feature | Source | Model | Gameplay | Placed | Tested |
| --- | --- | --- | --- | --- | --- |
| Store following the floor plan (bowed aisles, frozen lens, departments) | brief, plan | Done | Done | Done | Done |
| Route through three aisles, about 90 s | brief | n/a | Done | Done | Done |
| Shoppers walking the aisles (moving bouncers) | brief | Done | Done | Done | Done |
| Swinging from shoppers (balloon vines) | brief | Done | Done | Done | Done |
| Obstacles: gondolas, end caps, bins, columns, checkouts | brief | Done | Done | Done | Done |
| Misting produce displays | brief | Done | n/a | Done | Done |
| Frozen section (doors, cold light, coffin freezers) | brief | Done | Done | Done | Done |
| 5–10 product models per aisle, in planogram order | brief | Done | n/a | Done | Done |
| Lighting, palette and sizes from the photos | brief | n/a | n/a | Done | – |
| Food apes, small next to the people | brief | Done | n/a | n/a | Done |
| Gummy jaguar (Floor is Lava) | default GR-Q11 | Done | Done | n/a | Done |

## The person's requests

| # | Request (2026-10-18, after playing the first pass) | Status | Where |
| --- | --- | --- | --- |
| GR-T1 | The ape costumes as the store's workers' uniforms, a big banana in a circle as the logo | Done | `models/grocer.rs` (`Gear::Grocer`, six jobs, four palettes), `grocerymodels::banana_logo`; the food apes leave the map (the gummy jaguar stays) |
| GR-T2 | A shopping list of four items, one per zone, in side aisles, glowing; hit one to take it: 1st and 3rd +25% speed for good, 2nd and 4th +25% air control (added, the person's answer) | Done, with a divergence (GR11) | `grocery/list.rs`; items at four item aisles' mouths; HUD row "List"; the AI goes for its own (GR12) |
| GR-T3 | Each racer led to a different part of the same aisle, so all get theirs at about the same time | Done | `ITEM_SPOTS`: racer `k` at spot `k % 4`, all four across one mouth; only one's own glows |
| GR-T4 | At the end, out of the closed doors into the parking lot (about 110 s, the person's answer) | Done | the doors slide open as you come; the lot: canopy, cars, light poles, corrals, trees; 2,022 m |
| GR-T5 | With all four items, a plane flies by and picks up the order in the lot | Done | `groceryscape/lot.rs` `planes`: the ape holds up the bag, the plane snatches it on a hook |
| GR-T6 | More shoppers with carts as obstacles | Done | nine more walks (six in the store on the route, three in the lot) |
| GR-T7 | Swinging low to the ground in one place, then having to go up | Done | the frozen aisle's drop ceiling (ropes to 8–12 m over the floor), then the ice and its ramp (GR14) |
| GR-T8 | A few sideways and upside-down vines | Done | three ropes standing out level from sale signs along the front end; four balloon strings rising from the ice |
