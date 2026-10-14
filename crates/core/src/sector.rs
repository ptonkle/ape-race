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

use crate::fx::{self, deg, int, ONE};
use crate::geom::{v3, Affine, V3};
use crate::spacecraft::{Gait, Mech, MechAnchors, MechKind};

pub use layout::sector;

mod layout;
pub mod rules;

/// The pull toward the sun on this map (the other maps: `rules::GRAVITY`,
/// 17). A first guess; the course tunes it.
pub const SECTOR_GRAVITY: i32 = int(9);

/// Straight down: the way a vine hangs on every other map.
pub const DOWN: V3 = v3(0, -ONE, 0);

/// The hard cap on an ape's speed on this map (about 300 km/h; the other
/// maps top out near 60 m/s). The speedometer's scale reads to it.
pub const SECTOR_TOP: i32 = int(85);

/// The air's drag in open space, as a share of a kind's drag elsewhere.
pub const SPACE_DRAG: i32 = ONE * 2 / 5;

/// On an odd vine (one that does not hang straight down) the swing's drive
/// fades at this many times the kind's `drive_fade`, so a chain of odd
/// vines driven well reaches speeds no other map allows.
pub const ODD_FADE: i32 = ONE * 2;

/// Letting go of an odd vine adds up to this much speed along the motion
/// (the slingshot), the more the release lines up with the way on.
pub const SLING: i32 = int(10);

/// Touching a bouncy body reflects the velocity and adds this much along
/// the new velocity (never past `SECTOR_TOP`).
pub const REBOUND_BOOST: i32 = int(8);

/// The switch vine shuts the door once it is swung this far forward of
/// the way it hangs.
pub const DOOR_ANGLE: i32 = deg(55);

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
    /// the deck (an index in `Course.platforms`) just past the finish
    /// where finished apes come down and dance, in front of the black hole
    pub finish_deck: usize,
    /// the routes the track splits into (for the minimap and the AI)
    pub routes: Vec<Route>,
    /// physics only (the scene ignores it): for each stretch of
    /// `NEAR_SAMPLES` centreline samples, the solid bodies an ape in that
    /// stretch (anywhere inside the boundary) could touch
    pub near: Vec<Vec<u16>>,
}

/// Centreline samples to a stretch of `SectorLayout.near` (20 m).
pub const NEAR_SAMPLES: usize = 10;

impl SectorLayout {
    /// The solid bodies an ape whose nearest centreline sample is `hint`
    /// could touch.
    pub fn bodies_near(&self, hint: usize) -> &[u16] {
        self.near
            .get(hint / NEAR_SAMPLES)
            .map_or(&[][..], |v| &v[..])
    }

    /// Fills `near` from the bodies, the course's centreline and the
    /// boundary: a body is listed for a stretch if its ball comes within
    /// the boundary's reach of any sample of it.
    pub fn index_bodies(&mut self, path: &[V3]) {
        let n = path.len().div_ceil(NEAR_SAMPLES);
        let far = self
            .boundary
            .side
            .max(self.boundary.above.max(-self.boundary.below))
            + int(4);
        self.near = (0..n)
            .map(|k| {
                let lo = k * NEAR_SAMPLES;
                let hi = (lo + NEAR_SAMPLES).min(path.len());
                (0..self.bodies.len())
                    .filter(|&b| {
                        let body = &self.bodies[b];
                        body.solid()
                            && path[lo..hi]
                                .iter()
                                .any(|p| body.frame.t.dist(*p) <= body.reach() + far)
                    })
                    .map(|b| b as u16)
                    .collect()
            })
            .collect();
    }
}

/// Where a ball moving from `p0` to `p1` first touches a box (centre `c`,
/// unit axes `axes`, half sizes `half`), the ball's radius `r` added to
/// the box: the share of the way (Q16, 0 if it starts inside), the face's
/// outward normal, and (starting inside) how deep it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Touch {
    pub t: i32,
    pub normal: V3,
    pub depth: i32,
}

