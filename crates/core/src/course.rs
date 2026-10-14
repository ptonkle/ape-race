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
use crate::rules::{
    BODY_DROP, BODY_RADIUS, CEILING, MUSHROOM_HEIGHT, MUSHROOM_RADIUS, WALL_BOUNCE,
};

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

/// A bounce mushroom on the forest floor (TRIAGE T5): an ape that lands on
/// its cap, or runs into it, is thrown back up toward the vines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mushroom {
    /// the middle of the cap's top
    pub top: V3,
    /// the cap's radius
    pub radius: i32,
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

/// The kinds of tree (TRIAGE T16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeKind {
    /// a buttressed trunk under a cluster of leaf blobs
    Broadleaf,
    /// a giant standing out of the canopy, with a flat umbrella crown
    Emergent,
    /// a thin curved trunk and a star of fronds
    Palm,
    /// a strangler fig: roots twisted round a dead host
    Fig,
    /// a stand of bamboo canes
    Bamboo,
    /// a far tree on the valley's walls, a faceted cone or blob
    Hill,
}

/// How far a tree's crown spreads from its trunk, and how far below the
/// trunk's top it hangs (the drawing keeps inside these).
pub fn crown(kind: TreeKind, height: i32) -> (i32, i32) {
    match kind {
        TreeKind::Broadleaf => {
            let k = crown_scale(height);
            (mul(int(11), k), mul(int(5), k))
        }
        TreeKind::Emergent => (mul(int(17), GROW), mul(int(5), GROW)),
        TreeKind::Palm => (int(7), int(3)),
        TreeKind::Fig => (mul(int(12), GROW), mul(int(6), GROW)),
        TreeKind::Bamboo => (int(4), height),
        TreeKind::Hill => (int(9), height * 3 / 4),
    }
}

/// How much bigger the canopy trees' crowns are than at first (TRIAGE
/// T31: much larger trees, so the crowns nearest the vines close overhead).
pub const GROW: i32 = ONE * 3 / 2;

/// A broadleaf's crown grows with the tree: half size on a young tree,
/// twice the first size on a giant (TRIAGE T31).
pub fn crown_scale(height: i32) -> i32 {
    fx::div(height, int(46)).clamp(ONE / 2, ONE * 2)
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
        kind: TreeKind,
    },
    /// a fallen trunk lying on the floor
    Fallen {
        from: V3,
        to: V3,
        radius: i32,
    },
    /// a stretch of stream, `width` wide
    Water {
        from: V3,
        to: V3,
        width: i32,
    },
    /// a rock outcrop on the valley's walls
    Outcrop {
        at: V3,
        size: i32,
        yaw: i32,
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
    /// `right_of` each heading, worked out once (the physics and the AI's
    /// copies of the ape ask for it every tick)
    pub right: Vec<V3>,
    pub length: i32,
    pub half_width: i32,
    pub laps: u32,
    pub vines: Vec<Vine>,
    pub mushrooms: Vec<Mushroom>,
    pub platforms: Vec<Platform>,
    pub crates: Vec<V3>,
    /// bananas to collect (TRIAGE T43), in arcs along good swing lines:
    /// where the middle of a body passes
    pub bananas: Vec<V3>,
    pub props: Vec<Prop>,
    /// where the four starters stand, and the way they face
    pub starts: [V3; 4],
    pub start_yaw: i32,
    /// the City's buildings, cars, river, bridge and hangers (`city.rs`);
    /// `None` on the Jungle
    pub city: Option<Box<crate::city::CityLayout>>,
    /// a course with sharp corners (the City): its streets, whose union is
    /// the corridor the walls keep an ape in. Empty on the Jungle, whose
    /// walls are `half_width` either side of the centreline.
    pub streets: Vec<Street>,
    /// the Ski Slope's lifts, trees, hangers, forks and landmarks
    /// (`ski.rs`); `None` on the other maps
    pub ski: Option<Box<crate::ski::SkiLayout>>,
    /// how the ground's height is found: `Hills` (`ground`) on the Jungle
    /// and the City, the ski hill's slope on the Ski Slope. The centreline
    /// (`path`) stays at height 0 on every map; heights come from here.
    pub terrain: crate::ski::Terrain,
    /// Sector Ape's bodies, vine directions, boundary, door, mechs and set
    /// pieces (`sector.rs`); `None` on the other maps
    pub sector: Option<Box<crate::sector::SectorLayout>>,
}

/// A straight street of a course with sharp corners (the City). The
/// corridor is the union of the streets: each is a box `half_width` either
/// side of its line from `from` (on the ground, y 0) along `dir` for `len`,
/// carried on past each end by `ext` metres plus the mitre of the corner
/// there (the half width times `tan`, the tangent of half the turn), so two
/// streets meet in a sharp corner: square at 90°, the outside wall's two
/// faces meeting in a point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Street {
    pub from: V3,
    pub dir: V3,
    pub len: i32,
    pub tan0: i32,
    pub tan1: i32,
    pub ext0: i32,
    pub ext1: i32,
}

impl Street {
    /// How far the street's box runs, along it, for a wall `w` from the
    /// line: from the first value to the second.
    pub fn span(&self, w: i32) -> (i32, i32) {
        (
            -(self.ext0 + mul(w, self.tan0)),
            self.len + self.ext1 + mul(w, self.tan1),
        )
    }
}

