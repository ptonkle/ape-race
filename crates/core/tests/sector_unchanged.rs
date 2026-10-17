//! The step-cost work on Sector Ape (2026-10-14, SP rows in SECOND-ORDER)
//! was to make the AI's copies of the ape cheaper without changing a race:
//! races of four AI apes (one at each level, items on), hashed tick by tick
//! over several seeds, plain and with Floor is Lava, must come out as they
//! did before it, up to each ape's finish. A deliberate change to Sector Ape's physics or AI takes
//! new hashes (say so in SECOND-ORDER); a speed-up never does. Taken again
//! for T80 (a reset only onto a deck behind, and the Sloth's and Monkey's
//! Sector tuning after it), and for T88 (the wreck passage's launch and
//! wider gap, and the Sloth's and Monkey's tuning after it).

use tool_core::ai::{think, Brain, Difficulty};
use tool_core::map::Map;
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

fn race_hash(seed: u64, lava: bool) -> u64 {
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
    let mut race = Race::new(Map::Sector.course(), &entries, false, seed);
    race.lava = lava;
    // four seats, three levels: the Monkey's seat (removed, DM-4) is a Sloth's
    let mut brains: Vec<Brain> = SEATS
        .iter()
        .enumerate()
        .map(|(i, &d)| Brain::new(d, 0xA9E5 + seed * 7 + i as u64))
        .collect();
    let mut h = 0xCBF2_9CE4_8422_2325u64;
    let limit = 150 * TICKS_PER_SECOND as u32;
    while race.clock < limit && race.phase != Phase::Done {
        let c: Vec<_> = (0..4).map(|i| think(&race, i, &mut brains[i])).collect();
        race.step(&c);
        // (until each finishes: what it does after, braked and drawn to
        // the black hole, is the finish's, SF7)
        for r in race.racers.iter().filter(|r| r.finish.is_none()) {
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
fn sector_ape_plays_out_exactly_as_before_the_speed_ups() {
    assert_eq!(fingerprint(&Map::Sector.course()), LAYOUT, "the layout");
    let runs: Vec<(u64, bool)> = (0..SEEDS.len() as u64)
        .map(|s| (s, false))
        .chain([(0, true), (1, true)])
        .collect();
    let got: Vec<u64> = std::thread::scope(|sc| {
        let hs: Vec<_> = runs
            .iter()
            .map(|&(seed, lava)| sc.spawn(move || race_hash(seed, lava)))
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let want: Vec<u64> = SEEDS.iter().chain(LAVA.iter()).copied().collect();
    assert_eq!(got, want, "Sector Ape's races changed");
}

const LAYOUT: u32 = 2_063_114_272;
/// (seeds 1, 2, 3 and 6 retaken 2026-10-16 for TRIAGE T90: a caught Hook
/// tows its user and no longer pulls the hooked ape back, a deliberate
/// item change)
/// (all retaken 2026-10-17 for 1.14.4: the Monkey level was removed, so the
/// race's second seat is a Sloth, SECOND-ORDER DM-4)
const SEEDS: [u64; 6] = [
    1_852_813_321_290_551_993,
    13_750_823_436_147_438_851,
    8_537_279_962_408_538_083,
    9_335_109_251_823_115_830,
    1_822_746_278_256_832_855,
    9_917_851_283_649_035_894,
];
const LAVA: [u64; 2] = [1_042_971_815_671_394_990, 15_235_268_330_251_624_765];
