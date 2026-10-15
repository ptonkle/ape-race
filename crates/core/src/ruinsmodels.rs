//! Sunset Ruins' models and look (2026-10-14, `briefs/sunset-ruins.md`,
//! art phase 1): stand-alone builders for the jungle temple map's props, in
//! the faceted low-poly style of `vehicles.rs` and `spacecraft.rs`. No
//! course places them yet; a later phase does. Each is built in its own
//! metres, +z forward and +y up, then placed by a frame (`pose`) the caller
//! passes, so one builder serves a far landmark and a near prop alike.
//! Nothing here keeps state: the turtles' strokes, the glyphs' pulse, the
//! waterfalls and the hanging foliage come from the tick alone.
//!
//! **The hour: a vibrant sunset, clear, no weather.** The sun hangs 7°
//! over the horizon, a deep gold-orange light raking in low and long; the
//! sky runs from yellow at the horizon through orange and red to crimson
//! overhead (a dome, MAPS B2, with an orange band low down and a corona
//! round the sun); the air is a golden haze; shadows take the sky's rose
//! violet. Inside the temple it is dark (`look_inside`): a dim umber
//! ambient lit by the pulsing banana glyphs.
//!
//! A palette no other map uses: mossy grey-tan stone with olive moss,
//! Aztec sandstone with turquoise and red-ochre paint, deep teal-green
//! jungle against a red sky, and warm yellow glows. (The Jungle is
//! morning green and blue sky, the City dusk violet and cyan, the Ski
//! Slope white and blue, Sector Ape black and neon.)

use crate::fx::{self, cm, deg, int, mul, ratio, sin, ONE, TURN};
use crate::geom::{facing, v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, placed, rgb, rod, shade, Batch, Rgb};
use crate::rules::TICKS_PER_SECOND;
use crate::scene::Look;

// ---------------------------------------------------------------------------
// The look: sunset outside, a dark temple inside.

/// The sun's height over the horizon: low and long.
pub const SUN_HIGH: i32 = deg(7);
/// The sun's bearing in the model gallery (a course turns it its own way).
pub const SUN_YAW: i32 = deg(200);
/// The sunset light: a deep gold-orange, bright.
pub const LIGHT: Rgb = rgb(292, 182, 96);
/// The light from above (the ambient overhead, so the shadows' colour): a
/// rose violet, the complement of the sun, so shade reads cool on warm.
pub const SKY: Rgb = rgb(150, 88, 128);
/// The sky's dome overhead: crimson (`sky`).
pub const ZENITH: Rgb = rgb(132, 16, 50);
/// The low band of the sky, laid over the dome's yellow to orange: orange.
pub const BAND: Rgb = rgb(255, 112, 30);
/// The haze at the horizon, the clear colour and the fog: golden yellow.
pub const FOG: Rgb = rgb(255, 196, 92);
/// The ground's bounce under things: a warm umber.
pub const BOUNCE: Rgb = rgb(132, 82, 56);
/// A faint warm edge light, the sun behind the silhouettes.
pub const RIM: Rgb = rgb(58, 22, 14);
/// The sun's disc.
pub const SUN: Rgb = rgb(255, 238, 170);

/// Inside the temple: a dark umber ambient, a dim warm light (the glyphs
/// and the openings), a near-black brown fog that closes in sooner.
pub const INSIDE_SKY: Rgb = rgb(58, 42, 46);
pub const INSIDE_LIGHT: Rgb = rgb(120, 84, 52);
pub const INSIDE_BOUNCE: Rgb = rgb(86, 62, 30);
pub const INSIDE_FOG: Rgb = rgb(30, 20, 16);

/// Sunset Ruins outside: ready for `scene::look(Map::Ruins)`.
pub fn look() -> Look {
    Look {
        sky: SKY,
        fog: FOG,
        // a golden haze, fairly thick in the jungle's humid air
        fog_range: (int(55), int(200)),
        light_dir: facing(SUN_YAW, SUN_HIGH),
        light: LIGHT,
        ground: BOUNCE,
        rim: RIM,
        far: crate::render::FAR,
    }
}

/// The same with the sun at bearing `sun_yaw` (a course's own).
pub fn look_toward(sun_yaw: i32) -> Look {
    Look {
        light_dir: facing(sun_yaw, SUN_HIGH),
        ..look()
    }
}

/// Inside the temple: darker, the light warm and dim from the openings
/// behind and above, the glyphs giving the rest (they glow).
pub fn look_inside() -> Look {
    Look {
        sky: INSIDE_SKY,
        fog: INSIDE_FOG,
        fog_range: (int(30), int(130)),
        light_dir: v3(cm(20), cm(90), cm(40)).norm(),
        light: INSIDE_LIGHT,
        ground: INSIDE_BOUNCE,
        rim: [0; 3],
        far: crate::render::FAR,
    }
}

// ---------------------------------------------------------------------------
// The palette.

const STONE: Rgb = rgb(138, 128, 110);
const STONE_LIGHT: Rgb = rgb(166, 156, 134);
const STONE_DARK: Rgb = rgb(88, 82, 72);
const CRACK: Rgb = rgb(40, 36, 32);
const MOSS: Rgb = rgb(92, 122, 46);
const MOSS_DARK: Rgb = rgb(58, 86, 36);

const SANDSTONE: Rgb = rgb(184, 142, 100);
const SAND_LIGHT: Rgb = rgb(210, 172, 126);
const RELIEF: Rgb = rgb(116, 82, 58);
const TURQUOISE: Rgb = rgb(52, 156, 140);
const OCHRE: Rgb = rgb(172, 64, 40);
const IVORY: Rgb = rgb(226, 214, 184);
const MOUTH_DARK: Rgb = rgb(18, 12, 12);

const GLYPH: Rgb = rgb(255, 206, 64);
const GLYPH_DIM: Rgb = rgb(150, 104, 30);
const BANANA_GOLD: Rgb = rgb(255, 196, 40);
const BANANA_TIP: Rgb = rgb(96, 64, 26);

const SHELL: Rgb = rgb(70, 82, 50);
const SCUTE: Rgb = rgb(104, 108, 60);
const TURTLE_SKIN: Rgb = rgb(118, 118, 78);
const BEAK: Rgb = rgb(52, 48, 36);
const TURTLE_EYE: Rgb = rgb(255, 120, 30);

const PLANK: Rgb = rgb(132, 90, 54);
const PLANK_DARK: Rgb = rgb(96, 64, 40);
const ROPE: Rgb = rgb(200, 170, 116);

const IRON: Rgb = rgb(96, 90, 88);
const RUST: Rgb = rgb(150, 70, 40);
const WIRE: Rgb = rgb(224, 60, 40);

const WATER: Rgb = rgb(150, 206, 214);
const FOAM: Rgb = rgb(240, 246, 240);

const TRUNK: Rgb = rgb(72, 48, 40);
const TRUNK_LIGHT: Rgb = rgb(104, 74, 56);
const LEAF: Rgb = rgb(34, 88, 64);
const LEAF_LIGHT: Rgb = rgb(62, 112, 56);
const LEAF_DARK: Rgb = rgb(22, 60, 50);
const PALM: Rgb = rgb(70, 110, 48);
const BLOSSOM: Rgb = rgb(226, 56, 116);
const VINE: Rgb = rgb(70, 104, 40);

/// Shown full colour and unfogged, as the sky is (`scene.frag`).
const SKY_GLOW: i32 = ONE * 2;
/// The sky's dome's mark for `scene.frag` (over 3.5).
const DOME_GLOW: i32 = ONE * 4;

// ---------------------------------------------------------------------------
// Helpers.

/// The frame of a model at `at`, turned `yaw` about +y, `scale` times as
/// built (Q16).
pub fn pose(at: V3, yaw: i32, scale: i32) -> Affine {
    Affine::translate(at)
        .then(&Affine::rot_y(yaw))
        .then(&Affine::scale(v3(scale, scale, scale)))
}

/// Decimetres.
fn d3(x: i32, y: i32, z: i32) -> V3 {
    v3(ratio(x, 10), ratio(y, 10), ratio(z, 10))
}

fn dm(x: i32) -> i32 {
    ratio(x, 10)
}

/// A shape of `size` with its middle at `at`, in frame `f`.
fn put(b: &mut Batch, f: &Affine, mesh: Mesh, at: V3, size: V3, c: Rgb, glow: i32) {
    b.push(
        mesh,
        &f.then(&Affine::translate(at)).then(&Affine::scale(size)),
        c,
        glow,
    );
}

/// The same, turned by `turn` about its middle.
fn put_turned(
    b: &mut Batch,
    f: &Affine,
    mesh: Mesh,
    at: V3,
    turn: &Affine,
    size: V3,
    c: Rgb,
    glow: i32,
) {
    b.push(
        mesh,
        &f.then(&Affine::translate(at))
            .then(turn)
            .then(&Affine::scale(size)),
        c,
        glow,
    );
}

/// A rod of `mesh`, `w` across, from `a` to `z` in frame `f`.
fn bar(b: &mut Batch, f: &Affine, mesh: Mesh, a: V3, z: V3, w: i32, c: Rgb, glow: i32) {
    if a.dist(z) > ONE / 100 {
        b.push(mesh, &f.then(&rod(a, z, w)), c, glow);
    }
}

/// A halo through a shape of `size` at `at` in frame `f`.
fn halo(b: &mut Batch, f: &Affine, mesh: Mesh, at: V3, size: V3, c: Rgb, strength: i32) {
    b.push_halo(
        mesh,
        &f.then(&Affine::translate(at)).then(&Affine::scale(size)),
        c,
        strength,
    );
}

