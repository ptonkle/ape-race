//! The City's layout and the physics it adds.

use super::*;
use crate::ape::{Ape, Controls, Happening, Mode};
use crate::course::{Street, Vine, OUTRO};
use crate::fx::{int, ONE};
use crate::rules::{Kind, GRAVITY, MUSHROOM_BOUNCE, SWIM_SINK, TICKS_PER_SECOND};

fn hw(c: &Course) -> i32 {
    c.half_width
}

/// Whether `p` is inside the corridor (some street's box), `margin` in
/// from its walls.
fn in_corridor(c: &Course, p: V3, margin: i32) -> bool {
    c.streets.iter().any(|st| {
        let d = p.flat().sub(st.from);
        let (lo, hi) = st.span(hw(c) - margin);
        let a = d.dot(st.dir);
        a > lo && a < hi && d.dot(right_of(st.dir)).abs() < hw(c) - margin
    })
}

/// The corner between streets `k` and `k + 1`: where they cross, and the
/// turn in degrees (left positive).
fn corner(c: &Course, k: usize) -> (V3, Street, Street, i32) {
    let (a, b) = (c.streets[k], c.streets[k + 1]);
    let degrees = if a.dir.dot(b.dir).abs() < ONE / 5 {
        90
    } else {
        45
    };
    // a left turn (positive) has the cross product pointing up
    let sign = if a.dir.cross(b.dir).y > 0 { 1 } else { -1 };
    (b.from, a, b, sign * degrees)
}

#[test]
fn the_city_is_as_long_as_the_jungle_and_turns_sharply_both_ways() {
    let c = city();
    assert_eq!(c.name, "city");
    assert_eq!(c.length, int(1800));
    let turns: Vec<i32> = (0..c.streets.len() - 1).map(|k| corner(&c, k).3).collect();
    assert_eq!(turns, vec![90, -45, -90, 45, 90, -45], "{turns:?}");
    // and it never comes back near itself
    for i in (0..c.path.len()).step_by(10) {
        for j in (i + 80..c.path.len()).step_by(10) {
            assert!(c.path[i].dist(c.path[j]) > int(90), "{i} {j}");
        }
    }
}

#[test]
fn the_city_is_laid_out_the_same_every_time() {
    assert_eq!(city(), city());
    assert_ne!(city().vines, city_seeded(1).vines);
    assert_eq!(city().streets, city_seeded(1).streets);
}

#[test]
fn a_90_degree_corner_is_square_outside_and_sharp_inside() {
    let c = city();
    for k in [0, 2, 4] {
        let (p, a, b, turn) = corner(&c, k);
        assert_eq!(turn.abs(), 90);
        let inward = b.dir;
        let w = hw(&c) - int(1);
        // the whole crossing is open, out to its square outside corner
        let outside = p.add(a.dir.scale(w)).sub(inward.scale(w));
        let inside = p.sub(a.dir.scale(w)).add(inward.scale(w));
        for q in [outside, inside, p] {
            assert!(in_corridor(&c, q, int(0)), "corner {k}");
            let (back, _, hit) = c.contain(q, V3::ZERO, c.nearest_anywhere(q));
            assert!(
                !hit && back == q,
                "corner {k}: a body at a crossing's corner is pushed"
            );
        }
        // past either wall of the outside corner is a building
        for q in [
            p.add(a.dir.scale(hw(&c) + int(3))),
            p.sub(inward.scale(hw(&c) + int(3))),
            p.sub(a.dir.scale(hw(&c) + int(3)))
                .add(inward.scale(hw(&c) + int(3))),
        ] {
            let (back, _, hit) = c.contain(q, a.dir.scale(int(10)), c.nearest_anywhere(q));
            assert!(hit, "corner {k}: no wall");
            assert!(in_corridor(&c, back, int(0)));
        }
    }
}

