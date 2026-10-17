//! The AI on Sector Ape (2026-10-13): every difficulty finishes it, and King
//! Kong beats the Sloth on average, as on the Jungle (`tests/ai.rs`).
//!
//! Races in open space vary far more from layout to layout than on the
//! other maps (a whirl round an anchor, a reset onto a deck), so the order
//! is taken over 32 layouts, not 8, run in parallel (SECOND-ORDER SC rows).

use tool_core::ai::{time_trial, trial_at, Difficulty, Trial};
use tool_core::course::Course;
use tool_core::fx::ONE;
use tool_core::map::Map;
use tool_core::rules::{Kind, TICKS_PER_SECOND};
use tool_core::sector::{sector, sector_seeded};

fn secs(ticks: u32) -> f64 {
    ticks as f64 / TICKS_PER_SECOND as f64
}

const LIMIT: u32 = 150 * TICKS_PER_SECOND as u32;

/// How many layouts the order is taken over.
const LAYOUTS: u64 = 32;

/// Layout `k` of Sector Ape (0: the map itself).
fn layout(k: u64) -> Course {
    if k == 0 {
        sector()
    } else {
        sector_seeded(0x5EC7_5EED_0001 + k * 7919)
    }
}

/// The AI is there to show the map can be finished (SECOND-ORDER DM-3):
/// over `LAYOUTS` layouts (one Chimpanzee, Equal Stats) every race at every
/// level finishes inside the limit (150 s), and King Kong's mean finish
/// is faster than the Sloth's. No spacing between the levels is asked.
#[test]
fn sector_every_level_finishes_and_king_kong_beats_the_sloth() {
    let runs: Vec<_> = Difficulty::ALL
        .into_iter()
        .flat_map(|level| (0..LAYOUTS).map(move |k| (level, k)))
        .map(|(level, k)| {
            std::thread::spawn(move || {
                let t = trial_at(layout(k), &[Kind::Chimpanzee], &[level], true, LIMIT);
                let f = t[0].finish.unwrap_or_else(|| {
                    panic!(
                        "{} finishes sector layout {k} (at {} m)",
                        level.key(),
                        t[0].progress / ONE
                    )
                });
                (level, secs(f))
            })
        })
        .collect();
    let done: Vec<(Difficulty, f64)> = runs
        .into_iter()
        .map(|r| r.join().expect("every race finishes"))
        .collect();
    let mut means = Vec::new();
    for level in Difficulty::ALL {
        let row: Vec<f64> = done
            .iter()
            .filter(|(l, _)| *l == level)
            .map(|(_, t)| *t)
            .collect();
        let mean = row.iter().sum::<f64>() / row.len() as f64;
        println!("{:<10} {row:.1?} mean {mean:.1} s", level.key());
        means.push(mean);
    }
    let (sloth, king_kong) = (means[0], means[means.len() - 1]);
    assert!(
        king_kong < sloth,
        "king_kong ({king_kong:.1} s) is not faster than sloth ({sloth:.1} s) on average"
    );
}

/// Every kind of ape, at the easiest and the best level, finishes Sector
/// Ape (its own stats, not Equal Stats).
#[test]
fn every_kind_finishes_sector_ape_at_the_easiest_and_the_best_level() {
    let runs: Vec<_> = [Difficulty::Sloth, Difficulty::KingKong]
        .into_iter()
        .flat_map(|level| Kind::ALL.into_iter().map(move |k| (level, k)))
        .map(|(level, k)| {
            std::thread::spawn(move || {
                let t = trial_at(sector(), &[k], &[level], false, LIMIT);
                assert!(
                    t[0].finish.is_some(),
                    "{} {k:?} stopped at {} m",
                    level.key(),
                    t[0].progress / ONE
                );
            })
        })
        .collect();
    for r in runs {
        r.join().expect("every kind finishes");
    }
}

/// Four AI apes racing together, with items, all finish Sector Ape.
#[test]
fn four_ai_apes_with_items_finish_sector_ape() {
    let kinds = [Kind::Gibbon, Kind::Gorilla, Kind::Lemur, Kind::Orangutan];
    let levels = [
        Difficulty::Sloth,
        Difficulty::Sloth,
        Difficulty::Ape,
        Difficulty::KingKong,
    ];
    let t = trial_at(sector(), &kinds, &levels, false, LIMIT);
    for r in &t {
        assert!(
            r.finish.is_some(),
            "{:?} stopped at {} m",
            r.kind,
            r.progress / ONE
        );
    }
}

fn report(label: &str, trials: &[Trial]) {
    println!("{label}");
    for t in trials {
        println!(
            "  {:<11} finish {:>9}  mean {:>5.1} m/s  top {:>5.1} m/s  grabs {:>3}  landings {:>2}  ground {:>5.1} s",
            format!("{:?}", t.kind),
            t.finish
                .map(|f| format!("{:.2} s", secs(f)))
                .unwrap_or("-".into()),
            t.mean_speed as f64 / ONE as f64,
            t.top_speed as f64 / ONE as f64,
            t.grabs,
            t.landings,
            secs(t.ground_ticks),
        );
    }
}

/// Sector Ape's medal report: King Kong alone as a Time Trial runs, every
/// kind and Equal Stats, against the map's medal times.
#[test]
#[ignore]
fn sector_time_trial_report() {
    let runs: Vec<_> = Kind::ALL
        .into_iter()
        .map(|k| (k, false))
        .chain([(Kind::Chimpanzee, true)])
        .map(|(k, equal)| std::thread::spawn(move || time_trial(sector(), k, equal, LIMIT)))
        .collect();
    let row: Vec<Trial> = runs.into_iter().map(|r| r.join().unwrap()).collect();
    report("sector time trial (last: equal stats)", &row);
    let medals = tool_core::trial::medals(Map::Sector).map(secs);
    println!("medals bronze / silver / gold / author: {medals:.2?}");
}

/// Finish times per difficulty over the layouts, three kinds each: for
/// tuning.
#[test]
#[ignore]
fn sector_difficulty_report() {
    for level in Difficulty::ALL {
        let runs: Vec<_> = (0..LAYOUTS)
            .flat_map(|k| [Kind::Chimpanzee, Kind::Gorilla, Kind::Gibbon].map(|kind| (k, kind)))
            .map(|(k, kind)| {
                std::thread::spawn(move || {
                    let t = trial_at(
                        layout(k),
                        &[kind],
                        &[level],
                        kind == Kind::Chimpanzee,
                        LIMIT,
                    );
                    t[0].finish.map(secs).unwrap_or(f64::INFINITY)
                })
            })
            .collect();
        let mut times: Vec<f64> = runs.into_iter().map(|r| r.join().unwrap()).collect();
        let mean = times.iter().sum::<f64>() / times.len() as f64;
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!(
            "{:<10} best {:>6.1} s  median {:>6.1} s  mean {:>6.1} s  worst {:>6.1} s",
            level.key(),
            times[0],
            times[times.len() / 2],
            mean,
            times[times.len() - 1]
        );
    }
}