/// A straight (`turn` 0) or a bend of the centreline: metres, degrees.
/// A positive turn is to the left.
pub(crate) struct Leg(pub i32, pub i32);

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
pub(crate) const START_LEN: i32 = int(14);

/// The jungle goes on this far past the finish arch (TRIAGE T6): a clearing
/// with a podium, closed by a cliff and a waterfall.
pub const OUTRO: i32 = int(160);

/// Streams cross the course this many metres along.
pub const STREAMS: [i32; 3] = [int(405), int(1050), int(1700)];

/// How high the valley's walls rise above the ground, `across` metres from
/// the centreline at `s` metres along (TRIAGE T16: something beyond the
/// trees). Flat out to 45 m, then up to some 25–50 m at 120 m. Drawing
/// only: no ape gets past the corridor's walls at 20 m.
pub fn rise(s: i32, across: i32) -> i32 {
    let a = across.abs();
    let side = if across < 0 { 23_000 } else { 0 };
    // how high this stretch of wall is (60% to 140%), and a jagged ridge
    let m = ONE + mul(sin(mul(s, 280) + side), ONE * 2 / 5);
    let jag = mul(sin(mul(s, 1900) + 3 * side), int(4));
    let nodes = [
        (int(45), 0),
        (int(70), int(2)),
        (int(95), int(14)),
        (int(120), int(36) + jag),
    ];
    if a <= nodes[0].0 {
        return 0;
    }
    for w in nodes.windows(2) {
        let ((a0, h0), (a1, h1)) = (w[0], w[1]);
        if a <= a1 {
            let t = fx::div(a - a0, a1 - a0);
            return mul(h0 + mul(h1 - h0, t), m);
        }
    }
    mul(nodes[3].1 + (a - int(120)), m)
}

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

    /// The jungle course laid out from another seed: the same legs and
    /// rules, different vines (for tuning the layout rules and the AI).
    pub fn jungle_seeded(seed: u64) -> Course {
        build(JUNGLE, seed)
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
            // (Sector Ape: a vine on a mech moves with it)
            None => match &self.sector {
                Some(s) => match s.hangers.get(i) {
                    Some(crate::sector::Hanger::Mech { mech, at }) => {
                        s.mechs[*mech].point(*at, tick)
                    }
                    _ => v.anchor,
                },
                None => v.anchor,
            },
            Some(s) => {
                let phase = ((tick as i64 * fx::TURN as i64) / s.period as i64) as i32;
                v.anchor.add(s.reach.scale(sin(phase + i as i32 * 9_001)))
            }
        }
    }

    /// The unit direction vine `i` hangs from its anchor: straight down on
    /// every map but Sector Ape, whose bodies each pull their vines their
    /// own way.
    pub fn hang(&self, i: usize) -> V3 {
        match &self.sector {
            Some(s) => s.hangs.get(i).copied().unwrap_or(crate::sector::DOWN),
            None => crate::sector::DOWN,
        }
    }

    /// A point vine `i`'s anchor is never further than the second value
    /// from, at any tick: its anchor itself, unless it moves (a sliding
    /// vine, Sector Ape's mechs). For a cheap test before `anchor`.
    pub fn anchor_bound(&self, i: usize) -> (V3, i32) {
        let v = &self.vines[i];
        match v.slide {
            Some(s) => (v.anchor, s.reach.len()),
            None => match &self.sector {
                Some(s) => match s.hangers.get(i) {
                    // (its walk or drift along its path, plus its limbs' swing)
                    Some(crate::sector::Hanger::Mech { mech, .. }) => {
                        let m = &s.mechs[*mech];
                        // (along each axis: no square root, and never less)
                        let d = m.to.sub(m.from);
                        let path = d.x.abs() + d.y.abs() + d.z.abs();
                        (v.anchor, path + mul(int(16), m.scale))
                    }
                    _ => (v.anchor, 0),
                },
                None => (v.anchor, 0),
            },
        }
    }

    /// Where vine `i` ends at a tick: its anchor plus its length along the
    /// way it hangs.
    pub fn tip(&self, i: usize, tick: u32) -> V3 {
        let a = self.anchor(i, tick);
        let len = self.vines[i].length;
        match &self.sector {
            None => v3(a.x, a.y - len, a.z),
            Some(_) => a.add(self.hang(i).scale(len)),
        }
    }

    /// The lowest point of vine `i` at a tick: its end on every map but
    /// Sector Ape, where a vine hanging up is lowest at its anchor.
    pub fn vine_bottom(&self, i: usize, tick: u32) -> i32 {
        let a = self.anchor(i, tick);
        match &self.sector {
            None => a.y - self.vines[i].length,
            Some(_) => a.y.min(self.tip(i, tick).y),
        }
    }

    /// The point of vine `i` nearest `p`, at a tick: on every map but
    /// Sector Ape the vine is straight down from its anchor, so it is the
    /// anchor's column at `p`'s height, held to the vine.
    pub fn vine_point(&self, i: usize, tick: u32, p: V3) -> V3 {
        let a = self.anchor(i, tick);
        match &self.sector {
            None => {
                let bottom = a.y - self.vines[i].length;
                v3(a.x, p.y.clamp(bottom, a.y), a.z)
            }
            Some(_) => {
                let h = self.hang(i);
                let t = p.sub(a).dot(h).clamp(0, self.vines[i].length);
                a.add(h.scale(t))
            }
        }
    }

    /// Whether vine `i` is an odd one (Sector Ape: it does not hang
    /// straight down).
    pub fn odd_vine(&self, i: usize) -> bool {
        self.sector.is_some() && self.hang(i) != crate::sector::DOWN
    }

    /// The downward pull on an ape: `rules::GRAVITY`, less on Sector Ape.
    pub fn gravity(&self) -> i32 {
        match self.sector {
            Some(_) => crate::sector::SECTOR_GRAVITY,
            None => crate::rules::GRAVITY,
        }
    }

    /// The bounce mushroom whose cap a body at `p` (the hands) is on or in:
    /// its feet are over the cap and no higher than its top.
    pub fn mushroom_under(&self, p: V3) -> Option<usize> {
        let feet = p.y - BODY_DROP;
        self.mushrooms.iter().position(|m| {
            let d = p.flat().sub(m.top.flat());
            let r = m.radius + BODY_RADIUS / 2;
            d.x.abs() <= r
                && d.z.abs() <= r
                && d.len() <= r
                && feet <= m.top.y
                && feet >= ground(m.top.x, m.top.z) - int(1)
        })
    }

    /// Whether a body at `p` (the hands) stands on the jungle floor itself,
    /// not on a platform (Floor is Lava, TRIAGE T37).
    pub fn on_floor(&self, p: V3) -> bool {
        // Sector Ape has no floor: crossing the boundary is its lava
        if self.sector.is_some() {
            return false;
        }
        let floor = self.floor(p.x, p.z);
        self.support(p) <= floor && p.y - BODY_DROP <= floor + cm(10)
    }

    /// The open floor's height at (`x`, `z`): what Floor is Lava counts as
    /// the ground and the jaguar runs on (TRIAGE T37). Every map's floor
    /// today is `ground`; a map with another floor changes only this. On
    /// the City, over the river it is where a swimmer's feet hang (water
    /// counts as the floor), else the street.
    pub fn floor(&self, x: i32, z: i32) -> i32 {
        match self.city {
            None => self.terrain(x, z),
            Some(_) => self.city_floor(v3(x, 0, z)),
        }
    }

    /// The land's height at (`x`, `z`), without water: `ground` on the
    /// Jungle and the City, the hill on the Ski Slope.
    pub fn terrain(&self, x: i32, z: i32) -> i32 {
        self.terrain.height(x, z)
    }

    /// The highest surface under a point: the ground, or a platform's top.
    pub fn support(&self, p: V3) -> i32 {
        // the City's river: its bed is below the water (city.rs)
        let mut top = self.floor(p.x, p.z);
        let feet = p.y - BODY_DROP;
        for pl in &self.platforms {
            // (quick reject: no platform holds a point further off flat
            // than its half length and half width together)
            let reach = pl.half_len + pl.half_width;
            if (p.x - pl.centre.x).abs() > reach || (p.z - pl.centre.z).abs() > reach {
                continue;
            }
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
        let side = d.dot(self.right[i]);
        ((i as i32 * STEP + along).clamp(0, self.length), side)
    }

    /// Keeps a body inside the corridor and under the ceiling: returns the
    /// corrected position and velocity, and whether a wall was hit.
    pub fn contain(&self, pos: V3, vel: V3, i: usize) -> (V3, V3, bool) {
        // Sector Ape: no walls and no ceiling, only the boundary (the race
        // puts an ape that crosses it back on a deck)
        if self.sector.is_some() {
            return (pos, vel, false);
        }
        let (s, side) = self.progress(pos, i);
        let limit = self.half_width - BODY_RADIUS;
        let mut pos = pos;
        let mut vel = vel;
        let mut hit = false;
        if !self.streets.is_empty() {
            (pos, vel, hit) = self.contain_streets(pos, vel);
        } else if self.ski.is_some() {
            (pos, vel, hit) = self.contain_lanes(pos, vel, i, s, side);
        } else if side.abs() > limit {
            let right = self.right[i];
            let out = if side > 0 { right } else { right.neg() };
            pos = pos.sub(out.scale(side.abs() - limit));
            let into = vel.dot(out);
            if into > 0 {
                vel = vel.sub(out.scale(into + mul(into, WALL_BOUNCE)));
            }
            hit = true;
        }
        // the start's back wall and the finish's far side (a finished ape
        // coasts on into the clearing past the arch, never out of it)
        let behind = pos.flat().sub(self.path[0]).dot(self.heading[0]);
        if behind < -int(6) {
            pos = pos.add(self.heading[0].scale(-int(6) - behind));
            let into = -vel.dot(self.heading[0]);
            if into > 0 {
                vel = vel.add(self.heading[0].scale(into));
            }
        }
        let last = self.path.len() - 1;
        let fwd = self.heading[last];
        let past = pos.flat().sub(self.path[last]).dot(fwd);
        let stop = OUTRO - int(15);
        if i == last && past > stop {
            pos = pos.sub(fwd.scale(past - stop));
            let into = vel.dot(fwd);
            if into > 0 {
                vel = vel.sub(fwd.scale(into));
            }
        }
        let top = self.terrain(pos.x, pos.z) + CEILING;
        if pos.y > top {
            pos.y = top;
            vel.y = vel.y.min(0);
        }
        (pos, vel, hit)
    }

    /// The walls of the Ski Slope: a body outside every lane at its place
    /// along the run (`lanes`: the whole run, or a fork's two branches with
    /// the island between) goes back across to the nearest lane's edge.
    fn contain_lanes(&self, pos: V3, vel: V3, i: usize, s: i32, side: i32) -> (V3, V3, bool) {
        let mut best: Option<i32> = None;
        for (m, h) in self.lanes(s) {
            let lim = h - BODY_RADIUS;
            let t = side.clamp(m - lim, m + lim);
            if t == side {
                return (pos, vel, false);
            }
            if best.is_none_or(|b| (t - side).abs() < (b - side).abs()) {
                best = Some(t);
            }
        }
        let Some(t) = best else {
            return (pos, vel, false);
        };
        let right = right_of(self.heading[i]);
        let out = if side > t { right } else { right.neg() };
        let pos = pos.sub(out.scale((side - t).abs()));
        let into = vel.dot(out);
        let vel = if into > 0 {
            vel.sub(out.scale(into + mul(into, WALL_BOUNCE)))
        } else {
            vel
        };
        (pos, vel, true)
    }

    /// The walls of a course with streets: a body outside every street's
    /// box (less its radius) goes back to the nearest point inside one.
    fn contain_streets(&self, pos: V3, vel: V3) -> (V3, V3, bool) {
        let lim = self.half_width - BODY_RADIUS;
        let flat = pos.flat();
        let mut best: Option<(i64, V3)> = None;
        for st in &self.streets {
            let d = flat.sub(st.from);
            let r = right_of(st.dir);
            let (along, side) = (d.dot(st.dir), d.dot(r));
            let (lo, hi) = st.span(lim);
            let a = along.clamp(lo, hi);
            let s = side.clamp(-lim, lim);
            if a == along && s == side {
                return (pos, vel, false);
            }
            let q = st.from.add(st.dir.scale(a)).add(r.scale(s));
            let push = flat.sub(q);
            let dd = push.dot64(push);
            if best.is_none_or(|(b, _)| dd < b) {
                best = Some((dd, q));
            }
        }
        let Some((_, q)) = best else {
            return (pos, vel, false);
        };
        let out = flat.sub(q).norm();
        let pos = v3(q.x, pos.y, q.z);
        let into = vel.dot(out);
        let vel = if into > 0 {
            vel.sub(out.scale(into + mul(into, WALL_BOUNCE)))
        } else {
            vel
        };
        (pos, vel, true)
    }
}

