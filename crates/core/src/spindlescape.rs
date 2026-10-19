//! The Spindle's drawing (`spindle.rs`): a dropper map's block tube, drawn
//! in the flat frame and bent onto the cylinder once (`Warp::affine`): the
//! still scenery when it is built (`scenery`), the moving shapes every
//! frame (the app's view, `bend`).
//!
//! **The light (this map's time of day):** none outside: the tube hangs in
//! Sector Ape's black space. Inside, an even, bright block light: the
//! lamp blocks set in the wall and the axle, the same from every side (the
//! ambient's sky and ground are one colour, so the floor overhead is lit as
//! the floor underfoot), a light from behind the apes so the blocks' faces
//! toward them read, and a pale haze that whitens the far tube toward the
//! glare of the open end. No weather.

pub mod lobby;
pub mod space;
#[cfg(test)]
mod tests;

use crate::course::{Course, START_HEIGHT};
use crate::fx::{cm, int, Dice, ONE};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{placed, rgb, Batch, Camera, Rgb};
use crate::scene::{Look, Scenery};
use crate::spindle::{SpindleLayout, Warp, AXLE_UNDER, CIRCUMFERENCE};

/// The dye colours of the wool blocks (`BlockKind::Wool`).
pub const DYES: [Rgb; 12] = [
    rgb(236, 236, 236),
    rgb(240, 118, 20),
    rgb(190, 68, 180),
    rgb(58, 176, 218),
    rgb(248, 198, 40),
    rgb(112, 186, 26),
    rgb(238, 140, 172),
    rgb(22, 138, 146),
    rgb(122, 42, 174),
    rgb(52, 58, 160),
    rgb(84, 110, 28),
    rgb(160, 38, 34),
];
/// Stone, for the plain courses of the wall.
pub const STONE: Rgb = rgb(128, 128, 132);
/// The axle: dark obsidian.
pub const OBSIDIAN: Rgb = rgb(30, 22, 46);
/// Lamp blocks: glowstone yellow.
pub const LAMP: Rgb = rgb(255, 222, 130);
/// The vines' dispensers, on the axle.
pub const DISPENSER: Rgb = rgb(104, 104, 110);
/// Slime.
pub const SLIME: Rgb = rgb(118, 212, 94);
/// Glass.
pub const GLASS: Rgb = rgb(196, 236, 246);

/// The vines: a vivid green, apart from every dye but lime, which no wall
/// section uses (`SECTIONS`).
pub const VINE: Rgb = rgb(70, 255, 120);
/// The catch-up colour (T66): hot magenta; in the colourblind mode a
/// strong blue.
pub const CATCH_UP: Rgb = rgb(255, 30, 160);
pub const CATCH_UP_VIVID: Rgb = rgb(20, 60, 255);

/// The light inside: one colour from every side.
const AMBIENT: Rgb = rgb(206, 208, 218);
/// The haze toward the open end.
const HAZE: Rgb = rgb(236, 240, 250);

/// Wall tiles round the tube, and their length along it.
pub const TILES_AROUND: i32 = 24;
pub const TILE_LONG: i32 = int(8);
/// The axle's blocks are this long.
pub const AXLE_LONG: i32 = int(24);
/// The wall's blocks glow a little, so their dyes stay bright in the
/// even light (a dropper map is bright).
const WALL_GLOW: i32 = ONE / 5;
/// The wall's tiles are this thick, under the floor.
pub const TILE_THICK: i32 = int(2);
/// The tube's wall sections: each this long, in its dyes (dropper levels,
/// one after another).
pub const SECTION: i32 = int(250);
const SECTIONS: [[u8; 3]; 8] = [
    [0, 3, 9],
    [1, 4, 0],
    [9, 3, 0],
    [2, 6, 8],
    [7, 3, 0],
    [11, 1, 0],
    [8, 2, 6],
    [4, 1, 11],
];
/// A lamp block every this many tiles along and round.
const LAMP_EVERY: i32 = 4;
/// Still shapes are drawn this far from the eye (the haze closes in
/// before it).
pub const REACH: i32 = int(220);

