//! The Ski Slope's layout (2026-10-13): a run of about 1.9 km down a hill
//! at noon, from the summit to the base village. The course keeps within
//! 45° of the fall line (the start's heading), so it always runs downhill;
//! the traverses are gentler than the straights down the fall line.
//!
//! - **Forks** (`FORKS`): the run splits round an island twice and joins
//!   again. The first fork's left branch runs under a chairlift (its vines
//!   hang from the chairs), its right branch through big conifers; the
//!   second fork is the other way round, with a helicopter over its left
//!   branch. Each branch is narrower than the run (13 and 12 m either side
//!   of its line) and the island between is out of bounds (`Course::contain`).
//! - **Chairlifts** (`LIFTS`): five, each one straight line from its bottom
//!   terminal up to its top terminal, chained down the hill: each lift's
//!   bottom terminal stands near the next one's top terminal. They cross the
//!   run at a slant (the run bends; a lift never does), so their chairs are
//!   over the lanes only part of the way.
//! - **Vines**: stations every 22–28 m as on the Jungle. Each lane's vine
//!   hangs from the chair nearest its spot when one is close, else from a
//!   big conifer's branch at the nearer edge of its lane (a fence or an
//!   island), else (a middle lane far from any chair) now and then from a
//!   wire strung between two conifers across the run. A low vine at every
//!   other station, in each branch of a fork. Two helicopters.
//! - **Landmarks**: one of each kind, in order down the hill, just before
//!   the turn or fork they announce, outside the run.

use super::*;
use crate::course::{banana_arcs, centreline, Leg, Platform, Vine, START_HEIGHT, START_LEN};
use crate::fx::{int, Dice};
use crate::geom::yaw_of;
use crate::rules::BODY_DROP;

const SEED: u64 = 0x5C1_0001;

/// The run, metres and degrees (left turns positive). Its heading stays
/// within 45° of the start's (the fall line): 0, 35, -20, 25, -40, 0.
const LEGS: &[Leg] = &[
    Leg(180, 0),
    Leg(90, 35),
    Leg(440, 0),
    Leg(100, -55),
    Leg(140, 0),
    Leg(110, 45),
    Leg(330, 0),
    Leg(110, -65),
    Leg(150, 0),
    Leg(90, 40),
    Leg(160, 0),
];

/// The two forks, each on a straight of the spine (so a branch is straight
/// too, and a lift can run down one). The first: a long one, its left
/// branch further out (under lift 2), its right branch through the trees.
/// The second: shorter, the other way round (its right branch under lift
/// 4, a helicopter over its left).
pub const FORKS: [Fork; 2] = [
    Fork {
        from: int(330),
        to: int(670),
        left: int(30),
        right: int(22),
        branch_half: int(15),
        ramp: int(60),
    },
    Fork {
        from: int(1090),
        to: int(1360),
        left: int(20),
        right: int(30),
        branch_half: int(14),
        ramp: int(55),
    },
];

/// A place by the run: (metres along it, metres right of its line).
type Spot = (i32, i32);

/// The chairlifts, top of the hill first: each (top terminal, bottom
/// terminal) as (metres along the run, metres right of its line), and the
/// gauge between its ropes. Each bottom terminal stands near the next
/// lift's top terminal. Lifts 2 and 4 run down a fork's branch, just
/// outside its outer fence, with a wide gauge so the inner rope's chairs
/// hang 4 m in over the branch; the others cross the run at a slant. No
/// tower or terminal stands in a lane (the Art's B6: apes passed through
/// them): towers slide along the line out of the lanes (`lift`).
const LIFTS: &[(Spot, Spot, i32)] = &[
    ((-24, 28), (296, -42), 6),
    ((288, -47), (700, -47), 12),
    ((706, -38), (1060, 40), 6),
    ((1066, 46), (1392, 46), 12),
    ((1398, 38), (1905, 32), 6),
];

/// The haul rope's height above each support, and the lifts' other sizes.
pub const ROPE_HEIGHT: i32 = int(44);
const CHAIR_EVERY: i32 = int(8);
/// Towers stand at most this far apart.
const TOWER_EVERY: i32 = int(50);
/// A tower's foot stands at least this far outside every lane, slid along
/// its line by up to this many metres to get there (else it is left out:
/// a longer span).
const TOWER_CLEAR: i32 = int(2);
const TOWER_SLIDE: i32 = 40;

