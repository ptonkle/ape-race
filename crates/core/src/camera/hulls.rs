//! The eye on Banana Bay Battle keeps out of the sea and the ships' hulls
//! and decks (SECOND-ORDER BP1), smoothly: it no longer dodges the masts,
//! yards and rigging, nor lifts to see past a hull (that made it jiggle as
//! it crossed them, BX1); what stands between it and the ape is drawn
//! see-through instead (`crate::see`). The other maps have none of this.

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
    #[cfg(test)]
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
    /// the lowest the eye goes: over the highest the drawn sea rises
    sea: i32,
}

/// How far over a deck the eye is headed (its rails stand about a metre
/// up), and how far out past its sides it starts rising, so it is up
/// before it is over the deck (BX1: the rise glides).
pub const DECK_GAP: i32 = cm(120);
const DECK_REACH: i32 = cm(400);
/// How far over a deck, and past its sides, the eye must be at once
/// (where the glide has not caught up): just clear, the near plane out of
/// it.
const HARD_GAP: i32 = cm(40);
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
        Some(Hulls {
            floors,
            sea: level + 2 * b.water.peak() + OVER_SEA,
        })
    }

    /// Where the eye `eye` is headed: over the sea's highest swell and
    /// `DECK_GAP` over any hull or deck it is over or within `DECK_REACH`
    /// of. The chase eases toward this. Nothing is dodged to keep the ape
    /// in view any more (masts, yards, a hull between): `crate::see` draws
    /// what is in the way see-through.
    pub fn kept_out(&self, eye: V3) -> V3 {
        self.over_floors(eye, DECK_REACH, DECK_GAP)
    }

    /// The eye kept only out of what it must never be in: under the sea's
    /// top, in a hull or deck. The chase eases toward `kept_out` and then
    /// takes this, so it only bites where the glide is behind.
    pub fn kept_hard(&self, eye: V3) -> V3 {
        self.over_floors(eye, HARD_GAP, HARD_GAP)
    }

    /// The eye over the sea's highest swell and `gap` over the deck (or
    /// hull) it is over, within `reach` of its sides.
    fn over_floors(&self, mut eye: V3, reach: i32, gap: i32) -> V3 {
        eye.y = eye.y.max(self.sea);
        for f in &self.floors {
            if f.over(eye, reach) && eye.y < f.top + gap && eye.y > f.bottom {
                eye.y = f.top + gap;
            }
        }
        eye
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::Chase;
    use crate::fx::deg;
    use std::sync::Arc;

    #[test]
    fn on_the_bay_the_eye_glides_over_the_sea_and_hulls_without_jiggling() {
        // BA7: swinging over a ship the eye sank under its deck. BX1: it
        // dodged the masts and yards and jiggled as it crossed them. An AI
        // race, a chase camera on each ape, two of them looking up (the
        // mouse pushed the view low): every tick the eye is over the sea's
        // highest swell and out of every hull and deck, and it glides:
        // ticks on which it jumps over 50 cm against the ape's own move
        // are rare (BX1: 857 of 33,593 dodging spars, 18 now).
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
                costume: crate::costume::Costume::Own,
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
        let (mut jumps, mut ticks) = (0, 0);
        while race.tick < limit {
            crate::replay::drive(&mut race, &mut brains, usize::MAX, Default::default());
            for (i, c) in chases.iter_mut().enumerate() {
                let a = &race.racers[i].ape;
                let (pe, pt) = (c.eye, c.target);
                let ahead = race.course.heading[(a.hint + 8).min(last)];
                // the two looking up keep looking up (no easing back)
                c.follow(a.pos, a.vel, ahead, i < 2);
                let (e, t) = (c.eye, c.target);
                let why = format!("ape {i} tick {}: eye {e:?} ape {t:?}", race.tick);
                if race.tick > 1 && t.sub(pt).len() < cm(300) {
                    ticks += 1;
                    if e.sub(pe).sub(t.sub(pt)).len() > cm(50) {
                        jumps += 1;
                    }
                }
                assert!(e.y >= hulls.sea, "under the sea's top: {why}");
                for f in &hulls.floors {
                    assert!(!f.holds(e, 0), "in a hull or deck: {why}");
                }
            }
        }
        assert!(
            jumps * 500 <= ticks,
            "the eye jumped on {jumps} of {ticks} ticks"
        );
    }
}
