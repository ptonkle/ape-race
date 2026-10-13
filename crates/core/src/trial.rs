//! Time Trial (TRIAGE T1): one ape alone on the course, Speed Boosts only
//! (T13), against target times in the Trackmania manner: Bronze, Silver,
//! Gold, and a hidden Author time that shows once Gold is earned.
//!
//! The target times are the four constants below. Re-tune them whenever
//! the course or the physics changes: run
//! `cargo test --release -p tool_core --test ai -- --ignored --nocapture`
//! and set the Author time to about the AI's time alone, the others in
//! generous steps above it.

use serde::Serialize;

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

/// The target of a medal, in race ticks.
pub fn target(m: Medal) -> u32 {
    JUNGLE[m as usize]
}

/// The best medal a finish in `ticks` earns.
pub fn earned(ticks: u32) -> Option<Medal> {
    Medal::ALL
        .iter()
        .rev()
        .copied()
        .find(|m| ticks <= target(*m))
}

/// Whether the Author time may be shown: only once a best has earned Gold.
pub fn author_shown(best: Option<u32>) -> bool {
    best.map(|b| b <= target(Medal::Gold)).unwrap_or(false)
}

/// The next medal to aim for, given the best so far (None: Author earned).
pub fn next_target(best: Option<u32>) -> Option<Medal> {
    match best.and_then(earned) {
        None => Some(Medal::Bronze),
        Some(Medal::Author) => None,
        Some(m) => Medal::ALL.get(m as usize + 1).copied(),
    }
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