/// A lane's vine takes a chair this close to its spot (flat), if the seat
/// is over a lane this far in from its edges.
const CHAIR_NEAR: i32 = int(5);
const CHAIR_IN: i32 = int(3);
/// A conifer stands this far outside the edge of the lane it serves (behind
/// the fence, or on the island), and its branch reaches this far at most.
const TRUNK_OUT: i32 = cm(150);
const BRANCH_MAX: i32 = int(14);
/// A helicopter's rotor hub above where its cable leaves the belly.
const ROTOR_UP: i32 = int(3);
/// The helicopters: over the station nearest this far along, in this lane
/// (0 the left branch of a fork, or the middle of the run).
const HELICOPTERS: [i32; 2] = [int(1220), int(1450)];

/// Rows of item crates, metres along.
const CRATES: [i32; 7] = [150, 490, 740, 990, 1225, 1480, 1720];

/// The Ski Slope course.
pub fn ski() -> Course {
    ski_seeded(SEED)
}

/// What the builder keeps beside the course.
struct Build {
    lay: SkiLayout,
    /// chairs that carry a vine: (lift, chair, side)
    used: Vec<(usize, i32, i32)>,
    dice: Dice,
    /// where the island landmarks stand (metres along): no tree on their
    /// island from `CLEAR_BEFORE` before them to `CLEAR_AFTER` past
    clearings: Vec<i32>,
}

/// The stretch of an island kept clear of trees in front of its landmark
/// (the Art's B7: the trees hid the Snowman and the Ice Ape).
const CLEAR_BEFORE: i32 = int(40);
const CLEAR_AFTER: i32 = int(12);

impl Build {
    fn in_clearing(&self, s: i32) -> bool {
        self.clearings
            .iter()
            .any(|&m| s > m - CLEAR_BEFORE && s < m + CLEAR_AFTER)
    }
}

/// A point `off` metres right of the run's line at `s` metres along, on
/// the ground (y 0).
fn pt(c: &Course, s: i32, off: i32) -> V3 {
    c.point(s).add(right_of(c.heading_at(s)).scale(off)).flat()
}

fn on_ground(c: &Course, p: V3) -> V3 {
    v3(p.x, c.terrain(p.x, p.z), p.z)
}

/// The lanes at `s`: the whole run, or a fork's two branches (from the
/// fork's first metre: while they part they overlap). Each (centre, half
/// width, whether a tree may stand outside its left edge, and its right):
/// not where the other branch overlaps it.
fn lanes_at(c: &Course, s: i32) -> Vec<(i32, i32, bool, bool)> {
    let [(c0, h0), (c1, h1)] = c.lanes(s);
    if (c0, h0) == (c1, h1) {
        return vec![(c0, h0, true, true)];
    }
    let apart = c0 + h0 + TRUNK_OUT * 2 < c1 - h1;
    vec![(c0, h0, true, apart), (c1, h1, apart, true)]
}

/// Whether a point `off` right of the line at `s` is over a lane, at least
/// `margin` in from its edges.
fn in_lane(c: &Course, s: i32, off: i32, margin: i32) -> bool {
    c.lanes(s)
        .iter()
        .any(|&(m, h)| (off - m).abs() <= h - margin)
}

/// The run's (metres along, metres right) for a point.
fn place(c: &Course, p: V3) -> (i32, i32) {
    c.progress(p, c.nearest_anywhere(p))
}

impl Build {
    fn hang(&mut self, c: &mut Course, vine: Vine, kind: HangerKind, from: V3, to: V3) -> usize {
        c.vines.push(vine);
        let i = c.vines.len() - 1;
        self.lay.hangers.push(Hanger {
            vine: i,
            kind,
            from,
            to,
        });
        i
    }

    /// The conifer nearest `p` (flat) within 4 m (and within a branch's
    /// reach of `serves`, if given), or a new one there tall enough to carry
    /// a branch at `branch_y`.
    fn tree(&mut self, c: &Course, p: V3, branch_y: i32, serves: Option<V3>) -> usize {
        let foot = on_ground(c, p);
        if let Some(i) = self.lay.trees.iter().position(|t| {
            t.at.flat().dist(p.flat()) < int(4)
                && serves.is_none_or(|s| t.at.flat().dist(s.flat()) <= BRANCH_MAX)
        }) {
            let need = branch_y - self.lay.trees[i].at.y + int(6);
            let t = &mut self.lay.trees[i];
            t.height = t.height.max(need);
            return i;
        }
        let height = (branch_y - foot.y + int(self.dice.range(6, 14))).max(int(20));
        self.lay.trees.push(Conifer {
            at: foot,
            height,
            radius: mul(height, ONE * 15 / 100) + int(self.dice.range(0, 2)),
            shade: self.dice.below(4) as u8,
        });
        self.lay.trees.len() - 1
    }

