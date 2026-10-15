//! Banana Bay Battle's contract: the layout's wiring, the swell, the ships'
//! geometry, the hangers and the cannon fire.

use super::*;
use crate::map::Map;

/// A small test ship: a brig at `at` facing +x, one mast with two yards.
fn brig(at: V3) -> Ship {
    Ship {
        kind: ShipKind::Brig,
        side: Side::Navy,
        at,
        yaw: fx::QUARTER,
        length: int(28),
        beam: int(8),
        deck: int(4),
        raised: vec![(-int(14), -int(8), int(6))],
        bowsprit: int(8),
        masts: vec![Mast {
            along: int(4),
            height: int(24),
            yards: vec![
                Yard {
                    height: int(10),
                    half_span: int(8),
                    sail: Sail::Missing,
                },
                Yard {
                    height: int(18),
                    half_span: int(6),
                    sail: Sail::Set,
                },
            ],
            gaff: None,
        }],
        jib: Some(Sail::Torn),
        ports: vec![Port {
            along: 0,
            side: 1,
            up: int(2),
        }],
        decks: Vec::new(),
    }
}

#[test]
fn only_banana_bay_battle_has_a_bay_layout() {
    for map in Map::ALL {
        let c = map.course();
        assert_eq!(c.bay.is_some(), map == Map::Bay, "{map:?}");
        assert_eq!(Map::of(&c), map);
    }
    let c = bay();
    assert_eq!(c.name, "bay");
    let b = c.bay.as_ref().unwrap();
    assert_eq!(b.hangers.len(), c.vines.len(), "one hanger per vine");
    assert!(
        b.dock.platform < c.platforms.len(),
        "the dock is a platform"
    );
}

#[test]
fn the_swell_rises_and_falls_in_place_from_the_tick() {
    let w = Water {
        level: int(1),
        waves: vec![
            Wave {
                amp: cm(40),
                dir: v3(ONE, 0, 0),
                length: int(30),
                period: 480,
            },
            Wave {
                amp: cm(15),
                dir: v3(0, 0, ONE),
                length: int(11),
                period: 300,
            },
        ],
    };
    assert_eq!(w.peak(), cm(55));
    let mut lo = i32::MAX;
    let mut hi = i32::MIN;
    for tick in 0..2400 {
        let h = w.height(int(7), int(-3), tick);
        assert!((h - w.level).abs() <= w.peak() + cm(1), "{h}");
        lo = lo.min(h);
        hi = hi.max(h);
    }
    assert!(hi - lo > cm(60), "it moves: {lo}..{hi}");
    // periodic: the periods' common multiple
    assert_eq!(w.height(int(7), 0, 13), w.height(int(7), 0, 13 + 2400));
    // and it differs from place to place
    assert_ne!(w.height(0, 0, 50), w.height(int(9), 0, 50));
    // far out on a long course and late in a long race, no overflow
    let _ = w.height(int(20_000), -int(20_000), u32::MAX);
    // still water is flat
    let still = Water {
        level: int(1),
        waves: Vec::new(),
    };
    assert_eq!(still.height(int(5), int(5), 77), int(1));
}

#[test]
fn a_ship_knows_its_rigging_and_its_outline() {
    let s = brig(v3(int(100), 0, int(50)));
    // facing +x: the bow is +x, starboard (`geom::right_of`) is +z
    assert!(s.fwd().dist(v3(ONE, 0, 0)) < cm(1), "{:?}", s.fwd());
    assert!(s.right().dist(v3(0, 0, ONE)) < cm(1), "{:?}", s.right());
    let top = s.mast_top(0);
    assert!(top.dist(v3(int(104), int(28), int(50))) < cm(2), "{top:?}");
    let (l, r) = s.yard_ends(0, 0);
    assert!(l.dist(v3(int(104), int(14), int(42))) < cm(2), "{l:?}");
    assert!(r.dist(v3(int(104), int(14), int(58))) < cm(2), "{r:?}");
    assert!(s.over(int(113), int(53)) && !s.over(int(115), int(50)));
    assert!(!s.over(int(100), int(55)));
    assert!(Sail::Missing.clear() && Sail::Torn.clear() && Sail::Furled.clear());
    assert!(!Sail::Set.clear());
}

#[test]
fn hangers_name_real_rigging_and_the_ship_under_an_ape_is_found() {
    let mut b = bay().bay.unwrap();
    b.ships.clear();
    b.ships.push(brig(v3(int(100), 0, int(50))));
    b.ships.push(brig(v3(int(200), 0, int(50))));
    let yard = Hanger::Yard {
        ship: 1,
        mast: 0,
        yard: 0,
        out: int(5),
    };
    let p = yard.point(&b).unwrap();
    assert!(p.dist(v3(int(204), int(14), int(55))) < cm(2), "{p:?}");
    let mast = Hanger::Mast {
        ship: 0,
        mast: 0,
        up: int(20),
    };
    assert!(mast.point(&b).unwrap().dist(v3(int(104), int(24), int(50))) < cm(2));
    let sprit = Hanger::Bowsprit {
        ship: 0,
        out: int(6),
    };
    assert!(sprit.point(&b).unwrap().dist(v3(int(120), int(4), int(50))) < cm(2));
    let bad = Hanger::Yard {
        ship: 1,
        mast: 0,
        yard: 9,
        out: 0,
    };
    assert_eq!(bad.point(&b), None);
    assert_eq!(b.ship_under(int(205), int(52)), Some(1));
    assert_eq!(b.ship_under(int(100), int(51)), Some(0));
    assert_eq!(b.ship_under(int(150), int(50)), None);
}

#[test]
fn a_cannonball_flies_from_its_port_to_its_target_in_each_cycle() {
    let mut b = bay().bay.unwrap();
    b.ships.clear();
    b.volleys.clear();
    b.ships.push(brig(v3(int(100), 0, int(50))));
    b.ships.push(brig(v3(int(100), 0, int(10))));
    let target = v3(int(100), int(2), int(14));
    b.volleys.push(Volley {
        from: 0,
        port: 0,
        at: 1,
        target,
        hit: true,
        fire: 100,
        flight: 120,
    });
    assert_eq!(b.ball_at(0, 99), None, "not fired yet");
    let start = b.ball_at(0, 100).unwrap();
    // the starboard port of a ship facing +x is on its +z side
    assert!(
        start.dist(v3(int(100), int(2), int(54))) < cm(2),
        "{start:?}"
    );
    let mid = b.ball_at(0, 160).unwrap();
    assert!(mid.y > int(2) + ARC - cm(5), "it arcs: {mid:?}");
    assert!(
        b.ball_at(0, 219).unwrap().dist(target) < cm(60),
        "one tick out"
    );
    assert_eq!(b.ball_at(0, 220), None, "landed");
    assert_eq!(
        b.ball_at(0, 130),
        b.ball_at(0, 130 + VOLLEY_CYCLE),
        "repeats"
    );
    assert_eq!(b.ball_at(5, 130), None, "no such volley");
}
