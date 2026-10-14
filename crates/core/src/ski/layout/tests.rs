//! The Ski Slope's layout: the run, its forks and walls, the lifts, the
//! hangers, the landmarks, and what every map needs (starts, crates,
//! bananas, low vines, a way on everywhere).

use super::*;
use crate::fx::deg;
use crate::geom::right_of;
use crate::rules::{BODY_RADIUS, CEILING};

fn courses() -> Vec<Course> {
    (0..4u64)
        .map(|k| {
            if k == 0 {
                ski()
            } else {
                ski_seeded(0x5C1_5EED_0001 + k * 7919)
            }
        })
        .collect()
}

#[test]
fn the_run_is_about_two_km_and_always_runs_downhill() {
    let c = ski();
    assert!(
        (int(1800)..=int(2000)).contains(&c.length),
        "{} m",
        c.length / ONE
    );
    let fall = c.heading_at(0);
    // every heading within 50° of the fall line
    let limit = fx::cos(deg(50));
    for (i, h) in c.heading.iter().enumerate() {
        assert!(h.dot(fall) >= limit, "sample {i} heads {}", h.dot(fall));
    }
    // and the run drops all the way, past the finish too
    let mut last = i32::MAX;
    for s in (0..c.length / ONE + 140).step_by(5) {
        let p = c.point(int(s));
        let y = c.terrain(p.x, p.z);
        assert!(y < last || s == 0, "not downhill at {s} m");
        last = y;
    }
    // the last stretch is gentle, for the run-out to the village
    let g = |s: i32| {
        c.terrain
            .downhill(c.point(int(s)).x, c.point(int(s)).z)
            .len()
    };
    assert!(g(c.length / ONE + 80) * 3 < g(c.length / ONE - 400));
}

#[test]
fn two_forks_each_split_round_an_island_and_join() {
    let c = ski();
    let lay = c.ski.as_ref().unwrap();
    assert_eq!(lay.forks.len(), 2);
    let [a, b] = [lay.forks[0], lay.forks[1]];
    assert!(a.to < b.from, "one after the other");
    // asymmetric: each branch its own distance out, the forks different
    assert_ne!(a.left, a.right);
    assert_ne!(b.left, b.right);
    assert_ne!(
        (a.left, a.right, a.to - a.from),
        (b.left, b.right, b.to - b.from)
    );
    for f in [a, b] {
        let mid = (f.from + f.to) / 2;
        let [(m0, h0), (m1, h1)] = c.lanes(mid);
        assert!(m0 + h0 < m1 - h1, "an island between the branches");
        // before and after, one run
        assert_eq!(c.lanes(f.from - int(5))[0], (0, c.half_width));
        assert_eq!(c.lanes(f.to + int(5))[0], (0, c.half_width));
        // the spine is straight through a fork, well inside its bend radius
        let h0 = c.heading_at(f.from);
        let h1 = c.heading_at(f.to);
        assert!(h0.dot(h1) > ONE * 99 / 100);
    }
}

#[test]
fn the_walls_keep_an_ape_off_the_islands_and_inside_the_branches() {
    let c = ski();
    for f in &c.ski.as_ref().unwrap().forks {
        let s = (f.from + f.to) / 2;
        let i = (s / crate::course::STEP) as usize;
        let [(m0, h0), (m1, h1)] = c.lanes(s);
        let r = right_of(c.heading_at(s));
        let at = |off: i32| {
            let p = c.point(s).add(r.scale(off));
            v3(p.x, c.terrain(p.x, p.z) + int(20), p.z)
        };
        // the island's middle: put back into the nearer branch
        let island = (m0 + h0 + m1 - h1) / 2;
        let (p, _, hit) = c.contain(at(island), V3::ZERO, i);
        assert!(hit, "the island is a wall");
        let (_, side) = c.progress(p, i);
        let inside = |m: i32, h: i32| (side - m).abs() <= h - BODY_RADIUS + cm(5);
        assert!(
            inside(m0, h0) || inside(m1, h1),
            "into a branch: {}",
            side / ONE
        );
        // in a branch: left alone
        for m in [m0, m1] {
            let (p, _, hit) = c.contain(at(m), v3(0, 0, ONE), i);
            assert!(!hit, "branch at {}", m / ONE);
            assert_eq!(p, at(m));
        }
        // beyond a branch's outer edge: back inside
        let (p, _, hit) = c.contain(at(m1 + h1 + int(4)), V3::ZERO, i);
        assert!(hit);
        assert!(c.progress(p, i).1 <= m1 + h1);
    }
}

