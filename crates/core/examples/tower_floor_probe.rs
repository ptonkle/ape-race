//! Drops an ape at a grid of points in and round the trampoline room, the
//! keep's foot and each sub-tower's foot, lets it fall for 8 s with no
//! input, and prints every spot where it ends up grounded and still with
//! no reset and no trampoline under it: a place with no way on.
//!
//!     cargo run --release --example tower_floor_probe

use tool_core::ape::Mode;
use tool_core::costume::Costume;
use tool_core::fx::{int, ONE};
use tool_core::geom::{v3, V3};
use tool_core::race::{Driver, Entry, Event, Phase, Race};
use tool_core::rules::{Kind, BODY_DROP, TICKS_PER_SECOND};
use tool_core::tower::tower;

fn main() {
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    let mut spots: Vec<(&str, V3)> = Vec::new();
    let room = &t.room;
    for dx in (-16..=16).step_by(4) {
        for dz in (-16..=16).step_by(4) {
            let p = v3(
                room.centre.x + int(dx),
                room.centre.y + int(6),
                room.centre.z + int(dz),
            );
            spots.push(("room", p));
        }
    }
    let k = &t.keep;
    for dx in (-60..=60).step_by(10) {
        for dz in (-60..=60).step_by(10) {
            spots.push((
                "keep foot",
                v3(k.centre.x + int(dx), int(4), k.centre.z + int(dz)),
            ));
        }
    }
    for br in &t.branches {
        let s = br.sub;
        for dx in (-12..=12).step_by(6) {
            for dz in (-12..=12).step_by(6) {
                spots.push((
                    "sub foot",
                    v3(
                        s.centre.x + int(dx),
                        s.centre.y + int(4),
                        s.centre.z + int(dz),
                    ),
                ));
            }
        }
    }
    let entries = [Entry {
        name: "P".into(),
        kind: Kind::Chimpanzee,
        driver: Driver::Local,
        palette: 0,
        costume: Costume::Own,
    }];
    let mut stuck = 0;
    for (what, p) in spots {
        let mut r = Race::new(c.clone(), &entries, false, 1);
        r.phase = Phase::Running;
        let a = &mut r.racers[0].ape;
        a.pos = p;
        a.vel = V3::ZERO;
        a.mode = Mode::Flying;
        a.hint = r.course.nearest_anywhere(p);
        let mut resets = 0;
        let mut still = 0;
        for _ in 0..8 * TICKS_PER_SECOND {
            r.step(&[Default::default()]);
            resets += r
                .events
                .iter()
                .filter(|e| matches!(e, Event::Reset { .. }))
                .count();
            let a = &r.racers[0].ape;
            if a.vel.len() < ONE / 10 && a.mode == Mode::Grounded {
                still += 1;
            } else {
                still = 0;
            }
        }
        let a = &r.racers[0].ape;
        let under = r.course.bouncer_under(a.pos, r.tick).is_some();
        if resets == 0 && still > 2 * TICKS_PER_SECOND && !under {
            stuck += 1;
            let feet = a.pos.y - BODY_DROP;
            let ground = r.course.terrain(a.pos.x, a.pos.z);
            let support = r.course.support(a.pos);
            let stone = t.in_stone(a.pos);
            println!(
                "STUCK {what}: dropped at ({:.1}, {:.1}, {:.1}) ends at ({:.1}, {:.1}, {:.1}) feet {:.2} ground {:.2} support {:.2} in_stone {stone:?} out {}",
                m(p.x), m(p.y), m(p.z), m(a.pos.x), m(a.pos.y), m(a.pos.z), m(feet), m(ground), m(support),
                r.course.tower_out(a.pos, a.hint)
            );
        }
    }
    println!("{stuck} stuck spots");
}

fn m(v: i32) -> f64 {
    v as f64 / ONE as f64
}
