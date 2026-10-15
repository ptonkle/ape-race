//! One AI ape on Sunset Ruins, traced: every half second its progress, mode,
//! height and speed, and where it hit walls, landed, swam or was stuck.
//!
//! `cargo run --release -p tool_core --example ruins_trace -- LEVEL [SEED [KIND [RUN]]]` (a kind other than the
//! Chimpanzee races with its own stats; with `RUINS_TRACE_CANDS=1` its
//! target and candidates too)
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::ape::{Happening, Mode};
use tool_core::fx::ONE;
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::ruins::{ruins, ruins_seeded};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let level = Difficulty::from_key(&args[0]).unwrap();
    let k: u64 = args.get(1).map(|a| a.parse().unwrap()).unwrap_or(0);
    let course = if k == 0 {
        ruins()
    } else {
        ruins_seeded(0x2B1_5E75_0001 + k * 7919)
    };
    let kind = args
        .get(2)
        .and_then(|k| Kind::from_key(k))
        .unwrap_or(Kind::Chimpanzee);
    let entries = [Entry {
        name: String::new(),
        kind,
        driver: Driver::Ai,
        palette: 0,
    }];
    let run: u64 = args.get(3).map(|a| a.parse().unwrap()).unwrap_or(0);
    let mut race = Race::new(course, &entries, kind == Kind::Chimpanzee, 7 + run);
    let mut brain = Brain::new(level, 0xA9E5 + run * 101);
    let mut line = String::new();
    while race.clock < 150 * TICKS_PER_SECOND as u32 && race.phase != Phase::Done {
        let c = think(&race, 0, &mut brain);
        race.step(&[c]);
        for e in &race.events {
            if matches!(e, tool_core::race::Event::Reset { .. }) {
                line += "R";
            }
        }
        let a = &race.racers[0].ape;
        for h in &a.happened {
            match h {
                Happening::Wall => line += "W",
                Happening::Land => line += "L",
                Happening::Bounce => line += "B",
                Happening::Spiked => line += "S",
                Happening::Grab => line += &format!("g{}", a.swinging().unwrap_or(0)),
                _ => {}
            }
        }
        if race.clock.is_multiple_of(TICKS_PER_SECOND as u32 / 2) && race.running() {
            let m = match a.mode {
                Mode::Flying => "fly",
                Mode::Swinging { .. } => "vine",
                Mode::Grounded if a.swimming(&race.course) => "SWIM",
                Mode::Grounded if a.climb.is_some() => "CLIMB",
                Mode::Grounded => "RUN",
            };
            let side = race.course.progress(a.pos, a.hint).1;
            println!(
                "{:>5.1}s {:>5} m {:>+6.1} {:<5} y{:>4} v{:>3} {}",
                race.clock as f64 / TICKS_PER_SECOND as f64,
                race.racers[0].progress / ONE,
                side as f64 / ONE as f64,
                m,
                a.pos.y / ONE,
                a.speed() / ONE,
                line
            );
            if std::env::var("RUINS_TRACE_CANDS").is_ok() {
                println!(
                    "        target {:?} cands {:?} expect {:?}",
                    brain.target, brain.cands, brain.expect
                );
            }
            line.clear();
        }
    }
    println!(
        "finish {:?}, rescues {}, give-ups {}",
        race.racers[0].finish.map(|f| f as f64 / 120.0),
        brain.rescues,
        brain.give_ups
    );
}