#[test]
fn a_body_flying_into_a_corner_stays_in_the_streets() {
    // straight on into the outside corner of every corner, fast: the walls
    // stop it and it never gets out of the streets
    let c = city();
    for k in 0..c.streets.len() - 1 {
        let (p, a, b, _) = corner(&c, k);
        for lat in [-int(15), 0, int(15)] {
            let mut ape = Ape::new(Kind::Lemur, false, V3::ZERO, 0);
            ape.pos = p
                .sub(a.dir.scale(int(40)))
                .add(right_of(a.dir).scale(lat))
                .add(v3(0, int(20), 0));
            ape.vel = a.dir.scale(int(40)).add(b.dir.scale(int(5)));
            ape.mode = Mode::Flying;
            ape.hint = c.nearest_anywhere(ape.pos);
            for t in 0..240 {
                ape.step(&Controls::default(), &c, t);
                assert!(
                    in_corridor(&c, ape.pos, cm(50)),
                    "corner {k}, {} m across: out at tick {t}",
                    lat / ONE
                );
            }
        }
    }
}

#[test]
fn progress_runs_on_smoothly_round_every_corner() {
    // along the course at 1 m steps, two thirds of the way out to either
    // wall: progress never goes back (beyond the samples' grain inside a
    // corner) and never jumps far (inside a corner, near the bend's
    // middle, a few metres: the ape is in the crossing either way)
    let c = city();
    for off in [-int(14), -int(7), 0, int(7), int(14)] {
        let loose = off.abs() > int(10);
        let mut hint = 0;
        let mut last = 0;
        for s in (0..1800).map(int) {
            let p = c.point(s).add(right_of(c.heading_at(s)).scale(off));
            hint = c.nearest(p, hint);
            let (along, _) = c.progress(p, hint);
            assert!(
                if loose {
                    along >= last - int(3) && along <= last + int(8)
                } else {
                    along >= last - cm(20) && along <= last + int(3)
                },
                "{} m ({} across): {} after {}",
                s / ONE,
                off / ONE,
                along / ONE,
                last / ONE
            );
            last = along;
        }
        assert!(last > int(1790));
    }
}

#[test]
fn buildings_line_the_streets_on_the_walls_and_never_in_them() {
    let c = city();
    let lay = c.city.as_ref().unwrap();
    let river = lay.river.unwrap();
    for (i, b) in lay.blocks.iter().enumerate() {
        let r = right_of(b.fwd);
        for x in -4..=4 {
            for y in -4..=4 {
                let p = b
                    .centre
                    .add(b.fwd.scale((b.half_len - cm(10)) * x / 4))
                    .add(r.scale((b.half_width - cm(10)) * y / 4));
                assert!(!in_corridor(&c, p, 0), "block {i} stands in the street");
                assert!(c.water(p).is_none(), "block {i} stands in the river");
            }
        }
    }
    // every stretch of facade along the streets is a building, just
    // behind the wall: not over the river and its banks, nor round the
    // plaza, nor in a corner's mitre
    let mut faced = 0;
    let mut open = 0;
    for st in &c.streets {
        let (lo, hi) = (int(22), st.len - int(22));
        let mut a = lo;
        while a < hi {
            let p = st.from.add(st.dir.scale(a));
            let s = c.progress(p, c.nearest_anywhere(p)).0;
            a += int(3);
            if s > river.from - int(30) && s < river.to + int(30) || s > c.length {
                continue;
            }
            for side in [-1, 1] {
                let q = p.add(right_of(st.dir).scale(side * (hw(&c) + int(1))));
                let covered = lay.blocks.iter().any(|b| {
                    let d = q.flat().sub(b.centre.flat());
                    d.dot(b.fwd).abs() <= b.half_len && d.dot(right_of(b.fwd)).abs() <= b.half_width
                });
                if covered {
                    faced += 1;
                } else {
                    open += 1;
                }
            }
        }
    }
    assert!(
        open * 20 < faced,
        "{open} metres-ish of open wall to {faced}"
    );
    // low brownstones to skyscrapers
    let lo = lay
        .blocks
        .iter()
        .map(|b| b.height)
        .filter(|h| *h > int(5))
        .min()
        .unwrap();
    let hi = lay.blocks.iter().map(|b| b.height).max().unwrap();
    assert!(
        lo <= int(17) && hi >= int(150),
        "{} to {} m",
        lo / ONE,
        hi / ONE
    );
}

