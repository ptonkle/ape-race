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
    s.traps.clear();
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
    let k = r.turtles.len() - 1;
    let (a, yaw_out) = r.turtle_at(k, 0);
    assert_eq!(a, v3(0, 0, 0));
    let (b, _) = r.turtle_at(k, 100);
    assert!(b.dist(v3(int(10), 0, 0)) < cm(1), "{b:?}");
    let (_, yaw_back) = r.turtle_at(k, 150);
    assert_ne!(yaw_out, yaw_back, "it turns round at the far bank");
    assert_eq!(r.turtle_at(k, 37), r.turtle_at(k, 237), "periodic");
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

/// A running race of one ape on Sunset Ruins.
fn running(lava: bool) -> Race {
    let mut r = Race::new(ruins(), &entries(1), false, 1);
    r.lava = lava;
    while r.phase != crate::race::Phase::Running {
        r.step(&[Controls::default()]);
    }
    r
}

/// Puts ape 0 at `p`, moving `v`, in `mode`.
fn put(r: &mut Race, p: V3, v: V3, mode: Mode) {
    let a = &mut r.racers[0].ape;
    a.pos = p;
    a.vel = v;
    a.mode = mode;
    a.hint = r.course.nearest_anywhere(p);
}

/// How far along the rope bridge's boosted vines hang.
fn bridge_at(c: &Course) -> i32 {
    c.vines[c.ruins.as_ref().unwrap().bridge.boost[0]].at
}

#[test]
fn the_current_carries_a_swimmer_downstream_slower_than_a_swing() {
    let mut r = running(false);
    let s = int(300);
    let p = r.course.point(s);
    let surface = r.course.water(p).expect("the river runs here");
    let hands = surface - crate::rules::SWIM_SINK + crate::rules::BODY_DROP;
    put(&mut r, v3(p.x, hands, p.z), V3::ZERO, Mode::Grounded);
    assert!(r.racers[0].ape.swimming(&r.course));
    let h = r.course.heading_at(s);
    for _ in 0..TICKS_PER_SECOND {
        r.step(&[Controls::default()]);
    }
    let a = &r.racers[0].ape;
    let down = a.pos.sub(p).dot(h);
    assert!(down > int(2), "carried {} cm downstream", down * 100 / ONE);
    assert!(
        a.vel.dot(h) <= CURRENT + cm(10),
        "no faster than the current: {}",
        a.vel.dot(h)
    );
}

#[test]
fn a_fall_into_the_ravine_puts_you_back_on_the_bank_before_the_bridge() {
    let mut r = running(false);
    let p = r.course.point(int(580));
    let below = r.course.ruins.as_ref().unwrap().ravine.below;
    put(
        &mut r,
        v3(p.x, below - cm(10), p.z),
        v3(0, -int(20), 0),
        Mode::Flying,
    );
    r.step(&[Controls::default()]);
    let spot = r.course.ruins.as_ref().unwrap().ravine.spot;
    assert_eq!(r.racers[0].ape.pos, spot, "put back on the bank");
    assert!(r
        .events
        .contains(&crate::race::Event::Reset { who: 0, pad: 0 }));
    let (s, _) = r.course.progress(spot, r.racers[0].ape.hint);
    assert!(s < bridge_at(&r.course), "behind the bridge");
    // Floor is Lava: the kill
    let mut r = running(true);
    put(
        &mut r,
        v3(p.x, below - cm(10), p.z),
        v3(0, -int(20), 0),
        Mode::Flying,
    );
    r.step(&[Controls::default()]);
    assert!(r.racers[0].eaten.is_some());
}

/// Ape 0 swinging the middle boosted vine at the bottom of its arc, moving
/// `v`, then let go: how far along it is 4 s on and whether it was put back.
fn bridge_release(v: V3) -> (i32, bool) {
    let mut r = running(false);
    let lay = r.course.ruins.as_ref().unwrap();
    let vine = lay.bridge.boost[1];
    let an = r.course.vines[vine].anchor;
    let rope = int(16);
    put(
        &mut r,
        an.sub(v3(0, rope, 0)),
        v,
        Mode::Swinging { vine, rope },
    );
    let mut reset = false;
    for _ in 0..4 * TICKS_PER_SECOND {
        r.step(&[Controls::default()]);
        reset |= r
            .events
            .iter()
            .any(|e| matches!(e, crate::race::Event::Reset { .. }));
    }
    let a = &r.racers[0].ape;
    (r.course.progress(a.pos, a.hint).0, reset)
}

