//! Rattlesnake Ravine's Floor is Lava cat (`SPIDER_JAGUAR`, 2026-10-17; the
//! person: "should still be a jaguar but with 8 legs and arachnid
//! themed"): the jaguar's spotted body and head, but on eight long banded
//! legs that arch up and out like a spider's, walking in two alternating
//! sets of four on the jaguar's own gait; the jaguar's own eyes (the
//! person, 2026-10-17: no extra red ones), two dark fangs under the muzzle, and a striped spider's abdomen
//! swelling over the haunches with a red hourglass and spinnerets.

use super::*;

pub(super) const SPIDER_FANG: Rgb = rgb(44, 26, 26);
pub(super) const HOURGLASS: Rgb = rgb(214, 30, 36);

/// The leg pairs: centimetres along the spine from its middle, and how far
/// ahead (or behind) of the root each foot comes down.
const PAIRS: [(i32, i32); 4] = [(40, 26), (16, 8), (-12, -10), (-38, -28)];

/// The eight legs, posed from the jaguar's gait, leap and eating.
pub(super) fn legs(rig: &mut Rig, j: &crate::jaguar::Jaguar, body: &Frame, fwd: V3, cat: &BigCat) {
    let up = V3::UP;
    for (i, &(along, spread)) in PAIRS.iter().enumerate() {
        let front = i < 2;
        for side in SIDES {
            let root = body.at(-side * cm(16), cm(along), -cm(4));
            let out = dir_or(root.sub(body.at(0, cm(along), -cm(4))).flat(), body.x);
            let under = v3(root.x, cm(3), root.z)
                .add(out.scale(cm(42)))
                .add(fwd.scale(cm(spread)));
            // two sets of four in step, each half a stride from the other,
            // a little wave from front to back
            let set = (i as i32 + (side + 1) / 2) % 2;
            let lp = j.gait + set * HALF + i as i32 * (TURN / 12);
            let stride = mul(j.stride, cm(22));
            let lift = mul(j.stride, cm(16));
            let run = under
                .add(fwd.scale(mul(fx::sin(lp), stride)))
                .add(up.scale(mul(fx::cos(lp).max(0), lift)));
            // the leap: the front legs reach out ahead, the back ones push
            let reach = root
                .add(fwd.scale(if front { cm(54) } else { -cm(54) }))
                .add(out.scale(cm(24)))
                .sub(up.scale(cm(22)));
            let eat = under.add(fwd.scale(cm(12)));
            let target = run
                .lerp(reach, j.leap)
                .lerp(eat, if front { j.head_down } else { 0 });
            // the knee arches up and out
            let hint = dir_or(up.add(out), up);
            let (mid, end) = bend(root, target, cm(108), hint);
            rig.bone(root, mid, cm(12), cat.coat);
            rig.bone(mid, end, cm(8), shade(cat.coat, 90));
            // dark bands round the legs
            for (a, b, k, w) in [
                (root, mid, ONE / 2, cm(14)),
                (mid, end, ONE / 3, cm(10)),
                (mid, end, ONE * 2 / 3, cm(9)),
            ] {
                let at = a.lerp(b, k);
                let way = dir_or(b.sub(a), up);
                rig.put(
                    Mesh::Hex,
                    rod(at.sub(way.scale(cm(2))), at.add(way.scale(cm(2))), w),
                    cat.spot,
                );
            }
            // a dark claw at the tip
            rig.put(
                Mesh::Spike,
                rod(end.add(up.scale(cm(3))), end.sub(up.scale(cm(2))), cm(6)),
                cat.spot,
            );
        }
    }
}

/// The spider's abdomen swelling over the haunches: dark stripes across its
/// top, a red hourglass and two spinnerets at the back.
pub(super) fn abdomen(rig: &mut Rig, _j: &crate::jaguar::Jaguar, body: &Frame, cat: &BigCat) {
    rig.put(
        Mesh::Geo,
        body.shape(p3(0, -66, 22), p3(54, 64, 48)),
        cat.deep,
    );
    for (y, z) in [(-54, 41), (-66, 43), (-78, 41)] {
        rig.put(Mesh::Bevel, body.shape(p3(0, y, z), p3(40, 5, 4)), cat.spot);
    }
    for (y, tip) in [(-86, -92), (-98, -92)] {
        let a = body.at(0, cm(y), cm(36));
        let b = body.at(0, cm(tip), cm(37));
        rig.put(Mesh::Pyramid, rod(a, b, cm(9)), HOURGLASS);
    }
    for s in SIDES {
        let a = body.at(s * cm(4), -cm(94), cm(16));
        let b = body.at(s * cm(5), -cm(104), cm(12));
        rig.put(Mesh::Spike, rod(a, b, cm(5)), cat.spot);
    }
}

/// Two fangs under the muzzle (the eyes are the jaguar's own).
pub(super) fn head(rig: &mut Rig, _j: &crate::jaguar::Jaguar, h: &Frame) {
    for s in SIDES {
        let top = h.at(s * cm(4), -cm(8), cm(17));
        let tip = top.sub(h.y.scale(cm(9))).add(h.z.scale(cm(2)));
        rig.put(Mesh::Spike, rod(top, tip, cm(4)), SPIDER_FANG);
    }
}
