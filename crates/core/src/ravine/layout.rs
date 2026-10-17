//! Rattlesnake Ravine's course (the course agent, 2026-10-17): a winding
//! slot canyon 1.74 km long, laid only from spacings the AI already climbs
//! (MAPS "Getting the AI round a new map"): a station every 22–28 m, two or
//! three vines across the lanes, a low vine every other station, a
//! station 14 m short of each tight corner and 5 m past it (the Ruins'
//! corner pins), roofed stretches hung as the Ruins' corridor.
//!
//! In order (metres along the spine, `Course.path`):
//! - **The start** (0–180): the Jungle's start platform and its row of
//!   vines, the canyon closed 24 m behind it; a braid of low rock ribs.
//! - **The bends** (180–510): a long bend left under an arch, a bend right
//!   through a cave.
//! - **The hairpin** (510–850): a tight corner right, then the in-leg, two
//!   tight corners left round a 25 m rock fin, and the out-leg back beside
//!   the in-leg. Halfway up the fin's wall in the skull cave the T-rex
//!   skull's mouth goes through it to the out-leg: the optional shortcut,
//!   some 173 m shorter (`SHORTCUT_SAVES`, 5–8 s at the AI's pace).
//! - **The split** (1000–1360): the spine (the lower passage) swings left
//!   and back through the mine tunnel; the upper passage (`Split.upper`)
//!   runs straight under the sky, 36 m shorter, with sparser vines and more
//!   snakes. A rock island stands between them where they are far enough
//!   apart.
//! - **The mesa** (1400–1742): a bend left (a second braid), a second
//!   arch, a tight corner left, and the canyon opens onto the mesa: the
//!   finish at the sphinx's feet, the campfire and the tents.
//!
//! Snake vines (`Hanger.snake`) come from a fixed rule on the station's
//! number, never the dice: about 15% of the vines, more on the upper
//! passage, never a station's only vine and never a low vine.
//!
//! New placement draws from dice of its own (`seed ^ DICE`), so no other
//! map changes.

use super::solids::{WALL_ABOVE, WALL_BELOW};
use super::*;
use crate::course::{
    banana_arcs, centreline, ground, Course, Leg, Mushroom, Platform, Vine, START_HEIGHT,
    START_LEN, STEP,
};
use crate::fx::{int, Dice};
use crate::geom::{right_of, yaw_of, Affine};
use crate::rules::{MUSHROOM_HEIGHT, MUSHROOM_RADIUS};

const SEED: u64 = 0x5A4E_C0DE_0001;
/// This map's own dice (MAPS "Layout lessons": no other map's draws move).
const DICE: u64 = 0x5241_5649_4E45;

/// The canyon's half width (the corridor's: `Course.half_width`).
pub const HALF: i32 = int(20);
/// How far behind `path[0]` the canyon is closed (no solid within 16 m
/// behind the start, MAPS gotcha 5).
pub const START_BACK: i32 = int(24);
/// The mine tunnel's roof above its floor: the Ruins' corridor's, its vines
/// hung as that corridor's (18–21 m, bottoms 11–14 m up, lanes 9 m apart).
pub const TUNNEL_HEIGHT: i32 = int(32);
/// A cave's roof above its floor (its vines hang at the Jungle's highest
/// anchors).
pub const CAVE_HEIGHT: i32 = int(50);
/// An arch's underside above the floor (over every anchor).
pub const ARCH_HEIGHT: i32 = int(58);
/// The skull's mouth (the middle of its gap) above the floor: halfway up
/// the skull cave's wall.
pub const MOUTH_UP: i32 = int(25);
/// The skull stands this far short of the hairpin, and its hole comes out
/// as far past it: the shortcut saves twice this plus the hairpin's 88 m
/// less the 65 m across (`the_skull_saves_five_to_eight_seconds`).
pub const SKULL_SHORT: i32 = 75;
/// The AI's race pace on this course for the shortcut's estimate, m/s:
/// King Kong's (1,742 m in a mean 72.6 s over six layouts, SECOND-ORDER
/// RC4).
pub const PACE: i32 = 24;
/// The braids' boulders: how high (their tops are floors).
pub const RIB_HEIGHT: i32 = int(5);
/// A cart's bouncer stands this high over its rails.
const CART_TOP: i32 = cm(200);

/// Where things are along a route (metres along its spine).
pub(crate) struct Plan {
    pub legs: Vec<Leg>,
    /// stations pinned here (corners, the tunnel's mouths)
    pub pins: Vec<i32>,
    /// the start of every leg (and the end)
    pub marks: Vec<i32>,
}

/// Leg numbers in `Plan.marks`.
pub(crate) mod leg {
    pub const BEND_LEFT: usize = 2;
    pub const BEND_RIGHT: usize = 3;
    pub const HAIRPIN: usize = 7;
    pub const OUT_LEG: usize = 10;
    pub const OUT_BEND: usize = 11;
    pub const FORK: usize = 13;
    pub const SWING: usize = 14;
    pub const REJOIN: usize = 16;
    pub const MESA_BEND: usize = 17;
    pub const ARCH: usize = 18;
    pub const MESA: usize = 20;
}

