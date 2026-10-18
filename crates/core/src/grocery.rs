//! Clean-Up on Aisle Ape (2026-10-18, `briefs/clean-up-on-aisle-ape.md`):
//! food apes race through a supermarket laid out from a Food Lion floor
//! plan, from the stockroom doors at the back to the exit doors at the
//! front, down three of its thirteen aisles. Pitfalls every agent checks:
//! `notes/grocery-pitfalls.md`.
//!
//! **Scale:** one real metre is `SCALE` (12) game metres, so an ape (3.3 m
//! drawn) is a 27 cm snack, a shopper is 20 m tall, a shelf 23 m and the
//! ceiling 52 m. The aisles are widened to 38 m for swinging.
//!
//! **The store's physics is the Ravine's.** Everything solid is written as
//! Rattlesnake Ravine's layout (`Course.ravine`), whose rules, solids, AI
//! wall checks and camera keeps then apply unchanged: the gondolas, freezer
//! runs, store walls and counters are its `walls` (a wall's solid stands
//! `WALL_ABOVE` over its drawn top, so a gondola drawn with its top stock to
//! `STOCK_TOP` is solid to the ceiling and each aisle is sealed); the ceiling
//! is one `Arch` room over the whole store (`rooms[0]`, also the camera's
//! keep-inside volume); columns, fruit tables, display bins, the checkout
//! counters and the coffin freezers are its `rocks` (upright posts whose tops
//! are floors); and each shopper's head and shoulders are a `Lizard`, a
//! bouncer that walks to and fro. The layout here (`GroceryLayout`, on
//! `Course.grocery`) is what the Ravine has no word for: what each fixture
//! is, the shelves' faces, the shoppers and their balloons, the misters. The
//! scene draws the store from both, and never draws the Ravine's art on it.
//!
//! **Swinging from people:** a shopper with balloons holds a vine (its
//! string) whose anchor, the balloons, walks with them: a sliding vine
//! (`Vine.slide`) phased to match the shopper's lizard (`balloon_phase`),
//! so the physics, the AI and the drawing agree on where the person is.
//!
//! Everything is a pure function of the tick and the layout: nothing is
//! random at runtime.

pub mod layout;
pub mod list;
#[cfg(test)]
mod tests;

pub use layout::{grocery, grocery_seeded};

use crate::fx::{self, cm, int, ONE, TURN};
use crate::geom::{v3, V3};

/// Game metres per real metre.
pub const SCALE: i32 = 12;
/// The ceiling's underside over the floor (4.3 m real).
pub const CEILING_AT: i32 = int(52);
/// A gondola's shelving (1.9 m real) ...
pub const SHELF_TOP: i32 = int(23);
/// ... and the cases of top stock stacked on it: the drawn top, so that the
/// wall's solid (`WALL_ABOVE` over it) reaches the ceiling.
pub const STOCK_TOP: i32 = int(42);
/// A shopper's height (1.7 m real).
pub const SHOPPER_HEIGHT: i32 = cm(2040);

/// A length in real centimetres, as game metres.
pub const fn real_cm(c: i32) -> i32 {
    cm(c * SCALE)
}

/// What a gondola is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Run {
    /// shelving, products on both faces
    Shelves,
    /// a run of glass-door freezers, both faces
    Freezer,
}

/// A gondola or freezer run: its middle line on the floor from its front
/// end through its bowed middle to its back end (`line`), `half` either
/// side, drawn to `STOCK_TOP`. `faces` are the aisles (catalogue indices,
/// `grocerymodels::AISLES`) its left and right faces sell, looking from
/// its front end to its back. Solid as its wall, `wall` in `RavineLayout.walls`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gondola {
    pub run: Run,
    pub line: [V3; 3],
    pub half: i32,
    pub faces: [u8; 2],
    pub wall: usize,
}

/// A department's cases along a store wall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dept {
    Dairy,
    Meat,
    Deli,
    Bakery,
    /// produce's misted wet rack
    WetRack,
}

/// A stock cart loaded with cases parked across a closed aisle's mouth (a
/// wet-floor sign beside it: a clean-up on that aisle): `from` and `to`
/// its face toward the cross aisle on the floor, `out` toward it. Solid as
/// its wall, `wall` in `RavineLayout.walls`, `BLOCKER_TOP` tall (so its
/// solid stands 10 m higher: a runner cannot get into the aisle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blocker {
    pub from: V3,
    pub to: V3,
    pub out: V3,
    pub wall: usize,
    /// its drawn top: `BLOCKER_TOP`, or at an item aisle's pocket (GR-T2)
    /// cases stacked to `STOCK_TOP`, so it is solid to the ceiling and
    /// nothing flies over into the closed aisle
    pub top: i32,
}

/// A blocker's drawn top: a stock cart and its cases (0.75 m real).
pub const BLOCKER_TOP: i32 = int(9);

