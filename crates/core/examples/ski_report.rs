//! The Ski Slope's layout in numbers, and the AI on it: finish times by
//! difficulty, and where each ape got stuck if it did.
//!
//! `cargo run --release -p tool_core --example ski_report [-- SEEDS]`
use tool_core::ai::{trial_at, Difficulty};
use tool_core::fx::ONE;
use tool_core::rules::{Kind, TICKS_PER_SECOND};
use tool_core::ski::{ski, ski_seeded};

fn main() {
    let seeds: u64 = std::env::args()
        .nth(1)
        .map(|a| a.parse().unwrap())
        .unwrap_or(1);
    let c = ski();
    let lay = c.ski.as_ref().unwrap();
    println!(
        "length {} m, drop {} m, vines {}, lifts {}, trees {}, hangers {}, landmarks {}, bananas {}, crates {}",
        c.length / ONE,
        -c.terrain(c.path[c.path.len() - 1].x, c.path[c.path.len() - 1].z) / ONE,
        c.vines.len(),
        lay.lifts.len(),
        lay.trees.len(),
        lay.hangers.len(),
        lay.landmarks.len(),
        c.bananas.len(),
        c.crates.len()
    );
    let mut kinds = std::collections::BTreeMap::new();
    for h in &lay.hangers {
        *kinds
            .entry(
                format!("{:?}", h.kind)
                    .split(' ')
                    .next()
                    .unwrap()
                    .to_string(),
            )
            .or_insert(0) += 1;
    }
    println!("hangers {kinds:?}");
    let limit = 150 * TICKS_PER_SECOND as u32;
    let rows: Vec<_> = Difficulty::ALL
        .into_iter()
        .map(|level| std::thread::spawn(move || (level, row_for(level, seeds, limit))))
        .collect();
    for r in rows {
        let (level, row) = r.join().unwrap();
        let done: Vec<f64> = row.iter().filter_map(|r| r.parse().ok()).collect();
        println!(
            "{:<10} {}  mean {:.1}",
            level.key(),
            row.join(" "),
            done.iter().sum::<f64>() / done.len().max(1) as f64
        );
    }
}

fn row_for(level: Difficulty, seeds: u64, limit: u32) -> Vec<String> {
    let mut row = Vec::new();
    for k in 0..seeds {
        // (SKI_REPORT_JUNGLE=1: the Jungle's layouts, for comparison)
        let course = if std::env::var("SKI_REPORT_JUNGLE").is_ok() {
            tool_core::course::Course::jungle_seeded(0x0A9E_2ACE_0001 + k * 7919)
        } else if k == 0 {
            ski()
        } else {
            ski_seeded(0x5C1_5EED_0001 + k * 7919)
        };
        let t = trial_at(course, &[Kind::Chimpanzee], &[level], true, limit);
        row.push(match t[0].finish {
            Some(f) => format!("{:.1}", f as f64 / TICKS_PER_SECOND as f64),
            None => format!("DNF@{}m", t[0].progress / ONE),
        });
    }
    row
}
