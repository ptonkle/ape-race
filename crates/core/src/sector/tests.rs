//! Sector Ape's physics and rules, on small test courses in space, and its
//! layout (`sector()`).

use super::*;
use crate::ape::{Ape, Controls, Happening, Mode};
use crate::course::{Course, Platform, Vine};
use crate::fx::{cm, mul};
use crate::race::{Driver, Entry, Event, Race};
use crate::rules::{Kind, BODY_DROP, TICKS_PER_SECOND};

/// An empty layout round a test course: a boundary 40 m across, 30 m
/// below and 60 m above the line.
fn empty() -> SectorLayout {
    SectorLayout {
        hangs: Vec::new(),
        hangers: Vec::new(),
        bodies: Vec::new(),
        pads: Vec::new(),
        boundary: Boundary {
            side: int(40),
            below: -int(30),
            above: int(60),
        },
        door: Door {
            frame: Affine::IDENTITY,
            half: V3::ZERO,
            switch: usize::MAX,
            shut_at: None,
        },
        mechs: Vec::new(),
        wreck: Wreck {
            frame: Affine::IDENTITY,
            gap: crate::spacecraft::WRECK_GAP,
        },
        black_hole: (v3(0, int(20), int(600)), int(20)),
        set_pieces: Vec::new(),
        lobby: (V3::ZERO, 0),
        finish_deck: 0,
        routes: Vec::new(),
        near: Vec::new(),
    }
}

/// A straight test course in space (400 m along +z) with vines (each its
/// end, the way it hangs, its length), bodies, a start deck at the start
/// and a deck at 200 m, and (when `door`) a door across the course at
/// 60 m whose switch is vine 0.
fn space(vines: &[(V3, V3, i32)], bodies: Vec<Body>, door: bool) -> Course {
    let mut c = Course::test_line(&[]);
    c.terrain = crate::ski::Terrain::Flat(-int(400));
    let mut lay = empty();
    for &(tip, dir, len) in vines {
        let at = c.progress(tip, c.nearest_anywhere(tip)).0;
        c.vines.push(Vine {
            anchor: tip.sub(dir.scale(len)),
            length: len,
            slide: None,
            at,
        });
        lay.hangs.push(dir);
        lay.hangers.push(Hanger::Deck(0));
    }
    for s in [5, 200] {
        let p = c.point(int(s));
        c.platforms.push(Platform {
            centre: p,
            fwd: c.heading_at(int(s)),
            half_len: int(9),
            half_width: int(9),
            top: 0,
        });
        lay.pads.push(c.platforms.len() - 1);
    }
    for k in 0..4 {
        c.starts[k] = v3(int(-6) + int(4) * k as i32, BODY_DROP, int(2));
    }
    lay.bodies = bodies;
    if door {
        lay.door = Door {
            frame: Affine::translate(v3(0, int(5), int(60))),
            half: v3(int(10), int(10), ONE),
            switch: 0,
            shut_at: None,
        };
    }
    lay.index_bodies(&c.path);
    c.sector = Some(Box::new(lay));
    c
}

fn flying(at: V3, vel: V3, c: &Course) -> Ape {
    let mut a = Ape::new_on(c, Kind::Chimpanzee, true, at, 0);
    a.vel = vel;
    a.mode = Mode::Flying;
    a.hint = c.nearest_anywhere(at);
    a
}

fn hold() -> Controls {
    Controls {
        grab: true,
        ..Controls::default()
    }
}

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

#[test]
fn a_sideways_vine_is_caught_along_its_length_not_under_its_anchor() {
    // level, pointing +x: its anchor 20 m to the -x side of its end
    let right = v3(ONE, 0, 0);
    let c = space(
        &[(v3(int(5), int(10), int(100)), right, int(20))],
        vec![],
        false,
    );
    let a = c.anchor(0, 0);
    // half way along it, a metre off: caught
    let mid = flying(v3(-int(5), int(10) + ONE, int(100)), V3::ZERO, &c);
    assert_eq!(mid.vine_in_reach(&c, 0), Some(0));
    // where a vine hanging down from that anchor would be: not
    let under = flying(a.sub(v3(0, int(10), 0)), V3::ZERO, &c);
    assert_eq!(under.vine_in_reach(&c, 0), None);
    assert!(c.odd_vine(0));
    assert_eq!(c.tip(0, 0), v3(int(5), int(10), int(100)));
}

