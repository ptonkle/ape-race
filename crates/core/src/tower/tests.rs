//! Mape's Tower's skeleton (the second design): the map is wired, the 3D
//! centreline leg by leg, along-the-course on a path that climbs then
//! falls (TW1), the reset onto a pad with its boost (per leg), the climb
//! vines' lift, the tube's pads and air (TW2-5).

use super::*;
use crate::ape::{Ape, Controls, Mode};
use crate::course::STEP;
use crate::fx::{self, ONE};
use crate::geom::V3;
use crate::map::Map;
use crate::race::{Driver, Entry, Event, Phase, Race};
use crate::rules::{Kind, BODY_DROP, TICKS_PER_SECOND};

fn entries(n: usize) -> Vec<Entry> {
    (0..n)
        .map(|i| Entry {
            name: format!("P{i}"),
            kind: Kind::Chimpanzee,
            driver: Driver::Local,
            palette: 0,
            costume: crate::costume::Costume::Own,
        })
        .collect()
}

#[test]
fn the_map_is_wired_by_its_key_and_name() {
    assert_eq!(Map::from_key("tower"), Some(Map::Tower));
    assert_eq!(Map::Tower.key(), "tower");
    assert!(Map::ALL.contains(&Map::Tower));
    let c = Map::Tower.course();
    assert_eq!(c.name, "tower");
    assert_eq!(Map::of(&c), Map::Tower);
    assert!(c.tower.is_some());
    let strings: serde_json::Value =
        serde_json::from_str(include_str!("../../../../data/strings.json")).unwrap();
    assert_eq!(strings["map.tower"], "Mape's Tower");
    // the other maps have no tower layout
    for m in Map::ALL.into_iter().filter(|&m| m != Map::Tower) {
        assert!(m.course().tower.is_none(), "{m:?}");
    }
}

