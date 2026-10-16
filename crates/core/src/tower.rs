//! Mape's Tower (2026-10-15, `briefs/mapes-tower.md`): the wizard tower
//! map. The apes swing up and around a giant Ranger's-Apprentice stone keep
//! into the sky, out along offshoot bridges to little sub-towers, through a
//! trampoline room and the clouds, to the summit where an insane wizard
//! rips a tear in reality (a giant eldritch banana beyond it). Through the
//! tear they fall down the zipper space, the tower's half mirrored and
//! turned topsy-turvy, to infinite bananas and the finish.
//!
//! **The light:** two weathers, Twilight and Dawn (`snow::Weather::offered`);
//! Clear is a neutral late-afternoon sky until the art agent picks (TW7).
//!
//! **Along the course on a path that climbs then falls (TW1).** Every other
//! map's centreline (`Course.path`) lies at height 0 and progress is found
//! in plan (x and z). Here the centreline is fully 3D: each sample has its
//! height, `Course.heading` is the 3D unit tangent (it climbs, then falls),
//! `Course.right` stays level (`right_of` the heading). On this map alone
//! `Course::nearest`, `nearest_anywhere` and `progress` measure in 3D, so
//! the coils of the climb (one above another in plan) are told apart by
//! height. The centreline is the line the apes travel: vines hang to just
//! above it, reset pads are a few metres under it. Progress is the nearest
//! sample along the path plus the 3D projection on its tangent.
//!
//! **The two halves are one path.** The climb runs from `path[0]` to the
//! tear (`TowerLayout::split`, half the course's length); the zipper is the
//! climb mirrored across the plane `x = Zipper::mirror_x` and run backwards
//! (`TowerLayout::mirror`, `mirror_dir`), so both halves are exactly the same
//! length and the descent winds down a mirrored tower's worth of coils. The
//! climb's last leg is level and heads +x into the plane, so the path is
//! smooth through the tear.
//!
//! **Rules on this map** (each gated on `Course.tower`, so the other maps
//! are untouched):
//!
//! - *The stone is solid, nothing else is (TW12, `tower/stone.rs`):* the
//!   keep, the sub-towers and their roofs, the trampoline room's shell
//!   (its doorway and roof hole open), the bridges' masonry and the topsy
//!   towers push a body out (`Ape::collide`, `Happening::Wall`); their tops
//!   are ground (`Course::support`). There is no corridor wall and no
//!   ceiling (`Course::contain` does nothing) and `Course::on_floor` never
//!   fires (as on Sector Ape).
//! - *The boundary reset* (`tower/rules.rs`): an ape too far from the
//!   centreline (`OUT_FAR`), too far under it (`OUT_BELOW`), or down on the
//!   open ground is put on the reset pad (`Pad`) last behind it along the
//!   course, flying off it with the pad's boost (up and on, toward the next
//!   vines); with Floor is Lava it is the kill.
//! - *Climb vines* (`TowerLayout.lifts`): letting go of one while rising adds
//!   `CLIMB_LIFT` upward (a placeholder; tuned until climbing feels as easy
//!   as a flat map).
//! - *Bouncers* (the trampolines, the room's trampoline floor): plain
//!   `Course.mushrooms`.
//!
//! The types below are the contract between the course (placement, rules,
//! the AI), the models and the scene. Positions are world space, fixed
//! point (`fx::ONE` = 1 m); yaws are `fx::TURN` units, 0 facing +z.
//!
//! STAND-IN (the skeleton, 2026-10-15): `tower()` is a plain helix round
//! one keep and its mirror, enough to prove the plumbing (an AI climbs,
//! falls and finishes; the reset works). The course agent replaces it.

use crate::fx::{cm, int};
use crate::geom::{v3, V3};

#[cfg(test)]
mod audit;
mod layout;
mod rules;
mod stone;
#[cfg(test)]
mod tests;

pub use layout::{tower, tower_seeded, Climb, CLIMB, START_TOP};
pub use stone::{Gap, Solid};

/// A body further than this from the centreline (in 3D) is out.
pub const OUT_FAR: i32 = int(45);
/// A body further than this under the centreline is out.
pub const OUT_BELOW: i32 = int(18);
/// The upward speed a climb vine adds on release while rising
/// (PLACEHOLDER: the person asked for it tuned until climbing feels as
/// easy as a flat map; measure a Monkey's climb).
pub const CLIMB_LIFT: i32 = int(3);

