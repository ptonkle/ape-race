//! Banana Bay Battle's scenery and light (`bay.rs` is the contract it draws
//! from; `baymodels.rs` holds the models).
//!
//! **The hour: 3 pm in a vaguely Caribbean bay, clear** (step 5, 2026-10-15,
//! SECOND-ORDER BS rows). The sun stands 40° up (about where it is at
//! 3 pm in mid-October at 15° N), 55° to the left of the course's first
//! heading (`SUN_OFF`): high and warm, but from the side, so hulls and
//! sails take a lit side and a shaded side instead of a flat top light.
//! The sky is a dome (MAPS B2) from a pale turquoise haze at the horizon
//! to a deep clear blue overhead, with a few fair-weather cumulus low
//! round it; the shadows take the sky's blue and the sea's turquoise
//! bounces up under everything (`Look.ground`).
//!
//! **The sea** is drawn each frame round the eye (`dynamic`): tiles in
//! bands, each twice the size of the last (MAPS: fine detail on the
//! water), fixed to a world grid so they hold still as the eye moves, each
//! tilted to the swell's slope at its middle and lifted to
//! `Water::height` (the same function the physics uses; the bands further
//! out leave out the swells shorter than two of their tiles). Turquoise in
//! the shallows round the island and the dock, deeper blue out in the bay,
//! lighter on the crests; white flecks of foam on the crests and twinkling
//! sun glints near the eye. A flat deep-blue sheet under it all fills any
//! seam between tiles and the sea past the bands.
//!
//! **A Squall** (`Weather::Squall`, each player's own view, like the Ski
//! Slope's snow): a dark slate sky and a close grey fog (`squall`), heavy
//! low cloud, rain (`snow::flakes`), the sea greyer and its swell drawn
//! twice as high with a short chop on it (drawing only: the physics' water
//! is `Water::height` whatever the weather), whitecaps, and lightning from
//! the tick alone (`lightning`): a jagged bolt far off and the whole sky
//! flashing.
//!
//! STAND-IN still (the art step replaces it): the static scenery is the
//! Jungle's props, start trees and finish arch without its ground.

use crate::bay::{BayLayout, Water};
use crate::course::Course;
use crate::fx::{self, cm, deg, int, mul, sin, ONE, TURN};
use crate::geom::{facing, v3, yaw_of, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, placed, rgb, rod, shade, Batch, Camera, Rgb};
use crate::rules::TICKS_PER_SECOND;
use crate::scene::{Look, Scenery};
use crate::snow::Weather;

/// The sun's height over the horizon: 3 pm, high but not overhead.
pub const SUN_HIGH: i32 = deg(40);
/// The sun's bearing off the course's first heading (to the left, +yaw):
/// from the side and a little ahead, so the glints on the water are in
/// view.
pub const SUN_OFF: i32 = deg(55);
/// The sunlight: warm, bright.
pub const LIGHT: Rgb = rgb(276, 252, 218);
/// The light from above (the shadows' colour) and the sky's colour.
pub const SKY: Rgb = rgb(128, 184, 240);
/// The dome overhead: a deep clear blue.
pub const ZENITH: Rgb = rgb(28, 104, 222);
/// The haze at the horizon and the fog: a pale turquoise.
pub const FOG: Rgb = rgb(176, 226, 232);
/// The sea's bounce under things.
pub const BOUNCE: Rgb = rgb(64, 168, 172);
/// The sun's disc.
const SUN: Rgb = rgb(255, 250, 226);

/// The water: deep blue out in the bay, a clearer blue, turquoise over the
/// sand of the shallows, and the paler green-blue of a crest.
pub const DEEP: Rgb = rgb(16, 84, 148);
pub const BAY: Rgb = rgb(24, 122, 174);
pub const SHALLOW: Rgb = rgb(50, 196, 192);
const CREST: Rgb = rgb(116, 214, 222);
const FOAM: Rgb = rgb(248, 252, 252);
const GLINT: Rgb = rgb(255, 246, 214);

/// A squall's sky, fog, the dome overhead (× the sky) and its sea.
const SQUALL_SKY: Rgb = rgb(98, 110, 124);
const SQUALL_FOG: Rgb = rgb(118, 130, 138);
const SQUALL_ZENITH: Rgb = rgb(56, 66, 80);
const SQUALL_SEA: Rgb = rgb(52, 78, 88);
const STORM_CLOUD: Rgb = rgb(78, 86, 96);
/// A lightning flash's colour, and the bolt's.
const FLASH: Rgb = rgb(226, 230, 255);
const BOLT: Rgb = rgb(240, 236, 255);

