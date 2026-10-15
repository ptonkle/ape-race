//! Sector Ape's layout (`sector()` and its seeded layouts).

use super::*;
use crate::ape::{Ape, Controls, Happening, Mode, HIT_RADIUS};
use crate::fx::cm;
use crate::rules::Kind;
use crate::spacecraft::{wreck_passage, MechKind};

fn layouts() -> Vec<Course> {
    (0..4u64)
        .map(|k| {
            if k == 0 {
                sector()
            } else {
                sector_seeded(0x5EC7_5EED_0001 + k * 7919)
            }
        })
        .collect()
}

fn lay(c: &Course) -> &SectorLayout {
    c.sector.as_deref().unwrap()
}

#[test]
fn the_course_is_about_2_3_km_with_every_vine_described() {
    for c in layouts() {
        let s = lay(&c);
        assert!(
            (int(2200)..=int(2600)).contains(&c.length),
            "{}",
            c.length / ONE
        );
        assert_eq!(s.hangs.len(), c.vines.len());
        assert_eq!(s.hangers.len(), c.vines.len());
        assert_eq!(s.by_at.len(), c.vines.len());
        // no floor near: the "ground" is far below the boundary
        assert!(c.terrain(0, 0) < s.boundary.below - int(100));
        // every vine's end, and its anchor, inside the boundary
        for i in 0..c.vines.len() {
            for p in [c.anchor(i, 0), c.tip(i, 0)] {
                let h = c.nearest_anywhere(p);
                assert!(
                    !c.out_of_bounds(p, h),
                    "vine {i} at {} m",
                    c.vines[i].at / ONE
                );
            }
            let h = c.hang(i);
            assert!((h.len() - ONE).abs() < ONE / 100, "vine {i} hangs {h:?}");
        }
    }
}

#[test]
fn about_half_the_vines_hang_some_other_way_than_down() {
    for c in layouts() {
        let s = lay(&c);
        let odd = s.hangs.iter().filter(|h| **h != DOWN).count();
        let share = odd * 100 / s.hangs.len();
        assert!((40..=60).contains(&share), "{share}% odd");
        // and of every kind: up, level, slanted
        let up = s.hangs.iter().filter(|h| h.y > ONE * 9 / 10).count();
        let level = s.hangs.iter().filter(|h| h.y.abs() < ONE / 10).count();
        let slant = s
            .hangs
            .iter()
            .filter(|h| h.y.abs() > ONE / 10 && h.y.abs() < ONE * 9 / 10)
            .count();
        assert!(
            up >= 5 && level >= 10 && slant >= 10,
            "{up} {level} {slant}"
        );
    }
}

#[test]
fn every_stretch_has_a_way_on() {
    // (a vine hanging within 35 m ahead anywhere along the course)
    for c in layouts() {
        let mut s = int(10);
        while s < c.length - int(30) {
            let any = c.vines.iter().any(|v| v.at >= s && v.at < s + int(35));
            assert!(any, "{} m: no vine ahead ({:?})", s / ONE, {
                let mut a: Vec<i32> = c
                    .vines
                    .iter()
                    .map(|v| v.at / ONE)
                    .filter(|a| (a - s / ONE).abs() < 60)
                    .collect();
                a.sort();
                a
            });
            s += int(5);
        }
    }
}

#[test]
fn decks_to_be_put_back_on_come_often_with_a_way_off_each() {
    for c in layouts() {
        let s = lay(&c);
        let mut ats: Vec<i32> = s
            .pads
            .iter()
            .map(|&k| {
                let p = c.platforms[k].centre;
                c.progress(p, c.nearest_anywhere(p)).0
            })
            .collect();
        ats.sort();
        assert!(ats[0] < int(20));
        for w in ats.windows(2) {
            assert!(
                w[1] - w[0] <= int(290),
                "{} m to {} m",
                w[0] / ONE,
                w[1] / ONE
            );
        }
        assert!(c.length - ats[ats.len() - 1] <= int(290));
        // from each, running off its front edge holding grab catches a vine
        for &k in &s.pads {
            let (at, yaw) = c.pad_spot(k);
            let mut a = Ape::new_on(&c, Kind::Gorilla, false, at, yaw);
            a.hint = c.nearest_anywhere(at);
            let fwd = crate::geom::facing(yaw, 0);
            let mut caught = false;
            for t in 0..480u32 {
                let ctl = Controls {
                    fwd: 1,
                    grab: true,
                    yaw,
                    ..Controls::default()
                };
                a.step(&ctl, &c, t);
                if a.swinging().is_some() {
                    caught = true;
                    break;
                }
                let _ = fwd;
            }
            assert!(caught, "nothing caught off deck {k}");
        }
    }
}

