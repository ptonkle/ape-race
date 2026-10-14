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
            palette: 0,
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

/// A test course with two vines side by side, 50 m along, each carrying a
/// crate, and `n` apes in the air far from them.
fn crate_race(n: usize) -> Race {
    let mut r = race(n);
    let mut second = r.course.vines[0];
    second.anchor = second.anchor.add(v3(int(4), 0, 0));
    r.course.vines.push(second);
    r.base_vines = 2;
    r.crate_vines = vec![0, 1];
    r.crates = vec![0, 0];
    r
}

/// Ape `i` hangs on vine `v`, as just after catching it.
fn hang_on(r: &mut Race, i: usize, v: usize) {
    let a = r.course.anchor(v, r.tick);
    let ape = &mut r.racers[i].ape;
    ape.pos = a.sub(v3(0, int(10), 0));
    ape.mode = Mode::Swinging {
        vine: v,
        rope: int(10),
    };
}

#[test]
fn catching_a_crate_vine_gives_one_item_and_holding_one_blocks_a_second() {
    let mut r = crate_race(1);
    hang_on(&mut r, 0, 0);
    step(&mut r);
    let got = r.racers[0].item.expect("picked up");
    assert_eq!(r.events, vec![Event::Pickup { who: 0, item: got }]);
    assert_eq!(r.crates, vec![CRATE_RESPAWN, 0]);
    // onto the other crate's vine: holding an item blocks it
    hang_on(&mut r, 0, 1);
    r.events.clear();
    step(&mut r);
    assert_eq!(r.racers[0].item, Some(got));
    assert!(r.events.is_empty(), "{:?}", r.events);
    assert_eq!(r.crates[1], 0);
}

#[test]
fn a_crate_hangs_under_its_vine_and_flying_into_it_takes_it_too() {
    let mut r = crate_race(1);
    let a = r.course.anchor(0, r.tick);
    let end = a.sub(v3(0, r.course.vines[0].length, 0));
    let at = crate_at(&r, 0);
    assert_eq!((at.x, at.z), (end.x, end.z));
    assert!(at.y < end.y && at.y > end.y - int(2), "{at:?} {end:?}");
    // an ape swinging on it carries it toward its hands
    hang_on(&mut r, 0, 0);
    r.racers[0].ape.pos = a.add(v3(int(6), -int(8), 0));
    let held = crate_at(&r, 0);
    assert!(held.x > a.x + int(6), "{held:?}");
    put_at(&mut r, 0, at.x, at.z, 0);
    r.racers[0].ape.pos = at.add(v3(0, BODY_DROP / 2, 0));
    step(&mut r);
    assert!(r.racers[0].item.is_some());
}

#[test]
fn the_jungle_hangs_each_crate_on_its_own_vine_near_its_spot() {
    let c = Course::jungle();
    let r = Race::new(c, &[], false, 1);
    assert!(!r.crate_vines.is_empty());
    assert_eq!(r.crate_vines.len(), r.course.crates.len());
    assert_eq!(r.crates.len(), r.crate_vines.len());
    for (k, &v) in r.crate_vines.iter().enumerate() {
        assert!(!r.crate_vines[..k].contains(&v), "vine {v} twice");
        let d = r.course.vines[v]
            .anchor
            .flat()
            .sub(r.course.crates[k].flat())
            .len();
        assert!(d <= CRATE_VINE_NEAR, "crate {k}: {d}");
    }
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
    let mut r = crate_race(1);
    hang_on(&mut r, 0, 0);
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
    let count = |d: &mut fx::Dice, place, low| {
        let mut c = [0; 5];
        for _ in 0..4000 {
            c[roll(d, place, 4, low) as usize] += 1;
        }
        c
    };
    let first = count(&mut d, 1, false);
    let last = count(&mut d, 4, false);
    // Banana, Stick: more for the leader; Hook, Boost, Vine: more for the last
    for k in 0..2 {
        assert!(first[k] > last[k] + 300, "item {k}: {first:?} {last:?}");
    }
    for k in 2..5 {
        assert!(last[k] > first[k] + 200, "item {k}: {first:?} {last:?}");
    }
    assert!(first.iter().chain(last.iter()).all(|c| *c > 0));
    // an ape on or near the ground rolls far more Vines
    let low = count(&mut d, 1, true);
    let vine = Item::Vine as usize;
    assert!(low[vine] > first[vine] * 4, "{low:?} {first:?}");
}

