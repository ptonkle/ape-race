//! Rattlesnake Ravine's rules, each tested alone on a bare course (the
//! Jungle's centreline with an empty `RavineLayout`, one piece added per
//! test; MAPS.md: new mechanics are tested alone first): the snake vines,
//! the teeth, every kind of solid, the decks, the camera and the movers.

use super::*;
use crate::ape::{Ape, Controls, Happening, Mode};
use crate::camera::{Chase, Keeps};
use crate::fx::int;
use crate::geom::{facing, right_of, yaw_of};
use crate::race::{Driver, Entry, Event, Phase, Race};
use crate::rules::{Kind, TICKS_PER_SECOND};
use std::sync::Arc;

/// The Jungle's centreline and vines with an empty Rattlesnake Ravine
/// layout (the sphinx far off, 3 km out to the side).
fn bare() -> Course {
    let mut c = Course::jungle_seeded(0x5A4E_7E57_0001);
    c.name = "ravine";
    let hangers = c
        .vines
        .iter()
        .map(|v| Hanger {
            from: HangFrom::Ledge { wall: 0 },
            at: v.anchor,
            snake: false,
            snapped_at: None,
        })
        .collect();
    let far = v3(int(3000), 0, int(3000));
    c.ravine = Some(Box::new(RavineLayout {
        walls: Vec::new(),
        rooms: Vec::new(),
        decks: Vec::new(),
        hangers,
        lizards: Vec::new(),
        tracks: Vec::new(),
        carts: Vec::new(),
        skull: None,
        split: None,
        rocks: Vec::new(),
        glows: Vec::new(),
        finish: Finish {
            sphinx: far,
            sphinx_yaw: 0,
            campfire: far,
            sticks: Vec::new(),
            tents: Vec::new(),
        },
        lobby: (c.starts[0], c.start_yaw),
    }));
    c
}

/// The layout, to change (the solids are built again after).
fn lay(c: &mut Course) -> &mut RavineLayout {
    c.ravine_solids = Default::default();
    c.ravine.as_deref_mut().unwrap()
}

/// The centreline `s` metres along on the ground, the way on and right.
fn spot(c: &Course, s: i32) -> (V3, V3, V3) {
    let p = c.point(int(s));
    let h = c.heading_at(int(s)).flat().norm();
    (v3(p.x, c.terrain(p.x, p.z), p.z), h, right_of(h))
}

/// A one-ape race on `c`, running.
fn race(c: Course) -> Race {
    let entry = Entry {
        name: "P0".into(),
        kind: Kind::Chimpanzee,
        driver: Driver::Local,
        palette: 0,
        costume: crate::costume::Costume::Own,
    };
    let mut r = Race::new(c, &[entry], false, 1);
    while r.phase != Phase::Running {
        r.step(&[Controls::default()]);
    }
    r
}

/// Ape 0 put at `at` flying at `vel`.
fn put(r: &mut Race, at: V3, vel: V3) {
    let hint = r.course.nearest_anywhere(at);
    let a = &mut r.racers[0].ape;
    a.pos = at;
    a.vel = vel;
    a.mode = Mode::Flying;
    a.hint = hint;
    a.last_vine = None;
}

/// An ape flung from `from` at `vel` for `ticks` on `c` (no race): every
/// place its hands were, and whether it hit a wall.
fn fling(c: &Course, from: V3, vel: V3, ticks: u32) -> (Vec<V3>, bool) {
    let (at, wall, _) = fling_to(c, from, vel, ticks);
    (at, wall)
}

/// `fling`, and how the ape ends (on the ground, in the air).
fn fling_to(c: &Course, from: V3, vel: V3, ticks: u32) -> (Vec<V3>, bool, Mode) {
    let mut a = Ape::new(Kind::Chimpanzee, false, from, 0);
    a.vel = vel;
    a.mode = Mode::Flying;
    a.hint = c.nearest_anywhere(from);
    let mut at = vec![from];
    let mut wall = false;
    for t in 0..ticks {
        a.step(&Controls::default(), c, t);
        wall |= a.happened.contains(&Happening::Wall);
        at.push(a.pos);
    }
    (at, wall, a.mode)
}

