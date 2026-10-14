//! One AI ape on Sector Ape, traced: every half second its progress, mode,
//! height, side and speed, and where it grabbed (o: an odd vine), hit a
//! body (R: a rebound, W: a dead hit), landed or was put back on a deck
//! (RESET).
//!
//! `cargo run --release -p tool_core --example sector_trace -- LEVEL [SEED [KIND [RUN]]]`
//! (a kind other than the Chimpanzee races with its own stats;
//! SECTOR_TRACE_CANDS=1 prints the candidates, SECTOR_TRACE_LAVA=1 races
//! with Floor is Lava)
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::ape::{Happening, Mode};
use tool_core::fx::ONE;
use tool_core::race::{Driver, Entry, Event, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};
use tool_core::sector::{sector, sector_seeded};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let level = Difficulty::from_key(&args[0]).unwrap();
    let k: u64 = args.get(1).map(|a| a.parse().unwrap()).unwrap_or(0);
    let course = if k == 0 {
        sector()
    } else {
        sector_seeded(0x5EC7_5EED_0001 + k * 7919)
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
    race.lava = std::env::var("SECTOR_TRACE_LAVA").is_ok();
    let mut brain = Brain::new(level, 0xA9E5 + run * 101);
    let mut line = String::new();
    let (mut resets, mut rebounds, mut walls, mut top) = (0, 0, 0, 0);
    while race.clock < 150 * TICKS_PER_SECOND as u32 && race.phase != Phase::Done {
        let c = think(&race, 0, &mut brain);
        race.step(&[c]);
        let a = &race.racers[0].ape;
        top = top.max(a.speed());
        for e in &race.events {
            match e {
                Event::Reset { pad, .. } => {
                    resets += 1;
                    line += &format!(" RESET(pad {pad}) ");
                }
                Event::Door { .. } => line += " DOOR ",
                Event::Eaten { .. } => line += " EATEN ",
                _ => {}
            }
        }
        for h in &a.happened {
            match h {
                Happening::Wall => {
                    walls += 1;
                    line += "W"
                }
                Happening::Rebound => {
                    rebounds += 1;
                    // the body it hit: the nearest one's middle
                    let s = race.course.sector.as_ref().unwrap();
                    let (k, b) = s
                        .bodies
                        .iter()
                        .enumerate()
                        .filter(|(_, b)| b.solid())
                        .min_by_key(|(_, b)| b.frame.t.dist(a.pos) - b.reach())
                        .unwrap();
                    let carries: Vec<usize> = s
                        .hangers
                        .iter()
                        .enumerate()
                        .filter(|(_, h)| **h == tool_core::sector::Hanger::Body(k))
                        .map(|(i, _)| i)
                        .collect();
                    line += &format!(
                        "R[{k} {:?} y{} carries {carries:?}]",
                        b.model,
                        b.frame.t.y / ONE
                    )
                }
                Happening::Land => line += "L",
                Happening::Grab => {
                    let v = a.swinging().unwrap_or(0);
                    let odd = if race.course.odd_vine(v) { "o" } else { "" };
                    line += &format!("g{v}{odd}")
                }
                _ => {}
            }
        }
        if race.clock.is_multiple_of(TICKS_PER_SECOND as u32 / 2) && race.running() {
            let m = match a.mode {
                Mode::Flying => "fly",
                Mode::Swinging { .. } => "vine",
                Mode::Grounded => "RUN",
            };
            let c = &race.course;
            let where_ = |v: usize| {
                let tip = c.tip(v, race.tick);
                let (s, side) = c.progress(tip, c.nearest_anywhere(tip));
                format!("#{v}@{}{:+}h{}", s / ONE, side / ONE, tip.y / ONE)
            };
            let side = c.progress(a.pos, a.hint).1 / ONE;
            let held = a.swinging().map(where_).unwrap_or_default();
            let aim = brain
                .target
                .filter(|&t| t < c.vines.len())
                .map(where_)
                .unwrap_or_default();
            let cands: Vec<String> = if std::env::var("SECTOR_TRACE_CANDS").is_ok() {
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
                a.pos.y / ONE,
                a.speed() / ONE,
                line
            );
            line.clear();
        }
    }
    println!(
        "finish {:?}, top {} m/s, resets {resets}, rebounds {rebounds}, dead hits {walls}, rescues {}, give-ups {}",
        race.racers[0].finish.map(|f| f as f64 / 120.0),
        top / ONE,
        brain.rescues,
        brain.give_ups
    );
}
