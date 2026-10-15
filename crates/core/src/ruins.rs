//! Sunset Ruins (2026-10-14, `briefs/sunset-ruins.md`): a jungle temple
//! map at sunset, in three thirds.
//!
//! **The light (this map's "time of day"):** a vibrant sunset, the sky
//! fading yellow → orange → red → crimson; darker inside the temple, lit by
//! banana glyphs in the stonework pulsing slowly; a soft spotlight from the
//! skylight on the finish pile. `ruinsscape::look()` is the start.
//!
//! - **Third 1, the river:** a downhill river with rapids (`River`); its
//!   current carries a swimming ape downstream at a modest speed. Snapping
//!   turtles swim across it as moving bouncers (`Turtle`, placed from the
//!   tick by `RuinsLayout::turtle_at`, the one function the physics and the
//!   drawing share).
//! - **The falls:** the river pours over a lip into a ravine (`Ravine`)
//!   that is not playable: an ape that falls below `Ravine.below` is put
//!   back at once on the riverbank before the rope bridge (`Ravine.spot`);
//!   with Floor is Lava, that is the kill. The rope bridge (`Bridge`, three
//!   gorillas wide) spans above the lip; its boosted vines fling an ape
//!   toward the temple's gaping mouth (`Mouth`), which pours its own
//!   waterfall into the ravine.
//! - **Third 2, the temple:** corridor sections (`Section`), floor spike
//!   traps (`SpikeTrap`: fling up, no grab for `SPIKE_NO_GRAB`), and trip
//!   wires that open trapdoors over pits of glowing banana spikes
//!   (`Trapdoor`). A trapdoor is shared: whoever trips its wire opens it for
//!   everyone `TRAP_DELAY` later, the pit vines drop from the ceiling while
//!   it is open, and it closes and re-arms `TRAP_OPEN` after opening. Its
//!   state lives on the course (`Trapdoor.tripped_at`), as Sector Ape's
//!   switch door does, so the apes, the AI's plans, replays (save points
//!   keep it) and every browser (a remote ape trips it from its reported
//!   state) see the same doors.
//! - **Third 3, the chamber:** an enormous round room (`Chamber`); the path
//!   turns right and runs counter-clockwise round it, the temple wall on
//!   the right and an invisible wall (`Chamber.inner`) on the left; near
//!   the full circle it turns to the centre and the apes swing to the
//!   finish at a pile of glowing banana statues (`Pile`) under a skylight
//!   (`Skylight`) hung with foliage and vines.
//!
//! The types below are the contract between the course (placement,
//! physics, the AI) and the scene (drawing): the course fills a
//! `RuinsLayout` on `Course.ruins`; the scene draws from it.
//!
//! The course is in `ruins/layout.rs`, the rules it adds (the current,
//! the turtles, the ravine, the bridge's boost, the spikes, the pits, the
//! temple's walls and ceilings) in `ruins/rules.rs`; every one is gated on
//! `Course.ruins`, so the other maps do not change.

use crate::fx::{self, cm, int, ONE};
use crate::geom::{v3, V3};
use crate::race::Race;
use crate::rules::TICKS_PER_SECOND;

pub use layout::{ruins, ruins_seeded};

mod layout;
mod rules;
#[cfg(test)]
mod tests;

/// How long after a trip wire is tripped its trapdoor opens (ticks): the
/// person's 0.75 s.
pub const TRAP_DELAY: u32 = TICKS_PER_SECOND as u32 * 3 / 4;

/// How long a trapdoor stays open, its pit vines down (ticks; about 8 s,
/// the person's answer). Then it closes and re-arms.
pub const TRAP_OPEN: u32 = 8 * TICKS_PER_SECOND as u32;

/// How long a spiked ape cannot grab (ticks; about 1 s).
pub const SPIKE_NO_GRAB: u32 = TICKS_PER_SECOND as u32;

/// A gorilla's width, arms in, as drawn in a race (measured from
/// `models`: shoulder joints 31 cm from the middle plus half a 16 cm arm,
/// each side, at the gorilla's 122% and `scene::RACE_SCALE` 300%: 2.86 m).
/// The bridge is three of these wide.
pub const GORILLA_WIDTH: i32 = cm(286);

