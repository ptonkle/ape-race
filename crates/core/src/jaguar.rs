//! Floor is Lava (TRIAGE T37): touch the ground and a jaguar eats you.
//!
//! An eaten ape stops where it touched down. A jaguar runs in from the
//! trees beside the course ([`RUN`]), leaps on it ([`LEAP`], with a roar),
//! and the ape is gone at the bite ([`BITE`]). The jaguar eats a moment,
//! then trots back into the trees and is gone at [`GONE`]. Everything is a
//! function of the [`Eaten`] record and the race tick, so a replay, a seek
//! and every browser of an online race draw the same jaguar.

use crate::course::Course;
use crate::fx::{self, cm, int, mul, ONE};
use crate::geom::{facing, right_of, v3, yaw_of, V3};
use crate::rules::{BODY_DROP, TICKS_PER_SECOND};

const fn secs(hundredths: u32) -> u32 {
    hundredths * TICKS_PER_SECOND as u32 / 100
}

/// The run in from the trees, ticks after the ape touched the ground.
pub const RUN: u32 = secs(55);
/// The leap onto the ape (the roar is at its start).
pub const LEAP: u32 = secs(20);
/// The bite: the ape is gone from here on.
pub const BITE: u32 = RUN + LEAP;
/// Eating, head down, until here: the eating is over.
pub const EATEN: u32 = BITE + secs(45);
/// Then a trot back into the trees; the jaguar is gone at this tick.
pub const GONE: u32 = EATEN + secs(80);

/// Where the jaguar waits: at the course's edge (this far inside it, so the
/// way to the ape is open on any map: the Jungle's trees or the City's
/// walls stand beyond the edge) on the side the ape is on ...
const FROM_EDGE: i32 = int(1);
/// ... and this far ahead of it along the centreline, so the chase camera
/// behind the ape sees it come.
const FROM_AHEAD: i32 = int(18);
/// The leap starts this far short of the ape.
const LEAP_FROM: i32 = int(5);
/// How high the leap goes.
const LEAP_HIGH: i32 = cm(180);

/// One racer's end under the jaguar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eaten {
    /// the race tick it touched the ground
    pub tick: u32,
    /// where it stopped (the ape's hands, as `Ape::pos`)
    pub at: V3,
    /// where the jaguar comes from, in the trees
    pub from: V3,
}

impl Eaten {
    /// The ape at `at` (its hands), near centreline sample `hint`, eaten at
    /// race tick `tick`: the jaguar comes from the edge of the course on the
    /// side the ape is on (the right on the centreline), a little ahead.
    /// Only the shared course data is used (centreline, half width, floor),
    /// so it works on every map.
    pub fn new(course: &Course, at: V3, hint: usize, tick: u32) -> Eaten {
        let hint = hint.min(course.heading.len().saturating_sub(1));
        let (s, side) = course.progress(at, hint);
        let ahead = (s + FROM_AHEAD).min(course.length);
        let heading = course.heading_at(ahead);
        let right = right_of(heading);
        let way = if side >= 0 { right } else { right.neg() };
        let edge = (course.half_width - FROM_EDGE).max(0);
        let p = course.point(ahead).add(way.scale(edge));
        Eaten {
            tick,
            at,
            from: v3(p.x, course.floor(p.x, p.z), p.z),
        }
    }

    /// Ticks since it touched the ground, at race tick `tick`.
    pub fn ago(&self, tick: u32) -> u32 {
        tick.saturating_sub(self.tick)
    }

    /// Whether the ape is still there (drawn) at race tick `tick`.
    pub fn whole(&self, tick: u32) -> bool {
        self.ago(tick) < BITE
    }

    /// Where the ape's feet are: the jaguar's goal.
    pub fn feet(&self, course: &Course) -> V3 {
        let f = self.at.sub(v3(0, BODY_DROP, 0));
        v3(f.x, course.floor(f.x, f.z).max(f.y), f.z)
    }

