//! The finish island's shape and its chest's solid body (BI rows).
//!
//! The island is an oval round `Island.centre` (`radius` toward the
//! course, `back` past the finish, `wide` to each side), its shore pushed
//! out and pulled in by a few lobes and coves (`Island.lobes`; none on the
//! landing side, so the shore the racers land on is a smooth curve). Its
//! ground (`Island::ground`), the one the physics stands apes on and the
//! drawing draws:
//!
//! - out from the shoreline, wet sand sloping on under the sea (`WET`);
//! - from the waterline in, a sand beach rising to `ISLAND_TOP` over
//!   `SLOPE`;
//! - the flat beach top, the lanes run over;
//! - a few low grassy rises (`Island::mounds`), each a half ellipsoid on
//!   the beach top, all outside the lanes (never where a racer can be).
//!
//! The chest (`Island.chest`, `chest_size`: width, height to the rim,
//! depth) is a solid box: an ape stands on its heaped bananas (`chest_top`)
//! and is stopped against its sides (`Island::chest_push`).
//!
//! Everything is built from `centre` and `fwd` (`build`), so a moved island
//! keeps its shape and its chest.

use super::*;
use crate::course::{Course, Platform};
use crate::fx::{atan2, cos, div, mul, sqrt};
use crate::rules::{BODY_DROP, BODY_RADIUS, WALL_BOUNCE};

/// The beach's slope: from the waterline up to `ISLAND_TOP` over this.
pub const SLOPE: i32 = int(6);
/// The wet sand goes on under the sea this far out from the waterline.
pub const WET: i32 = int(8);
/// The island's reach past its centre and to each side.
pub const BACK: i32 = int(150);
pub const WIDE: i32 = int(72);
/// The shoreline's nearest point is this far in front of the beach the
/// course measures from (`Island.radius`).
pub const TIP: i32 = int(1);
/// The chest's place past the island's centre (the finish is 50 m before
/// the centre; finishers slide into it, BI2).
pub const CHEST_PAST: i32 = int(8);
/// The heap of bananas over the chest's rim, at its middle.
pub const HEAP: i32 = int(3);
/// The lanes' platform (`lanes_platform`) ends this far past the centre
/// at most (the course's far wall stops a finisher 95 m past it).
pub const LANES_END: i32 = int(97);

/// A grassy rise on the beach top: a half ellipsoid (its middle on the
/// flat top), in the island's frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mound {
    /// along `fwd` and to the right of the island's centre
    pub along: i32,
    pub across: i32,
    /// its half length and half width
    pub half_len: i32,
    pub half_wide: i32,
    /// its height over the beach top
    pub height: i32,
}

/// The island for a course whose finish is `c.length` and whose beach
/// starts `beach` along (just past the last ship's bow): its centre 50 m
/// past the finish along the heading there, its shape from `dice`, the
/// chest past the centre facing back down the course.
pub fn build(c: &Course, beach: i32, dice: &mut crate::fx::Dice) -> Island {
    let last = c.path.len() - 1;
    let fwd = c.heading[last].flat().norm();
    let centre = c.path[last].add(fwd.scale(int(50)));
    let radius = c.length + int(50) - beach;
    let mut lobes = [(0, 0, 0); 4];
    for (n, l) in lobes.iter_mut().enumerate() {
        // 2–5 waves round, small amplitudes: a few lobes and coves
        let k = [2, 3, 5, 7][n];
        let amp = [4, 3, 2, 2][n] * ONE / 100 + dice.range(0, 1) * ONE / 100;
        *l = (k, amp, dice.range(0, TURN - 1));
    }
    let chest = centre.add(fwd.scale(CHEST_PAST));
    Island {
        centre: v3(centre.x, ISLAND_TOP, centre.z),
        radius,
        fwd,
        back: BACK,
        wide: WIDE,
        lobes,
        chest: v3(chest.x, ISLAND_TOP, chest.z),
        chest_yaw: crate::geom::yaw_of(fwd.neg()),
        chest_size: v3(int(16), int(8), int(10)),
    }
}

