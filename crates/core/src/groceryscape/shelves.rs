//! The store's shelving: the gondolas and freezer runs drawn from their
//! walls (`RavineLayout.walls[g.wall]`, the same outline as their solid),
//! their shelves and the products on them, the end caps, the top stock,
//! and the stock carts closing the aisles off the route.
//!
//! **The planogram** (the brief: products "repeated / ordered to mimic the
//! real inside of a grocery store"): each shelf of a face takes its aisle's
//! products that fit under the shelf above, in turn from a start that
//! differs by shelf and face, each as a block of two to four facings
//! (about 40 cm real), stacked two or three high where short ones fit.
//! `blocks` walks it; the still scenery draws each shelf as bands of its
//! blocks' colours, and every frame `near` draws the products themselves
//! near the eye (each facing within `FULL_REACH`, each block as one shape
//! within `BLOCK_REACH`), in front of the bands.

use super::course::{Cells, LIT};
use crate::fx::{cm, int, ONE};
use crate::geom::{right_of, v3, yaw_of, V3};
use crate::grocery::{real_cm, GroceryLayout, Run, BLOCKER_TOP, SHELF_TOP, STOCK_TOP};
use crate::grocerymodels::{self as m, Detail, Product, AISLES};
use crate::mesh::Mesh;
use crate::ravine::RavineLayout;
use crate::render::{mix, placed, rgb, Batch, Rgb};

const GONDOLA: Rgb = rgb(217, 211, 195);
const GONDOLA_DARK: Rgb = rgb(190, 184, 168);
const SHELF_EDGE: Rgb = rgb(236, 232, 220);
const PRICE_STRIP: Rgb = rgb(246, 244, 236);
const PRICE_TAG: Rgb = rgb(243, 210, 43);
const PEGBOARD: Rgb = rgb(200, 192, 176);
const KRAFT: Rgb = rgb(184, 149, 106);
const KRAFT_DARK: Rgb = rgb(160, 126, 86);
const FREEZER_FRAME: Rgb = rgb(226, 220, 203);
const FREEZER_INSIDE: Rgb = rgb(206, 226, 236);
const GLASS: Rgb = rgb(222, 238, 246);
const HANDLE: Rgb = rgb(28, 27, 28);
const HEADER_BOARD: Rgb = rgb(42, 38, 38);
const WET_FLOOR: Rgb = rgb(250, 210, 30);

/// The shelves' tops over the floor, real centimetres (a 1.92 m gondola):
/// the bottom shelf has the most room (the big packs), the top the least.
pub const LEVELS: [i32; 5] = [6, 50, 88, 126, 160];
/// The gondola's top, real centimetres (`SHELF_TOP`).
const TOP_CM: i32 = 192;
/// A shelf board's thickness and depth (game metres): it runs from the
/// face back to the pegboard in the middle.
const BOARD: i32 = cm(36);
/// Products stand this far back from the face (the price strip in front).
const SET_BACK: i32 = cm(25);
/// The far bands of colour stand this far back, behind the near products.
const BAND_BACK: i32 = cm(160);
/// Each facing is drawn within this of the eye, each block as one shape
/// within `BLOCK_REACH`, and beyond that the still bands show.
pub const FULL_REACH: i32 = int(20);
pub const BLOCK_REACH: i32 = int(80);

/// One straight stretch of a run's outline, drawn as shelving facing out
/// of it: from `from` to `to` on the floor, `out` away from the run, the
/// aisle it sells (`AISLES`), what the run is, whether it is an end, and a
/// number for the planogram's start.
#[derive(Debug, Clone, Copy)]
pub struct Face {
    pub from: V3,
    pub to: V3,
    pub out: V3,
    pub aisle: u8,
    pub run: Run,
    pub end: bool,
    pub id: u32,
}

impl Face {
    pub fn len(&self) -> i32 {
        self.to.sub(self.from).flat().len()
    }

    pub fn along(&self) -> V3 {
        self.to.sub(self.from).flat().norm()
    }

    /// The yaw a product on it faces (out).
    pub fn yaw(&self) -> i32 {
        yaw_of(self.out)
    }
}

