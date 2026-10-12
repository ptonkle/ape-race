//! The course (PLAN P7): one hand-authored jungle run from a start tree to
//! a finish arch. Its centreline is a list of straights and bends, so it
//! turns but never comes back to the start (decision 3). Vines, crates,
//! moving parts and scenery are placed along it by fixed rules from a fixed
//! seed, so every race on it is the same course.
//!
//! The data model has room for a lap circuit: `laps` counts how many times
//! the centreline is run. The first course is run once.

use crate::fx::{self, cm, deg, int, mul, sin, Dice, ONE};
use crate::geom::{facing, right_of, v3, V3};
use crate::rules::{BODY_DROP, BODY_RADIUS, CEILING, WALL_BOUNCE};

/// Centreline samples are this far apart.
pub const STEP: i32 = int(2);

/// A vine: hangs from `anchor` for `length`. A sliding vine's anchor moves
/// to and fro along `slide` (a moving part).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vine {
    pub anchor: V3,
    pub length: i32,
    pub slide: Option<Slide>,
    /// where along the course it hangs, for the AI and the minimap
    pub at: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slide {
    /// half the travel, as a vector
    pub reach: V3,
    pub period: i32,
}

/// A swinging log (a moving part): hangs from `pivot` on two ropes and
/// swings across the course. Touching it knocks an ape aside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Log {
    pub pivot: V3,
    pub rope: i32,
    /// the direction it swings in (across the course), a unit vector
    pub across: V3,
    /// along the log (the course's direction), a unit vector
    pub along: V3,
    pub half_len: i32,
    pub radius: i32,
    pub amplitude: i32,
    pub period: i32,
}

/// A flat box an ape can stand on: the start tree's platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    pub centre: V3,
    /// the platform's own axes on the ground
    pub fwd: V3,
    pub half_len: i32,
    pub half_width: i32,
    /// its top
    pub top: i32,
}

