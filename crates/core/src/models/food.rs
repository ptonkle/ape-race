//! Clean-Up on Aisle Ape's apes (`Gear::Food`, 2026-10-18; the person:
//! "the apes in this map are all little food items, so one could be a
//! barnum cracker, another a fruit snack, maybe one shaped like an oreo,
//! you figure out the rest of them"). Each kind keeps its body, its poses
//! and its eyes, so the kinds still tell apart, and becomes one snack
//! (GR-Q10): the chimpanzee an animal cracker, the gorilla a sandwich
//! cookie, the orangutan a fruit snack, the gibbon a soft pretzel, the
//! proboscis monkey a cheese puff and the lemur a frosted circus cookie.
//! The snack recolours the ape's own body (`look`) and adds its marks
//! (`food`); each palette is a flavour. No brand: the sandwich cookie's
//! stamp is a ring of dots round a flower, never letters or a logo.
//!
//! The Floor is Lava cat on this map is a gummy jaguar (`GUMMY_JAGUAR`,
//! GR-Q11): raspberry gummy, glossy (`gummy_cat`).

use super::*;
use crate::render::mix;

/// the animal cracker's toasted edges and its pressed docking holes
pub(super) const CRACKER_TOAST: Rgb = rgb(164, 100, 40);
pub(super) const DOCKING: Rgb = rgb(140, 88, 40);
/// sugar: the cracker's sparkle, the fruit snack's dusting
pub(super) const SUGAR: Rgb = rgb(255, 248, 226);
pub(super) const SUGAR_DUST: Rgb = rgb(244, 244, 236);
/// the sandwich cookie's fillings, by flavour
pub(super) const CREAM: Rgb = rgb(246, 242, 228);
pub(super) const MINT_CREAM: Rgb = rgb(170, 232, 190);
pub(super) const BERRY_CREAM: Rgb = rgb(246, 170, 190);
/// the pretzel's coarse salt, and the cinnamon pretzel's sugar
pub(super) const SALT: Rgb = rgb(250, 250, 248);
pub(super) const CINNAMON_SUGAR: Rgb = rgb(232, 206, 160);
/// the circus cookie's icings
pub(super) const ICING_PINK: Rgb = rgb(246, 150, 190);
pub(super) const ICING_WHITE: Rgb = rgb(252, 248, 244);
/// rainbow nonpareils
pub(super) const SPRINKLES: [Rgb; 6] = [
    rgb(232, 48, 56),
    rgb(250, 150, 30),
    rgb(250, 220, 50),
    rgb(70, 190, 90),
    rgb(60, 130, 230),
    rgb(150, 80, 200),
];
/// the gummy jaguar
pub(super) const GUMMY_COAT: Rgb = rgb(214, 30, 58);
pub(super) const GUMMY_DEEP: Rgb = rgb(168, 16, 42);
pub(super) const GUMMY_CREAM: Rgb = rgb(246, 110, 130);
pub(super) const GUMMY_SPOT: Rgb = rgb(116, 8, 28);
pub(super) const GUMMY_EYE: Rgb = rgb(255, 236, 140);
pub(super) const GUMMY_SHINE: Rgb = rgb(255, 214, 224);
pub(super) const GUMMY_GLOSS: Rgb = rgb(255, 150, 170);

/// One kind's snack in one flavour: the five body colours `Look` names
/// (fur, limbs, chest, face, skin), then the snack's own two marks (the
/// cracker's toast and holes, the cookie's filling, the gummy's sugar, the
/// pretzel's salt, the puff's crust, the circus cookie's tail icing).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Flavour {
    pub coat: [Rgb; 5],
    pub mark: Rgb,
    pub other: Rgb,
}

