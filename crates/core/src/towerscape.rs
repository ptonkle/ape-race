//! Mape's Tower's scenery and light (`tower.rs` is the contract it draws
//! from). STAND-IN (the skeleton, 2026-10-15): the Jungle's scenery and a
//! neutral late-afternoon light; Twilight and Dawn are registered as the
//! map's weathers (`snow::Weather::offered`) but drawn as this light until
//! the art agent gives them their skies (TW7).

use crate::course::Course;
use crate::fx::{cm, int};
use crate::geom::v3;
use crate::render::rgb;
use crate::scene::{Look, Scenery};
use crate::tower::TowerLayout;

/// Mape's Tower's light. STAND-IN: a soft late-afternoon sky, a pale haze.
pub fn look() -> Look {
    Look {
        sky: rgb(120, 140, 200),
        fog: rgb(190, 180, 210),
        fog_range: (int(120), int(320)),
        light_dir: v3(cm(40), cm(70), cm(30)).norm(),
        light: rgb(255, 225, 200),
        ground: rgb(90, 110, 80),
        rim: [0; 3],
        far: crate::render::FAR,
    }
}

/// Mape's Tower at twilight. STAND-IN: the light deepened to violet and
/// dimmed; the art agent gives it its own sky.
pub fn twilight(look: Look) -> Look {
    Look {
        sky: rgb(70, 60, 140),
        fog: rgb(120, 90, 150),
        light: crate::render::shade(look.light, 60),
        ..look
    }
}

/// Mape's Tower at dawn. STAND-IN: a pink-gold sky, a warm low light.
pub fn dawn(look: Look) -> Look {
    Look {
        sky: rgb(240, 170, 160),
        fog: rgb(250, 205, 180),
        light: rgb(255, 200, 150),
        ..look
    }
}

/// Mape's Tower's scenery. STAND-IN: the Jungle's.
pub fn scenery(c: &Course, _t: &TowerLayout) -> Scenery {
    crate::scene::jungle_scenery(c)
}

/// The title's flyover on Mape's Tower: a slow glide along the centreline
/// (up the climb, down the zipper), 14 m over it, looking 30 m on.
pub fn flyover_camera(c: &Course, ticks: u32, from: i32) -> crate::render::Camera {
    let lap = (c.length - int(100)).max(int(1)) as i64;
    let s = ((ticks as i64 * int(6) as i64 / crate::rules::TICKS_PER_SECOND as i64 + from as i64)
        % lap) as i32
        + int(40);
    let eye = c.point(s).add(v3(0, int(14), 0));
    let look = c.point(s + int(30)).add(v3(0, int(11), 0));
    crate::render::Camera {
        eye,
        fwd: look.sub(eye).norm(),
        fov: crate::fx::deg(70),
    }
}
