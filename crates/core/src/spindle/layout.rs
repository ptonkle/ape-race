//! The Spindle's course: the straight tube, the start, the vines round the
//! axle (in the flat frame, `spindle.rs`).

use super::{SpindleLayout, Warp, AXLE_UNDER, CIRCUMFERENCE};
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
    let warp = Warp::SPINDLE;
    let step = CIRCUMFERENCE / AROUND;
    let mut z = FIRST_ROW;
    let mut row = 0;
    while z < c.length - int(20) {
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
            let low = row % 2 == 0 && x.abs() < step;
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
        blocks: Vec::new(),
        spinners: Vec::new(),
    }));
    c
}
