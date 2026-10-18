//! Rattlesnake Ravine's contract: the map's wiring, the hold, the movers,
//! the teeth's reach and the snake vines' saved state.

use super::*;
use crate::fx::int;
use crate::map::Map;

#[test]
fn only_rattlesnake_ravine_has_a_ravine_layout() {
    assert_eq!(Map::from_key("ravine"), Some(Map::Ravine));
    for map in Map::ALL {
        let c = map.course();
        // (Clean-Up on Aisle Ape's solids are a Ravine layout, `grocery.rs`)
        assert_eq!(
            c.ravine.is_some(),
            matches!(map, Map::Ravine | Map::Grocery),
            "{map:?}"
        );
        assert_eq!(Map::of(&c), map);
    }
    let c = ravine();
    assert_eq!(c.name, "ravine");
    let r = c.ravine.as_deref().unwrap();
    assert_eq!(r.hangers.len(), c.vines.len(), "one hanger per vine");
    let strings: serde_json::Value =
        serde_json::from_str(include_str!("../../../../data/strings.json")).unwrap();
    assert_eq!(strings["map.ravine"], "Rattlesnake Ravine");
}

#[test]
fn the_ravine_is_on_the_map_choice_and_in_random() {
    assert!(!Map::Ravine.held());
    assert!(Map::offered().any(|m| m == Map::Ravine));
    assert!(
        (0..500u64).any(|seed| Map::random(seed, Map::Jungle) == Map::Ravine),
        "Random never picks the Ravine"
    )
}

#[test]
fn a_lizard_walks_its_wall_and_back_eased_and_periodic() {
    let mut c = ravine();
    let r = c.ravine.as_deref_mut().unwrap();
    r.lizards.clear();
    let from = v3(int(10), int(2), int(40));
    let to = v3(int(10), int(14), int(40));
    r.lizards.push(Lizard {
        from,
        to,
        out: v3(-ONE, 0, 0),
        period: 400,
        phase: 0,
        bouncer: 0,
        colour: LizardColour::Red,
    });
    let (a, up) = r.lizard_at(0, 0);
    assert_eq!(a, from);
    assert!(up.y > ONE - cm(1), "walks up first: {up:?}");
    let (top, _) = r.lizard_at(0, 200);
    assert!(top.dist(to) < cm(1), "at the top half way: {top:?}");
    let (_, down) = r.lizard_at(0, 300);
    assert!(down.y < -ONE + cm(1), "walks back down: {down:?}");
    assert_eq!(r.lizard_at(0, 37), r.lizard_at(0, 437), "periodic");
    // its bouncer moves with it
    let rest = c.mushrooms[0].top;
    let moved = c.bouncer(0, 200).top;
    assert!(moved.sub(rest).sub(to.sub(from)).len() < cm(1), "{moved:?}");
}

/// The course with only one straight track from a hole at x=0 to one at
/// x=100 m, and a cart on it whose bouncer is `Course.mushrooms[0]`.
fn with_cart() -> Course {
    let mut c = ravine();
    let start = v3(0, int(3), int(500));
    let end = v3(int(100), int(3), int(500));
    let hole = |at: V3, out: V3| Hole {
        at,
        out,
        half_width: int(2),
        height: int(3),
    };
    c.mushrooms[0].top = start.add(v3(0, int(1), 0));
    let r = c.ravine.as_deref_mut().unwrap();
    r.tracks.clear();
    r.carts.clear();
    r.tracks.push(Track {
        points: vec![start, v3(int(50), int(3), int(500)), end],
        start: hole(start, v3(ONE, 0, 0)),
        end: hole(end, v3(-ONE, 0, 0)),
    });
    r.carts.push(Cart {
        track: 0,
        period: 1000,
        phase: 0,
        bouncer: 0,
    });
    c
}

