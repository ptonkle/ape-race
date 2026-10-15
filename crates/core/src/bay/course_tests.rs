//! Banana Bay Battle's course and rules: the line of ships, the sails a
//! swing passes, the water reset, the barrels.

use super::*;
use crate::ape::{Controls, Happening, Mode};
use crate::course::Course;
use crate::race::{Driver, Entry, Event, Race};
use crate::rules::{Kind, BODY_DROP, BODY_RADIUS};

fn running(lava: bool) -> Race {
    let entries = [Entry {
        name: "P0".into(),
        kind: Kind::Chimpanzee,
        driver: Driver::Local,
        palette: 0,
        costume: crate::costume::Costume::Own,
    }];
    let mut r = Race::new(bay(), &entries, false, 1);
    r.lava = lava;
    while r.phase != crate::race::Phase::Running {
        r.step(&[Controls::default()]);
    }
    r
}

/// Puts ape 0 at `p`, moving `v`, in `mode`.
fn put(r: &mut Race, p: V3, v: V3, mode: Mode) {
    let a = &mut r.racers[0].ape;
    a.pos = p;
    a.vel = v;
    a.mode = mode;
    a.hint = r.course.nearest_anywhere(p);
}

fn layout(c: &Course) -> &BayLayout {
    c.bay.as_deref().unwrap()
}

/// Metres along of a point.
fn along(c: &Course, p: V3) -> i32 {
    c.progress(p, c.nearest_anywhere(p)).0
}

#[test]
fn the_line_runs_from_the_dock_through_the_fleet_to_the_island() {
    for k in 0..4u64 {
        let c = if k == 0 {
            bay()
        } else {
            bay_seeded(0xBA7_BA77_0001 + k * 7919)
        };
        let b = layout(&c);
        assert!((20..=30).contains(&b.ships.len()), "{}", b.ships.len());
        let kinds = [
            ShipKind::Galleon,
            ShipKind::Frigate,
            ShipKind::Brig,
            ShipKind::Sloop,
        ];
        assert!(kinds
            .iter()
            .all(|k| b.ships.iter().filter(|s| s.kind == *k).count() >= 3));
        assert!(b.ships.iter().any(|s| s.side == Side::Pirate));
        assert!(b.ships.iter().any(|s| s.side == Side::Navy));
        // every rope hangs from its rigging
        assert_eq!(b.hangers.len(), c.vines.len());
        for (i, h) in b.hangers.iter().enumerate() {
            let p = h.point(b).expect("real rigging");
            let a = c.vines[i].anchor;
            match h {
                // a post's foot, under its top
                Hanger::Shore { foot } => {
                    assert!(foot.y < a.y && foot.flat().sub(a.flat()).len() <= int(13))
                }
                _ => assert!(p.dist(a) < cm(2), "vine {i}: {h:?}"),
            }
        }
        // in course order, hulls apart, decks walkable at deck height
        let mut last = -int(1000);
        for (i, s) in b.ships.iter().enumerate() {
            let a = along(&c, s.at);
            assert!(a > last, "ship {i} in order");
            last = a;
            assert_eq!(c.platforms[s.decks[0]].top, s.deck);
            assert_eq!(c.support(s.point(0, 0, s.deck + BODY_DROP)), s.deck);
            if let Some(n) = b.ships.get(i + 1) {
                let bow = along(&c, s.point(s.length / 2, 0, 0));
                let stern = along(&c, n.point(-n.length / 2, 0, 0));
                assert!(stern > bow + int(2), "ship {i}'s bow clears the next");
            }
        }
        // the dock is the start, the finish is on the island
        assert_eq!(c.platforms[b.dock.platform].top, DOCK_TOP);
        assert_eq!(c.support(c.starts[0]), DOCK_TOP);
        let finish = c.point(c.length);
        assert_eq!(c.support(v3(finish.x, int(5), finish.z)), ISLAND_TOP);
        assert!(finish.flat().sub(b.island.centre.flat()).len() < b.island.radius);
        assert!(b.island.chest.flat().sub(b.island.centre.flat()).len() < b.island.radius);
        // scenery: every ship fights and fires
        for k in 0..b.ships.len() {
            assert!(b.fighters.iter().any(|f| f.ship == k), "fighters on {k}");
            assert!(b.volleys.iter().any(|v| v.from == k), "{k} fires");
        }
        assert!(b.water.peak() > cm(20) && b.water.peak() < BARREL_TOP);
    }
}