#[test]
fn an_upward_vine_is_caught_above_its_anchor_not_below_it() {
    let c = space(&[(v3(0, 0, int(100)), V3::UP, int(20))], vec![], false);
    let near = flying(v3(ONE, -int(5), int(100)), V3::ZERO, &c);
    assert_eq!(near.vine_in_reach(&c, 0), Some(0));
    let below = flying(v3(0, -int(30), int(100)), V3::ZERO, &c);
    assert_eq!(below.vine_in_reach(&c, 0), None);
    assert_eq!(c.vine_bottom(0, 0), -int(20));
}

#[test]
fn a_swing_on_a_sideways_vine_turns_the_ape_round_its_anchor() {
    // its anchor 20 m to the left (-x) of its end; the ape comes along +z
    let c = space(
        &[(v3(0, int(10), int(100)), v3(ONE, 0, 0), int(20))],
        vec![],
        false,
    );
    let mut a = flying(v3(0, int(10), int(99)), v3(0, 0, int(25)), &c);
    a.step(&hold(), &c, 0);
    assert_eq!(a.swinging(), Some(0));
    let anchor = c.anchor(0, 0);
    for t in 1..60 {
        a.step(&hold(), &c, t);
    }
    // still on it, at the rope's length, and turned toward the anchor's
    // side (-x)
    assert_eq!(a.swinging(), Some(0));
    assert!((a.pos.dist(anchor) - int(20)).abs() < int(1));
    assert!(
        a.vel.x < -int(8),
        "{} m/s across",
        a.vel.x as f64 / ONE as f64
    );
}

#[test]
fn a_swing_on_an_upward_vine_whirls_forward_and_down() {
    let c = space(&[(v3(0, 0, int(100)), V3::UP, int(20))], vec![], false);
    let mut a = flying(v3(0, 0, int(99)), v3(0, 0, int(25)), &c);
    a.step(&hold(), &c, 0);
    assert_eq!(a.swinging(), Some(0));
    for t in 1..72 {
        a.step(&hold(), &c, t);
    }
    // over the anchor and on down its far side
    assert!(a.pos.z > int(110), "{}", a.pos.z / ONE);
    assert!(a.pos.y < -int(4), "{}", a.pos.y / ONE);
    assert!(a.vel.y < -int(10));
}

#[test]
fn gravity_is_weaker_in_space() {
    let c = space(&[], vec![], false);
    let mut a = flying(v3(0, int(20), int(100)), V3::ZERO, &c);
    for t in 0..TICKS_PER_SECOND as u32 {
        a.step(&Controls::default(), &c, t);
    }
    let fall = -a.vel.y;
    assert!(
        (fall - SECTOR_GRAVITY).abs() < ONE / 4,
        "{} m/s after a second",
        fall as f64 / ONE as f64
    );
    assert!(SECTOR_GRAVITY < crate::rules::GRAVITY * 2 / 3);
}

#[test]
fn nothing_goes_faster_than_the_cap() {
    let c = space(&[], vec![], false);
    let mut a = flying(v3(0, int(20), int(100)), v3(0, 0, int(120)), &c);
    a.step(&Controls::default(), &c, 0);
    assert!(a.speed() <= SECTOR_TOP, "{}", a.speed() / ONE);
}

