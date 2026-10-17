//! Mape's Tower's scenery and light (`tower.rs` is the contract it draws
//! from).
//!
//! **The time of day (step 5, 2026-10-15, SECOND-ORDER TS rows):** three
//! weathers, each its own sky, all changing with the height climbed
//! (`height_share`: the keep's base 0, the tear 1; TA8):
//! - **Clear** (the default): a golden late afternoon, the sun low in the
//!   west (−x), a periwinkle sky over a honey haze; bluer higher up.
//! - **Twilight**: purple-indigo dusk at the base, deepening to night with
//!   stars near the top (more stars the higher you are); a cool lavender
//!   light and a violet edge light so the tower reads against the dark.
//! - **Dawn**: a pink-gold sunrise, the sun's disc low in the east (+x),
//!   its warm light on the tower's east face, pale pink clouds; brighter
//!   the higher you climb.
//!
//! The **zipper space** (past the mirror plane, `TowerLayout::in_zipper`)
//! has its own eerie sea-green void under a violet dome with a violet edge
//! light, whatever the weather. Inside the **cloud band** the fog closes
//! in to the cloud's colour; outside it a sea of cloud is drawn at the
//! band's height round the eye (`dynamic`), cleared round the keep.
//! The sky is `Look.sky`/`fog`, the dome a multiple of the sky and the
//! stars, sun and streaks sky glows, so the colourblind mode and JOJO MODE
//! recolour them (`bayscape::follow`, called by the frame).

use crate::course::Course;
use crate::fx::{self, cm, deg, int, mul, ONE, TURN};
use crate::geom::{facing, v3, V3};
use crate::mesh::Mesh;
use crate::render::{mix, placed, rgb, Batch, Rgb};
use crate::scene::Look;
use crate::snow::Weather;
use crate::tower::TowerLayout;

mod art;
pub mod lobby;
pub use art::{
    beam_vine, cast_by, casters, parts, portal_frame, portal_pose, scenery, spell_of, worst_drawn,
    worst_drawn_at, PORTAL_SEEN, REACH,
};

/// Shown full colour and unfogged, as the sky is (`scene.frag`).
pub const SKY_GLOW: i32 = ONE * 2;
/// The sky's dome's mark for `scene.frag` (over 3.5).
pub const DOME_GLOW: i32 = ONE * 4;
/// The dome's radius: inside the far plane.
const DOME: i32 = int(560);

/// One sky: what the look takes, the dome overhead and the clouds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sky {
    pub sky: Rgb,
    pub fog: Rgb,
    /// the dome straight overhead
    pub zenith: Rgb,
    pub light: Rgb,
    pub ground: Rgb,
    pub rim: Rgb,
    /// the clouds' colour (the cloud sea, the fog inside the band)
    pub cloud: Rgb,
    pub fog_range: (i32, i32),
}

impl Sky {
    fn mixed(a: Sky, b: Sky, t: i32) -> Sky {
        Sky {
            sky: mix(a.sky, b.sky, t),
            fog: mix(a.fog, b.fog, t),
            zenith: mix(a.zenith, b.zenith, t),
            light: mix(a.light, b.light, t),
            ground: mix(a.ground, b.ground, t),
            rim: mix(a.rim, b.rim, t),
            cloud: mix(a.cloud, b.cloud, t),
            fog_range: (
                fx::lerp(a.fog_range.0, b.fog_range.0, t),
                fx::lerp(a.fog_range.1, b.fog_range.1, t),
            ),
        }
    }
}

/// Clear, at the base and at the summit.
const CLEAR: [Sky; 2] = [
    Sky {
        sky: rgb(150, 176, 230),
        fog: rgb(246, 210, 156),
        zenith: rgb(84, 116, 206),
        light: rgb(255, 220, 160),
        ground: rgb(120, 104, 70),
        rim: [0; 3],
        cloud: rgb(255, 238, 214),
        fog_range: (int(120), int(330)),
    },
    Sky {
        sky: rgb(136, 172, 236),
        fog: rgb(236, 220, 190),
        zenith: rgb(52, 92, 200),
        light: rgb(255, 228, 176),
        ground: rgb(150, 136, 110),
        rim: [0; 3],
        cloud: rgb(255, 246, 230),
        fog_range: (int(150), int(380)),
    },
];

