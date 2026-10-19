//! The tower map's Floor is Lava cat (`FLAME_CAT`, 2026-10-15; the person:
//! "can you make it look like the balrog somehow?"): a huge cat-shaped
//! demon of shadow and flame. The jaguar's body in smoke-black with glowing
//! ember cracks where the rosettes were, a crest of flame along the spine
//! and embers rising off it, ragged wings of smoke beating with its stride,
//! two great horns and burning eyes, and a long tail that is a whip of
//! fire. Drawn over `models::jaguar`'s cat; inspired by no one film's
//! design, only the idea of a fire demon.

use super::*;

pub(super) const SMOKE_BLACK: Rgb = rgb(34, 28, 30);
pub(super) const SMOKE_DEEP: Rgb = rgb(62, 44, 40);
pub(super) const SMOKE_BELLY: Rgb = rgb(66, 34, 28);
pub(super) const SMOKE_WING: Rgb = rgb(48, 42, 46);
pub(super) const SMOKE_PALE: Rgb = rgb(84, 74, 76);
pub(super) const EMBER: Rgb = rgb(255, 92, 18);
pub(super) const FLARE: Rgb = rgb(255, 196, 52);
pub(super) const CINDER: Rgb = rgb(200, 40, 14);
pub(super) const HORN: Rgb = rgb(26, 20, 22);
pub(super) const HELL_EYE: Rgb = rgb(255, 176, 36);

/// Where the ember cracks run on the body: two ends, each centimetres along
/// the spine from its middle and degrees round it from the top.
const CRACKS: [((i32, i32), (i32, i32)); 12] = [
    ((44, -10), (30, 40)),
    ((30, 40), (36, 80)),
    ((44, 10), (28, -45)),
    ((28, -45), (34, -90)),
    ((14, 0), (2, 60)),
    ((2, 60), (10, 105)),
    ((10, -20), (-4, -70)),
    ((-4, -70), (4, -110)),
    ((-18, 20), (-34, 70)),
    ((-22, -10), (-38, -60)),
    ((-40, 0), (-50, 50)),
    ((-44, -30), (-54, -85)),
];

/// A point on the body's surface (`along` cm along the spine, `round`
/// degrees from the top), and the way out of it.
fn on_body(body: &Frame, along: i32, round: i32) -> (V3, V3) {
    let a = deg(round);
    let n = body.dir(fx::sin(a), 0, fx::cos(a));
    let at = body.at(mul(fx::sin(a), cm(23)), cm(along), mul(fx::cos(a), cm(25)));
    (at, n)
}

/// The body's fire: glowing cracks, a crest of flame along the spine,
/// embers rising off the back and the smoke wings.
pub(super) fn body(rig: &mut Rig, j: &crate::jaguar::Jaguar, body: &Frame) {
    for ((a0, r0), (a1, r1)) in CRACKS {
        let (p, n) = on_body(body, a0, r0);
        let (q, _) = on_body(body, a1, r1);
        let out = n.scale(cm(1));
        rig.glow(Mesh::Hex, rod(p.add(out), q.add(out), cm(3)), EMBER, ONE);
    }
    // the crest: flames up the spine, flickering with the stride
    for (i, along) in [46, 32, 18, 4, -10, -24, -38].into_iter().enumerate() {
        let flick = fx::sin(j.gait.wrapping_mul(3).wrapping_add(deg(70) * i as i32));
        let high = cm(18) + mul(flick, cm(6)) + if i < 2 { cm(8) } else { 0 };
        let root = body.at(0, cm(along), cm(22));
        let tip = root.add(body.z.scale(high)).sub(body.y.scale(cm(10)));
        rig.glow(Mesh::Spike, rod(root, tip, cm(12)), EMBER, ONE);
        rig.glow(
            Mesh::Spike,
            rod(root, root.lerp(tip, ONE * 3 / 5), cm(6)),
            FLARE,
            ONE,
        );
    }
    // embers rising off the back, drifting up and shrinking (a turn is ONE)
    for k in 0..5 {
        let age = (j.gait / 4).wrapping_add(k * (TURN / 5)).rem_euclid(TURN);
        let up = mul(age, cm(70));
        let at = body
            .at(cm(18 - k * 10), cm(30 - k * 14), cm(30))
            .add(V3::UP.scale(up));
        let size = cm(5) - mul(age, cm(3));
        rig.glow(
            Mesh::Octa,
            Affine::translate(at).then(&Affine::scale(v3(size, size, size))),
            if k % 2 == 0 { FLARE } else { EMBER },
            ONE,
        );
    }
    // the wings: a bony spar from each shoulder out and up, with ragged
    // smoke hanging from it, beating with the stride
    let beat = mul(fx::sin(j.gait), deg(16)) + mul(j.leap, deg(20));
    for s in SIDES {
        let root = body.at(s * cm(16), cm(30), cm(20));
        let lift = deg(40) + beat;
        let out = body
            .x
            .scale(s * fx::cos(lift))
            .add(body.z.scale(fx::sin(lift)));
        let elbow = root.add(out.scale(cm(70))).sub(body.y.scale(cm(10)));
        let tip = elbow
            .add(out.scale(cm(56)))
            .add(body.z.scale(cm(26)))
            .sub(body.y.scale(cm(40)));
        rig.put(Mesh::Taper, rod(root, elbow, cm(9)), HORN);
        rig.put(Mesh::Taper, rod(elbow, tip, cm(6)), HORN);
        rig.glow(
            Mesh::Spike,
            rod(tip, tip.add(out.scale(cm(14))), cm(5)),
            EMBER,
            ONE,
        );
        // the smoke: shreds from the spar back and down to a ragged edge
        let hem = [
            (root.lerp(elbow, ONE / 2), 46),
            (elbow, 66),
            (elbow.lerp(tip, ONE / 2), 64),
            (tip, 52),
        ];
        for (k, (top, len)) in hem.into_iter().enumerate() {
            let low = top
                .sub(body.y.scale(cm(len)))
                .sub(body.z.scale(cm(len / 2)))
                .sub(out.scale(cm(8)));
            let c = if k % 2 == 0 { SMOKE_WING } else { SMOKE_PALE };
            rig.blade(top, low, out, cm(46), cm(4), c);
            // a glowing fringe at the shred's end
            rig.glow(
                Mesh::Octa,
                Affine::translate(low).then(&Affine::scale(v3(cm(6), cm(6), cm(6)))),
                CINDER,
                ONE,
            );
        }
    }
}