/// A small hash for per-piece variety.
fn hash(k: u32, salt: u32) -> u32 {
    let mut x = k.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^ (x >> 12)
}

/// A value from `lo` to `hi` (inclusive-ish) for piece `k`.
fn pick(k: u32, salt: u32, lo: i32, hi: i32) -> i32 {
    lo + (hash(k, salt) % ((hi - lo).max(0) as u32 + 1)) as i32
}

/// A slow wave in `-ONE..ONE` with a period of `secs` seconds.
fn wave(tick: u32, secs: i32, phase: i32) -> i32 {
    let period = (secs * TICKS_PER_SECOND).max(1) as u32;
    sin((tick % period) as i32 * (TURN / period as i32) + phase)
}

/// Moss over the top of a block: a flattened lump or two.
fn moss(b: &mut Batch, f: &Affine, top: V3, w: i32, k: u32) {
    put(
        b,
        f,
        Mesh::Blob,
        top,
        v3(w, w / 4, mul(w, ONE * 4 / 5)),
        if k % 2 == 0 { MOSS } else { MOSS_DARK },
        0,
    );
    if k % 3 != 0 {
        let off = v3(w / 3, -w / 10, -w / 5);
        put(b, f, Mesh::Blob, top.add(off), v3(w / 2, w / 5, w / 2), MOSS, 0);
    }
}

/// One banana from `base`, reaching out along the flat `out` and curving
/// up, `len` long and `w` thick: three facets and a dark tip.
fn banana(b: &mut Batch, f: &Affine, base: V3, out: V3, len: i32, w: i32, c: Rgb, glow: i32) {
    let o = out.flat().norm();
    let p = |along: i32, rise: i32| base.add(o.scale(mul(len, along))).add(v3(0, mul(len, rise), 0));
    let pts = [
        p(0, 0),
        p(ONE * 30 / 100, -ONE * 6 / 100),
        p(ONE * 60 / 100, ONE * 4 / 100),
        p(ONE * 78 / 100, ONE * 30 / 100),
    ];
    let widths = [w * 3 / 4, w, w * 4 / 5];
    for i in 0..3 {
        bar(b, f, Mesh::Hex, pts[i], pts[i + 1], widths[i], c, glow);
    }
    let tip = pts[3].add(pts[3].sub(pts[2]).scale(ONE / 4));
    bar(b, f, Mesh::Cone, pts[3], tip, w / 2, BANANA_TIP, glow / 2);
}

/// A bunch of `n` bananas round a stalk at `at` (the hand's middle), each
/// `len` long: a ring of them curving out and up, a stalk on top.
fn bunch(b: &mut Batch, f: &Affine, at: V3, n: i32, len: i32, c: Rgb, glow: i32, k: u32) {
    let w = len / 4;
    for i in 0..n {
        let yaw = i * TURN / n + (k as i32) * 977;
        let out = facing(yaw, 0);
        // two tiers: the lower reaching further
        let tier = if i % 2 == 0 { 0 } else { len / 5 };
        let base = at.add(out.scale(len / 8)).add(v3(0, tier, 0));
        banana(b, f, base, out, len - tier / 2, w, c, glow);
    }
    let crown = at.add(v3(0, len / 4, 0));
    put(b, f, Mesh::Ico, crown, v3(w * 2, w * 3 / 2, w * 2), shade(c, 80), glow);
    bar(b, f, Mesh::Taper, crown, crown.add(v3(0, len / 2, len / 8)), w * 3 / 4, shade(c, 60), glow / 2);
}

// ---------------------------------------------------------------------------
// Mossy stone ruins.

/// A mossy stone banana-bunch statue, about 5 m tall at scale 1, its foot
/// at the origin. `k` varies the bunch's turn, the moss and the wear; `big`
/// adds a stepped terrace, fallen blocks and hanging moss for a landmark
/// (draw it at 3–5×, clear of trees, MAPS B7).
pub fn banana_statue(b: &mut Batch, f: &Affine, k: u32, big: bool) {
    // plinth: a chunky stepped block
    put(b, f, Mesh::Bevel, d3(0, 4, 0), d3(26, 8, 26), STONE_DARK, 0);
    put(b, f, Mesh::Bevel, d3(0, 11, 0), d3(20, 6, 20), STONE, 0);
    // the bunch, stone, leaning a little
    let lean = Affine::rot_z(deg(pick(k, 1, -6, 6))).then(&Affine::rot_x(deg(pick(k, 2, -5, 5))));
    let g = f.then(&Affine::translate(d3(0, 14, 0))).then(&lean);
    bunch(b, &g, d3(0, 6, 0), 7, int(3), STONE_LIGHT, 0, k);
    // a broken-off banana at the foot
    if k % 2 == 1 {
        banana(b, f, d3(16, 1, 10), v3(ONE, 0, ONE / 3), int(2), cm(60), STONE, 0);
    }
    // moss on the plinth and over the crown
    moss(b, f, d3(-5, 14, 4), int(1), k);
    moss(b, &g, d3(0, 20, 0), cm(110), k + 1);
    moss(b, f, d3(8, 8, -9), cm(80), k + 2);
    // a crack down the plinth's front
    put(b, f, Mesh::Cube, d3(3, 5, 13), d3(1, 7, 1), CRACK, 0);
    if big {
        ruin_base(b, f, int(5), k);
    }
}

/// A moai-style ape head of mossy stone, about 7 m tall at scale 1, its
/// chin at about 1 m and the neck sunk into the ground at the origin,
/// facing +z: a long face, a heavy brow over deep-set eyes, a broad flat
/// nose, a jutting muzzle, small ears. `big` sets it on an ahu (a long
/// stone platform) with fallen blocks and hanging moss.
pub fn moai_head(b: &mut Batch, f: &Affine, k: u32, big: bool) {
    let tilt = Affine::rot_x(deg(pick(k, 3, -4, 6))).then(&Affine::rot_z(deg(pick(k, 4, -5, 5))));
    let g = f.then(&tilt);
    // neck and the long head
    put(b, &g, Mesh::Bevel, d3(0, 10, -2), d3(30, 20, 26), STONE_DARK, 0);
    put(b, &g, Mesh::Bevel, d3(0, 40, 0), d3(34, 44, 30), STONE, 0);
    // the crown: a low flat cranium with a sagittal crest (an ape's)
    put(b, &g, Mesh::Bevel, d3(0, 64, -2), d3(30, 8, 26), STONE, 0);
    put(b, &g, Mesh::Wedge, d3(0, 70, -4), d3(6, 6, 18), STONE_DARK, 0);
    // the brow ridge, heavy and overhanging
    put(b, &g, Mesh::Bevel, d3(0, 54, 15), d3(36, 7, 8), STONE_LIGHT, 0);
    // deep-set eyes: dark sockets under the brow
    for s in [-1, 1] {
        put(b, &g, Mesh::Cube, d3(s * 8, 48, 15), d3(8, 4, 2), CRACK, 0);
        put(b, &g, Mesh::Bevel, d3(s * 8, 45, 15), d3(9, 2, 3), STONE_LIGHT, 0);
    }
    // a broad flat nose
    put_turned(
        b,
        &g,
        Mesh::Wedge,
        d3(0, 40, 17),
        &Affine::rot_y(fx::HALF),
        d3(12, 10, 6),
        STONE_LIGHT,
        0,
    );
    for s in [-1, 1] {
        put(b, &g, Mesh::Cube, d3(s * 3, 36, 19), d3(3, 2, 2), CRACK, 0);
    }
    // the jutting muzzle and a long tight mouth line
    put(b, &g, Mesh::Bevel, d3(0, 28, 15), d3(28, 14, 12), STONE, 0);
    put(b, &g, Mesh::Cube, d3(0, 27, 21), d3(20, 1, 2), CRACK, 0);
    put(b, &g, Mesh::Bevel, d3(0, 20, 13), d3(20, 6, 10), STONE_DARK, 0);
    // small ears
    for s in [-1, 1] {
        put_turned(
            b,
            &g,
            Mesh::Hex,
            d3(s * 18, 46, -2),
            &Affine::rot_z(fx::QUARTER),
            d3(10, 3, 8),
            STONE_DARK,
            0,
        );
    }
    // moss on the crown and the brow, a streak down one cheek
    moss(b, &g, d3(0, 69, 0), cm(260), k);
    moss(b, &g, d3(-6, 58, 16), cm(120), k + 1);
    put(b, &g, Mesh::Cube, d3(-13, 38, 14), d3(3, 18, 2), MOSS_DARK, 0);
    // a crack across the face
    put_turned(
        b,
        &g,
        Mesh::Cube,
        d3(9, 33, 16),
        &Affine::rot_z(deg(30)),
        d3(1, 10, 1),
        CRACK,
        0,
    );
    if big {
        // the ahu: a long low platform of fitted blocks behind and under
        for i in -3..=3 {
            let h = 6 + (hash(k, 10 + i as u32) % 3) as i32;
            put(
                b,
                f,
                Mesh::Bevel,
                d3(i * 9, h / 2 - 2, -4),
                d3(9, h, 22),
                if i % 2 == 0 { STONE } else { STONE_DARK },
                0,
            );
        }
        ruin_base(b, f, int(4), k + 7);
    }
}