/// Every stretch of every run's outline, as faces. An end cap sells the
/// aisle of the face nearer it (the west face's, at the front).
pub fn faces(r: &RavineLayout, g: &GroceryLayout) -> Vec<Face> {
    let mut out = Vec::new();
    for (k, run) in g.gondolas.iter().enumerate() {
        let Some(w) = r.walls.get(run.wall) else {
            continue;
        };
        // the loop: west face front to back (0, 1), the back end (2), the
        // east face back to front (3, 4), the front end (5)
        for (e, q) in w.points.windows(2).enumerate() {
            let (a, b) = (q[0].at, q[1].at);
            let run_v = b.sub(a).flat();
            if run_v.len() <= 0 {
                continue;
            }
            // (the rock is on the right going round: out is the left)
            let outward = right_of(run_v.norm()).neg();
            let end = e == 2 || e == 5;
            let aisle = if e < 2 || e == 5 {
                run.faces[0]
            } else {
                run.faces[1]
            };
            out.push(Face {
                from: v3(a.x, 0, a.z),
                to: v3(b.x, 0, b.z),
                out: outward,
                aisle,
                run: run.run,
                end,
                id: (k * 8 + e) as u32,
            });
        }
    }
    out
}

/// One block of the planogram: `facings` of product `p` side by side,
/// `stack` high, starting `at` real centimetres along the face.
#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub p: &'static Product,
    pub at: i32,
    pub facings: i32,
    pub stack: i32,
}

impl Block {
    /// Its width along the face, real centimetres (a centimetre between
    /// facings).
    pub fn width(&self) -> i32 {
        self.facings * (self.p.size[0] + 1)
    }
}

/// The room under shelf `level`'s next one, real centimetres.
pub fn gap(level: usize) -> i32 {
    let next = LEVELS.get(level + 1).copied().unwrap_or(TOP_CM);
    next - LEVELS[level] - 5
}

/// Walks the planogram of shelf `level` on face `f` from its start to
/// `len_cm`, block by block.
pub fn blocks(f: &Face, level: usize, len_cm: i32, mut each: impl FnMut(Block)) {
    let Some(aisle) = AISLES.get(f.aisle as usize) else {
        return;
    };
    let room = gap(level);
    let fits: Vec<&'static Product> = aisle
        .products
        .iter()
        .filter(|p| p.size[1] + 2 <= room)
        .collect();
    // (nothing fits: the shortest product, laid in anyway)
    let fits = if fits.is_empty() {
        aisle
            .products
            .iter()
            .min_by_key(|p| p.size[1])
            .into_iter()
            .collect()
    } else {
        fits
    };
    let n = fits.len();
    let mut j = (level * 3 + f.id as usize) % n;
    let mut at = 2;
    while at < len_cm {
        let p = fits[j];
        let facings = ((40 + p.size[0] / 2) / (p.size[0] + 1)).clamp(2, 4);
        let stack = (room / (p.size[1] + 1)).clamp(1, if p.size[1] < 12 { 3 } else { 2 });
        let b = Block {
            p,
            at,
            facings,
            stack,
        };
        if at + b.width() > len_cm {
            break;
        }
        at += b.width() + 2;
        each(b);
        j = (j + 1) % n;
    }
}

/// The still shelving: every run's faces and ends, its top stock; the
/// closed aisles' stock carts.
pub fn scenery(cells: &mut Cells, r: &RavineLayout, g: &GroceryLayout) {
    for f in faces(r, g) {
        match (f.run, f.end) {
            (Run::Shelves, false) => shelf_face(cells, &f),
            (Run::Freezer, false) => freezer_face(cells, &f),
            (_, true) => end_cap(cells, &f),
        }
    }
    for run in &g.gondolas {
        top_stock(cells, run, r);
    }
    for (k, b) in g.blockers.iter().enumerate() {
        blocker(cells, b, k as u32);
    }
}

/// A point `along` metres down face `f`, `back` metres in from its face,
/// `up` over the floor.
fn on(f: &Face, along: i32, back: i32, up: i32) -> V3 {
    f.from
        .add(f.along().scale(along))
        .sub(f.out.scale(back))
        .add(v3(0, up, 0))
}

/// The height of shelf `level`'s top, game metres.
fn level_y(level: usize) -> i32 {
    real_cm(LEVELS[level])
}

