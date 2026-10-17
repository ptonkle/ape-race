//! Rattlesnake Ravine's models (2026-10-17, `briefs/rattlesnake-ravine.md`,
//! step 4 "Models"): stand-alone builders in the faceted low-poly style of
//! `ruinsmodels.rs` and `towermodels.rs`. No course places them yet, and
//! none of them knows the map's contract (`ravine.rs`): each takes plain
//! parameters (a frame, a length, a colour, a seed `k`, the race tick).
//!
//! Each model is built in its own metres, +y up, +z its front (a skull's
//! snout, a cart's way forward, a hole's open side), its feet or base at
//! y 0 unless its doc says otherwise, and placed by a frame (`pose`) the
//! caller passes.
//!
//! **The hour: night.** A starry sky, a low cool moon; the warm light
//! comes from torches, lanterns, the campfire and the glowing mushrooms.
//! Step 5 owns the map's real `Look`; `gallery_look` only lights the
//! model shots.
//!
//! A palette no other map uses: red-orange sandstone, moss green, pale
//! fossil ivory, weathered grey timber, and teal-cyan glows (mushrooms,
//! fireflies) against the warm amber flames.
//!
//! Only big pieces are solid (the person, 2026-10-17): the T-rex skull and
//! its teeth, the sphinx, the wall holes' walls. The skull's teeth come
//! from one list (`rex_teeth`) for the solids and the drawing alike. The
//! dressing (props, mushrooms, torches, tents, the campfire) has no
//! collision.
//!
//! The parts, by file: `ravinemodels/snake.rs` the snake vine and its
//! snapped halves; `lizard.rs` the wall lizards; `mine.rs` the mine cart,
//! track, wall hole and mining props; `skull.rs` the T-rex skull;
//! `finish.rs` the sphinx, campfire and tents; `dressing.rs` the canyon's
//! bridges, logs, bones, mushrooms, torches and fireflies.

use crate::fx::{deg, int, ONE};
use crate::geom::{facing, v3, Affine, V3};
use crate::render::{rgb, Batch, Rgb};
use crate::scene::Look;

pub use crate::ruinsmodels::pose;
pub(crate) use crate::ruinsmodels::{pick, wave};
pub(crate) use crate::towermodels::{bar, dm, halo_ball, put, put_turned};

mod dressing;
mod finish;
mod lizard;
mod mine;
mod skull;
mod snake;

pub use dressing::*;
pub use finish::*;
pub use lizard::*;
pub use mine::*;
pub use skull::*;
pub use snake::*;

#[cfg(test)]
mod tests;

// ---------------------------------------------------------------------------
// The gallery's light (step 5 owns the map's).

/// The moon's height in the gallery.
pub const MOON_HIGH: i32 = deg(35);
/// The moon's bearing in the gallery.
pub const MOON_YAW: i32 = deg(30);
pub const SKY: Rgb = rgb(22, 26, 58);
pub const FOG: Rgb = rgb(40, 40, 76);
/// Moonlight: cool, bright enough to read the models.
pub const LIGHT: Rgb = rgb(250, 250, 300);
/// The canyon's bounce light: warm from the sandstone.
pub const AMBIENT: Rgb = rgb(150, 110, 110);

/// Night light for the model gallery (`?gallery=snakevine`).
pub fn gallery_look() -> Look {
    Look {
        sky: SKY,
        fog: FOG,
        fog_range: (int(300), int(900)),
        light_dir: facing(MOON_YAW, MOON_HIGH),
        light: LIGHT,
        ground: AMBIENT,
        rim: [0; 3],
        far: crate::render::FAR,
    }
}

// ---------------------------------------------------------------------------
// The palette.

pub(crate) const SANDSTONE: Rgb = rgb(196, 98, 58);
pub(crate) const SANDSTONE_DARK: Rgb = rgb(140, 64, 40);
pub(crate) const MOSS: Rgb = rgb(70, 110, 46);
pub(crate) const BONE: Rgb = rgb(226, 212, 176);
pub(crate) const BONE_DARK: Rgb = rgb(170, 150, 116);
pub(crate) const TIMBER: Rgb = rgb(128, 104, 80);
pub(crate) const TIMBER_DARK: Rgb = rgb(86, 68, 52);
pub(crate) const ROPE: Rgb = rgb(196, 168, 118);
pub(crate) const IRON: Rgb = rgb(70, 66, 64);
pub(crate) const RUST: Rgb = rgb(124, 64, 40);
pub(crate) const GOLD: Rgb = rgb(255, 204, 60);
pub(crate) const FLAME: Rgb = rgb(255, 150, 40);
pub(crate) const FLAME_CORE: Rgb = rgb(255, 230, 140);
pub(crate) const VOID: Rgb = rgb(6, 4, 6);
pub(crate) const BANANA: Rgb = rgb(250, 214, 60);
pub(crate) const GLOW_TEAL: Rgb = rgb(80, 255, 220);