/// The furthest the hands got from `from` along `dir`.
fn furthest(at: &[V3], from: V3, dir: V3) -> i32 {
    at.iter().map(|p| p.sub(from).dot(dir)).max().unwrap()
}

// ---------------------------------------------------------------------------
// Snake vines

/// The bare course's vine 10 made a snake (or not), and ape 0 put on it
/// (half way down, moving on at 12 m/s).
fn on_a_snake(snake: bool) -> (Race, usize) {
    let mut c = bare();
    let i = 10;
    lay(&mut c).hangers[i].snake = snake;
    let mut r = race(c);
    let tick = r.tick;
    let a = r.course.anchor(i, tick);
    let h = r.course.heading_at(r.course.vines[i].at).flat().norm();
    let len = r.course.vines[i].length;
    put(&mut r, a.sub(v3(0, len / 2, 0)), h.scale(int(12)));
    (r, i)
}

fn grab() -> Controls {
    Controls {
        grab: true,
        ..Controls::default()
    }
}

#[test]
fn a_grabbed_snake_vine_snaps_and_the_ape_drops_to_the_ground() {
    // an ordinary vine there is caught
    let (mut r, i) = on_a_snake(false);
    r.step(&[grab()]);
    assert_eq!(
        r.racers[0].ape.swinging(),
        Some(i),
        "an ordinary vine holds"
    );
    // a snake vine snaps on the grab, shared on the course, and the ape
    // drops with its momentum gone
    let (mut r, i) = on_a_snake(true);
    r.step(&[grab()]);
    let tick = r.tick;
    let a = &r.racers[0].ape;
    assert!(a.happened.contains(&Happening::Snapped), "{:?}", a.happened);
    assert_eq!(a.swinging(), None);
    assert_eq!(a.vel.flat(), V3::ZERO, "the momentum is gone");
    let h = r.course.ravine.as_deref().unwrap().hangers[i];
    assert_eq!(h.snapped_at, Some(tick), "snapped on the grab's tick");
    assert!(
        snake_state(&r.course)[i].is_some(),
        "and kept in a save point"
    );
    // it falls to the ground, holding grab all the way, catching nothing
    let mut landed = false;
    for _ in 0..4 * TICKS_PER_SECOND {
        r.step(&[grab()]);
        let a = &r.racers[0].ape;
        assert_eq!(a.swinging(), None, "caught a vine on the way down");
        if a.mode == Mode::Grounded {
            landed = true;
            break;
        }
    }
    assert!(landed, "never reached the ground");
}

#[test]
fn a_snapped_snake_vine_cannot_be_grabbed_until_it_hangs_whole_again() {
    let (mut r, i) = on_a_snake(true);
    r.step(&[grab()]);
    let tick = r.tick;
    let lay = r.course.ravine.as_deref().unwrap();
    assert!(lay.vine_gone(i, tick + 1) && lay.vine_gone(i, tick + SNAKE_REGROW - 1));
    // another ape there now finds nothing to grab
    let a = r.course.anchor(i, tick + 60);
    let len = r.course.vines[i].length;
    let mut b = Ape::new(Kind::Gorilla, false, a.sub(v3(0, len / 2, 0)), 0);
    b.mode = Mode::Flying;
    assert_eq!(
        b.vine_in_reach(&r.course, tick + 60),
        None,
        "a snapped vine"
    );
    // it hangs whole again (and as deadly) after `SNAKE_REGROW`
    let back = tick + SNAKE_REGROW;
    assert!(!lay.vine_gone(i, back) && lay.vine_snaps(i, back));
    assert_eq!(b.vine_in_reach(&r.course, back), Some(i));
}

#[test]
fn a_remote_ape_reported_on_a_whole_snake_vine_snaps_it() {
    let (mut r, i) = on_a_snake(true);
    let p = r.racers[0].ape.pos;
    r.set_remote(
        0,
        p,
        V3::ZERO,
        Mode::Swinging {
            vine: i,
            rope: int(3),
        },
        0,
    );
    assert_eq!(r.racers[0].ape.mode, Mode::Flying);
    assert!(r.course.ravine.as_deref().unwrap().hangers[i]
        .snapped_at
        .is_some());
}