/// Twilight, at the base (dusk) and at the summit (night and stars).
const TWILIGHT: [Sky; 2] = [
    Sky {
        sky: rgb(126, 84, 168),
        fog: rgb(170, 106, 168),
        zenith: rgb(54, 40, 124),
        light: rgb(190, 168, 236),
        ground: rgb(64, 50, 96),
        rim: rgb(40, 24, 70),
        cloud: rgb(176, 136, 200),
        fog_range: (int(90), int(300)),
    },
    Sky {
        sky: rgb(44, 34, 96),
        fog: rgb(70, 52, 120),
        zenith: rgb(8, 8, 36),
        light: rgb(150, 150, 220),
        ground: rgb(36, 30, 70),
        rim: rgb(54, 40, 110),
        cloud: rgb(96, 84, 150),
        fog_range: (int(110), int(340)),
    },
];

/// Dawn, at the base and at the summit (brighter).
const DAWN: [Sky; 2] = [
    Sky {
        sky: rgb(236, 158, 170),
        fog: rgb(252, 196, 150),
        zenith: rgb(190, 120, 176),
        light: rgb(255, 186, 126),
        ground: rgb(120, 90, 96),
        rim: [0; 3],
        cloud: rgb(255, 214, 214),
        fog_range: (int(120), int(320)),
    },
    Sky {
        sky: rgb(252, 204, 186),
        fog: rgb(255, 226, 196),
        zenith: rgb(214, 170, 196),
        light: rgb(255, 210, 160),
        ground: rgb(150, 120, 120),
        rim: [0; 3],
        cloud: rgb(255, 236, 228),
        fog_range: (int(150), int(370)),
    },
];

/// The zipper's void: sea-green below a violet dome, a violet edge light.
pub const ZIPPER: Sky = Sky {
    sky: rgb(38, 120, 104),
    fog: rgb(52, 140, 120),
    zenith: rgb(74, 26, 120),
    light: rgb(170, 255, 214),
    ground: rgb(70, 30, 110),
    rim: rgb(110, 50, 170),
    cloud: rgb(80, 160, 140),
    fog_range: (int(70), int(260)),
};

/// The sun's (or the dusk's) bearing per weather: Clear's low in the west
/// (−x), Dawn's on the east horizon (+x), Twilight's the last of the day
/// in the west.
fn sun_yaw(w: Weather) -> i32 {
    match w {
        Weather::Dawn => deg(90),
        Weather::Twilight => deg(-120),
        _ => deg(-100),
    }
}

/// Toward the light: Dawn's a little over its disc so tops are lit too.
fn light_dir(w: Weather, zipper: bool) -> V3 {
    if zipper {
        return v3(cm(-30), cm(80), cm(50)).norm();
    }
    let up = match w {
        Weather::Dawn => deg(16),
        Weather::Twilight => deg(40),
        _ => deg(26),
    };
    facing(sun_yaw(w), up)
}

/// How far up the climb a height is: 0 at the keep's base, `ONE` from
/// the tear's middle (or the summit deck, if lower) up: the sky reaches
/// its top colours where the apes fly through the tear (TA8).
pub fn height_share(t: &TowerLayout, y: i32) -> i32 {
    let top = t.portal.tear.y - t.towers[0].centre.y;
    if top <= 0 {
        return 0;
    }
    fx::div(y - t.towers[0].centre.y, top).clamp(0, ONE)
}

/// The sky for weather `w` a share `h` of the way up (out of the zipper).
pub fn sky_of(w: Weather, h: i32) -> Sky {
    let pair = match w {
        Weather::Twilight => &TWILIGHT,
        Weather::Dawn => &DAWN,
        _ => &CLEAR,
    };
    Sky::mixed(pair[0], pair[1], h)
}

/// The sky round an eye: the zipper's whatever the weather, else the
/// weather's at the eye's height.
pub fn sky_at(t: &TowerLayout, eye: V3, w: Weather) -> Sky {
    if t.in_void(eye) {
        ZIPPER
    } else {
        sky_of(w, height_share(t, eye.y))
    }
}

/// How deep into a cloud band a height is: 0 outside, `ONE` from 3 m in.
pub fn in_cloud(t: &TowerLayout, y: i32) -> i32 {
    let edge = int(3);
    t.clouds
        .iter()
        .map(|c| {
            let d = (y - c.bottom).min(c.top - y);
            if d <= -edge {
                0
            } else {
                fx::div(d + edge, edge * 2).clamp(0, ONE)
            }
        })
        .max()
        .unwrap_or(0)
}

