//! The second pass's art (GR-T2–T8): the parking lot out of the exit doors
//! (asphalt, stall lines, cars, light poles, cart corrals, the entrance
//! canopy, the store's front seen from outside), the drop ceiling over the
//! frozen aisle and the bagged ice past it, the odd vines' fittings; and,
//! every frame, the sliding doors, the shopping list's items (as the viewer
//! sees them: their own lit) and the planes that pick up a full order.

use super::course::{Cells, LIT};
use crate::course::Course;
use crate::fx::{self, cm, int, mul, ONE, QUARTER};
use crate::geom::{right_of, v3, yaw_of, Affine, V3};
use crate::grocery::layout::{EAST_WALL, SOUTH, WEST};
use crate::grocery::{
    list, plan, plan_of, GroceryLayout, OddKind, DOOR_HEIGHT, DOOR_WIDTH, EAST, NORTH, PLANE_AFTER,
    PLANE_TICKS,
};
use crate::grocerymodels as m;
use crate::mesh::Mesh;
use crate::race::Race;
use crate::render::{placed, rgb, rod, Batch, Rgb};
use crate::rules::BODY_DROP;

const ASPHALT: Rgb = rgb(74, 76, 80);
const ASPHALT_B: Rgb = rgb(82, 84, 88);
const PAINT: Rgb = rgb(236, 236, 228);
const CURB: Rgb = rgb(190, 188, 180);
const GRASS: Rgb = rgb(96, 140, 70);
const HEDGE: Rgb = rgb(58, 104, 52);
const STUCCO: Rgb = rgb(206, 186, 156);
const STUCCO_DARK: Rgb = rgb(170, 150, 124);
const NAVY: Rgb = rgb(31, 44, 92);
const WINDOW: Rgb = rgb(250, 244, 220);
const RAMP: Rgb = rgb(214, 232, 246);
const RAMP_DARK: Rgb = rgb(150, 190, 225);

/// The lot's and the store front's still art, and the drop ceiling's, the
/// ice's and the odd vines' fittings.
pub fn scenery(cells: &mut Cells, c: &Course, g: &GroceryLayout) {
    lot(cells, g);
    facade(cells, g);
    inside(cells, c, g);
}