// ---------------------------------------------------------------------------
// The T-rex teeth

/// The bare course with a T-rex skull 300 m along, its mouth's middle
/// 15 m up, the way through along the course; its teeth and bone the
/// model's (`rex_teeth`, `rex_bone_boxes`) at the brief's gap.
fn with_skull() -> (Course, Skull) {
    let mut c = bare();
    let (g, h, _) = spot(&c, 300);
    let at = g.add(v3(0, int(15), 0));
    let yaw = yaw_of(h);
    let along = facing(yaw, 0).neg();
    let across = right_of(along);
    let world = |m: V3| {
        at.add(across.scale(m.x))
            .add(v3(0, m.y, 0))
            .add(along.scale(m.z))
    };
    let teeth = crate::ravinemodels::rex_teeth(MOUTH_GAP)
        .iter()
        .map(|t| Tooth {
            root: world(t.base),
            tip: world(t.tip),
            radius: t.r,
        })
        .collect();
    let floor = at.sub(v3(0, MOUTH_GAP / 2, 0));
    let skull = Skull {
        at,
        yaw,
        teeth,
        keep: Room {
            kind: RoomKind::Skull,
            from: floor.sub(h.scale(int(9))),
            to: floor.add(h.scale(int(9))),
            half_width: int(4),
            height: MOUTH_GAP,
        },
        floor_spot: v3(g.x, g.y + BODY_DROP, g.z),
        floor_yaw: yaw,
    };
    lay(&mut c).skull = Some(skull.clone());
    (c, skull)
}

/// Ape 0 flung through the skull's mouth `up` over its middle at `speed`
/// m/s from 14 m out (a little up, so it is back at that height at the
/// mouth's middle): whether it was put back (or eaten), and how far past
/// the mouth's middle it got.
fn through_mouth(up: i32, speed: i32, lava: bool) -> (Race, bool, i32) {
    let (c, sk) = with_skull();
    let h = facing(sk.yaw, 0);
    let mut r = race(c);
    r.lava = lava;
    let from = sk.at.sub(h.scale(int(14))).add(v3(0, up, 0));
    let t = int(14) / speed;
    let lift = crate::fx::mul(crate::rules::GRAVITY, t) / 2;
    put(&mut r, from, h.scale(int(speed)).add(v3(0, lift, 0)));
    let mut past = i32::MIN;
    for _ in 0..TICKS_PER_SECOND {
        r.step(&[Controls::default()]);
        if r.racers[0].eaten.is_some() || r.events.iter().any(|e| matches!(e, Event::Reset { .. }))
        {
            return (r, true, past);
        }
        past = past.max(r.racers[0].ape.pos.sub(sk.at).dot(h));
    }
    (r, false, past)
}

#[test]
fn flying_cleanly_through_the_t_rex_mouth_is_untouched() {
    for speed in [25, 40, 70] {
        let (_, reset, past) = through_mouth(cm(60), speed, false);
        assert!(!reset, "{speed} m/s through the gap's middle: put back");
        assert!(
            past > int(8),
            "{speed} m/s: only {} cm past",
            past * 100 / ONE
        );
    }
}

#[test]
fn brushing_a_t_rex_tooth_puts_the_ape_on_the_floor_under_the_jaw() {
    for speed in [25, 70] {
        // through the upper teeth at the snout
        let (r, reset, _) = through_mouth(cm(450), speed, false);
        assert!(reset, "{speed} m/s through the teeth: not put back");
        let a = &r.racers[0].ape;
        let sk = r.course.ravine.as_deref().unwrap().skull.clone().unwrap();
        assert_eq!(a.pos, sk.floor_spot, "on the floor under the jaw");
        assert_eq!(a.vel, V3::ZERO, "with no speed");
        assert_eq!(a.yaw, sk.floor_yaw);
        assert!(r.racers[0].eaten.is_none());
    }
    // with Floor is Lava it is the kill, where it touched the teeth (not
    // on the floor after a put-back)
    let (r, reset, _) = through_mouth(cm(450), 40, true);
    let e = r.racers[0].eaten.expect("not eaten");
    let sk = r.course.ravine.as_deref().unwrap().skull.clone().unwrap();
    assert!(
        reset && e.at.dist(sk.at) < int(10),
        "eaten at {:?}, not at the teeth",
        e.at
    );
}

