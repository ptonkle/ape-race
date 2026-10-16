//! Mape's Tower's course: the 3D centreline (a climb of legs, then its
//! mirror) and its placement (steps 6 and 7, 2026-10-15).
//!
//! The climb (`CLIMB`, 674 m): a level run-in from the start deck, then
//! four spirals up and round the keep (`RADIUS` from its middle, 70° or
//! 60° each), broken by three branches out to little sub-towers. Each
//! branch rises gently and runs straight (no dizzying all-spiral): a bridge
//! off the spiral's tangent, 90° round the sub-tower to the left and a
//! bridge back onto the tangent further round, landing on the same circle
//! (`RADIUS - r` each way). The second branch goes in by the trampoline
//! room's door, up across the room on its trampolines and the orb mage's
//! beams, and out through its roof. The third spiral rises through the
//! cloud band. Off the last spiral the climb turns 90° right, out from the
//! keep, and runs level heading +x (yaw 90°) to the tear, so the keep and
//! its sub-towers stay well behind the mirror (TC5: a tangent summit run
//! put the mirror plane through the keep, and the zipper crossed the
//! climb). Then the zipper, the climb mirrored and run backwards.

//! Vines: a station every `GAP` m, two beams a station (sparse, so the view
//! is the tower, not vines): on the spirals two of three lanes 7 m apart,
//! on the branches the two outer lanes 8 m apart (a choice either side of
//! the bridge). One beam in five leans in from outside (`LEAN`). Reset pads
//! a few metres under the centreline every 50 m, each throwing an ape up
//! and on to the next station (`boost`). The zipper's beams are the
//! climb's (the start row's too) mirrored, hung longer (`ZIP_LONGER`).
//!
//! Placement draws from `Dice(seed ^ 0x70E5)`.

use super::*;
use crate::course::{centreline, Course, Mushroom, Platform, Vine, STEP};
use crate::fx::{self, deg, mul, Dice, ONE};
use crate::geom::{facing, right_of, yaw_of};
use crate::rules::{BODY_DROP, GRAVITY, MUSHROOM_RADIUS};
use crate::sector::DOWN;

const SEED: u64 = 0x70E5_0000_0001;

/// The start deck's top, and so the centreline's height at the start and
/// at the finish (the zipper ends where the climb began, mirrored).
pub const START_TOP: i32 = int(16);
/// How far the tear's middle stands over the line's top (TA8).
const TEAR_RISE: i32 = int(13);

/// A leg of the climb: metres along the path (3D; even), the turn over it
/// in degrees (positive to the left), and the metres it rises.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Climb(pub i32, pub i32, pub i32);

/// The spirals' radius round the keep's middle (the keep's wall is
/// `WALL_GAP` inside). 38 m kept turning the line under a flying ape (TC4).
const RADIUS: i32 = 55;
const WALL_GAP: i32 = 14;
/// The climb: spirals at `RADIUS`, and three branches, each a straight
/// bridge off the spiral's tangent, 90° round a sub-tower (radius about
/// 20 m) and a straight bridge back onto the tangent further round
/// (`RADIUS - r` each way, so it lands on the circle again).
pub const CLIMB: &[Climb] = &[
    Climb(24, 0, 0),   // 0 run-in
    Climb(68, 70, 12), // 1 spiral A
    Climb(36, 0, 6),   // 2 branch 1: bridge out
    Climb(32, 90, 0),  // 3 round the sub-tower
    Climb(36, 0, 6),   // 4 bridge back
    Climb(68, 70, 12), // 5 spiral B
    Climb(12, 0, 0),   // 6 branch 2: bridge to the room's door
    Climb(30, 0, 18),  // 7 up across the trampoline room
    Climb(32, 90, 0),  // 8 out of its roof, round
    Climb(36, 0, 4),   // 9 bridge back
    Climb(68, 70, 12), // 10 spiral C, through the clouds
    Climb(36, 0, 6),   // 11 branch 3: bridge out
    Climb(32, 90, 0),  // 12 round the sub-tower
    Climb(36, 0, 6),   // 13 bridge back
    Climb(58, 60, 10), // 14 spiral D
    Climb(40, -90, 0), // 15 off the keep, out to the summit
    Climb(30, 0, 0),   // 16 the summit run to the tear
];
/// The legs off the keep out to the tear: their beams further than
/// `SUMMIT_REACH` from the keep's wall are cast by dragons, not by mages
/// on the crown (TA5: up to 54 m of beam).
const SUMMIT_RUN: usize = 15;
const SUMMIT_REACH: i32 = int(24);
/// The keep's crown stands this far over its highest balcony, so the
/// summit's balconies are on its wall, not in the air (TA5).
const CROWN_OVER: i32 = int(8);
/// The spiral legs.
const SPIRALS: [usize; 4] = [1, 5, 10, 14];
/// Each branch's legs: out (the first), round, back.
const BRANCHES: [[usize; 3]; 3] = [[2, 3, 4], [6, 8, 9], [11, 12, 13]];
/// The leg across the trampoline room (branch 2's second leg out).
const ROOM_LEG: usize = 7;