/// Shown full colour and unfogged, as the sky is (`scene.frag`).
const SKY_GLOW: i32 = ONE * 2;
/// The sky's dome's mark for `scene.frag` (over 3.5).
const DOME_GLOW: i32 = ONE * 4;
/// The dome's radius: inside the far plane.
const DOME: i32 = int(560);

/// The sea's tile bands round the eye: each band's tile side and how far
/// (each way, a square) it reaches (metres).
const BANDS: [(i32, i32); 4] = [(4, 40), (8, 96), (16, 208), (32, 416)];
/// How much higher a squall's swell is drawn (Q16; drawing only).
const ROUGH: i32 = ONE * 2;

/// The sun's bearing on this course.
pub fn sun_yaw(c: &Course) -> i32 {
    yaw_of(c.heading[0]) + SUN_OFF
}

/// Banana Bay Battle's light before a course is built: the sun at
/// `SUN_OFF` from +z.
pub fn look() -> Look {
    look_toward(SUN_OFF)
}

/// The light on this course: the sun from its bearing.
pub fn look_for(c: &Course) -> Look {
    look_toward(sun_yaw(c))
}

/// 3 pm with the sun at bearing `sun_yaw`.
fn look_toward(sun_yaw: i32) -> Look {
    Look {
        sky: SKY,
        fog: FOG,
        // the open sea sees far: the haze from 110 m, nothing past 380 m
        fog_range: (int(110), int(380)),
        light_dir: facing(sun_yaw, SUN_HIGH),
        light: LIGHT,
        ground: BOUNCE,
        rim: [0; 3],
        far: crate::render::FAR,
    }
}

/// A map's look in a squall: the sky and fog dark slate grey, the fog
/// close (from 25 m, nothing past 170 m), the sun dim and from high up,
/// the sea's bounce grey.
pub fn squall(look: Look) -> Look {
    Look {
        sky: mix(look.sky, SQUALL_SKY, ONE * 9 / 10),
        fog: mix(look.fog, SQUALL_FOG, ONE * 9 / 10),
        fog_range: (int(25), int(170)),
        light_dir: v3(look.light_dir.x / 3, ONE, look.light_dir.z / 3).norm(),
        light: shade(look.light, 52),
        ground: mix(look.ground, SQUALL_SEA, ONE * 3 / 4),
        ..look
    }
}

/// How often lightning may strike (ticks): each window has at most one
/// strike, three windows in four.
const STRIKE_EVERY: u32 = 7 * TICKS_PER_SECOND as u32;
/// How long a strike lights the sky (ticks).
const STRIKE_LONG: u32 = 40;

/// A window's strike: when in the window it falls (ticks), or `None`.
fn strike(window: u32) -> Option<u32> {
    let h = hash(window, 0x11);
    if h % 4 == 3 {
        return None;
    }
    Some((h >> 4) % (STRIKE_EVERY - STRIKE_LONG))
}

/// How bright a squall's lightning is at `tick` (0..=ONE): dark most of
/// the time; a strike flickers (bright, dim, bright again) and fades over
/// a third of a second. From the tick alone.
pub fn lightning(tick: u32) -> i32 {
    let window = tick / STRIKE_EVERY;
    let Some(at) = strike(window) else {
        return 0;
    };
    let into = tick % STRIKE_EVERY;
    if into < at || into >= at + STRIKE_LONG {
        return 0;
    }
    let k = (into - at) as i32;
    match k {
        0..=3 => ONE,
        4..=7 => ONE / 4,
        8..=11 => ONE * 4 / 5,
        _ => ONE * 3 / 5 * (STRIKE_LONG as i32 - k) / (STRIKE_LONG as i32 - 12),
    }
}

/// A squall's look while lightning flashes: the sky, the fog and the light
/// whitened by the flash.
pub fn flashed(look: Look, tick: u32) -> Look {
    let f = lightning(tick);
    if f == 0 {
        return look;
    }
    Look {
        sky: mix(look.sky, FLASH, f * 3 / 5),
        fog: mix(look.fog, FLASH, f * 2 / 5),
        light: mix(look.light, rgb(300, 300, 320), f / 2),
        ..look
    }
}

