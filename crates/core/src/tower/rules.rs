//! Mape's Tower's rules, each run only when `Course.tower` is set: the 3D
//! "along the course" (TW1), the boundary and the reset onto a pad with
//! its boost, and the climb vines' lift.

use super::*;
use crate::ape::{Ape, Mode};
use crate::course::{Course, STEP};
use crate::race::{Event, Race};
use crate::rules::BODY_DROP;

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
    /// it, or standing on the open ground. Never on another map.
    pub fn tower_out(&self, p: V3, hint: usize) -> bool {
        if self.tower.is_none() {
            return false;
        }
        let q = self.path[hint.min(self.path.len() - 1)];
        let feet = p.y - BODY_DROP;
        let ground = self.terrain(p.x, p.z);
        p.sub(q).longer_than(OUT_FAR)
            || p.y < q.y - OUT_BELOW
            || (feet <= ground + cm(10) && self.support(p) <= ground)
    }

    /// The reset pad for an ape out at `p`: the last pad not ahead of it
    /// along the course (the first, the start deck, if none).
    pub fn tower_pad(&self, t: &TowerLayout, p: V3, hint: usize) -> Pad {
        let (along, _) = self.progress(p, hint);
        t.pads
            .iter()
            .rev()
            .find(|pad| pad.at <= along)
            .or(t.pads.first())
            .copied()
            .expect("the tower has a start pad")
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
}

impl Race {
    /// Mape's Tower's boundary for local ape `i` after its step: out of
    /// bounds, it is put on the last pad behind it and leaves it flying
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
        let pad = self.course.tower_pad(t, pos, hint);
        let pl = &self.course.platforms[pad.platform];
        let at = v3(pl.centre.x, pl.top + BODY_DROP, pl.centre.z);
        let yaw = crate::geom::yaw_of(pl.fwd);
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
