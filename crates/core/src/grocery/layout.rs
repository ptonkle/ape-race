//! Clean-Up on Aisle Ape's store and course (2026-10-18): the floor plan
//! in its own metres (`plan`: east `x`, north `z`, the front-left inside
//! corner at 0), the route through it, and everything the course and the
//! scene share. The journal's route diagram is drawn from these numbers.
//!
//! **The store** (780 × 580 m, 65 × 48 m real): dairy cases along the west
//! wall, meat and seafood along the back (north), deli and bakery along the
//! east, produce's wet rack in the front-east corner. Thirteen runs stand
//! from the front cross aisle (`FRONT`) to the back one (`BACK`), bowed
//! `BOW` at their middles as on the plan: five shelving gondolas bowed
//! west, two freezer runs bowed apart (the frozen aisle between them opens
//! into a lens), six gondolas bowed east. An arc of checkout lanes, produce
//! tables round a misting U-turn, columns, display bins.
//!
//! **The route** (1,854 m): out of the stockroom doors in the back-west
//! corner, east along the back aisle, south down aisle 3 (cereal), east
//! along the front, north up the frozen aisle (6), east along the back,
//! south down aisle 9 (snacks), east into produce, a wide U-turn round the
//! misting tables, and west along the front end past the checkouts to the
//! finish by the exit doors. Stations every 22–28 m from the ceiling (the
//! Ravine's caves' spacings), two or three across, a low vine every other.
//!
//! New placement draws from dice of its own (`seed ^ DICE`), so no other
//! map changes.

use super::*;
use crate::course::{
    banana_arcs, centreline, Course, Leg, Mushroom, Platform, Vine, START_HEIGHT, START_LEN,
};
use crate::fx::Dice;
use crate::geom::{right_of, yaw_of};
use crate::ravine::solids::{SolidKind, Solids};
use crate::ravine::{
    Finish, HangFrom, Hanger, Lizard, LizardColour, RavineLayout, Rock, Room, RoomKind, Side, Wall,
    WallPoint,
};
use crate::rules::{MUSHROOM_HEIGHT, MUSHROOM_RADIUS};

const SEED: u64 = 0x6E0C_E2E5_0001;
/// This map's own dice (maps-archive "Layout lessons": no other map's
/// draws move).
const DICE: u64 = 0x4752_4F43_4552;

/// The start, in plan metres: in the back aisle, 70 m short of the corner
/// into aisle 3 (whose middle line is at x 210 where it meets the aisles'
/// ends), so that the corner's arc (Leg(56, -90), 35.65 m round) brings
/// the route onto it.
pub const START: (i32, i32) = (cm(10435), int(540));

/// The route: straights and bends (metres, degrees; + is left).
const LEGS: &[Leg] = &[
    Leg(60, 0),   // 0 the start, by the stockroom doors
    Leg(10, 0),   // 1 the back aisle
    Leg(56, -90), // 2 into aisle 3 (R 36 m: room to swing round the runs' ends)
    Leg(6, -7),   // 3 aisle 3 (cereal), bowed west
    Leg(134, 0),
    Leg(12, 14),
    Leg(134, 0),
    Leg(6, -7),
    Leg(2, 0),
    Leg(56, 90),  // 9 onto the front cross aisle
    Leg(86, 0),   // 10 east past the checkouts' backs
    Leg(56, 90),  // 11 into the frozen aisle
    Leg(286, 0),  // 12 the frozen aisle, north
    Leg(56, -90), // 13 onto the back aisle
    Leg(90, 0),   // 14 east under the meat sign
    Leg(56, -90), // 15 into aisle 9
    Leg(6, 7),    // 16 aisle 9 (snacks), bowed east
    Leg(136, 0),
    Leg(12, -14),
    Leg(132, 0),
    Leg(6, 7),
    Leg(2, 0),
    Leg(56, 90),    // 22 into produce
    Leg(40, 0),     // 23 east along the produce
    Leg(198, -180), // 24 the misting U-turn, down into the front end (R 63 m)
    Leg(160, 0),    // 25 the front end, to the doors
];