#[test]
fn a_cart_runs_its_track_hole_to_hole_and_round_again() {
    let c = with_cart();
    let r = c.ravine.as_deref().unwrap();
    let (a, yaw) = r.cart_at(0, 0);
    assert_eq!(a, v3(0, int(3), int(500)), "out of the start hole");
    assert_eq!(yaw, crate::geom::yaw_of(v3(ONE, 0, 0)));
    let (mid, _) = r.cart_at(0, 500);
    assert!(mid.dist(v3(int(50), int(3), int(500))) < cm(5), "{mid:?}");
    let (near_end, _) = r.cart_at(0, 999);
    assert!(near_end.x > int(99), "{near_end:?}");
    assert_eq!(r.cart_at(0, 1000), r.cart_at(0, 0), "round again");
    // like a City car: its bouncer moves with it, and an ape over it at
    // that tick is on it
    let cap = c.bouncer(0, 500).top;
    assert!(cap.dist(v3(int(50), int(4), int(500))) < cm(5), "{cap:?}");
    let hands = cap.add(v3(0, BODY_DROP - cm(10), 0));
    assert_eq!(c.bouncer_under(hands, 500), Some(0));
    assert_eq!(c.bouncer_under(hands, 0), None, "it was elsewhere then");
}

#[test]
fn the_teeth_reach_a_body_but_the_gap_between_them_is_clear() {
    let at = v3(0, int(20), 0);
    let tooth = |x: i32, up: bool| {
        let jaw = if up {
            MOUTH_GAP / 2 + int(1)
        } else {
            -MOUTH_GAP / 2 - int(1)
        };
        let tip = if up { MOUTH_GAP / 2 } else { -MOUTH_GAP / 2 };
        Tooth {
            root: at.add(v3(x, jaw, 0)),
            tip: at.add(v3(x, tip, 0)),
            radius: cm(30),
        }
    };
    let skull = Skull {
        at,
        yaw: 0,
        teeth: vec![tooth(0, true), tooth(0, false)],
        keep: Room {
            kind: RoomKind::Skull,
            from: at.sub(v3(0, MOUTH_GAP / 2, int(4))),
            to: at.sub(v3(0, MOUTH_GAP / 2, -int(4))),
            half_width: int(4),
            height: MOUTH_GAP,
        },
        floor_spot: v3(0, 0, 0),
        floor_yaw: 0,
    };
    assert_eq!(MOUTH_GAP, 2 * TALLEST_APE);
    // the body (hands down to feet) in the middle of the gap is clear
    let middle = at.add(v3(0, BODY_DROP / 2, 0));
    assert!(!skull.brushes(middle));
    // hands up near an upper tip, or feet near a lower one: brushed
    // (70 cm off a tip: outside the tooth, inside the body's radius)
    assert!(skull.brushes(at.add(v3(0, MOUTH_GAP / 2 - cm(70), 0))));
    assert!(skull.brushes(at.add(v3(0, -MOUTH_GAP / 2 + BODY_DROP + cm(70), 0))));
    // to the side of the teeth, clear
    assert!(!skull.brushes(at.add(v3(int(3), MOUTH_GAP / 2, 0))));
    assert!(skull.keep.contains(middle));
}

#[test]
fn a_room_and_a_deck_know_what_is_in_and_on_them() {
    let room = Room {
        kind: RoomKind::Tunnel,
        from: v3(0, 0, 0),
        to: v3(0, 0, int(40)),
        half_width: int(5),
        height: int(8),
    };
    assert!(room.contains(v3(int(4), int(7), int(20))));
    assert!(
        !room.contains(v3(int(6), int(4), int(20))),
        "past a side wall"
    );
    assert!(!room.contains(v3(0, int(9), int(20))), "over the roof");
    assert!(!room.contains(v3(0, int(4), int(41))), "out of the mouth");
    let deck = Deck {
        kind: DeckKind::Log,
        from: v3(0, int(10), 0),
        to: v3(int(20), int(12), 0),
        half_width: cm(80),
    };
    assert_eq!(deck.top_at(v3(int(10), 0, cm(50))), Some(int(11)));
    assert_eq!(deck.top_at(v3(int(10), 0, int(1))), None);
    assert_eq!(deck.top_at(v3(int(21), 0, 0)), None);
}

#[test]
fn a_snapped_snake_vine_is_kept_in_the_save_state() {
    let mut c = ravine();
    let r = c.ravine.as_deref_mut().unwrap();
    r.hangers[3].snake = true;
    r.hangers[3].snapped_at = Some(777);
    let saved = snake_state(&c);
    assert_eq!(saved[3], Some(777));
    let mut fresh = ravine();
    put_snake_state(&mut fresh, &saved);
    assert_eq!(
        fresh.ravine.as_deref().unwrap().hangers[3].snapped_at,
        Some(777)
    );
    // off this map there is nothing to keep
    assert!(snake_state(&Map::Jungle.course()).is_empty());
}