/// The rope bridge's deck width: three gorillas.
pub const BRIDGE_WIDTH: i32 = 3 * GORILLA_WIDTH;

/// How close (flat) to a trip wire an ape's body must come, on the floor,
/// to trip it (a first guess).
pub const WIRE_REACH: i32 = cm(60);

/// Everything on Sunset Ruins that the Jungle's `Course` has no word for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuinsLayout {
    /// the river of the first third, upstream first
    pub river: River,
    /// the snapping turtles swimming across the river
    pub turtles: Vec<Turtle>,
    /// the ravine below the falls: where an ape is put back from
    pub ravine: Ravine,
    /// the waterfalls (scenery and sound only): the river's over the lip,
    /// the temple mouth's into the ravine
    pub falls: Vec<Falls>,
    /// the rope bridge above the falls
    pub bridge: Bridge,
    /// the temple's gaping mouth: the way in from the bridge launch
    pub mouth: Mouth,
    /// the temple corridor, entrance first
    pub corridor: Vec<Section>,
    /// floor spike traps in the corridor
    pub spikes: Vec<SpikeTrap>,
    /// trip wires with their trapdoors, pits and pit vines
    pub traps: Vec<Trapdoor>,
    /// the round inner chamber of the last third
    pub chamber: Chamber,
    /// the finish: a pile of glowing banana statues at the chamber's centre
    pub pile: Pile,
    /// the opening in the chamber's roof over the pile
    pub skylight: Skylight,
    /// the one-of-a-kind ruins along the way (scenery; solid if the course
    /// says so)
    pub landmarks: Vec<Landmark>,
    /// what each vine hangs from (one per `Course.vines`)
    pub hangers: Vec<Hanger>,
    /// where an ape stands behind the menus (the lobby zone): the hands'
    /// place and the way it faces
    pub lobby: (V3, i32),
    /// where each part of the course begins, metres along it
    pub marks: Marks,
}

/// Where each part of Sunset Ruins begins, metres along the course
/// (fixed point): the river runs from the start to `lip`, the ravine from
/// `lip` to `mouth`, the temple corridor from `mouth` to `chamber`, the
/// ring round the chamber from `chamber` to `turn_in`, and from `turn_in`
/// the path heads for the pile at the centre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Marks {
    pub lip: i32,
    pub mouth: i32,
    pub chamber: i32,
    pub turn_in: i32,
}

/// The river's fall: metres of drop a metre along it (6%).
pub const GRADE: i32 = ONE * 6 / 100;
/// The temple's floor is this far below the lip.
pub const TEMPLE_BELOW_LIP: i32 = int(6);
/// The ravine's floor is this far below the lip (no ape reaches it).
pub const RAVINE_DEPTH: i32 = int(90);
/// The river's half width; its banks run on to the corridor's walls.
pub const RIVER_HALF: i32 = int(14);
/// The current's speed downstream (slower than a swim, far slower than a
/// swing).
pub const CURRENT: i32 = int(3);
/// The temple corridor's half width and its ceiling's height.
pub const CORRIDOR_HALF: i32 = int(15);
pub const CORRIDOR_HEIGHT: i32 = int(32);
/// The chamber's roof above its floor, and its ring's half width.
pub const CHAMBER_HEIGHT: i32 = int(46);
pub const RING_HALF: i32 = int(16);
/// How hard a spike flings an ape up.
pub const SPIKE_FLING: i32 = int(14);
/// The bridge's boost: the least speed toward the mouth it flings an ape
/// at, and how fast toward the mouth a release must be going to get it.
pub const BOOST_SPEED: i32 = int(26);
pub const BOOST_DECENT: i32 = int(6);
/// A turtle's shell top above the water, and its radius as a bouncer.
pub const TURTLE_TOP: i32 = cm(60);
pub const TURTLE_RADIUS: i32 = cm(220);
/// No bouncer on Sunset Ruins stands taller than this above the land.
pub const BOUNCER_TALLEST: i32 = int(5);