/// The legs; `upper` replaces the lower passage's swing with the upper
/// passage's straight (the same place on the ground: the swing is
/// symmetric, so it ends where the straight does, heading the same way).
pub(crate) fn plan(upper: bool) -> Plan {
    let mut legs = vec![
        Leg(60, 0),    // 0 the start
        Leg(120, 0),   // 1 a braid
        Leg(140, 45),  // 2 left, under the first arch
        Leg(150, -70), // 3 right, through the first cave
        Leg(40, 0),    // 4
        Leg(32, -90),  // 5 a tight corner right
        Leg(110, 0),   // 6 the in-leg: the skull cave
        Leg(48, 90),   // 7 the hairpin: left (R 31 m: the fin's tip round, 10 m)
        Leg(4, 0),     // 8
        Leg(48, 90),   // 9 and left again
        Leg(110, 0),   // 10 the out-leg
        Leg(120, -60), // 11 right
        Leg(30, 0),    // 12 to the fork
    ];
    if upper {
        // the swing's chord: 4 × 114.6 m × sin 45°
        legs.extend([Leg(324, 0), Leg(0, 0), Leg(0, 0)]);
    } else {
        legs.extend([Leg(90, 45), Leg(180, -90), Leg(90, 45)]); // 13–15
    }
    legs.extend([
        Leg(40, 0),   // 16 from the rejoin
        Leg(140, 50), // 17 left, a braid
        Leg(60, 0),   // 18 the second arch
        Leg(32, 90),  // 19 a tight corner left
        Leg(110, 0),  // 20 out onto the mesa
    ]);
    let mut marks = Vec::new();
    let mut s = 0;
    for l in &legs {
        marks.push(s);
        s += l.0;
    }
    marks.push(s);
    let m = |k: usize| marks[k];
    let mut pins = Vec::new();
    for corner in [5, 7, 9, 19] {
        pins.push(m(corner) - 14);
        pins.push(m(corner + 1) + 5);
    }
    // the hairpin: its two corners are long (48 m, R 31 m) and 4 m apart;
    // one station between them in place of the corners' own pins
    pins.retain(|&p| p != m(8) + 5 && p != m(9) - 14);
    pins.push(m(8) + 2);
    if !upper {
        let (a, b) = tunnel_span(&marks);
        pins.push(a - 6);
        pins.push(b + 6);
    }
    pins.sort();
    Plan { legs, pins, marks }
}

/// The mine tunnel's mouths on the lower passage: the swing's middle
/// 120 m, where the island between the passages is wide enough.
fn tunnel_span(marks: &[i32]) -> (i32, i32) {
    let mid = (marks[leg::SWING] + marks[leg::SWING + 1]) / 2;
    (mid - 60, mid + 60)
}

/// Where the skull stands along the spine, and where its hole comes out.
pub(crate) fn skull_spots(p: &Plan) -> (i32, i32) {
    (
        p.marks[leg::HAIRPIN] - SKULL_SHORT,
        p.marks[leg::OUT_LEG] + SKULL_SHORT,
    )
}

/// Rattlesnake Ravine's course, from its fixed seed.
pub fn ravine() -> Course {
    ravine_seeded(SEED)
}

/// Rattlesnake Ravine's course from `seed`.
pub fn ravine_seeded(seed: u64) -> Course {
    build(seed, false)
}

/// The same course with the upper passage as its spine (where the lower
/// passage's swing was, a straight): for measuring the AI along the upper
/// passage (`examples/ravine_report.rs`). Not a map.
pub fn ravine_upper_seeded(seed: u64) -> Course {
    build(seed, true)
}

/// A route a station line follows: a point and the way on at `u` metres.
trait Line {
    fn at(&self, u: i32) -> (V3, V3);
}

struct Spine<'a>(&'a Course);

impl Line for Spine<'_> {
    fn at(&self, u: i32) -> (V3, V3) {
        (self.0.point(int(u)), self.0.heading_at(int(u)))
    }
}

/// A straight from `from` heading `dir` (the upper passage).
struct Chord {
    from: V3,
    dir: V3,
}

impl Line for Chord {
    fn at(&self, u: i32) -> (V3, V3) {
        (self.from.add(self.dir.scale(int(u))), self.dir)
    }
}

/// What a station is like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// the open canyon: vines from bridges, logs and ledges
    Open,
    /// under a cave's roof (`rooms[room]`)
    Cave { room: usize },
    /// in the mine tunnel (`rooms[room]`)
    Tunnel { room: usize },
    /// the upper passage: sparse, more snakes
    Upper,
}

/// Where a vine hangs and what from, as it is placed.
struct Placing {
    vines: Vec<Vine>,
    hangers: Vec<Hanger>,
}

impl Placing {
    fn push(&mut self, anchor: V3, length: i32, at: i32, from: HangFrom, snake: bool) -> usize {
        self.vines.push(Vine {
            anchor,
            length,
            slide: None,
            at,
        });
        self.hangers.push(Hanger {
            from,
            at: anchor,
            snake,
            snapped_at: None,
        });
        self.vines.len() - 1
    }
}

/// The ground at `p`.
fn land(p: V3) -> i32 {
    ground(p.x, p.z)
}

fn on_ground(p: V3) -> V3 {
    v3(p.x, land(p), p.z)
}

/// Station positions from `from` to `to` (metres along a line): even hops
/// of about `hop` m (never over 28) between the pins, a metre of jitter.
fn stations(from: i32, to: i32, pins: &[i32], hop: i32, dice: &mut Dice) -> Vec<i32> {
    let mut fixed = vec![from];
    fixed.extend(
        pins.iter()
            .copied()
            .filter(|&p| p > from + 12 && p < to - 12),
    );
    fixed.push(to);
    let mut out = Vec::new();
    for w in fixed.windows(2) {
        let gap = w[1] - w[0];
        let n = ((gap + hop / 2) / hop).max((gap + 27) / 28).max(1);
        for k in 0..n {
            let base = w[0] + gap * k / n;
            let jitter = if k == 0 { 0 } else { dice.range(-1, 1) };
            out.push(base + jitter);
        }
    }
    out.push(to);
    out
}

/// Whether station `k` (counted along its line) of `kind` puts a snake on
/// one side lane: a fixed rule (no dice), about 15% of all vines.
fn snake_station(kind: Kind, k: usize) -> bool {
    match kind {
        Kind::Upper => k % 2 == 1,
        _ => k % 5 == 1 || k % 5 == 3,
    }
}