/// The centreline alone.
pub(crate) fn centreline(name: &'static str, legs: &[Leg]) -> Course {
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
    let right = heading.iter().map(|&h| right_of(h)).collect();
    Course {
        name,
        path,
        heading,
        right,
        length,
        half_width: int(20),
        laps: 1,
        vines: Vec::new(),
        mushrooms: Vec::new(),
        platforms: Vec::new(),
        crates: Vec::new(),
        bananas: Vec::new(),
        props: Vec::new(),
        starts: [V3::ZERO; 4],
        start_yaw: 0,
        city: None,
        streets: Vec::new(),
        ski: None,
        terrain: crate::ski::Terrain::Hills,
        sector: None,
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

    // Vines in stations every 22–28 m (TRIAGE rows 1 and 13, then T18:
    // with the long-vine physics an ape covers some 25 m a swing, so about
    // 60% of row 1's vines are enough). Each station has two vines, now and
    // then three, spread across the course (left, middle, right lanes, at
    // different heights), so there is always more than one way on; a low
    // vine at every other station lets an ape on the ground climb back up.
    // The start (TRIAGE row 7): a row of vines across the whole platform,
    // just past its edge, hanging to a metre above the deck, so running off
    // the edge and grabbing always catches one.
    let edge = START_LEN / 2 - int(2) + START_LEN / 2 + int(2);
    for k in -3..=3 {
        let at = edge + cm(150);
        let base = c
            .point(at)
            .add(right_of(c.heading_at(at)).scale(int(3) * k));
        c.vines.push(Vine {
            anchor: v3(base.x, START_HEIGHT + int(30), base.z),
            length: int(29),
            slide: None,
            at,
        });
    }
    let mut s = 32;
    let mut station = 0;
    let mut slides = 0;
    // the extra mushrooms (TRIAGE T48) draw from dice of their own, so the
    // rest of the layout is the same as before them
    let mut extra = Dice(seed ^ 0x6D75_7368);
    // each station's lane vines, for the banana arcs
    let mut stations: Vec<Vec<usize>> = Vec::new();
    while s < len_m - 30 {
        let mut here = Vec::new();
        let h = c.heading_at(int(s));
        let r = right_of(h);
        // which lanes get a vine: two of the three, now and then all three
        let skip = if station == 0 || dice.below(6) == 0 {
            3
        } else {
            dice.below(3) as i32
        };
        let mut lanes = Vec::new();
        for lane in 0..3 {
            if lane == skip {
                continue;
            }
            let (anchor_h, length, off) = if station == 0 {
                // off the start platform: close, and not too high
                (int(46), int(30), (lane - 1) * 6)
            } else {
                (
                    int(dice.range(38, 50)),
                    int(dice.range(23, 31)),
                    (lane - 1) * 10 + dice.range(-2, 2),
                )
            };
            let along = s + dice.range(-3, 3);
            let base = c.point(int(along)).add(r.scale(int(off)));
            let ground_y = ground(base.x, base.z);
            let mut slide = None;
            // three sliding vines, spread down the course
            if station > 0 && lane == 1 && station % 20 == 10 && slides < 3 {
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
                at: int(along),
            });
            if slide.is_none() {
                here.push(c.vines.len() - 1);
            }
            lanes.push(off);
        }
        stations.push(here);
        // a low vine to climb back up by
        if station % 2 == 1 {
            let off3 = dice.range(-9, 9);
            let b3 = c.point(int(s + 7)).add(r.scale(int(off3)));
            let g3 = ground(b3.x, b3.z);
            c.vines.push(Vine {
                anchor: v3(b3.x, g3 + int(27), b3.z),
                length: int(24) + cm(dice.range(0, 40)),
                slide: None,
                at: int(s + 7),
            });
        } else if station > 0 && station % 4 == 2 {
            // and between them a bounce mushroom (TRIAGE T5), short of one
            // of this station's vines, so the bounce rises into its reach
            let off = lanes[dice.below(lanes.len() as u32) as usize];
            let p = c.point(int(s - 11)).add(r.scale(int(off)));
            c.mushrooms.push(Mushroom {
                top: v3(p.x, ground(p.x, p.z) + MUSHROOM_HEIGHT, p.z),
                radius: MUSHROOM_RADIUS,
            });
        }
        if station % 16 == 7 {
            // a fifth more of them (TRIAGE T48): one on every sixteenth
            // station as well, under a lane, away from its low vine
            let off = lanes[extra.below(lanes.len() as u32) as usize];
            let p = c.point(int(s - 11)).add(r.scale(int(off)));
            c.mushrooms.push(Mushroom {
                top: v3(p.x, ground(p.x, p.z) + MUSHROOM_HEIGHT, p.z),
                radius: MUSHROOM_RADIUS,
            });
        }
        station += 1;
        s += dice.range(22, 28);
    }

    banana_arcs(&mut c, &stations);

    // item crates: rows of three across the course
    let mut at = 150;
    while at < len_m - 60 {
        let h = c.heading_at(int(at));
        let r = right_of(h);
        let base = c.point(int(at));
        let gy = ground(base.x, base.z);
        for off in [-9, 0, 9] {
            let p = base.add(r.scale(int(off)));
            c.crates.push(v3(p.x, gy + int(dice.range(16, 24)), p.z));
        }
        at += 240;
    }

    scenery(&mut c, &mut dice);
    c
}

