//! The mesa finish: the proboscis-monkey sphinx the finish line lies at the
//! feet of, the campfire with bananas roasting on sticks pushed into the
//! dirt, and the tents.
//!
//! Shape counts: the sphinx about 40 (solid: the skeleton boxes it in
//! `SPHINX_SIZE`), the campfire about 36 and a halo, a tent about 14.

use super::*;
use crate::fx::{cm, cos, deg, mul, sin, TURN};
use crate::render::{mix, shade};

const STATUE: Rgb = rgb(214, 160, 110);
const STATUE_DARK: Rgb = rgb(170, 118, 78);
const STATUE_PALE: Rgb = rgb(232, 196, 150);
pub(crate) const TENT_CANVAS: Rgb = rgb(222, 206, 168);
pub(crate) const TENT_RED: Rgb = rgb(170, 70, 50);
const DIRT: Rgb = rgb(120, 74, 48);
const ROAST: Rgb = rgb(150, 96, 40);

/// The sphinx's bounding box (width, height, length), its plinth's foot at
/// y 0, centred on the origin, facing +z: about 34 m long, 18 m high.
pub const SPHINX_SIZE: V3 = v3(int(16), int(19), int(34));

/// The sphinx: a lion's body lying low on a plinth like Giza's, chest
/// down and forepaws stretched flat toward +z, a proboscis monkey's head
/// with its long fleshy nose drooping over the mouth, a striped
/// headcloth, furry cheek ridges, a curled tail.
pub fn sphinx(b: &mut Batch, f: &Affine) {
    let m = |x: i32, y: i32, z: i32| v3(int(x), int(y), int(z));
    let d = |x: i32, y: i32, z: i32| v3(dm(x), dm(y), dm(z));
    put(
        b,
        f,
        Mesh::Bevel,
        m(0, 1, 0),
        m(16, 2, 34),
        SANDSTONE_DARK,
        0,
    );
    // the body, long and low like Giza's: back level, belly on the plinth
    put_turned(
        b,
        f,
        Mesh::Barrel,
        m(0, 5, -6),
        &Affine::rot_x(deg(90)),
        m(10, 18, 7),
        STATUE,
        0,
    );
    // the chest, low between the forelegs
    put(b, f, Mesh::Geo, d(0, 50, 30), m(8, 6, 6), STATUE, 0);
    for sx in [-1, 1] {
        // a haunch tucked under, the hind leg along the flank
        put(b, f, Mesh::Geo, d(sx * 42, 45, -100), m(5, 5, 8), STATUE, 0);
        put(
            b,
            f,
            Mesh::Bevel,
            d(sx * 52, 28, -60),
            m(3, 2, 8),
            STATUE_DARK,
            0,
        );
        // a foreleg stretched out flat along the plinth, three toe
        // grooves on its paw
        put(
            b,
            f,
            Mesh::Bevel,
            d(sx * 35, 30, 95),
            d(32, 22, 120),
            STATUE,
            0,
        );
        put(
            b,
            f,
            Mesh::Bevel,
            d(sx * 35, 27, 168),
            d(38, 18, 34),
            STATUE_PALE,
            0,
        );
        for t in -1..=1 {
            put(
                b,
                f,
                Mesh::Cube,
                d(sx * 35 + t * 10, 28, 184),
                d(1, 14, 4),
                STATUE_DARK,
                0,
            );
        }
        // the headcloth's lappets, striped, down onto the chest
        for i in 0..3 {
            put(
                b,
                f,
                Mesh::Bevel,
                d(sx * 36, 98 - i * 16, 58),
                d(8, 16, 28),
                if i % 2 == 0 { STATUE_PALE } else { STATUE_DARK },
                0,
            );
        }
        // the cheek ridges of fur, a small ear, a sunken eye
        put(
            b,
            f,
            Mesh::Blob,
            d(sx * 30, 98, 84),
            d(16, 30, 20),
            STATUE_PALE,
            0,
        );
        put(
            b,
            f,
            Mesh::Ico,
            d(sx * 36, 120, 68),
            d(10, 14, 8),
            STATUE_DARK,
            0,
        );
        put(b, f, Mesh::Ico, d(sx * 13, 120, 98), d(9, 7, 6), VOID, 0);
        // the nostrils, under the nose's tip
        put(b, f, Mesh::Ico, d(sx * 5, 63, 117), d(6, 3, 6), VOID, 0);
    }
    // the head and its headcloth, low over the chest
    put(b, f, Mesh::Geo, d(0, 112, 72), d(64, 66, 58), STATUE, 0);
    put(
        b,
        f,
        Mesh::Ico,
        d(0, 142, 67),
        d(70, 24, 60),
        STATUE_PALE,
        0,
    );
    put(
        b,
        f,
        Mesh::Cube,
        d(0, 128, 98),
        d(46, 6, 10),
        STATUE_DARK,
        0,
    );
    // the brow and the bridge between the eyes
    put(
        b,
        f,
        Mesh::Bevel,
        d(0, 116, 96),
        d(34, 12, 10),
        STATUE_PALE,
        0,
    );
    // the nose: a proboscis monkey's, long and fleshy, swelling as it
    // hangs from between the eyes down over the mouth, its tip drooping
    // below the chin
    bar(
        b,
        f,
        Mesh::Taper,
        d(0, 88, 112),
        d(0, 114, 100),
        dm(20),
        STATUE,
        0,
    );
    put_turned(
        b,
        f,
        Mesh::Geo,
        d(0, 78, 114),
        &Affine::rot_x(deg(12)),
        d(26, 30, 18),
        STATUE,
        0,
    );
    put_turned(
        b,
        f,
        Mesh::Geo,
        d(0, 66, 113),
        &Affine::rot_x(-deg(10)),
        d(22, 16, 16),
        STATUE_PALE,
        0,
    );
    // the mouth and chin, mostly hidden behind the nose
    put(b, f, Mesh::Cube, d(0, 72, 102), d(28, 4, 6), VOID, 0);
    put(b, f, Mesh::Ico, d(0, 64, 98), d(30, 14, 16), STATUE_PALE, 0);
    // the tail curled along the plinth
    let mut prev = m(-5, 3, -15);
    for i in 1..=4 {
        let a = i * TURN / 10;
        let p = v3(
            -int(5) + mul(int(4), sin(a)),
            int(2) + ONE / 2,
            -int(15) + mul(int(6), ONE - cos(a)),
        );
        bar(
            b,
            f,
            Mesh::Taper,
            prev,
            p,
            ONE * 3 / 2 - cm(20) * i,
            STATUE,
            0,
        );
        prev = p;
    }
    put(b, f, Mesh::Blob, prev, m(2, 1, 2), STATUE_DARK, 0);
    // weathering: fallen chunks at the plinth's foot
    for (i, x) in [-9, 8, -7].into_iter().enumerate() {
        put(
            b,
            f,
            Mesh::Rock,
            m(x, 0, 12 - i as i32 * 12),
            m(2, 1, 2),
            STATUE_DARK,
            0,
        );
    }
}