/// Fallen blocks, rubble and hanging moss round a landmark ruin of
/// radius `r`.
fn ruin_base(b: &mut Batch, f: &Affine, r: i32, k: u32) {
    // a low stepped terrace
    put(b, f, Mesh::Bevel, v3(0, -cm(30), 0), v3(r * 2 + int(2), cm(80), r * 2 + int(2)), STONE_DARK, 0);
    for i in 0..7u32 {
        let yaw = (i as i32) * TURN / 7 + pick(k, 20 + i, 0, 4000);
        let at = facing(yaw, 0).scale(r + int(1) + cm(pick(k, 30 + i, 0, 160)));
        let s = cm(pick(k, 40 + i, 50, 130));
        let mesh = if i % 3 == 0 { Mesh::Rock } else { Mesh::Bevel };
        put_turned(
            b,
            f,
            mesh,
            at.add(v3(0, s / 3, 0)),
            &Affine::rot_y(yaw).then(&Affine::rot_z(deg(pick(k, 50 + i, -20, 20)))),
            v3(s, s * 3 / 4, s),
            if i % 2 == 0 { STONE } else { STONE_DARK },
            0,
        );
        if i % 2 == 0 {
            moss(b, f, at.add(v3(0, s * 3 / 4, 0)), s * 3 / 4, k + i);
        }
    }
}

// ---------------------------------------------------------------------------
// The river's snapping turtle.

/// The turtle's stroke for race tick `tick` (`k` picks its own phase), for
/// `turtle`.
pub fn turtle_phase(tick: u32, k: u32) -> i32 {
    let period = (TICKS_PER_SECOND * 3 / 2) as u32;
    ((tick + k * 37) % period) as i32 * (TURN / period as i32)
}

/// A snapping turtle swimming toward +z, about 3.2 m long at scale 1 (big
/// enough to bounce on beside apes drawn at 2×), its shell's top at about
/// 0.9 m over the origin, the waterline: a ridged dark shell with three
/// rows of knobs, a hooked beak that snaps, front flippers sweeping with
/// `phase` (a turn per stroke, `turtle_phase`), the hind ones kicking
/// opposite, a ridged tail.
pub fn turtle(b: &mut Batch, f: &Affine, phase: i32) {
    let s = sin(phase);
    let bob = mul(s, cm(5));
    let g = f.then(&Affine::translate(v3(0, bob, 0)));
    // plastron and shell
    put(b, &g, Mesh::Bevel, d3(0, 2, 0), d3(20, 3, 26), TURTLE_SKIN, 0);
    put(b, &g, Mesh::Geo, d3(0, 4, 0), d3(24, 10, 30), SHELL, 0);
    // the marginal scutes: a jagged skirt round the shell
    for i in 0..12 {
        let yaw = i * TURN / 12;
        let at = facing(yaw, 0);
        let p = v3(mul(at.x, dm(12)), dm(3), mul(at.z, dm(15)));
        put_turned(
            b,
            &g,
            Mesh::Pyramid,
            p,
            &Affine::rot_y(yaw).then(&Affine::rot_x(fx::QUARTER)),
            d3(4, 4, 2),
            SHELL,
            0,
        );
    }
    // three ridges of knobs along the back
    for row in -1..=1 {
        for i in 0..4 {
            let z = dm(-9 + i * 6);
            let x = dm(row * 6);
            let y = dm(9) - (row.abs() * dm(15)) / 10 - (i - 1).abs() * dm(5) / 10;
            put(b, &g, Mesh::Pyramid, v3(x, y, z), d3(3, 3, 4), SCUTE, 0);
        }
    }
    // the head on a thick neck, the jaw snapping with the stroke
    let snap = (s.max(0) * 20) / ONE;
    put(b, &g, Mesh::Hex, d3(0, 3, 16), d3(7, 6, 6), TURTLE_SKIN, 0);
    put(b, &g, Mesh::Ico, d3(0, 4, 20), d3(8, 7, 9), TURTLE_SKIN, 0);
    put_turned(
        b,
        &g,
        Mesh::Wedge,
        d3(0, 5, 24),
        &Affine::rot_y(fx::HALF),
        d3(5, 4, 4),
        BEAK,
        0,
    );
    put_turned(
        b,
        &g,
        Mesh::Wedge,
        d3(0, 2, 23),
        &Affine::rot_x(deg(snap)),
        d3(5, 2, 4),
        BEAK,
        0,
    );
    for sx in [-1, 1] {
        put(b, &g, Mesh::Sphere, d3(sx * 3, 6, 22), d3(2, 2, 2), TURTLE_EYE, ONE / 2);
    }
    // flippers: the front pair sweep, the hind kick opposite
    for sx in [-1, 1] {
        let sweep = deg(30) * s / ONE;
        let front = Affine::rot_y(sx * (deg(50) + sweep)).then(&Affine::rot_z(sx * deg(-10)));
        put_turned(b, &g, Mesh::Shard, d3(sx * 13, 1, 8), &front.then(&Affine::rot_z(fx::QUARTER)), d3(3, 14, 6), TURTLE_SKIN, 0);
        let hind = Affine::rot_y(sx * (deg(140) - sweep));
        put_turned(b, &g, Mesh::Shard, d3(sx * 10, 1, -12), &hind.then(&Affine::rot_z(fx::QUARTER)), d3(2, 8, 5), TURTLE_SKIN, 0);
    }
    // the ridged tail
    bar(b, &g, Mesh::Cone, d3(0, 2, -14), d3(0, 1, -24), dm(4), TURTLE_SKIN, 0);
    for i in 0..3 {
        put(b, &g, Mesh::Pyramid, d3(0, 4, -16 - i * 3), d3(1, 2, 2), SCUTE, 0);
    }
    // its wake: foam at the bow and two trailing streaks
    put(b, f, Mesh::Blob, d3(0, 0, 18), d3(18, 2, 6), FOAM, ONE / 3);
    for sx in [-1, 1] {
        put_turned(b, f, Mesh::Blob, d3(sx * 10, 0, -10), &Affine::rot_y(sx * deg(15)), d3(4, 1, 20), FOAM, ONE / 4);
    }
}

// ---------------------------------------------------------------------------
// The rope bridge.

/// The rope bridge's deck width: three gorillas side by side as drawn
/// (`models` Gorilla at the race's 2×: arms out, about 2 m each).
pub const BRIDGE_WIDTH: i32 = int(6);
/// The hand ropes' height over the deck.
pub const BRIDGE_RAIL: i32 = cm(130);

/// A rope bridge segment `len` metres long from the origin along +z, its
/// deck at y 0 and centred on x 0: slatted planks on two foot ropes, two
/// sagging hand ropes and a few V suspenders, all thin and pale so the
/// map's vines stand out above them (MAPS Art lessons).
pub fn rope_bridge(b: &mut Batch, f: &Affine, len: i32) {
    let half = BRIDGE_WIDTH / 2;
    let n = (len / cm(55)).max(1);
    for i in 0..n {
        let z = cm(55) * i + cm(25);
        let k = i as u32;
        // the odd plank missing, the odd plank askew
        if hash(k, 70) % 11 == 0 {
            continue;
        }
        let skew = deg(pick(k, 71, -3, 3));
        put_turned(
            b,
            f,
            Mesh::Cube,
            v3(cm(pick(k, 72, -8, 8)), 0, z),
            &Affine::rot_y(skew),
            v3(BRIDGE_WIDTH + cm(30), cm(10), cm(38)),
            if k % 3 == 0 { PLANK_DARK } else { PLANK },
            0,
        );
    }
    // foot ropes under the planks' ends
    for sx in [-1, 1] {
        let x = sx * (half + cm(5));
        bar(b, f, Mesh::Cyl, v3(x, -cm(8), 0), v3(x, -cm(8), len), cm(9), ROPE, 0);
    }
    // the hand ropes, sagging a little over the segment
    let sag = |t: i32| BRIDGE_RAIL - mul(cm(25), sin(mul(t, fx::HALF)));
    let pieces = 6;
    for sx in [-1, 1] {
        let x = sx * (half + cm(20));
        for p in 0..pieces {
            let t0 = ratio(p, pieces);
            let t1 = ratio(p + 1, pieces);
            bar(b, f, Mesh::Cyl, v3(x, sag(t0), mul(len, t0)), v3(x, sag(t1), mul(len, t1)), cm(7), ROPE, 0);
        }
        // V suspenders every 2.4 m, hand rope to deck edge
        let mut z = cm(120);
        while z < len {
            let t = fx::div(z, len.max(1));
            let top = v3(x, sag(t), z);
            let foot = v3(sx * (half + cm(5)), 0, z);
            bar(b, f, Mesh::Cyl, top, foot.add(v3(0, 0, -cm(40))), cm(4), ROPE, 0);
            bar(b, f, Mesh::Cyl, top, foot.add(v3(0, 0, cm(40))), cm(4), ROPE, 0);
            z += cm(240);
        }
    }
}

/// A bridge's anchor post pair at the origin (a bank's end of the bridge),
/// the bridge running off along +z: two lashed timber posts on stone.
pub fn bridge_posts(b: &mut Batch, f: &Affine) {
    let half = BRIDGE_WIDTH / 2 + cm(30);
    for sx in [-1, 1] {
        put(b, f, Mesh::Bevel, v3(sx * half, -cm(20), 0), v3(cm(110), cm(60), cm(110)), STONE_DARK, 0);
        bar(b, f, Mesh::Taper, v3(sx * half, 0, 0), v3(sx * half, cm(210), 0), cm(34), TRUNK, 0);
        for i in 0..3 {
            put(b, f, Mesh::Hex, v3(sx * half, cm(140 + i * 18), 0), v3(cm(40), cm(8), cm(40)), ROPE, 0);
        }
    }
}

// ---------------------------------------------------------------------------
// The temple's mouth.

