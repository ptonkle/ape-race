//! The stone is solid (TW8), and every vine on both halves is placed
//! clear of it: the audit of every beam (its anchor and length, the swing
//! an ape makes on it, the caster's line to it) and the walls' tests.

use super::*;
use crate::ape::{Ape, Happening};
use crate::course::Course;
use crate::fx::{cos, deg, int, sin, ONE};
use crate::geom::right_of;
use crate::rules::{Kind, BODY_DROP};
use crate::sector::DOWN;

/// Where vine `i` is, for a listing: its half, leg and metres along.
fn place(c: &Course, i: usize) -> String {
    let v = c.vines[i];
    let half = split(c.length);
    let (side, s) = if v.at <= half {
        ("climb", v.at)
    } else {
        ("zipper", c.length - v.at)
    };
    let mut leg = 0;
    let mut at = 0;
    for (k, l) in CLIMB.iter().enumerate() {
        if at <= s {
            leg = k;
        }
        at += int(l.0);
    }
    format!("vine {i} ({side}, leg {leg}, {} m along)", v.at / ONE)
}

/// Every problem with the vines' placement against the stone, one line
/// each (empty: none).
fn audit(c: &Course) -> Vec<String> {
    let t = c.tower.as_deref().expect("the tower");
    let casters = crate::towerscape::casters(c, t);
    let mut bad = Vec::new();
    // (0) the line itself: an ape on it is clear of the stone
    for (k, p) in c.path.iter().enumerate() {
        if let Some(s) = t.in_stone(*p) {
            bad.push(format!(
                "the line at {} m runs through stone {s}",
                k as i32 * crate::course::STEP / ONE
            ));
        }
    }
    for (i, v) in c.vines.iter().enumerate() {
        let h = c.hang(i);
        let bottom = v.anchor.add(h.scale(v.length));
        let at = place(c, i);
        // (1) the beam, anchor to bottom, and an ape hanging at its bottom
        for k in 0..=20 {
            let p = v.anchor.add(h.scale(v.length * k / 20));
            if let Some(s) = t.point_in_stone(p) {
                bad.push(format!("{at}: beam in stone {s} at {}/20", k));
                break;
            }
        }
        if let Some(s) = t.in_stone(bottom) {
            bad.push(format!("{at}: an ape at its bottom is in stone {s}"));
        }
        // (2) the swing: out to 55° both ways along the course and 20°
        // either side of that
        let way = {
            let f = c.heading_at(v.at);
            v3(f.x, 0, f.z).norm()
        };
        'swing: for side in [-20, 0, 20] {
            for dir in [1, -1] {
                let d = way
                    .scale(dir * cos(deg(side)))
                    .add(right_of(way).scale(sin(deg(side))));
                // (square to the beam)
                let u = d.sub(h.scale(d.dot(h))).norm();
                for a in [15, 30, 45, 55] {
                    let p = v
                        .anchor
                        .add(h.scale(crate::fx::mul(v.length, cos(deg(a)))))
                        .add(u.scale(crate::fx::mul(v.length, sin(deg(a)))));
                    if let Some(s) = t.in_stone(p) {
                        bad.push(format!(
                            "{at}: its swing {}{a}° (aside {side}°) runs into stone {s}",
                            if dir > 0 { "on " } else { "back " }
                        ));
                        break 'swing;
                    }
                }
            }
        }
        // (3) the caster: its line to the anchor clear of stone, and a
        // balcony's mage casts out from its wall
        if let Some(from) = casters.get(i).copied().flatten() {
            for k in 1..20 {
                let p = from.add(v.anchor.sub(from).scale(ONE * k / 20));
                if let Some(s) = t.point_in_stone(p) {
                    bad.push(format!("{at}: its caster's line passes stone {s}"));
                    break;
                }
            }
        }
        if let Some(Hanger::Mage { balcony }) = t.hangers.get(i) {
            let b = t.balconies[*balcony];
            if v.anchor.sub(b.at).flat().dot(b.out) < 0 {
                bad.push(format!("{at}: cast from behind balcony {balcony}'s wall"));
            }
        }
    }
    bad
}