/// A sub-tower's radius is its turn's less this: the inner lane's beams
/// hang 5 m in from the line, so 8 m clear of its wall, and a swing
/// on a bridge toward it clears it too (TW8; it was 6).
const SUB_CLEAR: i32 = int(13);
/// A sub-tower's top stands this far over its turn (was 12 m: the
/// balconies, 22 m over the line, were in the air above it, TW8).
const SUB_OVER: i32 = int(25);
/// The room's roof hole's radius, the orb mage's beam's length and how far
/// under the roof its anchor is (TW8).
const ROOF_HOLE: i32 = int(8);
const ORB_BEAM: i32 = int(6);
const ROOF_UNDER: i32 = cm(150);
/// The beam over the roof hole hangs to this far over the roof.
const EXIT_OVER: i32 = int(4);

/// A balcony's depth out from its wall.
const BALCONY_DEPTH: i32 = int(3);

/// A station every this many metres.
const GAP: i32 = 22;
/// Vines hang from this far over the centreline at their place, to this
/// far over it.
const ANCHOR_OVER: i32 = int(26);
const BOTTOM_OVER: i32 = int(2);
/// How far a leaning beam leans (sideways per metre down).
const LEAN: i32 = cm(35);
/// The zipper's beams lean along the way this much more (alternately):
/// none for now, as the AI wandered on them (TC rows).
const TOPSY: i32 = cm(0);
/// The zipper's beams' bottoms hang this much lower than the climb's.
const ZIP_DROP: i32 = int(4);
/// The zipper's beams are this much longer than the climb's, their anchors
/// high over the line: a falling ape flies about 20 m over it, so at the
/// climb's length it caught them by the anchor and whirled (TC5).
const ZIP_LONGER: i32 = int(22);
/// Reset pads stand this far under the centreline: under the swings, and
/// in a jump's reach of the vines.
const PAD_UNDER: i32 = int(5);
/// Reset pads stand this far short of their station.
const PAD_BEFORE: i32 = int(10);

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

/// Where leg `k` of the climb begins, in metres along (fixed point).
fn leg_at(k: usize) -> i32 {
    CLIMB[..k].iter().map(|l| l.0 * ONE / STEP * STEP).sum()
}

