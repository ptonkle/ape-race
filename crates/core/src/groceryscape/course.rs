//! Clean-Up on Aisle Ape's course art (step 8): the store drawn from its
//! layout (`GroceryLayout`, and the solids' `RavineLayout`), so what is
//! drawn is what is solid (MAPS "Common gotchas" 1–2).
//!
//! - **The store** is chunked by place, not along the route: 100 m squares
//!   of the floor plan, each knowing its bounds (`Scenery::bound_chunks`),
//!   since the route doubles back past the same shelves.
//! - **The light** comes from the ceiling: rows of strip lights run north
//!   and south over the store; every surface carries a share of glow
//!   (`LIT`), as a store lit evenly from everywhere is, so the shelving's
//!   sides never go dark under a light from straight above.
//! - **The shelves** (`shelves.rs`): gondolas, freezer runs, end caps, top
//!   stock and the closed aisles' stock carts; far products as bands of
//!   colour per shelf, near ones each drawn (`shelves::near`, every frame).
//! - **The walls** (`walls.rs`): departments' cases and soffits, signs, the
//!   stockroom doors, the front windows and the glass storefront.
//! - **The fixtures** (`fixtures.rs`): checkouts, produce, columns, bins,
//!   the start's stock cart, the pallet, hanging signs and banners.
//! - **What moves** (`movers.rs`): shoppers, their balloons and carts, the
//!   misters.

use crate::course::Course;
use crate::fx::{cm, int, ONE};
use crate::geom::{v3, V3};
use crate::grocery::{plan, GroceryLayout, CEILING_AT};
use crate::mesh::Mesh;
use crate::render::{placed, rgb, Batch, Rgb};
use crate::scene::{Chunk, Scenery};

/// The share of glow every store surface carries (the strip lights reach
/// every face).
pub const LIT: i32 = ONE * 2 / 5;
/// A chunk is drawn while any of it is this near the eye (the fog's far
/// end, `groceryscape::look`, is nearer still in the aisles).
pub const REACH: i32 = int(200);
/// The store's chunks: squares this wide.
const CELL: i32 = int(100);

pub const FLOOR: Rgb = rgb(212, 207, 196);
pub const FLOOR_B: Rgb = rgb(222, 218, 208);
pub const FLOOR_SEAM: Rgb = rgb(172, 166, 154);
/// The strip lights' sheen on the glossy floor tiles under each row.
pub const SHEEN: Rgb = rgb(240, 238, 230);
pub const CEILING: Rgb = rgb(228, 226, 220);
pub const CEILING_GRID: Rgb = rgb(196, 194, 188);
pub const STRIP: Rgb = rgb(255, 255, 250);

/// The store's still things, by place: one batch per 100 m square.
pub struct Cells {
    lo: V3,
    nx: i32,
    nz: i32,
    pub batches: Vec<Batch>,
}

impl Cells {
    fn new(lo: V3, hi: V3) -> Cells {
        let nx = (hi.x - lo.x) / CELL + 1;
        let nz = (hi.z - lo.z) / CELL + 1;
        Cells {
            lo,
            nx,
            nz,
            batches: (0..nx * nz).map(|_| Batch::new()).collect(),
        }
    }

    /// The batch for a shape at `p`.
    pub fn at(&mut self, p: V3) -> &mut Batch {
        let cx = ((p.x - self.lo.x) / CELL).clamp(0, self.nx - 1);
        let cz = ((p.z - self.lo.z) / CELL).clamp(0, self.nz - 1);
        &mut self.batches[(cz * self.nx + cx) as usize]
    }

    /// Pushes one shape into the batch for its middle.
    pub fn push(&mut self, mesh: Mesh, a: &crate::geom::Affine, c: Rgb, glow: i32) {
        self.at(a.t).push(mesh, a, c, glow);
    }

    /// A box from its middle, yaw and size, lit as the store is.
    pub fn block(&mut self, mid: V3, yaw: i32, size: V3, c: Rgb) {
        self.push(Mesh::Cube, &placed(mid, yaw, size), c, LIT);
    }
}

/// A store point in plan metres (Q16) at height `y`.
pub fn at(x: i32, z: i32, y: i32) -> V3 {
    plan(x, z).add(v3(0, y, 0))
}

/// Clean-Up on Aisle Ape's scenery, chunked by place.
pub fn scenery(c: &Course, g: &GroceryLayout) -> Scenery {
    let Some(r) = c.ravine.as_deref() else {
        return Scenery::default();
    };
    let (lo, hi) = g.bounds;
    let (lo, hi) = (
        v3(lo.x.min(hi.x), 0, lo.z.min(hi.z)).sub(v3(int(60), 0, int(60))),
        v3(lo.x.max(hi.x), 0, lo.z.max(hi.z)).add(v3(int(60), 0, int(60))),
    );
    let mut cells = Cells::new(lo, hi);
    floor_and_ceiling(&mut cells);
    super::shelves::scenery(&mut cells, r, g);
    super::walls::scenery(&mut cells, c, r, g);
    super::fixtures::scenery(&mut cells, c, r, g);
    let mut values = Vec::new();
    let mut chunks = Vec::new();
    let nx = cells.nx;
    for (k, b) in cells.batches.into_iter().enumerate() {
        let (cx, cz) = (k as i32 % nx, k as i32 / nx);
        let centre = v3(lo.x + cx * CELL + CELL / 2, 0, lo.z + cz * CELL + CELL / 2);
        let draws = b.finish_into(crate::render::STATIC_BUFFER, &mut values);
        chunks.push(Chunk {
            centre,
            draws,
            bounds: None,
            heights: None,
            side: None,
        });
    }
    let mut s = Scenery {
        values,
        terrain: Vec::new(),
        chunks,
        reach: REACH,
    };
    s.bound_chunks();
    s
}