// ---- the course (the course agent, 2026-10-17) ----

use super::layout::{leg, plan, ravine_upper_seeded, skull_spots, HALF, PACE, START_BACK};
use crate::course::Course;
use crate::geom::right_of;

/// The real course and a few other layouts from other seeds.
fn layouts() -> Vec<Course> {
    let mut v = vec![ravine()];
    for k in 1..4u64 {
        v.push(ravine_seeded(0x5A4E_C0DE_0001 + k * 7919));
    }
    v
}

fn lay(c: &Course) -> &RavineLayout {
    c.ravine.as_deref().unwrap()
}

/// The vines whose bottom is within reach of an ape on the ground.
fn low(c: &Course, i: usize) -> bool {
    let v = c.vines[i];
    v.anchor.y - v.length - crate::course::ground(v.anchor.x, v.anchor.z) < int(6)
}

/// Every place along `line` (points, each with the way on) from `skip_m`
/// in to `skip_end` from its end has two plain (not snake) vines in the
/// 30 m ahead, across the canyon, and a low vine within 60 m.
fn way_on(c: &Course, line: &[(V3, V3)], what: &str) {
    let r = lay(c);
    for (k, &(p, h)) in line.iter().enumerate() {
        let ahead = |reach: i32, i: usize| {
            let d = c.vines[i].anchor.sub(p).flat();
            let a = d.dot(h);
            a > -int(4) && a <= reach && d.len() <= reach + int(4)
        };
        let plain = (0..c.vines.len())
            .filter(|&i| !r.hangers[i].snake && ahead(int(30), i))
            .count();
        assert!(
            plain >= 2,
            "{what} point {k} at {p:?}: {plain} plain vines in the 30 m ahead"
        );
        let lows = (0..c.vines.len())
            .filter(|&i| low(c, i) && ahead(int(60), i))
            .count();
        assert!(
            lows >= 1,
            "{what} point {k} at {p:?}: no low vine in the 60 m ahead"
        );
    }
}

#[test]
fn there_is_a_way_on_at_every_stretch_of_both_passages() {
    for c in layouts() {
        let len = c.length;
        let spine: Vec<(V3, V3)> = (int(20)..len - int(40))
            .step_by(int(4) as usize)
            .map(|s| (c.point(s), c.heading_at(s)))
            .collect();
        way_on(&c, &spine, "the spine");
        let sp = lay(&c).split.as_ref().unwrap();
        let dir = sp.upper[sp.upper.len() - 1].sub(sp.upper[0]).flat().norm();
        let upper: Vec<(V3, V3)> = sp.upper.iter().map(|&p| (p, dir)).collect();
        way_on(&c, &upper, "the upper passage");
    }
}

#[test]
fn about_one_vine_in_seven_is_a_snake_more_up_top_never_the_only_one() {
    for c in layouts() {
        let r = lay(&c);
        let snakes = r.hangers.iter().filter(|h| h.snake).count();
        let share = snakes * 100 / r.hangers.len();
        assert!(
            (12..=18).contains(&share),
            "{snakes} of {}: {share}%",
            r.hangers.len()
        );
        // the upper passage: vines nearer its line than the spine
        let sp = r.split.as_ref().unwrap();
        let on_upper = |i: usize| {
            let a = c.vines[i].anchor.flat();
            let to_upper = sp.upper.iter().map(|p| p.flat().dist(a)).min().unwrap();
            let s = c.nearest_anywhere(c.vines[i].anchor);
            let to_spine = c.path[s].flat().dist(a);
            to_upper + int(15) < to_spine
        };
        let (up, rest): (Vec<usize>, Vec<usize>) = (0..c.vines.len()).partition(|&i| on_upper(i));
        let pct = |v: &[usize]| v.iter().filter(|&&i| r.hangers[i].snake).count() * 100 / v.len();
        assert!(
            2 * pct(&up) > 3 * pct(&rest),
            "the upper passage {}% snakes, the rest {}%",
            pct(&up),
            pct(&rest)
        );
        for (i, h) in r.hangers.iter().enumerate() {
            if !h.snake {
                continue;
            }
            assert!(!low(&c, i), "vine {i}: a low vine is never a snake");
            // a plain vine of the same station beside it
            let beside = (0..c.vines.len()).any(|j| {
                j != i
                    && !r.hangers[j].snake
                    && c.vines[j].anchor.flat().dist(h.at.flat()) < int(16)
            });
            assert!(beside, "snake {i} at {:?} is the only vine there", h.at);
        }
        // a fixed list: the same on every layout's stations, never the dice
        assert_eq!(
            r.hangers.iter().filter(|h| h.snapped_at.is_some()).count(),
            0
        );
    }
}

