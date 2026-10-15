//! Sunset Ruins' contract: the shared trapdoors, the turtles, the river.

use super::*;
use crate::ape::{Controls, Mode};
use crate::course::Course;
use crate::map::Map;
use crate::race::{Driver, Entry, Race};
use crate::rules::Kind;

fn entries(n: usize) -> Vec<Entry> {
    (0..n)
        .map(|i| Entry {
            name: format!("P{i}"),
            kind: Kind::Chimpanzee,
            driver: Driver::Local,
            palette: 0,
        })
        .collect()
}

/// The stand-in course with one trapdoor whose wire lies under `at`.
fn with_trap(at: V3) -> Course {
    let mut c = ruins();
    let s = c.ruins.as_deref_mut().unwrap();
    s.traps.push(Trapdoor {
        wire: (at.sub(v3(int(2), 0, 0)), at.add(v3(int(2), 0, 0))),
        door: Slab {
            centre: at,
            fwd: v3(0, 0, ONE),
            half: v3(int(2), cm(20), int(2)),
        },
        pit: -int(4),
        vines: vec![0],
        tripped_at: None,
    });
    c
}

#[test]
fn only_sunset_ruins_has_a_ruins_layout() {
    for map in Map::ALL {
        let c = map.course();
        assert_eq!(c.ruins.is_some(), map == Map::Ruins, "{map:?}");
        assert_eq!(Map::of(&c), map);
    }
    let c = ruins();
    let r = c.ruins.as_ref().unwrap();
    assert_eq!(r.hangers.len(), c.vines.len(), "one hanger per vine");
    assert_eq!(r.bridge.width, BRIDGE_WIDTH);
}

#[test]
fn a_trapdoor_opens_after_the_delay_stays_open_then_rearms() {
    let t = Trapdoor {
        wire: (V3::ZERO, V3::ZERO),
        door: Slab {
            centre: V3::ZERO,
            fwd: v3(0, 0, ONE),
            half: V3::ZERO,
        },
        pit: 0,
        vines: vec![3],
        tripped_at: Some(100),
    };
    assert_eq!(TRAP_DELAY, 90, "0.75 s at 120 ticks a second");
    assert!(!t.open(100 + TRAP_DELAY - 1));
    assert!(t.open(100 + TRAP_DELAY));
    assert!(t.open(100 + TRAP_DELAY + TRAP_OPEN - 1));
    assert!(!t.open(100 + TRAP_DELAY + TRAP_OPEN));
    assert!(!t.armed(100 + TRAP_DELAY + TRAP_OPEN - 1));
    assert!(t.armed(100 + TRAP_DELAY + TRAP_OPEN));
    // the pit vine is up while the floor is shut, down while it is open
    let mut r = ruins().ruins.unwrap();
    r.traps.push(t);
    assert!(r.vine_up(3, 100) && !r.vine_up(3, 100 + TRAP_DELAY) && !r.vine_up(4, 0));
}

#[test]
fn an_ape_on_the_floor_trips_the_wire_for_everyone_and_save_points_keep_it() {
    let probe = Race::new(ruins(), &entries(2), false, 1);
    let at = probe.racers[0].ape.pos;
    assert_eq!(probe.racers[0].ape.mode, Mode::Grounded);
    let mut r = Race::new(with_trap(at), &entries(2), false, 1);
    let blank = r.course.clone();
    let save = crate::replay::SavePoint::of(&mut r, &[], &blank);
    // before Go the apes stand still and trip nothing
    while r.phase != crate::race::Phase::Running {
        r.step(&[Controls::default(); 2]);
    }
    assert_eq!(r.course.ruins.as_ref().unwrap().traps[0].tripped_at, None);
    r.step(&[Controls::default(); 2]);
    let tripped = r.course.ruins.as_ref().unwrap().traps[0].tripped_at;
    assert_eq!(tripped, Some(r.tick), "tripped by the ape standing on it");
    // armed again only once it has closed: a second ape does not re-trip it
    r.step(&[Controls::default(); 2]);
    assert_eq!(
        r.course.ruins.as_ref().unwrap().traps[0].tripped_at,
        tripped
    );
    // scrubbing back puts the floor back as it was
    let mut brains = Vec::new();
    crate::replay::put_back(save, &mut r, &mut brains);
    assert_eq!(r.course.ruins.as_ref().unwrap().traps[0].tripped_at, None);
}

#[test]
fn a_remote_ape_trips_the_wire_from_its_reported_state() {
    let far = v3(int(500), 0, int(500));
    let mut r = Race::new(with_trap(far), &entries(2), false, 1);
    r.set_remote(1, far.add(v3(0, 0, cm(80))), V3::ZERO, Mode::Flying, 0);
    assert_eq!(r.course.ruins.as_ref().unwrap().traps[0].tripped_at, None);
    r.set_remote(1, far.add(v3(0, 0, cm(80))), V3::ZERO, Mode::Grounded, 0);
    assert_eq!(
        r.course.ruins.as_ref().unwrap().traps[0].tripped_at,
        None,
        "out of reach"
    );
    r.set_remote(
        1,
        far.add(v3(cm(30), 0, cm(40))),
        V3::ZERO,
        Mode::Grounded,
        0,
    );
    assert_eq!(
        r.course.ruins.as_ref().unwrap().traps[0].tripped_at,
        Some(r.tick)
    );
}

#[test]
fn a_turtle_swims_across_and_back_from_the_tick() {
    let mut r = ruins().ruins.unwrap();
    r.turtles.push(Turtle {
        from: v3(0, 0, 0),
        to: v3(int(10), 0, 0),
        period: 200,
        phase: 0,
        bouncer: 0,
    });
    let (a, yaw_out) = r.turtle_at(0, 0);
    assert_eq!(a, v3(0, 0, 0));
    let (b, _) = r.turtle_at(0, 100);
    assert!(b.dist(v3(int(10), 0, 0)) < cm(1), "{b:?}");
    let (_, yaw_back) = r.turtle_at(0, 150);
    assert_ne!(yaw_out, yaw_back, "it turns round at the far bank");
    assert_eq!(r.turtle_at(0, 37), r.turtle_at(0, 237), "periodic");
}

#[test]
fn the_river_has_a_surface_and_a_current_only_over_it() {
    let river = River {
        points: vec![
            RiverPoint {
                at: v3(0, int(10), 0),
                half_width: int(5),
                current: int(3),
                rough: 0,
            },
            RiverPoint {
                at: v3(0, 0, int(100)),
                half_width: int(5),
                current: int(3),
                rough: ONE,
            },
        ],
    };
    let w = river.water_at(int(1), int(50)).expect("over the river");
    assert!((w.surface - int(5)).abs() < cm(2), "{}", w.surface);
    assert!(w.current.z > int(2) && w.current.x == 0);
    assert!(river.water_at(int(8), int(50)).is_none());
}
