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
}

/// The other maps have no tower layout, and keep their centreline on the
/// ground and level: the 3D path is the tower's only (split out of the
/// tower's own tests so a tower change does not build every map).
#[test]
fn the_other_maps_have_no_tower_and_a_flat_path() {
    for m in Map::ALL.into_iter().filter(|&m| m != Map::Tower) {
        let c = m.course();
        assert!(c.tower.is_none(), "{m:?}");
        assert!(c.path.iter().all(|p| p.y == 0), "{m:?}");
        assert!(c.heading.iter().all(|h| h.y == 0), "{m:?}");
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
            Leg::Tower1,
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
    // (from 40 m up the axis: the shaft's length follows the opening's
    // rise, T97)
    let inside = (t.leg_start(Leg::Inside, c.length) / STEP) as usize;
    let on_axis = (inside..)
        .find(|&k| c.path[k].flat().sub(t1.centre.flat()).len() < int(1))
        .unwrap();
    let from = on_axis + 20;
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

/// The beams of the inside climb (tower 1's rings).
fn inside_beams(c: &crate::course::Course) -> Vec<usize> {
    let t = c.tower.as_deref().unwrap();
    (0..c.vines.len())
        .filter(|&i| {
            matches!(
                t.hangers[i],
                Hanger::Window { .. } | Hanger::Floating { .. } | Hanger::Mage { balcony: _ }
            ) && t.leg_at(c.vines[i].at) == Leg::Inside
        })
        .collect()
}

/// T96: "like 3x as many vines as there should be inside the tower". The
/// inside climb hangs about a third of 1.14.0's beams (169 on 5aec39e),
/// still spread evenly up the hollow: every 30 m from the doorway up holds some,
/// and no band more than twice another.
#[test]
fn the_inside_climb_hangs_about_a_third_of_its_old_beams() {
    const BEFORE: usize = 169;
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let inside = inside_beams(&c);
    let n = inside.len();
    assert!(
        n * 4 >= BEFORE && n * 5 <= BEFORE * 2,
        "{n} beams inside (before: {BEFORE}), not about a third"
    );
    // (whole 30 m bands from the lowest inside beam to the rim)
    let foot = inside.iter().map(|&i| c.vines[i].anchor.y).min().unwrap();
    let bands = ((t.towers[0].top() - foot) / int(30)) as usize;
    let mut count = vec![0usize; bands];
    for &i in &inside {
        let k = ((c.vines[i].anchor.y - foot) / int(30)) as usize;
        if k < bands {
            count[k] += 1;
        }
    }
    let band = &count[..];
    let (lo, hi) = (*band.iter().min().unwrap(), *band.iter().max().unwrap());
    assert!(lo > 0 && hi <= lo * 2, "per 30 m band: {band:?}");
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
    let inside = inside_beams(&c);
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

/// The trampolines' spread (T100) over `discs`: the counts in `bands`
/// equal height bands between the highest and the lowest, and in the
/// inner and outer halves of the tube's area.
fn disc_spread(tube: &Tube, discs: &[Disc], bands: usize) -> (Vec<usize>, [usize; 2]) {
    let hi = discs.iter().map(|d| d.centre.y).max().unwrap();
    let lo = discs.iter().map(|d| d.centre.y).min().unwrap();
    let mut counts = vec![0; bands];
    let mut halves = [0; 2];
    // (the inner half of the area the middles may take)
    let across = tube.radius - DISC_RADIUS;
    let half_r = fx::mul(across, ONE * 707 / 1000);
    for d in discs {
        let b = ((d.centre.y - lo) as i64 * bands as i64 / (hi - lo + 1) as i64) as usize;
        counts[b] += 1;
        let rel = v3(d.centre.x - tube.axis.x, 0, d.centre.z - tube.axis.z);
        halves[rel.longer_than(half_r) as usize] += 1;
    }
    (counts, halves)
}

/// The tube holds `TUBE_DISCS` (150) trampolines, each wholly inside it,
/// leaning at most 10°, no two nearer than `DISC_SPACING` (so none
/// touch), spread through the whole volume (about 15 in every 30 m, as
/// many in the outer half of the area as the inner), the same placement
/// in every layout (T100).
#[test]
fn the_tube_holds_150_trampolines_spread_evenly_and_leaning_at_most_10_degrees() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let tube = &t.tube;
    let discs = &tube.discs;
    assert_eq!(TUBE_DISCS, 150);
    assert_eq!(discs.len(), TUBE_DISCS);
    let (mut most_lean, mut least_lean) = (0, i32::MAX);
    for (k, d) in discs.iter().enumerate() {
        let n = d.normal;
        assert!((n.len() - ONE).abs() < ONE / 100, "disc {k}: normal {n:?}");
        let lean = fx::QUARTER - crate::geom::pitch_of(n);
        assert!(
            lean <= DISC_TILT + fx::deg(1) / 4,
            "disc {k} leans {} deg",
            lean * 360 / fx::TURN
        );
        most_lean = most_lean.max(lean);
        least_lean = least_lean.min(lean);
        // wholly inside: its rim in plan inside the wall, its rim's
        // heights inside the tube's
        let rel = v3(d.centre.x - tube.axis.x, 0, d.centre.z - tube.axis.z);
        assert!(
            !rel.longer_than(tube.radius - DISC_RADIUS),
            "disc {k} in the wall"
        );
        assert!(
            d.centre.y + DISC_RADIUS < tube.top && d.centre.y - DISC_RADIUS > tube.bottom,
            "disc {k} out of the tube's height"
        );
    }
    // (random leans: some nearly flat, some nearly the most)
    assert!(least_lean < fx::deg(2) && most_lean > fx::deg(8));
    let mut nearest = i32::MAX;
    for i in 0..discs.len() {
        for j in i + 1..discs.len() {
            nearest = nearest.min(discs[i].centre.dist(discs[j].centre));
        }
    }
    assert!(
        nearest >= crate::tower::layout::DISC_SPACING,
        "two discs {} cm apart",
        nearest * 100 / ONE
    );
    // the volume: about 15 in every 30 m slice, no layers or gaps
    let hi = discs.iter().map(|d| d.centre.y).max().unwrap();
    let lo = discs.iter().map(|d| d.centre.y).min().unwrap();
    assert!(hi - lo > int(240), "spread over {} m", (hi - lo) / ONE);
    let slices = ((hi - lo) / int(30)) as usize;
    let (counts, halves) = disc_spread(tube, discs, slices);
    assert!(
        counts.iter().all(|&n| (12..=24).contains(&n)),
        "per ~30 m slice: {counts:?}"
    );
    assert!(
        halves[0].abs_diff(halves[1]) * 5 <= discs.len(),
        "inner and outer halves: {halves:?}"
    );
    // seen from the top they cover the tube about three times over
    let mut covered = 0;
    let mut n = 0;
    let r = tube.radius;
    let mut x = -r;
    while x <= r {
        let mut z = -r;
        while z <= r {
            if !v3(x, 0, z).longer_than(r) {
                n += 1;
                let p = tube.axis.add(v3(x, 0, z));
                if discs
                    .iter()
                    .any(|d| !v3(p.x - d.centre.x, 0, p.z - d.centre.z).longer_than(DISC_RADIUS))
                {
                    covered += 1;
                }
            }
            z += int(2);
        }
        x += int(2);
    }
    assert!(
        covered * 100 > n * 85,
        "only {covered} of {n} points covered from the top"
    );
    // one fixed placement, every layout
    for k in 1..4 {
        let other = seeded(k);
        let o = other.tower.as_deref().unwrap();
        assert_eq!(o.tube.discs.len(), discs.len());
        for (a, b) in o.tube.discs.iter().zip(discs) {
            assert_eq!(a.normal, b.normal);
            assert_eq!(a.centre.sub(o.tube.axis), b.centre.sub(tube.axis));
        }
    }
}

/// The placement's order keeps any first N spread evenly (T100): the
/// first N of the list are what `tube_discs` makes for N, and each
/// prefix fills the height and the area as the whole does.
#[test]
fn any_first_n_trampolines_stay_evenly_spread() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let tube = &t.tube;
    // (the band the layout places them in)
    let hi = tube.top - crate::tower::layout::DISCS_FROM_TOP;
    let lo = tube.bottom + t.tunnel.height + crate::tower::layout::DISCS_OVER_SLING;
    for n in [10, 20, 40, 75, 110] {
        let first = crate::tower::layout::tube_discs(tube.axis, lo, hi, n);
        assert_eq!(first[..], tube.discs[..n], "the first {n}");
        let bands = (n / 10).max(2);
        let (counts, halves) = disc_spread(tube, &first, bands);
        let each = n / bands;
        assert!(
            counts.iter().all(|&k| k * 2 >= each && k * 2 <= each * 3),
            "first {n}, per band of {bands}: {counts:?}"
        );
        assert!(
            halves[0].abs_diff(halves[1]) * 3 <= n,
            "first {n}, inner and outer: {halves:?}"
        );
        // (the top and bottom halves of the fall alike)
        let top = first.iter().filter(|d| d.centre.y > (hi + lo) / 2).count();
        assert!(
            top.abs_diff(n - top) * 4 <= n,
            "first {n}: {top} in the top half"
        );
    }
}

/// A fall at terminal speed onto a leaning trampoline bounces off its
/// face: up at no more than `DISC_BOUNCE`, its speed across along the
/// face's lean, and losing about 2 s against the same fall without it
/// (T100).
#[test]
fn a_trampoline_kills_the_fall_and_throws_the_ape_along_its_face() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let tube = &t.tube;
    // the most leaning disc high up and away from the wall
    let k = (0..tube.discs.len())
        .filter(|&k| {
            let d = tube.discs[k];
            d.centre.y > tube.bottom + int(140)
                && !v3(d.centre.x - tube.axis.x, 0, d.centre.z - tube.axis.z).longer_than(int(15))
        })
        .max_by_key(|&k| -tube.discs[k].normal.y)
        .unwrap();
    let d = tube.discs[k];
    assert!(d.normal.y < fx::cos(fx::deg(6)), "a leaning disc");
    // the tube with that disc alone, and with none
    let with_discs = |discs: Vec<Disc>| {
        let mut c2 = c.clone();
        c2.tower.as_mut().unwrap().tube.discs = discs;
        c2
    };
    let one = with_discs(vec![d]);
    let none = with_discs(vec![]);
    let start = d.centre.add(v3(0, BODY_DROP + int(8), 0));
    let fall = |course: &crate::course::Course| {
        let mut a = Ape::new(Kind::Chimpanzee, true, start, 0);
        a.mode = Mode::Flying;
        a.vel = v3(0, -TUBE_TERMINAL, 0);
        a.hint = course.nearest_anywhere(start);
        let mut after = None;
        let mut ticks = 0;
        for tick in 0..(12 * TICKS_PER_SECOND) as u32 {
            a.step(&Controls::default(), course, tick);
            if after.is_none() && a.happened.contains(&crate::ape::Happening::Bounce) {
                after = Some(a.vel);
            }
            ticks = tick;
            if a.pos.y < d.centre.y - int(80) {
                break;
            }
        }
        (after, ticks as i32)
    };
    let (bounce, slow) = fall(&one);
    let (none_bounce, fast) = fall(&none);
    assert!(none_bounce.is_none());
    let v = bounce.expect("the disc bounces the fall");
    assert!(
        v.y > 0 && v.y <= DISC_BOUNCE,
        "up at {} cm/s",
        v.y as i64 * 100 / ONE as i64
    );
    // across: along the face's lean
    let (across, lean) = (v.flat(), d.normal.flat());
    assert!(across.longer_than(ONE), "thrown across at {:?}", across);
    assert!(
        across.norm().dot(lean.norm()) > ONE * 95 / 100,
        "thrown {across:?}, the face leans {lean:?}"
    );
    let lost = (slow - fast) * ONE / TICKS_PER_SECOND;
    assert!(
        lost > ONE * 3 / 2 && lost < ONE * 5 / 2,
        "a hit loses {} ms",
        lost as i64 * 1000 / ONE as i64
    );
}