/// The climb's centreline from `legs`, starting at `start` facing yaw 0:
/// samples `STEP` apart in 3D and their 3D unit headings, as `centreline`
/// does in plan (each step's yaw is the leg's turn at the step's middle).
fn climb_path(legs: &[Climb], start: V3) -> (Vec<V3>, Vec<V3>) {
    let mut path = Vec::new();
    let mut heading = Vec::new();
    let mut p = start;
    let mut yaw = 0;
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

/// The whole centreline: the climb, then the climb mirrored across
/// `x = mirror_x` (its last point's x) and run backwards.
fn full_path(legs: &[Climb], start: V3) -> (Vec<V3>, Vec<V3>, i32) {
    let (cp, ch) = climb_path(legs, start);
    let n = cp.len();
    let mx = cp[n - 1].x;
    let mut path = cp.clone();
    let mut heading = ch[..n - 1].to_vec();
    // the joint: the way on is the climb's last step mirrored
    heading.push(TowerLayout::mirror_dir(ch[n - 2]));
    for k in 1..n {
        let q = cp[n - 1 - k];
        path.push(v3(2 * mx - q.x, q.y, q.z));
        let h = if k + 1 < n { ch[n - 2 - k] } else { ch[0] };
        heading.push(TowerLayout::mirror_dir(h));
    }
    (path, heading, mx)
}

fn flat(h: V3) -> V3 {
    v3(h.x, 0, h.z).norm()
}

/// The middle of the circle leg `k` turns round (in plan, at the leg's
/// start height): left of its start by its plan radius.
fn turn_centre(c: &Course, k: usize) -> (V3, i32) {
    let leg = CLIMB[k];
    let at = leg_at(k);
    let run = fx::sqrt(mul(int(leg.0), int(leg.0)) - mul(int(leg.2), int(leg.2)));
    // (plan radius: the run over the turn in radians)
    let r = (run as i64 * 360 * 10_000 / (62_832 * leg.1.abs() as i64)) as i32;
    let left = right_of(flat(c.heading_at(at))).neg();
    (c.point(at).add(left.scale(r)), r)
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

/// `DOWN` leaning `by` along `side` (per metre down), unit.
fn lean(side: V3, by: i32) -> V3 {
    DOWN.add(side.scale(by)).norm()
}

/// Mape's Tower from another seed (the beams shift a little).
pub fn tower_seeded(seed: u64) -> Course {
    let mut c = centreline("tower", &[]);
    let (path, heading, mirror_x) = full_path(CLIMB, v3(0, START_TOP, 0));
    c.right = heading.iter().map(|&h| right_of(h)).collect();
    c.spacing = path
        .windows(2)
        .map(|w| w[1].dist(w[0]) + 1)
        .max()
        .unwrap_or(STEP);
    c.length = (path.len() as i32 - 1) * STEP;
    c.path = path;
    c.heading = heading;
    c.half_width = int(20);
    let half = split(c.length);
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
    c.start_yaw = 0;
    // the start row, as the Jungle's: just past the deck's edge, to a
    // metre over it
    for k in -2..=2 {
        let at = int(15);
        let base = c.point(at).add(right.scale(int(3) * k));
        c.vines.push(Vine {
            anchor: v3(base.x, START_TOP + int(30), base.z),
            length: int(29),
            slide: None,
            at,
        });
        hangers.push(Hanger::Start);
        lifts.push(false);
        hangs.push(DOWN);
    }

    // the keep: the spirals' middle, left of the way in where they begin
    let in_at = leg_at(1);
    let left0 = right_of(flat(c.heading_at(in_at))).neg();
    let mid = c.point(in_at).add(left0.scale(int(RADIUS)));
    let keep_centre = v3(mid.x, 0, mid.z);
    let keep_radius = int(RADIUS - WALL_GAP);
    let top = c.path[(half / STEP) as usize].y;

    // the sub-towers: round the branches' turns (the room's is the room)
    let room_from = c.point(leg_at(ROOM_LEG));
    let room_to = c.point(leg_at(ROOM_LEG + 1));
    // the room's middle is under where the line crosses its roof's plane
    // (3 m under the line's top across it), so the hole there is the way
    // out, and the doorway is where the line meets its wall (TW8)
    let r0 = leg_at(ROOM_LEG);
    let room_len = leg_at(ROOM_LEG + 1) - r0;
    let roof = room_to.y - int(3);
    let room_leg = CLIMB[ROOM_LEG];
    let cross_at = r0 + room_len * (room_leg.2 - 3) / room_leg.2;
    let cross = c.point(cross_at);
    let room_mid = v3(cross.x, room_from.y - int(5), cross.z);
    let room_radius = int(14);
    // (metres along the leg per metre in plan)
    let room_run = fx::sqrt(int(room_leg.0 * room_leg.0 - room_leg.2 * room_leg.2));
    let door_at = cross_at - mul(room_radius, fx::div(int(room_leg.0), room_run));
    let mut subs = Vec::new();
    for (b, legs) in BRANCHES.iter().enumerate() {
        let sub = if b == 1 {
            SubTower {
                centre: v3(room_mid.x, 0, room_mid.z),
                radius: room_radius,
                top: roof,
            }
        } else {
            let (o, r) = turn_centre(&c, legs[1]);
            // (its foot stands over any lower coil passing under it, out
            // of reach of that coil's beams: on a corbel, not the ground,
            // TW8; the third sub-tower stood in spiral A's way)
            let own = (leg_at(legs[0]) - int(10), leg_at(legs[2] + 1) + int(10));
            let foot = c.path[..=(half / STEP) as usize]
                .iter()
                .enumerate()
                .filter(|&(k, p)| {
                    let at = k as i32 * STEP;
                    (at < own.0 || at > own.1)
                        && p.y < o.y
                        && p.flat().sub(o.flat()).len() < r - SUB_CLEAR + int(16)
                })
                .map(|(_, p)| p.y + ANCHOR_OVER + int(4))
                .max()
                .unwrap_or(0);
            SubTower {
                centre: v3(o.x, foot, o.z),
                // (well inside the inner lane's beams, `r - 8`, TW8)
                radius: r - SUB_CLEAR,
                // (tall enough for its balconies, 22 m over the line)
                top: o.y + SUB_OVER,
            }
        };
        subs.push(sub);
    }

    // the balconies and their mages: one on the nearest tower's wall
    // facing each station, a few metres under its beams
    let mut balconies: Vec<Balcony> = Vec::new();
    let mut balcony_for = |p: V3, y: i32| -> usize {
        // (a sub-tower only where it stands at that height)
        let towers = std::iter::once((keep_centre, keep_radius)).chain(
            subs.iter()
                .filter(|s| y >= s.centre.y && y <= s.top + int(3))
                .map(|s| (s.centre, s.radius)),
        );
        let (o, r) = towers
            .min_by_key(|&(o, r)| p.flat().sub(o.flat()).len() - r)
            .expect("the keep");
        let out = p.flat().sub(o.flat()).norm();
        // (its floor's outer edge: it stands out from the wall, its mage
        // outside the stone, TW8)
        let at = o.add(out.scale(r + BALCONY_DEPTH));
        balconies.push(Balcony {
            at: v3(at.x, y, at.z),
            out,
            half_width: int(3),
            depth: BALCONY_DEPTH,
            mage: true,
            deck: None,
        });
        balconies.len() - 1
    };
    let leg_of = |s: i32| -> usize {
        (0..CLIMB.len())
            .rev()
            .find(|&k| leg_at(k) <= s)
            .unwrap_or(0)
    };
    let off_keep = |p: V3| p.flat().sub(keep_centre.flat()).len() - keep_radius;
    let clouds = CloudBand {
        bottom: START_TOP + int(56),
        top: START_TOP + int(80),
    };

    // the climb's stations
    let room = (leg_at(ROOM_LEG) - int(8), leg_at(ROOM_LEG + 1) + int(4));
    let mut s = 36;
    let mut station = 0;
    let mut stations = Vec::new();
    while int(s) < half - int(12) {
        let at0 = int(s);
        s += GAP;
        if at0 > room.0 && at0 < room.1 {
            continue;
        }
        station += 1;
        let leg = leg_of(at0);
        let branch = BRANCHES.iter().any(|b| b.contains(&leg)) || leg == ROOM_LEG;
        let h = c.heading_at(at0);
        let r = right_of(h);
        let lanes: [i32; 2] = if branch {
            // (the inner lane 5 m in, not 8: its swing on toward the
            // sub-tower or the keep ran into the wall, TW8)
            [-5, 8]
        } else if leg >= SUMMIT_RUN {
            // (off the keep: no lane on its side, whose swing back ran
            // into its wall, TW8)
            [0, 7]
        } else {
            match dice.below(3) {
                0 => [0, 7],
                1 => [-7, 7],
                _ => [-7, 0],
            }
        };
        for (n, &lane) in lanes.iter().enumerate() {
            let along = at0 + int(dice.range(-2, 2));
            let base = c.point(along).add(r.scale(int(lane)));
            let bottom = v3(base.x, base.y + BOTTOM_OVER, base.z);
            // one beam in five leans in from outside (the spell's way)
            // (only an outer lane's: an inner lane's leaned its anchor
            // into the keep's or a sub-tower's stone, TW8)
            let leaning = lane > 0 && (station * 2 + n) % 5 == 2;
            let hang = if leaning {
                lean(r.scale(-lane.signum()), LEAN)
            } else {
                DOWN
            };
            let v = beam(bottom, hang, ANCHOR_OVER - BOTTOM_OVER, along);
            let hanger = if v.anchor.y >= clouds.bottom && v.anchor.y <= clouds.top + int(10) {
                Hanger::Cloud { band: 0 }
            } else if (!branch && lane > 0 && station % 3 == 0)
                || (leg >= SUMMIT_RUN && off_keep(v.anchor) > SUMMIT_REACH)
            {
                // a dragon outside the spiral; on the summit run, out past
                // a beam's reach from the keep's crown (TA5), dragons too
                // a dragon outside the spiral
                Hanger::Dragon {
                    at: v.anchor.add(r.scale(int(9))).add(v3(0, int(4), 0)),
                }
            } else {
                Hanger::Mage {
                    balcony: balcony_for(bottom, v.anchor.y - int(4)),
                }
            };
            if n == 0 || lane == 0 {
                // (the pads aim at the station's middlemost beam)
                if n == 1 {
                    stations.pop();
                }
                stations.push((along, bottom));
            }
            c.vines.push(v);
            hangers.push(hanger);
            // (only the spirals' beams lift: on the level branches and the
            // summit run a lift only carries an ape over the next beams)
            lifts.push(SPIRALS.contains(&leg));
            hangs.push(hang);
        }
    }

    // the trampoline room: a trampoline on the bridge's end before its
    // doorway, one inside, one under the roof hole; the orb mage's beam
    // hung from the roof inside, and a beam down into the hole (TW8)
    let mut trampolines = Vec::new();
    let right_in = right_of(flat(c.heading_at(r0)));
    for (k, under, aside) in [
        (int(2), int(5), 0),
        (int(12), int(5), 0),
        (cross_at - r0 - int(5), int(10), int(3)),
    ] {
        let p = c.point(r0 + k).add(right_in.scale(aside));
        c.mushrooms.push(Mushroom {
            top: v3(p.x, p.y - under, p.z),
            radius: MUSHROOM_RADIUS,
        });
        trampolines.push(c.mushrooms.len() - 1);
    }
    let hole_tramp = c.mushrooms[trampolines[2]].top;
    let orb = (v3(room_mid.x, room_from.y + int(6), room_mid.z), int(3));
    {
        let k = int(15);
        let p = c.point(r0 + k);
        let anchor = v3(p.x, roof - ROOF_UNDER, p.z);
        c.vines.push(Vine {
            anchor,
            length: ORB_BEAM,
            slide: None,
            at: r0 + k,
        });
        hangers.push(Hanger::Orb);
        lifts.push(true);
        hangs.push(DOWN);
    }
    let exit_at = cross_at + int(3);
    let exit = c.point(exit_at);
    c.vines.push(beam(
        // (its bottom over the roof: grabbed from under it, in the hole,
        // the swing pressed the ape into the roof's edge, TW8)
        v3(exit.x, roof + EXIT_OVER, exit.z),
        DOWN,
        int(14),
        exit_at,
    ));
    hangers.push(Hanger::Orb);
    lifts.push(true);
    hangs.push(DOWN);
    // (the start row too: the zipper's last beams, a swing onto the zone)
    let climb_vines = 0..c.vines.len();

    // a trampoline at each spiral's start, under the line
    for &k in &SPIRALS {
        let p = c.point(leg_at(k) + int(10));
        c.mushrooms.push(Mushroom {
            top: v3(p.x, p.y - int(5), p.z),
            radius: MUSHROOM_RADIUS,
        });
    }

    // the keep rises to its highest balcony and on (the summit's stand
    // above the line's top)
    let keep_height = balconies
        .iter()
        .map(|b| b.at.y + CROWN_OVER)
        .fold(top - int(6), i32::max);
    // the tear: giant (50 m tall, 56 m wide), its middle over the line's
    // top where the apes fly through it (they cross up to 24 m above the
    // line and 20 m aside on the summit run, TA8)
    let tear_half = (int(25), int(28));
    let tear = c.point(half).add(v3(0, TEAR_RISE, 0));
    let tear_left = right_of(flat(c.heading_at(half - int(2)))).neg();
    // the mad wizard (drawn 2.5 times life) beside its left edge, his
    // hands about level with its middle
    let wizard = tear
        .add(tear_left.scale(tear_half.1 + int(5)))
        .sub(v3(0, int(8), 0));
    let mut layout = TowerLayout {
        keep: Keep {
            centre: keep_centre,
            radius: keep_radius,
            height: keep_height,
            levels: (1..(keep_height / int(12))).map(|k| int(12) * k).collect(),
        },
        branches: Vec::new(),
        balconies,
        room: Room {
            centre: room_mid,
            radius: room_radius,
            height: roof - room_mid.y,
            door: c.point(door_at),
            roof_exit: (v3(cross.x, roof, cross.z), ROOF_HOLE),
            trampolines,
            orb,
        },
        clouds: vec![clouds],
        summit: Summit {
            deck: v3(keep_centre.x, keep_height, keep_centre.z),
            radius: keep_radius + int(4),
            wizard,
            // facing the apes coming up, arms flung along the tear
            wizard_yaw: yaw_of(v3(-ONE, 0, 0)),
            tear,
            tear_half,
            through: v3(ONE, 0, 0),
            banana: v3(tear.x + ONE / 2, tear.y, tear.z),
            banana_len: int(30),
        },
        zipper: Zipper {
            mirror_x,
            axis: V3::ZERO,
            top,
            bottom: 0,
        },
        banana_zone: BananaZone {
            centre: V3::ZERO,
            radius: int(40),
            platform: 0,
        },
        pads: Vec::new(),
        hangers: Vec::new(),
        lifts: Vec::new(),
        hangs: Vec::new(),
        lobby: (c.starts[0], 0),
        stone: Vec::new(),
    };
    layout.zipper.axis = layout.mirror(keep_centre);

    // the branches' bridges: decks a few metres under the line
    for (b, legs) in BRANCHES.iter().enumerate() {
        for &leg in &[legs[0], legs[2]] {
            let (from, to) = (c.point(leg_at(leg)), c.point(leg_at(leg + 1)));
            let deck = {
                let mid = v3((from.x + to.x) / 2, from.y - PAD_UNDER, (from.z + to.z) / 2);
                c.platforms.push(Platform {
                    centre: mid,
                    fwd: flat(to.sub(from)),
                    half_len: from.dist(to) / 2,
                    half_width: int(3),
                    top: mid.y,
                });
                Some(c.platforms.len() - 1)
            };
            layout.branches.push(Branch {
                from: v3(from.x, from.y - PAD_UNDER, from.z),
                to: v3(to.x, to.y - PAD_UNDER, to.z),
                half_width: if deck.is_some() { int(3) } else { 0 },
                deck,
                sub: subs[b],
            });
        }
    }

    // the zipper: every climb beam mirrored, along the descent, leaning
    // the climb's leaning ones mirrored (`TOPSY` more, alternately)
    for (n, i) in climb_vines.enumerate() {
        let v = c.vines[i];
        let h = hangs[i];
        let bottom = layout
            .mirror(v.anchor.add(h.scale(v.length)))
            .sub(v3(0, ZIP_DROP, 0));
        let at = c.length - v.at;
        let mh = v3(-h.x, h.y, h.z);
        let way = flat(c.heading_at(at));
        let tilt = if n % 2 == 0 { TOPSY } else { -TOPSY };
        let hang = mh.add(way.scale(tilt)).norm();
        let w = beam(bottom, hang, v.length + ZIP_LONGER, at);
        hangers.push(Hanger::Monster { at: w.anchor });
        c.vines.push(w);
        lifts.push(false);
        hangs.push(hang);
    }
    // (no trampolines in the zipper: going down, a bounce only throws an
    // ape back up the way it came)

    // item crates: a row across the course every 240 m, on every beam of
    // the station nearest each place, never on the start row (the
    // fallback put one on the first start vine alone: one starter got an
    // item, TI1)
    let mut row = 150;
    while int(row) < c.length - int(60) {
        let si = int(row);
        let near = (0..c.vines.len())
            .filter(|&i| hangers[i] != Hanger::Start && hangers[i] != Hanger::Orb)
            .min_by_key(|&i| (c.vines[i].at - si).abs());
        if let Some(n) = near {
            let at = c.vines[n].at;
            for i in 0..c.vines.len() {
                if hangers[i] != Hanger::Start && (c.vines[i].at - at).abs() <= int(4) {
                    c.crates.push(c.vines[i].anchor);
                }
            }
        }
        row += 240;
    }

    // the reset pads: the start deck, the room's floor, then one every
    // 50 m each way, each throwing an ape up to the next station's beams
    layout.pads.push(Pad {
        platform: 0,
        at: 0,
        boost: V3::ZERO,
    });
    let pad_at = |c: &mut Course, at: i32, to: V3, half_len: i32, half_width: i32| -> Pad {
        let p = c.point(at);
        let fwd = flat(c.heading_at(at));
        let top = p.y - PAD_UNDER;
        c.platforms.push(Platform {
            centre: v3(p.x, top, p.z),
            fwd,
            half_len,
            half_width,
            top,
        });
        Pad {
            platform: c.platforms.len() - 1,
            at,
            boost: boost(v3(p.x, top + BODY_DROP, p.z), to),
        }
    };
    // a pad `PAD_BEFORE` short of every second station, each half (the
    // zipper's stations are the climb's mirrored)
    let mirror = |(at, b): (i32, V3)| (c.length - at, v3(2 * mirror_x - b.x, b.y, b.z));
    let mut every: Vec<(i32, V3)> = stations
        .iter()
        .step_by(2)
        .copied()
        .chain(stations.iter().skip(1).step_by(2).copied().map(mirror))
        .collect();
    every.sort_by_key(|s| s.0);
    for (st, bottom) in every {
        let at = st - PAD_BEFORE;
        if (at - half).abs() < int(16) || at < int(30) || (at > room.0 && at < room.1) {
            continue;
        }
        let pad = pad_at(&mut c, at, bottom, int(4), int(5));
        layout.pads.push(pad);
    }
    // the room's floor (a wide pad): a reset in the room throws an ape
    // onto the trampoline under the roof hole
    c.platforms.push(Platform {
        centre: room_mid,
        fwd: flat(c.heading_at(r0)),
        half_len: room_radius - int(4),
        half_width: room_radius - int(4),
        top: room_mid.y,
    });
    layout.pads.push(Pad {
        platform: c.platforms.len() - 1,
        at: r0 + int(1),
        boost: boost(room_mid.add(v3(0, BODY_DROP, 0)), hole_tramp),
    });
    layout.pads.sort_by_key(|p| p.at);

    // the banana zone: a wide floor from the last vines to the end and on
    // past it (where the mirrored start deck would be)
    let end = c.point(c.length);
    let out = flat(c.heading_at(c.length));
    let zone = end.add(out.scale(int(10)));
    c.platforms.push(Platform {
        centre: v3(zone.x, end.y - int(1), zone.z),
        fwd: out,
        half_len: int(50),
        half_width: int(30),
        top: end.y - int(1),
    });
    layout.banana_zone = BananaZone {
        centre: v3(zone.x, end.y - int(1), zone.z),
        radius: int(40),
        platform: c.platforms.len() - 1,
    };
    layout.lobby = (c.starts[1], yaw_of(fwd));
    layout.hangers = hangers;
    layout.lifts = lifts;
    layout.hangs = hangs;
    layout.stone = super::stone::stonework(&layout);
    c.tower = Some(Box::new(layout));
    c
}
