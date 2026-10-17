//! The eye on Rattlesnake Ravine (MAPS gotcha 5, `notes/ravine-pitfalls.md`):
//! while the ape is in an arch, a cave, the mine tunnel or the T-rex's
//! mouth (`RavineLayout::keeps`), the eye stays inside that room's
//! cross-section (under its roof, between its sides, over its floor, and
//! on past its mouths a little, as `Temple::kept_in`); and anywhere, it
//! never sits in a solid (`ravine::Solids`: a wall, a roof, the bone). In
//! both cases it is brought in toward the ape along the line between them,
//! to the furthest point before it would leave. The other maps have none
//! of this.

use crate::course::Course;
use crate::fx::cm;
use crate::geom::{right_of, V3};
use crate::ravine::{Room, Solids};
use std::sync::Arc;

/// How near a wall, a roof or a solid the eye may come, and how far over
/// a room's floor it stays.
const GAP: i32 = cm(60);
const FLOOR_GAP: i32 = cm(80);
/// How far past its mouths a room's cross-section still holds the eye.
const MOUTH_RUN: i32 = cm(300);
/// The step along the line from the ape to the eye.
const STEP: i32 = cm(20);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keeps {
    rooms: Vec<Room>,
    solids: Arc<Solids>,
}

impl Keeps {
    /// Rattlesnake Ravine's rooms and solids (`None` on the other maps).
    pub fn of(c: &Course) -> Option<Keeps> {
        let r = c.ravine.as_deref()?;
        Some(Keeps {
            rooms: r.keeps().copied().collect(),
            solids: Arc::clone(c.ravine_solids()?),
        })
    }

    /// Whether the eye may be at `p` while following an ape in `room`.
    fn open_in(room: &Room, p: V3) -> bool {
        let run = room.to.sub(room.from).flat();
        let len = run.len();
        if len <= 0 {
            return true;
        }
        let fwd = run.norm();
        let d = p.sub(room.from).flat();
        let along = d.dot(fwd);
        if along < -MOUTH_RUN || along > len + MOUTH_RUN {
            return true;
        }
        let a = along.clamp(0, len);
        let floor = room.from.y + ((room.to.y - room.from.y) as i64 * a as i64 / len as i64) as i32;
        d.dot(right_of(fwd)).abs() <= room.half_width - GAP
            && p.y >= floor + FLOOR_GAP
            && p.y <= floor + room.height - GAP
    }

    /// Whether the eye may be at `p` (following an ape in `room`, if any).
    pub fn open(&self, room: Option<&Room>, p: V3) -> bool {
        room.is_none_or(|r| Keeps::open_in(r, p)) && !self.solids.holds(p, GAP)
    }

    /// The room the ape at `target` is in, if any.
    fn room_of(&self, target: V3) -> Option<&Room> {
        self.rooms.iter().find(|r| r.contains(target))
    }

    /// The eye `eye` kept inside the ape's room and out of the solids,
    /// following the ape at `target`. An ape where the eye could not be
    /// (in a solid's margin) is left be.
    pub fn kept_in(&self, target: V3, eye: V3) -> V3 {
        let room = self.room_of(target);
        if !self.open(room, target) {
            return eye;
        }
        let line = eye.sub(target);
        let len = line.len();
        if len < STEP {
            return eye;
        }
        let dir = line.norm();
        let mut d = 0;
        while d < len {
            let next = (d + STEP).min(len);
            if !self.open(room, target.add(dir.scale(next))) {
                return target.add(dir.scale(d));
            }
            d = next;
        }
        eye
    }
}