// ---------------------------------------------------------------------------
// Solids

/// Flung at `speed` m/s from `from` along `dir` for half a second (a
/// little up, so it is back level at the end): the furthest along `dir` it
/// got, and whether it hit a wall.
fn shoot(c: &Course, from: V3, dir: V3, speed: i32) -> (i32, bool) {
    let ticks = TICKS_PER_SECOND as u32 / 2;
    let lift = crate::rules::GRAVITY / 4;
    let (at, wall) = fling(c, from, dir.scale(int(speed)).add(v3(0, lift, 0)), ticks);
    (furthest(&at, from, dir), wall)
}

/// From a run's speed to past `MOVE` a tick (the long-move path) and on
/// to a move past a whole solid in one tick (3,000 m/s: 25 m).
const SPEEDS: [i32; 4] = [30, 90, 400, 3000];

#[test]
fn a_fast_ape_cannot_pass_a_canyon_wall() {
    let mut c = bare();
    let (g, h, right) = spot(&c, 300);
    // a wall 8 m right of the centreline, rock to its right, 30 m high
    let face = g.add(right.scale(int(8)));
    let pts = (-4..=4)
        .map(|k| {
            let p = face.add(h.scale(int(10 * k)));
            WallPoint {
                at: v3(p.x, c.terrain(p.x, p.z), p.z),
                height: int(30),
            }
        })
        .collect();
    lay(&mut c).walls.push(Wall {
        points: pts,
        side: Side::Right,
        depth: int(5),
    });
    for speed in SPEEDS {
        for up in [3, 20, 38] {
            let from = g.add(v3(0, int(up), 0));
            let (far, hit) = shoot(&c, from, right, speed);
            assert!(hit, "{speed} m/s, {up} m up: no hit");
            assert!(
                far <= int(8),
                "{speed} m/s, {up} m up: {} cm in",
                (far - int(8)) * 100 / ONE
            );
        }
    }
    // a move past the whole wall in one tick (a body flung at 10 km/s:
    // 83 m a tick) is caught too
    let (far, hit) = shoot(&c, g.add(v3(0, int(3), 0)), right, 10_000);
    assert!(
        hit && far <= int(8),
        "past the wall in a tick: {} m",
        far / ONE
    );
    // and with walls the Jungle's invisible corridor is gone: across the
    // way, nothing stops it
    let (far, _) = shoot(&c, g.add(v3(0, int(3), 0)), right.neg(), 90);
    assert!(
        far > c.half_width + int(2),
        "an invisible wall at {}",
        far / ONE
    );
}