#[test]
fn the_bridge_deck_holds_an_ape_and_the_river_runs_beside_it() {
    let c = city();
    let lay = c.city.as_ref().unwrap();
    let (river, bridge) = (lay.river.unwrap(), lay.bridge.clone().unwrap());
    let mid = (river.from + river.to) / 2;
    let p = c.point(mid);
    let r = right_of(c.heading_at(mid));
    let top = bridge.start.y;
    // on the deck (just above it) an ape stands on it, not in the water
    let on = v3(p.x, top + BODY_DROP + cm(5), p.z).add(r.scale(int(8)));
    assert_eq!(c.support(on), top);
    assert_eq!(c.water(on), Some(river.level));
    // beside it, and below it, is the river
    let beside = v3(p.x, top + BODY_DROP, p.z).add(r.scale(int(14)));
    assert_eq!(c.support(beside), river.level - SWIM_SINK);
    let under = v3(p.x, river.level, p.z);
    assert_eq!(c.support(under), river.level - SWIM_SINK);
    // and the river is only between its banks
    assert!(c.water(c.point(river.from - int(2))).is_none());
    assert!(c.water(c.point(river.to + int(2))).is_none());
    // the deck runs bank to bank, flush with the street, its towers on it
    assert!(c.progress(bridge.start, c.nearest_anywhere(bridge.start)).0 < river.from);
    assert!(c.progress(bridge.end, c.nearest_anywhere(bridge.end)).0 > river.to);
    for end in [bridge.start, bridge.end] {
        let g = ground(end.x, end.z);
        assert!(
            top <= g && top > g - int(2),
            "the deck at {} m, the street {}",
            top,
            g
        );
    }
    assert_eq!(bridge.towers.len(), 2);
    assert!(river.level < top - int(3));
    // the Jungle has no river
    let j = Course::jungle();
    for s in (0..1800).step_by(20).map(int) {
        assert!(j.water(j.point(s)).is_none());
    }
}

#[test]
fn an_ape_runs_off_the_street_onto_the_deck_and_across() {
    let c = city();
    let lay = c.city.as_ref().unwrap();
    let river = lay.river.unwrap();
    let s = river.from - int(20);
    let h = c.heading_at(s);
    let p = c.point(s);
    let mut ape = Ape::new(
        Kind::Gorilla,
        false,
        v3(p.x, ground(p.x, p.z) + BODY_DROP, p.z),
        0,
    );
    ape.hint = c.nearest_anywhere(ape.pos);
    let ctl = Controls {
        fwd: 1,
        yaw: crate::geom::yaw_of(h),
        ..Controls::default()
    };
    for t in 0..(TICKS_PER_SECOND as u32 * 22) {
        ape.step(&ctl, &c, t);
        assert!(!ape.swimming(&c), "fell in at tick {t}");
    }
    assert!(c.progress(ape.pos, ape.hint).0 > river.to + int(10));
}

/// How long an ape dropped into the river at `p` takes to be out of it
/// (climbed out, or bounced off a boat), in ticks.
fn time_to_get_out(c: &Course, p: V3) -> Option<u32> {
    let mut ape = Ape::new(Kind::Chimpanzee, true, p, 0);
    ape.mode = Mode::Flying;
    ape.hint = c.nearest_anywhere(p);
    let mut wet = false;
    for t in 0..(TICKS_PER_SECOND as u32 * 30) {
        ape.step(&Controls::default(), c, t);
        wet |= ape.swimming(c);
        let out = wet && !ape.swimming(c) && ape.climb.is_none();
        if out || ape.happened.contains(&Happening::Bounce) {
            return Some(t);
        }
    }
    None
}

