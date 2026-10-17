//! The City map (2026-10-12): New York. No gameplay differences from the
//! Jungle: the vines are electric wires and cables hanging from
//! helicopters, cranes, the bridge's cables, street lights and wires
//! strung between buildings; the course turns sharply like city blocks
//! (90° and 45° corners, each 90° one with a tall anchor to swing round);
//! the middle crosses a river on a suspension bridge; parked cars and cabs
//! bounce like the Jungle's mushrooms; vines are sparser than the Jungle's.
//!
//! The layout is in `city/layout.rs`; the physics the City adds (switched
//! off on the Jungle, whose `city` is `None`) is below: the river, bouncers
//! that move, and (in course.rs) `Course.streets`, the walls of straight
//! streets meeting in sharp corners. The types below are the contract
//! between the course (placement, physics) and the scene (drawing): the
//! course fills a `CityLayout` on `Course.city`; the scene draws from it.

use crate::course::{ground, Course, Mushroom};
use crate::fx::{self, cm, int};
use crate::geom::{right_of, v3, yaw_of, V3};
use crate::rules::{BODY_DROP, BODY_RADIUS, LADDER_EVERY, SWIM_SINK};

/// Everything the City has beyond the shared course data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CityLayout {
    /// buildings along the streets (their faces are the course's walls)
    pub blocks: Vec<Block>,
    /// parked cars and cabs; each one also bounces like a mushroom (its
    /// bouncer is `Course.mushrooms[car.bouncer]`)
    pub cars: Vec<Car>,
    /// boats on the river; each one bounces like a mushroom too (its
    /// bouncer is `Course.mushrooms[boat.bouncer]`)
    pub boats: Vec<Boat>,
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
    /// the middle of the car on the ground (a moving car: where it is at
    /// tick 0; call `Course::car_at` for where it is now)
    pub at: V3,
    /// which way it faces (`fx` angle)
    pub yaw: i32,
    pub kind: CarKind,
    /// the index of its bouncer in `Course.mushrooms` (a moving car's
    /// bouncer moves with it: `Course::bouncer(i, tick)`)
    pub bouncer: usize,
    /// a moving car's route; `None`: parked
    pub drive: Option<Drive>,
}

/// A moving car's route: along the course's centreline from `from` to `to`
/// metres along it (`to` < `from`: against the race), `lane` metres right
/// of the centreline, taking `period` ticks, then round again from `from`
/// (it turns off at the end of its street and the next one comes on).
/// `phase` ticks start it part-way. A pure function of the tick, so replays
/// and every browser online agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Drive {
    pub from: i32,
    pub to: i32,
    pub lane: i32,
    pub period: i32,
    pub phase: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatKind {
    Tug,
    Ferry,
    Barge,
}

/// A boat on the river, standing still on the water (moored or drifting
/// at anchor). It bounces like a mushroom: a way back up out of the water.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Boat {
    /// the middle of the boat at the waterline
    pub at: V3,
    pub yaw: i32,
    pub kind: BoatKind,
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

impl Course {
    /// Where car `i` of the City is at `tick`, on the ground, and which way
    /// it faces. A parked car: where it stands.
    pub fn car_at(&self, i: usize, tick: u32) -> (V3, i32) {
        let Some(car) = self.city.as_ref().and_then(|c| c.cars.get(i)) else {
            return (V3::ZERO, 0);
        };
        match car.drive {
            None => (car.at, car.yaw),
            Some(d) => drive_at(self, &d, tick),
        }
    }

    /// Bouncer `m` (`Course.mushrooms[m]`) at `tick`: a moving car's moves
    /// with it; every other one stands still.
    pub fn bouncer(&self, m: usize, tick: u32) -> Mushroom {
        if let Some(r) = self.ruins.as_deref() {
            return self.ruins_bouncer(r, m, tick);
        }
        if let Some(r) = self.ravine.as_deref() {
            return self.ravine_bouncer(r, m, tick);
        }
        let b = self.mushrooms[m];
        let Some(city) = self.city.as_ref() else {
            return b;
        };
        for (i, car) in city.cars.iter().enumerate() {
            if car.bouncer == m && car.drive.is_some() {
                let (at, _) = self.car_at(i, tick);
                let up = b.top.y - car.at.y;
                return Mushroom {
                    top: v3(at.x, at.y + up, at.z),
                    radius: b.radius,
                };
            }
        }
        b
    }
}

