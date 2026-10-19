//! The Spindle (2026-10-18, `briefs/spindle.md`): one long straightaway
//! inside a spinning tube in space, after Iron Vector's Spindle, built like
//! a Minecraft dropper map. Pitfalls every agent checks:
//! `notes/spindle-pitfalls.md`.
//!
//! **Simulated flat, drawn bent.** As in Iron Vector, the physics, the AI
//! and the solids run on the tube's floor unrolled: x across (round the
//! tube), z along it, y up from the floor toward the axis. The floor is
//! the plane y = 0, gravity pulls toward it as on every other map, and x
//! wraps: an ape that goes `CIRCUMFERENCE` round comes back where it began
//! (`Warp::wrap_x`, applied when it is not holding a vine). The drawing and
//! the camera are bent onto the inside of the cylinder (`Warp`): the floor
//! at x lies at angle x / `RADIUS` round the axis, and height climbs toward
//! the axis, so the floor curves up overhead and gravity throws an ape
//! outward onto the wall. The camera rolls with the ape, so "down" on the
//! screen is always the wall under it.
//!
//! **The axle:** a square spindle runs down the tube's middle, `AXLE` in
//! radius; its underside is the ceiling (`AXLE_UNDER` over the floor), and
//! the vines hang from it at every angle round the tube toward the wall.
//!
//! **Blocks:** the obstacles are blocks, as in a dropper map: boxes in the
//! flat frame (`Block`), solid on their sides and underside, their tops a
//! floor; a slime block's top bounces. Spinning rings (`Spinner`) are
//! columns from the floor to the axle that travel round the tube at their
//! own speed, a pure function of the tick.
//!
//! **The spin:** the apes turn with the tube, so the tube looks still round
//! them; the stars and Sector Ape's space past the open end turn the other
//! way (`spin_at`).
//!
//! The types below are the contract between the course (placement, physics,
//! the AI) and the scene (drawing): the course fills a `SpindleLayout` on
//! `Course.spindle`; the scene draws from it, in the flat frame, and the
//! frame is bent once at the end (`Warp::affine`).

pub mod layout;
pub mod rules;
#[cfg(test)]
mod tests;

pub use layout::{spindle, spindle_seeded};

use crate::fx::{self, int, ONE, TURN};
use crate::geom::{v3, Affine, V3};

/// The tube's inner radius: the floor's distance from the axis.
pub const RADIUS: i32 = int(44);
/// The way round the inside of the tube: 2π × `RADIUS` (276.5 m).
pub const CIRCUMFERENCE: i32 = ((RADIUS as i64 * 411_775) >> 16) as i32;
/// The axle's half width.
pub const AXLE: i32 = int(9);
/// The axle's underside over the floor, in the flat frame: the ceiling.
pub const AXLE_UNDER: i32 = RADIUS - AXLE;
/// Ticks for one turn of the tube against the stars (a turn in 40 s).
pub const SPIN_TICKS: u32 = 40 * crate::rules::TICKS_PER_SECOND as u32;

/// Bends the flat frame onto the inside of the tube. The tube's axis runs
/// along z through (0, `radius`, 0), so the floor under x = 0 stays at
/// y = 0 and the start of the course is where it would be flat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Warp {
    pub radius: i32,
    pub circumference: i32,
}

impl Warp {
    pub const SPINDLE: Warp = Warp {
        radius: RADIUS,
        circumference: CIRCUMFERENCE,
    };

    /// The angle round the tube at a floor distance `x` (0 at the bottom,
    /// growing toward +x).
    pub fn angle(&self, x: i32) -> i32 {
        (x as i64 * TURN as i64 / self.circumference as i64) as i32
    }

    /// `x` brought into [-C/2, C/2): the same place round the tube.
    pub fn wrap_x(&self, x: i32) -> i32 {
        let c = self.circumference;
        (x + c / 2).rem_euclid(c) - c / 2
    }

    /// Where a flat point is in the world: `radius - y` from the axis, at
    /// its angle.
    pub fn point(&self, p: V3) -> V3 {
        let a = self.angle(p.x);
        let r = self.radius - p.y;
        v3(
            fx::mul(fx::sin(a), r),
            self.radius - fx::mul(fx::cos(a), r),
            p.z,
        )
    }

    /// A direction at floor distance `x`, turned to the tube's frame there.
    pub fn turn(&self, x: i32, d: V3) -> V3 {
        Affine::rot_z(self.angle(x)).apply_dir(d)
    }

