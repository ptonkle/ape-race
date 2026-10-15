//! Unlockable costumes (TRIAGE T89): finish a race on a map under its
//! Author time, in any mode (Time Trial, a race against AIs, with or
//! without items, online), and that map's costume (`models::Gear::on`) is
//! unlocked. A player then wears it on any ape, on any map, chosen on Race
//! Setup and in the online lobby beside the colours (T49).
//!
//! `Own` (the default) is today's behaviour: each map dresses the apes in
//! its own gear. `Bare` wears nothing anywhere. The jaguar's gear stays
//! per map; only the apes change.

use serde::{Deserialize, Serialize};

use crate::map::Map;
use crate::models::Gear;
use crate::trial::{self, Medal};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Costume {
    /// each map's own gear (today's behaviour)
    #[default]
    Own,
    /// nothing, on every map
    Bare,
    /// Yeti Peak's skis, ski helmet and goggles
    Ski,
    /// Sector Ape's flight jacket, metal limb and sunglasses
    Space,
    /// Sunset Ruins' explorer outfit
    Explorer,
    /// Banana Bay Battle's pirate pieces
    Pirate,
}

impl Costume {
    /// The selector's order: the map's own, none, then the unlockable ones
    /// in the maps' order.
    pub const ALL: [Costume; 6] = [
        Costume::Own,
        Costume::Bare,
        Costume::Ski,
        Costume::Space,
        Costume::Explorer,
        Costume::Pirate,
    ];

    /// The key: the settings' and the messages' word, and `costume.<key>`
    /// in data/strings.json.
    pub fn key(self) -> &'static str {
        match self {
            Costume::Own => "own",
            Costume::Bare => "none",
            Costume::Ski => "ski",
            Costume::Space => "space",
            Costume::Explorer => "explorer",
            Costume::Pirate => "pirate",
        }
    }

    pub fn from_key(key: &str) -> Option<Costume> {
        Costume::ALL.into_iter().find(|c| c.key() == key)
    }

    pub fn is_own(&self) -> bool {
        *self == Costume::Own
    }

    /// The map whose Author time unlocks it (None: always worn freely).
    pub fn map(self) -> Option<Map> {
        Map::ALL
            .into_iter()
            .find(|m| Costume::of_map(*m) == Some(self))
    }

    /// The costume a map's Author time unlocks (None: a map with no gear).
    pub fn of_map(map: Map) -> Option<Costume> {
        match Gear::on(map) {
            Gear::None => None,
            Gear::Ski => Some(Costume::Ski),
            Gear::Space => Some(Costume::Space),
            Gear::Explorer => Some(Costume::Explorer),
            Gear::Pirate => Some(Costume::Pirate),
        }
    }

    /// What an ape wearing it is drawn in on `map`.
    pub fn gear(self, map: Map) -> Gear {
        match self {
            Costume::Own => Gear::on(map),
            Costume::Bare => Gear::None,
            c => match c.map() {
                Some(home) => Gear::on(home),
                None => Gear::None,
            },
        }
    }

    /// The costume a finish in `ticks` on `map` unlocks: the map's own,
    /// when the finish is within its Author time.
    pub fn earned(map: Map, ticks: u32) -> Option<Costume> {
        Costume::of_map(map).filter(|_| trial::earned_on(map, ticks) == Some(Medal::Author))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_maps_costume_unlocks_only_within_its_author_time() {
        for map in Map::ALL {
            let author = trial::target_on(map, Medal::Author);
            let c = Costume::of_map(map);
            assert_eq!(Costume::earned(map, author), c, "{map:?}");
            assert_eq!(Costume::earned(map, author - 1), c, "{map:?}");
            assert_eq!(Costume::earned(map, author + 1), None, "{map:?}");
        }
        // the maps with gear each unlock their own; the others none
        assert_eq!(Costume::of_map(Map::Ski), Some(Costume::Ski));
        assert_eq!(Costume::of_map(Map::Sector), Some(Costume::Space));
        assert_eq!(Costume::of_map(Map::Ruins), Some(Costume::Explorer));
        assert_eq!(Costume::of_map(Map::Bay), Some(Costume::Pirate));
        assert_eq!(Costume::of_map(Map::Jungle), None);
        assert_eq!(Costume::of_map(Map::City), None);
    }

    #[test]
    fn a_costume_dresses_the_ape_on_every_map() {
        for map in Map::ALL {
            assert_eq!(Costume::Own.gear(map), Gear::on(map));
            assert_eq!(Costume::Bare.gear(map), Gear::None);
            assert_eq!(Costume::Pirate.gear(map), Gear::Pirate);
            assert_eq!(Costume::Ski.gear(map), Gear::Ski);
            assert_eq!(Costume::Space.gear(map), Gear::Space);
            assert_eq!(Costume::Explorer.gear(map), Gear::Explorer);
        }
        for c in Costume::ALL {
            assert_eq!(Costume::from_key(c.key()), Some(c));
        }
        assert_eq!(Costume::Own.map(), None);
        assert_eq!(Costume::Bare.map(), None);
        assert_eq!(Costume::Pirate.map(), Some(Map::Bay));
    }
}
