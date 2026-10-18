//! The store's fixtures: the checkout lanes, the columns, the produce
//! tables and their wooden canopy, the display bins, the coffin freezers,
//! the bins of bouncy balls (the bounce bouncers), the hanging aisle
//! numbers and sale banners, the start's stock cart, the water pallet, and
//! the finish's chequered banner. Each solid one is drawn from its post
//! (`RavineLayout.rocks`) or its wall, the same numbers.

use super::course::{at, Cells, LIT};
use crate::course::Course;
use crate::fx::{cm, int, ONE};
use crate::geom::{right_of, v3, yaw_of, V3};
use crate::grocery::layout::{aisle_x, BACK, FRONT, RUNS};
use crate::grocery::{GroceryLayout, PostKind, EAST, NORTH};
use crate::grocerymodels as m;
use crate::mesh::Mesh;
use crate::ravine::RavineLayout;
use crate::render::{placed, rgb, Rgb};

const COLUMN: Rgb = rgb(236, 234, 228);
const COLUMN_BASE: Rgb = rgb(170, 166, 158);
const COUNTER: Rgb = rgb(220, 216, 204);
const CHECKOUT_BLUE: Rgb = rgb(44, 94, 168);
const BELT: Rgb = rgb(40, 40, 44);
const REGISTER: Rgb = rgb(60, 62, 70);
const SCREEN: Rgb = rgb(120, 200, 230);
const BAG: Rgb = rgb(70, 130, 210);
const COFFIN: Rgb = rgb(226, 220, 203);
const COFFIN_GLASS: Rgb = rgb(214, 232, 242);
const CANOPY: Rgb = rgb(123, 90, 63);
const TRACK: Rgb = rgb(30, 30, 32);
const BANNER: Rgb = rgb(240, 236, 220);
const BANNER_DARK: Rgb = rgb(40, 40, 46);
const FROZEN: [Rgb; 5] = [
    rgb(60, 120, 210),
    rgb(240, 200, 60),
    rgb(200, 40, 40),
    rgb(80, 170, 80),
    rgb(246, 237, 212),
];

pub fn scenery(cells: &mut Cells, c: &Course, r: &RavineLayout, g: &GroceryLayout) {
    posts(cells, r, g);
    coffins(cells, r, g);
    checkouts(cells, g);
    canopy(cells);
    hanging_signs(cells);
    // the start's stock cart (its top is the start platform)
    for pl in &c.platforms {
        let at = cells.at(pl.centre);
        m::stock_cart(
            at,
            v3(pl.centre.x, 0, pl.centre.z),
            yaw_of(pl.fwd),
            v3(pl.half_width * 2, pl.top, pl.half_len * 2),
        );
    }
    // the water pallet, exactly the Ravine's always-solid sphinx box
    let f = &r.finish;
    m::pallet(cells.at(f.sphinx), f.sphinx, f.sphinx_yaw);
    // the bins of bouncy balls: every still bouncer (a shopper's head is
    // one that moves)
    for (k, mu) in c.mushrooms.iter().enumerate() {
        if r.lizards.iter().any(|l| l.bouncer == k) {
            continue;
        }
        m::ball_bin(cells.at(mu.top), mu.top, mu.radius);
    }
    finish_banner(cells, c);
}

/// Columns, produce tables, display bins: each from its post.
fn posts(cells: &mut Cells, r: &RavineLayout, g: &GroceryLayout) {
    for p in &g.posts {
        let Some(k) = r.rocks.get(p.rock) else {
            continue;
        };
        match p.kind {
            PostKind::Column => {
                cells.push(
                    Mesh::Cyl,
                    &placed(
                        k.at.add(v3(0, k.height / 2, 0)),
                        0,
                        v3(k.radius * 2, k.height, k.radius * 2),
                    ),
                    COLUMN,
                    LIT,
                );
                cells.push(
                    Mesh::Cyl,
                    &placed(
                        k.at.add(v3(0, int(1), 0)),
                        0,
                        v3(k.radius * 2 + cm(60), int(2), k.radius * 2 + cm(60)),
                    ),
                    COLUMN_BASE,
                    LIT,
                );
            }
            PostKind::Table => m::table(cells.at(k.at), k.at, k.radius, k.height, p.stock),
            PostKind::Bin => m::bin(cells.at(k.at), k.at, k.radius, k.height, p.stock),
            // (drawn whole: `coffins`, `checkouts`)
            PostKind::Coffin | PostKind::Counter => {}
        }
    }
}