/// The boxes a swinging ape must never enter: every set sail, as the
/// drawing hangs it (a square sail from its yard down to the next yard
/// below, or to the deck), as thick as a body. Gaffs and jibs must not be
/// set on this course at all.
pub(crate) fn set_sails(b: &BayLayout) -> Vec<(usize, i32, i32, i32, i32)> {
    let mut out = Vec::new();
    for (k, s) in b.ships.iter().enumerate() {
        for m in &s.masts {
            for (y, yard) in m.yards.iter().enumerate() {
                if yard.sail != Sail::Set {
                    continue;
                }
                let low = if y == 0 { 0 } else { m.yards[y - 1].height };
                out.push((
                    k,
                    m.along,
                    yard.half_span,
                    s.deck + low,
                    s.deck + yard.height,
                ));
            }
        }
    }
    out
}

/// Whether a body at `p` is inside one of `sails`.
pub(crate) fn in_a_set_sail(b: &BayLayout, sails: &[(usize, i32, i32, i32, i32)], p: V3) -> bool {
    sails.iter().any(|&(k, along, half, lo, hi)| {
        let s = &b.ships[k];
        let d = p.sub(s.at);
        let a = d.dot(s.fwd()) - along;
        let x = d.dot(s.right());
        a.abs() <= BODY_RADIUS
            && x.abs() <= half + BODY_RADIUS
            && p.y >= lo - BODY_RADIUS
            && p.y - BODY_DROP <= hi
    })
}

#[test]
fn no_rope_swings_through_a_set_sail() {
    let c = bay();
    let b = layout(&c);
    for s in &b.ships {
        assert_ne!(s.jib, Some(Sail::Set));
        assert!(s.masts.iter().all(|m| m.gaff != Some(Sail::Set)));
    }
    // a mast that carries a rope has no sail set; a set sail is higher than
    // every rope's anchor near it, by more than a body
    for (i, h) in b.hangers.iter().enumerate() {
        if let Hanger::Yard { ship, mast, .. } = *h {
            let m = &b.ships[ship].masts[mast];
            assert!(m.yards.iter().all(|y| y.sail.clear()), "vine {i}'s mast");
        }
    }
    let sails = set_sails(b);
    for (i, v) in c.vines.iter().enumerate() {
        // the hanging rope, and the swing's arc along the course out to
        // 60° either side
        for deg in -6..=6 {
            let a = fx::deg(deg * 10);
            let h = c.heading_at(v.at);
            let reach = v.length + cm(50);
            let hands = v.anchor.add(h.scale(fx::mul(fx::sin(a), reach))).sub(v3(
                0,
                fx::mul(fx::cos(a), reach),
                0,
            ));
            assert!(
                !in_a_set_sail(b, &sails, hands),
                "vine {i} at {}°: through a set sail",
                deg * 10
            );
        }
    }
}