/// A run of cases along a store wall, from `from` to `to` (the cases' front
/// on the floor), `out` into the store (flat, unit). The store's wall is
/// solid from the cases' front (`RavineLayout.walls`); the cases and the
/// soffit over them are drawn flush with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cases {
    pub dept: Dept,
    pub from: V3,
    pub to: V3,
    pub out: V3,
}

/// What a post in `RavineLayout.rocks` is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostKind {
    /// a white column, floor to ceiling
    Column,
    /// a round produce table, fruit piled on it
    Table,
    /// a kraft cardboard display bin of product
    Bin,
    /// a stretch of a low coffin freezer in the frozen aisle
    Coffin,
    /// a stretch of a checkout counter
    Counter,
    /// a pole outdoors: the entrance canopy's, a light pole's (drawn with
    /// the lot, GR-T4)
    Pole,
}

/// One of `RavineLayout.rocks`: what it is drawn as, and (for the tables
/// and bins) which produce or product (`stock`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Post {
    pub rock: usize,
    pub kind: PostKind,
    pub stock: u8,
}

/// A checkout lane: its counter's middle on the floor and the way a
/// customer walks through it (yaw). The counter is solid as its posts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lane {
    pub at: V3,
    pub yaw: i32,
}

/// A shopper: walks with lizard `lizard` (`RavineLayout::lizard_at`, their
/// feet; their head's top is the bouncer). With balloons, vine `balloons`
/// is the string (a sliding vine, `balloon_phase`). `look` picks clothes,
/// skin and hair; `cart` whether they push a cart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shopper {
    pub lizard: usize,
    pub balloons: Option<usize>,
    pub cart: bool,
    pub look: u8,
}

/// A produce table's mister: puffs every `period` ticks, `phase` in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mister {
    pub at: V3,
    pub period: u32,
    pub phase: u32,
}

/// The shopping list (GR-T2): one item per zone of the store, each in a
/// side aisle the route passes. ...
pub const LIST_ZONES: usize = 4;
/// ... and the spots in each item aisle (GR-T3): racer `k` fetches from
/// spot `k % LIST_SLOTS`, so in a race of four each is led to its own part
/// of the aisle, all as far along.
pub const LIST_SLOTS: usize = 4;
/// An ape takes its item when its middle comes within this of the item's
/// middle (plus its body's radius), or when it grabs the item's string.
pub const ITEM_REACH: i32 = int(7);
/// The first and third items an ape takes each add this share of its
/// kind's top speed, the second and fourth this share of its air
/// steering, added and not compounded (GR-T2: with all four, 1.5 times
/// each).
pub const ITEM_GAIN: i32 = ONE / 4;

/// A shopping-list item: zone `zone`'s (0..`LIST_ZONES`, in the route's
/// order) for spot `slot`, its middle at `at`, hanging on vine `vine` (its
/// string, from the ceiling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListItem {
    pub zone: u8,
    pub slot: u8,
    pub at: V3,
    pub vine: usize,
}

/// A vine that does not hang straight down (GR-T8), for the art.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OddKind {
    /// a licorice rope standing out level from a hanging sign's arm
    Sideways,
    /// a balloon string rising from a weight (its anchor) to the balloons
    Upward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OddVine {
    pub vine: usize,
    pub kind: OddKind,
}

/// The drop ceiling over part of the frozen aisle (GR-T7): its underside
/// `under` over the floor, a box along the aisle from `from` to `to`,
/// `half_width` either side. Solid from `under` to the store's ceiling (as
/// `RavineLayout.rooms` stacked; the vines under it hang from it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LowCeiling {
    pub from: V3,
    pub to: V3,
    pub half_width: i32,
    pub under: i32,
}

/// The bagged-ice pallets stacked across the frozen aisle past the drop
/// ceiling (GR-T7: up and over): `from` to `to` its middle line on the
/// floor, `half` thick, drawn `top` tall. Solid as its wall, `wall` in
/// `RavineLayout.walls`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IceWall {
    pub from: V3,
    pub to: V3,
    pub half: i32,
    pub top: i32,
    pub wall: usize,
}

/// A parked car in the lot (GR-T4): its middle on the ground, facing
/// `yaw`, `look` its paint. Solid as its wall, `wall` in `RavineLayout.walls`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Car {
    pub centre: V3,
    pub yaw: i32,
    pub look: u8,
    pub wall: usize,
}

/// A parked car's size, game metres: 4.5 × 1.8 × 1.45 m real (length,
/// width, height).
pub const CAR_SIZE: (i32, i32, i32) = (int(54), cm(2160), cm(1740));

/// A parking lot light pole: its foot, its height (the arm's), and the
/// arm's end (where its vine hangs from).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lamp {
    pub foot: V3,
    pub height: i32,
    pub arm: V3,
}