#[test]
fn a_fall_in_the_river_is_short_a_swim_to_a_ladder_a_bank_or_a_boat() {
    // the person: falling in must not lose the race. From anywhere in the
    // water, with no keys, an ape is out in a few seconds
    let c = city();
    let lay = c.city.as_ref().unwrap();
    let river = lay.river.unwrap();
    let mut worst = 0;
    let mut s = river.from + int(3);
    while s < river.to - int(2) {
        let p = c.point(s);
        let r = right_of(c.heading_at(s));
        for lat in [-18, -14, -11, 11, 14, 18] {
            let q = p.add(r.scale(int(lat)));
            let start = v3(q.x, river.level + int(2), q.z);
            let t = time_to_get_out(&c, start)
                .unwrap_or_else(|| panic!("stuck in the river at {} m, {lat} m", s / ONE));
            worst = worst.max(t);
        }
        s += int(7);
    }
    assert!(
        worst <= 4 * TICKS_PER_SECOND as u32,
        "out after {} s at worst",
        worst / TICKS_PER_SECOND as u32
    );
}

#[test]
fn a_swimmer_climbs_a_ladder_onto_the_deck() {
    let c = city();
    let lay = c.city.as_ref().unwrap();
    let bridge = lay.bridge.as_ref().unwrap();
    let l = bridge.ladders()[6];
    let level = lay.river.unwrap().level;
    let out = l.flat().sub(bridge.start.flat());
    let fwd = bridge.end.sub(bridge.start).flat().norm();
    let away = out.sub(fwd.scale(out.dot(fwd))).with_len(int(2));
    let start = v3(l.x + away.x, level - SWIM_SINK + BODY_DROP, l.z + away.z);
    let mut ape = Ape::new(Kind::Chimpanzee, true, start, 0);
    ape.hint = c.nearest_anywhere(start);
    assert!(ape.swimming(&c));
    for t in 0..(TICKS_PER_SECOND as u32 * 5) {
        ape.step(&Controls::default(), &c, t);
    }
    assert!(!ape.swimming(&c) && ape.climb.is_none());
    assert_eq!(ape.feet().y, bridge.start.y, "on the deck");
    // and on the deck it runs again
    assert_eq!(ape.mode, Mode::Grounded);
}

#[test]
fn every_car_and_boat_bounces_from_its_own_bouncer() {
    let c = city();
    let lay = c.city.as_ref().unwrap();
    let mut used: Vec<usize> = lay.cars.iter().map(|k| k.bouncer).collect();
    used.extend(lay.boats.iter().map(|b| b.bouncer));
    let n = used.len();
    used.sort();
    used.dedup();
    assert_eq!(used.len(), n, "two share a bouncer");
    assert_eq!(n, c.mushrooms.len(), "a bouncer with no car or boat");
    for car in &lay.cars {
        let (top, radius) = car_size(car.kind);
        let m = c.mushrooms[car.bouncer];
        assert_eq!(m.radius, radius);
        assert_eq!(m.top, car.at.add(v3(0, top, 0)));
        assert!(in_corridor(&c, car.at, m.radius), "a car out of the street");
    }
    for boat in &lay.boats {
        let (top, _) = boat_size(boat.kind);
        let m = c.mushrooms[boat.bouncer];
        assert_eq!(m.top, boat.at.add(v3(0, top, 0)));
        assert!(c.water(boat.at).is_some(), "a boat out of the river");
        let at = c.progress(boat.at, c.nearest_anywhere(boat.at));
        assert!(
            at.1.abs() > DECK_HALF + m.radius - cm(10),
            "a boat under the deck"
        );
    }
    // none is taller than the bouncer search's cut-off
    for m in &c.mushrooms {
        assert!(m.top.y - ground(m.top.x, m.top.z) < BOUNCER_TALLEST);
    }
    let parked = lay.cars.iter().filter(|k| k.drive.is_none()).count();
    let moving = lay.cars.len() - parked;
    assert!(
        parked >= 15 && moving >= 8,
        "{parked} parked, {moving} moving"
    );
    assert!(lay.boats.len() >= 4);
}