/// A campfire on the dirt at the origin: a ring of stones, a teepee of
/// logs, flames flickering with `tick`, glowing embers, three sticks
/// pushed into the dirt leaning over the fire with a banana roasting on
/// each, two log seats.
pub fn campfire(b: &mut Batch, f: &Affine, tick: u32, k: u32) {
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, cm(2), 0),
        v3(int(4), cm(4), int(4)),
        DIRT,
        0,
    );
    for i in 0..8 {
        let a = i * TURN / 8 + pick(k, i as u32, 0, TURN / 30);
        put(
            b,
            f,
            Mesh::Rock,
            v3(mul(cm(80), sin(a)), cm(10), mul(cm(80), cos(a))),
            v3(cm(34), cm(24), cm(30)),
            SANDSTONE_DARK,
            0,
        );
    }
    for i in 0..4 {
        let a = i * TURN / 4 + TURN / 8;
        bar(
            b,
            f,
            Mesh::Hex,
            v3(mul(cm(55), sin(a)), cm(6), mul(cm(55), cos(a))),
            v3(0, cm(80), 0),
            cm(12),
            TIMBER_DARK,
            0,
        );
    }
    put(
        b,
        f,
        Mesh::Ico,
        v3(0, cm(10), 0),
        v3(cm(70), cm(16), cm(70)),
        FLAME,
        ONE * 3 / 4,
    );
    for i in 0..3 {
        let a = i * TURN / 3;
        let h = cm(90) + mul(wave(tick, 1, pick(k, 9 + i as u32, 0, TURN)), cm(25));
        put_turned(
            b,
            f,
            Mesh::Shard,
            v3(mul(cm(14), sin(a)), cm(20) + h / 2, mul(cm(14), cos(a))),
            &Affine::rot_y(a),
            v3(cm(40), h, cm(14)),
            if i == 0 { FLAME_CORE } else { FLAME },
            ONE * 9 / 10,
        );
    }
    halo_ball(b, f, v3(0, cm(70), 0), int(3), FLAME, ONE / 2);
    for i in 0..3 {
        let a = i * TURN / 3 + TURN / 6;
        let (sa, ca) = (sin(a), cos(a));
        let foot = v3(mul(cm(170), sa), 0, mul(cm(170), ca));
        let end = v3(mul(cm(55), sa), cm(120), mul(cm(55), ca));
        bar(b, f, Mesh::Hex, foot, end, cm(4), TIMBER, 0);
        let roast = mix(BANANA, ROAST, ONE * (i + 1) / 5);
        crate::ruinsmodels::banana(b, f, end, end.neg(), cm(40), cm(9), roast, 0);
    }
    for i in 0..2 {
        let a = i * TURN / 2 + TURN / 5;
        let at = v3(mul(cm(260), sin(a)), cm(22), mul(cm(260), cos(a)));
        put_turned(
            b,
            f,
            Mesh::Cyl,
            at,
            &Affine::rot_y(a).then(&Affine::rot_z(crate::fx::QUARTER)),
            v3(cm(44), cm(180), cm(44)),
            TIMBER,
            0,
        );
    }
}

