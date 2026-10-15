//! The AI on Sunset Ruins (2026-10-14): every difficulty finishes it, and the
//! difficulties stay ordered and spaced, as on the Jungle (`tests/ai.rs`).

use tool_core::ai::{time_trial, trial_at, Difficulty, Trial};
use tool_core::course::Course;
use tool_core::fx::ONE;
use tool_core::map::Map;
use tool_core::ruins::{ruins, ruins_seeded};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn secs(ticks: u32) -> f64 {
    ticks as f64 / TICKS_PER_SECOND as f64
}

const LIMIT: u32 = 150 * TICKS_PER_SECOND as u32;

/// Layout `k` of Sunset Ruins (0 is the course everyone races; the seeds
/// are `examples/ruins_report.rs`'s).
fn layout(k: u64) -> Course {
    if k == 0 {
        ruins()
    } else {
        ruins_seeded(0x2B1_5E75_0001 + k * 7919)
    }
}

/// Sunset Ruins and seven more layouts of it.
fn layouts() -> Vec<Course> {
    (0..8u64).map(layout).collect()
}

/// As `difficulties_are_ordered_and_spaced` on the Jungle: over eight
/// layouts of Sunset Ruins (one Chimpanzee, Equal Stats), each easier level's
/// mean finish is at least `GAP` slower than the next harder's, and every
/// race at every level finishes inside the limit (150 s).
#[test]
fn ruins_difficulties_are_ordered_and_spaced() {
    const GAP: f64 = 6.0;
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
                                "{} finishes ruins layout {k} (at {} m)",
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
    for (i, w) in means.windows(2).enumerate() {
        assert!(
            w[0] >= w[1] + GAP,
            "{} ({:.1} s) is not {GAP} s slower than {} ({:.1} s)",
            Difficulty::ALL[i].key(),
            w[0],
            Difficulty::ALL[i + 1].key(),
            w[1]
        );
    }
}

/// Every level finishes every one of sixteen layouts of Sunset Ruins inside
/// the limit (150 s), as the Chimpanzee (Equal Stats) and as the Gorilla and
/// the Gibbon (their own stats): the difficulty report's three kinds. The
/// Sloth Gorilla once never left the river on layout 1 (RT10).
#[test]
fn every_level_finishes_sixteen_ruins_layouts() {
    let runs: Vec<_> = Difficulty::ALL
        .into_iter()
        .flat_map(|level| {
            [Kind::Chimpanzee, Kind::Gorilla, Kind::Gibbon]
                .into_iter()
                .map(move |kind| {
                    std::thread::spawn(move || {
                        let mut stuck = Vec::new();
                        for k in 0..16u64 {
                            let t = trial_at(
                                layout(k),
                                &[kind],
                                &[level],
                                kind == Kind::Chimpanzee,
                                LIMIT,
                            );
                            if t[0].finish.is_none() {
                                stuck.push(format!(
                                    "{} {kind:?} layout {k} stopped at {} m",
                                    level.key(),
                                    t[0].progress / ONE
                                ));
                            }
                        }
                        stuck
                    })
                })
        })
        .collect();
    let stuck: Vec<String> = runs
        .into_iter()
        .flat_map(|r| r.join().expect("the races run"))
        .collect();
    assert!(stuck.is_empty(), "not finished in 150 s: {stuck:?}");
}

/// Every kind of ape, at the easiest and the best level, finishes Sunset Ruins
/// (its own stats, not Equal Stats).
#[test]
fn every_kind_finishes_the_ruins_at_the_easiest_and_the_best_level() {
    let runs: Vec<_> = [Difficulty::Sloth, Difficulty::KingKong]
        .into_iter()
        .map(|level| {
            std::thread::spawn(move || {
                for k in Kind::ALL {
                    let t = trial_at(ruins(), &[k], &[level], false, LIMIT);
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

/// Four AI apes racing together, with items, all finish Sunset Ruins.
#[test]
fn four_ai_apes_with_items_finish_the_ruins() {
    let kinds = [Kind::Gibbon, Kind::Gorilla, Kind::Lemur, Kind::Orangutan];
    let levels = [
        Difficulty::Sloth,
        Difficulty::Monkey,
        Difficulty::Ape,
        Difficulty::KingKong,
    ];
    let t = trial_at(ruins(), &kinds, &levels, false, LIMIT);
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

/// Sunset Ruins' medal report: King Kong alone as a Time Trial runs, every
/// kind and Equal Stats, against its medal times.
#[test]
#[ignore]
fn ruins_time_trial_report() {
    let mut row = Vec::new();
    for k in Kind::ALL {
        row.push(time_trial(ruins(), k, false, LIMIT));
    }
    row.push(time_trial(ruins(), Kind::Chimpanzee, true, LIMIT));
    report("ruins time trial (last: equal stats)", &row);
    let medals = tool_core::trial::medals(Map::Ruins).map(secs);
    println!("medals bronze / silver / gold / author: {medals:.2?}");
}

/// Finish times per difficulty over the eight Ruins layouts, three kinds
/// each: for tuning.
#[test]
#[ignore]
fn ruins_difficulty_report() {
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