/// The coffin freezers: each a run of its four posts, drawn as one low
/// cream case with a glass lid and frozen goods inside.
fn coffins(cells: &mut Cells, r: &RavineLayout, g: &GroceryLayout) {
    let posts: Vec<V3> = g
        .posts
        .iter()
        .filter(|p| p.kind == PostKind::Coffin)
        .filter_map(|p| r.rocks.get(p.rock))
        .map(|k| k.at)
        .collect();
    let radius = g
        .posts
        .iter()
        .find(|p| p.kind == PostKind::Coffin)
        .and_then(|p| r.rocks.get(p.rock))
        .map_or(int(6), |k| k.radius);
    for (n, four) in posts.chunks(4).enumerate() {
        let (a, b) = (four[0], four[four.len() - 1]);
        let d = b.sub(a).flat();
        let len = d.len() + radius * 2;
        let yaw = yaw_of(d);
        let mid = a.lerp(b, ONE / 2);
        let h = int(11);
        cells.block(
            mid.add(v3(0, h / 2 - cm(60), 0)),
            yaw,
            v3(radius * 2, h - cm(120), len),
            COFFIN,
        );
        cells.push(
            Mesh::Cube,
            &placed(
                mid.add(v3(0, h - cm(30), 0)),
                yaw,
                v3(radius * 2 - cm(60), cm(40), len - cm(60)),
            ),
            COFFIN_GLASS,
            ONE / 2,
        );
        cells.block(
            mid.add(v3(0, cm(30), 0)),
            yaw,
            v3(radius * 2 + cm(20), cm(60), len + cm(20)),
            rgb(150, 150, 146),
        );
        // the goods inside, under the glass
        let along = d.norm();
        let across = right_of(along);
        for k in 0..8 {
            let t = -len / 2 + cm(150) + (len - int(3)) * k / 8;
            for s in [-1, 1] {
                cells.block(
                    mid.add(along.scale(t))
                        .add(across.scale(s * int(2)))
                        .add(v3(0, h - int(2), 0)),
                    yaw,
                    v3(int(3), int(2), int(3)),
                    FROZEN[(k as usize + n * 3 + (s + 1) as usize) % FROZEN.len()],
                );
            }
        }
    }
}

/// The checkout lanes: a counter with its belt, register and screen, the
/// bag carousel, the lane's light pole and number.
fn checkouts(cells: &mut Cells, g: &GroceryLayout) {
    use crate::grocery::layout::{LANES_NORTH, LANES_SOUTH};
    let len = int(LANES_NORTH - LANES_SOUTH);
    for (k, lane) in g.lanes.iter().enumerate() {
        let along = crate::geom::facing(lane.yaw, 0);
        let across = right_of(along);
        let yaw = lane.yaw;
        let mid = lane.at;
        // the counter (the posts are its solid), its blue kick and trim
        cells.block(
            mid.add(v3(0, int(5), 0)),
            yaw,
            v3(int(9), int(10), len),
            COUNTER,
        );
        cells.block(
            mid.add(v3(0, int(1), 0)).add(across.scale(int(4) + cm(70))),
            yaw,
            v3(cm(40), int(2), len),
            CHECKOUT_BLUE,
        );
        cells.block(
            mid.add(v3(0, int(10) + cm(20), 0)),
            yaw,
            v3(int(9) + cm(20), cm(40), len + cm(20)),
            CHECKOUT_BLUE,
        );
        // the belt on the far half, the register and its screen at the end
        cells.push(
            Mesh::Cube,
            &placed(
                mid.add(along.neg().scale(int(12)))
                    .add(v3(0, int(10) + cm(50), 0)),
                yaw,
                v3(int(6), cm(30), int(24)),
            ),
            BELT,
            LIT,
        );
        let reg = mid.add(along.scale(int(10))).add(v3(0, int(13), 0));
        cells.block(reg, yaw, v3(int(4), int(5), int(4)), REGISTER);
        cells.push(
            Mesh::Cube,
            &placed(reg.add(v3(0, int(4), 0)), yaw, v3(int(3), int(3), cm(30))),
            SCREEN,
            ONE / 2,
        );
        // the bag carousel, blue bags
        let bags = mid.add(along.scale(int(20))).add(across.scale(int(7)));
        cells.block(
            bags.add(v3(0, int(6), 0)),
            0,
            v3(cm(60), int(12), cm(60)),
            REGISTER,
        );
        for s in 0..3 {
            let q = crate::scene::around(bags, s * crate::fx::TURN / 3, int(2), int(8));
            cells.push(
                Mesh::Blob,
                &placed(q, s * 7000, v3(int(3), int(4), int(2))),
                BAG,
                LIT,
            );
        }
        // the lane's pole and its number light
        let pole = mid.add(along.scale(int(23))).sub(across.scale(int(4)));
        cells.push(
            Mesh::Cyl,
            &placed(pole.add(v3(0, int(15), 0)), 0, v3(cm(60), int(30), cm(60))),
            REGISTER,
            LIT,
        );
        m::aisle_sign(
            cells.at(pole),
            pole.add(v3(0, int(31), 0)),
            yaw_of(across),
            k as u32 + 1,
        );
    }
}