#[test]
fn the_upper_passage_is_shorter_and_sparser_than_the_lower() {
    let c = ravine();
    let r = lay(&c);
    let sp = r.split.as_ref().unwrap();
    let upper: i32 = sp
        .upper
        .windows(2)
        .map(|w| w[1].sub(w[0]).flat().len())
        .sum();
    let lower = (sp.rejoin - sp.fork) as i32 * crate::course::STEP;
    assert!(upper + int(25) < lower, "upper {upper:?} lower {lower:?}");
    assert_eq!(c.path[sp.fork].flat().dist(sp.upper[0].flat()), 0);
    assert!(
        c.path[sp.rejoin]
            .flat()
            .dist(sp.upper[sp.upper.len() - 1].flat())
            < int(1)
    );
    // the upper passage is the same place on the AI's measuring course
    let u = ravine_upper_seeded(0x5A4E_C0DE_0001);
    assert!(
        (u.length - (c.length - lower + upper)).abs() < int(2),
        "{:?}",
        u.length
    );
    // sparse: fewer vines a hundred metres than the lower passage's, where
    // the island stands between them (each passage's own vines)
    let off = |p: V3| c.progress(p, c.nearest_anywhere(p)).1.abs();
    let upper_line: Vec<V3> = sp
        .upper
        .iter()
        .copied()
        .filter(|&p| off(p) >= int(50))
        .collect();
    let near = |p: V3, q: V3| q.flat().dist(p.flat()) < int(14);
    // (plain vines: the upper passage's snake stations have three lanes)
    let plain = |i: usize| !r.hangers[i].snake;
    let up = (0..c.vines.len())
        .filter(|&i| {
            let v = c.vines[i];
            plain(i) && upper_line.iter().any(|&q| near(v.anchor, q)) && off(v.anchor) > int(25)
        })
        .count() as i32;
    let up_len = upper_line.len() as i32 * 8 + 28;
    let (a, b) = (
        c.nearest_anywhere(upper_line[0]),
        c.nearest_anywhere(upper_line[upper_line.len() - 1]),
    );
    let lo = (0..c.vines.len())
        .filter(|&i| plain(i) && c.path[a..=b].iter().any(|&q| near(c.vines[i].anchor, q)))
        .count() as i32;
    let lo_len = (b - a) as i32 * 2 + 28;
    assert!(
        up * lo_len * 10 < lo * up_len * 9,
        "upper {up} plain vines in {up_len} m, lower {lo} in {lo_len} m"
    );
}