#[test]
fn a_time_trial_crate_gives_only_speed_boosts() {
    let mut r = crate_race(1);
    r.items = ItemSet::only(Item::Boost);
    let dice = r.dice;
    for _ in 0..50 {
        assert_eq!(roll_for(&mut r, 0), Some(Item::Boost));
    }
    // one item left: no throw of the dice (old Time Trial replays hold)
    assert_eq!(r.dice, dice);
    // the set reaches the crate
    hang_on(&mut r, 0, 0);
    step(&mut r);
    assert_eq!(r.racers[0].item, Some(Item::Boost));
    // every item: a low ape gets a mix
    r.items = ItemSet::ALL;
    put_at(&mut r, 0, 0, int(100), int(1));
    let got: Vec<Item> = (0..50).filter_map(|_| roll_for(&mut r, 0)).collect();
    assert!(got.iter().any(|i| *i != Item::Boost));
    assert!(got.contains(&Item::Vine), "{got:?}");
}

#[test]
fn items_left_out_of_the_set_never_come_out() {
    let mut d = fx::Dice(11);
    let allowed = ItemSet::ALL
        .with(Item::Stick, false)
        .with(Item::Hook, false)
        .with(Item::Boost, false);
    let mut c = [0; 5];
    for _ in 0..3000 {
        let item = roll_among(&mut d, 1, 4, false, allowed).expect("an item");
        c[item as usize] += 1;
    }
    assert_eq!((c[1], c[2], c[3]), (0, 0, 0), "{c:?}");
    // the rest keep their odds: the leader high up gets Bananas (weight 40)
    // far more than Vines (4)
    assert!(c[0] > c[4] * 5 && c[4] > 0, "{c:?}");
    // and a low ape still gets many more Vines
    let mut low = [0; 5];
    for _ in 0..3000 {
        low[roll_among(&mut d, 1, 4, true, allowed).unwrap() as usize] += 1;
    }
    assert!(low[4] > c[4] * 4, "{low:?} {c:?}");
    // every item allowed is the same throw as before the sets
    let (mut a, mut b) = (fx::Dice(5), fx::Dice(5));
    for place in 1..=4 {
        for low in [false, true] {
            assert_eq!(
                roll_among(&mut a, place, 4, low, ItemSet::ALL),
                Some(roll(&mut b, place, 4, low))
            );
        }
    }
}

