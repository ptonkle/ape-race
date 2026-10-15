//! Sunset Ruins' course (phase 1, 2026-10-14): the first real layout.
//!
//! - **Third 1, the river** (0 → `LIP`, 560 m): straight down the fall
//!   with a gentle S, falling 6% (`Terrain::Ruins`), the river 28 m wide
//!   between 6 m banks; vines from jungle trees on the banks and from
//!   ruins every 22–28 m, a low vine every other station, snapping turtles
//!   swimming across.
//! - **The falls** (`LIP` → `MOUTH`, 40 m): the rope bridge across the
//!   river just short of the lip, its three boosted vines; the ravine;
//!   the temple's mouth.
//! - **Third 2, the corridor** (`MOUTH` → the chamber, 572 m): 30 m wide,
//!   a 32 m ceiling its vines hang from, a jog; spike traps; three trip
//!   wires and trapdoors with pit vines.
//! - **Third 3, the chamber** (a ring of radius 90 m, 300°, then in to
//!   the pile at the centre).

use super::*;
use crate::course::{
    banana_arcs, centreline, Course, Leg, Mushroom, Platform, Vine, START_HEIGHT, START_LEN,
};
use crate::fx::Dice;
use crate::geom::right_of;
use crate::rules::BODY_DROP;
use crate::ski::Terrain;

const SEED: u64 = 0x2B1_5E75_0001;

/// Metres along: the lip, the mouth, the chamber's ring, the turn in.
const LIP: i32 = 560;
const MOUTH: i32 = 600;
const CHAMBER: i32 = 1172;
const TURN_IN: i32 = 1643;
/// The ring's radius (its arc of 300° is 471 m long).
const RING: i32 = 90;
/// The finish stops this far short of the chamber's centre (the pile).
const SHORT: i32 = 12;
/// The bridge crosses this far along.
const BRIDGE_AT: i32 = LIP - 6;
/// The corridor's and the chamber's stations are pinned here: 14 m short
/// of the bend into the chamber (`CHAMBER` − 32 to `CHAMBER`) and of the
/// turn in (`TURN_IN` to + 32), and 5 m past each.
const CORNER_PINS: [i32; 4] = [CHAMBER - 46, CHAMBER + 5, TURN_IN - 14, TURN_IN + 37];
/// The river's last station, just short of the bridge.
const LAST_RIVER: i32 = BRIDGE_AT - 14;

const LEGS: &[Leg] = &[
    // the river
    Leg(60, 0),
    Leg(140, 0),
    Leg(120, 20),
    Leg(120, -20),
    Leg(120, 0),
    // the ravine
    Leg(40, 0),
    // the corridor, with a jog
    Leg(150, 0),
    Leg(60, 30),
    Leg(60, -30),
    Leg(270, 0),
    // right, into the chamber
    Leg(32, -90),
    // counter-clockwise round it
    Leg(471, 300),
    // and in toward the centre (cos θ = −r/(R − r) for a bend of radius r
    // to point at it)
    Leg(32, 104),
];

/// Sunset Ruins' course, from its fixed seed.
pub fn ruins() -> Course {
    ruins_seeded(SEED)
}

/// Where a vine hangs and what from, as it is placed.
struct Placing {
    vines: Vec<Vine>,
    hangers: Vec<Hanger>,
}

impl Placing {
    fn push(&mut self, anchor: V3, length: i32, at: i32, hanger: Hanger) -> usize {
        self.vines.push(Vine {
            anchor,
            length,
            slide: None,
            at,
        });
        self.hangers.push(hanger);
        self.vines.len() - 1
    }
}

