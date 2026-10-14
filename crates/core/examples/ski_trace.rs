//! One AI ape on the Ski Slope, traced: every half second its progress, mode,
//! height and speed, and where it hit walls, landed, skied or was stuck.
//!
//! `cargo run --release -p tool_core --example ski_trace -- LEVEL [SEED [KIND [RUN]]]` (a kind other than the
//! Chimpanzee races with its own stats)
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::ape::{Happening, Mode};
use tool_core::fx::ONE;
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};
use tool_core::ski::{ski, ski_seeded};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let level = Difficulty::from_key(&args[0]).unwrap();
    let k: u64 = args.get(1).map(|a| a.parse().unwrap()).unwrap_or(0);
    // (SKI_TRACE_JUNGLE=1 traces the Jungle instead, for comparison)
    let course = if std::env::var("SKI_TRACE_JUNGLE").is_ok() {
        tool_core::course::Course::jungle_seeded(k)
    } else if k == 0 {
        ski()
    } else {
        ski_seeded(0x5C1_5EED_0001 + k * 7919)
    };
    let kind = args
        .get(2)
        .and_then(|k| Kind::from_key(k))
        .unwrap_or(Kind::Chimpanzee);
    let entries = [Entry {
        name: String::new(),
        kind,
        driver: Driver::Ai,
    }];
    let run: u64 = args.get(3).map(|a| a.parse().unwrap()).unwrap_or(0);
    let mut race = Race::new(course, &entries, kind == Kind::Chimpanzee, 7 + run);
    let mut brain = Brain::new(level, 0xA9E5 + run * 101);
    let mut line = String::new();
    let mut island = 0;
    while race.clock < 150 * TICKS_PER_SECOND as u32 && race.phase != Phase::Done {
        let c = think(&race, 0, &mut brain);
        race.step(&[c]);
        let a = &race.racers[0].ape;
        for h in &a.happened {
            match h {
                Happening::Wall => {
                    // an island's wall: the ape put back to a branch's
                    // inner edge
                    let c = &race.course;
                    let (s, side) = c.progress(a.pos, a.hint);
                    let [(m0, h0), (m1, h1)] = c.lanes(s);
                    let split = m0 + h0 < m1 - h1;
                    if split && side > m0 + h0 - ONE && side < m1 - h1 + ONE {
                        island += 1;
                        line += "I";
                    } else {
                        line += "W";
                    }
                }
                Happening::Land => line += "L",
                Happening::Bounce => line += "B",
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
                Mode::Grounded if a.on_snow(&race.course) => "SKI",
                Mode::Grounded => "RUN",
            };
            let c = &race.course;
            let where_ = |v: usize| {
                let an = c.vines[v].anchor;
                let (s, side) = c.progress(an, c.nearest_anywhere(an));
                format!(
                    "#{v}@{}{:+}h{}",
                    s / ONE,
                    side / ONE,
                    (an.y - c.vines[v].length - c.terrain(an.x, an.z)) / ONE
                )
            };
            let side = c.progress(a.pos, a.hint).1 / ONE;
            let held = a.swinging().map(where_).unwrap_or_default();
            let aim = brain
                .target
                .filter(|&t| t < c.vines.len())
                .map(where_)
                .unwrap_or_default();
            let cands: Vec<String> = if std::env::var("SKI_TRACE_CANDS").is_ok() {
                brain.cands.iter().map(|&v| where_(v)).collect()
            } else {
                Vec::new()
            };
            line = format!("side {side:+} {held} -> {aim} {line} {}", cands.join(" "));
            println!(
                "{:>5.1}s {:>5} m {:<5} y{:>4} v{:>3} {}",
                race.clock as f64 / TICKS_PER_SECOND as f64,
                race.racers[0].progress / ONE,
                m,
                (a.pos.y - race.course.terrain(a.pos.x, a.pos.z)) / ONE,
                a.speed() / ONE,
                line
            );
            line.clear();
        }
    }
    println!(
        "finish {:?}, rescues {}, give-ups {}, island wall ticks {island}",
        race.racers[0].finish.map(|f| f as f64 / 120.0),
        brain.rescues,
        brain.give_ups
    );
}