/// Everything on Mape's Tower that the Jungle's `Course` has no word for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TowerLayout {
    /// the main tower the climb winds round
    pub keep: Keep,
    /// offshoot bridges from the keep out to little sub-towers: the
    /// straight, level stretches of the climb (no dizzying all-spiral)
    pub branches: Vec<Branch>,
    /// balconies on the keep and sub-towers (mages stand on them; some
    /// are walkable `Course.platforms`)
    pub balconies: Vec<Balcony>,
    /// the room inside a tower full of trampolines, left through its roof
    pub room: Room,
    /// the cloud bands the climb passes through (clouds also hold vine
    /// anchors: `Hanger::Cloud`)
    pub clouds: Vec<CloudBand>,
    /// the top: the wizard, the tear, the banana beyond
    pub summit: Summit,
    /// the descent: the mirror plane and the zip
    pub zipper: Zipper,
    /// the finish: infinite bananas, the apes dancing
    pub banana_zone: BananaZone,
    /// where a reset puts an ape, in course order (the first is the start)
    pub pads: Vec<Pad>,
    /// what casts each vine (exactly one per `Course.vines`, same order)
    pub hangers: Vec<Hanger>,
    /// whether each vine is a climb vine (`CLIMB_LIFT` on release while
    /// rising); one per `Course.vines`, same order
    pub lifts: Vec<bool>,
    /// the unit way each vine hangs from its anchor (the spell sets the
    /// beam's orientation); one per `Course.vines`, same order. Most are
    /// straight down (`sector::DOWN`), a few lean, more in the zipper.
    pub hangs: Vec<V3>,
    /// where an ape stands behind the menus: the hands' place and the way
    /// it faces
    pub lobby: (V3, i32),
    /// the solid stonework (TW12: the keep, the sub-towers, the room's
    /// shell, the bridges' masonry, the topsy towers), built from the
    /// fields above (`stone.rs`)
    pub stone: Vec<Solid>,
}

/// The main tower: a round stone keep (battlements, arrow slits, a
/// conical roof under the summit deck).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keep {
    /// the middle of its base, on the ground
    pub centre: V3,
    /// the wall's outer radius
    pub radius: i32,
    /// its height from the base to the summit deck
    pub height: i32,
    /// the heights of its floors (storeys) above the base, lowest first:
    /// where windows, balconies and string courses line up
    pub levels: Vec<i32>,
}

/// An offshoot: a bridge out from the keep to a little sub-tower. The
/// course runs straight along it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    /// the bridge's deck end on the keep's wall, and at the sub-tower
    pub from: V3,
    pub to: V3,
    /// the bridge's half width (0: a drawn span only, nothing to stand on)
    pub half_width: i32,
    /// its deck in `Course.platforms`, if it can be stood on
    pub deck: Option<usize>,
    /// the sub-tower at its end
    pub sub: SubTower,
}

/// A little tower off the keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubTower {
    /// the middle of its base (on the ground, or on a corbel off the keep)
    pub centre: V3,
    pub radius: i32,
    /// its top (a pointed roof above)
    pub top: i32,
}

/// A balcony on a tower wall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Balcony {
    /// the middle of its floor's outer edge
    pub at: V3,
    /// the way it faces out from the wall (level, unit)
    pub out: V3,
    pub half_width: i32,
    pub depth: i32,
    /// whether a mage stands on it casting (its beams are vines:
    /// `Hanger::Mage`)
    pub mage: bool,
    /// its floor in `Course.platforms`, if an ape can land on it
    pub deck: Option<usize>,
}

/// The trampoline room: a round hall inside a tower, its floor all
/// trampolines (`Course.mushrooms`), one mage in the middle inside an orb
/// of vines (`Hanger::Orb`); the way on is up through a hole in its roof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Room {
    /// the middle of its floor
    pub centre: V3,
    pub radius: i32,
    /// floor to roof
    pub height: i32,
    /// where the course comes in through its wall (a doorway's middle)
    pub door: V3,
    /// the hole in the roof: its middle and radius
    pub roof_exit: (V3, i32),
    /// the trampolines, indices in `Course.mushrooms`
    pub trampolines: Vec<usize>,
    /// the orb round the mage: its middle and radius
    pub orb: (V3, i32),
}