/// The land's height `u` metres along the fall from the start: the
/// river's slope to the lip `lip`, the ravine's floor to the mouth
/// `mouth`, then the temple's floor.
pub fn height_at(u: i32, lip: i32, mouth: i32) -> i32 {
    let at_lip = -fx::mul(GRADE, lip);
    if u <= 0 {
        0
    } else if u < lip {
        -fx::mul(GRADE, u)
    } else if u < mouth {
        at_lip - RAVINE_DEPTH
    } else {
        at_lip - TEMPLE_BELOW_LIP
    }
}

/// The river: a chain of centreline points, upstream first. Between two
/// points the water runs straight; its surface height, half width and
/// current are lerped along the stretch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct River {
    pub points: Vec<RiverPoint>,
}

/// One point of the river's centreline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RiverPoint {
    /// on the centreline; `at.y` is the water's surface here
    pub at: V3,
    /// half the river's width here
    pub half_width: i32,
    /// the current's speed here, downstream (fixed point, m/s)
    pub current: i32,
    /// how rough the water is here, 0..ONE (rapids: drawing and sound only)
    pub rough: i32,
}

/// The river at a place: its surface height, its current (a velocity,
/// flat, downstream) and how rough it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Water {
    pub surface: i32,
    pub current: V3,
    pub rough: i32,
}

impl River {
    /// The water over (x, z), or `None` if (x, z) is not over the river.
    /// The nearest stretch wins.
    pub fn water_at(&self, x: i32, z: i32) -> Option<Water> {
        let p = v3(x, 0, z);
        let mut best: Option<(i64, Water)> = None;
        let last = self.points.len().saturating_sub(2);
        for (k, w) in self.points.windows(2).enumerate() {
            let (a, b) = (w[0], w[1]);
            // (quick reject: outside the stretch's box)
            let r = a.half_width.max(b.half_width);
            if x < a.at.x.min(b.at.x) - r
                || x > a.at.x.max(b.at.x) + r
                || z < a.at.z.min(b.at.z) - r
                || z > a.at.z.max(b.at.z) + r
            {
                continue;
            }
            let d = b.at.sub(a.at).flat();
            let len = d.len();
            if len == 0 {
                continue;
            }
            let dir = d.norm();
            let along = p.sub(a.at.flat()).dot(dir);
            // no water before the first point or past the last (the lip)
            if (k == 0 && along < 0) || (k == last && along > len) {
                continue;
            }
            let along = along.clamp(0, len);
            let t = fx::div(along, len);
            let mid = a.at.flat().add(dir.scale(along));
            let off = p.sub(mid).len_xz() as i64;
            let half = fx::lerp(a.half_width, b.half_width, t) as i64;
            if off > half || best.is_some_and(|(o, _)| o <= off) {
                continue;
            }
            let water = Water {
                surface: fx::lerp(a.at.y, b.at.y, t),
                current: dir.scale(fx::lerp(a.current, b.current, t)),
                rough: fx::lerp(a.rough, b.rough, t),
            };
            best = Some((off, water));
        }
        best.map(|(_, w)| w)
    }
}

/// A snapping turtle: swims from `from` to `to` and back (across the
/// river, on its surface) over `period` ticks, starting `phase` ticks into
/// its swim. Its shell is bouncer `bouncer`, an index in
/// `Course.mushrooms`, whose cap the course moves with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Turtle {
    pub from: V3,
    pub to: V3,
    pub period: u32,
    pub phase: u32,
    pub bouncer: usize,
}

impl RuinsLayout {
    /// Turtle `i` at race tick `tick`: where its shell's middle is and the
    /// way it faces (yaw). Shared by the physics and the drawing; eased at
    /// each bank, so it turns round there.
    pub fn turtle_at(&self, i: usize, tick: u32) -> (V3, i32) {
        let Some(t) = self.turtles.get(i) else {
            return (V3::ZERO, 0);
        };
        let period = t.period.max(1);
        let k = (tick.wrapping_add(t.phase)) % period;
        let phase = (k as i64 * fx::TURN as i64 / period as i64) as i32;
        let s = (ONE - fx::cos(phase)) / 2;
        let at = t.from.lerp(t.to, s);
        // the first half swims toward `to`, the second back
        let dir = if k < period / 2 {
            t.to.sub(t.from)
        } else {
            t.from.sub(t.to)
        };
        (at, crate::geom::yaw_of(dir.flat()))
    }

