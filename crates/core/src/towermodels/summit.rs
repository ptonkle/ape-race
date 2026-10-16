//! The summit: the insane wizard ripping a tear in reality, the tear, and
//! the giant eldritch evil banana seen through it.

use super::keep::{keep_crown, keep_section, Detail, CROWN_FLOOR};
use super::{bar, halo_ball, halo_bar, pick, pulse, put, put_turned, spell_beam, wave, Spell};
use super::{BEARD, FACE, FUR, GOLD};
use crate::fx::{cm, cos, deg, div, int, mul, sin, HALF, ONE, TURN};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, rgb, Batch, Rgb};

const WIZ_ROBE: Rgb = rgb(66, 40, 104);
const WIZ_ROBE_DARK: Rgb = rgb(42, 26, 70);
const WILD_EYE: Rgb = rgb(255, 244, 130);
const CRACKLE: Rgb = rgb(255, 170, 255);
const RIFT_HOT: Rgb = rgb(255, 214, 255);
const RIFT_EDGE: Rgb = rgb(200, 60, 255);
const RIFT_LIP: Rgb = rgb(26, 8, 44);
const BAN_YELLOW: Rgb = rgb(244, 214, 60);
const BAN_GOLD: Rgb = rgb(196, 158, 36);
const BAN_ROT: Rgb = rgb(36, 24, 14);
const BAN_BRUISE: Rgb = rgb(24, 18, 10);
const BAN_FLESH: Rgb = rgb(222, 206, 150);
const EYE_RED: Rgb = rgb(255, 34, 22);
const EYE_SLIT: Rgb = rgb(8, 0, 0);
const TENTACLE: Rgb = rgb(70, 32, 82);
const TENTACLE_TIP: Rgb = rgb(150, 60, 140);
const MAW: Rgb = rgb(150, 20, 40);
const FANG: Rgb = rgb(236, 230, 206);
const AURA: Rgb = rgb(96, 20, 150);
const WISP: Rgb = rgb(34, 12, 50);
const VOID: Rgb = rgb(44, 14, 78);
const VOID_DEEP: Rgb = rgb(26, 8, 52);
const VOID_STAR: Rgb = rgb(220, 180, 255);

/// The insane wizard's hands, in his frame: the tear's edges are pulled
/// from here.
pub const WIZARD_HANDS: [V3; 2] = [
    v3(-int(2), int(3) - cm(40), cm(90)),
    v3(int(2), int(3) - cm(40), cm(90)),
];