#[test]
fn the_bridge_flings_a_decent_release_into_the_temple_and_a_bad_one_falls() {
    let c = ruins();
    let h = c.heading_at(bridge_at(&c));
    let mouth = c.ruins.as_ref().unwrap().marks.mouth;
    for speed in [10, 18, 28] {
        let (s, reset) = bridge_release(h.scale(int(speed)).add(v3(0, int(4), 0)));
        assert!(
            !reset && s > mouth + int(5),
            "{speed} m/s: at {} m, reset {reset}",
            s / ONE
        );
    }
    // barely moving: no boost, into the ravine; swinging back toward the
    // river: no boost, back upstream, never into the temple
    let (_, slow) = bridge_release(h.scale(int(3)));
    assert!(slow, "a weak release falls into the ravine");
    let (s, _) = bridge_release(h.scale(-int(12)).add(v3(0, int(4), 0)));
    assert!(
        s < mouth,
        "a release backward is not flung in: at {} m",
        s / ONE
    );
}

#[test]
fn a_spike_trap_flings_you_up_and_you_cannot_grab_for_a_second() {
    let mut r = running(false);
    let area = r.course.ruins.as_ref().unwrap().spikes[0].area;
    let p = v3(
        area.centre.x,
        area.centre.y + crate::rules::BODY_DROP,
        area.centre.z,
    );
    put(&mut r, p, V3::ZERO, Mode::Grounded);
    r.step(&[Controls::default()]);
    let a = &r.racers[0].ape;
    assert_eq!(a.mode, Mode::Flying);
    assert!(a.vel.y > int(10), "flung up: {}", a.vel.y);
    assert!(a.happened.contains(&crate::ape::Happening::Spiked));
    assert!(a.no_grab > SPIKE_NO_GRAB as i32 - 3);
    // grab held at a vine in reach: nothing for a second
    let mut c = ruins();
    let mut a = r.racers[0].ape.clone();
    c.vines[0].anchor = a.pos.add(v3(0, int(3), 0));
    a.vel = V3::ZERO;
    let grab = Controls {
        grab: true,
        ..Controls::default()
    };
    a.step(&grab, &c, r.tick);
    assert!(a.swinging().is_none(), "no grab while spiked");
    a.no_grab = 0;
    a.step(&grab, &c, r.tick);
    assert_eq!(a.swinging(), Some(0), "grabs again after");
}

#[test]
fn pit_vines_hang_only_while_the_trapdoor_is_open() {
    let mut c = ruins();
    let lay = c.ruins.as_deref_mut().unwrap();
    let vine = lay.traps[0].vines[0];
    let pit = lay.traps[0].pit;
    let door = lay.traps[0].door.centre;
    lay.traps[0].tripped_at = Some(0);
    let an = c.vines[vine].anchor;
    let low = v3(an.x, an.y - c.vines[vine].length + int(1), an.z);
    let grab = Controls {
        grab: true,
        ..Controls::default()
    };
    // shut (before the delay): out of reach
    let mut a = crate::ape::Ape::new(Kind::Chimpanzee, false, low, 0);
    a.mode = Mode::Flying;
    assert_eq!(a.vine_in_reach(&c, 10), None);
    // open: in reach, and caught
    let open = TRAP_DELAY + 10;
    assert_eq!(a.vine_in_reach(&c, open), Some(vine));
    a.step(&grab, &c, open);
    assert_eq!(a.swinging(), Some(vine));
    // it closes: the vine goes up and lets the ape go
    a.step(&grab, &c, TRAP_DELAY + TRAP_OPEN + 1);
    assert_eq!(a.swinging(), None, "a raised vine lets go");
    // the open door's floor has dropped to the pit
    let over = v3(door.x, door.y + crate::rules::BODY_DROP, door.z);
    assert_eq!(c.support_at(over, open), pit);
    assert_eq!(c.support_at(over, 10), c.support(over));
}