/// Whether `p` is inside body `b`'s box (by `r` more each way).
fn inside(b: &Body, p: V3, r: i32) -> bool {
    let d = p.sub(b.frame.t);
    let ax = b.axes();
    d.dot(ax[0]).abs() < b.half.x + r
        && d.dot(ax[1]).abs() < b.half.y + r
        && d.dot(ax[2]).abs() < b.half.z + r
}

#[test]
fn no_vine_hangs_through_a_solid_body_but_its_own() {
    for c in layouts() {
        let s = lay(&c);
        for i in 0..c.vines.len() {
            let own = match s.hangers[i] {
                Hanger::Body(k) | Hanger::Switch(k) => Some(k),
                // (a mech's vines swing with its walk: they sweep past things)
                Hanger::Mech { .. } => continue,
                _ => None,
            };
            let (a, h, len) = (c.anchor(i, 0), c.hang(i), c.vines[i].length);
            let mut t = int(2);
            while t <= len {
                let p = a.add(h.scale(t));
                for (k, b) in s.bodies.iter().enumerate() {
                    if b.solid() && Some(k) != own {
                        assert!(
                            !inside(b, p, 0),
                            "vine {i} ({:?} at {} m, {:?}) through body {k} ({:?})",
                            s.hangers[i],
                            c.vines[i].at / ONE,
                            s.hangs[i],
                            b.model
                        );
                    }
                }
                t += ONE;
            }
        }
    }
}

#[test]
fn the_body_index_lists_every_body_an_ape_could_touch() {
    let c = sector();
    let s = lay(&c);
    for (k, b) in s.bodies.iter().enumerate() {
        if !b.solid() {
            continue;
        }
        let ax = b.axes();
        let out = HIT_RADIUS - cm(10);
        for &(i, j, l) in &[
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ] {
            let p = b
                .frame
                .t
                .add(ax[0].scale((b.half.x + out) * i))
                .add(ax[1].scale((b.half.y + out) * j))
                .add(ax[2].scale((b.half.z + out) * l));
            let hint = c.nearest_anywhere(p);
            if c.out_of_bounds(p, hint) {
                continue;
            }
            assert!(
                s.bodies_near(hint).contains(&(k as u16)),
                "body {k} ({:?}) missing at {} m",
                b.model,
                hint * 2
            );
        }
    }
}

#[test]
fn finding_a_vine_in_reach_by_its_place_is_the_same_as_looking_at_all() {
    let c = sector();
    let mut a = Ape::new_on(&c, Kind::Orangutan, false, V3::ZERO, 0);
    let mut checked = 0;
    for i in (0..c.vines.len()).step_by(2) {
        for (k, d) in [
            v3(cm(300), 0, 0),
            v3(0, cm(300), cm(200)),
            v3(-cm(500), cm(100), 0),
        ]
        .iter()
        .enumerate()
        {
            let tick = (i * 37 + k * 11) as u32;
            a.pos = c.tip(i, tick).add(*d);
            a.hint = c.nearest_anywhere(a.pos);
            a.mode = Mode::Flying;
            let near = a.vine_in_reach(&c, tick);
            assert_eq!(near, a.reach_any_way_all(&c, tick), "by vine {i}");
            checked += near.is_some() as u32;
        }
    }
    assert!(checked > 100, "{checked}");
}

#[test]
fn the_wreck_passage_is_clear_and_the_hull_round_it_solid() {
    for c in layouts() {
        let s = lay(&c);
        let p = wreck_passage(&s.wreck.frame, s.wreck.gap);
        let hidden: Vec<&Body> = s
            .bodies
            .iter()
            .filter(|b| b.model == Model::Hidden)
            .collect();
        let sweep_hits = |off: V3| {
            let from = p.entry.sub(p.axis.scale(int(10))).add(off);
            let to = p.exit.add(p.axis.scale(int(10))).add(off);
            hidden
                .iter()
                .any(|b| sweep(b.frame.t, &b.axes(), b.half, HIT_RADIUS, from, to).is_some())
        };
        assert!(!sweep_hits(V3::ZERO), "the passage is blocked");
        // a little off the middle still clear; well off it, the hull
        let up = s.wreck.frame.c1.norm();
        assert!(!sweep_hits(up.scale(cm(150))));
        assert!(
            sweep_hits(up.scale(p.height / 2 + int(2))),
            "no hull above it"
        );
        assert!(
            sweep_hits(up.scale(-(p.height / 2 + int(2)))),
            "no hull below it"
        );
        // and the gap is 14 by 10 m (T88)
        assert!((p.width - int(14)).abs() < ONE / 100 && (p.height - int(10)).abs() < ONE / 100);
        // its snipe vine is on its line short of the mouth
        let route = s.routes.iter().find(|r| r.hidden).expect("a hidden route");
        let snipe = c.tip(route.vines[0], 0);
        let along = snipe.sub(p.middle).dot(p.axis);
        let off = snipe.sub(p.middle).reject(p.axis).len();
        assert!(
            along < -p.length / 2 && off < int(2),
            "{} {}",
            along / ONE,
            off / ONE
        );
    }
}