#[test]
fn progress_runs_smoothly_down_each_branch() {
    let c = ski();
    for f in &c.ski.as_ref().unwrap().forks {
        for k in 0..2 {
            let mut hint = c.nearest_anywhere(c.point(f.from - int(20)));
            let mut last = i32::MIN;
            let mut s = f.from - int(20);
            while s < f.to + int(20) {
                let m = c.lanes(s)[k].0;
                let p = c.point(s).add(right_of(c.heading_at(s)).scale(m));
                hint = c.nearest(p, hint);
                let (got, side) = c.progress(p, hint);
                assert!(got > last, "progress went back at {} m", s / ONE);
                assert!((got - s).abs() < int(3), "{} vs {}", got / ONE, s / ONE);
                assert!((side - m).abs() < int(2));
                last = got;
                s += int(2);
            }
        }
    }
}

#[test]
fn every_lift_is_one_straight_line_and_they_chain_down_the_hill() {
    let c = ski();
    let lay = c.ski.as_ref().unwrap();
    assert!(lay.lifts.len() >= 3);
    for (k, l) in lay.lifts.iter().enumerate() {
        // the top terminal is higher, the towers on the line between
        assert!(l.top.y > l.bottom.y, "lift {k} runs uphill");
        let dir = l.dir();
        let r = right_of(dir);
        let mut last = 0;
        for t in &l.towers {
            let d = t.flat().sub(l.bottom.flat());
            assert!(d.dot(r).abs() < cm(5), "lift {k}: a tower off the line");
            assert!(d.dot(dir) > last, "lift {k}: towers in order");
            last = d.dot(dir);
            assert_eq!(t.y, c.terrain(t.x, t.z), "on the ground");
        }
        assert!(last < l.length());
        // no tower or terminal stands in a lane (apes would pass through)
        for (j, foot) in l.towers.iter().chain([&l.top, &l.bottom]).enumerate() {
            let (s, off) = c.progress(*foot, c.nearest_anywhere(*foot));
            for (m, h) in c.lanes(s) {
                assert!(
                    (off - m).abs() > h + int(1),
                    "lift {k}: support {j} in a lane at {} m, {} m right",
                    s / ONE,
                    off / ONE
                );
            }
        }
        // chained: its bottom terminal near the next lift's top terminal
        if let Some(next) = lay.lifts.get(k + 1) {
            let gap = l.bottom.flat().dist(next.top.flat());
            assert!(gap < int(25), "lift {k} to {}: {} m", k + 1, gap / ONE);
        }
    }
}

