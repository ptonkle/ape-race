//! Banana Bay Battle (2026-10-15, `briefs/banana-bay-battle.md`): the
//! pirate map. A golden-age sea battle at 3 pm in a vaguely Caribbean bay:
//! pirate ships and a nonspecific navy lie still at anchor, firing at each
//! other, and the apes swing across about 20 of them on ropes hung from
//! masts and yards, from a dock at the edge of the bay to a treasure island.
//!
//! **The light (this map's "time of day"):** 3 pm, a high warm sun, a
//! bright blue sky, turquoise water. `bayscape::look()` is the start.
//!
//! - **The start:** a wooden pier at the bay's edge (`Dock`), the battle
//!   out ahead.
//! - **The ships** (`Ship`): at least three kinds and sizes
//!   (`ShipKind`: galleon, frigate, brig, sloop), pirate or navy (`Side`;
//!   plain flags, no real nation), scaled so walking a big one end to end
//!   takes about 10 s. They never move, not even with the swell. Their
//!   decks are `Course.platforms` (`Ship.decks`): you land on them and run
//!   along them.
//! - **Rigging** (`Mast`, `Yard`, `Sail`): every vine hangs from a mast, a
//!   yard or other upright part (`Hanger`). Where the course swings
//!   through a ship, its sails there are missing or torn
//!   (`Sail::clear`), so a swinging ape never passes through canvas.
//! - **The water** (`Water`): a sea level with a deterministic up-and-down
//!   swell from the place and the tick (`Water::height`), the one function
//!   the physics and the drawing share.
//! - **Barrels:** flagged barrels float along both edges of the course
//!   (`BoundaryBarrel`) and mark the soft wall; bouncy barrels float inside
//!   it (`BouncyBarrel`), each a bouncer in `Course.mushrooms` like every
//!   other bouncer.
//! - **Scenery only:** the cannon fire between ships (`Volley`, from the
//!   tick: `BayLayout::ball_at`) and the low-detail monkey pirates and
//!   sailors fighting on the decks (`Fighter`). Nothing in either touches
//!   a racer.
//! - **The finish:** a treasure island (`Island`) with a big chest
//!   overflowing with oversized, jumbled golden bananas.
//!
//! **Rules the course agent owns** (named here, none of them built yet):
//!
//! - *Water reset:* an ape that falls into the water (below
//!   `Water::height` at its place) is put back on the deck of the last ship
//!   it was over (`BayLayout::ship_under`, remembered per racer); with
//!   Floor is Lava, the water is the floor (`Course::floor`) and that is
//!   the kill. Save points and replays must keep the "last ship" memory.
//! - *Barrel soft wall:* the boundary barrels mark the course's sides
//!   (`Course.half_width` there); a drifting ape is pushed back in like the
//!   side limits on the other maps (`Course::contain`). You cannot land on
//!   a boundary barrel.
//! - *Bouncy barrels:* ordinary `Mushroom` bounces; the physics' bouncer
//!   top is fixed (`Course.mushrooms[b].top`); the drawing may bob the
//!   barrel by at most `BOB_MAX` round it.
//!
//! The types below are the contract between the course (placement,
//! physics, the AI) and the scene (drawing, `bayscape.rs` and
//! `baymodels.rs`): the course fills a `BayLayout` on `Course.bay`; the
//! scene draws from it. Positions are world space, fixed point
//! (`fx::ONE` = 1 m); yaws are `fx::TURN` units, 0 facing +z (as
//! `geom::yaw_of`).
//!
//! STAND-IN (the skeleton, 2026-10-15): `bay()` is the Jungle's layout
//! under this map's name with an empty layout; the course agent replaces
//! it.

use crate::fx::{self, cm, int, ONE, TURN};
use crate::geom::{right_of, v3, V3};
use crate::rules::TICKS_PER_SECOND;