/// Where a car on route `d` is at `tick`, and its heading.
pub(crate) fn drive_at(c: &Course, d: &Drive, tick: u32) -> (V3, i32) {
    let period = d.period.max(1) as i64;
    let t = (tick as i64 + d.phase as i64).rem_euclid(period);
    let s = d.from + ((d.to - d.from) as i64 * t / period) as i32;
    let h = c.heading_at(s);
    let p = c.point(s).add(right_of(h).scale(d.lane));
    let yaw = yaw_of(h) + if d.to < d.from { fx::TURN / 2 } else { 0 };
    (v3(p.x, ground(p.x, p.z), p.z), fx::wrap(yaw))
}

impl Course {
    /// The river's surface under `p`, if `p` is over the City's river (in
    /// the corridor; the Jungle has none).
    pub fn water(&self, p: V3) -> Option<i32> {
        if let Some(r) = self.ruins.as_deref() {
            return self.ruins_water(r, p);
        }
        let r = self.city.as_ref()?.river?;
        let o = self.point(r.from);
        let h = self.heading_at(r.from);
        let d = p.flat().sub(o.flat());
        let a = d.dot(h);
        if a < 0 || a > r.to - r.from || d.dot(right_of(h)).abs() > self.half_width + int(10) {
            return None;
        }
        Some(r.level)
    }

    /// The City's floor under `p`: the street, or in the river where a
    /// swimmer's feet hang (`Course::floor` on the City).
    pub fn city_floor(&self, p: V3) -> i32 {
        match self.water(p) {
            Some(level) => level - SWIM_SINK,
            None => ground(p.x, p.z),
        }
    }

    /// The first bouncer moving cars use (`Course.mushrooms` from here on
    /// move with their cars).
    fn first_mover(&self) -> usize {
        let n = self.mushrooms.len();
        self.city.as_ref().map_or(n, |c| {
            c.cars
                .iter()
                .filter(|car| car.drive.is_some())
                .map(|car| car.bouncer)
                .min()
                .unwrap_or(n)
        })
    }

    /// The bouncer (a mushroom, or the City's car or boat) whose top a body
    /// at `p` (the hands) is on or in at `tick`: its feet are over it and
    /// no higher than its top, and no lower than the floor under it. On the
    /// Jungle exactly `mushroom_under`.
    pub fn bouncer_under(&self, p: V3, tick: u32) -> Option<usize> {
        if let Some(r) = self.ruins.as_deref() {
            return self.ruins_bouncer_under(r, p, tick);
        }
        if let Some(r) = self.ravine.as_deref() {
            return self.ravine_bouncer_under(r, p, tick);
        }
        let Some(city) = self.city.as_ref() else {
            return self.mushroom_under(p);
        };
        let feet = p.y - BODY_DROP;
        // no car or boat stands this tall: an ape up among the vines (the
        // AI's planned flights, every tick) is on none of them
        if feet > ground(p.x, p.z) + BOUNCER_TALLEST {
            return None;
        }
        let floor = self.city_floor(p) - int(1);
        let on = |m: &Mushroom| {
            let d = p.flat().sub(m.top.flat());
            let r = m.radius + BODY_RADIUS / 2;
            d.x.abs() <= r && d.z.abs() <= r && d.len() <= r && feet <= m.top.y && feet >= floor
        };
        let movers = self.first_mover();
        if let Some(m) = self.mushrooms[..movers].iter().position(on) {
            return Some(m);
        }
        city.cars.iter().find_map(|car| {
            let d = car.drive.as_ref()?;
            let (at, _) = drive_at(self, d, tick);
            let b = self.mushrooms[car.bouncer];
            let moved = Mushroom {
                top: v3(at.x, at.y + b.top.y - car.at.y, at.z),
                radius: b.radius,
            };
            on(&moved).then_some(car.bouncer)
        })
    }

    /// Where a swimmer at `p` makes for: the nearest way out of the river
    /// (a bank, a ladder up the bridge deck, or a boat to bounce off). The
    /// point to swim to, and where the hands end a climb out from it (a
    /// boat: none, it bounces).
    pub fn water_exit(&self, p: V3) -> Option<(V3, Option<V3>)> {
        if let Some(r) = self.ruins.as_deref() {
            return self.ruins_exit(r, p);
        }
        let city = self.city.as_ref()?;
        let r = city.river?;
        let o = self.point(r.from);
        let h = self.heading_at(r.from);
        let right = right_of(h);
        let flat = p.flat();
        let lat = flat.sub(o.flat()).dot(right);
        let mut best: Option<(i32, V3, Option<V3>)> = None;
        let mut offer = |to: V3, top: Option<V3>| {
            let dd = to.flat().dist(flat);
            if best.is_none_or(|(b, _, _)| dd < b) {
                best = Some((dd, to, top));
            }
        };
        // the banks, straight across
        let width = r.to - r.from;
        for (edge, inland) in [(0, -int(2)), (width, width + int(2))] {
            let to = o.add(h.scale(edge)).add(right.scale(lat));
            let up = o.add(h.scale(inland)).add(right.scale(lat));
            let top = v3(up.x, ground(up.x, up.z) + BODY_DROP, up.z);
            offer(v3(to.x, p.y, to.z), Some(top));
        }
        // the bridge deck's ladders: up, then a step in onto the deck
        if let Some(b) = city.bridge.as_ref() {
            let fwd = b.end.sub(b.start).flat().norm();
            for l in b.ladders() {
                let on_line = b
                    .start
                    .flat()
                    .add(fwd.scale(l.flat().sub(b.start.flat()).dot(fwd)));
                let top = l.flat().add(on_line.sub(l.flat()).with_len(cm(150)));
                offer(v3(l.x, p.y, l.z), Some(v3(top.x, l.y + BODY_DROP, top.z)));
            }
        }
        for boat in &city.boats {
            offer(self.mushrooms[boat.bouncer].top, None);
        }
        best.map(|(_, to, top)| (to, top))
    }
}