    /// The free chair nearest `p` (flat) within `CHAIR_NEAR`, its seat over
    /// a lane: (lift, chair, side, seat).
    fn chair_near(&self, c: &Course, p: V3) -> Option<(usize, i32, i32, V3)> {
        let mut best: Option<(i32, (usize, i32, i32, V3))> = None;
        for (l, lift) in self.lay.lifts.iter().enumerate() {
            let dir = lift.dir();
            let d = p.flat().sub(lift.bottom.flat()).dot(dir);
            let k0 = d / lift.chair_every;
            for k in [k0 - 1, k0, k0 + 1, k0 + 2] {
                // none right by a terminal
                if k < 1 || k >= lift.chairs() {
                    continue;
                }
                for side in [-1, 1] {
                    if self.used.contains(&(l, k, side)) {
                        continue;
                    }
                    let (_, seat) = lift.chair(k, side);
                    let gap = seat.flat().dist(p.flat());
                    if gap > CHAIR_NEAR || best.is_some_and(|(b, _)| gap >= b) {
                        continue;
                    }
                    let (s, off) = place(c, seat);
                    if in_lane(c, s, off, CHAIR_IN) {
                        best = Some((gap, (l, k, side, seat)));
                    }
                }
            }
        }
        best.map(|(_, b)| b)
    }

    /// A vine for the spot `off` right of the line at `along`, hanging
    /// `anchor_h` above the ground with its end `bottom` above it: from a
    /// chair near (unless `low`), else a conifer at the nearer edge of its
    /// lane. `None` if neither reaches (a middle lane far from any chair).
    fn vine(
        &mut self,
        c: &mut Course,
        along: i32,
        off: i32,
        anchor_h: i32,
        bottom: i32,
        low: bool,
    ) -> Option<usize> {
        let p = pt(c, along, off);
        if !low {
            if let Some((l, k, side, seat)) = self.chair_near(c, p) {
                self.used.push((l, k, side));
                let g = c.terrain(seat.x, seat.z);
                let length = (seat.y - g - bottom).clamp(int(18), int(34));
                let (s, _) = place(c, seat);
                let (grip, _) = self.lay.lifts[l].chair(k, side);
                return Some(self.hang(
                    c,
                    Vine {
                        anchor: seat,
                        length,
                        slide: None,
                        at: s,
                    },
                    HangerKind::Chair { lift: l },
                    grip,
                    seat,
                ));
            }
        }
        // the nearest lane edge a tree may stand by, of a lane it is over
        let mut edge: Option<(i32, i32, i32)> = None;
        let lanes = lanes_at(c, along);
        let split = lanes.len() == 2;
        let cleared = self.in_clearing(along);
        for (k, &(m, h, lo_tree, hi_tree)) in lanes.iter().enumerate() {
            // (an island's edge: none in front of its landmark)
            let lo_tree = lo_tree && !(split && k == 1 && cleared);
            let hi_tree = hi_tree && !(split && k == 0 && cleared);
            for (sign, ok) in [(-1, lo_tree), (1, hi_tree)] {
                let to_edge = h - (off - m) * sign;
                let fits = ok && (0..=BRANCH_MAX - TRUNK_OUT).contains(&to_edge);
                if fits && edge.is_none_or(|(d, _, _)| to_edge < d) {
                    edge = Some((to_edge, m + sign * h, sign));
                }
            }
        }
        let (_, at_edge, sign) = edge?;
        let trunk = pt(c, along, at_edge + sign * TRUNK_OUT);
        let g = c.terrain(p.x, p.z);
        let anchor = v3(p.x, g + anchor_h, p.z);
        let t = self.tree(c, trunk, anchor.y, Some(anchor));
        let foot = self.lay.trees[t].at;
        let from = v3(foot.x, anchor.y + int(1), foot.z);
        Some(self.hang(
            c,
            Vine {
                anchor,
                length: anchor_h - bottom,
                slide: None,
                at: along,
            },
            HangerKind::Tree { tree: t },
            from,
            anchor,
        ))
    }

