//! Sunset Ruins' scenery and light (`ruins.rs` is the contract it draws
//! from). STAND-IN (the skeleton, 2026-10-14): the Jungle's scenery and a
//! first sunset light; the art agent replaces both.

use crate::course::Course;
use crate::fx::{cm, int};
use crate::geom::v3;
use crate::render::rgb;
use crate::ruins::RuinsLayout;
use crate::scene::{Look, Scenery};

/// Sunset Ruins' light: a low warm sun. STAND-IN: the sky orange, the fog
/// a dusky rose; the art agent paints the yellow → crimson sky.
pub fn look() -> Look {
    Look {
        sky: rgb(250, 150, 70),
        fog: rgb(200, 110, 90),
        fog_range: (int(60), int(170)),
        light_dir: v3(cm(70), cm(25), cm(30)).norm(),
        light: rgb(255, 190, 120),
        ground: rgb(80, 90, 45),
        rim: [0; 3],
        far: crate::render::FAR,
    }
}

/// Sunset Ruins' scenery and ground, chunked. STAND-IN: the Jungle's.
pub fn scenery(c: &Course, _r: &RuinsLayout) -> Scenery {
    crate::scene::jungle_scenery(c)
}