/// The mouth opening in the facade: its width and height. Its lower lip's
/// middle is the facade's origin.
pub const MOUTH_W: i32 = int(26);
pub const MOUTH_H: i32 = int(14);

/// The temple's facade, facing +z: a great stepped wall carved in Aztec
/// stone relief as an ape's face, its jaw gaping in a wide mouth
/// (`MOUTH_W` × `MOUTH_H`, its lower lip's middle at the origin) the apes
/// fly into from the bridge, fanged above and below, the dark throat
/// behind, and a waterfall pouring out over the lower lip into the
/// ravine (moving with `tick`). About 64 m wide and 70 m high, from 30 m
/// under the lip.
pub fn temple_mouth(b: &mut Batch, f: &Affine, tick: u32) {
    let hw = MOUTH_W / 2;
    let top = int(42);
    let foot = -int(30);
    let wide = int(32);
    // the wall round the opening, in courses of block
    let course = |b: &mut Batch, x0: i32, x1: i32, y0: i32, y1: i32, z: i32, c: Rgb| {
        put(b, f, Mesh::Bevel, v3((x0 + x1) / 2, (y0 + y1) / 2, z), v3(x1 - x0, y1 - y0, int(4)), c, 0);
    };
    course(b, -wide, -hw, foot, top, 0, SANDSTONE);
    course(b, hw, wide, foot, top, 0, SANDSTONE);
    course(b, -hw, hw, MOUTH_H, top, 0, SANDSTONE);
    course(b, -hw, hw, foot, 0, 0, SANDSTONE);
    // stepped tiers behind and above, like a temple pyramid
    for t in 1..4 {
        let inset = int(5) * t;
        course(b, -wide + inset, wide - inset, top + int(6) * (t - 1), top + int(6) * t, -int(3) * t, if t % 2 == 0 { SANDSTONE } else { SAND_LIGHT });
    }
    // the throat: dark behind the opening, so the mouth reads as gaping
    put(b, f, Mesh::Cube, v3(0, MOUTH_H / 2, -int(8)), v3(MOUTH_W, MOUTH_H, int(12)), MOUTH_DARK, 0);
    put(b, f, Mesh::Cube, v3(0, -cm(10), -int(4)), v3(MOUTH_W, cm(20), int(8)), RELIEF, 0);
    // lips: a raised, painted band round the opening
    put(b, f, Mesh::Bevel, v3(0, MOUTH_H + int(1), int(2) + cm(50)), v3(MOUTH_W + int(4), int(2), int(2)), OCHRE, 0);
    put(b, f, Mesh::Bevel, v3(0, -int(1), int(2) + cm(50)), v3(MOUTH_W + int(4), int(2), int(2)), OCHRE, 0);
    for sx in [-1, 1] {
        put(b, f, Mesh::Bevel, v3(sx * (hw + int(1)), MOUTH_H / 2, int(2) + cm(50)), v3(int(2), MOUTH_H + int(4), int(2)), OCHRE, 0);
    }
    // fangs: two great ones at the corners, a row of smaller teeth
    for sx in [-1, 1] {
        let x = sx * (hw - int(2));
        put_turned(b, f, Mesh::Spike, v3(x, MOUTH_H - int(3), int(2)), &Affine::rot_x(fx::HALF), v3(int(3), int(6), int(3)), IVORY, 0);
        put(b, f, Mesh::Spike, v3(x, int(2), int(2)), v3(int(3), int(4), int(3)), IVORY, 0);
    }
    for i in -3..=3 {
        let x = int(i * 3);
        put_turned(b, f, Mesh::Pyramid, v3(x, MOUTH_H - cm(70), int(2)), &Affine::rot_x(fx::HALF), v3(int(2), cm(140), int(1)), IVORY, 0);
    }
    // the nose: a broad stepped relief over the upper lip
    put(b, f, Mesh::Bevel, v3(0, MOUTH_H + int(5), int(3)), v3(int(12), int(4), int(3)), SAND_LIGHT, 0);
    put(b, f, Mesh::Bevel, v3(0, MOUTH_H + int(8), int(3)), v3(int(6), int(3), int(2)), SAND_LIGHT, 0);
    for sx in [-1, 1] {
        put(b, f, Mesh::Cube, v3(sx * int(3), MOUTH_H + int(4), int(4) + cm(60)), v3(int(2), int(1), cm(20)), MOUTH_DARK, 0);
    }
    // the eyes: square sockets ringed in turquoise, glowing amber within
    for sx in [-1, 1] {
        let at = v3(sx * int(10), MOUTH_H + int(13), int(2) + cm(40));
        put(b, f, Mesh::Bevel, at, v3(int(8), int(6), cm(80)), TURQUOISE, 0);
        put(b, f, Mesh::Cube, at.add(v3(0, 0, cm(30))), v3(int(5), int(3), cm(60)), MOUTH_DARK, 0);
        put(b, f, Mesh::Octa, at.add(v3(0, 0, cm(60))), v3(int(2), int(2), cm(80)), GLYPH, ONE * 3 / 4);
        halo(b, f, Mesh::Sphere, at.add(v3(0, 0, int(1))), v3(int(5), int(4), int(2)), GLYPH, ONE / 3);
    }
    // the brow: a heavy band of step-fret relief across the face
    put(b, f, Mesh::Bevel, v3(0, MOUTH_H + int(18), int(2) + cm(50)), v3(int(40), int(3), int(2)), RELIEF, 0);
    step_fret(b, f, v3(-int(20), MOUTH_H + int(18), int(3) + cm(60)), int(40), int(1), SAND_LIGHT);
    // ears: round discs with plugs at the facade's sides
    for sx in [-1, 1] {
        let at = v3(sx * (wide - int(4)), MOUTH_H + int(8), int(2) + cm(50));
        put_turned(b, f, Mesh::Hex, at, &Affine::rot_x(fx::QUARTER), v3(int(8), int(1), int(8)), RELIEF, 0);
        put_turned(b, f, Mesh::Hex, at.add(v3(0, 0, cm(60))), &Affine::rot_x(fx::QUARTER), v3(int(4), int(1), int(4)), TURQUOISE, 0);
    }
    // a headdress: a crown of stone plumes and a banana glyph in the middle
    for i in -6..=6 {
        let x = int(i * 4);
        let h = int(9) - (i.abs() * int(1)) / 2;
        put_turned(b, f, Mesh::Shard, v3(x, top + int(19) + h / 2, -int(8)), &Affine::rot_z(deg(-i * 5)), v3(int(3), h, int(1)), if i % 2 == 0 { TURQUOISE } else { SAND_LIGHT }, 0);
    }
    let crown = f.then(&Affine::translate(v3(0, top + int(10), int(3))))
        .then(&Affine::scale(v3(int(3), int(3), int(3))));
    banana_glyph(b, &crown, tick, 0);
    // side panels of carved stonework down the cheeks and the jaw
    for sx in [-1, 1] {
        for row in 0..3 {
            let at = v3(sx * (hw + int(9)), MOUTH_H - int(4) - int(row * 10), int(2) + cm(20));
            put(b, f, Mesh::Bevel, at, v3(int(8), int(8), cm(40)), RELIEF, 0);
            let g = f.then(&Affine::translate(at.add(v3(0, 0, cm(40)))))
                .then(&Affine::scale(v3(int(2), int(2), int(2))));
            banana_glyph(b, &g, tick, (row + 2 * (sx + 1)) as u32);
        }
    }
    // a chin: step-fret band under the lower lip
    step_fret(b, f, v3(-hw, -int(4), int(2) + cm(60)), MOUTH_W, int(1), SAND_LIGHT);
    // moss and creepers over the stone
    for i in 0..9u32 {
        let x = int(pick(i, 80, -30, 30));
        let y = int(pick(i, 81, -20, 44));
        if x.abs() < hw + int(2) && y > -int(6) && y < MOUTH_H + int(2) {
            continue;
        }
        put(b, f, Mesh::Blob, v3(x, y, int(2)), v3(int(4), int(3), int(1)), if i % 2 == 0 { MOSS } else { MOSS_DARK }, 0);
        bar(b, f, Mesh::Cyl, v3(x, y, int(2) + cm(40)), v3(x + cm(40), y - int(6), int(2) + cm(40)), cm(20), VINE, 0);
    }
    waterfall(b, &f.then(&Affine::translate(v3(0, 0, int(2)))), MOUTH_W - int(6), int(30), tick);
}

/// A band of Aztec step-fret (stepped spirals as blocks) from `at` along
/// +x for `len`, `h` high.
fn step_fret(b: &mut Batch, f: &Affine, at: V3, len: i32, h: i32, c: Rgb) {
    let unit = h * 2;
    let n = (len / unit).max(1);
    for i in 0..n {
        let x = at.x + unit * i + unit / 2;
        let up = if i % 2 == 0 { h / 4 } else { -h / 4 };
        put(b, f, Mesh::Cube, v3(x, at.y + up, at.z), v3(unit / 2, h / 2, cm(30)), c, 0);
        put(b, f, Mesh::Cube, v3(x + unit / 4, at.y, at.z), v3(unit / 8, h, cm(30)), c, 0);
    }
}

