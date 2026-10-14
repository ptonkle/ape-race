//! Sector Ape's finish (SECOND-ORDER SF7): every ape that crosses the line
//! ends up dancing on the finish deck or floating at the black hole's
//! banana horizon, never falling past the deck into empty space (one in
//! four did: one crossing the line low fell under the deck for ever).

use tool_core::ai::{think, Brain, Difficulty};
use tool_core::fx::ONE;
use tool_core::map::Map;
use tool_core::race::{Driver, Entry, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

/// Where each finisher is 10 s after its finish, in races of four AIs.
fn finishers(seed: u64) -> Vec<(usize, Kind, i32, i32)> {
    let kinds = [
        [Kind::Gibbon, Kind::Gorilla, Kind::Lemur, Kind::Chimpanzee],
        [Kind::Orangutan, Kind::Proboscis, Kind::Gibbon, Kind::Lemur],
    ][(seed % 2) as usize];
    let entries: Vec<Entry> = kinds
        .iter()
        .map(|&kind| Entry {
            name: String::new(),
            kind,
            driver: Driver::Ai,
            palette: 0,
        })
        .collect();
    let mut race = Race::new(Map::Sector.course(), &entries, seed % 3 == 0, seed);
    let mut brains: Vec<Brain> = Difficulty::ALL
        .iter()
        .enumerate()
        .map(|(i, &d)| Brain::new(d, 0xF1E5 + seed * 11 + i as u64))
        .collect();
    let tps = TICKS_PER_SECOND as u32;
    let mut out = Vec::new();
    let mut seen = [false; 4];
    // (the race clock stops once all are in: count ticks)
    let mut since: [Option<u32>; 4] = [None; 4];
    while race.tick < 175 * tps {
        let c: Vec<_> = (0..4).map(|i| think(&race, i, &mut brains[i])).collect();
        race.step(&c);
        let s = race.course.sector.as_deref().unwrap();
        let deck = &race.course.platforms[s.finish_deck];
        let (hole, r) = s.black_hole;
        for (i, rc) in race.racers.iter().enumerate() {
            if rc.finish.is_none() {
                continue;
            }
            let f = *since[i].get_or_insert(race.tick);
            if seen[i] || race.tick < f + 10 * tps {
                continue;
            }
            seen[i] = true;
            let feet = rc.ape.feet();
            // how far under the deck's top, and how far from the horizon
            let under = deck.top - feet.y;
            let off_hole = rc.ape.pos.dist(hole) - r * 6 / 5;
            out.push((i, rc.ape.kind, under, off_hole));
        }
    }
    out
}

#[test]
fn every_finisher_ends_on_the_deck_or_at_the_horizon() {
    let runs: Vec<_> = (1..=12u64)
        .map(|seed| std::thread::spawn(move || (seed, finishers(seed))))
        .collect();
    let mut n = 0;
    for r in runs {
        let (seed, fins) = r.join().unwrap();
        for (i, kind, under, off_hole) in fins {
            n += 1;
            // on the deck (feet at its top) or at the horizon
            assert!(
                under <= ONE || off_hole <= int(3),
                "seed {seed} ape {i} ({kind:?}) ended {} m under the finish deck, {} m off the horizon",
                under / ONE,
                off_hole / ONE
            );
        }
    }
    assert!(n >= 30, "only {n} finishes");
}

const fn int(n: i32) -> i32 {
    n * ONE
}