pub use layout::{
    bay, bay_seeded, BARREL_RADIUS, BARREL_TOP, DOCK_TOP, ISLAND_TOP, RESET_AFT, SHIPS,
    SHIP_OFFSET, WATER_LEVEL,
};

mod layout;
mod rules;
#[cfg(test)]
mod tests;

/// The furthest the drawing may bob a bouncy barrel from its fixed bouncer
/// top (the physics never bobs it).
pub const BOB_MAX: i32 = cm(15);

/// How long one round of the cannon-fire schedule lasts (ticks; 20 s). The
/// volleys repeat every `VOLLEY_CYCLE` ticks.
pub const VOLLEY_CYCLE: u32 = 20 * TICKS_PER_SECOND as u32;

/// Everything on Banana Bay Battle that the Jungle's `Course` has no word
/// for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BayLayout {
    /// the still ships of the battle, start first (roughly in course
    /// order); `Hanger`s, `Volley`s and `Fighter`s refer to them by index
    pub ships: Vec<Ship>,
    /// what each vine hangs from (exactly one per `Course.vines`, same
    /// order)
    pub hangers: Vec<Hanger>,
    /// the flagged barrels floating along both edges: the soft wall's
    /// markers (scenery; the wall itself is `Course::contain`)
    pub boundary: Vec<BoundaryBarrel>,
    /// the bouncy barrels floating inside the course (each also a
    /// `Course.mushrooms` entry)
    pub bouncy: Vec<BouncyBarrel>,
    /// the sea: its level and swell
    pub water: Water,
    /// the start: a wooden pier at the bay's edge
    pub dock: Dock,
    /// the finish: the treasure island and its chest
    pub island: Island,
    /// the cannon-fire schedule (scenery only), repeating every
    /// `VOLLEY_CYCLE` ticks
    pub volleys: Vec<Volley>,
    /// the low-detail deck fighters (scenery only)
    pub fighters: Vec<Fighter>,
    /// where an ape stands behind the menus (the lobby zone): the hands'
    /// place and the way it faces
    pub lobby: (V3, i32),
}

/// Whose side a ship (or a deck fighter) is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// the pirates: black or red flags, patched hulls
    Pirate,
    /// a nonspecific navy: plain flags, no real nation's insignia
    Navy,
}

/// The kinds of ship, biggest first. Typical sizes (the course may vary
/// them): a galleon about 45 m long (walking end to end about 10 s), a
/// frigate about 38 m, a brig about 28 m, a sloop about 20 m.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShipKind {
    /// three masts, square-rigged, a high castled stern (quarterdeck and
    /// poop) and forecastle; two gun decks
    Galleon,
    /// three masts, square-rigged, one long flush gun deck, a low
    /// quarterdeck
    Frigate,
    /// two masts, square-rigged, one gun deck
    Brig,
    /// one mast, fore-and-aft rigged (a gaff mainsail and a jib); a few
    /// deck guns: the small ships that fill the gaps between big ones
    Sloop,
}

/// One still ship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ship {
    pub kind: ShipKind,
    pub side: Side,
    /// the middle of the hull at the waterline (`at.y` is `Water.level`):
    /// the ship's origin; `along`/`across`/`up` offsets below are from here
    pub at: V3,
    /// the way the bow points
    pub yaw: i32,
    /// the hull's length, stern to stem (bowsprit not counted)
    pub length: i32,
    /// the hull's greatest width
    pub beam: i32,
    /// the main (weather) deck's height above the waterline
    pub deck: i32,
    /// the raised decks, as (from, to, height): `from`/`to` along the
    /// ship (negative aft), `height` above the waterline; e.g. a galleon's
    /// quarterdeck and forecastle. Empty for a flush deck.
    pub raised: Vec<(i32, i32, i32)>,
    /// the bowsprit's length past the stem (0 for none)
    pub bowsprit: i32,
    /// the masts, fore first
    pub masts: Vec<Mast>,
    /// the jib (the triangular headsail from the foremast to the
    /// bowsprit); `None` if the ship has none rigged
    pub jib: Option<Sail>,
    /// the cannon ports, along both sides
    pub ports: Vec<Port>,
    /// the indices in `Course.platforms` of this ship's walkable decks
    /// (main deck first, then raised decks): what an ape lands and runs on,
    /// and where the water reset puts it back
    pub decks: Vec<usize>,
}