/// The insane wizard, feet at the origin facing +z, about 4 m tall: a big
/// old ape in tattered robes, a wild beard and hair flying, a crumpled
/// hat, arms flung apart with crackling magic at both hands. About 50
/// shapes.
pub fn mad_wizard(b: &mut Batch, f: &Affine, tick: u32) {
    let flap = |i: i32| mul(wave(tick, 1, i * 6_000), deg(14));
    put(
        b,
        f,
        Mesh::Cone,
        v3(0, cm(85), 0),
        v3(cm(140), cm(170), cm(120)),
        WIZ_ROBE,
        0,
    );
    // tatters round the hem
    for i in 0..7 {
        let a = TURN * i / 7;
        let g = f
            .then(&Affine::translate(super::round(cm(62), a, cm(25))))
            .then(&Affine::rot_y(a))
            .then(&Affine::rot_x(flap(i)));
        put(
            b,
            &g,
            Mesh::Shard,
            v3(0, -cm(10), 0),
            v3(cm(40), cm(60), cm(6)),
            WIZ_ROBE_DARK,
            0,
        );
    }
    put(
        b,
        f,
        Mesh::Ico,
        v3(0, cm(185), 0),
        v3(cm(95), cm(85), cm(75)),
        WIZ_ROBE,
        0,
    );
    put(
        b,
        f,
        Mesh::Ico,
        v3(0, cm(250), 0),
        v3(cm(75), cm(72), cm(70)),
        FUR,
        0,
    );
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, cm(244), cm(32)),
        v3(cm(52), cm(36), cm(12)),
        FACE,
        0,
    );
    // mad eyes, one bigger, and a shouting mouth
    put(
        b,
        f,
        Mesh::Octa,
        v3(-cm(13), cm(254), cm(39)),
        v3(cm(16), cm(16), cm(8)),
        WILD_EYE,
        ONE,
    );
    put(
        b,
        f,
        Mesh::Octa,
        v3(cm(13), cm(256), cm(39)),
        v3(cm(22), cm(22), cm(8)),
        WILD_EYE,
        ONE,
    );
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, cm(232), cm(39)),
        v3(cm(20), cm(14), cm(6)),
        rgb(40, 10, 16),
        0,
    );
    // the beard, fanned out and blowing
    let chin = v3(0, cm(228), cm(34));
    for i in 0..7 {
        let a = deg(-60) + deg(20) * i + flap(i) / 2;
        let g = f
            .then(&Affine::translate(chin))
            .then(&Affine::rot_z(a))
            .then(&Affine::rot_x(deg(20)));
        put(
            b,
            &g,
            Mesh::Shard,
            v3(0, -cm(55), cm(10)),
            v3(cm(30), cm(110), cm(8)),
            BEARD,
            ONE / 8,
        );
    }
    // wild hair sticking out from the sides and back
    for i in 0..6 {
        let a = deg(70) + TURN * i / 10 + flap(i + 3) / 2;
        let g = f
            .then(&Affine::translate(v3(0, cm(262), -cm(5))))
            .then(&Affine::rot_y(a))
            .then(&Affine::rot_z(-deg(70)));
        put(
            b,
            &g,
            Mesh::Shard,
            v3(0, cm(45), 0),
            v3(cm(26), cm(70), cm(6)),
            BEARD,
            ONE / 8,
        );
    }
    // the crumpled hat
    put_turned(
        b,
        f,
        Mesh::Cyl,
        v3(0, cm(284), 0),
        &Affine::rot_z(deg(10)),
        v3(cm(120), cm(8), cm(120)),
        WIZ_ROBE_DARK,
        0,
    );
    let h1 = v3(cm(10), cm(290), 0);
    let h2 = v3(cm(25), cm(345), -cm(10));
    let h3 = v3(cm(70), cm(370), -cm(25));
    bar(b, f, Mesh::Taper, h1, h2, cm(70), WIZ_ROBE_DARK, 0);
    bar(b, f, Mesh::Taper, h2, h3, cm(40), WIZ_ROBE_DARK, 0);
    bar(
        b,
        f,
        Mesh::Spike,
        h3,
        h3.add(v3(cm(45), -cm(15), -cm(10))),
        cm(22),
        WIZ_ROBE_DARK,
        0,
    );
    put(
        b,
        f,
        Mesh::Octa,
        v3(cm(18), cm(318), cm(26)),
        v3(cm(20), cm(20), cm(6)),
        GOLD,
        ONE / 2,
    );
    // arms flung apart, bell sleeves, crackling hands
    for (side, hand) in [(-1, WIZARD_HANDS[0]), (1, WIZARD_HANDS[1])] {
        let shoulder = v3(side * cm(50), cm(205), 0);
        let elbow = v3(side * cm(120), cm(230), cm(40));
        bar(b, f, Mesh::Taper, shoulder, elbow, cm(32), WIZ_ROBE, 0);
        bar(
            b,
            f,
            Mesh::Taper,
            hand.add(v3(-side * cm(20), -cm(5), -cm(10))),
            elbow,
            cm(48),
            WIZ_ROBE,
            0,
        );
        put(b, f, Mesh::Ico, hand, v3(cm(26), cm(26), cm(26)), FUR, 0);
        // a zigzag of lightning that jumps every few ticks
        let jump = tick / 4;
        let mut p = hand;
        for j in 0..3u32 {
            let q = p.add(v3(
                side * cm(30) + pick(jump * 7 + j, 1, -20, 20) * ONE / 100,
                pick(jump * 5 + j, 2, -40, 40) * ONE / 100,
                pick(jump * 3 + j, 3, -30, 30) * ONE / 100,
            ));
            bar(b, f, Mesh::Cube, p, q, cm(6), CRACKLE, ONE);
            p = q;
        }
        halo_ball(b, f, hand, int(1) + cm(40), CRACKLE, ONE * 3 / 4);
    }
}

