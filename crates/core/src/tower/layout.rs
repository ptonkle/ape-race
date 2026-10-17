//! Mape's Tower's course (the second design's stand-in, 2026-10-16): the
//! 3D centreline leg by leg and its placement.
//!
//! The legs (`LEGS`), each a straight or a turn with its rise, start
//! facing −x off the start deck at tower 1's foot: the run under the
//! lower bridge (two old spirals long), a corner onto the spiral round
//! tower 2 (one full turn at `R2`, plus the quarter corners either end:
//! a turn and a half of turning in all), a corner off it under the upper
//! bridge back to tower 1's doorway, in and a corner onto a loose coil up
//! tower 1's hollow (three turns at `R1_COIL`, what progress measures;
//! the beams inside do not follow it), a corner out over the rim and
//! sideways through the tear, straight down the tube (the heading is
//! straight down: `Course.right` there is the right of the way through
//! the portal), a bend into the tunnel's mouth and the tunnel to the
//! finish.
//!
//! Vines: under the bridges a station every `GAP` m, two of three lanes,
//! hung from the deck's underside (`Hanger::Bridge`); round tower 2 the
//! same from balcony mages, dragons and the cloud band; inside tower 1
//! rings of beams every `LAYER_RISE` m of height, from windows and inner
//! balconies leaning in (`LEAN_IN`) and from conjured anchors floating
//! in the middle, scattered round the hollow by the dice (no helix);
//! the sling row at the bottom of the tube, five beams hung from the
//! tunnel mouth's top edge `SLING_DIP` under level. The tube's pads:
//! `TUBE_PADS` across it, their gaps a sine weave of `WEAVE_AMPLITUDE`
//! over `WEAVE_PERIOD` pads, measured reachable (TW2-5). Reset pads a few
//! metres under the centreline every other station (ledges on tower 1's
//! inner wall inside), each throwing an ape up and on to the next beam
//! (`boost`); the portal's pad is where a reset in the tube goes.
//!
//! Placement draws from `Dice(seed ^ 0x70E5)`.

use super::*;
use crate::course::{centreline, Course, Platform, Vine, STEP};
use crate::fx::{self, cos, deg, mul, sin, Dice, TURN};
use crate::geom::{facing, yaw_of};
use crate::rules::{BODY_DROP, GRAVITY};
use crate::sector::DOWN;

const SEED: u64 = 0x70E5_0000_0001;

/// The start deck's top, and so the centreline's height at the start.
pub const START_TOP: i32 = int(16);
/// How far the tear's middle stands over the line (the apes fly high).
const TEAR_RISE: i32 = int(8);

/// A leg of the course: metres along the path (3D; even), the turn over
/// it in degrees (positive to the left), and the metres it rises
/// (negative: falls).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Climb(pub i32, pub i32, pub i32);

/// The course's legs, in order, each tagged with the leg of the race it
/// belongs to.
pub const LEGS: &[(Leg, Climb)] = &[
    (Leg::Start, Climb(30, 0, 0)),           // 0 the run-in off the start deck
    (Leg::LowerBridge, Climb(250, 0, 4)),    // 1 under the lower bridge
    (Leg::Tower2, Climb(50, -90, 8)),        // 2 the corner onto the spiral
    (Leg::Tower2, Climb(390, 360, 100)),     // 3 round tower 2
    (Leg::Tower2, Climb(50, -90, 8)),        // 4 the corner off it
    (Leg::UpperBridge, Climb(250, 0, 4)),    // 5 under the upper bridge
    (Leg::Inside, Climb(100, 0, 6)),         // 6 on to and through the doorway
    (Leg::Inside, Climb(50, -90, 8)),        // 7 the corner onto the coil
    (Leg::Inside, Climb(472, -1080, 150)),   // 8 the coil up the hollow
    (Leg::Portal, Climb(50, 90, 10)),        // 9 the corner out over the rim
    (Leg::Portal, Climb(40, 0, 10)),         // 10 through the tear to the tube
    (Leg::Tube, Climb(280, 0, -280)),        // 11 straight down the tube
    (Leg::Sling, Climb(40, 0, -24)),         // 12 the bend into the mouth
    (Leg::Tunnel, Climb(70, 0, 0)),          // 13 the tunnel to the finish
];
/// The legs' indices.
const SPIRAL: usize = 3;
const COIL: usize = 8;
const TUBE: usize = 11;
const BEND: usize = 12;
/// The start faces −x (toward tower 2).
const START_YAW: i32 = -TURN / 4;

/// Tower 2's radius (the old keep's, `towermodels::TOWER2_RADIUS`) and how far outside its wall
/// the spiral runs (the old keep's gap).
const R2: i32 = crate::towermodels::TOWER2_RADIUS;