    /// A wire across the run at `along`, between conifers behind the fences
    /// either side, carrying vines at `offs`; returns them.
    fn wire(
        &mut self,
        c: &mut Course,
        along: i32,
        offs: &[i32],
        anchor_y: i32,
        length: i32,
    ) -> Vec<usize> {
        let lanes = lanes_at(c, along);
        let lo = lanes.first().map_or(-c.half_width, |&(m, h, _, _)| m - h);
        let hi = lanes.last().map_or(c.half_width, |&(m, h, _, _)| m + h);
        let a = pt(c, along, lo - TRUNK_OUT);
        let b = pt(c, along, hi + TRUNK_OUT);
        let ta = self.tree(c, a, anchor_y, None);
        let tb = self.tree(c, b, anchor_y, None);
        let (fa, fb) = (self.lay.trees[ta].at, self.lay.trees[tb].at);
        let from = v3(fa.x, anchor_y + int(1), fa.z);
        let to = v3(fb.x, anchor_y + int(1), fb.z);
        offs.iter()
            .map(|&off| {
                let p = pt(c, along, off);
                self.hang(
                    c,
                    Vine {
                        anchor: v3(p.x, anchor_y, p.z),
                        length,
                        slide: None,
                        at: along,
                    },
                    HangerKind::Wire,
                    from,
                    to,
                )
            })
            .collect()
    }

    /// A helicopter hovering with its cable's end over `off` at `along`.
    fn helicopter(&mut self, c: &mut Course, along: i32, off: i32) -> usize {
        let p = pt(c, along, off);
        let g = c.terrain(p.x, p.z);
        let anchor = v3(p.x, g + int(46), p.z);
        self.hang(
            c,
            Vine {
                anchor,
                length: int(30),
                slide: None,
                at: along,
            },
            HangerKind::Helicopter,
            anchor.add(v3(0, ROTOR_UP, 0)),
            anchor,
        )
    }
}

/// A chairlift from `top` to `bottom` (each metres along, metres right),
/// `gauge` metres between its ropes, with towers evenly spaced between,
/// each slid along the line to the nearest place clear of the lanes.
fn lift(c: &Course, top: Spot, bottom: Spot, gauge: i32) -> Lift {
    let t = on_ground(c, pt(c, int(top.0), int(top.1)));
    let b = on_ground(c, pt(c, int(bottom.0), int(bottom.1)));
    let run = t.sub(b).flat();
    let len = run.len();
    let dir = run.norm();
    let n = (len + TOWER_EVERY - 1) / TOWER_EVERY;
    let mut towers: Vec<V3> = Vec::new();
    for k in 1..n {
        let d = mul(len, fx::div(k, n));
        let clear = |d: i32| {
            let p = b.add(dir.scale(d));
            let (s, off) = place(c, p);
            !in_lane(c, s, off, -TOWER_CLEAR)
        };
        let last = towers.last().map_or(0, |t| t.flat().sub(b.flat()).dot(dir));
        let mut at = None;
        for step in 0..=TOWER_SLIDE {
            for e in [d - int(step), d + int(step)] {
                if at.is_none() && e > last + int(10) && e < len - int(10) && clear(e) {
                    at = Some(e);
                }
            }
        }
        if let Some(e) = at {
            towers.push(on_ground(c, b.add(dir.scale(e))));
        }
    }
    Lift {
        bottom: b,
        top: t,
        towers,
        rope_height: ROPE_HEIGHT,
        gauge: int(gauge),
        chair_every: CHAIR_EVERY,
    }
}

