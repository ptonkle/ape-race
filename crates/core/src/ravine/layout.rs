//! Rattlesnake Ravine's course. STAND-IN (the skeleton, 2026-10-17): the
//! Jungle's AI-proven layout under the map's name, from its own seed (so
//! no other map's dice change), with an all-but-empty `RavineLayout`: no
//! walls, rooms, decks, lizards, tracks, carts, skull or split; every vine
//! hangs from a ledge and none is a snake. The course agent replaces all of
//! it.

use super::*;
use crate::course::Course;
use crate::fx::int;

const SEED: u64 = 0x5A4E_C0DE_0001;

/// Rattlesnake Ravine's course, from its fixed seed.
pub fn ravine() -> Course {
    ravine_seeded(SEED)
}

/// Rattlesnake Ravine's course from `seed`.
pub fn ravine_seeded(seed: u64) -> Course {
    let mut c = Course::jungle_seeded(seed);
    c.name = "ravine";
    let end = c.point(c.length);
    let at = c.starts[0];
    let hangers = c
        .vines
        .iter()
        .map(|v| Hanger {
            from: HangFrom::Ledge { wall: 0 },
            at: v.anchor,
            snake: false,
            snapped_at: None,
        })
        .collect();
    let h = c.heading_at(c.length);
    c.ravine = Some(Box::new(RavineLayout {
        walls: Vec::new(),
        rooms: Vec::new(),
        decks: Vec::new(),
        hangers,
        lizards: Vec::new(),
        tracks: Vec::new(),
        carts: Vec::new(),
        skull: None,
        split: None,
        rocks: Vec::new(),
        glows: Vec::new(),
        finish: Finish {
            sphinx: end.add(h.scale(int(30))),
            sphinx_yaw: crate::geom::yaw_of(h.scale(-ONE)),
            campfire: end.add(v3(int(12), 0, 0)),
            sticks: Vec::new(),
            tents: Vec::new(),
        },
        lobby: (at, c.start_yaw),
    }));
    c
}
