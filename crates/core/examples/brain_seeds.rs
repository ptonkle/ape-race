//! One map's real layout, one level, every kind (own stats), over several
//! brain seeds: how reliably the level finishes the map as a race runs it.
//!
//! `cargo run --release -p tool_core --example brain_seeds -- MAP LEVEL [SEEDS]`
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::fx::ONE;
use tool_core::map::Map;
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let map = Map::from_key(&a[0]).unwrap();
    let level = Difficulty::from_key(&a[1]).unwrap();
    let seeds: u64 = a.get(2).map(|s| s.parse().unwrap()).unwrap_or(8);
    for kind in Kind::ALL {
        let mut row = Vec::new();
        let mut sum = 0.0;
        for s in 0..seeds {
            let entries = [Entry {
                name: String::new(),
                kind,
                driver: Driver::Ai,
                palette: 0,
            }];
            let mut race = Race::new(map.course(), &entries, false, 7 + s);
            let mut brain = Brain::new(level, 0xA9E5 + s * 101);
            while race.clock < 150 * TICKS_PER_SECOND as u32 && race.phase != Phase::Done {
                let c = think(&race, 0, &mut brain);
                race.step(&[c]);
            }
            let r = &race.racers[0];
            sum +=
                r.finish.unwrap_or(150 * TICKS_PER_SECOND as u32) as f64 / TICKS_PER_SECOND as f64;
            row.push(match r.finish {
                Some(f) => format!("{:.0}", f as f64 / TICKS_PER_SECOND as f64),
                None => format!("DNF@{}", r.progress / ONE),
            });
        }
        println!(
            "{:<11} mean {:>5.1}  {}",
            format!("{kind:?}"),
            sum / seeds as f64,
            row.join(" ")
        );
    }
}