#[test]
fn a_fast_ape_cannot_pass_a_roof_or_a_cave_side() {
    let mut c = bare();
    let (g, h, right) = spot(&c, 300);
    for kind in [RoomKind::Arch, RoomKind::Cave, RoomKind::Tunnel] {
        {
            let r = lay(&mut c);
            r.rooms.clear();
            r.rooms.push(Room {
                kind,
                from: g.sub(h.scale(int(15))),
                to: g.add(h.scale(int(15))),
                half_width: int(8),
                height: int(7),
            });
        }
        for speed in SPEEDS {
            // up into the roof
            let from = g.add(v3(0, int(2), 0));
            let (at, _) = fling(&c, from, v3(0, int(speed), 0), TICKS_PER_SECOND as u32 / 2);
            let top = at.iter().map(|p| p.y).max().unwrap();
            assert!(
                top <= g.y + int(7),
                "{kind:?} {speed} m/s: {} cm into the roof",
                (top - g.y - int(7)) * 100 / ONE
            );
            // down onto it from over it: it is not passed either, and the
            // ape stands on the roof's top
            let from = g.add(v3(0, int(30), 0));
            let (at, _, mode) =
                fling_to(&c, from, v3(0, -int(speed), 0), TICKS_PER_SECOND as u32 / 2);
            let low = at.iter().map(|p| p.y).min().unwrap();
            assert!(
                low >= g.y + int(7),
                "{kind:?} {speed} m/s down through the roof"
            );
            let feet = at[at.len() - 1].y - BODY_DROP;
            assert!(
                mode == Mode::Grounded && (feet - g.y - int(7) - ROOF_THICK).abs() < cm(5),
                "{kind:?} {speed} m/s: {mode:?}, feet {} cm over the roof's top",
                (feet - g.y - int(7) - ROOF_THICK) * 100 / ONE
            );
            // a cave's and the tunnel's sides
            let (far, hit) = shoot(&c, g.add(v3(0, int(3), 0)), right, speed);
            if kind == RoomKind::Arch {
                continue;
            }
            assert!(
                hit && far <= int(8),
                "{kind:?} {speed} m/s: {} cm through the side",
                (far - int(8)) * 100 / ONE
            );
        }
        // its mouths are open: along the way, in one end and out the other
        let from = g.sub(h.scale(int(18))).add(v3(0, int(3), 0));
        let vel = h.scale(int(60)).add(v3(0, crate::rules::GRAVITY / 2, 0));
        let (at, _) = fling(&c, from, vel, TICKS_PER_SECOND as u32);
        let far = furthest(&at, from, h);
        assert!(
            far > int(36),
            "{kind:?}: a mouth is blocked ({} m)",
            far / ONE
        );
    }
}

#[test]
fn a_fast_ape_cannot_fly_into_a_track_hole() {
    let mut c = bare();
    let (g, h, right) = spot(&c, 300);
    let hole = |at: V3| Hole {
        at,
        out: right.neg(),
        half_width: int(2),
        height: int(3),
    };
    let a = g.add(right.scale(int(8))).add(v3(0, int(4), 0));
    let b = a.add(h.scale(int(40)));
    lay(&mut c).tracks.push(Track {
        points: vec![a, b],
        start: hole(a),
        end: hole(b),
    });
    for speed in SPEEDS {
        let (far, hit) = shoot(&c, g.add(v3(0, int(5), 0)), right, speed);
        assert!(
            hit && far <= int(8),
            "{speed} m/s: {} cm into the hole",
            (far - int(8)) * 100 / ONE
        );
    }
}

#[test]
fn a_fast_ape_cannot_pass_the_skull_bone_or_the_sphinx() {
    let (mut c, sk) = with_skull();
    let h = facing(sk.yaw, 0);
    for speed in SPEEDS {
        // over the mouth, into the upper skull
        let (far, hit) = shoot(
            &c,
            sk.at.sub(h.scale(int(20))).add(v3(0, int(9), 0)),
            h,
            speed,
        );
        assert!(
            hit && far < int(20) - int(7),
            "{speed} m/s: through the skull ({} m)",
            far / ONE
        );
    }
    // the sphinx, its front feet 300 m along, facing back down the way
    let (g, h, _) = spot(&c, 300);
    lay(&mut c).skull = None;
    {
        let f = &mut lay(&mut c).finish;
        f.sphinx = g;
        f.sphinx_yaw = yaw_of(h.neg());
    }
    for speed in SPEEDS {
        let from = g.sub(h.scale(int(8))).add(v3(0, int(6), 0));
        let (far, hit) = shoot(&c, from, h, speed);
        assert!(
            hit && far <= int(8),
            "{speed} m/s: {} cm into the sphinx",
            (far - int(8)) * 100 / ONE
        );
    }
}