    /// A shape placed in the flat frame, placed on the tube: moved to its
    /// bent place and turned with the floor under it (not stretched: a
    /// shape near the axle is drawn its flat size, so the scene narrows
    /// what must tile there, `narrow`).
    pub fn affine(&self, a: &Affine) -> Affine {
        Affine::rot_z(self.angle(a.t.x)).then(a).at(self.point(a.t))
    }

    /// How much narrower round the tube a span at height `y` is than on the
    /// floor (ONE on the floor, 0 at the axis).
    pub fn narrow(&self, y: i32) -> i32 {
        fx::div(self.radius - y, self.radius)
    }

    /// The flat point that bends to the world point `w` (`point`'s inverse;
    /// not on the axis itself).
    pub fn flat_of(&self, w: V3) -> V3 {
        let dx = w.x;
        let dy = self.radius - w.y;
        let r = v3(dx, dy, 0).len();
        let a = fx::atan2(dx, dy);
        let x = (a as i64 * self.circumference as i64 / TURN as i64) as i32;
        v3(self.wrap_x(x), self.radius - r, w.z)
    }

    /// The flat placement that bends to the world placement `a`: for shapes
    /// set in the world rather than on the tube (the space past the end).
    pub fn affine_of(&self, a: &Affine) -> Affine {
        let t = self.flat_of(a.t);
        Affine::rot_z(-self.angle(t.x)).then(a).at(t)
    }
}

/// The turn of the world outside against the tube at `tick`, in angle units
/// (the apes ride with the tube; the stars go round the other way).
pub fn spin_at(tick: u32) -> i32 {
    -(((tick % SPIN_TICKS) as i64 * TURN as i64 / SPIN_TICKS as i64) as i32)
}

/// What a block is made of: how it is drawn, and for slime how it plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    /// a wool block of one of the dye colours (`spindlescape::DYES`)
    Wool(u8),
    /// clear glass: solid, drawn pale and glowing faintly
    Glass,
    /// green slime: its top throws a landing ape back up to the vines
    Slime,
    /// a glowing lamp block (glowstone)
    Lamp,
    /// dark stone, for the rings' frames
    Obsidian,
}

/// A block obstacle in the flat frame: the box from `lo` to `hi` (least
/// then most corner). Solid on its sides and underside; its top is a floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    pub lo: V3,
    pub hi: V3,
    pub kind: BlockKind,
}

/// A spinning ring: `count` columns `width` wide and `depth` deep (along
/// the tube), from the floor to the axle, at `z`, travelling round the
/// tube once every `period` ticks (negative: the other way), the first at
/// floor distance `phase` at tick 0, the rest evenly round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spinner {
    pub z: i32,
    pub depth: i32,
    pub width: i32,
    pub count: u8,
    pub period: i32,
    pub phase: i32,
    pub kind: BlockKind,
}

impl Spinner {
    /// Column `k`'s middle, round the tube, at `tick` (in [-C/2, C/2)).
    pub fn x_at(&self, k: u8, tick: u32) -> i32 {
        let c = CIRCUMFERENCE as i64;
        let t = (tick as i64).rem_euclid(self.period.unsigned_abs() as i64);
        let run = t * c / self.period as i64;
        let x = self.phase as i64 + run + k as i64 * c / self.count.max(1) as i64;
        Warp::SPINDLE.wrap_x((x.rem_euclid(c)) as i32)
    }

    /// Its columns' speed round the tube (m/s, Q16).
    pub fn speed(&self) -> i32 {
        (CIRCUMFERENCE as i64 * crate::rules::TICKS_PER_SECOND as i64 / self.period as i64) as i32
    }
}

/// The Spindle's layout: the tube, its blocks and its spinning rings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpindleLayout {
    pub warp: Warp,
    /// the closed back end, behind the start
    pub back: i32,
    /// the open end, at the finish line: past it is space
    pub mouth: i32,
    pub blocks: Vec<Block>,
    pub spinners: Vec<Spinner>,
}

impl SpindleLayout {
    /// The tube's floor and axle stop here, past the finish.
    pub fn end(&self) -> i32 {
        self.mouth
    }

    /// Whether flat point `p` is out of the tube, in space past its end.
    pub fn in_space(&self, p: V3) -> bool {
        p.z > self.mouth + ONE
    }
}