#[test]
fn the_skull_saves_five_to_eight_seconds() {
    let c = ravine();
    let r = lay(&c);
    let skull = r.skull.as_ref().unwrap();
    let p = plan(false);
    let (into, out) = skull_spots(&p);
    // the way round along the spine against straight across through the
    // mouth (centreline to centreline)
    let round = out - into;
    let across = c.point(int(into)).flat().dist(c.point(int(out)).flat()) / ONE;
    let saves = round - across;
    // (in tenths of a second: the core is fixed point)
    let tenths = saves * 10 / PACE;
    assert!(
        (50..=80).contains(&tenths),
        "saves {saves} m: {}.{} s",
        tenths / 10,
        tenths % 10
    );
    // the mouth is halfway up the caves' wall (it stands between two, their
    // side walls being solid), in the fin, and its keep reaches through to
    // the out-leg
    let cave = r
        .rooms
        .iter()
        .find(|m| {
            m.kind == RoomKind::Cave
                && (m.from.flat().dist(c.point(int(into)).flat()) < int(12)
                    || m.to.flat().dist(c.point(int(into)).flat()) < int(12))
        })
        .expect("the skull cave");
    let up = skull.at.y - cave.from.y;
    assert!((up - cave.height / 2).abs() < int(2), "{up:?}");
    assert!(skull.keep.contains(skull.at));
    // a metre into the fin from the out-leg's face
    let far = c
        .point(int(out))
        .add(right_of(c.heading_at(int(out))).scale(-HALF - int(1)));
    assert!(
        skull.keep.contains(v3(far.x, skull.at.y, far.z)),
        "through to the out-leg"
    );
    assert_eq!(skull.teeth.len(), 28);
    // the way round: the hairpin is on the spine (nothing has to use it)
    assert!(p.marks[leg::HAIRPIN] > into && p.marks[leg::OUT_LEG] < out);
    // brushed apes land on the floor under the jaw, in the in-leg
    assert!(skull.floor_spot.y < int(2) + crate::rules::BODY_DROP);
    assert!(skull.floor_spot.flat().dist(skull.at.flat()) < int(6));
}

/// Every top (a wall's rim, a rib's or island's, a rock's, a deck's, a
/// roof's) within 5 m across of `at`, as heights.
fn tops_near(r: &RavineLayout, at: V3) -> Vec<(i32, String)> {
    let a = at.flat();
    let mut out = Vec::new();
    for (k, w) in r.walls.iter().enumerate() {
        for seg in w.points.windows(2) {
            let (p, q) = (seg[0].at.flat(), seg[1].at.flat());
            let d = q.sub(p);
            let l2 = d.dot64(d).max(1);
            let t =
                (a.sub(p).dot64(d) as i128 * ONE as i128 / l2 as i128).clamp(0, ONE as i128) as i32;
            if p.lerp(q, t).dist(a) < int(5) {
                let top = (seg[0].at.y + seg[0].height).max(seg[1].at.y + seg[1].height);
                out.push((top, format!("wall {k}")));
            }
        }
    }
    for (k, rock) in r.rocks.iter().enumerate() {
        if rock.at.flat().dist(a) < rock.radius + int(5) {
            out.push((rock.at.y + rock.height, format!("rock {k}")));
        }
    }
    for (k, d) in r.decks.iter().enumerate() {
        let probe = Deck {
            half_width: d.half_width + int(5),
            ..*d
        };
        if let Some(top) = probe.top_at(at) {
            out.push((top, format!("deck {k}")));
        }
    }
    for (k, m) in r.rooms.iter().enumerate() {
        let wide = Room {
            half_width: m.half_width + int(5),
            height: int(400),
            ..*m
        };
        if wide.contains(v3(at.x, m.from.y.min(m.to.y), at.z)) {
            out.push((m.from.y.max(m.to.y) + m.height, format!("room {k}")));
        }
    }
    out
}

#[test]
fn every_vine_hangs_under_its_top_or_clear_of_it_by_4_m() {
    for c in layouts() {
        let r = lay(&c);
        let mut bad = Vec::new();
        for (i, v) in c.vines.iter().enumerate() {
            let tip = v.anchor.y - v.length;
            for (top, what) in tops_near(r, v.anchor) {
                // hung from beneath it (a deck's, a roof's underside), or
                // its bottom 4 m over it
                if v.anchor.y > top && tip < top + int(4) {
                    bad.push(format!(
                        "vine {i} at {:?}: {what} top {}",
                        v.anchor,
                        top / ONE
                    ));
                }
            }
        }
        assert!(bad.is_empty(), "{}", bad.join("\n"));
    }
}