    /// Whether vine `i` is up out of reach at race tick `tick`: a pit vine
    /// whose trapdoor is shut. Every other vine is always down.
    pub fn vine_up(&self, i: usize, tick: u32) -> bool {
        self.traps
            .iter()
            .any(|t| t.vines.contains(&i) && !t.open(tick))
    }
}

/// The ravine below the falls: not playable. An ape in it lower than
/// `below` (a height) is put back at once at `spot`, facing `spot_yaw`.
/// `lip` is the waterfall's edge (two points across the river), `span` the
/// stretch of the course (metres along, fixed point) the ravine lies under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ravine {
    pub lip: (V3, V3),
    pub span: (i32, i32),
    pub below: i32,
    pub spot: V3,
    pub spot_yaw: i32,
}

impl Ravine {
    /// Whether an ape at `p`, `s` metres along the course, has fallen too
    /// deep into the ravine.
    pub fn fallen(&self, p: V3, s: i32) -> bool {
        s >= self.span.0 && s <= self.span.1 && p.y < self.below
    }
}

/// A waterfall sheet (scenery): water leaves the lip from `from` to `to`
/// (across) and falls `drop` metres.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Falls {
    pub from: V3,
    pub to: V3,
    pub drop: i32,
}

/// The rope bridge: its deck runs from `from` to `to` (each end's middle,
/// the deck's height in y), `width` wide (`BRIDGE_WIDTH`). Its boosted
/// vines (`boost`, indices in `Course.vines`) fling an ape toward `aim`
/// (the temple mouth) on release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bridge {
    pub from: V3,
    pub to: V3,
    pub width: i32,
    pub boost: Vec<usize>,
    pub aim: V3,
}

/// An oriented box: `half.x` across, `half.y` up, `half.z` along `fwd`
/// (flat, unit) about `centre`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slab {
    pub centre: V3,
    pub fwd: V3,
    pub half: V3,
}

impl Slab {
    /// Whether `p` is inside.
    pub fn contains(&self, p: V3) -> bool {
        let d = p.sub(self.centre);
        let across = crate::geom::right_of(self.fwd);
        d.dot(across).abs() <= self.half.x
            && d.y.abs() <= self.half.y
            && d.dot(self.fwd).abs() <= self.half.z
    }
}

/// The temple's mouth: `opening` is the gap an ape flies through (facing
/// out of the temple along `-opening.fwd`, `fwd` pointing in).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mouth {
    pub opening: Slab,
}

/// A straight stretch of temple corridor: its floor's middle line from
/// `from` to `to` (floor height in y), `half_width` either side, the
/// ceiling `height` above the floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Section {
    pub from: V3,
    pub to: V3,
    pub half_width: i32,
    pub height: i32,
}

/// A floor spike trap: an ape touching `area` is flung upward and cannot
/// grab for `SPIKE_NO_GRAB`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpikeTrap {
    pub area: Slab,
}

/// A trip wire and its trapdoor. The wire runs from `wire.0` to `wire.1`
/// just over the floor; tripping it opens `door` (the floor that drops
/// away) `TRAP_DELAY` later, showing a pit of glowing banana spikes whose
/// tips are at height `pit`; while open, vines `vines` (indices in
/// `Course.vines`, hanging from the ceiling into the pit) are down. It
/// closes and re-arms `TRAP_OPEN` after opening. `tripped_at` is the race
/// tick its wire was tripped, or `None` before.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trapdoor {
    pub wire: (V3, V3),
    pub door: Slab,
    pub pit: i32,
    pub vines: Vec<usize>,
    pub tripped_at: Option<u32>,
}

impl Trapdoor {
    /// Whether the floor is open (dropped away) at race tick `tick`.
    pub fn open(&self, tick: u32) -> bool {
        self.tripped_at
            .is_some_and(|t| tick >= t + TRAP_DELAY && tick < t + TRAP_DELAY + TRAP_OPEN)
    }

    /// Whether its wire can be tripped at race tick `tick`: never tripped,
    /// or closed again since.
    pub fn armed(&self, tick: u32) -> bool {
        self.tripped_at
            .is_none_or(|t| tick >= t + TRAP_DELAY + TRAP_OPEN)
    }