#[test]
fn a_fast_ape_cannot_pass_a_large_rock_and_can_stand_on_one() {
    let mut c = bare();
    let (g, h, _) = spot(&c, 300);
    let foot = g.add(h.scale(int(15)));
    lay(&mut c).rocks.push(Rock {
        at: foot,
        radius: int(3),
        height: int(4),
    });
    for speed in SPEEDS {
        let (far, hit) = shoot(&c, g.add(v3(0, int(2), 0)), h, speed);
        assert!(
            hit && far <= int(12),
            "{speed} m/s: {} cm into the rock",
            (far - int(12)) * 100 / ONE
        );
    }
    // dropped on it, it stands on its top (on the ground: it can jump)
    let (at, _, mode) = fling_to(
        &c,
        foot.add(v3(0, int(12), 0)),
        V3::ZERO,
        2 * TICKS_PER_SECOND as u32,
    );
    let end = at[at.len() - 1];
    assert_eq!(mode, Mode::Grounded, "not standing on the rock");
    assert!(
        (end.y - BODY_DROP - (foot.y + int(4))).abs() < cm(5),
        "feet {} cm over its top",
        (end.y - BODY_DROP - foot.y - int(4)) * 100 / ONE
    );
}

#[test]
fn a_deck_is_a_floor_and_dressing_is_not_solid() {
    let mut c = bare();
    let (g, h, right) = spot(&c, 300);
    let top = g.y + int(6);
    {
        let r = lay(&mut c);
        r.decks.push(Deck {
            kind: DeckKind::Bridge,
            from: v3(g.x, top, g.z).sub(right.scale(int(10))),
            to: v3(g.x, top, g.z).add(right.scale(int(10))),
            half_width: int(1),
        });
        // dressing: a torch and the campfire in the way
        r.glows.push(Glow {
            at: g.add(h.scale(int(5))).add(v3(0, int(2), 0)),
            kind: GlowKind::Torch,
        });
        r.finish.campfire = g.add(h.scale(int(8)));
    }
    let (at, _) = fling(
        &c,
        v3(g.x, top + int(5), g.z),
        V3::ZERO,
        2 * TICKS_PER_SECOND as u32,
    );
    let end = at[at.len() - 1];
    assert!(
        (end.y - BODY_DROP - top).abs() < cm(5),
        "feet {} cm over the deck",
        (end.y - BODY_DROP - top) * 100 / ONE
    );
    let (far, hit) = shoot(&c, g.add(v3(0, int(2), 0)), h, 30);
    assert!(!hit && far > int(12), "the dressing stopped it");
}

#[test]
fn the_solids_are_indexed_by_place() {
    // a long wall: a body in the middle of it looks at a handful of its
    // stretches, not all of them; far off, at none
    let mut c = bare();
    let (g, h, right) = spot(&c, 100);
    let pts = (0..200)
        .map(|k| WallPoint {
            at: g.add(right.scale(int(8))).add(h.scale(int(8 * k))),
            height: int(30),
        })
        .collect();
    lay(&mut c).walls.push(Wall {
        points: pts,
        side: Side::Right,
        depth: int(5),
    });
    let s = Arc::clone(c.ravine_solids().unwrap());
    assert_eq!(s.list.len(), 199 + 1, "a box a stretch, and the sphinx");
    let mid = g.add(h.scale(int(800)));
    let n = s.near(mid).len();
    assert!(n > 0 && n <= 12, "{n} solids near");
    assert!(s.near(mid.add(right.neg().scale(int(200)))).len() <= 1);
}

// ---------------------------------------------------------------------------
// The camera