/// A waterfall `w` wide pouring from the origin (its lip) forward over and
/// down `drop` metres: a curving sheet of bands streaming down with
/// `tick`, white streaks over pale water, mist at its foot.
pub fn waterfall(b: &mut Batch, f: &Affine, w: i32, drop: i32, tick: u32) {
    let rows = 10;
    let shift = ((tick % 60) as i32) * ONE / 60;
    let curve = |t: i32| -> V3 {
        // out over the lip, then falling nearly straight
        let out = int(3) - mul(int(3), mul(ONE - t, ONE - t));
        v3(0, -mul(drop, mul(t, t / 2 + ONE / 2)), out)
    };
    for r in 0..rows {
        let t0 = ratio(r, rows);
        let t1 = ratio(r + 1, rows);
        let (a, z) = (curve(t0), curve(t1));
        let mid = a.lerp(z, ONE / 2);
        let d = z.sub(a);
        let along = Affine::along(d);
        let len = d.len();
        put_turned(b, f, Mesh::Cube, mid, &along, v3(w, len + cm(10), cm(40)), WATER, ONE / 3);
        // streaks, sliding down with the tick
        let n = (w / int(2)).max(1);
        for s in 0..n {
            let k = (r * 31 + s * 7) as u32;
            let off = fx::wrap(shift * TURN / ONE + pick(k, 90, 0, TURN)) as i64;
            let tt = ((off + fx::HALF as i64) * ONE as i64 / TURN as i64) as i32;
            let p = a.lerp(z, tt);
            let x = -w / 2 + int(2) * s + int(1);
            put_turned(b, f, Mesh::Cube, v3(x, p.y, p.z + cm(25)), &along, v3(cm(pick(k, 91, 40, 120)), len / 2, cm(10)), FOAM, ONE / 2);
        }
    }
    // mist at the foot
    let foot = curve(ONE);
    for i in 0..5 {
        let x = -w / 2 + mul(w, ratio(i, 4));
        halo(b, f, Mesh::Sphere, v3(x, foot.y + int(2), foot.z + int(2)), v3(int(8), int(5), int(6)), FOAM, ONE / 3);
    }
    // the lip's foam
    put(b, f, Mesh::Blob, v3(0, cm(20), cm(60)), v3(w + int(1), cm(60), int(1)), FOAM, ONE / 2);
}

// ---------------------------------------------------------------------------
// The temple's stonework, glyphs and traps.

/// The banana glyphs' pulse at `tick` (`k` picks its own phase): 0.3 to 1,
/// slow (about 4 s a breath), warm.
pub fn glyph_pulse(tick: u32, k: u32) -> i32 {
    let w = wave(tick, 4, (k as i32).wrapping_mul(9_301));
    ONE * 3 / 10 + mul((w + ONE) / 2, ONE * 7 / 10)
}

/// A carved stone wall panel `w` × `h` metres in the xy plane, centred on
/// the origin, facing +z, about 0.6 m deep: a frame, step-fret bands top
/// and bottom, and recessed sockets for the banana glyphs at
/// `glyph_spots(w, h)` (draw the glyphs with `banana_glyph`, so they can
/// pulse). Turn it face-down for a ceiling.
pub fn stone_panel(b: &mut Batch, f: &Affine, w: i32, h: i32) {
    put(b, f, Mesh::Cube, v3(0, 0, -cm(30)), v3(w, h, cm(60)), RELIEF, 0);
    // a frame
    let edge = cm(50);
    for sy in [-1, 1] {
        put(b, f, Mesh::Bevel, v3(0, sy * (h / 2 - edge / 2), cm(10)), v3(w, edge, cm(40)), SANDSTONE, 0);
    }
    for sx in [-1, 1] {
        put(b, f, Mesh::Bevel, v3(sx * (w / 2 - edge / 2), 0, cm(10)), v3(edge, h, cm(40)), SANDSTONE, 0);
    }
    // step-fret bands inside the frame
    let band = cm(60);
    step_fret(b, f, v3(-w / 2 + edge, h / 2 - edge - band / 2, cm(10)), w - edge * 2, band, SAND_LIGHT);
    step_fret(b, f, v3(-w / 2 + edge, -h / 2 + edge + band / 2, cm(10)), w - edge * 2, band, SAND_LIGHT);
    // the field between: fitted blocks with sockets
    for (i, s) in glyph_spots(w, h).into_iter().enumerate() {
        put(b, f, Mesh::Bevel, s.add(v3(0, 0, -cm(5))), v3(int(2) + cm(40), int(2) + cm(40), cm(20)), if i % 2 == 0 { SANDSTONE } else { STONE }, 0);
        put(b, f, Mesh::Bevel, s.add(v3(0, 0, cm(2))), v3(int(2), int(2), cm(10)), MOUTH_DARK, 0);
        // turquoise studs at the socket's corners
        for (cx, cy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
            put(b, f, Mesh::Octa, s.add(v3(cx * cm(115), cy * cm(115), cm(10))), v3(cm(20), cm(20), cm(20)), TURQUOISE, 0);
        }
    }
}

/// Where `stone_panel` leaves sockets for glyphs, in its frame: a row
/// every 3.2 m across, as many rows as fit.
pub fn glyph_spots(w: i32, h: i32) -> Vec<V3> {
    let inner_w = w - int(2);
    let inner_h = h - int(3);
    let cols = (inner_w / cm(320)).max(1);
    let rows = (inner_h / cm(320)).max(1);
    let mut out = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let x = -inner_w / 2 + inner_w * (2 * c + 1) / (2 * cols);
            let y = -inner_h / 2 + inner_h * (2 * r + 1) / (2 * rows);
            out.push(v3(x, y, cm(10)));
        }
    }
    out
}

/// A banana glyph about 1.6 m tall at the origin, facing +z, carved flat:
/// a curved banana in three segments with its stalk and a stepped ring
/// round it, glowing warm yellow and pulsing with `tick` (`glyph_pulse`),
/// with a soft halo so it lights the stone round it.
pub fn banana_glyph(b: &mut Batch, f: &Affine, tick: u32, k: u32) {
    let p = glyph_pulse(tick, k);
    let c = mix(GLYPH_DIM, GLYPH, p);
    let glow = ONE / 3 + mul(p, ONE * 2 / 3);
    // a crescent: three facets curving left to right, the ends up
    let pts = [d3(-6, 3, 0), d3(-3, -3, 0), d3(3, -3, 0), d3(6, 3, 0)];
    let widths = [cm(30), cm(36), cm(30)];
    for i in 0..3 {
        bar(b, f, Mesh::Hex, pts[i], pts[i + 1], widths[i], c, glow);
    }
    bar(b, f, Mesh::Cone, pts[3], d3(7, 6, 0), cm(16), c, glow);
    bar(b, f, Mesh::Taper, pts[0], d3(-6, 6, 0), cm(14), shade(c, 70), glow);
    // a stepped ring of small blocks round it
    for i in 0..8 {
        let a = i * TURN / 8;
        let at = v3(mul(sin(a + fx::QUARTER), dm(8)), mul(sin(a), dm(8)), 0);
        put(b, f, Mesh::Cube, at, d3(1, 1, 1), shade(c, 85), glow / 2);
    }
    halo(b, f, Mesh::Sphere, d3(0, 0, 2), v3(int(3), int(2) + cm(40), int(1)), GLYPH, mul(p, ONE / 2));
}

/// A floor spike trap 2.4 m square at the origin, on the floor: a stone
/// plate with a grid of iron-rimmed holes, the spikes out by `up` (0 in,
/// ONE fully out, about 1.1 m).
pub fn spike_trap(b: &mut Batch, f: &Affine, up: i32) {
    put(b, f, Mesh::Bevel, v3(0, -cm(5), 0), v3(cm(240), cm(10), cm(240)), STONE_DARK, 0);
    put(b, f, Mesh::Cube, v3(0, cm(1), 0), v3(cm(250), cm(4), cm(10)), RELIEF, 0);
    put(b, f, Mesh::Cube, v3(0, cm(1), 0), v3(cm(10), cm(4), cm(250)), RELIEF, 0);
    let rise = mul(up.clamp(0, ONE), cm(110));
    for i in 0..4 {
        for j in 0..4 {
            let at = v3(cm(-90 + i * 60), 0, cm(-90 + j * 60));
            put(b, f, Mesh::Hex, at.add(v3(0, cm(1), 0)), v3(cm(22), cm(3), cm(22)), CRACK, 0);
            if rise > cm(2) {
                let h = rise + cm(pick((i * 4 + j) as u32, 60, -10, 10)).min(rise / 4);
                put(b, f, Mesh::Spike, at.add(v3(0, h / 2, 0)), v3(cm(18), h, cm(18)), IRON, 0);
                put(b, f, Mesh::Spike, at.add(v3(0, h - cm(8), 0)), v3(cm(8), cm(16), cm(8)), RUST, 0);
            }
        }
    }
}

/// A trip wire `len` metres along +x from the origin, a hand's breadth
/// over the floor between two carved stone pegs: taut red cord, or slack
/// on the floor once `sprung`.
pub fn trip_wire(b: &mut Batch, f: &Affine, len: i32, sprung: bool) {
    for x in [0, len] {
        put(b, f, Mesh::Bevel, v3(x, cm(15), 0), v3(cm(26), cm(30), cm(26)), SANDSTONE, 0);
        put(b, f, Mesh::Octa, v3(x, cm(36), 0), v3(cm(16), cm(16), cm(16)), TURQUOISE, 0);
    }
    if sprung {
        bar(b, f, Mesh::Cyl, v3(0, cm(4), 0), v3(len / 3, cm(1), cm(20)), cm(4), WIRE, 0);
        bar(b, f, Mesh::Cyl, v3(len, cm(4), 0), v3(len * 2 / 3, cm(1), -cm(15)), cm(4), WIRE, 0);
    } else {
        // taut and faintly bright, so a player can see it coming
        bar(b, f, Mesh::Cyl, v3(0, cm(20), 0), v3(len, cm(20), 0), cm(4), WIRE, ONE / 3);
    }
}