/// Sunset Ruins' course from `seed`.
pub fn ruins_seeded(seed: u64) -> Course {
    // the chamber's centre, from the ring's start; the last straight runs
    // from the turn in to just short of it
    let probe = centreline("ruins", LEGS);
    let ring_start = probe.point(int(CHAMBER));
    let left = right_of(probe.heading_at(int(CHAMBER))).neg();
    let centre = ring_start.add(left.scale(int(RING)));
    let end = probe.path[probe.path.len() - 1];
    let to_centre = centre.sub(end).dot(probe.heading[probe.heading.len() - 1]);
    let last = (to_centre / ONE - SHORT).max(10);
    let mut legs: Vec<Leg> = LEGS.iter().map(|l| Leg(l.0, l.1)).collect();
    legs.push(Leg(last, 0));
    let mut c = centreline("ruins", &legs);
    let fall = c.heading[0];
    let u_of = |p: V3| p.flat().dot(fall);
    let lip_u = u_of(c.point(int(LIP)));
    let mouth_u = u_of(c.point(int(MOUTH)));
    c.terrain = Terrain::Ruins {
        top: V3::ZERO,
        fall,
        lip: lip_u,
        mouth: mouth_u,
    };
    // (the lip's own point is the ravine's: the river's height there)
    let lip_h = -fx::mul(GRADE, lip_u);
    let floor = lip_h - TEMPLE_BELOW_LIP;
    let centre = v3(centre.x, floor, centre.z);
    let land = |c: &Course, p: V3| c.terrain(p.x, p.z);

    // new placement draws from dice of its own
    let mut dice = Dice(seed ^ 0x5275_696E);
    let mut put = Placing {
        vines: Vec::new(),
        hangers: Vec::new(),
    };

    // the start: the Jungle's platform and starters, on a rock at the
    // river's head
    let p0 = c.point(START_LEN / 2 - int(2));
    let fwd = c.heading_at(0);
    let right = right_of(fwd);
    c.platforms.push(Platform {
        centre: v3(p0.x, START_HEIGHT, p0.z),
        fwd,
        half_len: START_LEN / 2 + int(2),
        half_width: int(9),
        top: START_HEIGHT,
    });
    for k in 0..4 {
        let at = c.point(int(2)).add(right.scale(int(-6) + k * int(4)));
        c.starts[k as usize] = v3(at.x, START_HEIGHT + BODY_DROP, at.z);
    }
    c.start_yaw = 0;
    let edge = START_LEN;
    for k in -3..=3 {
        let at = edge + cm(150);
        let base = c
            .point(at)
            .add(right_of(c.heading_at(at)).scale(int(3) * k));
        let side = if k < 0 { -1 } else { 1 };
        let trunk = c.point(at).add(right.scale(side * (RIVER_HALF + int(3))));
        put.push(
            v3(base.x, START_HEIGHT + int(30), base.z),
            int(29),
            at,
            Hanger::Tree {
                trunk: v3(trunk.x, land(&c, trunk), trunk.z),
            },
        );
    }

    let mut stations: Vec<Vec<usize>> = Vec::new();
    let mut landmarks = Vec::new();

    // Third 1: the river's stations, vines from the banks' trees and ruins
    let mut s = 32;
    let mut station = 0;
    while s <= LAST_RIVER {
        let h = c.heading_at(int(s));
        let r = right_of(h);
        let skip = if station == 0 || dice.below(6) == 0 {
            3
        } else {
            dice.below(3) as i32
        };
        let mut here = Vec::new();
        for lane in 0..3 {
            if lane == skip {
                continue;
            }
            let (anchor_h, length, off) = if station == 0 {
                (int(46), int(30), (lane - 1) * 6)
            } else {
                (
                    int(dice.range(38, 50)),
                    int(dice.range(23, 31)),
                    (lane - 1) * 10 + dice.range(-2, 2),
                )
            };
            let along = s + dice.range(-3, 3);
            let base = c.point(int(along)).add(r.scale(int(off)));
            let side = if off < 0 || (off == 0 && dice.below(2) == 0) {
                -1
            } else {
                1
            };
            let bank = c
                .point(int(along))
                .add(r.scale(side * (RIVER_HALF + int(3))));
            let bank = v3(bank.x, land(&c, bank), bank.z);
            // every third station one vine hangs from a ruin on the bank
            let hanger = if station % 3 == 2 && here.is_empty() {
                landmarks.push(Landmark {
                    kind: if landmarks.len() % 2 == 0 {
                        LandmarkKind::Moai
                    } else {
                        LandmarkKind::BananaBunch
                    },
                    at: bank,
                    yaw: crate::geom::yaw_of(r.scale(-side)),
                    scale: ONE,
                });
                Hanger::Ruin {
                    landmark: landmarks.len() - 1,
                }
            } else {
                Hanger::Tree { trunk: bank }
            };
            let i = put.push(
                v3(base.x, land(&c, base) + anchor_h, base.z),
                length,
                int(along),
                hanger,
            );
            here.push(i);
        }
        stations.push(here);
        // a low vine, to climb back up by from the water or a bank
        if station % 2 == 1 {
            let off = dice.range(-9, 9);
            let b = c.point(int(s + 7)).add(r.scale(int(off)));
            let side = if off < 0 { -1 } else { 1 };
            let trunk = c
                .point(int(s + 7))
                .add(r.scale(side * (RIVER_HALF + int(3))));
            put.push(
                v3(b.x, land(&c, b) + int(27), b.z),
                int(25) + cm(dice.range(0, 40)),
                int(s + 7),
                Hanger::Tree {
                    trunk: v3(trunk.x, land(&c, trunk), trunk.z),
                },
            );
        }
        station += 1;
        let next = s + dice.range(22, 28);
        // the last station stands 14 m short of the bridge: from one 22 m
        // or more short, the weaker AI levels swung to and fro there for
        // seconds with nothing in reach (RT1)
        s = if next > BRIDGE_AT - 22 && s < LAST_RIVER - 12 {
            LAST_RIVER
        } else {
            next
        };
    }

    // the river: its centreline every 40 m, falling with the land
    let mut points = Vec::new();
    let mut k = -16;
    loop {
        let at = k.min(LIP);
        let p = if at < 0 {
            c.point(0).add(fwd.scale(int(at)))
        } else {
            c.point(int(at))
        };
        // the rapids: rougher in the steeper-looking stretches and at the lip
        let rough = (ONE + fx::sin(at * 1300)) / 2;
        let rough = if at > LIP - 60 { ONE } else { rough };
        points.push(RiverPoint {
            at: v3(p.x, land(&c, p) - cm(50), p.z),
            half_width: RIVER_HALF,
            current: CURRENT,
            rough,
        });
        if at == LIP {
            break;
        }
        k += 40;
    }
    let river = River { points };

    // the turtles, swimming across, each a bouncer
    let mut turtles = Vec::new();
    let mut t = 85;
    while t < LIP - 40 {
        let at = t + dice.range(-8, 8);
        let p = c.point(int(at));
        let r = right_of(c.heading_at(int(at)));
        let y = land(&c, p) - cm(50);
        let from = p.add(r.scale(-(RIVER_HALF - int(2))));
        let to = p.add(r.scale(RIVER_HALF - int(2)));
        let (from, to) = (v3(from.x, y, from.z), v3(to.x, y, to.z));
        let (from, to) = if dice.below(2) == 0 {
            (from, to)
        } else {
            (to, from)
        };
        c.mushrooms.push(Mushroom {
            top: v3(from.x, y + TURTLE_TOP, from.z),
            radius: TURTLE_RADIUS,
        });
        turtles.push(Turtle {
            from,
            to,
            period: crate::rules::secs_to_ticks(dice.range(600, 900)) as u32,
            phase: dice.below(900),
            bouncer: c.mushrooms.len() - 1,
        });
        t += dice.range(60, 80);
    }

    // the bank's low vine by the reset spot
    let spot_s = LIP - 30;
    let spot = c
        .point(int(spot_s))
        .add(right_of(c.heading_at(int(spot_s))).scale(int(17)));
    let spot = v3(spot.x, land(&c, spot) + BODY_DROP, spot.z);
    {
        let at = spot_s + 8;
        let r = right_of(c.heading_at(int(at)));
        let b = c.point(int(at)).add(r.scale(int(11)));
        let trunk = c.point(int(at)).add(r.scale(int(18)));
        put.push(
            v3(b.x, land(&c, b) + int(27), b.z),
            int(25),
            int(at),
            Hanger::Tree {
                trunk: v3(trunk.x, land(&c, trunk), trunk.z),
            },
        );
    }

    // the rope bridge across the river, its boosted vines
    let bh = c.heading_at(int(BRIDGE_AT));
    let br = right_of(bh);
    let bp = c.point(int(BRIDGE_AT));
    let deck = lip_h + int(28);
    let mut boost = Vec::new();
    for off in [-10, 0, 10] {
        let a = bp.add(br.scale(int(off)));
        boost.push(put.push(v3(a.x, deck, a.z), int(18), int(BRIDGE_AT), Hanger::Bridge));
    }
    stations.push(boost.clone());
    let mp = c.point(int(MOUTH));
    let mh = c.heading_at(int(MOUTH));
    // vines from the mouth's carved lip, just inside: where the boost
    // carries an ape, so there is always something to catch on landing
    // (and the AI sees a target across the ravine)
    {
        let at = MOUTH + 6;
        let p = c.point(int(at));
        let r = right_of(c.heading_at(int(at)));
        let mut here = Vec::new();
        for off in [-8, 0, 8] {
            let a = p.add(r.scale(int(off)));
            here.push(put.push(
                v3(a.x, floor + int(14), a.z),
                int(6),
                int(at),
                Hanger::Mouth,
            ));
        }
        stations.push(here);
    }
    let mouth = Mouth {
        opening: Slab {
            centre: v3(mp.x, floor + int(7), mp.z),
            fwd: mh,
            half: v3(int(13), int(7), int(2)),
        },
    };
    let bridge = Bridge {
        from: v3(bp.x, deck, bp.z).sub(br.scale(c.half_width)),
        to: v3(bp.x, deck, bp.z).add(br.scale(c.half_width)),
        width: BRIDGE_WIDTH,
        boost,
        aim: v3(mp.x, floor + int(6), mp.z).add(mh.scale(int(6))),
    };
    let lp = c.point(int(LIP));
    let lr = right_of(c.heading_at(int(LIP)));
    let lip = (
        v3(lp.x, lip_h, lp.z).sub(lr.scale(RIVER_HALF)),
        v3(lp.x, lip_h, lp.z).add(lr.scale(RIVER_HALF)),
    );
    let ravine = Ravine {
        lip,
        span: (int(LIP), int(MOUTH)),
        below: floor - int(8),
        spot,
        spot_yaw: crate::geom::yaw_of(c.heading_at(int(spot_s))),
    };
    let falls = vec![
        Falls {
            from: lip.0,
            to: lip.1,
            drop: RAVINE_DEPTH,
        },
        Falls {
            from: v3(mp.x, floor, mp.z).sub(right_of(mh).scale(int(6))),
            to: v3(mp.x, floor, mp.z).add(right_of(mh).scale(int(6))),
            drop: RAVINE_DEPTH - TEMPLE_BELOW_LIP,
        },
    ];

    // Third 2: the corridor's sections, every 30 m (and the bend in)
    let mut corridor = Vec::new();
    let mut a = MOUTH;
    while a < CHAMBER - 16 {
        let b = (a + 30).min(CHAMBER - 16);
        let (pa, pb) = (c.point(int(a)), c.point(int(b)));
        corridor.push(Section {
            from: v3(pa.x, floor, pa.z),
            to: v3(pb.x, floor, pb.z),
            half_width: CORRIDOR_HALF,
            height: CORRIDOR_HEIGHT,
        });
        a = b;
    }
    // the trapdoors: a wire across, the door 3–15 m on
    let trap_at = [700, 860, 1020];
    let mut traps = Vec::new();
    for (k, &w) in trap_at.iter().enumerate() {
        let wire = wire_across(&c, int(w), CORRIDOR_HALF);
        let wire = (
            v3(wire.0.x, floor + cm(20), wire.0.z),
            v3(wire.1.x, floor + cm(20), wire.1.z),
        );
        let dp = c.point(int(w + 9));
        let dh = c.heading_at(int(w + 9));
        let mut vines = Vec::new();
        for off in [-7, 7] {
            let v = dp.add(right_of(dh).scale(int(off)));
            vines.push(put.push(
                v3(v.x, floor + CORRIDOR_HEIGHT, v.z),
                CORRIDOR_HEIGHT + int(3),
                int(w + 9),
                Hanger::Pit { trap: k },
            ));
        }
        traps.push(Trapdoor {
            wire,
            door: Slab {
                centre: v3(dp.x, floor, dp.z),
                fwd: dh,
                half: v3(CORRIDOR_HALF, cm(30), int(6)),
            },
            pit: floor - int(4),
            vines,
            tripped_at: None,
        });
    }
    let mut spikes = Vec::new();
    for (k, &at) in [650, 780, 940, 1090].iter().enumerate() {
        let p = c
            .point(int(at))
            .add(right_of(c.heading_at(int(at))).scale(int((k as i32 % 3 - 1) * 8)));
        spikes.push(SpikeTrap {
            area: Slab {
                centre: v3(p.x, floor + cm(10), p.z),
                fwd: c.heading_at(int(at)),
                half: v3(int(3), cm(30), int(3)),
            },
        });
    }

    // the corridor's and the chamber's stations: vines from the ceiling
    let end = c.length / ONE;
    let mut s = MOUTH + 30;
    let mut station = 0;
    while s < end - 6 {
        let h = c.heading_at(int(s));
        let r = right_of(h);
        let in_chamber = s >= CHAMBER - 10;
        let roof = floor
            + if in_chamber {
                CHAMBER_HEIGHT
            } else {
                CORRIDOR_HEIGHT
            };
        let skip = if dice.below(6) == 0 {
            3
        } else {
            dice.below(3) as i32
        };
        let mut here = Vec::new();
        let spread = if in_chamber { 10 } else { 9 };
        let near_pile = s >= end - 40;
        for lane in 0..3 {
            if lane == skip {
                continue;
            }
            let off = (lane - 1) * spread + dice.range(-2, 2);
            let along = s + dice.range(-3, 3);
            let base = c.point(int(along)).add(r.scale(int(off)));
            // anchors at the roof, bottoms 11–14 m up in the corridor and
            // 15–22 m in the chamber, as the Jungle's
            let length = if in_chamber {
                int(dice.range(24, 31))
            } else {
                int(dice.range(18, 21))
            };
            let hanger = if near_pile {
                Hanger::Skylight
            } else {
                Hanger::Ceiling
            };
            here.push(put.push(v3(base.x, roof, base.z), length, int(along), hanger));
        }
        stations.push(here);
        if station % 2 == 1 && !near_pile {
            let off = dice.range(-8, 8);
            let b = c.point(int(s + 7)).add(r.scale(int(off)));
            put.push(
                v3(b.x, roof, b.z),
                roof - floor - int(3) - cm(dice.range(0, 40)),
                int(s + 7),
                Hanger::Ceiling,
            );
        }
        station += 1;
        let mut next = s + dice.range(22, 26);
        // a station 14 m short of each bend and one 5 m past it (MAPS.md:
        // without them the AI swung to and fro at the chamber's door and
        // at the turn in, RT1)
        for pin in CORNER_PINS {
            if s < pin - 10 && next > pin {
                next = pin;
            }
        }
        s = next;
    }

    c.vines = put.vines;
    banana_arcs(&mut c, &stations);

    // item crates: rows of three across the course, every 240 m, never
    // over the ravine
    let mut at = 150;
    while at < end - 60 {
        if !(LIP - 10..MOUTH + 10).contains(&at) {
            let h = c.heading_at(int(at));
            let r = right_of(h);
            let base = c.point(int(at));
            let gy = land(&c, base);
            let up = if at >= MOUTH { (14, 18) } else { (16, 24) };
            for off in [-9, 0, 9] {
                let p = base.add(r.scale(int(off)));
                c.crates
                    .push(v3(p.x, gy + int(dice.range(up.0, up.1)), p.z));
            }
        }
        at += 240;
    }

    let pile = Pile {
        centre,
        radius: int(9),
        height: int(10),
    };
    c.ruins = Some(Box::new(RuinsLayout {
        river,
        turtles,
        ravine,
        falls,
        bridge,
        mouth,
        corridor,
        spikes,
        traps,
        chamber: Chamber {
            centre,
            radius: int(RING) + RING_HALF,
            inner: int(RING) - RING_HALF,
            height: CHAMBER_HEIGHT,
        },
        pile,
        skylight: Skylight {
            centre: centre.add(v3(0, CHAMBER_HEIGHT, 0)),
            radius: int(14),
        },
        landmarks,
        hangers: put.hangers,
        lobby: (c.starts[0], c.start_yaw),
        marks: Marks {
            lip: int(LIP),
            mouth: int(MOUTH),
            chamber: int(CHAMBER),
            turn_in: int(TURN_IN),
        },
    }));
    c
}
