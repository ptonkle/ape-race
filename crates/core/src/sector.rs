//! Sector Ape (2026-10-13, `briefs/space-map.md`): a race through open
//! space after Star Fox's Sector X, Y and Z.
//!
//! **The light (this map's "time of day"):** there is no sky and no day. A
//! blazing orange-white sun hangs far below the track and lights everything
//! from underneath, with a strong rim; the sky is deep black-blue with
//! stars, a violet gas giant off to one side, and a purple-green haze
//! (Sector X) far off. No weather. `spacecraft::look()` is the start.
//!
//! - **Gravity:** a weak pull toward the sun below (`SECTOR_GRAVITY`),
//!   less than the other maps', so a good release flies far and a bad one
//!   blasts the ape off into space. Each ship or piece of debris carrying
//!   vines has its own "gravity", which sets only which way its vines hang
//!   (`SectorLayout.hangs`): down, sideways, up from below, at angles. The
//!   ape itself only ever feels the pull toward the sun.
//! - **No floor:** past an invisible boundary round the track
//!   (`Boundary`) the ape is reset to the nearest piece of ground (a deck,
//!   `SectorLayout.pads`). With Floor is Lava, crossing the boundary is the
//!   kill: the space jaguar (in its bubble helmet) pops into being beside
//!   the ape.
//! - **Bodies:** the ships and debris that carry vines are solid and
//!   bouncy: touching one throws the ape off elastically with a boost.
//! - **The switch door:** one shared door on one route; swinging the switch
//!   vine past it far enough forward shuts it for everyone behind, for a
//!   few seconds. Its state lives on the course (`Door.shut_at`), as
//!   summoned vines do, so the apes, the AI's plans, replays and every
//!   browser see the same door.
//! - **Mechs:** an armada of monkey mechs near the end, marching or
//!   drifting on fixed paths from the tick, with vines hanging from them.
//! - **The finish** is the banana event horizon of a black hole.
//!
//! The types below are the contract between the course (placement,
//! physics, the AI) and the scene (drawing): the course fills a
//! `SectorLayout` on `Course.sector`; the scene draws from it.

use crate::fx::{int, ONE};
use crate::geom::{v3, Affine, V3};
use crate::spacecraft::{Gait, Mech, MechKind};

pub use layout::sector;

mod layout;

/// The pull toward the sun on this map (the other maps: `rules::GRAVITY`,
/// 17). A first guess; the course tunes it.
pub const SECTOR_GRAVITY: i32 = int(9);

/// Straight down: the way a vine hangs on every other map.
pub const DOWN: V3 = v3(0, -ONE, 0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectorLayout {
    /// for each course vine: the unit direction it hangs from its anchor
    /// (its body's own "gravity"); `DOWN` for an ordinary vine. Summoned
    /// vines (past the end) hang down.
    pub hangs: Vec<V3>,
    /// for each course vine: what it hangs from
    pub hangers: Vec<Hanger>,
    /// ships and debris along the track: solid, and bouncy when `bouncy`
    pub bodies: Vec<Body>,
    /// the indices in `Course.platforms` that are ground an ape can be
    /// reset onto (decks), along the track
    pub pads: Vec<usize>,
    /// the invisible edge round the track
    pub boundary: Boundary,
    /// the shared switch door
    pub door: Door,
    /// the armada's mechs (their vines move with them)
    pub mechs: Vec<MechPath>,
    /// the destroyed mothership with the hidden route through its bow
    pub wreck: Wreck,
    /// the black hole at the finish: its middle and radius; its banana
    /// event horizon is the finish line
    pub black_hole: (V3, i32),
    /// scenery only: the mothership's pass overhead, distant battles
    pub set_pieces: Vec<SetPiece>,
    /// where an ape stands behind the menus (the lobby zone): the hands'
    /// place and the way it faces
    pub lobby: (V3, i32),
    /// the routes the track splits into (for the minimap and the AI)
    pub routes: Vec<Route>,
}

/// What a vine hangs from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hanger {
    /// a body (`SectorLayout.bodies`)
    Body(usize),
    /// a mech (`SectorLayout.mechs`): the anchor moves with it
    Mech { mech: usize, at: MechPoint },
    /// the underside (or side) of a deck (`Course.platforms`)
    Deck(usize),
    /// the switch vine: hangs from its body like any other
    Switch(usize),
}