#[test]
fn crossing_the_boundary_puts_the_ape_back_on_the_nearest_deck() {
    let c = space(&[], vec![], false);
    let mut r = Race::new(c, &entries(2), false, 1);
    r.phase = crate::race::Phase::Running;
    // 90 m along, flying out past the side
    let a = &mut r.racers[0].ape;
    a.pos = v3(int(41), int(5), int(90));
    a.hint = r.course.nearest_anywhere(a.pos);
    a.vel = v3(int(10), 0, int(20));
    a.mode = Mode::Flying;
    r.step(&[Controls::default(); 2]);
    assert!(r.events.contains(&Event::Reset { who: 0, pad: 0 }));
    let a = &r.racers[0].ape;
    // on the start's deck (85 m back; the one at 200 m is 110 m on),
    // standing still, facing on down the course
    assert_eq!(a.mode, Mode::Grounded);
    assert_eq!(a.vel, V3::ZERO);
    assert_eq!(a.pos, v3(0, BODY_DROP, int(5)));
    assert_eq!(a.yaw, 0);
    assert!(r.racers[0].eaten.is_none());
    // the nearest either way (of two as near, the one behind); below
    // and above count too
    for (pos, pad) in [
        (v3(0, -int(31), int(250)), 1),
        (v3(0, int(61), int(250)), 1),
        (v3(0, -int(31), int(110)), 1),
        (v3(0, -int(31), int(100)), 0),
        (v3(0, -int(31), int(102)), 0),
    ] {
        let a = &mut r.racers[1].ape;
        a.pos = pos;
        a.hint = r.course.nearest_anywhere(pos);
        a.mode = Mode::Flying;
        r.step(&[Controls::default(); 2]);
        assert!(
            r.events.contains(&Event::Reset { who: 1, pad }),
            "{:?}",
            r.events
        );
    }
}

#[test]
fn in_floor_is_lava_crossing_the_boundary_is_the_kill() {
    let c = space(&[], vec![], false);
    let mut r = Race::new(c, &entries(1), false, 1);
    r.lava = true;
    r.phase = crate::race::Phase::Running;
    let a = &mut r.racers[0].ape;
    a.pos = v3(0, -int(31), int(150));
    a.hint = r.course.nearest_anywhere(a.pos);
    a.mode = Mode::Flying;
    r.step(&[Controls::default()]);
    assert!(r.events.contains(&Event::Eaten { who: 0 }));
    assert!(!r.events.iter().any(|e| matches!(e, Event::Reset { .. })));
    assert!(r.racers[0].eaten.is_some());
    // and the decks are not lava: standing on one is safe
    let mut r = Race::new(space(&[], vec![], false), &entries(1), false, 1);
    r.lava = true;
    for _ in 0..crate::race::COUNTDOWN as u32 + 240 {
        r.step(&[Controls::default()]);
    }
    assert!(r.running() && r.racers[0].eaten.is_none());
    assert_eq!(r.racers[0].ape.mode, Mode::Grounded);
    assert!(!r.course.on_floor(r.racers[0].ape.pos));
}

/// A bouncy (or dead) box 10 m square, `thick` through along z, its near
/// face at 100 m along.
fn wall(thick: i32, bouncy: bool) -> Body {
    Body {
        model: Model::Chunk(0),
        frame: Affine::translate(v3(0, int(5), int(100) + thick / 2)),
        half: v3(int(5), int(5), thick / 2),
        bouncy,
        pull: DOWN,
    }
}

#[test]
fn a_bouncy_body_throws_an_ape_off_elastically_with_a_boost() {
    // its top at 10 m; the ape comes down on it at 45°
    let c = space(&[], vec![wall(int(10), true)], false);
    let mut a = flying(
        v3(0, int(10) + HIT_DROP + int(2), int(102)),
        v3(0, -int(20), int(20)),
        &c,
    );
    let mut before = a.vel;
    let mut hit = false;
    for t in 0..30 {
        before = a.vel;
        a.step(&Controls::default(), &c, t);
        if a.happened.contains(&Happening::Rebound) {
            hit = true;
            break;
        }
    }
    assert!(hit, "never hit");
    // the part into the face is turned round (elastic), the rest kept, and
    // the boost added on along the course (+z)
    let near = |a: i32, b: i32| (a - b).abs() < ONE / 4;
    assert!(near(a.vel.y, -before.y), "{} vs {}", a.vel.y, before.y);
    assert!(
        near(a.vel.z, before.z + REBOUND_BOOST),
        "{} m/s on",
        a.vel.z as f64 / ONE as f64
    );
    assert!(a.speed() > before.len() + REBOUND_BOOST / 2);
}

/// How far above the hands' point the ball's bottom is not: the hands sit
/// this far over the ball's middle.
const HIT_DROP: i32 = BODY_DROP / 2 + crate::ape::HIT_RADIUS;

