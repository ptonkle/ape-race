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
        assert_eq!(c.ravine.is_some(), map == Map::Ravine, "{map:?}");
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
fn the_ravine_is_held_from_the_map_choice_and_random() {
    assert!(Map::Ravine.held());
    assert!(!Map::offered().any(|m| m == Map::Ravine));
    // every other map is still offered
    assert_eq!(Map::offered().count(), Map::ALL.len() - 1);
    for seed in 0..500u64 {
        assert_ne!(Map::random(seed, Map::Jungle), Map::Ravine, "seed {seed}");
    }
}

#[test]
fn a_lizard_walks_its_wall_and_back_eased_and_periodic() {
    let mut c = ravine();
    let r = c.ravine.as_deref_mut().unwrap();
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

/// The stand-in with one straight track from a hole at x=0 to one at
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
