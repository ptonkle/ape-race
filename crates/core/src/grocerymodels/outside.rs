//! The second pass's models (GR-T1–T8): the store's banana logo, the
//! shopping list's items and the order bag, the parking lot (cars, light
//! poles, cart corrals, the entrance canopy, the sliding doors), the plane
//! that picks up the order, the drop ceiling and the bagged ice, and the
//! odd vines' fittings (a sign's arm for a sideways rope, a balloon weight
//! for an upward string).
//!
//! STUBS: plain boxes with the contract's sizes, to be replaced by the
//! models agent; every signature and doc comment here is the contract.

use crate::fx::{cm, int, ONE};
use crate::geom::{v3, yaw_of, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{placed, rgb, rod, Batch};

/// The store's logo: a big yellow banana in a circle (GR-T1), in `frame`'s
/// x-y plane facing its +z, `radius` round (its ring's outside), a flat
/// disc a twentieth of the radius thick. The uniforms, the plane and the
/// doors wear it.
pub fn banana_logo(b: &mut Batch, frame: &Affine, radius: i32) {
    let disc = frame
        .then(&Affine::rot_x(crate::fx::QUARTER))
        .then(&Affine::scale(v3(radius * 2, radius / 10, radius * 2)));
    b.push(Mesh::Cyl, &disc, rgb(31, 44, 92), 0);
    let banana = frame.then(&Affine::translate(v3(0, 0, radius / 10)));
    let p = |x: i32, y: i32| banana.apply(v3(x * radius / 10, y * radius / 10, 0));
    let pts = [p(-6, 3), p(-4, -2), p(0, -4), p(4, -2), p(6, 3)];
    for w in pts.windows(2) {
        b.push(
            Mesh::Cyl,
            &rod(w[0], w[1], radius / 4),
            rgb(246, 212, 40),
            ONE / 4,
        );
    }
}

/// Shopping-list item `zone` (0 a box of spaghetti, 1 a can of coffee, 2 a
/// box of frozen waffles, 3 a chocolate bar), its middle at `at`, turning
/// slowly with `tick`. `lit` (the viewer's own): glowing brightly, a halo
/// round it, sparkles; not lit (another racer's): its plain colours, dim.
/// About 4 m tall, so it reads from a cross aisle away. The string it
/// hangs on is the vine, drawn by the scene.
pub fn list_item(b: &mut Batch, at: V3, zone: u8, lit: bool, tick: u32) {
    let colour = [
        rgb(40, 90, 200),
        rgb(150, 70, 30),
        rgb(240, 200, 60),
        rgb(90, 50, 30),
    ][zone as usize % 4];
    let spin = tick as i32 * 200;
    b.push(
        Mesh::Cube,
        &placed(at.sub(v3(0, int(2), 0)), spin, v3(int(3), int(4), cm(120))),
        colour,
        if lit { ONE } else { 0 },
    );
    if lit {
        b.push_halo(
            Mesh::Sphere,
            &placed(at.sub(v3(0, int(4), 0)), 0, v3(int(8), int(8), int(8))),
            rgb(255, 240, 160),
            ONE,
        );
    }
}

/// A full paper grocery bag held up (the order, GR-T5): its bottom's middle
/// at `foot`, facing `yaw`, celery and a baguette out of the top, the
/// banana logo on its side. About 4 m tall.
pub fn order_bag(b: &mut Batch, foot: V3, yaw: i32) {
    b.push(
        Mesh::Cube,
        &placed(foot.add(v3(0, int(2), 0)), yaw, v3(int(3), int(4), int(2))),
        rgb(196, 160, 110),
        0,
    );
}

/// The plane that picks up the order (GR-T5): a small high-wing cargo
/// plane in the store's livery (white, a navy stripe, the banana logo on
/// the tail), 60 m across the wings, its middle at `at`, nose toward
/// `facing(yaw, pitch)`, banked `roll`, its propeller turning with `tick`.
/// With `hook`, a line hangs from its belly to a hook at that point.
pub fn plane(b: &mut Batch, at: V3, yaw: i32, pitch: i32, roll: i32, tick: u32, hook: Option<V3>) {
    let _ = (pitch, roll, tick);
    b.push(
        Mesh::Cube,
        &placed(at, yaw, v3(int(60), int(2), int(10))),
        rgb(246, 246, 242),
        0,
    );
    b.push(
        Mesh::Cube,
        &placed(at, yaw, v3(int(8), int(7), int(44))),
        rgb(246, 246, 242),
        0,
    );
    if let Some(h) = hook {
        b.push(Mesh::Cyl, &rod(at, h, cm(30)), rgb(40, 40, 40), 0);
    }
}

/// A parked car: its middle on the ground at `centre`, facing `yaw`,
/// `look` (0..8) its body colour and shape (a sedan, a hatchback, a pickup,
/// a minivan), filling `grocery::CAR_SIZE` (its solid box).
pub fn car(b: &mut Batch, centre: V3, yaw: i32, look: u8) {
    let (l, w, h) = crate::grocery::CAR_SIZE;
    let colour = [
        rgb(180, 30, 30),
        rgb(220, 220, 220),
        rgb(30, 30, 40),
        rgb(40, 80, 160),
        rgb(150, 150, 155),
    ][look as usize % 5];
    b.push(
        Mesh::Cube,
        &placed(centre.add(v3(0, h / 2, 0)), yaw, v3(w, h, l)),
        colour,
        0,
    );
}

/// A parking lot light pole on `foot`, `height` tall, its arm out to
/// `arm` (a point at the arm's height, where the vine hangs), a lamp head
/// at the arm's end lit from below.
pub fn lamp_post(b: &mut Batch, foot: V3, height: i32, arm: V3) {
    let top = foot.add(v3(0, height, 0));
    b.push(Mesh::Cyl, &rod(foot, top, cm(120)), rgb(90, 95, 100), 0);
    b.push(Mesh::Cyl, &rod(top, arm, cm(80)), rgb(90, 95, 100), 0);
}

/// A cart corral: its two long rails from `from` to `to` on the ground, a
/// sign on a post at `from`'s end, a row of nested carts inside.
pub fn corral(b: &mut Batch, from: V3, to: V3) {
    let up = v3(0, int(10), 0);
    b.push(
        Mesh::Cyl,
        &rod(from.add(up), to.add(up), cm(60)),
        rgb(190, 190, 195),
        0,
    );
}

/// The entrance canopy over the sidewalk: a flat roof from the store's
/// front out over the lot, its corners `a` and `b` on the ground (least and
/// most), its underside `under` over the ground, on posts at the outer
/// corners, the banana logo on its front fascia.
pub fn canopy(b: &mut Batch, a: V3, c: V3, under: i32) {
    let mid = a.lerp(c, ONE / 2);
    let size = c.sub(a);
    b.push(
        Mesh::Cube,
        &placed(
            v3(mid.x, under + int(3), mid.z),
            0,
            v3(size.x.abs(), int(6), size.z.abs()),
        ),
        rgb(31, 44, 92),
        0,
    );
}

/// The exit's sliding glass doors in the store's front: the opening's
/// middle on the ground at `mid`, the way across it `along` (flat, unit),
/// `width` across and `height` up; each leaf slid `open` (0 shut .. ONE
/// fully open) into the wall. A frame, a green exit sign, the banana logo
/// on each leaf.
pub fn doors(b: &mut Batch, mid: V3, along: V3, width: i32, height: i32, open: i32) {
    let yaw = yaw_of(along);
    for s in [-1, 1] {
        let off = width / 4 + crate::fx::mul(open, width / 2);
        b.push(
            Mesh::Cube,
            &placed(
                mid.add(along.scale(s * off)).add(v3(0, height / 2, 0)),
                yaw,
                v3(cm(40), height, width / 2),
            ),
            rgb(200, 225, 240),
            ONE / 3,
        );
    }
}

/// The drop ceiling over part of the frozen aisle (GR-T7): a box along
/// `fwd` (flat, unit) centred on `centre`, `half_width` across and
/// `half_len` along, its underside `under` over the floor, up to the
/// store's ceiling: white acoustic tiles on a grid, lit panels, a navy
/// fascia at each end with a frozen snowflake.
pub fn low_ceiling(b: &mut Batch, centre: V3, fwd: V3, half_width: i32, half_len: i32, under: i32) {
    let top = crate::grocery::CEILING_AT;
    b.push(
        Mesh::Cube,
        &placed(
            v3(centre.x, (under + top) / 2, centre.z),
            yaw_of(fwd),
            v3(half_width * 2, top - under, half_len * 2),
        ),
        rgb(236, 236, 232),
        ONE / 4,
    );
}

/// The bagged-ice pallets stacked across the aisle (GR-T7): along `from`
/// to `to` (their middle line on the floor), `half` thick, `top` tall:
/// pallets of clear bags of ice cubes with a blue print, a sign on top.
pub fn ice_wall(b: &mut Batch, from: V3, to: V3, half: i32, top: i32) {
    let mid = from.lerp(to, ONE / 2);
    let run = to.sub(from);
    b.push(
        Mesh::Cube,
        &placed(
            mid.add(v3(0, top / 2, 0)),
            yaw_of(run),
            v3(half * 2, top, run.len()),
        ),
        rgb(210, 230, 245),
        ONE / 4,
    );
}

/// A hanging sale sign's arm, for a sideways vine (GR-T8): chains from the
/// ceiling (`CEILING_AT`) down to `anchor`, a red-and-yellow sign there,
/// and a short arm along `way` (flat, unit) that the rope leaves from.
pub fn sign_arm(b: &mut Batch, anchor: V3, way: V3) {
    let top = v3(anchor.x, crate::grocery::CEILING_AT, anchor.z);
    b.push(Mesh::Cyl, &rod(top, anchor, cm(20)), rgb(90, 95, 100), 0);
    b.push(
        Mesh::Cube,
        &placed(anchor, yaw_of(way), v3(cm(40), int(6), int(8))),
        rgb(215, 50, 43),
        0,
    );
}

/// A balloon weight (GR-T8: an upward vine's anchor): a shiny foil-wrapped
/// weight sitting at `at` (its bottom's middle), the string tied on top.
pub fn balloon_weight(b: &mut Batch, at: V3) {
    b.push(
        Mesh::Cube,
        &placed(at.add(v3(0, cm(90), 0)), 0, v3(int(2), cm(180), int(2))),
        rgb(200, 60, 140),
        0,
    );
}