// ---------------------------------------------------------------------------
// Helpers.

/// A frame at `at` whose +y is the surface normal `normal` and whose +z is
/// as near to `up` as it can be: a creature lying flat on a wall, head up.
pub fn on_surface(at: V3, normal: V3, up: V3) -> Affine {
    Affine::translate(at).then(&Affine::looking(up.reject(normal.norm()), normal))
}

// ---------------------------------------------------------------------------
// The gallery (`?gallery=<key>`).

pub const GALLERY: [&str; 25] = [
    "snakevine",
    "snakesnap",
    "snakes",
    "lizard",
    "lizards",
    "minecart",
    "minetrack",
    "wallhole",
    "rexskull",
    "sphinx",
    "campfire",
    "tents",
    "pickaxe",
    "lantern",
    "orebucket",
    "minesupport",
    "digmarks",
    "minegear",
    "ropebridge",
    "logspan",
    "fossils",
    "mushrooms",
    "torch",
    "fireflies",
    "mesacamp",
];

/// The middle and rough radius of a gallery model, metres.
pub fn framing(key: &str) -> (V3, i32) {
    match key {
        "snakevine" => (v3(0, -int(5), 0), int(7)),
        "snakesnap" => (v3(0, -int(6), 0), int(8)),
        "snakes" => (v3(0, -int(5), 0), int(9)),
        "lizard" => (v3(0, int(1), 0), int(3)),
        "lizards" => (v3(0, int(5), 0), int(8)),
        "minecart" => (v3(0, int(1), 0), int(3)),
        "minetrack" => (v3(0, int(2), 0), int(9)),
        "wallhole" => (v3(0, int(3), 0), int(7)),
        "rexskull" => (v3(0, int(2), 0), int(11)),
        "sphinx" => (v3(0, int(9), 0), int(26)),
        "campfire" => (v3(0, int(1), 0), int(3)),
        "tents" => (v3(0, int(1), 0), int(8)),
        "pickaxe" | "lantern" | "orebucket" => (v3(0, ONE / 2, 0), ONE * 3 / 2),
        "minesupport" => (v3(0, int(3), 0), int(5)),
        "digmarks" => (v3(0, int(2), 0), int(4)),
        "minegear" => (v3(0, int(1), 0), int(5)),
        "ropebridge" => (v3(0, 0, 0), int(10)),
        "logspan" => (v3(0, 0, 0), int(10)),
        "fossils" => (v3(0, int(3), 0), int(6)),
        "mushrooms" => (v3(0, ONE / 2, 0), ONE * 2),
        "torch" => (v3(0, int(1), 0), ONE * 2),
        "fireflies" => (v3(0, int(2), 0), int(4)),
        "mesacamp" => (v3(0, int(6), int(6)), int(30)),
        _ => (V3::ZERO, int(5)),
    }
}

/// A plank hung across the gallery's snake vines and creatures: something
/// for the snakes to hang from.
fn ledge(b: &mut Batch, f: &Affine, w: i32) {
    bar(
        b,
        f,
        Mesh::Taper,
        v3(-w / 2, 0, 0),
        v3(w / 2, ONE / 5, 0),
        ONE / 3,
        TIMBER_DARK,
        0,
    );
}

use crate::mesh::Mesh;