fn build(seed: u64, upper_spine: bool) -> Course {
    let plan = plan(upper_spine);
    let mut c = centreline("ravine", &plan.legs);
    let mut dice = Dice(seed ^ DICE);
    let len = c.length / ONE;
    let m = |k: usize| plan.marks[k];
    let (skull_s, exit_s) = skull_spots(&plan);
    let has_skull = !upper_spine;

    // the start: the Jungle's platform, starters and row of vines
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
        let at = c.point(int(2)).add(right.scale(int(-6) + k * int(4)));
        c.starts[k as usize] = v3(at.x, START_HEIGHT + BODY_DROP, at.z);
    }
    c.start_yaw = 0;

    // ---- the rooms: arches, caves, the tunnel ----
    let mut rooms = Vec::new();
    let room = |c: &Course, kind, a: i32, b: i32, height| Room {
        kind,
        from: on_ground(c.point(int(a))),
        to: on_ground(c.point(int(b))),
        half_width: HALF + int(2),
        height,
    };
    for a in [m(leg::BEND_LEFT) + 70, m(leg::ARCH) + 30] {
        rooms.push(room(&c, RoomKind::Arch, a - 7, a + 7, ARCH_HEIGHT));
    }
    // the first cave, on the bend right (in short straight boxes, so the
    // camera keeps inside), and the skull caves on the in-leg
    let mut roofed = Vec::new();
    let cave_spans = [
        (m(leg::BEND_RIGHT) + 50, m(leg::BEND_RIGHT) + 100),
        // either side of the skull: a cave's side walls are solid, and
        // the mouth must open through the fin (the skull stands in the
        // gap between the two, halfway up their wall)
        (skull_s - 40, skull_s - 8),
        (skull_s + 8, skull_s + 40),
    ];
    for &(a, b) in &cave_spans {
        let mut s = a;
        while s < b {
            let e = (s + 25).min(b);
            roofed.push(rooms.len());
            rooms.push(room(&c, RoomKind::Cave, s, e, CAVE_HEIGHT));
            s = e;
        }
    }
    let tspan = (!upper_spine).then(|| tunnel_span(&plan.marks));
    if let Some((a, b)) = tspan {
        let mut s = a;
        while s < b {
            let e = (s + 26).min(b);
            roofed.push(rooms.len());
            rooms.push(room(&c, RoomKind::Tunnel, s, e, TUNNEL_HEIGHT));
            s = e;
        }
    }
    let kinds: Vec<Kind> = (0..=len)
        .map(|s| {
            let p = c.point(int(s));
            for &i in &roofed {
                let r: &Room = &rooms[i];
                if r.contains(v3(p.x, r.from.y + int(1), p.z)) {
                    return match r.kind {
                        RoomKind::Tunnel => Kind::Tunnel { room: i },
                        _ => Kind::Cave { room: i },
                    };
                }
            }
            if upper_spine && s > m(leg::FORK) && s < m(leg::REJOIN) {
                return Kind::Upper;
            }
            Kind::Open
        })
        .collect();
    let kind_of = |s: i32| kinds[s.clamp(0, len) as usize];

    // ---- the vines ----
    let mut put = Placing {
        vines: Vec::new(),
        hangers: Vec::new(),
    };
    let mut decks = Vec::new();
    for k in -3..=3 {
        let at = START_LEN + cm(150);
        let base = c
            .point(at)
            .add(right_of(c.heading_at(at)).scale(int(3) * k));
        put.push(
            v3(base.x, START_HEIGHT + int(30), base.z),
            int(29),
            at,
            HangFrom::Ledge {
                wall: if k < 0 { 0 } else { 1 },
            },
            false,
        );
    }
    let skull_at = c.point(int(skull_s));
    // nothing hangs on the fin's side in front of the teeth
    let near_skull = |p: V3| has_skull && p.flat().dist(skull_at.flat()) < int(16);
    let mut all_stations: Vec<Vec<usize>> = Vec::new();
    let mut mushrooms = Vec::new();
    let spine_at = stations(32, len - 30, &plan.pins, 25, &mut dice);
    lay_line(
        &Spine(&c),
        &spine_at,
        &kind_of,
        &|u| u,
        &rooms,
        &mut Lay {
            put: &mut put,
            decks: &mut decks,
            all: &mut all_stations,
            mushrooms: &mut mushrooms,
            dice: &mut dice,
        },
        &near_skull,
        false,
    );
    // the upper passage, on the real course: its own stations between the
    // fork and the rejoin (the passages are one at either end)
    let (fork, rejoin) = (m(leg::FORK), m(leg::REJOIN));
    let split = (!upper_spine).then(|| {
        let a = c.point(int(fork));
        let b = c.point(int(rejoin));
        let dir = b.sub(a).flat().norm();
        let chord_len = b.sub(a).flat().len() / ONE;
        let chord = Chord { from: a, dir };
        // (from where the passages are 24 m apart: nearer, the spine's
        // stations serve both, and the decks would cross)
        let apart = |u: i32| {
            let p = chord.at(u).0;
            c.progress(p, c.nearest_anywhere(p)).1.abs() >= int(24)
        };
        let u0 = (0..chord_len).find(|&u| apart(u)).unwrap_or(0);
        let u1 = (0..chord_len)
            .rev()
            .find(|&u| apart(u))
            .unwrap_or(chord_len);
        let at_u = stations(u0, u1, &[], 27, &mut dice);
        let sp: Vec<i32> = at_u
            .iter()
            .map(|&u| {
                let p = chord.at(u).0;
                c.progress(p, c.nearest_anywhere(p)).0 / ONE
            })
            .collect();
        lay_line(
            &chord,
            &at_u,
            &|_| Kind::Upper,
            &|u| sp[at_u.iter().position(|&x| x == u).unwrap_or(0)],
            &rooms,
            &mut Lay {
                put: &mut put,
                decks: &mut decks,
                all: &mut all_stations,
                mushrooms: &mut mushrooms,
                dice: &mut dice,
            },
            &near_skull,
            true,
        );
        let mut upper: Vec<V3> = (0..=chord_len / 8)
            .map(|k| on_ground(chord.at(k * 8).0))
            .collect();
        upper.push(on_ground(b));
        Split {
            fork: (int(fork) / STEP) as usize,
            rejoin: (int(rejoin) / STEP) as usize,
            upper,
        }
    });
    c.vines = put.vines;
    c.mushrooms = mushrooms;
    banana_arcs(&mut c, &all_stations);

    // item crates: rows of three across the canyon every 240 m, never in a
    // roofed stretch
    let mut at = 150;
    while at < len - 60 {
        if matches!(kind_of(at), Kind::Open | Kind::Upper) {
            let h = c.heading_at(int(at));
            let r = right_of(h);
            let base = c.point(int(at));
            for off in [-9, 0, 9] {
                let p = base.add(r.scale(int(off)));
                c.crates
                    .push(v3(p.x, land(p) + int(dice.range(16, 24)), p.z));
            }
        }
        at += 240;
    }

    let side_pt = |c: &Course, s: i32, off: i32, up: i32| {
        let p = c
            .point(int(s))
            .add(right_of(c.heading_at(int(s))).scale(off));
        v3(p.x, land(p) + up, p.z)
    };
    let side_out = |c: &Course, s: i32, sign: i32| {
        let r = right_of(c.heading_at(int(s)));
        if sign > 0 {
            r.neg()
        } else {
            r
        }
    };

    // ---- the walls and the skull ----
    let walls = walls(&c, &plan, split.as_ref(), has_skull);
    let skull = has_skull.then(|| skull(&c, skull_s, exit_s));

    // ---- lizards walking the walls (bouncers that move) ----
    let mut lizards = Vec::new();
    let colours = [LizardColour::Red, LizardColour::Yellow, LizardColour::Blue];
    let spots = [
        (130, -1),
        (290, 1),
        (455, -1),
        (m(leg::OUT_LEG) + 40, -1),
        (m(leg::OUT_BEND) + 105, -1),
        (m(leg::REJOIN) + 40, -1),
        (m(leg::MESA_BEND) + 120, 1),
        (m(leg::MESA) + 8, -1),
    ];
    for (k, &(s, sign)) in spots.iter().enumerate() {
        if kind_of(s) != Kind::Open {
            continue;
        }
        let face = side_pt(&c, s, sign * (HALF - cm(40)), 0);
        let out = side_out(&c, s, sign);
        let from = face.add(v3(0, int(3), 0));
        let to = face.add(v3(0, int(16 + (k as i32 % 3) * 2), 0));
        let bouncer = c.mushrooms.len();
        c.mushrooms.push(Mushroom {
            top: from.add(out.scale(int(1))).add(v3(0, cm(60), 0)),
            radius: MUSHROOM_RADIUS,
        });
        lizards.push(Lizard {
            from,
            to,
            out,
            period: crate::rules::secs_to_ticks(700 + 90 * (k as i32 % 4)) as u32,
            phase: (k as u32 * 377) % 500,
            bouncer,
            colour: colours[k % 3],
        });
    }

    // ---- mine tracks out of holes in the walls, carts on them ----
    let mut tracks = Vec::new();
    let mut carts = Vec::new();
    let mut track = |c: &mut Course, pts: Vec<V3>, outs: (V3, V3), period: i32, n: u32| {
        let hole = |at: V3, out: V3| Hole {
            at,
            out,
            half_width: int(2),
            height: int(3),
        };
        let t = Track {
            start: hole(pts[0], outs.0),
            end: hole(pts[pts.len() - 1], outs.1),
            points: pts,
        };
        let ticks = crate::rules::secs_to_ticks(period) as u32;
        for k in 0..n {
            let cart = Cart {
                track: tracks.len(),
                period: ticks,
                phase: ticks * k / n,
                bouncer: c.mushrooms.len(),
            };
            c.mushrooms.push(Mushroom {
                top: cart_start(&t, &cart).add(v3(0, CART_TOP, 0)),
                radius: cm(170),
            });
            carts.push(cart);
        }
        tracks.push(t);
    };
    // across the canyon on the bend after the out-leg, left wall to
    // right, rising
    {
        let (a, b) = (m(leg::OUT_BEND) + 20, m(leg::OUT_BEND) + 90);
        let pts = vec![
            side_pt(&c, a, -HALF, int(4)),
            side_pt(&c, a + 25, -int(8), int(6)),
            side_pt(&c, b - 25, int(8), int(9)),
            side_pt(&c, b, HALF, int(11)),
        ];
        let outs = (side_out(&c, a, -1), side_out(&c, b, 1));
        track(&mut c, pts, outs, 900, 2);
    }
    // along the right wall of the out-leg, falling
    {
        let (a, b) = (m(leg::OUT_LEG) + 20, m(leg::OUT_LEG) + 100);
        let mut pts: Vec<V3> = (0..=4)
            .map(|k| side_pt(&c, a + (b - a) * k / 4, HALF - int(3), int(8) - int(k)))
            .collect();
        pts[0] = side_pt(&c, a, HALF, int(8));
        pts[4] = side_pt(&c, b, HALF, int(4));
        let outs = (side_out(&c, a, 1), side_out(&c, b, 1));
        track(&mut c, pts, outs, 1100, 2);
    }
    // in the mine tunnel, along its left wall
    if let Some((a, b)) = tspan {
        let mut pts = vec![side_pt(&c, a + 6, -HALF, int(3))];
        let mut s = a + 20;
        while s < b - 10 {
            pts.push(side_pt(&c, s, -HALF + int(3), int(3)));
            s += 20;
        }
        pts.push(side_pt(&c, b - 6, -HALF, int(3)));
        let outs = (side_out(&c, a, -1), side_out(&c, b, -1));
        track(&mut c, pts, outs, 1400, 3);
    }
    // across the mesa straight, right wall to left, falling
    {
        let (a, b) = (m(leg::MESA) + 20, m(leg::MESA) + 80);
        let pts = vec![
            side_pt(&c, a, HALF, int(10)),
            side_pt(&c, a + 30, 0, int(7)),
            side_pt(&c, b, -HALF, int(4)),
        ];
        let outs = (side_out(&c, a, 1), side_out(&c, b, -1));
        track(&mut c, pts, outs, 800, 1);
    }

    // ---- large rocks (solid) by the walls, clear of every vine; and the
    // braids: boulders down the middle between stations, on the first
    // straight and the bend after the rejoin, the strands either side ----
    let mut rocks = Vec::new();
    let braids = [(50, 180), (m(leg::MESA_BEND), m(leg::ARCH))];
    for w in spine_at.windows(2) {
        let (a, b) = (w[0] + 9, w[1] - 7);
        if b - a < 6 || !braids.iter().any(|&(x, y)| a >= x && b <= y) {
            continue;
        }
        rocks.push(Rock {
            at: side_pt(&c, (a + b) / 2, 0, 0),
            radius: int(3),
            height: RIB_HEIGHT,
        });
    }
    let mut s = 70;
    let mut k = 0;
    while s < len - 40 {
        let sign = if k % 2 == 0 { 1 } else { -1 };
        let at = side_pt(&c, s, sign * int(15), 0);
        let radius = int(2) + cm(dice.range(0, 150));
        let height = int(2) + cm(dice.range(0, 300));
        let clear = c
            .vines
            .iter()
            .all(|v| v.anchor.flat().dist(at.flat()) > radius + int(6))
            && kind_of(s) == Kind::Open;
        if clear {
            rocks.push(Rock { at, radius, height });
        }
        k += 1;
        s += dice.range(80, 110);
    }

    // ---- glows: torches (lanterns in the tunnel), glowing mushrooms and
    // fireflies, closer together toward the finish ----
    let mut glows = Vec::new();
    let mut s = 20;
    let mut k = 0;
    while s < len + 60 {
        let sign = if k % 2 == 0 { -1 } else { 1 };
        let in_tunnel = matches!(kind_of(s), Kind::Tunnel { .. });
        glows.push(Glow {
            at: side_pt(&c, s, sign * (HALF - cm(50)), int(7)),
            kind: if in_tunnel {
                GlowKind::Lantern
            } else {
                GlowKind::Torch
            },
        });
        if k % 2 == 0 {
            glows.push(Glow {
                at: side_pt(&c, s + 4, -sign * (HALF - int(1)), cm(20)),
                kind: GlowKind::Mushroom,
            });
        }
        if k % 3 == 1 && !in_tunnel {
            glows.push(Glow {
                at: side_pt(&c, s + 8, sign * int(5), int(22)),
                kind: GlowKind::Fireflies,
            });
        }
        // from 40 m apart at the start to 18 m at the finish
        s += 40 - 22 * s.min(len) / len;
        k += 1;
    }
    if let Some(sp) = &split {
        for p in sp.upper.iter().skip(2).step_by(4) {
            glows.push(Glow {
                at: p.add(v3(0, int(18), 0)),
                kind: GlowKind::Fireflies,
            });
        }
    }

    // ---- the mesa finish ----
    let end = c.point(c.length);
    let h = c.heading_at(c.length);
    let r = right_of(h);
    // big enough to read at race distance (the models' own sizes are a
    // person's: `CAMPFIRE_SCALE`, `TENT_SCALE`), round the sphinx
    let campfire = on_ground(end.add(h.scale(int(18))).add(r.scale(int(22))));
    // the four roasting sticks' feet round the fire, where the model's own
    // sticks stand (1.7 m out at the model's size)
    let sticks = (0..4)
        .map(|k| {
            let a = k * fx::TURN / 4 + fx::TURN / 8;
            let out = fx::mul(cm(170), CAMPFIRE_SCALE);
            on_ground(campfire.add(v3(fx::mul(fx::sin(a), out), 0, fx::mul(fx::cos(a), out))))
        })
        .collect();
    let tents = [(-30, 20), (-42, 40), (40, 44), (26, 64)]
        .iter()
        .map(|&(x, z)| {
            let p = on_ground(end.add(h.scale(int(z))).add(r.scale(int(x))));
            (p, yaw_of(campfire.sub(p)))
        })
        .collect();
    let finish = Finish {
        sphinx: on_ground(end.add(h.scale(int(26)))),
        sphinx_yaw: yaw_of(h.neg()),
        campfire,
        sticks,
        tents,
    };

    c.ravine = Some(Box::new(RavineLayout {
        walls,
        rooms,
        decks,
        hangers: put.hangers,
        lizards,
        tracks,
        carts,
        skull,
        split,
        rocks,
        glows,
        finish,
        lobby: (c.starts[0], c.start_yaw),
    }));
    c
}