/// The floor's tiles (two tones, seams, the lights' sheen under each row)
/// and the ceiling (tiles, the grid, the rows of strip lights).
fn floor_and_ceiling(cells: &mut Cells) {
    use crate::grocery::layout::{EAST_WALL, NORTH_WALL, SOUTH, WEST};
    let (x0, x1) = (WEST - 14, EAST_WALL + 14);
    let (z0, z1) = (SOUTH - 6, NORTH_WALL + 14);
    // the floor in 50 m slabs, a little lighter and darker by turns (the
    // vinyl tiles' batches), seams every 25 m both ways
    let mut x = x0;
    while x < x1 {
        let mut z = z0;
        while z < z1 {
            let w = (x1 - x).min(50);
            let d = (z1 - z).min(50);
            let tone = if ((x / 50) + (z / 50)) % 2 == 0 {
                FLOOR
            } else {
                FLOOR_B
            };
            cells.push(
                Mesh::Cube,
                &placed(
                    at(int(x) + int(w) / 2, int(z) + int(d) / 2, -cm(50)),
                    0,
                    v3(int(d), cm(100), int(w)),
                ),
                tone,
                LIT,
            );
            z += 50;
        }
        x += 50;
    }
    for k in 0..=((x1 - x0) / 25) {
        let x = x0 + k * 25;
        cells.push(
            Mesh::Cube,
            &placed(
                at(int(x), int(z0 + z1) / 2, cm(1)),
                0,
                v3(int(z1 - z0), cm(4), cm(30)),
            ),
            FLOOR_SEAM,
            LIT,
        );
    }
    for k in 0..=((z1 - z0) / 25) {
        let z = z0 + k * 25;
        cells.push(
            Mesh::Cube,
            &placed(
                at(int(x0 + x1) / 2, int(z), cm(1)),
                0,
                v3(cm(30), cm(4), int(x1 - x0)),
            ),
            FLOOR_SEAM,
            LIT,
        );
    }
    // the ceiling: tiles (glowing softly: lit from the strips), the grid
    let thick = crate::ravine::solids::ROOF_THICK;
    let mut x = x0;
    while x < x1 {
        let mut z = z0;
        while z < z1 {
            let w = (x1 - x).min(100);
            let d = (z1 - z).min(100);
            cells.push(
                Mesh::Cube,
                &placed(
                    at(
                        int(x) + int(w) / 2,
                        int(z) + int(d) / 2,
                        CEILING_AT + thick / 2,
                    ),
                    0,
                    v3(int(d), thick, int(w)),
                ),
                CEILING,
                ONE * 3 / 5,
            );
            z += 100;
        }
        x += 100;
    }
    // the tile grid (every 2 ft real, 7.3 m: drawn every other line one
    // way, every 15 m the other)
    let mut k = 0;
    let mut x = x0;
    while x < x1 {
        if k % 2 == 0 {
            cells.push(
                Mesh::Cube,
                &placed(
                    at(int(x), int(z0 + z1) / 2, CEILING_AT - cm(5)),
                    0,
                    v3(int(z1 - z0), cm(10), cm(25)),
                ),
                CEILING_GRID,
                ONE / 2,
            );
        }
        x += 7;
        k += 1;
    }
    let mut z = z0;
    while z < z1 {
        cells.push(
            Mesh::Cube,
            &placed(
                at(int(x0 + x1) / 2, int(z), CEILING_AT - cm(5)),
                0,
                v3(cm(25), cm(10), int(x1 - x0)),
            ),
            CEILING_GRID,
            ONE / 2,
        );
        z += 15;
    }
    // the rows of strip lights, north to south over the store every 32 m,
    // and their sheen on the floor
    let mut x = x0 + 12;
    while x < x1 {
        let mut z = z0 + 10;
        while z < z1 - 10 {
            let len = (z1 - 10 - z).min(100);
            let mid = int(z) + int(len) / 2;
            cells.push(
                Mesh::Cube,
                &placed(
                    at(int(x), mid, CEILING_AT - cm(30)),
                    0,
                    v3(int(len), cm(50), cm(140)),
                ),
                STRIP,
                ONE,
            );
            if (x / 32) % 2 == 0 {
                cells.push(
                    Mesh::Cube,
                    &placed(at(int(x), mid, cm(3)), 0, v3(int(len), cm(2), int(3))),
                    SHEEN,
                    ONE * 3 / 4,
                );
            }
            z += len + 4;
        }
        x += 32;
    }
}

/// What moves, every frame: the shoppers and their balloons and carts, the
/// misters' spray, and the products near the eye.
pub fn dynamic(
    b: &mut Batch,
    c: &Course,
    g: &GroceryLayout,
    eye: V3,
    t: u32,
    tick: u32,
    vivid: bool,
) {
    let Some(r) = c.ravine.as_deref() else {
        return;
    };
    super::movers::dynamic(b, c, r, g, eye, t, tick, vivid);
    super::shelves::near(b, r, g, eye);
}