/// A face of shelving: the kick plate, the uprights, the shelves with
/// their price strips, the pegboard behind, and each shelf's products as
/// bands of colour (the near ones are drawn every frame, `near`).
fn shelf_face(cells: &mut Cells, f: &Face) {
    let len = f.len();
    let yaw = yaw_of(f.along());
    let mid = |back: i32, up: i32| on(f, len / 2, back, up);
    // the kick plate under the bottom shelf
    let kick = level_y(0);
    cells.block(
        mid(cm(150), kick / 2),
        yaw,
        v3(int(3), kick, len),
        GONDOLA_DARK,
    );
    // the pegboard back, to the shelving's top
    cells.block(
        mid(int(5) - cm(30), SHELF_TOP / 2),
        yaw,
        v3(cm(60), SHELF_TOP, len),
        PEGBOARD,
    );
    // the uprights every 8 ft real (2.44 m, 29 m)
    let step = real_cm(244);
    let mut a = 0;
    while a <= len {
        cells.block(
            on(f, a.min(len - cm(30)).max(cm(30)), cm(220), SHELF_TOP / 2),
            yaw,
            v3(int(4) + cm(40), SHELF_TOP, cm(60)),
            GONDOLA,
        );
        a += step;
    }
    // the shelves: a board, its price strip; the top of the gondola
    for level in 1..LEVELS.len() {
        let y = level_y(level);
        cells.block(
            mid(int(2) + cm(20), y - BOARD / 2),
            yaw,
            v3(int(4) + cm(40), BOARD, len),
            SHELF_EDGE,
        );
        cells.block(
            mid(cm(5), y - cm(30)),
            yaw,
            v3(cm(10), cm(60), len),
            PRICE_STRIP,
        );
    }
    cells.block(
        mid(int(2) + cm(20), SHELF_TOP - cm(20)),
        yaw,
        v3(int(4) + cm(40), cm(40), len),
        GONDOLA,
    );
    bands(cells, f, len, yaw);
}

/// Each shelf's products as bands of their blocks' colours, set back
/// behind where the near products stand.
fn bands(cells: &mut Cells, f: &Face, len: i32, yaw: i32) {
    let len_cm = len * 100 / real_cm(100);
    for level in 0..LEVELS.len() {
        let base = level_y(level);
        // runs of blocks merged into bands of about 34 m (each its first
        // block's colour, the next a shade off)
        let mut run_from = -1;
        let mut run_colour = [0; 3];
        let mut run_h = 0;
        let mut last_end = 0;
        let flush = |cells: &mut Cells, from: i32, to: i32, c: Rgb, h: i32| {
            if from < 0 || to <= from {
                return;
            }
            let a = real_cm(from);
            let b = real_cm(to);
            cells.block(
                on(f, (a + b) / 2, BAND_BACK, base + h / 2),
                yaw,
                v3(cm(260), h, b - a),
                c,
            );
        };
        blocks(f, level, len_cm, |b| {
            if run_from >= 0 && b.at - run_from > 280 {
                flush(cells, run_from, last_end, run_colour, run_h);
                run_from = -1;
            }
            if run_from < 0 {
                run_from = b.at;
                run_colour = mix(b.p.colours[0], b.p.colours[1], ONE / 4);
                run_h = real_cm((b.p.size[1] * b.stack).min(gap(level)));
            }
            last_end = b.at + b.width();
        });
        flush(cells, run_from, last_end, run_colour, run_h);
    }
}

/// A face of glass-door freezers: the frame, a door every 9 m with its
/// handle and a pale glass streak, the cold-lit inside, its shelves and
/// the products' bands behind the doors.
fn freezer_face(cells: &mut Cells, f: &Face) {
    let len = f.len();
    let yaw = yaw_of(f.along());
    let mid = |back: i32, up: i32| on(f, len / 2, back, up);
    let top = SHELF_TOP + int(2);
    // the inside, lit cold
    cells.push(
        Mesh::Cube,
        &placed(mid(int(4), top / 2), yaw, v3(cm(40), top, len)),
        FREEZER_INSIDE,
        ONE * 3 / 4,
    );
    // the kick, the header with its light, the frame's top rail
    cells.block(
        mid(cm(40), real_cm(14) / 2),
        yaw,
        v3(cm(80), real_cm(14), len),
        FREEZER_FRAME,
    );
    cells.block(
        mid(cm(40), top + int(1)),
        yaw,
        v3(cm(80), int(2), len),
        FREEZER_FRAME,
    );
    cells.push(
        Mesh::Cube,
        &placed(mid(cm(10), top - cm(30)), yaw, v3(cm(20), cm(30), len)),
        GLASS,
        ONE,
    );
    for level in 1..LEVELS.len() {
        let y = level_y(level);
        cells.block(
            mid(int(2), y - cm(18)),
            yaw,
            v3(int(3), cm(36), len),
            FREEZER_INSIDE,
        );
    }
    // the doors: a mullion, a handle, a streak of glass each
    let step = int(9);
    let mut a = 0;
    let mut k = 0;
    while a < len {
        cells.block(
            on(f, a, cm(30), top / 2),
            yaw,
            v3(cm(60), top, cm(70)),
            FREEZER_FRAME,
        );
        let h = if k % 2 == 0 {
            a + step - cm(120)
        } else {
            a + cm(120)
        };
        if h < len {
            cells.push(
                Mesh::Cube,
                &placed(on(f, h, -cm(30), top / 2), yaw, v3(cm(30), top / 3, cm(30))),
                HANDLE,
                0,
            );
        }
        let streak = a + step / 3;
        if streak < len && k % 2 == 0 {
            cells.push(
                Mesh::Cube,
                &placed(
                    on(f, streak, -cm(5), top * 2 / 3),
                    yaw + int(0) + crate::fx::deg(25),
                    v3(cm(10), top / 2, cm(50)),
                ),
                GLASS,
                ONE * 3 / 5,
            );
        }
        a += step;
        k += 1;
    }
    bands(cells, f, len, yaw);
}

