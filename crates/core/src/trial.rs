//! Time Trial (TRIAGE T1): one ape alone on the course, Speed Boosts only
//! (T13), against target times in the Trackmania manner: Bronze, Silver,
//! Gold, and a hidden Author time that shows once Gold is earned.
//!
//! The target times are per map: the Jungle's four constants and the
//! City's (`medals(map)`). Re-tune them whenever
//! the course or the physics changes: run
//! `cargo test --release -p tool_core --test ai -- --ignored --nocapture`
//! and set the Author time to about the AI's time alone, the others in
//! generous steps above it.

use serde::Serialize;

use crate::map::Map;
use crate::rules::TICKS_PER_SECOND;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Medal {
    Bronze,
    Silver,
    Gold,
    Author,
}

impl Medal {
    pub const ALL: [Medal; 4] = [Medal::Bronze, Medal::Silver, Medal::Gold, Medal::Author];

    pub fn key(self) -> &'static str {
        match self {
            Medal::Bronze => "bronze",
            Medal::Silver => "silver",
            Medal::Gold => "gold",
            Medal::Author => "author",
        }
    }
}

/// Hundredths of a second as race ticks.
const fn secs(hundredths: u32) -> u32 {
    hundredths * TICKS_PER_SECOND as u32 / 100
}

/// The jungle's target times in hundredths of a second (2026-10-12, TRIAGE
/// T29, after the drag cut, timing-based grabs, bigger mushrooms with a
/// push, banana arcs and catch-up vines: King Kong alone as a Time Trial
/// runs finishes in 68.9–76.4 s by kind; `time_trial_report` in
/// `tests/ai.rs`). Author: about the AI's best kind, a strong run. Then
/// generous steps above (a person is slower than the AI, TRIAGE row 16).
pub const AUTHOR: u32 = 6_800;
pub const GOLD: u32 = 7_500;
pub const SILVER: u32 = 8_500;
pub const BRONZE: u32 = 10_000;

/// The jungle's target times, in race ticks, slowest first.
pub const JUNGLE: [u32; 4] = [secs(BRONZE), secs(SILVER), secs(GOLD), secs(AUTHOR)];

/// The City's target times in hundredths of a second (2026-10-12, set as
/// the Jungle's were: King Kong alone as a Time Trial runs finishes in
/// 64.4–94.5 s by kind, Orangutan best (after the outside of the turns
/// was cleared and the vines made denser); `city_time_trial_report` in
/// `tests/city_ai.rs`, SECOND-ORDER C rows). Author: just under the best
/// kind; then the Jungle's steps above it (7, 10, 15 s).
pub const CITY_AUTHOR: u32 = 6_400;
pub const CITY_GOLD: u32 = 7_100;
pub const CITY_SILVER: u32 = 8_100;
pub const CITY_BRONZE: u32 = 9_600;

/// The City's target times, in race ticks, slowest first.
pub const CITY: [u32; 4] = [
    secs(CITY_BRONZE),
    secs(CITY_SILVER),
    secs(CITY_GOLD),
    secs(CITY_AUTHOR),
];

/// The Ski Slope's target times in hundredths of a second (2026-10-13, set
/// as the City's were: King Kong alone as a Time Trial runs finishes in
/// 57.8–88.7 s by kind, Lemur best, on the widened run; `ski_time_trial_report` in
/// `tests/ski_ai.rs`). Author: just under the best kind; then the
/// Jungle's steps above it (7, 10, 15 s).
pub const SKI_AUTHOR: u32 = 5_700;
pub const SKI_GOLD: u32 = 6_400;
pub const SKI_SILVER: u32 = 7_400;
pub const SKI_BRONZE: u32 = 8_900;

/// The Ski Slope's target times, in race ticks, slowest first.
pub const SKI: [u32; 4] = [
    secs(SKI_BRONZE),
    secs(SKI_SILVER),
    secs(SKI_GOLD),
    secs(SKI_AUTHOR),
];

/// A map's target times, in race ticks, slowest first (Bronze, Silver,
/// Gold, Author).
pub fn medals(map: Map) -> [u32; 4] {
    match map {
        Map::Jungle => JUNGLE,
        Map::City => CITY,
        Map::Ski => SKI,
        // STAND-IN: the Jungle's until measured
        Map::Sector => JUNGLE,
    }
}

