//! The maps a race can be run on (the City, 2026-10-12). Each map is one
//! course; a race, a Time Trial and an online lobby pick one. Bests, medals
//! and replays are kept per map (by the course's `name`, which is the
//! map's key).

use crate::course::Course;
use crate::fx::Dice;
use serde::{Deserialize, Serialize};

/// Random's key among the maps' on the map choice (TRIAGE T64), and
/// `map.random` in data/strings.json.
pub const RANDOM_MAP: &str = "random";
/// Random's picture on the map choice: a question mark over the maps'
/// colours.
pub const RANDOM_PICTURE: &str = "data/maps/random.svg";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Map {
    #[default]
    Jungle,
    City,
    Ski,
    Sector,
    Ruins,
}

impl Map {
    pub const ALL: [Map; 5] = [Map::Jungle, Map::City, Map::Ski, Map::Sector, Map::Ruins];

    /// The map's key: the course's `name`, the replay library's and the
    /// settings' word for it, and `map.<key>` in data/strings.json.
    pub fn key(self) -> &'static str {
        match self {
            Map::Jungle => "jungle",
            Map::City => "city",
            Map::Ski => "ski",
            Map::Sector => "sector",
            Map::Ruins => "ruins",
        }
    }

    /// The map's picture on the setup's map grid (TRIAGE T54): a still of
    /// the map shipped in `data/maps/<key>.webp`, made from the built game
    /// by `scripts/map_cards.py`. A new map needs only its `ALL` entry and
    /// this file (`every_map_has_its_picture` in `tests/map_pictures.rs`).
    pub fn picture(self) -> String {
        format!("data/maps/{}.webp", self.key())
    }

    /// A map for a race on Random (TRIAGE T64), from a seed the caller
    /// chose (the core has no dice of its own: BUILDER-SPEC D2): any map but
    /// `last`, so Random never runs the same map twice in a row.
    pub fn random(seed: u64, last: Map) -> Map {
        let others: Vec<Map> = Map::ALL.into_iter().filter(|m| *m != last).collect();
        others[Dice(seed).below(others.len() as u32) as usize]
    }

    pub fn from_key(key: &str) -> Option<Map> {
        Map::ALL.into_iter().find(|m| m.key() == key)
    }

    /// The map's course, laid out from its fixed seed.
    pub fn course(self) -> Course {
        match self {
            Map::Jungle => Course::jungle(),
            Map::City => crate::city::city(),
            Map::Ski => crate::ski::ski(),
            Map::Sector => crate::sector::sector(),
            Map::Ruins => crate::ruins::ruins(),
        }
    }

    /// The map a course was built for.
    pub fn of(course: &Course) -> Map {
        Map::from_key(course.name).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_map_builds_a_course_named_by_its_key() {
        for m in Map::ALL {
            let c = m.course();
            assert_eq!(c.name, m.key());
            assert_eq!(Map::of(&c), m);
            assert_eq!(Map::from_key(m.key()), Some(m));
        }
    }

    #[test]
    fn random_picks_every_other_map_and_never_the_last() {
        for last in Map::ALL {
            let mut seen = Vec::new();
            for seed in 0..200u64 {
                let m = Map::random(seed, last);
                assert_ne!(m, last, "seed {seed} repeated {last:?}");
                if !seen.contains(&m) {
                    seen.push(m);
                }
            }
            assert_eq!(seen.len(), Map::ALL.len() - 1, "after {last:?}: {seen:?}");
            assert_eq!(
                Map::random(7, last),
                Map::random(7, last),
                "the same seed, the same map"
            );
        }
    }
}
