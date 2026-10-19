//! Rattlesnake Ravine's scenery and light (`ravine.rs` is the contract it
//! draws from).
//!
//! **The sky and light (step 5, 2026-10-17, SECOND-ORDER RK rows):** a
//! clear desert night. A deep indigo dome, hundreds of stars, the Milky
//! Way arching across it (a pale haze thick with small stars), and a low
//! moon over the rim ahead of the start. The moon gives a cool silver
//! light; the sandstone bounces a warm red back (`look`'s `ground`), and a
//! faint cool edge light (`rim`) keeps the vines, snakes and apes apart
//! from the night walls. Each torch, lantern, mushroom patch and firefly
//! swarm of the layout's `glows` throws a soft pool of light round it
//! (`pools`), so the way to the finish is lit.
//!
//! **Enclosed parts have their own light** (MAPS gotcha 7): inside the
//! mine tunnel a warm lantern amber, inside a cave a cool mushroom teal;
//! both push the fog out and lift the bounce so the walls never go
//! near-black (`look_at`, blended over the mouths). **The mesa** opens up
//! round the finish: the fog pushed far out under the same stars.
//!
//! The dome is a multiple of the sky's colour and the stars, moon and
//! haze are sky glows, so the colourblind mode and JOJO MODE recolour them
//! (`bayscape::follow`, called by the frame), and the walls and ground are
//! static scenery the shader recolours (as Sunset Ruins, 1.10.1).
//!
//! The canyon itself (walls, roofs, decks, the movers) is `course.rs`.

use crate::course::Course;
use crate::fx::{self, cm, deg, int, ONE, TURN};
use crate::geom::{facing, v3, yaw_of, V3};
use crate::mesh::Mesh;
use crate::ravine::{GlowKind, RavineLayout, Room, RoomKind};
use crate::render::{mix, placed, rgb, Batch, Rgb};
use crate::scene::{Look, Scenery};
use crate::towerscape::{DOME_GLOW, SKY_GLOW};

/// The night sky low down (and the fog's base).
pub const SKY: Rgb = rgb(16, 22, 58);
/// Straight overhead: darker, nearly black-blue.
pub const ZENITH: Rgb = rgb(5, 8, 28);
/// The night haze down the canyon.
pub const FOG: Rgb = rgb(28, 30, 66);
/// Moonlight: cool silver, bright enough to read the canyon.
pub const MOON_LIGHT: Rgb = rgb(196, 204, 240);
/// The sandstone's bounce: warm red, so the walls read red-orange at night.
pub const BOUNCE: Rgb = rgb(120, 70, 50);
/// The cool edge light on every outline (vines, snakes, apes on the walls).
pub const RIM: Rgb = rgb(34, 44, 84);
/// The moon's disc.
pub const MOON: Rgb = rgb(246, 240, 214);

/// The vines: a pale moss green, apart from the red-orange walls by hue
/// and brightness under the moon (the colourblind mode keeps the Jungle's
/// vivid green, against walls taken toward grey).
pub const VINE: Rgb = rgb(170, 244, 150);
/// The vines' soft halo, in vine widths.
pub const VINE_HALO: i32 = 3;

/// The moon's height over the horizon: low, just over the rim seen from
/// the canyon floor ahead of the start.
pub const MOON_HIGH: i32 = deg(34);
/// The moon's bearing off the course's first heading (ahead, to the right).
pub const MOON_OFF: i32 = deg(24);

/// The sky dome's radius (inside the far plane, as the tower map's).
const DOME: i32 = int(560);
/// Stars scattered over the whole sky (320 before the sky was cut to 600
/// shapes, RA-15).
pub const FIELD_STARS: u32 = 210;
/// Small stars thick along the Milky Way, a third in clumps (580 before
/// RA-15: the haze now carries the band's density).
pub const BAND_STARS: u32 = 300;
/// One haze streak's strength over a sky glow's 3 (unfogged) at the
/// galaxy's centre (fading to a third at the band's ends). Streaks overlap
/// five deep and add, so the band reads as a faint blue-white river
/// (brighter round blobs read as purple clouds, RL3, RA-15).
pub const HAZE_CORE: i32 = ONE / 28;
/// The haze's passes (each streak overlaps its neighbours this deep).
const HAZE_PASSES: i32 = 5;
/// How far round the eye a glow's pool of light is drawn.
pub const POOL_SEEN: i32 = int(110);

/// The light outside before a course is known: moonlight from a fixed
/// bearing.
pub fn look() -> Look {
    look_toward(0)
}

