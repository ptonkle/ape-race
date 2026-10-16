//! The zipper space, the descent (the brief: like the zip in SpongeBob's
//! "Shanghaied", the climb mirrored and topsy-turvy, eerie sea-green and
//! violet): its light, the giant zip running down the void, the tower's
//! pieces hung upside down and tilted in a sickly palette, and the
//! floating debris.

use super::{bar, halo_ball, pick, put, put_turned, round, wave};
use super::{keep_crown, keep_section, Detail, SECTION_H};
use crate::fx::{cm, deg, int, mul, HALF, ONE, TURN};
use crate::geom::{facing, v3, Affine};
use crate::mesh::Mesh;
use crate::render::{mix, rgb, Batch, Rgb};
use crate::scene::Look;

/// The void's deep sea-green, overhead and below.
pub const DEEP_SKY: Rgb = rgb(20, 44, 54);
pub const DEEP_FOG: Rgb = rgb(38, 70, 78);
/// A cold sea-green light from high up one side.
pub const DEEP_LIGHT: Rgb = rgb(150, 250, 210);
/// Violet from the void all round.
pub const DEEP_AMBIENT: Rgb = rgb(100, 62, 146);

const SICK_DARK: Rgb = rgb(28, 50, 58);
const SICK_PALE: Rgb = rgb(150, 198, 172);
const SICK_VIOLET: Rgb = rgb(86, 46, 124);
const GLOW_SEA: Rgb = rgb(110, 255, 200);
const TOOTH: Rgb = rgb(206, 214, 178);
const TOOTH_DULL: Rgb = rgb(170, 180, 150);
const TAPE: Rgb = rgb(58, 32, 88);
const SLIDER: Rgb = rgb(112, 186, 164);
const SLIDER_DARK: Rgb = rgb(54, 100, 92);
const STONE_SICK: Rgb = rgb(120, 150, 134);
const WOOD_SICK: Rgb = rgb(78, 70, 64);
const SLATE_SICK: Rgb = rgb(70, 52, 108);
const BUBBLE: Rgb = rgb(170, 250, 230);

/// The zipper space's light, for the gallery (step 5 owns the map's).
pub fn zipper_look() -> Look {
    Look {
        sky: DEEP_SKY,
        fog: DEEP_FOG,
        fog_range: (int(200), int(800)),
        light_dir: facing(deg(-60), deg(40)),
        light: DEEP_LIGHT,
        ground: DEEP_AMBIENT,
        rim: rgb(40, 20, 64),
        far: crate::render::FAR,
    }
}

/// A colour of the climb turned sickly: lit things glow sea-green, the
/// rest goes to sea-green greys by brightness with violet in the shade.
pub fn sickly(c: Rgb, glow: i32) -> Rgb {
    if glow > ONE / 2 {
        return mix(c, GLOW_SEA, ONE * 3 / 4);
    }
    let lum = ((c[0] + c[1] + c[2]) / 3).clamp(0, ONE);
    let tone = mix(SICK_DARK, SICK_PALE, lum);
    mix(mix(tone, c, ONE / 5), SICK_VIOLET, (ONE - lum) / 3)
}

/// Draws `draw` hung upside down from frame `f`: turned `yaw` about the
/// up axis, flipped and tilted `tilt` off straight down, so the piece's
/// foot is at the origin and it hangs below; then recoloured `sickly`.
pub fn topsy(
    b: &mut Batch,
    f: &Affine,
    yaw: i32,
    tilt: i32,
    draw: impl FnOnce(&mut Batch, &Affine),
) {
    let marks = b.marks();
    let g = f
        .then(&Affine::rot_y(yaw))
        .then(&Affine::rot_z(HALF + tilt));
    draw(b, &g);
    b.recolour_since(&marks, sickly);
}

/// A stretch of the keep hung upside down and tilted: `sections` keep
/// sections (apothem `r`) hanging from the origin and the crown with its
/// turrets pointing down at the bottom. About 85 shapes a section and 160
/// for the crown.
pub fn topsy_keep(b: &mut Batch, f: &Affine, r: i32, sections: u32, tilt: i32, k: u32) {
    topsy(b, f, deg(17) * k as i32, tilt, |b, g| {
        for s in 0..sections {
            let at = g.then(&Affine::translate(v3(0, SECTION_H * s as i32, 0)));
            keep_section(b, &at, r, SECTION_H, k + s, Detail::Near);
        }
        let top = g.then(&Affine::translate(v3(0, SECTION_H * sections as i32, 0)));
        keep_crown(b, &top, r, 4, k);
    });
}

/// How many teeth each side of the zip has.
pub const ZIP_TEETH: i32 = 40;