/// Tower 1's outer radius (1.25× the old keep's) and its wall's
/// thickness; the coil's radius inside it.
const R1: i32 = int(51);
const WALL_THICK: i32 = int(6);
/// Tower 1's height: about twice the old keep's.
const H1: i32 = int(300);
/// Tower 2's top stands this far over the upper bridge's deck.
const CROWN_OVER: i32 = int(3);
/// The bridges: half width, the deck's thickness (the beams hang from
/// its underside).
const BRIDGE_HALF: i32 = int(8);
const BRIDGE_THICK: i32 = int(3);
/// The beams' anchors hang this far under a deck's underside.
const DECK_CLEAR: i32 = int(2);
/// The doorway's half width and height.
const DOOR_HALF: i32 = int(10);
const DOOR_HEIGHT: i32 = int(22);

/// A balcony's depth out from its wall.
const BALCONY_DEPTH: i32 = int(3);

/// A station every this many metres.
const GAP: i32 = 22;
/// Vines hang from this far over the centreline at their place, to this
/// far over it.
const ANCHOR_OVER: i32 = int(26);
const BOTTOM_OVER: i32 = int(2);
/// Inside tower 1: a ring of beams every this many metres of height, this
/// many round, their length. A beam cast from a window or an inner
/// balcony is anchored out in the hollow, `CAST_IN` from the axis (the
/// caster's spell reaches to it), so a swing on it stays clear of the
/// wall (the audit); a floating one within `FLOAT_IN` of the axis.
pub const LAYER_RISE: i32 = int(7);
const LAYER_BEAMS: usize = 8;
/// The rings go on this far over the rim, so the last swings inside reach
/// the portal's beams over the top.
const RINGS_OVER_RIM: i32 = int(20);
const INSIDE_BEAM: i32 = int(22);
const CAST_IN: (i32, i32) = (14, 22);
const FLOAT_IN: i32 = 10;
/// Reset pads stand this far under the centreline: under the swings, and
/// in a jump's reach of the vines.
const PAD_UNDER: i32 = int(5);
/// Reset pads stand this far short of their station.
const PAD_BEFORE: i32 = int(10);

/// The tube: its inside radius (2/3 of an old spiral's width), its wall,
/// the pads' count and the sine weave of their gaps (TW2-5), the pads'
/// gap's radius.
const TUBE_RADIUS: i32 = int(17) + cm(50);
const TUBE_WALL: i32 = int(4);
pub const TUBE_PADS: usize = 22;
pub const WEAVE_AMPLITUDE: i32 = int(7);
pub const WEAVE_PERIOD: i32 = 12;
const HOLE: i32 = int(6);
/// The first pad this far under the tube's top, the last this far over
/// the sling row's anchors.
const PADS_FROM_TOP: i32 = int(30);
const PADS_OVER_SLING: i32 = int(24);
/// The tunnel: its mouth's height (the sling anchors on its top edge),
/// its half width, its length past the mouth, the sling row's dip under
/// level, its beams' length and lanes.
const MOUTH_HEIGHT: i32 = int(30);
const TUNNEL_HALF: i32 = int(16);
const TUNNEL_LEN: i32 = int(250);
const SLING_DIP: i32 = deg(10);
const SLING_LEN: i32 = int(24);
/// The tunnel's floor is this far under the tube leg's end less the bend.
const FLOOR_UNDER_LINE: i32 = int(6);

/// A reset's boost from the hands at `from` to a beam's bottom `to`: the
/// throw that tops out a metre over it, right at it.
fn boost(from: V3, to: V3) -> V3 {
    let d = to.sub(from);
    let up = (d.y + int(1)).max(int(3));
    let vy = fx::sqrt(2 * mul(GRAVITY, up));
    // (seconds to the top: vy / g; the speed across to be there then)
    let t = fx::div(vy, GRAVITY);
    let across = d.flat();
    across
        .norm()
        .scale(fx::div(across.len(), t))
        .add(v3(0, vy, 0))
}

/// Mape's Tower, laid out from its fixed seed.
pub fn tower() -> Course {
    tower_seeded(SEED)
}

/// Where leg `k` begins, in metres along (fixed point).
fn leg_at(k: usize) -> i32 {
    LEGS[..k].iter().map(|l| l.1 .0 * ONE / STEP * STEP).sum()
}

