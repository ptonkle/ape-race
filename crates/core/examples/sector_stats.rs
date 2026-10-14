//! What costs the AI time on Sector Ape, by the way each vine hangs: over
//! eight layouts and the four levels, how often each kind of vine is
//! grabbed, how long it is held, how often a hold runs past 2.5 s (a whirl
//! round the anchor with no way on), and how often a reset follows within
//! 5 s of letting go; and what came just before each reset.
//!
//! `cargo run --release -p tool_core --example sector_stats [-- SEEDS]`
use std::collections::BTreeMap;
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::ape::{Happening, Mode};
use tool_core::course::Course;
use tool_core::fx::ONE;
use tool_core::race::{Driver, Entry, Event, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};
use tool_core::sector::{sector, sector_seeded, DOWN};

fn layout(k: u64) -> Course {
    if k == 0 {
        sector()
    } else {
        sector_seeded(0x5EC7_5EED_0001 + k * 7919)
    }
}

/// The way vine `v` hangs, as a word.
fn way(c: &Course, v: usize) -> &'static str {
    let h = c.hang(v);
    if v >= c.sector.as_ref().unwrap().hangs.len() {
        return "summoned";
    }
    if matches!(
        c.sector.as_ref().unwrap().hangers[v],
        tool_core::sector::Hanger::Mech { .. }
    ) {
        return if h == DOWN { "mech down" } else { "mech slant" };
    }
    if h == DOWN {
        "down"
    } else if h.y > ONE * 9 / 10 {
        "up"
    } else if h.y.abs() < ONE / 10 {
        "side"
    } else if h.y < 0 {
        "slant down"
    } else {
        "slant up"
    }
}

#[derive(Default, Clone)]
struct Row {
    grabs: u32,
    ticks: u64,
    whirls: u32,
    resets_after: u32,
}

fn main() {
    let seeds: u64 = std::env::args()
        .nth(1)
        .map(|a| a.parse().unwrap())
        .unwrap_or(8);
    let jobs: Vec<_> = Difficulty::ALL
        .into_iter()
        .flat_map(|l| (0..seeds).map(move |k| (l, k)))
        .map(|(level, k)| {
            std::thread::spawn(move || {
                let course = layout(k);
                let entries = [Entry {
                    name: String::new(),
                    kind: Kind::Chimpanzee,
                    driver: Driver::Ai,
                    palette: 0,
                }];
                let mut race = Race::new(course, &entries, true, 7);
                let mut brain = Brain::new(level, 0xA9E5);
                let mut rows: BTreeMap<&'static str, Row> = BTreeMap::new();
                let mut before: BTreeMap<String, u32> = BTreeMap::new();
                let mut held: Option<(usize, u32)> = None;
                let mut last_let_go: Option<(&'static str, u32)> = None;
                let mut last_hit: Option<(String, u32)> = None;
                while race.clock < 150 * TICKS_PER_SECOND as u32 && race.phase != Phase::Done {
                    let c = think(&race, 0, &mut brain);
                    race.step(&[c]);
                    let t = race.tick;
                    let a = &race.racers[0].ape;
                    for h in &a.happened {
                        if matches!(h, Happening::Rebound | Happening::Wall) {
                            let what = if *h == Happening::Rebound { "rebound" } else { "dead hit" };
                            let mode = match held {
                                Some(_) => "on a vine",
                                None => "flying",
                            };
                            last_hit = Some((format!("{what} {mode}"), t));
                        }
                    }
                    match (held, a.swinging()) {
                        (None, Some(v)) => held = Some((v, t)),
                        (Some((v, since)), now) if now != Some(v) => {
                            let w = way(&race.course, v);
                            let r = rows.entry(w).or_default();
                            r.grabs += 1;
                            r.ticks += (t - since) as u64;
                            if t - since > 300 {
                                r.whirls += 1;
                            }
                            last_let_go = Some((w, t));
                            held = now.map(|n| (n, t));
                        }
                        _ => {}
                    }
                    for e in &race.events {
                        if let Event::Reset { .. } = e {
                            if let Some((w, at)) = last_let_go {
                                if t - at < 600 {
                                    rows.entry(w).or_default().resets_after += 1;
                                }
                            }
                            let cause = match (&last_hit, last_let_go) {
                                (Some((h, at)), _) if t - at < 600 => h.clone(),
                                (_, Some((w, at))) if t - at < 600 => format!("let go of a {w} vine"),
                                _ => "long flight".to_string(),
                            };
                            *before.entry(cause).or_insert(0) += 1;
                        }
                    }
                    let _ = Mode::Flying;
                }
                (rows, before)
            })
        })
        .collect();
    let mut total: BTreeMap<&'static str, Row> = BTreeMap::new();
    let mut causes: BTreeMap<String, u32> = BTreeMap::new();
    for j in jobs {
        let (rows, before) = j.join().unwrap();
        for (k, r) in rows {
            let t = total.entry(k).or_default();
            t.grabs += r.grabs;
            t.ticks += r.ticks;
            t.whirls += r.whirls;
            t.resets_after += r.resets_after;
        }
        for (k, n) in before {
            *causes.entry(k).or_insert(0) += n;
        }
    }
    println!("{:<12} {:>6} {:>8} {:>7} {:>8}", "vine", "grabs", "held s", "whirls", "resets");
    for (k, r) in &total {
        println!(
            "{:<12} {:>6} {:>8.2} {:>7} {:>8}",
            k,
            r.grabs,
            r.ticks as f64 / r.grabs.max(1) as f64 / TICKS_PER_SECOND as f64,
            r.whirls,
            r.resets_after
        );
    }
    println!("before each reset:");
    for (k, n) in &causes {
        println!("  {n:>4}  {k}");
    }
}
