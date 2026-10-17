//! The AI on the City (2026-10-12): every difficulty finishes it, and King
//! Kong beats the Sloth on average, as on the Jungle (`tests/ai.rs`).

use tool_core::ai::{think, time_trial, trial_at, Brain, Difficulty, Trial};
use tool_core::city::{city, city_seeded};
use tool_core::costume::Costume;
use tool_core::course::Course;
use tool_core::fx::ONE;
use tool_core::items::ItemSet;
use tool_core::map::Map;
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn secs(ticks: u32) -> f64 {
    ticks as f64 / TICKS_PER_SECOND as f64
}

const LIMIT: u32 = 150 * TICKS_PER_SECOND as u32;

/// The City and seven more layouts of it.
fn layouts() -> Vec<Course> {
    (0..8u64)
        .map(|k| {
            if k == 0 {
                city()
            } else {
                city_seeded(0xC17E_5EED_0001 + k * 7919)
            }
        })
        .collect()
}

/// The AI is there to show the map can be finished (SECOND-ORDER DM-3):
/// over eight layouts (one Chimpanzee, Equal Stats) every race at every
/// level finishes inside the limit (150 s), and King Kong's mean finish
/// is faster than the Sloth's. No spacing between the levels is asked.
#[test]
fn city_every_level_finishes_and_king_kong_beats_the_sloth() {
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
                                "{} finishes city layout {k} (at {} m)",
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

/// Every kind of ape, at the easiest and the best level, finishes the City
/// (its own stats, not Equal Stats).
#[test]
fn every_kind_finishes_the_city_at_the_easiest_and_the_best_level() {
    let runs: Vec<_> = [Difficulty::Sloth, Difficulty::KingKong]
        .into_iter()
        .map(|level| {
            std::thread::spawn(move || {
                for k in Kind::ALL {
                    let t = trial_at(city(), &[k], &[level], false, LIMIT);
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

/// Four AI apes racing together, with items, all finish the City.
#[test]
fn four_ai_apes_with_items_finish_the_city() {
    let kinds = [Kind::Gibbon, Kind::Gorilla, Kind::Lemur, Kind::Orangutan];
    let levels = [
        Difficulty::Sloth,
        Difficulty::Sloth,
        Difficulty::Ape,
        Difficulty::KingKong,
    ];
    let t = trial_at(city(), &kinds, &levels, false, LIMIT);
    for r in &t {
        assert!(
            r.finish.is_some(),
            "{:?} stopped at {} m",
            r.kind,
            r.progress / ONE
        );
    }
}

/// TRIAGE T90: on the City an AI held past the give-up time lets go even
/// when every release it finds turns out a dead end. In the race above the
/// Sloth had whirled on vine after vine (10.3 s on one, no give-ups) and
/// did not finish once the Hook towed it onto another line.
#[test]
fn a_city_ai_overdue_on_a_vine_gives_up_with_releases_queued() {
    let kinds = [Kind::Gibbon, Kind::Gorilla, Kind::Lemur, Kind::Orangutan];
    let entries: Vec<Entry> = kinds
        .iter()
        .map(|&kind| Entry {
            name: String::new(),
            kind,
            driver: Driver::Ai,
            palette: 0,
            costume: Costume::Own,
        })
        .collect();
    let mut race = Race::new(city(), &entries, false, 7);
    race.items = ItemSet::ALL;
    // four seats, three levels: with the Monkey removed (DM-5) this order
    // still overdues the Sloth (3 give-ups; none with the T90 fix undone)
    let mut brains: Vec<Brain> = [
        Difficulty::Sloth,
        Difficulty::Ape,
        Difficulty::KingKong,
        Difficulty::Sloth,
    ]
    .iter()
    .enumerate()
    .map(|(i, &d)| Brain::new(d, 0xA9E5 + i as u64))
    .collect();
    let (mut held, mut since, mut longest) = (None, 0, 0);
    while race.clock < LIMIT && race.phase != Phase::Done {
        let c: Vec<_> = (0..4).map(|i| think(&race, i, &mut brains[i])).collect();
        race.step(&c);
        let now = race.racers[0].ape.swinging();
        if now != held {
            held = now;
            since = race.clock;
        }
        if held.is_some() {
            longest = longest.max(race.clock - since);
        }
    }
    assert!(brains[0].give_ups > 0, "the Sloth never gave up on a vine");
    assert!(
        longest < 10 * TICKS_PER_SECOND as u32,
        "the Sloth held one vine {:.2} s",
        secs(longest)
    );
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

/// The City's medal report: King Kong alone as a Time Trial runs, every
/// kind and Equal Stats, against the City's medal times.
#[test]
#[ignore]
fn city_time_trial_report() {
    let mut row = Vec::new();
    for k in Kind::ALL {
        row.push(time_trial(city(), k, false, LIMIT));
    }
    row.push(time_trial(city(), Kind::Chimpanzee, true, LIMIT));
    report("city time trial (last: equal stats)", &row);
    let medals = tool_core::trial::medals(Map::City).map(secs);
    println!("medals bronze / silver / gold / author: {medals:.2?}");
}

/// Finish times per difficulty over the eight City layouts, three kinds
/// each: for tuning.
#[test]
#[ignore]
fn city_difficulty_report() {
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