/// The asphalt and its lines, the verge round it, the cars, poles, corrals
/// and the canopy.
fn lot(cells: &mut Cells, g: &GroceryLayout) {
    let l = &g.lot;
    let (lo, hi) = (plan_of(l.bounds.0), plan_of(l.bounds.1));
    let (x0, x1, z0, z1) = (lo.0 >> 16, hi.0 >> 16, lo.1 >> 16, hi.1 >> 16);
    // the asphalt in 50 m slabs, and a verge of grass round it
    let mut x = x0 - 60;
    while x < x1 + 60 {
        let mut z = z0 - 60;
        while z < z1 {
            let inside = x >= x0 && x < x1 && z >= z0;
            let tone = match (inside, ((x / 50) + (z / 50)) % 2 == 0) {
                (false, _) => GRASS,
                (true, true) => ASPHALT,
                (true, false) => ASPHALT_B,
            };
            cells.push(
                Mesh::Cube,
                &placed(
                    plan(int(x + 25), int(z + 25)).sub(v3(0, cm(50), 0)),
                    0,
                    v3(int(50), cm(100), int(50)),
                ),
                tone,
                ONE / 5,
            );
            z += 50;
        }
        x += 50;
    }
    // the grass on out to the haze, and a row of trees round the lot
    // (west of it, east of it and past its far end: x0..x1, z0..z1)
    for (xa, xb, za, zb) in [
        (x0 - 260, x0 - 60, z0 - 260, SOUTH),
        (x1 + 60, x1 + 260, z0 - 260, SOUTH),
        (x0 - 60, x1 + 60, z0 - 260, z0 - 60),
    ] {
        cells.push(
            Mesh::Cube,
            &placed(
                plan(int(xa + xb) / 2, int(za + zb) / 2).sub(v3(0, cm(60), 0)),
                0,
                v3(int(zb - za), cm(100), int(xb - xa)),
            ),
            GRASS,
            ONE / 5,
        );
    }
    let mut k = 0;
    let mut tree = |cells: &mut Cells, x: i32, z: i32| {
        let foot = plan(int(x), int(z));
        let h = int(40 + (k * 7) % 14);
        cells.push(
            Mesh::Cyl,
            &rod(foot, foot.add(v3(0, h / 2, 0)), int(3)),
            rgb(98, 72, 50),
            ONE / 6,
        );
        cells.push(
            Mesh::Blob,
            &placed(
                foot.add(v3(0, h * 3 / 4, 0)),
                k * 9_000,
                v3(int(30), h * 3 / 5, int(30)),
            ),
            [HEDGE, GRASS, rgb(70, 118, 60)][k as usize % 3],
            ONE / 6,
        );
        k += 1;
    };
    let mut x = x0 - 30;
    while x <= x1 + 30 {
        tree(cells, x, z0 - 30);
        x += 36;
    }
    let mut z = z0;
    while z < SOUTH - 40 {
        tree(cells, x0 - 30, z);
        tree(cells, x1 + 30, z);
        z += 36;
    }
    // a hedge along the lot's far edge and its sides
    // (a low box along the ground from `a` to `b`, 6 m thick, 5 m tall)
    let hedge = |cells: &mut Cells, a: V3, b: V3| {
        let run = b.sub(a);
        cells.push(
            Mesh::Bevel,
            &placed(
                a.lerp(b, ONE / 2).add(v3(0, cm(250), 0)),
                yaw_of(run),
                v3(int(6), int(5), run.len()),
            ),
            HEDGE,
            ONE / 5,
        );
    };
    hedge(cells, plan(int(x0), int(z0)), plan(int(x1), int(z0)));
    hedge(
        cells,
        plan(int(x0), int(z0)),
        plan(int(x0), int(SOUTH - 30)),
    );
    hedge(
        cells,
        plan(int(x1), int(z0)),
        plan(int(x1), int(SOUTH - 30)),
    );
    // the sidewalk under the canopy, and its curb
    let (ca, cb) = (plan_of(l.canopy.0), plan_of(l.canopy.1));
    let mid = plan((ca.0 + cb.0) / 2, (ca.1 + cb.1) / 2);
    cells.push(
        Mesh::Cube,
        &placed(
            mid.add(v3(0, cm(15), 0)),
            0,
            v3(cb.1 - ca.1, cm(30), int(EAST_WALL - WEST)),
        ),
        CURB,
        ONE / 4,
    );
    // stall lines between the cars, both rows, and the drive's arrows
    for car in &l.cars {
        let fwd = crate::geom::facing(car.yaw, 0);
        let across = right_of(fwd);
        for s in [-1, 1] {
            let line = car.centre.add(across.scale(s * int(16)));
            cells.push(
                Mesh::Cube,
                &placed(
                    line.add(v3(0, cm(3), 0)),
                    car.yaw,
                    v3(cm(60), cm(6), int(56)),
                ),
                PAINT,
                ONE / 3,
            );
        }
        m::car(cells.at(car.centre), car.centre, car.yaw, car.look);
    }
    for lamp in &l.lamps {
        m::lamp_post(cells.at(lamp.foot), lamp.foot, lamp.height, lamp.arm);
    }
    for &(a, b) in &l.corrals {
        m::corral(cells.at(a.lerp(b, ONE / 2)), a, b);
    }
    m::canopy(cells.at(mid), l.canopy.0, l.canopy.1, l.canopy_under);
}