#[test]
fn the_eye_stays_inside_a_cave_and_out_of_the_rock() {
    // a cave 40 m long, 5 m either side and 6 m high, between canyon walls
    // 7 m either side: an ape running through it, weaving, the camera
    // turned now and then; the eye is never in the rock, the roof or a
    // wall
    let mut c = bare();
    let (g, h, right) = spot(&c, 300);
    let wall = |side: i32, rock: Side| Wall {
        points: (-8..=8)
            .map(|k| {
                let p = g.add(right.scale(side * int(7))).add(h.scale(int(10 * k)));
                WallPoint {
                    at: v3(p.x, g.y, p.z),
                    height: int(25),
                }
            })
            .collect(),
        side: rock,
        depth: int(4),
    };
    let room = Room {
        kind: RoomKind::Cave,
        from: g.sub(h.scale(int(20))),
        to: g.add(h.scale(int(20))),
        half_width: int(5),
        height: int(6),
    };
    {
        let r = lay(&mut c);
        r.walls.push(wall(1, Side::Right));
        r.walls.push(wall(-1, Side::Left));
        r.rooms.push(room);
    }
    let keeps = Arc::new(Keeps::of(&c).unwrap());
    let solids = Arc::clone(c.ravine_solids().unwrap());
    let start = g.sub(h.scale(int(40))).add(v3(0, BODY_DROP, 0));
    let mut chase =
        Chase::new(start, yaw_of(h), c.terrain).with_keeps(start, Some(Arc::clone(&keeps)));
    let mut inside = 0;
    for k in 0..(80 * 10) {
        let at = start
            .add(h.scale(k * ONE / 10))
            .add(right.scale(crate::fx::sin(k * crate::fx::deg(2)) * 3));
        if k % 50 == 25 {
            chase.turn(crate::fx::deg(70), crate::fx::deg(10));
        }
        chase.follow(at, h.scale(int(12)), h, true);
        let e = chase.eye;
        assert!(
            !solids.holds(e, 0),
            "tick {k}: the eye in the rock at {e:?}"
        );
        if room.contains(at) {
            // under the roof and between the sides (it may trail out of a
            // mouth)
            inside += 1;
            let d = e.sub(room.from).flat();
            let side = d.dot(right).abs();
            assert!(
                side <= room.half_width && e.y <= room.from.y + room.height && e.y >= room.from.y,
                "tick {k}: the ape in the cave, the eye out of it at {e:?}"
            );
        }
    }
    assert!(inside > 100, "the ape was in the cave {inside} ticks");
}

// ---------------------------------------------------------------------------
// The movers

#[test]
fn an_ape_landing_on_a_walking_lizard_or_a_moving_cart_bounces_as_on_a_turtle() {
    // a lizard walking 10 m up a wall and a cart running a 100 m track,
    // each carrying a bouncer: an ape dropped where the mover is by then
    // (not where its bouncer was at the start) bounces off it
    for cart in [false, true] {
        let mut c = bare();
        let (g, h, right) = spot(&c, 300);
        let from = g.add(right.scale(int(6))).add(v3(0, int(2), 0));
        let m = c.mushrooms.len();
        c.mushrooms.push(crate::course::Mushroom {
            top: from.add(v3(0, int(1), 0)),
            radius: int(2),
        });
        let to = if cart {
            from.add(h.scale(int(100)))
        } else {
            from.add(v3(0, int(10), 0))
        };
        {
            let r = lay(&mut c);
            if cart {
                let hole = |at: V3| Hole {
                    at,
                    out: h,
                    half_width: int(1),
                    height: int(2),
                };
                r.tracks.push(Track {
                    points: vec![from, to],
                    start: hole(from.sub(h.scale(int(5)))),
                    end: hole(to.add(h.scale(int(5)))),
                });
                r.carts.push(Cart {
                    track: 0,
                    period: 2000,
                    phase: 0,
                    bouncer: m,
                });
            } else {
                r.lizards.push(Lizard {
                    from,
                    to,
                    out: right.neg(),
                    period: 400,
                    phase: 0,
                    bouncer: m,
                    colour: LizardColour::Blue,
                });
            }
        }
        let mut r = race(c);
        // where the mover's bouncer will be a second from now
        let later = r.tick + 120;
        let cap = r.course.bouncer(m, later).top;
        assert!(
            cap.dist(r.course.bouncer(m, r.tick).top) > int(1),
            "the mover never moved"
        );
        let t = ONE;
        let lift = crate::fx::mul(crate::rules::GRAVITY, t) / 2;
        put(
            &mut r,
            cap.add(v3(0, BODY_DROP + cm(30), 0)),
            v3(0, lift, 0),
        );
        let mut bounced = false;
        for _ in 0..2 * TICKS_PER_SECOND {
            r.step(&[Controls::default()]);
            if r.racers[0].ape.happened.contains(&Happening::Bounce) {
                bounced = true;
                break;
            }
        }
        assert!(bounced, "cart {cart}: no bounce off the mover");
    }
}