/// T88: flying into the wreck's passage on the way on launches the ape
/// straight down it, faster (at least `PASSAGE_FLOOR`), once; it threads the
/// rest without touching the hull. Going in the wrong way does nothing.
#[test]
fn flying_into_the_wreck_passage_launches_the_ape_through_it() {
    let lift = v3(0, crate::rules::BODY_DROP / 2, 0);
    for c in layouts() {
        let s = lay(&c);
        let p = s.wreck.passage;
        // (speed along it, drift across it and up it, m/s; and which way)
        for (speed, side, up, on) in [(45, 3, 2, true), (15, 0, 2, true), (30, 0, 0, false)] {
            let (mouth, way) = if on {
                (p.entry, p.axis)
            } else {
                (p.exit, p.axis.scale(-ONE))
            };
            let start = mouth.sub(way.scale(int(12))).add(lift);
            let mut a = Ape::new_on(&c, Kind::Chimpanzee, true, start, 0);
            a.vel = way
                .scale(int(speed))
                .add(p.across.scale(int(side)))
                .add(p.up.scale(int(up)));
            a.mode = Mode::Flying;
            a.hint = c.nearest_anywhere(start);
            let (mut launches, mut hits) = (0, 0);
            for t in 0..360 {
                let before = a.vel;
                a.step(&Controls::default(), &c, t);
                if a.happened.contains(&Happening::Launched) {
                    launches += 1;
                    let want = (before.len() + PASSAGE_LAUNCH).max(PASSAGE_FLOOR);
                    assert!(
                        a.vel.dot(p.axis) > want - ONE,
                        "launched at {} m/s, want {}",
                        a.vel.dot(p.axis) / ONE,
                        want / ONE
                    );
                    assert!(a.vel.reject(p.axis).len() < ONE, "not straight down it");
                }
                if a.happened.contains(&Happening::Wall) || a.happened.contains(&Happening::Rebound)
                {
                    hits += 1;
                }
                if a.pos.sub(mouth).dot(way) > p.length + int(5) {
                    break;
                }
            }
            if on {
                assert_eq!((launches, hits), (1, 0), "{speed} m/s in");
                assert!(a.pos.sub(p.exit).dot(p.axis) > 0, "not through");
            } else {
                assert_eq!(launches, 0, "launched going back");
            }
        }
    }
}

#[test]
fn the_hulk_has_its_door_and_switch_and_a_clear_corridor() {
    let c = sector();
    let s = lay(&c);
    let d = s.door;
    let (door_s, _) = c.progress(d.frame.t, c.nearest_anywhere(d.frame.t));
    assert!(door_s > int(HULK.0) && door_s < int(HULK.1));
    let sw = c.vines[d.switch].at;
    assert!(
        sw > door_s && sw < door_s + int(20),
        "switch {} m",
        sw / ONE
    );
    assert!(matches!(s.hangers[d.switch], Hanger::Switch(_)));
    assert_eq!(s.hangs[d.switch], DOWN);
    // the corridor is clear end to end while the door is open
    let from = c.point(int(HULK.0 - 5));
    let to = c.point(int(HULK.1 + 5));
    let lift = v3(0, HULK_Y, 0);
    for b in s.bodies.iter().filter(|b| b.model == Model::Hidden) {
        assert!(sweep(
            b.frame.t,
            &b.axes(),
            b.half,
            HIT_RADIUS,
            from.add(lift),
            to.add(lift)
        )
        .is_none());
    }
    // the three ways round it
    for name in ["corridor", "over the hulk", "round the hulk"] {
        assert!(s.routes.iter().any(|r| r.name == name), "{name}");
    }
    assert!(s.routes.iter().any(|r| r.door));
}