/// `kind`'s snack in palette `palette` (0 the classic; 1 to 3 other
/// flavours; anything past the last is the classic).
pub(super) fn flavour(kind: Kind, palette: u8) -> Flavour {
    let f = |coat: [Rgb; 5], mark: Rgb, other: Rgb| Flavour { coat, mark, other };
    // the chocolate wafer, under every sandwich cookie but the golden one
    let wafer = [
        rgb(44, 30, 26),
        rgb(40, 27, 24),
        rgb(54, 37, 32),
        rgb(62, 44, 38),
        rgb(34, 23, 20),
    ];
    match (kind, palette) {
        // animal cracker: golden, cocoa, honey graham, vanilla
        (Kind::Chimpanzee, 1) => f(
            [
                rgb(120, 72, 44),
                rgb(108, 64, 38),
                rgb(136, 86, 54),
                rgb(158, 106, 70),
                rgb(96, 56, 32),
            ],
            rgb(72, 40, 22),
            SUGAR,
        ),
        (Kind::Chimpanzee, 2) => f(
            [
                rgb(196, 124, 56),
                rgb(184, 112, 48),
                rgb(208, 140, 70),
                rgb(224, 162, 94),
                rgb(170, 100, 40),
            ],
            rgb(132, 76, 28),
            SUGAR,
        ),
        (Kind::Chimpanzee, 3) => f(
            [
                rgb(236, 214, 168),
                rgb(226, 202, 152),
                rgb(242, 224, 184),
                rgb(246, 232, 198),
                rgb(214, 186, 130),
            ],
            rgb(186, 148, 90),
            SUGAR,
        ),
        (Kind::Chimpanzee, _) => f(
            [
                rgb(218, 166, 92),
                rgb(206, 150, 76),
                rgb(228, 182, 110),
                rgb(236, 198, 136),
                rgb(196, 136, 66),
            ],
            CRACKER_TOAST,
            SUGAR,
        ),
        // sandwich cookie: chocolate and cream, golden and cream,
        // chocolate and mint, chocolate and strawberry
        (Kind::Gorilla, 1) => f(
            [
                rgb(214, 168, 92),
                rgb(204, 156, 82),
                rgb(222, 178, 104),
                rgb(228, 188, 118),
                rgb(190, 142, 72),
            ],
            CREAM,
            rgb(170, 120, 54),
        ),
        (Kind::Gorilla, 2) => f(wafer, MINT_CREAM, rgb(24, 16, 14)),
        (Kind::Gorilla, 3) => f(wafer, BERRY_CREAM, rgb(24, 16, 14)),
        (Kind::Gorilla, _) => f(wafer, CREAM, rgb(24, 16, 14)),
        // fruit snack: orange, strawberry, lime, grape
        (Kind::Orangutan, 1) => f(
            [
                rgb(226, 36, 56),
                rgb(214, 30, 50),
                rgb(240, 80, 96),
                rgb(246, 120, 130),
                rgb(196, 24, 44),
            ],
            SUGAR_DUST,
            rgb(255, 170, 180),
        ),
        (Kind::Orangutan, 2) => f(
            [
                rgb(120, 200, 40),
                rgb(108, 188, 34),
                rgb(156, 220, 80),
                rgb(188, 234, 122),
                rgb(92, 168, 28),
            ],
            SUGAR_DUST,
            rgb(214, 250, 170),
        ),
        (Kind::Orangutan, 3) => f(
            [
                rgb(128, 48, 160),
                rgb(116, 42, 148),
                rgb(158, 82, 188),
                rgb(188, 122, 212),
                rgb(100, 34, 128),
            ],
            SUGAR_DUST,
            rgb(222, 180, 246),
        ),
        (Kind::Orangutan, _) => f(
            [
                rgb(240, 104, 30),
                rgb(232, 92, 24),
                rgb(250, 140, 64),
                rgb(252, 162, 98),
                rgb(220, 80, 22),
            ],
            SUGAR_DUST,
            rgb(255, 196, 140),
        ),
        // soft pretzel: classic, chocolate-dipped, lightly baked, cinnamon
        (Kind::Gibbon, 1) => f(
            [
                rgb(70, 40, 24),
                rgb(62, 36, 22),
                rgb(84, 50, 30),
                rgb(50, 28, 18),
                rgb(58, 32, 20),
            ],
            SALT,
            rgb(150, 110, 84),
        ),
        (Kind::Gibbon, 2) => f(
            [
                rgb(204, 140, 64),
                rgb(194, 130, 56),
                rgb(216, 156, 80),
                rgb(150, 92, 40),
                rgb(180, 116, 48),
            ],
            SALT,
            rgb(246, 210, 150),
        ),
        (Kind::Gibbon, 3) => f(
            [
                rgb(176, 92, 44),
                rgb(166, 84, 38),
                rgb(194, 112, 60),
                rgb(120, 64, 30),
                rgb(150, 76, 32),
            ],
            CINNAMON_SUGAR,
            rgb(240, 180, 130),
        ),
        (Kind::Gibbon, _) => f(
            [
                rgb(156, 86, 30),
                rgb(146, 78, 26),
                rgb(174, 102, 42),
                rgb(110, 58, 20),
                rgb(132, 70, 24),
            ],
            SALT,
            rgb(236, 180, 110),
        ),
        // cheese puff: cheddar, flamin' hot, white cheddar, barbecue
        (Kind::Proboscis, 1) => f(
            [
                rgb(226, 40, 24),
                rgb(214, 34, 20),
                rgb(240, 80, 50),
                rgb(246, 112, 82),
                rgb(190, 28, 16),
            ],
            rgb(244, 70, 40),
            rgb(176, 22, 12),
        ),
        (Kind::Proboscis, 2) => f(
            [
                rgb(244, 232, 196),
                rgb(236, 222, 184),
                rgb(250, 242, 214),
                rgb(250, 236, 206),
                rgb(220, 204, 160),
            ],
            rgb(252, 244, 214),
            rgb(220, 206, 166),
        ),
        (Kind::Proboscis, 3) => f(
            [
                rgb(196, 96, 36),
                rgb(186, 86, 30),
                rgb(214, 120, 58),
                rgb(226, 146, 86),
                rgb(166, 74, 24),
            ],
            rgb(214, 112, 44),
            rgb(150, 66, 20),
        ),
        (Kind::Proboscis, _) => f(
            [
                rgb(250, 140, 24),
                rgb(244, 128, 20),
                rgb(254, 170, 60),
                rgb(252, 184, 92),
                rgb(226, 110, 16),
            ],
            rgb(255, 156, 36),
            rgb(220, 104, 14),
        ),
        // frosted circus cookie: pink on cookie, white on cookie, pink all
        // over, white all over
        (Kind::Lemur, 1) => f(
            [
                ICING_WHITE,
                rgb(236, 208, 156),
                ICING_PINK,
                rgb(248, 242, 238),
                rgb(226, 194, 140),
            ],
            ICING_PINK,
            ICING_WHITE,
        ),
        (Kind::Lemur, 2) => f(
            [
                ICING_PINK,
                rgb(240, 160, 198),
                ICING_WHITE,
                rgb(248, 242, 238),
                rgb(226, 194, 140),
            ],
            ICING_PINK,
            ICING_WHITE,
        ),
        (Kind::Lemur, 3) => f(
            [
                ICING_WHITE,
                rgb(248, 244, 240),
                ICING_PINK,
                rgb(248, 242, 238),
                rgb(226, 194, 140),
            ],
            ICING_PINK,
            ICING_WHITE,
        ),
        (Kind::Lemur, _) => f(
            [
                ICING_PINK,
                rgb(236, 208, 156),
                ICING_WHITE,
                rgb(248, 242, 238),
                rgb(226, 194, 140),
            ],
            ICING_PINK,
            ICING_WHITE,
        ),
    }
}