#[test]
fn a_fall_into_the_water_puts_you_back_on_the_last_ship_you_passed() {
    let mut r = running(false);
    let n = layout(&r.course).ships.len();
    for k in [0, 1, 7, n - 1] {
        let s = layout(&r.course).ships[k].clone();
        // just past its bow, in the water, falling
        let p = s.point(s.length / 2 + int(2), 0, cm(40) + BODY_DROP);
        put(&mut r, p, v3(0, -int(10), 0), Mode::Flying);
        let mut reset = false;
        for _ in 0..30 {
            r.step(&[Controls::default()]);
            reset = r.events.iter().any(|e| matches!(e, Event::Reset { .. }));
            if reset {
                break;
            }
        }
        assert!(reset, "past ship {k}: in the water");
        let a = &r.racers[0].ape;
        let spot = s.point(s.masts[0].along - RESET_AFT, 0, s.deck + BODY_DROP);
        assert_eq!(a.pos, spot, "put back on ship {k}");
        assert_eq!(a.mode, Mode::Grounded);
        assert!(
            r.events.contains(&Event::Reset {
                who: 0,
                pad: s.decks[0]
            }),
            "{k}"
        );
        assert_eq!(r.course.support(a.pos), s.deck, "on its deck");
        // and its low rope is ahead, in a jump's reach
        let low = r.course.vines.iter().any(|v| {
            let d = v.anchor.sub(a.pos);
            v.anchor.y - v.length <= s.deck + int(4)
                && d.dot(s.fwd()) > 0
                && d.dot(s.fwd()) < int(12)
        });
        assert!(low, "a low rope ahead on ship {k}");
        // standing on the deck is not the water
        for _ in 0..30 {
            r.step(&[Controls::default()]);
        }
        assert!(!r.events.iter().any(|e| matches!(e, Event::Reset { .. })));
    }
    // short of the first ship: the dock
    let p = r.course.point(int(18));
    put(
        &mut r,
        v3(p.x, BODY_DROP, p.z),
        v3(0, -int(5), 0),
        Mode::Flying,
    );
    for _ in 0..30 {
        r.step(&[Controls::default()]);
        if !r.events.is_empty() {
            break;
        }
    }
    let b = layout(&r.course);
    assert!(r.events.contains(&Event::Reset {
        who: 0,
        pad: b.dock.platform
    }));
    assert_eq!(r.course.support(r.racers[0].ape.pos), DOCK_TOP);
    // Floor is Lava: the water is the floor, and the kill
    let mut r = running(true);
    let q = r.course.point(int(18));
    put(
        &mut r,
        v3(q.x, BODY_DROP, q.z),
        v3(0, -int(5), 0),
        Mode::Flying,
    );
    for _ in 0..30 {
        r.step(&[Controls::default()]);
    }
    assert!(r.racers[0].eaten.is_some(), "eaten in the water");
    assert!(!r.events.iter().any(|e| matches!(e, Event::Reset { .. })));
}

#[test]
fn the_barrels_mark_a_soft_wall_you_cannot_land_on() {
    let mut r = running(false);
    let c = r.course.clone();
    let b = layout(&c);
    for bb in &b.boundary {
        let i = c.nearest_anywhere(bb.at);
        let (_, side) = c.progress(bb.at, i);
        assert!(
            (side.abs() - c.half_width).abs() < cm(50) && side.signum() == bb.edge,
            "a barrel on the edge: {side}"
        );
        // nothing to stand on
        assert_eq!(
            c.support(v3(bb.at.x, int(2), bb.at.z)),
            c.floor(bb.at.x, bb.at.z)
        );
    }
    // a drift outward at 15 m/s is held inside the barrels
    let s = int(400);
    let h = c.heading_at(s);
    let right = crate::geom::right_of(h);
    for edge in [-1, 1] {
        let p = c
            .point(s)
            .add(right.scale(edge * int(17)))
            .add(v3(0, int(30), 0));
        put(&mut r, p, right.scale(edge * int(15)), Mode::Flying);
        for _ in 0..60 {
            r.step(&[Controls::default()]);
            let (_, side) = c.progress(r.racers[0].ape.pos, r.racers[0].ape.hint);
            assert!(side.abs() <= c.half_width, "out past the barrels: {side}");
        }
    }
}

#[test]
fn a_bouncy_barrel_bounces_you_before_the_water() {
    let mut r = running(false);
    let b = layout(&r.course).clone();
    assert_eq!(b.bouncy.len(), b.ships.len());
    for bb in b.bouncy.iter().take(5) {
        let m = r.course.mushrooms[bb.bouncer];
        assert_eq!(m.top.y, WATER_LEVEL + BARREL_TOP);
        // clear of every hull
        assert_eq!(b.ship_under(m.top.x, m.top.z), None);
        put(
            &mut r,
            v3(m.top.x, m.top.y + BODY_DROP + int(3), m.top.z),
            v3(0, -int(12), 0),
            Mode::Flying,
        );
        let mut bounced = false;
        for _ in 0..60 {
            r.step(&[Controls::default()]);
            assert!(!r.events.iter().any(|e| matches!(e, Event::Reset { .. })));
            bounced |= r.racers[0].ape.happened.contains(&Happening::Bounce);
            if bounced {
                break;
            }
        }
        assert!(bounced, "bounced off barrel {}", bb.bouncer);
        assert!(r.racers[0].ape.vel.y > int(10), "up");
    }
}