/// Where `cart` on track `t` is at tick 0 (before the layout exists).
fn cart_start(t: &Track, cart: &Cart) -> V3 {
    let probe = RavineLayout {
        walls: Vec::new(),
        rooms: Vec::new(),
        decks: Vec::new(),
        hangers: Vec::new(),
        lizards: Vec::new(),
        tracks: vec![t.clone()],
        carts: vec![Cart { track: 0, ..*cart }],
        skull: None,
        split: None,
        rocks: Vec::new(),
        glows: Vec::new(),
        finish: Finish {
            sphinx: V3::ZERO,
            sphinx_yaw: 0,
            campfire: V3::ZERO,
            sticks: Vec::new(),
            tents: Vec::new(),
        },
        lobby: (V3::ZERO, 0),
    };
    probe.cart_at(0, 0).0
}

/// What laying a line of stations adds to.
struct Lay<'a> {
    put: &'a mut Placing,
    decks: &'a mut Vec<Deck>,
    all: &'a mut Vec<Vec<usize>>,
    mushrooms: &'a mut Vec<Mushroom>,
    dice: &'a mut Dice,
}

/// Lays the stations at `at` (metres along `line`) as their kind says.
/// `spine_s` gives a station's place along the spine (`Vine.at`).
#[allow(clippy::too_many_arguments)]
fn lay_line(
    line: &dyn Line,
    at: &[i32],
    kind_of: &dyn Fn(i32) -> Kind,
    spine_s: &dyn Fn(i32) -> i32,
    rooms: &[Room],
    out: &mut Lay,
    near_skull: &dyn Fn(V3) -> bool,
    upper_line: bool,
) {
    let dice = &mut *out.dice;
    for (station, &u) in at.iter().enumerate() {
        let kind = kind_of(u);
        let (p, h) = line.at(u);
        let r = right_of(h);
        let s_here = spine_s(u);
        let first = station == 0 && !upper_line;
        // a snake station has all three lanes, the snake on a side one
        // (never the middle, never the only vine there)
        // (and by the skull: no left lane there, and no snake)
        let by_skull = near_skull(p.sub(r.scale(int(10))));
        let snaky = !first && !by_skull && snake_station(kind, station);
        let snake_lane = snaky.then_some(if (station / 3) % 2 == 0 { 0 } else { 2 });
        // (the upper passage's snake stations: both side lanes, the middle
        // and the low vine plain)
        let snakes_both = snaky && kind == Kind::Upper;
        // which lanes: two of three, now and then all three (the upper
        // passage: the middle and one side, sparse)
        let skip = match kind {
            _ if snaky => 3,
            _ if by_skull => 0,
            Kind::Upper => {
                if station % 4 == 0 {
                    0
                } else {
                    2
                }
            }
            _ if first || dice.below(6) == 0 => 3,
            _ => dice.below(3) as i32,
        };
        let mut here = Vec::new();
        let mut lanes = Vec::new();
        // an open station's vines hang from a bridge or a log across the
        // canyon two times in three, else from ledges on the walls
        let deck_i =
            (!first && matches!(kind, Kind::Open | Kind::Upper) && station % 3 != 2).then(|| {
                let skew = int(dice.range(-3, 3));
                let lo = int(dice.range(38, 50));
                let hi = int(dice.range(38, 50));
                let a = p.sub(r.scale(HALF + int(3))).sub(h.scale(skew));
                let b = p.add(r.scale(HALF + int(3))).add(h.scale(skew));
                let kind = if station % 2 == 0 {
                    DeckKind::Bridge
                } else {
                    DeckKind::Log
                };
                out.decks.push(Deck {
                    kind,
                    from: v3(a.x, land(a) + lo, a.z),
                    to: v3(b.x, land(b) + hi, b.z),
                    half_width: if kind == DeckKind::Bridge {
                        int(1)
                    } else {
                        cm(80)
                    },
                });
                out.decks.len() - 1
            });
        for lane in 0..3 {
            if lane == skip {
                continue;
            }
            let spread = if matches!(kind, Kind::Tunnel { .. }) {
                9
            } else {
                10
            };
            let off = if first {
                (lane - 1) * 6
            } else {
                (lane - 1) * spread + dice.range(-2, 2)
            };
            let base = p.add(r.scale(int(off)));
            if near_skull(base) && off < -5 {
                continue;
            }
            let (anchor, length, from) = match (kind, deck_i) {
                (_, Some(d)) => {
                    // on the deck's line, under it
                    let dk = out.decks[d];
                    let t = fx::div(int(off) + HALF + int(3), 2 * (HALF + int(3)));
                    let b = dk.from.lerp(dk.to, t);
                    let y = b.y - cm(40);
                    (
                        v3(b.x, y, b.z),
                        int(dice.range(23, 31)).min(y - land(b) - int(7)),
                        HangFrom::Deck { deck: d },
                    )
                }
                (Kind::Cave { .. } | Kind::Tunnel { .. }, None)
                    if room_over(rooms, base).is_some() =>
                {
                    let room = room_over(rooms, base).unwrap_or(0);
                    let rm = rooms[room];
                    let roof = roof_at(&rm, base);
                    let length = if matches!(kind, Kind::Tunnel { .. }) {
                        int(dice.range(18, 21))
                    } else {
                        int(dice.range(23, 31))
                    };
                    (v3(base.x, roof, base.z), length, HangFrom::Roof { room })
                }
                _ => {
                    let (anchor_h, length) = if first {
                        (int(46), int(30))
                    } else {
                        (int(dice.range(38, 50)), int(dice.range(23, 31)))
                    };
                    (
                        v3(base.x, land(base) + anchor_h, base.z),
                        length,
                        HangFrom::Ledge {
                            wall: if off < 0 { 0 } else { 1 },
                        },
                    )
                }
            };
            let snake = snake_lane == Some(lane) || (snakes_both && lane != 1);
            let i = out.put.push(anchor, length, int(s_here), from, snake);
            if !snake {
                here.push(i);
            }
            lanes.push(off);
        }
        out.all.push(here);
        // a low vine every other station, to climb back up by (never a
        // snake), off the braids' ribs; and at every snake station, on the
        // snake's side, under where a snapped ape lands (SECOND-ORDER RC7)
        if station % 2 == 1 || snaky {
            let side_of_snake = match snake_lane {
                Some(0) if !snakes_both => -1,
                Some(_) if !snakes_both => 1,
                _ => 0,
            };
            let off = if station % 2 == 1 {
                let off = dice.range(7, 9) * if dice.below(2) == 0 { -1 } else { 1 };
                if side_of_snake == 0 {
                    off
                } else {
                    off.abs() * side_of_snake
                }
            } else {
                8 * side_of_snake
            };
            let (b, _) = line.at(u + 7);
            let b = b.add(r.scale(int(off)));
            let (anchor, length, from) = match room_over(rooms, b) {
                Some(room) => {
                    let rm = rooms[room];
                    let roof = roof_at(&rm, b);
                    (
                        v3(b.x, roof, b.z),
                        roof - land(b)
                            - int(3)
                            - cm(if station % 2 == 1 {
                                dice.range(0, 40)
                            } else {
                                20
                            }),
                        HangFrom::Roof { room },
                    )
                }
                _ => (
                    v3(b.x, land(b) + int(27), b.z),
                    int(24)
                        + cm(if station % 2 == 1 {
                            dice.range(0, 40)
                        } else {
                            20
                        }),
                    HangFrom::Ledge {
                        wall: if off < 0 { 0 } else { 1 },
                    },
                ),
            };
            if !near_skull(b) {
                out.put.push(anchor, length, int(s_here + 7), from, false);
            }
        }
        if station % 4 == 2 && !lanes.is_empty() {
            // a bounce mushroom short of a side lane's vine
            let off = lanes[dice.below(lanes.len() as u32) as usize];
            let off = if off.abs() < 6 { 10 } else { off };
            let (q, _) = line.at(u - 11);
            let q = q.add(r.scale(int(off)));
            out.mushrooms.push(Mushroom {
                top: v3(q.x, land(q) + MUSHROOM_HEIGHT, q.z),
                radius: MUSHROOM_RADIUS,
            });
        }
    }
}

