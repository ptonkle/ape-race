//! Sunset Ruins' course. STAND-IN (the skeleton, 2026-10-14): the Jungle's
//! layout under the map's name, from its own seed, with an empty
//! `RuinsLayout` (no river, turtles, traps or chamber; every vine hangs
//! from a tree). The course agent replaces all of it.

use super::*;
use crate::course::Course;

const SEED: u64 = 0x2B1_5E75_0001;

/// Sunset Ruins' course, from its fixed seed.
pub fn ruins() -> Course {
    ruins_seeded(SEED)
}

/// Sunset Ruins' course from `seed`.
pub fn ruins_seeded(seed: u64) -> Course {
    let mut c = Course::jungle_seeded(seed);
    c.name = "ruins";
    let end = c.point(c.length);
    let at = c.starts[0];
    let hangers = c
        .vines
        .iter()
        .map(|v| Hanger::Tree {
            trunk: v3(v.anchor.x, 0, v.anchor.z),
        })
        .collect();
    let nowhere = Slab {
        centre: V3::ZERO,
        fwd: v3(0, 0, ONE),
        half: V3::ZERO,
    };
    c.ruins = Some(Box::new(RuinsLayout {
        river: River { points: Vec::new() },
        turtles: Vec::new(),
        ravine: Ravine {
            lip: (V3::ZERO, V3::ZERO),
            span: (0, 0),
            below: -int(1000),
            spot: at,
            spot_yaw: c.start_yaw,
        },
        falls: Vec::new(),
        bridge: Bridge {
            from: V3::ZERO,
            to: V3::ZERO,
            width: BRIDGE_WIDTH,
            boost: Vec::new(),
            aim: V3::ZERO,
        },
        mouth: Mouth { opening: nowhere },
        corridor: Vec::new(),
        spikes: Vec::new(),
        traps: Vec::new(),
        chamber: Chamber {
            centre: end,
            radius: int(60),
            inner: int(40),
            height: int(40),
        },
        pile: Pile {
            centre: end,
            radius: int(6),
            height: int(5),
        },
        skylight: Skylight {
            centre: end.add(v3(0, int(40), 0)),
            radius: int(10),
        },
        landmarks: Vec::new(),
        hangers,
        lobby: (at, c.start_yaw),
    }));
    c
}