/// The light before a course is known, and on it.
pub fn look() -> Look {
    Look {
        sky: AMBIENT,
        fog: HAZE,
        fog_range: (int(110), REACH),
        // from behind the apes, along the tube
        light_dir: v3(0, 0, -ONE),
        light: rgb(255, 252, 240),
        ground: AMBIENT,
        rim: [0; 3],
        far: crate::render::FAR,
    }
}

/// The light for an eye at `eye` on the course: the tube's block light,
/// going over to Sector Ape's space (its black sky, its sun far below, its
/// clear distance) over the last `OPENING` before the open end.
pub fn look_at(_c: &Course, s: &SpindleLayout, eye: V3) -> Look {
    let into = eye.z - (s.mouth - OPENING);
    if into <= 0 {
        return look();
    }
    space::blend(
        &look(),
        &space::look(),
        crate::fx::div(into.min(OPENING), OPENING),
    )
}

/// Over this much before the open end the tube's light gives way to space.
pub const OPENING: i32 = int(240);

/// A box in the flat frame from (`x` ± `w`/2, `y0`..`y1`, `z0`..`z1`),
/// narrowed round the tube for its height so that tiles side by side meet
/// once bent.
pub fn tile(
    b: &mut Batch,
    w: &Warp,
    x: i32,
    width: i32,
    y: (i32, i32),
    z: (i32, i32),
    c: Rgb,
    glow: i32,
) {
    let mid = (y.0 + y.1) / 2;
    let size = v3(crate::fx::mul(width, w.narrow(mid)), y.1 - y.0, z.1 - z.0);
    b.push(
        Mesh::Cube,
        &placed(v3(x, mid, (z.0 + z.1) / 2), 0, size),
        c,
        glow,
    );
}

/// A wall standing from the floor to the axle across the tube at `z`, `x`
/// ± `width`/2 round it, built of pieces narrowing toward the axle.
pub fn upright(
    b: &mut Batch,
    w: &Warp,
    x: i32,
    width: i32,
    z: (i32, i32),
    y: (i32, i32),
    c: Rgb,
    glow: i32,
) {
    let pieces = ((y.1 - y.0) / int(6)).max(1);
    for k in 0..pieces {
        let y0 = y.0 + (y.1 - y.0) * k / pieces;
        let y1 = y.0 + (y.1 - y.0) * (k + 1) / pieces;
        tile(b, w, x, width, (y0, y1), z, c, glow);
    }
}