/// Draws gallery model `key` in frame `f` (turning) at `tick`.
pub fn gallery(b: &mut Batch, key: &str, f: &Affine, tick: u32) {
    let at = |x: i32, y: i32, z: i32| f.then(&Affine::translate(v3(x, y, z)));
    match key {
        "snakevine" => {
            ledge(b, f, int(3));
            snake_vine(b, f, int(8), tick, 1);
        }
        "snakesnap" => {
            ledge(b, f, int(9));
            snake_snapped(b, &at(-int(3), 0, 0), int(8), int(3), tick, 1);
            snake_vine(b, &at(int(3), 0, 0), int(8), tick, 2);
        }
        "snakes" => {
            // a snake beside two plain vine stand-ins: the shape cues side by side
            ledge(b, f, int(10));
            for (i, x) in [-3, 0, 3].into_iter().enumerate() {
                let g = at(int(x), 0, 0);
                if i == 1 {
                    snake_vine(b, &g, int(10), tick, 3);
                } else {
                    plain_vine(b, &g, int(10));
                }
            }
        }
        "lizard" => lizard(b, f, lizard_phase(tick, 0), LizardHue::Red),
        "lizards" => {
            // three on a wall (the +z face of a slab), climbing
            put(
                b,
                f,
                Mesh::Cube,
                v3(0, int(5), -ONE / 2),
                v3(int(14), int(12), ONE),
                SANDSTONE,
                0,
            );
            for (i, hue) in LizardHue::ALL.into_iter().enumerate() {
                let x = int(4) * (i as i32 - 1);
                let g = f.then(&on_surface(
                    v3(x, int(3 + i as i32 * 2), 0),
                    v3(0, 0, ONE),
                    V3::UP,
                ));
                lizard(b, &g, lizard_phase(tick, i as u32), hue);
            }
        }
        "minecart" => {
            mine_track(b, f, v3(0, 0, -int(3)), v3(0, 0, int(3)), None, 1);
            mine_cart(b, &at(0, RAIL_TOP, 0), tick, 1, true);
        }
        "minetrack" => {
            mine_track(b, f, v3(0, int(4), -int(12)), v3(0, int(2), 0), Some(0), 2);
            mine_track(b, f, v3(0, int(2), 0), v3(int(4), 0, int(10)), Some(0), 3);
            mine_cart(b, &at(0, int(3) + RAIL_TOP, -int(6)), tick, 2, false);
        }
        "wallhole" => {
            wall_hole(b, f, int(5), int(5), 4);
            mine_track(b, f, v3(0, 0, -int(3)), v3(0, 0, int(6)), None, 4);
        }
        "rexskull" => {
            let gap = REX_GAP;
            rex_skull(b, f, gap, &rex_teeth(gap));
        }
        "sphinx" => sphinx(b, f),
        "campfire" => campfire(b, f, tick, 1),
        "tents" => {
            tent(
                b,
                &at(-int(3), 0, 0).then(&Affine::rot_y(deg(20))),
                TENT_CANVAS,
                1,
            );
            tent(
                b,
                &at(int(3), 0, -int(2)).then(&Affine::rot_y(-deg(30))),
                TENT_RED,
                2,
            );
        }
        "pickaxe" => pickaxe(b, f, deg(20)),
        "lantern" => lantern(b, f, tick, 1),
        "orebucket" => ore_bucket(b, f, 1),
        "minesupport" => mine_support(b, f, int(6), int(5), 1),
        "digmarks" => dig_marks(b, f, int(4), int(3), tick, 1),
        "minegear" => {
            mine_support(b, f, int(6), int(5), 2);
            dig_marks(b, &at(0, int(1), -int(2)), int(3), int(2), tick, 2);
            pickaxe(b, &at(-int(2), 0, 0), deg(30));
            ore_bucket(b, &at(int(1), 0, ONE), 2);
            ore_bucket(b, &at(int(2), 0, 0), 3);
            lantern(b, &at(-int(2), int(4), 0), tick, 2);
        }
        "ropebridge" => rope_bridge_span(b, f, int(16), int(3), tick, 1),
        "logspan" => log_span(b, f, int(16), ONE * 6 / 10, 1),
        "fossils" => {
            put(
                b,
                f,
                Mesh::Cube,
                v3(0, int(3), -ONE / 2),
                v3(int(10), int(7), ONE),
                SANDSTONE,
                0,
            );
            fossil_inlay(b, &at(-int(2), int(3), 0), Fossil::Ribs, 1);
            fossil_inlay(b, &at(int(2), int(4), 0), Fossil::Spine, 2);
            fossil_inlay(b, &at(int(2), int(1), 0), Fossil::Ammonite, 3);
        }
        "mushrooms" => mushrooms(b, f, tick, 1),
        "torch" => torch(b, f, tick, 1),
        "fireflies" => fireflies(b, f, int(3), tick, 1),
        "mesacamp" => {
            sphinx(b, f);
            campfire(b, &at(int(4), 0, int(22)), tick, 1);
            tent(
                b,
                &at(-int(4), 0, int(24)).then(&Affine::rot_y(deg(40))),
                TENT_CANVAS,
                1,
            );
            tent(
                b,
                &at(int(10), 0, int(26)).then(&Affine::rot_y(-deg(60))),
                TENT_RED,
                2,
            );
        }
        _ => {}
    }
}
