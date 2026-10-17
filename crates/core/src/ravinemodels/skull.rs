//! The T-rex fossil skull of the shortcut: a huge skull set halfway up a
//! cave wall, its jaws open, the apes flying in at the snout between the
//! teeth and out of the open back of the mouth.
//!
//! The frame's origin is the middle of the gap between the jaws; the snout
//! points toward +z, the ape flies along -z through it. The gap is the
//! height between the lowest upper tooth tip and the highest lower tooth
//! tip (the brief: 2× the tallest ape), and everything scales with it.
//! The teeth come from one list (`rex_teeth`), so the skeleton's solids and
//! the drawing share the numbers; the bone's solids are `rex_bone_boxes`.
//!
//! About 52 shapes (28 teeth, 24 bone).

use super::*;
use crate::fx::{cm, div, mul};

const TOOTH: Rgb = rgb(252, 246, 222);

/// The gallery's gap: 2× a 2 m ape. The course passes its own.
pub const REX_GAP: i32 = int(4);

/// One tooth: a cone from `base` (in the gum) to `tip`, `r` across at the
/// base. An upper tooth's tip is below its base.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tooth {
    pub base: V3,
    pub tip: V3,
    pub r: i32,
}

impl Tooth {
    pub fn upper(&self) -> bool {
        self.tip.y < self.base.y
    }
}

/// The skull's size share for gap `gap` (built at a 4 m gap).
fn share(gap: i32) -> i32 {
    div(gap, int(4))
}

/// The longest tooth, base to tip, for gap `gap`.
pub fn rex_tooth_len(gap: i32) -> i32 {
    mul(int(1), share(gap))
}

/// The height of the upper gum (the palate) over the gap's middle; the
/// lower gum is as far under it (to a 64th of a millimetre: the two
/// halves of an odd gap).
pub fn rex_gum(gap: i32) -> i32 {
    gap - gap / 2 + rex_tooth_len(gap)
}

/// The gum's height for row `sy` (1 the upper, -1 the lower).
fn gum_of(gap: i32, sy: i32) -> i32 {
    if sy > 0 {
        rex_gum(gap)
    } else {
        -(gap / 2 + rex_tooth_len(gap))
    }
}

/// The teeth for a gap `gap` high: four across the front, five down each
/// side, top and bottom (28). The canines (the second side tooth) reach
/// the gap's edge exactly; the others are shorter.
pub fn rex_teeth(gap: i32) -> Vec<Tooth> {
    let s = share(gap);
    let m = |x: i32| mul(x, s);
    let long = rex_tooth_len(gap);
    // (x, z, share of the longest) in a 4 m skull's metres
    let mut spots: Vec<(i32, i32, i32)> = Vec::new();
    for (x, l) in [(-cm(120), 70), (-cm(40), 80), (cm(40), 80), (cm(120), 70)] {
        spots.push((x, cm(300), l));
    }
    for sx in [-1, 1] {
        for (z, l) in [
            (cm(240), 85),
            (cm(160), 100),
            (cm(60), 80),
            (-cm(40), 65),
            (-cm(140), 50),
        ] {
            spots.push((sx * (cm(180) + (cm(240) - z) / 8), z, l));
        }
    }
    let mut teeth = Vec::with_capacity(spots.len() * 2);
    for sy in [1, -1] {
        for &(x, z, l) in &spots {
            let len = long * l / 100;
            // the lower row sits a little inside the upper, as it closes
            let inset = if sy < 0 { cm(15) * x.signum() } else { 0 };
            let base = v3(
                m(x - inset),
                gum_of(gap, sy),
                m(z) - if sy < 0 { m(cm(10)) } else { 0 },
            );
            teeth.push(Tooth {
                base,
                tip: base.sub(v3(0, sy * len, 0)).add(v3(0, 0, m(cm(6)))),
                r: len * 2 / 5,
            });
        }
    }
    teeth
}

/// The skull's bone as solid boxes (middle, size) in its frame, for the
/// skeleton: the upper skull above the upper gum and the lower jaw under
/// the lower gum, the mouth between them open front to back.
pub fn rex_bone_boxes(gap: i32) -> Vec<(V3, V3)> {
    let s = share(gap);
    let m = |x: i32| mul(x, s);
    let gum = rex_gum(gap);
    vec![
        // the upper skull: snout and braincase
        (
            v3(0, gum + m(cm(150)), m(cm(40))),
            v3(m(cm(560)), m(cm(300)), m(cm(760))),
        ),
        // the lower jaw
        (
            v3(0, -gum - m(cm(40)), m(cm(40))),
            v3(m(cm(560)), m(cm(80)), m(cm(720))),
        ),
    ]
}