/// Leg numbers (`Plan.marks`) the art and the tests look up.
pub mod leg {
    pub const AISLE_A: usize = 3;
    pub const FRONT: usize = 10;
    pub const AISLE_B: usize = 12;
    pub const BACK: usize = 14;
    pub const AISLE_C: usize = 16;
    pub const PRODUCE: usize = 23;
    pub const TURN: usize = 24;
    pub const FRONT_END: usize = 25;
}

/// Where each leg starts along the route (and, last, its length), metres.
pub fn marks() -> Vec<i32> {
    let mut out = vec![0];
    for l in LEGS {
        out.push(out[out.len() - 1] + l.0);
    }
    out
}

// ---- the plan (metres) ----

/// The store's inside: west, south (the front windows), east, north.
pub const WEST: i32 = 14;
pub const SOUTH: i32 = 20;
pub const EAST_WALL: i32 = 762;
pub const NORTH_WALL: i32 = 570;
/// The runs' front and back ends, and their middles (bowed `BOW`).
pub const FRONT: i32 = 206;
pub const BACK: i32 = 510;
pub const MIDDLE: i32 = 358;
pub const BOW: i32 = 20;
/// A gondola's half thickness, and a freezer run's.
pub const GONDOLA_HALF: i32 = 5;
pub const FREEZER_HALF: i32 = 9;

/// The thirteen runs west to east: their middle line's x at the ends, the
/// way they bow (-1 west), what they are, and the aisles their west and
/// east faces sell (`grocerymodels::AISLES`).
pub const RUNS: [(i32, i32, Run, [u8; 2]); 13] = [
    (90, -1, Run::Shelves, [0, 1]),
    (138, -1, Run::Shelves, [1, 2]),
    (186, -1, Run::Shelves, [2, 3]),
    (234, -1, Run::Shelves, [3, 4]),
    (282, -1, Run::Shelves, [4, 5]),
    (336, -1, Run::Freezer, [7, 6]),
    (400, 1, Run::Freezer, [6, 7]),
    (457, 1, Run::Shelves, [5, 8]),
    (505, 1, Run::Shelves, [8, 9]),
    (553, 1, Run::Shelves, [9, 10]),
    (601, 1, Run::Shelves, [10, 11]),
    (649, 1, Run::Shelves, [11, 12]),
    (697, 1, Run::Shelves, [12, 13]),
];

/// The runs' ends cut at an angle where the route turns round them (the
/// inside of each turn between an aisle and a cross aisle): which run,
/// which end (0 the front, 2 the back), which face (0 west, 1 east), and how
/// far up that face the cut starts. A swing round the turn clears the
/// end; the plan's own ends are angled across the fan.
pub const CHAMFERS: [(usize, usize, usize, i32); 6] = [
    (2, 2, 1, 16),
    (3, 0, 0, 16),
    (5, 0, 1, 16),
    (6, 2, 0, 16),
    (8, 2, 1, 16),
    (9, 0, 0, 16),
];

/// The route's three aisles: between which runs (west, east).
pub const ROUTE_AISLES: [(usize, usize); 3] = [(2, 3), (5, 6), (8, 9)];

/// A run's middle line's plan x at plan north `z`.
pub fn run_x(x0: i32, sign: i32, z: i32) -> i32 {
    // (`z` and the result in plan metres, Q16)
    let half = int((BACK - FRONT) / 2);
    let t = (half - (z - int(MIDDLE)).abs()).max(0);
    int(x0) + sign * fx::mul(int(BOW), fx::div(t, half))
}

/// An aisle's middle at plan north `z` (Q16), between runs `w` and `e`.
pub fn aisle_x(w: usize, e: usize, z: i32) -> i32 {
    let (a, b) = (RUNS[w], RUNS[e]);
    (run_x(a.0, a.1, z) + run_x(b.0, b.1, z)) / 2
}

/// A plan point from whole metres.
fn p(x: i32, z: i32) -> V3 {
    plan(int(x), int(z))
}

/// Checkout lanes: their counters' west edges (plan x), from `LANES_SOUTH`
/// to `LANES_NORTH`.
const LANE_XS: [i32; 9] = [270, 304, 338, 372, 406, 440, 474, 508, 542];
pub const LANES_SOUTH: i32 = 100;
pub const LANES_NORTH: i32 = 150;
/// A counter's half width.
const COUNTER_HALF: i32 = cm(450);

