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

/// Sector Ape's target times in hundredths of a second (2026-10-13, set as
/// the others' were: King Kong alone as a Time Trial runs finishes in
/// 57.0–90.9 s by kind, the Orangutan best; `sector_time_trial_report` in
/// `tests/sector_ai.rs`, SECOND-ORDER SC rows). Author: just under the
/// best kind; then the Jungle's steps above it (7, 10, 15 s).
pub const SECTOR_AUTHOR: u32 = 5_600;
pub const SECTOR_GOLD: u32 = 6_300;
pub const SECTOR_SILVER: u32 = 7_300;
pub const SECTOR_BRONZE: u32 = 8_800;

/// Sector Ape's target times, in race ticks, slowest first.
pub const SECTOR: [u32; 4] = [
    secs(SECTOR_BRONZE),
    secs(SECTOR_SILVER),
    secs(SECTOR_GOLD),
    secs(SECTOR_AUTHOR),
];

/// Sunset Ruins' target times in hundredths of a second (2026-10-14, set
/// as the Jungle's were: King Kong alone as a Time Trial runs finishes in
/// 56.8–61.9 s by kind, Lemur best, the Proboscis 80.1 s; `tests/ruins_ai.rs`
/// `ruins_time_trial_report`, re-measured after the layout fixes of RT6).
/// Author just under the best kind, then the Jungle's steps of 7, 10 and
/// 15 s.
pub const RUINS_AUTHOR: u32 = 5_600;
pub const RUINS_GOLD: u32 = 6_300;
pub const RUINS_SILVER: u32 = 7_300;
pub const RUINS_BRONZE: u32 = 8_800;

/// Sunset Ruins' target times, in race ticks, slowest first.
pub const RUINS: [u32; 4] = [
    secs(RUINS_BRONZE),
    secs(RUINS_SILVER),
    secs(RUINS_GOLD),
    secs(RUINS_AUTHOR),
];

/// Banana Bay Battle's target times in hundredths of a second, set as the
/// others' were: King Kong alone as a Time Trial runs; `tests/bay_ai.rs`
/// `bay_time_trial_report`. Author just under the best kind, then the
/// Jungle's steps of 7, 10 and 15 s. First 0:35 (BT2); re-measured for
/// the fleet overhaul (2026-10-15, two battle lines of bigger ships: 50.8–
/// 58.0 s by kind, the Gorilla best, the Lemur slowest; SECOND-ORDER BO);
/// again once the AI stopped whirling on ropes (BT5: 53.4–60.0 s, the
/// Proboscis best, the Gibbon slowest).
/// Re-measure if the course's length changes.
pub const BAY_AUTHOR: u32 = 5_300;
pub const BAY_GOLD: u32 = 6_000;
pub const BAY_SILVER: u32 = 7_000;
pub const BAY_BRONZE: u32 = 8_500;

/// Banana Bay Battle's target times, in race ticks, slowest first.
pub const BAY: [u32; 4] = [
    secs(BAY_BRONZE),
    secs(BAY_SILVER),
    secs(BAY_GOLD),
    secs(BAY_AUTHOR),
];

/// Mape's Tower's target times in hundredths of a second (the second
/// design, set as the others' were: King Kong alone as a Time Trial runs
/// finishes in 90.6-109.0 s by kind, the Chimpanzee best (90.58 s), the
/// Gibbon and the Proboscis 2-3 s behind, the Orangutan slowest;
/// `tests/tower_ai.rs` `tower_time_trial_report`, SECOND-ORDER TC2-20).
/// Author just under the best kind, then the Jungle's steps of 7, 10 and
/// 15 s. TF-2: after T96-T100 the best run is 95.05 s (King Kong
/// reaches higher beams inside, `TOWER_OVERHEAD` 12 m), so 94.5 / 101.5 /
/// 111.5 / 126.5 s; before that 90 / 97 / 107 / 122 s. (Before the climb's arms and the porch beam, TC2-4: 94.5 / 101.5
/// / 111.5 / 126.5 s; the first design's were 61 / 68 / 78 / 93 s.)
pub const TOWER_AUTHOR: u32 = 9_450;
pub const TOWER_GOLD: u32 = 10_150;
pub const TOWER_SILVER: u32 = 11_150;
pub const TOWER_BRONZE: u32 = 12_650;

/// Mape's Tower's target times, in race ticks, slowest first.
pub const TOWER: [u32; 4] = [
    secs(TOWER_BRONZE),
    secs(TOWER_SILVER),
    secs(TOWER_GOLD),
    secs(TOWER_AUTHOR),
];

/// Rattlesnake Ravine's target times in hundredths of a second (RMT1,
/// 2026-10-17, the real 1,754 m route): King Kong alone as a Time Trial
/// runs finishes in 65.3-86.0 s by kind, the Chimpanzee best (65.31 s,
/// Equal Stats the same), the Gorilla next (67.80 s), the Proboscis
/// slowest; `tests/ravine_ai.rs` `ravine_time_trial_report`. Author just
/// under the best kind, then the Jungle's steps of 7, 10 and 15 s. (The
/// skeleton's stand-ins were 58.5 / 65.5 / 75.5 / 90.5 s.) Re-measure if
/// the course's length changes.
pub const RAVINE_AUTHOR: u32 = 6_500;
pub const RAVINE_GOLD: u32 = 7_200;
pub const RAVINE_SILVER: u32 = 8_200;
pub const RAVINE_BRONZE: u32 = 9_700;

/// Rattlesnake Ravine's target times, in race ticks, slowest first.
pub const RAVINE: [u32; 4] = [
    secs(RAVINE_BRONZE),
    secs(RAVINE_SILVER),
    secs(RAVINE_GOLD),
    secs(RAVINE_AUTHOR),
];

/// Aisle Ape's target times in hundredths of a second (GR8; GR10,
/// 2026-10-18, the second pass's 2,082 m route with the shopping list and
/// the shoppers' swats): King Kong alone as a Time Trial finishes in
/// 93.5-124.7 s by kind, the Lemur best (93.47 s), the Gibbon and the
/// Proboscis next (104.5 s); the Chimpanzee did not finish (it fell off a
/// corner and stayed down); `tests/grocery_ai.rs`
/// `grocery_time_trial_report`. Author just under the best kind, then the
/// Jungle's steps of 7, 10 and 15 s.
pub const GROCERY_AUTHOR: u32 = 9_300;
pub const GROCERY_GOLD: u32 = 10_000;
pub const GROCERY_SILVER: u32 = 11_000;
pub const GROCERY_BRONZE: u32 = 12_500;

/// Aisle Ape's target times, in race ticks, slowest first.
pub const GROCERY: [u32; 4] = [
    secs(GROCERY_BRONZE),
    secs(GROCERY_SILVER),
    secs(GROCERY_GOLD),
    secs(GROCERY_AUTHOR),
];

/// A map's target times, in race ticks, slowest first (Bronze, Silver,
/// Gold, Author).
pub fn medals(map: Map) -> [u32; 4] {
    match map {
        Map::Jungle => JUNGLE,
        Map::City => CITY,
        Map::Ski => SKI,
        Map::Sector => SECTOR,
        Map::Ruins => RUINS,
        Map::Bay => BAY,
        Map::Tower => TOWER,
        Map::Ravine => RAVINE,
        Map::Grocery => GROCERY,
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
        for map in Map::ALL.into_iter().filter(|&m| m != Map::Jungle) {
            assert_ne!(medals(map), JUNGLE, "{map:?} races the Jungle's medals");
        }
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
