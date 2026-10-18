//! The store's walls and departments, drawn flush with the walls' solid
//! faces (`grocery::layout`: the store's wall stands at the cases' fronts,
//! so each department's cases are drawn behind that face and a soffit
//! fills the face above them to the ceiling, as in the photos: a cream
//! wall, a blue-grey band, a department's picture sign).
//!
//! - **Dairy** (west): glass-door coolers of milk, juice, eggs, yogurt and
//!   cheese; the stockroom's swinging doors behind the start.
//! - **Meat and seafood** (back): black open cases, trays of meat in rows.
//! - **Deli** and **bakery** (east): a glass service case, bread racks.
//! - **Produce's wet rack** (east, by the front): an angled bed of greens.
//! - **The front**: windows, the exit doors by the finish, and the glass
//!   storefront between the checkouts and the front-end run.

use super::course::{at, Cells, LIT};
use crate::course::Course;
use crate::fx::{cm, deg, int, ONE};
use crate::geom::{v3, yaw_of, V3};
use crate::grocery::layout::{EAST_WALL, GLASS_EAST, GLASS_WEST, GLASS_Z, NORTH_WALL, SOUTH, WEST};
use crate::grocery::{Dept, GroceryLayout, CEILING_AT, EAST, NORTH};
use crate::grocerymodels as m;
use crate::mesh::Mesh;
use crate::ravine::RavineLayout;
use crate::render::{placed, rgb, Rgb};

const WALL: Rgb = rgb(231, 225, 210);
const BAND: Rgb = rgb(163, 182, 200);
const BAND_DARK: Rgb = rgb(120, 140, 160);
const CASE_BLACK: Rgb = rgb(35, 34, 36);
const CASE_CREAM: Rgb = rgb(226, 220, 203);
const COOLER_INSIDE: Rgb = rgb(214, 230, 238);
const GLASS: Rgb = rgb(214, 232, 242);
const MULLION: Rgb = rgb(150, 156, 160);
const DOOR: Rgb = rgb(150, 150, 146);
const DOOR_WINDOW: Rgb = rgb(60, 70, 80);
const WOOD: Rgb = rgb(123, 90, 63);
const WOOD_DARK: Rgb = rgb(75, 58, 45);
const GREENS: [Rgb; 4] = [
    rgb(70, 150, 60),
    rgb(110, 180, 70),
    rgb(50, 120, 60),
    rgb(160, 200, 90),
];
const MEAT: [Rgb; 4] = [
    rgb(196, 64, 70),
    rgb(226, 120, 120),
    rgb(170, 40, 50),
    rgb(240, 196, 180),
];
const DAIRY: [Rgb; 6] = [
    rgb(246, 246, 240),
    rgb(60, 110, 200),
    rgb(246, 160, 40),
    rgb(232, 214, 170),
    rgb(240, 200, 60),
    rgb(230, 120, 160),
];
const BREAD: [Rgb; 3] = [rgb(214, 160, 96), rgb(176, 118, 64), rgb(236, 206, 150)];

/// How tall each department's cases stand (game metres): the soffit fills
/// the wall above.
fn case_top(d: Dept) -> i32 {
    match d {
        Dept::Dairy => int(26),
        Dept::Meat => int(18),
        Dept::Deli => int(16),
        Dept::Bakery => int(22),
        Dept::WetRack => int(16),
    }
}

/// The picture on a department's signs (`grocerymodels::dept_sign`).
fn sign_of(d: Dept) -> u8 {
    match d {
        Dept::Dairy => 0,
        Dept::Meat => 1,
        Dept::Deli => 2,
        Dept::Bakery => 3,
        Dept::WetRack => 4,
    }
}

pub fn scenery(cells: &mut Cells, c: &Course, r: &RavineLayout, g: &GroceryLayout) {
    let _ = (c, r);
    for k in &g.cases {
        cases(cells, k);
    }
    bare_walls(cells, g);
    stockroom_doors(cells);
    front(cells);
    glass(cells);
}