/// The night outside with the moon at bearing `yaw`.
pub fn look_toward(yaw: i32) -> Look {
    Look {
        sky: SKY,
        fog: FOG,
        // far enough out that the next bend's walls show in the dark
        fog_range: (int(70), int(260)),
        light_dir: facing(yaw, MOON_HIGH),
        light: MOON_LIGHT,
        ground: BOUNCE,
        rim: RIM,
        far: crate::render::FAR,
    }
}

/// The moon's bearing on a course: off its first heading.
pub fn moon_yaw(c: &Course) -> i32 {
    yaw_of(c.heading[0]) + MOON_OFF
}

/// The light outside on this course.
pub fn look_for(c: &Course) -> Look {
    look_toward(moon_yaw(c))
}

/// The mine tunnel's light: warm lantern amber, the fog far out.
pub fn tunnel_look(yaw: i32) -> Look {
    Look {
        sky: rgb(40, 24, 12),
        fog: rgb(70, 42, 22),
        fog_range: (int(70), int(260)),
        light_dir: facing(yaw, deg(60)),
        light: rgb(236, 170, 100),
        ground: rgb(170, 110, 66),
        rim: rgb(60, 40, 20),
        far: crate::render::FAR,
    }
}

/// A cave's light: the glowing mushrooms' cool teal, the fog far out.
pub fn cave_look(yaw: i32) -> Look {
    Look {
        sky: rgb(14, 34, 40),
        fog: rgb(26, 54, 60),
        fog_range: (int(70), int(260)),
        light_dir: facing(yaw, deg(55)),
        light: rgb(170, 220, 210),
        ground: rgb(150, 100, 76),
        rim: rgb(30, 70, 70),
        far: crate::render::FAR,
    }
}

/// The open mesa round the finish: the moonlit night with the fog pushed
/// far out over the valley.
pub fn mesa_look(yaw: i32) -> Look {
    Look {
        fog_range: (int(140), int(460)),
        ..look_toward(yaw)
    }
}

/// How far inside a room `p` is, 0..=ONE: 0 outside, rising to ONE over
/// the first `MOUTH` metres past either mouth (so the light changes as
/// the eye goes in, not at a line).
pub fn inside_share(room: &Room, p: V3) -> i32 {
    const MOUTH: i32 = int(8);
    let run = room.to.sub(room.from).flat();
    let len = run.len();
    if len <= 0 {
        return 0;
    }
    let along = run.norm();
    let d = p.sub(room.from).flat();
    let a = d.dot(along);
    let side = d.dot(v3(-along.z, 0, along.x)).abs();
    let floor = room.from.y + fx::mul(room.to.y - room.from.y, fx::div(a.clamp(0, len), len));
    let margin = int(1);
    if a < 0
        || a > len
        || side > room.half_width + margin
        || p.y < floor - margin
        || p.y > floor + room.height + margin
    {
        return 0;
    }
    // a short cave is lit through its middle
    fx::div(a.min(len - a), MOUTH.min(len / 2).max(1)).clamp(0, ONE)
}

/// How open the sky round `p` is toward the mesa, 0..=ONE: full within
/// 60 m of the finish's sphinx, none past 150 m.
pub fn mesa_share(r: &RavineLayout, p: V3) -> i32 {
    let d = p.sub(r.finish.sphinx).flat().len();
    ONE - fx::div((d - int(60)).max(0), int(90)).min(ONE)
}

/// The light for an eye at `eye`: the moonlit canyon, opening out on the
/// mesa, warm in the mine tunnel and teal in the caves (blended over the
/// mouths).
pub fn look_at(c: &Course, r: &RavineLayout, eye: V3) -> Look {
    let yaw = moon_yaw(c);
    let mut l = blend(&look_toward(yaw), &mesa_look(yaw), mesa_share(r, eye));
    for room in &r.rooms {
        let inside = match room.kind {
            RoomKind::Tunnel => Some(tunnel_look(yaw)),
            RoomKind::Cave => Some(cave_look(yaw)),
            RoomKind::Arch | RoomKind::Skull => None,
        };
        if let Some(inside) = inside {
            let k = inside_share(room, eye);
            if k > 0 {
                l = blend(&l, &inside, k);
            }
        }
    }
    // the skull's torches light the bone and the teeth from the approach
    if let Some(s) = &r.skull {
        let k = skull_share(s, eye);
        if k > 0 {
            l = blend(&l, &skull_look(s), k);
        }
    }
    l
}