/// The target of a medal on a map, in race ticks.
pub fn target_on(map: Map, m: Medal) -> u32 {
    medals(map)[m as usize]
}

/// The best medal a finish in `ticks` on a map earns.
pub fn earned_on(map: Map, ticks: u32) -> Option<Medal> {
    Medal::ALL
        .iter()
        .rev()
        .copied()
        .find(|m| ticks <= target_on(map, *m))
}

/// Whether a map's Author time may be shown: only once a best on it has
/// earned Gold.
pub fn author_shown_on(map: Map, best: Option<u32>) -> bool {
    best.map(|b| b <= target_on(map, Medal::Gold))
        .unwrap_or(false)
}

/// The next medal to aim for on a map, given the best on it so far (None:
/// Author earned).
pub fn next_target_on(map: Map, best: Option<u32>) -> Option<Medal> {
    match best.and_then(|b| earned_on(map, b)) {
        None => Some(Medal::Bronze),
        Some(Medal::Author) => None,
        Some(m) => Medal::ALL.get(m as usize + 1).copied(),
    }
}

/// The Jungle's: [`target_on`] for the Jungle.
pub fn target(m: Medal) -> u32 {
    target_on(Map::Jungle, m)
}

/// The Jungle's: [`earned_on`] for the Jungle.
pub fn earned(ticks: u32) -> Option<Medal> {
    earned_on(Map::Jungle, ticks)
}

/// The Jungle's: [`author_shown_on`] for the Jungle.
pub fn author_shown(best: Option<u32>) -> bool {
    author_shown_on(Map::Jungle, best)
}

/// The Jungle's: [`next_target_on`] for the Jungle.
pub fn next_target(best: Option<u32>) -> Option<Medal> {
    next_target_on(Map::Jungle, best)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_get_harder_up_to_the_author() {
        for w in JUNGLE.windows(2) {
            assert!(w[1] < w[0]);
        }
        assert_eq!(target(Medal::Author), 68 * TICKS_PER_SECOND as u32);
    }

    #[test]
    fn each_map_has_its_own_medals_and_the_jungles_are_unchanged() {
        assert_eq!(medals(Map::Jungle), [12_000, 10_200, 9_000, 8_160]);
        for map in Map::ALL {
            for w in medals(map).windows(2) {
                assert!(w[1] < w[0], "{map:?}");
            }
        }
        assert_ne!(medals(Map::City), medals(Map::Jungle));
        let gold = target_on(Map::City, Medal::Gold);
        assert_eq!(earned_on(Map::City, gold), Some(Medal::Gold));
        assert_eq!(earned_on(Map::City, gold + 1), Some(Medal::Silver));
        assert!(author_shown_on(Map::City, Some(gold)));
        assert!(!author_shown_on(Map::City, Some(gold + 1)));
        assert_eq!(next_target_on(Map::City, Some(gold)), Some(Medal::Author));
        assert_eq!(next_target_on(Map::City, None), Some(Medal::Bronze));
    }

    #[test]
    fn a_finish_earns_the_best_medal_it_beats() {
        assert_eq!(earned(target(Medal::Bronze) + 1), None);
        assert_eq!(earned(target(Medal::Bronze)), Some(Medal::Bronze));
        assert_eq!(earned(target(Medal::Gold) - 1), Some(Medal::Gold));
        assert_eq!(earned(target(Medal::Author)), Some(Medal::Author));
    }

    #[test]
    fn the_author_target_hides_until_gold() {
        assert!(!author_shown(None));
        assert!(!author_shown(Some(target(Medal::Silver))));
        assert!(author_shown(Some(target(Medal::Gold))));
    }

    #[test]
    fn the_next_target_follows_the_best() {
        assert_eq!(next_target(None), Some(Medal::Bronze));
        assert_eq!(
            next_target(Some(target(Medal::Bronze))),
            Some(Medal::Silver)
        );
        assert_eq!(next_target(Some(target(Medal::Gold))), Some(Medal::Author));
        assert_eq!(next_target(Some(target(Medal::Author))), None);
    }
}