impl Bridge {
    /// Where ladders go down from the deck's edges to the water, on the
    /// deck's top: every `LADDER_EVERY` along both sides.
    pub fn ladders(&self) -> Vec<V3> {
        let run = self.end.sub(self.start).flat();
        let fwd = run.norm();
        let right = right_of(fwd);
        let mut out = Vec::new();
        let mut t = LADDER_EVERY / 2;
        while t < run.len() {
            for side in [-1, 1] {
                out.push(
                    self.start
                        .add(fwd.scale(t))
                        .add(right.scale(side * self.half_width)),
                );
            }
            t += LADDER_EVERY;
        }
        out
    }

    /// The main cables' height above the deck `t` metres along it: from
    /// the anchorages at the deck's ends up to the towers' saddles, and
    /// sagging between the towers to `CABLE_SAG`.
    pub fn cable_height(&self, t: i32) -> i32 {
        let len = self.end.sub(self.start).flat().len();
        let top = CABLE_TOP;
        let near = t.min(len - t).max(0);
        if near <= TOWER_IN {
            return CABLE_FOOT + fx::mul(top - CABLE_FOOT, fx::div(near, TOWER_IN));
        }
        let half = len / 2 - TOWER_IN;
        let x = fx::div(len / 2 - near, half.max(1));
        CABLE_SAG + fx::mul(top - CABLE_SAG, fx::mul(x, x))
    }

    /// A point on main cable `k` (of `CABLES`, across the deck) `t` metres
    /// along it.
    pub fn cable(&self, k: usize, t: i32) -> V3 {
        let fwd = self.end.sub(self.start).flat().norm();
        let p = self
            .start
            .add(fwd.scale(t))
            .add(right_of(fwd).scale(CABLES[k % CABLES.len()]));
        v3(p.x, self.start.y + self.cable_height(t), p.z)
    }
}

/// No car's roof or boat's deck stands higher than this above the street.
pub const BOUNCER_TALLEST: i32 = int(5);

/// The river runs this far either side of the course (for the drawing:
/// no other street comes this near it, and no building stands in it).
pub const RIVER_REACH: i32 = int(220);

/// The bridge's four main cables, metres right of the deck's middle (the
/// outer two over the water beside the deck, as on the Brooklyn Bridge's
/// four). Each hangs from both towers' tops: the towers' legs stand just
/// outside the corridor and their crossbeams above the ceiling.
pub const CABLES: [i32; 4] = [-int(17), -int(6), int(6), int(17)];
/// The cables' height above the deck at the towers, at the deck's ends
/// (the anchorages) and at the middle of the main span.
pub const CABLE_TOP: i32 = int(68);
pub const CABLE_FOOT: i32 = int(4);
pub const CABLE_SAG: i32 = int(40);
/// The towers stand this far in from the deck's ends.
pub const TOWER_IN: i32 = int(30);

/// Block styles (`Block.style`), for the scene.
pub mod style {
    /// a row of brownstones: 12–17 m
    pub const BROWNSTONE: u8 = 0;
    /// brick walk-ups and mid-rise flats: 20–36 m
    pub const MIDRISE: u8 = 1;
    /// stone and art-deco offices: 40–80 m
    pub const OFFICE: u8 = 2;
    /// glass and steel skyscrapers: 90–190 m
    pub const TOWER: u8 = 3;
    /// a construction site's hoarding (3 m), a crane stands in it
    pub const HOARDING: u8 = 4;
    /// the grand building closing the plaza past the finish
    pub const TERMINAL: u8 = 5;
    /// low brick warehouses by the river
    pub const WAREHOUSE: u8 = 6;
}

mod layout;
#[cfg(test)]
mod tests;
pub use layout::{boat_size, car_size, city, city_seeded, DECK_HALF};
