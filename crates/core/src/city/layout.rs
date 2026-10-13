//! The City's layout (2026-10-12): New York, from Midtown's towers down
//! to the river, over a Brooklyn-Bridge-style suspension bridge, through
//! Brooklyn's brownstones to a plaza past the finish. About 1.8 km, like
//! the Jungle, with the same vine heights and the same physics.
//!
//! - **Streets** (`Course.streets`): straight, 40 m between the facades,
//!   meeting in sharp corners of 90° and 45° both ways. The centreline
//!   bends through each corner on an arc of just over the half width (so
//!   progress along it stays smooth), but the walls are the streets':
//!   buildings stand square on them.
//! - **Corners**: at each 90° corner two short cables hang 27 m up over the
//!   intersection, one from a crane standing in a building site on the
//!   inside corner, one from a hovering helicopter further out. Caught
//!   high, a cable whirls an ape round (`examples/corner_swing.rs`: a
//!   quarter turn in 0.4–1 s, keeping 80–95% of the speed), so it lets go
//!   facing down the new street.
//! - **Vines**: stations every 24–30 m with two lanes of three (the
//!   Jungle: every 22–28 m, two or three), a low vine at every other
//!   station to climb back up by.
//! - **The bridge**: a deck 20 m wide at street level over the river,
//!   narrower than the corridor, so an ape can fall in beside it. Vines
//!   hang from the four main cables. Boats beside the deck bounce a
//!   swimmer back up; ladders climb to the deck.
//! - **Cars**: stopped cabs, cars and vans under the lanes bounce like the
//!   Jungle's mushrooms; cabs and buses drive along each street.

use super::*;
use crate::course::{
    banana_arcs, centreline, Leg, Platform, Street, Vine, OUTRO, START_HEIGHT, START_LEN,
};
use crate::fx::{cos, deg, div, mul, sin, Dice, ONE, TURN};
use crate::rules::{secs_to_ticks, RIVER_DROP, TICKS_PER_SECOND};

const SEED: u64 = 0xC17E_5EED_0001;

/// The streets and corners, metres and degrees, left turns positive. A
/// corner is a bend of radius just over the half width (32 m for 90°, 16 m
/// for 45°): its inside walls meet in a point.
const LEGS: &[Leg] = &[
    Leg(260, 0),
    Leg(32, 90),
    Leg(220, 0),
    Leg(16, -45),
    Leg(200, 0),
    Leg(32, -90),
    Leg(420, 0),
    Leg(16, 45),
    Leg(220, 0),
    Leg(32, 90),
    Leg(200, 0),
    Leg(16, -45),
    Leg(136, 0),
];

/// The river crosses the fourth street (the long one), this far along it
/// and this wide.
const RIVER_STREET: usize = 3;
const RIVER_IN: i32 = int(130);
const RIVER_WIDE: i32 = int(150);
/// The deck reaches this far onto each bank, and is this wide either side.
const DECK_ON: i32 = int(4);
pub const DECK_HALF: i32 = int(10);

/// The corner anchors: 27 m up, on 13 m cables.
const CORNER_HEIGHT: i32 = int(27);
const CORNER_CABLE: i32 = int(13);

#[derive(Debug, Clone, Copy)]
struct Straight {
    /// its first and last centreline samples
    i0: usize,
    i1: usize,
    dir: V3,
}

#[derive(Debug, Clone, Copy)]
struct Corner {
    /// where the two streets' lines cross, on the ground
    p: V3,
    din: V3,
    /// degrees, left positive
    turn: i32,
    /// the bend's first and last metres along the course
    s0: i32,
    s1: i32,
}

impl Corner {
    /// Toward the inside of the turn, from the incoming street.
    fn inward(&self) -> V3 {
        let r = right_of(self.din);
        if self.turn > 0 {
            r.neg()
        } else {
            r
        }
    }
}

/// A box on the ground, for overlap tests.
#[derive(Debug, Clone, Copy)]
struct Obb {
    c: V3,
    u: V3,
    hl: i32,
    hw: i32,
}

impl Obb {
    fn of(b: &Block) -> Obb {
        Obb {
            c: b.centre.flat(),
            u: b.fwd,
            hl: b.half_len,
            hw: b.half_width,
        }
    }

    fn street(st: &Street, hw: i32) -> Obb {
        let (lo, hi) = st.span(hw);
        Obb {
            c: st.from.add(st.dir.scale(lo + (hi - lo) / 2)),
            u: st.dir,
            hl: (hi - lo) / 2,
            hw,
        }
    }

