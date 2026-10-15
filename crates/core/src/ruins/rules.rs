//! The rules Sunset Ruins adds, each called only when `Course.ruins` is
//! set: the river (its floor, its current, the way out), the turtles as
//! moving bouncers, the temple's walls and ceilings, the trapdoors' pits,
//! the spikes, the bridge's boost and the ravine's reset.

use super::*;
use crate::ape::{Ape, Happening, Mode};
use crate::course::{Course, Mushroom};
use crate::fx::mul;
use crate::geom::right_of;
use crate::race::Event;
use crate::rules::{BODY_DROP, BODY_RADIUS, CEILING, SWIM_SINK, WALL_BOUNCE};

impl RuinsLayout {
    /// The bouncer index's turtle, if a turtle carries it.
    fn turtle_of(&self, m: usize) -> Option<usize> {
        self.turtles.iter().position(|t| t.bouncer == m)
    }

    /// Whether `p` (flat) is over trapdoor `k`'s door.
    fn over_door(&self, k: usize, p: V3) -> bool {
        let d = &self.traps[k].door;
        let q = v3(p.x, d.centre.y, p.z);
        d.contains(q)
    }
}

impl Course {
    /// Sunset Ruins' floor at (`x`, `z`): in the river where a swimmer's
    /// feet hang (water counts as the floor), else the land.
    pub(crate) fn ruins_floor(&self, r: &RuinsLayout, x: i32, z: i32) -> i32 {
        let land = self.terrain(x, z);
        match r.river.water_at(x, z) {
            Some(w) => w.surface - SWIM_SINK,
            None => land,
        }
    }

    /// The river's surface under `p`, if `p` is over it.
    pub(crate) fn ruins_water(&self, r: &RuinsLayout, p: V3) -> Option<i32> {
        r.river.water_at(p.x, p.z).map(|w| w.surface)
    }

    /// Where a swimmer makes for: the nearer bank, straight across the
    /// river, and the bank's top to climb out onto.
    pub(crate) fn ruins_exit(&self, r: &RuinsLayout, p: V3) -> Option<(V3, Option<V3>)> {
        r.river.water_at(p.x, p.z)?;
        let i = self.nearest(p, self.nearest_anywhere_cheap(p));
        let h = self.heading[i];
        let right = right_of(h);
        let mid = self.path[i];
        let side = p.flat().sub(mid).dot(right);
        let out = if side >= 0 { right } else { right.neg() };
        let to = mid.add(out.scale(RIVER_HALF - cm(30)));
        let up = mid.add(out.scale(RIVER_HALF + cm(150)));
        let top = v3(up.x, self.terrain(up.x, up.z) + BODY_DROP, up.z);
        Some((v3(to.x, p.y, to.z), Some(top)))
    }

    /// `nearest_anywhere`, for a swimmer (the river is short: the first
    /// third of the course).
    fn nearest_anywhere_cheap(&self, p: V3) -> usize {
        let end = self
            .ruins
            .as_ref()
            .map_or(self.path.len(), |r| {
                (r.marks.lip / crate::course::STEP) as usize + 1
            })
            .min(self.path.len());
        let mut best = 0;
        let mut best_d = i64::MAX;
        for i in (0..end).step_by(4) {
            let d = p.flat().sub(self.path[i]);
            let dd = d.dot64(d);
            if dd < best_d {
                best_d = dd;
                best = i;
            }
        }
        best
    }

    /// Sunset Ruins' bouncer `m` at `tick`: a turtle's moves with it.
    pub(crate) fn ruins_bouncer(&self, r: &RuinsLayout, m: usize, tick: u32) -> Mushroom {
        let b = self.mushrooms[m];
        match r.turtle_of(m) {
            Some(t) => {
                let (at, _) = r.turtle_at(t, tick);
                Mushroom {
                    top: v3(at.x, at.y + TURTLE_TOP, at.z),
                    radius: b.radius,
                }
            }
            None => b,
        }
    }

    /// The bouncer a body at `p` is on at `tick` (a mushroom, or a turtle
    /// where it swims now). None above the tallest bouncer.
    pub(crate) fn ruins_bouncer_under(&self, r: &RuinsLayout, p: V3, tick: u32) -> Option<usize> {
        let feet = p.y - BODY_DROP;
        if feet > self.terrain(p.x, p.z) + BOUNCER_TALLEST {
            return None;
        }
        let floor = self.ruins_floor(r, p.x, p.z) - int(1);
        let on = |m: &Mushroom| {
            let d = p.flat().sub(m.top.flat());
            let rr = m.radius + BODY_RADIUS / 2;
            d.x.abs() <= rr && d.z.abs() <= rr && d.len() <= rr && feet <= m.top.y && feet >= floor
        };
        (0..self.mushrooms.len()).find(|&m| on(&self.ruins_bouncer(r, m, tick)))
    }

    /// The highest surface under `p` at `tick`: `support`, but an open
    /// trapdoor's floor has dropped away to its pit.
    pub fn support_at(&self, p: V3, tick: u32) -> i32 {
        let top = self.support(p);
        let Some(r) = self.ruins.as_deref() else {
            return top;
        };
        for (k, t) in r.traps.iter().enumerate() {
            if t.open(tick) && r.over_door(k, p) && top <= t.door.centre.y + cm(50) {
                return t.pit;
            }
        }
        top
    }