/// The store's front from outside: stucco walls either side of the doors
/// to a parapet, a navy band, the windows lit from within, and the big
/// banana logo over the entrance.
fn facade(cells: &mut Cells, g: &GroceryLayout) {
    let (door, _) = g.doors;
    let (dx, _) = plan_of(door);
    let z = int(SOUTH) - int(10) - cm(20);
    let top = int(62);
    let yaw = yaw_of(EAST);
    let wall = |cells: &mut Cells, x0: i32, x1: i32, y0: i32, y1: i32, c: Rgb| {
        let mid = plan((x0 + x1) / 2, z).add(v3(0, (y0 + y1) / 2, 0));
        cells.push(
            Mesh::Cube,
            &placed(mid, yaw, v3(cm(40), y1 - y0, x1 - x0)),
            c,
            LIT,
        );
    };
    let (w, e) = (int(WEST - 10), int(EAST_WALL + 10));
    let (dw, de) = (dx - DOOR_WIDTH / 2, dx + DOOR_WIDTH / 2);
    wall(cells, w, dw, 0, top, STUCCO);
    wall(cells, de, e, 0, top, STUCCO);
    wall(cells, dw, de, DOOR_HEIGHT, top, STUCCO);
    // the navy band under the parapet, the cap
    let band = |cells: &mut Cells, y: i32, h: i32, c: Rgb| {
        let mid = plan((w + e) / 2, z - cm(40)).add(v3(0, y, 0));
        cells.push(Mesh::Cube, &placed(mid, yaw, v3(cm(60), h, e - w)), c, LIT);
    };
    band(cells, int(50), int(6), NAVY);
    band(cells, top, int(1), STUCCO_DARK);
    // windows either side of the entrance, lit from within
    for (x0, x1) in [(w + int(30), dw - int(14)), (de + int(14), e - int(30))] {
        let mut x = x0;
        while x + int(30) <= x1 {
            let mid = plan(x + int(15), z - cm(30)).add(v3(0, int(20), 0));
            cells.push(
                Mesh::Cube,
                &placed(mid, yaw, v3(cm(20), int(28), int(26))),
                WINDOW,
                ONE * 4 / 5,
            );
            x += int(34);
        }
    }
    // the big logo over the entrance, facing out
    let at = plan(dx, z - cm(80)).add(v3(0, int(45), 0));
    let f = Affine::translate(at).then(&Affine::rot_y(yaw_of(NORTH.neg())));
    m::banana_logo(cells.at(at), &f, int(10));
}

/// The drop ceiling, the ice and its ramp, the sideways ropes' sign arms
/// and the upward strings' balloon weights.
fn inside(cells: &mut Cells, c: &Course, g: &GroceryLayout) {
    for lc in &g.low {
        let run = lc.to.sub(lc.from).flat();
        let mid = lc.from.lerp(lc.to, ONE / 2);
        m::low_ceiling(
            cells.at(mid),
            mid,
            run.norm(),
            lc.half_width,
            run.len() / 2,
            lc.under,
        );
    }
    for ice in &g.ice {
        let mid = ice.from.lerp(ice.to, ONE / 2);
        m::ice_wall(cells.at(mid), ice.from, ice.to, ice.half, ice.top);
    }
    // the ramp of bags heaped up the ice's near side (its deck)
    if let Some(r) = c.ravine.as_deref() {
        for d in &r.decks {
            let run = d.to.sub(d.from);
            let flat = run.flat();
            let mid = d.from.lerp(d.to, ONE / 2);
            let yaw = yaw_of(flat.neg());
            // (a wedge: its back up, sloping down toward the front)
            cells.push(
                Mesh::Wedge,
                &placed(
                    v3(mid.x, d.to.y / 2, mid.z),
                    yaw,
                    v3(d.half_width * 2, d.to.y, flat.len()),
                ),
                RAMP,
                LIT,
            );
            // the bags' rows across it
            for k in 1..6 {
                let p = d.from.lerp(d.to, k * ONE / 6);
                let across = right_of(flat.norm()).scale(d.half_width);
                cells.push(
                    Mesh::Cube,
                    &rod(
                        p.sub(across).add(v3(0, cm(20), 0)),
                        p.add(across).add(v3(0, cm(20), 0)),
                        cm(90),
                    ),
                    RAMP_DARK,
                    LIT,
                );
            }
        }
    }
    for o in &g.odd {
        let v = &c.vines[o.vine];
        match o.kind {
            OddKind::Sideways => {
                let way = g.hangs.get(o.vine).copied().unwrap_or(EAST);
                m::sign_arm(cells.at(v.anchor), v.anchor, way);
            }
            OddKind::Upward => {
                m::balloon_weight(cells.at(v.anchor), v.anchor.sub(v3(0, int(1), 0)));
            }
        }
    }
}

/// Every frame: the doors sliding open as the eye comes near, and the
/// upward strings' balloons.
pub fn dynamic(b: &mut Batch, c: &Course, g: &GroceryLayout, eye: V3, tick: u32) {
    let (door, _) = g.doors;
    let d = eye.flat().sub(door.flat()).len();
    if d < int(250) {
        let open = ONE - fx::div((d - int(50)).clamp(0, int(60)), int(60));
        m::doors(b, door, EAST, DOOR_WIDTH, DOOR_HEIGHT, open);
    }
    for o in &g.odd {
        if o.kind == OddKind::Upward {
            let tip = c.tip(o.vine, tick);
            if !tip.flat().sub(eye.flat()).longer_than(int(250)) {
                m::balloons(b, tip, tick, o.vine as u32);
            }
        }
    }
}