/// The ape's build `l` (its kind's, in its palette) recoloured as its
/// snack: the shape is kept, only the five body colours change.
pub(super) fn look(kind: Kind, palette: u8, l: Look) -> Look {
    let [fur, limb, chest, face, skin] = flavour(kind, palette).coat;
    Look {
        fur,
        limb,
        chest,
        face,
        skin,
        ..l
    }
}

/// A point on the torso's surface: `y` percent up it, `round` degrees
/// from the belly toward the ape's left (180 the back), `out` percent of
/// its half width and depth.
fn on_torso(m: &Dims, s: &Skel, y: i32, round: i32, out: i32) -> V3 {
    let a = deg(round);
    s.body.at(
        mul(fx::sin(a), pct(m.w, out) / 2),
        pct(m.t, y),
        mul(fx::cos(a), pct(m.d, out) / 2),
    )
}

/// A point on the head's surface, as `on_torso` (`up` degrees above its
/// middle, `round` degrees from the face toward the left).
fn on_head(h: &Frame, d: i32, up: i32, round: i32) -> V3 {
    let (u, r) = (deg(up), deg(round));
    let rad = pct(d, 50);
    h.at(
        mul(mul(fx::cos(u), fx::sin(r)), rad),
        mul(fx::sin(u), rad),
        mul(mul(fx::cos(u), fx::cos(r)), rad),
    )
}

/// The arm on side `i`: shoulder, elbow and wrist, as `limbs` draws it.
fn arm(m: &Dims, s: &Skel, i: usize) -> [V3; 3] {
    let sh = shoulder_of(&s.body, m, SIDES[i]);
    let (elbow, wrist) = bend(sh, s.hands[i], m.arm, s.arm_hint[i]);
    [sh, elbow, wrist]
}

/// The leg on side `i`: hip, knee and ankle, as `limbs` draws it.
fn leg(m: &Dims, s: &Skel, i: usize) -> [V3; 3] {
    let hip = hip_of(&s.body, m, SIDES[i]);
    let (knee, ankle) = bend(hip, s.feet[i], m.leg, s.leg_hint[i]);
    [hip, knee, ankle]
}

/// The tail's links (from, to, width) as `tail` draws them, the same
/// numbers from the same skeleton and tick.
fn tail_links(kind: Kind, m: &Dims, s: &Skel, tick: u32) -> Vec<(V3, V3, i32)> {
    let (count, seg, w0, w1, bend_by) = match kind {
        Kind::Lemur => (12, cm(8), cm(9), cm(7), s.tail_bend),
        Kind::Proboscis => (8, cm(9), cm(6), cm(4), s.tail_bend / 3),
        _ => return Vec::new(),
    };
    let (lift, curl) = match kind {
        Kind::Lemur => (cm(40), s.tail_curl),
        _ => (cm(30), V3::UP.neg()),
    };
    let back = dir_or(s.tail_back.add(V3::UP.scale(lift)), s.body.z.neg());
    let curl = dir_or(curl.reject(back), s.body.z.neg());
    let side = back.cross(curl).norm();
    let mut at = s.body.at(0, pct(m.t, 10), -pct(m.d, 42));
    let mut out = Vec::new();
    for i in 0..count {
        let a = bend_by * (i + 1);
        let sway = mul(fx::sin(spin(tick, deg(3)) - i * deg(20)), cm(25));
        let way = dir_or(
            back.scale(fx::cos(a))
                .add(curl.scale(fx::sin(a)))
                .add(side.scale(sway)),
            back,
        );
        let next = at.add(way.scale(seg));
        out.push((at, next, w0 + (w1 - w0) * i / count));
        at = next;
    }
    out
}

/// A tiny diamond `size` across: a sugar speck, a salt grain.
fn speck(rig: &mut Rig, at: V3, size: i32, turn: i32, c: Rgb, glow: i32) {
    rig.glow(
        Mesh::Octa,
        Affine::translate(at)
            .then(&Affine::rot_y(turn))
            .then(&Affine::scale(v3(size, size, size))),
        c,
        glow,
    );
}

/// A soft glow over `a` (a unit ball's frame), drawn last and added over
/// the shape (`Batch::push_halo`): a strength from 2 to 3 is glass, a
/// crisp bright rim (the gloss); up to 1 a light from inside (the gummy's
/// translucence).
fn sheen(rig: &mut Rig, a: Affine, c: Rgb, strength: i32) {
    let a = rig.base.then(&a);
    rig.b.push_halo(Mesh::Sphere, &a, c, strength);
}

/// Everything a food ape has over its recoloured body, by kind.
#[allow(clippy::too_many_arguments)]
pub(super) fn food(
    rig: &mut Rig,
    kind: Kind,
    palette: u8,
    l: &Look,
    m: &Dims,
    s: &Skel,
    pose: &Pose,
) {
    let f = flavour(kind, palette);
    match kind {
        Kind::Chimpanzee => cracker(rig, l, m, s, &f),
        Kind::Gorilla => sandwich_cookie(rig, l, m, s, &f),
        Kind::Orangutan => fruit_snack(rig, l, m, s, &f),
        Kind::Gibbon => pretzel(rig, l, m, s, &f),
        Kind::Proboscis => cheese_puff(rig, l, m, s, &f, pose),
        Kind::Lemur => circus_cookie(rig, l, m, s, &f, pose.tick),
    }
}

