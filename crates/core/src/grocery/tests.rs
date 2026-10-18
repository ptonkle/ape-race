//! Clean-Up on Aisle Ape's store and course (`notes/grocery-pitfalls.md`).

use super::layout::{aisle_x, leg, marks, BACK, FRONT, ROUTE_AISLES};
use super::*;
use crate::course::{Course, STEP};
use crate::map::Map;
use crate::ravine::solids::{SolidKind, Solids};
use crate::rules::{BODY_DROP, BODY_RADIUS};

fn store() -> (Course, Solids) {
    let c = grocery();
    let s = Solids::of(c.ravine.as_deref().unwrap());
    (c, s)
}

#[test]
fn the_grocery_is_held_from_the_map_choice_and_random() {
    assert!(Map::Grocery.held());
    assert!(Map::offered().all(|m| m != Map::Grocery));
    assert_eq!(Map::from_key("grocery"), Some(Map::Grocery));
    assert_eq!(Map::Grocery.course().name, "grocery");
}

/// The route's three aisles keep to their aisles' middle lines (bowed as
/// the runs are) within 4 m, from end to end (it rounds the bow's vertex).
#[test]
fn the_route_follows_its_three_aisles() {
    let c = grocery();
    let m = marks();
    // (each aisle's legs: the bowed ones five, the frozen aisle one)
    for (k, (first, legs)) in [(leg::AISLE_A, 5), (leg::AISLE_B, 1), (leg::AISLE_C, 5)]
        .into_iter()
        .enumerate()
    {
        let (w, e) = ROUTE_AISLES[k];
        let mut s = m[first] + 4;
        let mut seen = 0;
        while s < m[first + legs] {
            let (x, z) = plan_of(c.point(int(s)));
            if z > int(FRONT + 4) && z < int(BACK - 4) {
                let off = (x - aisle_x(w, e, z)).abs();
                assert!(
                    off < int(4),
                    "aisle {k} at {s} m: {} cm off its middle",
                    off * 100 / ONE
                );
                seen += 1;
            }
            s += 2;
        }
        assert!(
            seen > 120,
            "aisle {k}: only {seen} samples between the ends"
        );
    }
}

/// Nothing solid stands within 16 m behind the start (MAPS gotcha 5), and
/// the starters stand on the stock cart's top.
#[test]
fn the_start_is_clear_behind_and_the_starters_stand_on_the_cart() {
    let (c, s) = store();
    let back = c.heading[0].neg();
    for d in 0..=16 {
        for across in [-8, 0, 8] {
            let q = c.path[0]
                .add(back.scale(int(d)))
                .add(c.right[0].scale(int(across)))
                .add(v3(0, int(20), 0));
            assert!(
                !s.near(q).iter().any(|&k| {
                    let sol = &s.list[k as usize];
                    sol.kind != SolidKind::Roof && sol.holds(q, 0)
                }),
                "solid {d} m behind the start, {across} m across"
            );
        }
    }
    for p in c.starts {
        assert_eq!(c.support(p), crate::course::START_HEIGHT);
        assert!(p.y - BODY_DROP == crate::course::START_HEIGHT);
    }
}

/// Every aisle is sealed: over a run's drawn top stock the solid reaches
/// the ceiling's roof, so no ape flies over a shelf into the next aisle.
#[test]
fn every_aisle_is_sealed_under_the_ceiling() {
    let (c, s) = store();
    let g = c.grocery.as_deref().unwrap();
    for (k, run) in g.gondolas.iter().enumerate() {
        for z in [FRONT + 20, MIDDLE_Z, BACK - 20] {
            let q = gondola_at(run, int(z));
            for y in [int(5), STOCK_TOP + int(1), CEILING_AT - int(1)] {
                let p = v3(q.x, y, q.z);
                assert!(
                    s.holds(p, 0),
                    "run {k} at plan z {z}, {} m up: open",
                    y / ONE
                );
            }
        }
    }
    // and the ceiling itself is solid everywhere over the floor
    let (lo, hi) = g.bounds;
    for k in 0..=8 {
        let p = v3(
            lo.x + (hi.x - lo.x) * k / 8,
            CEILING_AT + int(2),
            lo.z + (hi.z - lo.z) * (8 - k) / 8,
        );
        assert!(s.holds(p, 0), "the ceiling is open at {k}");
    }
}

const MIDDLE_Z: i32 = layout::MIDDLE;

/// Every vine hangs clear of the walls and posts (its foot and its
/// middle), from the ceiling, under it (out in the lot, no higher than
/// the light poles' arms).
#[test]
fn every_vine_hangs_clear_of_the_shelves() {
    let (c, s) = store();
    let g = c.grocery.as_deref().unwrap();
    let hit = |q: V3| {
        s.near(q).iter().any(|&k| {
            let sol = &s.list[k as usize];
            sol.kind != SolidKind::Roof && sol.holds(q, BODY_RADIUS)
        })
    };
    for (i, v) in c.vines.iter().enumerate() {
        let top = if g.outside(v.anchor) {
            layout::LAMP_HEIGHT
        } else {
            CEILING_AT
        };
        assert!(v.anchor.y <= top, "vine {i} above the ceiling");
        let foot = c.tip(i, 0);
        let mid = c.anchor(i, 0).lerp(foot, ONE / 2);
        assert!(!hit(foot) && !hit(mid), "vine {i} in a solid");
    }
}

