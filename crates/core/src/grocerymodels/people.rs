//! The shoppers (seven looks, one a store employee) and their carts. A
//! shopper is drawn in real centimetres of a 170 cm person (`u`), which at
//! the store's scale is the 20.4 m `grocery::SHOPPER_HEIGHT`.

use super::{Fr, CHROME, WIRE};
use crate::fx::{self, cm, deg, mul, sin, ONE, QUARTER, TURN};
use crate::geom::{v3, Affine, V3};
use crate::grocery::real_cm as u;
use crate::mesh::Mesh;
use crate::render::{rgb, shade, Batch, Rgb};

/// The cart's red plastic handle.
const CART_RED: Rgb = rgb(206, 38, 44);
const WHEEL: Rgb = rgb(52, 52, 56);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Hair {
    Short,
    Bun,
    Long,
    /// a fringe round the back of a bald head
    Fringe,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Over {
    None,
    /// the employee's apron
    Apron(Rgb),
    /// an open jacket over a shirt of this colour
    Shirt(Rgb),
    /// a hood down the back
    Hood,
}

#[derive(Clone, Copy)]
struct Look {
    skin: Rgb,
    hair: Rgb,
    style: Hair,
    top: Rgb,
    /// long sleeves (else the forearms are skin)
    sleeves: bool,
    legs: Rgb,
    skirt: bool,
    shoes: Rgb,
    over: Over,
}

const LOOKS: [Look; 7] = [
    // the store employee: navy polo, dark apron, khakis
    Look {
        skin: rgb(222, 178, 140),
        hair: rgb(62, 42, 30),
        style: Hair::Short,
        top: rgb(32, 44, 90),
        sleeves: false,
        legs: rgb(176, 156, 118),
        skirt: false,
        shoes: rgb(30, 28, 28),
        over: Over::Apron(rgb(44, 44, 48)),
    },
    Look {
        skin: rgb(244, 212, 186),
        hair: rgb(226, 190, 112),
        style: Hair::Long,
        top: rgb(214, 82, 96),
        sleeves: true,
        legs: rgb(40, 52, 92),
        skirt: true,
        shoes: rgb(236, 232, 226),
        over: Over::None,
    },
    Look {
        skin: rgb(141, 92, 60),
        hair: rgb(26, 22, 22),
        style: Hair::Short,
        top: rgb(58, 112, 72),
        sleeves: true,
        legs: rgb(52, 78, 122),
        skirt: false,
        shoes: rgb(120, 80, 50),
        over: Over::None,
    },
    Look {
        skin: rgb(234, 190, 156),
        hair: rgb(204, 204, 208),
        style: Hair::Bun,
        top: rgb(150, 128, 192),
        sleeves: true,
        legs: rgb(118, 116, 120),
        skirt: false,
        shoes: rgb(70, 60, 56),
        over: Over::None,
    },
    Look {
        skin: rgb(176, 118, 80),
        hair: rgb(130, 58, 30),
        style: Hair::Short,
        top: rgb(240, 186, 52),
        sleeves: true,
        legs: rgb(36, 36, 40),
        skirt: false,
        shoes: rgb(232, 60, 50),
        over: Over::Hood,
    },
    Look {
        skin: rgb(92, 60, 42),
        hair: rgb(20, 16, 16),
        style: Hair::Long,
        top: rgb(36, 150, 152),
        sleeves: false,
        legs: rgb(72, 104, 156),
        skirt: true,
        shoes: rgb(40, 36, 36),
        over: Over::None,
    },
    Look {
        skin: rgb(250, 222, 198),
        hair: rgb(150, 146, 140),
        style: Hair::Fringe,
        top: rgb(108, 110, 118),
        sleeves: true,
        legs: rgb(36, 46, 76),
        skirt: false,
        shoes: rgb(60, 40, 30),
        over: Over::Shirt(rgb(244, 244, 240)),
    },
];

/// The cart's size (real cm): length, width, the handle's height.
const CART_LONG: i32 = 90;
const CART_WIDE: i32 = 55;
const HANDLE_UP: i32 = 100;
/// How far ahead of the shopper's feet the cart's middle is (real cm).
const CART_AHEAD: i32 = 92;

/// Where a cart's handle is, each end: `[left, right]` from behind.
pub(super) fn cart_handle(foot: V3, yaw: i32) -> [V3; 2] {
    let fr = Fr::new(foot, yaw);
    let z = -u(CART_LONG / 2 + 4);
    [
        fr.pt(v3(u(18), u(HANDLE_UP), z)),
        fr.pt(v3(-u(18), u(HANDLE_UP), z)),
    ]
}

pub(super) fn cart(b: &mut Batch, foot: V3, yaw: i32) {
    let fr = Fr::new(foot, yaw);
    let (half_w, back, front) = (u(CART_WIDE / 2), -u(40), u(45));
    let basket = (u(48), u(95));
    let mid_z = (back + front) / 2;
    let len = front - back;
    let wall_y = (basket.0 + basket.1) / 2;
    // the basket: a wire floor and chrome rails round it at two heights
    // (open, so the shopper shows through)
    fr.put(
        b,
        Mesh::Cube,
        v3(0, basket.0, mid_z),
        v3(half_w * 2, u(2), len),
        WIRE,
        0,
    );
    for (i, y) in [basket.1, wall_y].into_iter().enumerate() {
        for s in [-1, 1] {
            fr.rod(
                b,
                Mesh::Cyl,
                v3(s * half_w, y, back),
                v3(s * half_w, y, front),
                u(if i == 0 { 3 } else { 2 }),
                CHROME,
                0,
            );
        }
        let ends: &[i32] = if i == 0 { &[back, front] } else { &[front] };
        for &z in ends {
            fr.rod(
                b,
                Mesh::Cyl,
                v3(-half_w, y, z),
                v3(half_w, y, z),
                u(if i == 0 { 3 } else { 2 }),
                CHROME,
                0,
            );
        }
    }
    // the bottom rack
    fr.put(
        b,
        Mesh::Cube,
        v3(0, u(14), mid_z),
        v3(half_w * 3 / 2, u(2), len),
        CHROME,
        0,
    );
    // the back legs up to the handle
    let hz = -u(CART_LONG / 2 + 4);
    for s in [-1, 1] {
        fr.rod(
            b,
            Mesh::Cyl,
            v3(s * u(22), u(6), back + u(4)),
            v3(s * u(26), u(HANDLE_UP), hz),
            u(3),
            CHROME,
            0,
        );
    }
    fr.rod(
        b,
        Mesh::Cyl,
        v3(-u(28), u(HANDLE_UP), hz),
        v3(u(28), u(HANDLE_UP), hz),
        u(4),
        CART_RED,
        0,
    );
    // four wheels, their axles across
    for (x, z) in [
        (-1, back + u(5)),
        (1, back + u(5)),
        (-1, front - u(6)),
        (1, front - u(6)),
    ] {
        fr.turned(
            b,
            Mesh::Cyl,
            v3(x * u(21), u(5), z),
            Affine::rot_z(QUARTER),
            v3(u(10), u(3), u(10)),
            WHEEL,
            0,
        );
    }
}

/// A two-bone limb from `root` reaching `to` (lengths `l1`, `l2`), its
/// joint bent toward `bend`: the joint and the end (the end is `to`, even
/// past its reach).
fn reach(root: V3, to: V3, l1: i32, l2: i32, bend: V3) -> (V3, V3) {
    let d = to.sub(root);
    let dist = d.len();
    if dist == 0 {
        return (root.add(bend.norm().scale(l1)), to);
    }
    let dir = d.norm();
    if dist >= l1 + l2 - cm(1) {
        return (root.add(dir.scale(l1)), to);
    }
    // the joint's foot along the line, and how far off it
    let a = fx::div(mul(l1, l1) - mul(l2, l2) + mul(dist, dist), dist * 2);
    let off = fx::sqrt((mul(l1, l1) - mul(a, a)).max(0));
    let side = bend.reject(dir).norm();
    (root.add(dir.scale(a)).add(side.scale(off)), to)
}

/// A unit direction down a limb swung `angle` forward (in the frame).
fn swung(angle: i32) -> V3 {
    v3(0, -fx::cos(angle), sin(angle))
}

pub(super) fn shopper(
    b: &mut Batch,
    feet: V3,
    yaw: i32,
    look: u8,
    stride: i32,
    string: Option<V3>,
    with_cart: bool,
) {
    let lk = LOOKS[(look as usize) % LOOKS.len()];
    let fr = Fr::new(feet, yaw);
    // the walk: one step cycle per `stride` 0..ONE; 0 stands
    let a = mul(stride.rem_euclid(ONE), TURN);
    let swing = mul(sin(a), deg(26));
    let bend = mul(sin(a * 2), deg(30));
    let legs = [(1, swing, bend.max(0)), (-1, -swing, (-bend).max(0))];
    let (thigh, shin) = (u(40), u(41));
    // the stance leg reaches the floor: its ankle 8 cm up
    let reach_of = |s: i32, k: i32| mul(thigh, fx::cos(s)) + mul(shin, fx::cos(s - k));
    let hip_y = u(8)
        + legs
            .iter()
            .map(|&(_, s, k)| reach_of(s, k))
            .max()
            .unwrap_or(0);
    let lift = hip_y - u(89);
    let up = |y: i32| u(y) + lift;
    let leg_colour = if lk.skirt { lk.skin } else { lk.legs };
    for &(side, s, k) in &legs {
        let hip = v3(side * u(10), hip_y, 0);
        let knee = hip.add(swung(s).scale(thigh));
        let ankle = knee.add(swung(s - k).scale(shin));
        fr.rod(b, Mesh::Taper, hip, knee, u(13), leg_colour, 0);
        fr.rod(b, Mesh::Taper, knee, ankle, u(11), leg_colour, 0);
        fr.put(
            b,
            Mesh::Bevel,
            ankle.add(v3(0, -u(4), u(6))),
            v3(u(10), u(8), u(26)),
            lk.shoes,
            0,
        );
    }
    // hips (or a skirt), the torso and what is over it
    if lk.skirt {
        fr.put(
            b,
            Mesh::Taper,
            v3(0, up(86), 0),
            v3(u(36), u(30), u(28)),
            lk.legs,
            0,
        );
    } else {
        fr.put(
            b,
            Mesh::Bevel,
            v3(0, up(92), 0),
            v3(u(25), u(16), u(17)),
            lk.legs,
            0,
        );
    }
    // the torso: a tapered prism turned over, broad at the shoulders
    fr.turned(
        b,
        Mesh::Taper,
        v3(0, up(121), 0),
        Affine::rot_z(fx::HALF),
        v3(u(40), u(46), u(23)),
        lk.top,
        0,
    );
    match lk.over {
        Over::None => {}
        Over::Apron(c) => fr.put(
            b,
            Mesh::Cube,
            v3(0, up(102), u(12)),
            v3(u(30), u(52), u(2)),
            c,
            0,
        ),
        Over::Shirt(c) => fr.put(
            b,
            Mesh::Cube,
            v3(0, up(124), u(12)),
            v3(u(9), u(40), u(2)),
            c,
            0,
        ),
        Over::Hood => fr.put(
            b,
            Mesh::Sphere,
            v3(0, up(142), -u(10)),
            v3(u(22), u(12), u(10)),
            shade(lk.top, 88),
            0,
        ),
    }
    // the head
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, up(147), 0),
        v3(u(10), u(9), u(10)),
        lk.skin,
        0,
    );
    let head = v3(0, up(158), 0);
    fr.put(b, Mesh::Sphere, head, v3(u(19), u(22), u(21)), lk.skin, 0);
    match lk.style {
        Hair::Fringe => fr.put(
            b,
            Mesh::Sphere,
            head.add(v3(0, -u(2), -u(3))),
            v3(u(20), u(11), u(19)),
            lk.hair,
            0,
        ),
        _ => fr.put(
            b,
            Mesh::Sphere,
            head.add(v3(0, u(5), -u(2))),
            v3(u(21), u(15), u(22)),
            lk.hair,
            0,
        ),
    }
    match lk.style {
        Hair::Bun => fr.put(
            b,
            Mesh::Sphere,
            head.add(v3(0, u(10), -u(8))),
            v3(u(9), u(9), u(9)),
            lk.hair,
            0,
        ),
        Hair::Long => fr.put(
            b,
            Mesh::Bevel,
            head.add(v3(0, -u(6), -u(7))),
            v3(u(20), u(26), u(8)),
            lk.hair,
            0,
        ),
        Hair::Short | Hair::Fringe => {}
    }
    for s in [-1, 1] {
        fr.put(
            b,
            Mesh::Sphere,
            head.add(v3(s * u(4), u(2), u(9))),
            v3(u(2) + cm(10), u(3), u(2)),
            rgb(30, 24, 22),
            0,
        );
    }
    fr.put(
        b,
        Mesh::Cube,
        head.add(v3(0, -u(2), u(10))),
        v3(u(3), u(4), u(3)),
        shade(lk.skin, 90),
        0,
    );
    fr.put(
        b,
        Mesh::Cube,
        head.add(v3(0, -u(6), u(9))),
        v3(u(5), u(1), u(1)),
        shade(lk.skin, 66),
        0,
    );
    // the arms: swinging, or reaching for the string and the cart
    let handles = with_cart.then(|| {
        let at = fr.pt(v3(0, 0, u(CART_AHEAD)));
        cart(b, at, yaw);
        cart_handle(at, yaw)
    });
    let inv = fr.rot;
    let to_local = |w: V3| {
        let d = w.sub(feet);
        // the inverse of a turn about up is its transpose
        v3(inv.c0.dot(d), inv.c1.dot(d), inv.c2.dot(d))
    };
    let (upper, fore) = (u(30), u(29));
    for side in [1, -1] {
        let shoulder = v3(side * u(20), up(140), 0);
        // the hand nearer the string's end holds it; the other the cart
        let held = string.map(to_local);
        let holder = held.map(|e| if e.x < 0 { -1 } else { 1 });
        let target = if holder == Some(side) {
            held
        } else {
            handles.map(|h| to_local(if side == 1 { h[0] } else { h[1] }))
        };
        let (elbow, hand) = match target {
            Some(t) => {
                let bend = if t.y > shoulder.y {
                    v3(side * ONE, 0, -ONE / 2)
                } else {
                    v3(side * ONE / 2, -ONE / 2, -ONE)
                };
                reach(shoulder, t, upper, fore, bend)
            }
            None => {
                let s = -mul(swing, ONE * 4 / 5) * side;
                let elbow = shoulder
                    .add(swung(s).scale(upper))
                    .add(v3(side * u(2), 0, 0));
                let wrist = elbow.add(swung(s + deg(18)).scale(fore - u(4)));
                (elbow, wrist)
            }
        };
        fr.rod(b, Mesh::Taper, shoulder, elbow, u(10), lk.top, 0);
        let forearm = if lk.sleeves { lk.top } else { lk.skin };
        fr.rod(b, Mesh::Taper, elbow, hand, u(8), forearm, 0);
        fr.put(b, Mesh::Sphere, hand, v3(u(8), u(9), u(8)), lk.skin, 0);
    }
}