/// The animal cracker: one baked biscuit shaped like an ape, its edges
/// toasted darker (a rim behind the head, seams down the torso's sides),
/// pressed docking holes on the chest and back, a light sugar sparkle.
fn cracker(rig: &mut Rig, l: &Look, m: &Dims, s: &Skel, f: &Flavour) {
    let (h, d) = (&s.head, cm(l.head));
    let toast = f.mark;
    // the toasted rim, seen round the head and down the sides
    disc(
        rig,
        h,
        v3(0, 0, -pct(d, 12)),
        pct(d, 108),
        pct(d, 12),
        toast,
    );
    for side in SIDES {
        rig.blade(
            on_torso(m, s, 92, side * 90, 98),
            on_torso(m, s, 22, side * 90, 90),
            s.body.z,
            pct(m.d, 46),
            pct(m.w, 10),
            toast,
        );
    }
    // the docking holes: three on the chest, three on the back
    for (y, round) in [
        (70, -22),
        (70, 22),
        (50, 0),
        (72, 160),
        (72, 200),
        (52, 180),
    ] {
        let at = on_torso(m, s, y, round, if round < 90 { 112 } else { 102 });
        let out = dir_or(at.sub(s.body.at(0, pct(m.t, y), 0)), s.body.z);
        rig.put(
            Mesh::Hex,
            rod(
                at.sub(out.scale(pct(m.d, 4))),
                at.add(out.scale(pct(m.d, 3))),
                pct(m.w, 12),
            ),
            docking(toast),
        );
    }
    // the sugar sparkle
    let grain = pct(d, 7);
    for (i, &(up, round)) in [(60, 20), (70, -40), (44, 150), (52, 210)]
        .iter()
        .enumerate()
    {
        speck(
            rig,
            on_head(h, d, up, round),
            grain,
            i as i32 * deg(37),
            f.other,
            ONE,
        );
    }
    for (i, &(y, round)) in [(94, 60), (94, -60), (80, 180), (60, 140), (40, 220)]
        .iter()
        .enumerate()
    {
        speck(
            rig,
            on_torso(m, s, y, round, 104),
            grain,
            i as i32 * deg(53),
            f.other,
            ONE,
        );
    }
}

/// The docking holes are pressed a little darker than the toast.
fn docking(toast: Rgb) -> Rgb {
    if toast == CRACKER_TOAST {
        DOCKING
    } else {
        shade(toast, 86)
    }
}

/// The sandwich cookie: dark wafer all over, a thick band of filling round
/// the waist and a thinner one at the neck, and the wafer's stamp on the
/// chest (a raised disc, a ring of dots round a six-petal flower; no
/// letters, no logo). The wafer covers the silver back too.
fn sandwich_cookie(rig: &mut Rig, l: &Look, m: &Dims, s: &Skel, f: &Flavour) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    let (h, hd) = (&s.head, cm(l.head));
    // the wafer over the silver saddle
    rig.put(
        Mesh::Geo,
        body.shape(at(0, 46, -27), at(88, 76, 58)),
        shade(l.fur, 92),
    );
    // the filling: a thick band round the waist, a ring at the neck
    rig.put(
        Mesh::Hex,
        body.shape(at(0, 40, 2), at(114, 20, 124)),
        f.mark,
    );
    rig.put(
        Mesh::Hex,
        h.shape(
            v3(0, -pct(hd, 46), -pct(hd, 2)),
            v3(pct(hd, 84), pct(hd, 14), pct(hd, 88)),
        ),
        f.mark,
    );
    // the stamp: a raised disc on the chest with an edge, a ring of
    // twelve dots and a flower in the middle
    let face = body.z;
    let middle = body.at(0, pct(t, 68), pct(d, 58));
    let across = pct(w, 54);
    rig.put(
        Mesh::Hex,
        rod(
            middle.sub(face.scale(pct(d, 4))),
            middle.add(face.scale(pct(d, 3))),
            across + pct(w, 6),
        ),
        f.other,
    );
    rig.put(
        Mesh::Hex,
        rod(middle, middle.add(face.scale(pct(d, 5))), across),
        stamp(l.fur, false),
    );
    let front = middle.add(face.scale(pct(d, 5)));
    let r = across * 38 / 100;
    for i in 0..12 {
        let a = i * TURN / 12;
        let p = front
            .add(body.x.scale(mul(fx::sin(a), r)))
            .add(body.y.scale(mul(fx::cos(a), r)));
        rig.ball(p, pct(w, 6), stamp(l.fur, true));
    }
    for i in 0..6 {
        let a = i * TURN / 6;
        let p = front
            .add(body.x.scale(mul(fx::sin(a), r / 2)))
            .add(body.y.scale(mul(fx::cos(a), r / 2)));
        rig.put(
            Mesh::Octa,
            Affine {
                c0: body.x.scale(pct(w, 7)),
                c1: body.y.scale(pct(w, 7)),
                c2: face.scale(pct(d, 4)),
                t: p,
            },
            stamp(l.fur, true),
        );
    }
    rig.ball(front, pct(w, 8), stamp(l.fur, true));
}

/// The stamp's colour on wafer `fur`: the disc (or, `raised`, its dots and
/// flower) a little lighter than the wafer, more on a dark one.
pub(super) fn stamp(fur: Rgb, raised: bool) -> Rgb {
    let dark = fur[0] < ONE / 2;
    shade(
        fur,
        if dark {
            150 + 40 * raised as i32
        } else {
            108 + 8 * raised as i32
        },
    )
}