/// A shopper's balloons walk with them: the string's anchor (the slide)
/// stays over the shopper's hand (the lizard plus the hand's reach) at
/// every tick, within a few centimetres.
#[test]
fn the_balloons_walk_with_their_shopper() {
    let c = grocery();
    let g = c.grocery.as_deref().unwrap();
    let r = c.ravine.as_deref().unwrap();
    let mut held = 0;
    for s in &g.shoppers {
        let Some(i) = s.balloons else {
            continue;
        };
        held += 1;
        let l = r.lizards[s.lizard];
        let hand = l.out.scale(layout::HAND_OUT);
        for tick in (0..4_000).step_by(37) {
            let (feet, _) = r.lizard_at(s.lizard, tick);
            let want = v3(feet.x, layout::BALLOONS_UP, feet.z).add(hand);
            let got = c.anchor(i, tick);
            assert!(
                !got.sub(want).longer_than(cm(30)),
                "shopper {} at tick {tick}: balloons {} cm off",
                s.lizard,
                got.sub(want).len() * 100 / ONE
            );
        }
    }
    assert!(held >= 5, "only {held} shoppers hold balloons");
}

/// The shoppers' heads bounce an ape that lands on them, wherever they
/// have walked to.
#[test]
fn a_shopper_s_head_is_a_moving_bouncer() {
    let c = grocery();
    let g = c.grocery.as_deref().unwrap();
    let r = c.ravine.as_deref().unwrap();
    for s in &g.shoppers {
        let l = r.lizards[s.lizard];
        for tick in [0, 500, 1_234] {
            let (feet, _) = r.lizard_at(s.lizard, tick);
            let on = feet.add(v3(0, SHOPPER_HEIGHT + BODY_DROP - cm(20), 0));
            assert_eq!(c.bouncer_under(on, tick), Some(l.bouncer));
        }
    }
}

/// The route is about 110 s long (the person, GR-T4: the detours and the
/// lot may grow it from 90 s): some 2.1 km.
#[test]
fn the_route_is_about_two_kilometres() {
    let c = grocery();
    let len = c.length / ONE;
    assert!((2_000..=2_250).contains(&len), "{len} m");
    assert_eq!(c.path.len() as i32 - 1, c.length / STEP);
}

/// Every aisle off the route is closed at both mouths (a runner cannot get
/// in: `ai::walled_off` would leave it no vine), but an item aisle's mouth
/// on the route, closed `POCKET` in (GR-T2); the route's three are open
/// from end to end.
#[test]
fn the_aisles_off_the_route_are_closed_and_the_route_s_are_open() {
    let (_, s) = store();
    let mouth = |w: i32, e: i32, z: i32| plan(int(w + e) / 2, int(z)).add(v3(0, int(5), 0));
    let runs = layout::RUNS;
    for a in 1..runs.len() {
        let (w, e) = (runs[a - 1].0, runs[a].0);
        let on_route = ROUTE_AISLES.iter().any(|&(x, _)| x + 1 == a);
        for (z, back, inward) in [(FRONT, false, 1), (BACK, true, -1)] {
            let pocket = layout::ITEM_AISLES.contains(&(a, back));
            let open_to = if pocket { layout::POCKET - 2 } else { 0 };
            let depths: &[i32] = if pocket {
                &[4, layout::POCKET - 2, layout::POCKET + 3]
            } else {
                &[4]
            };
            for &d in depths {
                let shut = s.holds(mouth(w, e, z + inward * d), 0);
                let want = !on_route && d > open_to;
                assert_eq!(shut, want, "aisle {a} at plan z {}", z + inward * d);
            }
        }
    }
}

/// The shopping list (GR-T2, GR-T3): four zones in the route's order, a
/// spot for each racer in each, all four spots of a zone in the same
/// aisle's mouth and as far along as each other (within the aisle's
/// width), clear of the shelves, each on its own string.
#[test]
fn every_racer_has_an_item_in_each_zone_s_aisle() {
    let (c, s) = store();
    let g = c.grocery.as_deref().unwrap();
    assert_eq!(g.items.len(), LIST_ZONES * LIST_SLOTS);
    let mut last = 0;
    for zone in 0..LIST_ZONES {
        let spots: Vec<_> = (0..LIST_SLOTS)
            .map(|slot| *g.item(zone, slot).expect("a spot"))
            .collect();
        let along: Vec<i32> = spots.iter().map(|it| c.vines[it.vine].at).collect();
        let (lo, hi) = (*along.iter().min().unwrap(), *along.iter().max().unwrap());
        assert!(lo > last, "zone {zone} comes before zone {}", zone - 1);
        assert!(
            hi - lo <= int(45),
            "zone {zone}'s spots {} m apart",
            (hi - lo) / ONE
        );
        last = hi;
        for it in &spots {
            assert!(
                !s.holds(it.at, BODY_RADIUS + ITEM_REACH / 2),
                "item {it:?} in a solid"
            );
            assert_eq!(
                c.tip(it.vine, 0).y,
                it.at.y + int(2),
                "item {it:?} off its string"
            );
        }
    }
}