/// Coming into the tube through the portal stops the speed across, so
/// the fall starts straight down whatever the entry (T99); in the tube
/// the steering moves the ape across again.
#[test]
fn coming_into_the_tube_stops_the_speed_across() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let tube = &t.tube;
    let through = t.portal.through;
    let side = crate::geom::right_of(through);
    // just short of the tear, a few metres off its middle, flying in
    // fast and across
    let start = t
        .portal
        .tear
        .sub(through.scale(int(2)))
        .add(side.scale(int(4)));
    let start = v3(start.x, tube.top - int(2), start.z);
    assert!(!t.in_void(start));
    let mut a = Ape::new(Kind::Chimpanzee, true, start, 0);
    a.mode = Mode::Flying;
    a.vel = through.scale(int(20)).add(side.scale(int(7)));
    a.hint = c.nearest_anywhere(start);
    let mut entered = false;
    for tick in 0..TICKS_PER_SECOND as u32 {
        a.step(&Controls::default(), &c, tick);
        if tube.holds(a.pos) && t.in_void(a.pos) {
            assert_eq!(a.vel.flat(), V3::ZERO, "no speed across once in");
            entered = true;
            break;
        }
    }
    assert!(entered, "the ape came into the tube");
    // a few seconds on it is still under where it came in
    let at = a.pos;
    for tick in 0..(2 * TICKS_PER_SECOND) as u32 {
        a.step(&Controls::default(), &c, tick);
    }
    assert!(!v3(a.pos.x - at.x, 0, a.pos.z - at.z).longer_than(int(1)));
    // steering in the tube moves it across again
    let ctl = Controls {
        fwd: 1,
        ..Controls::default()
    };
    for tick in 0..(TICKS_PER_SECOND / 2) as u32 {
        a.step(&ctl, &c, tick);
    }
    assert!(a.vel.flat().longer_than(ONE));
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

