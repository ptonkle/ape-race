//! Clean-Up on Aisle Ape's course art: the store drawn from its layout
//! (`GroceryLayout` and the solids' `RavineLayout`), so what is drawn is
//! what is solid (MAPS "Common gotchas" 1–2).
//!
//! STAND-IN (the skeleton): the floor, the ceiling, the store's walls, the
//! runs as plain blocks to their solid's height, the posts as cylinders,
//! the shoppers as simple figures. Step 8 replaces it.

use crate::course::Course;
use crate::fx::{cm, int, ONE};
use crate::geom::{right_of, v3, yaw_of, V3};
use crate::grocery::{GroceryLayout, PostKind, Run, CEILING_AT, SHOPPER_HEIGHT};
use crate::mesh::Mesh;
use crate::ravine::solids::{ROOF_THICK, WALL_ABOVE};
use crate::ravine::RavineLayout;
use crate::render::{placed, rgb, rod, Batch, Rgb};
use crate::scene::{assemble, chunk_count, chunk_of, Scenery};

const FLOOR: Rgb = rgb(211, 206, 195);
const FLOOR_FLECK: Rgb = rgb(176, 170, 158);
const CEILING: Rgb = rgb(228, 226, 220);
const WALL: Rgb = rgb(231, 225, 210);
const SHELVING: Rgb = rgb(217, 211, 195);
const FREEZER: Rgb = rgb(168, 195, 207);
const COLUMN: Rgb = rgb(236, 234, 228);
const KRAFT: Rgb = rgb(184, 149, 106);
const WOOD: Rgb = rgb(75, 58, 45);
const COUNTER: Rgb = rgb(220, 216, 204);
const CHECKOUT_BLUE: Rgb = rgb(44, 94, 168);

/// The store's chunks are drawn while their middle is this near the eye.
pub const REACH: i32 = int(300);

/// Clean-Up on Aisle Ape's scenery, chunked.
pub fn scenery(c: &Course, g: &GroceryLayout) -> Scenery {
    let Some(r) = c.ravine.as_deref() else {
        return Scenery::default();
    };
    let n = chunk_count(c);
    let mut batches: Vec<Batch> = (0..n).map(|_| Batch::new()).collect();
    let chunk = |p: V3| chunk_of(c, p, n);
    floor_and_ceiling(&mut batches, &chunk, g);
    walls(&mut batches, &chunk, r, g);
    posts(&mut batches, &chunk, r, g);
    // the start's stock cart, loaded with cases to its platform's top
    for pl in &c.platforms {
        batches[chunk(pl.centre)].push(
            Mesh::Cube,
            &placed(
                v3(pl.centre.x, pl.top / 2, pl.centre.z),
                yaw_of(pl.fwd),
                v3(pl.half_width * 2, pl.top, pl.half_len * 2),
            ),
            KRAFT,
            0,
        );
    }
    Scenery {
        reach: REACH,
        ..assemble(c, batches, Vec::new(), Vec::new())
    }
}

/// The floor and the ceiling in tiles of 48 m, each in its own chunk.
fn floor_and_ceiling(batches: &mut [Batch], chunk: &dyn Fn(V3) -> usize, g: &GroceryLayout) {
    let (lo, hi) = g.bounds;
    let tile = int(48);
    let mut x = lo.x - int(12);
    while x < hi.x + int(12) {
        let mut z = lo.z - int(12);
        while z < hi.z + int(12) {
            let mid = v3(x + tile / 2, 0, z + tile / 2);
            let k = chunk(mid);
            let fleck = ((x / tile + z / tile) & 1) == 0;
            batches[k].push(
                Mesh::Cube,
                &placed(v3(mid.x, -cm(50), mid.z), 0, v3(tile, cm(100), tile)),
                if fleck { FLOOR } else { FLOOR_FLECK },
                0,
            );
            batches[k].push(
                Mesh::Cube,
                &placed(
                    v3(mid.x, CEILING_AT + ROOF_THICK / 2, mid.z),
                    0,
                    v3(tile, ROOF_THICK, tile),
                ),
                CEILING,
                0,
            );
            z += tile;
        }
        x += tile;
    }
}