/// A flat box under the lanes wholly inside the island's flat beach top
/// (so it holds exactly where the island does): the camera's floor there
/// (`camera::hulls` reads `Course.platforms`).
pub fn lanes_platform(isl: &Island, half_width: i32) -> Platform {
    // the first and last places along where the whole width is beach top
    let flat = |along: i32| {
        (-half_width / ONE..=half_width / ONE).all(|w| {
            let p = isl.at(along, int(w));
            isl.inland(p.x, p.z) >= SLOPE
        })
    };
    let mut from = -isl.radius;
    while !flat(from) && from < 0 {
        from += cm(50);
    }
    let mut to = LANES_END;
    while !flat(to) && to > from {
        to -= cm(50);
    }
    let mid = isl.at((from + to) / 2, 0);
    Platform {
        centre: v3(mid.x, ISLAND_TOP, mid.z),
        fwd: isl.fwd,
        half_len: (to - from) / 2,
        half_width,
        top: ISLAND_TOP,
    }
}

impl Island {
    /// The point `along` the island's `fwd` and `across` to its right of
    /// the centre, on the water.
    pub fn at(&self, along: i32, across: i32) -> V3 {
        let p = self
            .centre
            .add(self.fwd.scale(along))
            .add(right_of(self.fwd).scale(across));
        v3(p.x, 0, p.z)
    }

    /// (along, across) of (x, z) in the island's frame.
    pub fn local(&self, x: i32, z: i32) -> (i32, i32) {
        let d = v3(x - self.centre.x, 0, z - self.centre.z);
        (d.dot(self.fwd), d.dot(right_of(self.fwd)))
    }

    /// The shore's push out (ONE = none) in the direction `angle` (TURN
    /// units, 0 along `fwd`, a quarter to the right): the lobes, none on
    /// the landing side (half a turn).
    fn lobed(&self, angle: i32) -> i32 {
        let towards = (ONE + cos(angle)).min(ONE);
        let mut f = ONE;
        for &(k, amp, phase) in &self.lobes {
            f += mul(
                mul(amp, cos(angle.wrapping_mul(k).wrapping_add(phase))),
                towards,
            );
        }
        f
    }

    /// The shoreline's distance from the centre in the direction (`along`,
    /// `across`) (not both 0).
    pub fn shore(&self, along: i32, across: i32) -> i32 {
        let a = if along >= 0 {
            self.back
        } else {
            self.radius + TIP
        };
        let len = v3(along, 0, across).len().max(1);
        let (cu, sw) = (div(along, len), div(across, len));
        // the oval's radius that way: 1 / |(cu / a, sw / b)|
        let e = v3(div(cu, a / 64), 0, div(sw, self.wide / 64)).len().max(1);
        let oval = div(ONE * 64, e);
        mul(oval, self.lobed(atan2(across, along)))
    }

    /// How far (x, z) is in from the shoreline (negative: out in the sea).
    pub fn inland(&self, x: i32, z: i32) -> i32 {
        let (u, w) = self.local(x, z);
        if u == 0 && w == 0 {
            return self.shore(ONE, 0);
        }
        self.shore(u, w) - v3(u, 0, w).len()
    }

    /// The grassy rises: one each side by the finish, one each side
    /// further on, one across the far end; all clear of the lanes.
    pub fn mounds(&self) -> [Mound; 13] {
        let r = self.radius;
        let (b, w) = (self.back, self.wide);
        // each island's rises a little different (from its shore's phases)
        let vary = |k: usize, span: i32| {
            let p = self.lobes[k % 4].2;
            mul(span, (p % 41 - 20) * ONE / 100)
        };
        let m = |along, across, half_len, half_wide, height| Mound {
            along,
            across,
            half_len,
            half_wide,
            height,
        };
        // a long rise, and a lower knoll at each end of it
        let rise = |k: usize, along: i32, across: i32, half_len: i32, height: i32| {
            let along = along + vary(k, half_len / 2);
            let small = half_len * 45 / 100;
            [
                m(along, across, half_len, w / 6, height),
                m(
                    along - half_len * 7 / 10,
                    across,
                    small,
                    w / 8,
                    height * 6 / 10,
                ),
                m(
                    along + half_len * 7 / 10,
                    across,
                    small,
                    w / 8,
                    height * 7 / 10,
                ),
            ]
        };
        let [a0, a1, a2] = rise(0, -r / 4, w * 55 / 100, r / 3, int(3));
        let [b0, b1, b2] = rise(1, -r / 4, -w * 55 / 100, r / 3, cm(350));
        let [c0, c1, c2] = rise(2, b * 3 / 10, w * 55 / 100, b / 4, cm(450));
        let [d0, d1, d2] = rise(3, b * 3 / 10, -w * 55 / 100, b / 4, int(4));
        [
            a0,
            a1,
            a2,
            b0,
            b1,
            b2,
            c0,
            c1,
            c2,
            d0,
            d1,
            d2,
            m(b * 69 / 100, vary(1, w / 10), b / 20, w * 45 / 100, cm(450)),
        ]
    }