/// An end cap: shelving across the run's end, one product a shelf (each
/// facing as one shape), a dark category board over it.
fn end_cap(cells: &mut Cells, f: &Face) {
    let len = f.len();
    if len < int(4) {
        return;
    }
    let yaw = yaw_of(f.along());
    let mid = |back: i32, up: i32| on(f, len / 2, back, up);
    cells.block(
        mid(int(2), SHELF_TOP / 2),
        yaw,
        v3(int(4), SHELF_TOP, len),
        GONDOLA,
    );
    let Some(aisle) = AISLES.get(f.aisle as usize) else {
        return;
    };
    let n = aisle.products.len();
    for level in 0..LEVELS.len() {
        let y = level_y(level);
        if level > 0 {
            cells.block(
                mid(cm(10), y - BOARD / 2),
                yaw,
                v3(cm(60), BOARD, len),
                SHELF_EDGE,
            );
        }
        let p = &aisle.products[(level + f.id as usize) % n];
        if p.size[1] + 2 > gap(level) {
            continue;
        }
        // the shelf's facings as one block of the product's colour, its
        // front's label colour as a band across it
        let s = p.size_m();
        let stack = (gap(level) / (p.size[1] + 1)).clamp(1, 2);
        let h = s.y * stack;
        cells.block(
            mid(SET_BACK + s.z / 2, y + h / 2),
            yaw,
            v3(s.z, h, len - cm(80)),
            p.colours[0],
        );
        cells.block(
            mid(SET_BACK - cm(5), y + h * 3 / 5),
            yaw,
            v3(cm(10), h / 4, len - cm(80)),
            p.colours[1],
        );
    }
    // the category board over the end cap
    cells.push(
        Mesh::Cube,
        &placed(
            mid(int(1), SHELF_TOP + int(2)),
            yaw,
            v3(cm(40), int(3), len * 3 / 4),
        ),
        HEADER_BOARD,
        LIT,
    );
}

/// The top stock on a run: stacks of shipping cases to `STOCK_TOP`, the
/// run's full width, a printed band on each.
fn top_stock(cells: &mut Cells, run: &crate::grocery::Gondola, r: &RavineLayout) {
    let Some(w) = r.walls.get(run.wall) else {
        return;
    };
    let pts: Vec<V3> = w.points.iter().map(|q| q.at).collect();
    if pts.len() < 6 {
        return;
    }
    let width = run.half * 2 - cm(40);
    let h = STOCK_TOP - SHELF_TOP;
    // along the middle line, between the outline's ends (the cut ends'
    // middles), in stacks of about 14 m
    let ends = [pts[0].lerp(pts[5], ONE / 2), pts[2].lerp(pts[3], ONE / 2)];
    let mid = run.line[1];
    let mut k = 0u32;
    for (a, b) in [(ends[0], mid), (mid, ends[1])] {
        let d = b.sub(a).flat();
        let len = d.len();
        let n = (len / int(14)).max(1);
        let yaw = yaw_of(d);
        for j in 0..n {
            let t0 = (j as i64 * ONE as i64 / n as i64) as i32;
            let t1 = ((j + 1) as i64 * ONE as i64 / n as i64) as i32;
            let c = a.lerp(b, (t0 + t1) / 2);
            let shade = if (k * 7 + run.wall as u32).is_multiple_of(3) {
                KRAFT_DARK
            } else {
                KRAFT
            };
            let l = len / n - cm(30);
            cells.block(v3(c.x, SHELF_TOP + h / 2, c.z), yaw, v3(width, h, l), shade);
            // a printed band on its side, in one of the aisle's colours
            let ink = AISLES
                .get(run.faces[(k % 2) as usize] as usize)
                .and_then(|a| a.products.get(k as usize % a.products.len()))
                .map_or(KRAFT_DARK, |p| p.colours[0]);
            cells.block(
                v3(c.x, SHELF_TOP + h * (1 + (k % 3) as i32) / 5, c.z),
                yaw,
                v3(width + cm(20), int(2), l - int(1)),
                ink,
            );
            k += 1;
        }
    }
}

