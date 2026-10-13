//! Every kind (its own stats) at one difficulty on a map: finish times.
//!
//! `cargo run --release -p tool_core --example kinds_report -- MAP LEVEL [SEEDS]`
use tool_core::ai::{trial_at, Difficulty};
use tool_core::fx::ONE;
use tool_core::map::Map;
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let map = Map::from_key(&a[0]).unwrap();
    let level = Difficulty::from_key(&a[1]).unwrap();
    let seeds: u64 = a.get(2).map(|s| s.parse().unwrap()).unwrap_or(1);
    for k in Kind::ALL {
        let mut row = Vec::new();
        for s in 0..seeds {
            let course = match (map, s) {
                (_, 0) => map.course(),
                (Map::Jungle, s) => {
                    tool_core::course::Course::jungle_seeded(0x0A9E_2ACE_0001 + s * 7919)
                }
                (Map::City, s) => tool_core::city::city_seeded(0xC17E_5EED_0001 + s * 7919),
            };
            let t = trial_at(course, &[k], &[level], false, 150 * TICKS_PER_SECOND as u32);
            row.push(match t[0].finish {
                Some(f) => format!("{:.0}", f as f64 / TICKS_PER_SECOND as f64),
                None => format!("DNF@{}", t[0].progress / ONE),
            });
        }
        println!("{:<11} {}", format!("{k:?}"), row.join(" "));
    }
}