    /// Whether the two overlap by more than `tol` (touching is not).
    fn overlaps(&self, o: &Obb, tol: i32) -> bool {
        let t = o.c.sub(self.c);
        let (av, ov) = (right_of(self.u), right_of(o.u));
        for axis in [self.u, av, o.u, ov] {
            let ra = mul(self.hl, self.u.dot(axis).abs()) + mul(self.hw, av.dot(axis).abs());
            let rb = mul(o.hl, o.u.dot(axis).abs()) + mul(o.hw, ov.dot(axis).abs());
            if t.dot(axis).abs() >= ra + rb - tol {
                return false;
            }
        }
        true
    }

    fn holds(&self, p: V3) -> bool {
        let d = p.flat().sub(self.c);
        d.dot(self.u).abs() <= self.hl && d.dot(right_of(self.u)).abs() <= self.hw
    }
}

/// The City laid out from its fixed seed.
pub fn city() -> Course {
    city_seeded(SEED)
}

/// The City laid out from another seed: the same streets, river and
/// bridge, different vines, buildings and cars (for the AI's tests).
pub fn city_seeded(seed: u64) -> Course {
    let mut c = centreline("city", LEGS);
    let (straights, corners) = shape(&c);
    c.streets = streets(&c, &straights, &corners);
    let mut dice = Dice(seed);
    let mut lay = CityLayout::default();

    // the river and the bridge
    let rs = straights[RIVER_STREET].i0 as i32 * crate::course::STEP;
    let river = River {
        from: rs + RIVER_IN,
        to: rs + RIVER_IN + RIVER_WIDE,
        level: -RIVER_DROP,
    };
    lay.river = Some(river);
    let bridge = bridge(&c, &river);
    let fwd = c.heading_at(river.from);
    let deck_len = bridge.end.sub(bridge.start).flat().len();
    c.platforms.push(Platform {
        centre: bridge.start.add(fwd.scale(deck_len / 2)),
        fwd,
        half_len: deck_len / 2,
        half_width: DECK_HALF,
        top: bridge.start.y,
    });
    lay.bridge = Some(bridge.clone());

    start(&mut c);
    lay.blocks = blocks(&c, &straights, &corners, &river, &mut dice);

    // the vines, and what each hangs from
    let mut stations: Vec<Vec<usize>> = Vec::new();
    let mut stopped: Vec<(V3, i32)> = Vec::new();
    let mut hangs: Vec<(usize, Hang)> = (0..c.vines.len()).map(|i| (i, Hang::Wire)).collect();
    let deck_s0 = river.from - DECK_ON;
    let deck_s1 = river.to + DECK_ON;
    let len_m = c.length / ONE;
    // the stretches without stations: 90° corners (their cables instead)
    // and the bridge (its cables); a station always stands 12–16 m short
    // of one, so there is a way into it
    let mut gaps: Vec<(i32, i32)> = corners
        .iter()
        .filter(|k| k.turn.abs() == 90)
        .map(|k| (k.s0 / ONE - 12, k.s1 / ONE + 4))
        .collect();
    gaps.push((deck_s0 / ONE - 10, deck_s1 / ONE + 10));
    let mut s = 32;
    let mut last = 0;
    let mut station = 0;
    while s < len_m - 30 {
        if let Some(&(g0, g1)) = gaps.iter().find(|&&(g0, g1)| s + 4 > g0 && s < g1) {
            if last < g0 - 18 && s + 4 > g0 {
                s = g0 - 4;
            } else {
                s = g1 + 1;
                continue;
            }
        }
        last = s;
        let si = int(s);
        let h = c.heading_at(si);
        let r = right_of(h);
        let skip = if station == 0 {
            3
        } else {
            dice.below(3) as i32
        };
        let mut here = Vec::new();
        let mut lanes = Vec::new();
        for lane in 0..3 {
            if lane == skip {
                continue;
            }
            let (anchor_h, length, off) = if station == 0 {
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
            c.vines.push(Vine {
                anchor: v3(base.x, ground(base.x, base.z) + anchor_h, base.z),
                length,
                slide: None,
                at: int(along),
            });
            here.push(c.vines.len() - 1);
            hangs.push((c.vines.len() - 1, Hang::Street));
            lanes.push((off, anchor_h - length));
        }
        stations.push(here);
        if station % 2 == 1 {
            // a low vine to climb back up by
            let off = dice.range(-9, 9);
            let b = c.point(int(s + 7)).add(r.scale(int(off)));
            c.vines.push(Vine {
                anchor: v3(b.x, ground(b.x, b.z) + int(27), b.z),
                length: int(24) + cm(dice.range(0, 40)),
                slide: None,
                at: int(s + 7),
            });
            hangs.push((c.vines.len() - 1, Hang::Street));
        } else if station > 0 && (station % 4 == 2 || station % 8 == 4) {
            // a stopped car short of a lane vine low enough for its bounce
            if let Some(&(off, _)) = lanes.iter().find(|(_, bottom)| *bottom < int(18)) {
                let at = s - 11;
                let p = c.point(int(at)).add(r.scale(int(off)));
                let flip = if dice.below(3) == 0 { TURN / 2 } else { 0 };
                stopped.push((v3(p.x, ground(p.x, p.z), p.z), fx::wrap(yaw_of(h) + flip)));
            }
        }
        station += 1;
        s += dice.range(24, 30);
    }

    // the corners' cables
    for k in corners.iter().filter(|k| k.turn.abs() == 90) {
        let inn = k.inward();
        for (u, w, hang) in [
            (-int(8), int(6), Hang::Crane(*k)),
            (-int(2), -int(8), Hang::Helicopter),
        ] {
            let p = k.p.add(k.din.scale(u)).add(inn.scale(w));
            let anchor = v3(p.x, ground(p.x, p.z) + CORNER_HEIGHT, p.z);
            let at = c.progress(anchor, c.nearest_anywhere(anchor)).0;
            c.vines.push(Vine {
                anchor,
                length: CORNER_CABLE,
                slide: None,
                at,
            });
            hangs.push((c.vines.len() - 1, hang));
        }
    }

    // the bridge's cables: a station at each end's side span and three on
    // the main span, two vines each (one inner cable, one outer), and a long
    // one in the middle down to the deck
    for (j, t) in [19, 50, 79, 108, 139].into_iter().enumerate() {
        let pair = if j % 2 == 0 { [0, 2] } else { [1, 3] };
        let mut here = Vec::new();
        for k in pair {
            let anchor = bridge.cable(k, int(t));
            let bottom = bridge.start.y + int(dice.range(11, 15));
            let at = c.progress(anchor, c.nearest_anywhere(anchor)).0;
            c.vines.push(Vine {
                anchor,
                length: anchor.y - bottom,
                slide: None,
                at,
            });
            here.push(c.vines.len() - 1);
            hangs.push((c.vines.len() - 1, Hang::Cable));
        }
        stations.push(here);
        if t == 79 {
            let anchor = bridge.cable(1, int(t + 6));
            let at = c.progress(anchor, c.nearest_anywhere(anchor)).0;
            c.vines.push(Vine {
                anchor,
                length: anchor.y - bridge.start.y - int(3),
                slide: None,
                at,
            });
            hangs.push((c.vines.len() - 1, Hang::Cable));
        }
    }

    lay.hangers = hangers(&c, &lay.blocks, &hangs, &mut dice);

    // the bouncers: stopped cars, then boats, then the moving cars
    for (at, yaw) in stopped {
        let kind = match dice.below(10) {
            0..=4 => CarKind::Cab,
            5..=7 => CarKind::Sedan,
            _ => CarKind::Van,
        };
        let (top, radius) = car_size(kind);
        c.mushrooms.push(Mushroom {
            top: at.add(v3(0, top, 0)),
            radius,
        });
        lay.cars.push(Car {
            at,
            yaw,
            kind,
            bouncer: c.mushrooms.len() - 1,
            drive: None,
        });
    }
    boats(&mut c, &mut lay, &bridge, &river);
    for (k, st) in straights.iter().enumerate() {
        let s0 = st.i0 as i32 * crate::course::STEP + int(15);
        let s1 = st.i1 as i32 * crate::course::STEP - int(15);
        if s1 - s0 < int(60) {
            continue;
        }
        for (from, to, lane) in [(s0, s1, int(6)), (s1, s0, -int(6))] {
            let kind = if dice.below(4) == 0 && k != RIVER_STREET {
                CarKind::Bus
            } else {
                CarKind::Cab
            };
            let speed = dice.range(9, 13);
            let period = (to - from).abs() / ONE * TICKS_PER_SECOND / speed;
            let drive = Drive {
                from,
                to,
                lane,
                period,
                phase: dice.range(0, period),
            };
            let (at, yaw) = super::drive_at(&c, &drive, 0);
            let (top, radius) = car_size(kind);
            c.mushrooms.push(Mushroom {
                top: at.add(v3(0, top, 0)),
                radius,
            });
            lay.cars.push(Car {
                at,
                yaw,
                kind,
                bouncer: c.mushrooms.len() - 1,
                drive: Some(drive),
            });
        }
    }

    banana_arcs(&mut c, &stations);

    // item crates: rows of three across the streets, as on the Jungle
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
    let _ = secs_to_ticks(0);
    c.city = Some(Box::new(lay));
    c
}

/// A car's roof above the ground, and its bouncer's radius.
pub fn car_size(kind: CarKind) -> (i32, i32) {
    match kind {
        CarKind::Cab => (cm(160), cm(260)),
        CarKind::Sedan => (cm(150), cm(260)),
        CarKind::Van => (cm(240), cm(300)),
        CarKind::Bus => (cm(320), cm(450)),
    }
}

/// A boat's deck above the water, and its bouncer's radius.
pub fn boat_size(kind: BoatKind) -> (i32, i32) {
    match kind {
        BoatKind::Tug => (cm(250), cm(300)),
        BoatKind::Barge => (cm(180), cm(380)),
        BoatKind::Ferry => (cm(350), cm(400)),
    }
}

/// The straights and the corners between them.
fn shape(c: &Course) -> (Vec<Straight>, Vec<Corner>) {
    let mut straights = Vec::new();
    let mut turns = Vec::new();
    let mut i = 0usize;
    for leg in LEGS {
        let steps = (leg.0 * ONE / crate::course::STEP) as usize;
        if leg.1 == 0 {
            straights.push(Straight {
                i0: i,
                i1: i + steps,
                dir: c.heading[i],
            });
        } else {
            turns.push((leg.1, i, i + steps));
        }
        i += steps;
    }
    let mut corners = Vec::new();
    for (k, &(turn, j0, j1)) in turns.iter().enumerate() {
        let (a, b) = (straights[k], straights[k + 1]);
        corners.push(Corner {
            p: cross(c.path[a.i1], a.dir, c.path[b.i0], b.dir),
            din: a.dir,
            turn,
            s0: j0 as i32 * crate::course::STEP,
            s1: j1 as i32 * crate::course::STEP,
        });
    }
    (straights, corners)
}

/// Where the line through `a` along `u` crosses the line through `b`
/// along `w`, on the ground.
fn cross(a: V3, u: V3, b: V3, w: V3) -> V3 {
    let cr = |p: V3, q: V3| p.x as i64 * q.z as i64 - p.z as i64 * q.x as i64;
    let den = cr(u, w);
    if den == 0 {
        return a;
    }
    let t = (cr(b.sub(a), w) << 16) / den;
    a.add(u.scale(t as i32)).flat()
}

/// The tangent of half a turn of `turn` degrees.
fn tan_half(turn: i32) -> i32 {
    let a = deg(turn.abs()) / 2;
    div(sin(a), cos(a))
}

fn streets(c: &Course, straights: &[Straight], corners: &[Corner]) -> Vec<Street> {
    let n = straights.len();
    let last = c.path.len() - 1;
    (0..n)
        .map(|k| {
            let st = straights[k];
            let from = if k == 0 { c.path[0] } else { corners[k - 1].p };
            let to = if k == n - 1 {
                c.path[last]
            } else {
                corners[k].p
            };
            Street {
                from: from.flat(),
                dir: st.dir,
                len: to.sub(from).dot(st.dir),
                tan0: if k == 0 {
                    0
                } else {
                    tan_half(corners[k - 1].turn)
                },
                tan1: if k == n - 1 {
                    0
                } else {
                    tan_half(corners[k].turn)
                },
                ext0: if k == 0 { int(7) } else { 0 },
                ext1: if k == n - 1 { OUTRO } else { 0 },
            }
        })
        .collect()
}

fn bridge(c: &Course, river: &River) -> Bridge {
    let s0 = river.from - DECK_ON;
    let s1 = river.to + DECK_ON;
    let fwd = c.heading_at(river.from);
    let r = right_of(fwd);
    // flush with the lower of its two ends, so it never stands proud of
    // the street (a step down onto it, a step up off it)
    let mut top = i32::MAX;
    for s in [s0, s1] {
        for w in [-DECK_HALF, 0, DECK_HALF] {
            let p = c.point(s).add(r.scale(w));
            top = top.min(ground(p.x, p.z));
        }
    }
    let top = top - cm(10);
    let at = |s: i32| {
        let p = c.point(s);
        v3(p.x, top, p.z)
    };
    Bridge {
        start: at(s0),
        end: at(s1),
        half_width: DECK_HALF,
        towers: vec![at(s0 + TOWER_IN), at(s1 - TOWER_IN)],
        tower_height: CABLE_TOP + int(4),
    }
}

/// The start: a deck 16 m up (an elevated rail line's platform) and a row
/// of cables across the street just past its edge, as on the Jungle.
fn start(c: &mut Course) {
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
    let edge = START_LEN;
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
}

/// A block's look and size for a stretch of the course (metres along):
/// Midtown's towers, then mid-rise and brownstones to the river, warehouses
/// on its banks, Brooklyn's brownstones, and offices toward the plaza.
fn pick(s: i32, river: &River, dice: &mut Dice) -> (u8, i32, i32, i32) {
    use style::*;
    let near_river = s > river.from - int(90) && s < river.to + int(90);
    let roll = dice.below(10);
    let st = if s < int(560) {
        match roll {
            0..=4 => TOWER,
            5..=7 => OFFICE,
            _ => MIDRISE,
        }
    } else if near_river {
        match roll {
            0..=5 => WAREHOUSE,
            _ => MIDRISE,
        }
    } else if s < river.from {
        match roll {
            0..=3 => MIDRISE,
            4..=6 => BROWNSTONE,
            _ => OFFICE,
        }
    } else if s < int(1480) {
        match roll {
            0..=4 => BROWNSTONE,
            5..=7 => MIDRISE,
            8 => OFFICE,
            _ => TOWER,
        }
    } else {
        match roll {
            0..=3 => OFFICE,
            4..=6 => TOWER,
            _ => MIDRISE,
        }
    };
    let (len, depth, h) = match st {
        TOWER => (dice.range(30, 44), dice.range(28, 40), dice.range(90, 190)),
        OFFICE => (dice.range(24, 40), dice.range(22, 32), dice.range(40, 80)),
        MIDRISE => (dice.range(18, 30), dice.range(18, 26), dice.range(20, 36)),
        WAREHOUSE => (dice.range(24, 40), dice.range(20, 30), dice.range(10, 18)),
        _ => (dice.range(14, 24), dice.range(14, 18), dice.range(12, 17)),
    };
    (st, int(len), int(depth), int(h))
}

/// The buildings: rows along both walls of every street, square on the
/// wall the physics uses; none in the river or over the bridge's
/// approaches, set back round the plaza, and a grand building closing it.
fn blocks(
    c: &Course,
    straights: &[Straight],
    corners: &[Corner],
    river: &River,
    dice: &mut Dice,
) -> Vec<Block> {
    let hw = c.half_width;
    let mut out: Vec<Block> = Vec::new();
    let walls: Vec<Obb> = c.streets.iter().map(|st| Obb::street(st, hw)).collect();
    // the river, wide, for keeping every street's buildings out of it
    let ro = c.point(river.from);
    let rh = c.heading_at(river.from);
    let in_river = |p: V3| {
        let d = p.flat().sub(ro.flat());
        let a = d.dot(rh);
        a > -int(25)
            && a < river.to - river.from + int(25)
            && d.dot(right_of(rh)).abs() < RIVER_REACH
    };
    let fits = |b: &Block, out: &[Block]| {
        let o = Obb::of(b);
        let corners = [(-1, -1), (-1, 1), (1, -1), (1, 1)].map(|(i, j)| {
            o.c.add(o.u.scale(o.hl * i))
                .add(right_of(o.u).scale(o.hw * j))
        });
        !walls.iter().any(|w| w.overlaps(&o, cm(5)))
            && !out.iter().any(|q| Obb::of(q).overlaps(&o, cm(5)))
            && !corners.iter().any(|p| in_river(*p))
            && !in_river(o.c)
    };
    // the building sites on the 90° corners' insides, where the cranes stand
    for k in corners.iter().filter(|k| k.turn.abs() == 90) {
        let inner = k.p.add(k.din.scale(-hw)).add(k.inward().scale(hw));
        let centre = inner
            .add(k.din.scale(-int(15)))
            .add(k.inward().scale(int(15)));
        out.push(Block {
            centre: v3(centre.x, ground(centre.x, centre.z), centre.z),
            fwd: k.din,
            half_len: int(15),
            half_width: int(15),
            height: int(3),
            style: style::HOARDING,
        });
    }
    // the start's back: an office across the street's end
    let back = c.path[0].sub(c.heading[0].scale(int(7) + int(12)));
    out.push(Block {
        centre: v3(back.x, ground(back.x, back.z), back.z),
        fwd: c.heading[0],
        half_len: int(12),
        half_width: hw + int(30),
        height: int(70),
        style: style::OFFICE,
    });
    // the plaza's end: a grand terminal across it
    let last = c.streets.len() - 1;
    let end_st = c.streets[last];
    let term = end_st
        .from
        .add(end_st.dir.scale(end_st.len + OUTRO + int(14)));
    out.push(Block {
        centre: v3(term.x, ground(term.x, term.z), term.z),
        fwd: end_st.dir,
        half_len: int(14),
        half_width: hw + int(40),
        height: int(34),
        style: style::TERMINAL,
    });
    let n = c.streets.len();
    for (k, st) in c.streets.iter().enumerate() {
        let s_from = if k == 0 {
            0
        } else {
            (corners[k - 1].s0 + corners[k - 1].s1) / 2
        };
        let r = right_of(st.dir);
        for side in [-1, 1] {
            // inside a turn the wall stops short of the crossing by the
            // mitre, outside it runs on past it
            let inner_at = |k: &Corner| (k.turn > 0) == (side < 0);
            let a0 = if k == 0 {
                -int(7)
            } else {
                let e = mul(hw, st.tan0);
                if inner_at(&corners[k - 1]) {
                    e
                } else {
                    -e
                }
            };
            let a1 = if k == n - 1 {
                st.len + int(10)
            } else {
                let e = mul(hw, st.tan1);
                if inner_at(&corners[k]) {
                    st.len - e
                } else {
                    st.len + e
                }
            };
            let mut a = a0;
            while a < a1 - int(6) {
                let (sty, len, depth, height) = pick(s_from + a, river, dice);
                let mut len = len.min(a1 - a);
                let mut placed = false;
                while len >= int(8) {
                    let mid = st
                        .from
                        .add(st.dir.scale(a + len / 2))
                        .add(r.scale(side * (hw + depth / 2)));
                    let b = Block {
                        centre: v3(mid.x, ground(mid.x, mid.z), mid.z),
                        fwd: st.dir,
                        half_len: len / 2,
                        half_width: depth / 2,
                        height,
                        style: sty,
                    };
                    if fits(&b, &out) {
                        out.push(b);
                        placed = true;
                        break;
                    }
                    len -= int(4);
                }
                a += if placed { len } else { int(2) };
            }
            // round the plaza past the finish: set back
            if k == n - 1 {
                let mut a = st.len + int(16);
                while a < st.len + OUTRO - int(10) {
                    let (_, len, depth, height) = pick(int(1600), river, dice);
                    let mid = st
                        .from
                        .add(st.dir.scale(a + len / 2))
                        .add(r.scale(side * (hw + int(25) + depth / 2)));
                    let b = Block {
                        centre: v3(mid.x, ground(mid.x, mid.z), mid.z),
                        fwd: st.dir,
                        half_len: len / 2,
                        half_width: depth / 2,
                        height,
                        style: style::OFFICE,
                    };
                    if fits(&b, &out) {
                        out.push(b);
                    }
                    a += len;
                }
            }
        }
    }
    let _ = straights;
    out
}

/// What a vine was placed to hang from, before the buildings are known.
#[derive(Debug, Clone, Copy)]
enum Hang {
    /// across the street: a wire if both facades are tall enough
    Street,
    /// the start's row: wires between the towers
    Wire,
    Crane(Corner),
    Helicopter,
    Cable,
}

/// The street a point is in: one whose own length (not a corner's mitre)
/// it is beside, else the nearest.
fn street_of(c: &Course, p: V3) -> Street {
    let at = |st: &Street| p.flat().sub(st.from).dot(st.dir);
    c.streets
        .iter()
        .copied()
        .min_by_key(|st| {
            let a = at(st);
            let side = p.flat().sub(st.from).dot(right_of(st.dir)).abs();
            let off = (-a).max(a - st.len).max(0);
            off as i64 * 4 + side as i64
        })
        .unwrap_or(c.streets[0])
}

/// How tall the building just behind the wall at `p` is (0: none).
fn height_at(blocks: &[Block], p: V3) -> i32 {
    blocks
        .iter()
        .filter(|b| Obb::of(b).holds(p))
        .map(|b| b.height)
        .max()
        .unwrap_or(0)
}

fn hangers(c: &Course, blocks: &[Block], hangs: &[(usize, Hang)], dice: &mut Dice) -> Vec<Hanger> {
    let hw = c.half_width;
    let mut out = Vec::new();
    for &(i, hang) in hangs {
        let v = c.vines[i];
        let a = v.anchor;
        let st = street_of(c, a);
        let r = right_of(st.dir);
        let on_line = st.from.add(st.dir.scale(a.flat().sub(st.from).dot(st.dir)));
        let facade = |side: i32, y: i32| {
            let p = on_line.add(r.scale(side * hw));
            v3(p.x, y, p.z)
        };
        let behind = |side: i32| height_at(blocks, on_line.add(r.scale(side * (hw + int(2)))));
        let (kind, from, to) = match hang {
            Hang::Cable => (HangerKind::BridgeCable, a, a),
            Hang::Helicopter => (HangerKind::Helicopter, a.add(v3(0, int(3), 0)), a),
            Hang::Crane(corner) => {
                let site = corner
                    .p
                    .add(corner.din.scale(-hw - int(8)))
                    .add(corner.inward().scale(hw + int(8)));
                (
                    HangerKind::Crane,
                    v3(site.x, ground(site.x, site.z), site.z),
                    a,
                )
            }
            Hang::Wire | Hang::Street => {
                let tall = behind(-1).min(behind(1)) >= a.y + int(3);
                let side = if a.flat().sub(on_line).dot(r) >= 0 {
                    1
                } else {
                    -1
                };
                if tall || matches!(hang, Hang::Wire) {
                    (HangerKind::Wire, facade(-1, a.y), facade(1, a.y))
                } else if a.y - ground(a.x, a.z) <= int(30) {
                    // a tall lamp mast at the kerb, its arm out over the street
                    let p = on_line.add(r.scale(side * (hw - cm(50))));
                    let foot = v3(p.x, 0, p.z);
                    (
                        HangerKind::StreetLight,
                        v3(foot.x, ground(foot.x, foot.z), foot.z),
                        a,
                    )
                } else if dice.below(6) > 0 && behind(-1).min(behind(1)) < a.y - int(4) {
                    // a crane on the lower roof beside it
                    let low = if behind(-1) <= behind(1) { -1 } else { 1 };
                    let p = on_line.add(r.scale(low * (hw + int(6))));
                    (HangerKind::Crane, v3(p.x, behind(low), p.z), a)
                } else {
                    (HangerKind::Helicopter, a.add(v3(0, int(3), 0)), a)
                }
            }
        };
        out.push(Hanger {
            vine: i,
            kind,
            from,
            to,
        });
    }
    out.sort_by_key(|h| h.vine);
    out
}

/// Boats on the river beside the bridge's deck, both sides by turns.
fn boats(c: &mut Course, lay: &mut CityLayout, bridge: &Bridge, river: &River) {
    let fwd = bridge.end.sub(bridge.start).flat().norm();
    let r = right_of(fwd);
    let kinds = [BoatKind::Tug, BoatKind::Barge, BoatKind::Ferry];
    let mut t = int(26);
    let mut k = 0;
    while t < RIVER_WIDE + DECK_ON - int(16) {
        let side = if k % 2 == 0 { 1 } else { -1 };
        let p = bridge
            .start
            .add(fwd.scale(t))
            .add(r.scale(side * (DECK_HALF + cm(450))));
        let kind = kinds[k % kinds.len()];
        let at = v3(p.x, river.level, p.z);
        let (top, radius) = boat_size(kind);
        c.mushrooms.push(Mushroom {
            top: at.add(v3(0, top, 0)),
            radius,
        });
        lay.boats.push(Boat {
            at,
            yaw: yaw_of(r),
            kind,
            bouncer: c.mushrooms.len() - 1,
        });
        t += int(24);
        k += 1;
    }
}