    /// The island's ground at (x, z), if the island reaches there: the wet
    /// sand under the sea, the beach's slope, the flat top or a rise.
    pub fn ground(&self, x: i32, z: i32) -> Option<i32> {
        // (quick reject: the physics asks every tick)
        let reach = self.back.max(self.radius + TIP).max(self.wide) * 5 / 4 + WET;
        if (x - self.centre.x).abs() > reach || (z - self.centre.z).abs() > reach {
            return None;
        }
        let d = self.inland(x, z);
        if d < -WET {
            return None;
        }
        if d < SLOPE {
            return Some(WATER_LEVEL + mul(ISLAND_TOP - WATER_LEVEL, div(d, SLOPE)));
        }
        let (u, w) = self.local(x, z);
        let mut top = ISLAND_TOP;
        for m in self.mounds() {
            let (du, dw) = (div(u - m.along, m.half_len), div(w - m.across, m.half_wide));
            if du.abs() >= ONE || dw.abs() >= ONE {
                continue;
            }
            let r2 = mul(du, du) + mul(dw, dw);
            if r2 < ONE {
                top = top.max(ISLAND_TOP + mul(m.height, sqrt(ONE - r2)));
            }
        }
        Some(top)
    }

    /// The chest's axes: across its front, and the way its front faces.
    fn chest_axes(&self) -> (V3, V3) {
        let face = v3(fx::sin(self.chest_yaw), 0, fx::cos(self.chest_yaw));
        (right_of(face), face)
    }

    /// (across, out the front) of (x, z) from the chest's middle.
    fn chest_local(&self, x: i32, z: i32) -> (i32, i32) {
        let (r, f) = self.chest_axes();
        let d = v3(x - self.chest.x, 0, z - self.chest.z);
        (d.dot(r), d.dot(f))
    }

    /// The top of the chest's heap of bananas over (x, z), if (x, z) is
    /// over the chest: the rim, heaped `HEAP` higher in the middle.
    pub fn chest_top(&self, x: i32, z: i32) -> Option<i32> {
        let (hw, hd) = (self.chest_size.x / 2, self.chest_size.z / 2);
        if (x - self.chest.x).abs() > hw + hd || (z - self.chest.z).abs() > hw + hd {
            return None;
        }
        let (a, f) = self.chest_local(x, z);
        if a.abs() > hw || f.abs() > hd {
            return None;
        }
        Some(self.heap(a, f))
    }

    /// The heap's top over (`a` across, `f` out the front) of the chest's
    /// middle (clamped to the chest).
    fn heap(&self, a: i32, f: i32) -> i32 {
        let (hw, hd) = (self.chest_size.x / 2, self.chest_size.z / 2);
        let (da, df) = (div(a.clamp(-hw, hw), hw), div(f.clamp(-hd, hd), hd));
        let r2 = (mul(da, da) + mul(df, df)).min(ONE);
        self.chest.y + self.chest_size.y + mul(HEAP, ONE - r2)
    }