// ---------------------------------------------------------------------------
// Helpers.

fn hash(k: u32, salt: u32) -> u32 {
    let mut x = k.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^ (x >> 12)
}

fn pick(k: u32, salt: u32, lo: i32, hi: i32) -> i32 {
    lo + (hash(k, salt) % ((hi - lo).max(0) as u32 + 1)) as i32
}

/// A tile's id from its band and grid cell.
fn cell_id(band: usize, ix: i32, iz: i32) -> u32 {
    (ix as u32).wrapping_mul(0x8DA6_B343)
        ^ (iz as u32).wrapping_mul(0xD816_3841)
        ^ (band as u32).wrapping_mul(0xCB1A_B31F)
}

/// Banana Bay Battle's scenery, chunked. STAND-IN: the Jungle's props,
/// start trees and finish without its ground (the sea is drawn round the
/// eye each frame instead, `dynamic`).
pub fn scenery(c: &Course, _b: &BayLayout) -> Scenery {
    crate::scene::jungle_scenery_on(c, false)
}

// ---------------------------------------------------------------------------
// What moves: the sky and the sea round the eye.

/// Everything on Banana Bay Battle drawn each frame round the eye: the sky
/// (the dome, the clouds, the sun or the storm and its lightning) and the
/// sea. `tick` drives the swell (the race's tick in a race, so the sea is
/// where the physics has it); `t` is the frame's clock (the glints'
/// twinkle, the lightning).
pub fn dynamic(
    b: &mut Batch,
    c: &Course,
    bay: &BayLayout,
    cam: &Camera,
    tick: u32,
    t: u32,
    weather: Weather,
) {
    let storm = weather == Weather::Squall;
    sky(b, cam.eye, sun_yaw(c), storm, t);
    sea(b, bay, cam, tick, t, storm);
}

/// The sky round `eye`: the dome, and either the sun and a few fair-weather
/// cumulus, or a squall's low dark cloud and lightning.
pub fn sky(b: &mut Batch, eye: V3, sun_yaw: i32, storm: bool, t: u32) {
    let (zenith, sky) = if storm {
        (SQUALL_ZENITH, SQUALL_SKY)
    } else {
        (ZENITH, SKY)
    };
    // the dome's colour is a multiple of the sky's, so the colourblind
    // mode and JOJO MODE (which change the sky) change it too
    let deep = [0, 1, 2].map(|k| fx::div(zenith[k], sky[k].max(1)).min(ONE));
    b.push(
        Mesh::Sphere,
        &placed(eye, 0, v3(DOME * 2, DOME * 2, DOME * 2)),
        deep,
        DOME_GLOW,
    );
    if storm {
        storm_clouds(b, eye, t);
        bolt(b, eye, t);
        return;
    }
    // the sun, high and to the side, in a soft corona
    let sun_at = eye.add(facing(sun_yaw, SUN_HIGH).scale(DOME - int(80)));
    b.push(
        Mesh::Sphere,
        &placed(sun_at, 0, v3(int(26), int(26), int(26))),
        SUN,
        SKY_GLOW,
    );
    b.push_halo(
        Mesh::Sphere,
        &placed(sun_at, 0, v3(int(120), int(120), int(120))),
        rgb(255, 244, 200),
        ONE * 3 + ONE / 2,
    );
    cumulus(b, eye, sun_yaw);
}

