//! Clean-Up on Aisle Ape's scenery and light (`grocery.rs` is the contract
//! it draws from, with the store's solids in `Course.ravine`).
//!
//! **The light (step 5):** a supermarket at opening time, lit only by its
//! rows of strip lights: an even, slightly cool white from overhead, the
//! cream floor and shelving bouncing a warm light back up, and a faint
//! haze down the long aisles. The frozen aisle is colder and bluer
//! (`frozen_look`, blended in by place as the Ravine's caves are).
//!
//! The Ravine's art never runs on this map: the scene checks
//! `Course.grocery` before `Course.ravine` everywhere it draws.

pub mod course;
pub mod fixtures;
pub mod lobby;
pub mod lot;
pub mod movers;
pub mod shelves;
#[cfg(test)]
mod tests;
pub mod walls;

use crate::course::Course;
use crate::fx::{self, cm, int, ONE};
use crate::geom::{v3, V3};
use crate::grocery::GroceryLayout;
use crate::render::{mix, rgb, Batch, Rgb};
use crate::scene::{Look, Scenery};

/// The ceiling's tiles (and the haze's base: the far end of an aisle fades
/// to the ceiling's colour, as under real strip lights).
pub const CEILING_TILE: Rgb = rgb(228, 226, 220);
/// The haze down the aisles: a warm light grey.
pub const HAZE: Rgb = rgb(214, 210, 200);
/// The strip lights' light: a slightly cool white.
pub const STRIP_LIGHT: Rgb = rgb(250, 250, 244);
/// The floor's and the shelving's bounce: warm cream.
pub const BOUNCE: Rgb = rgb(196, 188, 170);
/// The frozen aisle's light and haze: cold blue-white.
pub const FROZEN_LIGHT: Rgb = rgb(214, 232, 255);
pub const FROZEN_HAZE: Rgb = rgb(186, 210, 228);

/// The vines: sour-apple licorice, a bright green that stands out on the
/// cream shelving and the many-coloured products.
pub const VINE: Rgb = rgb(96, 214, 64);
/// The catch-up colour (T66): a hot magenta, apart from the green vines,
/// the yellow in-reach look, the item flowers' violet and the red banners;
/// in the colourblind mode a strong blue.
pub const CATCH_UP: Rgb = rgb(255, 40, 170);
pub const CATCH_UP_VIVID: Rgb = rgb(20, 60, 255);

/// The light before a course is known, and on it.
pub fn look() -> Look {
    Look {
        sky: CEILING_TILE,
        fog: HAZE,
        // the store's far wall shows down a long aisle, softened
        fog_range: (int(90), int(250)),
        // the strip lights overhead, a little toward the back of the store
        light_dir: v3(cm(30), cm(100), cm(18)).norm(),
        light: STRIP_LIGHT,
        ground: BOUNCE,
        rim: [0; 3],
        far: crate::render::FAR,
    }
}

/// The light on this course (the same everywhere but the frozen aisle).
pub fn look_for(_c: &Course) -> Look {
    look()
}

/// The frozen aisle's light: colder, bluer, a frosty haze.
pub fn frozen_look() -> Look {
    Look {
        sky: rgb(214, 226, 236),
        fog: FROZEN_HAZE,
        fog_range: (int(110), int(420)),
        light: FROZEN_LIGHT,
        ground: rgb(170, 190, 210),
        ..look()
    }
}

/// How far into the frozen aisle `p` is, 0..=ONE: full between the two
/// freezer runs, fading over 30 m past their ends.
pub fn frozen_share(g: &GroceryLayout, p: V3) -> i32 {
    let (Some(a), Some(b)) = (
        g.gondolas
            .iter()
            .find(|r| r.run == crate::grocery::Run::Freezer),
        g.gondolas
            .iter()
            .filter(|r| r.run == crate::grocery::Run::Freezer)
            .nth(1),
    ) else {
        return 0;
    };
    let (x, z) = crate::grocery::plan_of(p);
    let (_, z0) = crate::grocery::plan_of(a.line[0]);
    let (_, z1) = crate::grocery::plan_of(a.line[2]);
    let w = crate::grocery::plan_of(crate::grocery::gondola_at(a, z.clamp(z0, z1))).0;
    let e = crate::grocery::plan_of(crate::grocery::gondola_at(b, z.clamp(z0, z1))).0;
    if x < w || x > e {
        return 0;
    }
    let past = (z0 - z).max(z - z1).max(0);
    ONE - fx::div(past, int(30)).min(ONE)
}

/// The parking lot's light (GR-T4): a clear afternoon, a blue sky, warm
/// sun from the west, the haze far off.
pub fn outdoor_look() -> Look {
    Look {
        sky: rgb(150, 196, 236),
        fog: rgb(198, 216, 230),
        fog_range: (int(140), int(460)),
        light_dir: v3(cm(-40), cm(100), cm(20)).norm(),
        light: rgb(255, 246, 226),
        ground: rgb(140, 146, 130),
        ..look()
    }
}

/// How far out of the store `p` is, 0..=ONE: none inside the front wall,
/// all of it 40 m out.
pub fn outdoor_share(p: V3) -> i32 {
    let (_, z) = crate::grocery::plan_of(p);
    let out = int(crate::grocery::layout::SOUTH) - z;
    fx::div(out.clamp(0, int(40)), int(40))
}

/// The light for an eye at `eye`.
pub fn look_at(_c: &Course, g: &GroceryLayout, eye: V3) -> Look {
    let out = outdoor_share(eye);
    if out > 0 {
        return blend(&look(), &outdoor_look(), out);
    }
    let k = frozen_share(g, eye);
    if k <= 0 {
        return look();
    }
    blend(&look(), &frozen_look(), k)
}

fn blend(a: &Look, b: &Look, t: i32) -> Look {
    let lerp = |x: i32, y: i32| fx::lerp(x, y, t);
    Look {
        sky: mix(a.sky, b.sky, t),
        fog: mix(a.fog, b.fog, t),
        fog_range: (
            lerp(a.fog_range.0, b.fog_range.0),
            lerp(a.fog_range.1, b.fog_range.1),
        ),
        light_dir: v3(
            lerp(a.light_dir.x, b.light_dir.x),
            lerp(a.light_dir.y, b.light_dir.y),
            lerp(a.light_dir.z, b.light_dir.z),
        )
        .norm(),
        light: mix(a.light, b.light, t),
        ground: mix(a.ground, b.ground, t),
        rim: mix(a.rim, b.rim, t),
        far: a.far,
    }
}

/// The store's still scenery, chunked.
pub fn scenery(c: &Course, g: &GroceryLayout) -> Scenery {
    course::scenery(c, g)
}

/// What moves, every frame: the shoppers and their balloons and carts, the
/// misters' spray.
pub fn dynamic(
    b: &mut Batch,
    c: &Course,
    g: &GroceryLayout,
    eye: V3,
    t: u32,
    tick: u32,
    vivid: bool,
) {
    course::dynamic(b, c, g, eye, t, tick, vivid);
}