/// The legs in order along the course, each where the plan puts it: level
/// under the bridges, climbing round tower 2 and inside tower 1, out
/// through the tear over tower 1's rim, straight down the tube, level
/// along the tunnel to the finish.
#[test]
fn the_centreline_runs_the_legs_in_order_and_climbs_then_falls() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let n = c.path.len();
    // every step is STEP long (to a centimetre) and its heading points to
    // the next
    for i in 0..n - 1 {
        let d = c.path[i + 1].sub(c.path[i]);
        assert!((d.len() - STEP).abs() < cm(1), "step {i}: {}", d.len());
        assert!(d.norm().sub(c.heading[i]).len() < cm(1), "heading {i}");
    }
    let order: Vec<Leg> = t.legs.iter().map(|l| l.leg).collect();
    assert_eq!(
        order,
        vec![
            Leg::Start,
            Leg::LowerBridge,
            Leg::Tower2,
            Leg::UpperBridge,
            Leg::Inside,
            Leg::Portal,
            Leg::Tube,
            Leg::Sling,
            Leg::Tunnel
        ]
    );
    let at = |leg: Leg| t.leg_start(leg, c.length);
    let mid = |leg: Leg, next: Leg| c.point((at(leg) + at(next)) / 2);
    let [t1, t2] = &t.towers;
    // under the bridges: the deck over the line, the line level
    for (leg, next, b) in [
        (Leg::LowerBridge, Leg::Tower2, &t.bridges[0]),
        (Leg::UpperBridge, Leg::Inside, &t.bridges[1]),
    ] {
        let p = mid(leg, next);
        let deck = c.platforms[b.deck.unwrap()];
        assert!(
            c.support(v3(p.x, deck.top + BODY_DROP, p.z)) == deck.top,
            "{leg:?}: a deck over the line"
        );
        assert!(
            deck.top - p.y > int(20) && deck.top - p.y < int(38),
            "{leg:?}: the deck {} m over the line",
            (deck.top - p.y) / ONE
        );
        let k = (at(leg) / STEP) as usize + 5;
        assert!(c.heading[k].y.abs() < ONE / 20, "{leg:?} is level");
    }
    // round tower 2, outside its wall, rising
    let p = mid(Leg::Tower2, Leg::UpperBridge);
    let off = p.flat().sub(t2.centre.flat()).len() - t2.radius;
    assert!(
        off > int(8) && off < int(20),
        "round tower 2: {} m off its wall",
        off / ONE
    );
    assert!(c.point(at(Leg::UpperBridge)).y - c.point(at(Leg::Tower2)).y > int(80));
    // the doorway on tower 1's wall at the upper bridge's end, half way up
    let d = t.doorway;
    let r = d.at.flat().sub(t1.centre.flat()).len();
    assert!(
        (r - t1.radius).abs() < int(2),
        "the doorway {} m from the axis (wall {})",
        r / ONE,
        t1.radius / ONE
    );
    let half = fx::div(d.at.y - t1.centre.y, t1.height);
    assert!(
        half > ONE * 4 / 10 && half < ONE * 6 / 10,
        "the doorway {}% up",
        half * 100 / ONE
    );
    // inside tower 1 up to the rim, then out over it
    let p = mid(Leg::Inside, Leg::Portal);
    assert!(
        !p.flat()
            .sub(t1.centre.flat())
            .longer_than(t1.inner - int(5)),
        "inside the hollow"
    );
    let out = c.point(at(Leg::Tube));
    assert!(
        out.flat()
            .sub(t1.centre.flat())
            .longer_than(t1.radius + int(10)),
        "out past the wall"
    );
    assert!(out.y > t1.top(), "over the rim");
    assert!(t.in_void(out) && !t.in_void(p));
    // straight down the tube to its bottom, inside its wall
    let k = (at(Leg::Tube) / STEP) as usize + 10;
    assert_eq!(c.heading[k], v3(0, -ONE, 0), "straight down");
    assert!(
        c.right[k].flat().longer_than(ONE / 2),
        "a level right down the tube"
    );
    let p = mid(Leg::Tube, Leg::Sling);
    assert!(t.tube.holds(p));
    assert!(
        c.point(at(Leg::Tube)).y - c.point(at(Leg::Sling)).y > int(250),
        "a long fall"
    );
    // the tunnel: level, the finish part way along, the floor under it
    let k = (at(Leg::Tunnel) / STEP) as usize + 5;
    assert_eq!(c.heading[k].y, 0);
    let end = c.point(c.length);
    assert!(t.tunnel.finish > int(40) && t.tunnel.finish < t.tunnel.length);
    let floor = c.platforms[t.tunnel.floor];
    assert!(
        c.support(v3(end.x, floor.top + BODY_DROP, end.z)) == floor.top,
        "the tunnel's floor"
    );
    // the top of the course is the tube's top; nothing is below the ground
    let top = c.path.iter().map(|p| p.y).max().unwrap();
    assert_eq!(top, t.tube.top);
    assert!(c.path.iter().all(|p| p.y > int(10)));
}

/// Inside tower 1 progress is height (TW2-9): a point anywhere across
/// the hollow at a sample's height is found at that sample, and one 3 m
/// higher 3 m further on; round tower 2 the spiral's turn is told from
/// the corner under it by height.
#[test]
fn along_the_course_tells_the_climbs_apart_by_height() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let t1 = &t.towers[0];
    let from = (t.leg_start(Leg::Inside, c.length) / STEP) as usize + 80;
    let to = (t.leg_start(Leg::Portal, c.length) / STEP) as usize - 10;
    for k in [from, (from + to) / 2, to] {
        // (the sample is on the axis)
        assert!(
            c.path[k].flat().sub(t1.centre.flat()).len() < int(1),
            "sample {k} on the axis"
        );
        for off in [-int(30), int(20)] {
            let p = c.path[k].add(v3(off, int(3), off / 2));
            let i = c.nearest_anywhere(p);
            assert!(
                (i as i32 - k as i32).abs() <= 2,
                "sample {k} off {}: nearest {i}",
                off / ONE
            );
            let (s, _) = c.progress(p, i);
            assert!(
                (s - (k as i32 * STEP + int(3))).abs() < int(1),
                "sample {k}: {} m",
                s / ONE
            );
            assert!((c.nearest(p, k) as i32 - k as i32).abs() <= 2);
        }
    }
    // the spiral round tower 2: a point over its last quarter, right
    // above its first, is found on its own turn
    let s0 = t.leg_start(Leg::Tower2, c.length) + int(80);
    let p0 = c.point(s0);
    let k = ((s0 / STEP) as usize..(t.leg_start(Leg::UpperBridge, c.length) / STEP) as usize)
        .skip(60)
        .min_by_key(|&k| c.path[k].flat().sub(p0.flat()).len())
        .unwrap();
    let p = v3(p0.x, c.path[k].y + int(3), p0.z);
    let i = c.nearest_anywhere(p);
    assert!(
        (i as i32 - k as i32).abs() <= 3,
        "spiral sample {k}: nearest {i}"
    );
}