/// Bananas in arcs along good swing lines (TRIAGE T43): under the vines of
/// every `BANANA_EVERY`-th station, round the bottom of the swing a clean
/// catch near the vine's end makes. The AI's swings (measured) catch some
/// 4 m short of a vine within a few metres of its end and let go some 5 m
/// past it, so a swing like that, along the course, sweeps the whole arc;
/// a catch high up the vine, or a swing across the course, misses it.
pub(crate) fn banana_arcs(c: &mut Course, stations: &[Vec<usize>]) {
    use crate::rules::{BANANA_ARC, BANANA_EVERY};
    for (k, here) in stations.iter().enumerate() {
        if k == 0 || k % BANANA_EVERY != 0 {
            continue;
        }
        for &i in here {
            let v = c.vines[i];
            let fwd = c.heading_at(v.at);
            // where the hands swing, and the body's middle under them
            let rope = v.length + cm(50);
            for j in -2..=2 {
                let angle = j * BANANA_ARC;
                let hands = v.anchor.add(fwd.scale(mul(fx::sin(angle), rope)).sub(v3(
                    0,
                    mul(fx::cos(angle), rope),
                    0,
                )));
                c.bananas.push(hands.sub(v3(0, BODY_DROP / 2, 0)));
            }
        }
    }
}