/// Layout `k` (0: the course everyone races).
fn layout(k: u64) -> Course {
    if k == 0 {
        tower()
    } else {
        tower_seeded(0x70E5_0000_0001 + k * 7919)
    }
}

/// The person's report (TW8): "why can you go through like all the
/// walls?" Every vine on both halves, over sixteen layouts: its beam, its
/// swing and its caster's line are clear of the stone.
#[test]
fn every_vine_is_clear_of_the_stone() {
    let mut all = Vec::new();
    for k in 0..16 {
        for line in audit(&layout(k)) {
            all.push(format!("layout {k}: {line}"));
        }
    }
    assert!(
        all.is_empty(),
        "{} vine placements run into stone:\n{}",
        all.len(),
        all.join("\n")
    );
}

/// An ape flown at `to` from `from` (the hands) at `speed` m/s: where
/// it is a second on, and whether it hit a wall.
fn fly(c: &Course, from: V3, to: V3, speed: i32) -> (V3, bool) {
    let mut a = Ape::new(Kind::Chimpanzee, true, from, 0);
    a.vel = to.sub(from).with_len(int(speed));
    a.mode = crate::ape::Mode::Flying;
    a.hint = c.nearest_anywhere(from);
    let mut hit = false;
    for tick in 0..crate::rules::TICKS_PER_SECOND as u32 {
        a.step(&Default::default(), c, tick);
        hit |= a.happened.contains(&Happening::Wall);
        a.happened.clear();
    }
    (a.pos, hit)
}

fn flat_dist(a: V3, b: V3) -> i32 {
    a.sub(b).flat().len()
}

/// Flown at the keep, at a sub-tower and at the room's wall from outside,
/// an ape is pushed back: it ends no nearer the axis than the wall (and
/// its radius), with a `Wall` hit.
#[test]
fn the_keep_the_sub_towers_and_the_rooms_wall_are_solid() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let k = &t.keep;
    let sub = t.branches[0].sub;
    let room = &t.room;
    let wall_y = room.centre.y + room.height / 2;
    // (the room from the side away from its doorway)
    let back = room
        .centre
        .sub(room.door.sub(room.centre).flat().norm().scale(int(30)));
    let cases = [
        ("the keep", k.centre, k.radius, int(30)),
        ("a sub-tower", sub.centre, sub.radius, sub.top / 2),
        ("the room's wall", room.centre, room.radius, wall_y),
    ];
    for (what, centre, radius, y) in cases {
        let axis = v3(centre.x, y, centre.z);
        // (a sub-tower from the side away from the keep, clear of its
        // bridges)
        let away = axis.sub(k.centre).flat().norm();
        let from = match what {
            "the room's wall" => v3(back.x, y, back.z),
            "a sub-tower" => axis.add(away.scale(radius + int(15))),
            _ => axis.add(v3(radius + int(15), 0, 0)),
        };
        let (end, hit) = fly(&c, from, axis, 25);
        assert!(hit, "{what}: no wall hit");
        assert!(
            flat_dist(end, axis) >= radius,
            "{what}: the ape went through to {} m from its axis (wall at {} m)",
            flat_dist(end, axis) / ONE,
            radius / ONE
        );
    }
}

/// The room's doorway and its roof hole let an ape through: flown in at
/// the door it ends up inside; thrown up under the hole it ends up over
/// the roof; thrown up away from the hole it is stopped by the roof.
#[test]
fn the_rooms_door_and_roof_hole_let_an_ape_through() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let room = &t.room;
    let out = room.door.sub(room.centre).flat().norm();
    let roof = room.centre.y + room.height;
    let at_door = v3(room.door.x, room.centre.y + int(6), room.door.z);
    let (end, hit) = fly(
        &c,
        at_door.add(out.scale(int(4))),
        at_door.sub(out.scale(int(10))),
        16,
    );
    assert!(!hit, "the doorway is open");
    assert!(
        flat_dist(end, room.centre) < room.radius,
        "flown in at the door it is inside the room"
    );
    let (hole, _) = room.roof_exit;
    let under = v3(hole.x, roof - int(6), hole.z);
    let (end, hit) = fly(&c, under, under.add(v3(0, ONE, 0)), 22);
    assert!(!hit, "the roof hole is open");
    assert!(
        end.y > roof + BODY_DROP,
        "thrown up under the hole it is out over the roof"
    );
    let aside = room.centre.add(out.scale(room.radius - int(3)));
    let under = v3(aside.x, roof - int(6), aside.z);
    let (end, hit) = fly(&c, under, under.add(v3(0, ONE, 0)), 22);
    assert!(hit, "the roof stops it");
    assert!(
        end.y < roof,
        "thrown up away from the hole it stays under the roof"
    );
}