/// A band of cloud the climb rises through (drawing: soft puffs between
/// the heights, thinner near the keep so the course stays seen).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloudBand {
    pub bottom: i32,
    pub top: i32,
}

/// The summit: a deck on the keep's top where an insane wizard rips a
/// tear in reality, a giant eldritch banana on the other side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Summit {
    /// the summit deck's middle (its top)
    pub deck: V3,
    pub radius: i32,
    /// where the wizard stands (feet), and the way he faces
    pub wizard: V3,
    pub wizard_yaw: i32,
    /// the tear: its middle (on the mirror plane, where the path crosses
    /// it), half its height and width, and the way through it (unit)
    pub tear: V3,
    pub tear_half: (i32, i32),
    pub through: V3,
    /// the eldritch banana just beyond the tear, seen only through it:
    /// its middle and length
    pub banana: V3,
    pub banana_len: i32,
}

/// The zipper space: the climb mirrored across `x = mirror_x` and run
/// backwards, going down. Drawn eerie sea-green and violet, a giant zip
/// running down the void, blob and sea-monster creatures casting the
/// vines (`Hanger::Monster`). "Topsy-turvy" is the course agent's to add on
/// top of the mirror (it must stay as long as the climb).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zipper {
    pub mirror_x: i32,
    /// the zip's axis (the mirrored keep's middle) and its top and bottom
    pub axis: V3,
    pub top: i32,
    pub bottom: i32,
}

/// The finish: a field of bananas without end, where the apes dance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BananaZone {
    pub centre: V3,
    pub radius: i32,
    /// its floor in `Course.platforms`
    pub platform: usize,
}

/// A reset pad: a flat surface tuned for a good return into the course.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pad {
    /// its surface in `Course.platforms`
    pub platform: usize,
    /// how far along the course it is (its middle's progress)
    pub at: i32,
    /// the velocity an ape put back on it leaves with: up and on, toward
    /// the next vines (PLACEHOLDER strengths; the course agent tunes them)
    pub boost: V3,
}

/// What casts a vine (a spell beam the apes swing on).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hanger {
    /// the start gantry over the start deck
    Start,
    /// a mage on balcony `n` of `TowerLayout.balconies`
    Mage { balcony: usize },
    /// a mage riding a dragon outside the tower, circling round `at`
    Dragon { at: V3 },
    /// a cloud in band `n` of `TowerLayout.clouds`
    Cloud { band: usize },
    /// the mage in the trampoline room's orb
    Orb,
    /// a blob or sea-monster creature in the zipper space at `at`
    Monster { at: V3 },
}

impl TowerLayout {
    /// The point mirrored into (or out of) the zipper space.
    pub fn mirror(&self, p: V3) -> V3 {
        v3(2 * self.zipper.mirror_x - p.x, p.y, p.z)
    }

    /// A direction along the climb as the zipper runs it (mirrored and
    /// reversed): `(x, -y, -z)`.
    pub fn mirror_dir(d: V3) -> V3 {
        v3(d.x, -d.y, -d.z)
    }

    /// Whether a point is in the zipper space (past the mirror plane).
    pub fn in_zipper(&self, p: V3) -> bool {
        p.x > self.zipper.mirror_x
    }

    /// Where sub-tower `s`'s topsy-turvy twin hangs in the zipper (its
    /// foot, it hangs down from there `TOPSY_TILT` off straight): in the
    /// void where the mirrored keep would be, 20 m from the zip, out of
    /// every swing's way (TW12; hung at the mirrored sub-tower's top the
    /// zipper's beams hung inside them).
    pub fn topsy_hang(&self, s: &SubTower) -> V3 {
        let m = self.mirror(v3(s.centre.x, s.top, s.centre.z));
        let axis = v3(self.zipper.axis.x, m.y, self.zipper.axis.z);
        axis.add(m.sub(axis).flat().with_len(int(20)))
    }

    /// Whether vine `i` is a climb vine.
    pub fn lifts(&self, i: usize) -> bool {
        self.lifts.get(i).copied().unwrap_or(false)
    }
}

/// The climb's share of the course: the tear is at half its length.
pub fn split(course_length: i32) -> i32 {
    course_length / 2
}

/// How far a topsy tower leans off straight down (a turn's share).
pub const TOPSY_TILT: i32 = crate::fx::ONE / 40;

/// The thickness of a reset pad's slab (drawing).
pub const PAD_THICK: i32 = cm(60);
