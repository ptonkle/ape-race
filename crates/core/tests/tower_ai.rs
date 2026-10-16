//! The AI on Mape's Tower's stand-in course (the skeleton, 2026-10-15):
//! every difficulty climbs the tower, falls down the zipper and finishes,
//! proving the 3D centreline's plumbing (TW1). The course agent replaces
//! the stand-in and adds the ordering test.

use tool_core::ai::{trial_at, Difficulty};
use tool_core::fx::ONE;
use tool_core::rules::{Kind, TICKS_PER_SECOND};
use tool_core::tower::tower;

const LIMIT: u32 = 150 * TICKS_PER_SECOND as u32;

#[test]
fn every_difficulty_climbs_falls_and_finishes_the_stand_in() {
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
