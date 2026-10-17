//! One AI ape on Rattlesnake Ravine, traced: where it stalls (3 s with
//! under 25 m gained), with its mode, height and side, and its finish.
//!
//! `cargo run --release -p tool_core --example ravine_trace -- LEVEL [SEED [upper [full] [solids]]]`
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::ape::{Happening, Mode};
use tool_core::fx::ONE;
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::ravine::{ravine, ravine_seeded, ravine_upper_seeded};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let level = Difficulty::from_key(&args[0]).unwrap();
    let k: u64 = args.get(1).map(|a| a.parse().unwrap()).unwrap_or(0);
    let upper = args.iter().any(|a| a == "upper");
    let full = args.iter().any(|a| a == "full");
    // `solids`: at a stall, the solids within 3 m of the ape
    let show_solids = args.iter().any(|a| a == "solids");
    let seed = 0x5A4E_C0DE_0001 + k * 7919;
    let course = match (k, upper) {
        (0, false) => ravine(),
        (_, false) => ravine_seeded(seed),
        (_, true) => ravine_upper_seeded(seed),
    };
    let kind = Kind::Chimpanzee;
    let entries = [Entry {
        name: String::new(),
        kind,
        driver: Driver::Ai,
        palette: 0,
        costume: tool_core::costume::Costume::Own,
    }];
    let mut race = Race::new(course, &entries, true, 7);
    let mut brain = Brain::new(level, 0xA9E5);
    let half = TICKS_PER_SECOND as u32 / 2;
    let mut hist: Vec<i32> = Vec::new();
    let mut walls = 0;
    let mut line = String::new();
    while race.clock < 150 * TICKS_PER_SECOND as u32 && race.phase != Phase::Done {
        let c = think(&race, 0, &mut brain);
        race.step(&[c]);
        let a = &race.racers[0].ape;
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
            let p = race.racers[0].progress / ONE;
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
                    if stalled { "STALL " } else { "" },
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
        "finish {:?}, walls {walls}, rescues {}, give-ups {}",
        race.racers[0]
            .finish
            .map(|f| f as f64 / TICKS_PER_SECOND as f64),
        brain.rescues,
        brain.give_ups
    );
}
