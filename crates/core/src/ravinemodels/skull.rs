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
//! 57 shapes (28 teeth, 29 bone).

use super::*;
use crate::fx::{cm, deg, div, mul};

const TOOTH: Rgb = rgb(252, 246, 222);
/// The teeth catch the torchlight: a little of their own light, so they
/// read white against the dark fin from the approach (RA-12).
const TOOTH_GLOW: i32 = ONE / 6;
/// old bone stained by the cave
const WEATHER: Rgb = rgb(150, 128, 92);

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
/// skeleton: the low snout and the tall back of the skull above the upper
/// gum, the lower jaw under the lower gum, the mouth between them open
/// front to back. The jaw's rising back ends and their hinge stand
/// beside the mouth, inside the gap's height, and are left out (as the
/// hinge posts were before): a box there would close the gap.
pub fn rex_bone_boxes(gap: i32) -> Vec<(V3, V3)> {
    let s = share(gap);
    let m = |x: i32| mul(x, s);
    let gum = rex_gum(gap);
    vec![
        // the snout, low at the front
        (
            v3(0, gum + m(cm(130)), m(cm(230))),
            v3(m(cm(500)), m(cm(260)), m(cm(340))),
        ),
        // the deep back of the skull: eyes, cheeks, braincase
        (
            v3(0, gum + m(cm(230)), -m(cm(270))),
            v3(m(cm(600)), m(cm(460)), m(cm(660))),
        ),
        // the lower jaw
        (
            v3(0, -gum - m(cm(55)), m(cm(80))),
            v3(m(cm(540)), m(cm(110)), m(cm(560))),
        ),
    ]
}