/// The tear in reality, in the xy plane facing +z, `w` across at its
/// widest and `h` tall, its foot at y 0; `open` 0..ONE is how far it is
/// torn open. Jagged glowing edges with a dark torn lip inside them,
/// sparks flying off, and a dark violet void a few metres behind (see
/// `torn_rift`). About 130 shapes.
pub fn rift(b: &mut Batch, f: &Affine, w: i32, h: i32, open: i32, tick: u32) {
    torn_rift(b, f, w, h, open, int(3), tick);
}

/// How far in front of a tear its void is drawn to fit: the void `depth`
/// behind is scaled up by `(VIEW + depth) / VIEW` about the tear's foot,
/// so from about this far in front it exactly fills the opening.
pub const RIFT_VIEW: i32 = int(20);

/// The tear with its void `depth` behind it (-z): a jagged dark violet
/// backdrop with faint stars, scaled to fill the opening from `RIFT_VIEW`
/// in front, so whatever lies between (the banana) is seen only through
/// the tear.
pub fn torn_rift(b: &mut Batch, f: &Affine, w: i32, h: i32, open: i32, depth: i32, tick: u32) {
    let n = 12;
    let jump = tick / 6;
    let half = |i: i32| mul(mul(w / 2, sin(HALF * i / n)), open.clamp(0, ONE));
    let edge = |i: i32, s: i32| {
        let y = h * i / n;
        // jags alternate in and out, and shiver a little
        let jag = if (i + (s + 1) / 2) % 2 == 0 {
            w / 14
        } else {
            -w / 20
        };
        let shiver = pick(jump * 13 + (i * 2 + (s + 1) / 2) as u32, 4, -10, 10) * (w / 400);
        v3(s * (half(i) + jag + shiver), y, 0)
    };
    for s in [-1, 1] {
        for i in 0..n {
            let (p, q) = (edge(i, s), edge(i + 1, s));
            bar(
                b,
                f,
                Mesh::Cube,
                p.add(v3(0, 0, -cm(10))),
                q.add(v3(0, 0, -cm(10))),
                cm(110),
                RIFT_EDGE,
                ONE,
            );
            bar(b, f, Mesh::Cube, p, q, cm(40), RIFT_HOT, ONE);
            // the dark lip just inside
            let inward = |v: V3| v.add(v3(-s * (w / 16), 0, cm(5)));
            if i > 0 && i < n - 1 {
                bar(
                    b,
                    f,
                    Mesh::Cube,
                    inward(p),
                    inward(q),
                    w / 14,
                    RIFT_LIP,
                    ONE,
                );
            }
            halo_bar(b, f, p, q, int(3), RIFT_EDGE, ONE * 3 / 4);
        }
    }
    // the void: rows of dark violet reaching just under the edges' lip
    // (so from aside they stay behind the edges: not a staircase, TZ3),
    // scaled out with depth
    let k = ONE + div(depth, RIFT_VIEW);
    let back = |p: V3| v3(mul(p.x, k), mul(p.y, k), -depth);
    for i in 0..n {
        let hw = half(i).min(half(i + 1)) + w / 30;
        let mid = back(v3(0, h * (2 * i + 1) / (2 * n), 0));
        put(
            b,
            f,
            Mesh::Cube,
            mid,
            v3(mul(2 * hw, k), mul(h / n + cm(20), k), cm(30)),
            if i % 3 == 1 { VOID_DEEP } else { VOID },
            ONE * 2 / 3,
        );
    }
    // faint stars drifting in the void, just in front of it
    for j in 0..14u32 {
        let i = pick(j, 21, 2, n - 2);
        let x = mul(half(i), pick(j, 22, -70, 70) * ONE / 100);
        let p = back(v3(x, h * i / n, 0)).add(v3(0, 0, cm(40)));
        let tw = pulse(tick, 2, j, ONE / 4);
        let s = mul(cm(50) + mul(k, cm(12)), tw);
        put(b, f, Mesh::Octa, p, v3(s, s, s), VOID_STAR, ONE);
    }
    halo_ball(
        b,
        f,
        back(v3(0, h / 2, 0)).add(v3(0, 0, cm(60))),
        mul(w, k),
        VOID_DEEP,
        ONE / 3,
    );
    // sparks streaming off the edges
    for j in 0..10u32 {
        let life = ((tick + j * 11) % 60) as i32;
        let i = 1 + pick(j, 5, 0, n - 2);
        let s = if j % 2 == 0 { 1 } else { -1 };
        let from = edge(i, s);
        let out = v3(s * cm(12), cm(4), cm(6)).scale(int(life) / 10);
        put(
            b,
            f,
            Mesh::Shard,
            from.add(out),
            v3(cm(30), cm(60), cm(6)),
            mix(RIFT_HOT, RIFT_EDGE, ONE * life / 60),
            ONE,
        );
    }
}