/// The glass storefront's south face (plan north) and its ends: the
/// front-end vestibule runs between it and the front windows (`SOUTH`).
pub const GLASS_Z: i32 = 85;
pub const GLASS_WEST: i32 = 230;
pub const GLASS_EAST: i32 = 620;

/// White columns (plan metres).
const COLUMNS: [(i32, i32); 11] = [
    (60, 560),
    (300, 560),
    (460, 562),
    (620, 560),
    (735, 560),
    (60, 170),
    (130, 165),
    (440, 172),
    (725, 172),
    (200, 32),
    (330, 30),
];

/// Produce tables (plan metres, radius): round the U-turn's middle and
/// beyond its ring, each with a mister.
const TABLES: [(i32, i32, i32); 6] = [
    (606, 119, 14),
    (628, 130, 7),
    (634, 100, 7),
    (578, 120, 7),
    (700, 60, 9),
    (702, 150, 9),
];

/// Display bins (plan metres): kraft cardboard rounds of product on the
/// corridors' edges.
const BINS: [(i32, i32, u8); 6] = [
    (258, 198, 9),
    (318, 198, 3),
    (420, 180, 10),
    (470, 186, 8),
    (586, 198, 13),
    (100, 186, 0),
];

/// The coffin freezers down the frozen aisle's middle (plan north of their
/// middles); each is 32 m long, 12 m wide, 11 m tall.
const COFFINS: [i32; 3] = [308, 358, 408];

/// The shoppers' walks (plan metres): from, to, balloons, cart.
#[allow(clippy::type_complexity)]
fn walks() -> Vec<((i32, i32), (i32, i32), bool, bool)> {
    // an aisle's walk `off` metres east of its middle line, between two
    // plan norths in one straight half of it
    let along = |w: usize, e: usize, off: i32, z0: i32, z1: i32| {
        let x0 = (aisle_x(w, e, int(z0)) >> 16) + off;
        let x1 = (aisle_x(w, e, int(z1)) >> 16) + off;
        ((x0, z0), (x1, z1))
    };
    let mut out = Vec::new();
    let mut put =
        |(a, b): ((i32, i32), (i32, i32)), balloons, cart| out.push((a, b, balloons, cart));
    // aisle 3, cereal
    put(along(2, 3, -9, 380, 470), true, false);
    put(along(2, 3, 9, 225, 320), false, true);
    // the frozen aisle (wider at its middle: the coffins between)
    put(along(5, 6, -17, 230, 300), false, true);
    put(along(5, 6, 17, 410, 490), true, false);
    // aisle 9, snacks
    put(along(8, 9, -9, 230, 330), true, false);
    put(along(8, 9, 9, 395, 480), false, false);
    // the back aisle and the front cross aisle
    put(((130, 553), (175, 553)), false, true);
    put(((405, 527), (480, 527)), true, false);
    put(((250, 195), (330, 195)), false, true);
    // produce, and the front end
    put(((560, 62), (620, 62)), true, false);
    put(((712, 140), (712, 175)), false, true);
    put(((470, 36), (540, 36)), false, true);
    // two aisles the route passes the ends of
    put(along(0, 1, 6, 220, 300), false, true);
    put(along(10, 11, -6, 400, 480), false, false);
    out
}

/// A shopper walks this fast on average, m/s (half a metre a second real).
const WALK_SPEED: i32 = 6;
/// A balloon bunch's middle over the floor, and how far out from the
/// shopper's middle the hand holds its string (to their right).
pub const BALLOONS_UP: i32 = int(46);
pub const HAND_OUT: i32 = cm(450);
/// The hand holding the string, over the floor.
pub const HAND_UP: i32 = int(19);

/// Clean-Up on Aisle Ape's course, from its fixed seed.
pub fn grocery() -> Course {
    grocery_seeded(SEED)
}

