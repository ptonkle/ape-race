//! Sector Ape's layout in numbers, and the AI on it: finish times by
//! difficulty over seeds (in parallel), with the resets, and the mean time
//! each level spends in each section of the course.
//!
//! `cargo run --release -p tool_core --example sector_report [-- SEEDS [KIND]]`
//! (SECTOR_REPORT_KINDS=1: every kind at King Kong and the Sloth instead)
use tool_core::ai::{think, Brain, Difficulty};
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

/// The sections, by where they end (metres along).
const SECTIONS: [(i32, &str); 9] = [
    (250, "start"),
    (460, "debris"),
    (720, "hulk"),
    (1140, "bend"),
    (1340, "wreck"),
    (1700, "belt in"),
    (1880, "belt"),
    (2040, "pre-armada"),
    (2330, "armada"),
];

struct Run {
    finish: Option<f64>,
    at: i32,
    resets: u32,
    top: i32,
    hidden: bool,
    /// seconds in each section (to the first time past its end)
    split: Vec<f64>,
}

/// One AI ape alone.
fn run(course: Course, kind: Kind, level: Difficulty, equal: bool) -> Run {
    let entries = [Entry {
        name: String::new(),
        kind,
        driver: Driver::Ai,
        palette: 0,
    }];
    let mut race = Race::new(course, &entries, equal, 7);
    let mut brain = Brain::new(level, 0xA9E5);
    let (mut resets, mut top, mut hidden) = (0, 0, false);
    let w = race.course.sector.as_ref().unwrap().wreck;
    let pass = tool_core::spacecraft::wreck_passage(&w.frame, w.gap);
    let mut reached = vec![None; SECTIONS.len()];
    while race.clock < 150 * TICKS_PER_SECOND as u32 && race.phase != Phase::Done {
        let c = think(&race, 0, &mut brain);
        race.step(&[c]);
        resets += race
            .events
            .iter()
            .filter(|e| matches!(e, Event::Reset { .. }))
            .count() as u32;
        let a = &race.racers[0].ape;
        top = top.max(a.speed());
        // in the passage: its middle within the opening, inside the hull
        let d = a.middle().sub(pass.middle);
        let along = d.dot(pass.axis);
        hidden |= along.abs() < pass.length / 2 && d.reject(pass.axis).len() < pass.width / 2;
        let p = race.racers[0].progress / ONE;
        for (k, (end, _)) in SECTIONS.iter().enumerate() {
            if reached[k].is_none() && (p >= *end || race.racers[0].finish.is_some()) {
                reached[k] = Some(race.clock);
            }
        }
    }
    let r = &race.racers[0];
    let mut split = Vec::new();
    let mut last = 0;
    for t in reached {
        let t = t.unwrap_or(race.clock);
        split.push((t - last) as f64 / TICKS_PER_SECOND as f64);
        last = t;
    }
    Run {
        finish: r.finish.map(|f| f as f64 / TICKS_PER_SECOND as f64),
        at: r.progress / ONE,
        resets,
        top: top / ONE,
        hidden,
        split,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let seeds: u64 = args.first().map(|a| a.parse().unwrap()).unwrap_or(1);
    let kind = args
        .get(1)
        .and_then(|k| Kind::from_key(k))
        .unwrap_or(Kind::Chimpanzee);
    let c = sector();
    let lay = c.sector.as_ref().unwrap();
    let odd = lay.hangs.iter().filter(|h| **h != DOWN).count();
    println!(
        "length {} m, vines {} ({} odd), bodies {}, pads {}, mechs {}, bananas {}, crates {}, routes {}",
        c.length / ONE,
        c.vines.len(),
        odd,
        lay.bodies.len(),
        lay.pads.len(),
        lay.mechs.len(),
        c.bananas.len(),
        c.crates.len(),
        lay.routes.len()
    );
    if std::env::var("SECTOR_REPORT_KINDS").is_ok() {
        let rows: Vec<_> = [Difficulty::Sloth, Difficulty::KingKong]
            .into_iter()
            .flat_map(|level| Kind::ALL.into_iter().map(move |k| (level, k)))
            .map(|(level, k)| {
                std::thread::spawn(move || (level, k, run(sector(), k, level, false)))
            })
            .collect();
        for r in rows {
            let (level, k, r) = r.join().unwrap();
            println!(
                "{:<10} {:<11} {}  resets {} top {} m/s{}",
                level.key(),
                format!("{k:?}"),
                r.finish
                    .map_or(format!("DNF@{}m", r.at), |f| format!("{f:.1} s")),
                r.resets,
                r.top,
                if r.hidden { " (passage)" } else { "" }
            );
        }
        return;
    }
    let rows: Vec<_> = Difficulty::ALL
        .into_iter()
        .flat_map(|level| (0..seeds).map(move |k| (level, k)))
        .map(|(level, k)| {
            std::thread::spawn(move || {
                (
                    level,
                    k,
                    run(layout(k), kind, level, kind == Kind::Chimpanzee),
                )
            })
        })
        .collect();
    let mut table: Vec<(Difficulty, u64, Run)> =
        rows.into_iter().map(|r| r.join().unwrap()).collect();
    table.sort_by_key(|(l, k, _)| (*l, *k));
    for level in Difficulty::ALL {
        let row: Vec<&Run> = table
            .iter()
            .filter(|(l, _, _)| *l == level)
            .map(|(_, _, r)| r)
            .collect();
        let cells: Vec<String> = row
            .iter()
            .map(|r| {
                let t = r
                    .finish
                    .map_or(format!("DNF@{}", r.at), |f| format!("{f:.1}"));
                format!(
                    "{t}{}{}",
                    "r".repeat(r.resets as usize),
                    if r.hidden { "*" } else { "" }
                )
            })
            .collect();
        let done: Vec<f64> = row.iter().filter_map(|r| r.finish).collect();
        let tops: Vec<i32> = row.iter().map(|r| r.top).collect();
        println!(
            "{:<10} {}  mean {:.1} ({} of {})  top {:?}",
            level.key(),
            cells.join(" "),
            done.iter().sum::<f64>() / done.len().max(1) as f64,
            done.len(),
            row.len(),
            tops
        );
        let means: Vec<String> = (0..SECTIONS.len())
            .map(|k| {
                let m = row.iter().map(|r| r.split[k]).sum::<f64>() / row.len() as f64;
                format!("{} {m:.1}", SECTIONS[k].1)
            })
            .collect();
        println!("           sections: {}", means.join(", "));
    }
    println!("(r: a reset onto a deck; *: through the wreck's passage)");
}
