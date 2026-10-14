//! Sector Ape's course. STAND-IN (the skeleton, 2026-10-13): the Jungle's
//! layout under the map's name, with every vine hanging down and no
//! bodies. The course agent replaces all of it.

use super::*;
use crate::course::Course;

const SEED: u64 = 0x5EC7_02A9_E000;

pub fn sector() -> Course {
    let mut c = Course::jungle_seeded(SEED);
    c.name = "sector";
    c.props.clear();
    let n = c.vines.len();
    let at = c.starts[0];
    c.sector = Some(Box::new(SectorLayout {
        hangs: vec![DOWN; n],
        hangers: vec![Hanger::Deck(0); n],
        bodies: Vec::new(),
        pads: vec![0],
        boundary: Boundary {
            side: int(60),
            below: -int(40),
            above: int(120),
        },
        door: Door {
            frame: Affine::IDENTITY,
            half: V3::ZERO,
            switch: 0,
            shut_at: None,
        },
        mechs: Vec::new(),
        wreck: Wreck {
            frame: Affine::IDENTITY,
            gap: crate::spacecraft::WRECK_GAP,
        },
        black_hole: (c.point(c.length), int(20)),
        set_pieces: Vec::new(),
        lobby: (at, c.start_yaw),
        finish_deck: 0,
        near: Vec::new(),
        routes: Vec::new(),
    }));
    c
}