/// A point `along` metres from `from` toward `to` (on the wall's face),
/// `back` metres behind the face (out of the store), `up` over the floor.
fn on(from: V3, to: V3, out: V3, along: i32, back: i32, up: i32) -> V3 {
    from.add(to.sub(from).flat().norm().scale(along))
        .sub(out.scale(back))
        .add(v3(0, up, 0))
}

/// A department's cases along its wall, and the soffit over them.
fn cases(cells: &mut Cells, k: &crate::grocery::Cases) {
    let len = k.to.sub(k.from).flat().len();
    let yaw = yaw_of(k.to.sub(k.from));
    let top = case_top(k.dept);
    let mid = |back: i32, up: i32| on(k.from, k.to, k.out, len / 2, back, up);
    // the soffit: cream to the ceiling, the blue-grey band, pictures
    let soffit = CEILING_AT - top;
    cells.block(
        mid(int(1), top + soffit / 2),
        yaw,
        v3(int(2), soffit, len),
        WALL,
    );
    cells.block(
        mid(-cm(8), top + soffit * 2 / 5),
        yaw,
        v3(cm(20), soffit * 2 / 5, len),
        BAND,
    );
    cells.block(
        mid(-cm(10), top + cm(60)),
        yaw,
        v3(cm(30), cm(120), len),
        BAND_DARK,
    );
    let mut a = int(40);
    while a < len - int(20) {
        let p = on(k.from, k.to, k.out, a, -cm(40), top + soffit * 3 / 5);
        m::dept_sign(cells.at(p), p, yaw_of(k.out), sign_of(k.dept), int(9));
        a += int(110);
    }
    match k.dept {
        Dept::Dairy => coolers(cells, k, len, yaw, top),
        Dept::Meat => meat(cells, k, len, yaw, top),
        Dept::Deli => deli(cells, k, len, yaw, top),
        Dept::Bakery => bakery(cells, k, len, yaw, top),
        Dept::WetRack => wet_rack(cells, k, len, yaw, top),
    }
}

/// Dairy: glass-door coolers, rows of milk, juice, eggs, yogurt, cheese.
fn coolers(cells: &mut Cells, k: &crate::grocery::Cases, len: i32, yaw: i32, top: i32) {
    let mid = |back: i32, up: i32| on(k.from, k.to, k.out, len / 2, back, up);
    cells.push(
        Mesh::Cube,
        &placed(mid(int(6), top / 2), yaw, v3(int(10), top, len)),
        COOLER_INSIDE,
        ONE * 3 / 4,
    );
    cells.block(
        mid(cm(40), cm(80)),
        yaw,
        v3(cm(80), cm(160), len),
        CASE_CREAM,
    );
    cells.block(
        mid(cm(40), top - cm(80)),
        yaw,
        v3(cm(80), cm(160), len),
        CASE_CREAM,
    );
    let shelves = 5;
    for s in 0..shelves {
        let y = cm(160) + (top - int(4)) * s / shelves;
        cells.block(mid(int(4), y), yaw, v3(int(6), cm(30), len), CASE_CREAM);
        // products in runs of colour, every 6 m a new one
        let mut a = int(1);
        let mut j = s as usize;
        while a < len - int(6) {
            let c = DAIRY[j % DAIRY.len()];
            let h = int(3) + cm(40) * ((j % 3) as i32);
            cells.block(
                on(k.from, k.to, k.out, a + int(3), int(3), y + cm(15) + h / 2),
                yaw,
                v3(int(3), h, int(6) - cm(30)),
                c,
            );
            a += int(10);
            j += 2;
        }
    }
    let mut a = 0;
    while a < len {
        cells.block(
            on(k.from, k.to, k.out, a, cm(30), top / 2),
            yaw,
            v3(cm(60), top, cm(70)),
            CASE_CREAM,
        );
        let h = a + int(9) - cm(120);
        if h < len {
            cells.push(
                Mesh::Cube,
                &placed(
                    on(k.from, k.to, k.out, h, -cm(20), top / 2),
                    yaw,
                    v3(cm(30), top / 3, cm(30)),
                ),
                rgb(28, 27, 28),
                0,
            );
        }
        a += int(9);
    }
}