/// The fruit snack: glossy, see-through-looking gummy (a light from
/// inside and a glass rim round the torso and the head, pale highlights
/// where the light would catch it) with a dusting of sugar.
fn fruit_snack(rig: &mut Rig, l: &Look, m: &Dims, s: &Skel, f: &Flavour) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    let (h, hd) = (&s.head, cm(l.head));
    let shine = f.other;
    sheen(
        rig,
        body.shape(at(0, 56, 0), at(108, 94, 108)),
        mix(l.fur, shine, ONE / 2),
        GLASS + ONE * 3 / 5,
    );
    sheen(
        rig,
        body.shape(at(0, 52, 0), at(96, 84, 96)),
        l.fur,
        ONE * 2 / 5,
    );
    sheen(
        rig,
        h.shape(V3::ZERO, v3(pct(hd, 108), pct(hd, 102), pct(hd, 104))),
        mix(l.fur, shine, ONE / 2),
        GLASS + ONE / 2,
    );
    // highlights: a long one down the back, one on the chest, one on the
    // crown
    rig.glow(
        Mesh::Geo,
        body.shape(at(-20, 70, -48), at(18, 40, 8)),
        shine,
        ONE * 2 / 3,
    );
    rig.glow(
        Mesh::Geo,
        body.shape(at(22, 76, 50), at(16, 24, 6)),
        shine,
        ONE * 2 / 3,
    );
    rig.glow(
        Mesh::Geo,
        h.shape(
            v3(-pct(hd, 16), pct(hd, 40), pct(hd, 10)),
            v3(pct(hd, 26), pct(hd, 10), pct(hd, 18)),
        ),
        shine,
        ONE * 2 / 3,
    );
    // the sugar
    let grain = pct(hd, 6);
    for (i, &(up, round)) in [(64, 0), (50, 60), (50, -60), (40, 160), (30, 120)]
        .iter()
        .enumerate()
    {
        speck(
            rig,
            on_head(h, hd, up, round),
            grain,
            i as i32 * deg(41),
            f.mark,
            ONE / 2,
        );
    }
    for (i, &(y, round)) in [
        (96, 40),
        (96, -40),
        (84, 180),
        (66, 150),
        (66, 210),
        (44, 170),
        (60, 30),
        (40, -30),
    ]
    .iter()
    .enumerate()
    {
        speck(
            rig,
            on_torso(m, s, y, round, 106),
            grain,
            i as i32 * deg(29),
            f.mark,
            ONE / 2,
        );
    }
    // the cheek flanges in gummy too
    disc(
        rig,
        h,
        v3(0, -pct(hd, 6), pct(hd, 15)),
        pct(hd, 180),
        pct(hd, 14),
        shade(l.fur, 92),
    );
}

/// The soft pretzel: glossy baked brown, coarse salt on the head, back
/// and arms, and its long arms the pretzel's twisted rope (diagonal
/// creases winding down each).
fn pretzel(rig: &mut Rig, l: &Look, m: &Dims, s: &Skel, f: &Flavour) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    let (h, hd) = (&s.head, cm(l.head));
    sheen(
        rig,
        body.shape(at(0, 56, 0), at(110, 94, 110)),
        f.other,
        GLASS + ONE * 2 / 5,
    );
    sheen(
        rig,
        h.shape(V3::ZERO, v3(pct(hd, 106), pct(hd, 100), pct(hd, 102))),
        f.other,
        GLASS + ONE * 2 / 5,
    );
    let grain = pct(hd, 12);
    let mut n = 0;
    let mut salt = |rig: &mut Rig, p: V3| {
        n += 1;
        rig.put(
            Mesh::Cube,
            Affine::translate(p)
                .then(&Affine::rot_y(n * deg(47)))
                .then(&Affine::scale(v3(grain, grain * 4 / 5, grain))),
            f.mark,
        );
    };
    for (up, round) in [(70, 0), (56, 50), (56, -50), (46, 150), (40, 210)] {
        salt(rig, on_head(h, hd, up, round));
    }
    for (y, round) in [
        (92, 160),
        (92, 200),
        (72, 180),
        (56, 150),
        (56, 210),
        (36, 180),
    ] {
        salt(rig, on_torso(m, s, y, round, 104));
    }
    for i in 0..2 {
        let [sh, elbow, wrist] = arm(m, s, i);
        for (a, b, wide) in [(sh, elbow, 130), (elbow, wrist, 112)] {
            let along = b.sub(a);
            let way = dir_or(along, V3::UP);
            let across = dir_or(way.cross(body.z), body.x);
            let out = way.cross(across).norm();
            let r = pct(m.arm_w, wide) / 2;
            // the twist: two creases winding round each segment, front
            // and back
            for k in 0..2 {
                for face in SIDES {
                    let from = a.lerp(b, ONE * (2 * k + 1) / 6);
                    let to = a.lerp(b, ONE * (2 * k + 2) / 6);
                    rig.blade(
                        from.add(across.scale(r * 3 / 4)).add(out.scale(face * r)),
                        to.sub(across.scale(r * 3 / 4)).add(out.scale(face * r)),
                        way,
                        pct(m.arm_w, 26),
                        pct(m.arm_w, 12),
                        shade(l.limb, 72),
                    );
                }
            }
            // salt on top of the arm, clear of the hands
            salt(rig, a.lerp(b, ONE * 2 / 5).add(V3::UP.scale(r)));
        }
    }
}

