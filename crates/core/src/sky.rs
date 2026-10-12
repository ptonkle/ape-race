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
/// Ticks per colour.
const STEP: u32 = 60;
/// km/h over the redline at which the sky is fully changed.
const RAMP_KMH: i32 = 30;

/// How changed the sky is, 0..=ONE, from a speed in Q16 m/s.
pub fn trip(speed: i32) -> i32 {
    let over = speedo::kmh(speed) - speedo::REDLINE_KMH;
    if over <= 0 {
        return 0;
    }
    fx::div(int(over.min(RAMP_KMH)), int(RAMP_KMH))
}

/// The cycle's colour at a tick, blended between its stops.
fn cycle(tick: u32) -> Rgb {
    let k = (tick / STEP) as usize % CYCLE.len();
    let next = (k + 1) % CYCLE.len();
    let t = ((tick % STEP) as i32 * ONE) / STEP as i32;
    mix(CYCLE[k], CYCLE[next], t)
}

/// The sky and fog colours for a speed and a tick.
pub fn colours(sky: Rgb, fog: Rgb, speed: i32, tick: u32) -> (Rgb, Rgb) {
    let t = trip(speed);
    if t == 0 {
        return (sky, fog);
    }
    let wild = cycle(tick);
    // the fog takes the colour a little behind the sky, for depth
    let wild_fog = cycle(tick + STEP / 2);
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
        // past the redline (90 km/h, a person's best) it eases in, fully on
        // 30 km/h above
        assert!(trip(int(100) * 10 / 36) > 0 && trip(int(100) * 10 / 36) < ONE);
        assert_eq!(trip(int(125) * 10 / 36), ONE);
        let (s, _) = colours(sky, fog, int(60), 0);
        assert_ne!(s, sky);
    }

    #[test]
    fn the_colours_cycle_over_ticks() {
        assert_ne!(cycle(0), cycle(STEP));
        assert_eq!(cycle(0), cycle(STEP * CYCLE.len() as u32));
    }
}