/// A few fair-weather cumulus low round the horizon: flat grey-blue bases,
/// heaped white tops, brighter toward the sun.
fn cumulus(b: &mut Batch, eye: V3, sun_yaw: i32) {
    for i in 0..9u32 {
        let a = deg(pick(i, 1, 0, 359)) + i as i32 * (TURN / 9);
        let el = deg(pick(i, 2, 5, 17));
        let at = eye.add(facing(a, el).scale(DOME - int(70)));
        let toward = (fx::cos(a - sun_yaw) + ONE) / 2;
        let top = mix(rgb(236, 242, 250), rgb(255, 255, 252), toward);
        let base = mix(rgb(176, 196, 220), rgb(214, 226, 240), toward);
        let w = int(pick(i, 3, 26, 52));
        let side = facing(a + TURN / 4, 0);
        // the flat base
        b.push(
            Mesh::Geo,
            &placed(at, a, v3(w * 2, w / 4, w)),
            base,
            SKY_GLOW,
        );
        // the heaps on it
        let heaps = 3 + (hash(i, 4) % 3) as i32;
        for k in 0..heaps {
            let s = fx::div(int(k) - int(heaps - 1) / 2, int(heaps)) * 2;
            let r = mul(w, ONE * pick(i * 7 + k as u32, 5, 45, 80) / 100);
            let p = at
                .add(side.scale(mul(w, s)))
                .add(v3(0, r * 2 / 3 + w / 8, 0));
            b.push(
                Mesh::Geo,
                &placed(p, a + k * 900, v3(r * 2, r * 3 / 2, r * 2)),
                top,
                SKY_GLOW,
            );
        }
    }
}

/// A squall's cloud: heavy dark slabs low all round, close in.
fn storm_clouds(b: &mut Batch, eye: V3, t: u32) {
    let drift = (t as i32).wrapping_mul(6) % TURN;
    for i in 0..28u32 {
        let a = i as i32 * (TURN / 28) + deg(pick(i, 21, -8, 8)) + drift;
        let el = deg(pick(i, 22, 2, 18));
        let at = eye.add(facing(a, el).scale(DOME - int(90)));
        let g = pick(i, 23, 0, 18);
        let c = mix(STORM_CLOUD, rgb(48, 54, 62), ONE * g / 18);
        b.push(
            Mesh::Geo,
            &placed(
                at,
                a,
                v3(
                    int(pick(i, 24, 200, 300)),
                    int(pick(i, 25, 30, 60)),
                    int(120),
                ),
            ),
            c,
            SKY_GLOW,
        );
    }
}

/// A lightning bolt while a strike flashes: a jagged white line from the
/// cloud down to the sea, far off at the strike's own bearing.
fn bolt(b: &mut Batch, eye: V3, t: u32) {
    let f = lightning(t);
    if f < ONE / 2 {
        return;
    }
    let window = t / STRIKE_EVERY;
    let a = deg(pick(window, 31, 0, 359));
    let foot = eye.flat().add(facing(a, 0).scale(int(420)));
    let side = facing(a + TURN / 4, 0);
    let top_y = eye.y + int(150);
    let mut p = v3(foot.x, top_y, foot.z);
    let steps = 8;
    for k in 1..=steps {
        let y = top_y - (top_y - eye.y + int(40)) * k / steps;
        let jag = int(pick(window * 16 + k as u32, 32, -18, 18));
        let q = v3(foot.x, y, foot.z).add(side.scale(jag));
        b.push(Mesh::Cube, &rod(p, q, int(2)), BOLT, SKY_GLOW);
        p = q;
    }
    b.push_halo(
        Mesh::Sphere,
        &placed(
            foot.add(v3(0, eye.y + int(60), 0)),
            0,
            v3(int(90), int(200), int(90)),
        ),
        FLASH,
        ONE * 3 + mul(f, ONE / 2),
    );
}

/// The drawn sea surface's height at (x, z): `Water::height` (the
/// physics' own) in clear weather; in a squall its swell drawn `ROUGH`
/// times as high, with a short fast chop on it (drawing only). `min_len`
/// leaves out swells shorter than it (the far bands' big tiles).
pub fn surface(w: &Water, x: i32, z: i32, tick: u32, storm: bool, min_len: i32) -> i32 {
    surface_of(
        &longer_than(w, min_len),
        x,
        z,
        tick,
        storm,
        min_len <= int(8),
    )
}

/// The swells of `w` at least `min_len` long (all of them for 0).
fn longer_than(w: &Water, min_len: i32) -> Water {
    Water {
        level: w.level,
        waves: w
            .waves
            .iter()
            .copied()
            .filter(|v| v.length >= min_len)
            .collect(),
    }
}

/// `surface` from the swells already chosen, with a squall's chop or not.
fn surface_of(w: &Water, x: i32, z: i32, tick: u32, storm: bool, chop: bool) -> i32 {
    let swell = w.swell(x, z, tick);
    if !storm {
        return w.level + swell;
    }
    let chop = if chop {
        mul(
            cm(14),
            sin((x / 11)
                .wrapping_add(z / 15)
                .wrapping_sub(tick as i32 * 220)),
        ) + mul(
            cm(9),
            sin((z / 9).wrapping_sub(x / 17).wrapping_sub(tick as i32 * 290)),
        )
    } else {
        0
    };
    w.level + mul(swell, ROUGH) + chop
}

