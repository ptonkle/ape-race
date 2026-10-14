//! Seconds an AI spends on each stretch of the City (straights, corners,
//! the bridge), per kind: where a level loses its time.
//!
//! `cargo run --release -p tool_core --example city_segments -- LEVEL [SEED [RUNS]]`
//! (RUNS brain seeds per kind; the last row is the mean over them all)
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::city::{city, city_seeded};
use tool_core::fx::{int, ONE};
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let level = Difficulty::from_key(&a[0]).unwrap();
    let seed: u64 = a.get(1).map(|s| s.parse().unwrap()).unwrap_or(0);
    let runs: u64 = a.get(2).map(|s| s.parse().unwrap()).unwrap_or(1);
    let mut total = [0.0; 13];
    let mut count = 0.0;
    let mut kind_total = [0.0; 13];
    let marks = [
        230, 300, 490, 550, 720, 800, 1170, 1240, 1390, 1470, 1620, 1690, 1800,
    ];
    println!("            0-230 90L  ..490 45R  ..720 90R  bridge 45L ..1390 90L ..1620 45R  end");
    for (kind, run) in Kind::ALL
        .into_iter()
        .flat_map(|k| (0..runs).map(move |r| (k, r)))
    {
        let course = if seed == 0 {
            city()
        } else {
            city_seeded(0xC17E_5EED_0001 + seed * 7919)
        };
        let entries = [Entry {
            name: String::new(),
            kind,
            driver: Driver::Ai,
            palette: 0,
        }];
        let mut race = Race::new(course, &entries, false, 7 + run);
        let mut brain = Brain::new(level, 0xA9E5 + run * 101);
        let mut at = vec![None; marks.len()];
        while race.clock < 150 * TICKS_PER_SECOND as u32 && race.phase != Phase::Done {
            let c = think(&race, 0, &mut brain);
            race.step(&[c]);
            let p = race.racers[0].progress;
            for (k, m) in marks.iter().enumerate() {
                if at[k].is_none() && p >= int(*m) - ONE / 2 {
                    at[k] = Some(race.clock);
                }
            }
        }
        let mut prev = 0;
        let mut row = format!("{:<11}", format!("{kind:?}"));
        let end = 150 * TICKS_PER_SECOND as u32;
        for (k, t) in at.into_iter().enumerate() {
            let t = t.unwrap_or(end).max(prev);
            let d = (t - prev) as f64 / TICKS_PER_SECOND as f64;
            total[k] += d;
            row += &format!(" {:>5.1}", d);
            prev = t;
        }
        count += 1.0;
        if runs == 1 {
            println!("{row}");
        }
        for (k, d) in row
            .split_whitespace()
            .skip(1)
            .map(|x| x.parse::<f64>().unwrap_or(0.0))
            .enumerate()
        {
            kind_total[k] += d;
        }
        if run == runs - 1 && runs > 1 {
            let mut line = format!("{:<11}", format!("{kind:?}"));
            for t in kind_total.iter_mut() {
                line += &format!(" {:>5.1}", *t / runs as f64);
                *t = 0.0;
            }
            println!("{line}");
        }
    }
    let mut row = format!("{:<11}", "mean");
    for t in total {
        row += &format!(" {:>5.1}", t / count);
    }
    println!("{row}");
}