/// The AI's steering down the tube (`tube_controls`, dodging the
/// trampoline under it) falls the whole tube to the sling row, from the
/// top, from a start off the axis and from the portal's throw (which
/// once overflew the tube, TC2-9: T99 stops it over the column), and
/// stays in the tube.
#[test]
fn the_tube_steering_falls_the_whole_tube_dodging_trampolines() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let tube = &t.tube;
    for (name, start) in [
        ("the axis", v3(tube.axis.x, tube.top - int(1), tube.axis.z)),
        (
            "20 m off",
            v3(tube.axis.x, tube.top - int(1), tube.axis.z + int(20)),
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
        let (mut bounces, mut ticks, mut reached) = (0, 0, false);
        for tick in 0..(25 * TICKS_PER_SECOND) as u32 {
            let ctl = t.tube_controls(&a).unwrap_or_default();
            a.step(&ctl, &c, tick);
            ticks = tick;
            if a.happened.contains(&crate::ape::Happening::Bounce) {
                bounces += 1;
            }
            if a.pos.y - BODY_DROP < tube.bottom + int(40) {
                reached = a.pos.y > tube.bottom && tube.over(a.pos);
                break;
            }
        }
        // (the core prints nothing: the numbers are in the message)
        assert!(
            reached,
            "from {name}: not down the tube ({bounces} bounces, {ticks} ticks, at {:?})",
            a.pos
        );
        assert!(
            ticks < (20 * TICKS_PER_SECOND) as u32,
            "from {name}: too slow ({bounces} bounces, {ticks} ticks)"
        );
    }
}