/// A swept test of a ball against a box (`Touch`): no tunnelling however
/// fast it moves.
pub fn sweep(c: V3, axes: &[V3; 3], half: V3, r: i32, p0: V3, p1: V3) -> Option<Touch> {
    let hs = [half.x + r, half.y + r, half.z + r];
    let d0 = p0.sub(c);
    let d1 = p1.sub(c);
    let s: [i32; 3] = [d0.dot(axes[0]), d0.dot(axes[1]), d0.dot(axes[2])];
    let e: [i32; 3] = [d1.dot(axes[0]), d1.dot(axes[1]), d1.dot(axes[2])];
    if (0..3).all(|k| s[k].abs() < hs[k]) {
        // inside: out through the nearest face
        let k = (0..3).min_by_key(|&k| hs[k] - s[k].abs()).unwrap_or(0);
        let sign = if s[k] >= 0 { ONE } else { -ONE };
        return Some(Touch {
            t: 0,
            normal: axes[k].scale(sign),
            depth: hs[k] - s[k].abs(),
        });
    }
    let (mut enter, mut exit, mut axis, mut sign) = (i64::MIN, i64::MAX, 0, ONE);
    for k in 0..3 {
        let d = (e[k] - s[k]) as i64;
        let (lo, hi) = (-hs[k] as i64 - s[k] as i64, hs[k] as i64 - s[k] as i64);
        if d == 0 {
            if lo > 0 || hi < 0 {
                return None;
            }
            continue;
        }
        // the share of the way at each plane, Q16
        let (mut t0, mut t1) = ((lo << 16) / d, (hi << 16) / d);
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }
        if t0 > enter {
            enter = t0;
            axis = k;
            sign = if d > 0 { -ONE } else { ONE };
        }
        exit = exit.min(t1);
        if enter > exit {
            return None;
        }
    }
    if enter < 0 || enter > ONE as i64 {
        return None;
    }
    Some(Touch {
        t: enter as i32,
        normal: axes[axis].scale(sign),
        depth: 0,
    })
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
    /// collision only, never drawn on its own: the walls of the wreck round
    /// its passage, and of the hulk round its corridor (those models are
    /// drawn whole, from `Wreck` and from the `Hulk` body)
    Hidden,
}

/// A solid piece in space. Its collision shape is the box `half` (world
/// metres along its own axes: `frame`'s columns, normalised) about
/// `frame.t`; `frame` places the model (rotation and scale, as
/// `spacecraft::pose`). A `Hulk` body is drawn only: its solid parts are
/// `Hidden` bodies round its corridor. A `Deck` body's top face is its
/// platform's top: an ape stands on it there (no bounce), and bounces off
/// its sides and underside.
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

impl Body {
    /// Its box's axes in the world (unit vectors).
    pub fn axes(&self) -> [V3; 3] {
        [
            self.frame.c0.norm(),
            self.frame.c1.norm(),
            self.frame.c2.norm(),
        ]
    }

    /// Whether it stops an ape (a hulk is drawn only).
    pub fn solid(&self) -> bool {
        self.model != Model::Hulk
    }

    /// The radius of a ball round `frame.t` that holds the whole box.
    pub fn reach(&self) -> i32 {
        self.half.len()
    }
}

/// The invisible edge: an ape further than `side` from the centreline
/// across, or lower than `below` / higher than `above` (heights, the
/// centreline being at 0), or more than `side` behind the start, is out.
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
    /// The mech's frame at race tick `tick` (as `spacecraft::pose`): it
    /// eases from `from` to `to` and back over `period` ticks, from the
    /// tick alone, so the physics and the drawing agree.
    pub fn frame(&self, tick: u32) -> Affine {
        let period = self.period.max(1);
        let phase = ((tick % period) as i64 * fx::TURN as i64 / period as i64) as i32;
        let t = (ONE - fx::cos(phase)) / 2;
        let at = self.from.lerp(self.to, t);
        crate::spacecraft::pose(at, self.yaw, 0, 0, self.scale)
    }

    /// Its vine anchors at race tick `tick`.
    pub fn anchors(&self, tick: u32) -> MechAnchors {
        crate::spacecraft::mech_anchors(&self.frame(tick), &self.mech, tick)
    }

    /// Where on it a vine hangs from, at race tick `tick`.
    pub fn point(&self, at: MechPoint, tick: u32) -> V3 {
        let a = self.anchors(tick);
        match at {
            MechPoint::Head => a.head,
            MechPoint::Hand(k) => a.hands[k as usize & 1],
            MechPoint::Elbow(k) => a.elbows[k as usize & 1],
            MechPoint::Shoulder(k) => a.shoulders[k as usize & 1],
        }
    }

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
