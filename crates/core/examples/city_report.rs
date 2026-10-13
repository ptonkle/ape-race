//! The City's layout in numbers, and the AI on it: finish times by
//! difficulty, and where each ape got stuck if it did.
//!
//! `cargo run --release -p tool_core --example city_report [-- SEEDS]`
use tool_core::ai::{trial_at, Difficulty};
use tool_core::city::{city, city_seeded};
use tool_core::fx::ONE;
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let seeds: u64 = std::env::args()
        .nth(1)
        .map(|a| a.parse().unwrap())
        .unwrap_or(1);
    let c = city();
    let lay = c.city.as_ref().unwrap();
    println!(
        "length {} m, vines {}, bouncers {}, cars {} ({} moving), boats {}, blocks {}, hangers {}, bananas {}, crates {}",
        c.length / ONE,
        c.vines.len(),
        c.mushrooms.len(),
        lay.cars.len(),
        lay.cars.iter().filter(|k| k.drive.is_some()).count(),
        lay.boats.len(),
        lay.blocks.len(),
        lay.hangers.len(),
        c.bananas.len(),
        c.crates.len()
    );
    let mut kinds = std::collections::BTreeMap::new();
    for h in &lay.hangers {
        *kinds.entry(format!("{:?}", h.kind)).or_insert(0) += 1;
    }
    println!("hangers {kinds:?}");
    let limit = 150 * TICKS_PER_SECOND as u32;
    for level in Difficulty::ALL {
        let mut row = Vec::new();
        for k in 0..seeds {
            let course = if k == 0 {
                city()
            } else {
                city_seeded(0xC17E_5EED_0001 + k * 7919)
            };
            let t = trial_at(course, &[Kind::Chimpanzee], &[level], true, limit);
            row.push(match t[0].finish {
                Some(f) => format!("{:.1}", f as f64 / TICKS_PER_SECOND as f64),
                None => format!("DNF@{}m", t[0].progress / ONE),
            });
        }
        println!("{:<10} {}", level.key(), row.join(" "));
    }
}