#[test]
fn the_armada_has_ten_grunts_and_two_brutes_carrying_moving_vines() {
    let c = sector();
    let s = lay(&c);
    let brutes = s
        .mechs
        .iter()
        .filter(|m| m.mech.kind == MechKind::Brute)
        .count();
    assert_eq!((s.mechs.len() - brutes, brutes), (10, 2));
    let mut moving = 0;
    for (i, h) in s.hangers.iter().enumerate() {
        if let Hanger::Mech { mech, at } = *h {
            assert_eq!(c.anchor(i, 0), c.vines[i].anchor);
            let (c0, slack) = c.anchor_bound(i);
            for t in (0..8000).step_by(97) {
                let a = c.anchor(i, t);
                assert_eq!(a, s.mechs[mech].point(at, t));
                assert!(a.dist(c0) <= slack, "vine {i} out of its bound");
            }
            moving += (c.anchor(i, 0) != c.anchor(i, 600)) as u32;
        }
    }
    assert!(moving >= 30, "{moving}");
}

#[test]
fn a_mechs_frame_is_its_pose_wherever_it_is() {
    // (`MechPath::frame` moves the pose it worked out once: the same frame
    // as posing it afresh)
    let c = sector();
    for m in &lay(&c).mechs {
        for t in (0..20_000).step_by(37) {
            let period = m.period.max(1);
            let phase = ((t % period) as i64 * fx::TURN as i64 / period as i64) as i32;
            let at = m.from.lerp(m.to, (ONE - fx::cos(phase)) / 2);
            let posed = crate::spacecraft::pose(at, m.yaw, 0, 0, m.scale);
            assert_eq!(m.frame(t), posed, "tick {t}");
        }
    }
}

#[test]
fn the_finish_is_the_black_hole_horizon_with_a_deck_under_it() {
    let c = sector();
    let s = lay(&c);
    let (hole, r) = s.black_hole;
    let line = c.point(c.length);
    let rim = mul(r, super::super::rules::HORIZON);
    let at_height = v3(line.x, hole.y, line.z);
    assert!(
        (hole.dist(at_height) - rim).abs() < ONE,
        "{}",
        hole.dist(at_height) / ONE
    );
    let deck = &c.platforms[s.finish_deck];
    assert!(deck.top < hole.y - rim && deck.top > s.boundary.below);
    // an ape dropped from over the line comes down on it
    let p = v3(line.x, int(10), line.z);
    let mut a = Ape::new_on(&c, Kind::Lemur, false, p, 0);
    a.mode = Mode::Flying;
    a.hint = c.nearest_anywhere(p);
    assert!(c.support(v3(deck.centre.x, deck.top + int(5), deck.centre.z)) == deck.top);
}

#[test]
fn the_start_deck_holds_the_starters_and_nothing_solid_is_behind_it() {
    let c = sector();
    let s = lay(&c);
    for p in c.starts {
        assert_eq!(c.support(p), 0);
    }
    assert_eq!(s.lobby.1, c.start_yaw);
    // the chase camera sits some 10 m behind: nothing solid within 16 m
    let back = c.path[0]
        .sub(c.heading[0].scale(int(10)))
        .add(v3(0, int(4), 0));
    for b in s.bodies.iter().filter(|b| b.solid()) {
        assert!(!inside(b, back, int(6)), "{:?} behind the start", b.model);
    }
}

#[test]
fn the_set_pieces_pass_over_and_fight_far_off() {
    let c = sector();
    let s = lay(&c);
    let pass = s.set_pieces.iter().find_map(|p| match *p {
        SetPiece::Pass {
            from,
            to,
            start,
            end,
        } => Some((from, to, start, end)),
        _ => None,
    });
    let (from, to, start, end) = pass.expect("the mothership's pass");
    assert!(
        from.y > int(60) && to.y > int(60) && start > crate::race::COUNTDOWN as u32 && end > start
    );
    let far = |f: &Affine| {
        let h = c.nearest_anywhere(f.t);
        c.progress(f.t, h).1.abs() > int(150) || f.t.y.abs() > int(150)
    };
    let fights = s
        .set_pieces
        .iter()
        .filter(|p| match p {
            SetPiece::Dogfight { frame } | SetPiece::Battle { frame } => far(frame),
            _ => false,
        })
        .count();
    assert!(fights >= 4, "{fights}");
}
