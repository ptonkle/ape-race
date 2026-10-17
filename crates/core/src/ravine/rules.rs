//! The rules Rattlesnake Ravine adds, each run only when `Course.ravine` is
//! set: the snake vines' snap, the T-rex teeth's reset, the solids
//! (`solids.rs`) and the decks as floors.
//!
//! **A snapped snake vine comes back** (the rules agent's call,
//! SECOND-ORDER RR1): `SNAKE_REGROW` after its snap a fresh snake hangs
//! there again, whole and as deadly; until then the snapped halves hang
//! and nobody can grab it. So the leader's snap does not clear the way for
//! the apes behind it for good, and a snapped vine is never a gap in the
//! route for long.

use super::*;
use crate::ape::{Ape, Happening, Mode};
use crate::fx::int;
use crate::race::{Event, Phase, Race};
use crate::rules::{secs_to_ticks, TICKS_PER_SECOND};

/// A snapped snake vine hangs whole again this many ticks after its snap.
pub const SNAKE_REGROW: u32 = 10 * TICKS_PER_SECOND as u32;

/// An ape dropped by a snake vine cannot grab again for this long (it
/// falls to the ground rather than catching the next vine at once): 1.5 s
/// is a fall of 19 m, deeper than any vine's foot over the floor.
pub const SNAKE_NO_GRAB: i32 = secs_to_ticks(150);

/// The step along a move when looking for a brushed tooth.
const TOOTH_STEP: i32 = cm(25);

impl Hanger {
    /// Whether it is a snake vine snapped at `tick` (its halves hanging,
    /// nobody can grab it).
    pub fn snapped(&self, tick: u32) -> bool {
        self.snake
            && self
                .snapped_at
                .is_some_and(|at| tick >= at && tick < at.saturating_add(SNAKE_REGROW))
    }

    /// Whether a grab of it at `tick` snaps it: a whole snake vine.
    pub fn snaps(&self, tick: u32) -> bool {
        self.snake && !self.snapped(tick)
    }
}

impl RavineLayout {
    /// Whether vine `i` cannot be grabbed at `tick` (a snapped snake vine).
    pub fn vine_gone(&self, i: usize, tick: u32) -> bool {
        self.hangers.get(i).is_some_and(|h| h.snapped(tick))
    }

    /// Whether grabbing vine `i` at `tick` snaps it.
    pub fn vine_snaps(&self, i: usize, tick: u32) -> bool {
        self.hangers.get(i).is_some_and(|h| h.snaps(tick))
    }
}

impl Course {
    /// Rattlesnake Ravine's floors over `p` other than the ground: a
    /// bridge's or a log's deck, or a solid's top (a rock, the sphinx).
    pub(crate) fn ravine_top(&self, r: &RavineLayout, p: V3) -> Option<i32> {
        let feet = p.y - BODY_DROP;
        let deck = r
            .decks
            .iter()
            .filter_map(|d| d.top_at(p))
            .filter(|&t| feet >= t - cm(60))
            .max();
        let solid = self.ravine_solids().and_then(|s| s.top_under(p));
        deck.max(solid)
    }
}

impl Ape {
    /// Rattlesnake Ravine on a grab of vine `vine`: a whole snake vine
    /// snaps in half (the race marks it, `Race::rattlesnake`) and the ape
    /// drops, its momentum gone, falling to the ground. True if it snapped.
    pub(crate) fn snake_snap(&mut self, vine: usize, course: &Course, tick: u32) -> bool {
        let Some(r) = course.ravine.as_deref() else {
            return false;
        };
        if !r.vine_snaps(vine, tick) {
            return false;
        }
        self.vel = V3::ZERO;
        self.mode = Mode::Flying;
        self.last_vine = Some((vine, 0));
        self.release_bonus = 0;
        self.no_grab = SNAKE_NO_GRAB;
        self.happened.push(Happening::Snapped);
        true
    }