/// Produce's wooden canopy over the tables: beams both ways at 44 m,
/// track lights hanging from them.
fn canopy(cells: &mut Cells) {
    let (x0, x1, z0, z1) = (560, 750, 40, 200);
    let y = int(44);
    let mut x = x0;
    while x <= x1 {
        cells.block(
            at(int(x), int(z0 + z1) / 2, y),
            0,
            v3(int(z1 - z0), int(2), int(2)),
            CANOPY,
        );
        x += 19;
    }
    let mut z = z0;
    let mut k = 0;
    while z <= z1 {
        cells.block(
            at(int(x0 + x1) / 2, int(z), y + int(2)),
            0,
            v3(int(2), int(2), int(x1 - x0)),
            CANOPY,
        );
        let mut x = x0 + 10;
        while x < x1 {
            let p = at(int(x), int(z), y - int(2));
            cells.push(
                Mesh::Cyl,
                &placed(p, 0, v3(int(1), int(2), int(1))),
                TRACK,
                0,
            );
            cells.push(
                Mesh::Sphere,
                &placed(p.sub(v3(0, int(1), 0)), 0, v3(cm(80), cm(40), cm(80))),
                rgb(255, 240, 200),
                ONE,
            );
            x += 38;
        }
        z += 32;
        k += 1;
    }
    let _ = k;
}

/// Each aisle's number at both ends, and sale banners down some.
fn hanging_signs(cells: &mut Cells) {
    use crate::grocery::layout::{EAST_WALL, FREEZER_HALF, GONDOLA_HALF, WEST};
    let half = |k: usize| match RUNS[k].2 {
        crate::grocery::Run::Shelves => GONDOLA_HALF,
        crate::grocery::Run::Freezer => FREEZER_HALF,
    };
    for a in 0..=RUNS.len() {
        let x = |z: i32| match a {
            0 => int(WEST + RUNS[0].0 - half(0)) / 2,
            n if n == RUNS.len() => int(RUNS[n - 1].0 + half(n - 1) + EAST_WALL) / 2,
            n => aisle_x(n - 1, n, int(z)),
        };
        for (z, yaw) in [
            (FRONT + 26, yaw_of(NORTH.neg())),
            (BACK - 26, yaw_of(NORTH)),
        ] {
            let p = at(x(z), int(z), int(42));
            m::aisle_sign(cells.at(p), p, yaw, a as u32 + 1);
        }
        if a % 2 == 1 {
            let z = (FRONT + BACK) / 2 + if a % 4 == 1 { -60 } else { 60 };
            let p = at(x(z) + int(8), int(z), int(36));
            m::banner(cells.at(p), p, yaw_of(NORTH), a as u32);
        }
    }
    let _ = EAST;
}

/// The finish: a chequered banner across the front-end vestibule over the
/// line, and a glowing line on the floor.
fn finish_banner(cells: &mut Cells, c: &Course) {
    let n = c.path.len() - 1;
    let line = c.path[n];
    let fwd = c.heading[n];
    let right = right_of(fwd);
    let yaw = yaw_of(fwd);
    let span = int(32);
    let squares = 16;
    for k in 0..squares {
        for row in 0..2 {
            let x = -span + 2 * span * k / squares + span / squares;
            let p = line
                .add(right.scale(x))
                .add(v3(0, int(40) + row * int(2), 0));
            cells.push(
                Mesh::Cube,
                &placed(p, yaw, v3(2 * span / squares, int(2), cm(40))),
                if (k + row) % 2 == 0 {
                    BANNER
                } else {
                    BANNER_DARK
                },
                ONE / 3,
            );
        }
    }
    for s in [-1, 1] {
        let p = line.add(right.scale(s * span)).add(v3(0, int(46), 0));
        cells.push(
            Mesh::Cyl,
            &placed(p, 0, v3(cm(20), int(12), cm(20))),
            BANNER_DARK,
            0,
        );
    }
    cells.push(
        Mesh::Cube,
        &placed(line.add(v3(0, cm(5), 0)), yaw, v3(span * 2, cm(10), cm(80))),
        BANNER,
        ONE / 2,
    );
}