/// The cave or tunnel room over `p`, if any.
fn room_over(rooms: &[Room], p: V3) -> Option<usize> {
    rooms.iter().position(|m| {
        matches!(m.kind, RoomKind::Cave | RoomKind::Tunnel)
            && m.contains(v3(p.x, land(p) + int(1), p.z))
    })
}

/// The underside of room `m`'s roof over `p` (its floor slopes from one
/// mouth to the other, as `Room::contains` takes it).
fn roof_at(m: &Room, p: V3) -> i32 {
    let run = m.to.sub(m.from).flat();
    let len = run.len().max(1);
    let a = p.sub(m.from).flat().dot(run.norm()).clamp(0, len);
    m.from.y + fx::mul(m.to.y - m.from.y, fx::div(a, len)) + m.height
}

/// The skull at `s` along the in-leg, its mouth in the fin's wall halfway
/// up the skull cave, through the fin to `exit` along the out-leg.
fn skull(c: &Course, s: i32, exit: i32) -> Skull {
    let p = c.point(int(s));
    let h = c.heading_at(int(s));
    let left = right_of(h).neg();
    let face = p.add(left.scale(HALF));
    let at = v3(face.x, land(face) + MOUTH_UP, face.z);
    // the model's frame: its +z (the snout) back into the in-leg, the way
    // through along its -z (toward the out-leg)
    let turn = Affine::rot_y(yaw_of(left.neg())).at(at);
    let teeth = crate::ravinemodels::rex_teeth(MOUTH_GAP)
        .into_iter()
        .map(|t| Tooth {
            root: turn.apply(t.base),
            tip: turn.apply(t.tip),
            radius: t.r,
        })
        .collect();
    // the fin is on the out-leg's left too (the hairpin turns left twice)
    let q = c.point(int(exit));
    let out_left = right_of(c.heading_at(int(exit))).neg();
    let far = q.add(out_left.scale(HALF));
    let floor = at.y - MOUTH_GAP / 2;
    let keep = Room {
        kind: RoomKind::Skull,
        from: v3(at.x, floor, at.z).add(left.scale(-int(7))),
        to: v3(far.x, floor, far.z),
        half_width: int(5),
        height: MOUTH_GAP,
    };
    Skull {
        at,
        yaw: yaw_of(left),
        teeth,
        keep,
        // (the hands' place: `BODY_DROP` over the floor, SECOND-ORDER RR2)
        floor_spot: on_ground(face.add(left.scale(-int(3)))).add(v3(0, BODY_DROP, 0)),
        floor_yaw: yaw_of(h),
    }
}