/// A trapdoor `w` across (x) and `l` along (z), its middle at the origin
/// flush with the floor: a stone frame and two leaves hinged on the long
/// sides, swung down by `open` (0 shut, ONE hanging nearly straight down).
pub fn trapdoor(b: &mut Batch, f: &Affine, w: i32, l: i32, open: i32) {
    let rim = cm(40);
    for sx in [-1, 1] {
        put(b, f, Mesh::Bevel, v3(sx * (w / 2 + rim / 2), -cm(10), 0), v3(rim, cm(20), l + rim * 2), STONE_DARK, 0);
    }
    for sz in [-1, 1] {
        put(b, f, Mesh::Bevel, v3(0, -cm(10), sz * (l / 2 + rim / 2)), v3(w, cm(20), rim), STONE_DARK, 0);
    }
    let swing = mul(open.clamp(0, ONE), deg(84));
    for sx in [-1, 1] {
        let hinge = v3(sx * w / 2, 0, 0);
        let leaf = f
            .then(&Affine::translate(hinge))
            .then(&Affine::rot_z(sx * swing));
        let half = w / 2 - cm(4);
        put(b, &leaf, Mesh::Cube, v3(-sx * half / 2, -cm(8), 0), v3(half, cm(16), l - cm(8)), STONE, 0);
        // the leaves' joint lines (so a shut door shows where it splits)
        for i in 0..3 {
            let z = -l / 2 + l * (2 * i + 1) / 6;
            put(b, &leaf, Mesh::Cube, v3(-sx * half / 2, cm(1), z), v3(half, cm(2), cm(6)), CRACK, 0);
        }
    }
}

/// A pit's floor `w` × `l` at the origin, bristling with glowing stone
/// banana spikes, curving and pulsing faintly with `tick`, and a warm glow
/// rising from it.
pub fn pit_spikes(b: &mut Batch, f: &Affine, w: i32, l: i32, tick: u32) {
    put(b, f, Mesh::Cube, v3(0, -cm(20), 0), v3(w, cm(40), l), RELIEF, 0);
    let p = glyph_pulse(tick, 99);
    let c = mix(BANANA_GOLD, GLYPH, p);
    let glow = ONE / 2 + mul(p, ONE / 3);
    let step = cm(70);
    let cols = (w / step).max(1);
    let rows = (l / step).max(1);
    for i in 0..cols {
        for j in 0..rows {
            let k = (i * 97 + j) as u32;
            let x = -w / 2 + step * i + step / 2 + cm(pick(k, 61, -15, 15));
            let z = -l / 2 + step * j + step / 2 + cm(pick(k, 62, -15, 15));
            let h = cm(pick(k, 63, 90, 150));
            let lean = facing(pick(k, 64, 0, TURN), 0).scale(h / 5);
            let base = v3(x, 0, z);
            let mid = base.add(v3(lean.x / 3, h / 2, lean.z / 3));
            let tip = base.add(v3(lean.x, h, lean.z));
            bar(b, f, Mesh::Hex, base, mid, cm(22), c, glow);
            bar(b, f, Mesh::Spike, mid, tip, cm(18), c, glow);
            bar(b, f, Mesh::Cone, tip, tip.add(tip.sub(mid).scale(ONE / 6)), cm(6), BANANA_TIP, 0);
        }
    }
    halo(b, f, Mesh::Sphere, v3(0, cm(80), 0), v3(w, int(2), l), GLYPH, mul(p, ONE / 3));
}

// ---------------------------------------------------------------------------
// The finish: the banana pile and the skylight.

/// The finish: a huge mound (about 22 m across, 14 m high) of golden
/// banana statues heaped on a stepped dais, glowing and breathing with
/// `tick`, one great bunch on top.
pub fn banana_pile(b: &mut Batch, f: &Affine, tick: u32) {
    // the dais
    for t in 0..3 {
        let r = int(13 - t * 2);
        put(b, f, Mesh::Hex, v3(0, cm(60) + int(t) * 6 / 5, 0), v3(r * 2, int(1) + cm(20), r * 2), if t % 2 == 0 { SANDSTONE } else { SAND_LIGHT }, 0);
    }
    let base = int(4);
    // rings of bunches, smaller toward the top, tumbled every which way
    let rings: [(i32, i32, i32, i32); 5] = [
        (8, int(9), int(3), 0),
        (7, int(6), int(3) + cm(50), int(2)),
        (6, int(4), int(3), int(5)),
        (4, int(2), int(3), int(8)),
        (1, 0, int(5), int(11)),
    ];
    let mut k = 0u32;
    for (n, r, len, y) in rings {
        for i in 0..n {
            let yaw = i * TURN / n + (y / ONE) * 3_000;
            let at = facing(yaw, 0).scale(r).add(v3(0, base + y, 0));
            let tumble = Affine::rot_y(yaw + pick(k, 66, 0, TURN))
                .then(&Affine::rot_z(deg(pick(k, 67, -35, 35))))
                .then(&Affine::rot_x(deg(pick(k, 68, -25, 25))));
            let g = f.then(&Affine::translate(at)).then(&tumble);
            let p = glyph_pulse(tick, k);
            let c = mix(BANANA_GOLD, GLYPH, p);
            bunch(b, &g, V3::ZERO, 6, len, c, ONE / 3 + mul(p, ONE / 3), k);
            k += 1;
        }
    }
    // loose bananas round the foot
    for i in 0..14u32 {
        let yaw = (i as i32) * TURN / 14 + 1500;
        let at = facing(yaw, 0).scale(int(11) + cm(pick(i, 69, 0, 150))).add(v3(0, int(2) + cm(30), 0));
        banana(b, f, at, facing(yaw + deg(pick(i, 70, -60, 60)), 0), int(2), cm(55), BANANA_GOLD, ONE / 3);
    }
    // a warm glow round the heap
    let p = glyph_pulse(tick, 7);
    halo(b, f, Mesh::Sphere, v3(0, int(9), 0), v3(int(30), int(22), int(30)), GLYPH, ONE / 4 + mul(p, ONE / 5));
}

/// A skylight in the temple's ceiling: a ring of carved stone of radius
/// `r` round an opening at the origin (the ceiling's underside at y 0),
/// foliage spilling over its lip, jungle vines hanging `hang` metres down
/// and swaying with `tick`, and a soft shaft of sunset light falling
/// `shaft` metres (a spotlight on whatever is under it).
pub fn skylight(b: &mut Batch, f: &Affine, r: i32, hang: i32, shaft: i32, tick: u32) {
    let n = 20;
    for i in 0..n {
        let a = i * TURN / n;
        let at = facing(a, 0).scale(r + int(1));
        put_turned(b, f, Mesh::Bevel, at.add(v3(0, int(1), 0)), &Affine::rot_y(a), v3(int(2) + cm(40), int(2), mul(r, ONE * 34 / 100)), if i % 2 == 0 { SANDSTONE } else { RELIEF }, 0);
        if i % 4 == 0 {
            let g = f.then(&Affine::translate(at.add(v3(0, -cm(10), 0))))
                .then(&Affine::rot_x(fx::QUARTER))
                .then(&Affine::scale(v3(ONE, ONE, ONE)));
            banana_glyph(b, &g, tick, i as u32);
        }
    }
    // foliage over the lip: clumps of leaves and ferns leaning in
    for i in 0..14u32 {
        let a = (i as i32) * TURN / 14 + pick(i, 73, 0, 2000);
        let at = facing(a, 0).scale(r + cm(pick(i, 74, -40, 80)));
        let s = cm(pick(i, 75, 180, 320));
        put(b, f, Mesh::Blob, at.add(v3(0, cm(40), 0)), v3(s, s * 2 / 3, s), if i % 3 == 0 { LEAF_LIGHT } else { LEAF }, 0);
        for j in 0..3 {
            let lean = Affine::rot_y(a + fx::HALF + deg(j * 30 - 30)).then(&Affine::rot_x(deg(60)));
            put_turned(b, f, Mesh::Shard, at.add(v3(0, cm(20), 0)), &lean, v3(cm(60), s, cm(10)), PALM, 0);
        }
    }
    // vines hanging into the light, swaying
    for i in 0..12u32 {
        let a = (i as i32) * TURN / 12 + pick(i, 76, 0, 3000);
        let top = facing(a, 0).scale(r - cm(pick(i, 77, 20, 120)));
        let len = mul(hang, ratio(pick(i, 78, 50, 100), 100));
        let segs = 5;
        let mut prev = top;
        for s in 1..=segs {
            let t = ratio(s, segs);
            let sway = mul(wave(tick, 6 + (i % 3) as i32, (i as i32) * 5_000), mul(t, cm(80)));
            let p = top.add(v3(sway, -mul(len, t), sway / 2));
            bar(b, f, Mesh::Taper, prev, p, cm(14) - s * cm(1), VINE, 0);
            if s % 2 == 0 {
                put_turned(b, f, Mesh::Shard, p, &Affine::rot_y(a + s * 9000), v3(cm(40), cm(50), cm(8)), LEAF_LIGHT, 0);
            }
            prev = p;
        }
    }
    // the shaft of light: a soft cone of warm gold, and a pool at its foot
    halo(
        b,
        f,
        Mesh::Cone,
        v3(0, -shaft / 2, 0),
        v3(r * 2 + r / 2, shaft, r * 2 + r / 2),
        rgb(255, 196, 110),
        ONE / 5,
    );
    halo(b, f, Mesh::Sphere, v3(0, -shaft, 0), v3(r * 3, int(2), r * 3), rgb(255, 200, 120), ONE / 4);
    // the bit of sky seen through it
    put(b, f, Mesh::Hex, v3(0, int(3), 0), v3(r * 2, cm(10), r * 2), mix(ZENITH, BAND, ONE / 3), SKY_GLOW);
}