    /// Sunset Ruins' walls, `s` metres along and `side` right of the
    /// centreline at sample `i`: the river's banks and the ravine as the
    /// Jungle's (the course's half width), the corridor narrower, the
    /// chamber's ring between its wall and the invisible inner wall, and
    /// past the turn in only the chamber's wall.
    pub(crate) fn ruins_walls(
        &self,
        r: &RuinsLayout,
        pos: V3,
        vel: V3,
        i: usize,
        s: i32,
        side: i32,
    ) -> (V3, V3, bool) {
        if s >= r.marks.chamber {
            let c = r.chamber.centre.flat();
            let d = pos.flat().sub(c);
            let dist = d.len();
            let hi = r.chamber.radius - BODY_RADIUS;
            let lo = if s < r.marks.turn_in {
                r.chamber.inner + BODY_RADIUS
            } else {
                0
            };
            let t = dist.clamp(lo, hi);
            if t == dist || dist == 0 {
                return (pos, vel, false);
            }
            let out = if dist > hi { d.norm() } else { d.norm().neg() };
            let q = c.add(d.norm().scale(t));
            let pos = v3(q.x, pos.y, q.z);
            let into = vel.dot(out);
            let vel = if into > 0 {
                vel.sub(out.scale(into + mul(into, WALL_BOUNCE)))
            } else {
                vel
            };
            return (pos, vel, true);
        }
        let half = if s >= r.marks.mouth {
            CORRIDOR_HALF
        } else {
            self.half_width
        };
        let limit = half - BODY_RADIUS;
        if side.abs() <= limit {
            return (pos, vel, false);
        }
        let right = self.right[i];
        let out = if side > 0 { right } else { right.neg() };
        let pos = pos.sub(out.scale(side.abs() - limit));
        let into = vel.dot(out);
        let vel = if into > 0 {
            vel.sub(out.scale(into + mul(into, WALL_BOUNCE)))
        } else {
            vel
        };
        (pos, vel, true)
    }

    /// The highest the hands may go `s` metres along: the sky over the
    /// river and the ravine (as the Jungle, from the lip's height over the
    /// ravine), the corridor's ceiling, the chamber's roof.
    pub(crate) fn ruins_ceiling(&self, r: &RuinsLayout, s: i32, pos: V3) -> i32 {
        let floor = r.chamber.centre.y;
        if s >= r.marks.chamber {
            floor + CHAMBER_HEIGHT - cm(50)
        } else if s >= r.marks.mouth {
            floor + CORRIDOR_HEIGHT - cm(50)
        } else if s >= r.marks.lip {
            floor + TEMPLE_BELOW_LIP + CEILING
        } else {
            self.terrain(pos.x, pos.z) + CEILING
        }
    }
}

impl Ape {
    /// Sunset Ruins on a let-go: a boosted vine of the rope bridge flings
    /// an ape going toward the temple's mouth on a line into it; one going
    /// any other way gets nothing (and falls).
    pub(crate) fn bridge_boost(&mut self, vine: usize, course: &Course) {
        let Some(r) = course.ruins.as_deref() else {
            return;
        };
        if !r.bridge.boost.contains(&vine) {
            return;
        }
        let to = r.bridge.aim.sub(self.pos);
        let flat = to.flat();
        let d = flat.len();
        if d < int(4) {
            return;
        }
        let dir = flat.norm();
        let toward = self.vel.flat().dot(dir);
        if toward < BOOST_DECENT {
            return;
        }
        let h = toward.max(BOOST_SPEED);
        // a ballistic line through the aim (drag takes a little: aim a
        // tenth higher)
        let t = fx::div(d, h);
        let g = course.gravity();
        let vy = fx::div(to.y + mul(g, mul(t, t)) / 2, t);
        let vy = vy + vy.abs() / 10 + int(1);
        self.vel = dir.scale(h).add(v3(0, vy, 0));
    }

    /// Sunset Ruins on the floor: a spike trap, or the spikes in an open
    /// pit, fling the ape up, and it cannot grab for `SPIKE_NO_GRAB`.
    pub(crate) fn spikes(&mut self, course: &Course, tick: u32) {
        let Some(r) = course.ruins.as_deref() else {
            return;
        };
        if self.vel.y > 0 || self.swinging().is_some() {
            return;
        }
        let feet = self.feet();
        let on_trap = r.spikes.iter().any(|t| {
            let a = t.area;
            let q = v3(feet.x, a.centre.y, feet.z);
            a.contains(q) && feet.y <= a.centre.y + a.half.y + cm(20)
        });
        let in_pit = r
            .traps
            .iter()
            .enumerate()
            .any(|(k, t)| t.open(tick) && r.over_door(k, feet) && feet.y <= t.pit + cm(30));
        if on_trap || in_pit {
            self.vel = v3(self.vel.x / 2, SPIKE_FLING, self.vel.z / 2);
            self.mode = Mode::Flying;
            self.no_grab = SPIKE_NO_GRAB as i32;
            self.happened.push(Happening::Spiked);
        }
    }
}

impl Race {
    /// The ravine: ape `i` fallen too deep is put back at once on the
    /// riverbank before the bridge (with Floor is Lava it is eaten). True
    /// if it was eaten.
    pub(crate) fn ravine(&mut self, i: usize) -> bool {
        let Some(s) = self.course.ruins.as_deref() else {
            return false;
        };
        let r = &self.racers[i];
        if r.finish.is_some() || r.eaten.is_some() {
            return false;
        }
        let (along, _) = self.course.progress(r.ape.pos, r.ape.hint);
        if !s.ravine.fallen(r.ape.pos, along) {
            return false;
        }
        if self.lava && self.running() {
            self.eat(i);
            return true;
        }
        let (at, yaw) = (s.ravine.spot, s.ravine.spot_yaw);
        self.racers[i].ape.put_on(at, yaw, &self.course);
        self.events.push(Event::Reset { who: i, pad: 0 });
        false
    }
}
