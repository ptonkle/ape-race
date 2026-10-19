//! The Spindle's course: the straight tube, the start, the vines round the
//! axle (in the flat frame, `spindle.rs`).

use super::{Block, BlockKind, SpindleLayout, Spinner, Warp, AXLE_UNDER, CIRCUMFERENCE};
use crate::course::{centreline, Course, Leg, Platform, Vine, START_HEIGHT, START_LEN};
use crate::fx::{cm, int, Dice};
use crate::geom::{right_of, v3};
use crate::rules::BODY_DROP;

/// The race's fixed seed.
pub const SEED: u64 = 0x5_9141_D1E5;
/// Mixed into the seed for this map's own placement (other maps' dice are
/// untouched).
const DICE: u64 = 0x5D1_7D1E;

/// The tube from the start to the open end, in metres.
pub const LENGTH: i32 = 2000;
/// The closed back end, behind the start (16 m clear behind `path[0]`).
pub const BACK: i32 = int(-20);
/// A row of vines round the axle every this many metres along ...
pub const ROW: i32 = int(24);
/// ... the first this far along ...
pub const FIRST_ROW: i32 = int(30);
/// ... and this many vines round the tube in a full row (half a step round
/// on every other row, so the vines stand in a diamond lattice).
pub const AROUND: i32 = 12;
/// Rows near the route (within this of x = 0) are full; further round, every
/// other row (the far wall's vines are for the brave).
pub const NEAR: i32 = int(60);
/// A vine hangs from just under the axle ...
pub const ANCHOR: i32 = AXLE_UNDER - cm(50);
/// ... this long, give or take `LONG_JITTER` ...
pub const LONG: i32 = int(22);
const LONG_JITTER: i32 = int(1);
/// ... and on every other row the vines nearest the route reach low, for an
/// ape on the floor.
pub const LOW: i32 = int(30);
/// No vine nearer the seam (x = ±C/2) than this: grabbing reaches across
/// nothing there.
pub const SEAM_CLEAR: i32 = int(8);

/// The Spindle's course, from its fixed seed.
pub fn spindle() -> Course {
    spindle_seeded(SEED)
}

/// The Spindle's course from `seed` (the tube is fixed; the vines' jitter
/// follows the seed).
pub fn spindle_seeded(seed: u64) -> Course {
    let mut c = centreline("spindle", &[Leg(LENGTH, 0)]);
    c.terrain = crate::ski::Terrain::Flat(0);
    // (no side walls: round the tube is open, `Course::contain`)
    c.half_width = CIRCUMFERENCE / 2;
    let mut dice = Dice(seed ^ DICE);

    // ---- the start: a block platform at the closed end ----
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

    // ---- the vines: rows round the axle ----
    let (blocks, spinners) = obstacles();
    // (no low vine next to a wall: its swing would run into it)
    let walled = |z: i32| {
        blocks.iter().any(|b| {
            b.hi.y > PAD_HIGH && (b.lo.z + b.hi.z) / 2 - z < ROW && z - (b.lo.z + b.hi.z) / 2 < ROW
        })
    };
    let warp = Warp::SPINDLE;
    let step = CIRCUMFERENCE / AROUND;
    let mut z = FIRST_ROW;
    let mut row = 0;
    while z < c.length - int(4) {
        let shift = if row % 2 == 0 { step / 2 } else { 0 };
        for k in -AROUND / 2..AROUND / 2 {
            let x = warp.wrap_x(k * step + shift);
            if x.abs() > CIRCUMFERENCE / 2 - SEAM_CLEAR {
                continue;
            }
            if x.abs() > NEAR && row % 2 == 1 {
                continue;
            }
            let jx = dice.range(-int(2), int(2));
            let low = row % 2 == 0 && x.abs() < 2 * step && !walled(z);
            let length = if low {
                LOW
            } else {
                LONG + dice.range(-LONG_JITTER, LONG_JITTER)
            };
            c.vines.push(Vine {
                anchor: v3(x + jx, ANCHOR, z),
                length,
                slide: None,
                at: z,
            });
        }
        z += ROW;
        row += 1;
    }

    c.spindle = Some(Box::new(SpindleLayout {
        warp,
        back: BACK,
        mouth: c.length,
        blocks,
        spinners,
    }));
    c
}

/// The middle between vine row `k` and the next: where every obstacle
/// stands, 10 m clear of the vines either side along the tube.
pub fn between(k: i32) -> i32 {
    FIRST_ROW + ROW * k + ROW / 2
}

/// An obstacle's thickness along the tube.
pub const THICK: i32 = int(4);
/// A plate: a ring of blocks all round the axle, hanging down to this
/// over the floor (stay low under it). Nothing stands on the floor across
/// the lane: an ape on the floor there could not leave it (MAPS gotcha 17).
pub const PLATE: i32 = int(24);
/// A window wall: open from the floor to `WINDOW_HIGH` between `SPOKES`
/// spokes `SPOKE` wide, a band of blocks over that to the axle.
pub const WINDOW_HIGH: i32 = int(27);
pub const SPOKES: i32 = 6;
pub const SPOKE: i32 = int(6);
/// A pillar's size round the tube.
pub const PILLAR: i32 = int(6);
/// A slime pad's size and height.
pub const PAD: i32 = int(8);
pub const PAD_HIGH: i32 = cm(150);

/// The dye of each section's blocks (a colour its wall does not use,
/// `spindlescape::SECTIONS`).
const BLOCK_DYES: [u8; 8] = [11, 9, 4, 3, 1, 5, 4, 7];