/// Meat and seafood: black open cases, a lit top, trays of meat on three
/// stepped shelves.
fn meat(cells: &mut Cells, k: &crate::grocery::Cases, len: i32, yaw: i32, top: i32) {
    let mid = |back: i32, up: i32| on(k.from, k.to, k.out, len / 2, back, up);
    cells.block(
        mid(int(6), int(4)),
        yaw,
        v3(int(11), int(8), len),
        CASE_BLACK,
    );
    cells.block(mid(int(10), top / 2), yaw, v3(int(3), top, len), CASE_BLACK);
    cells.block(
        mid(int(6), top - int(1)),
        yaw,
        v3(int(11), int(2), len),
        CASE_BLACK,
    );
    cells.push(
        Mesh::Cube,
        &placed(
            mid(int(5), top - int(2) - cm(20)),
            yaw,
            v3(int(8), cm(30), len),
        ),
        rgb(255, 250, 240),
        ONE,
    );
    cells.block(
        mid(cm(30), int(8) + cm(20)),
        yaw,
        v3(cm(60), cm(40), len),
        CASE_CREAM,
    );
    for s in 0..3 {
        let y = int(8) + s * int(3);
        let back = int(2) + s * int(2);
        let mut a = int(2);
        let mut j = s as usize;
        while a < len - int(4) {
            cells.block(
                on(k.from, k.to, k.out, a + int(2), back + int(2), y + cm(60)),
                yaw,
                v3(int(3), cm(100), int(4) - cm(40)),
                MEAT[j % MEAT.len()],
            );
            a += int(8);
            j += 1;
        }
    }
}

/// The deli: a glass service case of cold cuts and cheese, the wall's
/// cabinets behind.
fn deli(cells: &mut Cells, k: &crate::grocery::Cases, len: i32, yaw: i32, top: i32) {
    let mid = |back: i32, up: i32| on(k.from, k.to, k.out, len / 2, back, up);
    cells.block(
        mid(int(3), int(4)),
        yaw,
        v3(int(6), int(8), len),
        CASE_CREAM,
    );
    cells.push(
        Mesh::Cube,
        &placed(mid(int(3), int(10)), yaw, v3(int(5), int(4), len)),
        GLASS,
        ONE / 2,
    );
    let mut a = int(2);
    let mut j = 0;
    while a < len - int(3) {
        cells.block(
            on(k.from, k.to, k.out, a + int(1), int(3), int(9)),
            yaw,
            v3(int(3), int(1), int(2)),
            [MEAT[3], rgb(240, 200, 60), MEAT[1]][j % 3],
        );
        a += int(3);
        j += 1;
    }
    cells.block(
        mid(int(10), top - int(4)),
        yaw,
        v3(int(4), int(8), len),
        WOOD,
    );
}

/// The bakery: wooden racks of loaves and cakes.
fn bakery(cells: &mut Cells, k: &crate::grocery::Cases, len: i32, yaw: i32, top: i32) {
    let mid = |back: i32, up: i32| on(k.from, k.to, k.out, len / 2, back, up);
    cells.block(mid(int(8), top / 2), yaw, v3(int(4), top, len), WOOD_DARK);
    for s in 0..4 {
        let y = int(2) + s * int(5);
        cells.block(mid(int(4), y), yaw, v3(int(8), cm(50), len), WOOD);
        let mut a = int(2);
        let mut j = s as usize;
        while a < len - int(3) {
            cells.push(
                Mesh::Blob,
                &placed(
                    on(k.from, k.to, k.out, a + cm(150), int(3), y + cm(110)),
                    yaw,
                    v3(int(3), int(2), int(3) - cm(20)),
                ),
                BREAD[j % BREAD.len()],
                LIT,
            );
            a += int(3);
            j += 1;
        }
    }
}

