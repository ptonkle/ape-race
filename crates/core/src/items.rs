//! Items (PLAN P9): crates, pickups, the Banana, Stick, Hook and Speed Boost.
//! Placeholder until the items work lands: crates give nothing yet.

use crate::ape::Controls;
use crate::race::Race;

/// Ape `who` pressed the item key.
pub fn use_item(race: &mut Race, who: usize, c: &Controls) {
    let _ = (race, who, c);
}

/// Crates, pickups and thrown things, once a tick after the apes move.
pub fn step(race: &mut Race) {
    let _ = race;
}