/// The Ski Slope laid out from another seed: the same run, forks, lifts
/// and landmarks, different vines and trees (for the AI's tests).
pub fn ski_seeded(seed: u64) -> Course {
    let mut c = centreline("ski", LEGS);
    let fall = c.heading_at(0);
    c.terrain = Terrain::Slope {
        top: c.path[0],
        fall,
    };
    c.ski = Some(Box::new(SkiLayout {
        forks: FORKS.to_vec(),
        ..SkiLayout::default()
    }));
    let mut b = Build {
        lay: SkiLayout {
            forks: FORKS.to_vec(),
            ..SkiLayout::default()
        },
        used: Vec::new(),
        dice: Dice(seed),
        clearings: Vec::new(),
    };
    b.clearings = landmarks(&c)
        .iter()
        .filter(|m| matches!(m.kind, LandmarkKind::Snowman | LandmarkKind::IceApe))
        .map(|m| m.s)
        .collect();
    b.lay.lifts = LIFTS.iter().map(|&(t, bt, g)| lift(&c, t, bt, g)).collect();
    let len_m = c.length / ONE;

    // the summit platform; the starters stand across it
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
    c.start_yaw = 0;

    // the start row (TRIAGE row 7): seven vines across the platform's front
    // edge, on a wire between two conifers, hanging to a metre above the deck
    let edge = START_LEN + cm(150);
    let offs: Vec<i32> = (-3..=3).map(|k| int(3) * k).collect();
    b.wire(&mut c, edge, &offs, deck + int(30), int(29));

    // the stations
    let mut stations: Vec<Vec<usize>> = Vec::new();
    let mut helis = HELICOPTERS.to_vec();
    let mut s = 32;
    let mut station = 0;
    let mut middle_skipped = false;
    while s < len_m - 30 {
        let si = int(s);
        let lanes = lanes_at(&c, si);
        let mut here = Vec::new();
        if station == 0 {
            // off the summit: close, and not too high, on a second wire
            let g = c.terrain(c.point(si).x, c.point(si).z);
            let offs = [-int(6), 0, int(6)];
            here = b.wire(&mut c, si, &offs, g + int(46), int(30));
        } else {
            // a helicopter over this station?
            let heli = helis.first().copied().filter(|&h| si + int(12) >= h);
            if heli.is_some() {
                helis.remove(0);
            }
            if let [(m, h, _, _)] = lanes[..] {
                // the whole run: two of three lanes, now and then all
                // three; the middle is skipped one time in four at most and
                // never twice running (the City: a station with only the
                // outer lanes leaves no way on but along a wall), and never
                // under a helicopter
                let side = (h - int(10)).max(int(6));
                let skip = if b.dice.below(6) == 0 {
                    3
                } else if !middle_skipped && heli.is_none() && b.dice.below(4) == 0 {
                    1
                } else {
                    2 * b.dice.below(2) as i32
                };
                middle_skipped = skip == 1;
                for lane in 0..3 {
                    if lane == skip {
                        continue;
                    }
                    let along = int(s + b.dice.range(-3, 3));
                    let off = m + (lane - 1) * side + int(b.dice.range(-1, 1));
                    if lane == 1 && heli.is_some() {
                        here.push(b.helicopter(&mut c, along, m));
                        continue;
                    }
                    let anchor_h = int(b.dice.range(38, 50));
                    let bottom = anchor_h - int(b.dice.range(23, 31));
                    // (the middle, far from a chair: a wire across the run)
                    match b.vine(&mut c, along, off, anchor_h, bottom, false) {
                        Some(v) => here.push(v),
                        None => {
                            let g = c.terrain(c.point(along).x, c.point(along).z);
                            here.extend(b.wire(
                                &mut c,
                                along,
                                &[off],
                                g + anchor_h,
                                anchor_h - bottom,
                            ));
                        }
                    }
                }
            } else {
                // a fork: one or two vines in each branch
                // (while the branches part, one each, out by its fence)
                // (the lanes are taken where each vine hangs: they move)
                for (k, &(m, _, _, _)) in lanes.iter().enumerate() {
                    if k == 0 && heli.is_some() {
                        here.push(b.helicopter(&mut c, int(s), m));
                        continue;
                    }
                    let along = int(s + b.dice.range(-3, 3));
                    let two = b.dice.below(2) == 0;
                    let first = if b.dice.below(2) == 0 { -1 } else { 1 };
                    let here_lanes = lanes_at(&c, along);
                    let (m, h, lo_tree, hi_tree) = here_lanes[k.min(here_lanes.len() - 1)];
                    // (9 m in from an edge, for the branch from its tree)
                    let inset = (h - int(9)).max(int(4));
                    let spots = match (lo_tree, hi_tree) {
                        (true, true) if two => vec![m - inset, m + inset],
                        (true, true) => vec![m + first * inset],
                        (true, false) => vec![m - h + int(9)],
                        _ => vec![m + h - int(9)],
                    };
                    for spot in spots {
                        let off = spot + int(b.dice.range(-1, 1));
                        let anchor_h = int(b.dice.range(38, 50));
                        let bottom = anchor_h - int(b.dice.range(23, 31));
                        // (or the other side of the branch, by its fence,
                        // where its island is kept clear)
                        let v = b
                            .vine(&mut c, along, off, anchor_h, bottom, false)
                            .or_else(|| {
                                b.vine(&mut c, along, 2 * m - off, anchor_h, bottom, false)
                            });
                        here.extend(v);
                    }
                }
            }
        }
        stations.push(here);
        // a low vine to climb back up by, in each lane
        if station % 2 == 1 {
            let lanes = lanes_at(&c, int(s + 7));
            for &(m, h, lo_tree, hi_tree) in &lanes {
                let sign = match (lo_tree, hi_tree) {
                    (true, false) => -1,
                    (false, true) => 1,
                    _ if b.dice.below(2) == 0 => -1,
                    _ => 1,
                };
                // (9–12 m in from the edge its tree stands by)
                let reach = (h - int(b.dice.range(9, 12))).max(0);
                let length = int(24) + cm(b.dice.range(0, 40));
                let (at, off) = (int(s + 7), m + sign * reach);
                if b.vine(&mut c, at, off, int(27), int(27) - length, true)
                    .is_none()
                {
                    b.vine(&mut c, at, 2 * m - off, int(27), int(27) - length, true);
                }
            }
        }
        station += 1;
        s += b.dice.range(22, 28);
    }

    banana_arcs(&mut c, &stations);

    // item crates: rows across the run about every 240 m, one in the
    // middle of each fork (in both branches)
    for at in CRATES {
        let si = int(at);
        let offs: Vec<i32> = match lanes_at(&c, si)[..] {
            [(m, _, _, _)] => vec![m - int(9), m, m + int(9)],
            ref two => two
                .iter()
                .flat_map(|&(m, _, _, _)| [m - int(4), m + int(4)])
                .collect(),
        };
        for off in offs {
            let p = pt(&c, si, off);
            let gy = c.terrain(p.x, p.z);
            c.crates.push(v3(p.x, gy + int(b.dice.range(16, 24)), p.z));
        }
    }

    island_trees(&c, &mut b);
    b.lay.landmarks = landmarks(&c);
    c.ski = Some(Box::new(b.lay));
    c
}