/// Clean-Up on Aisle Ape's course from `seed` (the store is fixed; the
/// vines' jitter, lanes and bounce bins follow the seed).
pub fn grocery_seeded(seed: u64) -> Course {
    let mut c = centreline("grocery", LEGS);
    c.terrain = crate::ski::Terrain::Flat(0);
    let mut dice = Dice(seed ^ DICE);
    let len = c.length / ONE;
    let marks = marks();

    // ---- the start: a stock cart loaded with cases, by the doors ----
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

    // ---- the solids: runs, the store's walls, the ceiling, posts ----
    let mut walls = Vec::new();
    let mut gondolas = Vec::new();
    for &(x0, sign, run, faces) in &RUNS {
        let half = int(match run {
            Run::Shelves => GONDOLA_HALF,
            Run::Freezer => FREEZER_HALF,
        });
        let line = [FRONT, MIDDLE, BACK].map(|z| plan(run_x(x0, sign, int(z)), int(z)));
        // a loop round its outline, the inside on the right: north up its
        // west face, east over the back, south down its east face
        let mut west: Vec<V3> = line.iter().map(|q| q.sub(EAST.scale(half))).collect();
        let mut east: Vec<V3> = line.iter().map(|q| q.add(EAST.scale(half))).collect();
        let k = gondolas.len();
        for &(run, end, face, cut) in &CHAMFERS {
            if run != k {
                continue;
            }
            let along = if end == 0 { NORTH } else { NORTH.neg() };
            let q = if face == 0 {
                &mut west[end]
            } else {
                &mut east[end]
            };
            *q = q.add(along.scale(int(cut)));
        }
        east.reverse();
        let mut points: Vec<WallPoint> = west
            .iter()
            .chain(east.iter())
            .map(|&at| WallPoint {
                at,
                height: STOCK_TOP,
            })
            .collect();
        points.push(points[0]);
        gondolas.push(Gondola {
            run,
            line,
            half,
            faces,
            wall: walls.len(),
        });
        walls.push(Wall {
            points,
            side: Side::Right,
            depth: half,
        });
    }
    // the store's walls at the cases' fronts, the rock outside (on the
    // right going round: east along the front, north, west, south)
    let corners = [
        p(WEST, SOUTH),
        p(EAST_WALL, SOUTH),
        p(EAST_WALL, NORTH_WALL),
        p(WEST, NORTH_WALL),
        p(WEST, SOUTH),
    ];
    walls.push(Wall {
        points: corners
            .iter()
            .map(|&at| WallPoint {
                at,
                height: CEILING_AT,
            })
            .collect(),
        side: Side::Right,
        depth: int(10),
    });
    // the glass storefront between the checkouts and the front windows:
    // the front end's run is a vestibule closed on both sides, so the
    // finish line catches an ape anywhere across it (map_checks Finish)
    let glass = [
        p(GLASS_WEST, GLASS_Z),
        p(GLASS_EAST, GLASS_Z),
        p(GLASS_EAST, GLASS_Z + 3),
        p(GLASS_WEST, GLASS_Z + 3),
        p(GLASS_WEST, GLASS_Z),
    ];
    walls.push(Wall {
        points: glass
            .iter()
            .map(|&at| WallPoint {
                at,
                height: CEILING_AT,
            })
            .collect(),
        side: Side::Left,
        depth: cm(150),
    });
    // the aisles off the route are closed at both ends: a loaded stock
    // cart across each mouth (an AI that falls by a mouth would run in and
    // find no vine it can see, `ai::walled_off`)
    let mut blockers = Vec::new();
    let mut close = |walls: &mut Vec<Wall>, w: i32, e: i32, z: i32, inward: i32| {
        // (a box from x `w` to `e`, its face at `z`, `inward` deep into the
        // aisle, 1 m into the runs either side)
        let (z0, z1) = (z.min(z + inward), z.max(z + inward));
        let corners = [
            (w - 1, z0),
            (w - 1, z1),
            (e + 1, z1),
            (e + 1, z0),
            (w - 1, z0),
        ];
        blockers.push(Blocker {
            from: p(w, z),
            to: p(e, z),
            out: if inward > 0 { NORTH.neg() } else { NORTH },
            wall: walls.len(),
        });
        walls.push(Wall {
            points: corners
                .iter()
                .map(|&(x, z)| WallPoint {
                    at: p(x, z),
                    height: BLOCKER_TOP,
                })
                .collect(),
            side: Side::Right,
            depth: int(3),
        });
    };
    let half = |k: usize| match RUNS[k].2 {
        Run::Shelves => GONDOLA_HALF,
        Run::Freezer => FREEZER_HALF,
    };
    for a in 0..=RUNS.len() {
        if ROUTE_AISLES.iter().any(|&(w, _)| w + 1 == a) {
            continue;
        }
        let w = if a == 0 {
            WEST
        } else {
            RUNS[a - 1].0 + half(a - 1)
        };
        let e = if a == RUNS.len() {
            EAST_WALL
        } else {
            RUNS[a].0 - half(a)
        };
        close(&mut walls, w, e, FRONT + 1, 6);
        close(&mut walls, w, e, BACK - 1, -6);
    }
    let cases = vec![
        Cases {
            dept: Dept::Dairy,
            from: p(WEST, 160),
            to: p(WEST, 560),
            out: EAST,
        },
        Cases {
            dept: Dept::Meat,
            from: p(30, NORTH_WALL),
            to: p(740, NORTH_WALL),
            out: NORTH.neg(),
        },
        Cases {
            dept: Dept::Deli,
            from: p(EAST_WALL, 380),
            to: p(EAST_WALL, 560),
            out: EAST.neg(),
        },
        Cases {
            dept: Dept::Bakery,
            from: p(EAST_WALL, 200),
            to: p(EAST_WALL, 370),
            out: EAST.neg(),
        },
        Cases {
            dept: Dept::WetRack,
            from: p(EAST_WALL, 40),
            to: p(EAST_WALL, 180),
            out: EAST.neg(),
        },
    ];
    // the ceiling: one room over the whole store, west to east
    let mid_z = (SOUTH + NORTH_WALL) / 2;
    let rooms = vec![Room {
        kind: RoomKind::Arch,
        from: p(WEST - 12, mid_z),
        to: p(EAST_WALL + 12, mid_z),
        half_width: int((NORTH_WALL - SOUTH) / 2 + 12),
        height: CEILING_AT,
    }];
    let mut rocks = Vec::new();
    let mut posts = Vec::new();
    let mut post = |rocks: &mut Vec<Rock>, at: V3, radius, height, kind, stock| {
        posts.push(Post {
            rock: rocks.len(),
            kind,
            stock,
        });
        rocks.push(Rock { at, radius, height });
    };
    for &(x, z) in &COLUMNS {
        post(&mut rocks, p(x, z), int(2), CEILING_AT, PostKind::Column, 0);
    }
    for (k, &(x, z, r)) in TABLES.iter().enumerate() {
        post(
            &mut rocks,
            p(x, z),
            int(r),
            int(10),
            PostKind::Table,
            k as u8,
        );
    }
    for &(x, z, stock) in &BINS {
        post(&mut rocks, p(x, z), cm(480), int(11), PostKind::Bin, stock);
    }
    let frozen_x = |z: i32| aisle_x(5, 6, int(z));
    for &z in &COFFINS {
        for k in [-12, -4, 4, 12] {
            let zz = int(z + k);
            post(
                &mut rocks,
                plan(frozen_x(z + k), zz),
                int(6),
                int(11),
                PostKind::Coffin,
                0,
            );
        }
    }
    let mut lanes = Vec::new();
    for &x in &LANE_XS {
        let mid = plan(int(x) + COUNTER_HALF, int(LANES_SOUTH + LANES_NORTH) / 2);
        lanes.push(Lane {
            at: mid,
            yaw: yaw_of(NORTH.neg()),
        });
        let mut z = LANES_SOUTH * ONE + COUNTER_HALF;
        while z <= LANES_NORTH * ONE - COUNTER_HALF {
            post(
                &mut rocks,
                plan(int(x) + COUNTER_HALF, z),
                COUNTER_HALF,
                int(11),
                PostKind::Counter,
                0,
            );
            z += int(8);
        }
    }
    // the water-case pallet display: exactly the Ravine's sphinx box (its
    // `Finish.sphinx` is always solid), behind the glass's east end
    let pallet = p(255, 125);
    let finish = Finish {
        sphinx: pallet,
        sphinx_yaw: yaw_of(NORTH),
        campfire: pallet,
        sticks: Vec::new(),
        tents: Vec::new(),
    };

    // the solids so far, for keeping the vines and walks clear of them
    let probe = RavineLayout {
        walls: walls.clone(),
        rooms: rooms.clone(),
        decks: Vec::new(),
        hangers: Vec::new(),
        lizards: Vec::new(),
        tracks: Vec::new(),
        carts: Vec::new(),
        skull: None,
        split: None,
        rocks: rocks.clone(),
        glows: Vec::new(),
        finish: finish.clone(),
        lobby: (V3::ZERO, 0),
    };
    let solids = Solids::of(&probe);
    // whether a body hanging at `q` (the hands) is clear of every wall and
    // post by `margin` (the ceiling's roof not counted)
    let clear = |q: V3, margin: i32| {
        !solids.near(q).iter().any(|&k| {
            let s = &solids.list[k as usize];
            s.kind != SolidKind::Roof && s.holds(q, margin)
        })
    };

    // ---- the vines: licorice ropes from the ceiling ----
    let mut vines: Vec<Vine> = Vec::new();
    let mut hangers: Vec<Hanger> = Vec::new();
    let mut hang = |vines: &mut Vec<Vine>, anchor: V3, length: i32, at: i32| {
        vines.push(Vine {
            anchor,
            length,
            slide: None,
            at,
        });
        hangers.push(Hanger {
            from: HangFrom::Roof { room: 0 },
            at: anchor,
            snake: false,
            snapped_at: None,
        });
        vines.len() - 1
    };
    // the start's row across the back aisle
    for k in -3..=3 {
        let at = START_LEN + cm(150);
        let base = c
            .point(at)
            .add(right_of(c.heading_at(at)).scale(int(3) * k));
        hang(&mut vines, v3(base.x, CEILING_AT, base.z), int(35), at);
    }
    let mut pins = Vec::new();
    for k in [2, 9, 11, 13, 15, 22] {
        pins.push(marks[k] - 14);
        pins.push(marks[k + 1] + 5);
    }
    pins.sort();
    let at = stations(32, len - 30, &pins, 25, &mut dice);
    let mut all: Vec<Vec<usize>> = Vec::new();
    let mut mushrooms = Vec::new();
    for (station, &u) in at.iter().enumerate() {
        let q = c.point(int(u));
        let h = c.heading_at(int(u));
        let r = right_of(h);
        let first = station == 0;
        let skip = if first || dice.below(6) == 0 {
            3
        } else {
            dice.below(3) as i32
        };
        let mut here = Vec::new();
        let mut lanes_at = Vec::new();
        for lane in 0..3 {
            if lane == skip {
                continue;
            }
            let off = if first {
                (lane - 1) * 6
            } else {
                (lane - 1) * 10 + dice.range(-2, 2)
            };
            let base = q.add(r.scale(int(off)));
            let length = if first {
                int(30)
            } else {
                int(dice.range(23, 31))
            };
            let foot = v3(base.x, CEILING_AT - length, base.z);
            if !clear(foot, int(3)) || !clear(foot.add(v3(0, length / 2, 0)), int(2)) {
                continue;
            }
            here.push(hang(
                &mut vines,
                v3(base.x, CEILING_AT, base.z),
                length,
                int(u),
            ));
            lanes_at.push(off);
        }
        all.push(here);
        // a low vine every other station, to climb back up by (further
        // out where the middle is taken: the frozen aisle's coffins)
        if station % 2 == 1 {
            let sign = if dice.below(2) == 0 { -1 } else { 1 };
            let off = dice.range(7, 9) * sign;
            let length = CEILING_AT - int(3) - cm(dice.range(0, 40));
            let s7 = int(u + 7);
            let spot = [off, 14 * sign, 18 * sign, -14 * sign, -18 * sign]
                .into_iter()
                .map(|o| c.point(s7).add(right_of(c.heading_at(s7)).scale(int(o))))
                .find(|b| clear(v3(b.x, CEILING_AT - length, b.z), int(3)));
            if let Some(b) = spot {
                hang(&mut vines, v3(b.x, CEILING_AT, b.z), length, s7);
            }
        }
        // a bin of bouncy balls short of a side lane's vine
        if station % 4 == 2 && !lanes_at.is_empty() {
            let off = lanes_at[dice.below(lanes_at.len() as u32) as usize];
            let off = if off.abs() < 6 { 10 } else { off };
            let b = c
                .point(int(u - 11))
                .add(right_of(c.heading_at(int(u - 11))).scale(int(off)));
            if clear(v3(b.x, MUSHROOM_HEIGHT, b.z), MUSHROOM_RADIUS) {
                mushrooms.push(Mushroom {
                    top: v3(b.x, MUSHROOM_HEIGHT, b.z),
                    radius: MUSHROOM_RADIUS,
                });
            }
        }
    }

    // ---- shoppers: lizards that walk the floor; some hold balloons ----
    c.mushrooms = mushrooms;
    let mut lizards = Vec::new();
    let mut shoppers = Vec::new();
    for (k, &(a, b, balloons, cart)) in walks().iter().enumerate() {
        let (from, to) = (p(a.0, a.1), p(b.0, b.1));
        let run = to.sub(from).flat();
        let length = run.len();
        let way = run.norm();
        let period = (2 * length / ONE * crate::rules::TICKS_PER_SECOND / WALK_SPEED).max(1) as u32;
        let vine = balloons.then(|| {
            let mid = from.lerp(to, ONE / 2);
            let hand = right_of(way).scale(HAND_OUT);
            let anchor = v3(mid.x, BALLOONS_UP, mid.z).add(hand);
            let i = hang(&mut vines, anchor, BALLOONS_UP - HAND_UP, int(len / 2));
            vines[i].slide = Some(crate::course::Slide {
                reach: to.sub(from).scale(ONE / 2),
                period: period as i32,
            });
            vines[i].at = int(c.progress(mid, c.nearest_anywhere(mid)).0 / ONE);
            i
        });
        let phase = match vine {
            Some(i) => balloon_phase(i, period),
            None => (k as u32 * 977) % period,
        };
        let bouncer = c.mushrooms.len();
        let lizard = Lizard {
            from,
            to,
            out: right_of(way),
            period,
            phase,
            bouncer,
            colour: LizardColour::Red,
        };
        // (the bouncer is set where the shopper is at tick 0: the Ravine
        // moves it on from there, `ravine_bouncer`)
        let at0 = RavineLayout {
            lizards: vec![lizard],
            ..probe.clone()
        }
        .lizard_at(0, 0)
        .0;
        c.mushrooms.push(Mushroom {
            top: at0.add(v3(0, SHOPPER_HEIGHT, 0)),
            radius: MUSHROOM_RADIUS,
        });
        shoppers.push(Shopper {
            lizard: lizards.len(),
            balloons: vine,
            cart,
            look: (k * 5 % 7) as u8,
        });
        lizards.push(lizard);
    }
    c.vines = vines;
    banana_arcs(&mut c, &all);

    // item crates: rows of three across the route every 240 m
    let mut s = 150;
    while s < len - 60 {
        let h = c.heading_at(int(s));
        let r = right_of(h);
        let base = c.point(int(s));
        for off in [-9, 0, 9] {
            let q = base.add(r.scale(int(off)));
            let q = v3(q.x, int(dice.range(16, 24)), q.z);
            if clear(q, int(2)) {
                c.crates.push(q);
            }
        }
        s += 240;
    }

    let doors = (p(410, SOUTH), NORTH.neg());
    let lobby = (c.starts[0], c.start_yaw);
    c.ravine = Some(Box::new(RavineLayout {
        walls,
        rooms,
        decks: Vec::new(),
        hangers,
        lizards,
        tracks: Vec::new(),
        carts: Vec::new(),
        skull: None,
        split: None,
        rocks,
        glows: Vec::new(),
        finish,
        lobby,
    }));
    let misters = TABLES
        .iter()
        .enumerate()
        .map(|(k, &(x, z, _))| Mister {
            at: p(x, z).add(v3(0, int(11), 0)),
            period: crate::rules::secs_to_ticks(600 + 70 * k as i32) as u32,
            phase: (k as u32 * 331) % 500,
        })
        .collect();
    c.grocery = Some(Box::new(GroceryLayout {
        gondolas,
        cases,
        blockers,
        posts,
        lanes,
        shoppers,
        misters,
        bounds: (p(WEST, SOUTH), p(EAST_WALL, NORTH_WALL)),
        doors,
        lobby,
    }));
    c
}

/// Station positions from `from` to `to` (metres along the route): even
/// hops of about `hop` m (never over 28) between the pins, a metre of
/// jitter (the Ravine's rule).
fn stations(from: i32, to: i32, pins: &[i32], hop: i32, dice: &mut Dice) -> Vec<i32> {
    let mut fixed = vec![from];
    fixed.extend(
        pins.iter()
            .copied()
            .filter(|&q| q > from + 12 && q < to - 12),
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

use crate::rules::BODY_DROP;