/// The torchlight round the T-rex skull (`course::skull_lights`): warm,
/// from the in-leg the apes come in from (so the snout, the teeth and the
/// gap face the light), the fog pushed out.
pub fn skull_look(s: &crate::ravine::Skull) -> Look {
    Look {
        sky: rgb(30, 20, 34),
        fog: rgb(60, 36, 34),
        fog_range: (int(90), int(300)),
        light_dir: facing(s.yaw + TURN / 2, deg(30)),
        light: rgb(250, 186, 120),
        ground: rgb(176, 104, 66),
        rim: rgb(70, 44, 30),
        far: crate::render::FAR,
    }
}

/// How strongly the skull's torchlight holds at `p`: `SKULL_LIT` within
/// 30 m of the mouth, none past 80 m (the approach down the in-leg).
pub fn skull_share(s: &crate::ravine::Skull, p: V3) -> i32 {
    let d = p.sub(s.at).flat().len();
    let t = ONE - fx::div((d - int(30)).max(0), int(50)).min(ONE);
    fx::mul(t, SKULL_LIT)
}

/// The skull's torchlight at most: most of the way, the night still in it.
pub const SKULL_LIT: i32 = ONE * 3 / 4;

fn blend(a: &Look, b: &Look, t: i32) -> Look {
    if t <= 0 {
        return *a;
    }
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

/// A well-mixed number from an index and a salt.
fn hash(k: u32, salt: u32) -> u32 {
    let mut x = k.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B) ^ 0x51C3_A7E5;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^ (x >> 15)
}

/// A value in `lo..hi` from `hash(k, salt)`.
fn pick(k: u32, salt: u32, lo: i32, hi: i32) -> i32 {
    lo + ((hash(k, salt) & 0xFFFF) as i64 * (hi - lo) as i64 / 0x1_0000) as i32
}

/// The Milky Way's frame: `u` along the horizon where it rises (to the
/// left of the course's first heading), `v` its highest point (ahead, 38°
/// up, so it arches across the view down the canyon), `p` across it.
fn band_frame(moon: i32) -> (V3, V3, V3) {
    let yaw = moon - MOON_OFF - TURN / 4;
    let tilt = deg(38);
    (
        facing(yaw, 0),
        facing(yaw + TURN / 4, tilt),
        facing(yaw + TURN / 4, tilt + TURN / 4),
    )
}

/// The direction of the Milky Way's point at `t` along it (0..TURN/2, from
/// horizon to horizon) and `w` across it (a share of a radian, small).
fn band_dir(f: (V3, V3, V3), t: i32, w: i32) -> V3 {
    let (u, v, p) = f;
    u.scale(fx::cos(t))
        .add(v.scale(fx::sin(t)))
        .add(p.scale(w))
        .norm()
}

/// An angle in degrees as a share of a radian (the band's `w` across).
const fn rad(d: i32) -> i32 {
    d * ONE / 57
}

/// The dust lane (the Great Rift): along this stretch of the band its
/// bright core splits in two either side of a dark gap.
fn in_rift(t: i32) -> bool {
    (deg(40)..deg(118)).contains(&t)
}

/// The star clouds along the band: where (degrees along, degrees across)
/// a third of its small stars clump.
const CLUMPS: [(i32, i32); 9] = [
    (18, 1),
    (34, -2),
    (52, 4),
    (66, -4),
    (78, 0),
    (96, 4),
    (124, -1),
    (142, 2),
    (162, 0),
];

/// One streak of the Milky Way's haze at `t` along the band and `w`
/// across: an ellipsoid `len` long along the band and `wide` across
/// (degrees), `deep` toward the eye. The page draws glows with depth
/// writes, so where streaks overlap the nearer face wins: all at one reach,
/// the brighter middle of each is the nearer, and the band composes as the
/// brightest streak at each point (no seams). Long (deep under half its
/// length) the band stays even along; deep for its width, it fades softly
/// to nothing across (round overlapping blobs read as lumpy purple puffs
/// with outlines, RA-15).
#[allow(clippy::too_many_arguments)]
fn streak(
    b: &mut Batch,
    eye: V3,
    f: (V3, V3, V3),
    reach: i32,
    t: i32,
    w: i32,
    len: i32,
    wide: i32,
    deep: i32,
    c: Rgb,
    glow: i32,
) {
    let dir = band_dir(f, t, w);
    if dir.y < -ONE / 6 {
        return;
    }
    let (u, v, _) = f;
    let along = u.scale(-fx::sin(t)).add(v.scale(fx::cos(t))).norm();
    let across = along.cross(dir).norm();
    let size = |d: i32| fx::mul(reach, d * 1144);
    let a = crate::geom::Affine {
        c0: along.scale(size(len)),
        // (the sphere's poles toward the eye: its rings lie round the view)
        c1: dir.scale(size(deep)),
        c2: across.scale(size(wide)),
        t: eye.add(dir.scale(reach)),
    };
    b.push_halo(Mesh::Sphere, &a, c, glow);
}

