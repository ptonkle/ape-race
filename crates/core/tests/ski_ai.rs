//! The AI on the Ski Slope (2026-10-13): every difficulty finishes it, and King
//! Kong beats the Sloth on average, as on the Jungle (`tests/ai.rs`).

use tool_core::ai::{time_trial, trial_at, Difficulty, Trial};
use tool_core::course::Course;
use tool_core::fx::ONE;
use tool_core::map::Map;
use tool_core::rules::{Kind, TICKS_PER_SECOND};
use tool_core::ski::{ski, ski_seeded};

fn secs(ticks: u32) -> f64 {
    ticks as f64 / TICKS_PER_SECOND as f64
}

const LIMIT: u32 = 150 * TICKS_PER_SECOND as u32;

/// The Ski Slope and seven more layouts of it.
fn layouts() -> Vec<Course> {
    (0..8u64)
        .map(|k| {
            if k == 0 {
                ski()
            } else {
                ski_seeded(0x5C1_5EED_0001 + k * 7919)
            }
        })
        .collect()
}

/// The AI is there to show the map can be finished (SECOND-ORDER DM-3):
/// over eight layouts (one Chimpanzee, Equal Stats) every race at every
/// level finishes inside the limit (150 s), and King Kong's mean finish
/// is faster than the Sloth's. No spacing between the levels is asked.
#[test]
fn ski_every_level_finishes_and_king_kong_beats_the_sloth() {
    let runs: Vec<_> = Difficulty::ALL
        .into_iter()
        .map(|level| {
            std::thread::spawn(move || {
                layouts()
                    .into_iter()
                    .enumerate()
                    .map(|(k, course)| {
                        let t = trial_at(course, &[Kind::Chimpanzee], &[level], true, LIMIT);
                        let f = t[0].finish.unwrap_or_else(|| {
                            panic!(
                                "{} finishes ski layout {k} (at {} m)",
                                level.key(),
                                t[0].progress / ONE
                            )
                        });
                        secs(f)
                    })
                    .collect::<Vec<f64>>()
            })
        })
        .collect();
    let mut means = Vec::new();
    for (level, run) in Difficulty::ALL.into_iter().zip(runs) {
        let row = run.join().expect("every race finishes");
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

/// Every kind of ape, at the easiest and the best level, finishes the Ski Slope
/// (its own stats, not Equal Stats).
#[test]
fn every_kind_finishes_the_ski_slope_at_the_easiest_and_the_best_level() {
    let runs: Vec<_> = [Difficulty::Sloth, Difficulty::KingKong]
        .into_iter()
        .map(|level| {
            std::thread::spawn(move || {
                for k in Kind::ALL {
                    let t = trial_at(ski(), &[k], &[level], false, LIMIT);
                    assert!(
                        t[0].finish.is_some(),
                        "{} {k:?} stopped at {} m",
                        level.key(),
                        t[0].progress / ONE
                    );
                }
            })
        })
        .collect();
    for r in runs {
        r.join().expect("every kind finishes");
    }
}

/// Four AI apes racing together, with items, all finish the Ski Slope.
#[test]
fn four_ai_apes_with_items_finish_the_ski_slope() {
    let kinds = [Kind::Gibbon, Kind::Gorilla, Kind::Lemur, Kind::Orangutan];
    let levels = [
        Difficulty::Sloth,
        Difficulty::Sloth,
        Difficulty::Ape,
        Difficulty::KingKong,
    ];
    let t = trial_at(ski(), &kinds, &levels, false, LIMIT);
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

/// The Ski Slope's medal report: King Kong alone as a Time Trial runs, every
/// kind and Equal Stats, against the Ski Slope's medal times.
#[test]
#[ignore]
fn ski_time_trial_report() {
    let mut row = Vec::new();
    for k in Kind::ALL {
        row.push(time_trial(ski(), k, false, LIMIT));
    }
    row.push(time_trial(ski(), Kind::Chimpanzee, true, LIMIT));
    report("ski time trial (last: equal stats)", &row);
    let medals = tool_core::trial::medals(Map::Ski).map(secs);
    println!("medals bronze / silver / gold / author: {medals:.2?}");
}

/// Finish times per difficulty over the eight Ski Slope layouts, three kinds
/// each: for tuning.
#[test]
#[ignore]
fn ski_difficulty_report() {
    for level in Difficulty::ALL {
        let mut times = Vec::new();
        for course in layouts() {
            for kind in [Kind::Chimpanzee, Kind::Gorilla, Kind::Gibbon] {
                let t = trial_at(
                    course.clone(),
                    &[kind],
                    &[level],
                    kind == Kind::Chimpanzee,
                    LIMIT,
                );
                times.push(t[0].finish.map(secs).unwrap_or(f64::INFINITY));
            }
        }
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
