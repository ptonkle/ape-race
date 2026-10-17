//! Mape's Tower, second design (2026-10-16, `briefs/mapes-tower-v2.md`,
//! `notes/tower2-plan.md`): two great stone towers joined by two long
//! straight bridges. The apes start at the foot of tower 1, swing under
//! the lower bridge to tower 2, climb tower 2's outside, swing back under
//! the upper bridge to a doorway half way up tower 1, climb tower 1's
//! hollow inside on scattered beams, and leave through the wizard's tear
//! beside its open top. Past the tear the void is a vertical tube
//! (Dropper-style): a long fall past bounce pads to dodge, a row of
//! near-horizontal vines at the bottom that sling a falling ape level into
//! a horizontal tunnel, the finish part way along it, the banana sea past
//! that.
//!
//! Sizes, as the person asks, are comparisons to the first design (its
//! keep: 137 m tall, 41 m radius, spirals at 55 m): tower 1 is twice the
//! keep's height and 1.25× its radius, hollow; tower 2 is 60% of tower 1
//! (1.2× the keep); each bridge is about two of the old spirals long
//! (10 s of a Monkey's swinging); the tube is about 2.2 keeps deep and
//! 2/3 of a spiral across.
//!
//! **The light:** two weathers, Twilight and Dawn (`snow::Weather::offered`);
//! Clear is a golden late afternoon (TW7).
//!
//! **Along the course on a path that climbs then falls (TW1).** Every other
//! map's centreline (`Course.path`) lies at height 0 and progress is found
//! in plan (x and z). Here the centreline is fully 3D: each sample has its
//! height, `Course.heading` is the 3D unit tangent (level under the
//! bridges, rising round tower 2 and inside tower 1, straight down the
//! tube, level along the tunnel), `Course.right` stays level (`right_of`
//! the heading; down the tube, where the heading has no plan direction, it
//! is the right of the way through the portal). On this map alone
//! `Course::nearest`, `nearest_anywhere` and `progress` measure in 3D, so
//! the climbs (the spiral's turns one above another in plan; the line up
//! tower 1's axis with no plan direction at all) are told apart by
//! height. The centreline is only what progress measures: under the
//! bridges and round tower 2 the beams hang to just above it; inside
//! tower 1 it is a straight line up the hollow's axis (progress there is
//! height, TW2-9) that the beams do not follow.
//!
//! **The legs** (`Leg`, `TowerLayout::leg_at`): `Start`, `LowerBridge`,
//! `Tower2`, `UpperBridge`, `Inside`, `Portal`, `Tube`, `Sling`, `Tunnel`,
//! each a run of samples along the course (`TowerLayout.legs`).
//!
//! **Rules on this map** (each gated on `Course.tower`, so the other maps
//! are untouched):
//!
//! - *The stone is solid, nothing else is (TW12, `tower/stone.rs`):* both
//!   towers (tower 1 a hollow ring with its doorway open and its top open),
//!   the bridges' masonry and the tube's wall (its mouth into the tunnel
//!   open) push a body out (`Ape::collide`, `Happening::Wall`); their tops
//!   are ground (`Course::support`). There is no corridor wall and no
//!   ceiling (`Course::contain` does nothing) and `Course::on_floor` never
//!   fires (as on Sector Ape).
//! - *The boundary reset* (`tower/rules.rs`): an ape too far from the
//!   centreline (`OUT_FAR`), too far under it (`OUT_BELOW`), or down on the
//!   open ground (outside the towers, or on tower 1's floor) is put on the
//!   reset pad (`Pad`) last behind it along the course, flying off it with
//!   the pad's boost (up and on, toward the next beams); inside tower 1
//!   the pads are ledges on the inner wall; in the tube and on the sling
//!   the pad is the portal's (`TowerLayout::reset_pad`); with Floor is
//!   Lava it is the kill. A bridge's deck can be walked but the race never
//!   goes over it; falling off it is this reset, to a pad under it.
//! - *Climb vines* (`TowerLayout.lifts`): letting go of one while rising adds
//!   `CLIMB_LIFT` upward.
//! - *The tube's air* (`TUBE_TERMINAL`, `TUBE_AIR`, `Ape::tube_air`): the
//!   void's air is thicker, so a fall settles at a lower terminal speed
//!   than open air, and steering in the air is `TUBE_AIR` times the kind's,
//!   so the gaps can be reached one from the next (measured: TW2-5,
//!   `MEASUREMENTS.md`). The AI falls the tube with the same steering
//!   (`TowerLayout::tube_controls`).
//! - *The tube's pads* (`TubePad`, `Tube::pad_hit`): a falling body whose
//!   feet cross a pad's covered part is thrown back up at the mushroom's
//!   bounce (`MUSHROOM_BOUNCE`, no push along), losing a second or two.
//!   Nothing else in the tube is a bouncer.
//!
//! The types below are the contract between the course (placement, rules,
//! the AI), the models and the scene. Positions are world space, fixed
//! point (`fx::ONE` = 1 m); yaws are `fx::TURN` units, 0 facing +z.
//!
//! STAND-IN (the skeleton, 2026-10-16): `tower()` lays the legs out with
//! plain beams (lanes under the bridges, a spiral's lanes round tower 2,
//! rings of scattered beams inside tower 1), a sinusoidal weave of pads down
//! the tube and the sling row, enough to prove the plumbing (every AI
//! level finishes; the resets work; the tube's air is measured). The
//! course agent replaces the placement; the contract stays.