/// The Milky Way's haze: long soft streaks every few degrees along the
/// band, brightest (and a little warm) round the galaxy's centre, split by
/// the dust lane into two streams with a dark gap between for part of its
/// length.
fn haze(b: &mut Batch, eye: V3, f: (V3, V3, V3)) {
    let white = rgb(176, 190, 232);
    let warm = rgb(208, 204, 222);
    // how bright the band is at `t`: brightest round the centre (70°),
    // with a slow swell and fade along it
    let bright = |t: i32| {
        let centre = ONE - ((t - deg(70)).abs() * 2 / 3 * ONE / deg(90)).min(ONE * 2 / 3);
        let swell = ONE * 5 / 6 + fx::sin(t * 4 + deg(30)) / 6;
        fx::mul(centre, swell)
    };
    // drawn in five passes, each nearer the eye than the last, so a
    // streak is drawn over the ones it overlaps and their light adds
    for pass in 0..HAZE_PASSES {
        let reach = int(460) - pass * int(24);
        let mut t = -deg(6) + pass * deg(5);
        let mut i = pass as u32;
        while t < deg(186) {
            let g = fx::mul(HAZE_CORE, bright(t));
            let c = if (t - deg(70)).abs() < deg(24) {
                warm
            } else {
                white
            };
            let wob = pick(i, 52, -rad(1), rad(1));
            if in_rift(t) {
                // the two streams either side of the lane, one fainter
                streak(
                    b,
                    eye,
                    f,
                    reach,
                    t,
                    -rad(4) + wob,
                    28,
                    6,
                    12,
                    c,
                    ONE * 3 + g,
                );
                streak(
                    b,
                    eye,
                    f,
                    reach,
                    t,
                    rad(4) + wob,
                    28,
                    5,
                    10,
                    c,
                    ONE * 3 + g * 3 / 4,
                );
            } else {
                streak(b, eye, f, reach, t, wob, 30, 10, 18, c, ONE * 3 + g);
            }
            t += deg(5) * HAZE_PASSES;
            i += HAZE_PASSES as u32;
        }
    }
}

/// The night sky round `eye`, drawn each frame: the dome, the Milky Way's
/// haze and stars, the field stars twinkling, and the moon; and the glows'
/// pools of light near the eye.
pub fn dynamic(b: &mut Batch, c: &Course, r: &RavineLayout, eye: V3, ticks: u32) {
    let deep = [0, 1, 2].map(|k| fx::div(ZENITH[k], SKY[k].max(1)).min(ONE));
    b.push(
        Mesh::Sphere,
        &placed(eye, 0, v3(DOME * 2, DOME * 2, DOME * 2)),
        deep,
        DOME_GLOW,
    );
    sky(b, eye, moon_yaw(c), ticks);
    pools(b, r, eye, ticks);
}