#[test]
fn out_of_bounds_puts_the_ape_on_the_pad_behind_flying_with_its_boost() {
    let c = tower();
    let pads = c.tower.as_deref().unwrap().pads.clone();
    let mut r = Race::new(c, &entries(2), false, 1);
    r.phase = Phase::Running;
    // 130 m along (under the lower bridge), far under the centreline
    let s = int(130);
    let p = r.course.point(s);
    let a = &mut r.racers[0].ape;
    a.pos = p.sub(v3(0, OUT_BELOW + int(2), 0));
    a.hint = r.course.nearest_anywhere(p);
    a.vel = v3(0, -int(10), 0);
    a.mode = Mode::Flying;
    r.step(&[Default::default(); 2]);
    // the last pad behind (not the next one ahead)
    let pad = pads.iter().rev().find(|p| p.at <= s).copied().unwrap();
    assert!(pad.at > 0 && pads.iter().any(|p| p.at > s));
    assert!(r.events.contains(&Event::Reset {
        who: 0,
        pad: pad.platform
    }));
    let pl = r.course.platforms[pad.platform];
    let a = &r.racers[0].ape;
    assert_eq!(a.mode, Mode::Flying);
    assert!(a.vel.y > 0, "boosted up");
    assert!(a.vel.flat().longer_than(int(5)), "and on");
    assert!(a.pos.flat().sub(pl.centre.flat()).len() < int(1));
    assert!((a.pos.y - (pl.top + BODY_DROP)).abs() < int(1));
    assert!(r.racers[0].eaten.is_none());
    // the other ape, still on the start deck, is in bounds
    assert!(!r
        .events
        .iter()
        .any(|e| matches!(e, Event::Reset { who: 1, .. })));
}

/// Inside tower 1 a fall to its floor is a reset onto a ledge on the
/// inner wall; a reset on the sling (out under the tunnel's mouth) goes
/// to the portal's pad, which throws the ape on into the tube.
#[test]
fn resets_inside_go_to_a_ledge_and_resets_on_the_sling_go_to_the_portal() {
    let c = tower();
    let t = c.tower.as_deref().unwrap().clone();
    let t1 = &t.towers[0];
    let mut r = Race::new(c, &entries(1), false, 1);
    r.phase = Phase::Running;
    // down on tower 1's floor, in the middle, off a beam 100 m up the
    // shaft (progress inside is height, so the fall has taken its
    // progress to the leg's start: the pad is by the beam it held, TW2-9)
    let shaft = t.leg_start(Leg::Inside, r.course.length) + int(100);
    let held = (0..r.course.vines.len())
        .filter(|&i| matches!(t.hangers[i], Hanger::Window { .. }))
        .min_by_key(|&i| (r.course.vines[i].at - shaft).abs())
        .unwrap();
    let p = v3(t1.centre.x, BODY_DROP, t1.centre.z);
    let a = &mut r.racers[0].ape;
    a.pos = p;
    // (its hint followed it down the shaft)
    a.hint = r.course.nearest_anywhere(
        r.course
            .point(t.leg_start(Leg::Inside, r.course.length) + int(60)),
    );
    a.mode = Mode::Grounded;
    a.last_vine = Some((held, 30));
    r.step(&[Default::default()]);
    let Some(Event::Reset { pad, .. }) = r.events.iter().find(|e| matches!(e, Event::Reset { .. }))
    else {
        panic!("no reset off tower 1's floor");
    };
    let pl = r.course.platforms[*pad];
    let off = pl.centre.flat().sub(t1.centre.flat()).len();
    let chosen = t.pads.iter().find(|p| p.platform == *pad).unwrap();
    assert!(
        off > t1.inner - int(8) && off < t1.inner,
        "a ledge on the inner wall ({} m from the axis; pad at {} m, {:?}, progress {} m)",
        off / ONE,
        chosen.at / ONE,
        t.leg_at(chosen.at),
        r.racers[0].progress / ONE
    );
    assert!(r.racers[0].ape.vel.y > 0, "boosted up");
    assert!(
        chosen.at > r.course.vines[held].at - int(45) && chosen.at < r.course.vines[held].at,
        "the ledge under the beam it held ({} m, beam at {} m)",
        chosen.at / ONE,
        r.course.vines[held].at / ONE
    );
    // out under the tunnel's mouth (on the sling leg, far under its line)
    // (on the open ground under the sling leg's line, under the tube's
    // floor)
    let sling = t.leg_start(Leg::Sling, r.course.length) + int(4);
    let q = r.course.point(sling);
    let q = v3(q.x, BODY_DROP, q.z);
    let a = &mut r.racers[0].ape;
    a.pos = q;
    a.hint = r.course.nearest_anywhere(r.course.point(sling));
    a.mode = Mode::Flying;
    a.vel = V3::ZERO;
    r.events.clear();
    assert!(
        r.course.tower_out(q, r.racers[0].ape.hint),
        "under the tube is out"
    );
    r.step(&[Default::default()]);
    let portal = t.pads[t.portal_pad];
    assert!(
        r.events.contains(&Event::Reset {
            who: 0,
            pad: portal.platform
        }),
        "the sling's reset goes to the portal's pad: {:?}",
        r.events
    );
    let a = &r.racers[0].ape;
    assert!(
        a.vel.dot(t.portal.through) > int(3),
        "thrown on through the tear"
    );
}

