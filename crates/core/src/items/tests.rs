use super::*;
use crate::ape::Mode;
use crate::course::{ground, Course};
use crate::fx::int;
use crate::race::{Entry, Phase};
use crate::rules::Kind;

/// A running race on a straight test course (+z), one vine 50 m along, with
/// `kinds.len()` apes all driven here, hanging in the air side by side.
fn race_of(kinds: &[Kind], equal: bool) -> Race {
    let course = Course::test_line(&[(int(50), ground(0, int(50)) + int(30), int(12))]);
    let entries: Vec<Entry> = kinds
        .iter()
        .enumerate()
        .map(|(i, k)| Entry {
            name: format!("P{i}"),
            kind: *k,
            driver: Driver::Local,
        })
        .collect();
    let mut r = Race::new(course, &entries, equal, 7);
    r.phase = Phase::Running;
    for i in 0..r.racers.len() {
        put(&mut r, i, int(i as i32 * 10), int(20));
    }
    r
}

fn race(n: usize) -> Race {
    race_of(&[Kind::Chimpanzee; 4][..n], false)
}

/// Puts ape `i` in the air at (x, height above the ground, z) with its
/// progress set to `z`.
fn put_at(r: &mut Race, i: usize, x: i32, z: i32, height: i32) {
    let a = &mut r.racers[i].ape;
    a.pos = v3(x, ground(x, z) + height, z);
    a.vel = V3::ZERO;
    a.mode = Mode::Flying;
    r.racers[i].progress = z;
}

fn put(r: &mut Race, i: usize, x: i32, height: i32) {
    put_at(r, i, x, int(100), height);
}

fn hits(r: &Race) -> Vec<(usize, usize, Item)> {
    r.events
        .iter()
        .filter_map(|e| match e {
            Event::Hit { who, by, item } => Some((*who, *by, *item)),
            _ => None,
        })
        .collect()
}

fn press() -> Controls {
    Controls {
        item: true,
        ..Controls::default()
    }
}

/// Steps the items only (the apes stay where the test put them) until a
/// hit or `ticks` run out.
fn fly_until_hit(r: &mut Race, ticks: i32) {
    for _ in 0..ticks {
        step(r);
        if !hits(r).is_empty() {
            return;
        }
    }
}

#[test]
fn a_crate_gives_one_item_and_holding_one_blocks_a_second() {
    let mut r = race(1);
    let at = r.racers[0].ape.middle();
    r.course.crates = vec![at, at.add(v3(0, 0, cm(50)))];
    r.crates = vec![0, 0];
    step(&mut r);
    let got = r.racers[0].item.expect("picked up");
    assert_eq!(r.events, vec![Event::Pickup { who: 0, item: got }]);
    // one crate taken, the other still there: holding an item blocks it
    assert_eq!(r.crates.iter().filter(|c| **c > 0).count(), 1);
    r.events.clear();
    step(&mut r);
    assert_eq!(r.racers[0].item, Some(got));
    assert!(r.events.is_empty(), "{:?}", r.events);
    assert_eq!(r.crates.iter().filter(|c| **c == 0).count(), 1);
}

#[test]
fn using_an_item_consumes_it() {
    let mut r = race(1);
    r.racers[0].item = Some(Item::Boost);
    use_item(&mut r, 0, &press());
    assert_eq!(r.racers[0].item, None);
    assert_eq!(
        r.events,
        vec![Event::Used {
            who: 0,
            item: Item::Boost
        }]
    );
    r.events.clear();
    use_item(&mut r, 0, &press());
    assert!(r.events.is_empty());
}

#[test]
fn a_crate_comes_back_after_its_respawn_time() {
    let mut r = race(1);
    r.course.crates = vec![r.racers[0].ape.middle()];
    r.crates = vec![0];
    step(&mut r);
    assert!(r.racers[0].item.is_some());
    for _ in 0..CRATE_RESPAWN - 1 {
        r.racers[0].item = None;
        step(&mut r);
        assert_eq!(r.racers[0].item, None, "came back early");
    }
    step(&mut r);
    assert!(r.racers[0].item.is_some(), "did not come back");
}

