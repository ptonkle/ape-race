//! The step-cost work on Sector Ape (2026-10-14, SP rows in SECOND-ORDER)
//! was to make the AI's copies of the ape cheaper without changing a race:
//! races of four AI apes (one at each level, items on), hashed tick by tick
//! over several seeds, plain and with Floor is Lava, must come out as they
//! did before it. A deliberate change to Sector Ape's physics or AI takes
//! new hashes (say so in SECOND-ORDER); a speed-up never does.

use tool_core::ai::{think, Brain, Difficulty};
use tool_core::map::Map;
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::replay::fingerprint;
use tool_core::rules::{Kind, TICKS_PER_SECOND};

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
        })
        .collect();
    let mut race = Race::new(Map::Sector.course(), &entries, false, seed);
    race.lava = lava;
    let mut brains: Vec<Brain> = Difficulty::ALL
        .iter()
        .enumerate()
        .map(|(i, &d)| Brain::new(d, 0xA9E5 + seed * 7 + i as u64))
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
const SEEDS: [u64; 6] = [
    13_429_847_654_000_318_286,
    4_980_814_365_964_618_372,
    9_956_122_176_243_341_526,
    18_187_769_242_450_866_680,
    1_062_557_235_219_045_084,
    16_876_798_273_135_000_165,
];
const LAVA: [u64; 2] = [15_354_224_398_972_486_079, 9_180_943_159_725_750_702];
