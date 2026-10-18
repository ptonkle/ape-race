//! The AI on Clean-Up on Aisle Ape's shopping list (GR-T2, GR-T3): races
//! of four AI apes (one at each item spot) by level over several layouts,
//! each ape's finish time and how many of its four items it took.
//!
//! `cargo run --release -p tool_core --example grocery_list_report [-- LAYOUTS]`
use tool_core::ai::{trial_at, Difficulty};
use tool_core::fx::ONE;
use tool_core::grocery::{grocery, grocery_seeded};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let layouts: u64 = std::env::args()
        .nth(1)
        .map(|a| a.parse().unwrap())
        .unwrap_or(1);
    let kinds = [
        Kind::Chimpanzee,
        Kind::Gorilla,
        Kind::Orangutan,
        Kind::Lemur,
    ];
    let limit = 180 * TICKS_PER_SECOND as u32;
    let rows: Vec<_> = Difficulty::ALL
        .into_iter()
        .flat_map(|level| (0..layouts).map(move |k| (level, k)))
        .map(|(level, k)| {
            std::thread::spawn(move || {
                let course = match k {
                    0 => grocery(),
                    _ => grocery_seeded(0x6E0C_E2E5_0001 + k * 7919),
                };
                (level, k, trial_at(course, &kinds, &[level; 4], true, limit))
            })
        })
        .collect();
    let mut sums: Vec<(Difficulty, f64, u32, u32, u32)> = Vec::new();
    for r in rows {
        let (level, k, t) = r.join().unwrap();
        let cells: Vec<String> = t
            .iter()
            .map(|r| {
                let items = r.list.count_ones();
                match r.finish {
                    Some(f) => format!("{:.1}s/{items}", f as f64 / TICKS_PER_SECOND as f64),
                    None => format!("DNF@{}m/{items}", r.progress / ONE),
                }
            })
            .collect();
        println!("{:<10} layout {k}: {}", level.key(), cells.join("  "));
        let done: Vec<f64> = t
            .iter()
            .filter_map(|r| r.finish.map(|f| f as f64 / TICKS_PER_SECOND as f64))
            .collect();
        let items: u32 = t.iter().map(|r| r.list.count_ones()).sum();
        match sums.iter_mut().find(|s| s.0 == level) {
            Some(s) => {
                s.1 += done.iter().sum::<f64>();
                s.2 += done.len() as u32;
                s.3 += items;
                s.4 += t.len() as u32;
            }
            None => sums.push((
                level,
                done.iter().sum(),
                done.len() as u32,
                items,
                t.len() as u32,
            )),
        }
    }
    for (level, sum, n, items, apes) in sums {
        println!(
            "{:<10} finished {n}/{apes}, mean {:.1} s, items {items}/{}",
            level.key(),
            sum / n.max(1) as f64,
            apes * 4
        );
    }
}
