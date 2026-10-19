//! The AI on the tower map: every difficulty climbs the tower, falls down
//! the zipper and finishes (TW1), and King Kong beats the Sloth on average
//! over sixteen layouts, as on the other maps (TC5, DM-3).

use tool_core::ai::{time_trial, trial_at, Difficulty, Trial};
use tool_core::course::Course;
use tool_core::fx::ONE;
use tool_core::map::Map;
use tool_core::rules::{Kind, TICKS_PER_SECOND};
use tool_core::tower::{tower, tower_seeded, Leg};

const LIMIT: u32 = 150 * TICKS_PER_SECOND as u32;

/// Every level finishes the course inside the limit, each reported before
/// any is judged, with each level's time on each leg (the plan: about
/// 10 s under each bridge).
#[test]
fn every_difficulty_climbs_falls_and_finishes() {
    let runs: Vec<_> = Difficulty::ALL
        .into_iter()
        .map(|level| std::thread::spawn(move || (level, leg_times(0, level))))
        .collect();
    let all: Vec<_> = runs
        .into_iter()
        .map(|r| r.join().expect("the race runs"))
        .collect();
    for (level, (finish, progress, legs)) in &all {
        let legs: Vec<String> = legs
            .iter()
            .map(|(leg, at)| format!("{leg:?} {:.1}", secs(*at)))
            .collect();
        println!(
            "{:<10} {}  reached {} m  legs: {}",
            level.key(),
            finish
                .map(|f| format!("{:.1} s", secs(f)))
                .unwrap_or("-".into()),
            progress / ONE,
            legs.join(", ")
        );
    }
    let stuck: Vec<String> = all
        .iter()
        .filter(|(_, (f, _, _))| f.is_none())
        .map(|(level, (_, p, _))| format!("{} (stuck at {} m)", level.key(), p / ONE))
        .collect();
    assert!(
        stuck.is_empty(),
        "every level finishes the tower map: {}",
        stuck.join(", ")
    );
    let times: Vec<u32> = all.iter().map(|(_, (f, _, _))| f.unwrap()).collect();
    assert!(
        times.windows(2).all(|w| w[0] > w[1]),
        "the levels finish in order: {times:?}"
    );
}

/// One AI ape of `level` on layout `k`: its finish, its progress, and the
/// tick it first reached each leg.
fn leg_times(k: u64, level: Difficulty) -> (Option<u32>, i32, Vec<(Leg, u32)>) {
    use tool_core::ai::Brain;
    use tool_core::race::{Driver, Entry, Race};
    let entry = Entry {
        name: String::new(),
        kind: Kind::Chimpanzee,
        driver: Driver::Ai,
        palette: 0,
        costume: tool_core::costume::Costume::Own,
    };
    let mut race = Race::new(layout(k), &[entry], true, 7);
    let mut brains = vec![Brain::new(level, 0xA9E5)];
    let mut legs: Vec<(Leg, u32)> = Vec::new();
    let mut best = 0;
    while race.tick < LIMIT && race.racers[0].finish.is_none() {
        tool_core::replay::drive(&mut race, &mut brains, usize::MAX, Default::default());
        let p = race.racers[0].progress;
        best = best.max(p);
        let leg = race.course.tower.as_ref().unwrap().leg_at(p);
        if legs.last().is_none_or(|l| l.0 < leg) {
            legs.push((leg, race.clock));
        }
    }
    (race.racers[0].finish, best, legs)
}

/// Layout `k` of the tower map (0 is the course everyone races; the seeds
/// are `examples/tower_report.rs`'s).
fn layout(k: u64) -> Course {
    if k == 0 {
        tower()
    } else {
        tower_seeded(0x70E5_0000_0001 + k * 7919)
    }
}

/// The AI is there to show the map can be finished (SECOND-ORDER DM-3):
/// over sixteen layouts (one Chimpanzee, Equal Stats) every race at every
/// level finishes inside the limit (150 s), and King Kong's mean finish
/// is faster than the Sloth's. No spacing between the levels is asked.
#[test]
fn tower_every_level_finishes_and_king_kong_beats_the_sloth() {
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
    let (sloth, king_kong) = (means[0], means[means.len() - 1]);
    assert!(
        king_kong < sloth,
        "king_kong ({king_kong:.1} s) is not faster than sloth ({sloth:.1} s) on average"
    );
}

/// The chase camera follows an Ape up the climb and down the zipper:
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
    let mut brains = vec![Brain::new(Difficulty::Ape, 0xA9E5)];
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
    assert!(race.racers[0].finish.is_some(), "the Ape finishes");
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

/// The tower map's medal report: King Kong alone as a Time Trial runs,
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

/// Where an AI ape of `level` crosses the mirror plane on layout `k`
/// (first crossing after the climb's last third), relative to the tear's
/// middle: (up, across) in m.
fn tear_crossing(k: u64, level: Difficulty) -> (i32, i32, i32) {
    use tool_core::ai::Brain;
    use tool_core::race::{Driver, Entry, Race};
    let entry = Entry {
        name: String::new(),
        kind: Kind::Chimpanzee,
        driver: Driver::Ai,
        palette: 0,
        costume: tool_core::costume::Costume::Own,
    };
    let mut race = Race::new(layout(k), &[entry], true, 7);
    let s = race.course.tower.as_ref().unwrap().portal;
    let mut brains = vec![Brain::new(level, 0xA9E5 + k)];
    let side = |x: i32| (x - s.tear.x).signum();
    let mut was = side(race.racers[0].ape.pos.x);
    let from = race.course.length / 3;
    while race.tick < LIMIT && race.racers[0].finish.is_none() {
        tool_core::replay::drive(&mut race, &mut brains, usize::MAX, Default::default());
        let p = race.racers[0].ape.pos;
        let now = side(p.x);
        if race.racers[0].progress > from && now != was && now != 0 && was != 0 {
            let (up, across) = (p.y - s.tear.y, p.z - s.tear.z);
            let (hh, hw) = s.tear_half;
            let clear = tool_core::towermodels::portal_clear(2 * hw, 2 * hh, up + hh);
            return (up, across, clear);
        }
        if now != 0 {
            was = now;
        }
    }
    panic!("{} never crosses the tear on layout {k}", level.key());
}

/// The person's brief: the eldritch banana is on the other side of the
/// tear, "which you fly through". Every AI level's line passes through
/// the tear's opening (TA8), over a few layouts.
#[test]
fn every_level_flies_through_the_tear() {
    let s = tower().tower.as_ref().unwrap().portal;
    let (hh, hw) = s.tear_half;
    let runs: Vec<_> = Difficulty::ALL
        .into_iter()
        .flat_map(|level| (0..16u64).map(move |k| (level, k)))
        .map(|(level, k)| std::thread::spawn(move || (level, k, tear_crossing(k, level))))
        .collect();
    let all: Vec<_> = runs
        .into_iter()
        .map(|r| r.join().expect("the race runs"))
        .collect();
    let m = |v: i32| v as f64 / ONE as f64;
    for (level, k, (up, across, clear)) in &all {
        println!(
            "{:<10} layout {k:>2}: up {:+.1} m, across {:+.1} m (clear ±{:.1})",
            level.key(),
            m(*up),
            m(*across),
            m(*clear)
        );
    }
    for (level, k, (up, across, clear)) in all {
        assert!(
            across.abs() < clear,
            "{} on layout {k} crosses {:+.1} m up, {:+.1} m across the tear's middle (clear ±{:.1} m there; tear ±{} by ±{} m)",
            level.key(),
            m(up),
            m(across),
            m(clear),
            hh / ONE,
            hw / ONE
        );
    }
}
