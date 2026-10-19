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
    /// Concrete Jungle's tourist pieces
    Tourist,
    /// The tower map's wizard and witch outfits
    Wizard,
    /// Rattlesnake Ravine's Wild West outfits
    Cowboy,
    /// Aisle Ape's store uniforms
    Grocer,
    /// The Spindle's mech armour
    Mech,
    /// Aisle Ape's second outfit, Grocery Run (GR-T11): everyday fits for
    /// doing the groceries
    Errands,
}

impl Costume {
    /// The selector's order: the map's own, none, then the unlockable ones
    /// in the maps' order.
    pub const ALL: [Costume; 12] = [
        Costume::Own,
        Costume::Bare,
        Costume::Ski,
        Costume::Space,
        Costume::Explorer,
        Costume::Pirate,
        Costume::Tourist,
        Costume::Wizard,
        Costume::Cowboy,
        Costume::Grocer,
        Costume::Mech,
        Costume::Errands,
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
            Costume::Tourist => "tourist",
            Costume::Wizard => "wizard",
            Costume::Cowboy => "cowboy",
            Costume::Grocer => "grocer",
            Costume::Mech => "mech",
            Costume::Errands => "errands",
        }
    }

    pub fn from_key(key: &str) -> Option<Costume> {
        Costume::ALL.into_iter().find(|c| c.key() == key)
    }

    pub fn is_own(&self) -> bool {
        *self == Costume::Own
    }

    /// Worn without unlocking: the map's own and none. Every other costume
    /// is unlocked by its map's Author time (`map`).
    pub fn free(self) -> bool {
        matches!(self, Costume::Own | Costume::Bare)
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
            Gear::Tourist => Some(Costume::Tourist),
            Gear::Wizard => Some(Costume::Wizard),
            Gear::Cowboy => Some(Costume::Cowboy),
            Gear::Grocer => Some(Costume::Grocer),
            Gear::Mech => Some(Costume::Mech),
            // (no map's own gear: Grocery Run is Aisle Ape's second outfit)
            Gear::Errands => Some(Costume::Errands),
        }
    }

    /// What an ape wearing it is drawn in on `map`.
    pub fn gear(self, map: Map) -> Gear {
        match self {
            Costume::Own => Gear::on(map),
            Costume::Bare => Gear::None,
            Costume::Ski => Gear::Ski,
            Costume::Space => Gear::Space,
            Costume::Explorer => Gear::Explorer,
            Costume::Pirate => Gear::Pirate,
            Costume::Tourist => Gear::Tourist,
            Costume::Wizard => Gear::Wizard,
            Costume::Cowboy => Gear::Cowboy,
            Costume::Grocer => Gear::Grocer,
            Costume::Mech => Gear::Mech,
            Costume::Errands => Gear::Errands,
        }
    }

    /// The costume a finish in `ticks` on `map` unlocks: the map's own,
    /// when the finish is within its Author time.
    pub fn earned(map: Map, ticks: u32) -> Option<Costume> {
        Costume::of_map(map).filter(|_| trial::earned_on(map, ticks) == Some(Medal::Author))
    }
}

impl Costume {
    /// Whether a finish in `ticks` on `map` earns Grocery Run (GR-T11): on
    /// Aisle Ape, within its Author time, with every item on the shopping
    /// list (`full_list`) in the same race.
    pub fn errands_earned(map: Map, ticks: u32, full_list: bool) -> bool {
        map == Map::Grocery && full_list && trial::earned_on(map, ticks) == Some(Medal::Author)
    }

    /// Whether it may be worn without unlocking in an online lobby on
    /// `map` (Sam, GR-T11: "you can select between costumes in a dropdown
    /// online when you play aisle ape, and is only selectable on other maps
    /// if you've unlocked it"): Grocery Run on Aisle Ape.
    pub fn lent_on(self, map: Map) -> bool {
        self == Costume::Errands && map == Map::Grocery
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
        assert_eq!(Costume::of_map(Map::City), Some(Costume::Tourist));
    }

    #[test]
    fn the_wizard_costume_is_the_wizard_gear_and_comes_from_its_map() {
        // The tower map (briefs/mapes-tower.md): its costume is "Wizard",
        // the gear's key round trips, and every map with the wizard gear
        // unlocks it (the tower, once `Gear::on(Map::Tower)` is wired)
        assert_eq!(Costume::from_key("wizard"), Some(Costume::Wizard));
        assert_eq!(Gear::from_key(Costume::Wizard.key()), Some(Gear::Wizard));
        for map in Map::ALL {
            assert_eq!(
                Costume::of_map(map) == Some(Costume::Wizard),
                Gear::on(map) == Gear::Wizard,
                "{map:?}"
            );
            assert_eq!(Costume::Wizard.gear(map), Gear::Wizard);
        }
        assert_eq!(
            Costume::Wizard.map().map(Gear::on).unwrap_or(Gear::Wizard),
            Gear::Wizard
        );
        // every unlockable costume wears its own home map's gear
        for c in Costume::ALL {
            if let Some(home) = c.map() {
                assert_eq!(c.gear(Map::Jungle), Gear::on(home), "{c:?}");
            }
        }
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
            assert_eq!(Costume::Tourist.gear(map), Gear::Tourist);
            assert_eq!(Costume::Wizard.gear(map), Gear::Wizard);
            assert_eq!(Costume::Cowboy.gear(map), Gear::Cowboy);
            assert_eq!(Costume::Grocer.gear(map), Gear::Grocer);
            assert_eq!(Costume::Mech.gear(map), Gear::Mech);
        }
        for c in Costume::ALL {
            assert_eq!(Costume::from_key(c.key()), Some(c));
        }
        // (the cowboy's map, Rattlesnake Ravine, is wired by its skeleton;
        // Grocery Run is unlocked on Aisle Ape by its own rule, GR-T11)
        for c in Costume::ALL {
            assert_eq!(
                c.free(),
                c.map().is_none()
                    && !matches!(c, Costume::Wizard | Costume::Cowboy | Costume::Errands),
                "{c:?}"
            );
        }
        assert_eq!(Costume::Own.map(), None);
        assert_eq!(Costume::Bare.map(), None);
        assert_eq!(Costume::Pirate.map(), Some(Map::Bay));
        assert_eq!(Costume::Tourist.map(), Some(Map::City));
    }
}
