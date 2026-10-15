//! Sector Ape (2026-10-13) must leave the City and the Ski Slope exactly as
//! they were, as `jungle_unchanged` holds the Jungle: each map's layout
//! fingerprint, and a race of four AI apes (one at each level, items on)
//! hashed tick by tick, plain and with Floor is Lava. The hashes were taken
//! on main before any Sector Ape physics, so a saved replay of either map
//! plays back with the same finish time.

use tool_core::ai::{think, Brain, Difficulty};
use tool_core::course::Course;
use tool_core::map::Map;
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::replay::fingerprint;
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn mix(h: &mut u64, v: i64) {
    *h ^= v as u64;
    *h = h.wrapping_mul(0x0000_0100_0000_01B3);
}

fn race_hash(course: Course, lava: bool) -> u64 {
    let kinds = [Kind::Gibbon, Kind::Gorilla, Kind::Lemur, Kind::Chimpanzee];
    let entries: Vec<Entry> = kinds
        .iter()
        .map(|&kind| Entry {
            name: String::new(),
            kind,
            driver: Driver::Ai,
            palette: 0,
        })
        .collect();
    let mut race = Race::new(course, &entries, false, 7);
    race.lava = lava;
    let mut brains: Vec<Brain> = Difficulty::ALL
        .iter()
        .enumerate()
        .map(|(i, &d)| Brain::new(d, 0xA9E5 + i as u64))
        .collect();
    let mut h = 0xCBF2_9CE4_8422_2325u64;
    let limit = 150 * TICKS_PER_SECOND as u32;
    while race.clock < limit && race.phase != Phase::Done {
        let c: Vec<_> = (0..4).map(|i| think(&race, i, &mut brains[i])).collect();
        race.step(&c);
        for r in &race.racers {
            for v in [
                r.ape.pos.x,
                r.ape.pos.y,
                r.ape.pos.z,
                r.ape.vel.x,
                r.ape.vel.y,
                r.ape.vel.z,
            ] {
                mix(&mut h, v as i64);
            }
        }
    }
    for r in &race.racers {
        mix(&mut h, r.finish.map_or(-1, |f| f as i64));
    }
    h
}

fn check(map: Map, layout: u32, plain: u64, lava: u64) {
    let c = map.course();
    let got = (
        fingerprint(&c),
        race_hash(c.clone(), false),
        race_hash(c, true),
    );
    assert_eq!(got, (layout, plain, lava), "{map:?} changed");
}

#[test]
fn the_city_plays_out_exactly_as_before_sector_ape() {
    check(Map::City, CITY.0, CITY.1, CITY.2);
}

#[test]
fn the_ski_slope_plays_out_exactly_as_before_sector_ape() {
    check(Map::Ski, SKI.0, SKI.1, SKI.2);
}

#[test]
fn the_jungle_with_floor_is_lava_plays_out_as_before_sector_ape() {
    assert_eq!(race_hash(Map::Jungle.course(), true), JUNGLE_LAVA);
}

const CITY: (u32, u64, u64) = (
    3_200_548_462,
    11_378_391_952_088_175_385,
    16_180_813_438_469_971_061,
);
/// The Ski Slope's, retaken 2026-10-14 for TRIAGE T74 (low vines at
/// every station, two across the whole run): a deliberate layout change,
/// so a Ski Slope replay saved before it no longer plays back the same.
/// The City's and the Jungle's are untouched.
const SKI: (u32, u64, u64) = (
    266_190_231,
    16_287_703_392_448_572_623,
    6_905_027_659_217_288_284,
);
const JUNGLE_LAVA: u64 = 17_624_450_600_069_762_721;