/// How shallow the water is at (x, z), 0 (the open bay) to ONE (by the
/// island's beach or under the dock).
fn shallows(bay: &BayLayout, x: i32, z: i32) -> i32 {
    let p = v3(x, 0, z);
    let isl = bay.island;
    let d = p.sub(isl.centre.flat()).len() - isl.radius;
    let by_island = ONE - fx::div(d.clamp(0, int(60)), int(60));
    // the dock runs out from the shore: shallow along it and behind it
    let (a, e) = (bay.dock.from.flat(), bay.dock.to.flat());
    let along = e.sub(a);
    let len = along.len().max(1);
    let dir = along.scale(fx::div(ONE, len));
    let s = p.sub(a).dot(dir).min(len);
    let off = p.sub(a.add(dir.scale(s))).len();
    let by_dock = ONE - fx::div((off - int(14)).clamp(0, int(50)), int(50));
    let by_dock = mul(by_dock, ONE * 9 / 10);
    by_island.max(by_dock)
}

/// The bay's slow large patches, deeper and clearer (0..ONE), so the open
/// water is not one flat colour.
fn patches(x: i32, z: i32) -> i32 {
    let a = sin(x / 53 + z / 211);
    let b = sin(z / 71 - x / 97);
    (a + b + ONE * 2) / 4
}

/// The water's colour at (x, z) with the drawn swell `rise` there (of a
/// greatest `peak`): deep to clear blue by `patches`, turquoise by
/// `shallows`, lighter on a crest, a squall's grey over it all.
fn water_colour(
    bay: &BayLayout,
    x: i32,
    z: i32,
    rise: i32,
    peak: i32,
    storm: bool,
    id: u32,
) -> Rgb {
    let open = mix(DEEP, BAY, patches(x, z));
    let c = mix(open, SHALLOW, shallows(bay, x, z));
    let crest = fx::div(rise.max(0), peak.max(cm(10))).min(ONE);
    let c = mix(c, CREST, mul(crest, ONE * 2 / 5));
    let c = shade(c, 99 + (hash(id, 41) % 3) as i32);
    if storm {
        mix(c, SQUALL_SEA, ONE * 3 / 5)
    } else {
        c
    }
}

