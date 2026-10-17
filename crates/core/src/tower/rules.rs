//! Mape's Tower's rules, each run only when `Course.tower` is set: the 3D
//! "along the course" (TW1), the boundary and the reset onto a pad with
//! its boost (per leg: the tube's and the sling's resets go to the
//! portal), the climb vines' lift, and the tube's air, its entry (T99)
//! and its trampolines (T100).

use super::*;
use crate::ape::{Ape, Controls, Mode};
use crate::course::{Course, STEP};
use crate::fx::{mul, ONE};
use crate::geom::yaw_of;
use crate::race::{Event, Race};
use crate::rules::{BODY_DROP, DT};

/// The fall's terminal speed in the tube: the void's air is thicker than
/// open air (where a fall from the tear reached about 50 m/s). Measured
/// with `TUBE_AIR` so each pad's gap is reachable from the last by a
/// human-like weave (TW2-5, `MEASUREMENTS.md`).
pub const TUBE_TERMINAL: i32 = int(38);
/// How fast a fall faster than `TUBE_TERMINAL` settles back to it: the
/// share of the excess taken each second.
pub const TUBE_SETTLE: i32 = ONE * 8;
/// Air steering in the tube is this many times the kind's (the void
/// pushes back).
pub const TUBE_AIR: i32 = ONE * 3;
/// The steering aims this many seconds of its sideways speed short of
/// where it wants to be.
const TUBE_LEAD: i32 = ONE * 3 / 10;
/// A trampoline's bounce (T100): the speed into its face becomes this out
/// of it, and `DISC_KEEP` of the speed along it is kept. From terminal
/// speed a hit loses about (38 + 13)² / (2 × 17 × 38) ≈ 2 s against
/// falling on.
pub const DISC_BOUNCE: i32 = int(13);
const DISC_KEEP: i32 = ONE / 2;
/// The steering looks this far down for a trampoline to dodge, and
/// passes one this far clear of its rim.
const DODGE_LOOK: i32 = int(30);
const DODGE_CLEAR: i32 = int(2);

impl Course {
    /// Mape's Tower's `nearest`: the nearest centreline sample to `p` in
    /// 3D, searched 40 samples either side of `hint` (of two as near, the
    /// first).
    pub(crate) fn nearest_3d(&self, p: V3, hint: usize) -> usize {
        let n = self.path.len();
        let lo = hint.saturating_sub(40);
        let hi = (hint + 40).min(n - 1);
        let mut best = hint.min(n - 1);
        let mut best_d = i64::MAX;
        for i in lo..=hi {
            let d = p.sub(self.path[i]);
            let dd = d.dot64(d);
            if dd < best_d {
                best_d = dd;
                best = i;
            }
        }
        best
    }

    /// Mape's Tower's `nearest_anywhere`: the whole centreline, in 3D.
    pub(crate) fn nearest_anywhere_3d(&self, p: V3) -> usize {
        let mut best = 0;
        let mut best_d = i64::MAX;
        for (i, q) in self.path.iter().enumerate() {
            let d = p.sub(*q);
            let dd = d.dot64(d);
            if dd < best_d {
                best_d = dd;
                best = i;
            }
        }
        best
    }

    /// Mape's Tower's `progress`: along the 3D tangent at sample `i`, and
    /// across (level, right of it).
    pub(crate) fn progress_3d(&self, p: V3, i: usize) -> (i32, i32) {
        let d = p.sub(self.path[i]);
        let along = d.dot(self.heading[i]);
        let side = d.dot(self.right[i]);
        ((i as i32 * STEP + along).clamp(0, self.length), side)
    }

