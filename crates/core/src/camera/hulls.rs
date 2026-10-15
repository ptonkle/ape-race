//! The eye on Banana Bay Battle keeps out of the ships (SECOND-ORDER BP1):
//! over the sea's highest swell, above every hull and deck it is over (and
//! high enough that no hull stands between it and the ape), and out of the
//! yards and masts (brought in toward the ape past one it would be in).
//! The other maps have none of this.

use crate::course::Course;
use crate::fx::cm;
use crate::geom::{right_of, V3};

/// A box standing on the flat: `centre` (its y unused) with `fwd` along
/// it, `half_len` along and `half_wide` across, from `bottom` to `top`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Block {
    centre: V3,
    fwd: V3,
    half_len: i32,
    half_wide: i32,
    bottom: i32,
    top: i32,
}

impl Block {
    /// Whether (x, z) is over the box, within `gap` of it.
    fn over(&self, p: V3, gap: i32) -> bool {
        let d = p.flat().sub(self.centre.flat());
        d.dot(self.fwd).abs() <= self.half_len + gap
            && d.dot(right_of(self.fwd)).abs() <= self.half_wide + gap
    }

    /// Whether `p` is in the box, or within `gap` of it.
    fn holds(&self, p: V3, gap: i32) -> bool {
        p.y >= self.bottom - gap && p.y <= self.top + gap && self.over(p, gap)
    }
}

/// What the eye keeps out of on the bay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hulls {
    /// hulls, decks, the pier and the beach: solid from under the water up
    /// to their tops (a ship is solid from the waterline up, BS7)
    floors: Vec<Block>,
    /// the yards with their furled sails, and the masts
    spars: Vec<Block>,
    /// the lowest the eye goes: over the highest the drawn sea rises
    sea: i32,
}

/// How far over a deck the eye stays (its rails stand about a metre up).
pub const DECK_GAP: i32 = cm(120);
/// How near the line from the ape to the eye may pass a hull.
const LINE_GAP: i32 = cm(30);
/// How near a yard or mast the eye may come (the view's near plane must
/// not cut into it, nor a spar fill the view: at 1.5 m a yard under the
/// eye still crossed the whole view).
pub const SPAR_GAP: i32 = cm(300);
/// The steps along the line, and the eye's rise, when keeping clear.
const STEP: i32 = cm(40);
const RISE_STEP: i32 = cm(50);
/// The most the eye is lifted to see past a hull.
const MOST_RISE: i32 = cm(2400);
/// The line is not checked this near the ape (its own deck under it).
const NEAR: i32 = cm(150);
/// The eye's height over the sea's highest swell (the squall draws the
/// swell twice as high, plus a chop: `bayscape::surface`).
pub const OVER_SEA: i32 = cm(70);

impl Hulls {
    /// The bay's ships, pier and beach (`None` off the bay).
    pub fn of(c: &Course) -> Option<Hulls> {
        let b = c.bay.as_ref()?;
        let level = b.water.level;
        let floors = c
            .platforms
            .iter()
            .map(|p| Block {
                centre: p.centre,
                fwd: p.fwd.flat().norm(),
                half_len: p.half_len,
                half_wide: p.half_width,
                bottom: level - cm(300),
                top: p.top,
            })
            .collect();
        let mut spars = Vec::new();
        for s in &b.ships {
            for m in &s.masts {
                spars.push(Block {
                    centre: s.point(m.along, 0, 0),
                    fwd: s.fwd(),
                    // the mast and the tops (platforms) on it
                    half_len: cm(120),
                    half_wide: cm(120),
                    bottom: s.at.y + s.deck,
                    top: s.at.y + s.deck + m.height,
                });
                for y in &m.yards {
                    let h = s.at.y + s.deck + y.height;
                    spars.push(Block {
                        centre: s.point(m.along, 0, 0),
                        fwd: s.fwd(),
                        half_len: cm(60),
                        half_wide: y.half_span + cm(20),
                        // the furled sail rolled under the spar
                        bottom: h - cm(140),
                        top: h + cm(40),
                    });
                }
            }
        }
        Some(Hulls {
            floors,
            spars,
            sea: level + 2 * b.water.peak() + OVER_SEA,
        })
    }