/// The stars, the Milky Way and the moon round `eye`.
pub fn sky(b: &mut Batch, eye: V3, moon: i32, ticks: u32) {
    let f = band_frame(moon);
    // the haze: long soft streaks laid end to end along the band, a broad
    // faint one and a narrow brighter core split by the dust lane (RA-15)
    haze(b, eye, f);
    // the band's small stars, thick near its middle line, a third of them
    // in clumps (the star clouds) and few in the dust lane
    for i in 0..BAND_STARS {
        let (t, w) = if i % 3 == 0 {
            let k = (i / 3) % CLUMPS.len() as u32;
            let (ct, cw) = CLUMPS[k as usize];
            (
                deg(ct) + (pick(i, 45, -deg(5), deg(5)) + pick(i, 46, -deg(5), deg(5))) / 2,
                rad(cw) + (pick(i, 47, -rad(5), rad(5)) + pick(i, 48, -rad(5), rad(5))) / 4,
            )
        } else {
            let t = pick(i, 41, 0, TURN / 2);
            let mut w = (pick(i, 42, -ONE / 8, ONE / 8) + pick(i, 43, -ONE / 8, ONE / 8)) / 2;
            // most stars that fall in the lane are pushed out to its banks
            if in_rift(t) && w.abs() < rad(2) && i % 4 != 1 {
                w = if w < 0 { w - rad(2) } else { w + rad(2) };
            }
            (t, w)
        };
        let dir = band_dir(f, t, w);
        if dir.y < ONE / 40 {
            continue;
        }
        let at = eye.add(dir.scale(DOME - int(40)));
        let rad = cm(pick(i, 44, 90, 190));
        let c = if i % 4 == 0 {
            rgb(255, 236, 214)
        } else {
            rgb(214, 222, 255)
        };
        b.push(Mesh::Ico, &placed(at, 0, v3(rad, rad, rad)), c, SKY_GLOW);
    }
    // the field stars over the whole sky, a few bright, each twinkling on
    // its own beat
    for i in 0..FIELD_STARS {
        let el = deg(pick(i, 11, 3, 89));
        let dir = facing(pick(i, 12, 0, TURN), el);
        let at = eye.add(dir.scale(DOME - int(40)));
        let rad = if i % 23 == 0 {
            cm(pick(i, 13, 320, 420))
        } else {
            cm(pick(i, 13, 140, 260))
        };
        let tw = (ticks / 8).wrapping_add(hash(i, 14)) % 9;
        let c = if tw == 0 {
            rgb(150, 156, 200)
        } else if i % 6 == 0 {
            rgb(255, 222, 180)
        } else if i % 7 == 0 {
            rgb(196, 214, 255)
        } else {
            rgb(244, 244, 255)
        };
        b.push(Mesh::Ico, &placed(at, 0, v3(rad, rad, rad)), c, SKY_GLOW);
    }
    moon_disc(b, eye, moon);
}

/// The moon: a pale disc with darker seas, in a wide soft corona.
fn moon_disc(b: &mut Batch, eye: V3, moon: i32) {
    let dir = facing(moon, MOON_HIGH);
    let at = eye.add(dir.scale(DOME - int(80)));
    let rad = int(17);
    b.push(
        Mesh::Sphere,
        &placed(at, 0, v3(rad, rad, rad)),
        MOON,
        SKY_GLOW,
    );
    let side = facing(moon + TURN / 4, 0);
    let up = facing(moon, MOON_HIGH + TURN / 4);
    for (k, (dx, dy, s)) in [(-4, 3, 4), (3, -2, 4), (1, 5, 3)].into_iter().enumerate() {
        let spot = at
            .sub(dir.scale(rad - int(3)))
            .add(side.scale(int(dx)))
            .add(up.scale(int(dy)));
        let sz = int(s);
        b.push(
            Mesh::Sphere,
            &placed(spot, k as i32 * deg(40), v3(sz, sz, sz)),
            rgb(206, 202, 186),
            SKY_GLOW,
        );
    }
    b.push_halo(
        Mesh::Sphere,
        &placed(at, 0, v3(rad * 5, rad * 5, rad * 5)),
        rgb(170, 186, 240),
        ONE * 3,
    );
}

/// The colour of a glow's pool of light.
pub fn pool_colour(kind: GlowKind) -> Rgb {
    match kind {
        GlowKind::Torch => rgb(255, 150, 60),
        GlowKind::Lantern => rgb(255, 196, 110),
        GlowKind::Mushroom => rgb(80, 255, 220),
        GlowKind::Fireflies => rgb(200, 255, 110),
    }
}

/// Each glow near the eye throws a soft pool of light round it: a wide
/// halo, breathing gently (the flames flicker faster than the mushrooms).
pub fn pools(b: &mut Batch, r: &RavineLayout, eye: V3, ticks: u32) {
    for (i, g) in r.glows.iter().enumerate() {
        if g.at.sub(eye).longer_than(POOL_SEEN) {
            continue;
        }
        let (size, rate) = match g.kind {
            GlowKind::Torch | GlowKind::Lantern => (int(7), 260),
            GlowKind::Mushroom => (int(6), 60),
            GlowKind::Fireflies => (int(5), 110),
        };
        let beat = fx::sin((ticks as i32).wrapping_mul(rate) + pick(i as u32, 51, 0, TURN));
        let s = size + fx::mul(beat, cm(40));
        b.push_halo(
            Mesh::Sphere,
            &placed(g.at.add(v3(0, cm(150), 0)), 0, v3(s, s, s)),
            pool_colour(g.kind),
            ONE / 4,
        );
    }
}

/// Rattlesnake Ravine's scenery and ground, chunked: the canyon drawn from
/// its layout (`course.rs`).
pub fn scenery(c: &Course, r: &RavineLayout) -> Scenery {
    course::scenery(c, r)
}

pub mod course;
pub mod lobby;

#[cfg(test)]
mod tests;
