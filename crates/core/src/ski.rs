//! The Ski Slope map (2026-10-13): a ski run at noon, clear and bright, not
//! snowing (a Weather setting can add falling snow; it changes nothing but
//! the drawing). The course runs downhill, but not at one steady slope:
//! pitches steeper and gentler than `SLOPE_DEG`, the mean, which is the one
//! number to change for a steeper or gentler hill.
//!
//! - The vines are cables hanging from the chairs of chairlifts, from
//!   conifers' branches, and from one or two helicopters.
//! - A chairlift is one straight line (its towers, its haul rope, two
//!   terminals); lifts never branch. They chain: one lift's bottom terminal
//!   stands near the next one's top terminal.
//! - The course forks twice: it splits round an island and joins again, then
//!   later splits and joins again before the finish (`Fork`).
//! - Orange plastic ski fences bound the sides. Nothing bounces on this map
//!   (`Course.mushrooms` is empty).
//! - One-of-a-kind landmarks mark the way, so players feel their progress and
//!   remember the turns that come next (`Landmark`).
//! - On this map the apes wear skis, a ski helmet and goggles.
//!
//! The types below are the contract between the course (placement, physics)
//! and the scene (drawing): the course fills a `SkiLayout` on `Course.ski`;
//! the scene draws from it. The hill itself is `Terrain::Slope`, a pure
//! function of (x, z), so the camera and the models can use it without the
//! course.

use crate::course::{centreline, Course, Leg, Platform, Vine, START_HEIGHT, START_LEN};
use crate::fx::{self, deg, int, mul, Dice, ONE};
use crate::geom::{right_of, v3, V3};
use crate::rules::BODY_DROP;

/// The mean slope of the hill, in degrees (the person: 5–10°, to be
/// adjusted). Each pitch in `PITCHES` is a share of it.
pub const SLOPE_DEG: i32 = 7;

/// The hill's profile down its fall line: (metres, the slope there as a
/// percentage of `SLOPE_DEG`). Between two pitches the slope changes
/// smoothly over `BLEND` metres, so there are no kinks. Past the last
/// pitch the hill keeps that pitch's slope.
pub const PITCHES: &[(i32, i32)] = &[
    (90, 60),
    (220, 130),
    (140, 55),
    (240, 140),
    (160, 70),
    (220, 160),
    (180, 60),
    (260, 120),
    (200, 90),
    (400, 25),
];

/// Metres over which one pitch's slope blends into the next.
pub const BLEND: i32 = 40;

/// How the ground's height is found: the Jungle's and the City's gentle
/// hills (`course::ground`), or the ski hill.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Terrain {
    #[default]
    Hills,
    /// falling along `fall` (a unit vector on the ground) from `top`, a
    /// point on the hill at height 0: the course's start
    Slope { top: V3, fall: V3 },
}

impl Terrain {
    /// The ground's height at (`x`, `z`).
    pub fn height(self, x: i32, z: i32) -> i32 {
        match self {
            Terrain::Hills => crate::course::ground(x, z),
            Terrain::Slope { top, fall } => {
                let u = v3(x, 0, z).sub(top.flat()).dot(fall);
                drop_at(u)
            }
        }
    }
}

/// The hill's height `u` metres (Q16) down the fall line from the top
/// (negative: below the top). Before the top the first pitch goes on up.
/// The grade is piecewise linear in `u` (each pitch's grade, blended into
/// the next over `BLEND` metres centred on their boundary), so the height
/// is its exact integral, a handful of pieces a call.
pub fn drop_at(u: i32) -> i32 {
    let mean = fx::div(fx::sin(deg(SLOPE_DEG)), fx::cos(deg(SLOPE_DEG)));
    let grade = |pct: i32| mul(mean, pct * ONE / 100) as i64;
    if u <= 0 {
        return -((u as i64 * grade(PITCHES[0].1)) >> 16) as i32;
    }
    // knots of the grade: (metres Q16, grade Q16)
    let mut h: i64 = 0;
    let (mut x0, mut g0) = (0i64, grade(PITCHES[0].1));
    let mut boundary = 0i64;
    let u = u as i64;
    let half = (BLEND as i64 * ONE as i64) / 2;
    let mut knots = [(0i64, 0i64); 2 * 16];
    let mut n = 0;
    for k in 0..PITCHES.len() - 1 {
        boundary += PITCHES[k].0 as i64 * ONE as i64;
        knots[n] = (boundary - half, grade(PITCHES[k].1));
        knots[n + 1] = (boundary + half, grade(PITCHES[k + 1].1));
        n += 2;
    }
    for &(x1, g1) in &knots[..n] {
        if u <= x1 {
            // part of this piece: the grade at u by linear interpolation
            let gu = g0 + (g1 - g0) * (u - x0) / (x1 - x0).max(1);
            h += (g0 + gu) * (u - x0) / 2;
            return -((h >> 16) as i32);
        }
        h += (g0 + g1) * (x1 - x0) / 2;
        (x0, g0) = (x1, g1);
    }
    h += g0 * (u - x0);
    -((h >> 16) as i32)
}

