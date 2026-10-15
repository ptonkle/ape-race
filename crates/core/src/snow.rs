//! The Ski Slope's weather (2026-10-13): Clear (the default: noon, sunny),
//! Snowing, or a Blizzard (TRIAGE T63, 2026-10-14). It is each player's own view, a setting like the colourblind
//! mode: it changes nothing but the drawing (not the physics, not a race's
//! outcome, not what goes online).
//!
//! The snow falls "a bit like Minecraft": chunky square white flakes
//! falling steadily and swaying gently sideways while the whole fall drifts
//! on a light wind. The flakes fill a volume round the eye, so they surround
//! the camera at any speed. Each one is fixed in the world (it does not
//! move with the eye), and where it is comes only from the tick and a hash
//! of its cell, so the same tick always draws the same snow. They are
//! sparse, so they never hide the vines. While it snows the sky and the
//! fog turn grey and the fog comes in a little closer.
//!
//! A blizzard is the same fall, thicker and wilder: about twice as many
//! flakes, driven hard sideways by a gale so each shows as a short streak
//! along its flight, a near-white sky, the fog close in (the far mountains
//! gone in it) and the sun dim. It is still only the drawing.

use serde::{Deserialize, Serialize};

use crate::fx::{self, cm, mul, ONE, TURN};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, rgb, shade, Batch, Rgb};
use crate::scene::Look;

/// The weather on the Ski Slope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Weather {
    #[default]
    Clear,
    Snowing,
    Blizzard,
}

impl Weather {
    /// Every weather, in the order the Weather selects offer them.
    pub const ALL: [Weather; 3] = [Weather::Clear, Weather::Snowing, Weather::Blizzard];

    /// The setting's word, and `weather.<key>` in data/strings.json.
    pub fn key(self) -> &'static str {
        match self {
            Weather::Clear => "clear",
            Weather::Snowing => "snowing",
            Weather::Blizzard => "blizzard",
        }
    }

    /// Whether snow falls in it.
    pub fn falls(self) -> bool {
        self != Weather::Clear
    }

    pub fn from_key(key: &str) -> Option<Weather> {
        Weather::ALL.into_iter().find(|w| w.key() == key)
    }
}

/// How a weather's snow falls: the side of a column (metres) and the
/// flakes in each, how far round the eye they fall and how tall a column
/// is before it repeats (metres: every flake is in each column once), the
/// falling speed and the wind (centimetres a second; each flake falls at
/// its own speed in the range), a flake's side (centimetres), and how long
/// a streak each flake draws along its flight (milliseconds of it; 0: a
/// tumbling square).
#[derive(Debug, Clone, Copy)]
struct Fall {
    cell: i32,
    per_cell: u64,
    reach: i32,
    span: i32,
    fall: (i32, i32),
    wind: (i32, i32),
    size: (i32, i32),
    streak: i32,
    /// no flake nearer the eye than this (centimetres), so none fills the
    /// screen
    clear: i32,
}

/// Snowing: about 3 flakes a 3 m column within 27 m, falling steadily on
/// a light wind.
const SNOWING: Fall = Fall {
    cell: 3,
    per_cell: 3,
    reach: 27,
    span: 30,
    fall: (95, 160),
    wind: (45, 25),
    size: (11, 18),
    streak: 0,
    clear: 150,
};
/// A blizzard: well over twice as thick (4 a 2 m column), a little nearer
/// in (the fog hides the rest), falling faster and driven along x by a gale (the course
/// runs down z, so it blows across it) in streaks.
const BLIZZARD: Fall = Fall {
    cell: 2,
    per_cell: 4,
    reach: 24,
    span: 24,
    fall: (220, 340),
    wind: (1150, 280),
    size: (4, 7),
    streak: 70,
    clear: 250,
};
const FLAKE: Rgb = rgb(246, 248, 252);
/// Ticks a second (the app's frame tick).
const TICKS: i64 = crate::rules::TICKS_PER_SECOND as i64;

