//! Mape's Tower's skeleton: the map is wired, the 3D centreline and its
//! mirror, along-the-course on a path that climbs then falls (TW1), the
//! reset onto a pad with its boost, and the climb vines' lift.

use super::*;
use crate::ape::Mode;
use crate::course::STEP;
use crate::fx::ONE;
use crate::geom::V3;
use crate::map::Map;
use crate::race::{Driver, Entry, Event, Phase, Race};
use crate::rules::{Kind, BODY_DROP};

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

#[test]
fn the_centreline_climbs_to_the_tear_then_falls_its_mirror_as_far() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let n = c.path.len();
    assert_eq!(n % 2, 1, "a middle sample at the tear");
    let mid = n / 2;
    assert_eq!(mid as i32 * STEP, split(c.length));
    // the tear is on the mirror plane, the top of the course
    assert_eq!(c.path[mid].x, t.zipper.mirror_x);
    let top = c.path.iter().map(|p| p.y).max().unwrap();
    assert_eq!(c.path[mid].y, top);
    assert!(
        top >= START_TOP + int(60),
        "climbs {} m",
        (top - START_TOP) / ONE
    );
    // each zipper sample is its climb sample mirrored
    for k in 0..n {
        assert_eq!(c.path[n - 1 - k], t.mirror(c.path[k]), "sample {k}");
    }
    // every step is STEP long (to a centimetre) and its heading points to
    // the next
    for i in 0..n - 1 {
        let d = c.path[i + 1].sub(c.path[i]);
        assert!((d.len() - STEP).abs() < cm(1), "step {i}: {}", d.len());
        assert!(d.norm().sub(c.heading[i]).len() < cm(1), "heading {i}");
    }
    // up to the tear (level on the branches), then down
    assert!((0..mid).all(|i| c.heading[i].y >= 0));
    assert!((mid + 1..n).all(|i| c.heading[i].y <= 0));
    assert!(c.heading[25].y > 0, "the first spiral climbs");
}

#[test]
fn along_the_course_tells_the_coils_apart_by_height() {
    let c = tower();
    let n = c.path.len();
    let mid = n / 2;
    // on the climb's top coil (the last spiral, the first of its samples
    // from 35 short of the tear that lies over a lower coil) and the
    // zipper's matching one: a point 3 m over the centreline, right above
    // a sample of the coil below (the nearest sample in plan), is found on
    // its own coil
    let under_of = |k: usize| {
        (0..k - 50)
            .min_by_key(|&j| c.path[j].flat().sub(c.path[k].flat()).len())
            .unwrap()
    };
    let k = (mid - 70..mid - 35)
        .rev()
        .find(|&k| c.path[under_of(k)].flat().sub(c.path[k].flat()).len() < int(2))
        .unwrap_or(mid - 35);
    let under = under_of(k);
    assert!(
        c.path[under].flat().sub(c.path[k].flat()).len() < int(2),
        "the climb passes under itself in plan"
    );
    for (k, j) in [(k, under), (n - 1 - k, n - 1 - under)] {
        let p = v3(c.path[j].x, c.path[k].y + int(3), c.path[j].z);
        let i = c.nearest_anywhere(p);
        assert!((i as i32 - k as i32).abs() <= 2, "sample {k}: nearest {i}");
        let (s, _) = c.progress(p, i);
        assert!(
            (s - k as i32 * STEP).abs() < int(3),
            "sample {k}: {} m",
            s / ONE
        );
        // and the windowed search from the ape's last place agrees
        assert!((c.nearest(p, k) as i32 - k as i32).abs() <= 2);
    }
}

#[test]
fn out_of_bounds_puts_the_ape_on_the_pad_behind_flying_with_its_boost() {
    let c = tower();
    let pads = c.tower.as_deref().unwrap().pads.clone();
    let mut r = Race::new(c, &entries(2), false, 1);
    r.phase = Phase::Running;
    // 130 m along, far under the centreline
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
        let mut a = crate::ape::Ape::new(Kind::Chimpanzee, false, c.vines[vine].anchor, 0);
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

/// A vine summoned high over the first spiral is placed along the course
/// by its own coil, not the coil above (TC5: from its anchor, the nearest
/// sample was on spiral C, 400 m on, and the AI swung on it looking for
/// vines up there).
#[test]
fn a_summoned_vine_is_placed_along_its_own_coil() {
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
fn the_summit_balconies_stand_on_the_keep_under_its_crown() {
    // (TA5: they stood at y 123-130 over a keep 102 m tall)
    let c = tower();
    let t = c.tower.as_ref().unwrap();
    for (n, b) in t.balconies.iter().enumerate() {
        let off = b.at.flat().sub(t.keep.centre.flat()).len() - t.keep.radius;
        if off.abs() < ONE {
            assert!(
                b.at.y + 4 * ONE <= t.keep.centre.y + t.keep.height,
                "balcony {n} at y {} over a keep {} m tall",
                b.at.y / ONE,
                t.keep.height / ONE
            );
        }
    }
    assert_eq!(t.summit.deck.y, t.keep.centre.y + t.keep.height);
}

#[test]
fn the_eldritch_banana_is_in_the_tear_not_beyond_it() {
    // (TA6: 150 m beyond and 40 m up, seen from all over the climb)
    let c = tower();
    let s = c.tower.as_ref().unwrap().summit;
    let off = s.banana.sub(s.tear);
    let behind = off.dot(s.through);
    assert!(
        behind > 0 && behind <= ONE,
        "{} m behind the tear",
        behind / ONE
    );
    assert!(!off.sub(s.through.scale(behind)).longer_than(2 * ONE));
    assert!(
        s.banana_len <= 2 * s.tear_half.0 * 4 / 5,
        "longer than the tear"
    );
}

#[test]
fn item_crates_hang_on_whole_stations_and_never_on_the_start_row() {
    for seed in 0..4 {
        let c = tower_seeded(seed);
        let t = c.tower.as_deref().unwrap();
        let carry = crate::items::crate_vines(&c);
        assert!(carry.len() >= 8, "seed {seed}: {} crates", carry.len());
        for &v in &carry {
            assert_ne!(
                t.hangers[v],
                Hanger::Start,
                "seed {seed}: start vine {v} has a crate"
            );
            // every beam of its station carries one too
            for (i, w) in c.vines.iter().enumerate() {
                if t.hangers[i] != Hanger::Start && (w.at - c.vines[v].at).abs() <= int(4) {
                    assert!(
                        carry.contains(&i),
                        "seed {seed}: vine {i} beside crate vine {v}"
                    );
                }
            }
        }
    }
}