/// The mesa's low rims: the walls' height once the canyon has opened onto
/// the mesa top, past the finish (over twice a jump's 3.6 m rise, so they
/// still close the course; low enough to see the sky and the far buttes
/// all round).
pub const MESA_RIM: i32 = int(8);
/// The mesa's campfire and tents are drawn this many times their models'
/// size (a person's), so they read at race distance.
pub const CAMPFIRE_SCALE: i32 = ONE * 5 / 2;
pub const TENT_SCALE: i32 = ONE * 11 / 5;

/// The walls fall to `MESA_RIM` over this many metres past the finish
/// line. Up to the line they are as they were (the canyon's last 60 m
/// already a little lower, a little wider): the AI's run in is unchanged
/// (opening them from 24 m short pushed a Sloth's 148.9 s run past the
/// 150 s cut, SECOND-ORDER).
pub const RIM_FALL: i32 = 30;

/// Rim heights: tall slot-canyon walls, 60–80 m, easing down over the
/// last 60 m and falling past the finish to the mesa's low rims
/// (`MESA_RIM`, a metre's wave): the last stretch climbs out of the slot
/// onto the open mesa top.
pub(crate) fn rim(s: i32, side: i32, len: i32) -> i32 {
    let wave = fx::sin(s * 97 + side * 20_000);
    let tall = int(70) + fx::mul(wave, int(10));
    let to_mesa = (s.min(len) - (len - 60)).clamp(0, 220);
    let at_line = tall - (tall - int(14)) * to_mesa / 220;
    let low = MESA_RIM + fx::mul(wave, int(1));
    let out = (s - len).clamp(0, RIM_FALL);
    at_line - (at_line - low) * out / RIM_FALL
}