    /// A body at `pos` (the hands) moving at `vel` that has run into the
    /// chest's side: put back out of it (by the side it is least far
    /// through), its speed into that side gone but a little bounce. `None`
    /// if it is clear of the chest, or high enough to stand on it.
    pub fn chest_push(&self, pos: V3, vel: V3) -> Option<(V3, V3)> {
        let (hw, hd) = (
            self.chest_size.x / 2 + BODY_RADIUS,
            self.chest_size.z / 2 + BODY_RADIUS,
        );
        if (pos.x - self.chest.x).abs() > hw + hd || (pos.z - self.chest.z).abs() > hw + hd {
            return None;
        }
        let (a, f) = self.chest_local(pos.x, pos.z);
        let feet = pos.y - BODY_DROP;
        if pos.y < self.chest.y || feet >= self.heap(a, f) - cm(60) {
            return None;
        }
        let (in_a, in_f) = (hw - a.abs(), hd - f.abs());
        if in_a <= 0 || in_f <= 0 {
            return None;
        }
        let (r, fw) = self.chest_axes();
        let (out, by) = if in_f <= in_a {
            (if f >= 0 { fw } else { fw.neg() }, in_f)
        } else {
            (if a >= 0 { r } else { r.neg() }, in_a)
        };
        let pos = pos.add(out.scale(by));
        let into = -vel.dot(out);
        let vel = if into > 0 {
            vel.add(out.scale(into + mul(into, WALL_BOUNCE)))
        } else {
            vel
        };
        Some((pos, vel))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ape::{Controls, Mode};
    use crate::race::{Driver, Entry, Race};
    use crate::rules::Kind;

    fn running(seed: u64) -> Race {
        let entries = [Entry {
            name: "P0".into(),
            kind: Kind::Gorilla,
            driver: Driver::Local,
            palette: 0,
            costume: crate::costume::Costume::Own,
        }];
        let mut r = Race::new(crate::bay::bay_seeded(seed), &entries, false, 1);
        while r.phase != crate::race::Phase::Running {
            r.step(&[Controls::default()]);
        }
        r
    }

    fn put(r: &mut Race, p: V3, v: V3) {
        let a = &mut r.racers[0].ape;
        a.pos = p;
        a.vel = v;
        a.mode = Mode::Flying;
        a.hint = r.course.nearest_anywhere(p);
    }

    #[test]
    fn the_island_is_land_inside_its_irregular_shore_and_sea_past_it() {
        for seed in [1, 2, 3, 4] {
            let c = crate::bay::bay_seeded(seed);
            let b = c.bay.as_deref().unwrap();
            let isl = &b.island;
            // not an oval, nor a square: the shore (against the plain
            // oval) goes in and out round the sides and back
            let mut plain = *isl;
            plain.lobes = [(0, 0, 0); 4];
            let ratio: Vec<i32> = (0..64)
                .map(|i| {
                    let (u, w) = (fx::cos(i * TURN / 64), fx::sin(i * TURN / 64));
                    div(isl.shore(u, w), plain.shore(u, w))
                })
                .collect();
            let coves = (0..64)
                .filter(|&i| {
                    let (p, q, n) = (ratio[(i + 63) % 64], ratio[i], ratio[(i + 1) % 64]);
                    q < p && q <= n
                })
                .count();
            assert!(coves >= 2, "seed {seed}: {coves} coves {ratio:?}");
            let spread = ratio.iter().max().unwrap() - ratio.iter().min().unwrap();
            assert!(spread > ONE / 20, "seed {seed}: shore spread {spread}");
            // the landing side is the plain oval's curve
            assert!((ratio[32] - ONE).abs() < ONE / 200, "seed {seed}");
            // land just in, sea just out, all round: the water rule agrees
            for i in 0..96 {
                let (u, w) = (fx::cos(i * TURN / 96), fx::sin(i * TURN / 96));
                let r = isl.shore(u, w);
                for (d, wet) in [(-int(2), false), (int(2), true)] {
                    let p = isl.at(mul(u, r + d), mul(w, r + d));
                    let feet = isl.ground(p.x, p.z).unwrap_or(0).max(0);
                    let body = v3(p.x, feet + BODY_DROP, p.z);
                    assert_eq!(c.in_bay_water(b, body, 0), wet, "seed {seed} dir {i} {d}");
                }
            }
            // the lanes: flat beach top from the platform's front to its
            // end (but the chest)
            let pl = c.platforms.last().unwrap();
            assert_eq!(pl.top, ISLAND_TOP);
            let finish = c.path[c.path.len() - 1];
            assert_eq!(c.support(v3(finish.x, int(5), finish.z)), ISLAND_TOP);
            let front = pl.centre.sub(pl.fwd.scale(pl.half_len));
            let (fu, _) = isl.local(front.x, front.z);
            assert!(fu < -int(60), "seed {seed}: lanes' top from {}", fu / ONE);
            let mut u = fu;
            while u < LANES_END {
                for w in -20..=20 {
                    let p = isl.at(u, int(w));
                    if isl.chest_top(p.x, p.z).is_none() {
                        assert_eq!(isl.ground(p.x, p.z), Some(ISLAND_TOP), "{seed} {u} {w}");
                    }
                }
                u += int(1);
            }
            // the palm ropes' feet stand on the beach top
            for h in &b.hangers {
                if let Hanger::Shore { foot } = h {
                    if foot.y == ISLAND_TOP {
                        assert_eq!(isl.ground(foot.x, foot.z), Some(ISLAND_TOP));
                    }
                }
            }
            // every rise sits on the flat top, out of the lanes
            for m in isl.mounds() {
                for (du, dw) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let p = isl.at(m.along + du * m.half_len, m.across + dw * m.half_wide);
                    assert!(isl.inland(p.x, p.z) >= SLOPE, "seed {seed}: {m:?}");
                }
                assert!(
                    m.across.abs() - m.half_wide > int(24) || m.along - m.half_len > int(95),
                    "{m:?}"
                );
            }
            // the chest stands in the lanes past the finish
            let (cu, cw) = isl.local(isl.chest.x, isl.chest.z);
            assert!(cu > -int(40) && cu < LANES_END - int(20) && cw.abs() < cm(5));
        }
    }