/// Every wall stretch as a block to its solid's height (the runs to their
/// top stock plus `WALL_ABOVE`, so the drawing and the solid agree).
fn walls(batches: &mut [Batch], chunk: &dyn Fn(V3) -> usize, r: &RavineLayout, g: &GroceryLayout) {
    for (k, w) in r.walls.iter().enumerate() {
        let run = g.gondolas.iter().find(|q| q.wall == k).map(|q| q.run);
        let colour = match run {
            Some(Run::Shelves) => SHELVING,
            Some(Run::Freezer) => FREEZER,
            None => WALL,
        };
        for s in w.points.windows(2) {
            let (a, b) = (s[0], s[1]);
            let run_v = b.at.sub(a.at).flat();
            let len = run_v.len();
            if len <= 0 {
                continue;
            }
            let fwd = run_v.norm();
            let into = right_of(fwd);
            let thick = w.depth;
            let top = a.height.max(b.height) + if run.is_some() { 0 } else { WALL_ABOVE };
            let mid = a.at.lerp(b.at, ONE / 2).add(into.scale(thick / 2));
            batches[chunk(mid)].push(
                Mesh::Cube,
                &placed(v3(mid.x, top / 2, mid.z), yaw_of(fwd), v3(thick, top, len)),
                colour,
                0,
            );
        }
    }
}

fn posts(batches: &mut [Batch], chunk: &dyn Fn(V3) -> usize, r: &RavineLayout, g: &GroceryLayout) {
    for p in &g.posts {
        let Some(k) = r.rocks.get(p.rock) else {
            continue;
        };
        let colour = match p.kind {
            PostKind::Column => COLUMN,
            PostKind::Table => WOOD,
            PostKind::Bin => KRAFT,
            PostKind::Coffin => FREEZER,
            PostKind::Counter => COUNTER,
        };
        batches[chunk(k.at)].push(
            Mesh::Cyl,
            &placed(
                k.at.add(v3(0, k.height / 2, 0)),
                0,
                v3(k.radius * 2, k.height, k.radius * 2),
            ),
            colour,
            0,
        );
    }
    let _ = CHECKOUT_BLUE;
}

/// The moving things near the eye: the shoppers, their balloons.
pub fn dynamic(
    b: &mut Batch,
    c: &Course,
    g: &GroceryLayout,
    eye: V3,
    _t: u32,
    tick: u32,
    _vivid: bool,
) {
    let Some(r) = c.ravine.as_deref() else {
        return;
    };
    for s in &g.shoppers {
        let (feet, way) = r.lizard_at(s.lizard, tick);
        if feet.flat().sub(eye.flat()).longer_than(int(260)) {
            continue;
        }
        let yaw = yaw_of(way);
        let h = SHOPPER_HEIGHT;
        b.push(
            Mesh::Cyl,
            &placed(
                feet.add(v3(0, h * 9 / 40, 0)),
                yaw,
                v3(int(4), h * 9 / 20, int(3)),
            ),
            rgb(58, 74, 106),
            0,
        );
        b.push(
            Mesh::Barrel,
            &placed(
                feet.add(v3(0, h * 13 / 20, 0)),
                yaw,
                v3(int(6), h * 2 / 5, int(4)),
            ),
            rgb(200, 79, 58),
            0,
        );
        b.push(
            Mesh::Sphere,
            &placed(
                feet.add(v3(0, h - int(2), 0)),
                yaw,
                v3(int(3), int(4), int(3)),
            ),
            rgb(226, 180, 142),
            0,
        );
        if let Some(i) = s.balloons {
            let a = c.anchor(i, tick);
            let hand = c.tip(i, tick);
            b.push(
                Mesh::Cyl,
                &rod(hand, a, cm(30)),
                crate::groceryscape::VINE,
                0,
            );
            for (k, col) in [rgb(232, 70, 58), rgb(242, 194, 46), rgb(61, 122, 214)]
                .into_iter()
                .enumerate()
            {
                let o = v3((k as i32 - 1) * int(2), int(1) + (k as i32 % 2) * int(2), 0);
                b.push(
                    Mesh::Sphere,
                    &placed(a.add(o), 0, v3(int(4), int(5), int(4))),
                    col,
                    0,
                );
            }
        }
    }
}
