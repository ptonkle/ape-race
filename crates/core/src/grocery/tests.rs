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
/// middle), from the ceiling, under it.
#[test]
fn every_vine_hangs_clear_of_the_shelves() {
    let (c, s) = store();
    let hit = |q: V3| {
        s.near(q).iter().any(|&k| {
            let sol = &s.list[k as usize];
            sol.kind != SolidKind::Roof && sol.holds(q, BODY_RADIUS)
        })
    };
    for (i, v) in c.vines.iter().enumerate() {
        assert!(v.anchor.y <= CEILING_AT, "vine {i} above the ceiling");
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

/// The route is about 90 s long: some 2 km.
#[test]
fn the_route_is_about_two_kilometres() {
    let c = grocery();
    let len = c.length / ONE;
    assert!((1_850..=2_050).contains(&len), "{len} m");
    assert_eq!(c.path.len() as i32 - 1, c.length / STEP);
}

/// Every aisle off the route is closed at both mouths (a runner cannot get
/// in: `ai::walled_off` would leave it no vine), and the route's three are
/// open from end to end.
#[test]
fn the_aisles_off_the_route_are_closed_and_the_route_s_are_open() {
    let (_, s) = store();
    let mouth = |w: i32, e: i32, z: i32| plan(int(w + e) / 2, int(z)).add(v3(0, int(5), 0));
    let runs = layout::RUNS;
    for a in 1..runs.len() {
        let (w, e) = (runs[a - 1].0, runs[a].0);
        let on_route = ROUTE_AISLES.iter().any(|&(x, _)| x + 1 == a);
        for z in [FRONT + 4, BACK - 4] {
            let shut = s.holds(mouth(w, e, z), 0);
            assert_eq!(shut, !on_route, "aisle {a} at plan z {z}");
        }
    }
}
