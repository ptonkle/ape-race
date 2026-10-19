//! The tower map's AI pace: every level (one Chimpanzee, Equal Stats) over
//! `N` layouts (default 16; layout 0 is the course everyone races), mean
//! and max finish. `cargo run --release -p tool_core --example tower_report -- [N]`
use tool_core::ai::{trial_at, Difficulty};
use tool_core::fx::ONE;
use tool_core::rules::{Kind, TICKS_PER_SECOND};
use tool_core::tower::{tower, tower_seeded};

fn main() {
    let n: u64 = std::env::args()
        .nth(1)
        .map(|a| a.parse().unwrap())
        .unwrap_or(16);
    let limit = 200 * TICKS_PER_SECOND as u32;
    println!("length {} m", tower().length / ONE);
    let runs: Vec<_> = Difficulty::ALL
        .into_iter()
        .flat_map(|level| (0..n).map(move |k| (level, k)))
        .map(|(level, k)| {
            std::thread::spawn(move || {
                let c = if k == 0 {
                    tower()
                } else {
                    tower_seeded(0x70E5_0000_0001 + k * 7919)
                };
                let t = trial_at(c, &[Kind::Chimpanzee], &[level], true, limit);
                (level, k, t[0].finish, t[0].progress / ONE)
            })
        })
        .collect();
    let res: Vec<_> = runs.into_iter().map(|r| r.join().unwrap()).collect();
    for level in Difficulty::ALL {
        let row: Vec<f64> = res
            .iter()
            .filter(|r| r.0 == level)
            .map(|r| match r.2 {
                Some(f) => f as f64 / TICKS_PER_SECOND as f64,
                None => {
                    println!("  {} layout {} stuck at {} m", level.key(), r.1, r.3);
                    999.0
                }
            })
            .collect();
        let mean = row.iter().sum::<f64>() / row.len() as f64;
        let max = row.iter().cloned().fold(0.0, f64::max);
        println!(
            "{:<10} mean {mean:6.1} max {max:6.1} {row:.0?}",
            level.key()
        );
    }
}