/// The head's demon parts over the cat's head `h` (30 cm across): two
/// great horns sweeping out, up and forward, a heavy brow, burning eyes
/// and a mane of flame behind the ears.
pub(super) fn head(rig: &mut Rig, j: &crate::jaguar::Jaguar, h: &Frame) {
    rig.put(Mesh::Bevel, h.shape(p3(0, 9, 12), p3(26, 6, 10)), HORN);
    for s in SIDES {
        let base = h.at(s * cm(9), cm(12), -cm(2));
        let mid = base
            .add(h.x.scale(s * cm(16)))
            .add(h.y.scale(cm(8)))
            .sub(h.z.scale(cm(6)));
        let tip = mid
            .add(h.x.scale(s * cm(4)))
            .add(h.y.scale(cm(16)))
            .add(h.z.scale(cm(12)));
        rig.put(Mesh::Taper, rod(base, mid, cm(10)), HORN);
        rig.put(Mesh::Spike, rod(mid, tip, cm(7)), HORN);
        // burning eyes, over the cat's own
        rig.glow(
            Mesh::Ico,
            h.shape(p3(s * 7, 4, 15), p3(7, 4, 4)),
            HELL_EYE,
            ONE,
        );
    }
    for (k, x) in [-8, 0, 8].into_iter().enumerate() {
        let flick = fx::sin(j.gait.wrapping_mul(3).wrapping_add(deg(110) * k as i32));
        let root = h.at(cm(x), cm(10), -cm(10));
        let tip = root
            .add(h.y.scale(cm(16) + mul(flick, cm(5))))
            .sub(h.z.scale(cm(18)));
        rig.glow(Mesh::Spike, rod(root, tip, cm(10)), EMBER, ONE);
    }
}

/// The tail: a long whip of fire, swaying with the stride and lashing
/// higher while leaping, deep red at the root to yellow at the crackling
/// tip.
pub(super) fn tail(rig: &mut Rig, j: &crate::jaguar::Jaguar, body: &Frame) {
    let back = body.y.neg();
    let side = body.x;
    let mut at = body.at(0, -cm(60), cm(8));
    let count = 14;
    let colours = [SMOKE_BLACK, CINDER, EMBER, FLARE];
    for i in 0..count {
        // out, up and over in an S, lashing side to side
        let rise = deg(30) - deg(i * 7) + mul(j.leap, deg(30));
        let lash = mul(fx::sin(j.gait.wrapping_sub(i * deg(30))), ONE / 2);
        let way = dir_or(
            back.scale(fx::cos(rise))
                .add(V3::UP.scale(fx::sin(rise)))
                .add(side.scale(lash)),
            back,
        );
        let next = at.add(way.scale(cm(11)));
        let w = cm(9) - cm(6) * i / count;
        let c = colours[(i * 4 / count) as usize];
        if c == SMOKE_BLACK {
            rig.put(Mesh::Hex, rod(at, next, w), c);
        } else {
            rig.glow(Mesh::Hex, rod(at, next, w), c, ONE);
        }
        at = next;
    }
    // the crackle at the end
    for k in 0..3 {
        let a = j.gait.wrapping_mul(2).wrapping_add(k * (TURN / 3));
        let out = side.scale(fx::cos(a)).add(V3::UP.scale(fx::sin(a)));
        rig.glow(
            Mesh::Spike,
            rod(at, at.add(out.scale(cm(14))), cm(6)),
            FLARE,
            ONE,
        );
    }
}