#[test]
fn with_no_items_the_crates_give_nothing() {
    // (their flowers are not drawn either: `scene` test
    // item_vines_carry_flowers_up_their_middle)
    let mut r = crate_race(1);
    r.items = ItemSet::NONE;
    assert_eq!(roll_for(&mut r, 0), None);
    hang_on(&mut r, 0, 0);
    step(&mut r);
    assert_eq!(r.racers[0].item, None);
    assert!(r.events.is_empty(), "{:?}", r.events);
    assert_eq!(r.crates, vec![0, 0]);
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
    put(&mut r, 2, int(-9), int(20));
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
fn crates_are_not_drawn_and_shots_are() {
    // TRIAGE T46: an item vine is marked by flowers, drawn with the vines
    // (`scene::item_flowers`); there is no crate to draw
    let mut r = race(2);
    assert_eq!(
        r.crate_vines,
        vec![0],
        "the test course: a crate on its one vine"
    );
    let mut b = Batch::new();
    draw(&mut b, &r);
    assert_eq!(b.count(), 0);
    r.racers[0].item = Some(Item::Banana);
    use_item(&mut r, 0, &press());
    let mut b = Batch::new();
    draw(&mut b, &r);
    assert!(b.count_of(Mesh::Cyl) > 0);
}

// ---- TRIAGE T10: the Banana homes ----

/// Ape 0 throws a Banana along +z (straight ahead) while ape 1 is `side`
/// metres off to the side, `ahead` metres along, and moving along +z.
fn banana_past(side: i32, ahead: i32) -> Race {
    let mut r = race(2);
    put_at(&mut r, 0, 0, int(100), int(20));
    put_at(&mut r, 1, int(side), int(100 + ahead), int(20));
    r.racers[1].ape.vel = v3(0, 0, int(15));
    r.racers[0].item = Some(Item::Banana);
    use_item(&mut r, 0, &press());
    r
}

#[test]
fn a_banana_homes_on_an_ape_inside_the_cone_of_the_aim() {
    // 25 m along and 9 m aside: 20 degrees off the aim, a sure miss thrown
    // straight
    let mut r = banana_past(9, 25);
    assert_eq!(r.shots[0].target, Some(1));
    fly_until_hit(&mut r, BANANA_LIFE);
    assert_eq!(hits(&r), vec![(1, 0, Item::Banana)]);
    // outside the cone (45 degrees off) it is thrown at no one, and misses
    let mut r = banana_past(20, 20);
    assert_eq!(r.shots[0].target, None);
    fly_until_hit(&mut r, BANANA_LIFE);
    assert!(hits(&r).is_empty());
}

#[test]
fn a_homing_banana_turns_no_faster_than_its_turn_rate() {
    let mut r = banana_past(9, 25);
    let limit = fx::cos(mul(BANANA_TURN, DT)) - ONE / 2000;
    let mut turned = false;
    for _ in 0..30 {
        let before = r.shots[0].vel.norm();
        step(&mut r);
        let after = r.shots[0].vel.norm();
        assert!(before.dot(after) >= limit, "{before:?} {after:?}");
        turned |= before.dot(after) < ONE - ONE / 10_000;
    }
    assert!(turned, "it does turn");
}

// ---- TRIAGE T11: the Stick's reach ----

#[test]
fn the_stick_reaches_six_metres_and_lights_up_who_it_would_hit() {
    let mut r = race(3);
    put(&mut r, 0, 0, int(20));
    put(&mut r, 1, int(6), int(20));
    put(&mut r, 2, int(-8), int(20));
    assert_eq!(stick_targets(&r, 0), vec![1]);
    // the ring and the marker show only while the player holds a Stick
    let drawn = |r: &Race| {
        let mut b = Batch::new();
        draw(&mut b, r);
        (b.count_of(Mesh::Cyl), b.count_of(Mesh::Cone))
    };
    let (rods, cones) = drawn(&r);
    r.racers[0].item = Some(Item::Stick);
    let (rods_held, cones_held) = drawn(&r);
    assert!(rods_held >= rods + 16, "{rods} {rods_held}");
    assert_eq!(cones_held, cones + 1, "one ape in reach, one marker");
    // an AI's Stick is not shown
    r.racers[0].driver = Driver::Ai;
    assert_eq!(drawn(&r), (rods, cones));
    r.racers[0].driver = Driver::Local;
    use_item(&mut r, 0, &press());
    assert_eq!(hits(&r), vec![(1, 0, Item::Stick)]);
}

// ---- TRIAGE T12: the Vine ----

/// Ape 0 stands on the ground at z = 100, facing +z, holding a Vine.
fn standing_with_a_vine() -> Race {
    let mut r = race(1);
    put_at(&mut r, 0, 0, int(100), 0);
    let a = &mut r.racers[0].ape;
    a.pos.y = r.course.support(a.pos) + BODY_DROP;
    a.mode = Mode::Grounded;
    r.racers[0].item = Some(Item::Vine);
    r
}

#[test]
fn a_summoned_vine_hangs_ahead_and_can_be_caught_from_the_ground() {
    let mut r = standing_with_a_vine();
    let base = r.course.vines.len();
    let hands = r.racers[0].ape.pos;
    let c = Controls { yaw: 0, ..press() };
    use_item(&mut r, 0, &c);
    assert_eq!(r.course.vines.len(), base + 1);
    assert!(r.events.contains(&Event::Summoned { who: 0, vine: base }));
    let v = r.course.vines[base];
    let ahead = v.anchor.flat().sub(hands.flat());
    assert!(ahead.z >= SUMMON_AHEAD_MIN - ONE / 100 && ahead.x.abs() < ONE / 10);
    assert!(v.anchor.y >= hands.y + SUMMON_RISE - ONE / 100);
    assert!(
        v.anchor.y - v.length < hands.y,
        "the end hangs below the hands"
    );
    assert!(v.at > int(100) && v.at < int(104), "{}", v.at);
    // hold grab: the ape catches it at once
    let grab = Controls {
        grab: true,
        ..Controls::default()
    };
    r.step(&[grab]);
    assert_eq!(r.racers[0].ape.swinging(), Some(base));
}

#[test]
fn a_moving_ape_gets_its_vine_further_ahead_along_its_motion() {
    let mut r = race(1);
    put_at(&mut r, 0, 0, int(100), int(10));
    r.racers[0].ape.vel = v3(int(20), -int(5), 0);
    r.racers[0].item = Some(Item::Vine);
    let hands = r.racers[0].ape.pos;
    // the camera looks elsewhere: the vine follows the motion
    let c = Controls {
        yaw: fx::HALF,
        ..press()
    };
    use_item(&mut r, 0, &c);
    let v = r.course.vines[r.base_vines];
    let ahead = v.anchor.sub(hands);
    assert!(
        (ahead.x - mul(int(20), SUMMON_LEAD)).abs() < ONE / 10,
        "{ahead:?}"
    );
    assert!(ahead.z.abs() < ONE / 10);
    // falling: the end reaches below where the hands will be
    assert!(v.anchor.y - v.length < hands.y - int(4));
}

#[test]
fn a_summoned_vine_lasts_its_life_and_stays_while_held() {
    let mut r = standing_with_a_vine();
    use_item(&mut r, 0, &press());
    let slot = r.base_vines;
    let placed = r.course.vines[slot];
    for _ in 0..SUMMON_LIFE - 1 {
        step(&mut r);
    }
    assert_eq!(r.course.vines[slot], placed, "gone early");
    // held when its time is up: it stays until let go
    r.racers[0].ape.mode = Mode::Swinging {
        vine: slot,
        rope: int(10),
    };
    step(&mut r);
    step(&mut r);
    assert_eq!(r.course.vines[slot], placed);
    r.racers[0].ape.mode = Mode::Flying;
    step(&mut r);
    assert_ne!(r.course.vines[slot], placed);
    assert!(r.summons.is_empty());
    // the slot stays, out of everyone's reach
    assert_eq!(r.course.vines.len(), slot + 1);
    let mut hands = r.racers[0].ape.clone();
    hands.pos = placed.anchor.sub(v3(0, int(3), 0));
    assert_eq!(hands.vine_in_reach(&r.course, r.tick), None);
}

#[test]
fn summon_slots_depend_on_who_and_how_many_so_every_browser_agrees() {
    let mut r = race(3);
    let base = r.base_vines;
    let at = |r: &Race, by: usize, serial: u32| Summon {
        slot: r.summon_slot(by, serial),
        by,
        serial,
        anchor: v3(0, int(40), int(100)),
        length: int(20),
        life: SUMMON_LIFE,
    };
    // ape 2's first summon heard before anyone else's: the list grows with
    // buried slots, and the ape-0 slot is still free for ape 0
    let s = at(&r, 2, 0);
    assert_eq!(s.slot, base + 2 * SUMMON_SLOTS);
    r.place_vine(s);
    assert_eq!(r.course.vines.len(), base + 2 * SUMMON_SLOTS + 1);
    assert_eq!(r.course.vines[s.slot].anchor, s.anchor);
    assert!(r.course.vines[base].anchor.y < -int(100), "a buried slot");
    // the same summon again changes nothing
    r.events.clear();
    r.place_vine(s);
    assert!(r.events.is_empty());
    assert_eq!(r.summons.len(), 1);
    // ape 0's next summon takes its own first slot
    r.racers[0].item = Some(Item::Vine);
    use_item(&mut r, 0, &press());
    assert_eq!(r.summons.len(), 2);
    assert_eq!(r.summons[1].slot, base);
    assert_eq!(r.summon_count, vec![1, 0, 1]);
    // the fourth summon reuses the first slot
    assert_eq!(r.summon_slot(0, SUMMON_SLOTS as u32), base);
    // a remote ape said to hold a vine not heard of yet is drawn flying
    r.racers[1].driver = Driver::Remote;
    let far = Mode::Swinging {
        vine: base + 50,
        rope: int(5),
    };
    let p = r.racers[1].ape.pos;
    r.set_remote(1, p, V3::ZERO, far, 0);
    assert_eq!(r.racers[1].ape.mode, Mode::Flying);
}

// ---- TRIAGE T19: icons ----

#[test]
fn every_item_has_its_own_svg_icon() {
    let icons: Vec<String> = Item::ALL.iter().map(|i| icon_svg(*i)).collect();
    for (k, s) in icons.iter().enumerate() {
        assert!(s.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 64 64\""));
        assert!(s.ends_with("</svg>"));
        assert!(s.len() > 150, "{s}");
        assert!(
            !icons[..k].contains(s),
            "{:?} is like another",
            Item::ALL[k]
        );
        // well formed enough: every tag closes
        assert_eq!(s.matches('<').count(), s.matches('>').count());
    }
}