/// The eldritch banana seen through a tear `w` across and `h` tall (in
/// the tear's frame, as `torn_rift`), `depth` behind it: stood up the
/// opening on a slant, `len` long and pressed flat (an eighth deep), so it
/// fills the void from the front and is only a sliver edge-on, hidden by
/// the void behind it from the back (TA6).
pub fn banana_in_rift(b: &mut Batch, f: &Affine, h: i32, len: i32, depth: i32, tick: u32) {
    let g = f
        .then(&Affine::translate(v3(0, h / 2 - len / 10, -depth)))
        .then(&Affine::rot_z(deg(-68)))
        .then(&Affine::scale(v3(ONE, ONE, ONE / 8)));
    eldritch_banana(b, &g, len, tick);
}

/// The giant eldritch evil banana, `len` long, its middle at the origin,
/// facing +z, curving up at both ends: a faceted yellow peel going
/// sickly gold toward the ends, black bruises, a rotten stem; at the far
/// end the peel has split into strips curling back (rotten at their
/// tips) and tentacles writhe out of it; a fanged maw split down its
/// belly, a crowd of glowing red slit-pupilled eyes darting about, dark
/// wisps and a purple-black aura. About 160 shapes.
pub fn eldritch_banana(b: &mut Batch, f: &Affine, len: i32, tick: u32) {
    let n = 12;
    let arc = |i: i32| deg(-55) + deg(110) * i / n;
    let r = mul(len, ONE * 4 / 7);
    let pt = |i: i32| {
        let a = arc(i);
        v3(mul(r, sin(a)), mul(r, ONE - cos(a)) - r / 6, 0)
    };
    // toward the curve's middle (up the inside of the bend)
    let inner = |i: i32| v3(-sin(arc(i)), cos(arc(i)), 0);
    let along = |i: i32| v3(cos(arc(i)), sin(arc(i)), 0);
    let thick = |i: i32| len / 34 + mul(len / 8, sin(HALF * i / n));
    // a point on the skin at `t` along segment i, `phi` round from the
    // front (+z) toward the inside of the bend
    let skin = |i: i32, t: i32, phi: i32, out: i32| {
        let th = (thick(i) + thick(i + 1)) / 2;
        let rr = th / 2 + out;
        pt(i)
            .lerp(pt(i + 1), t)
            .add(inner(i).scale(mul(rr, sin(phi))))
            .add(v3(0, 0, mul(rr, cos(phi))))
    };
    let split = n - 3;
    for i in 0..split {
        let (p, q) = (pt(i), pt(i + 1));
        let d = q.sub(p).scale(ONE / 8);
        let off = (2 * i + 1 - n).abs() * ONE / n;
        let c = if i == 0 {
            mix(BAN_GOLD, BAN_ROT, ONE / 2)
        } else {
            mix(BAN_YELLOW, BAN_GOLD, off)
        };
        bar(
            b,
            f,
            Mesh::Hex,
            p.sub(d),
            q.add(d),
            (thick(i) + thick(i + 1)) / 2,
            c,
            ONE / 3,
        );
    }
    // the rotten stem end
    put(
        b,
        f,
        Mesh::Ico,
        pt(0),
        v3(thick(0), thick(0), thick(0)),
        BAN_ROT,
        0,
    );
    bar(
        b,
        f,
        Mesh::Taper,
        pt(0),
        pt(0).add(v3(-len / 12, len / 11, 0)),
        thick(0) * 3 / 4,
        BAN_ROT,
        0,
    );
    // black bruises over the peel
    for e in 0..14u32 {
        let i = pick(e, 1, 0, split - 1);
        let t = pick(e, 2, 10, 90) * ONE / 100;
        let phi = deg(pick(e, 3, -80, 100));
        let s = thick(i) * pick(e, 4, 18, 40) / 100;
        put_turned(
            b,
            f,
            Mesh::Ico,
            skin(i, t, phi, -s / 6),
            &Affine::rot_x(-phi),
            v3(s, s * 3 / 4, s / 3),
            BAN_BRUISE,
            0,
        );
    }
    // the split end: pale rotting flesh inside, peel strips curling back
    let mouth = pt(split);
    let th = thick(split);
    bar(
        b,
        f,
        Mesh::Taper,
        mouth.sub(along(split).scale(th / 2)),
        mouth.add(along(split).scale(th)),
        th * 3 / 4,
        BAN_FLESH,
        ONE / 6,
    );
    for s in 0..4 {
        let rho = TURN / 8 + TURN * s / 4;
        let radial = inner(split).scale(sin(rho)).add(v3(0, 0, cos(rho)));
        let curl = wave(tick, 5, s * 11_000);
        let mut p = mouth.add(radial.scale(th / 2));
        let mut w = th / 3;
        for j in 0..3 {
            let a = deg(20 + 55 * j) + mul(curl, deg(8));
            let dir = along(split).scale(cos(a)).add(radial.scale(sin(a)));
            let q = p.add(dir.scale(len / 14));
            let c = [BAN_GOLD, mix(BAN_GOLD, BAN_ROT, ONE / 2), BAN_ROT][j as usize];
            bar(b, f, Mesh::Cube, p, q, w, c, ONE / 12);
            p = q;
            w = w * 3 / 4;
        }
    }
    // tentacles writhing out of the split peel
    for t in 0..6 {
        let rho = TURN * t / 6;
        let radial = inner(split).scale(sin(rho)).add(v3(0, 0, cos(rho)));
        let mut p = mouth.add(radial.scale(th / 4));
        let mut w = th / 4;
        let seg = len / 18;
        for j in 1..=7 {
            let sway = wave(tick, 4, t * 9_000 + j * 4_500);
            let a = deg(10 + j * 11) + mul(sway, deg(35));
            let dir = along(split)
                .scale(cos(a))
                .add(radial.scale(sin(a)))
                .add(v3(0, mul(sway, ONE / 3), 0))
                .norm();
            let q = p.add(dir.scale(seg));
            let c = if j > 4 { TENTACLE_TIP } else { TENTACLE };
            bar(b, f, Mesh::Taper, p, q, w, c, ONE / 8);
            p = q;
            w = w * 4 / 5;
        }
    }
    // the maw, split down the belly in the middle
    let mid = n / 2;
    let maw = skin(mid - 1, ONE * 3 / 4, -deg(30), 0);
    put(
        b,
        f,
        Mesh::Geo,
        maw,
        v3(len / 6, len / 16, len / 30),
        MAW,
        ONE * 3 / 4,
    );
    for i in 0..8 {
        let x = -len / 14 + len * i / 56;
        let up = i % 2 == 0;
        let base = maw.add(v3(x, if up { -len / 40 } else { len / 40 }, len / 80));
        let tip = base.add(v3(0, if up { len / 46 } else { -len / 46 }, len / 160));
        bar(b, f, Mesh::Spike, base, tip, len / 90, FANG, ONE / 6);
    }
    // eyes all over, glowing red with black slits that dart about
    for e in 0..26u32 {
        let i = pick(e, 7, 1, split - 1);
        let t = pick(e, 8, 5, 95) * ONE / 100;
        let phi = deg(pick(e, 9, -50, 85));
        let at = skin(i, t, phi, 0);
        if at.dist(maw) < len / 9 {
            continue;
        }
        let s = thick(i) * pick(e, 10, 18, 36) / 100;
        let g = f.then(&Affine::translate(at)).then(&Affine::rot_x(-phi));
        put(b, &g, Mesh::Ico, V3::ZERO, v3(s, s, s / 2), EYE_RED, ONE);
        let look = mul(wave(tick, 3, e as i32 * 4_000), s / 4);
        put(
            b,
            &g,
            Mesh::Octa,
            v3(look, 0, s / 4),
            v3(s / 5, s * 4 / 5, s / 6),
            EYE_SLIT,
            0,
        );
    }
    // dark wisps drifting round it
    for j in 0..8u32 {
        let a = TURN * j as i32 / 8 + (tick as i32).wrapping_mul(TURN / 1_200);
        let at = v3(
            mul(len * 3 / 5, sin(a)),
            len / 6 + mul(len / 6, wave(tick, 6, j as i32 * 8_000)),
            mul(len / 5, cos(a)),
        );
        put_turned(
            b,
            f,
            Mesh::Shard,
            at,
            &Affine::rot_z(a),
            v3(len / 24, len / 10, len / 80),
            WISP,
            ONE / 4,
        );
    }
    halo_ball(
        b,
        f,
        v3(0, len / 10, -len / 4),
        len + len / 3,
        AURA,
        ONE / 5,
    );
    halo_ball(b, f, mouth, len / 3, AURA, ONE / 2);
    halo_ball(b, f, maw, len / 5, MAW, ONE / 2);
}

