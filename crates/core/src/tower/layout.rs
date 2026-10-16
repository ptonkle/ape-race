//! Mape's Tower's course: the 3D centreline (a climb of legs, then its
//! mirror) and the STAND-IN layout of the skeleton (2026-10-15).
//!
//! The stand-in: a level 40 m run from the start deck to the keep, a helix
//! of 450° round it at `RADIUS` rising `RISE` m, a level 80 m summit run
//! heading +x to the tear; then the same mirrored and backwards, down to
//! the banana zone. Two vines a station every 24 m (climb vines on the
//! way up), a reset pad every 50 m, one trampoline each half.
//!
//! New placement draws from `Dice(seed ^ 0x70E5)`.

use super::*;
use crate::course::{centreline, Course, Mushroom, Platform, Vine, STEP};
use crate::fx::{self, deg, mul, Dice, ONE};
use crate::geom::{facing, right_of, yaw_of};
use crate::rules::{BODY_DROP, MUSHROOM_RADIUS};

const SEED: u64 = 0x70E5_0000_0001;

/// The start deck's top, and so the centreline's height at the start and
/// at the finish (the zipper ends where the climb began, mirrored).
pub const START_TOP: i32 = int(16);

/// A leg of the climb: metres along the path (3D), the turn over it in
/// degrees (positive to the left), and the metres it rises.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Climb(pub i32, pub i32, pub i32);

/// The helix's radius round the keep's middle (the keep's wall is inside).
const RADIUS: i32 = 45;
/// The stand-in's climb: run in, 450° round the keep rising 70 m, the
/// summit run heading +x (yaw 90°) into the mirror plane.
pub const CLIMB: &[Climb] = &[
    Climb(40, 0, 0),
    // 450° at radius 45 m: 353 m along
    Climb(353, 450, 70),
    Climb(80, 0, 0),
];

/// Vines hang from this far over the centreline at their place, to this
/// far over it.
const ANCHOR_OVER: i32 = int(26);
const BOTTOM_OVER: i32 = int(2);
/// Reset pads stand this far under the centreline: under the swings, and
/// in a jump's reach of the vines.
const PAD_UNDER: i32 = int(5);

/// Mape's Tower, laid out from its fixed seed.
pub fn tower() -> Course {
    tower_seeded(SEED)
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

/// Mape's Tower from another seed (the stand-in's vines shift a little).
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

    let flat = |h: V3| v3(h.x, 0, h.z).norm();
    let mut hangers = Vec::new();
    let mut lifts = Vec::new();

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
    }

    // the climb's stations: two lanes of three, climb vines
    let first_climb_vine = c.vines.len();
    let mut s = 38;
    while int(s) < half - int(12) {
        let h = c.heading_at(int(s));
        let r = right_of(h);
        let skip = dice.below(3) as i32;
        for lane in 0..3 {
            if lane == skip {
                continue;
            }
            let along = int(s + dice.range(-2, 2));
            let base = c.point(along).add(r.scale(int((lane - 1) * 7)));
            c.vines.push(Vine {
                anchor: v3(base.x, base.y + ANCHOR_OVER, base.z),
                length: ANCHOR_OVER - BOTTOM_OVER,
                slide: None,
                at: along,
            });
            hangers.push(Hanger::Mage { balcony: 0 });
            lifts.push(true);
        }
        s += 24;
    }
    let climb_vines = first_climb_vine..c.vines.len();

    // a trampoline under the climb, short of a station
    let tramp = c.point(int(150));
    c.mushrooms.push(Mushroom {
        top: v3(tramp.x, tramp.y - int(4), tramp.z),
        radius: MUSHROOM_RADIUS,
    });

    // the zipper: every climb vine mirrored, along the descent
    let mut layout = TowerLayout {
        keep: Keep {
            centre: V3::ZERO,
            radius: int(RADIUS - 15),
            height: 0,
            levels: Vec::new(),
        },
        branches: Vec::new(),
        balconies: Vec::new(),
        room: Room {
            centre: V3::ZERO,
            radius: int(14),
            height: int(20),
            door: V3::ZERO,
            roof_exit: (V3::ZERO, int(5)),
            trampolines: Vec::new(),
            orb: (V3::ZERO, int(4)),
        },
        clouds: Vec::new(),
        summit: Summit {
            deck: V3::ZERO,
            radius: int(12),
            wizard: V3::ZERO,
            wizard_yaw: 0,
            tear: V3::ZERO,
            tear_half: (int(14), int(6)),
            through: v3(ONE, 0, 0),
            banana: V3::ZERO,
            banana_len: int(60),
        },
        zipper: Zipper {
            mirror_x,
            axis: V3::ZERO,
            top: 0,
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
        lobby: (c.starts[0], 0),
    };
    for i in climb_vines {
        let v = c.vines[i];
        let anchor = layout.mirror(v.anchor);
        c.vines.push(Vine {
            anchor,
            at: c.length - v.at,
            ..v
        });
        hangers.push(Hanger::Monster { at: anchor });
        lifts.push(false);
    }
    // and the trampoline
    let m = c.mushrooms[0];
    c.mushrooms.push(Mushroom {
        top: layout.mirror(m.top),
        ..m
    });

    // the keep: the helix's middle, left of the way in where it begins
    let in_at = int(CLIMB[0].0);
    let into = flat(c.heading_at(in_at));
    let left = right_of(into).neg();
    let mid = c.point(in_at).add(left.scale(int(RADIUS)));
    let top = c.path[c.length as usize / 2 / STEP as usize].y;
    layout.keep = Keep {
        centre: v3(mid.x, 0, mid.z),
        radius: int(RADIUS - 15),
        height: top - int(4),
        levels: (1..8).map(|k| int(12) * k).collect(),
    };
    layout.clouds = vec![CloudBand {
        bottom: START_TOP + int(40),
        top: START_TOP + int(55),
    }];
    let tear = c.point(half);
    layout.summit = Summit {
        deck: v3(mid.x, top - int(4), mid.z),
        radius: int(RADIUS - 15),
        wizard: v3(tear.x - int(10), tear.y - int(4), tear.z + int(8)),
        wizard_yaw: deg(90),
        tear,
        through: v3(ONE, 0, 0),
        banana: v3(tear.x + int(150), tear.y + int(40), tear.z),
        ..layout.summit
    };
    layout.zipper.axis = layout.mirror(layout.keep.centre);
    layout.zipper.top = top;
    layout.zipper.bottom = 0;

    // the reset pads: the start deck, then one every 50 m each way
    layout.pads.push(Pad {
        platform: 0,
        at: 0,
        boost: V3::ZERO,
    });
    let mut s = 50;
    while int(s) < c.length - int(30) {
        if (int(s) - half).abs() < int(20) {
            s += 50;
            continue;
        }
        let at = int(s);
        let p = c.point(at);
        let fwd = flat(c.heading_at(at));
        c.platforms.push(Platform {
            centre: v3(p.x, p.y - PAD_UNDER, p.z),
            fwd,
            half_len: int(4),
            half_width: int(5),
            top: p.y - PAD_UNDER,
        });
        // up and on: harder up on the climb
        let up = if at < half { int(12) } else { int(6) };
        layout.pads.push(Pad {
            platform: c.platforms.len() - 1,
            at,
            boost: fwd.scale(int(9)).add(v3(0, up, 0)),
        });
        s += 50;
    }

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
    c.tower = Some(Box::new(layout));
    c
}