fn look_from(s: Sky, w: Weather, zipper: bool) -> Look {
    Look {
        sky: s.sky,
        fog: s.fog,
        fog_range: s.fog_range,
        light_dir: light_dir(w, zipper),
        light: s.light,
        ground: s.ground,
        rim: s.rim,
        far: crate::render::FAR,
    }
}

/// Mape's Tower's light before a course is built: Clear, at the base.
pub fn look() -> Look {
    look_from(sky_of(Weather::Clear, 0), Weather::Clear, false)
}

/// The look for an eye at `eye` in weather `w`: the weather's sky at the
/// eye's height (or the zipper's), its fog closed in to the cloud's colour
/// inside the cloud band.
pub fn look_in(t: &TowerLayout, eye: V3, w: Weather) -> Look {
    let zipper = t.in_void(eye);
    let s = sky_at(t, eye, w);
    let mut l = look_from(s, w, zipper);
    let k = if zipper { 0 } else { in_cloud(t, eye.y) };
    if k > 0 {
        l.fog = mix(l.fog, s.cloud, k);
        l.fog_range = (
            fx::lerp(l.fog_range.0, int(8), k),
            fx::lerp(l.fog_range.1, int(110), k),
        );
    }
    l
}

/// Mape's Tower at twilight, at the base (`snow::sky_in`, where no eye is
/// known; the frame uses `look_in`).
pub fn twilight(look: Look) -> Look {
    Look {
        far: look.far,
        ..look_from(sky_of(Weather::Twilight, 0), Weather::Twilight, false)
    }
}

/// Mape's Tower at dawn, at the base (as `twilight`).
pub fn dawn(look: Look) -> Look {
    Look {
        far: look.far,
        ..look_from(sky_of(Weather::Dawn, 0), Weather::Dawn, false)
    }
}

/// A well-mixed number from an index and a salt.
fn hash(k: u32, salt: u32) -> u32 {
    let mut x = k.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B) ^ 0x7A3E_5C11;
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

/// The sky round `eye` in weather `w`, drawn each frame: the dome (its
/// colour a multiple of the sky's), then the zipper's motes, Twilight's
/// stars, or Dawn's or Clear's sun and streaks, and the cloud sea; and
/// the portal, turning and breathing, from within `PORTAL_SEEN` of it.
pub fn dynamic(b: &mut Batch, t: &TowerLayout, eye: V3, ticks: u32, w: Weather) {
    portal_frame(b, t, eye, ticks);
    let zipper = t.in_void(eye);
    let s = sky_at(t, eye, w);
    let deep = [0, 1, 2].map(|k| fx::div(s.zenith[k], s.sky[k].max(1)).min(ONE));
    b.push(
        Mesh::Sphere,
        &placed(eye, 0, v3(DOME * 2, DOME * 2, DOME * 2)),
        deep,
        DOME_GLOW,
    );
    if zipper {
        // violet and green motes all round the void, above and below
        for i in 0..160u32 {
            let dir = facing(pick(i, 1, 0, TURN), deg(pick(i, 2, -70, 80)));
            let at = eye.add(dir.scale(DOME - int(60)));
            let c = if i % 3 == 0 {
                rgb(140, 255, 200)
            } else {
                rgb(200, 120, 255)
            };
            let r = int(pick(i, 3, 2, 5));
            b.push(Mesh::Sphere, &placed(at, 0, v3(r, r, r)), c, SKY_GLOW);
        }
        return;
    }
    let h = height_share(t, eye.y);
    let yaw = sun_yaw(w);
    match w {
        Weather::Twilight => stars(b, eye, h, ticks),
        Weather::Dawn => {
            sun(b, eye, facing(yaw, deg(3)), rgb(255, 214, 150), int(34));
            streaks(b, eye, yaw, s.cloud);
        }
        _ => {
            sun(b, eye, facing(yaw, deg(14)), rgb(255, 236, 190), int(26));
            streaks(b, eye, yaw, s.cloud);
        }
    }
    cloud_sea(b, t, eye, s.cloud);
}

/// How many stars Twilight shows a share `h` of the way up: a few in the
/// dusk at the base, hundreds near the summit.
pub fn star_count(h: i32) -> u32 {
    (40 + mul(int(360), h) / ONE) as u32
}