fn seeded(k: u64) -> crate::course::Course {
    if k == 0 {
        tower()
    } else {
        tower_seeded(0x70E5_0000_0001 + k * 7919)
    }
}

/// The swing in at tower 1's doorway (TC2-17): from the upper bridge to
/// the entry beams inside the doorway no two beams are more than 35 m
/// apart along the course, over sixteen layouts (without the porch beam
/// the last station stood 24 m or more short of the wall and the entry
/// beams 53-56 m on, and the Sloth and the Monkey swung there to the
/// time limit).
#[test]
fn the_swing_in_at_the_doorway_is_a_hop_from_beam_to_beam() {
    for k in 0..16 {
        let c = seeded(k);
        let t = c.tower.as_deref().unwrap();
        let from = t.leg_start(Leg::UpperBridge, c.length);
        let to = t.leg_start(Leg::Inside, c.length) + 30 * ONE;
        let mut at: Vec<i32> = c
            .vines
            .iter()
            .map(|v| v.at)
            .filter(|a| (from..=to).contains(a))
            .collect();
        at.sort();
        let worst = at.windows(2).map(|w| w[1] - w[0]).max().unwrap();
        assert!(
            worst <= 35 * ONE,
            "layout {k}: a gap of {} m on the swing in",
            worst / ONE
        );
    }
}

/// Out over tower 1's rim (TC2-8, TC2-18): a portal beam within 20 m of
/// the wall hangs its bottom at least 3 m over the rim, over sixteen
/// layouts (at the rim's height a swing out from it crossed the wall's
/// top with the feet in the stone and fell to the floor).
#[test]
fn the_portal_beams_by_the_wall_hang_clear_over_the_rim() {
    for k in 0..16 {
        let c = seeded(k);
        let t = c.tower.as_deref().unwrap();
        let t1 = &t.towers[0];
        let from = t.leg_start(Leg::Portal, c.length);
        let to = t.leg_start(Leg::Tube, c.length);
        for (i, v) in c.vines.iter().enumerate() {
            if !(from..to).contains(&v.at) {
                continue;
            }
            let bottom = v.anchor.add(c.hang(i).scale(v.length));
            let off = bottom.flat().sub(t1.centre.flat());
            if off.longer_than(t1.inner - 20 * ONE) && !off.longer_than(t1.radius + 20 * ONE) {
                assert!(
                    bottom.y >= t1.top() + 3 * ONE,
                    "layout {k}: beam {i}'s bottom {} m by the wall (rim {} m)",
                    bottom.y / ONE,
                    t1.top() / ONE
                );
            }
        }
    }
}