/// Where on a mech a vine hangs (`spacecraft::MechAnchors`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MechPoint {
    Head,
    Hand(u8),
    Elbow(u8),
    Shoulder(u8),
}

/// What a body is drawn as (`spacecraft.rs` builders, or a deck).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Model {
    /// `hull_chunk` variant
    Chunk(u32),
    /// `girder`, this long (Q16 metres)
    Girder(i32),
    /// `asteroid` variant
    Asteroid(u32),
    /// a flat ship's deck or landing pad (a platform's own drawing)
    Deck,
    /// a whole ship hull with a corridor through it (the door's ship)
    Hulk,
    /// an `ape_wing` fighter, parked, as a small body
    Wing,
}

/// A solid piece in space. Its collision shape is the box `half` (its own
/// axes, before `frame`'s rotation) about `frame.t`; `frame` places the
/// model (rotation and scale, as `spacecraft::pose`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Body {
    pub model: Model,
    pub frame: Affine,
    pub half: V3,
    /// touching it throws an ape off with a boost
    pub bouncy: bool,
    /// the way its vines hang (its own "gravity")
    pub pull: V3,
}

/// The invisible edge: an ape further than `side` from the centreline
/// across, or lower than `below` / higher than `above` (heights, the
/// centreline being at 0) is out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Boundary {
    pub side: i32,
    pub below: i32,
    pub above: i32,
}

/// The shared door: a slab (`half`, about `frame.t`) that is solid while
/// shut. `shut_at` is the race tick it shut, or `None` while open; it
/// reopens `DOOR_SHUT` ticks later. The switch is vine `switch`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Door {
    pub frame: Affine,
    pub half: V3,
    pub switch: usize,
    pub shut_at: Option<u32>,
}

/// How long the door stays shut (ticks; a first guess).
pub const DOOR_SHUT: u32 = 5 * crate::rules::TICKS_PER_SECOND as u32;

impl Door {
    /// Whether the door is shut at race tick `tick`.
    pub fn shut(&self, tick: u32) -> bool {
        self.shut_at
            .is_some_and(|t| tick >= t && tick < t + DOOR_SHUT)
    }
}

/// A mech on its fixed path: built as `mech`, moving from `from` to `to`
/// and back over `period` ticks (or one way, then holding), facing `yaw`.
/// Its frame at a tick is `MechPath::frame`, shared by the physics and the
/// drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MechPath {
    pub mech: Mech,
    pub from: V3,
    pub to: V3,
    pub period: u32,
    pub yaw: i32,
    pub scale: i32,
}

impl MechPath {
    /// A stand-in: a grunt marching nowhere.
    pub fn still(at: V3, yaw: i32) -> MechPath {
        MechPath {
            mech: Mech {
                kind: MechKind::Grunt,
                gait: Gait::March,
                phase: 0,
            },
            from: at,
            to: at,
            period: 1,
            yaw,
            scale: ONE,
        }
    }
}

/// The wreck: its frame (as `spacecraft::pose`) and its passage's gap
/// (`spacecraft::wreck_passage`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wreck {
    pub frame: Affine,
    pub gap: (i32, i32),
}

/// Scenery that moves on its own clock (from the race tick).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetPiece {
    /// the mothership flies over from `from` to `to`, between race ticks
    /// `start` and `end`
    Pass {
        from: V3,
        to: V3,
        start: u32,
        end: u32,
    },
    /// a distant dogfight (`spacecraft::dogfight`)
    Dogfight { frame: Affine },
    /// a distant capital-ship battle (`spacecraft::capital_battle`)
    Battle { frame: Affine },
}

/// A route between a split and a join: which vines are on it, from and
/// to where along the centreline (metres), and whether it is the hidden
/// one (through the wreck) or the door's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub name: &'static str,
    pub from: i32,
    pub to: i32,
    pub vines: Vec<usize>,
    pub hidden: bool,
    pub door: bool,
}
