//! An ape in open space (Sector Ape, `sector.rs`): vines that hang every
//! way, the slingshot off an odd vine, and the ships, debris and door it
//! can hit. Nothing here runs on another map (`Ape::step` calls it only
//! when the course has a `SectorLayout`).

use super::*;
use crate::fx::int;
use crate::sector::{
    sweep, Hanger, Model, Touch, PASSAGE_FLOOR, PASSAGE_LAUNCH, REBOUND_BOOST, SECTOR_TOP, SLING,
};

/// The ape's ball for hits with ships and debris: round the middle of its
/// body (the hands are `BODY_DROP / 2` above, the feet as far below).
pub const HIT_RADIUS: i32 = cm(80);

/// A hit is put back this far off the face it touched.
const SKIN: i32 = cm(2);

/// A rebound sends an ape back down the course no faster than this, and
/// down no faster than this.
pub const REBOUND_BACK: i32 = int(6);
pub const REBOUND_DOWN: i32 = int(8);
/// After a rebound's boost, none again for this many ticks.
pub const REBOUND_COOL: u8 = 30;

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
        let s = course.sector.as_deref()?;
        // only the course's vines hanging near along the course (and every
        // summoned one): the same answer as looking at them all
        let (here, _) = course.progress(self.pos, self.hint);
        let summoned = s.by_at.len()..course.vines.len();
        let mut best: Option<(i32, usize)> = None;
        for i in s.vines_near(here).chain(summoned) {
            if let Some((v, ago)) = self.last_vine {
                if v == i && ago < RELEASE_IGNORE {
                    continue;
                }
            }
            let reach = best.map_or(self.phys.reach, |(d, _)| d);
            let a = course.anchor(i, tick);
            let len = course.vines[i].length;
            let d = self.pos.sub(a);
            // quick reject: out of a box round the anchor the vine's length
            // and the reach wide
            let lim = len + reach;
            if d.x.abs() > lim || d.y.abs() > lim || d.z.abs() > lim {
                continue;
            }
            let h = course.hang(i);
            let t = d.dot(h).clamp(0, len);
            let dist = self.pos.dist(a.add(h.scale(t)));
            // (of two as near, the later vine, as a search of them all in
            // order would find)
            if dist < reach || (dist == reach && best.is_none_or(|(_, b)| i > b)) {
                best = Some((dist, i));
            }
        }
        best.map(|(_, i)| i)
    }

    /// `reach_any_way` by looking at every vine (for the tests).
    #[cfg(test)]
    pub(crate) fn reach_any_way_all(&self, course: &Course, tick: u32) -> Option<usize> {
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

    /// The wreck's launch (T88): flying into its passage's opening on the
    /// way on, the ape is turned straight down the passage and sped up by
    /// `PASSAGE_LAUNCH` (at least to `PASSAGE_FLOOR`, never past
    /// `SECTOR_TOP`), so the hidden shortcut pays and threads itself once
    /// in. Only on the tick it goes in (its middle outside the opening at
    /// the start of the tick, inside at the end).
    pub(super) fn passage_launch(&mut self, course: &Course, before: V3) {
        let Some(s) = course.sector.as_deref() else {
            return;
        };
        if self.swinging().is_some() {
            return;
        }
        let w = &s.wreck;
        let lift = v3(0, BODY_DROP / 2, 0);
        if !w.in_passage(self.pos.sub(lift)) || w.in_passage(before.sub(lift)) {
            return;
        }
        let axis = w.passage.axis;
        if self.vel.dot(axis) <= 0 {
            return;
        }
        let speed = (self.vel.len() + PASSAGE_LAUNCH).clamp(PASSAGE_FLOOR, SECTOR_TOP);
        self.vel = axis.scale(speed);
        self.happened.push(Happening::Launched);
    }

    /// The ships, debris and (while shut) the door: the body's ball swept
    /// from where it was at the start of the tick to where it is, so
    /// nothing thin is passed through however fast. The first touch puts
    /// the ape back at it: a bouncy body throws it off elastically with a
    /// boost (`REBOUND_BOOST`, never past `SECTOR_TOP`), anything else stops
    /// the part of the motion into it (as a wall). A hit knocks the ape off
    /// its vine. A deck's top is ground (its platform), not a hit.
    pub(super) fn hit_bodies(&mut self, course: &Course, tick: u32, before: V3) {
        if self.rebound_cool > 0 {
            self.rebound_cool -= 1;
        }
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
            let (axes, reach) = s.fit(b as usize);
            let r = reach + HIT_RADIUS + span;
            if (p1.x - c.x).abs() > r || (p1.y - c.y).abs() > r || (p1.z - c.z).abs() > r {
                continue;
            }
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
        let at = if t.depth > 0 {
            // (it started the tick inside: the move is kept, and put back
            // out through the face it was nearest, so it slides on and out
            // rather than sticking)
            p1.add(n.scale(t.end_depth + SKIN))
        } else {
            p0.add(step.scale(t.t)).add(n.scale(SKIN))
        };
        self.pos = at.add(lift);
        let into = self.vel.dot(n);
        let swinging = self.swinging().is_some();
        let pinned = t.depth > 0;
        if into < 0 {
            if bouncy && swinging && !pinned {
                // on a vine it holds on: the swing bounces off the debris
                // (elastic, no boost), back the way it came
                self.vel = self.vel.sub(n.scale(2 * into));
            } else if bouncy {
                // elastic: v - 2(v·n)n, then the boost on along the
                // course (along the new way, a hit from below threw apes
                // down out of the boundary: SECOND-ORDER SC rows)
                let (s, _) = course.progress(self.pos, self.hint);
                let on = course.heading_at(s);
                // (the boost once a touch: not again for a quarter second, nor
                // while pressed in)
                let boost = if self.rebound_cool > 0 || pinned {
                    0
                } else {
                    REBOUND_BOOST
                };
                self.rebound_cool = REBOUND_COOL;
                let mut out = self.vel.sub(n.scale(2 * into)).add(on.scale(boost));
                // a head-on hit is a crash, not a boost back down the
                // course: no more than `REBOUND_BACK` of it backward
                let back = -out.dot(on);
                if back > REBOUND_BACK {
                    out = out.add(on.scale(back - REBOUND_BACK));
                }
                // ... nor a dunk: off the underside of a piece of debris
                // it drops away gently, not out through the boundary
                if out.y < -REBOUND_DOWN {
                    out.y = -REBOUND_DOWN;
                }
                self.vel = out.clamp_len(SECTOR_TOP);
            } else {
                self.vel = self.vel.sub(n.scale(into + mul(into, WALL_BOUNCE)));
            }
        }
        // a ship's dead hull knocks it off the vine
        // a ship's dead hull knocks it off the vine, and so does debris the
        // rope holds it into (it would only be pressed in and out again)
        if let (Mode::Swinging { vine, .. }, true) = (self.mode, !bouncy || pinned) {
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
