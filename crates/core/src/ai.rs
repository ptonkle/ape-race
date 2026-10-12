//! The AI (PLAN P12): one driver per AI ape, using the same controls as a
//! player. Placeholder until the AI work lands: it holds W.

use crate::ape::Controls;
use crate::race::Race;

/// What an AI remembers between ticks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Brain {
    pub target: Option<usize>,
}

/// This tick's controls for AI ape `who`.
pub fn think(race: &Race, who: usize, brain: &mut Brain) -> Controls {
    let _ = (race, who, brain);
    Controls {
        fwd: 1,
        ..Controls::default()
    }
}
