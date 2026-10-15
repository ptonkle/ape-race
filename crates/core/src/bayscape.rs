//! Banana Bay Battle's scenery and light (`bay.rs` is the contract it draws
//! from; `baymodels.rs` will hold the models). STAND-IN (the skeleton,
//! 2026-10-15): the Jungle's scenery and a first 3 pm light; the art agent
//! replaces both.

use crate::bay::BayLayout;
use crate::course::Course;
use crate::fx::{cm, int};
use crate::geom::v3;
use crate::render::rgb;
use crate::scene::{Look, Scenery};

/// Banana Bay Battle's light: 3 pm in the Caribbean, a high warm sun.
/// STAND-IN: a bright blue sky, a pale sea haze; the art agent tunes it.
pub fn look() -> Look {
    Look {
        sky: rgb(110, 180, 245),
        fog: rgb(170, 215, 235),
        fog_range: (int(80), int(220)),
        light_dir: v3(cm(30), cm(85), cm(25)).norm(),
        light: rgb(255, 240, 210),
        ground: rgb(60, 170, 170),
        rim: [0; 3],
        far: crate::render::FAR,
    }
}

/// Banana Bay Battle's scenery and ground, chunked. STAND-IN: the Jungle's.
pub fn scenery(c: &Course, _b: &BayLayout) -> Scenery {
    crate::scene::jungle_scenery(c)
}