#[test]
fn a_hit_while_swinging_knocks_the_ape_off_its_vine() {
    // a vine whose swing runs into a body ahead
    let c = space(
        &[(v3(0, 0, int(95)), DOWN, int(20))],
        vec![wall(int(4), true)],
        false,
    );
    let mut a = flying(v3(0, 0, int(94)), v3(0, 0, int(20)), &c);
    a.step(&hold(), &c, 0);
    assert_eq!(a.swinging(), Some(0));
    let mut knocked = false;
    for t in 1..120 {
        a.step(&hold(), &c, t);
        if a.happened.contains(&Happening::Rebound) {
            knocked = true;
            break;
        }
    }
    assert!(knocked);
    assert_eq!(a.swinging(), None);
}

#[test]
fn no_tunnelling_through_thin_debris_at_top_speed() {
    // a plate 30 cm thick, dead (no bounce), straight across the way
    for bouncy in [false, true] {
        let c = space(&[], vec![wall(cm(30), bouncy)], false);
        let mut a = flying(
            v3(0, int(5) + BODY_DROP / 2, int(60)),
            v3(0, 0, SECTOR_TOP),
            &c,
        );
        for t in 0..120 {
            a.step(&Controls::default(), &c, t);
            assert!(
                a.pos.z < int(100),
                "passed through at {} m (bouncy {bouncy})",
                a.pos.z as f64 / ONE as f64
            );
        }
    }
    // and a head-on hit on a bouncy body never throws it back down the
    // course fast
    let c = space(&[], vec![wall(int(2), true)], false);
    let mut a = flying(
        v3(0, int(5) + BODY_DROP / 2, int(90)),
        v3(0, 0, int(60)),
        &c,
    );
    for t in 0..30 {
        a.step(&Controls::default(), &c, t);
    }
    assert!(
        a.vel.z >= -crate::ape::REBOUND_BACK - ONE / 10,
        "{}",
        a.vel.z / ONE
    );
}

/// A race on a course whose start row is the switch vine: the ape runs off
/// the deck holding grab and W, catches it and swings it forward.
fn switch_course() -> Course {
    space(&[(v3(0, ONE, int(16)), DOWN, int(29))], vec![], true)
}

fn swing_it(r: &mut Race, ticks: u32) -> Option<u32> {
    let go = Controls {
        fwd: 1,
        grab: true,
        ..Controls::default()
    };
    let mut shut = None;
    for _ in 0..ticks {
        r.step(&[go]);
        if r.events.contains(&Event::Door { who: 0 }) {
            shut = Some(r.tick);
        }
    }
    shut
}

#[test]
fn swinging_the_switch_far_forward_shuts_the_door_for_a_while() {
    let mut r = Race::new(switch_course(), &entries(1), false, 1);
    let start = crate::race::COUNTDOWN as u32;
    let shut = swing_it(&mut r, start + 4 * TICKS_PER_SECOND as u32).expect("the door shut");
    let door = r.course.sector.as_ref().unwrap().door;
    assert_eq!(door.shut_at, Some(shut));
    assert!(door.shut(shut) && door.shut(shut + DOOR_SHUT - 1));
    assert!(!door.shut(shut + DOOR_SHUT));
    // at that tick the rope was more than DOOR_ANGLE forward
    let mut again = Race::new(switch_course(), &entries(1), false, 1);
    swing_it(&mut again, shut - 1);
    let before = rules::swung_forward(&again.course, 0, again.racers[0].ape.pos, again.tick);
    swing_it(&mut again, 1);
    let at = rules::swung_forward(&again.course, 0, again.racers[0].ape.pos, again.tick);
    assert!(before < DOOR_ANGLE && at >= DOOR_ANGLE, "{before} {at}");
}

#[test]
fn a_shut_door_blocks_then_opens() {
    let mut c = space(&[], vec![], true);
    c.sector.as_mut().unwrap().door.shut_at = Some(0);
    let mut a = flying(
        v3(0, int(5) + BODY_DROP / 2, int(40)),
        v3(0, 0, int(30)),
        &c,
    );
    let mut blocked = false;
    for t in 0..DOOR_SHUT {
        a.step(&Controls::default(), &c, t);
        assert!(a.pos.z < int(59), "through the shut door at tick {t}");
        blocked |= a.happened.contains(&Happening::Wall);
        // (held level, as if floating at the door)
        a.vel.y = 0;
        a.vel.z = a.vel.z.max(int(5));
        a.pos.y = int(5) + BODY_DROP / 2;
    }
    assert!(blocked);
    for t in DOOR_SHUT..DOOR_SHUT + 120 {
        a.step(&Controls::default(), &c, t);
    }
    assert!(a.pos.z > int(62), "still held at {} m", a.pos.z / ONE);
}

