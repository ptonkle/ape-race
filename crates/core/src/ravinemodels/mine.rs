//! The old mine: the mine cart (a moving bouncer, the City's cars' rule), its
//! rickety track with trestle legs for the raised runs, the dark timber-
//! framed holes the track comes out of, and the lower passage's props.
//!
//! Shape counts: a cart about 26; track about 2 a metre (sleepers, rails,
//! a trestle bent every 4 m); a wall hole about 15; a pickaxe 4, a lantern
//! 9 and a halo, an ore bucket 10, a timber support 8, dig marks about 18.

use super::*;
use crate::fx::{cm, mul, sin, TURN};

/// The rails' top over the sleepers' feet: a cart's frame sits here.
pub const RAIL_TOP: i32 = cm(24);
/// The rails' gauge (centre to centre).
pub const GAUGE: i32 = cm(110);
/// A cart's length, width and height over the rails (its bounce box).
pub const CART_SIZE: V3 = v3(cm(170), cm(150), cm(290));

/// The track from `a` to `z` in frame `f`: sleepers (a few skewed, one in
/// seven missing: rickety), two rails, and, where the track runs more than
/// half a metre over `ground` (the frame's y of the floor below), a trestle
/// bent every 4 m down to it. `k` picks the rickety gaps.
pub fn mine_track(b: &mut Batch, f: &Affine, a: V3, z: V3, ground: Option<i32>, k: u32) {
    let d = z.sub(a);
    let len = d.len();
    if len < ONE / 2 {
        return;
    }
    let t = f
        .then(&Affine::translate(a))
        .then(&Affine::looking(d, V3::UP));
    let n = len / (ONE * 9 / 10);
    for i in 0..=n {
        let s = (k.wrapping_mul(97)).wrapping_add(i as u32);
        if i > 0 && i < n && pick(s, 1, 0, 6) == 0 {
            continue;
        }
        let along = mul(len, ONE * i / n.max(1));
        put_turned(
            b,
            &t,
            Mesh::Cube,
            v3(0, cm(6), along),
            &Affine::rot_y(deg(pick(s, 2, -6, 6))),
            v3(cm(170), cm(12), cm(24)),
            if i % 3 == 0 { TIMBER_DARK } else { TIMBER },
            0,
        );
    }
    for sx in [-1, 1] {
        put(
            b,
            &t,
            Mesh::Cube,
            v3(sx * GAUGE / 2, cm(18), len / 2),
            v3(cm(8), cm(12), len),
            IRON,
            0,
        );
    }
    let Some(gy) = ground else {
        return;
    };
    let side = v3(d.z, 0, -d.x).norm();
    let bents = (len / int(4)).max(1);
    for i in 0..=bents {
        let p = a.lerp(z, ONE * i / bents);
        if p.y - gy < ONE / 2 {
            continue;
        }
        let top = |sx: i32| p.add(side.scale(sx * cm(80))).sub(v3(0, cm(4), 0));
        let foot = |sx: i32| v3(p.x, gy, p.z).add(side.scale(sx * cm(125)));
        for sx in [-1, 1] {
            bar(b, f, Mesh::Cube, top(sx), foot(sx), cm(18), TIMBER, 0);
        }
        bar(b, f, Mesh::Cube, top(-1), top(1), cm(16), TIMBER_DARK, 0);
        let mid = |sx: i32| top(sx).lerp(foot(sx), ONE / 2);
        bar(b, f, Mesh::Cube, top(-1), mid(1), cm(10), TIMBER_DARK, 0);
        if p.y - gy > int(3) {
            bar(b, f, Mesh::Cube, mid(-1), foot(1), cm(10), TIMBER_DARK, 0);
        }
    }
}