// ---------------------------------------------------------------------------
// Jungle trees for the sunset.

/// The trees of the sunset jungle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tree {
    /// a tall kapok: straight grey trunk, buttress roots, a flat crown
    /// spread high over the rest (silhouettes against the red sky)
    Kapok,
    /// a leaning palm, its fronds drooping
    Palm,
    /// a strangler fig: a knotted trunk, aerial roots hanging, a wide
    /// dark crown
    Fig,
    /// a flowering tree, its crown in magenta blossom
    Blossom,
}

/// A jungle tree of kind `t` with its foot at the origin; `k` varies its
/// height and lean. Heights at scale 1: Kapok about 24 m, Palm 11 m, Fig
/// 14 m, Blossom 9 m.
pub fn jungle_tree(b: &mut Batch, f: &Affine, t: Tree, k: u32) {
    let v = ratio(pick(k, 79, 85, 115), 100);
    match t {
        Tree::Kapok => {
            let h = mul(int(22), v);
            bar(b, f, Mesh::Taper, V3::ZERO, v3(0, h, 0), cm(140), TRUNK_LIGHT, 0);
            for i in 0..5 {
                let a = i * TURN / 5 + (k as i32) * 701;
                let out = facing(a, 0);
                put_turned(b, f, Mesh::Wedge, out.scale(cm(110)).add(v3(0, cm(120), 0)), &Affine::rot_y(a + fx::HALF), v3(cm(30), cm(240), cm(180)), TRUNK, 0);
            }
            // branches and a flat, layered crown
            for i in 0..4 {
                let a = i * TURN / 4 + (k as i32) * 333;
                let tip = facing(a, 0).scale(int(5)).add(v3(0, h + int(1), 0));
                bar(b, f, Mesh::Taper, v3(0, h - int(2), 0), tip, cm(50), TRUNK, 0);
                put(b, f, Mesh::Barrel, tip.add(v3(0, cm(80), 0)), v3(int(7), int(2), int(6)), if i % 2 == 0 { LEAF } else { LEAF_DARK }, 0);
            }
            put(b, f, Mesh::Barrel, v3(0, h + int(2), 0), v3(int(9), int(3), int(9)), LEAF, 0);
            put(b, f, Mesh::Blob, v3(0, h + int(3) + cm(50), 0), v3(int(6), int(2), int(6)), LEAF_LIGHT, 0);
        }
        Tree::Palm => {
            let h = mul(int(10), v);
            let lean = cm(pick(k, 82, 150, 300));
            let a = (k as i32) * 4_111;
            let side = facing(a, 0);
            let mut prev = V3::ZERO;
            for s in 1..=4 {
                let t = ratio(s, 4);
                let p = side.scale(mul(lean, mul(t, t))).add(v3(0, mul(h, t), 0));
                bar(b, f, Mesh::Taper, prev, p, cm(46) - s * cm(4), TRUNK_LIGHT, 0);
                prev = p;
            }
            put(b, f, Mesh::Ico, prev, v3(cm(90), cm(70), cm(90)), TRUNK, 0);
            for i in 0..8 {
                let fa = i * TURN / 8 + a;
                let droop = Affine::rot_y(fa).then(&Affine::rot_x(deg(70 + (i % 3) * 12)));
                put_turned(b, f, Mesh::Shard, prev.add(facing(fa, 0).scale(cm(170)).add(v3(0, -cm(30), 0))), &droop, v3(cm(80), int(4), cm(10)), if i % 2 == 0 { PALM } else { LEAF_LIGHT }, 0);
            }
            // coconuts
            for i in 0..3 {
                put(b, f, Mesh::Ico, prev.add(facing(i * 21_000, 0).scale(cm(40)).add(v3(0, -cm(50), 0))), v3(cm(36), cm(36), cm(36)), TRUNK, 0);
            }
        }
        Tree::Fig => {
            let h = mul(int(12), v);
            // a knot of trunks twisting together
            for i in 0..3 {
                let a = i * TURN / 3 + (k as i32) * 2_000;
                let foot = facing(a, 0).scale(cm(70));
                let mid = facing(a + deg(60), 0).scale(cm(40)).add(v3(0, h / 2, 0));
                bar(b, f, Mesh::Taper, foot, mid, cm(80), TRUNK, 0);
                bar(b, f, Mesh::Taper, mid, v3(0, h, 0), cm(60), TRUNK_LIGHT, 0);
            }
            // the wide crown
            for i in 0..5 {
                let a = i * TURN / 5 + (k as i32) * 999;
                let at = facing(a, 0).scale(int(4)).add(v3(0, h + cm(pick(k + i as u32, 83, 0, 200)), 0));
                put(b, f, Mesh::Blob, at, v3(int(6), int(3), int(6)), if i % 2 == 0 { LEAF_DARK } else { LEAF }, 0);
                // aerial roots hanging from it
                for j in 0..2 {
                    let rx = at.add(facing(a + deg(j * 40), 0).scale(cm(150)));
                    bar(b, f, Mesh::Cyl, rx.add(v3(0, -int(1), 0)), v3(rx.x, cm(pick(k + j as u32, 84, 20, 300)), rx.z), cm(10), TRUNK_LIGHT, 0);
                }
            }
            put(b, f, Mesh::Blob, v3(0, h + int(2), 0), v3(int(8), int(3), int(8)), LEAF, 0);
        }
        Tree::Blossom => {
            let h = mul(int(7), v);
            bar(b, f, Mesh::Taper, V3::ZERO, v3(0, h, 0), cm(50), TRUNK, 0);
            for i in 0..4 {
                let a = i * TURN / 4 + (k as i32) * 555;
                let tip = facing(a, 0).scale(int(2)).add(v3(0, h + int(1), 0));
                bar(b, f, Mesh::Taper, v3(0, h - int(1), 0), tip, cm(24), TRUNK, 0);
                put(b, f, Mesh::Blob, tip.add(v3(0, cm(60), 0)), v3(int(4), int(2), int(4)), if i % 2 == 0 { BLOSSOM } else { shade(BLOSSOM, 80) }, 0);
            }
            put(b, f, Mesh::Blob, v3(0, h + int(2), 0), v3(int(5), int(2) + cm(50), int(5)), BLOSSOM, 0);
            put(b, f, Mesh::Blob, v3(0, h + cm(60), 0), v3(int(4), int(1), int(4)), LEAF_DARK, 0);
        }
    }
}

// ---------------------------------------------------------------------------
// The sky round the eye.

/// The dome's radius: inside the far plane.
const DOME: i32 = int(560);

/// The sunset sky round `eye`, the sun at bearing `sun_yaw`: a dome from
/// the yellow haze at the horizon to crimson overhead (`scene.frag` paints
/// it from the fog's colour to `uSky` × the dome's colour, so the
/// colourblind mode and JOJO MODE still recolour it), an orange band
/// laid over its low part, the sun's disc on the horizon in a gold
/// corona, and long low streaks of cloud lit gold toward the sun and
/// crimson away from it. All at a fixed distance, as if infinitely far.
pub fn sky(b: &mut Batch, eye: V3, sun_yaw: i32) {
    let deep = [0, 1, 2].map(|k| fx::div(ZENITH[k], SKY[k].max(1)).min(ONE));
    b.push(Mesh::Sphere, &placed(eye, 0, v3(DOME * 2, DOME * 2, DOME * 2)), deep, DOME_GLOW);
    // the orange band: a ring of soft glows round the horizon (sky glows,
    // unfogged), stronger toward the sun
    let n = 16;
    let at_r = DOME - int(40);
    for i in 0..n {
        let a = sun_yaw + i * TURN / n;
        let toward = (fx::cos(a - sun_yaw) + ONE) / 2;
        let at = eye.add(facing(a, deg(12)).scale(at_r));
        b.push_halo(
            Mesh::Sphere,
            &placed(at, a, v3(int(260), int(110), int(60))),
            BAND,
            ONE * 3 + ONE / 4 + mul(toward, ONE / 3),
        );
    }
    // clouds: long flat streaks low in the sky
    for i in 0..9u32 {
        let a = sun_yaw + deg(pick(i, 85, -140, 140));
        let el = deg(pick(i, 86, 4, 16));
        let at = eye.add(facing(a, el).scale(DOME - int(60)));
        let toward = (fx::cos(a - sun_yaw) + ONE) / 2;
        let c = mix(rgb(150, 40, 70), rgb(255, 170, 70), mul(toward, toward));
        b.push(
            Mesh::Bevel,
            &placed(at, a + fx::QUARTER, v3(int(pick(i, 87, 70, 140)), int(pick(i, 88, 4, 8)), int(pick(i, 89, 12, 24)))),
            c,
            SKY_GLOW,
        );
    }
    // the sun on the horizon, in a corona
    let sun_at = eye.add(facing(sun_yaw, SUN_HIGH).scale(DOME - int(80)));
    b.push(Mesh::Sphere, &placed(sun_at, 0, v3(int(36), int(36), int(36))), SUN, SKY_GLOW);
    b.push_halo(Mesh::Sphere, &placed(sun_at, 0, v3(int(160), int(130), int(160))), rgb(255, 210, 90), ONE * 3 + ONE * 7 / 10);
}

