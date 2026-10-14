//! What one AI's thinking costs a tick on Sector Ape (or another map): a
//! race of four AIs (one per level), each `think` timed, the worst ticks
//! listed with where the ape was and what it was doing, and the step.
//!
//! `cargo run --release -p tool_core --example sector_think_cost [-- MAP]`
use std::time::{Duration, Instant};
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::fx::ONE;
use tool_core::map::Map;
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let map = std::env::args()
        .nth(1)
        .and_then(|k| Map::from_key(&k))
        .unwrap_or(Map::Sector);
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
    let mut race = Race::new(map.course(), &entries, false, 7);
    let mut brains: Vec<Brain> = Difficulty::ALL
        .iter()
        .enumerate()
        .map(|(i, &d)| Brain::new(d, 0xA9E5 + i as u64))
        .collect();
    let mut worst: Vec<(Duration, String)> = Vec::new();
    let mut tick_worst = Duration::ZERO;
    let mut total = Duration::ZERO;
    let mut ticks = 0u32;
    while race.clock < 150 * TICKS_PER_SECOND as u32 && race.phase != Phase::Done {
        let t0 = Instant::now();
        let mut c = Vec::new();
        for i in 0..4 {
            let t = Instant::now();
            c.push(think(&race, i, &mut brains[i]));
            let e = t.elapsed();
            let a = &race.racers[i].ape;
            worst.push((
                e,
                format!(
                    "tick {} ape {i} ({}) at {} m y {} v {} {:?}",
                    race.tick,
                    brains[i].difficulty.key(),
                    race.racers[i].progress / ONE,
                    a.pos.y / ONE,
                    a.speed() / ONE,
                    a.mode
                ),
            ));
        }
        race.step(&c);
        let e = t0.elapsed();
        total += e;
        ticks += 1;
        tick_worst = tick_worst.max(e);
        worst.sort_by(|a, b| b.0.cmp(&a.0));
        worst.truncate(12);
    }
    for (d, what) in &worst {
        println!("{d:>10.2?}  {what}");
    }
    println!(
        "worst tick {tick_worst:.2?}, mean {:.2?} over {ticks} ticks (two ticks a frame)",
        total / ticks.max(1)
    );
}