#[test]
fn every_hanger_hangs_from_what_it_says() {
    for c in layouts() {
        let r = lay(&c);
        assert_eq!(r.hangers.len(), c.vines.len());
        for (i, h) in r.hangers.iter().enumerate() {
            assert_eq!(h.at, c.vines[i].anchor, "vine {i}");
            match h.from {
                HangFrom::Deck { deck } => {
                    let top = r.decks[deck].top_at(h.at).expect("over its deck");
                    assert!(
                        (0..int(1)).contains(&(top - h.at.y)),
                        "vine {i} under its deck"
                    );
                }
                HangFrom::Roof { room } => {
                    let m = r.rooms[room];
                    assert!(
                        m.contains(h.at.sub(v3(0, cm(10), 0))),
                        "vine {i} in its room"
                    );
                    assert!(m.kind != RoomKind::Arch);
                }
                HangFrom::Ledge { wall } => assert!(wall < r.walls.len()),
            }
        }
        // there are bridges and logs, caves, arches and the tunnel
        for kind in [DeckKind::Bridge, DeckKind::Log] {
            assert!(r.decks.iter().filter(|d| d.kind == kind).count() >= 10);
        }
        for kind in [RoomKind::Arch, RoomKind::Cave, RoomKind::Tunnel] {
            assert!(r.rooms.iter().any(|m| m.kind == kind), "{kind:?}");
        }
    }
}

#[test]
fn nothing_solid_stands_within_16_m_behind_the_start() {
    let c = ravine();
    let r = lay(&c);
    let (p0, fwd) = (c.path[0], c.heading[0]);
    let mut solids: Vec<V3> = r
        .walls
        .iter()
        .flat_map(|w| w.points.iter().map(|p| p.at))
        .collect();
    solids.extend(r.rocks.iter().map(|k| k.at));
    solids.extend(r.tracks.iter().flat_map(|t| [t.start.at, t.end.at]));
    for m in &r.rooms {
        solids.extend([m.from, m.to]);
    }
    for s in solids {
        let d = s.sub(p0).flat();
        if d.dot(fwd) < 0 {
            assert!(
                d.len() >= int(16),
                "a solid at {s:?}, {:?} m from the start",
                d.len() / ONE
            );
        }
    }
    // and the canyon is closed behind it, past the camera
    let back = r
        .walls
        .iter()
        .flat_map(|w| &w.points)
        .map(|p| p.at.sub(p0).flat().dot(fwd))
        .min();
    assert_eq!(back, Some(-START_BACK));
}

#[test]
fn the_canyon_is_as_long_as_the_other_maps_and_never_crosses_itself() {
    let c = ravine();
    assert!(
        (int(1500)..=int(1800)).contains(&c.length),
        "{:?}",
        c.length / ONE
    );
    let n = c.path.len();
    for i in (0..n).step_by(3) {
        for j in (i + 60..n).step_by(3) {
            let d = c.path[i].flat().dist(c.path[j].flat());
            assert!(
                d >= 2 * HALF + int(10),
                "{i} and {j}: {:?} m apart",
                d / ONE
            );
        }
    }
}

#[test]
fn the_mine_tracks_run_from_holes_in_the_walls_and_carry_their_carts() {
    let c = ravine();
    let r = lay(&c);
    assert!(r.tracks.len() >= 3 && r.carts.len() >= r.tracks.len());
    let mut crossing = 0;
    for t in &r.tracks {
        let side = |p: V3| c.progress(p, c.nearest_anywhere(p)).1;
        for h in [t.start, t.end] {
            assert!(
                (side(h.at).abs() - HALF).abs() < int(1),
                "a hole in a wall: {:?}",
                h.at
            );
            // out of the wall, into the canyon
            assert!(side(h.at.add(h.out.scale(int(2)))).abs() < side(h.at).abs());
        }
        crossing += (side(t.start.at).signum() != side(t.end.at).signum()) as i32;
        let heights: Vec<i32> = t.points.iter().map(|p| p.y).collect();
        assert!(heights.iter().max() > heights.iter().min() || crossing > 0);
    }
    assert!(crossing >= 2, "tracks crossing the canyon: {crossing}");
    for (i, cart) in r.carts.iter().enumerate() {
        let at = r.cart_at(i, 0).0;
        let cap = c.mushrooms[cart.bouncer].top;
        assert!(
            cap.flat().dist(at.flat()) < cm(5) && cap.y > at.y,
            "cart {i}"
        );
    }
    // lizards on both walls, of all three colours, bouncers on their backs
    assert!(r.lizards.len() >= 6);
    for colour in [LizardColour::Red, LizardColour::Yellow, LizardColour::Blue] {
        assert!(r.lizards.iter().any(|l| l.colour == colour));
    }
    for (i, l) in r.lizards.iter().enumerate() {
        assert!(
            c.mushrooms[l.bouncer]
                .top
                .flat()
                .dist(r.lizard_at(i, 0).0.flat())
                < int(2)
        );
    }
}