/// Twilight's stars, each twinkling on its own.
fn stars(b: &mut Batch, eye: V3, h: i32, ticks: u32) {
    for i in 0..star_count(h) {
        let el = deg(pick(i, 11, 14, 88));
        let at = eye.add(facing(pick(i, 12, 0, TURN), el).scale(DOME - int(40)));
        let r = cm(pick(i, 13, 110, 240));
        let tw = (ticks / 9).wrapping_add(hash(i, 14)) % 7;
        let c = if tw == 0 {
            rgb(180, 180, 220)
        } else if i % 5 == 0 {
            rgb(255, 230, 190)
        } else {
            rgb(240, 240, 255)
        };
        b.push(Mesh::Sphere, &placed(at, 0, v3(r, r, r)), c, SKY_GLOW);
    }
}

/// A low sun's disc in a soft corona, toward `dir`.
fn sun(b: &mut Batch, eye: V3, dir: V3, c: Rgb, r: i32) {
    let at = eye.add(dir.scale(DOME - int(80)));
    b.push(Mesh::Sphere, &placed(at, 0, v3(r, r, r)), c, SKY_GLOW);
    b.push_halo(
        Mesh::Sphere,
        &placed(at, 0, v3(r * 5, r * 5, r * 5)),
        c,
        ONE * 3 + ONE / 2,
    );
}

/// Long pale streaks of cloud low round the horizon, most toward the sun.
fn streaks(b: &mut Batch, eye: V3, sun_yaw: i32, c: Rgb) {
    for i in 0..14u32 {
        let away = if i % 3 == 0 { TURN / 2 } else { 0 };
        let a = sun_yaw + deg(pick(i, 21, -80, 80)) + away;
        let el = deg(pick(i, 22, 2, 12));
        let at = eye.add(facing(a, el).scale(DOME - int(70)));
        let w = int(pick(i, 23, 50, 110));
        b.push(Mesh::Geo, &placed(at, a, v3(w, w / 12, w / 5)), c, SKY_GLOW);
    }
}

/// The cloud band seen from above or below: flat soft puffs on a world
/// grid at the band's height out to 320 m round the eye, none within 50 m
/// of the keep's wall (the climb stays in view) or past the mirror plane.
fn cloud_sea(b: &mut Batch, t: &TowerLayout, eye: V3, c: Rgb) {
    let cell = int(32);
    let reach = int(320);
    let clear = t.towers[1].radius + int(50);
    let keep = t.towers[1].centre.flat();
    let (x0, x1) = (
        (eye.x - reach).div_euclid(cell),
        (eye.x + reach).div_euclid(cell),
    );
    let (z0, z1) = (
        (eye.z - reach).div_euclid(cell),
        (eye.z + reach).div_euclid(cell),
    );
    for band in &t.clouds {
        for cx in x0..=x1 {
            for cz in z0..=z1 {
                let k = (cx as u32).wrapping_mul(7919) ^ (cz as u32).wrapping_mul(104_729);
                if hash(k, 31) % 10 < 3 {
                    continue;
                }
                let p = v3(
                    cx * cell + pick(k, 32, 0, cell),
                    pick(k, 33, band.bottom, band.top),
                    cz * cell + pick(k, 34, 0, cell),
                );
                if t.in_void(p)
                    || !p.flat().sub(keep).longer_than(clear)
                    || p.flat().sub(eye.flat()).longer_than(reach)
                {
                    continue;
                }
                let w = int(pick(k, 35, 18, 34));
                let th = int(pick(k, 36, 3, 6));
                b.push(
                    Mesh::Geo,
                    &placed(p, pick(k, 37, 0, TURN), v3(w, th, w * 3 / 4)),
                    c,
                    0,
                );
            }
        }
    }
}

