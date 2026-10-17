//! The wall lizards: big, bright climbing lizards, the map's moving bouncers
//! (the Sunset Ruins turtles' rule), in red, yellow or blue. Built lying on
//! its belly on the frame's xz plane (the wall), head toward +z, about 3.4 m
//! nose to tail at scale 1 (to bounce on beside apes drawn at 2×); place it
//! on a wall with `on_surface(at, normal, up)`. The walk is a diagonal gait
//! driven by `phase` (a turn per stride, `lizard_phase`): the body and tail
//! wave side to side, the feet lift as they swing forward.
//!
//! About 48 shapes.

use super::*;
use crate::fx::{cm, cos, mul, sin, TURN};
use crate::rules::TICKS_PER_SECOND as TICKS_PER_SECOND_I32;

/// A lizard's colours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LizardHue {
    Red,
    Yellow,
    Blue,
}

impl LizardHue {
    pub const ALL: [LizardHue; 3] = [LizardHue::Red, LizardHue::Yellow, LizardHue::Blue];

    /// Hue `k` of the three, round and round.
    pub fn nth(k: u32) -> LizardHue {
        LizardHue::ALL[(k % 3) as usize]
    }

    /// Skin, spots and crest, belly.
    pub const fn colours(self) -> (Rgb, Rgb, Rgb) {
        match self {
            LizardHue::Red => (rgb(214, 44, 34), rgb(255, 210, 60), rgb(250, 160, 120)),
            LizardHue::Yellow => (rgb(246, 202, 30), rgb(30, 120, 110), rgb(255, 238, 160)),
            LizardHue::Blue => (rgb(36, 90, 226), rgb(255, 140, 30), rgb(150, 200, 255)),
        }
    }
}

/// The lizard's stride for race tick `tick` (`k` picks its own phase).
pub fn lizard_phase(tick: u32, k: u32) -> i32 {
    let period = (TICKS_PER_SECOND_I32 * 6 / 5) as u32;
    ((tick + k * 53) % period) as i32 * (TURN / period as i32)
}

/// A wall lizard (see the module doc) walking with stride `phase`.
pub fn lizard(b: &mut Batch, f: &Affine, phase: i32, hue: LizardHue) {
    let (skin, spot, belly) = hue.colours();
    let s = sin(phase);
    let c = cos(phase);
    // the spine: six points, nose last, waving side to side with the stride
    let spine: Vec<V3> = (0..7)
        .map(|i| {
            let z = cm(-40) + cm(30) * i;
            let x = mul(cm(10), sin(phase + i * TURN / 7));
            v3(x, cm(30), z)
        })
        .collect();
    for (i, w) in spine.windows(2).enumerate() {
        let wd = if i == 0 {
            cm(40)
        } else if i < 4 {
            cm(52)
        } else {
            cm(36)
        };
        bar(b, f, Mesh::Barrel, w[0], w[1], wd, skin, 0);
    }
    put(
        b,
        f,
        Mesh::Bevel,
        v3(spine[3].x, cm(20), cm(50)),
        v3(cm(46), cm(14), cm(110)),
        belly,
        0,
    );
    // the head: a wedge snout, a frill, eyes
    let neck = spine[6];
    let g = f
        .then(&Affine::translate(neck))
        .then(&Affine::rot_y(mul(deg(12), s)));
    put(
        b,
        &g,
        Mesh::Ico,
        v3(0, cm(4), cm(22)),
        v3(cm(40), cm(28), cm(46)),
        skin,
        0,
    );
    put(
        b,
        &g,
        Mesh::Wedge,
        v3(0, 0, cm(52)),
        v3(cm(30), cm(18), cm(30)),
        skin,
        0,
    );
    put_turned(
        b,
        &g,
        Mesh::Wedge,
        v3(0, 0, cm(52)),
        &Affine::rot_y(crate::fx::HALF),
        v3(cm(30), cm(18), cm(30)),
        skin,
        0,
    );
    for sx in [-1, 1] {
        put(
            b,
            &g,
            Mesh::Sphere,
            v3(sx * cm(17), cm(14), cm(30)),
            v3(cm(12), cm(12), cm(12)),
            rgb(250, 250, 230),
            ONE / 3,
        );
        put(
            b,
            &g,
            Mesh::Sphere,
            v3(sx * cm(22), cm(15), cm(31)),
            v3(cm(5), cm(7), cm(7)),
            VOID,
            0,
        );
        // the frill behind the head
        put_turned(
            b,
            &g,
            Mesh::Shard,
            v3(sx * cm(22), cm(10), 0),
            &Affine::rot_y(sx * deg(50)).then(&Affine::rot_z(sx * deg(60))),
            v3(cm(22), cm(36), cm(4)),
            spot,
            0,
        );
    }
    // the crest along the back and the spots down the flanks
    for (i, p) in spine.iter().enumerate().skip(1).take(5) {
        put(
            b,
            f,
            Mesh::Spike,
            p.add(v3(0, cm(28), 0)),
            v3(cm(8), cm(18), cm(12)),
            spot,
            0,
        );
        if i % 2 == 1 {
            for sx in [-1, 1] {
                put(
                    b,
                    f,
                    Mesh::Ico,
                    p.add(v3(sx * cm(22), cm(10), 0)),
                    v3(cm(12), cm(10), cm(14)),
                    spot,
                    0,
                );
            }
        }
    }
    // the tail: five tapering segments, waving opposite the head
    let mut prev = spine[0];
    for i in 1..=5 {
        let x = mul(cm(16) * i, sin(phase - i * TURN / 9 + TURN / 2));
        let p = v3(x, cm(26) - cm(3) * i, cm(-40) - cm(32) * i);
        bar(b, f, Mesh::Taper, prev, p, cm(34) - cm(5) * i, skin, 0);
        prev = p;
    }
    put(
        b,
        f,
        Mesh::Cone,
        prev.add(v3(0, 0, -cm(12))),
        v3(cm(8), cm(8), cm(24)),
        spot,
        0,
    );
    // the legs: front-left with hind-right, then the other pair
    for (front, z, sx) in [
        (true, cm(80), 1),
        (true, cm(80), -1),
        (false, -cm(30), 1),
        (false, -cm(30), -1),
    ] {
        let pair = if front == (sx > 0) { ONE } else { -ONE };
        let swing = mul(mul(cm(28), s), pair);
        // a foot swinging forward is lifted
        let lift = (mul(c, pair)).max(0);
        let lift = mul(lift, cm(16));
        let hip = v3(sx * cm(22), cm(30), z);
        let knee = v3(sx * cm(62), cm(40) + lift, z + swing / 2);
        let foot = v3(sx * cm(78), cm(4) + lift, z + swing);
        bar(b, f, Mesh::Taper, hip, knee, cm(16), skin, 0);
        bar(b, f, Mesh::Taper, knee, foot, cm(12), skin, 0);
        for t in -1..=1 {
            let dir = v3(
                sx * cm(10) + t * cm(6),
                -lift / 4,
                if front { cm(12) } else { -cm(4) } + t * cm(8),
            );
            bar(b, f, Mesh::Cube, foot, foot.add(dir), cm(4), belly, 0);
        }
    }
}
