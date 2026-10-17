//! The AI on Rattlesnake Ravine: finish times by difficulty over several
//! layouts, with means and the gaps between the levels. With `upper` the
//! course's spine is the upper passage (`ravine_upper_seeded`).
//!
//! `cargo run --release -p tool_core --example ravine_report [-- SEEDS [KIND] [upper|jungle]]` (`jungle`: the Jungle's layout from the same seeds, to compare`
use tool_core::ai::{trial_at, Difficulty};
use tool_core::fx::ONE;
use tool_core::ravine::{ravine, ravine_seeded, ravine_upper_seeded};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seeds: u64 = args.get(1).map(|a| a.parse().unwrap()).unwrap_or(1);
    let kind = args
        .get(2)
        .and_then(|k| Kind::from_key(k))
        .unwrap_or(Kind::Chimpanzee);
    let upper = args.iter().any(|a| a == "upper");
    let jungle = args.iter().any(|a| a == "jungle");
    let limit = 150 * TICKS_PER_SECOND as u32;
    let rows: Vec<_> = Difficulty::ALL
        .into_iter()
        .map(|level| {
            std::thread::spawn(move || (level, row_for(level, kind, seeds, limit, upper, jungle)))
        })
        .collect();
    let mut means = Vec::new();
    for r in rows {
        let (level, row) = r.join().unwrap();
        let done: Vec<f64> = row.iter().filter_map(|r| r.parse().ok()).collect();
        let mean = done.iter().sum::<f64>() / done.len().max(1) as f64;
        means.push(mean);
        println!("{:<10} {}  mean {:.1}", level.key(), row.join(" "), mean);
    }
    let gaps: Vec<String> = means
        .windows(2)
        .map(|w| format!("{:.1}", w[0] - w[1]))
        .collect();
    println!(
        "gaps between the means (easier minus harder) {}",
        gaps.join(" ")
    );
}

/// The layout for seed `k` (0 is the course everyone races).
fn course_for(k: u64, upper: bool, jungle: bool) -> tool_core::course::Course {
    let seed = 0x5A4E_C0DE_0001 + k * 7919;
    if jungle {
        return tool_core::course::Course::jungle_seeded(seed);
    }
    match (k, upper) {
        (0, false) => ravine(),
        (_, false) => ravine_seeded(seed),
        (_, true) => ravine_upper_seeded(seed),
    }
}

fn row_for(
    level: Difficulty,
    kind: Kind,
    seeds: u64,
    limit: u32,
    upper: bool,
    jungle: bool,
) -> Vec<String> {
    let mut row = Vec::new();
    for k in 0..seeds {
        let t = trial_at(
            course_for(k, upper, jungle),
            &[kind],
            &[level],
            kind == Kind::Chimpanzee,
            limit,
        );
        row.push(match t[0].finish {
            Some(f) => format!("{:.1}", f as f64 / TICKS_PER_SECOND as f64),
            None => format!("DNF@{}m", t[0].progress / ONE),
        });
    }
    row
}
