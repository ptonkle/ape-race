//! The maps a race can be run on (the City, 2026-10-12). Each map is one
//! course; a race, a Time Trial and an online lobby pick one. Bests, medals
//! and replays are kept per map (by the course's `name`, which is the
//! map's key).

use crate::course::Course;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Map {
    #[default]
    Jungle,
    City,
    Ski,
}

impl Map {
    pub const ALL: [Map; 3] = [Map::Jungle, Map::City, Map::Ski];

    /// The map's key: the course's `name`, the replay library's and the
    /// settings' word for it, and `map.<key>` in data/strings.json.
    pub fn key(self) -> &'static str {
        match self {
            Map::Jungle => "jungle",
            Map::City => "city",
            Map::Ski => "ski",
        }
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
}