    /// Whether a body at `p` (the hands; `hint` its nearest sample) is out
    /// of Mape's Tower's bounds: too far from the centreline, too far under
    /// it, or standing on the open ground. Never on another map, and never
    /// in the tube (its wall holds; its floor is the tunnel's).
    pub fn tower_out(&self, p: V3, hint: usize) -> bool {
        let Some(t) = self.tower.as_deref() else {
            return false;
        };
        if t.tube.holds(p) {
            return false;
        }
        let feet = p.y - BODY_DROP;
        let ground = self.terrain(p.x, p.z);
        let on_ground = feet <= ground + cm(10) && self.support(p) <= ground;
        // (inside tower 1's hollow the line up the axis is only what
        // progress measures, and a missed swing falls far past it: only
        // the floor is out there, TW2-6)
        if t.inside_tower_1(p) {
            return on_ground;
        }
        let q = self.path[hint.min(self.path.len() - 1)];
        p.sub(q).longer_than(OUT_FAR) || p.y < q.y - OUT_BELOW || on_ground
    }

    /// The reset pad for an ape out at `p`: by its leg
    /// (`TowerLayout::reset_pad`). Inside tower 1 progress is height, so
    /// a fall to the floor (or off the portal's beams, down the outside
    /// of the wall) has taken its progress down with it: off a beam of
    /// the climb inside or the portal's, the pad is the one behind that
    /// beam (TW2-9).
    pub fn tower_pad(&self, t: &TowerLayout, a: &Ape) -> Pad {
        let (mut along, _) = self.progress(a.pos, a.hint);
        if let Some((v, _)) = a.last_vine {
            let at = self.vines[v].at;
            if matches!(t.leg_at(at), Leg::Inside | Leg::Portal) {
                along = along.max(at);
            }
        }
        t.reset_pad(along)
    }
}

impl TowerLayout {
    /// The AI's steering down the tube: with a trampoline under it
    /// (within `DODGE_LOOK`), steer out past its rim on the side the ape
    /// is already on (toward the axis if that side is the wall's); with
    /// none under it, under the lowest trampoline steer for the axis (the
    /// sling row), else no steering. None outside the tube.
    pub fn tube_controls(&self, a: &Ape) -> Option<Controls> {
        let z = &self.tube;
        if !z.holds(a.pos) {
            return None;
        }
        let flat = |p: V3| v3(p.x, 0, p.z);
        let lowest = z.discs.iter().map(|d| d.centre.y).min().unwrap_or(z.top);
        let aim = if let Some(k) = z.disc_ahead(a.pos, DODGE_LOOK, DODGE_CLEAR) {
            let c = flat(z.discs[k].centre);
            let axis = flat(z.axis);
            let mut out = flat(a.pos).sub(c);
            if !out.longer_than(cm(50)) {
                out = axis.sub(c);
            }
            let mut aim = c.add(out.with_len(DISC_RADIUS + DODGE_CLEAR * 2));
            if aim.sub(axis).longer_than(z.radius - int(2)) {
                // (the wall's side: go round the other way)
                aim = c.sub(out.with_len(DISC_RADIUS + DODGE_CLEAR * 2));
            }
            aim
        } else if a.pos.y - BODY_DROP < lowest {
            flat(z.axis)
        } else {
            return Some(Controls::default());
        };
        // (aim short by the sideways speed's lead, as a player does, so
        // the ape settles instead of swinging past)
        let to = v3(aim.x - a.pos.x, 0, aim.z - a.pos.z).sub(a.vel.flat().scale(TUBE_LEAD));
        let mut c = Controls::default();
        if to.longer_than(cm(60)) {
            c.fwd = 1;
            c.yaw = yaw_of(to);
        }
        Some(c)
    }
}

impl Ape {
    /// Mape's Tower: a climb vine let go of while rising adds `CLIMB_LIFT`
    /// upward.
    pub(crate) fn climb_lift(&mut self, vine: usize, course: &Course) {
        if let Some(t) = course.tower.as_deref() {
            if t.lifts(vine) && self.vel.y > 0 {
                self.vel.y += CLIMB_LIFT;
            }
        }
    }

    /// Whether the ape is in Mape's Tower's tube.
    pub(crate) fn in_tube(&self, course: &Course) -> bool {
        course
            .tower
            .as_deref()
            .is_some_and(|t| t.tube.holds(self.pos))
    }

