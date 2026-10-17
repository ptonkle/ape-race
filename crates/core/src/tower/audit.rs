//! The stone is solid (TW12), and every vine on both halves is placed
//! clear of it: the audit of every beam (its anchor and length, the swing
//! an ape makes on it, the caster's line to it) and the walls' tests.

use super::*;
use crate::ape::{Ape, Happening};
use crate::course::Course;
use crate::fx::{cos, deg, int, sin, ONE};
use crate::geom::right_of;
use crate::rules::{Kind, BODY_DROP};
use crate::sector::DOWN;

/// Where vine `i` is, for a listing: its leg and metres along.
fn place(c: &Course, i: usize) -> String {
    let v = c.vines[i];
    let t = c.tower.as_deref().expect("the tower");
    format!("vine {i} ({:?}, {} m along)", t.leg_at(v.at), v.at / ONE)
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
                bad.push(format!(
                    "{at}: beam in stone {s} at {}/20 (anchor {:?})",
                    k,
                    v.anchor.scale(100).scale(ONE / 100)
                ));
                break;
            }
        }
        if let Some(s) = t.in_stone(bottom) {
            bad.push(format!("{at}: an ape at its bottom is in stone {s}"));
        }
        // (2) the swing: out to 55° both ways along the course and 20°
        // either side of that (not for a beam hung near level, the sling
        // row: a swing along the course is along the beam itself there,
        // and the swing that matters, under the anchor into the tunnel,
        // is the mouth's width, TW2-4)
        if h.y.abs() < ONE / 2 {
            continue;
        }
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

/// The person's report (TW12): "why can you go through like all the
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
    fly_for(c, from, to, speed, crate::rules::TICKS_PER_SECOND as u32)
}

/// `fly` for `ticks`.
fn fly_for(c: &Course, from: V3, to: V3, speed: i32, ticks: u32) -> (V3, bool) {
    let mut a = Ape::new(Kind::Chimpanzee, true, from, 0);
    a.vel = to.sub(from).with_len(int(speed));
    a.mode = crate::ape::Mode::Flying;
    a.hint = c.nearest_anywhere(from);
    let mut hit = false;
    for tick in 0..ticks {
        a.step(&Default::default(), c, tick);
        hit |= a.happened.contains(&Happening::Wall);
        a.happened.clear();
    }
    (a.pos, hit)
}

fn flat_dist(a: V3, b: V3) -> i32 {
    a.sub(b).flat().len()
}

/// Flown at tower 2, at tower 1 from outside and at tower 1's inner wall
/// from inside, an ape is pushed back: it ends no nearer the wall than
/// its face (and its radius), with a `Wall` hit; flown at the tube's wall
/// from inside, the same.
#[test]
fn both_towers_and_the_tube_are_solid() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let [t1, t2] = &t.towers;
    let tube = &t.tube;
    let (end, hit) = fly(
        &c,
        v3(t2.centre.x + t2.radius + int(15), int(60), t2.centre.z),
        v3(t2.centre.x, int(60), t2.centre.z),
        25,
    );
    assert!(hit, "tower 2: no wall hit");
    assert!(flat_dist(end, t2.centre) >= t2.radius, "tower 2: went in");
    let y = t1.top() - int(40);
    let (end, hit) = fly(
        &c,
        v3(t1.centre.x, y, t1.centre.z + t1.radius + int(15)),
        v3(t1.centre.x, y, t1.centre.z),
        25,
    );
    assert!(hit, "tower 1 from outside: no wall hit");
    assert!(flat_dist(end, t1.centre) >= t1.radius, "tower 1: went in");
    let (end, hit) = fly(
        &c,
        v3(t1.centre.x, y, t1.centre.z + t1.inner - int(15)),
        v3(t1.centre.x, y, t1.centre.z + t1.inner),
        25,
    );
    assert!(hit, "tower 1 from inside: no wall hit");
    assert!(
        flat_dist(end, t1.centre) <= t1.inner,
        "tower 1 from inside: went into the wall"
    );
    let y = tube.top - int(100);
    let (end, hit) = fly(
        &c,
        v3(tube.axis.x, y, tube.axis.z),
        v3(tube.axis.x, y, tube.axis.z + tube.radius),
        25,
    );
    assert!(hit, "the tube: no wall hit");
    assert!(flat_dist(end, tube.axis) <= tube.radius, "the tube: went out");
}

/// Tower 1's doorway lets an ape through: flown in at it, it ends up
/// inside the hollow; flown at the wall beside it, it is stopped.
#[test]
fn the_doorway_lets_an_ape_in_and_the_wall_beside_it_does_not() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let d = &t.doorway;
    let t1 = &t.towers[0];
    let at = v3(d.at.x, d.at.y + int(5), d.at.z);
    let (end, hit) = fly_for(
        &c,
        at.add(d.out.scale(int(6))),
        at.sub(d.out.scale(int(12))),
        16,
        crate::rules::TICKS_PER_SECOND as u32,
    );
    assert!(!hit, "the doorway is open");
    assert!(
        flat_dist(end, t1.centre) < t1.inner,
        "flown in at the door it is inside tower 1"
    );
    let aside = right_of(d.out).scale(d.half_width + int(6));
    let (end, hit) = fly_for(
        &c,
        at.add(d.out.scale(int(6))).add(aside),
        at.sub(d.out.scale(int(12))).add(aside),
        16,
        crate::rules::TICKS_PER_SECOND as u32 / 2,
    );
    assert!(hit, "the wall beside the doorway stops it");
    assert!(flat_dist(end, t1.centre) >= t1.radius - int(1));
}

/// The tops are ground: an ape stands on tower 2's crown, and on a
/// bridge's deck at its drawn height at both ends (TL1).
#[test]
fn the_crown_and_the_bridge_decks_are_stood_on_where_drawn() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let t2 = &t.towers[1];
    let crown = v3(t2.centre.x, t2.top() + BODY_DROP, t2.centre.z);
    assert_eq!(c.support(crown), t2.top(), "the crown holds an ape");
    let br = &t.bridges[0];
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
