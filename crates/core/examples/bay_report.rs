//! The AI on Banana Bay Battle: finish times by difficulty over several layouts,
//! with means and the gaps between the levels.
//!
//! `cargo run --release -p tool_core --example bay_report [-- SEEDS [KIND]]`
use tool_core::ai::{trial_at, Difficulty};
use tool_core::bay::{bay, bay_seeded};
use tool_core::fx::ONE;
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let seeds: u64 = std::env::args()
        .nth(1)
        .map(|a| a.parse().unwrap())
        .unwrap_or(1);
    let kind = std::env::args()
        .nth(2)
        .and_then(|k| Kind::from_key(&k))
        .unwrap_or(Kind::Chimpanzee);
    let limit = 150 * TICKS_PER_SECOND as u32;
    let rows: Vec<_> = Difficulty::ALL
        .into_iter()
        .map(|level| std::thread::spawn(move || (level, row_for(level, kind, seeds, limit))))
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
fn course_for(k: u64) -> tool_core::course::Course {
    if k == 0 {
        bay()
    } else {
        bay_seeded(0xBA7_BA77_0001 + k * 7919)
    }
}

fn row_for(level: Difficulty, kind: Kind, seeds: u64, limit: u32) -> Vec<String> {
    let mut row = Vec::new();
    for k in 0..seeds {
        let t = trial_at(
            course_for(k),
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