#[test]
fn with_floor_is_lava_out_of_bounds_is_the_kill() {
    let mut r = Race::new(tower(), &entries(1), false, 1);
    r.phase = Phase::Running;
    r.lava = true;
    let p = r.course.point(int(130));
    let a = &mut r.racers[0].ape;
    a.pos = p.sub(v3(0, OUT_BELOW + int(2), 0));
    a.hint = r.course.nearest_anywhere(p);
    a.mode = Mode::Flying;
    r.step(&[Default::default()]);
    assert!(r.racers[0].eaten.is_some());
}

#[test]
fn a_climb_vine_lifts_a_rising_release_and_a_plain_one_does_not() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let climb = (0..c.vines.len()).find(|&i| t.lifts(i)).unwrap();
    let plain = (0..c.vines.len()).find(|&i| !t.lifts(i)).unwrap();
    let rising = |vine: usize| {
        let mut a = Ape::new(Kind::Chimpanzee, false, c.vines[vine].anchor, 0);
        a.vel = v3(0, int(4), int(8));
        a.mode = Mode::Swinging {
            vine,
            rope: int(10),
        };
        a.release(vine);
        let before = a.vel;
        a.climb_lift(vine, &c);
        a.vel.y - before.y
    };
    let lifted = rising(climb);
    let not = rising(plain);
    assert_eq!(lifted, CLIMB_LIFT);
    assert_eq!(not, 0);
}

#[test]
fn the_path_is_three_d_only_here() {
    // every other map keeps its centreline on the ground and level
    for m in Map::ALL.into_iter().filter(|&m| m != Map::Tower) {
        let c = m.course();
        assert!(c.path.iter().all(|p| p.y == 0), "{m:?}");
        assert!(c.heading.iter().all(|h| h.y == 0), "{m:?}");
    }
    let _ = V3::ZERO;
}

/// A vine summoned high over the lower bridge is placed along the course
/// by its own leg, not the upper bridge's line above (TC5).
#[test]
fn a_summoned_vine_is_placed_along_its_own_leg() {
    use crate::race::Summon;
    let mut r = Race::new(tower(), &entries(1), true, 7);
    let at = 100 * ONE;
    let p = r.course.point(at);
    let slot = r.summon_slot(0, 0);
    r.place_vine(Summon {
        slot,
        by: 0,
        serial: 0,
        anchor: p.add(v3(0, 36 * ONE, 0)),
        length: 34 * ONE,
        life: crate::rules::SUMMON_LIFE,
    });
    let placed = r.course.vines[slot].at;
    assert!(
        (placed - at).abs() < 20 * ONE,
        "placed at {} m",
        placed / ONE
    );
}

