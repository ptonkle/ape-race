//! The Time Trial's target times on each map (the City, 2026-10-12). The
//! medal rules are `crate::trial`'s; this only picks a map's four times.
//!
//! The course agent is making medal times per map (`trial::medals(map)`).
//! Until that lands every map races against the Jungle's, and
//! [`of`] is the one place their change slots in.

use crate::map::Map;
use crate::trial::{self, Medal};

/// One map's target times, in race ticks, slowest (Bronze) first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Targets([u32; 4]);

/// A map's target times.
pub(super) fn of(_map: Map) -> Targets {
    Targets(trial::JUNGLE)
}

impl Targets {
    pub(super) fn target(&self, m: Medal) -> u32 {
        self.0[m as usize]
    }

    /// The best medal a finish in `ticks` earns.
    pub(super) fn earned(&self, ticks: u32) -> Option<Medal> {
        Medal::ALL
            .iter()
            .rev()
            .copied()
            .find(|m| ticks <= self.target(*m))
    }

    /// Whether the Author time may be shown: only once a best earned Gold.
    pub(super) fn author_shown(&self, best: Option<u32>) -> bool {
        best.is_some_and(|b| b <= self.target(Medal::Gold))
    }

    /// The next medal to aim for, given the best so far (None: Author earned).
    pub(super) fn next_target(&self, best: Option<u32>) -> Option<Medal> {
        match best.and_then(|b| self.earned(b)) {
            None => Some(Medal::Bronze),
            Some(Medal::Author) => None,
            Some(m) => Medal::ALL.get(m as usize + 1).copied(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_jungles_targets_are_the_trials_and_follow_its_rules() {
        let j = of(Map::Jungle);
        for m in Medal::ALL {
            assert_eq!(j.target(m), trial::target(m));
            assert_eq!(j.earned(j.target(m)), trial::earned(trial::target(m)));
            assert_eq!(
                j.next_target(Some(j.target(m))),
                trial::next_target(Some(trial::target(m)))
            );
            assert_eq!(
                j.author_shown(Some(j.target(m))),
                trial::author_shown(Some(trial::target(m)))
            );
        }
        assert_eq!(j.next_target(None), Some(Medal::Bronze));
        // every map has four targets, harder up to the Author
        for map in Map::ALL {
            let t = of(map);
            for w in Medal::ALL.windows(2) {
                assert!(t.target(w[1]) < t.target(w[0]));
            }
        }
    }
}