use crate::fx::{cm, int, ONE};
use crate::geom::{right_of, v3, V3};

#[cfg(test)]
mod audit;
mod layout;
mod rules;
mod stone;
#[cfg(test)]
mod tests;

pub use layout::{
    tower, tower_seeded, Climb, LEGS, START_TOP, TUBE_PADS, WEAVE_AMPLITUDE, WEAVE_PERIOD,
};
pub use rules::{TUBE_AIR, TUBE_SETTLE, TUBE_TERMINAL};
pub use stone::{Gap, Solid};

/// A body further than this from the centreline (in 3D) is out.
pub const OUT_FAR: i32 = int(45);
/// A body further than this under the centreline is out.
pub const OUT_BELOW: i32 = int(18);
/// The upward speed a climb vine adds on release while rising
/// (PLACEHOLDER: the person asked for it tuned until climbing feels as
/// easy as a flat map; measure a Monkey's climb).
pub const CLIMB_LIFT: i32 = int(3);

/// The legs of the race, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Leg {
    /// the run-in off the start deck at tower 1's foot
    Start,
    /// under the lower bridge, tower 1 to tower 2
    LowerBridge,
    /// up and round the outside of tower 2
    Tower2,
    /// under the upper bridge, tower 2 back to tower 1's doorway
    UpperBridge,
    /// up the hollow inside tower 1
    Inside,
    /// out over tower 1's open top and sideways through the tear
    Portal,
    /// the fall down the tube past its pads
    Tube,
    /// the sling row at the tube's bottom
    Sling,
    /// along the tunnel to the finish and the banana sea
    Tunnel,
}

/// A leg's span along the course: from `from` metres to the next leg's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegSpan {
    pub leg: Leg,
    pub from: i32,
}

/// Everything on Mape's Tower that the Jungle's `Course` has no word for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TowerLayout {
    /// tower 1 (the start, hollow, climbed inside) and tower 2 (climbed
    /// outside)
    pub towers: [Tower; 2],
    /// the lower bridge (tower 1 to tower 2) and the upper (back)
    pub bridges: [Bridge; 2],
    /// the doorway into tower 1 where the upper bridge's vines end
    pub doorway: Doorway,
    /// balconies: on tower 2's outside (mages cast the climb's beams) and
    /// on tower 1's inside (`Balcony::inside`)
    pub balconies: Vec<Balcony>,
    /// windows in tower 1's wall: light in, and anchors for inside beams
    pub windows: Vec<Window>,
    /// the cloud band round tower 2's top
    pub clouds: Vec<CloudBand>,
    /// the tear beside tower 1's top, the wizard, the banana beyond
    pub portal: Portal,
    /// the void's tube and its pads
    pub tube: Tube,
    /// the tunnel's mouth in the tube's wall, the sling row hung from its
    /// top edge, the tunnel and the finish
    pub tunnel: Tunnel,
    /// the finish: infinite bananas, the apes dancing
    pub banana_zone: BananaZone,
    /// where each leg begins along the course, in order
    pub legs: Vec<LegSpan>,
    /// where a reset puts an ape, in course order (the first is the start)
    pub pads: Vec<Pad>,
    /// which pad a reset in the tube or on the sling goes to (the portal's)
    pub portal_pad: usize,
    /// what casts each vine (exactly one per `Course.vines`, same order)
    pub hangers: Vec<Hanger>,
    /// whether each vine is a climb vine (`CLIMB_LIFT` on release while
    /// rising); one per `Course.vines`, same order
    pub lifts: Vec<bool>,
    /// the unit way each vine hangs from its anchor (the spell sets the
    /// beam's orientation); one per `Course.vines`, same order. Most are
    /// straight down (`sector::DOWN`); inside tower 1 the wall's beams lean
    /// in; the sling row hangs nearly level.
    pub hangs: Vec<V3>,
    /// where an ape stands behind the menus: the hands' place and the way
    /// it faces
    pub lobby: (V3, i32),
    /// the solid stonework (TW12: both towers, the bridges' masonry, the
    /// tube's wall), built from the fields above (`stone.rs`)
    pub stone: Vec<Solid>,
}

