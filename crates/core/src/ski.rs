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

use crate::course::Course;
use crate::fx::{self, cm, mul, ONE};
use crate::geom::{right_of, v3, V3};

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
    /// Sector Ape: no floor, only a level "ground" this far down (far
    /// below the boundary, so nothing ever stands on it)
    Flat(i32),
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
            Terrain::Flat(h) => h,
        }
    }

    /// Which way is downhill at (`x`, `z`), and how steeply: a flat vector
    /// along the fall whose length is the grade (metres of drop a metre);
    /// zero on the Hills (they are gentle: no ape slides on them).
    pub fn downhill(self, x: i32, z: i32) -> V3 {
        match self {
            Terrain::Hills | Terrain::Flat(_) => V3::ZERO,
            Terrain::Slope { top, fall } => {
                let u = v3(x, 0, z).sub(top.flat()).dot(fall);
                fall.scale(grade_at(u))
            }
        }
    }

    /// The ground's unit normal at (`x`, `z`) (straight up on the Hills).
    pub fn normal(self, x: i32, z: i32) -> V3 {
        let d = self.downhill(x, z);
        v3(d.x, ONE, d.z).norm()
    }
}

/// tan(d°) in Q16, for whole degrees 0 to 15 (`SLOPE_DEG`'s range).
const TAN_DEG: [i64; 16] = [
    0, 1144, 2289, 3435, 4583, 5734, 6888, 8047, 9210, 10380, 11556, 12739, 13930, 15130, 16340,
    17560,
];

/// The pitches' grade, as knots of a piecewise linear function of the
/// distance down the fall line: (metres Q16, grade Q16, the drop from the
/// top to there Q16), worked out once at compile time.
const KNOTS: usize = 2 * (PITCHES.len() - 1) + 1;
const HILL: [(i64, i64, i64); KNOTS] = hill();

const fn pitch_grade(pct: i32) -> i64 {
    TAN_DEG[SLOPE_DEG as usize] * pct as i64 / 100
}

const fn hill() -> [(i64, i64, i64); KNOTS] {
    let mut out = [(0i64, 0i64, 0i64); KNOTS];
    out[0] = (0, pitch_grade(PITCHES[0].1), 0);
    let half = (BLEND as i64 * ONE as i64) / 2;
    let mut boundary = 0i64;
    let mut n = 1;
    let mut k = 0;
    while k < PITCHES.len() - 1 {
        boundary += PITCHES[k].0 as i64 * ONE as i64;
        out[n] = (boundary - half, pitch_grade(PITCHES[k].1), 0);
        out[n + 1] = (boundary + half, pitch_grade(PITCHES[k + 1].1), 0);
        n += 2;
        k += 1;
    }
    // the drop at each knot: the exact integral of the grade
    let mut i = 1;
    while i < KNOTS {
        let (x0, g0, h0) = out[i - 1];
        let (x1, g1, _) = out[i];
        out[i].2 = h0 + (g0 + g1) * (x1 - x0) / 2;
        i += 1;
    }
    out
}

/// The knot at or before `u` (Q16 metres down the fall line, not negative).
fn knot(u: i64) -> usize {
    let mut k = 0;
    while k + 1 < KNOTS && HILL[k + 1].0 <= u {
        k += 1;
    }
    k
}

/// The grade of the hill `u` metres (Q16) down the fall line from the top
/// (drop per metre, Q16): each pitch's grade, blended into the next over
/// `BLEND` metres. `drop_at` is its integral.
pub fn grade_at(u: i32) -> i32 {
    let u = (u as i64).max(0);
    let k = knot(u);
    let (x0, g0, _) = HILL[k];
    match HILL.get(k + 1) {
        Some(&(x1, g1, _)) => (g0 + (g1 - g0) * (u - x0) / (x1 - x0).max(1)) as i32,
        None => g0 as i32,
    }
}

/// The hill's height `u` metres (Q16) down the fall line from the top
/// (negative: below the top). Before the top the first pitch goes on up.
/// The grade is piecewise linear in `u` (each pitch's grade, blended into
/// the next over `BLEND` metres centred on their boundary), so the height
/// is its exact integral (`HILL`, worked out once).
pub fn drop_at(u: i32) -> i32 {
    let u = u as i64;
    if u <= 0 {
        return -((u * HILL[0].1) >> 16) as i32;
    }
    let k = knot(u);
    let (x0, g0, h0) = HILL[k];
    let gu = grade_at(u as i32) as i64;
    -((h0 + (g0 + gu) * (u - x0) / 2) >> 16) as i32
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

/// A chair's seat hangs this far below its grip on the haul rope.
pub const CHAIR_DROP: i32 = cm(250);

impl Lift {
    /// Along the line, from the bottom terminal toward the top (flat, unit).
    pub fn dir(&self) -> V3 {
        self.top.sub(self.bottom).flat().norm()
    }

    /// The line's length on the ground (flat).
    pub fn length(&self) -> i32 {
        self.top.sub(self.bottom).flat().len()
    }

    /// The haul rope `d` metres along the line from the bottom terminal,
    /// on side `side` (-1 left of the line looking uphill, 1 right): it
    /// runs straight from support to support (the two terminals and the
    /// towers, each `rope_height` above its foot), `gauge / 2` either side
    /// of the line. No sag: the chairs that carry vines sit exactly on it.
    pub fn rope(&self, d: i32, side: i32) -> V3 {
        let dir = self.dir();
        let base = self.bottom.flat();
        let mut prev = (0, self.bottom.y);
        let mut y = self.top.y;
        for p in self.towers.iter().chain(std::iter::once(&self.top)) {
            let at = p.flat().sub(base).dot(dir);
            if d <= at {
                let t = fx::div(d - prev.0, (at - prev.0).max(1));
                y = fx::lerp(prev.1, p.y, t.clamp(0, ONE));
                break;
            }
            prev = (at, p.y);
        }
        let r = right_of(dir).scale(side.signum() * self.gauge / 2);
        let p = base.add(dir.scale(d)).add(r);
        v3(p.x, y + self.rope_height, p.z)
    }

    /// How many chairs hang on each side: chair `k` (1 up to this, less
    /// one) is `k * chair_every` metres from the bottom terminal.
    pub fn chairs(&self) -> i32 {
        self.length() / self.chair_every.max(1)
    }

    /// Chair `k` on side `side`: its grip on the rope and its seat below.
    pub fn chair(&self, k: i32, side: i32) -> (V3, V3) {
        let grip = self.rope(k * self.chair_every, side);
        (grip, grip.sub(v3(0, CHAIR_DROP, 0)))
    }
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

mod layout;
pub use layout::{ski, ski_seeded};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::int;

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