/// The cheese puff: bright puffed orange with a lumpy crust (small blobs
/// all over), the pale cape and the grey tail puffed orange too, and its
/// big nose a puffed orange curl.
fn cheese_puff(rig: &mut Rig, l: &Look, m: &Dims, s: &Skel, f: &Flavour, pose: &Pose) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    let (h, hd) = (&s.head, cm(l.head));
    let (light, dark) = (f.mark, f.other);
    // the cape round the shoulders, puffed
    rig.put(
        Mesh::Barrel,
        body.shape(at(0, 90, 0), at(124, 34, 116)),
        shade(l.fur, 104),
    );
    // the crust: lumps over the torso (the pot belly is deeper), the head
    // and the upper limbs
    let lump = pct(w, 16);
    for (i, &(y, round, out)) in [
        (90, 0, 116),
        (88, 120, 116),
        (88, 240, 116),
        (70, 60, 104),
        (70, 180, 104),
        (70, 300, 104),
        (50, 20, 160),
        (50, 120, 102),
        (50, 240, 102),
        (32, 340, 150),
        (30, 180, 96),
        (24, 90, 96),
        (24, 270, 96),
    ]
    .iter()
    .enumerate()
    {
        let c = if i % 2 == 0 { light } else { dark };
        rig.ball(on_torso(m, s, y, round, out), lump, c);
    }
    for (i, &(up, round)) in [(60, 30), (60, -30), (40, 150), (40, 210)]
        .iter()
        .enumerate()
    {
        let c = if i % 2 == 0 { light } else { dark };
        rig.ball(on_head(h, hd, up, round), pct(hd, 22), c);
    }
    for i in 0..2 {
        let [sh, elbow, _] = arm(m, s, i);
        rig.ball(sh.lerp(elbow, ONE / 2), pct(m.arm_w, 150), light);
        let [hip, knee, _] = leg(m, s, i);
        rig.ball(hip.lerp(knee, ONE / 2), pct(m.leg_w, 150), dark);
    }
    // the nose: a puffed curl over the bare one, lumps along it
    let n = nose_points(h, hd, pose.vel, pose.tick);
    let p = |x: i32, y: i32, z: i32| v3(pct(hd, x), pct(hd, y), pct(hd, z));
    rig.put(Mesh::Ico, h.shape(V3::ZERO, p(32, 32, 32)).at(n[0]), light);
    for (i, size) in [36, 42, 50].into_iter().enumerate() {
        let c = if i % 2 == 0 { dark } else { light };
        rig.put(
            Mesh::Ico,
            h.shape(V3::ZERO, p(size, size, size))
                .at(n[i].lerp(n[i + 1], ONE / 2)),
            c,
        );
        rig.put(
            Mesh::Taper,
            rod(n[i + 1], n[i], pct(hd, size * 9 / 10)),
            l.fur,
        );
    }
    rig.put(Mesh::Geo, h.shape(V3::ZERO, p(66, 60, 62)).at(n[3]), l.fur);
    rig.put(
        Mesh::Ico,
        h.shape(V3::ZERO, p(30, 30, 30))
            .at(n[3].add(h.z.scale(pct(hd, 22)))),
        light,
    );
    // the tail
    for (i, (a, b, wd)) in tail_links(Kind::Proboscis, m, s, pose.tick)
        .into_iter()
        .enumerate()
    {
        rig.put(Mesh::Taper, rod(a, b, pct(wd, 150)), l.fur);
        if i % 2 == 1 {
            rig.ball(b, pct(wd, 140), if i % 4 == 1 { light } else { dark });
        }
    }
}

/// The frosted circus cookie: pale cookie under pink and white icing (the
/// body iced, the limbs bare cookie or iced by flavour), rainbow
/// nonpareils on the back, shoulders and head, and the ringed tail iced
/// in pink and white rings.
fn circus_cookie(rig: &mut Rig, l: &Look, m: &Dims, s: &Skel, f: &Flavour, tick: u32) {
    let (h, hd) = (&s.head, cm(l.head));
    let bead = pct(hd, 9);
    let mut k = 0;
    for &(y, round) in &[
        (94, 50),
        (94, -50),
        (90, 180),
        (80, 140),
        (80, 220),
        (68, 170),
        (64, 200),
        (54, 150),
        (50, 190),
        (40, 220),
    ] {
        rig.ball(on_torso(m, s, y, round, 104), bead, SPRINKLES[k % 6]);
        k += 1;
    }
    // (round the sides and back of the head: its top is right under the
    // hands on a vine)
    for &(up, round) in &[(6, 100), (6, -100), (10, 160), (10, 200)] {
        rig.ball(on_head(h, hd, up, round), bead, SPRINKLES[k % 6]);
        k += 1;
    }
    // the tail's icing rings, over its black and white ones
    let links = tail_links(Kind::Lemur, m, s, tick);
    for (i, &(a, b, w)) in links.iter().enumerate() {
        let c = if i % 2 == 0 { f.other } else { f.mark };
        rig.put(Mesh::Hex, rod(a, b, pct(w, 114)), c);
    }
    if let Some(&(a, b, w)) = links.last() {
        let tip = b.add(b.sub(a));
        rig.put(Mesh::Spike, rod(b, tip, pct(w, 112)), f.mark);
    }
}