#[test]
fn leaders_roll_attacks_and_those_behind_roll_catch_ups() {
    let mut d = fx::Dice(3);
    let count = |d: &mut fx::Dice, place| {
        let mut c = [0; 4];
        for _ in 0..4000 {
            c[roll(d, place, 4) as usize] += 1;
        }
        c
    };
    let first = count(&mut d, 1);
    let last = count(&mut d, 4);
    // Banana, Stick: more for the leader; Hook, Boost: more for the last
    for k in 0..2 {
        assert!(first[k] > last[k] + 300, "item {k}: {first:?} {last:?}");
        assert!(
            last[k + 2] > first[k + 2] + 300,
            "item {}: {first:?} {last:?}",
            k + 2
        );
    }
    assert!(first.iter().chain(last.iter()).all(|c| *c > 0));
}

/// Ape 0 throws a Banana straight at ape 1, 10 m away along +x.
fn banana_at_ape_one(r: &mut Race) {
    r.racers[0].item = Some(Item::Banana);
    let c = Controls {
        yaw: fx::QUARTER,
        ..press()
    };
    use_item(r, 0, &c);
    assert_eq!(r.shots.len(), 1);
    fly_until_hit(r, 120);
}

#[test]
fn a_banana_hit_cuts_speed_and_stuns() {
    let mut r = race(2);
    r.racers[1].ape.vel = v3(0, 0, int(20));
    banana_at_ape_one(&mut r);
    assert_eq!(hits(&r), vec![(1, 0, Item::Banana)]);
    let ape = &r.racers[1].ape;
    assert!(ape.speed() < int(20) * 7 / 10, "speed {}", ape.speed());
    assert!(ape.stun > 0);
    assert!(r.shots.is_empty(), "the banana is used up");
}

#[test]
fn a_banana_that_misses_falls_and_is_removed() {
    let mut r = race(2);
    r.racers[0].item = Some(Item::Banana);
    // thrown the other way, away from ape 1
    let c = Controls {
        yaw: -fx::QUARTER,
        ..press()
    };
    use_item(&mut r, 0, &c);
    for _ in 0..BANANA_LIFE {
        step(&mut r);
    }
    assert!(hits(&r).is_empty());
    assert!(r.shots.is_empty());
}

#[test]
fn a_stick_knocks_a_swinging_ape_off_its_vine_and_misses_one_out_of_range() {
    let mut r = race(3);
    put(&mut r, 1, int(2), int(20));
    put(&mut r, 2, int(-6), int(20));
    for i in 1..3 {
        r.racers[i].ape.mode = Mode::Swinging {
            vine: 0,
            rope: int(10),
        };
        r.racers[i].ape.vel = v3(0, 0, int(15));
    }
    r.racers[0].item = Some(Item::Stick);
    use_item(&mut r, 0, &press());
    assert_eq!(hits(&r), vec![(1, 0, Item::Stick)]);
    assert_eq!(r.racers[1].ape.mode, Mode::Flying);
    assert!(r.racers[1].ape.vel.z < int(15));
    assert!(r.racers[1].ape.vel.x > 0, "shoved away from the swinger");
    assert!(r.racers[2].ape.swinging().is_some());
    assert_eq!(r.racers[2].ape.vel, v3(0, 0, int(15)));
}

#[test]
fn a_hook_takes_the_nearest_ape_ahead_not_one_behind_and_pulls_it_back() {
    let mut r = race(4);
    put_at(&mut r, 0, 0, int(100), int(20));
    // behind and closest; ahead and near; ahead and further
    put_at(&mut r, 1, 0, int(95), int(20));
    put_at(&mut r, 2, int(3), int(120), int(20));
    put_at(&mut r, 3, 0, int(130), int(20));
    for i in 1..4 {
        r.racers[i].ape.vel = v3(0, 0, int(15));
    }
    assert_eq!(hook_target(&r, 0), Some(2));
    r.racers[0].item = Some(Item::Hook);
    use_item(&mut r, 0, &press());
    assert_eq!(r.shots[0].target, Some(2));
    fly_until_hit(&mut r, HOOK_LIFE);
    assert_eq!(hits(&r), vec![(2, 0, Item::Hook)]);
    assert!(
        r.racers[2].ape.vel.z < 0,
        "pulled back: {:?}",
        r.racers[2].ape.vel
    );
    assert!(r.racers[2].ape.stun > 0);
    assert!(r.racers[0].ape.vel.z > int(5), "the user is pulled forward");
    assert_eq!(r.racers[3].ape.vel, v3(0, 0, int(15)));
    assert!(r.shots.is_empty());
}