/// The summit together: the keep's top section and crown (apothem 12 m),
/// the insane wizard on the roof walk tearing the rift open in the air
/// in front of the keep (+z), bolts from his hands to its edges, and the
/// eldritch banana in the void beyond it.
pub fn summit(b: &mut Batch, f: &Affine, tick: u32) {
    let r = int(12);
    keep_section(
        b,
        &f.then(&Affine::translate(v3(0, -int(12), 0))),
        r,
        int(12),
        5,
        Detail::Near,
    );
    keep_crown(b, f, r, 4, 3);
    let wiz = f.then(&Affine::translate(v3(0, CROWN_FLOOR, int(6))));
    mad_wizard(b, &wiz, tick);
    let rift_at = v3(0, CROWN_FLOOR + int(2), int(24));
    let (w, h) = (int(16), int(30));
    // facing back at the keep, its void deep beyond the banana
    let rf = f
        .then(&Affine::translate(rift_at))
        .then(&Affine::rot_y(HALF));
    torn_rift(b, &rf, w, h, ONE, int(1), tick);
    for (i, hand) in WIZARD_HANDS.into_iter().enumerate() {
        let s = if i == 0 { -1 } else { 1 };
        let from = wiz.apply(hand);
        let to = rf.apply(v3(-s * (w / 2 - int(1)), h * 2 / 5, 0));
        spell_beam(
            b,
            &Affine::IDENTITY,
            from,
            to,
            Spell::Arcane,
            tick,
            i as u32,
        );
    }
    // in the tear, beyond it
    banana_in_rift(b, &rf, h, int(20), ONE / 2, tick);
}
