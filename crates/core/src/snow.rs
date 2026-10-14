//! The Ski Slope's weather (2026-10-13): Clear (the default: noon, sunny)
//! or Snowing. It is each player's own view, a setting like the colourblind
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
}

impl Weather {
    pub const ALL: [Weather; 2] = [Weather::Clear, Weather::Snowing];

    /// The setting's word, and `weather.<key>` in data/strings.json.
    pub fn key(self) -> &'static str {
        match self {
            Weather::Clear => "clear",
            Weather::Snowing => "snowing",
        }
    }

    pub fn from_key(key: &str) -> Option<Weather> {
        Weather::ALL.into_iter().find(|w| w.key() == key)
    }
}

/// The side of a column of snow: each holds `PER_CELL` flakes.
const CELL: i32 = 3;
const PER_CELL: u64 = 3;
/// How far round the eye the flakes fall (metres), and how tall a column
/// is before it repeats (metres): every flake is in each column once.
const REACH: i32 = 27;
const SPAN: i32 = 30;
/// No flake nearer the eye than this, so none fills the screen.
const CLEAR_OF_EYE: i32 = cm(150);
/// Falling speed, centimetres a second (each flake its own, in this range).
const FALL: (i32, i32) = (95, 160);
/// The wind: centimetres a second along x and z.
const WIND: (i32, i32) = (45, 25);
/// A flake's side, centimetres (each its own, in this range).
const SIZE: (i32, i32) = (11, 18);
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
    lo + ((h >> shift) & 0xFFFF) as i32 * (hi - lo) / 0x1_0000
}

/// `a` wrapped into `0..m` (both Q16, as i64).
fn wrap(a: i64, m: i64) -> i64 {
    a.rem_euclid(m)
}

/// Where each flake near `eye` is at `tick`, and its size and spin: the
/// flakes the frame draws, in no particular order.
pub fn flakes_at(eye: V3, tick: u32) -> Vec<(V3, i32, i32)> {
    let t = tick as i64;
    let one = ONE as i64;
    let cell = CELL as i64 * one;
    let span = SPAN as i64 * one;
    // the whole fall drifts with the wind: find the columns in the
    // drifted frame, then move them back
    let wind = (
        WIND.0 as i64 * one / 100 * t / TICKS,
        WIND.1 as i64 * one / 100 * t / TICKS,
    );
    let ex = eye.x as i64 - wind.0;
    let ez = eye.z as i64 - wind.1;
    let reach = REACH as i64 * one;
    let (x0, x1) = ((ex - reach).div_euclid(cell), (ex + reach).div_euclid(cell));
    let (z0, z1) = ((ez - reach).div_euclid(cell), (ez + reach).div_euclid(cell));
    let low = eye.y as i64 - span / 2;
    let mut out = Vec::new();
    for cx in x0..=x1 {
        for cz in z0..=z1 {
            for k in 0..PER_CELL {
                let h = hash(cx, cz, k);
                let ox = pick(h, 0, 0, ONE) as i64 * CELL as i64;
                let oz = pick(h, 16, 0, ONE) as i64 * CELL as i64;
                let fall = pick(h, 32, FALL.0, FALL.1) as i64 * one / 100;
                let start = pick(h, 48, 0, ONE) as i64 * SPAN as i64;
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
                if p.dist(eye) < CLEAR_OF_EYE {
                    continue;
                }
                let size = cm(pick(h2, 48, SIZE.0, SIZE.1));
                let spin = (phase as i64 * 3 + rate as i64 * 2 * t) as i32;
                out.push((p, size, spin));
            }
        }
    }
    out
}

/// The falling snow round `eye` at `tick`.
pub fn flakes(b: &mut Batch, eye: V3, tick: u32) {
    for (p, size, spin) in flakes_at(eye, tick) {
        // a chunky square, tumbling slowly
        let a = Affine::translate(p)
            .then(&Affine::rot_y(spin))
            .then(&Affine::rot_x(spin / 2))
            .then(&Affine::scale(v3(size, size, size)));
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

/// Whether a point is inside the snow's reach of `eye` (for tests).
#[cfg(test)]
fn within(eye: V3, p: V3) -> bool {
    use crate::fx::int;
    p.sub(eye).len_xz() <= int(REACH) + cm(60) && (p.y - eye.y).abs() <= int(SPAN) / 2 + ONE
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
            assert!(p.dist(eye) >= CLEAR_OF_EYE);
            assert!((cm(SIZE.0)..=cm(SIZE.1)).contains(size));
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
        for eye in [V3::ZERO, v3(int(-700), int(-90), int(1234)), v3(int(5), 0, int(-9))] {
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
        let shared = a
            .iter()
            .filter(|f| b.contains(f))
            .count();
        assert!(shared * 10 > a.len() * 6, "{shared} of {} kept", a.len());
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
