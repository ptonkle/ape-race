//! An ape in open space (Sector Ape, `sector.rs`): vines that hang every
//! way, the slingshot off an odd vine, and the ships, debris and door it
//! can hit. Nothing here runs on another map (`Ape::step` calls it only
//! when the course has a `SectorLayout`).

use super::*;
use crate::fx::int;
use crate::sector::{sweep, Hanger, Model, Touch, REBOUND_BOOST, SECTOR_TOP, SLING};

/// The ape's ball for hits with ships and debris: round the middle of its
/// body (the hands are `BODY_DROP / 2` above, the feet as far below).
pub const HIT_RADIUS: i32 = cm(80);

/// A hit is put back this far off the face it touched.
const SKIN: i32 = cm(2);

/// A rebound sends an ape back down the course no faster than this.
pub const REBOUND_BACK: i32 = int(6);

impl Ape {
    /// Puts the ape on a deck at `at` (its hands), standing still, facing
    /// `yaw`: the boundary's reset. It keeps its kind, stats and bananas.
    pub fn put_on(&mut self, at: V3, yaw: i32, course: &Course) {
        *self = Ape {
            phys: self.phys,
            bananas: self.bananas,
            banana_drag: self.banana_drag,
            ..Ape::new(self.kind, false, at, yaw)
        };
        self.hint = course.nearest_anywhere(at);
    }

    /// The nearest vine in reach of the hands, its vine the segment from its
    /// anchor along the way it hangs.
    pub(super) fn reach_any_way(&self, course: &Course, tick: u32) -> Option<usize> {
        let mut best = None;
        let mut best_d = self.phys.reach;
        for i in 0..course.vines.len() {
            if let Some((v, ago)) = self.last_vine {
                if v == i && ago < RELEASE_IGNORE {
                    continue;
                }
            }
            let a = course.anchor(i, tick);
            let len = course.vines[i].length;
            let d = self.pos.sub(a);
            // quick reject: out of a box round the anchor the vine's length
            // and the reach wide
            let lim = len + best_d;
            if d.x.abs() > lim || d.y.abs() > lim || d.z.abs() > lim {
                continue;
            }
            let h = course.hang(i);
            let t = d.dot(h).clamp(0, len);
            let dist = self.pos.dist(a.add(h.scale(t)));
            if dist <= best_d {
                best_d = dist;
                best = Some(i);
            }
        }
        best
    }

    /// Letting go of an odd vine: the slingshot, up to `SLING` more speed
    /// along the motion, the more the motion lines up with the way on (none
    /// at 60° off it or worse). Never past `SECTOR_TOP`.
    pub(super) fn sling(&mut self, vine: usize, course: &Course) {
        if !course.odd_vine(vine) {
            return;
        }
        let speed = self.vel.len();
        if speed < ONE {
            return;
        }
        let (s, _) = course.progress(self.pos, self.hint);
        let on = course.heading_at(s);
        let align = div(self.vel.dot(on), speed);
        let k = ((align - ONE / 2) * 2).clamp(0, ONE);
        let gain = mul(SLING, mul(k, k));
        if gain > 0 {
            let top = (speed + gain).min(SECTOR_TOP).max(speed);
            self.vel = self.vel.with_len(top);
        }
    }

    /// The ships, debris and (while shut) the door: the body's ball swept
    /// from where it was at the start of the tick to where it is, so
    /// nothing thin is passed through however fast. The first touch puts
    /// the ape back at it: a bouncy body throws it off elastically with a
    /// boost (`REBOUND_BOOST`, never past `SECTOR_TOP`), anything else stops
    /// the part of the motion into it (as a wall). A hit knocks the ape off
    /// its vine. A deck's top is ground (its platform), not a hit.
    pub(super) fn hit_bodies(&mut self, course: &Course, tick: u32, before: V3) {
        let Some(s) = course.sector.as_deref() else {
            return;
        };
        let lift = v3(0, BODY_DROP / 2, 0);
        let p0 = before.sub(lift);
        let p1 = self.pos.sub(lift);
        let step = p1.sub(p0);
        let span = step.x.abs().max(step.y.abs()).max(step.z.abs());
        let mut best: Option<(Touch, bool)> = None;
        let mut take = |t: Touch, bouncy: bool| {
            if best.is_none_or(|(b, _)| t.t < b.t) {
                best = Some((t, bouncy));
            }
        };
        // a swing never runs into the piece of debris its own vine hangs
        // from (the vine holds the ape off it); a ship's hull still stops it
        let own = self.swinging().and_then(|v| match s.hangers.get(v) {
            Some(Hanger::Body(k)) if s.bodies[*k].model != Model::Hidden => Some(*k),
            _ => None,
        });
        for &b in s.bodies_near(self.hint) {
            if Some(b as usize) == own {
                continue;
            }
            let body = &s.bodies[b as usize];
            let c = body.frame.t;
            let r = body.reach() + HIT_RADIUS + span;
            if (p1.x - c.x).abs() > r || (p1.y - c.y).abs() > r || (p1.z - c.z).abs() > r {
                continue;
            }
            let axes = body.axes();
            if let Some(t) = sweep(c, &axes, body.half, HIT_RADIUS, p0, p1) {
                // a deck's top face is its platform: stood on, not hit
                if body.model == Model::Deck && t.normal.dot(axes[1]) > ONE / 2 {
                    continue;
                }
                take(t, body.bouncy);
            }
        }
        if s.door.shut(tick) {
            let d = &s.door;
            let axes = [d.frame.c0.norm(), d.frame.c1.norm(), d.frame.c2.norm()];
            if let Some(t) = sweep(d.frame.t, &axes, d.half, HIT_RADIUS, p0, p1) {
                take(t, false);
            }
        }
        let Some((t, bouncy)) = best else {
            return;
        };
        let n = t.normal;
        let at = p0.add(step.scale(t.t)).add(n.scale(t.depth + SKIN));
        self.pos = at.add(lift);
        let into = self.vel.dot(n);
        if into < 0 {
            if bouncy {
                // elastic: v - 2(v·n)n, then the boost on along the
                // course (along the new way, a hit from below threw apes
                // down out of the boundary: SECOND-ORDER SC rows)
                let (s, _) = course.progress(self.pos, self.hint);
                let on = course.heading_at(s);
                let mut out = self.vel.sub(n.scale(2 * into)).add(on.scale(REBOUND_BOOST));
                // a head-on hit is a crash, not a boost back down the
                // course: no more than `REBOUND_BACK` of it backward
                let back = -out.dot(on);
                if back > REBOUND_BACK {
                    out = out.add(on.scale(back - REBOUND_BACK));
                }
                self.vel = out.clamp_len(SECTOR_TOP);
            } else {
                self.vel = self.vel.sub(n.scale(into + mul(into, WALL_BOUNCE)));
            }
        }
        if let Mode::Swinging { vine, .. } = self.mode {
            self.mode = Mode::Flying;
            self.last_vine = Some((vine, 0));
            self.release_bonus = 0;
        }
        if bouncy {
            if self.mode == Mode::Grounded {
                self.mode = Mode::Flying;
            }
            self.hopped = false;
            self.happened.push(Happening::Rebound);
        } else {
            self.happened.push(Happening::Wall);
        }
    }
}