#[test]
fn a_bounce_off_a_parked_car_reaches_a_vine_ahead() {
    // as the Jungle's mushrooms: each stands under a lane, a vine ahead of
    // it in that lane within 25 m low enough for the bounce, and no vine's
    // end over it
    let c = city();
    let lay = c.city.as_ref().unwrap();
    let reach = Kind::Chimpanzee.race_stats(true).physics().reach;
    for car in lay.cars.iter().filter(|k| k.drive.is_none()) {
        let m = c.mushrooms[car.bouncer];
        let (along, _) = c.progress(m.top, c.nearest_anywhere(m.top));
        let peak =
            m.top.y + BODY_DROP + fx::div(fx::mul(MUSHROOM_BOUNCE, MUSHROOM_BOUNCE), 2 * GRAVITY);
        // in its own lane: the bounce goes up, not across
        let ahead = c.vines.iter().any(|v| {
            let across = c.progress(v.anchor, c.nearest_anywhere(v.anchor)).1;
            let lane = c.progress(m.top, c.nearest_anywhere(m.top)).1;
            v.at > along
                && v.at < along + int(25)
                && (across - lane).abs() < int(5)
                && v.anchor.y - v.length < peak + reach
        });
        assert!(ahead, "{} m: no vine to bounce to", along / ONE);
        for (i, v) in c.vines.iter().enumerate() {
            let flat = v.anchor.flat().sub(m.top.flat()).len();
            assert!(
                flat > m.radius + cm(60) || v.anchor.y - v.length > m.top.y + int(2),
                "vine {i} ends over a car"
            );
        }
    }
}

#[test]
fn moving_cars_drive_their_street_the_same_every_time() {
    let c = city();
    let lay = c.city.as_ref().unwrap();
    for (i, car) in lay.cars.iter().enumerate() {
        let Some(d) = car.drive else {
            assert_eq!(c.car_at(i, 999), (car.at, car.yaw));
            continue;
        };
        assert_eq!(c.car_at(i, 0).0, car.at);
        for t in (31..d.period as u32 * 2).step_by(30) {
            let last = c.car_at(i, t - 30).0;
            let (p, _) = c.car_at(i, t);
            assert_eq!(c.car_at(i, t), city().car_at(i, t));
            assert!(in_corridor(&c, p, int(2)), "car {i} off the street");
            assert_eq!(c.bouncer(car.bouncer, t).top.flat(), p.flat());
            if (t + d.phase as u32) % d.period as u32 > 30 {
                // a cab moves a few metres in a quarter second
                assert!(
                    p.dist(last) > int(2) && p.dist(last) < int(6),
                    "car {i} at {t}"
                );
            }
        }
    }
}

#[test]
fn a_moving_cab_bounces_an_ape_standing_in_its_way() {
    let c = city();
    let lay = c.city.as_ref().unwrap();
    let i = lay.cars.iter().position(|k| k.drive.is_some()).unwrap();
    let (ahead, _) = c.car_at(i, 120);
    let mut ape = Ape::new(Kind::Gorilla, false, ahead.add(v3(0, BODY_DROP, 0)), 0);
    ape.hint = c.nearest_anywhere(ape.pos);
    let mut bounced = false;
    for t in 0..180 {
        ape.step(&Controls::default(), &c, t);
        bounced |= ape.happened.contains(&Happening::Bounce);
    }
    assert!(bounced);
}