/// The tube's still scenery: the wall, the axle and its dispensers, the
/// closed back end, the start's blocks and the open end's frame, chunked
/// along the course and bent.
pub fn scenery(c: &Course, s: &SpindleLayout) -> Scenery {
    let n = crate::scene::chunk_count(c);
    let mut batches: Vec<Batch> = (0..n).map(|_| Batch::new()).collect();
    let chunk = |z: i32| crate::scene::chunk_of(c, v3(0, 0, z), n);
    let w = &s.warp;
    let mut dice = Dice(0x5_D1E5_CA9E);
    let step = CIRCUMFERENCE / TILES_AROUND;
    // the wall, a ring of blocks every `TILE_LONG`, each of its section's
    // dyes at random, lamps in a grid
    let mut z = s.back;
    let mut row = 0;
    while z < s.mouth {
        let z1 = (z + TILE_LONG).min(s.mouth);
        let b = &mut batches[chunk((z + z1) / 2)];
        let dyes = SECTIONS[((z.max(0) / SECTION) as usize) % SECTIONS.len()];
        for k in 0..TILES_AROUND {
            let x = w.wrap_x(k * step - CIRCUMFERENCE / 2 + step / 2);
            let lamp = row % LAMP_EVERY == 0 && k % LAMP_EVERY == (row / LAMP_EVERY) % LAMP_EVERY;
            let base = if lamp {
                LAMP
            } else {
                DYES[dyes[dice.below(dyes.len() as u32) as usize] as usize]
            };
            // each block a touch lighter or darker, as a block's texture
            let shade = 90 + dice.below(20) as i32;
            let glow = if lamp { ONE * 3 / 4 } else { WALL_GLOW };
            tile(
                b,
                w,
                x,
                step,
                (-TILE_THICK, 0),
                (z, z1),
                crate::render::shade(base, shade),
                glow,
            );
        }
        z = z1;
        row += 1;
    }
    // the axle, longer blocks, a lamp now and then
    let mut z = s.back;
    let mut row = 0;
    while z < s.mouth {
        let z1 = (z + AXLE_LONG).min(s.mouth);
        let b = &mut batches[chunk((z + z1) / 2)];
        for k in 0..TILES_AROUND {
            let x = w.wrap_x(k * step - CIRCUMFERENCE / 2 + step / 2);
            let lamp = (k + row) % 6 == 0;
            let (c, glow) = if lamp {
                (LAMP, ONE * 3 / 4)
            } else {
                (OBSIDIAN, ONE / 8)
            };
            tile(
                b,
                w,
                x,
                step,
                (AXLE_UNDER, AXLE_UNDER + TILE_THICK),
                (z, z1),
                c,
                glow,
            );
        }
        z = z1;
        row += 1;
    }
    // a dispenser block on the axle over each vine
    for v in &c.vines {
        let b = &mut batches[chunk(v.anchor.z)];
        tile(
            b,
            w,
            v.anchor.x,
            int(2),
            (v.anchor.y - cm(150), AXLE_UNDER),
            (v.anchor.z - int(1), v.anchor.z + int(1)),
            DISPENSER,
            0,
        );
    }
    // the blocks
    for (k, blk) in s.blocks.iter().enumerate() {
        let b = &mut batches[chunk((blk.lo.z + blk.hi.z) / 2)];
        block(b, w, blk, k as u64);
    }
    // the closed back end: a disc of stone
    let back = &mut batches[0];
    for k in 0..TILES_AROUND {
        let x = w.wrap_x(k * step - CIRCUMFERENCE / 2 + step / 2);
        upright(
            back,
            w,
            x,
            step,
            (s.back - int(2), s.back),
            (0, AXLE_UNDER),
            STONE,
            0,
        );
    }
    // the start's blocks under its platform
    if let Some(pl) = c.platforms.first() {
        let b = &mut batches[0];
        let (hx, hz) = (pl.half_width, pl.half_len);
        for (i, k) in (-2..2).enumerate() {
            let x = pl.centre.x + k * hx / 2 + hx / 4;
            let colour = DYES[[4, 1, 4, 1][i]];
            tile(
                b,
                w,
                x,
                hx / 2,
                (0, START_HEIGHT),
                (pl.centre.z - hz, pl.centre.z + hz),
                colour,
                0,
            );
        }
    }
    // the open end's frame: an obsidian ring round the mouth
    let last = n - 2;
    for k in 0..TILES_AROUND {
        let x = w.wrap_x(k * step - CIRCUMFERENCE / 2 + step / 2);
        let b = &mut batches[last];
        tile(
            b,
            w,
            x,
            step,
            (-int(6), 0),
            (s.mouth, s.mouth + int(3)),
            OBSIDIAN,
            0,
        );
        tile(
            b,
            w,
            x,
            step,
            (AXLE_UNDER, AXLE_UNDER + int(4)),
            (s.mouth, s.mouth + int(3)),
            LAMP,
            ONE / 2,
        );
    }
    for b in &mut batches {
        bend(b, w);
    }
    let mut sc = crate::scene::assemble(c, batches, Vec::new(), Vec::new());
    sc.reach = REACH;
    sc
}