/// The centreline from `legs`, starting at `start` facing `yaw`: samples
/// `STEP` apart in 3D and their 3D unit headings, as `centreline` does in
/// plan (each step's yaw is the leg's turn at the step's middle).
fn climb_path(legs: &[Climb], start: V3, yaw: i32) -> (Vec<V3>, Vec<V3>) {
    let mut path = Vec::new();
    let mut heading = Vec::new();
    let mut p = start;
    let mut yaw = yaw;
    for leg in legs {
        let steps = leg.0 * ONE / STEP;
        let turn = deg(leg.1);
        // each step's rise, and its run in plan, so the step is STEP long
        let dy = (STEP as i64 * leg.2 as i64 / leg.0 as i64) as i32;
        let run = fx::sqrt(mul(STEP, STEP) - mul(dy, dy));
        for k in 0..steps {
            let y = yaw + (turn as i64 * (2 * k as i64 + 1) / (2 * steps as i64)) as i32;
            let f = facing(y, 0);
            let step = v3(mul(f.x, run), dy, mul(f.z, run));
            path.push(p);
            heading.push(v3(
                fx::div(step.x, STEP),
                fx::div(step.y, STEP),
                fx::div(step.z, STEP),
            ));
            p = p.add(step);
        }
        yaw += turn;
    }
    path.push(p);
    heading.push(facing(yaw, 0));
    (path, heading)
}

fn flat(h: V3) -> V3 {
    v3(h.x, 0, h.z).norm()
}

/// The middle of the circle leg `k` turns round (in plan, at the leg's
/// start height): left of its start by its plan radius.
fn turn_centre(c: &Course, k: usize) -> (V3, i32) {
    let leg = LEGS[k].1;
    let at = leg_at(k);
    // (the run in plan, in metres: in fixed point the squares of these
    // legs overflow, TW2-3; then the plan radius, the run over the turn in
    // radians)
    let run = ((leg.0 as i64).pow(2) - (leg.2 as i64).pow(2)).isqrt();
    let r = (run * 3_600_000 * ONE as i64 / (62_832 * leg.1.abs() as i64)) as i32;
    let left = right_of(flat(c.heading_at(at))).neg();
    let side = if leg.1 > 0 { left } else { left.neg() };
    (c.point(at).add(side.scale(r)), r)
}

/// A beam's layout: its lowest point (`bottom`), its way down (`hang`,
/// unit) and length: the anchor is up the beam from the bottom.
fn beam(bottom: V3, hang: V3, length: i32, at: i32) -> Vine {
    Vine {
        anchor: bottom.sub(hang.scale(length)),
        length,
        slide: None,
        at,
    }
}

/// The pad `k` of `n` down the tube: its gap's offset from the axis (a
/// sine weave across z) and its shape (a ring round the gap, a slab open
/// past it, or quarters where the weave crosses the middle).
fn tube_pad(k: usize, y: i32) -> TubePad {
    let phase = (k as i64 * TURN as i64 / WEAVE_PERIOD as i64) as i32;
    let off = mul(WEAVE_AMPLITUDE, sin(phase));
    let across = v3(0, 0, ONE);
    let gap = across.scale(off);
    let shape = match k % 3 {
        0 => PadShape::Ring { at: gap, hole: HOLE },
        1 if off.abs() > HOLE => PadShape::Slab {
            open: across.scale(ONE * off.signum()),
            edge: off.abs() - HOLE,
        },
        1 => PadShape::Ring { at: gap, hole: HOLE },
        _ if off.abs() < int(2) => PadShape::Quarters {
            axis: v3(ONE, 0, 0),
        },
        _ => PadShape::Ring { at: gap, hole: HOLE },
    };
    TubePad { y, shape, gap }
}

