//! The City map (2026-10-12): New York. No gameplay differences from the
//! Jungle: the vines are electric wires and cables hanging from
//! helicopters, cranes, the bridge's cables, street lights and wires
//! strung between buildings; the course turns sharply like city blocks
//! (90° and 45° corners, each 90° one with a tall anchor to swing round);
//! the middle crosses a river on a suspension bridge; parked cars and cabs
//! bounce like the Jungle's mushrooms; vines are sparser than the Jungle's.
//!
//! SKELETON: `city()` is a stand-in (the Jungle's layout renamed) until the
//! course agent lays out the real one. The types below are the contract
//! between the course (placement, physics) and the scene (drawing): the
//! course fills a `CityLayout` on `Course.city`; the scene draws from it.

use crate::course::Course;
use crate::geom::V3;

/// Everything the City has beyond the shared course data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CityLayout {
    /// buildings along the streets (their faces are the course's walls)
    pub blocks: Vec<Block>,
    /// parked cars and cabs; each one also bounces like a mushroom (its
    /// bouncer is `Course.mushrooms[car.bouncer]`)
    pub cars: Vec<Car>,
    /// the river across the middle, if any
    pub river: Option<River>,
    /// the bridge over it
    pub bridge: Option<Bridge>,
    /// what each vine hangs from (one per vine that hangs from something
    /// the scene should draw)
    pub hangers: Vec<Hanger>,
}

/// A building: a box on the ground with its own axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    /// the middle of its footprint, on the ground
    pub centre: V3,
    /// its own axes on the ground (unit, flat)
    pub fwd: V3,
    pub half_len: i32,
    pub half_width: i32,
    pub height: i32,
    /// which look (facade, colour, roof dressing) the scene gives it
    pub style: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarKind {
    Cab,
    Sedan,
    Van,
    Bus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Car {
    /// the middle of the car on the ground
    pub at: V3,
    /// which way it faces (`fx` angle)
    pub yaw: i32,
    pub kind: CarKind,
    /// the index of its bouncer in `Course.mushrooms`
    pub bouncer: usize,
}

/// A river crossing the course between `from` and `to` metres along it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct River {
    pub from: i32,
    pub to: i32,
    /// the water's surface height
    pub level: i32,
}

/// A suspension bridge: a deck from `start` to `end` (on its top, the
/// height an ape stands at), and towers that carry the main cables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bridge {
    pub start: V3,
    pub end: V3,
    pub half_width: i32,
    /// the towers' feet on the deck, and how tall they stand above it
    pub towers: Vec<V3>,
    pub tower_height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HangerKind {
    /// hovering: `from` is the rotor's hub
    Helicopter,
    /// a tower crane: `from` its mast's foot, `to` the jib's tip
    Crane,
    /// a suspender from the bridge's main cable: `from` the cable point
    BridgeCable,
    /// a street-light or traffic-light gantry: `from` its foot, `to` its arm's end
    StreetLight,
    /// a wire strung between two buildings: `from` and `to` its ends
    Wire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hanger {
    /// the vine it carries (`Course.vines[vine]`)
    pub vine: usize,
    pub kind: HangerKind,
    pub from: V3,
    pub to: V3,
}

/// The City course. SKELETON: the Jungle's layout under the City's name.
pub fn city() -> Course {
    let mut c = Course::jungle();
    c.name = "city";
    c.city = Some(Box::default());
    c
}
