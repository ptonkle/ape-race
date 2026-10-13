//! The AI on the jungle course (PLAN P12, and P7's race length).

use tool_core::ai::{time_trial, trial, trial_at, Brain, Difficulty, Trial};
use tool_core::course::Course;
use tool_core::fx::ONE;
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn secs(ticks: u32) -> f64 {
    ticks as f64 / TICKS_PER_SECOND as f64
}

fn report(label: &str, trials: &[Trial]) {
    println!("{label}");
    for t in trials {
        println!(
            "  {:<11} finish {:>9}  at {:>5.0} m  mean {:>5.1} m/s  top {:>5.1} m/s  grabs {:>3}  landings {:>2}  ground {:>5.1} s  bananas {:>3}",
            format!("{:?}", t.kind),
            t.finish
                .map(|f| format!("{:.2} s", secs(f)))
                .unwrap_or("-".into()),
            t.progress as f64 / ONE as f64,
            t.mean_speed as f64 / ONE as f64,
            t.top_speed as f64 / ONE as f64,
            t.grabs,
            t.landings,
            secs(t.ground_ticks),
            t.bananas,
        );
    }
}

const LIMIT: u32 = 150 * TICKS_PER_SECOND as u32;

#[test]
fn one_ai_finishes_the_jungle_in_one_to_two_minutes() {
    let t = trial(Course::jungle(), &[Kind::Chimpanzee], true, LIMIT);
    report("one AI, equal stats", &t);
    let f = t[0].finish.expect("the AI finishes");
    assert!(
        (60 * TICKS_PER_SECOND as u32..=120 * TICKS_PER_SECOND as u32).contains(&f),
        "{:.2} s",
        secs(f)
    );
}

#[test]
#[ignore]
fn speed_report_every_kind_alone_and_four_together() {
    for k in Kind::ALL {
        let t = trial(Course::jungle(), &[k], false, LIMIT);
        report("alone", &t);
    }
    let t = trial(
        Course::jungle(),
        &[Kind::Gibbon, Kind::Gorilla, Kind::Lemur, Kind::Orangutan],
        false,
        LIMIT,
    );
    report("four together", &t);
}

/// The medal report (TRIAGE T29): King Kong alone as a Time Trial runs
/// (Speed Boosts only), every kind and Equal Stats, against the medal times.
#[test]
#[ignore]
fn time_trial_report() {
    let mut row = Vec::new();
    for k in Kind::ALL {
        row.push(time_trial(Course::jungle(), k, false, LIMIT));
    }
    row.push(time_trial(Course::jungle(), Kind::Chimpanzee, true, LIMIT));
    report("time trial (last: equal stats)", &row);
    let medals = tool_core::trial::JUNGLE.map(secs);
    println!("medals bronze / silver / gold / author: {medals:.2?}");
}

/// Finish times over eight layouts of the jungle (its seed and seven more),
/// three kinds each: for judging AI and layout changes beyond one course.
#[test]
#[ignore]
fn finish_times_over_course_seeds() {
    let mut times = Vec::new();
    for k in 0..8u64 {
        let course = if k == 0 {
            Course::jungle()
        } else {
            Course::jungle_seeded(0x0A9E_2ACE_0001 + k * 7919)
        };
        let mut row = Vec::new();
        for kind in [Kind::Chimpanzee, Kind::Gorilla, Kind::Gibbon] {
            let t = trial(course.clone(), &[kind], kind == Kind::Chimpanzee, LIMIT);
            row.push(t[0].finish.map(secs));
            times.push(t[0].finish.map(secs).unwrap_or(f64::INFINITY));
        }
        println!("layout {k}: {row:.1?}");
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "best {:.1} s, median {:.1} s, worst {:.1} s",
        times[0],
        times[times.len() / 2],
        times[times.len() - 1]
    );
}

/// Finish times per difficulty over eight layouts (TRIAGE T14): for tuning
/// the levels' spacing.
#[test]
#[ignore]
fn difficulty_report_over_course_seeds() {
    for level in Difficulty::ALL {
        let mut times = Vec::new();
        for k in 0..8u64 {
            let course = if k == 0 {
                Course::jungle()
            } else {
                Course::jungle_seeded(0x0A9E_2ACE_0001 + k * 7919)
            };
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

/// The jungle and seven more layouts of it.
fn layouts() -> Vec<Course> {
    (0..8u64)
        .map(|k| {
            if k == 0 {
                Course::jungle()
            } else {
                Course::jungle_seeded(0x0A9E_2ACE_0001 + k * 7919)
            }
        })
        .collect()
}

/// The difficulties (TRIAGE T14) are ordered and spaced: over eight layouts
/// (one Chimpanzee, Equal Stats), each easier level's mean finish is at
/// least `GAP` slower than the next harder's, and every race at every level
/// finishes inside the limit (150 s). One thread per level.
#[test]
fn difficulties_are_ordered_and_spaced() {
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
                                "{} finishes layout {k} (at {} m)",
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

/// TRIAGE T32: the four AI starters each take the start-row vine in front
/// of their own spot, not all the same one, and still get away cleanly.
#[test]
fn ai_starters_fan_out_onto_different_start_vines() {
    use tool_core::ai::think;
    use tool_core::race::{Driver, Entry, Race};
    let kinds = [Kind::Gibbon, Kind::Gorilla, Kind::Lemur, Kind::Orangutan];
    let entries: Vec<Entry> = kinds
        .iter()
        .map(|&kind| Entry {
            name: String::new(),
            kind,
            driver: Driver::Ai,
        })
        .collect();
    let mut race = Race::new(Course::jungle(), &entries, false, 7);
    let mut brains: Vec<Brain> = (0..4)
        .map(|i| Brain::new(Difficulty::KingKong, 0xA9E5 + i as u64))
        .collect();
    let mut first: Vec<Option<usize>> = vec![None; 4];
    for _ in 0..12 * TICKS_PER_SECOND {
        let c: Vec<_> = (0..4).map(|i| think(&race, i, &mut brains[i])).collect();
        race.step(&c);
        for (i, r) in race.racers.iter().enumerate() {
            if first[i].is_none() {
                first[i] = r.ape.swinging();
            }
        }
    }
    let mut got: Vec<usize> = first.iter().map(|f| f.expect("every AI grabs")).collect();
    got.sort();
    got.dedup();
    assert_eq!(got.len(), 4, "first vines {first:?}");
    // and twelve seconds on, every one is well on its way
    for r in &race.racers {
        assert!(r.progress > 120 * ONE, "at {} m", r.progress / ONE);
    }
}

#[test]
fn difficulty_keys_round_trip_and_the_default_is_the_best() {
    for d in Difficulty::ALL {
        assert_eq!(Difficulty::from_key(d.key()), Some(d));
    }
    assert_eq!(Difficulty::from_key("gorilla_glue"), None);
    assert_eq!(Difficulty::default(), Difficulty::KingKong);
    assert_eq!(Brain::default().difficulty, Difficulty::KingKong);
}