/// A mast, standing on the centreline of the ship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mast {
    /// where it stands along the ship from `Ship.at` (+ toward the bow)
    pub along: i32,
    /// its top's height above the main deck
    pub height: i32,
    /// the yards across it, lowest first (square-rigged masts; empty on a
    /// sloop's)
    pub yards: Vec<Yard>,
    /// the fore-and-aft (gaff) sail aft of the mast: a sloop's mainsail, a
    /// galleon's or frigate's mizzen spanker; `None` if none is rigged
    pub gaff: Option<Sail>,
}

/// A yard: a spar across a mast that a square sail hangs from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Yard {
    /// its height above the main deck
    pub height: i32,
    /// half its span, port to starboard
    pub half_span: i32,
    /// the square sail below it (down to the next yard below, or to the
    /// deck for the lowest yard)
    pub sail: Sail,
}

/// The state of one sail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sail {
    /// full canvas: solid in the drawing, so the course must never swing
    /// through it
    Set,
    /// furled (rolled up on its spar): the space below is clear
    Furled,
    /// shot to rags: a few strips hang from the spar, the space below is
    /// clear for a swing (the strips are drawn short enough not to touch
    /// a swinging ape)
    Torn,
    /// gone (the spar may be bare or broken): clear
    Missing,
}

impl Sail {
    /// Whether an ape may pass through where this sail would hang.
    pub fn clear(self) -> bool {
        self != Sail::Set
    }
}

/// A cannon port in a hull side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port {
    /// where it is along the ship from `Ship.at` (+ toward the bow)
    pub along: i32,
    /// which side: +1 starboard (the ship's right), -1 larboard (its left)
    pub side: i32,
    /// its height above the waterline
    pub up: i32,
}

/// What a vine hangs from (one per `Course.vines`). The vine's anchor is
/// the world point of the hanger; the art draws the rope from there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hanger {
    /// yard `yard` of mast `mast` of ship `ship`, `out` from the mast
    /// along the yard (+ starboard)
    Yard {
        ship: usize,
        mast: usize,
        yard: usize,
        out: i32,
    },
    /// mast `mast` of ship `ship` itself (a top, a crosstree or the gaff's
    /// jaws), `up` above the main deck
    Mast { ship: usize, mast: usize, up: i32 },
    /// ship `ship`'s bowsprit, `out` past the stem
    Bowsprit { ship: usize, out: i32 },
    /// a palm on the island, the dock's crane or other shore post; `foot`
    /// is the post's foot (STAND-IN: every vine of the skeleton hangs from
    /// one of these, over the Jungle's trees)
    Shore { foot: V3 },
}

/// A flagged barrel on the course's edge: a soft-wall marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundaryBarrel {
    /// its middle at the sea level (`at.y` is `Water.level`; the drawing
    /// adds the swell)
    pub at: V3,
    /// which edge: +1 the course's right, -1 its left (flags may be
    /// coloured by edge)
    pub edge: i32,
}

/// A bouncy barrel floating inside the course: a bouncer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BouncyBarrel {
    /// the index of its bouncer in `Course.mushrooms`: the physics' top and
    /// radius
    pub bouncer: usize,
    /// a lean or turn for the drawing, so they are not all alike
    pub yaw: i32,
}

/// The sea.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Water {
    /// the still water's height
    pub level: i32,
    /// the swells summed on it (up and down only; nothing drifts)
    pub waves: Vec<Wave>,
}