// ---------------------------------------------------------------------------
// The model gallery (`?gallery=<key>`, `app/gallery.rs`).

/// The keys `app/gallery.rs` shows from here.
pub const GALLERY: [&str; 15] = [
    "bananastatue",
    "bananabig",
    "moai",
    "moaibig",
    "turtle",
    "bridge",
    "mouth",
    "wall",
    "traps",
    "pit",
    "pile",
    "skylight",
    "trees",
    "glyph",
    "ruinsall",
];

/// The middle and rough radius of a gallery model, metres.
pub fn framing(key: &str) -> (V3, i32) {
    match key {
        "bananastatue" => (v3(0, int(3), 0), int(4)),
        "bananabig" => (v3(0, int(10), 0), int(16)),
        "moai" => (v3(0, int(4), 0), int(5)),
        "moaibig" => (v3(0, int(12), 0), int(18)),
        "turtle" => (v3(0, cm(50), 0), int(3)),
        "bridge" => (v3(0, cm(60), int(6)), int(8)),
        "mouth" => (v3(0, int(10), 0), int(52)),
        "wall" => (v3(0, int(4), 0), int(8)),
        "traps" => (v3(0, 0, 0), int(6)),
        "pit" => (v3(0, 0, 0), int(5)),
        "pile" => (v3(0, int(9), 0), int(18)),
        "skylight" => (v3(0, -int(10), 0), int(22)),
        "trees" => (v3(0, int(10), 0), int(24)),
        "glyph" => (V3::ZERO, int(2)),
        _ => (v3(0, int(6), 0), int(40)),
    }
}

/// Draws gallery model `key` in frame `f` (turning) at `tick`.
pub fn gallery(b: &mut Batch, key: &str, f: &Affine, tick: u32) {
    let scaled = |s: i32| f.then(&Affine::scale(v3(s, s, s)));
    match key {
        "bananastatue" => banana_statue(b, f, 1, false),
        "bananabig" => banana_statue(b, &scaled(ONE * 3), 2, true),
        "moai" => moai_head(b, f, 1, false),
        "moaibig" => moai_head(b, &scaled(ONE * 2 + ONE / 2), 3, true),
        "turtle" => turtle(b, f, turtle_phase(tick, 0)),
        "bridge" => {
            rope_bridge(b, f, int(12));
            bridge_posts(b, f);
            // a gorilla-wide ruler: three 2 m blocks across the deck
            for i in -1..=1 {
                put(b, f, Mesh::Cube, v3(int(2) * i, cm(5), int(6)), v3(int(2) - cm(10), cm(4), cm(40)), [ONE, 0, 0], ONE);
            }
        }
        "mouth" => temple_mouth(b, f, tick),
        "wall" => {
            let w = int(14);
            let h = int(8);
            let g = f.then(&Affine::translate(v3(0, int(4), 0)));
            stone_panel(b, &g, w, h);
            for (i, s) in glyph_spots(w, h).into_iter().enumerate() {
                banana_glyph(b, &g.then(&Affine::translate(s.add(v3(0, 0, cm(10))))), tick, i as u32);
            }
        }
        "traps" => {
            put(b, f, Mesh::Cube, v3(0, -cm(30), 0), v3(int(14), cm(40), int(10)), STONE_DARK, 0);
            spike_trap(b, &f.then(&Affine::translate(v3(-int(4), 0, -int(2)))), ONE);
            spike_trap(b, &f.then(&Affine::translate(v3(-int(4), 0, int(2)))), ONE / 3);
            trip_wire(b, &f.then(&Affine::translate(v3(-int(1), 0, int(3)))), int(5), false);
            trip_wire(b, &f.then(&Affine::translate(v3(-int(1), 0, int(4)))), int(5), true);
            trapdoor(b, &f.then(&Affine::translate(v3(int(3), 0, -int(1)))), int(4), int(4), ONE * 3 / 5);
        }
        "pit" => {
            trapdoor(b, f, int(6), int(6), ONE);
            pit_spikes(b, &f.then(&Affine::translate(v3(0, -int(4), 0))), int(6), int(6), tick);
        }
        "pile" => banana_pile(b, f, tick),
        "skylight" => {
            skylight(b, f, int(6), int(14), int(30), tick);
            banana_pile(b, &f.then(&Affine::translate(v3(0, -int(30), 0))).then(&Affine::scale(v3(ONE / 2, ONE / 2, ONE / 2))), tick);
        }
        "trees" => {
            jungle_tree(b, &f.then(&Affine::translate(v3(-int(16), 0, 0))), Tree::Kapok, 1);
            jungle_tree(b, &f.then(&Affine::translate(v3(-int(4), 0, int(4)))), Tree::Palm, 2);
            jungle_tree(b, &f.then(&Affine::translate(v3(int(7), 0, 0))), Tree::Fig, 3);
            jungle_tree(b, &f.then(&Affine::translate(v3(int(18), 0, int(4)))), Tree::Blossom, 4);
        }
        "glyph" => banana_glyph(b, f, tick, 0),
        // a sample of everything, as it might stand by the river
        _ => {
            put(b, f, Mesh::Cube, v3(0, -cm(30), 0), v3(int(90), cm(60), int(60)), rgb(84, 92, 44), 0);
            banana_statue(b, &pose(v3(-int(14), 0, -int(6)), deg(20), ONE * 2), 1, true);
            moai_head(b, &pose(v3(int(10), 0, -int(10)), -deg(15), ONE * 2), 2, true);
            jungle_tree(b, &pose(v3(-int(30), 0, -int(18)), 0, ONE), Tree::Kapok, 5);
            jungle_tree(b, &pose(v3(int(28), 0, -int(16)), 0, ONE), Tree::Fig, 6);
            jungle_tree(b, &pose(v3(-int(26), 0, int(8)), 0, ONE), Tree::Palm, 7);
            jungle_tree(b, &pose(v3(int(26), 0, int(10)), 0, ONE), Tree::Blossom, 8);
            put(b, f, Mesh::Cube, v3(0, cm(5), int(14)), v3(int(60), cm(10), int(10)), WATER, ONE / 4);
            turtle(b, &pose(v3(-int(4), 0, int(14)), deg(90), ONE), turtle_phase(tick, 1));
            turtle(b, &pose(v3(int(8), 0, int(12)), -deg(90), ONE), turtle_phase(tick, 2));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_ruins_model_draws_something() {
        for key in GALLERY {
            let mut b = Batch::new();
            gallery(&mut b, key, &Affine::IDENTITY, 240);
            assert!(b.count() > 10, "{key}");
        }
    }

    #[test]
    fn the_bridge_is_three_gorillas_wide() {
        // a gorilla drawn at the race's 2× spans about 2 m arms out
        assert_eq!(BRIDGE_WIDTH, int(6));
        // the planks span the width, the hand ropes stand just outside it
        let mut b = Batch::new();
        rope_bridge(&mut b, &Affine::IDENTITY, int(10));
        assert!(b.count_of(Mesh::Cube) >= 14, "{}", b.count_of(Mesh::Cube));
    }

    #[test]
    fn the_glyphs_pulse_slowly_and_never_go_dark() {
        let all: Vec<i32> = (0..(TICKS_PER_SECOND * 4) as u32)
            .map(|t| glyph_pulse(t, 0))
            .collect();
        let lo = *all.iter().min().unwrap();
        let hi = *all.iter().max().unwrap();
        assert!(lo >= ONE * 29 / 100 && hi <= ONE, "{lo} {hi}");
        assert!(hi - lo > ONE / 2, "{lo} {hi}");
        // slow: a tick changes it by under 1%
        for w in all.windows(2) {
            assert!((w[1] - w[0]).abs() < ONE / 100);
        }
        // the glyphs breathe out of step
        assert_ne!(glyph_pulse(100, 0), glyph_pulse(100, 1));
    }

    #[test]
    fn the_turtle_strokes_from_the_tick() {
        let a = turtle_phase(0, 0);
        let b = turtle_phase(45, 0);
        assert_ne!(a, b);
        assert_eq!(turtle_phase(0, 0), turtle_phase(180, 0));
        let mut one = Batch::new();
        let mut two = Batch::new();
        turtle(&mut one, &Affine::IDENTITY, a);
        turtle(&mut two, &Affine::IDENTITY, b);
        assert_ne!(one.finish(1).0, two.finish(1).0);
    }

    #[test]
    fn the_sunset_is_its_own_palette() {
        use crate::map::Map;
        let l = look();
        for m in [Map::Jungle, Map::City, Map::Ski, Map::Sector] {
            let o = crate::scene::look(m);
            assert_ne!((o.sky, o.fog), (l.sky, l.fog), "{m:?}");
        }
        // the sun is low, the light warm (red over green over blue)
        assert!(l.light_dir.y < ONE / 6 && l.light_dir.y > 0);
        assert!(l.light[0] > l.light[1] && l.light[1] > l.light[2]);
        // the horizon yellow (red and green high), the zenith crimson
        assert!(FOG[0] > ONE * 9 / 10 && FOG[1] > ONE * 7 / 10);
        assert!(ZENITH[0] > ZENITH[2] * 2 && ZENITH[1] < ONE / 10);
        // the dome's colour is a share of the sky's (at most 1 each)
        for k in 0..3 {
            assert!(ZENITH[k] <= SKY[k]);
        }
        // inside is darker than outside
        let i = look_inside();
        let sum = |c: Rgb| c[0] + c[1] + c[2];
        assert!(sum(i.sky) * 2 < sum(l.sky) && sum(i.light) * 2 < sum(l.light));
    }
}