#[test]
fn every_vine_hangs_from_something_of_the_right_kind() {
    for c in courses() {
        let lay = c.ski.as_ref().unwrap();
        assert!(c.mushrooms.is_empty(), "nothing bounces");
        assert_eq!(lay.hangers.len(), c.vines.len());
        let mut kinds = [0; 4];
        for (i, h) in lay.hangers.iter().enumerate() {
            assert_eq!(h.vine, i);
            let v = &c.vines[i];
            match h.kind {
                HangerKind::Chair { lift } => {
                    kinds[0] += 1;
                    // a chair of that lift, its seat the anchor
                    let l = &lay.lifts[lift];
                    let on = (1..l.chairs()).any(|k| {
                        [-1, 1]
                            .iter()
                            .any(|&side| l.chair(k, side) == (h.from, h.to))
                    });
                    assert!(on, "vine {i} on a chair of lift {lift}");
                    assert_eq!(h.to, v.anchor);
                }
                HangerKind::Tree { tree } => {
                    kinds[1] += 1;
                    let t = lay.trees[tree];
                    assert_eq!(h.to, v.anchor);
                    assert_eq!(h.from.flat(), t.at.flat(), "from its trunk");
                    let reach = t.at.flat().dist(v.anchor.flat());
                    assert!(
                        reach <= BRANCH_MAX + cm(10),
                        "vine {i}: a {} m branch",
                        reach / ONE
                    );
                    assert!(
                        t.at.y + t.height > h.from.y + int(4),
                        "vine {i}: tree too short"
                    );
                }
                HangerKind::Helicopter => {
                    kinds[2] += 1;
                    assert_eq!(h.to, v.anchor);
                    assert!(h.from.y > h.to.y);
                }
                HangerKind::Wire => {
                    kinds[3] += 1;
                    // between two conifers, across the run
                    for end in [h.from, h.to] {
                        assert!(lay.trees.iter().any(|t| t.at.flat() == end.flat()));
                    }
                }
            }
            // and every anchor is at vine height above the snow
            let g = c.terrain(v.anchor.x, v.anchor.z);
            let up = v.anchor.y - g;
            assert!(
                (int(20)..CEILING - int(8)).contains(&up),
                "vine {i}: {} m up",
                up / ONE
            );
            assert!(v.anchor.y - v.length > g, "vine {i} reaches the snow");
        }
        assert!(kinds[0] >= 15, "chairs {kinds:?}");
        assert!(kinds[1] >= 60, "trees {kinds:?}");
        assert!((1..=2).contains(&kinds[2]), "helicopters {kinds:?}");
    }
}

#[test]
fn every_lane_has_a_way_on_and_low_vines_to_climb_back_by() {
    for c in courses() {
        // walking down each lane (both branches at a fork), there is always
        // a vine ahead within 36 m over that lane (stations are 22–28 m
        // apart, each vine up to 3 m off), and a low one within 70 m
        for k in 0..2 {
            let mut s = int(20);
            while s < c.length - int(60) {
                let lanes = c.lanes(s);
                let (m, h) = lanes[k];
                let over = |i: usize, reach: i32, low: bool| {
                    let v = &c.vines[i];
                    let (vs, side) = c.progress(v.anchor, c.nearest_anywhere(v.anchor));
                    let g = c.terrain(v.anchor.x, v.anchor.z);
                    vs > s
                        && vs <= s + reach
                        && (side - c.lanes(vs)[k].0).abs() <= c.lanes(vs)[k].1
                        && (!low || v.anchor.y - v.length - g < int(5))
                };
                let n = c.vines.len();
                assert!(
                    (0..n).any(|i| over(i, int(36), false)),
                    "no vine ahead at {} m in lane {k} ({}, {})",
                    s / ONE,
                    m / ONE,
                    h / ONE
                );
                if s > int(60) && s < c.length - int(100) {
                    assert!(
                        (0..n).any(|i| over(i, int(70), true)),
                        "no low vine ahead at {} m in lane {k}",
                        s / ONE
                    );
                }
                s += int(4);
            }
        }
    }
}

#[test]
fn crates_and_bananas_are_in_both_branches_of_each_fork() {
    let c = ski();
    for f in &c.ski.as_ref().unwrap().forks {
        for k in 0..2 {
            let in_branch = |p: V3| {
                let (s, side) = c.progress(p, c.nearest_anywhere(p));
                let [(m0, h0), (m1, h1)] = c.lanes(s);
                s > f.from + f.ramp
                    && s < f.to - f.ramp
                    && m0 + h0 < m1 - h1
                    && (side - c.lanes(s)[k].0).abs() <= c.lanes(s)[k].1
            };
            assert!(
                c.bananas.iter().any(|&b| in_branch(b)),
                "bananas, branch {k}"
            );
            assert!(c.crates.iter().any(|&b| in_branch(b)), "crates, branch {k}");
        }
    }
}

