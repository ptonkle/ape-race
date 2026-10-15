//! Sector Ape's race rules (the course agent's): the boundary and the reset
//! onto a deck, the switch door, and the pull past the finish. `Race::step`
//! calls them only on this map.

use super::*;
use crate::ape::Mode;
use crate::course::Course;
use crate::fx::mul;
use crate::race::{Event, Race};
use crate::rules::{BODY_DROP, DT};

/// Past the finish a flying ape is braked this hard (a share of its speed
/// a second) and drawn toward the black hole this hard, so it comes down
/// on the finish deck rather than drifting off.
pub const FINISH_BRAKE: i32 = ONE * 3 / 2;
pub const FINISH_PULL: i32 = int(2);
/// An ape drawn in that reaches the black hole floats at its horizon,
/// this many radii from its middle.
pub const HORIZON: i32 = ONE * 6 / 5;
/// A finished ape whose feet are this far under the finish deck's top has
/// fallen past it: it is carried up onto the deck, this far in from its
/// edge at the least.
pub const FINISH_UNDER: i32 = int(2);
pub const FINISH_INSET: i32 = int(4);

impl Course {
    /// Whether a body at `p` (the hands; `hint` its nearest sample) is out
    /// past Sector Ape's boundary. Never on another map.
    pub fn out_of_bounds(&self, p: V3, hint: usize) -> bool {
        let Some(s) = self.sector.as_deref() else {
            return false;
        };
        let b = s.boundary;
        let (_, side) = self.progress(p, hint);
        let behind = p.flat().sub(self.path[0]).dot(self.heading[0]);
        side.abs() > b.side || p.y < b.below || p.y > b.above || behind < -b.side
    }

    /// The deck an ape at `p` is put back on: the closest piece of ground
    /// behind it (the person, T80: "only put it on a deck behind you"),
    /// the pad nearest it along the course whose middle is not ahead of it.
    /// An ape behind every pad (only off the back of the start) goes back
    /// on the first.
    pub fn pad_for(&self, p: V3, hint: usize) -> Option<usize> {
        let s = self.sector.as_deref()?;
        let (along, _) = self.progress(p, hint);
        // (behind it, how far behind) beats (ahead, how far ahead)
        let mut best: Option<((bool, i32), usize)> = None;
        for &k in &s.pads {
            let pl = &self.platforms[k];
            let (at, _) = self.progress(pl.centre, self.nearest_anywhere(pl.centre));
            let key = (at > along, (at - along).abs());
            if best.is_none_or(|(b, _)| key < b) {
                best = Some((key, k));
            }
        }
        best.map(|(_, k)| k)
    }

    /// Where an ape put back on pad `k` stands (its hands), and the way it
    /// faces: the middle of the deck, along the course.
    pub fn pad_spot(&self, k: usize) -> (V3, i32) {
        let pl = &self.platforms[k];
        let hint = self.nearest_anywhere(pl.centre);
        let yaw = crate::geom::yaw_of(self.heading[hint]);
        (v3(pl.centre.x, pl.top + BODY_DROP, pl.centre.z), yaw)
    }
}

impl Race {
    /// Sector Ape's rules for local ape `i` after its step: the boundary
    /// (a reset, or the kill in Floor is Lava), the switch, and the pull
    /// past the finish. Returns whether the ape is now out (eaten).
    pub(crate) fn space_rules(&mut self, i: usize) -> bool {
        if self.course.sector.is_none() {
            return false;
        }
        if self.racers[i].finish.is_some() {
            self.finish_pull(i);
            return false;
        }
        if self.racers[i].eaten.is_some() {
            return false;
        }
        let (pos, hint) = (self.racers[i].ape.pos, self.racers[i].ape.hint);
        if self.course.out_of_bounds(pos, hint) {
            if self.lava && self.running() {
                self.eat(i);
                return true;
            }
            if let Some(pad) = self.course.pad_for(pos, hint) {
                let (at, yaw) = self.course.pad_spot(pad);
                self.racers[i].ape.put_on(at, yaw, &self.course);
                self.events.push(Event::Reset { who: i, pad });
            }
            return false;
        }
        self.switch(i);
        false
    }

    /// The switch vine: ape `i` swinging it more than `DOOR_ANGLE` forward
    /// of the way it hangs shuts the door now, unless it is shut already
    /// (it cannot be shut again until it has reopened). Runs for remote
    /// apes too, from their reported state.
    pub(crate) fn switch(&mut self, i: usize) {
        let tick = self.tick;
        let Some(s) = self.course.sector.as_deref() else {
            return;
        };
        let door = s.door;
        let Mode::Swinging { vine, .. } = self.racers[i].ape.mode else {
            return;
        };
        if vine != door.switch || door.shut(tick) || vine >= self.course.vines.len() {
            return;
        }
        if swung_forward(&self.course, vine, self.racers[i].ape.pos, tick) < DOOR_ANGLE {
            return;
        }
        if let Some(s) = self.course.sector.as_deref_mut() {
            s.door.shut_at = Some(tick);
        }
        self.events.push(Event::Door { who: i });
    }

    /// Past the finish: braked and drawn toward the black hole, landing on
    /// the finish deck to dance, or floating at the horizon.
    fn finish_pull(&mut self, i: usize) {
        let Some(s) = self.course.sector.as_deref() else {
            return;
        };
        let (hole, r) = s.black_hole;
        let a = &mut self.racers[i].ape;
        if a.mode != Mode::Flying {
            return;
        }
        let to = hole.sub(a.pos);
        a.vel = a.vel.sub(a.vel.scale(mul(FINISH_BRAKE, DT)));
        a.vel = a.vel.add(to.norm().scale(mul(FINISH_PULL, DT)));
        let rim = mul(r, HORIZON);
        if to.len() < rim {
            let n = to.norm().neg();
            a.pos = hole.add(n.scale(rim));
            a.vel = a.vel.reject(n);
        }
        // one that crossed the line low falls past the deck (gravity beats
        // the pull) and would dance in empty space for ever: the hole's
        // pull carries it up onto the deck, at the nearest place on it
        // (SECOND-ORDER SF7)
        let deck = &self.course.platforms[s.finish_deck];
        if a.feet().y < deck.top - FINISH_UNDER {
            let d = a.pos.sub(deck.centre);
            let right = crate::geom::right_of(deck.fwd);
            let inset = |half: i32| (half - FINISH_INSET).max(0);
            let along = d
                .dot(deck.fwd)
                .clamp(-inset(deck.half_len), inset(deck.half_len));
            let side = d
                .dot(right)
                .clamp(-inset(deck.half_width), inset(deck.half_width));
            let spot = deck
                .centre
                .add(deck.fwd.scale(along))
                .add(right.scale(side));
            let at = v3(spot.x, deck.top + BODY_DROP, spot.z);
            let yaw = a.yaw;
            a.put_on(at, yaw, &self.course);
        }
    }
}

/// How far vine `vine`'s rope (to `hands`) is swung forward of the way it
/// hangs, an angle (negative: back).
pub fn swung_forward(course: &Course, vine: usize, hands: V3, tick: u32) -> i32 {
    let a = course.anchor(vine, tick);
    let d = hands.sub(a).norm();
    let h = course.hang(vine);
    let f = course.heading_at(course.vines[vine].at).reject(h).norm();
    fx::atan2(d.dot(f), d.dot(h))
}