/// The obstacles, section by section (each `spindlescape::SECTION`, 250 m,
/// a dropper level of its own): the warm-up's slime pads, the hurdles, the
/// pillars, the first spinning rings, the window walls, slime and glass,
/// the spinning gauntlet, and the last hurdles and window before the open
/// end.
pub fn obstacles() -> (Vec<Block>, Vec<Spinner>) {
    let mut b = Vec::new();
    let mut sp = Vec::new();
    let dye = |z: i32| BlockKind::Wool(BLOCK_DYES[((z / int(250)) as usize).min(7)]);
    let slime = |b: &mut Vec<Block>, x: i32, z: i32| {
        b.push(Block {
            lo: v3(x - PAD / 2, 0, z - PAD / 2),
            hi: v3(x + PAD / 2, PAD_HIGH, z + PAD / 2),
            kind: BlockKind::Slime,
        })
    };
    let row = |k: i32| FIRST_ROW + ROW * k;
    // 1, the warm-up: slime pads under the route's vines
    for k in [3, 6, 8] {
        for x in [-int(23), 0, int(23)] {
            slime(&mut b, x, row(k));
        }
    }
    // 2, the plates: keep low
    for k in [11, 14, 17] {
        let z = between(k);
        ring(&mut b, z, (PLATE, AXLE_UNDER), &[], dye(z));
    }
    // 3, the pillars, staggered across the lane
    for (n, k) in [21, 23, 25, 27].into_iter().enumerate() {
        let z = between(k);
        let xs: &[i32] = if n % 2 == 0 {
            &[-int(34), 0, int(34)]
        } else {
            &[-int(51), -int(17), int(17), int(51)]
        };
        for &x in xs {
            b.push(Block {
                lo: v3(x - PILLAR / 2, 0, z - THICK / 2),
                hi: v3(x + PILLAR / 2, AXLE_UNDER, z + THICK / 2),
                kind: dye(z),
            });
        }
    }
    // 4, the first spinning rings
    for (k, count, secs, phase) in [(32, 4, 12, 0), (35, 3, -10, int(30)), (38, 4, 9, int(15))] {
        sp.push(spinner(between(k), count, secs, phase, BlockKind::Obsidian));
    }
    // 5, the window walls: through a window, its place changing
    for (k, at) in [(41, int(23)), (44, 0), (47, int(12))] {
        let z = between(k);
        window_wall(&mut b, z, at, dye(z));
    }
    // 6, slime and glass
    for k in [50, 52, 54, 56, 58] {
        for x in [-int(30), -int(10), int(10), int(30)] {
            slime(&mut b, x, row(k));
        }
    }
    for k in [51, 54, 57] {
        let z = between(k);
        for x in [-int(40), -int(12), int(16), int(44)] {
            b.push(Block {
                lo: v3(x - PILLAR / 2, 0, z - THICK / 2),
                hi: v3(x + PILLAR / 2, AXLE_UNDER, z + THICK / 2),
                kind: BlockKind::Glass,
            });
        }
    }
    // 7, the spinning gauntlet
    for (k, count, secs, phase) in [
        (61, 6, 8, 0),
        (63, 5, -7, int(20)),
        (65, 6, 6, int(9)),
        (67, 4, -6, int(40)),
    ] {
        sp.push(spinner(between(k), count, secs, phase, BlockKind::Obsidian));
    }
    // 8, the last plates and a window wall, then the open end
    for k in [71, 74] {
        let z = between(k);
        ring(&mut b, z, (PLATE, AXLE_UNDER), &[], dye(z));
    }
    let z = between(77);
    window_wall(&mut b, z, int(23), dye(z));
    (b, sp)
}

/// A spinning ring at `z`: `count` columns going round once every `secs`
/// seconds (negative: the other way).
fn spinner(z: i32, count: u8, secs: i32, phase: i32, kind: BlockKind) -> Spinner {
    Spinner {
        z,
        depth: THICK,
        width: PILLAR,
        count,
        period: secs * crate::rules::TICKS_PER_SECOND,
        phase,
        kind,
    }
}

/// A wall all round the tube at `z`, from height `y.0` to `y.1`, but for
/// the `gaps` (middle, width) round it; split at the seam.
fn ring(b: &mut Vec<Block>, z: i32, y: (i32, i32), gaps: &[(i32, i32)], kind: BlockKind) {
    let c = CIRCUMFERENCE;
    // the spans between gaps, as [from, to) round from -C/2, unwrapped
    let mut cuts: Vec<(i32, i32)> = gaps
        .iter()
        .map(|&(m, w)| {
            let m = Warp::SPINDLE.wrap_x(m);
            (m - w / 2, m + w / 2)
        })
        .collect();
    cuts.sort();
    let mut spans = Vec::new();
    let mut from = -c / 2;
    for (a, z1) in cuts {
        if a > from {
            spans.push((from, a));
        }
        from = from.max(z1);
    }
    if from < c / 2 {
        spans.push((from, c / 2));
    }
    for (x0, x1) in spans {
        b.push(Block {
            lo: v3(x0, y.0, z - THICK / 2),
            hi: v3(x1, y.1, z + THICK / 2),
            kind,
        });
    }
}

/// A wall across the whole tube at `z`, open up to `WINDOW_HIGH` but for
/// `SPOKES` spokes evenly round it, the first at `at`: tall windows between
/// the spokes.
fn window_wall(b: &mut Vec<Block>, z: i32, at: i32, kind: BlockKind) {
    for k in 0..SPOKES {
        let x = Warp::SPINDLE.wrap_x(at + k * CIRCUMFERENCE / SPOKES);
        b.push(Block {
            lo: v3(x - SPOKE / 2, 0, z - THICK / 2),
            hi: v3(x + SPOKE / 2, WINDOW_HIGH, z + THICK / 2),
            kind,
        });
    }
    ring(b, z, (WINDOW_HIGH, AXLE_UNDER), &[], kind);
}