/// The gummy jaguar's gloss over its raspberry body (`GUMMY_JAGUAR`): a
/// light from inside, a glass rim and two pale highlights along its back.
pub(super) fn gummy_cat(rig: &mut Rig, body: &Frame) {
    let p = |x: i32, y: i32, z: i32| v3(cm(x), cm(y), cm(z));
    sheen(
        rig,
        body.shape(p(0, 0, 0), p(50, 122, 54)),
        GUMMY_GLOSS,
        GLASS + ONE * 3 / 5,
    );
    sheen(
        rig,
        body.shape(p(0, 0, 0), p(40, 104, 44)),
        GUMMY_COAT,
        ONE * 2 / 5,
    );
    for (along, x, len) in [(24, -8, 40), (-30, 8, 30)] {
        rig.glow(
            Mesh::Geo,
            body.shape(p(x, along, 24), p(8, len, 4)),
            GUMMY_SHINE,
            ONE * 2 / 3,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{at_rest, poses, triangles, Shape};
    use super::*;
    use crate::render::{DYNAMIC_BUFFER, INSTANCE_VALUES};

    /// A batch's shapes (halos aside), as `models::tests` reads them.
    fn read(b: Batch) -> Vec<Shape> {
        let (values, draws) = b.finish(DYNAMIC_BUFFER);
        let mut out = Vec::new();
        for d in draws.chunks(crate::render::DRAW_VALUES) {
            let first = d[2] as usize / 4 / INSTANCE_VALUES;
            for i in first..first + d[3] as usize {
                let v = &values[i * INSTANCE_VALUES..(i + 1) * INSTANCE_VALUES];
                out.push(Shape {
                    mesh: d[1],
                    axes: [
                        v3(v[0], v[4], v[8]),
                        v3(v[1], v[5], v[9]),
                        v3(v[2], v[6], v[10]),
                    ],
                    at: v3(v[3], v[7], v[11]),
                    colour: [v[12], v[13], v[14]],
                    glow: v[15],
                });
            }
        }
        out
    }

    /// `kind` drawn in `palette` and `pose`, and its halos' count.
    fn drawn_in(kind: Kind, palette: u8, pose: &Pose) -> (Vec<Shape>, usize) {
        let mut b = Batch::new();
        draw_in(&mut b, kind, palette, pose);
        let halos = b.take_halos().count();
        (read(b), halos)
    }

    /// The colours only the food draws, which mark each kind's snack.
    fn marks(kind: Kind, palette: u8) -> Vec<Rgb> {
        let f = flavour(kind, palette);
        match kind {
            Kind::Chimpanzee => vec![f.mark, f.other, docking(f.mark)],
            Kind::Gorilla => vec![f.mark, f.other],
            Kind::Orangutan => vec![f.mark, f.other],
            Kind::Gibbon => vec![f.mark, shade(f.coat[1], 72)],
            Kind::Proboscis => vec![f.mark, f.other],
            Kind::Lemur => {
                let mut c = vec![f.mark, f.other];
                c.extend(SPRINKLES);
                c
            }
        }
    }

    #[test]
    fn on_clean_up_on_aisle_ape_every_kind_is_its_own_snack_in_every_palette_and_pose() {
        // the person, 2026-10-18 (briefs/clean-up-on-aisle-ape.md): "the
        // apes in this map are all little food items"; each kind keeps its
        // body and becomes one snack (GR-Q10)
        assert_eq!(Gear::from_key("food"), Some(Gear::Food));
        for map in crate::map::Map::ALL {
            assert_eq!(
                Gear::on(map) == Gear::Food,
                map.key() == "grocery",
                "{map:?}"
            );
        }
        for kind in Kind::ALL {
            // every flavour its own colours
            for a in 0..PALETTES {
                for b in a + 1..PALETTES {
                    assert_ne!(flavour(kind, a), flavour(kind, b), "{kind:?} {a} {b}");
                }
            }
            assert_eq!(flavour(kind, PALETTES), flavour(kind, 0));
            for palette in 0..PALETTES {
                let f = flavour(kind, palette);
                for p in poses() {
                    let pose = Pose {
                        gear: Gear::Food,
                        ..p
                    };
                    let (s, halos) = drawn_in(kind, palette, &pose);
                    let (bare, _) = drawn_in(kind, palette, &p);
                    assert!(s.len() > bare.len() + 8, "{kind:?} {palette}");
                    assert!(
                        (50..=300).contains(&s.len()),
                        "{kind:?} {palette}: {} shapes",
                        s.len()
                    );
                    let has = |c: Rgb| s.iter().any(|x| x.colour == c);
                    // the body is the snack's, all five colours
                    for c in f.coat {
                        assert!(has(c), "{kind:?} {palette} {pose:?}: no body colour {c:?}");
                    }
                    for c in marks(kind, palette) {
                        assert!(has(c), "{kind:?} {palette} {pose:?}: no mark {c:?}");
                    }
                    // the ape's own fur is gone, its eyes kept
                    let own = look_in(kind, palette);
                    assert!(!has(own.fur), "{kind:?} {palette}: its own fur");
                    let eye = if kind == Kind::Lemur {
                        LEMUR_EYE
                    } else {
                        EYE_WHITE
                    };
                    assert_eq!(s.iter().filter(|x| x.colour == eye).count(), 2);
                    // the gummy and the pretzel are glossy
                    let glossy = matches!(kind, Kind::Orangutan | Kind::Gibbon);
                    assert_eq!(halos > 0, glossy, "{kind:?}: {halos} halos");
                    // no other costume's pieces
                    for c in [JACKET, KHAKI, PITH, TRICORN, cowboy::HAT_SAND] {
                        assert!(!has(c), "{kind:?}: {c:?}");
                    }
                    for x in &s {
                        for a in x.axes {
                            assert!(
                                a.len() >= ONE / 500 && a.len() <= int(2),
                                "{kind:?} mesh {} colour {:?} axis {}",
                                x.mesh,
                                x.colour,
                                a.len()
                            );
                        }
                        assert!(x.at.dist(pose.hands) <= int(3));
                    }
                    // swinging, the hands stay free: no mark at the grip
                    if pose.rope.is_some() {
                        // (a mark the body shares, the lemur's pink
                        // icing on its ears, is the ape's own)
                        let mine: Vec<Rgb> = marks(kind, palette)
                            .into_iter()
                            .filter(|c| !f.coat.contains(c))
                            .collect();
                        for x in s.iter().filter(|x| mine.contains(&x.colour)) {
                            assert!(
                                x.at.dist(pose.hands) > cm(10),
                                "{kind:?}: {:?} mesh {} at the grip ({})",
                                x.colour,
                                x.mesh,
                                x.at.dist(pose.hands)
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn the_sandwich_cookie_is_stamped_with_dots_and_a_flower_round_a_filling_band() {
        // generic, never a logo: twelve dots, six petals, one middle dot
        let p = Pose {
            gear: Gear::Food,
            ..at_rest()
        };
        for palette in 0..PALETTES {
            let (s, _) = drawn_in(Kind::Gorilla, palette, &p);
            let l = look(Kind::Gorilla, palette, look_in(Kind::Gorilla, palette));
            let raised = stamp(l.fur, true);
            let dots = s.iter().filter(|x| x.colour == raised).count();
            assert_eq!(dots, 12 + 6 + 1, "palette {palette}");
            // the filling: the waist band and the neck ring
            let f = flavour(Kind::Gorilla, palette);
            let fill: Vec<&Shape> = s.iter().filter(|x| x.colour == f.mark).collect();
            assert_eq!(fill.len(), 2, "palette {palette}");
            let face = s.iter().find(|x| x.colour == l.face).unwrap().at;
            assert!(
                fill.iter().any(|x| x.at.y < face.y - cm(30)),
                "no waist band"
            );
        }
    }

    #[test]
    fn the_food_apes_cost_about_what_the_cowboys_do() {
        let mut lines = Vec::new();
        let (mut food_all, mut cowboy_all) = (0, 0);
        for kind in Kind::ALL {
            let p = at_rest();
            let tris = |g: Gear| triangles(&drawn_in(kind, 0, &Pose { gear: g, ..p }).0);
            let shapes = |g: Gear| drawn_in(kind, 0, &Pose { gear: g, ..p }).0.len();
            let bare = tris(Gear::None);
            let (food, cowboy) = (tris(Gear::Food) - bare, tris(Gear::Cowboy) - bare);
            lines.push(format!(
                "{kind:?}: bare {bare} ({} shapes), cowboy +{cowboy} ({} shapes), food +{food} ({} shapes)",
                shapes(Gear::None),
                shapes(Gear::Cowboy),
                shapes(Gear::Food),
            ));
            food_all += food;
            cowboy_all += cowboy;
        }
        let table = lines.join("; ");
        assert!(food_all <= cowboy_all * 5 / 4, "{table}");
        for l in &lines {
            let n: Vec<usize> = l
                .split('+')
                .skip(1)
                .map(|x| {
                    x.split(|c: char| !c.is_ascii_digit())
                        .next()
                        .unwrap()
                        .parse()
                        .unwrap()
                })
                .collect();
            assert!(n[1] <= n[0] * 3 / 2, "{table}");
        }
    }

    #[test]
    fn only_the_grocerys_cat_is_the_gummy_jaguar() {
        use crate::map::Map;
        for map in Map::ALL {
            assert_eq!(BigCat::on(map).gummy, map.key() == "grocery", "{map:?}");
            assert_eq!(BigCat::on(map) == GUMMY_JAGUAR, map.key() == "grocery");
        }
        const { assert!(GUMMY_JAGUAR.gummy && !JAGUAR.gummy && !SPIDER_JAGUAR.gummy) };
        let j = crate::jaguar::Jaguar {
            feet: V3::ZERO,
            yaw: deg(30),
            gait: deg(50),
            stride: ONE,
            leap: 0,
            head_down: 0,
            jaw: ONE / 2,
        };
        let draw = |cat: &BigCat| {
            let mut b = Batch::new();
            jaguar(&mut b, &j, 100, cat);
            let halos = b.take_halos().count();
            (read(b), halos)
        };
        let (s, halos) = draw(&GUMMY_JAGUAR);
        // still a jaguar, in gummy: its coat and rosettes red, glossy
        assert!(s.iter().any(|x| x.colour == GUMMY_COAT));
        assert!(s.iter().any(|x| x.colour == GUMMY_SPOT));
        assert!(!s.iter().any(|x| x.colour == JAG_COAT));
        assert_eq!(s.iter().filter(|x| x.colour == GUMMY_SHINE).count(), 2);
        assert_eq!(halos, 2);
        let (plain, plain_halos) = draw(&JAGUAR);
        assert_eq!(plain.len() + 2, s.len());
        assert_eq!(plain_halos, 0);
        for other in [
            JAGUAR,
            SPACE_JAGUAR,
            RUINS_JAGUAR,
            NAVY_JAGUAR,
            SNOW_LEOPARD,
            FLAME_CAT,
            SPIDER_JAGUAR,
        ] {
            let (o, _) = draw(&other);
            assert!(!o.iter().any(|x| x.colour == GUMMY_SHINE), "{other:?}");
        }
    }
}
