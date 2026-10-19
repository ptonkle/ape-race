//! One AI ape on Aisle Ape, traced: where it stalls (3 s with
//! under 25 m gained), with its mode, height and side, and its finish.
//!
//! `cargo run --release -p tool_core --example grocery_trace -- LEVEL [SEED [full] [solids] [slot=K]]`
//! (`slot=K`: a race of four, tracing racer K, so its own item spots;
//! `kind=KEY`: that kind; `boosts`: crates give only the Speed Boost, as in
//! a Time Trial)
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::ape::{Happening, Mode};
use tool_core::fx::ONE;
use tool_core::grocery::{grocery, grocery_seeded, plan_of};
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let level = Difficulty::from_key(&args[0]).unwrap();
    let k: u64 = args.get(1).map(|a| a.parse().unwrap()).unwrap_or(0);
    let full = args.iter().any(|a| a == "full");
    // `solids`: at a stall, the solids within 3 m of the ape
    let show_solids = args.iter().any(|a| a == "solids");
    let seed = 0x6E0C_E2E5_0001 + k * 7919;
    let course = match k {
        0 => grocery(),
        _ => grocery_seeded(seed),
    };
    let kind = args
        .iter()
        .find_map(|a| a.strip_prefix("kind=").and_then(Kind::from_key))
        .unwrap_or(Kind::Chimpanzee);
    let slot: Option<usize> = args
        .iter()
        .find_map(|a| a.strip_prefix("slot=").map(|k| k.parse().unwrap()));
    let entries: Vec<Entry> = (0..slot.map_or(1, |_| 4))
        .map(|_| Entry {
            name: String::new(),
            kind,
            driver: Driver::Ai,
            palette: 0,
            costume: tool_core::costume::Costume::Own,
        })
        .collect();
    let me = slot.unwrap_or(0);
    let boosts = args.iter().any(|a| a == "boosts");
    let mut race = Race::new(course, &entries, !boosts, 7);
    if boosts {
        race.items = tool_core::items::ItemSet::only(tool_core::race::Item::Boost);
    }
    let mut brains: Vec<Brain> = (0..entries.len())
        .map(|i| Brain::new(level, 0xA9E5 + i as u64))
        .collect();
    let half = TICKS_PER_SECOND as u32 / 2;
    let mut hist: Vec<i32> = Vec::new();
    let mut walls = 0;
    let mut line = String::new();
    while race.clock < 150 * TICKS_PER_SECOND as u32 && race.phase != Phase::Done {
        let cs: Vec<_> = (0..entries.len())
            .map(|i| think(&race, i, &mut brains[i]))
            .collect();
        race.step(&cs);
        let brain = &brains[me];
        let a = &race.racers[me].ape;
        for h in &a.happened {
            match h {
                Happening::Wall => {
                    walls += 1;
                    line += "W"
                }
                Happening::Land => line += "L",
                Happening::Bounce => line += "B",
                Happening::Snapped => line += "S",
                Happening::Grab => line += &format!("g{}", a.swinging().unwrap_or(0)),
                _ => {}
            }
        }
        if race.clock.is_multiple_of(half) && race.running() {
            let p = race.racers[me].progress / ONE;
            hist.push(p);
            let n = hist.len();
            let stalled = n > 6 && p - hist[n - 7] < 25;
            if stalled || full {
                let m = match a.mode {
                    Mode::Flying => "fly",
                    Mode::Swinging { .. } => "vine",
                    Mode::Grounded => "RUN",
                };
                let side = race.course.progress(a.pos, a.hint).1;
                println!(
                    "{:>5.1}s {:>5} m {:>+6.1} {:<5} y{:>4} v{:>3} {}{}",
                    race.clock as f64 / TICKS_PER_SECOND as f64,
                    p,
                    side as f64 / ONE as f64,
                    m,
                    a.pos.y / ONE,
                    a.speed() / ONE,
                    if stalled {
                        format!(
                            "STALL t{:?} plan ({},{}) ",
                            brain.target,
                            plan_of(a.pos).0 / ONE,
                            plan_of(a.pos).1 / ONE
                        )
                    } else {
                        String::new()
                    },
                    line
                );
            }
            line.clear();
            if stalled && show_solids {
                if let Some(s) = race.course.ravine_solids() {
                    for &k in s.near(a.pos) {
                        let sol = &s.list[k as usize];
                        if sol.holds(a.pos, 3 * ONE) {
                            println!("        at {:?}: {:?}", a.pos.scale(1), sol);
                        }
                    }
                }
            }
        }
    }
    println!(
        "finish {:?}, walls {walls}, rescues {}, give-ups {}, list {:04b}",
        race.racers[me]
            .finish
            .map(|f| f as f64 / TICKS_PER_SECOND as f64),
        brains[me].rescues,
        brains[me].give_ups,
        race.racers[me].ape.list
    );
}