/// A well-mixed number from a column and a flake in it.
fn hash(cx: i64, cz: i64, k: u64) -> u64 {
    let mut d = fx::Dice(
        (cx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (cz as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
            ^ k.wrapping_mul(0x1656_67B1_9E37_79F9)
            ^ 0x5_0000_F1A4,
    );
    d.next();
    d.next()
}

/// A value in `lo..hi` from bits of `h`.
fn pick(h: u64, shift: u32, lo: i32, hi: i32) -> i32 {
    lo + (((h >> shift) & 0xFFFF) as i64 * (hi - lo) as i64 / 0x1_0000) as i32
}

/// `a` wrapped into `0..m` (both Q16, as i64).
fn wrap(a: i64, m: i64) -> i64 {
    a.rem_euclid(m)
}

/// The fall of a weather with snow.
fn fall_of(w: Weather) -> Option<Fall> {
    match w {
        Weather::Clear => None,
        Weather::Snowing => Some(SNOWING),
        Weather::Blizzard => Some(BLIZZARD),
    }
}

/// Where each flake near `eye` is at `tick`, and its size and spin, in
/// the snow: the flakes the frame draws, in no particular order.
pub fn flakes_at(eye: V3, tick: u32) -> Vec<(V3, i32, i32)> {
    flakes_of(SNOWING, eye, tick)
}

fn flakes_of(f: Fall, eye: V3, tick: u32) -> Vec<(V3, i32, i32)> {
    let Fall {
        cell: cell_m,
        per_cell,
        reach,
        span,
        fall: fall_range,
        wind,
        size: size_range,
        ..
    } = f;
    let t = tick as i64;
    let one = ONE as i64;
    let cell = cell_m as i64 * one;
    let span_m = span;
    let span = span as i64 * one;
    // the whole fall drifts with the wind: find the columns in the
    // drifted frame, then move them back
    let wind = (
        wind.0 as i64 * one / 100 * t / TICKS,
        wind.1 as i64 * one / 100 * t / TICKS,
    );
    let ex = eye.x as i64 - wind.0;
    let ez = eye.z as i64 - wind.1;
    let reach = reach as i64 * one;
    let (x0, x1) = ((ex - reach).div_euclid(cell), (ex + reach).div_euclid(cell));
    let (z0, z1) = ((ez - reach).div_euclid(cell), (ez + reach).div_euclid(cell));
    let low = eye.y as i64 - span / 2;
    let mut out = Vec::new();
    for cx in x0..=x1 {
        for cz in z0..=z1 {
            for k in 0..per_cell {
                let h = hash(cx, cz, k);
                let ox = pick(h, 0, 0, ONE) as i64 * cell_m as i64;
                let oz = pick(h, 16, 0, ONE) as i64 * cell_m as i64;
                let fall = pick(h, 32, fall_range.0, fall_range.1) as i64 * one / 100;
                let start = pick(h, 48, 0, ONE) as i64 * span_m as i64;
                let h2 = hash(cx, cz, k + 1000);
                // down the column, wrapped into the span round the eye
                let drop = start + fall * t / TICKS;
                let y = low + wrap(-drop - low, span);
                // a gentle sway, each flake its own
                let rate = pick(h2, 0, TURN / 500, TURN / 260);
                let phase = pick(h2, 16, 0, TURN);
                let a = (phase as i64 + rate as i64 * t) as i32;
                let sway = pick(h2, 32, cm(20), cm(55));
                let dx = mul(fx::sin(a), sway) as i64;
                let dz = mul(fx::cos(a + TURN / 7), sway / 2) as i64;
                let x = cx * cell + ox + dx - ex;
                let z = cz * cell + oz + dz - ez;
                if x * x + z * z > reach * reach {
                    continue;
                }
                let p = v3(
                    (eye.x as i64 + x) as i32,
                    y as i32,
                    (eye.z as i64 + z) as i32,
                );
                if p.dist(eye) < cm(f.clear) {
                    continue;
                }
                let size = cm(pick(h2, 48, size_range.0, size_range.1));
                let spin = (phase as i64 * 3 + rate as i64 * 2 * t) as i32;
                out.push((p, size, spin));
            }
        }
    }
    out
}

/// The falling snow round `eye` at `tick` in weather `w` (none when it
/// is clear).
pub fn flakes(b: &mut Batch, eye: V3, tick: u32, w: Weather) {
    let Some(f) = fall_of(w) else {
        return;
    };
    // a streak lies along the flight: the wind across, the fall down
    let flight = v3(
        f.wind.0 * ONE / 100,
        -(f.fall.0 + f.fall.1) / 2 * ONE / 100,
        f.wind.1 * ONE / 100,
    );
    let length = mul(flight.len(), f.streak * ONE / 1000);
    let along = Affine::along(flight);
    for (p, size, spin) in flakes_of(f, eye, tick) {
        let a = if f.streak > 0 {
            // a thin streak, each a little longer or shorter
            Affine::translate(p).then(&along).then(&Affine::scale(v3(
                size,
                length * size / cm(8),
                size,
            )))
        } else {
            // a chunky square, tumbling slowly
            Affine::translate(p)
                .then(&Affine::rot_y(spin))
                .then(&Affine::rot_x(spin / 2))
                .then(&Affine::scale(v3(size, size, size)))
        };
        b.push(Mesh::Cube, &a, FLAKE, ONE * 7 / 10);
    }
}

/// The grey of a snowy sky, and of the fog in the snow.
const SNOW_SKY: Rgb = rgb(196, 204, 216);
const SNOW_FOG: Rgb = rgb(214, 220, 230);

/// A map's look under falling snow: the sky and the fog grey, the fog a
/// little closer, the sun weaker.
pub fn overcast(look: Look) -> Look {
    Look {
        sky: mix(look.sky, SNOW_SKY, ONE * 3 / 4),
        fog: mix(look.fog, SNOW_FOG, ONE * 3 / 4),
        fog_range: (look.fog_range.0 * 3 / 5, look.fog_range.1 * 4 / 5),
        light: shade(look.light, 82),
        ..look
    }
}

/// The near-white of a blizzard's sky, and of its fog.
const WHITEOUT_SKY: Rgb = rgb(226, 231, 238);
const WHITEOUT_FOG: Rgb = rgb(236, 239, 244);

/// A map's look in a blizzard: the sky and the fog near white, the fog
/// close in (the haze from 12 m, nothing past 130 m), the sun dim.
pub fn whiteout(look: Look) -> Look {
    Look {
        sky: mix(look.sky, WHITEOUT_SKY, ONE * 9 / 10),
        fog: mix(look.fog, WHITEOUT_FOG, ONE * 9 / 10),
        fog_range: (fx::int(12), fx::int(130)),
        light: shade(look.light, 64),
        ..look
    }
}

/// A map's look in weather `w`.
pub fn sky_in(w: Weather, look: Look) -> Look {
    match w {
        Weather::Clear => look,
        Weather::Snowing => overcast(look),
        Weather::Blizzard => whiteout(look),
    }
}

/// The glow `scene.frag` reads as a far backdrop (the Ski Slope's
/// mountains), from 2.5 to 3.5: in a blizzard these are hazed all the way
/// into the fog, so the mountains are lost in it.
pub fn bury_backdrop(glow: i32, w: Weather) -> i32 {
    let far = (ONE * 5 / 2..ONE * 7 / 2).contains(&glow);
    if w == Weather::Blizzard && far {
        ONE * 7 / 2 - ONE / 100
    } else {
        glow
    }
}

/// A shape's colour and glow in weather `w`: in a blizzard the far
/// backdrop is buried in the fog (`bury_backdrop`) and the sky's dome (a
/// glow of 3.5 or more, its colour a multiple of the sky's) is the sky's
/// own near-white overhead instead of a deep blue, so the buried mountains
/// do not show against it.
pub fn in_storm(colour: Rgb, glow: i32, w: Weather) -> (Rgb, i32) {
    if w == Weather::Blizzard && glow >= ONE * 7 / 2 {
        ([ONE; 3], glow)
    } else {
        (colour, bury_backdrop(glow, w))
    }
}

/// Whether a point is inside the snow's reach of `eye` (for tests).
#[cfg(test)]
fn within(eye: V3, p: V3) -> bool {
    use crate::fx::int;
    p.sub(eye).len_xz() <= int(SNOWING.reach) + cm(60)
        && (p.y - eye.y).abs() <= int(SNOWING.span) / 2 + ONE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::int;

    #[test]
    fn the_same_tick_draws_the_same_snow_and_it_falls() {
        let eye = v3(int(40), int(-120), int(300));
        let a = flakes_at(eye, 500);
        assert_eq!(a, flakes_at(eye, 500), "deterministic from the tick");
        let n = a.len();
        // about 3 flakes a 3 m column within 27 m: sparse, but all round
        assert!((450..=900).contains(&n), "{n} flakes");
        for (p, size, _) in &a {
            assert!(within(eye, *p), "{p:?} outside the snow round the eye");
            assert!(p.dist(eye) >= cm(SNOWING.clear));
            assert!((cm(SNOWING.size.0)..=cm(SNOWING.size.1)).contains(size));
        }
        // a second later most flakes are lower than they were, by about a
        // metre (matched by their sideways place, which hardly changes)
        let b = flakes_at(eye, 500 + 120);
        let mut fell = 0;
        for (p, _, _) in &a {
            if let Some((q, _, _)) = b
                .iter()
                .find(|(q, _, _)| q.flat().dist(p.flat()) < cm(80) && q.y < p.y)
            {
                let d = p.y - q.y;
                if (cm(80)..=cm(200)).contains(&d) {
                    fell += 1;
                }
            }
        }
        assert!(fell * 2 > n, "only {fell} of {n} fell about a metre");
    }

    #[test]
    fn the_snow_surrounds_the_eye_wherever_it_goes() {
        // every side of the eye has flakes, near and far, at any place
        for eye in [
            V3::ZERO,
            v3(int(-700), int(-90), int(1234)),
            v3(int(5), 0, int(-9)),
        ] {
            let f = flakes_at(eye, 9_000);
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let side = f
                    .iter()
                    .filter(|(p, _, _)| {
                        let d = p.sub(eye);
                        d.x * dx + d.z * dz > int(3)
                    })
                    .count();
                assert!(side > 100, "{side} flakes on one side");
            }
            let near = f.iter().filter(|(p, _, _)| p.dist(eye) < int(8)).count();
            assert!(near >= 3, "{near} flakes within 8 m");
        }
    }

    #[test]
    fn the_flakes_stay_put_in_the_world_as_the_eye_moves() {
        // a flake seen from two eyes a few metres apart is the same flake
        let a = flakes_at(V3::ZERO, 777);
        let b = flakes_at(v3(int(4), int(1), int(-3)), 777);
        let shared = a.iter().filter(|f| b.contains(f)).count();
        assert!(shared * 10 > a.len() * 6, "{shared} of {} kept", a.len());
    }

    #[test]
    fn a_blizzard_is_thick_driven_sideways_in_streaks_white_and_close() {
        // TRIAGE T63
        let eye = v3(int(40), int(-120), int(300));
        let snow = flakes_of(SNOWING, eye, 500);
        let storm = flakes_of(BLIZZARD, eye, 500);
        assert!(
            storm.len() > snow.len() * 2,
            "{} flakes in the blizzard, {} in the snow",
            storm.len(),
            snow.len()
        );
        // a tenth of a second later the flakes have flown about a metre
        // sideways (the gale along x) and a few centimetres down
        let later = flakes_of(BLIZZARD, eye, 512);
        let mut blown = 0;
        for (p, _, _) in &storm {
            let moved = later.iter().any(|(q, _, _)| {
                let d = q.sub(*p);
                (cm(80)..cm(160)).contains(&d.x) && d.y < 0 && d.y > -cm(60)
            });
            if moved {
                blown += 1;
            }
        }
        assert!(blown * 2 > storm.len(), "only {blown} blown sideways");
        // each is drawn as a streak along its flight, long and thin
        let mut b = Batch::new();
        flakes(&mut b, eye, 500, Weather::Blizzard);
        let (values, _) = b.finish(0);
        let first = &values[..16];
        let axis = |k: usize| v3(first[k], first[4 + k], first[8 + k]).len();
        assert!(axis(1) > axis(0) * 5, "{} by {}", axis(1), axis(0));
        let mut clear = Batch::new();
        flakes(&mut clear, eye, 500, Weather::Clear);
        assert_eq!(clear.count(), 0, "no snow in clear air");
        // the sky near white, the fog far nearer than the snow's
        let look = crate::scene::look(crate::map::Map::Ski);
        let (grey, white) = (
            sky_in(Weather::Snowing, look),
            sky_in(Weather::Blizzard, look),
        );
        assert_eq!(sky_in(Weather::Clear, look), look);
        assert!(white.fog_range.1 * 2 < grey.fog_range.1);
        assert!(white.fog_range.0 < grey.fog_range.0);
        let least = |c: Rgb| *c.iter().min().unwrap();
        assert!(least(white.sky) > least(grey.sky) && least(white.fog) > least(grey.fog));
        assert!(least(white.sky) > rgb(215, 0, 0)[0]);
        // the far mountains are buried in its fog, nothing else
        assert_eq!(bury_backdrop(ONE * 3, Weather::Snowing), ONE * 3);
        assert!(bury_backdrop(ONE * 3, Weather::Blizzard) > ONE * 7 / 2 - ONE / 50);
        assert_eq!(bury_backdrop(ONE / 4, Weather::Blizzard), ONE / 4);
        assert_eq!(bury_backdrop(ONE * 4, Weather::Blizzard), ONE * 4);
        // and the dome is the sky's own near-white overhead
        let deep = rgb(60, 140, 250);
        assert_eq!(
            in_storm(deep, ONE * 4, Weather::Blizzard),
            ([ONE; 3], ONE * 4)
        );
        assert_eq!(in_storm(deep, ONE * 4, Weather::Snowing), (deep, ONE * 4));
        assert_eq!(in_storm(deep, ONE / 2, Weather::Blizzard), (deep, ONE / 2));
        assert_eq!(Weather::from_key("blizzard"), Some(Weather::Blizzard));
        assert!(Weather::Blizzard.falls() && !Weather::Clear.falls());
    }

    #[test]
    fn snow_greys_the_sky_and_brings_the_fog_closer() {
        let clear = crate::scene::look(crate::map::Map::Ski);
        let snow = overcast(clear);
        assert!(snow.fog_range.1 < clear.fog_range.1);
        assert!(snow.fog_range.0 < clear.fog_range.0);
        // greyer: the channels closer together
        let spread = |c: Rgb| c.iter().max().unwrap() - c.iter().min().unwrap();
        assert!(spread(snow.sky) < spread(clear.sky));
        assert_eq!(Weather::from_key("snowing"), Some(Weather::Snowing));
        assert_eq!(Weather::default(), Weather::Clear);
    }
}