/// The parking lot (GR-T4) out of the exit doors: the entrance canopy
/// (from the store's front out to `canopy.1`, its underside `canopy_under`
/// over the ground, `canopy.0` to `canopy.1` its corners), the cars, the
/// light poles, the cart corrals (each `from` to `to`, its rails), and its
/// corners on the ground (least x and z, most).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lot {
    pub canopy: (V3, V3),
    pub canopy_under: i32,
    pub cars: Vec<Car>,
    pub lamps: Vec<Lamp>,
    pub corrals: Vec<(V3, V3)>,
    pub bounds: (V3, V3),
}

/// The exit doors' opening, across (GR-T4: the route goes out through it).
pub const DOOR_WIDTH: i32 = int(36);
/// ... and up to the frame over it.
pub const DOOR_HEIGHT: i32 = int(34);

/// The plane that picks up an ape's order (GR-T5) when it has finished
/// with all four items: it comes this many ticks after the finish ...
pub const PLANE_AFTER: u32 = 120;
/// ... and is gone this many after that.
pub const PLANE_TICKS: u32 = 900;

/// Clean-Up on Aisle Ape's own layout: what the store's solids are, and
/// the things the Ravine's layout has no word for (see the module's notes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroceryLayout {
    pub gondolas: Vec<Gondola>,
    pub cases: Vec<Cases>,
    pub blockers: Vec<Blocker>,
    pub posts: Vec<Post>,
    pub lanes: Vec<Lane>,
    pub shoppers: Vec<Shopper>,
    pub misters: Vec<Mister>,
    /// the store's inside corners on the floor (least x and z, most)
    pub bounds: (V3, V3),
    /// the exit doors' middle (in the front wall) and the way out
    pub doors: (V3, V3),
    /// the lobby spot (where the ape stands behind the menus) and its yaw
    pub lobby: (V3, i32),
    /// the shopping list's items (GR-T2), zone by zone, spot by spot
    pub items: Vec<ListItem>,
    /// for each vine: the unit way it hangs from its anchor (`sector::DOWN`
    /// for most; `Course::hang`)
    pub hangs: Vec<V3>,
    /// the vines that do not hang down (GR-T8)
    pub odd: Vec<OddVine>,
    /// the drop ceiling (GR-T7) and the ice past it
    pub low: Vec<LowCeiling>,
    pub ice: Vec<IceWall>,
    /// the parking lot (GR-T4)
    pub lot: Lot,
}

impl GroceryLayout {
    /// Spot `slot`'s item in zone `zone`.
    pub fn item(&self, zone: usize, slot: usize) -> Option<&ListItem> {
        self.items
            .iter()
            .find(|it| it.zone as usize == zone && it.slot as usize == slot)
    }

    /// Whether `p` is out of the store (in the parking lot).
    pub fn outside(&self, p: V3) -> bool {
        let (_, z) = plan_of(p);
        z < int(layout::SOUTH)
    }
}

/// The slide phase (`Vine.slide`) at which balloon vine `i`'s anchor keeps
/// step with a shopper lizard walking `period` ticks there and back: the
/// lizard is at `from + (to - from)(1 - cos φ)/2` with `φ` its phase, the
/// vine's anchor at `mid + reach sin θ`, so `θ = φ - TURN/4`. `Course::anchor`
/// sets `θ = tick·TURN/period + i·9001`, so the lizard's `phase` (ticks)
/// is `(i·9001 + TURN/4) · period / TURN`, folded into one turn.
pub fn balloon_phase(i: usize, period: u32) -> u32 {
    let a = ((i as i64 * 9_001 + (TURN / 4) as i64).rem_euclid(TURN as i64)) as u64;
    (a * period as u64 / TURN as u64) as u32
}

/// The plan's metres (east `x`, north `z`, the floor plan's frame, its
/// front-left inside corner at 0) as the course's frame, whose origin is
/// the start (`layout::START`) and whose +z is east: plan north is the
/// course's +x (left of east), so the plan and the course turn alike.
pub fn plan(x: i32, z: i32) -> V3 {
    v3(z - layout::START.1, 0, x - layout::START.0)
}

/// The plan's metres of a course point (the inverse of `plan`).
pub fn plan_of(p: V3) -> (i32, i32) {
    (p.z + layout::START.0, p.x + layout::START.1)
}

/// A gondola's middle line's point at plan north `z` (between its ends),
/// for the art and the tests.
pub fn gondola_at(g: &Gondola, z: i32) -> V3 {
    let (_, z0) = plan_of(g.line[0]);
    let (_, zm) = plan_of(g.line[1]);
    let (_, z1) = plan_of(g.line[2]);
    let (a, b, lo, hi) = if z <= zm {
        (g.line[0], g.line[1], z0, zm)
    } else {
        (g.line[1], g.line[2], zm, z1)
    };
    let t = fx::div((z - lo).clamp(0, (hi - lo).max(1)), (hi - lo).max(1));
    a.lerp(b, t)
}

/// A unit vector `ONE` long along plan east.
pub const EAST: V3 = v3(0, 0, ONE);
/// ... and along plan north.
pub const NORTH: V3 = v3(ONE, 0, 0);