#[test]
fn a_hook_with_no_one_ahead_in_range_is_used_up_and_expires() {
    let mut r = race(2);
    put_at(&mut r, 0, 0, int(100), int(20));
    put_at(&mut r, 1, 0, int(100) + HOOK_RANGE + int(5), int(20));
    r.racers[0].item = Some(Item::Hook);
    use_item(&mut r, 0, &press());
    assert_eq!(r.racers[0].item, None);
    assert_eq!(r.shots[0].target, None);
    for _ in 0..HOOK_LIFE {
        step(&mut r);
    }
    assert!(hits(&r).is_empty());
    assert!(r.shots.is_empty());
}

#[test]
fn a_speed_boost_adds_speed_along_the_velocity() {
    let mut r = race(1);
    r.racers[0].ape.vel = v3(int(3), 0, int(4));
    r.racers[0].item = Some(Item::Boost);
    // the camera looks elsewhere: the boost follows the motion, not the aim
    let c = Controls {
        yaw: fx::HALF,
        ..press()
    };
    use_item(&mut r, 0, &c);
    let v = r.racers[0].ape.vel;
    assert!((v.len() - (int(5) + BOOST_GAIN)).abs() < ONE / 50, "{v:?}");
    assert!(
        (mul(v.x, int(4)) - mul(v.z, int(3))).abs() < ONE / 20,
        "{v:?}"
    );
    // at rest it goes where the camera looks
    r.racers[0].ape.vel = V3::ZERO;
    r.racers[0].item = Some(Item::Boost);
    use_item(&mut r, 0, &c);
    assert!(r.racers[0].ape.vel.z < -(BOOST_GAIN - ONE / 10));
}

#[test]
fn weight_scales_hits_except_under_equal_stats() {
    let kept = |equal: bool, kind: Kind| {
        let mut r = race_of(&[Kind::Chimpanzee, kind], equal);
        r.racers[1].ape.vel = v3(0, 0, int(20));
        banana_at_ape_one(&mut r);
        assert_eq!(hits(&r).len(), 1);
        (r.racers[1].ape.speed(), r.racers[1].ape.stun)
    };
    let gorilla = kept(false, Kind::Gorilla);
    let lemur = kept(false, Kind::Lemur);
    assert!(
        gorilla.0 > lemur.0 && gorilla.1 < lemur.1,
        "{gorilla:?} {lemur:?}"
    );
    let gorilla = kept(true, Kind::Gorilla);
    let lemur = kept(true, Kind::Lemur);
    assert_eq!(gorilla, lemur);
    // the average ape still feels the hit
    assert!(gorilla.0 < int(20) * 7 / 10 && gorilla.1 > 0);
}

#[test]
fn a_remote_ape_is_not_changed_here_but_its_hit_is_reported() {
    let mut r = race(2);
    r.racers[1].driver = Driver::Remote;
    r.racers[1].ape.vel = v3(0, 0, int(20));
    let before = r.racers[1].ape.clone();
    banana_at_ape_one(&mut r);
    assert_eq!(hits(&r), vec![(1, 0, Item::Banana)]);
    assert_eq!(r.racers[1].ape, before);
    // apply_hit is for apes this browser moves: a Remote one is left alone
    apply_hit(&mut r, 1, 0, Item::Banana);
    assert_eq!(r.racers[1].ape, before);
}

#[test]
fn a_hit_from_another_browser_applies_to_a_local_ape() {
    let mut r = race(2);
    r.racers[0].driver = Driver::Remote;
    r.racers[1].ape.mode = Mode::Swinging {
        vine: 0,
        rope: int(10),
    };
    r.racers[1].ape.vel = v3(0, 0, int(20));
    apply_hit(&mut r, 1, 0, Item::Stick);
    assert_eq!(hits(&r), vec![(1, 0, Item::Stick)]);
    assert_eq!(r.racers[1].ape.mode, Mode::Flying);
    assert!(r.racers[1].ape.stun > 0);
    assert!(r.racers[1].ape.vel.z < int(20));
}

#[test]
fn crates_vanish_while_they_respawn_and_shots_are_drawn() {
    let mut r = race(2);
    r.course.crates = vec![v3(0, int(-50), 0)];
    r.crates = vec![0];
    let mut b = Batch::new();
    draw(&mut b, &r);
    let with_crate = b.count();
    assert!(with_crate > 0);
    r.crates = vec![5];
    let mut b = Batch::new();
    draw(&mut b, &r);
    assert_eq!(b.count(), 0);
    r.racers[0].item = Some(Item::Banana);
    use_item(&mut r, 0, &press());
    let mut b = Batch::new();
    draw(&mut b, &r);
    assert!(b.count_of(Mesh::Cyl) > 0);
}