/// The list's gains (GR-T2): the first and third items each add a quarter
/// of the kind's top speed (the drag falls by the square), the second and
/// fourth a quarter of its air steering, added, not compounded: with all
/// four, 1.5 times each.
#[test]
fn the_list_s_items_add_speed_then_air_control_a_quarter_each() {
    let c = grocery();
    let a0 = crate::ape::Ape::new_on(&c, crate::rules::Kind::Gibbon, false, c.starts[0], 0);
    let mut a = a0.clone();
    let gains: Vec<_> = (0..LIST_ZONES).map(|z| list::take(&mut a, z)).collect();
    use crate::race::Reward::{ListAir, ListSpeed};
    assert_eq!(gains, [ListSpeed, ListAir, ListSpeed, ListAir]);
    assert!(list::full(&a));
    // (`got` × `k` against `want` × `j`, within a unit of `got`: the drag
    // is a small whole number)
    let near = |got: i32, k: i64, want: i32, j: i64| (got as i64 * k - want as i64 * j).abs() <= k;
    // drag × (1/1.5)², air, drive fade and run × 1.5
    let p = (a.phys, a0.phys);
    assert!(
        near(p.0.drag, 9, p.1.drag, 4),
        "drag {} of {}",
        p.0.drag,
        p.1.drag
    );
    assert!(
        near(p.0.air, 2, p.1.air, 3),
        "air {} of {}",
        p.0.air,
        p.1.air
    );
    assert!(near(p.0.run_speed, 2, p.1.run_speed, 3));
    assert!(near(p.0.drive_fade, 2, p.1.drive_fade, 3));
    assert_eq!(p.0.reach, p.1.reach);
}

/// Each racer takes only its own spot's item (GR-T3), by touching it or
/// by grabbing its string, once; the race says so (the HUD's flash).
#[test]
fn a_racer_takes_only_its_own_spot_s_item_once() {
    use crate::ape::Controls;
    use crate::race::{Driver, Entry, Event, Phase, Race};
    let c = grocery();
    let g = c.grocery.clone().unwrap();
    let e: Vec<Entry> = (0..LIST_SLOTS)
        .map(|_| Entry {
            name: String::new(),
            kind: crate::rules::Kind::Gorilla,
            driver: Driver::Ai,
            palette: 0,
            costume: crate::costume::Costume::Own,
        })
        .collect();
    let mut r = Race::new(c, &e, true, 7);
    r.phase = Phase::Running;
    let hold = |it: &ListItem| it.at.add(v3(0, BODY_DROP / 2, 0));
    // each at the spot two along: nobody takes anything
    for k in 0..LIST_SLOTS {
        r.racers[k].ape.pos = hold(g.item(0, (k + 2) % LIST_SLOTS).unwrap());
    }
    r.step(&[Controls::default(); LIST_SLOTS]);
    assert!(r.racers.iter().all(|x| x.ape.list == 0));
    // each at its own: each takes it, and says so, once
    for k in 0..LIST_SLOTS {
        r.racers[k].ape.pos = hold(g.item(0, k).unwrap());
        r.racers[k].ape.vel = V3::ZERO;
    }
    r.step(&[Controls::default(); LIST_SLOTS]);
    let said = |r: &Race| {
        r.events
            .iter()
            .filter(|e| matches!(e, Event::Reward { .. }))
            .count()
    };
    assert!(
        r.racers.iter().all(|x| x.ape.list == 1),
        "{:?}",
        r.racers.iter().map(|x| x.ape.list).collect::<Vec<_>>()
    );
    assert_eq!(said(&r), LIST_SLOTS);
    r.step(&[Controls::default(); LIST_SLOTS]);
    assert_eq!(said(&r), 0);
    // racer 0 grabbing its zone-1 string, far above the item, takes it
    let it = *g.item(1, 0).unwrap();
    let a = &mut r.racers[0].ape;
    a.pos = r.course.vines[it.vine].anchor.sub(v3(0, int(6), 0));
    a.vel = V3::ZERO;
    let grab = Controls {
        grab: true,
        ..Controls::default()
    };
    r.step(&[
        grab,
        Controls::default(),
        Controls::default(),
        Controls::default(),
    ]);
    assert_eq!(r.racers[0].ape.swinging(), Some(it.vine));
    assert_eq!(r.racers[0].ape.list, 0b11);
}