/// A mine cart on the rails at the frame's origin (rail top, between the
/// rails), facing +z: an iron-banded wooden tub on four spoked wheels that
/// turn with `tick`, buffers at both ends, a heap of gold ore if `load`.
pub fn mine_cart(b: &mut Batch, f: &Affine, tick: u32, k: u32, load: bool) {
    let roll = (tick as i32).wrapping_mul(TURN / 90);
    let (hw, hl) = (cm(75), cm(130));
    // wheels with a spoke each, so the turn shows
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            let at = v3(sx * GAUGE / 2, cm(26), sz * cm(80));
            put_turned(
                b,
                f,
                Mesh::Cyl,
                at,
                &Affine::rot_z(crate::fx::QUARTER),
                v3(cm(52), cm(12), cm(52)),
                IRON,
                0,
            );
            put_turned(
                b,
                f,
                Mesh::Cube,
                at.add(v3(sx * cm(7), 0, 0)),
                &Affine::rot_x(roll + pick(k, sx as u32 + 7, 0, TURN)),
                v3(cm(3), cm(46), cm(6)),
                RUST,
                0,
            );
        }
    }
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, cm(50), 0),
        v3(cm(120), cm(14), cm(200)),
        IRON,
        0,
    );
    // the tub: a floor and four walls leaning out
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, cm(62), 0),
        v3(hw * 2, cm(10), hl * 2),
        TIMBER_DARK,
        0,
    );
    for sx in [-1, 1] {
        put_turned(
            b,
            f,
            Mesh::Cube,
            v3(sx * (hw + cm(6)), cm(108), 0),
            &Affine::rot_z(-sx * deg(8)),
            v3(cm(10), cm(90), hl * 2 + cm(20)),
            TIMBER,
            0,
        );
        put_turned(
            b,
            f,
            Mesh::Cube,
            v3(0, cm(108), sx * (hl + cm(6))),
            &Affine::rot_x(sx * deg(8)),
            v3(hw * 2 + cm(20), cm(90), cm(10)),
            TIMBER,
            0,
        );
        // iron bands round the tub
        for y in [cm(78), cm(140)] {
            put(
                b,
                f,
                Mesh::Cube,
                v3(sx * (hw + cm(10) + (y - cm(78)) / 8), y, 0),
                v3(cm(4), cm(10), hl * 2 + cm(30)),
                RUST,
                0,
            );
        }
        // a timber buffer
        put(
            b,
            f,
            Mesh::Bevel,
            v3(0, cm(60), sx * (hl + cm(22))),
            v3(cm(90), cm(20), cm(22)),
            TIMBER_DARK,
            0,
        );
    }
    if load {
        for i in 0..3 {
            let x = cm(40) * (i - 1);
            put(
                b,
                f,
                Mesh::Rock,
                v3(x, cm(150), cm(30) * (1 - i)),
                v3(cm(80), cm(50), cm(90)),
                SANDSTONE_DARK,
                0,
            );
            put(
                b,
                f,
                Mesh::Octa,
                v3(x + cm(10), cm(176), cm(28) * (1 - i)),
                v3(cm(20), cm(20), cm(20)),
                GOLD,
                ONE / 2,
            );
        }
    }
}

/// A dark hole in a wall the track comes out of: the wall's face is the
/// frame's z = 0 plane, the hole opening toward +z, its floor at y 0, `w`
/// wide and `h` high; a black void just proud of the wall, a timber frame
/// (posts, lintel, knee braces) and fallen rocks round it. The hole is a
/// barrier: its solid is the box `w` by `h` from z -1 m to z 0.3 m.
pub fn wall_hole(b: &mut Batch, f: &Affine, w: i32, h: i32, k: u32) {
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, h / 2, -cm(45)),
        v3(w, h, cm(100)),
        VOID,
        0,
    );
    let px = w / 2 + cm(16);
    for sx in [-1, 1] {
        put(
            b,
            f,
            Mesh::Cube,
            v3(sx * px, h / 2 + cm(10), cm(18)),
            v3(cm(32), h + cm(20), cm(32)),
            TIMBER,
            0,
        );
        bar(
            b,
            f,
            Mesh::Cube,
            v3(sx * px, h - cm(80), cm(22)),
            v3(sx * (px - cm(90)), h + cm(4), cm(22)),
            cm(16),
            TIMBER_DARK,
            0,
        );
        // a rock heap at each foot
        put(
            b,
            f,
            Mesh::Rock,
            v3(sx * (px + cm(70)), cm(30), cm(40)),
            v3(cm(110), cm(70), cm(90)),
            SANDSTONE_DARK,
            0,
        );
        put(
            b,
            f,
            Mesh::Rock,
            v3(sx * (px + cm(50)), cm(90) + h / 2, cm(10)),
            v3(cm(80), cm(60), cm(50)),
            SANDSTONE,
            0,
        );
    }
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, h + cm(26), cm(18)),
        v3(w + cm(110), cm(36), cm(36)),
        TIMBER,
        0,
    );
    // a skewed board nailed over the corner, a fringe of moss on the lintel
    put_turned(
        b,
        f,
        Mesh::Cube,
        v3(-w / 4, h + cm(26), cm(40)),
        &Affine::rot_z(deg(pick(k, 1, 6, 16))),
        v3(w / 2, cm(16), cm(6)),
        TIMBER_DARK,
        0,
    );
    put(
        b,
        f,
        Mesh::Blob,
        v3(w / 5, h + cm(46), cm(18)),
        v3(w / 2, cm(16), cm(40)),
        MOSS,
        0,
    );
    put(
        b,
        f,
        Mesh::Rock,
        v3(0, h + cm(110), -cm(10)),
        v3(w + int(2), cm(140), cm(90)),
        SANDSTONE,
        0,
    );
}