    /// Rattlesnake Ravine after a move from `before`: the solids stop it
    /// (a hit on a vine lets go, as stone does), and a brushed tooth is
    /// noted (`Happening::Bitten`; the race puts the ape back).
    pub(crate) fn hit_ravine(&mut self, course: &Course, before: V3) {
        let Some(r) = course.ravine.as_deref() else {
            return;
        };
        if let Some(s) = course.ravine_solids() {
            if let Some((p, v, wall)) = s.push(before, self.pos, self.vel) {
                (self.pos, self.vel) = (p, v);
                if wall {
                    self.happened.push(Happening::Wall);
                    if let Mode::Swinging { vine, .. } = self.mode {
                        self.mode = Mode::Flying;
                        self.last_vine = Some((vine, 0));
                        self.release_bonus = 0;
                    }
                }
            }
        }
        if let Some(sk) = &r.skull {
            if brushed(sk, before, self.pos) {
                self.happened.push(Happening::Bitten);
            }
        }
    }
}

/// Whether a body moving from `p0` to `p1` brushes a tooth anywhere on the
/// way (stepped, so a fast ape cannot pass through one).
fn brushed(sk: &Skull, p0: V3, p1: V3) -> bool {
    // (cheap: nowhere near the mouth)
    let reach = MOUTH_GAP * 2 + int(4);
    if p0.dist(sk.at) > reach + p1.dist(p0) && p1.dist(sk.at) > reach {
        return false;
    }
    let len = p1.dist(p0);
    let steps = (len / TOOTH_STEP).clamp(1, 400);
    (1..=steps).any(|k| sk.brushes(p0.lerp(p1, (k as i64 * ONE as i64 / steps as i64) as i32)))
}

impl Race {
    /// Rattlesnake Ravine after ape `i`'s step: a snake vine it snapped is
    /// marked snapped (shared: every ape sees it gone), and a tooth it
    /// brushed puts it on the canyon floor under the jaw with no speed
    /// (with Floor is Lava it is eaten). True if it was eaten.
    pub(crate) fn rattlesnake(&mut self, i: usize) -> bool {
        let tick = self.tick;
        let Some(r) = self.racers.get(i) else {
            return false;
        };
        let happened = &r.ape.happened;
        let snapped = happened.contains(&Happening::Snapped);
        let bitten = happened.contains(&Happening::Bitten);
        let vine = r.ape.last_vine.map(|(v, _)| v);
        if !snapped && !bitten {
            return false;
        }
        let Some(lay) = self.course.ravine.as_deref_mut() else {
            return false;
        };
        if snapped {
            if let Some(h) = vine.and_then(|v| lay.hangers.get_mut(v)) {
                h.snapped_at = Some(tick);
            }
        }
        let Some(sk) = lay.skull.as_ref() else {
            return false;
        };
        let (at, yaw) = (sk.floor_spot, sk.floor_yaw);
        let r = &self.racers[i];
        if !bitten || r.finish.is_some() || r.eaten.is_some() {
            return false;
        }
        if self.lava && self.phase == Phase::Running {
            self.eat(i);
            return true;
        }
        self.racers[i].ape.put_on(at, yaw, &self.course);
        self.events.push(Event::Reset { who: i, pad: 0 });
        false
    }

    /// Rattlesnake Ravine: a remote ape reported on a whole snake vine
    /// snapped it (from its state, so every browser snaps it on the same
    /// tick as near as can be); it is drawn falling.
    pub(crate) fn remote_snap(&mut self, i: usize) {
        let tick = self.tick;
        let Some(Mode::Swinging { vine, .. }) = self.racers.get(i).map(|r| r.ape.mode) else {
            return;
        };
        let Some(lay) = self.course.ravine.as_deref_mut() else {
            return;
        };
        if lay.vine_snaps(vine, tick) {
            lay.hangers[vine].snapped_at = Some(tick);
            let a = &mut self.racers[i].ape;
            a.mode = Mode::Flying;
            a.last_vine = Some((vine, 0));
        }
    }
}