/// Conifers on the forks' islands, clear of the landmarks standing there.
fn island_trees(c: &Course, b: &mut Build) {
    let marks = landmarks(c);
    for f in FORKS {
        let mut s = f.from + f.ramp / 2;
        while s < f.to - f.ramp / 3 {
            let [(m0, h0), (m1, h1)] = c.lanes(s);
            {
                let (lo, hi) = (m0 + h0 + int(3), m1 - h1 - int(3));
                if hi > lo {
                    let off = lo + mul(hi - lo, b.dice.range(0, 100) * ONE / 100);
                    let p = pt(c, s, off);
                    let clear = marks.iter().all(|m| m.at.flat().dist(p) > int(16));
                    if clear && !b.in_clearing(s) {
                        let y = c.terrain(p.x, p.z) + int(b.dice.range(10, 26));
                        b.tree(c, p, y, None);
                    }
                }
            }
            s += int(b.dice.range(6, 11));
        }
    }
}

/// One of each landmark, in order down the hill.
fn landmarks(c: &Course) -> Vec<Landmark> {
    // the island's middle, across
    let island = |s: i32| {
        let [(m0, h0), (m1, h1)] = c.lanes(s);
        ((m0 + h0) + (m1 - h1)) / 2
    };
    let mark = |kind, s: i32, off: i32| {
        let at = on_ground(c, pt(c, s, off));
        // facing up the run, toward the apes coming
        let look = c.point(s - int(60)).flat().sub(at.flat());
        Landmark {
            kind,
            at,
            yaw: yaw_of(look),
            s: s.clamp(0, c.length),
        }
    };
    use LandmarkKind::*;
    vec![
        // behind the start, clear of the chase camera there
        mark(Summit, -int(46), 0),
        mark(Snowman, int(400), island(int(400))),
        mark(Lodge, int(700), -int(72)),
        mark(SkiJump, int(890), int(58)),
        mark(IceApe, int(1150), island(int(1150))),
        mark(FrozenFalls, int(1420), -int(56)),
        mark(Village, c.length + int(150), 0),
    ]
}

#[cfg(test)]
mod tests;