/// The skull for gap `gap` with teeth `teeth` (`rex_teeth(gap)`, or the
/// course's own list): a long, deep T-rex skull, its snout sloping down
/// to the front, big holes in its side (the eye socket, the opening in
/// front of it, the nostril, the one behind the eye), the lower jaw
/// dropped open and hinged far back at the skull's lower corner, its
/// rising back ends beside the mouth. Pale bone with darker weathered
/// patches and cracks, a moss cap.
pub fn rex_skull(b: &mut Batch, f: &Affine, gap: i32, teeth: &[Tooth]) {
    let s = share(gap);
    let m = |x: i32| mul(x, s);
    let p = |x: i32, y: i32, z: i32| v3(m(x), m(y), m(z));
    let gum = rex_gum(gap);
    let up = |x: i32, y: i32, z: i32| v3(m(x), gum + m(y), m(z));
    let dn = |x: i32, y: i32, z: i32| v3(m(x), -gum + m(y), m(z));
    for t in teeth {
        bar(b, f, Mesh::Spike, t.base, t.tip, t.r, TOOTH, TOOTH_GLOW);
    }
    // the upper jaw's tooth row, the snout over it sloping down to the
    // front, the deep back of the skull, the ridge of the nose
    put(
        b,
        f,
        Mesh::Bevel,
        up(0, cm(35), cm(70)),
        p(cm(480), cm(70), cm(560)),
        BONE_DARK,
        0,
    );
    put(
        b,
        f,
        Mesh::Wedge,
        up(0, cm(170), cm(170)),
        p(cm(440), cm(240), cm(400)),
        BONE,
        0,
    );
    put(
        b,
        f,
        Mesh::Bevel,
        up(0, cm(110), cm(250)),
        p(cm(400), cm(150), cm(220)),
        BONE,
        0,
    );
    put(
        b,
        f,
        Mesh::Bevel,
        up(0, cm(220), -cm(170)),
        p(cm(560), cm(420), cm(460)),
        BONE,
        0,
    );
    put(
        b,
        f,
        Mesh::Geo,
        up(0, cm(330), -cm(330)),
        p(cm(420), cm(240), cm(380)),
        BONE_DARK,
        0,
    );
    bar(
        b,
        f,
        Mesh::Hex,
        up(0, cm(180), cm(340)),
        up(0, cm(400), -cm(60)),
        m(cm(110)),
        BONE,
        0,
    );
    for sx in [-1, 1] {
        // the holes in the side: the tall eye socket, the big opening in
        // front of it, the nostril at the snout, the one behind the eye
        put(
            b,
            f,
            Mesh::Ico,
            up(sx * cm(270), cm(300), -cm(90)),
            p(cm(60), cm(150), cm(110)),
            VOID,
            0,
        );
        put(
            b,
            f,
            Mesh::Ico,
            up(sx * cm(240), cm(160), cm(110)),
            p(cm(50), cm(140), cm(190)),
            VOID,
            0,
        );
        put(
            b,
            f,
            Mesh::Ico,
            up(sx * cm(195), cm(150), cm(320)),
            p(cm(40), cm(60), cm(90)),
            VOID,
            0,
        );
        put(
            b,
            f,
            Mesh::Ico,
            up(sx * cm(285), cm(230), -cm(380)),
            p(cm(50), cm(200), cm(90)),
            VOID,
            0,
        );
        // the bony horn over the eye
        put(
            b,
            f,
            Mesh::Spike,
            up(sx * cm(220), cm(440), -cm(50)),
            p(cm(70), cm(70), cm(90)),
            BONE_DARK,
            0,
        );
        // the cheek bar under the eye, back to the skull's lower back
        // corner, and the hinge knob there
        bar(
            b,
            f,
            Mesh::Taper,
            up(sx * cm(290), cm(30), -cm(560)),
            up(sx * cm(255), cm(40), cm(60)),
            m(cm(90)),
            BONE_DARK,
            0,
        );
        put(
            b,
            f,
            Mesh::Ico,
            up(sx * cm(300), -cm(30), -cm(600)),
            p(cm(120), cm(120), cm(140)),
            BONE_DARK,
            0,
        );
        // the lower jaw dropped open: a deep, thick plate each side, its
        // top edge sloping gently up from the chin to the hinge at the
        // back (only to the gap's middle: a plate up to the upper gum
        // read as a strut beside the mouth, RA-3), and its long tapering
        // lower edge
        let (top, foot) = (m(cm(10)), -gum - m(cm(70)));
        put(
            b,
            f,
            Mesh::Wedge,
            v3(m(sx * cm(310)), (top + foot) / 2, -m(cm(250))),
            v3(m(cm(120)), top - foot, m(cm(800))),
            BONE,
            0,
        );
        bar(
            b,
            f,
            Mesh::Taper,
            dn(sx * cm(270), -cm(50), -cm(230)),
            dn(sx * cm(160), -cm(40), cm(340)),
            m(cm(120)),
            BONE,
            0,
        );
        // weathering: a crack across the snout, a stained patch on the
        // cheek
        put_turned(
            b,
            f,
            Mesh::Cube,
            up(sx * cm(205), cm(230), cm(190)),
            &Affine::rot_z(sx * deg(30)),
            p(cm(8), cm(110), cm(8)),
            VOID,
            0,
        );
        put(
            b,
            f,
            Mesh::Blob,
            up(sx * cm(265), cm(90), -cm(200)),
            p(cm(60), cm(110), cm(160)),
            WEATHER,
            0,
        );
    }
    // the chin, the lower gum, a crack along the brow and a moss cap
    put(
        b,
        f,
        Mesh::Bevel,
        dn(0, -cm(50), cm(320)),
        p(cm(340), cm(100), cm(110)),
        BONE,
        0,
    );
    put(
        b,
        f,
        Mesh::Bevel,
        dn(0, -cm(25), cm(60)),
        p(cm(420), cm(50), cm(560)),
        BONE_DARK,
        0,
    );
    put(
        b,
        f,
        Mesh::Blob,
        up(cm(60), cm(440), -cm(250)),
        p(cm(260), cm(60), cm(200)),
        MOSS,
        0,
    );
}