/// Produce's wet rack: an angled bed of greens, a mirror strip over it,
/// the mist bar.
fn wet_rack(cells: &mut Cells, k: &crate::grocery::Cases, len: i32, yaw: i32, top: i32) {
    let mid = |back: i32, up: i32| on(k.from, k.to, k.out, len / 2, back, up);
    cells.block(
        mid(int(5), int(4)),
        yaw,
        v3(int(10), int(8), len),
        WOOD_DARK,
    );
    cells.push(
        Mesh::Cube,
        &placed(mid(int(8), top - int(3)), yaw, v3(cm(40), int(5), len)),
        rgb(200, 214, 220),
        ONE / 2,
    );
    cells.block(
        mid(int(4), top - cm(50)),
        yaw,
        v3(int(8), cm(60), len),
        MULLION,
    );
    for row in 0..3 {
        let mut a = int(2);
        let mut j = row as usize;
        while a < len - int(3) {
            cells.push(
                Mesh::Blob,
                &placed(
                    on(
                        k.from,
                        k.to,
                        k.out,
                        a,
                        int(1) + row * int(3),
                        int(8) + row * int(2),
                    ),
                    j as i32 * 9000,
                    v3(int(3), int(2), int(3)),
                ),
                GREENS[j % GREENS.len()],
                LIT,
            );
            a += int(3);
            j += 1;
        }
    }
}

/// The walls with no cases: cream to the ceiling, the blue-grey band.
fn bare_walls(cells: &mut Cells, g: &GroceryLayout) {
    let p = |x: i32, z: i32| at(int(x), int(z), 0);
    // (each stretch: its two ends on the face, and out of the store)
    let mut stretches = vec![
        (p(WEST, SOUTH), p(WEST, 160), EAST),
        (p(WEST, 560), p(WEST, NORTH_WALL), EAST),
        (p(WEST, NORTH_WALL), p(30, NORTH_WALL), NORTH.neg()),
        (p(740, NORTH_WALL), p(EAST_WALL, NORTH_WALL), NORTH.neg()),
        (p(EAST_WALL, 560), p(EAST_WALL, NORTH_WALL), EAST.neg()),
        (p(EAST_WALL, 370), p(EAST_WALL, 380), EAST.neg()),
        (p(EAST_WALL, 180), p(EAST_WALL, 200), EAST.neg()),
        (p(EAST_WALL, SOUTH), p(EAST_WALL, 40), EAST.neg()),
    ];
    let _ = g;
    for (a, b, out) in stretches.drain(..) {
        let len = b.sub(a).flat().len();
        if len <= 0 {
            continue;
        }
        let yaw = yaw_of(b.sub(a));
        let mid = a.lerp(b, ONE / 2);
        cells.block(
            mid.sub(out.scale(int(1))).add(v3(0, CEILING_AT / 2, 0)),
            yaw,
            v3(int(2), CEILING_AT, len),
            WALL,
        );
        cells.block(
            mid.add(out.scale(cm(8))).add(v3(0, int(38), 0)),
            yaw,
            v3(cm(20), int(10), len),
            BAND,
        );
        cells.block(
            mid.add(out.scale(cm(8))).add(v3(0, int(3), 0)),
            yaw,
            v3(cm(20), int(6), len),
            BAND_DARK,
        );
    }
}

/// The stockroom's swinging doors in the west wall behind the start (the
/// food escapes through them): two grey doors with round windows, a
/// rubber bumper strip.
fn stockroom_doors(cells: &mut Cells) {
    for (k, z) in [530, 548].into_iter().enumerate() {
        let c = at(int(WEST), int(z), int(13));
        cells.block(
            c.add(EAST.scale(cm(20))),
            yaw_of(NORTH),
            v3(cm(40), int(26), int(16)),
            DOOR,
        );
        cells.push(
            Mesh::Cyl,
            &placed(
                c.add(EAST.scale(cm(50))).add(v3(0, int(5), 0)),
                yaw_of(EAST),
                v3(int(5), cm(20), int(5)),
            ),
            DOOR_WINDOW,
            ONE / 4,
        );
        cells.block(
            c.add(EAST.scale(cm(45))).sub(v3(0, int(9), 0)),
            yaw_of(NORTH),
            v3(cm(20), int(4), int(16)),
            rgb(40, 40, 40),
        );
        let _ = k;
    }
}

