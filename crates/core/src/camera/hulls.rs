//! The eye on Banana Bay Battle keeps out of the ships (SECOND-ORDER BP1):
//! over the sea's highest swell, above every hull and deck it is over (and
//! high enough that no hull stands between it and the ape), and out of the
//! yards and masts (brought in toward the ape past one it would be in).
//! The other maps have none of this.

use crate::course::Course;
use crate::fx::{cm, mul, ONE};
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

    /// Whether (x, z) is within `along` of the box along it and `across`
    /// of it across.
    fn beside(&self, p: V3, along: i32, across: i32) -> bool {
        let d = p.flat().sub(self.centre.flat());
        d.dot(self.fwd).abs() <= self.half_len + along
            && d.dot(right_of(self.fwd)).abs() <= self.half_wide + across
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
    /// the yards alone (the eye keeps above or below their band, BR1)
    yards: Vec<Block>,
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
/// How near a yard or mast the eye may come before it is brought in at
/// once (nearer than `SPAR_GAP`, where the eased shift keeps it).
const HARD_GAP: i32 = cm(190);
/// The steps along the line, and the eye's rise, when keeping clear.
const STEP: i32 = cm(40);
const RISE_STEP: i32 = cm(50);
/// The most the eye is lifted to see past a hull (24 m left it high over
/// the ships looking steeply down, BP7.2).
const MOST_RISE: i32 = cm(1200);
/// The steepest the eye looks down at the ape (rise over the flat
/// distance, Q16: about 38 degrees) when lifted to see past a hull; past
/// that it comes in toward the ape instead.
pub const STEEPEST: i32 = ONE * 4 / 5;
/// How far, as fractions of its distance (Q16), the eye comes in toward
/// the ape to see past a hull without going steeper than `STEEPEST`.
const PULLS: [i32; 4] = [ONE, ONE * 4 / 5, ONE * 13 / 20, ONE / 2];
/// A yard this near the eye along the ship and across past its ends
/// (and less than `YARD_BAND` above or below it) crosses the view close
/// up (BP7.1): the eye slides up or down out of the band.
pub const YARD_ALONG: i32 = cm(400);
pub const YARD_ACROSS: i32 = cm(300);
pub const YARD_BAND: i32 = cm(300);
/// The most the eye slides up or down out of a yard's band.
const MOST_SLIDE: i32 = cm(1000);
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
        let mut yards = Vec::new();
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
                    let yard = Block {
                        centre: s.point(m.along, 0, 0),
                        fwd: s.fwd(),
                        half_len: cm(60),
                        half_wide: y.half_span + cm(20),
                        // the furled sail rolled under the spar
                        bottom: h - cm(140),
                        top: h + cm(40),
                    };
                    spars.push(yard);
                    yards.push(yard);
                }
            }
        }
        Some(Hulls {
            floors,
            spars,
            yards,
            sea: level + 2 * b.water.peak() + OVER_SEA,
        })
    }

    /// The eye `eye` kept out of the ships, following the ape's `target`.
    pub fn kept_out(&self, target: V3, eye: V3) -> V3 {
        let mut eye = self.over_floors(eye);
        // high enough that no hull stands between it and the ape (one the
        // ape is on or by is not in the way), but no steeper than
        // `STEEPEST`: past that it comes in toward the ape (BR1)
        let blocking: Vec<&Block> = self
            .floors
            .iter()
            .filter(|f| !f.holds(target, NEAR))
            .collect();
        if self.hidden(target, eye, &blocking) {
            let mut found = None;
            for &pull in &PULLS {
                let mut e = self.over_floors(target.add(eye.sub(target).scale(pull)));
                let flat = e.sub(target).len_xz();
                let most = (target.y + mul(flat, STEEPEST)).min(e.y + MOST_RISE);
                while e.y <= most && self.hidden(target, e, &blocking) {
                    e.y += RISE_STEP;
                }
                if e.y <= most {
                    found = Some(e);
                    break;
                }
            }
            eye = match found {
                Some(e) => e,
                // nowhere gentle sees past: lifted as before, at most
                // `MOST_RISE`
                None => {
                    let mut e = eye;
                    let mut risen = 0;
                    while risen < MOST_RISE && self.hidden(target, e, &blocking) {
                        e.y += RISE_STEP;
                        risen += RISE_STEP;
                    }
                    e
                }
            };
        }
        let mut eye = self.out_of_spars(target, eye, SPAR_GAP);
        // out of a yard's band where one would cross the view close up,
        // and from behind a yard that hides the ape (BR1): slid down or
        // up, the nearer first, down first; where no height clears both,
        // out of the band at least
        let banded = self.in_yard_band(target, eye);
        if banded || self.yard_between(target, eye) {
            let passes: &[bool] = if banded { &[true, false] } else { &[true] };
            'passes: for &line in passes {
                let mut dy = RISE_STEP;
                while dy <= MOST_SLIDE {
                    let down = self.over_floors(eye.add(V3::UP.scale(-dy)));
                    let up = self.over_floors(eye.add(V3::UP.scale(dy)));
                    if down.y < eye.y && self.open(target, down, &blocking, line) {
                        eye = down;
                        break 'passes;
                    }
                    if self.open(target, up, &blocking, line) {
                        eye = up;
                        break 'passes;
                    }
                    dy += RISE_STEP;
                }
            }
        }
        eye
    }

    /// The eye `eye` over the sea's highest swell and over the deck (or
    /// hull) it is over.
    fn over_floors(&self, mut eye: V3) -> V3 {
        eye.y = eye.y.max(self.sea);
        for f in &self.floors {
            if f.over(eye, DECK_GAP) && eye.y < f.top + DECK_GAP && eye.y > f.bottom {
                eye.y = f.top + DECK_GAP;
            }
        }
        eye
    }

    /// Whether `eye` is out of every yard's band, out of the spars' gap and
    /// sees the ape past `floors` (and, with `line`, past every yard).
    fn open(&self, target: V3, eye: V3, floors: &[&Block], line: bool) -> bool {
        !self.in_yard_band(target, eye)
            && !(line && self.yard_between(target, eye))
            && !self
                .spars
                .iter()
                .any(|s| !s.holds(target, SPAR_GAP / 2) && s.holds(eye, SPAR_GAP))
            && !self.hidden(target, eye, floors)
    }

    /// Whether a yard (one the ape is not at) stands between the ape's
    /// `target` and `eye`, hiding the ape behind it.
    pub fn yard_between(&self, target: V3, eye: V3) -> bool {
        let line = eye.sub(target);
        let len = line.len();
        if len <= NEAR {
            return false;
        }
        let dir = line.norm();
        // the yard the ape hangs from too: from the eye above it, it
        // crossed the view right in front of the ape (`swing-720`)
        let mut d = NEAR;
        while d <= len {
            let p = target.add(dir.scale(d));
            if self.yards.iter().any(|y| y.holds(p, LINE_GAP)) {
                return true;
            }
            d += STEP;
        }
        false
    }

    /// Whether a yard near `eye` crosses the view close up: within
    /// `YARD_ALONG` along its ship, `YARD_ACROSS` past its ends and
    /// `YARD_BAND` over or under it (one the ape is at is left be).
    pub fn in_yard_band(&self, target: V3, eye: V3) -> bool {
        self.yards.iter().any(|y| {
            !y.holds(target, SPAR_GAP / 2)
                && y.beside(eye, YARD_ALONG, YARD_ACROSS)
                && eye.y >= y.bottom - YARD_BAND
                && eye.y <= y.top + YARD_BAND
        })
    }

    /// The eye kept only out of what it must never be in: under the sea's
    /// top, in a hull or deck, in a yard or mast (brought in toward the
    /// ape past it). The chase eases toward `kept_out` and then takes
    /// this, so a hull or spar that comes suddenly never cuts the view.
    pub fn kept_hard(&self, target: V3, eye: V3) -> V3 {
        let eye = self.over_floors(eye);
        self.out_of_spars(target, eye, HARD_GAP)
    }

    /// `eye` brought in toward the ape past a yard or mast it is in.
    fn out_of_spars(&self, target: V3, mut eye: V3, gap: i32) -> V3 {
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
            while d > NEAR && spars.iter().any(|s| s.holds(target.add(dir.scale(d)), gap)) {
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
        // BP7.1 (BR1): where the eye is headed (the ships' wish for it,
        // `kept_out`) never has a yard crossing the view close up; and it
        // glides there: ticks on which it jumps over 1.5 m (against the
        // ape's own move) stay rare (368 of 33,600 unhurried, 147 eased)
        let mut jumps = 0;
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
                if race.tick > 1
                    && t.sub(pt).len() < cm(100)
                    && e.sub(pe).sub(t.sub(pt)).len() > cm(150)
                {
                    jumps += 1;
                }
                assert!(
                    !hulls.in_yard_band(t, hulls.kept_out(t, c.free)),
                    "headed for a yard's band: {why}"
                );
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
        assert!(jumps <= 200, "the eye jumped on {jumps} ticks");
    }
}