#[test]
fn the_start_stands_on_the_summit_platform() {
    let c = ski();
    let deck = c.platforms[0].top;
    for s in c.starts {
        assert_eq!(c.support(s), deck);
        assert_eq!(s.y, deck + BODY_DROP);
    }
    // the start row hangs just off the platform's front edge
    let edge = c.platforms[0]
        .centre
        .add(c.platforms[0].fwd.scale(c.platforms[0].half_len));
    let near = c
        .vines
        .iter()
        .filter(|v| v.anchor.flat().dist(edge.flat()) < int(12))
        .count();
    assert!(near >= 5, "{near}");
}

#[test]
fn one_of_each_landmark_in_order_down_the_hill_outside_the_run() {
    use LandmarkKind::*;
    let c = ski();
    let marks = &c.ski.as_ref().unwrap().landmarks;
    let order = [
        Summit,
        Snowman,
        Lodge,
        SkiJump,
        IceApe,
        FrozenFalls,
        Village,
    ];
    assert_eq!(marks.iter().map(|m| m.kind).collect::<Vec<_>>(), order);
    for w in marks.windows(2) {
        assert!(w[0].s < w[1].s, "{:?} before {:?}", w[0].kind, w[1].kind);
    }
    // the summit stands behind the start, clear of the start camera
    let back = marks[0].at.flat().sub(c.path[0]).dot(c.heading[0]);
    assert!(back <= -int(16), "{}", back / ONE);
    // the snowman and the ice ape on the forks' islands
    let forks = &c.ski.as_ref().unwrap().forks;
    for (m, f) in [(&marks[1], forks[0]), (&marks[4], forks[1])] {
        assert!(m.s > f.from && m.s < f.to, "{:?} on its fork", m.kind);
    }
    // and no conifer on their island for 40 m in front of them
    for c in courses() {
        let lay = c.ski.as_ref().unwrap();
        for m in [&lay.landmarks[1], &lay.landmarks[4]] {
            for t in &lay.trees {
                let (s, off) = c.progress(t.at, c.nearest_anywhere(t.at));
                let [(m0, h0), (m1, h1)] = c.lanes(s);
                let on_island = off > m0 + h0 && off < m1 - h1;
                assert!(
                    !(on_island && s > m.s - int(40) && s < m.s),
                    "a tree {} m in front of the {:?}",
                    (m.s - s) / ONE,
                    m.kind
                );
            }
        }
    }
    // none in a lane, with room to be drawn big (5 m clear of the lanes)
    for m in marks
        .iter()
        .filter(|m| m.kind != Summit && m.kind != Village)
    {
        let i = c.nearest_anywhere(m.at);
        let (s, side) = c.progress(m.at, i);
        for (mid, half) in c.lanes(s) {
            assert!(
                (side - mid).abs() >= half + int(5),
                "{:?} is {} m from a lane",
                m.kind,
                ((side - mid).abs() - half) / ONE
            );
        }
    }
}

#[test]
fn the_jaguar_comes_in_from_the_edge_of_the_branch_an_ape_is_in() {
    // Floor is Lava at a fork: from the branch's own edge, never the island
    let c = ski();
    for f in &c.ski.as_ref().unwrap().forks {
        let s = (f.from + f.to) / 2;
        let lanes = c.lanes(s + int(18));
        for (k, &(m, h)) in c.lanes(s).iter().enumerate() {
            for d in [-int(5), int(5)] {
                let p = c.point(s).add(right_of(c.heading_at(s)).scale(m + d));
                let at = v3(p.x, c.terrain(p.x, p.z) + BODY_DROP, p.z);
                let hint = c.nearest_anywhere(at);
                let e = crate::jaguar::Eaten::new(&c, at, hint, 0);
                let (_, side) = c.progress(e.from, c.nearest_anywhere(e.from));
                let (lm, lh) = lanes[k];
                assert!(
                    (side - lm).abs() <= lh && (side - lm).abs() > lh - int(2),
                    "branch {k} ({} ± {}): from {}",
                    m / ONE,
                    h / ONE,
                    side / ONE
                );
            }
        }
    }
}