/// A great round stone tower.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tower {
    /// the middle of its base, on the ground
    pub centre: V3,
    /// the wall's outer radius
    pub radius: i32,
    /// the wall's inner radius: 0 for a solid tower (tower 2); tower 1 is
    /// hollow, its inside about 2.2 old keeps across
    pub inner: i32,
    /// its height from the base to its top (tower 1's is open; tower 2's
    /// is a walkable crown where the upper bridge leaves)
    pub height: i32,
    /// the heights of its floors (storeys) above the base, lowest first:
    /// where windows, balconies and string courses line up
    pub levels: Vec<i32>,
}

impl Tower {
    /// The height of its top.
    pub fn top(&self) -> i32 {
        self.centre.y + self.height
    }
}

/// A straight bridge between the towers. The race runs under it, on
/// beams hung from its underside; its deck can be walked but the course
/// never goes over it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bridge {
    /// the deck's top at each end, on a tower's wall
    pub from: V3,
    pub to: V3,
    pub half_width: i32,
    /// the deck's thickness: the beams hang from this far under its top
    pub thick: i32,
    /// its deck in `Course.platforms`
    pub deck: Option<usize>,
}

/// The doorway in tower 1's wall where the upper bridge's vines end and
/// the inside climb begins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Doorway {
    /// the middle of its sill, on the wall's outer face
    pub at: V3,
    /// the way out of the tower there (level, unit)
    pub out: V3,
    pub half_width: i32,
    pub height: i32,
}

/// A balcony on a tower wall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Balcony {
    /// the middle of its floor's outer edge
    pub at: V3,
    /// the way it faces out from the wall (level, unit): into the hollow
    /// for an inside balcony
    pub out: V3,
    pub half_width: i32,
    pub depth: i32,
    /// whether a mage stands on it casting (its beams are vines:
    /// `Hanger::Mage`)
    pub mage: bool,
    /// its floor in `Course.platforms`, if an ape can land on it
    pub deck: Option<usize>,
    /// which tower (0 or 1) and whether it is on the inner wall (tower 1)
    pub tower: usize,
    pub inside: bool,
}

/// A window in tower 1's wall (light in; an anchor for an inside beam).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    /// the middle of its sill on the wall's inner face
    pub at: V3,
    /// the way into the hollow there (level, unit)
    pub into: V3,
    pub half_width: i32,
    pub height: i32,
}

/// A band of cloud (drawing: soft puffs between the heights, thinner near
/// the towers so the course stays seen).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloudBand {
    pub bottom: i32,
    pub top: i32,
}

/// The tear beside tower 1's top, on its far side: the insane wizard rips
/// it, the void and the eldritch banana are seen through it. It moves like
/// Sector Ape's black hole, simpler (the art's); the race passes through
/// its middle sideways.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Portal {
    /// where the wizard stands (feet), and the way he faces
    pub wizard: V3,
    pub wizard_yaw: i32,
    /// the tear: its middle (where the path crosses it), half its height
    /// and width, and the way through it (unit, level)
    pub tear: V3,
    pub tear_half: (i32, i32),
    pub through: V3,
    /// the eldritch banana just beyond the tear, seen only through it:
    /// its middle and length
    pub banana: V3,
    pub banana_len: i32,
}

/// The void's tube: a vertical fall, Dropper-style, with pads to dodge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tube {
    /// its axis (x and z), its top and its floor (the tunnel's floor)
    pub axis: V3,
    pub top: i32,
    pub bottom: i32,
    /// its inside radius
    pub radius: i32,
    /// its wall's thickness
    pub wall: i32,
    /// the pads, top first
    pub pads: Vec<TubePad>,
}

/// A bounce pad across part of the tube at one height.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TubePad {
    /// its top's height
    pub y: i32,
    pub shape: PadShape,
    /// the middle of its gap, in plan, from the tube's axis (what the
    /// fastest line steers for)
    pub gap: V3,
}

/// Which part of the tube's width a pad covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadShape {
    /// the disc less a segment: open beyond `edge` along `open` (unit,
    /// level) from the axis
    Slab { open: V3, edge: i32 },
    /// the disc less a round hole of radius `hole` at `at` (in plan, from
    /// the axis)
    Ring { at: V3, hole: i32 },
    /// two opposite quarters, round ±`axis` (unit, level); open round
    /// the other two
    Quarters { axis: V3 },
}

/// The pads' thickness (drawing, and the band a crossing is found in).
pub const TUBE_PAD_THICK: i32 = cm(80);

impl Tube {
    /// Whether a body with its hands at `p` is inside the tube.
    pub fn holds(&self, p: V3) -> bool {
        p.y <= self.top
            && p.y >= self.bottom
            && !v3(p.x - self.axis.x, 0, p.z - self.axis.z).longer_than(self.radius)
    }

