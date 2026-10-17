//! Rattlesnake Ravine's scenery and light (`ravine.rs` is the contract it
//! draws from). STAND-IN (the skeleton, 2026-10-17): the Jungle's scenery
//! and a first night light; the art agent replaces both (its models go in
//! `ravinemodels.rs`).

use crate::course::Course;
use crate::fx::{cm, int};
use crate::geom::v3;
use crate::ravine::RavineLayout;
use crate::render::rgb;
use crate::scene::{Look, Scenery};

/// Rattlesnake Ravine's light: night under the stars. STAND-IN: a dark
/// blue sky and fog, a cool moonlight from high up; the art agent paints
/// the starry sky and the warm torch and mushroom glows.
pub fn look() -> Look {
    Look {
        sky: rgb(14, 22, 58),
        fog: rgb(22, 30, 64),
        fog_range: (int(60), int(190)),
        light_dir: v3(cm(30), cm(85), cm(40)).norm(),
        light: rgb(150, 165, 220),
        ground: rgb(70, 45, 35),
        rim: [0; 3],
        far: crate::render::FAR,
    }
}

/// Rattlesnake Ravine's scenery and ground, chunked. STAND-IN: the
/// Jungle's.
pub fn scenery(c: &Course, _r: &RavineLayout) -> Scenery {
    crate::scene::jungle_scenery(c)
}