#[test]
fn item_crates_hang_on_whole_stations_and_never_on_the_start_row() {
    for seed in 0..4 {
        let c = tower_seeded(seed);
        let t = c.tower.as_deref().unwrap();
        let carry = crate::items::crate_vines(&c);
        assert!(carry.len() >= 6, "seed {seed}: {} crates", carry.len());
        for &v in &carry {
            assert!(
                t.hangers[v] != Hanger::Start && t.hangers[v] != Hanger::Sling,
                "seed {seed}: start or sling vine {v} has a crate"
            );
            // every beam of its station carries one too
            for (i, w) in c.vines.iter().enumerate() {
                if t.hangers[i] != Hanger::Start
                    && t.hangers[i] != Hanger::Sling
                    && (w.at - c.vines[v].at).abs() <= int(4)
                {
                    assert!(
                        carry.contains(&i),
                        "seed {seed}: vine {i} beside crate vine {v}"
                    );
                }
            }
        }
    }
}

/// Every vine inside tower 1 hangs inside the hollow (its anchor and its
/// bottom clear of the wall), and each is within a swing of others: at
/// least two other inside beams' bottoms within a beam's length and a
/// layer's rise of its anchor (the person: "random vines and the player
/// is expected to find their own way up").
#[test]
fn inside_beams_fill_the_hollow_each_within_a_swing_of_two_others() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let t1 = &t.towers[0];
    let inside: Vec<usize> = (0..c.vines.len())
        .filter(|&i| {
            matches!(
                t.hangers[i],
                Hanger::Window { .. } | Hanger::Floating { .. } | Hanger::Mage { balcony: _ }
            ) && t.leg_at(c.vines[i].at) == Leg::Inside
        })
        .collect();
    assert!(inside.len() >= 40, "{} beams inside", inside.len());
    let bottom = |i: usize| c.vines[i].anchor.add(t.hangs[i].scale(c.vines[i].length));
    for &i in &inside {
        let b = bottom(i);
        assert!(
            !b.flat()
                .sub(t1.centre.flat())
                .longer_than(t1.inner - int(4)),
            "vine {i}'s bottom is {} m from the axis",
            b.flat().sub(t1.centre.flat()).len() / ONE
        );
        let near = inside
            .iter()
            .filter(|&&j| {
                j != i && {
                    let d = bottom(j).sub(c.vines[i].anchor);
                    let reach = c.vines[i].length * 3 / 2;
                    !d.flat().longer_than(reach) && d.y.abs() <= reach
                }
            })
            .count();
        assert!(near >= 2, "vine {i} has {near} beams within a swing");
    }
}

/// A fall onto a pad's covered part bounces the ape back up at the
/// mushroom's speed; a fall through its gap does not (TW2-5).
#[test]
fn a_tube_pad_bounces_a_fall_on_its_cover_and_lets_one_through_its_gap() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let tube = &t.tube;
    let drop = |rel: V3| {
        let k = 3;
        let pad = tube.pads[k];
        let start = tube.axis.add(rel).add(v3(0, pad.y + BODY_DROP + int(6), 0));
        let mut a = Ape::new(Kind::Chimpanzee, true, start, 0);
        a.mode = Mode::Flying;
        a.vel = v3(0, -int(30), 0);
        a.hint = c.nearest_anywhere(start);
        let mut bounced = false;
        for tick in 0..TICKS_PER_SECOND as u32 / 2 {
            a.step(&Controls::default(), &c, tick);
            bounced |= a.happened.contains(&crate::ape::Happening::Bounce);
        }
        (bounced, a.pos.y - BODY_DROP < pad.y, a.vel.y)
    };
    let gap = tube.pads[3].gap;
    let (bounced, through, _) = drop(gap);
    assert!(!bounced && through, "the gap lets the ape through");
    // the covered side: across from the gap
    let side = v3(0, 0, if gap.z > 0 { -int(12) } else { int(12) });
    assert!(tube.covered(3, side));
    let (bounced, through, vy) = drop(side);
    assert!(bounced && !through, "the cover bounces it");
    assert!(vy > 0 && vy <= crate::rules::MUSHROOM_BOUNCE);
}

