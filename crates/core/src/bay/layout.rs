//! Banana Bay Battle's course. STAND-IN (the skeleton, 2026-10-15): the
//! Jungle's layout under the map's name, from its own seed, with an empty
//! `BayLayout` (no ships, barrels, volleys or fighters; the water far
//! below; every vine hangs from a `Hanger::Shore` post over its tree). The
//! course agent replaces all of it.

use super::*;
use crate::course::Course;

const SEED: u64 = 0xBA7_BA77_0001;

/// Banana Bay Battle's course, from its fixed seed.
pub fn bay() -> Course {
    bay_seeded(SEED)
}

/// Banana Bay Battle's course from `seed`.
pub fn bay_seeded(seed: u64) -> Course {
    let mut c = Course::jungle_seeded(seed);
    c.name = "bay";
    let end = c.point(c.length);
    let at = c.starts[0];
    let hangers = c
        .vines
        .iter()
        .map(|v| Hanger::Shore {
            foot: v3(v.anchor.x, 0, v.anchor.z),
        })
        .collect();
    let (from, to, half_width) = match c.platforms.first() {
        Some(p) => {
            let top = v3(0, p.top, 0);
            (
                p.centre.sub(p.fwd.scale(p.half_len)).flat().add(top),
                p.centre.add(p.fwd.scale(p.half_len)).flat().add(top),
                p.half_width,
            )
        }
        None => (at, at, int(2)),
    };
    c.bay = Some(Box::new(BayLayout {
        ships: Vec::new(),
        hangers,
        boundary: Vec::new(),
        bouncy: Vec::new(),
        // STAND-IN: far below the Jungle's ground, so nothing touches it
        water: Water {
            level: -int(100),
            waves: Vec::new(),
        },
        dock: Dock {
            from,
            to,
            half_width,
            platform: 0,
        },
        island: Island {
            centre: end,
            radius: int(25),
            chest: end,
            chest_yaw: 0,
            chest_size: v3(int(3), int(2), int(2)),
        },
        volleys: Vec::new(),
        fighters: Vec::new(),
        lobby: (at, c.start_yaw),
    }));
    c
}