#[test]
fn every_vine_hangs_from_one_thing_and_all_five_kinds_are_used() {
    let c = city();
    let lay = c.city.as_ref().unwrap();
    let vines: Vec<usize> = lay.hangers.iter().map(|h| h.vine).collect();
    assert_eq!(vines, (0..c.vines.len()).collect::<Vec<_>>());
    for kind in [
        HangerKind::Helicopter,
        HangerKind::Crane,
        HangerKind::BridgeCable,
        HangerKind::StreetLight,
        HangerKind::Wire,
    ] {
        assert!(lay.hangers.iter().any(|h| h.kind == kind), "no {kind:?}");
    }
    for h in &lay.hangers {
        let a = c.vines[h.vine].anchor;
        match h.kind {
            HangerKind::BridgeCable => assert!(c.water(a).is_some()),
            HangerKind::Wire => {
                // strung at the vine's height, wall to wall, through it
                assert_eq!((h.from.y, h.to.y), (a.y, a.y));
                let w = h.to.sub(h.from).flat();
                assert!((w.len() - 2 * hw(&c)).abs() < int(1));
                let d = a.flat().sub(h.from.flat());
                assert!(d.sub(w.norm().scale(d.dot(w.norm()))).len() < cm(20));
            }
            HangerKind::Helicopter => assert!(h.from.y > a.y),
            HangerKind::Crane | HangerKind::StreetLight => {
                // its mast at the kerb or beyond the wall, not out in the street
                assert_eq!(h.to, a);
                assert!(
                    !in_corridor(&c, h.from, int(1)),
                    "a mast in the street: vine {} {:?} at {} m",
                    h.vine,
                    h.kind,
                    c.vines[h.vine].at / ONE
                );
            }
        }
    }
}

#[test]
fn vines_are_sparser_than_the_jungles_but_always_offer_a_way_on() {
    let c = city();
    let j = Course::jungle();
    assert!(
        c.vines.len() * 10 < j.vines.len() * 9,
        "{} vines to the Jungle's {}",
        c.vines.len(),
        j.vines.len()
    );
    let mut ats: Vec<i32> = c.vines.iter().map(|v| v.at).collect();
    ats.sort();
    for w in ats.windows(2) {
        assert!(
            w[1] - w[0] <= int(40),
            "gap {} m at {}",
            (w[1] - w[0]) / ONE,
            w[0] / ONE
        );
    }
    assert!(c.length - ats[ats.len() - 1] <= int(50));
    // at least two vines in every 35 m, and a low vine within 60 m (the
    // corners and the bridge have none of their own)
    let mut from = int(30);
    while from < c.length - int(60) {
        let here = c
            .vines
            .iter()
            .filter(|v| v.at >= from && v.at < from + int(35))
            .count();
        assert!(here >= 2, "{} m: {here} vines", from / ONE);
        let low = c.vines.iter().any(|v| {
            (v.at - from).abs() < int(60)
                && v.anchor.y - v.length - c.support(v3(v.anchor.x, v.anchor.y, v.anchor.z))
                    < int(4)
        });
        assert!(low, "{} m: no low vine near", from / ONE);
        from += int(5);
    }
}

#[test]
fn every_90_degree_corner_has_cables_to_swing_round() {
    let c = city();
    for k in [0, 2, 4] {
        let (p, ..) = corner(&c, k);
        let near: Vec<&Vine> = c
            .vines
            .iter()
            .filter(|v| v.anchor.flat().dist(p) < int(20) && v.length < int(15))
            .collect();
        assert_eq!(near.len(), 2, "corner {k}");
    }
}

#[test]
fn an_ape_swings_round_a_90_degree_corner_on_its_crane_cable() {
    // TRIAGE (the City brief): at a 90° corner an ape catches the crane's
    // cable, whirls round it and lets go facing down the new street,
    // keeping most of its speed, and flies on along it
    let c = city();
    let lay = c.city.as_ref().unwrap();
    for k in [0, 2, 4] {
        let (p, a, b, _) = corner(&c, k);
        let (i, cable) = c
            .vines
            .iter()
            .enumerate()
            .find(|(i, v)| {
                v.anchor.flat().dist(p) < int(20) && lay.hangers[*i].kind == HangerKind::Crane
            })
            .unwrap();
        let speed = int(28);
        let start = cable
            .anchor
            .sub(a.dir.scale(int(20)))
            .sub(b.dir.scale(int(3)))
            .sub(v3(0, int(2), 0));
        let mut ape = Ape::new(Kind::Chimpanzee, true, start, 0);
        ape.mode = Mode::Flying;
        ape.vel = a.dir.scale(speed);
        ape.hint = c.nearest_anywhere(start);
        let mut held = false;
        let mut let_go = None;
        for t in 0..(3 * TICKS_PER_SECOND as u32) {
            let facing = ape.vel.flat().norm().dot(b.dir) > ONE * 95 / 100;
            let grab = let_go.is_none() && !(held && facing);
            ape.step(
                &Controls {
                    grab,
                    ..Controls::default()
                },
                &c,
                t,
            );
            if ape.swinging() == Some(i) {
                held = true;
            }
            if held && !grab && let_go.is_none() {
                let_go = Some((t, ape.speed()));
            }
            assert!(
                !ape.happened.contains(&Happening::Wall),
                "corner {k}: into the wall at tick {t}"
            );
        }
        let (t, kept) = let_go.unwrap_or_else(|| panic!("corner {k}: never faced the new street"));
        assert!(held, "corner {k}: never caught the cable");
        assert!(t < 2 * TICKS_PER_SECOND as u32, "corner {k}: {t} ticks");
        assert!(kept * 10 > speed * 7, "corner {k}: {} m/s", kept / ONE);
        // and a couple of seconds on it is well down the new street
        assert!(ape.pos.flat().sub(p).dot(b.dir) > int(30), "corner {k}");
    }
}