/// The tops are ground: an ape stands on the keep's crown, and on a
/// bridge's deck at its drawn height at both ends (TL1).
#[test]
fn the_crown_and_the_bridge_decks_are_stood_on_where_drawn() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let crown = v3(t.keep.centre.x, t.keep.height + BODY_DROP, t.keep.centre.z);
    assert_eq!(c.support(crown), t.keep.height, "the crown holds an ape");
    let br = &t.branches[0];
    for (u, end) in [(ONE / 10, br.from), (ONE * 9 / 10, br.to)] {
        let p = br.from.add(br.to.sub(br.from).scale(u));
        let deck = br.from.y + crate::fx::mul(br.to.y - br.from.y, u);
        let s = c.support(v3(p.x, deck + BODY_DROP, p.z));
        assert!(
            (s - deck).abs() < crate::fx::cm(20),
            "the deck near {:?} holds an ape at {} m, drawn at {} m",
            end,
            s / ONE,
            deck / ONE
        );
    }
    let _ = DOWN;
}

#[test]
#[ignore]
fn print_sub_towers() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let mut seen = Vec::new();
    for br in &t.branches {
        let s = br.sub;
        if seen.contains(&s.centre) {
            continue;
        }
        seen.push(s.centre);
        let near: Vec<(i32, i32)> = c
            .path
            .iter()
            .enumerate()
            .filter(|(_, p)| p.sub(s.centre).flat().len() < s.radius + int(14) && p.y < s.top)
            .map(|(k, p)| (k as i32 * crate::course::STEP / ONE, p.y / ONE))
            .collect();
        println!(
            "sub at {:?} r {} top {} near: {:?}",
            (s.centre.x / ONE, s.centre.z / ONE),
            s.radius / ONE,
            s.top / ONE,
            near
        );
    }
    println!("keep r {} h {}", t.keep.radius / ONE, t.keep.height / ONE);
}

#[test]
#[ignore]
fn print_vine() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let i: usize = std::env::var("VINE")
        .map(|v| v.parse().unwrap())
        .unwrap_or(39);
    let v = c.vines[i];
    let m = |p: V3| {
        (
            p.x as f64 / 65536.0,
            p.y as f64 / 65536.0,
            p.z as f64 / 65536.0,
        )
    };
    println!(
        "vine {i} anchor {:?} len {} hang {:?} at {}",
        m(v.anchor),
        v.length / ONE,
        m(c.hang(i)),
        v.at / ONE
    );
    for (k, s) in t.stone.iter().enumerate() {
        println!("stone {k}: {:?}", s);
    }
    let h = c.hang(i);
    let f = c.heading_at(v.at);
    let way = v3(f.x, 0, f.z).norm();
    println!("way {:?}", m(way));
    for a in [15, 30, 45, 55] {
        let d = way
            .scale(cos(deg(-20)))
            .add(right_of(way).scale(sin(deg(-20))));
        let u = d.sub(h.scale(d.dot(h))).norm();
        let p = v
            .anchor
            .add(h.scale(crate::fx::mul(v.length, cos(deg(a)))))
            .add(u.scale(crate::fx::mul(v.length, sin(deg(a)))));
        println!("{a}: {:?} {:?}", m(p), t.in_stone(p));
    }
    for br in &t.branches {
        println!(
            "branch from {:?} to {:?} sub {:?} r {} top {}",
            m(br.from),
            m(br.to),
            m(br.sub.centre),
            br.sub.radius / ONE,
            br.sub.top / ONE
        );
    }
}
