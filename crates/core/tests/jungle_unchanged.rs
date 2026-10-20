//! The City (2026-10-12) must leave the Jungle exactly as it was: its
//! layout, its physics and its AI. A race of four AI apes on the Jungle is
//! hashed tick by tick (every ape's place and velocity, and the finishes);
//! the hash was taken before the City's course, water and corner work, so a
//! Jungle replay plays back as it did.

use tool_core::ai::{think, Brain, Difficulty};
use tool_core::course::Course;
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::replay::fingerprint;
use tool_core::rules::{Kind, TICKS_PER_SECOND};

/// The AI levels of a race's four seats.
const SEATS: [Difficulty; 4] = [
    Difficulty::Sloth,
    Difficulty::Sloth,
    Difficulty::Ape,
    Difficulty::KingKong,
];

fn mix(h: &mut u64, v: i64) {
    *h ^= v as u64;
    *h = h.wrapping_mul(0x0000_0100_0000_01B3);
}

fn jungle_race_hash() -> u64 {
    let kinds = [Kind::Gibbon, Kind::Gorilla, Kind::Lemur, Kind::Chimpanzee];
    let entries: Vec<Entry> = kinds
        .iter()
        .map(|&kind| Entry {
            name: String::new(),
            kind,
            driver: Driver::Ai,
            palette: 0,
            costume: tool_core::costume::Costume::Own,
        })
        .collect();
    let mut race = Race::new(Course::jungle(), &entries, false, 7);
    // four seats, three levels: the Monkey's seat (removed, DM-4) is a Sloth's
    let mut brains: Vec<Brain> = SEATS
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

#[test]
fn the_jungle_layout_is_unchanged() {
    let c = Course::jungle();
    assert_eq!(fingerprint(&c), JUNGLE_FINGERPRINT);
    assert_eq!(c.mushrooms.len(), JUNGLE_MUSHROOMS);
    assert!(c.city.is_none());
}

#[test]
fn a_jungle_race_plays_out_exactly_as_before_the_city() {
    assert_eq!(jungle_race_hash(), JUNGLE_RACE);
}

const JUNGLE_FINGERPRINT: u32 = 3_788_111_273;
const JUNGLE_MUSHROOMS: usize = 22;
/// (retaken 2026-10-16 for TRIAGE T90: a caught Hook tows its user and no
/// longer pulls the hooked ape back, a deliberate item change)
/// (retaken 2026-10-17 for 1.14.4: the Monkey level was removed, so the
/// race's second seat is a Sloth, SECOND-ORDER DM-4)
/// (retaken 2026-10-19 for 1.19.1: the Hook pulls much harder and lets go
/// once past the hooked ape, a deliberate item change)
const JUNGLE_RACE: u64 = 17_973_973_468_558_708_902;