/// A closed aisle's mouth: a stock cart loaded with cases, a yellow
/// wet-floor sign beside it (a clean-up on this aisle).
fn blocker(cells: &mut Cells, b: &crate::grocery::Blocker, k: u32) {
    let d = b.to.sub(b.from).flat();
    let len = d.len();
    let mid = b.from.lerp(b.to, ONE / 2).sub(b.out.scale(int(3)));
    let yaw = yaw_of(d);
    let at = cells.at(mid);
    m::stock_cart(at, mid, yaw, v3(int(6), BLOCKER_TOP, len - int(2)));
    // the sign: an A-frame of two yellow boards, in front of the cart
    let s = b
        .from
        .lerp(
            b.to,
            if k.is_multiple_of(2) {
                ONE / 4
            } else {
                ONE * 3 / 4
            },
        )
        .add(b.out.scale(int(4)));
    for side in [-1, 1] {
        cells.push(
            Mesh::Cube,
            &placed(
                s.add(b.out.scale(side * cm(60))).add(v3(0, int(3), 0)),
                yaw_of(b.out) + side * crate::fx::deg(12),
                v3(int(3), int(6), cm(20)),
            ),
            WET_FLOOR,
            LIT,
        );
    }
}

/// Every frame: the products near the eye on the faces it is in front of
/// (each facing within `FULL_REACH`, each block as one shape within
/// `BLOCK_REACH`), and their price tags.
pub fn near(b: &mut Batch, r: &RavineLayout, g: &GroceryLayout, eye: V3) {
    for f in faces(r, g) {
        if f.end {
            continue;
        }
        let len = f.len();
        let rel = eye.sub(f.from).flat();
        let front = rel.dot(f.out);
        // (behind the face, or far off it: nothing of it shows near)
        if !(-int(2)..=BLOCK_REACH).contains(&front) {
            continue;
        }
        let along = rel.dot(f.along());
        if along < -BLOCK_REACH || along > len + BLOCK_REACH {
            continue;
        }
        let len_cm = len * 100 / real_cm(100);
        for level in 0..LEVELS.len() {
            let y = level_y(level);
            blocks(&f, level, len_cm, |blk| {
                let a0 = real_cm(blk.at);
                let w = real_cm(blk.width());
                let centre = a0 + w / 2;
                let off = (centre - along).abs() - w / 2;
                let dist = crate::geom::v3(off.max(0), y - eye.y, front.max(0)).len();
                if dist > BLOCK_REACH {
                    return;
                }
                let s = blk.p.size_m();
                if dist > FULL_REACH {
                    // the block as one shape
                    let foot = on(&f, centre, SET_BACK, y);
                    let size = v3(w, s.y * blk.stack, s.z);
                    let mid = foot.add(v3(0, size.y / 2, 0)).sub(f.out.scale(size.z / 2));
                    b.push(
                        Mesh::Cube,
                        &placed(mid, f.yaw(), size),
                        blk.p.colours[0],
                        LIT,
                    );
                    return;
                }
                for k in 0..blk.facings {
                    for h in 0..blk.stack {
                        let a = a0 + k * real_cm(blk.p.size[0] + 1) + s.x / 2;
                        let foot = on(&f, a, SET_BACK, y + h * (s.y + real_cm(1)));
                        m::product(b, blk.p, foot, f.yaw(), Detail::Full);
                    }
                }
                // its price tag on the strip
                b.push(
                    Mesh::Cube,
                    &placed(
                        on(&f, centre, -cm(4), y - cm(30)),
                        f.yaw(),
                        v3(real_cm(6), cm(45), cm(6)),
                    ),
                    PRICE_TAG,
                    LIT,
                );
            });
        }
    }
}