/// Something in the jungle that is only drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prop {
    /// a trunk at its foot, its radius and height
    Tree {
        at: V3,
        radius: i32,
        height: i32,
        shade: u8,
    },
    /// a branch from `from` to `to`, carrying a vine
    Branch {
        from: V3,
        to: V3,
    },
    Bush {
        at: V3,
        size: i32,
        shade: u8,
    },
    Fern {
        at: V3,
        size: i32,
        yaw: i32,
    },
    Flower {
        at: V3,
        colour: u8,
    },
    Rock {
        at: V3,
        size: i32,
        yaw: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Course {
    pub name: &'static str,
    /// centreline points on the ground, `STEP` apart, and their headings
    pub path: Vec<V3>,
    pub heading: Vec<V3>,
    pub length: i32,
    pub half_width: i32,
    pub laps: u32,
    pub vines: Vec<Vine>,
    pub logs: Vec<Log>,
    pub platforms: Vec<Platform>,
    pub crates: Vec<V3>,
    pub props: Vec<Prop>,
    /// where the four starters stand, and the way they face
    pub starts: [V3; 4],
    pub start_yaw: i32,
}

/// A straight (`turn` 0) or a bend of the centreline: metres, degrees.
struct Leg(i32, i32);

/// The jungle course. 1,800 m: straights and bends turning both ways.
const JUNGLE: &[Leg] = &[
    Leg(60, 0),
    Leg(150, 0),
    Leg(140, 50),
    Leg(110, 0),
    Leg(180, -80),
    Leg(100, 0),
    Leg(120, 40),
    Leg(120, -40),
    Leg(150, 0),
    Leg(200, 90),
    Leg(130, 0),
    Leg(160, -45),
    Leg(180, 0),
];

const SEED: u64 = 0x0A9E_2ACE_0001;

/// Where the start platform is, and how high.
pub const START_HEIGHT: i32 = int(16);
const START_LEN: i32 = int(14);

/// The ground's gentle hills.
pub fn ground(x: i32, z: i32) -> i32 {
    // about 0.05 and 0.04 radians a metre
    let a = sin(mul(x, 521 * ONE / ONE));
    let b = fx::cos(mul(z, 417));
    mul(mul(a, b), cm(90))
}

impl Course {
    pub fn jungle() -> Course {
        build(JUNGLE, SEED)
    }

    /// A short flat test course: one straight, vines in a line.
    pub fn test_line(vines: &[(i32, i32, i32)]) -> Course {
        let mut c = centreline("test", &[Leg(400, 0)]);
        for (at, height, length) in vines {
            let p = c.point(*at);
            c.vines.push(Vine {
                anchor: v3(p.x, *height, p.z),
                length: *length,
                slide: None,
                at: *at,
            });
        }
        c
    }

    pub fn samples(&self) -> usize {
        self.path.len()
    }

    /// The centreline point `s` metres along.
    pub fn point(&self, s: i32) -> V3 {
        let i = (s / STEP).clamp(0, self.path.len() as i32 - 1) as usize;
        let rest = s - i as i32 * STEP;
        self.path[i].add(self.heading[i].scale(rest))
    }

    pub fn heading_at(&self, s: i32) -> V3 {
        let i = (s / STEP).clamp(0, self.path.len() as i32 - 1) as usize;
        self.heading[i]
    }

    /// Where a vine's anchor is at a tick (a sliding vine moves).
    pub fn anchor(&self, i: usize, tick: u32) -> V3 {
        let v = &self.vines[i];
        match v.slide {
            None => v.anchor,
            Some(s) => {
                let phase = ((tick as i64 * fx::TURN as i64) / s.period as i64) as i32;
                v.anchor.add(s.reach.scale(sin(phase + i as i32 * 9_001)))
            }
        }
    }

    /// A log's angle from hanging straight down, at a tick.
    pub fn log_angle(&self, i: usize, tick: u32) -> i32 {
        let l = &self.logs[i];
        let phase = ((tick as i64 * fx::TURN as i64) / l.period as i64) as i32;
        mul(l.amplitude, sin(phase + i as i32 * 21_000))
    }

    /// A log's middle at a tick.
    pub fn log_centre(&self, i: usize, tick: u32) -> V3 {
        let l = &self.logs[i];
        let a = self.log_angle(i, tick);
        l.pivot
            .add(l.across.scale(mul(l.rope, sin(a))))
            .sub(V3::UP.scale(mul(l.rope, fx::cos(a))))
    }

    /// The highest surface under a point: the ground, or a platform's top.
    pub fn support(&self, p: V3) -> i32 {
        let mut top = ground(p.x, p.z);
        let feet = p.y - BODY_DROP;
        for pl in &self.platforms {
            let d = p.sub(pl.centre);
            let along = d.dot(pl.fwd);
            let side = d.dot(right_of(pl.fwd));
            // a platform holds an ape whose feet are above it (or just through it)
            if along.abs() <= pl.half_len
                && side.abs() <= pl.half_width
                && feet >= pl.top - cm(60)
                && pl.top > top
            {
                top = pl.top;
            }
        }
        top
    }

    /// The nearest centreline sample to `p`, searched near `hint`.
    pub fn nearest(&self, p: V3, hint: usize) -> usize {
        let n = self.path.len();
        let lo = hint.saturating_sub(40);
        let hi = (hint + 40).min(n - 1);
        let mut best = hint.min(n - 1);
        let mut best_d = i64::MAX;
        for i in lo..=hi {
            let d = p.flat().sub(self.path[i]);
            let dd = d.dot64(d);
            if dd < best_d {
                best_d = dd;
                best = i;
            }
        }
        best
    }

    /// The nearest sample by a search of the whole centreline.
    pub fn nearest_anywhere(&self, p: V3) -> usize {
        let mut best = 0;
        let mut best_d = i64::MAX;
        for i in 0..self.path.len() {
            let d = p.flat().sub(self.path[i]);
            let dd = d.dot64(d);
            if dd < best_d {
                best_d = dd;
                best = i;
            }
        }
        best
    }

    /// How far along the course `p` is, and how far right of the centreline.
    pub fn progress(&self, p: V3, i: usize) -> (i32, i32) {
        let d = p.flat().sub(self.path[i]);
        let along = d.dot(self.heading[i]);
        let side = d.dot(right_of(self.heading[i]));
        ((i as i32 * STEP + along).clamp(0, self.length), side)
    }

    /// Keeps a body inside the corridor and under the ceiling: returns the
    /// corrected position and velocity, and whether a wall was hit.
    pub fn contain(&self, pos: V3, vel: V3, i: usize) -> (V3, V3, bool) {
        let (_, side) = self.progress(pos, i);
        let limit = self.half_width - BODY_RADIUS;
        let mut pos = pos;
        let mut vel = vel;
        let mut hit = false;
        if side.abs() > limit {
            let right = right_of(self.heading[i]);
            let out = if side > 0 { right } else { right.neg() };
            pos = pos.sub(out.scale(side.abs() - limit));
            let into = vel.dot(out);
            if into > 0 {
                vel = vel.sub(out.scale(into + mul(into, WALL_BOUNCE)));
            }
            hit = true;
        }
        // the start's back wall and the finish's far side
        let behind = pos.flat().sub(self.path[0]).dot(self.heading[0]);
        if behind < -int(6) {
            pos = pos.add(self.heading[0].scale(-int(6) - behind));
            let into = -vel.dot(self.heading[0]);
            if into > 0 {
                vel = vel.add(self.heading[0].scale(into));
            }
        }
        let top = ground(pos.x, pos.z) + CEILING;
        if pos.y > top {
            pos.y = top;
            vel.y = vel.y.min(0);
        }
        (pos, vel, hit)
    }
}

/// The centreline alone.
fn centreline(name: &'static str, legs: &[Leg]) -> Course {
    let mut path = Vec::new();
    let mut heading = Vec::new();
    let mut p = V3::ZERO;
    let mut yaw = 0;
    for leg in legs {
        let steps = leg.0 * ONE / STEP;
        let turn = deg(leg.1);
        for k in 0..steps {
            let y = yaw + (turn as i64 * (2 * k as i64 + 1) / (2 * steps as i64)) as i32;
            let h = facing(y, 0);
            path.push(p);
            heading.push(h);
            p = p.add(h.scale(STEP));
        }
        yaw += turn;
    }
    path.push(p);
    heading.push(facing(yaw, 0));
    let length = (path.len() as i32 - 1) * STEP;
    Course {
        name,
        path,
        heading,
        length,
        half_width: int(20),
        laps: 1,
        vines: Vec::new(),
        logs: Vec::new(),
        platforms: Vec::new(),
        crates: Vec::new(),
        props: Vec::new(),
        starts: [V3::ZERO; 4],
        start_yaw: 0,
    }
}

fn build(legs: &[Leg], seed: u64) -> Course {
    let mut c = centreline("jungle", legs);
    let mut dice = Dice(seed);
    let len_m = c.length / ONE;

    // the start tree's platform; the starters stand across it
    let p0 = c.point(START_LEN / 2 - int(2));
    let fwd = c.heading_at(0);
    let right = right_of(fwd);
    c.platforms.push(Platform {
        centre: v3(p0.x, START_HEIGHT, p0.z),
        fwd,
        half_len: START_LEN / 2 + int(2),
        half_width: int(9),
        top: START_HEIGHT,
    });
    for k in 0..4 {
        let side = int(-6) + k * int(4);
        let at = c.point(int(2)).add(right.scale(side));
        c.starts[k as usize] = v3(at.x, START_HEIGHT + BODY_DROP, at.z);
    }
    c.start_yaw = 0;

    // Vines in stations. A station's vines sit off to one side, the next
    // station's often to the other, so a straight line through them all
    // does not exist; the gap between stations varies, so a slow ape must
    // drop to a lower vine, and a low vine every few stations lets an ape
    // on the ground climb back up.
    let mut s = 30;
    let mut side_sign = 1;
    let mut station = 0;
    let mut slides = 0;
    while s < len_m - 40 {
        let h = c.heading_at(int(s));
        let r = right_of(h);
        // the first vine is close, so the jump off the platform reaches it
        let (anchor_h, length) = if station == 0 {
            (int(30), int(12))
        } else {
            (int(dice.range(23, 33)), int(dice.range(10, 17)))
        };
        let off = if station == 0 {
            0
        } else {
            side_sign * dice.range(3, 11)
        };
        let base = c.point(int(s)).add(r.scale(int(off)));
        let ground_y = ground(base.x, base.z);
        let mut slide = None;
        // three sliding vines, spread down the course
        if station > 0 && station % 17 == 8 && slides < 3 {
            slide = Some(Slide {
                reach: r.scale(int(5)),
                period: crate::rules::secs_to_ticks(450 + slides * 70),
            });
            slides += 1;
        }
        c.vines.push(Vine {
            anchor: v3(base.x, ground_y + anchor_h, base.z),
            length,
            slide,
            at: int(s),
        });
        // a second choice at some stations: higher or lower, on the other side
        if station > 0 && dice.below(3) == 0 {
            let off2 = -side_sign * dice.range(4, 12);
            let b2 = c.point(int(s + dice.range(-4, 4))).add(r.scale(int(off2)));
            let g2 = ground(b2.x, b2.z);
            c.vines.push(Vine {
                anchor: v3(b2.x, g2 + int(dice.range(20, 34)), b2.z),
                length: int(dice.range(9, 16)),
                slide: None,
                at: int(s),
            });
        }
        // a low vine to climb back up by
        if station % 3 == 1 {
            let off3 = dice.range(-8, 8);
            let b3 = c.point(int(s + 8)).add(r.scale(int(off3)));
            let g3 = ground(b3.x, b3.z);
            c.vines.push(Vine {
                anchor: v3(b3.x, g3 + int(13), b3.z),
                length: int(10) + cm(dice.range(0, 40)),
                slide: None,
                at: int(s + 8),
            });
        }
        if dice.below(4) != 0 {
            side_sign = -side_sign;
        }
        station += 1;
        s += if dice.below(5) == 0 {
            dice.range(34, 38)
        } else {
            dice.range(24, 31)
        };
    }

    // two swinging logs, across the course in the racing line
    for (k, at) in [len_m * 30 / 100, len_m * 72 / 100].into_iter().enumerate() {
        let h = c.heading_at(int(at));
        let base = c.point(int(at));
        let gy = ground(base.x, base.z);
        c.logs.push(Log {
            pivot: v3(base.x, gy + int(36), base.z),
            rope: int(20),
            across: right_of(h),
            along: h,
            half_len: int(4),
            radius: cm(90),
            amplitude: deg(55),
            period: crate::rules::secs_to_ticks(380 + k as i32 * 40),
        });
    }

    // item crates: rows of three across the course
    let mut at = 150;
    while at < len_m - 60 {
        let h = c.heading_at(int(at));
        let r = right_of(h);
        let base = c.point(int(at));
        let gy = ground(base.x, base.z);
        for off in [-9, 0, 9] {
            let p = base.add(r.scale(int(off)));
            c.crates.push(v3(p.x, gy + int(dice.range(13, 17)), p.z));
        }
        at += 240;
    }

    scenery(&mut c, &mut dice);
    c
}

/// Trunks along both walls and deeper in, branches to the vines, and the
/// undergrowth. None of it hangs where an ape swings.
fn scenery(c: &mut Course, dice: &mut Dice) {
    let n = c.path.len();
    let hw = c.half_width;
    let mut i = 0;
    while i < n {
        let p = c.path[i];
        let r = right_of(c.heading[i]);
        for sign in [-1, 1] {
            for row in 0..3 {
                let off = hw + int(2) + int(row * 7) + int(dice.range(0, 4));
                let q = p.add(r.scale(off * sign));
                let q = v3(
                    q.x + int(dice.range(-2, 2)),
                    0,
                    q.z + int(dice.range(-2, 2)),
                );
                let gy = ground(q.x, q.z);
                c.props.push(Prop::Tree {
                    at: v3(q.x, gy, q.z),
                    radius: cm(dice.range(70, 160)) + row * cm(20),
                    height: int(dice.range(44, 58)),
                    shade: dice.below(4) as u8,
                });
            }
        }
        i += (dice.range(9, 13) / 2).max(1) as usize * 2;
    }
    // a branch from the nearer wall to each vine's anchor
    for v in &c.vines {
        let k = c.nearest_anywhere(v.anchor);
        let r = right_of(c.heading[k]);
        let side = v.anchor.flat().sub(c.path[k]).dot(r);
        let sign = if side >= 0 { 1 } else { -1 };
        let reach = match v.slide {
            Some(s) => s.reach.len() + int(3),
            None => int(3),
        };
        let wall = c.path[k].add(r.scale((hw + int(3)) * sign));
        let from = v3(wall.x, v.anchor.y + int(2), wall.z);
        let beyond = v.anchor.add(r.scale(-reach * sign));
        c.props.push(Prop::Branch {
            from,
            to: v3(beyond.x, v.anchor.y, beyond.z),
        });
    }
    // undergrowth on the floor: low, and thinner in the middle
    let mut s = 4;
    while s < c.length / ONE {
        let h = c.heading_at(int(s));
        let r = right_of(h);
        let base = c.point(int(s));
        for _ in 0..3 {
            let off = int(dice.range(-19, 19));
            let p = base.add(r.scale(off)).add(h.scale(int(dice.range(-2, 2))));
            let gy = ground(p.x, p.z);
            let at = v3(p.x, gy, p.z);
            match dice.below(9) {
                0..=2 => c.props.push(Prop::Fern {
                    at,
                    size: cm(dice.range(80, 160)),
                    yaw: dice.range(0, fx::TURN),
                }),
                3..=4 => c.props.push(Prop::Bush {
                    at,
                    size: cm(dice.range(90, 200)),
                    shade: dice.below(3) as u8,
                }),
                5..=7 => c.props.push(Prop::Flower {
                    at,
                    colour: dice.below(4) as u8,
                }),
                _ => c.props.push(Prop::Rock {
                    at,
                    size: cm(dice.range(50, 140)),
                    yaw: dice.range(0, fx::TURN),
                }),
            }
        }
        s += 5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_jungle_turns_and_does_not_come_back() {
        let c = Course::jungle();
        assert_eq!(c.length, int(1800));
        // total turning, both ways
        let mut turned = 0i64;
        for w in c.heading.windows(2) {
            turned += fx::wrap(crate::geom::yaw_of(w[1]) - crate::geom::yaw_of(w[0])).abs() as i64;
        }
        assert!(turned > deg(60) as i64 * 3, "{turned}");
        // the finish is far from the start
        let end = *c.path.last().unwrap();
        assert!(end.dist(c.path[0]) > int(1000));
        // and the centreline never comes back near an earlier part
        for i in (0..c.path.len()).step_by(10) {
            for j in (i + 60..c.path.len()).step_by(10) {
                assert!(c.path[i].dist(c.path[j]) > int(60), "{i} {j}");
            }
        }
    }

    #[test]
    fn vines_are_sparse_but_never_too_far_apart() {
        let c = Course::jungle();
        let mut ats: Vec<i32> = c.vines.iter().map(|v| v.at).collect();
        ats.sort();
        for w in ats.windows(2) {
            assert!(w[1] - w[0] <= int(40), "gap {}", (w[1] - w[0]) / ONE);
        }
        let first = ats[0];
        assert!(first <= int(40));
        let last = *ats.last().unwrap();
        assert!(c.length - last <= int(50));
        // vines hang at different heights
        let lo = c.vines.iter().map(|v| v.anchor.y - v.length).min().unwrap();
        let hi = c.vines.iter().map(|v| v.anchor.y - v.length).max().unwrap();
        assert!(hi - lo >= int(6), "{} {}", lo / ONE, hi / ONE);
        // and they are not all in a line: many sit well off the centreline
        let off = c
            .vines
            .iter()
            .filter(|v| {
                let k = c.nearest_anywhere(v.anchor);
                c.progress(v.anchor, k).1.abs() > int(5)
            })
            .count();
        assert!(off * 2 > c.vines.len(), "{off} of {}", c.vines.len());
    }

    #[test]
    fn some_vines_can_be_reached_from_the_ground() {
        let c = Course::jungle();
        let low = c
            .vines
            .iter()
            .filter(|v| v.anchor.y - v.length - ground(v.anchor.x, v.anchor.z) < int(4))
            .count();
        assert!(low >= 15, "{low}");
    }

    #[test]
    fn no_more_than_five_moving_parts() {
        let c = Course::jungle();
        let sliding = c.vines.iter().filter(|v| v.slide.is_some()).count();
        assert!(sliding + c.logs.len() <= 5);
        assert!(sliding + c.logs.len() >= 3);
    }

    #[test]
    fn progress_follows_the_centreline() {
        let c = Course::jungle();
        let mut hint = 0;
        for s in (0..1800).step_by(50) {
            let p = c
                .point(int(s))
                .add(right_of(c.heading_at(int(s))).scale(int(5)));
            hint = c.nearest(p, hint);
            let (along, side) = c.progress(p, hint);
            assert!((along - int(s)).abs() < int(2), "{s}: {}", along / ONE);
            assert!((side - int(5)).abs() < int(1), "{s}: {}", side / ONE);
        }
    }

    #[test]
    fn the_walls_push_back() {
        let c = Course::jungle();
        let p = c
            .point(int(100))
            .add(right_of(c.heading_at(int(100))).scale(int(25)));
        let k = c.nearest_anywhere(p);
        let (q, v, hit) = c.contain(p, right_of(c.heading_at(int(100))).scale(int(10)), k);
        assert!(hit);
        assert!(c.progress(q, k).1 < c.half_width);
        assert!(v.dot(right_of(c.heading_at(int(100)))) < 0);
    }

    #[test]
    fn the_start_platform_holds_the_starters() {
        let c = Course::jungle();
        for s in c.starts {
            assert_eq!(c.support(s), START_HEIGHT);
        }
        // and a point beside it falls to the ground
        let below = v3(c.starts[0].x, int(5), c.starts[0].z);
        assert!(c.support(below) < int(2));
    }

    #[test]
    fn sliding_vines_and_logs_move() {
        let c = Course::jungle();
        let i = c.vines.iter().position(|v| v.slide.is_some()).unwrap();
        assert_ne!(c.anchor(i, 0), c.anchor(i, 200));
        assert_ne!(c.log_centre(0, 0), c.log_centre(0, 100));
        // a log's middle stays a rope's length from its pivot
        let l = &c.logs[0];
        assert!((c.log_centre(0, 77).dist(l.pivot) - l.rope).abs() < int(1) / 10);
    }
}
