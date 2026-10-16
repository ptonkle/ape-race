//! The rules Banana Bay Battle adds, each run only when `Course.bay` is
//! set: the water reset (or, with Floor is Lava, the kill). The barrel soft
//! wall is the plain side limit in `Course::contain` (the boundary barrels
//! stand on it, at `Course.half_width`), and the bouncy barrels are plain
//! `Course.mushrooms`, so neither needs a rule of its own here.
//!
//! "The last ship you were over" is worked out from the place, not
//! remembered: the ship whose reset spot (`Ship::reset_spot`) is furthest
//! along the course without being ahead of the place, else the dock. The
//! two battle lines overlap (BO), so this may be the enemy ship beside the
//! one the ape fell from; never one ahead. Nothing has to be kept in save
//! points, replays or online.

use super::*;
use crate::ape::Mode;
use crate::course::Course;
use crate::race::{Event, Race};
use crate::rules::BODY_DROP;

impl Course {
    /// Banana Bay Battle: whether a body at `p` (the hands) is in the water
    /// at `tick`: nothing holds it up (no deck, dock or beach under it) and
    /// its feet are down to the swell (or standing on the still water).
    pub fn in_bay_water(&self, b: &BayLayout, p: V3, tick: u32) -> bool {
        let feet = p.y - BODY_DROP;
        let still = self.floor(p.x, p.z);
        if feet > still + b.water.peak() + cm(10) {
            return false;
        }
        if self.support(p) > still {
            return false;
        }
        feet <= b.water.height(p.x, p.z, tick).max(still + cm(10))
    }

    /// Banana Bay Battle: where an ape that fell in the water at `p` is put
    /// back (its hands) and the way it faces, and the deck's platform: the
    /// main deck of the last ship whose stern is behind `p` along the
    /// course, `RESET_AFT` aft of its foremast (its low rope ahead), or the
    /// dock if no ship is.
    pub fn bay_reset(&self, b: &BayLayout, p: V3, hint: usize) -> (V3, i32, usize) {
        let (along, _) = self.progress(p, self.nearest(p, hint));
        // the ship whose reset spot is furthest along without being ahead
        // of the fall (the two battle lines overlap, so "the last ship
        // passed" is the one whose spot is last behind)
        let mut best: Option<(i32, usize)> = None;
        for (k, s) in b.ships.iter().enumerate() {
            let Some(spot) = s.reset_spot() else {
                continue;
            };
            let (st, _) = self.progress(spot, self.nearest_anywhere(spot));
            if st <= along && best.is_none_or(|(a, _)| st > a) {
                best = Some((st, k));
            }
        }
        match best.and_then(|(_, k)| b.ships.get(k)) {
            Some(s) => (s.reset_spot().unwrap_or(s.at), s.yaw, s.decks[0]),
            _ => {
                let at = self.point(int(4));
                (
                    v3(at.x, b.dock.from.y + BODY_DROP, at.z),
                    crate::geom::yaw_of(self.heading_at(0)),
                    b.dock.platform,
                )
            }
        }
    }
}

impl Race {
    /// Banana Bay Battle's water for local ape `i` after its step: in the
    /// water, it is put back on the last ship it passed (`Event::Reset`
    /// with that deck's platform), or with Floor is Lava eaten. Returns
    /// whether it is now out.
    pub(crate) fn bay_water(&mut self, i: usize) -> bool {
        let Some(b) = self.course.bay.as_deref() else {
            return false;
        };
        let r = &self.racers[i];
        if r.finish.is_some() || r.eaten.is_some() {
            return false;
        }
        if matches!(r.ape.mode, Mode::Swinging { .. }) {
            return false;
        }
        if !self.course.in_bay_water(b, r.ape.pos, self.tick) {
            return false;
        }
        if self.lava && self.running() {
            self.eat(i);
            return true;
        }
        let (at, yaw, pad) = self.course.bay_reset(b, r.ape.pos, r.ape.hint);
        self.racers[i].ape.put_on(at, yaw, &self.course);
        self.events.push(Event::Reset { who: i, pad });
        false
    }
}