/// An A-frame canvas tent at the origin, its ridge along z, the open flap
/// toward +z: two canvas sides, a closed back, a dark open front with a
/// flap tied back, poles, guy ropes and pegs. `c` is the canvas.
pub fn tent(b: &mut Batch, f: &Affine, c: Rgb, k: u32) {
    let (hw, h, hl) = (cm(130), cm(200), cm(160));
    let side = crate::fx::sqrt(mul(hw, hw) + mul(h, h));
    let lean = crate::fx::atan2(hw, h);
    for sx in [-1, 1] {
        put_turned(
            b,
            f,
            Mesh::Cube,
            v3(sx * hw / 2, h / 2, 0),
            &Affine::rot_z(sx * lean),
            v3(cm(5), side, hl * 2),
            if sx > 0 { c } else { shade(c, 88) },
            0,
        );
        // guy ropes off each end of the ridge, pegged in the dirt
        for sz in [-1, 1] {
            let peg = v3(sx * cm(60), 0, sz * (hl + cm(120)));
            bar(b, f, Mesh::Cube, v3(0, h, sz * hl), peg, cm(2), ROPE, 0);
            put(
                b,
                f,
                Mesh::Cube,
                peg,
                v3(cm(5), cm(16), cm(5)),
                TIMBER_DARK,
                0,
            );
        }
    }
    put(
        b,
        f,
        Mesh::Pyramid,
        v3(0, h / 2, -hl),
        v3(hw * 2, h, cm(4)),
        shade(c, 80),
        0,
    );
    put(
        b,
        f,
        Mesh::Pyramid,
        v3(0, h * 2 / 5, hl - cm(8)),
        v3(hw * 3 / 2, h * 4 / 5, cm(4)),
        VOID,
        0,
    );
    put_turned(
        b,
        f,
        Mesh::Shard,
        v3(-hw / 2, h / 3, hl + cm(4)),
        &Affine::rot_z(deg(pick(k, 1, 10, 25))),
        v3(cm(70), h * 2 / 3, cm(6)),
        shade(c, 94),
        0,
    );
    bar(
        b,
        f,
        Mesh::Hex,
        v3(0, h, -hl - cm(10)),
        v3(0, h, hl + cm(10)),
        cm(7),
        TIMBER_DARK,
        0,
    );
    for sz in [-1, 1] {
        bar(
            b,
            f,
            Mesh::Hex,
            v3(0, 0, sz * (hl + cm(6))),
            v3(0, h + cm(15), sz * (hl + cm(6))),
            cm(6),
            TIMBER_DARK,
            0,
        );
    }
}
