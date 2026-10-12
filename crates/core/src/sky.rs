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
const STEP_M: i32 = int(60);
/// km/h over the redline at which the sky is fully changed: it builds
/// with the exact speed all the way up (TRIAGE row 15, a try).
const RAMP_KMH: i32 = 40;
/// The colours start here (TRIAGE row 22: a wider, gentler ramp).
const FROM_KMH: i32 = 80;

/// How changed the sky is, 0..=ONE, from a speed in Q16 m/s: zero at the
/// redline, eased in (smoothstep) to full at the redline + `RAMP_KMH`.
pub fn trip(speed: i32) -> i32 {
    let over = speedo::kmh(speed) - FROM_KMH;
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

/// The jungle's wild colours (TRIAGE rows 18 and 20, a try): the ground,
/// the wood and the leaves each take their own colour from the cycle, a
/// stop apart and apart from the sky, so they stay distinct from each
/// other; the vines, apes and items keep theirs.
pub fn world(speed: i32, metres: i32) -> (i32, [Rgb; 3]) {
    (
        trip(speed),
        [
            cycle(metres + STEP_M),
            cycle(metres + STEP_M * 2),
            cycle(metres + STEP_M * 3),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sky_changes_only_at_the_top_of_the_dial() {
        let sky = rgb(150, 206, 196);
        let fog = rgb(120, 178, 150);
        // a good ordinary pace (65 km/h across the ground) leaves it alone
        let fast = int(65) * 10 / 36;
        assert_eq!(colours(sky, fog, fast, 77), (sky, fog));
        assert_eq!(trip(int(79) * 10 / 36), 0);
        // from 80 km/h across the ground it builds gently with the speed,
        // full at 120 (TRIAGE rows 17, 19, 22 and 26)
        let at = |kmh: i32| trip(int(kmh) * 10 / 36);
        assert!(at(85) > 0 && at(85) < at(100) && at(100) < at(115) && at(115) < ONE);
        assert_eq!(at(121), ONE);
        let (s, _) = colours(sky, fog, int(60), 0);
        assert_ne!(s, sky);
    }

    #[test]
    fn the_jungle_changes_with_the_sky() {
        assert_eq!(world(int(19), 0).0, 0);
        assert_eq!(world(int(40), 0).0, ONE);
        let (_, [ground, wood, leaves]) = world(int(40), 0);
        let (sky, _) = colours(rgb(1, 1, 1), rgb(1, 1, 1), int(40), 0);
        // four different colours at once, wherever the cycle is
        for (a, b) in [(sky, ground), (ground, wood), (wood, leaves), (leaves, sky)] {
            assert_ne!(a, b);
        }
    }

    #[test]
    fn the_colours_turn_with_the_ground_covered() {
        assert_ne!(cycle(0), cycle(STEP_M));
        assert_eq!(cycle(0), cycle(STEP_M * CYCLE.len() as i32));
    }
}
