//! Sunset Ruins' scenery and light (`ruins.rs` is the contract it draws
//! from). STAND-IN (the skeleton, 2026-10-14): the Jungle's scenery and a
//! first sunset light; the art agent replaces both.

use crate::course::Course;
use crate::ruins::RuinsLayout;
use crate::scene::{Look, Scenery};

/// Sunset Ruins' light: the sunset (`ruinsmodels::look`, art phase 1):
/// a sun 7° up raking gold-orange, rose-violet shade, a golden haze. The
/// sky's dome (`ruinsmodels::sky`) is drawn round the eye once the scenery
/// is placed (a later phase).
pub fn look() -> Look {
    crate::ruinsmodels::look()
}

/// Sunset Ruins' scenery and ground, chunked. STAND-IN: the Jungle's.
pub fn scenery(c: &Course, _r: &RuinsLayout) -> Scenery {
    crate::scene::jungle_scenery(c)
}