/// The giant zip hanging down the void, `len` long from its top (y 0)
/// down to y -len, in the xy plane facing +z, `w` across the meshed
/// teeth. `open` 0..ONE is how far down the slider has been pulled: above
/// it the two rows of teeth part in a V on their violet tapes, below it
/// they mesh. A big slider, and its pull tab hanging and swaying. About
/// 95 shapes.
pub fn zip(b: &mut Batch, f: &Affine, len: i32, w: i32, open: i32, tick: u32) {
    let n = ZIP_TEETH;
    let step = len / n;
    let slider_y = -mul(len, open.clamp(0, ONE));
    // half the gap between the rows at height y
    let spread = |y: i32| {
        if y > slider_y {
            (y - slider_y) / 3
        } else {
            0
        }
    };
    for s in [-1, 1] {
        let tape = |y: i32| v3(s * (w * 3 / 4 + spread(y)), y, -cm(30));
        bar(b, f, Mesh::Cube, tape(0), tape(slider_y), w / 2, TAPE, 0);
        bar(b, f, Mesh::Cube, tape(slider_y), tape(-len), w / 2, TAPE, 0);
        for i in 0..n {
            let y = -step * i - if s > 0 { step / 2 } else { 0 } - step / 4;
            let at = v3(s * (w / 5 + spread(y)), y, 0);
            put(
                b,
                f,
                Mesh::Bevel,
                at,
                v3(w * 3 / 5, step * 9 / 20, w / 3),
                if i % 5 == 2 { TOOTH_DULL } else { TOOTH },
                ONE / 8,
            );
        }
    }
    // the stop at the bottom
    put(
        b,
        f,
        Mesh::Bevel,
        v3(0, -len - step / 2, 0),
        v3(w * 2, step, w / 2),
        SLIDER_DARK,
        0,
    );
    // the slider and its pull tab
    let sl = v3(0, slider_y + step, 0);
    put(
        b,
        f,
        Mesh::Bevel,
        sl,
        v3(w * 2, step * 3, w),
        SLIDER,
        ONE / 6,
    );
    put(
        b,
        f,
        Mesh::Wedge,
        sl.add(v3(0, step * 2, 0)),
        v3(w * 9 / 5, step, w * 4 / 5),
        SLIDER_DARK,
        0,
    );
    let pivot = sl.add(v3(0, -step, w / 2 + cm(20)));
    put(
        b,
        f,
        Mesh::Cyl,
        pivot,
        v3(w / 3, w / 2, w / 3),
        SLIDER_DARK,
        0,
    );
    let tab = f
        .then(&Affine::translate(pivot))
        .then(&Affine::rot_x(mul(wave(tick, 4, 0), deg(18))))
        .then(&Affine::rot_z(mul(wave(tick, 6, 9_000), deg(10))));
    put(
        b,
        &tab,
        Mesh::Bevel,
        v3(0, -w * 3 / 2, 0),
        v3(w, w * 5 / 2, w / 5),
        SLIDER,
        ONE / 4,
    );
    put(
        b,
        &tab,
        Mesh::Cube,
        v3(0, -w * 2, 0),
        v3(w / 2, w * 3 / 4, w / 4),
        DEEP_SKY,
        0,
    );
    halo_ball(b, f, sl, w * 4, GLOW_SEA, ONE / 4);
}

/// Floating debris round the void, within `r` of the origin: blocks of
/// sickly stone, broken planks, barrels, slates and bubbles, tumbling
/// slowly and bobbing. About 18 shapes.
pub fn debris(b: &mut Batch, f: &Affine, r: i32, tick: u32, k: u32) {
    for j in 0..18u32 {
        let kk = k * 31 + j;
        let a = TURN * pick(kk, 1, 0, 999) / 1000;
        let d = r * pick(kk, 2, 25, 100) / 100;
        let y = r * pick(kk, 3, -60, 60) / 100 + mul(wave(tick, 7, j as i32 * 5_000), cm(60));
        let tumble = (tick as i32).wrapping_mul(TURN / 3_000 * (1 + (j % 3) as i32));
        let turn = Affine::rot_y(a + tumble).then(&Affine::rot_x(tumble / 2 + j as i32 * 9_000));
        let (mesh, size, c, glow) = match j % 6 {
            0 => (
                Mesh::Rock,
                v3(int(2), int(1) + cm(40), int(2)),
                STONE_SICK,
                0,
            ),
            1 => (Mesh::Cube, v3(int(3), cm(20), cm(50)), WOOD_SICK, 0),
            2 => (Mesh::Barrel, v3(cm(90), cm(120), cm(90)), WOOD_SICK, 0),
            3 => (Mesh::Bevel, v3(cm(140), int(1), int(1)), STONE_SICK, 0),
            4 => (Mesh::Pyramid, v3(int(2), int(1), int(2)), SLATE_SICK, 0),
            _ => (Mesh::Sphere, v3(cm(80), cm(80), cm(80)), BUBBLE, ONE / 2),
        };
        put_turned(b, f, mesh, round(d, a, y), &turn, size, c, glow);
    }
}