/// A block obstacle (`spindle::Block`), drawn as a dropper map's blocks:
/// cut into block-sized cubes round the tube and up it (each narrowed for
/// its height, so they meet once bent), each a shade lighter or darker.
pub fn block(b: &mut Batch, w: &Warp, blk: &crate::spindle::Block, k: u64) {
    let mut dice = Dice(0xB10C_0000 + k);
    let colour = block_colour(blk.kind);
    let glow = block_glow(blk.kind);
    let wide = blk.hi.x - blk.lo.x;
    let high = blk.hi.y - blk.lo.y;
    let across = (wide / BLOCK).max(1);
    let up = (high / BLOCK).max(1);
    for i in 0..across {
        let x0 = blk.lo.x + wide * i / across;
        let x1 = blk.lo.x + wide * (i + 1) / across;
        for j in 0..up {
            let y0 = blk.lo.y + high * j / up;
            let y1 = blk.lo.y + high * (j + 1) / up;
            let shade = 90 + dice.below(20) as i32;
            tile(
                b,
                w,
                (x0 + x1) / 2,
                x1 - x0,
                (y0, y1),
                (blk.lo.z, blk.hi.z),
                crate::render::shade(colour, shade),
                glow,
            );
        }
    }
}

/// A block obstacle's cubes are about this big.
pub const BLOCK: i32 = int(6);

/// Every shape in `b`, placed in the flat frame, moved onto the tube.
pub fn bend(b: &mut Batch, w: &Warp) {
    b.transform(|a| w.affine(&a));
}

/// The camera, set in the flat frame, as drawn on the tube: its eye bent
/// (kept under the axle, so it never sees through it), turned with the
/// floor under it, and the way up there (toward the axis).
pub fn camera(w: &Warp, cam: &Camera) -> (Camera, V3) {
    let eye = v3(cam.eye.x, cam.eye.y.min(AXLE_UNDER - int(3)), cam.eye.z);
    (
        Camera {
            eye: w.point(eye),
            fwd: w.turn(eye.x, cam.fwd),
            fov: cam.fov,
        },
        w.turn(eye.x, V3::UP),
    )
}

/// What moves, in the flat frame (bent with the rest of the frame): the
/// spinning rings' columns.
pub fn dynamic(b: &mut Batch, c: &Course, s: &SpindleLayout, eye: V3, tick: u32) {
    for sp in &s.spinners {
        if (sp.z - eye.z).abs() > REACH {
            continue;
        }
        for k in 0..sp.count {
            let x = sp.x_at(k, tick);
            let column = crate::spindle::Block {
                lo: v3(x - sp.width / 2, 0, sp.z - sp.depth / 2),
                hi: v3(x + sp.width / 2, AXLE_UNDER, sp.z + sp.depth / 2),
                kind: sp.kind,
            };
            block(b, &s.warp, &column, k as u64);
            // a lamp block at its foot and its head, so it reads moving
            tile(
                b,
                &s.warp,
                x,
                sp.width,
                (0, int(2)),
                (sp.z - sp.depth / 2 - cm(20), sp.z + sp.depth / 2 + cm(20)),
                LAMP,
                ONE * 3 / 4,
            );
        }
    }
    space::dynamic(b, c, s, eye, tick);
}

/// A block's colour.
pub fn block_colour(k: crate::spindle::BlockKind) -> Rgb {
    use crate::spindle::BlockKind::*;
    match k {
        Wool(d) => DYES[d as usize % DYES.len()],
        Glass => GLASS,
        Slime => SLIME,
        Lamp => LAMP,
        Obsidian => OBSIDIAN,
    }
}

/// A block's glow.
pub fn block_glow(k: crate::spindle::BlockKind) -> i32 {
    use crate::spindle::BlockKind::*;
    match k {
        Lamp => ONE * 3 / 4,
        Glass => ONE / 4,
        Slime => ONE / 6,
        _ => 0,
    }
}

/// A shape placed in the world (not on the tube): placed so that bending
/// puts it back where it was (`Warp::affine_of`).
pub fn in_world(b: &mut Batch, w: &Warp, mesh: Mesh, a: &Affine, c: Rgb, glow: i32) {
    b.push(mesh, &w.affine_of(a), c, glow);
}