/// A pickaxe leaning `lean` from upright, its foot at the origin: a hickory
/// handle and a two-pointed iron head.
pub fn pickaxe(b: &mut Batch, f: &Affine, lean: i32) {
    let g = f.then(&Affine::rot_x(lean));
    let top = v3(0, cm(95), 0);
    bar(b, &g, Mesh::Hex, V3::ZERO, top, cm(6), TIMBER, 0);
    put(b, &g, Mesh::Cube, top, v3(cm(12), cm(12), cm(12)), IRON, 0);
    for sx in [-1, 1] {
        bar(
            b,
            &g,
            Mesh::Spike,
            top,
            top.add(v3(sx * cm(42), -cm(10), 0)),
            cm(8),
            IRON,
            0,
        );
    }
}

/// A hurricane lantern hanging from the frame's origin (its hook), glowing
/// and flickering with `tick`: hook, cap, glass, a four-bar cage, base, a
/// soft halo.
pub fn lantern(b: &mut Batch, f: &Affine, tick: u32, k: u32) {
    let flick = ONE * 3 / 4 + mul(wave(tick, 1, pick(k, 1, 0, TURN)), ONE / 8);
    bar(
        b,
        f,
        Mesh::Cube,
        V3::ZERO,
        v3(0, -cm(14), 0),
        cm(2),
        IRON,
        0,
    );
    put(
        b,
        f,
        Mesh::Pyramid,
        v3(0, -cm(20), 0),
        v3(cm(20), cm(12), cm(20)),
        IRON,
        0,
    );
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, -cm(36), 0),
        v3(cm(13), cm(20), cm(13)),
        FLAME_CORE,
        flick,
    );
    for (sx, sz) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        bar(
            b,
            f,
            Mesh::Cube,
            v3(sx * cm(8), -cm(25), sz * cm(8)),
            v3(sx * cm(8), -cm(48), sz * cm(8)),
            cm(2),
            IRON,
            0,
        );
    }
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, -cm(50), 0),
        v3(cm(20), cm(5), cm(20)),
        IRON,
        0,
    );
    halo_ball(b, f, v3(0, -cm(36), 0), cm(110), FLAME, ONE / 2);
}