/// One swell: a sine in space and time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wave {
    /// its height (half crest to trough)
    pub amp: i32,
    /// its direction across the water (flat, unit length)
    pub dir: V3,
    /// crest to crest
    pub length: i32,
    /// how long it takes to rise and fall once (ticks)
    pub period: u32,
}

impl Water {
    /// The water's surface height at (x, z) at `tick`: the level plus the
    /// swells. Deterministic from place and tick: the physics and the
    /// drawing both use it.
    pub fn height(&self, x: i32, z: i32, tick: u32) -> i32 {
        self.level + self.swell(x, z, tick)
    }

    /// The swell alone (the height above or below `level`).
    pub fn swell(&self, x: i32, z: i32, tick: u32) -> i32 {
        let mut sum = 0;
        for w in &self.waves {
            if w.length <= 0 || w.period == 0 {
                continue;
            }
            // the place's share of a wavelength, and the tick's of a period,
            // in turns (i64: no overflow over a long course or race)
            let d = (x as i64 * w.dir.x as i64 + z as i64 * w.dir.z as i64) >> fx::FRAC_BITS;
            let space = (d * TURN as i64 / w.length as i64) as i32;
            let beat = ((tick % w.period) as i64 * TURN as i64 / w.period as i64) as i32;
            sum += fx::mul(w.amp, fx::sin(space.wrapping_sub(beat)));
        }
        sum
    }

    /// The highest the swell can raise the water above `level`.
    pub fn peak(&self) -> i32 {
        self.waves.iter().map(|w| w.amp.abs()).sum()
    }
}

/// The start: a wooden pier out into the bay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dock {
    /// the landward end of the pier, on its centreline, at deck height
    pub from: V3,
    /// the seaward end, at deck height
    pub to: V3,
    /// half its width
    pub half_width: i32,
    /// its index in `Course.platforms` (the start platform)
    pub platform: usize,
}

/// The finish: a treasure island.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Island {
    /// the middle of the island at the beach's top (`centre.y`)
    pub centre: V3,
    /// the beach's radius at the waterline
    pub radius: i32,
    /// the chest's place (the middle of its base) and the way it faces
    pub chest: V3,
    pub chest_yaw: i32,
    /// the chest's size: an oversized chest, overflowing with oversized,
    /// jumbled golden bananas (like the Ruins' pile)
    pub chest_size: V3,
}

/// One shot in the cannon-fire schedule (scenery only: nothing hits a
/// racer). Fired at `fire` ticks into every `VOLLEY_CYCLE`, it flies for
/// `flight` ticks from port `port` of ship `from` to `target`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Volley {
    /// the firing ship and its port
    pub from: usize,
    pub port: usize,
    /// the ship it is fired at
    pub at: usize,
    /// where the ball lands: on ship `at`'s hull if `hit` (smoke and
    /// splinters), else in the water near it (a splash)
    pub target: V3,
    pub hit: bool,
    /// when it fires, ticks into the cycle
    pub fire: u32,
    /// how long the ball flies (ticks)
    pub flight: u32,
}

/// A low-detail monkey fighting on a deck (scenery only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fighter {
    /// the ship it stands on
    pub ship: usize,
    /// its feet, on that ship's deck (world space)
    pub at: V3,
    pub yaw: i32,
    /// pirate or sailor
    pub side: Side,
    /// the fighter it duels, if any (an index in `BayLayout.fighters`)
    pub foe: Option<usize>,
    /// a phase offset for its loop (ticks), so they don't all swing at once
    pub phase: u32,
}

impl Ship {
    /// The ship's bow direction (flat, unit).
    pub fn fwd(&self) -> V3 {
        v3(fx::sin(self.yaw), 0, fx::cos(self.yaw))
    }

    /// The ship's starboard (right) direction (flat, unit).
    pub fn right(&self) -> V3 {
        right_of(self.fwd())
    }

