//! One AI ape on the tower map, traced: every half second its progress, mode,
//! height and speed, and where it hit walls, landed, swam or was stuck.
//!
//! `cargo run --release -p tool_core --example tower_trace -- LEVEL [SEED [KIND [RUN]]]` (a kind other than the
//! Chimpanzee races with its own stats; with `TOWER_TRACE_CANDS=1` its
//! target and candidates too)
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::ape::{Happening, Mode};
use tool_core::fx::ONE;
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};
use tool_core::tower::{tower, tower_seeded};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let level = Difficulty::from_key(&args[0]).unwrap();
    let k: u64 = args.get(1).map(|a| a.parse().unwrap()).unwrap_or(0);
    let course = if k == 0 {
        tower()
    } else {
        tower_seeded(0x70E5_0000_0001 + k * 7919)
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
        costume: tool_core::costume::Costume::Own,
    }];
    if std::env::var("TOWER_VINES").is_ok() {
        let t = course.tower.as_deref().unwrap();
        for (i, v) in course.vines.iter().enumerate() {
            let p = course.point(v.at);
            println!(
                "vine {i} at {} m: anchor {:?} hang {:?} len {} (path {:?}) {:?}",
                v.at / ONE,
                v.anchor.scale(100).x / ONE,
                t.hangs[i],
                v.length / ONE,
                (p.x / ONE, p.y / ONE, p.z / ONE),
                (v.anchor.x / ONE, v.anchor.y / ONE, v.anchor.z / ONE),
            );
        }
        for p in &t.pads {
            println!("pad at {} m boost {:?}", p.at / ONE, p.boost);
        }
    }
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
                "{:>5.1}s {:>5} m {:>+6.1} {:<5} x{:>4} y{:>4} z{:>4} v{:>3} h{} {}",
                race.clock as f64 / TICKS_PER_SECOND as f64,
                race.racers[0].progress / ONE,
                side as f64 / ONE as f64,
                m,
                a.pos.x / ONE,
                a.pos.y / ONE,
                a.pos.z / ONE,
                a.speed() / ONE,
                a.hint * 2,
                line
            );
            if std::env::var("TOWER_TRACE_CANDS").is_ok() {
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