    /// Mape's Tower's tube: the steering's strength there (`TUBE_AIR`
    /// times the kind's), else the kind's.
    pub(crate) fn air_now(&self, course: &Course) -> i32 {
        match course.tower.as_deref() {
            Some(t) if t.tube.holds(self.pos) => mul(self.phys.air, TUBE_AIR),
            _ => self.phys.air,
        }
    }

    /// Mape's Tower's tube: the thick air settles a fall faster than
    /// `TUBE_TERMINAL` back toward it (after gravity, each flying tick).
    pub(crate) fn tube_air(&mut self, course: &Course) {
        let Some(t) = course.tower.as_deref() else {
            return;
        };
        if !t.tube.holds(self.pos) {
            return;
        }
        let excess = -self.vel.y - TUBE_TERMINAL;
        if excess > 0 {
            self.vel.y += mul(excess, mul(TUBE_SETTLE, DT)).min(excess);
        }
    }

    /// Mape's Tower's tube (T99): an ape flying into the tube's column
    /// past the portal (from outside its radius, or from over its top,
    /// to within it under its top) loses its speed across, so the fall
    /// starts straight down whatever the entry. Whether it did.
    pub(crate) fn tube_entry(&mut self, course: &Course, before: V3) -> bool {
        let Some(t) = course.tower.as_deref() else {
            return false;
        };
        let z = &t.tube;
        let inside =
            |p: V3| t.in_void(p) && z.over(p) && p.y <= z.top && p.y > z.bottom + t.tunnel.height;
        if self.swinging().is_some() || inside(before) || !inside(self.pos) {
            return false;
        }
        self.vel = v3(0, self.vel.y, 0);
        true
    }

    /// Mape's Tower's tube (T100): feet crossing a trampoline's face from
    /// above bounce off it: the speed into the face becomes `DISC_BOUNCE`
    /// out of it and `DISC_KEEP` of the speed along it is kept, so a
    /// leaning disc throws the ape sideways too. Whether it bounced.
    pub(crate) fn tube_discs(&mut self, course: &Course, before: V3) -> bool {
        let Some(t) = course.tower.as_deref() else {
            return false;
        };
        if self.swinging().is_some() {
            return false;
        }
        let Some(k) = t.tube.disc_hit(before, self.pos) else {
            return false;
        };
        let d = t.tube.discs[k];
        // (the feet back on the face)
        let feet = self.pos.sub(v3(0, BODY_DROP, 0));
        let under = feet.sub(d.centre).dot(d.normal);
        if under < 0 {
            self.pos = self.pos.add(d.normal.scale(-under));
        }
        let along = self.vel.reject(d.normal);
        self.vel = d.normal.scale(DISC_BOUNCE).add(along.scale(DISC_KEEP));
        self.mode = Mode::Flying;
        true
    }
}

impl Race {
    /// Mape's Tower's boundary for local ape `i` after its step: out of
    /// bounds, it is put on the pad its leg resets to and leaves it flying
    /// with the pad's boost (`Event::Reset` with the pad's platform), or
    /// with Floor is Lava it is eaten. Returns whether it is now out.
    pub(crate) fn tower_rules(&mut self, i: usize) -> bool {
        if self.course.tower.is_none() {
            return false;
        }
        let r = &self.racers[i];
        if r.finish.is_some() || r.eaten.is_some() {
            return false;
        }
        let (pos, hint) = (r.ape.pos, r.ape.hint);
        if !self.course.tower_out(pos, hint) {
            return false;
        }
        if self.lava && self.running() {
            self.eat(i);
            return true;
        }
        let Some(t) = self.course.tower.as_deref() else {
            return false;
        };
        let pad = self.course.tower_pad(t, &r.ape);
        let pl = &self.course.platforms[pad.platform];
        let at = v3(pl.centre.x, pl.top + BODY_DROP, pl.centre.z);
        let yaw = yaw_of(pl.fwd);
        let a = &mut self.racers[i].ape;
        a.put_on(at, yaw, &self.course);
        if pad.boost != V3::ZERO {
            a.vel = pad.boost;
            a.mode = Mode::Flying;
        }
        self.events.push(Event::Reset {
            who: i,
            pad: pad.platform,
        });
        false
    }
}
