//! Rattlesnake Ravine's lobby spot (step 9, `RavineLayout.lobby`): chosen
//! on purpose, not the start platform (MAPS gotcha 11).
//!
//! **The spot:** the middle of a fallen log over the start straight, some
//! 45 m up the slot, looking on down the canyon. From there a low, level
//! camera sees the next rope bridge strung across the slot, the vines
//! glowing under it, the first arch spanning the
//! canyon beyond, the banded walls either side and the strip of night sky
//! with the Milky Way over it. (The start platform, 17 m up, looked along
//! the braid's dark boulders and the floor; a rope bridge's deck put its
//! hand ropes across the whole view: SECOND-ORDER RL1.)
//!
//! The log is the last one at least `ARCH_AHEAD` short of the first
//! arch (so the arch reads whole, not as a roof over the ape) and past
//! `START_CLEAR`. Kept out of the
//! finish: the mesa is being reworked, and the start's straight is the
//! map's opening view anyway.

use super::{DeckKind, RavineLayout, RoomKind};
use crate::course::Course;
use crate::fx::int;
use crate::geom::{yaw_of, V3};

/// The lobby's log lies at least this far short of the first arch.
pub const ARCH_AHEAD: i32 = int(90);
/// And at least this far past the start.
pub const START_CLEAR: i32 = int(60);

/// The lobby spot on `c`'s layout `r`: the feet on the chosen log
/// log (its middle) and the yaw the camera looks along (down the
/// canyon); the start, looking along the course, if no log fits.
pub fn lobby_spot(c: &Course, r: &RavineLayout) -> (V3, i32) {
    let fallback = (c.starts[0], c.start_yaw);
    let Some(arch) = r.rooms.iter().find(|m| m.kind == RoomKind::Arch) else {
        return fallback;
    };
    let start = c.path.first().copied().unwrap_or(V3::ZERO);
    let best = r
        .decks
        .iter()
        .filter(|d| d.kind == DeckKind::Log)
        .map(|d| {
            let mid = d.from.lerp(d.to, crate::fx::ONE / 2);
            (d, mid)
        })
        .filter(|(_, mid)| {
            mid.sub(arch.from).flat().longer_than(ARCH_AHEAD)
                && mid.sub(start).flat().longer_than(START_CLEAR)
                && mid.sub(start).flat().len() < arch.from.sub(start).flat().len()
        })
        .max_by_key(|(_, mid)| mid.sub(start).flat().len());
    let Some((d, mid)) = best else {
        return fallback;
    };
    let feet = V3 {
        y: d.top_at(mid).unwrap_or(mid.y),
        ..mid
    };
    // down the canyon: the course's heading where it passes under the
    // log
    let i = (0..c.path.len())
        .min_by_key(|&i| c.path[i].sub(mid).flat().len())
        .unwrap_or(0);
    let look = c.heading.get(i).copied().unwrap_or(V3 {
        x: 0,
        y: 0,
        z: crate::fx::ONE,
    });
    (feet, yaw_of(look.flat()))
}