/// The shopping list's items near the eye, as `viewer` sees them: their own
/// spot's glowing until they take it, everyone else's plain.
pub fn items(
    b: &mut Batch,
    race: &Race,
    g: &GroceryLayout,
    viewer: Option<usize>,
    eye: V3,
    t: u32,
) {
    let me = viewer.and_then(|v| race.racers.get(v)).map(|r| &r.ape);
    items_seen(
        b,
        g,
        me.map(|a| a.slot as usize),
        me.map_or(0, |a| a.list),
        eye,
        t,
    );
}

/// The items as one at spot `slot` with `list` taken sees them from `eye`:
/// their own glowing out to `OWN_SEEN`, the others plain within
/// `OTHERS_SEEN`.
pub fn items_seen(
    b: &mut Batch,
    g: &GroceryLayout,
    slot: Option<usize>,
    list: u8,
    eye: V3,
    t: u32,
) {
    for it in &g.items {
        let mine = slot == Some(it.slot as usize);
        let seen = if mine { OWN_SEEN } else { OTHERS_SEEN };
        if it.at.flat().sub(eye.flat()).longer_than(seen) {
            continue;
        }
        if mine && list & (1 << it.zone) != 0 {
            continue;
        }
        m::list_item(b, it.at, it.zone, mine, t);
    }
}

/// A racer's own item glows from this far (a cross aisle away and more) ...
const OWN_SEEN: i32 = int(260);
/// ... and the others' show only this near.
const OTHERS_SEEN: i32 = int(90);

/// The planes (GR-T5): an ape that finished with all four items holds its
/// order up, and `PLANE_AFTER` ticks on a plane comes in low over the lot,
/// snatches the bag on its hook and climbs away with it.
pub fn planes(b: &mut Batch, race: &Race, t: u32) {
    for r in &race.racers {
        let Some(at) = r.finish else {
            continue;
        };
        if !list::full(&r.ape) {
            continue;
        }
        let since = race.clock.saturating_sub(at);
        let hands = r.ape.pos;
        let held = hands.add(v3(0, cm(40), 0));
        let yaw = yaw_of(EAST.neg());
        if since < PLANE_AFTER {
            m::order_bag(b, held, r.ape.yaw);
            continue;
        }
        let u = (since - PLANE_AFTER) as i32;
        if u > PLANE_TICKS as i32 {
            continue;
        }
        // (the snatch comes `SNATCH` ticks in; the plane flies west at
        // `SPEED`, lowest over the ape)
        const SNATCH: i32 = 300;
        const SPEED: i32 = int(45);
        let way = crate::geom::facing(yaw, 0);
        let dt = u - SNATCH;
        let along = mul(SPEED, fx::ratio(dt, crate::rules::TICKS_PER_SECOND));
        let low = int(72);
        let climb = if dt <= 0 {
            mul(int(40), fx::div(dt.abs().min(SNATCH), SNATCH))
        } else {
            let s = fx::ratio(dt, crate::rules::TICKS_PER_SECOND);
            mul(int(5), mul(s, s))
        };
        let plane = hands.add(way.scale(along)).add(v3(0, low + climb, 0));
        let pitch = if dt <= 0 { -fx::deg(6) } else { fx::deg(10) };
        let line = int(62);
        let hook = if dt < 0 {
            // the line trails back and down, coming to the bag
            plane.sub(way.scale(int(14))).sub(v3(0, line, 0))
        } else {
            plane.sub(way.scale(int(20))).sub(v3(
                0,
                line - mul(cm(300), fx::ratio(dt.min(240), 240)),
                0,
            ))
        };
        m::plane(b, plane, yaw, pitch, 0, t, Some(hook));
        if dt < 0 {
            m::order_bag(b, held, r.ape.yaw);
        } else {
            m::order_bag(b, hook.sub(v3(0, int(4), 0)), yaw);
        }
        let _ = (BODY_DROP, QUARTER);
    }
}