/// The sea round the eye: tiles in bands (`BANDS`), each tilted to the
/// drawn swell at its middle, coloured by `water_colour`, with foam and
/// glints near the eye, over a flat deep sheet. Tiles well behind the
/// camera are not drawn.
pub fn sea(b: &mut Batch, bay: &BayLayout, cam: &Camera, tick: u32, t: u32, storm: bool) {
    let w = &bay.water;
    let eye = cam.eye;
    let rough = if storm { ROUGH } else { ONE };
    let peak = mul(w.peak(), rough) + if storm { cm(23) } else { 0 };
    // the sheet under it all, just under the lowest trough
    let base = if storm {
        mix(BAY, SQUALL_SEA, ONE * 3 / 5)
    } else {
        mix(DEEP, BAY, ONE / 2)
    };
    b.push(
        Mesh::Quad,
        &placed(
            v3(eye.x, w.level - peak - cm(12), eye.z),
            0,
            v3(int(1200), ONE, int(1200)),
        ),
        base,
        0,
    );
    // which way the camera looks across the water (none: straight down)
    let flat = cam.fwd.flat();
    let look = if flat.len() > ONE / 4 {
        Some(flat.norm())
    } else {
        None
    };
    let thick = if storm { cm(70) } else { cm(30) };
    let mut inner = 0;
    for (band, &(side_m, reach_m)) in BANDS.iter().enumerate() {
        // a squall's fog hides the outermost band
        if storm && inner >= int(170) {
            break;
        }
        let cell = int(side_m);
        let reach = int(reach_m);
        let half = cell / 2;
        let x0 = (eye.x - reach).div_euclid(cell);
        let x1 = (eye.x + reach).div_euclid(cell);
        let z0 = (eye.z - reach).div_euclid(cell);
        let z1 = (eye.z + reach).div_euclid(cell);
        // the bands further out leave out the swells their tiles cannot
        // show, and sit a hair lower where they overlap the band inside
        let min_len = if band == 0 { 0 } else { cell * 2 };
        let drop = cm(3) * band as i32;
        let long = longer_than(w, min_len);
        let chop = min_len <= int(8);
        for ix in x0..=x1 {
            for iz in z0..=z1 {
                let (x, z) = (ix * cell + half, iz * cell + half);
                let (dx, dz) = (x - eye.x, z - eye.z);
                if dx.abs() + half <= inner && dz.abs() + half <= inner {
                    continue;
                }
                if let Some(f) = look {
                    let d = v3(dx, 0, dz);
                    let ahead = d.dot(f);
                    if ahead < -(d.len() / 5) - cell * 2 {
                        continue;
                    }
                }
                let h = |x: i32, z: i32| surface_of(&long, x, z, tick, storm, chop);
                let y = h(x, z);
                let sx = fx::div(h(x + half, z) - h(x - half, z), cell);
                let sz = fx::div(h(x, z + half) - h(x, z - half), cell);
                // a slab a little larger than its cell, so neighbours
                // overlap, and deep enough that where two meet at different
                // heights the step shows water, not a seam
                let size = cell + cell / 12;
                let tile = Affine {
                    c0: v3(size, mul(sx, size), 0),
                    c1: v3(0, thick, 0),
                    c2: v3(0, mul(sz, size), size),
                    t: v3(x, y - drop - thick / 2, z),
                };
                let id = cell_id(band, ix, iz);
                let colour = water_colour(bay, x, z, y - w.level, peak, storm, id);
                b.push(Mesh::Cube, &tile, colour, 0);
                if band <= 1 {
                    sparkle(
                        b,
                        (x, y, z),
                        cell,
                        (sx, sz),
                        id,
                        y - w.level,
                        peak,
                        t,
                        storm,
                    );
                }
            }
        }
        inner = reach;
    }
}

/// Foam on a tile's crest and a twinkling sun glint, near the eye: small
/// flat flecks, fixed in the world, laid on the tile's slope.
#[allow(clippy::too_many_arguments)]
fn sparkle(
    b: &mut Batch,
    (x, y, z): (i32, i32, i32),
    cell: i32,
    (sx, sz): (i32, i32),
    id: u32,
    rise: i32,
    peak: i32,
    t: u32,
    storm: bool,
) {
    let on = |dx: i32, dz: i32, lift: i32| v3(x + dx, y + mul(sx, dx) + mul(sz, dz) + lift, z + dz);
    // foam on the crests (a squall's whitecaps: more of it, bigger)
    let crest = fx::div(rise, peak.max(cm(10)));
    let foam_from = if storm { ONE / 4 } else { ONE * 3 / 5 };
    if crest > foam_from && !hash(id, 51).is_multiple_of(3) {
        let n = if storm { 3 } else { 2 };
        for k in 0..n {
            let kid = id.wrapping_add(k * 977);
            let dx = mul(cell, ONE * pick(kid, 52, -40, 40) / 100);
            let dz = mul(cell, ONE * pick(kid, 53, -40, 40) / 100);
            let big = if storm { 140 } else { 70 };
            let s = mul(cm(pick(kid, 54, 30, big)), cell / 4);
            b.push(
                Mesh::Quad,
                &placed(on(dx, dz, cm(3)), pick(kid, 55, 0, TURN), v3(s, ONE, s / 2)),
                FOAM,
                ONE / 3,
            );
        }
    }
    // a sun glint, flickering on and off (not in a squall)
    if storm || !hash(id, 56).is_multiple_of(3) {
        return;
    }
    let flick = sin((t as i32)
        .wrapping_mul(420)
        .wrapping_add(pick(id, 57, 0, TURN)));
    if flick < ONE / 3 {
        return;
    }
    let dx = mul(cell, ONE * pick(id, 58, -40, 40) / 100);
    let dz = mul(cell, ONE * pick(id, 59, -40, 40) / 100);
    let s = mul(cm(pick(id, 60, 14, 30)), cell / 4);
    b.push(
        Mesh::Quad,
        &placed(on(dx, dz, cm(2)), deg(45), v3(s, ONE, s)),
        GLINT,
        SKY_GLOW,
    );
}

#[cfg(test)]
mod tests;
