//! The finish island drawn (BI1): exactly the ground the physics stands
//! apes on (`bay::Island::ground`): a sand top inside an irregular shore of
//! lobes and coves, the beach sloping into the sea, wet sand on under the
//! turquoise shallows, low grassy rises with bushes, rocks round the shore,
//! palms, the three rope palms reaching their ropes' anchors, and two black
//! flags on posts at the finish line. Drawn round the island's own centre
//! and heading, so a moved island draws in its new place.

use crate::bay::island::{SLOPE, WET};
use crate::bay::{BayLayout, Hanger, Island, ISLAND_TOP, WATER_LEVEL};
use crate::baymodels as m;
use crate::course::Course;
use crate::fx::{self, cm, cos, int, mul, sin, ONE, TURN};
use crate::geom::{right_of, v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{placed, rgb, rod, shade, Batch, Rgb};

const SAND: Rgb = rgb(222, 196, 138);
const SAND_SLOPE: Rgb = rgb(206, 180, 124);
const SAND_WET: Rgb = rgb(168, 144, 100);
const GRASS: Rgb = rgb(88, 156, 60);
const GRASS_DARK: Rgb = rgb(62, 128, 50);
const BUSH: Rgb = rgb(44, 112, 48);
const CLIFF: Rgb = rgb(150, 130, 104);
const CLIFF_DARK: Rgb = rgb(112, 96, 78);
const TRUNK: Rgb = rgb(132, 100, 70);
const FROND: Rgb = rgb(70, 160, 60);
const FROND_DARK: Rgb = rgb(44, 120, 50);
const PILE: Rgb = rgb(78, 60, 46);
const FLAG_BLACK: Rgb = rgb(20, 20, 22);
const BONE: Rgb = rgb(238, 232, 214);

/// Directions round the shore the outline is drawn from.
const SIDES: i32 = 128;
/// Nothing is drawn standing this near the lanes' middle (they run down
/// the island's middle, to `LANES_END` past the centre).
const LANES_CLEAR: i32 = int(28);

fn hash(k: u32, salt: u32) -> u32 {
    let mut x = k.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^ (x >> 12)
}

fn pick(k: u32, salt: u32, lo: i32, hi: i32) -> i32 {
    lo + (hash(k, salt) % ((hi - lo).max(0) as u32 + 1)) as i32
}

/// A slab whose top face is the triangle (`a`, `b`, `c`), `thick` deep
/// under it (the `Wedge` mesh's triangular ends laid along the triangle).
fn tri(b: &mut Batch, a: V3, p: V3, q: V3, thick: i32, colour: Rgb) {
    let (mut c1, mut c2) = (p.sub(a), q.sub(a));
    let n = c1.norm().cross(c2.norm());
    if n.y < 0 {
        std::mem::swap(&mut c1, &mut c2);
    }
    let n = c1.norm().cross(c2.norm());
    if n.len() < ONE / 1000 {
        return;
    }
    let c0 = n.norm().scale(thick);
    // the wedge's x = +½ end is the triangle: its corner (y, z) = (-½, -½)
    // at `a`, (+½, -½) at a + c1, (-½, +½) at a + c2
    let t = a
        .sub(c0.scale(ONE / 2))
        .add(c1.scale(ONE / 2))
        .add(c2.scale(ONE / 2));
    b.push(Mesh::Wedge, &Affine { c0, c1, c2, t }, colour, 0);
}

/// The point at distance `r` from the island's centre in direction `a`
/// (0 along its heading, a quarter to the right), at height `y`.
fn polar(isl: &Island, a: i32, r: i32, y: i32) -> V3 {
    let p = isl.at(mul(cos(a), r), mul(sin(a), r));
    v3(p.x, y, p.z)
}

/// The shoreline's distance in direction `a`.
fn shore(isl: &Island, a: i32) -> i32 {
    isl.shore(cos(a), sin(a))
}

/// Whether a thing standing at (`along`, `across`) of the island's centre
/// is clear of the lanes (so it is never in a racer's way).
fn clear_of_lanes(along: i32, across: i32) -> bool {
    across.abs() > LANES_CLEAR || along > crate::bay::island::LANES_END + int(12)
}

/// The island, into `batches` (the big ground into the last, the rest by
/// `chunk`).
pub fn scenery(batches: &mut [Batch], c: &Course, bay: &BayLayout, chunk: &dyn Fn(V3) -> usize) {
    let isl = &bay.island;
    let last = batches.len() - 1;
    let top = |p: V3| isl.ground(p.x, p.z).unwrap_or(ISLAND_TOP);
    {
        let b = &mut batches[last];
        let centre = polar(isl, 0, 0, ISLAND_TOP);
        let wet_low = WATER_LEVEL - mul(ISLAND_TOP - WATER_LEVEL, fx::div(WET, SLOPE));
        for i in 0..SIDES {
            let (a0, a1) = (i * TURN / SIDES, (i + 1) * TURN / SIDES);
            let (r0, r1) = (shore(isl, a0), shore(isl, a1));
            let t0 = polar(isl, a0, r0 - SLOPE, ISLAND_TOP);
            let t1 = polar(isl, a1, r1 - SLOPE, ISLAND_TOP);
            // the fan's slices overlap their neighbours a little (the same
            // flat sand: no cracks between them)
            let e = TURN / SIDES / 5;
            let ta = polar(isl, a0 - e, shore(isl, a0 - e) - SLOPE, ISLAND_TOP);
            let tb = polar(isl, a1 + e, shore(isl, a1 + e) - SLOPE, ISLAND_TOP);
            let s0 = polar(isl, a0, r0, WATER_LEVEL);
            let s1 = polar(isl, a1, r1, WATER_LEVEL);
            let w0 = polar(isl, a0, r0 + WET, wet_low);
            let w1 = polar(isl, a1, r1 + WET, wet_low);
            // the flat top, a fan from the middle
            tri(b, centre, ta, tb, int(3), SAND);
            // the beach sloping into the sea
            tri(b, t0, t1, s0, int(2), SAND_SLOPE);
            tri(b, t1, s1, s0, int(2), SAND_SLOPE);
            // the wet sand on under the shallows
            tri(b, s0, s1, w0, int(2), SAND_WET);
            tri(b, s1, w1, w0, int(2), SAND_WET);
        }
        // the grassy rises, bushes on them
        for (k, md) in isl.mounds().iter().enumerate() {
            let mid = isl.at(md.along, md.across).add(v3(0, ISLAND_TOP, 0));
            b.push(
                Mesh::Geo,
                &Affine::translate(mid)
                    .then(&Affine::looking(isl.fwd, V3::UP))
                    .then(&Affine::scale(v3(
                        md.half_wide * 2,
                        md.height * 2,
                        md.half_len * 2,
                    ))),
                if k % 2 == 0 { GRASS } else { GRASS_DARK },
                0,
            );
            for j in 0..4u32 {
                let kk = k as u32 * 16 + j;
                let along = md.along + mul(md.half_len, pick(kk, 3, -60, 60) * ONE / 100);
                let across = md.across + mul(md.half_wide, pick(kk, 4, -55, 55) * ONE / 100);
                let p = isl.at(along, across);
                let s = int(pick(kk, 5, 3, 6));
                let foot = p.add(v3(0, top(p), 0));
                b.push(
                    Mesh::Blob,
                    &placed(foot, pick(kk, 6, 0, TURN), v3(s * 3 / 2, s, s * 3 / 2)),
                    if j % 2 == 0 { BUSH } else { FROND_DARK },
                    0,
                );
            }
        }
    }
    // rocks round the shore, half in the sea, and a few on the beach
    for k in 0..26u32 {
        let a = pick(k, 20, 0, TURN - 1);
        let r = shore(isl, a);
        let on_beach = k % 3 == 0;
        let d = if on_beach {
            r - int(pick(k, 21, 8, 14))
        } else {
            r + int(pick(k, 21, -2, 5))
        };
        let (along, across) = (mul(cos(a), d), mul(sin(a), d));
        if !clear_of_lanes(along, across) {
            continue;
        }
        let p = polar(isl, a, d, 0);
        let s = int(pick(k, 22, 2, 5));
        let y = if on_beach { top(p) } else { -cm(60) };
        b_push_rock(&mut batches[chunk(p)], p.add(v3(0, y, 0)), s, k);
    }
    // palms over the beach and on the rises, out of the lanes
    let mut planted = 0;
    let mut k = 0u32;
    while planted < 26 && k < 200 {
        k += 1;
        let a = pick(k, 27, 0, TURN - 1);
        let r = shore(isl, a);
        let d = r - int(pick(k, 28, 9, 40));
        let (along, across) = (mul(cos(a), d), mul(sin(a), d));
        if d <= 0 || !clear_of_lanes(along, across) {
            continue;
        }
        let p = polar(isl, a, d, 0);
        let foot = p.add(v3(0, top(p), 0));
        m::palm(
            &mut batches[chunk(foot)],
            &m::pose(foot, 0, ONE * 2),
            V3::ZERO,
            int(pick(k, 29, 6, 10)),
            k + 60,
        );
        planted += 1;
    }
    // the rope palms: a curved trunk from the foot to the rope's anchor
    for (i, h) in bay.hangers.iter().enumerate() {
        let Hanger::Shore { foot } = *h else { continue };
        if foot.y != ISLAND_TOP {
            continue;
        }
        let Some(v) = c.vines.get(i) else { continue };
        let foot = v3(foot.x, top(foot), foot.z);
        rope_palm(&mut batches[chunk(foot)], foot, v.anchor, i as u32);
    }
    // the finish line: two posts with black flags, out past the lanes
    let n = c.path.len() - 1;
    let fin = c.path[n];
    let r = right_of(c.heading[n]);
    for s in [-1, 1] {
        let at = fin.add(r.scale(s * (c.half_width + int(6))));
        let foot = v3(at.x, top(at), at.z);
        let b = &mut batches[chunk(foot)];
        let head = foot.add(v3(0, int(14), 0));
        b.push(Mesh::Hex, &rod(foot, head, cm(40)), PILE, 0);
        let mid = head.sub(v3(0, int(2), 0)).add(r.scale(-s * int(2)));
        b.push(
            Mesh::Cube,
            &Affine::translate(mid)
                .then(&Affine::looking(r, V3::UP))
                .then(&Affine::scale(v3(cm(12), int(3), int(4)))),
            FLAG_BLACK,
            0,
        );
        b.push(
            Mesh::Ico,
            &placed(mid.add(v3(0, cm(30), 0)), 0, v3(cm(90), cm(80), cm(90))),
            BONE,
            0,
        );
    }
}

fn b_push_rock(b: &mut Batch, at: V3, s: i32, k: u32) {
    b.push(
        Mesh::Rock,
        &Affine::translate(at)
            .then(&Affine::rot_y(pick(k, 23, 0, TURN)))
            .then(&Affine::scale(v3(s, s * 3 / 4, s))),
        if k % 3 == 1 { CLIFF_DARK } else { CLIFF },
        0,
    );
}

/// A tall palm whose trunk curves from `foot` up to the rope's `anchor`
/// (the rope hangs from under its crown), and a crown of fronds.
fn rope_palm(b: &mut Batch, foot: V3, anchor: V3, k: u32) {
    let top = anchor.add(v3(0, cm(60), 0));
    let lean = top.sub(foot).flat();
    let n = 7;
    let mut p = foot;
    for i in 1..=n {
        let t = i * ONE / n;
        // the lean grows with the square of the height: a palm's curve
        let q = foot.add(lean.scale(mul(t, t)));
        let q = v3(q.x, foot.y + mul(top.y - foot.y, t), q.z);
        // thick enough for a 34 m palm (BP4)
        let w = cm(240) - i * cm(16);
        b.push(
            Mesh::Taper,
            &rod(p, q, w),
            if i % 2 == 0 { TRUNK } else { shade(TRUNK, 85) },
            0,
        );
        p = q;
    }
    for i in 0..11 {
        let yaw = i * TURN / 11 + pick(k, 30, 0, 3000);
        let d = v3(fx::sin(yaw), 0, fx::cos(yaw));
        let mid = top.add(d.scale(int(5))).add(v3(0, cm(120), 0));
        let tip = top.add(d.scale(int(9))).sub(v3(0, int(3), 0));
        for (a, z) in [(top, mid), (mid, tip)] {
            b.push(
                Mesh::Shard,
                &rod(a, z, ONE).then(&Affine::scale(v3(int(3), ONE, cm(18)))),
                if i % 2 == 0 { FROND } else { FROND_DARK },
                0,
            );
        }
    }
    for i in 0..3 {
        let d = v3(fx::sin(i * TURN / 3), 0, fx::cos(i * TURN / 3));
        b.push(
            Mesh::Ico,
            &placed(
                top.add(d.scale(cm(70))).sub(v3(0, cm(50), 0)),
                0,
                v3(cm(60), cm(60), cm(60)),
            ),
            PILE,
            0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bay::bay_seeded;

    /// The drawn island's sand is the physics' ground: the top of every
    /// shape the island draws over open ground (not a palm, rock, bush or
    /// post) is at most the ground there (nothing looks walkable that
    /// isn't), and the flat top is drawn exactly at `ISLAND_TOP`.
    #[test]
    fn the_drawn_shore_is_where_the_physics_has_it() {
        for seed in [1, 2, 3] {
            let c = bay_seeded(seed);
            let bay = c.bay.as_deref().unwrap();
            let isl = &bay.island;
            // every direction: just inside the shore is land, just out is sea
            for i in 0..SIDES {
                let a = i * TURN / SIDES;
                let r = shore(isl, a);
                let inside = polar(isl, a, r - cm(50), 0);
                let out = polar(isl, a, r + cm(50), 0);
                let g_in = isl.ground(inside.x, inside.z).unwrap();
                let g_out = isl.ground(out.x, out.z).unwrap();
                assert!(g_in > WATER_LEVEL, "seed {seed} dir {i}: {g_in}");
                assert!(g_out < WATER_LEVEL, "seed {seed} dir {i}: {g_out}");
                // the drawn top edge is the flat top's
                let t = polar(isl, a, r - SLOPE - cm(20), 0);
                assert_eq!(
                    isl.ground(t.x, t.z),
                    Some(ISLAND_TOP),
                    "seed {seed} dir {i}"
                );
            }
        }
    }
}