#[test]
fn the_door_cannot_be_shut_again_until_it_has_reopened() {
    let mut r = Race::new(switch_course(), &entries(1), false, 1);
    let shut = swing_it(
        &mut r,
        crate::race::COUNTDOWN as u32 + 4 * TICKS_PER_SECOND as u32,
    )
    .expect("shut");
    // while shut, a swing past the angle changes nothing
    let mut later = r.clone();
    let a = later.course.anchor(0, later.tick);
    later.racers[0].ape.pos = a.add(v3(0, -int(10), int(20)));
    later.racers[0].ape.mode = Mode::Swinging {
        vine: 0,
        rope: int(22),
    };
    later.switch(0);
    assert_eq!(
        later.course.sector.as_ref().unwrap().door.shut_at,
        Some(shut)
    );
    // once open again it can
    later.tick = shut + DOOR_SHUT + 5;
    later.switch(0);
    assert_eq!(
        later.course.sector.as_ref().unwrap().door.shut_at,
        Some(shut + DOOR_SHUT + 5)
    );
}

#[test]
fn an_ape_from_another_browser_shuts_the_door_from_its_state() {
    let mut es = entries(2);
    es[1].driver = Driver::Remote;
    let mut r = Race::new(switch_course(), &es, false, 1);
    r.phase = crate::race::Phase::Running;
    let a = r.course.anchor(0, r.tick);
    // 60° forward of hanging straight down, on a 20 m rope
    let at = a.add(v3(0, -int(10), mul(int(20), 56_756)));
    let swing = Mode::Swinging {
        vine: 0,
        rope: int(20),
    };
    r.set_remote(1, at, V3::ZERO, swing, 0);
    assert_eq!(r.course.sector.as_ref().unwrap().door.shut_at, Some(r.tick));
    assert!(r.events.contains(&Event::Door { who: 1 }));
}

#[test]
fn a_replay_and_a_save_point_keep_the_door() {
    use crate::replay::{put_back, Mode as RecMode, Recording, SavePoint};
    let course = switch_course();
    let mut live = Race::new(course.clone(), &entries(1), false, 3);
    let mut rec = Recording::of(&live, 0, RecMode::Race);
    let go = Controls {
        fwd: 1,
        grab: true,
        ..Controls::default()
    };
    let mut shut = None;
    for _ in 0..crate::race::COUNTDOWN as u32 + 4 * TICKS_PER_SECOND as u32 {
        rec.controls.push(go);
        live.step(&[go]);
        if live.events.contains(&Event::Door { who: 0 }) {
            shut = Some(live.tick);
        }
    }
    let shut = shut.expect("shut");
    // raced again from the recording: the same door
    let mut again = rec.race(&course);
    while (again.tick as usize) < rec.controls.len() {
        let c = rec.at(again.tick);
        again.step(&[c]);
    }
    assert_eq!(again.racers[0].ape.pos, live.racers[0].ape.pos);
    assert_eq!(
        again.course.sector.as_ref().unwrap().door,
        live.course.sector.as_ref().unwrap().door
    );
    // a save point while it is shut, then on past its reopening and
    // shutting again: scrubbing back puts the door back as it was
    let mut race = Race::new(course.clone(), &entries(1), false, 3);
    for k in 0..shut + 10 {
        race.step(&[rec.at(k)]);
    }
    let save = SavePoint::of(&mut race, &[], &Course::test_line(&[]));
    assert_eq!(save.door, Some(shut));
    race.course.sector.as_mut().unwrap().door.shut_at = Some(shut + 900);
    let mut brains = Vec::new();
    put_back(save, &mut race, &mut brains);
    assert_eq!(
        race.course.sector.as_ref().unwrap().door.shut_at,
        Some(shut)
    );
    assert_eq!(race.tick, shut + 10);
}