/// Everything the Ski Slope has beyond the shared course data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SkiLayout {
    /// the chairlifts, top of the hill first; each straight, none branching
    pub lifts: Vec<Lift>,
    /// conifers inside the fences (and on the forks' islands) that carry a
    /// vine or stand near the course; the forest beyond the fences is the
    /// scene's own
    pub trees: Vec<Conifer>,
    /// what each vine hangs from (one per vine)
    pub hangers: Vec<Hanger>,
    /// where the course splits round an island and joins again
    pub forks: Vec<Fork>,
    /// the one-of-a-kind landmarks along the way
    pub landmarks: Vec<Landmark>,
}

/// A chairlift: one straight line from its bottom terminal up to its top
/// terminal (both on the ground), with towers between carrying the haul
/// rope `rope_height` above the ground at each tower. Chairs hang from the
/// rope every `chair_every` metres on both sides of the line (up and down),
/// `gauge` metres apart. The lift is stopped (no moving chairs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lift {
    pub bottom: V3,
    pub top: V3,
    /// the towers' feet, bottom first
    pub towers: Vec<V3>,
    pub rope_height: i32,
    pub gauge: i32,
    pub chair_every: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conifer {
    /// the foot of its trunk on the ground
    pub at: V3,
    pub height: i32,
    /// the widest skirt of its branches
    pub radius: i32,
    pub shade: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HangerKind {
    /// from a chair of lift `lift`: `from` the chair's grip on the rope,
    /// `to` the chair's seat (the vine hangs from the seat)
    Chair { lift: usize },
    /// from a conifer's branch: `from` the trunk where the branch leaves
    /// it, `to` the branch's tip (the vine hangs from the tip); `tree`
    /// indexes `SkiLayout.trees`
    Tree { tree: usize },
    /// hovering: `from` is the rotor's hub, `to` where the cable leaves
    /// the helicopter's belly
    Helicopter,
    /// a cable strung from one thing to another (a lift tower to a tree,
    /// across a landmark): `from` and `to` its ends
    Wire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hanger {
    /// the vine it carries (`Course.vines[vine]`)
    pub vine: usize,
    pub kind: HangerKind,
    pub from: V3,
    pub to: V3,
}

/// The course splits round an island between `from` and `to` metres along
/// its centreline (the spine): the left branch runs `left` metres left of
/// the spine and the right branch `right` metres right of it (both
/// positive), each `branch_half` metres either side of its own line. Over
/// `ramp` metres at each end the branches' lines ease in from the spine,
/// so the corridor opens into a Y and closes again. The island between the
/// branches is out of bounds (fenced, with trees on it). Progress is still
/// measured along the spine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fork {
    pub from: i32,
    pub to: i32,
    pub left: i32,
    pub right: i32,
    pub branch_half: i32,
    pub ramp: i32,
}

impl Fork {
    /// How far the branches have opened at `s` metres along: 0 at the
    /// ends, ONE between the ramps (smooth in between).
    pub fn open(&self, s: i32) -> i32 {
        if s <= self.from || s >= self.to {
            return 0;
        }
        let t = (s - self.from).min(self.to - s);
        if t >= self.ramp {
            return ONE;
        }
        let x = fx::div(t, self.ramp);
        // smoothstep
        mul(mul(x, x), 3 * ONE - 2 * x)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LandmarkKind {
    /// the start: a summit lift station and weather mast at the top
    Summit,
    /// a giant snowman, at the first fork's island
    Snowman,
    /// a timber lodge with a smoking chimney, between the forks
    Lodge,
    /// a ski-jump tower and its in-run, beside the course
    SkiJump,
    /// a giant ice sculpture of an ape, at the second fork's island
    IceApe,
    /// a frozen waterfall down a rock face beside the course
    FrozenFalls,
    /// the base village and finish: chalets, flags, the podium
    Village,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Landmark {
    pub kind: LandmarkKind,
    /// its middle on the ground
    pub at: V3,
    /// the way it faces (`fx` angle)
    pub yaw: i32,
    /// where along the course it is seen best, for the minimap and tests
    pub s: i32,
}

impl Course {
    /// The corridor across the course at `s` metres along: one or two
    /// lanes, each (its centre's offset right of the spine, its half
    /// width). Outside a fork both are the whole corridor.
    pub fn lanes(&self, s: i32) -> [(i32, i32); 2] {
        let whole = (0, self.half_width);
        let Some(ski) = self.ski.as_ref() else {
            return [whole, whole];
        };
        for f in &ski.forks {
            let k = f.open(s);
            if k > 0 {
                // the lanes start as the whole corridor and narrow as they
                // move out
                let half = fx::lerp(self.half_width, f.branch_half, k);
                return [(-mul(f.left, k), half), (mul(f.right, k), half)];
            }
        }
        [whole, whole]
    }
}

const SEED: u64 = 0x5C1_0001;

/// The stand-in layout's legs (the course agent replaces it): gentle bends
/// across the fall line.
const LEGS: &[Leg] = &[
    Leg(60, 0),
    Leg(140, 0),
    Leg(120, 30),
    Leg(140, 0),
    Leg(160, -50),
    Leg(120, 0),
    Leg(140, 35),
    Leg(160, 0),
    Leg(120, -30),
    Leg(160, 0),
    Leg(140, 25),
    Leg(140, 0),
];

/// The Ski Slope course.
pub fn ski() -> Course {
    ski_seeded(SEED)
}

/// The Ski Slope laid out from another seed (for the AI's tests).
pub fn ski_seeded(seed: u64) -> Course {
    let mut c = centreline("ski", LEGS);
    let fall = c.heading_at(0);
    c.terrain = Terrain::Slope {
        top: c.path[0],
        fall,
    };
    let mut dice = Dice(seed);
    let mut lay = SkiLayout::default();
    let len_m = c.length / ONE;

    // a stand-in: the Jungle's start, stations and low vines on the slope
    let p0 = c.point(START_LEN / 2 - int(2));
    let fwd = c.heading_at(0);
    let right = right_of(fwd);
    let deck = c.terrain(p0.x, p0.z) + START_HEIGHT;
    c.platforms.push(Platform {
        centre: v3(p0.x, deck, p0.z),
        fwd,
        half_len: START_LEN / 2 + int(2),
        half_width: int(9),
        top: deck,
    });
    for k in 0..4 {
        let at = c.point(int(2)).add(right.scale(int(-6) + k * int(4)));
        c.starts[k as usize] = v3(at.x, deck + BODY_DROP, at.z);
    }
    let edge = START_LEN + fx::cm(150);
    for k in -3..=3 {
        let base = c.point(edge).add(right.scale(int(3) * k));
        c.vines.push(Vine {
            anchor: v3(base.x, deck + int(30), base.z),
            length: int(29),
            slide: None,
            at: edge,
        });
    }
    let mut s = 32;
    let mut station = 0;
    while s < len_m - 30 {
        let r = right_of(c.heading_at(int(s)));
        let skip = dice.below(3) as i32;
        for lane in 0..3 {
            if lane == skip {
                continue;
            }
            let along = s + dice.range(-3, 3);
            let base = c
                .point(int(along))
                .add(r.scale(int((lane - 1) * 10 + dice.range(-2, 2))));
            let g = c.terrain(base.x, base.z);
            c.vines.push(Vine {
                anchor: v3(base.x, g + int(dice.range(38, 50)), base.z),
                length: int(dice.range(23, 31)),
                slide: None,
                at: int(along),
            });
        }
        if station % 2 == 1 {
            let b = c.point(int(s + 7)).add(r.scale(int(dice.range(-9, 9))));
            let g = c.terrain(b.x, b.z);
            c.vines.push(Vine {
                anchor: v3(b.x, g + int(27), b.z),
                length: int(24),
                slide: None,
                at: int(s + 7),
            });
        }
        station += 1;
        s += dice.range(22, 28);
    }
    for i in 0..c.vines.len() {
        let a = c.vines[i].anchor;
        lay.hangers.push(Hanger {
            vine: i,
            kind: HangerKind::Helicopter,
            from: a.add(v3(0, int(3), 0)),
            to: a,
        });
    }
    c.ski = Some(Box::new(lay));
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hill_falls_along_the_course_at_about_the_mean_slope() {
        let c = ski();
        let h0 = c.terrain(c.path[0].x, c.path[0].z);
        assert_eq!(h0, 0);
        let end = c.point(c.length);
        let h1 = c.terrain(end.x, end.z);
        assert!(h1 < -int(100), "the hill drops {}", h1 / ONE);
        // never uphill along the fall line
        let mut last = drop_at(0);
        for m in 1..2000 {
            let h = drop_at(int(m));
            assert!(h <= last, "uphill at {m} m");
            last = h;
        }
    }

    #[test]
    fn the_slope_changes_from_pitch_to_pitch() {
        let g = |m: i32| drop_at(int(m)) - drop_at(int(m + 1));
        assert!(g(40) * 2 < g(200), "a steep pitch after a gentle one");
        assert!(g(200) > g(380) * 2, "a bench after a steep pitch");
    }

    #[test]
    fn a_fork_opens_and_closes_smoothly() {
        let f = Fork {
            from: int(100),
            to: int(300),
            left: int(30),
            right: int(30),
            branch_half: int(14),
            ramp: int(50),
        };
        assert_eq!(f.open(int(100)), 0);
        assert_eq!(f.open(int(200)), ONE);
        assert_eq!(f.open(int(300)), 0);
        assert!(f.open(int(110)) < f.open(int(140)));
    }
}