/// Mape's Tower from another seed (the beams shift a little).
pub fn tower_seeded(seed: u64) -> Course {
    let mut c = centreline("tower", &[]);
    let legs: Vec<Climb> = LEGS.iter().map(|l| l.1).collect();
    let (path, heading) = climb_path(&legs, v3(0, START_TOP, 0), START_YAW);
    // (down the tube the heading has no plan direction: the right stays
    // the last level one)
    let mut right = Vec::with_capacity(heading.len());
    let mut last = right_of(facing(START_YAW, 0));
    for h in &heading {
        if h.flat().longer_than(ONE / 10) {
            last = right_of(*h);
        }
        right.push(last);
    }
    c.right = right;
    c.spacing = path
        .windows(2)
        .map(|w| w[1].dist(w[0]) + 1)
        .max()
        .unwrap_or(STEP);
    c.length = (path.len() as i32 - 1) * STEP;
    c.path = path;
    c.heading = heading;
    c.half_width = int(20);
    let mut dice = Dice(seed ^ 0x70E5);

    let mut hangers = Vec::new();
    let mut lifts = Vec::new();
    let mut hangs = Vec::new();

    // the start deck, the starters across it
    let fwd = flat(c.heading_at(0));
    let right = right_of(fwd);
    let p0 = c.point(int(4));
    c.platforms.push(Platform {
        centre: v3(p0.x, START_TOP, p0.z),
        fwd,
        half_len: int(9),
        half_width: int(9),
        top: START_TOP,
    });
    for k in 0..4 {
        let at = c.point(int(2)).add(right.scale(int(-6) + k * int(4)));
        c.starts[k as usize] = v3(at.x, START_TOP + BODY_DROP, at.z);
    }
    c.start_yaw = START_YAW;
    // the start row, as the Jungle's: just past the deck's edge, to a
    // metre over it
    // (hung from the lower bridge's underside: TW2-4)
    for k in -2..=2 {
        let at = int(15);
        let base = c.point(at).add(right.scale(int(3) * k));
        c.vines.push(Vine {
            anchor: v3(base.x, START_TOP + ANCHOR_OVER, base.z),
            length: ANCHOR_OVER - int(1),
            slide: None,
            at,
        });
        hangers.push(Hanger::Start);
        lifts.push(false);
        hangs.push(DOWN);
    }

    // the towers: tower 2 round the spiral, tower 1 round the coil
    let (c2, _) = turn_centre(&c, SPIRAL);
    let (c1, _) = turn_centre(&c, COIL);
    let t2_centre = v3(c2.x, 0, c2.z);
    let t1_centre = v3(c1.x, 0, c1.z);
    let t2_radius = R2;
    let upper_end = c.point(leg_at(5 + 1));
    let t2_height = upper_end.y + ANCHOR_OVER + BRIDGE_THICK + CROWN_OVER;

    // the doorway: where the upper bridge's line meets tower 1's wall
    let door_at = {
        let from = leg_at(5);
        let to = leg_at(COIL);
        let mut s = from;
        while s < to && c.point(s).flat().sub(t1_centre.flat()).longer_than(R1) {
            s += ONE;
        }
        s
    };
    let door_p = c.point(door_at);
    let door_out = door_p.flat().sub(t1_centre.flat()).norm();
    let doorway = Doorway {
        at: v3(door_p.x, door_p.y - PAD_UNDER, door_p.z),
        out: door_out,
        half_width: DOOR_HALF,
        height: DOOR_HEIGHT,
    };

    // the bridges: from tower 1's wall to tower 2's and back, their decks
    // over the line (the beams hang from the underside)
    // (where a level ray from `p` along `dir` meets a tower's wall, at
    // `p`'s height; `p` itself if it never does)
    let wall_hit = |p: V3, dir: V3, centre: V3, r: i32| -> V3 {
        let to = centre.flat().sub(p.flat());
        let along = to.dot(dir);
        let perp = to.sub(dir.scale(along));
        if perp.longer_than(r) || along <= 0 {
            return p;
        }
        let in_by = fx::sqrt(mul(r, r) - perp.dot(perp));
        p.add(dir.scale(along - in_by))
    };
    // (the deck's underside a body's clearance over the anchors)
    let deck_over = ANCHOR_OVER + BRIDGE_THICK + DECK_CLEAR;
    let lower_dir = flat(c.heading_at(leg_at(1)));
    let lower_from = wall_hit(c.point(int(4)), lower_dir.neg(), t1_centre, R1);
    let lower_to = wall_hit(c.point(leg_at(1)), lower_dir, t2_centre, t2_radius);
    let upper_dir = flat(c.heading_at(leg_at(5)));
    let upper_from = wall_hit(door_p, upper_dir.neg(), t2_centre, t2_radius);
    let mut bridges = Vec::new();
    let lower_end = c.point(leg_at(2));
    for (from, to, high) in [
        (lower_from, lower_to, lower_end.y),
        (upper_from, door_p, door_p.y),
    ] {
        // (level, over the higher end of the line under it)
        let deck = from.y.max(to.y).max(high) + deck_over;
        let (from, to) = (v3(from.x, deck, from.z), v3(to.x, deck, to.z));
        let mid = from.add(to).scale(ONE / 2);
        c.platforms.push(Platform {
            centre: mid,
            fwd: flat(to.sub(from)),
            half_len: from.dist(to) / 2,
            half_width: BRIDGE_HALF,
            top: mid.y,
        });
        bridges.push(Bridge {
            from,
            to,
            half_width: BRIDGE_HALF,
            thick: BRIDGE_THICK,
            deck: Some(c.platforms.len() - 1),
        });
    }

    // the balconies and their mages: one on tower 2's wall facing each
    // station of the spiral, a few metres under its beams; inside tower 1
    // the inner balconies the inside beams are cast from
    let mut balconies: Vec<Balcony> = Vec::new();
    let mut balcony_for = |p: V3, y: i32, tower: usize, inside: bool| -> usize {
        let (o, r) = if tower == 1 {
            (t2_centre, t2_radius)
        } else {
            (t1_centre, R1 - WALL_THICK)
        };
        let out = if inside {
            o.flat().sub(p.flat()).norm()
        } else {
            p.flat().sub(o.flat()).norm()
        };
        let wall = if inside {
            o.add(out.neg().scale(r))
        } else {
            o.add(out.scale(r))
        };
        let at = wall.add(out.scale(BALCONY_DEPTH));
        balconies.push(Balcony {
            at: v3(at.x, y, at.z),
            out,
            half_width: int(3),
            depth: BALCONY_DEPTH,
            mage: true,
            deck: None,
            tower,
            inside,
        });
        balconies.len() - 1
    };
    let leg_of = |s: i32| -> usize {
        (0..LEGS.len())
            .rev()
            .find(|&k| leg_at(k) <= s)
            .unwrap_or(0)
    };
    let clouds = CloudBand {
        bottom: t2_height - int(50),
        top: t2_height - int(26),
    };

    // the stations under the bridges and round tower 2
    let mut s = 36;
    let mut station = 0;
    // (the last station a swing short of the doorway's wall)
    while int(s) < door_at - int(24) {
        let at0 = int(s);
        s += GAP;
        station += 1;
        let leg = leg_of(at0);
        let tag = LEGS[leg].0;
        let h = c.heading_at(at0);
        let r = right_of(h);
        let lanes: [i32; 2] = match dice.below(3) {
            0 => [0, 7],
            1 => [-7, 7],
            _ => [-7, 0],
        };
        for &lane in lanes.iter() {
            let along = at0 + int(dice.range(-2, 2));
            let base = c.point(along).add(r.scale(int(lane)));
            let bottom = v3(base.x, base.y + BOTTOM_OVER, base.z);
            let mut v = beam(bottom, DOWN, ANCHOR_OVER - BOTTOM_OVER, along);
            // (a beam whose anchor would be in a deck hangs from the
            // deck's underside instead: the bridges run on over the
            // spiral's first and last stations, TW2-4)
            let in_deck = bridges.iter().position(|b| {
                let run = b.to.sub(b.from).flat();
                let fwd = run.norm();
                let rel = v.anchor.sub(b.from).flat();
                let along = rel.dot(fwd);
                along > -int(2)
                    && along < run.len() + int(2)
                    && rel.dot(right_of(fwd)).abs() < b.half_width + int(2)
                    && v.anchor.y > b.from.y.min(b.to.y) - b.thick - int(2)
                    && v.anchor.y < b.from.y.max(b.to.y) + int(12)
            });
            if let Some(n) = in_deck {
                let b = &bridges[n];
                let under = b.from.y.min(b.to.y) - b.thick - DECK_CLEAR;
                v.length -= v.anchor.y - under;
                v.anchor.y = under;
            }
            let under_bridge = in_deck.is_some()
                || matches!(tag, Leg::LowerBridge | Leg::UpperBridge | Leg::Start)
                || (tag == Leg::Inside && along < door_at);
            let hanger = if under_bridge {
                Hanger::Bridge {
                    bridge: in_deck.unwrap_or(if along < leg_at(SPIRAL) { 0 } else { 1 }),
                }
            } else if v.anchor.y >= clouds.bottom && v.anchor.y <= clouds.top + int(10) {
                Hanger::Cloud { band: 0 }
            } else if lane > 0 && station % 3 == 0 || leg != SPIRAL {
                // a dragon outside the spiral, and on the corners
                Hanger::Dragon {
                    at: v.anchor.add(r.scale(int(9))).add(v3(0, int(4), 0)),
                }
            } else {
                Hanger::Mage {
                    balcony: balcony_for(bottom, v.anchor.y - int(4), 1, false),
                }
            };
            c.vines.push(v);
            hangers.push(hanger);
            // (only the spiral's beams lift: under the level bridges a
            // lift only carries an ape over the next beams)
            lifts.push(tag == Leg::Tower2);
            hangs.push(DOWN);
        }
    }

    // inside tower 1: rings of beams every LAYER_RISE of height, from
    // windows and inner balconies leaning in and from anchors floating
    // in the middle, scattered round by the dice
    let mut windows: Vec<Window> = Vec::new();
    let inner = R1 - WALL_THICK;
    let rim = t1_centre.y + H1;
    // (two entry beams just inside the doorway, over the line as the
    // bridge's, so the climb inside can be started from the swing in)
    for k in [1, 2] {
        let at = door_at + int(10) * k;
        let r = right_of(flat(c.heading_at(at)));
        for lane in [-4, 4] {
            let base = c.point(at).add(r.scale(int(lane)));
            let bottom = v3(base.x, base.y + BOTTOM_OVER, base.z);
            let v = beam(bottom, DOWN, ANCHOR_OVER - BOTTOM_OVER, at);
            hangers.push(Hanger::Floating { at: v.anchor });
            c.vines.push(v);
            lifts.push(true);
            hangs.push(DOWN);
        }
    }
    // the rings: the first's anchors a beam's length over the entry line
    let mut y = doorway.at.y + PAD_UNDER + ANCHOR_OVER;
    let mut ring = 0u32;
    while y < rim + RINGS_OVER_RIM {
        let base_yaw = dice.range(0, TURN);
        for n in 0..LAYER_BEAMS {
            let yaw = base_yaw + (TURN / LAYER_BEAMS as i32) * n as i32 + dice.range(-TURN / 24, TURN / 24);
            let out = facing(yaw, 0);
            let floating = (ring as usize + n) % 4 == 3;
            let (anchor, hang, hanger) = if floating {
                let rad = int(dice.range(0, FLOAT_IN));
                let a = t1_centre.add(out.scale(rad)).add(v3(0, y, 0));
                (a, DOWN, Hanger::Floating { at: a })
            } else {
                let wall = t1_centre.add(out.scale(inner - int(1))).add(v3(0, y, 0));
                let rad = int(dice.range(CAST_IN.0, CAST_IN.1));
                let a = t1_centre.add(out.scale(rad)).add(v3(0, y, 0));
                let hanger = if n % 2 == 0 {
                    windows.push(Window {
                        at: v3(wall.x, wall.y - int(2), wall.z),
                        into: out.neg(),
                        half_width: cm(150),
                        height: int(4),
                    });
                    Hanger::Window {
                        window: windows.len() - 1,
                    }
                } else {
                    Hanger::Mage {
                        balcony: balcony_for(wall, wall.y - int(4), 0, true),
                    }
                };
                (a, DOWN, hanger)
            };
            let bottom = anchor.add(hang.scale(INSIDE_BEAM));
            // (in 3D: `Course.tower` is not set until the layout is done,
            // so the course's own search would be in plan)
            let hint = c.nearest_anywhere_3d(bottom);
            let (at, _) = c.progress_3d(bottom, hint);
            c.vines.push(Vine {
                anchor,
                length: INSIDE_BEAM,
                slide: None,
                at,
            });
            hangers.push(hanger);
            lifts.push(true);
            hangs.push(hang);
        }
        y += LAYER_RISE;
        ring += 1;
    }

    // the portal's beams: the corner out over the rim and through the
    // tear, cast by dragons
    let portal_from = leg_at(9);
    let tube_from = leg_at(TUBE);
    let mut s = portal_from + int(8);
    while s < tube_from - int(10) {
        let h = c.heading_at(s);
        let r = right_of(h);
        for lane in [-6, 6] {
            let base = c.point(s).add(r.scale(int(lane)));
            let bottom = v3(base.x, base.y + BOTTOM_OVER, base.z);
            let v = beam(bottom, DOWN, ANCHOR_OVER - BOTTOM_OVER, s);
            hangers.push(Hanger::Dragon {
                at: v.anchor.add(r.scale(int(9) * lane.signum())).add(v3(0, int(4), 0)),
            });
            c.vines.push(v);
            lifts.push(false);
            hangs.push(DOWN);
        }
        s += GAP * ONE;
    }

    // the tear: giant, its middle over the line where the apes fly
    // through it, the wizard beside its left edge
    let tear_at = portal_from + int(60);
    let tear_half = (int(25), int(28));
    let tear = c.point(tear_at).add(v3(0, TEAR_RISE, 0));
    let through = flat(c.heading_at(tear_at));
    let tear_left = right_of(through).neg();
    let wizard = tear
        .add(tear_left.scale(tear_half.1 + int(5)))
        .sub(v3(0, int(8), 0));

    // the tube: down from the portal leg's end; its pads
    let tube_top_p = c.point(tube_from);
    let bend_from = leg_at(BEND);
    let bend_to = leg_at(BEND + 1);
    let floor = c.point(bend_to).y - FLOOR_UNDER_LINE;
    let axis = v3(tube_top_p.x, 0, tube_top_p.z);
    let mouth_top = floor + MOUTH_HEIGHT;
    let mut pads = Vec::new();
    let first = tube_top_p.y - PADS_FROM_TOP;
    let last = mouth_top + PADS_OVER_SLING;
    for k in 0..TUBE_PADS {
        let y = first - (first - last) * k as i32 / (TUBE_PADS as i32 - 1);
        pads.push(tube_pad(k, y));
    }
    let tube = Tube {
        axis,
        top: tube_top_p.y,
        bottom: floor,
        radius: TUBE_RADIUS,
        wall: TUBE_WALL,
        pads,
    };

    // the tunnel off the tube's bottom, its floor from the tube's far
    // wall to its end, the sling row hung from its mouth's top edge
    let along = flat(c.heading_at(bend_to));
    let mouth = axis.add(along.scale(TUBE_RADIUS)).add(v3(0, floor, 0));
    let floor_from = axis.sub(along.scale(TUBE_RADIUS));
    let floor_to = mouth.add(along.scale(TUNNEL_LEN));
    let floor_mid = floor_from.add(floor_to).scale(ONE / 2);
    c.platforms.push(Platform {
        centre: v3(floor_mid.x, floor, floor_mid.z),
        fwd: along,
        half_len: floor_from.dist(floor_to) / 2,
        half_width: TUNNEL_HALF,
        top: floor,
    });
    let floor_platform = c.platforms.len() - 1;
    let finish = c.point(c.length).sub(mouth).dot(along);
    let tunnel = Tunnel {
        mouth,
        along,
        half_width: TUNNEL_HALF,
        height: MOUTH_HEIGHT,
        length: TUNNEL_LEN,
        finish,
        floor: floor_platform,
        sling_dip: SLING_DIP,
    };
    let sling_hang = along
        .neg()
        .scale(cos(SLING_DIP))
        .add(v3(0, -sin(SLING_DIP), 0))
        .norm();
    let sling_at = bend_from + int(8);
    for k in -2..=2 {
        let anchor = v3(mouth.x, mouth_top, mouth.z).add(right_of(along).scale(int(3) * k));
        c.vines.push(Vine {
            anchor,
            length: SLING_LEN,
            slide: None,
            at: sling_at,
        });
        hangers.push(Hanger::Sling);
        lifts.push(false);
        hangs.push(sling_hang);
    }

    // the legs' spans (the inside begins at the doorway)
    let mut legs: Vec<LegSpan> = Vec::new();
    for (k, (leg, _)) in LEGS.iter().enumerate() {
        let from = if *leg == Leg::Inside && legs.last().is_none_or(|l| l.leg != Leg::Inside) {
            door_at
        } else {
            leg_at(k)
        };
        if legs.last().is_none_or(|l| l.leg != *leg) {
            legs.push(LegSpan { leg: *leg, from });
        }
    }

    let mut layout = TowerLayout {
        towers: [
            Tower {
                centre: t1_centre,
                radius: R1,
                inner,
                height: H1,
                levels: (1..(H1 / int(12))).map(|k| int(12) * k).collect(),
            },
            Tower {
                centre: t2_centre,
                radius: t2_radius,
                inner: 0,
                height: t2_height,
                levels: (1..(t2_height / int(12))).map(|k| int(12) * k).collect(),
            },
        ],
        bridges: [bridges.remove(0), bridges.remove(0)],
        doorway,
        balconies,
        windows,
        clouds: vec![clouds],
        portal: Portal {
            wizard,
            // facing the apes coming up, arms flung along the tear
            wizard_yaw: yaw_of(through.neg()),
            tear,
            tear_half,
            through,
            banana: tear.add(through.scale(ONE / 2)),
            banana_len: int(30),
        },
        tube,
        tunnel,
        banana_zone: BananaZone {
            centre: V3::ZERO,
            radius: int(40),
            platform: floor_platform,
        },
        legs,
        pads: Vec::new(),
        portal_pad: 0,
        hangers: Vec::new(),
        lifts: Vec::new(),
        hangs: Vec::new(),
        lobby: (c.starts[0], 0),
        stone: Vec::new(),
    };

    // item crates: a row across the course every 240 m, on every beam of
    // the station nearest each place, never on the start row or the
    // sling (TI1)
    // (not the sling, not the start row, and not the scattered beams
    // inside tower 1: a row is a station's beams, and theirs cluster)
    let carries = |h: Hanger| {
        matches!(
            h,
            Hanger::Bridge { .. } | Hanger::Dragon { .. } | Hanger::Cloud { .. }
        ) || matches!(h, Hanger::Mage { balcony } if !layout.balconies[balcony].inside)
    };
    let mut row = 150;
    while int(row) < c.length - int(60) {
        let si = int(row);
        let near = (0..c.vines.len())
            .filter(|&i| carries(hangers[i]))
            .min_by_key(|&i| (c.vines[i].at - si).abs());
        if let Some(n) = near {
            let at = c.vines[n].at;
            for i in 0..c.vines.len() {
                if carries(hangers[i]) && (c.vines[i].at - at).abs() <= int(4) {
                    c.crates.push(c.vines[i].anchor);
                }
            }
        }
        row += 240;
    }

    // the reset pads: the start deck, then one every other station under
    // the bridges and round tower 2, ledges on tower 1's inner wall
    // inside, the portal's at the tear's foot; each throws an ape up to
    // the next beam's bottom
    layout.pads.push(Pad {
        platform: 0,
        at: 0,
        boost: V3::ZERO,
    });
    // (the nearest beam's bottom ahead of the pad along the course and
    // not under it, by distance: by `at` alone a ledge inside tower 1
    // aimed at a beam 100 m up and threw an ape 70 m/s, TW2-6)
    let next_bottom = |c: &Course, at: i32, from: V3| -> V3 {
        (0..c.vines.len())
            .filter(|&i| {
                c.vines[i].at >= at + int(4) && hangers[i] != Hanger::Start && hangers[i] != Hanger::Sling
            })
            .map(|i| c.vines[i].anchor.add(hangs[i].scale(c.vines[i].length)))
            .filter(|b| b.y >= from.y - int(2))
            .min_by_key(|b| b.sub(from).len())
            .unwrap_or_else(|| c.point(at + int(20)))
    };
    let pad_at = |c: &mut Course, at: i32, centre: V3, fwd: V3, half_len: i32, half_width: i32| -> Pad {
        let top = centre.y;
        c.platforms.push(Platform {
            centre,
            fwd,
            half_len,
            half_width,
            top,
        });
        let to = next_bottom(c, at, v3(centre.x, top + BODY_DROP, centre.z));
        Pad {
            platform: c.platforms.len() - 1,
            at,
            boost: boost(v3(centre.x, top + BODY_DROP, centre.z), to),
        }
    };
    let mut s = 36 + GAP;
    while int(s) < door_at - int(16) {
        let at = int(s) - PAD_BEFORE;
        s += 2 * GAP;
        let p = c.point(at);
        let fwd = flat(c.heading_at(at));
        let pad = pad_at(&mut c, at, v3(p.x, p.y - PAD_UNDER, p.z), fwd, int(4), int(5));
        layout.pads.push(pad);
    }
    // inside: a ledge just inside the doorway, then one on the inner wall
    // every 40 m of height at the coil's angle there, facing the middle
    {
        let sill = doorway.at.sub(doorway.out.scale(WALL_THICK + int(3)));
        let pad = pad_at(
            &mut c,
            door_at + int(2),
            v3(sill.x, doorway.at.y, sill.z),
            doorway.out.neg(),
            int(3),
            int(5),
        );
        layout.pads.push(pad);
    }
    let coil_from = leg_at(COIL);
    let mut s = coil_from + int(20);
    while s < leg_at(COIL + 1) - int(20) {
        let p = c.point(s);
        let out = p.flat().sub(t1_centre.flat()).norm();
        let wall = t1_centre.add(out.scale(inner - int(3)));
        let pad = pad_at(
            &mut c,
            s,
            v3(wall.x, p.y - PAD_UNDER, wall.z),
            out.neg(),
            int(3),
            int(5),
        );
        layout.pads.push(pad);
        s += int(120);
    }
    // the portal's: at the tear's foot, throwing an ape on into the tube
    {
        let at = tear_at - int(10);
        let p = c.point(at);
        let centre = v3(p.x, p.y - PAD_UNDER, p.z);
        c.platforms.push(Platform {
            centre,
            fwd: through,
            half_len: int(4),
            half_width: int(5),
            top: centre.y,
        });
        layout.pads.push(Pad {
            platform: c.platforms.len() - 1,
            at,
            boost: boost(
                v3(centre.x, centre.y + BODY_DROP, centre.z),
                v3(axis.x, tube_top_p.y + int(2), axis.z),
            ),
        });
    }
    layout.pads.sort_by_key(|p| p.at);
    layout.portal_pad = layout
        .pads
        .iter()
        .position(|p| p.at == tear_at - int(10))
        .expect("the portal's pad");

    // the banana zone: the tunnel's floor past the finish
    let end = c.point(c.length);
    let out = flat(c.heading_at(c.length));
    let zone = end.add(out.scale(int(40)));
    layout.banana_zone = BananaZone {
        centre: v3(zone.x, floor, zone.z),
        radius: int(40),
        platform: floor_platform,
    };
    layout.lobby = (c.starts[1], yaw_of(fwd));
    layout.hangers = hangers;
    layout.lifts = lifts;
    layout.hangs = hangs;
    layout.stone = super::stone::stonework(&layout);
    c.tower = Some(Box::new(layout));
    c
}