/// The title's flyover on Mape's Tower: up round the keep to the tear
/// (`lobby::flyover_camera`).
pub fn flyover_camera(c: &Course, ticks: u32, from: i32) -> crate::render::Camera {
    lobby::flyover_camera(c, ticks, from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Map;

    fn luma(c: Rgb) -> i32 {
        (c[0] * 30 + c[1] * 59 + c[2] * 11) / 100
    }

    /// An eye a share `h` of the way up the keep, outside it on the
    /// climb's side of the mirror plane.
    fn eye_at(t: &TowerLayout, h: i32) -> V3 {
        let k = t.towers[0].centre;
        v3(
            k.x - t.towers[0].radius - int(20),
            k.y + mul(t.portal.tear.y - k.y, h),
            k.z,
        )
    }

    #[test]
    fn each_weather_has_its_own_sky_and_it_changes_with_the_climb() {
        let c = Map::Tower.course();
        let t = c.tower.as_ref().unwrap();
        // away from the cloud band, low and high
        let (low, high) = (eye_at(t, 0), eye_at(t, ONE));
        assert_eq!(in_cloud(t, low.y), 0);
        assert_eq!(in_cloud(t, high.y), 0);
        let ws = [Weather::Clear, Weather::Twilight, Weather::Dawn];
        let looks: Vec<Look> = ws.iter().map(|&w| look_in(t, low, w)).collect();
        for i in 0..3 {
            for j in i + 1..3 {
                assert_ne!(looks[i].sky, looks[j].sky, "{:?} vs {:?}", ws[i], ws[j]);
                assert_ne!(looks[i].light_dir, looks[j].light_dir);
            }
        }
        // twilight darkens to night with stars higher up; dawn brightens
        let tw = (
            look_in(t, low, Weather::Twilight),
            look_in(t, high, Weather::Twilight),
        );
        assert!(
            luma(tw.1.sky) < luma(tw.0.sky) / 2,
            "twilight darkens up the tower"
        );
        assert!(star_count(height_share(t, high.y)) > 3 * star_count(height_share(t, low.y)));
        // (the top colours where the apes fly through the tear, TA8)
        assert_eq!(height_share(t, t.portal.tear.y), ONE);
        assert!(height_share(t, t.portal.tear.y - int(10)) < ONE);
        let dawn = (
            look_in(t, low, Weather::Dawn),
            look_in(t, high, Weather::Dawn),
        );
        assert!(
            luma(dawn.1.sky) > luma(dawn.0.sky),
            "dawn brightens up the tower"
        );
        // dawn's sun is low in the east (+x)
        assert!(dawn.0.light_dir.x > ONE * 9 / 10);
        // sky_in (no eye) gives the base's sky
        assert_eq!(
            crate::snow::sky_in(Weather::Twilight, look()).sky,
            TWILIGHT[0].sky
        );
    }

    #[test]
    fn the_zipper_has_its_own_void_whatever_the_weather() {
        let c = Map::Tower.course();
        let t = c.tower.as_ref().unwrap();
        let out = eye_at(t, ONE / 2);
        let inside = t.portal.tear.add(t.portal.through.scale(int(30)));
        assert!(t.in_void(inside) && !t.in_void(out));
        let count = |b: &mut Batch| {
            let n = std::cell::Cell::new(0);
            b.restyle(|c, g| {
                n.set(n.get() + 1);
                (c, g)
            });
            n.get()
        };
        for w in [Weather::Clear, Weather::Twilight, Weather::Dawn] {
            let l = look_in(t, inside, w);
            assert_eq!((l.sky, l.fog, l.rim), (ZIPPER.sky, ZIPPER.fog, ZIPPER.rim));
            assert_ne!(look_in(t, out, w).sky, ZIPPER.sky);
            // the void draws its dome and motes, no sun, stars or clouds
            // (far from the portal, which is drawn each frame near it)
            let deep = t
                .portal
                .tear
                .add(t.portal.through.scale(PORTAL_SEEN + int(50)));
            let mut b = Batch::new();
            dynamic(&mut b, t, deep, 0, w);
            assert_eq!(count(&mut b), 161, "{w:?}");
            // near the tear the portal is drawn too, and it turns
            let mut b = Batch::new();
            dynamic(&mut b, t, inside, 0, w);
            let near = count(&mut b);
            assert!(near > 161 + 150, "{w:?}: {near} shapes by the portal");
            let (mut a, mut z) = (Batch::new(), Batch::new());
            dynamic(&mut a, t, inside, 0, w);
            dynamic(&mut z, t, inside, 30, w);
            assert_ne!(
                a.finish(crate::render::DYNAMIC_BUFFER).0,
                z.finish(crate::render::DYNAMIC_BUFFER).0,
                "{w:?}: the portal moves"
            );
        }
    }

    #[test]
    fn inside_the_cloud_band_the_fog_closes_in() {
        let c = Map::Tower.course();
        let t = c.tower.as_ref().unwrap();
        let band = t.clouds[0];
        let mut eye = eye_at(t, 0);
        eye.y = (band.bottom + band.top) / 2;
        let l = look_in(t, eye, Weather::Clear);
        assert_eq!(in_cloud(t, eye.y), ONE);
        assert!(l.fog_range.1 <= int(110));
        eye.y = band.top + int(10);
        assert!(look_in(t, eye, Weather::Clear).fog_range.1 > int(300));
    }
}