#[test]
fn the_plaza_past_the_finish_is_open_and_closed_by_a_grand_building() {
    let c = city();
    let lay = c.city.as_ref().unwrap();
    let last = c.path.len() - 1;
    let end = c.path[last];
    let fwd = c.heading[last];
    let terminal = lay
        .blocks
        .iter()
        .find(|b| b.style == style::TERMINAL)
        .unwrap();
    let past = terminal.centre.flat().sub(end).dot(fwd) - terminal.half_len;
    assert!(past >= OUTRO, "{} m past the finish", past / ONE);
    // and a finished ape coasting on is stopped short of it
    let p = end.add(fwd.scale(OUTRO));
    let (q, _, _) = c.contain(p, fwd.scale(int(20)), last);
    assert!(q.flat().sub(end).dot(fwd) < OUTRO - int(10));
}

#[test]
fn every_starter_can_run_off_the_platform_and_catch_a_vine() {
    let c = city();
    for kind in [Kind::Gorilla, Kind::Lemur, Kind::Chimpanzee] {
        for start in c.starts {
            assert_eq!(c.support(start), crate::course::START_HEIGHT);
            let mut ape = Ape::new(kind, false, start, 0);
            let mut caught = false;
            for t in 0..360u32 {
                let off_edge = ape.pos.sub(c.path[0]).dot(c.heading[0])
                    > c.platforms[1].centre.sub(c.path[0]).dot(c.heading[0])
                        + c.platforms[1].half_len;
                ape.step(
                    &Controls {
                        fwd: 1,
                        grab: off_edge,
                        ..Controls::default()
                    },
                    &c,
                    t,
                );
                if ape.swinging().is_some() {
                    caught = true;
                    break;
                }
            }
            assert!(caught, "{kind:?} from {start:?} never caught a vine");
        }
    }
}

#[test]
fn the_start_streets_end_is_well_behind_the_start_for_the_camera() {
    // Y13: the chase camera, some 10 m behind an ape at the start, was
    // inside the office closing the start street (7 m behind it)
    let c = city();
    let lay = c.city.as_ref().unwrap();
    let back = lay
        .blocks
        .iter()
        .filter(|b| b.centre.flat().sub(c.path[0]).dot(c.heading[0]) < 0)
        .filter(|b| {
            let d = b.centre.flat().sub(c.path[0]);
            d.dot(right_of(c.heading[0])).abs() < b.half_width
        })
        .map(|b| -(b.centre.flat().sub(c.path[0]).dot(c.heading[0])) - b.half_len)
        .min()
        .unwrap();
    assert!(
        back >= int(16),
        "the start's back wall {} m behind it",
        back / ONE
    );
    // and an ape still cannot run off backwards
    let p = c.path[0].sub(c.heading[0].scale(int(12)));
    let (q, _, _) = c.contain(v3(p.x, int(1), p.z), V3::ZERO, 0);
    assert!(q.flat().sub(c.path[0]).dot(c.heading[0]) >= -int(6));
}