    /// The jaguar at race tick `tick` (`None` once it is gone). `blend` is
    /// the share of the way to the next tick (Q16), for smooth drawing.
    pub fn jaguar(&self, course: &Course, tick: u32, blend: i32) -> Option<Jaguar> {
        let ago = self.ago(tick);
        if ago >= GONE {
            return None;
        }
        // in Q16 ticks
        let t = ago as i32 * ONE + blend.clamp(0, ONE - 1);
        let ticks = |n: u32| n as i32 * ONE;
        let goal = self.feet(course);
        let to_goal = goal.sub(self.from).flat();
        let way = if to_goal.len() > ONE {
            to_goal.norm()
        } else {
            v3(0, 0, ONE)
        };
        let leap_at = goal.sub(way.scale(LEAP_FROM));
        let on_ground = |p: V3| v3(p.x, course.floor(p.x, p.z), p.z);
        let toward = yaw_of(way);
        let j = if t < ticks(RUN) {
            // the run: quick from the start, easing a little into the crouch
            let s = fx::div(t, ticks(RUN));
            let s = s + mul(s, mul(ONE - s, ONE / 4));
            Jaguar {
                feet: on_ground(self.from.lerp(leap_at, s.min(ONE))),
                yaw: toward,
                gait: gait(ago, 18),
                stride: ONE,
                leap: 0,
                head_down: 0,
                jaw: 0,
            }
        } else if t < ticks(BITE) {
            // the leap: an arc onto the ape, mouth open
            let s = fx::div(t - ticks(RUN), ticks(LEAP));
            let p = on_ground(leap_at.lerp(goal, s));
            let up = mul(4 * LEAP_HIGH, mul(s, ONE - s));
            Jaguar {
                feet: p.add(v3(0, up, 0)),
                yaw: toward,
                gait: 0,
                stride: 0,
                leap: fx::sin(mul(s, fx::HALF)).max(ONE / 3),
                head_down: 0,
                jaw: ONE,
            }
        } else if t < ticks(EATEN) {
            // eating, head down, the jaw working
            let chew = fx::sin(gait(ago, 20)).abs();
            Jaguar {
                feet: on_ground(goal),
                yaw: toward,
                gait: 0,
                stride: 0,
                leap: 0,
                head_down: ONE,
                jaw: chew / 2,
            }
        } else {
            // back into the trees, at a trot
            let s = fx::div(t - ticks(EATEN), ticks(GONE - EATEN));
            Jaguar {
                feet: on_ground(goal.lerp(self.from, s)),
                yaw: fx::wrap(toward + fx::TURN / 2),
                gait: gait(ago, 12),
                stride: ONE * 2 / 3,
                leap: 0,
                head_down: 0,
                jaw: 0,
            }
        };
        Some(j)
    }
}

/// A cycle's angle at `ago` ticks, turning `per_tick` degrees each tick.
fn gait(ago: u32, per_tick: i32) -> i32 {
    fx::wrap(ago as i32 * fx::deg(per_tick))
}

/// How the jaguar is posed: everything its model needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Jaguar {
    /// the ground under its middle
    pub feet: V3,
    /// the way it faces
    pub yaw: i32,
    /// the run cycle's angle, and the stride (0 to ONE)
    pub gait: i32,
    pub stride: i32,
    /// the leap's stretch, 0 to ONE: forelegs reaching, hind legs back
    pub leap: i32,
    /// 0 to ONE: the head down to the ground, eating
    pub head_down: i32,
    /// 0 to ONE: the jaw open
    pub jaw: i32,
}

impl Jaguar {
    /// The way it faces, as a unit vector on the ground.
    pub fn fwd(&self) -> V3 {
        facing(self.yaw, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eaten() -> (Course, Eaten) {
        eaten_on(Course::test_line(&[]))
    }

    fn eaten_on(c: Course) -> (Course, Eaten) {
        let p = c.point(int(100));
        let at = v3(p.x + int(3), c.floor(p.x, p.z) + BODY_DROP, p.z);
        let hint = c.nearest_anywhere(at);
        let e = Eaten::new(&c, at, hint, 1000);
        (c, e)
    }

    #[test]
    fn the_jaguar_comes_from_the_trees_leaps_and_leaves() {
        let (c, e) = eaten();
        // from the course's edge on the side the ape is on, ahead of it:
        // inside the course, so its way in is open on any map
        for (c, e) in [eaten(), {
            let mut narrow = Course::test_line(&[]);
            narrow.half_width = int(7);
            eaten_on(narrow)
        }] {
            let (s, side) = c.progress(e.from, c.nearest_anywhere(e.from));
            let (ape_s, ape_side) = c.progress(e.at, c.nearest_anywhere(e.at));
            assert!(side.abs() <= c.half_width, "{} m", side / ONE);
            assert!(side.abs() >= c.half_width - int(2), "{} m", side / ONE);
            assert_eq!(side.signum(), ape_side.signum());
            assert!(s > ape_s + int(10), "ahead, where the camera looks");
            assert!(e.from.dist(e.at) > int(10), "a run, not a pop");
        }
        // it runs in: nearer the ape all the way to the bite
        let mut last = i32::MAX;
        for k in (0..BITE).step_by(10) {
            let j = e.jaguar(&c, e.tick + k, 0).expect("there");
            let d = j.feet.flat().sub(e.feet(&c).flat()).len();
            assert!(d <= last, "at {k}: {d} after {last}");
            last = d;
        }
        // in the air mid-leap, on the ape at the bite
        let mid = e.jaguar(&c, e.tick + RUN + LEAP / 2, 0).unwrap();
        assert!(mid.feet.y > e.feet(&c).y + cm(100));
        let bite = e.jaguar(&c, e.tick + BITE, 0).unwrap();
        assert!(bite.feet.flat().sub(e.feet(&c).flat()).len() < cm(5));
        assert!(e.whole(e.tick + BITE - 1) && !e.whole(e.tick + BITE));
        // and back into the trees, gone
        let late = e.jaguar(&c, e.tick + GONE - 1, 0).unwrap();
        assert!(late.feet.flat().sub(e.from.flat()).len() < int(1));
        assert_eq!(e.jaguar(&c, e.tick + GONE, 0), None);
        // the whole eating is about a second (the person, 2026-10-12:
        // twice as fast as the first two seconds)
        assert!((secs(75)..=secs(125)).contains(&EATEN));
    }
}
