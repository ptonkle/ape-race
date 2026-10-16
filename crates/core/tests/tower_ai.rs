//! The AI on Mape's Tower: every difficulty climbs the tower, falls down
//! the zipper and finishes (TW1), and the difficulties stay ordered and
//! spaced over sixteen layouts, as on the other maps (TC5).

use tool_core::ai::{time_trial, trial_at, Difficulty, Trial};
use tool_core::course::Course;
use tool_core::fx::ONE;
use tool_core::map::Map;
use tool_core::rules::{Kind, TICKS_PER_SECOND};
use tool_core::tower::{tower, tower_seeded};

const LIMIT: u32 = 150 * TICKS_PER_SECOND as u32;

#[test]
fn every_difficulty_climbs_falls_and_finishes() {
    let runs: Vec<_> = Difficulty::ALL
        .into_iter()
        .map(|level| {
            std::thread::spawn(move || {
                let t = trial_at(tower(), &[Kind::Chimpanzee], &[level], true, LIMIT);
                (level, t[0].finish, t[0].progress)
            })
        })
        .collect();
    for run in runs {
        let (level, finish, progress) = run.join().expect("the race runs");
        let f = finish.unwrap_or_else(|| {
            panic!(
                "{} finishes Mape's Tower (stuck at {} m)",
                level.key(),
                progress / ONE
            )
        });
        println!(
            "{:<10} {:.1} s",
            level.key(),
            f as f64 / TICKS_PER_SECOND as f64
        );
    }
}

/// Layout `k` of Mape's Tower (0 is the course everyone races; the seeds
/// are `examples/tower_report.rs`'s).
fn layout(k: u64) -> Course {
    if k == 0 {
        tower()
    } else {
        tower_seeded(0x70E5_0000_0001 + k * 7919)
    }
}

/// As `bay_difficulties_are_ordered_and_spaced`: over sixteen layouts of
/// Mape's Tower (one Chimpanzee, Equal Stats), each easier level's mean
/// finish is at least `GAP` slower than the next harder's, and every race
/// at every level finishes inside the limit (150 s).
#[test]
fn tower_difficulties_are_ordered_and_spaced() {
    const GAP: f64 = 6.0;
    let runs: Vec<_> = Difficulty::ALL
        .into_iter()
        .map(|level| {
            std::thread::spawn(move || {
                (0..16u64)
                    .map(|k| {
                        let t = trial_at(layout(k), &[Kind::Chimpanzee], &[level], true, LIMIT);
                        let f = t[0].finish.unwrap_or_else(|| {
                            panic!(
                                "{} finishes tower layout {k} (at {} m)",
                                level.key(),
                                t[0].progress / ONE
                            )
                        });
                        f as f64 / TICKS_PER_SECOND as f64
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

/// The chase camera follows a Monkey up the climb and down the zipper:
/// the eye stays near the ape all the way, rises with it to the tear and
/// comes back down with it.
#[test]
fn the_chase_camera_follows_up_the_climb_and_down_the_zipper() {
    use tool_core::ai::Brain;
    use tool_core::camera::Chase;
    use tool_core::race::{Driver, Entry, Race};
    use tool_core::tower::START_TOP;
    let entry = Entry {
        name: String::new(),
        kind: Kind::Chimpanzee,
        driver: Driver::Ai,
        palette: 0,
        costume: tool_core::costume::Costume::Own,
    };
    let mut race = Race::new(tower(), &[entry], true, 7);
    let mut brains = vec![Brain::new(Difficulty::Monkey, 0xA9E5)];
    let a = &race.racers[0].ape;
    let mut chase = Chase::new(a.pos, race.course.start_yaw, race.course.terrain);
    let (mut far, mut top, mut after_top) = (0, i32::MIN, i32::MAX);
    let half = race.course.length / 2;
    while race.tick < LIMIT && race.racers[0].finish.is_none() {
        tool_core::replay::drive(&mut race, &mut brains, usize::MAX, Default::default());
        let a = &race.racers[0].ape;
        let last = race.course.heading.len() - 1;
        let ahead = race.course.heading[(a.hint + 8).min(last)];
        chase.follow(a.pos, a.vel, ahead, true);
        far = far.max(chase.eye.dist(a.pos));
        top = top.max(chase.eye.y);
        if race.racers[0].progress > half + half / 2 {
            after_top = after_top.min(chase.eye.y);
        }
    }
    assert!(race.racers[0].finish.is_some(), "the Monkey finishes");
    assert!(
        far < 40 * ONE,
        "the eye strays {} m from the ape",
        far / ONE
    );
    assert!(
        top > START_TOP + 60 * ONE,
        "the eye rises to {} m",
        top / ONE
    );
    assert!(
        after_top < START_TOP + 40 * ONE,
        "the eye comes back down to {} m",
        after_top / ONE
    );
}

fn secs(ticks: u32) -> f64 {
    ticks as f64 / TICKS_PER_SECOND as f64
}

/// Mape's Tower's medal report: King Kong alone as a Time Trial runs,
/// every kind and Equal Stats, against its medal times (TM rows).
#[test]
#[ignore]
fn tower_time_trial_report() {
    let runs: Vec<_> = Kind::ALL
        .into_iter()
        .map(|k| (k, false))
        .chain([(Kind::Chimpanzee, true)])
        .map(|(k, eq)| std::thread::spawn(move || time_trial(tower(), k, eq, LIMIT)))
        .collect();
    let row: Vec<Trial> = runs.into_iter().map(|r| r.join().unwrap()).collect();
    println!("tower time trial (last: equal stats)");
    for t in &row {
        println!(
            "  {:<11} finish {:>9}  grabs {:>3}  landings {:>2}  ground {:>5.1} s",
            format!("{:?}", t.kind),
            t.finish
                .map(|f| format!("{:.2} s", secs(f)))
                .unwrap_or("-".into()),
            t.grabs,
            t.landings,
            secs(t.ground_ticks),
        );
    }
    let medals = tool_core::trial::medals(Map::Tower).map(secs);
    println!("medals bronze / silver / gold / author: {medals:.2?}");
}

/// The tower's medals follow its time trial (TM1): the best kind's run
/// (King Kong alone, Speed Boosts only) beats Gold, and the Author time
/// is just under it (inside 1.5 s).
#[test]
fn the_tower_medals_follow_its_time_trial() {
    let runs: Vec<_> = Kind::ALL
        .into_iter()
        .map(|k| std::thread::spawn(move || time_trial(tower(), k, false, LIMIT)))
        .collect();
    let best = runs
        .into_iter()
        .filter_map(|r| r.join().unwrap().finish)
        .min()
        .expect("some kind finishes");
    let target = |m| tool_core::trial::target_on(Map::Tower, m);
    use tool_core::trial::Medal;
    assert!(
        best <= target(Medal::Gold),
        "best run {:.2} s misses Gold",
        secs(best)
    );
    let author = target(Medal::Author);
    assert!(
        author <= best && best - author <= 3 * TICKS_PER_SECOND as u32 / 2,
        "Author {:.2} s is not just under the best run {:.2} s",
        secs(author),
        secs(best)
    );
}