    /// Whether pad `k` covers the plan point `rel` (from the axis).
    pub fn covered(&self, k: usize, rel: V3) -> bool {
        if rel.longer_than(self.radius) {
            return false;
        }
        match self.pads[k].shape {
            PadShape::Slab { open, edge } => rel.dot(open) <= edge,
            PadShape::Ring { at, hole } => rel.sub(at).longer_than(hole),
            PadShape::Quarters { axis } => rel.dot(axis).abs() >= rel.dot(right_of(axis)).abs(),
        }
    }

    /// The pad a body falling from hands at `before` to `after` lands on:
    /// its feet cross the pad's top (to its thickness) over a covered
    /// part.
    pub fn pad_hit(&self, before: V3, after: V3) -> Option<usize> {
        let drop = crate::rules::BODY_DROP;
        let (f0, f1) = (before.y - drop, after.y - drop);
        if f1 >= f0 {
            return None;
        }
        let rel = v3(after.x - self.axis.x, 0, after.z - self.axis.z);
        (0..self.pads.len()).find(|&k| {
            let y = self.pads[k].y;
            f0 >= y - TUBE_PAD_THICK && f1 < y && self.covered(k, rel)
        })
    }

    /// The next pad under a body with its hands at `p`, if any.
    pub fn next_pad(&self, p: V3) -> Option<usize> {
        let feet = p.y - crate::rules::BODY_DROP;
        (0..self.pads.len()).find(|&k| self.pads[k].y < feet)
    }
}

/// The tunnel off the tube's bottom: its mouth in the tube's wall, the
/// sling row hung from the mouth's top edge, the finish part way along.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tunnel {
    /// the mouth: the middle of its floor at the tube's wall, the way
    /// along the tunnel (unit, level), its half width and its height
    pub mouth: V3,
    pub along: V3,
    pub half_width: i32,
    pub height: i32,
    /// its length from the mouth
    pub length: i32,
    /// how far along the tunnel the finish line is
    pub finish: i32,
    /// its floor in `Course.platforms`
    pub floor: usize,
    /// how far under level the sling row hangs (a turn's share: 10°)
    pub sling_dip: i32,
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
    /// a rune under bridge `n`'s deck
    Bridge { bridge: usize },
    /// a mage on balcony `n` of `TowerLayout.balconies`
    Mage { balcony: usize },
    /// a mage riding a dragon outside tower 2, circling round `at`
    Dragon { at: V3 },
    /// a cloud in band `n` of `TowerLayout.clouds`
    Cloud { band: usize },
    /// a rune in window `n` of `TowerLayout.windows` (tower 1's inside)
    Window { window: usize },
    /// a conjured anchor floating in tower 1's hollow at `at`
    Floating { at: V3 },
    /// the sling row's rune on the tunnel mouth's top edge
    Sling,
}

impl TowerLayout {
    /// Which leg `along` metres of the course is on.
    pub fn leg_at(&self, along: i32) -> Leg {
        self.legs
            .iter()
            .rev()
            .find(|l| l.from <= along)
            .map_or(Leg::Start, |l| l.leg)
    }

    /// Where leg `leg` begins along the course (the course's length if it
    /// has no such leg).
    pub fn leg_start(&self, leg: Leg, length: i32) -> i32 {
        self.legs
            .iter()
            .find(|l| l.leg == leg)
            .map_or(length, |l| l.from)
    }

    /// Whether a body with its hands at `p` is inside tower 1's hollow
    /// (under its rim).
    pub fn inside_tower_1(&self, p: V3) -> bool {
        let t1 = &self.towers[0];
        p.y < t1.top()
            && p.y >= t1.centre.y
            && !v3(p.x - t1.centre.x, 0, p.z - t1.centre.z).longer_than(t1.inner)
    }

    /// Whether a point is in the void (past the portal's plane).
    pub fn in_void(&self, p: V3) -> bool {
        p.sub(self.portal.tear).dot(self.portal.through) > 0
    }

    /// Whether vine `i` is a climb vine.
    pub fn lifts(&self, i: usize) -> bool {
        self.lifts.get(i).copied().unwrap_or(false)
    }

    /// The reset pad for an ape `along` metres along the course: the last
    /// pad not ahead of it (the first, the start deck, if none); in the
    /// tube and on the sling, the portal's.
    pub fn reset_pad(&self, along: i32) -> Pad {
        match self.leg_at(along) {
            Leg::Tube | Leg::Sling => self.pads[self.portal_pad],
            _ => self
                .pads
                .iter()
                .rev()
                .find(|pad| pad.at <= along)
                .or(self.pads.first())
                .copied()
                .expect("the tower has a start pad"),
        }
    }
}

/// The thickness of a reset pad's slab (drawing).
pub const PAD_THICK: i32 = cm(60);

/// One metre, for the art's comparisons.
pub const METRE: i32 = ONE;