/// The skull for gap `gap` with teeth `teeth` (`rex_teeth(gap)`, or the
/// course's own list).
pub fn rex_skull(b: &mut Batch, f: &Affine, gap: i32, teeth: &[Tooth]) {
    let s = share(gap);
    let m = |x: i32| mul(x, s);
    let p = |x: i32, y: i32, z: i32| v3(m(x), m(y), m(z));
    let gum = rex_gum(gap);
    let up = |x: i32, y: i32, z: i32| v3(m(x), gum + m(y), m(z));
    let dn = |x: i32, y: i32, z: i32| v3(m(x), -gum + m(y), m(z));
    for t in teeth {
        bar(b, f, Mesh::Spike, t.base, t.tip, t.r, TOOTH, 0);
    }
    // the upper skull: palate, snout, a nasal ridge, the braincase
    put(
        b,
        f,
        Mesh::Bevel,
        up(0, cm(30), cm(40)),
        p(cm(500), cm(60), cm(640)),
        BONE_DARK,
        0,
    );
    put(
        b,
        f,
        Mesh::Ico,
        up(0, cm(150), cm(140)),
        p(cm(470), cm(250), cm(460)),
        BONE,
        0,
    );
    put(
        b,
        f,
        Mesh::Wedge,
        up(0, cm(270), cm(260)),
        p(cm(160), cm(110), cm(240)),
        BONE,
        0,
    );
    put(
        b,
        f,
        Mesh::Geo,
        up(0, cm(220), -cm(220)),
        p(cm(460), cm(340), cm(340)),
        BONE,
        0,
    );
    for sx in [-1, 1] {
        // the eye socket, the hole in front of it, a brow hornlet
        put(
            b,
            f,
            Mesh::Ico,
            up(sx * cm(205), cm(230), -cm(120)),
            p(cm(80), cm(130), cm(110)),
            VOID,
            0,
        );
        put(
            b,
            f,
            Mesh::Ico,
            up(sx * cm(225), cm(130), cm(80)),
            p(cm(40), cm(100), cm(170)),
            VOID,
            0,
        );
        put(
            b,
            f,
            Mesh::Spike,
            up(sx * cm(180), cm(330), -cm(110)),
            p(cm(50), cm(60), cm(50)),
            BONE_DARK,
            0,
        );
        // the cheek bar and the jaw's hinge post
        bar(
            b,
            f,
            Mesh::Hex,
            up(sx * cm(250), cm(30), cm(160)),
            up(sx * cm(280), cm(70), -cm(300)),
            m(cm(70)),
            BONE,
            0,
        );
        bar(
            b,
            f,
            Mesh::Taper,
            dn(sx * cm(285), -cm(30), -cm(310)),
            up(sx * cm(270), cm(80), -cm(260)),
            m(cm(45)),
            BONE_DARK,
            0,
        );
        // the lower jaw: a long tapering bone each side, a ridge under it
        bar(
            b,
            f,
            Mesh::Taper,
            dn(sx * cm(285), -cm(40), -cm(320)),
            dn(sx * cm(170), -cm(40), cm(330)),
            m(cm(90)),
            BONE,
            0,
        );
        bar(
            b,
            f,
            Mesh::Hex,
            dn(sx * cm(260), -cm(90), -cm(250)),
            dn(sx * cm(180), -cm(80), cm(200)),
            m(cm(50)),
            BONE_DARK,
            0,
        );
        // a crack across the snout
        put_turned(
            b,
            f,
            Mesh::Cube,
            up(sx * cm(150), cm(200), cm(230)),
            &Affine::rot_z(sx * deg(30)),
            p(cm(8), cm(90), cm(8)),
            VOID,
            0,
        );
    }
    // the chin, the lower gum and a moss cap on the skull
    put(
        b,
        f,
        Mesh::Bevel,
        dn(0, -cm(40), cm(320)),
        p(cm(340), cm(80), cm(110)),
        BONE,
        0,
    );
    put(
        b,
        f,
        Mesh::Bevel,
        dn(0, -cm(25), cm(40)),
        p(cm(420), cm(50), cm(600)),
        BONE_DARK,
        0,
    );
    put(
        b,
        f,
        Mesh::Blob,
        up(cm(40), cm(390), -cm(240)),
        p(cm(300), cm(60), cm(200)),
        MOSS,
        0,
    );
}