/// The canyon's half width on its left at `s` m along a spine `len` m
/// long: `HALF`, a quarter metre a metre wider from 20 m short of the
/// finish (as before the mesa opened), then past the line a metre a metre
/// for 60 m (some 85 m either side 60 m past it) and a quarter metre a
/// metre on (some 110 m at the far end). The scene's floor reaches the
/// walls by it.
pub(crate) fn open_half(len: i32, s: i32) -> i32 {
    let short = int((s.min(len) - (len - 20)).clamp(0, 20)) / 4;
    let d = (s - len).clamp(0, 200);
    HALF + short + int(d.min(60)) + int((d - 60).max(0)) / 4
}

/// Wall points are this far apart along the spine (and a hole's ends are
/// snapped to them).
pub(crate) const WALL_STEP: i32 = 6;

/// The canyon's walls: the left and right faces at `HALF` (the corridor),
/// from 24 m behind the start to the mesa (opening out past the finish);
/// the back wall behind the start and the far end past the mesa; the
/// island between the passages; the hole through the fin at the
/// skull (the fin's face split under and over the mouth on both sides).
fn walls(c: &Course, plan: &Plan, split: Option<&Split>, has_skull: bool) -> Vec<Wall> {
    let len = c.length / ONE;
    let end = len + crate::course::OUTRO / ONE;
    let (fork, rejoin) = (plan.marks[leg::FORK], plan.marks[leg::REJOIN]);
    let face = |s: i32, off: i32| {
        on_ground(
            c.point(int(s))
                .add(right_of(c.heading_at(int(s))).scale(off)),
        )
    };
    // how far right of the spine (along its right at `s`) the upper
    // passage's middle is, and its half width that way: where the spine's
    // right meets the chord, `d / cos θ` (θ the angle between the two),
    // and `HALF / cos θ` (a projection, `d cos θ`, put the walls inside
    // the upper passage in the swing: the AI ran into them)
    let upper_ray = |s: i32| -> (i32, i32) {
        let Some(sp) = split else { return (0, HALF) };
        if s <= fork || s >= rejoin {
            return (0, HALF);
        }
        let a = sp.upper[0];
        let dir = sp.upper[sp.upper.len() - 1].sub(a).flat().norm();
        let p = c.point(int(s));
        let q = a.add(dir.scale(p.sub(a).flat().dot(dir)));
        let r = right_of(c.heading_at(int(s)));
        let cos = right_of(dir).dot(r).abs().max(ONE / 2);
        (
            fx::div(fx::div(q.sub(p).flat().dot(r), cos), cos),
            fx::div(HALF, cos),
        )
    };
    // the mesa: the canyon opens out past the finish
    let open = |s: i32| open_half(len, s);
    let mut left_pts = Vec::new();
    let mut right_pts = Vec::new();
    let mut s = -START_BACK / ONE;
    while s <= end {
        left_pts.push((s, face(s, -open(s)), rim(s, -1, len)));
        let (off, half) = upper_ray(s);
        let rw = if off > 0 {
            (off + half).max(HALF)
        } else {
            open(s)
        };
        right_pts.push((s, face(s, rw), rim(s, 1, len)));
        s += WALL_STEP;
    }
    let depth = int(12);
    let lo = MOUTH_UP - MOUTH_GAP / 2 - int(2);
    let hi = MOUTH_UP + MOUTH_GAP / 2 + int(2);
    let mut walls = Vec::new();
    // the left face, with the skull's two holes (in the in-leg and out of
    // the out-leg, both in the fin): under and over each mouth
    let holes: Vec<(i32, i32)> = if has_skull {
        let (a, b) = skull_spots(plan);
        let snap = |x: i32, up: bool| {
            let k = x.div_euclid(WALL_STEP) + up as i32;
            k * WALL_STEP
        };
        vec![
            (snap(a - 7, false), snap(a + 7, true)),
            (snap(b - 7, false), snap(b + 7, true)),
        ]
    } else {
        Vec::new()
    };
    let point = |&(_, at, height): &(i32, V3, i32)| WallPoint { at, height };
    let mut from = left_pts[0].0;
    for &(a, b) in &holes {
        walls.push(Wall {
            points: left_pts
                .iter()
                .filter(|p| p.0 >= from && p.0 <= a)
                .map(point)
                .collect(),
            side: Side::Left,
            depth,
        });
        let inside: Vec<_> = left_pts
            .iter()
            .filter(|p| p.0 >= a && p.0 <= b)
            .copied()
            .collect();
        walls.push(Wall {
            points: inside
                .iter()
                .map(|&(_, at, _)| WallPoint {
                    at,
                    height: lo - WALL_ABOVE,
                })
                .collect(),
            side: Side::Left,
            depth,
        });
        walls.push(Wall {
            points: inside
                .iter()
                .map(|&(_, at, h)| WallPoint {
                    at: at.add(v3(0, hi + WALL_BELOW, 0)),
                    height: h - hi - WALL_BELOW,
                })
                .collect(),
            side: Side::Left,
            depth,
        });
        from = b;
    }
    walls.push(Wall {
        points: left_pts.iter().filter(|p| p.0 >= from).map(point).collect(),
        side: Side::Left,
        depth,
    });
    walls.push(Wall {
        points: right_pts.iter().map(point).collect(),
        side: Side::Right,
        depth,
    });
    // the back wall behind the start, left to right (rock behind it)
    let back = -START_BACK / ONE;
    walls.push(Wall {
        points: (-4..=4)
            .map(|k| WallPoint {
                at: face(back, k * HALF / 4),
                height: int(70),
            })
            .collect(),
        side: Side::Right,
        depth: int(10),
    });
    // the island between the passages, where they are far enough apart: a
    // loop along the spine's right face and back along the upper's left
    if split.is_some() {
        let mut there = Vec::new();
        let mut back = Vec::new();
        let mut s = fork;
        while s <= rejoin {
            let (off, half) = upper_ray(s);
            if off - half >= HALF + int(6) {
                there.push(WallPoint {
                    at: face(s, HALF),
                    height: rim(s, 1, len),
                });
                back.push(WallPoint {
                    at: face(s, off - half),
                    height: rim(s, 1, len),
                });
            }
            s += WALL_STEP;
        }
        back.reverse();
        let first = there.first().copied();
        let mut points = there;
        points.extend(back);
        points.extend(first);
        walls.push(Wall {
            points,
            side: Side::Right,
            depth: int(3),
        });
    }
    // the far end, past the mesa's campfire (rock beyond it)
    walls.push(Wall {
        points: (-4..=4)
            .map(|k| WallPoint {
                at: face(end, k * open(end) / 4),
                height: rim(end, 1, len),
            })
            .collect(),
        side: Side::Left,
        depth: int(10),
    });
    walls
}