    /// The eye `eye` kept out of the ships, following the ape's `target`.
    pub fn kept_out(&self, target: V3, eye: V3) -> V3 {
        let mut eye = eye;
        eye.y = eye.y.max(self.sea);
        // over the deck (or hull) it is over
        let lift = |eye: &mut V3| {
            for f in &self.floors {
                if f.over(*eye, DECK_GAP) && eye.y < f.top + DECK_GAP && eye.y > f.bottom {
                    eye.y = f.top + DECK_GAP;
                }
            }
        };
        lift(&mut eye);
        // high enough that no hull stands between it and the ape (one the
        // ape is on or by is not in the way)
        let blocking: Vec<&Block> = self
            .floors
            .iter()
            .filter(|f| !f.holds(target, NEAR))
            .collect();
        let mut risen = 0;
        while risen < MOST_RISE && self.hidden(target, eye, &blocking) {
            eye.y += RISE_STEP;
            risen += RISE_STEP;
        }
        // out of a yard or mast: brought in toward the ape past it
        let line = eye.sub(target);
        let len = line.len();
        if len > STEP {
            let dir = line.norm();
            let spars: Vec<&Block> = self
                .spars
                .iter()
                .filter(|s| !s.holds(target, SPAR_GAP / 2))
                .collect();
            let mut d = len;
            while d > NEAR
                && spars
                    .iter()
                    .any(|s| s.holds(target.add(dir.scale(d)), SPAR_GAP))
            {
                d -= STEP;
            }
            if d < len {
                eye = target.add(dir.scale(d.max(NEAR)));
                eye.y = eye.y.max(self.sea);
            }
        }
        eye
    }

    /// Whether one of `floors` stands between the ape's `target` and `eye`.
    fn hidden(&self, target: V3, eye: V3, floors: &[&Block]) -> bool {
        let line = eye.sub(target);
        let len = line.len();
        if len <= NEAR {
            return false;
        }
        let dir = line.norm();
        let mut d = NEAR;
        while d <= len {
            let p = target.add(dir.scale(d));
            if floors.iter().any(|f| f.holds(p, LINE_GAP)) {
                return true;
            }
            d += STEP;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::Chase;
    use crate::fx::deg;
    use std::sync::Arc;

    #[test]
    fn on_the_bay_the_eye_keeps_out_of_the_sea_hulls_and_yards() {
        // BA7: swinging over a ship the eye sank under its deck (a brown
        // plane over half the view) and rode through a yard and its furled
        // sail. An AI race, a chase camera on each ape, two of them looking
        // up (the mouse pushed the view low): every tick the eye is over
        // the sea's highest swell, out of every hull and deck, and not in
        // a yard or mast that the ape is not itself by.
        use crate::ai::Brain;
        use crate::race::{Driver, Entry, Race};
        use crate::rules::Kind;
        let course = crate::bay::bay();
        let hulls = Arc::new(Hulls::of(&course).unwrap());
        let entries: Vec<Entry> = Kind::ALL[..4]
            .iter()
            .map(|&kind| Entry {
                name: String::new(),
                kind,
                driver: Driver::Ai,
                palette: 0,
            })
            .collect();
        let mut race = Race::new(course, &entries, false, 1);
        let mut brains: Vec<Brain> = (0..entries.len())
            .map(|i| Brain::new(race.ai_level, race.dice.0 ^ (0x9E37_79B9 * (i as u64 + 1))))
            .collect();
        let mut chases: Vec<Chase> = race
            .racers
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let mut c = Chase::new(x.ape.pos, race.course.start_yaw, race.course.terrain)
                    .with_hulls(x.ape.pos, Some(Arc::clone(&hulls)));
                if i >= 2 {
                    c.turn(0, deg(40));
                }
                c
            })
            .collect();
        let limit = crate::rules::TICKS_PER_SECOND as u32 * 70;
        let last = race.course.heading.len() - 1;
        while race.tick < limit {
            crate::replay::drive(&mut race, &mut brains, usize::MAX, Default::default());
            for (i, c) in chases.iter_mut().enumerate() {
                let a = &race.racers[i].ape;
                let ahead = race.course.heading[(a.hint + 8).min(last)];
                // the two looking up keep looking up (no easing back)
                c.follow(a.pos, a.vel, ahead, i < 2);
                let (e, t) = (c.eye, c.target);
                let why = format!("ape {i} tick {}: eye {e:?} ape {t:?}", race.tick);
                assert!(e.y >= hulls.sea, "under the sea's top: {why}");
                for f in &hulls.floors {
                    assert!(!f.holds(e, 0), "in a hull or deck: {why}");
                }
                for s in &hulls.spars {
                    if !s.holds(t, SPAR_GAP) {
                        assert!(!s.holds(e, SPAR_GAP / 2), "in a yard or mast: {why}");
                    }
                }
            }
        }
    }
}