    /// The world point `along` the ship (+ bow), `across` it (+ starboard)
    /// and `up` from the waterline.
    pub fn point(&self, along: i32, across: i32, up: i32) -> V3 {
        self.at
            .add(self.fwd().scale(along))
            .add(self.right().scale(across))
            .add(v3(0, up, 0))
    }

    /// The top of mast `m`.
    pub fn mast_top(&self, m: usize) -> V3 {
        let mast = &self.masts[m];
        self.point(mast.along, 0, self.deck + mast.height)
    }

    /// The two ends of yard `y` of mast `m`, larboard first.
    pub fn yard_ends(&self, m: usize, y: usize) -> (V3, V3) {
        let mast = &self.masts[m];
        let yard = mast.yards[y];
        let up = self.deck + yard.height;
        (
            self.point(mast.along, -yard.half_span, up),
            self.point(mast.along, yard.half_span, up),
        )
    }

    /// Whether the flat point (x, z) is over the hull (its rough outline: a
    /// box `length` by `beam`). The water reset takes the last ship an ape
    /// was over.
    pub fn over(&self, x: i32, z: i32) -> bool {
        let d = v3(x, 0, z).sub(self.at.flat());
        d.dot(self.fwd()).abs() <= self.length / 2 && d.dot(self.right()).abs() <= self.beam / 2
    }
}

impl Hanger {
    /// The hanger's world point (where its rope is tied), or `None` for a
    /// hanger that names a ship, mast or yard the layout doesn't have.
    pub fn point(&self, b: &BayLayout) -> Option<V3> {
        match *self {
            Hanger::Yard {
                ship,
                mast,
                yard,
                out,
            } => {
                let s = b.ships.get(ship)?;
                let m = s.masts.get(mast)?;
                let y = m.yards.get(yard)?;
                Some(s.point(m.along, out, s.deck + y.height))
            }
            Hanger::Mast { ship, mast, up } => {
                let s = b.ships.get(ship)?;
                let m = s.masts.get(mast)?;
                Some(s.point(m.along, 0, s.deck + up))
            }
            Hanger::Bowsprit { ship, out } => {
                let s = b.ships.get(ship)?;
                Some(s.point(s.length / 2 + out, 0, s.deck))
            }
            Hanger::Shore { foot } => Some(foot),
        }
    }
}

impl BayLayout {
    /// The ship whose hull is under the flat point (x, z), if any (the
    /// first, if hulls overlap).
    pub fn ship_under(&self, x: i32, z: i32) -> Option<usize> {
        self.ships.iter().position(|s| s.over(x, z))
    }

    /// Where volley `i`'s ball is at `tick`, if it is in the air: a
    /// parabola from the port to the target whose middle is `ARC` above
    /// the straight line. Deterministic from the tick. It fires (muzzle
    /// smoke) when it leaves the port and lands (splash or hit) at
    /// `fire + flight`.
    pub fn ball_at(&self, i: usize, tick: u32) -> Option<V3> {
        let v = self.volleys.get(i)?;
        let s = self.ships.get(v.from)?;
        let p = s.ports.get(v.port)?;
        let into = (tick % VOLLEY_CYCLE + VOLLEY_CYCLE - v.fire % VOLLEY_CYCLE) % VOLLEY_CYCLE;
        if v.flight == 0 || into >= v.flight {
            return None;
        }
        let start = s.point(p.along, p.side * s.beam / 2, p.up);
        let t = (into as i64 * ONE as i64 / v.flight as i64) as i32;
        let line = start.add(v.target.sub(start).scale(t));
        // 4·t·(1−t): 0 at both ends, 1 at the middle
        let lift = fx::mul(4 * t, ONE - t);
        Some(line.add(v3(0, fx::mul(ARC, lift), 0)))
    }
}

/// How high a cannonball's arc rises above the straight line from port to
/// target, at its middle.
pub const ARC: i32 = int(4);
