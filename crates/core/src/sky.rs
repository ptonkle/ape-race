//! The sky at speed (TRIAGE T20): at the very top of the speedometer the
//! sky and fog swing through strong colours, as in JoJo's Bizarre Adventure
//! Part 4. Only at the top (the person: not to oversaturate), easing in
//! from the redline and fully on a little above it.

use crate::fx::{self, int, ONE};
use crate::render::{mix, rgb, Rgb};
use crate::speedo;

/// The colours the sky cycles through.
const CYCLE: [Rgb; 5] = [
    rgb(255, 92, 170),
    rgb(150, 86, 236),
    rgb(255, 214, 72),
    rgb(64, 214, 196),
    rgb(255, 140, 60),
];
/// Metres travelled per colour: the colours turn with the ground covered,
/// so they spin faster the faster the ape goes (TRIAGE row 15, a try).
const STEP_M: i32 = int(25);
/// km/h over the redline at which the sky is fully changed: it builds
/// with the exact speed all the way up (TRIAGE row 15, a try).
const RAMP_KMH: i32 = 30;

/// How changed the sky is, 0..=ONE, from a speed in Q16 m/s: zero at the
/// redline, eased in (smoothstep) to full at the redline + `RAMP_KMH`.
pub fn trip(speed: i32) -> i32 {
    let over = speedo::kmh(speed) - speedo::REDLINE_KMH;
    if over <= 0 {
        return 0;
    }
    let x = fx::div(int(over.min(RAMP_KMH)), int(RAMP_KMH));
    fx::mul(fx::mul(x, x), int(3) - 2 * x)
}

/// The cycle's colour after `metres` (Q16) travelled, blended between stops.
fn cycle(metres: i32) -> Rgb {
    let m = metres.max(0);
    let k = (m / STEP_M) as usize % CYCLE.len();
    let next = (k + 1) % CYCLE.len();
    let t = fx::div(m % STEP_M, STEP_M);
    mix(CYCLE[k], CYCLE[next], t)
}

/// The sky and fog colours for a speed and the metres travelled.
pub fn colours(sky: Rgb, fog: Rgb, speed: i32, metres: i32) -> (Rgb, Rgb) {
    let t = trip(speed);
    if t == 0 {
        return (sky, fog);
    }
    let wild = cycle(metres);
    // the fog takes the colour a little behind the sky, for depth
    let wild_fog = cycle(metres + STEP_M / 2);
    (mix(sky, wild, t), mix(fog, mix(wild_fog, fog, ONE / 3), t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sky_changes_only_at_the_top_of_the_dial() {
        let sky = rgb(150, 206, 196);
        let fog = rgb(120, 178, 150);
        // a good ordinary swing (75 km/h) leaves the sky alone
        let fast = int(75) * 10 / 36;
        assert_eq!(colours(sky, fog, fast, 77), (sky, fog));
        assert_eq!(trip(int(85) * 10 / 36), 0);
        // past the redline (90 km/h, a person's best) it builds with the
        // exact speed, full 30 km/h above (TRIAGE row 17)
        let at = |kmh: i32| trip(int(kmh) * 10 / 36);
        assert!(at(95) > 0 && at(95) < at(105) && at(105) < at(115) && at(115) < ONE);
        assert_eq!(at(121), ONE);
        let (s, _) = colours(sky, fog, int(60), 0);
        assert_ne!(s, sky);
    }

    #[test]
    fn the_colours_turn_with_the_ground_covered() {
        assert_ne!(cycle(0), cycle(STEP_M));
        assert_eq!(cycle(0), cycle(STEP_M * CYCLE.len() as i32));
    }
}