    #[test]
    fn a_king_kong_landing_stops_against_the_chest_and_can_stand_on_it() {
        for seed in [1, 2] {
            let mut r = running(seed);
            let isl = r.course.bay.as_deref().unwrap().island;
            let (hw, hd) = (isl.chest_size.x / 2, isl.chest_size.z / 2);
            let rim = isl.chest.y + isl.chest_size.y;
            let face = v3(fx::sin(isl.chest_yaw), 0, fx::cos(isl.chest_yaw));
            let across = right_of(face);
            // flying in fast and low at its front (King Kong's 40 m/s, the
            // hook's 80), straight and slanting, and sliding on the sand
            for (speed, off, up, slant) in [
                (40, 0, 3, 0),
                (80, 0, 1, 0),
                (40, 5, 2, 0),
                (60, -6, 0, 0),
                (45, 0, 5, 3),
                (45, 3, 0, -2),
            ] {
                let start = isl
                    .chest
                    .add(face.scale(hd + int(24)))
                    .add(across.scale(int(off)));
                let p = v3(start.x, ISLAND_TOP + int(up) + BODY_DROP, start.z);
                let v = face.neg().scale(int(speed)).add(across.scale(int(slant)));
                put(&mut r, p, v);
                let mut hit = false;
                for t in 0..240 {
                    r.step(&[Controls::default()]);
                    let a = &r.racers[0].ape;
                    let d = a.pos.sub(isl.chest);
                    let (f, s) = (d.dot(face), d.dot(across));
                    assert!(
                        !(f < hd && s.abs() < hw - int(1) && a.pos.y - BODY_DROP < rim - cm(60)),
                        "seed {seed} {speed} m/s at {off}: into the chest at tick {t}, {} cm behind its front",
                        (hd - f) * 100 / ONE
                    );
                    if f < hd + BODY_RADIUS + cm(5) && s.abs() < hw {
                        hit = true;
                    }
                }
                assert!(
                    hit,
                    "seed {seed} {speed} m/s at {off}: never reached the chest"
                );
            }
            // dropped on it: stands on the heap
            let top = isl.chest.add(v3(0, isl.chest_size.y + int(8), 0));
            put(&mut r, top, v3(0, -int(5), 0));
            for _ in 0..240 {
                r.step(&[Controls::default()]);
            }
            let a = &r.racers[0].ape;
            assert_eq!(a.mode, Mode::Grounded);
            assert_eq!(a.pos.y - BODY_DROP, rim + HEAP, "on the heap's top");
        }
    }
}