#[test]
fn the_glows_lead_to_the_mesa_finish() {
    let c = ravine();
    let r = lay(&c);
    let along = |p: V3| c.progress(p, c.nearest_anywhere(p)).0;
    let first = r
        .glows
        .iter()
        .filter(|g| along(g.at) < c.length / 3)
        .count();
    let last = r
        .glows
        .iter()
        .filter(|g| along(g.at) > c.length * 2 / 3)
        .count();
    assert!(
        2 * last > 3 * first,
        "glows: {first} in the first third, {last} in the last"
    );
    for kind in [
        GlowKind::Torch,
        GlowKind::Fireflies,
        GlowKind::Mushroom,
        GlowKind::Lantern,
    ] {
        assert!(r.glows.iter().any(|g| g.kind == kind), "{kind:?}");
    }
    let f = &r.finish;
    let end = c.point(c.length);
    let past = f.sphinx.sub(end).flat().dot(c.heading_at(c.length));
    assert!(
        (int(15)..int(40)).contains(&past),
        "the sphinx just past the line"
    );
    assert!(f.sticks.len() >= 3 && f.tents.len() >= 3);
    assert!(f.campfire.flat().dist(f.sphinx.flat()) > int(10));
}

/// The hairpin's fin has a round tip: its inner face (the left wall's,
/// both corners) bends round at least 8 m from the corners' middle, so a
/// runner on the ground can get round (SECOND-ORDER RC7: at a 0 m inner
/// radius King Kong ran into the fin's tip for ever).
#[test]
fn the_hairpins_fin_has_a_round_tip() {
    let c = ravine();
    let p = plan(false);
    let face = |s: i32| {
        c.point(int(s))
            .add(right_of(c.heading_at(int(s))).scale(-HALF))
            .flat()
    };
    // 6 m along the spine at the corners' R moves the inner face
    // 6 r / R: r ≥ 8 m with R = r + HALF
    let least = int(6) * 8 / (8 + HALF / ONE);
    for s in (p.marks[leg::HAIRPIN]..p.marks[leg::OUT_LEG] - 6).step_by(2) {
        let step = face(s).dist(face(s + 6));
        assert!(
            step >= least,
            "at {s} m the fin's face moves {:?} cm in 6 m",
            step * 100 / ONE
        );
    }
}

/// The upper passage's walls are clear of its middle all the way (they
/// stood inside it in the swing, where the spine heads off at 45°, and
/// the AI ran into them, SECOND-ORDER RC7).
#[test]
fn the_upper_passages_walls_are_clear_of_its_middle() {
    for c in layouts() {
        let r = lay(&c);
        let sp = r.split.as_ref().unwrap();
        let solids = super::solids::Solids::of(r);
        for (k, p) in sp.upper.iter().enumerate() {
            let at = p.add(v3(0, int(3), 0));
            let near = solids
                .list
                .iter()
                .filter(|s| s.kind == super::solids::SolidKind::Wall)
                .any(|s| s.holds(at, int(12)));
            assert!(!near, "upper point {k} at {at:?}: a wall within 12 m");
        }
    }
}

/// Every snake on the spine has a low vine within 12 m past it on its
/// side, where an ape it drops lands (SECOND-ORDER RC7).
#[test]
fn a_low_vine_hangs_past_every_snake_on_its_side() {
    for c in layouts() {
        let r = lay(&c);
        let place = |i: usize| {
            let a = c.vines[i].anchor;
            let (s, side) = c.progress(a, c.nearest_anywhere(a));
            (s, side)
        };
        for i in 0..c.vines.len() {
            let (s, side) = place(i);
            if !r.hangers[i].snake || side.abs() > HALF {
                continue;
            }
            let found = (0..c.vines.len()).any(|j| {
                let (t, o) = place(j);
                low(&c, j) && t >= s && t <= s + int(12) && o.signum() == side.signum()
            });
            assert!(
                found,
                "snake {i} at {} m, side {}: no low vine past it",
                s / ONE,
                side / ONE
            );
        }
    }
}