/// The front: windows along the front wall, the exit doors by the finish,
/// the wall over them.
fn front(cells: &mut Cells) {
    let yaw = yaw_of(EAST);
    let z = SOUTH;
    let (x0, x1) = (WEST, EAST_WALL);
    let mid = at(int(x0 + x1) / 2, int(z), 0);
    let len = int(x1 - x0);
    // the wall over the windows, and under them
    cells.block(
        mid.sub(NORTH.scale(int(1))).add(v3(0, int(44), 0)),
        yaw,
        v3(int(2), int(16), len),
        WALL,
    );
    cells.block(
        mid.sub(NORTH.scale(int(1))).add(v3(0, int(2), 0)),
        yaw,
        v3(int(2), int(4), len),
        BAND_DARK,
    );
    // the windows: daylight beyond, mullions every 18 m
    cells.push(
        Mesh::Cube,
        &placed(
            mid.sub(NORTH.scale(int(2))).add(v3(0, int(20), 0)),
            yaw,
            v3(cm(40), int(32), len),
        ),
        rgb(220, 236, 250),
        ONE,
    );
    let mut x = x0;
    while x <= x1 {
        cells.block(
            at(int(x), int(z), int(20)).sub(NORTH.scale(cm(60))),
            yaw,
            v3(cm(80), int(32), cm(80)),
            MULLION,
        );
        x += 18;
    }
    // the exit doors: two sliding leaves, a frame, a green exit light
    let d = at(int(410), int(z), 0);
    cells.block(
        d.add(v3(0, int(30), 0)).sub(NORTH.scale(cm(40))),
        yaw,
        v3(cm(80), cm(80), int(40)),
        MULLION,
    );
    for s in [-1, 1] {
        cells.block(
            d.add(EAST.scale(s * int(16)))
                .add(v3(0, int(15), 0))
                .sub(NORTH.scale(cm(30))),
            yaw,
            v3(cm(60), int(30), cm(80)),
            MULLION,
        );
    }
    cells.push(
        Mesh::Cube,
        &placed(
            d.add(v3(0, int(33), 0)).add(NORTH.scale(cm(20))),
            yaw,
            v3(cm(30), int(2), int(6)),
        ),
        rgb(60, 220, 120),
        ONE,
    );
}

/// The glass storefront between the checkouts and the front-end run: an
/// aluminium frame, glass panes, posters here and there.
fn glass(cells: &mut Cells) {
    let yaw = yaw_of(EAST);
    let len = int(GLASS_EAST - GLASS_WEST);
    let mid = at(int(GLASS_WEST + GLASS_EAST) / 2, int(GLASS_Z) + cm(150), 0);
    cells.push(
        Mesh::Cube,
        &placed(
            mid.add(v3(0, CEILING_AT / 2, 0)),
            yaw,
            v3(int(3), CEILING_AT, len),
        ),
        GLASS,
        ONE / 2,
    );
    cells.block(
        mid.add(v3(0, int(1), 0)),
        yaw,
        v3(int(3) + cm(20), int(2), len),
        MULLION,
    );
    cells.block(
        mid.add(v3(0, int(30), 0)),
        yaw,
        v3(int(3) + cm(20), cm(60), len),
        MULLION,
    );
    let mut x = GLASS_WEST;
    let mut k = 0;
    while x <= GLASS_EAST {
        cells.block(
            at(int(x), int(GLASS_Z) + cm(150), CEILING_AT / 2),
            yaw,
            v3(int(3) + cm(20), CEILING_AT, cm(70)),
            MULLION,
        );
        if k % 3 == 1 && x + 12 < GLASS_EAST {
            // a poster on the glass
            cells.push(
                Mesh::Cube,
                &placed(
                    at(int(x + 7), int(GLASS_Z) - cm(5), int(16)),
                    yaw,
                    v3(cm(10), int(9), int(6)),
                ),
                [rgb(215, 50, 43), rgb(245, 209, 30), rgb(44, 94, 168)][(k / 3) as usize % 3],
                LIT,
            );
        }
        x += 15;
        k += 1;
    }
    let _ = deg(0);
}