/// In the tube the fall settles at `TUBE_TERMINAL` and the steering is
/// `TUBE_AIR` times the kind's; outside it (over the start) neither
/// applies.
#[test]
fn the_tubes_air_is_thick_and_its_steering_strong() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let tube = &t.tube;
    let fall = |start: V3, secs: i32| {
        let mut a = Ape::new(Kind::Chimpanzee, true, start, 0);
        a.mode = Mode::Flying;
        a.hint = c.nearest_anywhere(start);
        let mut fastest = 0;
        for tick in 0..(secs * TICKS_PER_SECOND) as u32 {
            a.step(&Controls::default(), &c, tick);
            fastest = fastest.max(-a.vel.y);
        }
        fastest
    };
    let top = v3(tube.axis.x, tube.top - int(2), tube.axis.z);
    let fastest = fall(top, 5);
    assert!(
        (fastest - TUBE_TERMINAL).abs() < int(2),
        "the fall settles at {} m/s (terminal {})",
        fastest / ONE,
        TUBE_TERMINAL / ONE
    );
    let open = v3(c.starts[0].x, c.starts[0].y + int(200), c.starts[0].z);
    let a = Ape::new(Kind::Chimpanzee, true, open, 0);
    assert_eq!(a.air_now(&c), a.phys.air, "the kind's steering in open air");
    let mut b = Ape::new(Kind::Chimpanzee, true, top, 0);
    assert_eq!(b.air_now(&c), fx::mul(a.phys.air, TUBE_AIR));
    b.vel = v3(0, -int(60), 0);
    b.tube_air(&c);
    assert!(
        b.vel.y > -int(60),
        "thick air slows a fall faster than terminal"
    );
}

/// The measurement (TW2-5, `MEASUREMENTS.md`): a human-like weave (steer
/// for the next gap from the moment the last pad is passed: the AI's
/// `tube_controls`) falls the whole tube touching no pad, from the top
/// and from a start off the axis, and reports the terminal speed, the
/// fall's time and the worst miss of a gap's middle.
#[test]
fn a_weave_steered_for_each_gap_falls_the_tube_touching_no_pad() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let tube = &t.tube;
    for (name, start) in [
        ("the axis", v3(tube.axis.x, tube.top - int(1), tube.axis.z)),
        (
            "8 m off",
            v3(tube.axis.x, tube.top - int(1), tube.axis.z + int(8)),
        ),
        (
            "the portal's throw",
            c.point(t.leg_start(Leg::Portal, c.length) + int(70))
                .add(v3(0, int(6), 0)),
        ),
    ] {
        let mut a = Ape::new(Kind::Chimpanzee, true, start, 0);
        a.mode = Mode::Flying;
        a.hint = c.nearest_anywhere(start);
        if name == "the portal's throw" {
            a.vel = t.portal.through.scale(int(18));
        }
        let (mut fastest, mut bounces, mut worst, mut ticks) = (0, 0, 0, 0);
        let mut passed = 0;
        for tick in 0..(12 * TICKS_PER_SECOND) as u32 {
            let ctl = t.tube_controls(&a).unwrap_or_default();
            let before = a.pos;
            a.step(&ctl, &c, tick);
            ticks = tick;
            fastest = fastest.max(-a.vel.y);
            if a.happened.contains(&crate::ape::Happening::Bounce) {
                bounces += 1;
            }
            // the miss at each pad crossed
            for (k, pad) in tube.pads.iter().enumerate() {
                if before.y - BODY_DROP >= pad.y && a.pos.y - BODY_DROP < pad.y {
                    let rel = v3(a.pos.x - tube.axis.x, 0, a.pos.z - tube.axis.z);
                    worst = worst.max(rel.sub(pad.gap).len());
                    passed = passed.max(k + 1);
                }
            }
            if a.pos.y - BODY_DROP < tube.bottom + int(30) {
                break;
            }
        }
        println!(
            "weave from {name}: terminal {:.1} m/s, {:.1} s to the sling's height, {} pads passed, worst miss {:.1} m, {bounces} bounces",
            fastest as f64 / ONE as f64,
            ticks as f64 / TICKS_PER_SECOND as f64,
            passed,
            worst as f64 / ONE as f64
        );
        assert_eq!(bounces, 0, "from {name}: {bounces} bounces");
        assert_eq!(passed, tube.pads.len(), "from {name}: {passed} pads passed");
        assert!(
            ticks < (10 * TICKS_PER_SECOND) as u32,
            "from {name}: too slow"
        );
    }
}