/// Whether `s` metres along is in a stream's bed (nothing grows there).
fn in_stream(s: i32) -> bool {
    STREAMS.iter().any(|w| (s - w).abs() < int(8))
}

/// Trunks along both walls and deeper in, branches to the vines, the
/// undergrowth, streams, and the valley's walls beyond the trees; on past
/// the finish into the clearing (TRIAGE T6). None of it hangs where an ape
/// swings: the crowns nearest the vines stand above their anchors.
fn scenery(c: &mut Course, dice: &mut Dice) {
    use TreeKind::*;
    let hw = c.half_width;
    let end = c.length + OUTRO;
    // four rows of trees along each wall, of five kinds and of heights
    // from 9 m (bamboo) to 115 m, with thick trunks (TRIAGE T16, T21, T31)
    let mut s = 0;
    while s < end {
        let p = c.point(s);
        let r = right_of(c.heading_at(s));
        for sign in [-1, 1] {
            for row in 0..4 {
                let off = hw + int(2) + int(row * 6) + int(dice.range(0, 4));
                let q = p.add(r.scale(off * sign));
                let q = v3(
                    q.x + int(dice.range(-2, 2)),
                    0,
                    q.z + int(dice.range(-2, 2)),
                );
                let radius = cm(dice.range(130, 260)) + row * cm(30);
                let shade = dice.below(4) as u8;
                let roll = dice.below(20);
                let (kind, height) = if row == 0 {
                    // nearest the vines: crowns well above their anchors
                    match roll {
                        0..=10 => (Broadleaf, int(dice.range(66, 90))),
                        11..=15 => (Emergent, int(dice.range(92, 115))),
                        16..=17 => (Fig, int(dice.range(68, 82))),
                        _ => (Palm, int(dice.range(16, 28))),
                    }
                } else {
                    // deeper in: young trees and giants
                    match roll {
                        0..=3 => (Broadleaf, int(dice.range(24, 38))),
                        4..=6 => (Broadleaf, int(dice.range(64, 90))),
                        7..=9 => (Emergent, int(dice.range(90, 115))),
                        10..=13 => (Palm, int(dice.range(14, 30))),
                        14..=15 => (Fig, int(dice.range(64, 80))),
                        _ => (Bamboo, int(dice.range(9, 16))),
                    }
                };
                if in_stream(s) {
                    continue;
                }
                c.props.push(Prop::Tree {
                    at: v3(q.x, ground(q.x, q.z), q.z),
                    radius,
                    height,
                    shade,
                    kind,
                });
            }
        }
        s += int(dice.range(6, 9) * 2);
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
    // the valley's walls: far trees climbing them, and rock outcrops
    let mut s = -int(50);
    while s < end + int(20) {
        let p = c.point(s);
        let r = right_of(c.heading_at(s));
        for sign in [-1, 1] {
            for _ in 0..2 {
                let a = int(dice.range(50, 116));
                let q = p.add(r.scale(a * sign));
                let height = int(dice.range(14, 30));
                let radius = cm(dice.range(60, 120));
                let shade = dice.below(4) as u8;
                if in_stream(s) {
                    continue;
                }
                c.props.push(Prop::Tree {
                    at: v3(q.x, ground(q.x, q.z) + rise(s, a * sign) - int(2), q.z),
                    radius,
                    height,
                    shade,
                    kind: Hill,
                });
            }
        }
        if dice.below(4) == 0 {
            let a = int(dice.range(78, 114)) * if dice.below(2) == 0 { 1 } else { -1 };
            let q = p.add(r.scale(a));
            c.props.push(Prop::Outcrop {
                at: v3(q.x, ground(q.x, q.z) + rise(s, a), q.z),
                size: int(dice.range(6, 14)),
                yaw: dice.range(0, fx::TURN),
            });
        }
        s += int(dice.range(10, 14));
    }
    // streams across the valley, down its walls and over the floor
    for w in STREAMS {
        let mut last: Option<V3> = None;
        let mut a = -int(126);
        while a <= int(126) {
            let at = w + mul(sin(mul(a, 700)), int(5));
            let q = c.point(at).add(right_of(c.heading_at(at)).scale(a));
            let q = v3(q.x, ground(q.x, q.z) + rise(at, a) + cm(8), q.z);
            if let Some(from) = last {
                c.props.push(Prop::Water {
                    from,
                    to: q,
                    width: if a.abs() < int(60) { int(4) } else { cm(260) },
                });
            }
            if dice.below(3) == 0 && a.abs() < int(90) {
                let side = right_of(c.heading_at(at)).cross(V3::UP).norm();
                let k = int(dice.range(3, 4)) * if dice.below(2) == 0 { 1 } else { -1 };
                let rock = q.add(side.scale(k));
                c.props.push(Prop::Rock {
                    at: v3(rock.x, ground(rock.x, rock.z) + rise(at, a), rock.z),
                    size: cm(dice.range(50, 120)),
                    yaw: dice.range(0, fx::TURN),
                });
            }
            last = Some(q);
            a += int(6);
        }
    }
    // undergrowth on the floor: low, and thinner in the middle; past the
    // finish it leaves the podium's clearing free
    let mut s = int(4);
    while s < end - int(20) {
        let h = c.heading_at(s);
        let r = right_of(h);
        let base = c.point(s);
        let clearing = s > c.length + int(25) && s < c.length + int(70);
        for k in 0..6 {
            // three on the floor, three deeper among the trunks (TRIAGE
            // T31: denser cover)
            let off = if k < 3 {
                int(dice.range(-19, 19))
            } else {
                int(dice.range(21, 46)) * if dice.below(2) == 0 { 1 } else { -1 }
            };
            let p = base.add(r.scale(off)).add(h.scale(int(dice.range(-2, 2))));
            let at = v3(p.x, ground(p.x, p.z), p.z);
            let roll = dice.below(20);
            let (size, yaw, shade) = (
                dice.range(50, 200) + if k < 3 { 0 } else { 80 },
                dice.range(0, fx::TURN),
                dice.below(4) as u8,
            );
            if in_stream(s) || (clearing && off.abs() < int(12)) {
                continue;
            }
            c.props.push(match roll {
                0..=5 => Prop::Fern {
                    at,
                    size: cm(size.clamp(80, 240)),
                    yaw,
                },
                6..=9 => Prop::Bush {
                    at,
                    size: cm(size.max(90)),
                    shade,
                },
                10..=15 => Prop::Flower { at, colour: shade },
                16..=18 => Prop::Rock {
                    at,
                    size: cm(size.min(140)),
                    yaw,
                },
                _ if off.abs() < hw => Prop::Rock {
                    at,
                    size: cm(size.min(140)),
                    yaw,
                },
                _ => {
                    // a fallen trunk among the trees, lying roughly along
                    // the course so it stays out of the corridor
                    let turn = yaw % deg(30) - deg(15);
                    let d = crate::geom::facing(crate::geom::yaw_of(h) + turn, 0)
                        .scale(int(4) + cm(size * 3));
                    Prop::Fallen {
                        from: at.sub(d),
                        to: at.add(d),
                        radius: cm(50) + cm(size / 2),
                    }
                }
            });
        }
        s += int(5);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{GRAVITY, MUSHROOM_BOUNCE};

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
    fn every_starter_can_run_off_the_platform_and_catch_a_vine() {
        use crate::ape::{Ape, Controls};
        use crate::rules::Kind;
        let c = Course::jungle();
        for kind in [Kind::Gorilla, Kind::Lemur, Kind::Chimpanzee] {
            for start in c.starts {
                let mut ape = Ape::new(kind, false, start, 0);
                let mut caught = false;
                for t in 0..360u32 {
                    // run forward; once past the deck's edge, hold grab
                    let off_edge = ape.pos.sub(c.path[0]).dot(c.heading[0])
                        > c.platforms[0].centre.sub(c.path[0]).dot(c.heading[0])
                            + c.platforms[0].half_len;
                    let ctl = Controls {
                        fwd: 1,
                        grab: off_edge,
                        ..Controls::default()
                    };
                    ape.step(&ctl, &c, t);
                    if ape.swinging().is_some() {
                        caught = true;
                        break;
                    }
                }
                assert!(caught, "{kind:?} from {:?} never caught a vine", start);
            }
        }
    }

    #[test]
    fn every_stretch_offers_more_than_one_way_on() {
        // TRIAGE row 1: in every stretch of the course there are at least
        // two vines, at least 6 m apart across it, and a low vine within
        // 40 m. T18 thinned the vines, so a stretch is now any 35 m (it was
        // every 25 m).
        let c = Course::jungle();
        let side = |v: &Vine| c.progress(v.anchor, c.nearest_anywhere(v.anchor)).1;
        let mut from = int(30);
        while from < c.length - int(60) {
            let here: Vec<&Vine> = c
                .vines
                .iter()
                .filter(|v| v.at >= from && v.at < from + int(35))
                .collect();
            assert!(here.len() >= 2, "{} m: {} vines", from / ONE, here.len());
            let lo = here.iter().map(|v| side(v)).min().unwrap();
            let hi = here.iter().map(|v| side(v)).max().unwrap();
            assert!(
                hi - lo >= int(6),
                "{} m: lanes {} to {}",
                from / ONE,
                lo / ONE,
                hi / ONE
            );
            let low = c.vines.iter().any(|v| {
                (v.at - from).abs() < int(40)
                    && v.anchor.y - v.length - ground(v.anchor.x, v.anchor.z) < int(4)
            });
            assert!(low, "{} m: no low vine near", from / ONE);
            from += int(5);
        }
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
        // TRIAGE T17: the swinging logs are gone; the sliding vines remain
        let c = Course::jungle();
        let sliding = c.vines.iter().filter(|v| v.slide.is_some()).count();
        assert_eq!(sliding, 3);
    }

    #[test]
    fn there_are_about_sixty_percent_of_the_old_vines() {
        // TRIAGE T18: 310 vines before; the start row and the low vines stay
        let c = Course::jungle();
        let n = c.vines.len();
        assert!((165..=205).contains(&n), "{n} vines");
        let low = c
            .vines
            .iter()
            .filter(|v| v.anchor.y - v.length - ground(v.anchor.x, v.anchor.z) < int(4))
            .count();
        assert!(low >= 30, "{low} low vines");
    }

    #[test]
    fn bounce_mushrooms_stand_along_the_course_under_vines() {
        let c = Course::jungle();
        assert!(c.mushrooms.len() >= 15, "{}", c.mushrooms.len());
        for m in &c.mushrooms {
            let k = c.nearest_anywhere(m.top);
            let (along, side) = c.progress(m.top, k);
            assert!(side.abs() < c.half_width - m.radius, "{} m off", side / ONE);
            // a vine ahead of it, within 25 m, that the bounce reaches
            let reach = crate::rules::Kind::Chimpanzee
                .race_stats(true)
                .physics()
                .reach;
            let peak =
                m.top.y + BODY_DROP + fx::div(mul(MUSHROOM_BOUNCE, MUSHROOM_BOUNCE), 2 * GRAVITY);
            let ahead = c.vines.iter().any(|v| {
                v.at > along && v.at < along + int(25) && v.anchor.y - v.length < peak + reach
            });
            assert!(ahead, "{} m: no vine to bounce to", along / ONE);
        }
    }

    #[test]
    fn mushrooms_are_a_fifth_more_and_stand_clear_of_vines_and_trees() {
        // TRIAGE T48: 18 before; 1.5 times the size, out from under every
        // vine's end (with its sway) and every trunk
        let c = Course::jungle();
        let n = c.mushrooms.len();
        assert!((21..=24).contains(&n), "{n} mushrooms");
        for m in &c.mushrooms {
            assert_eq!(m.radius, MUSHROOM_RADIUS);
            for (i, v) in c.vines.iter().enumerate() {
                let sway = v.slide.map(|s| s.reach.len()).unwrap_or(0) + cm(60);
                let flat = v.anchor.flat().sub(m.top.flat()).len();
                let bottom = v.anchor.y - v.length;
                assert!(
                    flat > m.radius + sway || bottom > m.top.y + int(2),
                    "vine {i} ends {} m above a cap {} m away",
                    (bottom - m.top.y) / ONE,
                    flat / ONE
                );
            }
            for p in &c.props {
                if let Prop::Tree { at, radius, .. } = *p {
                    let flat = at.flat().sub(m.top.flat()).len();
                    assert!(flat > m.radius + radius, "a trunk {} m away", flat / ONE);
                }
            }
        }
    }

    #[test]
    fn banana_arcs_hang_under_vines_along_the_course() {
        // TRIAGE T43: enough for a good run to fill up, in arcs of five
        // round the bottom of a vine's swing, clear of the ground
        use crate::rules::BANANAS_FULL;
        let c = Course::jungle();
        let n = c.bananas.len();
        assert!(
            n >= 3 * BANANAS_FULL as usize && n.is_multiple_of(5),
            "{n} bananas"
        );
        for arc in c.bananas.chunks(5) {
            let mid = arc[2];
            // under a vine's end, a body's middle below the hands
            let under = c.vines.iter().any(|v| {
                v.anchor.flat().sub(mid.flat()).len() < cm(10)
                    && (v.anchor.y - v.length - mid.y - BODY_DROP / 2 - cm(50)).abs() < cm(10)
            });
            assert!(under, "an arc at {} m is under no vine", mid.y / ONE);
            for b in arc {
                assert!(
                    b.y - ground(b.x, b.z) > int(3),
                    "a banana {} m up",
                    b.y / ONE
                );
                let k = c.nearest_anywhere(*b);
                assert!(c.progress(*b, k).1.abs() < c.half_width - int(2));
            }
            // and spread along the course, not across it
            let k = c.nearest_anywhere(mid);
            let (a0, _) = c.progress(arc[0], k);
            let (a4, _) = c.progress(arc[4], k);
            assert!(a4 - a0 > int(4), "{} m long", (a4 - a0) / ONE);
        }
    }

    #[test]
    fn tree_crowns_stay_clear_of_the_vines() {
        // TRIAGE T21: heights vary, but a crown never spreads over a vine
        // unless it is wholly above its anchor
        let c = Course::jungle();
        let mut checked = 0;
        for p in &c.props {
            let Prop::Tree {
                at, height, kind, ..
            } = *p
            else {
                continue;
            };
            let (reach, below) = crown(kind, height);
            let bottom = at.y + height - below;
            for (i, v) in c.vines.iter().enumerate() {
                let sway = v.slide.map(|s| s.reach.len()).unwrap_or(0);
                let flat = v.anchor.flat().sub(at.flat()).len();
                if flat < reach + sway + int(1) {
                    checked += 1;
                    assert!(
                        bottom > v.anchor.y + int(1),
                        "vine {i}: a {kind:?} {} m tall, {} m away, crown bottom {} m, anchor {} m",
                        height / ONE,
                        flat / ONE,
                        bottom / ONE,
                        v.anchor.y / ONE
                    );
                }
            }
        }
        assert!(checked > 0);
    }

    #[test]
    fn trees_vary_in_height_and_kind() {
        let c = Course::jungle();
        let mut heights = Vec::new();
        let mut kinds = Vec::new();
        for p in &c.props {
            if let Prop::Tree { height, kind, .. } = *p {
                if kind != TreeKind::Hill {
                    heights.push(height);
                }
                if !kinds.contains(&kind) {
                    kinds.push(kind);
                }
            }
        }
        heights.sort();
        let lo = heights[heights.len() / 10];
        let hi = heights[heights.len() * 9 / 10];
        assert!(
            lo <= int(25) && hi >= int(80),
            "{} to {}",
            lo / ONE,
            hi / ONE
        );
        assert_eq!(kinds.len(), 6, "{kinds:?}");
    }

    #[test]
    fn the_trees_are_large_and_the_cover_dense() {
        // TRIAGE T31: thick trunks and wide crowns, close together
        let c = Course::jungle();
        let mut radii = Vec::new();
        let mut spreads = Vec::new();
        for p in &c.props {
            if let Prop::Tree {
                radius,
                height,
                kind,
                ..
            } = *p
            {
                if kind != TreeKind::Hill && kind != TreeKind::Bamboo && height > int(55) {
                    radii.push(radius);
                    spreads.push(crown(kind, height).0);
                }
            }
        }
        radii.sort();
        spreads.sort();
        let per_km = radii.len() as i32 * 1000 / ((c.length + OUTRO) / ONE);
        let (r, w) = (radii[radii.len() / 2], spreads[spreads.len() / 2]);
        assert!(
            r >= int(2) && w >= int(18) && per_km >= 250,
            "canopy trees: trunk {} cm, crown {} m, {per_km} a km",
            r * 100 / ONE,
            w / ONE
        );
    }

    #[test]
    fn the_jungle_goes_on_past_the_finish() {
        // TRIAGE T6: trees and undergrowth past the arch, up to the cliff
        let c = Course::jungle();
        let last = c.path.len() - 1;
        let past = |p: V3| p.flat().sub(c.path[last]).dot(c.heading[last]);
        let beyond = c
            .props
            .iter()
            .filter(|p| match **p {
                Prop::Tree { at, .. } | Prop::Bush { at, .. } | Prop::Fern { at, .. } => {
                    past(at) > int(100)
                }
                _ => false,
            })
            .count();
        assert!(beyond > 20, "{beyond}");
        // and a finished ape coasting on is stopped short of the cliff
        let fwd = c.heading[last];
        let p = c.path[last].add(fwd.scale(OUTRO));
        let (q, v, _) = c.contain(p, fwd.scale(int(20)), last);
        assert!(past(q) < OUTRO - int(10));
        assert!(v.dot(fwd) <= 0);
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
    fn sliding_vines_move() {
        let c = Course::jungle();
        let i = c.vines.iter().position(|v| v.slide.is_some()).unwrap();
        assert_ne!(c.anchor(i, 0), c.anchor(i, 200));
    }
}