    /// Whether `p` (flat) is within `WIRE_REACH` of the wire.
    pub fn near_wire(&self, p: V3) -> bool {
        let (a, b) = (self.wire.0.flat(), self.wire.1.flat());
        let d = b.sub(a);
        let len = d.len();
        let q = p.flat();
        let nearest = if len == 0 {
            a
        } else {
            let dir = d.norm();
            a.add(dir.scale(q.sub(a).dot(dir).clamp(0, len)))
        };
        q.dist(nearest) <= WIRE_REACH
    }
}

/// The round inner chamber: the temple wall at `radius` from `centre`
/// (floor height in y), an invisible wall at `inner` keeping apes out of
/// the middle until the path turns in to the pile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chamber {
    pub centre: V3,
    pub radius: i32,
    pub inner: i32,
    /// the roof's height above the floor
    pub height: i32,
}

/// The finish pile of glowing banana statues: a mound `radius` across at
/// its foot and `height` tall about `centre` (on the floor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pile {
    pub centre: V3,
    pub radius: i32,
    pub height: i32,
}

/// The skylight: a round opening `radius` across in the chamber's roof,
/// its middle at `centre`; its light falls on the pile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Skylight {
    pub centre: V3,
    pub radius: i32,
}

/// A one-of-a-kind ruin: what it is, where it stands (its foot), the way
/// it faces and its size (ONE: as drawn).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Landmark {
    pub kind: LandmarkKind,
    pub at: V3,
    pub yaw: i32,
    pub scale: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LandmarkKind {
    /// a mossy stone statue of a bunch of bananas
    BananaBunch,
    /// a moai-style stone ape head
    Moai,
}

/// What a vine hangs from (one per vine).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hanger {
    /// a jungle tree's branch; `trunk` is the trunk's foot
    Tree { trunk: V3 },
    /// the rope bridge (a boosted vine if it is in `Bridge.boost`)
    Bridge,
    /// the temple corridor's or the chamber's ceiling
    Ceiling,
    /// the ceiling over trapdoor `trap`'s pit: down only while it is open
    Pit { trap: usize },
    /// the skylight's rim, hung with foliage
    Skylight,
    /// landmark `landmark` (`RuinsLayout.landmarks`)
    Ruin { landmark: usize },
    /// the temple mouth's carved lip
    Mouth,
}

impl Race {
    /// The trip wires: ape `i` on the floor near an armed wire trips it
    /// now. Runs for remote apes too, from their reported state, so every
    /// browser opens the same trapdoor on the same tick.
    pub(crate) fn trip_wires(&mut self, i: usize) {
        let tick = self.tick;
        let Some(r) = self.racers.get(i) else {
            return;
        };
        if r.ape.mode != crate::ape::Mode::Grounded {
            return;
        }
        let pos = r.ape.pos;
        let Some(s) = self.course.ruins.as_deref_mut() else {
            return;
        };
        for t in s.traps.iter_mut() {
            if t.armed(tick) && t.near_wire(pos) {
                t.tripped_at = Some(tick);
            }
        }
    }
}

/// The trapdoors' shared state: when each was last tripped (for save
/// points, which keep the course out of their copy).
pub fn trap_state(c: &crate::course::Course) -> Vec<Option<u32>> {
    c.ruins
        .as_ref()
        .map(|s| s.traps.iter().map(|t| t.tripped_at).collect())
        .unwrap_or_default()
}

/// Puts back the trapdoors' state from `trap_state`.
pub fn put_trap_state(c: &mut crate::course::Course, state: &[Option<u32>]) {
    if let Some(s) = c.ruins.as_deref_mut() {
        for (t, at) in s.traps.iter_mut().zip(state) {
            t.tripped_at = *at;
        }
    }
}

/// A trip wire across the course at `s` metres along (a helper for the
/// course and the tests): the wire `half` either side of the centreline.
pub fn wire_across(c: &crate::course::Course, s: i32, half: i32) -> (V3, V3) {
    let p = c.point(s);
    let across = crate::geom::right_of(c.heading_at(s)).scale(half);
    (p.sub(across), p.add(across))
}

/// The bridge's width is three gorillas (kept here so a change shows).
const _: () = assert!(BRIDGE_WIDTH == 3 * cm(286));