/// A wooden ore bucket on the floor at the origin: iron hoops, a handle,
/// heaped ore with gold glints.
pub fn ore_bucket(b: &mut Batch, f: &Affine, k: u32) {
    let r = cm(70);
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, cm(28), 0),
        v3(r, cm(56), r),
        TIMBER,
        0,
    );
    for y in [cm(10), cm(46)] {
        put(
            b,
            f,
            Mesh::Cyl,
            v3(0, y, 0),
            v3(r + cm(4), cm(5), r + cm(4)),
            RUST,
            0,
        );
    }
    let lean = deg(pick(k, 1, -30, 30));
    let top = v3(mul(cm(10), sin(lean)), cm(86), 0);
    for sx in [-1, 1] {
        bar(
            b,
            f,
            Mesh::Cube,
            v3(sx * r / 2, cm(54), 0),
            top.add(v3(sx * cm(14), 0, 0)),
            cm(3),
            IRON,
            0,
        );
    }
    bar(
        b,
        f,
        Mesh::Cube,
        top.add(v3(-cm(14), 0, 0)),
        top.add(v3(cm(14), 0, 0)),
        cm(3),
        IRON,
        0,
    );
    put(
        b,
        f,
        Mesh::Rock,
        v3(0, cm(60), 0),
        v3(cm(62), cm(28), cm(62)),
        SANDSTONE_DARK,
        0,
    );
    for i in 0..2 {
        put(
            b,
            f,
            Mesh::Octa,
            v3(cm(14) * (2 * i - 1), cm(72), cm(8)),
            v3(cm(12), cm(12), cm(12)),
            GOLD,
            ONE / 2,
        );
    }
}

/// A mine's timber set: two posts `w` apart (outside), a cap `h` up, knee
/// braces and wedges, standing on y 0 across x.
pub fn mine_support(b: &mut Batch, f: &Affine, w: i32, h: i32, k: u32) {
    let px = w / 2 - cm(18);
    for sx in [-1, 1] {
        put_turned(
            b,
            f,
            Mesh::Cube,
            v3(sx * px, h / 2, 0),
            &Affine::rot_z(sx * deg(pick(k, sx as u32 + 3, 1, 4))),
            v3(cm(34), h, cm(34)),
            TIMBER,
            0,
        );
        bar(
            b,
            f,
            Mesh::Cube,
            v3(sx * px, h - cm(120), 0),
            v3(sx * (px - cm(110)), h - cm(18), 0),
            cm(18),
            TIMBER_DARK,
            0,
        );
        put(
            b,
            f,
            Mesh::Wedge,
            v3(sx * (px - cm(30)), h - cm(26), 0),
            v3(cm(20), cm(16), cm(30)),
            TIMBER_DARK,
            0,
        );
    }
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, h + cm(18), 0),
        v3(w + cm(40), cm(36), cm(40)),
        TIMBER,
        0,
    );
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, h + cm(42), 0),
        v3(w - cm(20), cm(12), cm(80)),
        TIMBER_DARK,
        0,
    );
}

/// Gold-flecked dig marks on a wall patch `w` by `h` (the wall's face the
/// frame's z = 0 plane, toward +z; the patch's foot at y 0): pick scars,
/// gold flecks glinting in turn with `tick`, a heap of spoil at the foot.
pub fn dig_marks(b: &mut Batch, f: &Affine, w: i32, h: i32, tick: u32, k: u32) {
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, h / 2, -cm(4)),
        v3(w, h, cm(12)),
        SANDSTONE_DARK,
        0,
    );
    for i in 0..6 {
        let s = k.wrapping_mul(13).wrapping_add(i);
        let x = pick(s, 1, -w / 2 + cm(30), w / 2 - cm(30));
        let y = pick(s, 2, cm(40), h - cm(30));
        put_turned(
            b,
            f,
            Mesh::Cube,
            v3(x, y, cm(4)),
            &Affine::rot_z(deg(pick(s, 3, 20, 70))),
            v3(cm(70), cm(7), cm(6)),
            VOID,
            0,
        );
    }
    for i in 0..8 {
        let s = k.wrapping_mul(31).wrapping_add(i);
        let x = pick(s, 4, -w / 2 + cm(20), w / 2 - cm(20));
        let y = pick(s, 5, cm(30), h - cm(20));
        let glint = (wave(tick, 2, pick(s, 6, 0, TURN)).max(0) * 3 / 4) + ONE / 4;
        put(
            b,
            f,
            Mesh::Octa,
            v3(x, y, cm(6)),
            v3(cm(12), cm(12), cm(8)),
            GOLD,
            glint,
        );
    }
    for i in 0..3 {
        put(
            b,
            f,
            Mesh::Rock,
            v3(cm(60) * (i - 1), cm(15), cm(40)),
            v3(cm(70), cm(40), cm(60)),
            SANDSTONE,
            0,
        );
    }
}
