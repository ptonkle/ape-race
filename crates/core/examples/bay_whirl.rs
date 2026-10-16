//! Rope whirls on Banana Bay Battle (BT4): for each level and layout, one
//! Chimpanzee (Equal Stats) races alone as in `trial_at`; prints its finish
//! and the longest it held one rope (seconds, which rope, when it grabbed),
//! then each level's mean and max finish and its longest hold.
//!
//! `cargo run --release -p tool_core --example bay_whirl [-- SEEDS]`
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::bay::{bay, bay_seeded};
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

const LIMIT: u32 = 150 * TICKS_PER_SECOND as u32;

fn secs(t: u32) -> f64 {
    t as f64 / TICKS_PER_SECOND as f64
}

/// (finish, longest hold, its rope, the clock at its grab)
fn run(level: Difficulty, k: u64) -> (Option<u32>, u32, usize, u32) {
    let course = if k == 0 {
        bay()
    } else {
        bay_seeded(0xBA7_BA77_0001 + k * 7919)
    };
    let entries = [Entry {
        name: String::new(),
        kind: Kind::Chimpanzee,
        driver: Driver::Ai,
        palette: 0,
        costume: tool_core::costume::Costume::Own,
    }];
    let mut race = Race::new(course, &entries, true, 7);
    let mut brain = Brain::new(level, 0xA9E5);
    let mut held: Option<(usize, u32)> = None;
    let mut worst = (0, 0, 0);
    while race.clock < LIMIT && race.phase != Phase::Done {
        let c = think(&race, 0, &mut brain);
        race.step(&[c]);
        let now = race.racers[0].ape.swinging();
        match (held, now) {
            (Some((v, _)), Some(n)) if v == n => {}
            _ => held = now.map(|n| (n, race.clock)),
        }
        if let Some((v, from)) = held {
            if race.clock - from > worst.0 {
                worst = (race.clock - from, v, from);
            }
        }
    }
    if std::env::var("BAY_WHIRL_WHY").is_ok() {
        eprintln!(
            "{} layout {k}: replans {} give-ups {} rescues {} misses {}",
            level.key(),
            brain.replans,
            brain.give_ups,
            brain.rescues,
            brain.misses
        );
    }
    (race.racers[0].finish, worst.0, worst.1, worst.2)
}

fn main() {
    let seeds: u64 = std::env::args()
        .nth(1)
        .map(|a| a.parse().unwrap())
        .unwrap_or(16);
    let rows: Vec<_> = Difficulty::ALL
        .into_iter()
        .map(|level| {
            std::thread::spawn(move || {
                (level, (0..seeds).map(|k| run(level, k)).collect::<Vec<_>>())
            })
        })
        .collect();
    for r in rows {
        let (level, row) = r.join().unwrap();
        let fin: Vec<f64> = row.iter().filter_map(|r| r.0.map(secs)).collect();
        let mean = fin.iter().sum::<f64>() / fin.len().max(1) as f64;
        let max = fin.iter().cloned().fold(0.0, f64::max);
        let (k, w) = row.iter().enumerate().max_by_key(|(_, r)| r.1).unwrap();
        println!(
            "{:<10} mean {mean:.1} max {max:.1} dnf {}  longest hold {:.1} s (layout {k}, rope {}, at {:.1} s)",
            level.key(),
            row.len() - fin.len(),
            secs(w.1),
            w.2,
            secs(w.3)
        );
        let cells: Vec<String> = row
            .iter()
            .map(|r| {
                format!(
                    "{}/{:.1}",
                    r.0.map_or("DNF".into(), |f| format!("{:.1}", secs(f))),
                    secs(r.1)
                )
            })
            .collect();
        println!("           finish/hold {}", cells.join(" "));
    }
}
