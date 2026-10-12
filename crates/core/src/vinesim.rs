//! Vines that bend (TRIAGE row 12): each vine is drawn as a chain of
//! `SEGMENTS` links with its own simple physics (Verlet, with the links
//! held to their length a few times a tick). The top is pinned to the
//! anchor; a held vine's bottom-most held point follows the hands. So a
//! slack vine sags, a swung one lags and curves behind the hands, and a
//! let-go one whips back and settles. This is drawing only: the ape's own
//! physics stays the rope in `ape.rs`.

use crate::fx::{self, mul, ONE};
use crate::geom::{v3, V3};
use crate::rules::{DT, GRAVITY};

pub const SEGMENTS: usize = 8;
const POINTS: usize = SEGMENTS + 1;
/// Share of each point's motion kept from tick to tick.
const KEEP: i32 = ONE * 985 / 1000;
/// How many times a tick the links are pulled to length (fewer is softer).
const PASSES: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chain {
    pub pts: [V3; POINTS],
    prev: [V3; POINTS],
}

impl Chain {
    /// Hanging straight down from `anchor`.
    pub fn hanging(anchor: V3, length: i32) -> Chain {
        let mut pts = [anchor; POINTS];
        for (k, p) in pts.iter_mut().enumerate() {
            *p = anchor.sub(v3(0, length * k as i32 / SEGMENTS as i32, 0));
        }
        Chain { pts, prev: pts }
    }

    /// One tick. `hands` is where a holder grips, if any: the chain's link
    /// nearest the rope's length is pinned there, the rest hangs below.
    pub fn step(&mut self, anchor: V3, length: i32, hands: Option<V3>, tick: u32, seed: u32) {
        let fall = mul(mul(GRAVITY, DT), DT);
        // a light breeze, different for each vine
        let phase = (tick.wrapping_mul(70).wrapping_add(seed.wrapping_mul(9001))) as i32;
        let breeze = v3(
            mul(fx::sin(phase), ONE / 40_000),
            0,
            mul(fx::cos(phase / 2 + 3000), ONE / 50_000),
        );
        for k in 1..POINTS {
            let p = self.pts[k];
            let moved = p.sub(self.prev[k]).scale(KEEP);
            self.prev[k] = p;
            self.pts[k] = p.add(moved).sub(v3(0, fall, 0)).add(breeze);
        }
        // a held vine's grip point: the link the rope length reaches
        let grip = hands.map(|h| {
            let reach = h.sub(anchor).len();
            let k = ((reach as i64 * SEGMENTS as i64 + length as i64 / 2) / length.max(1) as i64)
                .clamp(1, SEGMENTS as i64) as usize;
            (k, h, reach)
        });
        // links stretch to fit a grip beyond the vine's end
        let link = match grip {
            Some((k, _, reach)) if reach > length * k as i32 / SEGMENTS as i32 => reach / k as i32,
            _ => length / SEGMENTS as i32,
        };
        for _ in 0..PASSES {
            self.pts[0] = anchor;
            if let Some((k, h, _)) = grip {
                self.pts[k] = h;
            }
            for k in 0..SEGMENTS {
                let a = self.pts[k];
                let b = self.pts[k + 1];
                let d = b.sub(a);
                let len = d.len();
                if len == 0 {
                    continue;
                }
                let fix = d.scale(fx::div(len - link, len));
                let pinned_a = k == 0 || grip.map(|g| g.0 == k).unwrap_or(false);
                let pinned_b = grip.map(|g| g.0 == k + 1).unwrap_or(false);
                match (pinned_a, pinned_b) {
                    (true, true) => {}
                    (true, false) => self.pts[k + 1] = b.sub(fix),
                    (false, true) => self.pts[k] = a.add(fix),
                    (false, false) => {
                        let half = fix.scale(ONE / 2);
                        self.pts[k] = a.add(half);
                        self.pts[k + 1] = b.sub(half);
                    }
                }
            }
        }
        self.pts[0] = anchor;
        if let Some((k, h, _)) = grip {
            self.pts[k] = h;
        }
    }

    /// Moved bodily (a sliding vine's anchor travels): shift every point.
    pub fn shift(&mut self, by: V3) {
        for k in 0..POINTS {
            self.pts[k] = self.pts[k].add(by);
            self.prev[k] = self.prev[k].add(by);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::int;

    #[test]
    fn a_free_vine_hangs_straight_and_stays_its_length() {
        let a = v3(0, int(50), 0);
        let mut c = Chain::hanging(a, int(40));
        for t in 0..600 {
            c.step(a, int(40), None, t, 1);
        }
        let bottom = c.pts[SEGMENTS];
        assert!((bottom.y - int(10)).abs() < int(1), "{}", bottom.y / ONE);
        // a breeze sways it a little
        assert!(
            bottom.flat().len() < int(2),
            "{}",
            bottom.flat().len() / 655
        );
    }

    #[test]
    fn a_held_vine_bends_behind_moving_hands() {
        let a = v3(0, int(50), 0);
        let mut c = Chain::hanging(a, int(40));
        // hands sweep along +z on a 30 m rope
        let mut bent = 0;
        for t in 0..120u32 {
            let ang = fx::deg(-40) + fx::deg(80) * t as i32 / 120;
            let h = a.add(v3(
                0,
                -mul(int(30), fx::cos(ang)),
                mul(int(30), fx::sin(ang)),
            ));
            c.step(a, int(40), Some(h), t, 1);
            // the middle of the held part is off the straight line
            let mid = c.pts[3];
            let line = a.lerp(h, ONE * 3 / 6);
            bent = bent.max(mid.dist(line));
            assert_eq!(c.pts[0], a);
            assert!(c.pts.contains(&h), "the grip point is at the hands");
        }
        assert!(bent > ONE / 4, "{}", bent / 655);
    }

    #[test]
    fn a_grip_beyond_the_end_stretches_the_links() {
        let a = v3(0, int(50), 0);
        let mut c = Chain::hanging(a, int(20));
        let h = a.add(v3(0, -int(24), 0));
        for t in 0..60 {
            c.step(a, int(20), Some(h), t, 1);
        }
        assert_eq!(c.pts[SEGMENTS], h);
    }
}