/// T97: "the spawn should be moved so that you climb around tower 1 at the
/// beginning a bit". Before the lower bridge the line goes round tower 1's
/// outside, the tower on the inside of the turn: from the start deck it
/// sweeps over a third of a turn round tower 1's axis, between 12 and 25 m
/// off its wall, rising over 20 m, under beams cast from mages on balconies
/// on tower 1's outside; the start deck and its row stand by tower 1.
#[test]
fn the_course_climbs_round_tower_1_before_the_lower_bridge() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let t1 = &t.towers[0];
    let order: Vec<Leg> = t.legs.iter().take(3).map(|l| l.leg).collect();
    assert_eq!(order, [Leg::Start, Leg::Tower1, Leg::LowerBridge]);
    let from = t.leg_start(Leg::Tower1, c.length);
    let to = t.leg_start(Leg::LowerBridge, c.length);
    let off = |s: i32| c.point(s).flat().sub(t1.centre.flat());
    // the sweep round the axis while the line hugs the wall
    let (mut swept, mut last, mut top) = (0i64, None, 0);
    let mut s = from;
    while s < to {
        let d = off(s);
        let gap = d.len() - t1.radius;
        if gap < int(25) {
            assert!(
                gap > int(12),
                "{} m along: {} m off tower 1's wall",
                s / ONE,
                gap / ONE
            );
            let yaw = crate::geom::yaw_of(d);
            if let Some(y) = last {
                swept += (yaw.wrapping_sub(y) as i64 + fx::TURN as i64 / 2)
                    .rem_euclid(fx::TURN as i64)
                    - fx::TURN as i64 / 2;
            }
            last = Some(yaw);
            top = s;
        }
        s += STEP;
    }
    let swept = swept.abs() * 360 / fx::TURN as i64;
    assert!(swept >= 120, "the line goes {swept} deg round tower 1");
    let rise = c.point(top).y - c.point(from).y;
    assert!(rise > int(20), "it rises {} m round tower 1", rise / ONE);
    // the start deck by tower 1's foot, the starters on it
    let deck = c.platforms[0];
    let gap = deck.centre.flat().sub(t1.centre.flat()).len() - t1.radius;
    assert!(
        gap > int(12) && gap < int(30),
        "the start deck {} m off tower 1",
        gap / ONE
    );
    for p in c.starts {
        assert_eq!(c.support(p), deck.top, "a starter off the deck");
    }
    // its beams cast by mages on tower 1's outside
    let mages = (0..c.vines.len())
        .filter(|&i| {
            t.leg_at(c.vines[i].at) == Leg::Tower1
                && matches!(t.hangers[i], Hanger::Mage { balcony }
                    if t.balconies[balcony].tower == 0 && !t.balconies[balcony].inside)
        })
        .count();
    assert!(mages >= 6, "{mages} beams cast from tower 1's outside");
}

#[test]
fn an_eye_round_the_start_and_the_climbs_is_never_in_the_void() {
    // (the start deck stands by tower 1 past the portal's plane, T97: a
    // camera round it must stay in the towers' world)
    for seed in 0..16u64 {
        let c = tower_seeded(seed);
        let t = c.tower.as_ref().unwrap();
        let mut s = 0;
        while t.leg_at(s) != Leg::Portal {
            let p = c.point(s);
            for k in 0..16 {
                let a = k * fx::TURN / 16;
                for (r, up) in [(int(8), int(4)), (int(14), int(9)), (int(14), int(-6))] {
                    let eye = p.add(v3(fx::mul(fx::cos(a), r), up, fx::mul(fx::sin(a), r)));
                    assert!(
                        !t.in_void(eye),
                        "layout {seed}: an eye {:?} round {} m along is in the void",
                        eye,
                        s / ONE
                    );
                }
            }
            s += int(4);
        }
        // while the tube, the sling and the tunnel past the tear are
        let mut s = c.length - int(1);
        while t.leg_at(s) != Leg::Portal {
            let p = c.point(s);
            assert!(
                t.in_void(p),
                "layout {seed}: {} m along is not in the void",
                s / ONE
            );
            s -= int(4);
        }
    }
}
