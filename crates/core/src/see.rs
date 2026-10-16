//! See-through (SECOND-ORDER BX2): in a race, the scenery between the
//! chase camera and the followed ape, or right by the camera, is drawn with
//! a hole in it, so a mast, a yard, a tree, a trunk, a wall or a hull never
//! hides the ape and the camera need not dodge them (dodging made it
//! jiggle, BX1). On every map.
//!
//! The core decides the hole: where the ape is and the floor under it, the
//! hole's sizes, and which draws it cuts (by their kind, `render`). The
//! shader (`data/shaders/scene.frag`, `seeCut`) only applies it: a
//! fragment of a cut kind above the floor is cut where it is within `NEAR`
//! of the eye, or within a cone from the eye to the ape, `LENS` wide at the
//! eye and `HEAD` wide at the ape; solid inside `SOLID` of either width,
//! then thinning out in an ordered dither (reads as glass) to nothing at
//! the edge, and thinning out over the last `FADE` before the ape.
//! Generous on purpose: the person found long-time-comin's hole (1.6 m at
//! the head, 6.4 m back) much too small; ours is 4.5 m at the ape, 10 to
//! 13 m back, about two fifths of the view's height.

use crate::fx::{cm, ONE};
use crate::geom::V3;
use crate::render::{SCENERY, VINES};
use crate::rules::BODY_DROP;

/// Everything of a cut kind this near the eye goes (and thins out from
/// `SOLID` of it).
pub const NEAR: i32 = cm(500);
/// The cone's half width at the eye and at the ape.
pub const LENS: i32 = cm(300);
pub const HEAD: i32 = cm(450);
/// The cone thins out to nothing over this much before the ape, so what
/// the ape is touching stays.
pub const FADE: i32 = cm(150);
/// The share of each width that is cut solid (Q16); past it, dithered.
pub const SOLID: i32 = ONE * 55 / 100;
/// What stays under the ape: everything below its feet and a little over
/// them (the deck, branch or floor it stands on).
pub const FLOOR_OVER_FEET: i32 = cm(15);
/// The kinds of draw the cone cuts (the scenery: trees, buildings, temple,
/// ships), and those cut only by the eye (the vines and ropes, which a
/// swinging ape needs to see ahead but which read as thick poles right by
/// the lens, BR7.2). The ground, the apes, items and glows are never cut.
pub const CONE_KIND: u32 = SCENERY;
pub const NEAR_KIND: u32 = VINES;

/// The view values the hole adds: the ape's middle (3) and the floor's
/// height (1); the sizes `NEAR`, `LENS`, `HEAD`, `FADE` (4); `SOLID`, the
/// cone's kind and the eye's kind (3).
pub const VALUES: usize = 11;

/// A hole round the ape at `focus`, keeping everything under `floor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hole {
    pub focus: V3,
    pub floor: i32,
}

impl Hole {
    /// The hole round an ape whose hands are at `hands` (`Ape::pos`).
    pub fn on(hands: V3) -> Hole {
        Hole {
            focus: hands.sub(V3::UP.scale(BODY_DROP / 2)),
            floor: hands.y - BODY_DROP + FLOOR_OVER_FEET,
        }
    }
}

/// The view values for `hole` (all zero, nothing cut, without one: the
/// menus, the title's flyover, the gallery).
pub fn values(hole: Option<Hole>) -> [i32; VALUES] {
    match hole {
        None => [0; VALUES],
        Some(h) => [
            h.focus.x,
            h.focus.y,
            h.focus.z,
            h.floor,
            NEAR,
            LENS,
            HEAD,
            FADE,
            SOLID,
            CONE_KIND as i32 * ONE,
            NEAR_KIND as i32 * ONE,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::int;
    use crate::geom::v3;

    #[test]
    fn the_hole_is_round_the_apes_body_and_keeps_its_floor() {
        let h = Hole::on(v3(int(3), int(10), int(-4)));
        assert_eq!(h.focus, v3(int(3), int(10) - BODY_DROP / 2, int(-4)));
        // the feet are BODY_DROP under the hands: the floor is just over them
        assert_eq!(h.floor, int(10) - BODY_DROP + FLOOR_OVER_FEET);
        let v = values(Some(h));
        assert_eq!(v[..4], [h.focus.x, h.focus.y, h.focus.z, h.floor]);
        assert!(v[4] > 0, "on");
        assert_eq!(values(None), [0; VALUES]);
    }

    #[test]
    fn the_hole_is_generous() {
        // the person: long-time-comin's (1.6 m wide at the head, 0.9 m
        // round the lens, 6.4 m back) is much too small. At the ape ours
        // spans at least a third of the chase camera's distance on each
        // side, and the eye's sphere holds a mast or trunk right by it.
        // as the page gets them: near, lens, head, fade, solid
        let v = values(Some(Hole::on(V3::ZERO)));
        let [near, lens, head, fade, solid] = [v[4], v[5], v[6], v[7], v[8]];
        assert!(head * 3 >= cm(1000) + cm(300), "{head}");
        assert!(near >= cm(300) && lens >= cm(200), "{near} {lens}");
        assert!(solid >= ONE / 2, "solid over half its width");
        assert!(fade < head, "the cone reaches near the ape");
    }
}