/// The last stretch climbs out of the slot onto the open mesa (SECOND-ORDER
/// RA-8 item 1): the walls stand 50 m or more round the last station, fall
/// to low rims past the finish (still over 1.5 jumps' rise, so they
/// close the course: no invisible wall), and stand 70 m or more off the
/// line either side 60 m past it.
#[test]
fn the_mesa_opens_onto_low_rims_past_the_finish() {
    use super::layout::{MESA_RIM, RIM_FALL};
    use crate::rules::{GRAVITY, JUMP_SPEED};
    let rise = fx::mul(JUMP_SPEED, JUMP_SPEED) / (2 * GRAVITY / ONE);
    for c in layouts() {
        let r = lay(&c);
        let end = c.point(c.length);
        let h = c.heading_at(c.length);
        let (mut past, mut tall, mut wide) = (0, 0, 0);
        for w in &r.walls {
            for p in &w.points {
                let d = p.at.sub(end).flat();
                let (along, across) = (d.dot(h), d.dot(right_of(h)).abs());
                if d.len() > int(200) {
                    continue; // the canyon's earlier stretches
                }
                if along > int(RIM_FALL + 6) {
                    past += 1;
                    assert!(p.height <= MESA_RIM + int(1), "{} m past", along / ONE);
                    assert!(p.height > rise * 3 / 2, "{} m past", along / ONE);
                }
                if along < -int(24) && along > -int(60) && across < int(30) {
                    tall += 1;
                    assert!(p.height >= int(50), "{} m short", -along / ONE);
                }
                if (int(56)..int(64)).contains(&along) && w.points.len() > 20 {
                    wide += 1;
                    assert!(across >= int(70), "{} m across", across / ONE);
                }
            }
        }
        assert!(past > 30 && tall >= 6 && wide >= 2, "{past} {tall} {wide}");
    }
}

/// The campfire's roasting sticks stand at the layout's four spots
/// (`Finish.sticks`, SECOND-ORDER RA-8 item 3), round the fire where the
/// model's own stand at its drawn size, and the fire and tents stand clear
/// of the sphinx and of one another.
#[test]
fn the_campfires_four_sticks_ring_the_fire() {
    use super::layout::CAMPFIRE_SCALE;
    let c = ravine();
    let f = &lay(&c).finish;
    let out = fx::mul(cm(170), CAMPFIRE_SCALE);
    assert_eq!(f.sticks.len(), 4);
    for s in &f.sticks {
        let d = s.sub(f.campfire).flat().len();
        assert!((d - out).abs() < cm(10), "a stick {} cm out", d * 100 / ONE);
    }
    assert!(f.campfire.flat().dist(f.sphinx.flat()) > int(20));
    for (k, &(t, _)) in f.tents.iter().enumerate() {
        assert!(t.flat().dist(f.campfire.flat()) > int(14), "tent {k}");
        for &(u, _) in &f.tents[k + 1..] {
            assert!(t.flat().dist(u.flat()) > int(12), "tent {k}");
        }
    }
}

/// The cheap rejects (RP1) never miss what the full test finds: on the
/// real course a body standing on each lizard and each cart's moved top,
/// at ticks right round their periods, is on that bouncer, and a body on
/// each deck's middle and near its ends gets that deck's top.
#[test]
fn every_mover_and_deck_still_holds_a_body_on_it_past_the_quick_rejects() {
    let c = ravine();
    let r = c.ravine.as_deref().unwrap();
    let movers = r
        .lizards
        .iter()
        .map(|l| (l.bouncer, l.period))
        .chain(r.carts.iter().map(|k| (k.bouncer, k.period)));
    for (m, period) in movers {
        for k in 0..16 {
            let tick = period.max(1) * k / 16;
            let top = c.ravine_bouncer(r, m, tick).top;
            let p = top.add(v3(0, BODY_DROP - cm(5), 0));
            assert_eq!(
                c.ravine_bouncer_under(r, p, tick),
                Some(m),
                "bouncer {m} at tick {tick}"
            );
        }
    }
    for (n, d) in r.decks.iter().enumerate() {
        for f in [ONE / 50, ONE / 2, ONE - ONE / 50] {
            let at = d.from.lerp(d.to, f);
            assert!(d.top_at(at).is_some(), "deck {n} at {f}");
        }
    }
}
