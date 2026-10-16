//! The summit: the insane wizard ripping a tear in reality, the tear, and
//! the giant eldritch evil banana seen through it.

use super::keep::{keep_crown, keep_section, Detail, CROWN_FLOOR};
use super::{bar, halo_ball, halo_bar, pick, put, put_turned, spell_beam, wave, Spell};
use super::{BEARD, FACE, FUR, GOLD};
use crate::fx::{cm, cos, deg, int, mul, sin, HALF, ONE, TURN};
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
const BAN_SKIN: Rgb = rgb(118, 104, 30);
const BAN_BRUISE: Rgb = rgb(52, 44, 20);
const BAN_STEM: Rgb = rgb(40, 30, 18);
const SCLERA: Rgb = rgb(236, 226, 176);
const PUPIL: Rgb = rgb(255, 40, 30);
const TENTACLE: Rgb = rgb(70, 32, 82);
const TENTACLE_TIP: Rgb = rgb(150, 60, 140);
const MAW: Rgb = rgb(150, 20, 40);
const FANG: Rgb = rgb(236, 230, 206);
const ELDRITCH_GLOW: Rgb = rgb(120, 255, 90);

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
/// torn open. Jagged glowing edges with a dark torn lip inside them and
/// sparks flying off; its middle is open (what is beyond shows through).
/// About 110 shapes.
pub fn rift(b: &mut Batch, f: &Affine, w: i32, h: i32, open: i32, tick: u32) {
    let n = 12;
    let jump = tick / 6;
    let edge = |i: i32, s: i32| {
        let y = h * i / n;
        let half = mul(mul(w / 2, sin(HALF * i / n)), open.clamp(0, ONE));
        // jags alternate in and out, and shiver a little
        let jag = if (i + (s + 1) / 2) % 2 == 0 {
            w / 14
        } else {
            -w / 20
        };
        let shiver = pick(jump * 13 + (i * 2 + (s + 1) / 2) as u32, 4, -10, 10) * (w / 400);
        v3(s * (half + jag + shiver), y, 0)
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

/// The giant eldritch evil banana, `len` long, its middle at the origin,
/// facing +z, curving up at both ends: a bruised, sickly skin, a peel
/// split open on a fanged maw, a crowd of red-pupilled eyes that look
/// about, tentacles writhing below, and a green glow round it. About 120
/// shapes.
pub fn eldritch_banana(b: &mut Batch, f: &Affine, len: i32, tick: u32) {
    let n = 9;
    // the curve: an arc of 100°, ends up
    let r = mul(len, ONE * 4 / 7);
    let pt = |i: i32| {
        let a = deg(-50) + deg(100) * i / n;
        v3(mul(r, sin(a)), mul(r, ONE - cos(a)) - r / 6, 0)
    };
    let thick = |i: i32| mul(len, ONE / 20) + mul(mul(len, ONE / 9), sin(HALF * i / n));
    for i in 0..n {
        let (p, q) = (pt(i), pt(i + 1));
        let d = q.sub(p);
        let a = p.sub(d.scale(ONE / 6));
        let z = q.add(d.scale(ONE / 6));
        let c = if pick(i as u32, 1, 0, 2) == 0 {
            BAN_BRUISE
        } else {
            BAN_SKIN
        };
        bar(
            b,
            f,
            Mesh::Barrel,
            a,
            z,
            (thick(i) + thick(i + 1)) / 2,
            c,
            ONE / 10,
        );
    }
    bar(
        b,
        f,
        Mesh::Taper,
        pt(0),
        pt(0).add(v3(-len / 14, len / 12, 0)),
        thick(0) * 2 / 3,
        BAN_STEM,
        0,
    );
    bar(
        b,
        f,
        Mesh::Cone,
        pt(n),
        pt(n).add(v3(len / 18, len / 16, 0)),
        thick(n) * 2 / 3,
        BAN_STEM,
        0,
    );
    // the maw, where the peel has split
    let mid = pt(n / 2).lerp(pt(n / 2 + 1), ONE / 2);
    let front = |p: V3, i: i32| p.add(v3(0, 0, thick(i) * 2 / 5));
    let maw = front(mid, n / 2).add(v3(0, -thick(n / 2) / 6, 0));
    put(
        b,
        f,
        Mesh::Geo,
        maw,
        v3(len / 6, len / 14, len / 30),
        MAW,
        ONE * 3 / 4,
    );
    for i in 0..8 {
        let x = -len / 14 + len * i / 56;
        let up = i % 2 == 0;
        let base = maw.add(v3(x, if up { -len / 34 } else { len / 34 }, len / 80));
        let tip = base.add(v3(0, if up { len / 40 } else { -len / 40 }, len / 160));
        bar(b, f, Mesh::Spike, base, tip, len / 90, FANG, ONE / 6);
    }
    for (i, a) in [-deg(40), 0, deg(40)].into_iter().enumerate() {
        let g = f
            .then(&Affine::translate(maw.add(v3(0, thick(n / 2) / 3, 0))))
            .then(&Affine::rot_z(a))
            .then(&Affine::rot_x(-deg(35)));
        put(
            b,
            &g,
            Mesh::Shard,
            v3(0, len / 16, 0),
            v3(len / 12, len / 8, len / 200),
            if i == 1 { BAN_BRUISE } else { BAN_SKIN },
            ONE / 10,
        );
    }
    // eyes all over, pupils darting
    for e in 0..14u32 {
        let i = pick(e, 7, 0, n - 1);
        let t = pick(e, 8, 10, 90) * ONE / 100;
        let on = pt(i).lerp(pt(i + 1), t);
        let lift = pick(e, 9, -30, 40) * thick(i) / 100;
        if (on.x - maw.x).abs() < len / 9 && lift < thick(i) / 5 {
            continue;
        }
        let at = front(on, i).add(v3(0, lift, 0));
        let s = thick(i) * pick(e, 10, 14, 34) / 100;
        put(b, f, Mesh::Ico, at, v3(s, s, s / 2), SCLERA, ONE * 3 / 5);
        let look = mul(wave(tick, 3, e as i32 * 4_000), s / 5);
        put(
            b,
            f,
            Mesh::Octa,
            at.add(v3(look, 0, s / 4)),
            v3(s / 4, s * 4 / 5, s / 6),
            PUPIL,
            ONE,
        );
    }
    // tentacles from the underside
    for t in 0..8 {
        let i = 1 + t;
        let root = pt(i).add(v3(
            0,
            -thick(i) * 2 / 5,
            pick(t as u32, 11, -20, 20) * thick(i) / 100,
        ));
        let out = if t < 4 { -1 } else { 1 };
        let mut p = root;
        let mut w = thick(i) / 4;
        let seg = len / 22;
        for j in 1..=7 {
            let sway = wave(tick, 4, t * 9_000 + j * 4_500);
            let a = deg(180) + out * deg(10 + j * 6) + mul(sway, deg(30));
            let q = p.add(v3(
                mul(seg, sin(a)),
                mul(seg, cos(a)),
                mul(mul(sway, seg), ONE / 3),
            ));
            let c = if j > 4 { TENTACLE_TIP } else { TENTACLE };
            bar(b, f, Mesh::Taper, p, q, w, c, ONE / 8);
            p = q;
            w = w * 4 / 5;
        }
    }
    halo_ball(
        b,
        f,
        v3(0, len / 10, 0),
        len + len / 3,
        ELDRITCH_GLOW,
        ONE / 4,
    );
    halo_ball(b, f, maw, len / 4, MAW, ONE / 2);
}

/// The summit together: the keep's top section and crown (apothem 12 m),
/// the insane wizard on the roof walk tearing the rift open in the air
/// in front of the keep (+z), bolts from his hands to its edges, and the
/// eldritch banana far beyond it.
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
    let rf = f.then(&Affine::translate(rift_at));
    rift(b, &rf, w, h, ONE, tick);
    for (i, hand) in WIZARD_HANDS.into_iter().enumerate() {
        let s = if i == 0 { -1 } else { 1 };
        let from = wiz.apply(hand);
        let to = rf.apply(v3(s * (w / 2 - int(1)), h * 2 / 5, 0));
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
    // far beyond, facing back at the keep
    let beyond = f
        .then(&Affine::translate(v3(0, int(30), int(120))))
        .then(&Affine::rot_y(HALF));
    eldritch_banana(b, &beyond, int(80), tick);
}
