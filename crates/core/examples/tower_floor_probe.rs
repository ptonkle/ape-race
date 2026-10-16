//! Drops an ape at a grid of points in and round the trampoline room, on
//! every stone top it can stand on (the keep's crown, the sub-towers'
//! roof steps, the room's roof, the bridges' decks), round the start deck
//! and the banana zone and on the ground under them, lets it fall for 4 s
//! with no input, and prints every spot where it ends up grounded with no
//! reset and no bounce: a place with no way on. For each, it then walks
//! the ape 5 s in each of eight directions and says whether any walk
//! finds a reset or a trampoline ("soft": a way on by walking) or none
//! does ("HARD": stuck).
//!
//!     cargo run --release --example tower_floor_probe [-- --all]
//!
//! `--all` lists every spot, not only the stuck ones.

use tool_core::ape::{Controls, Happening, Mode};
use tool_core::costume::Costume;
use tool_core::fx::{int, ONE, TURN};
use tool_core::geom::{v3, V3};
use tool_core::race::{Driver, Entry, Event, Phase, Race};
use tool_core::rules::{Kind, BODY_DROP, TICKS_PER_SECOND};
use tool_core::tower::{tower, Solid};

fn main() {
    let all = std::env::args().any(|a| a == "--all");
    let c = tower();
    let t = c.tower.as_deref().unwrap();
    // `--trace X Y Z`: drop an ape there and print its first 3 s tick by
    // tick (every 10th)
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--trace") {
        let f = |k: usize| (args[i + k].parse::<f64>().unwrap() * ONE as f64) as i32;
        let p = v3(f(1), f(2), f(3));
        let entries = [Entry {
            name: "P".into(),
            kind: Kind::Chimpanzee,
            driver: Driver::Local,
            palette: 0,
            costume: Costume::Own,
        }];
        let mut r = Race::new(c.clone(), &entries, false, 1);
        r.phase = Phase::Running;
        drop_at(&mut r, p);
        {
            let a = &r.racers[0].ape;
            let q = r.course.path[a.hint];
            println!(
                "start: pos ({:.1}, {:.1}, {:.1}) hint {} line ({:.1}, {:.1}, {:.1}) far {:.1} support {:.1} terrain {:.1} out {} in_stone {:?}",
                m(a.pos.x), m(a.pos.y), m(a.pos.z), a.hint, m(q.x), m(q.y), m(q.z), m(a.pos.sub(q).len()),
                m(r.course.support(a.pos)), m(r.course.terrain(a.pos.x, a.pos.z)), r.course.tower_out(a.pos, a.hint), t.in_stone(a.pos)
            );
        }
        for k in 0..3 * TICKS_PER_SECOND {
            r.step(&[Controls::default()]);
            let a = &r.racers[0].ape;
            let q = r.course.path[a.hint];
            if k % 10 == 0 || !r.events.is_empty() {
                println!(
                    "t {k}: pos ({:.1}, {:.1}, {:.1}) vel ({:.1}, {:.1}, {:.1}) {:?} hint {} line ({:.1}, {:.1}, {:.1}) support {:.1} out {} events {:?}",
                    m(a.pos.x), m(a.pos.y), m(a.pos.z), m(a.vel.x), m(a.vel.y), m(a.vel.z), a.mode, a.hint,
                    m(q.x), m(q.y), m(q.z), m(r.course.support(a.pos)), r.course.tower_out(a.pos, a.hint), r.events
                );
            }
        }
        return;
    }
    let mut spots: Vec<(String, V3)> = Vec::new();
    let room = &t.room;
    // (a) the room: a 2 m grid over its whole floor, dropped from 6 m up
    // and from 1.5 m up (under the trampolines' tops)
    for dy in [6, 1] {
        for dx in (-14..=14).step_by(2) {
            for dz in (-14..=14).step_by(2) {
                let p = v3(
                    room.centre.x + int(dx),
                    room.centre.y + int(dy) + ONE / 2,
                    room.centre.z + int(dz),
                );
                spots.push((format!("room (from {dy} m up)"), p));
            }
        }
    }
    // (b) every stone top: a grid over each drum's top and each span's deck
    for (n, s) in t.stone.iter().enumerate() {
        match *s {
            Solid::Drum {
                centre,
                inner,
                outer,
                top,
                ..
            } => {
                let step = (outer / ONE / 4).max(1) as usize;
                let r = outer / ONE;
                for dx in (-r..=r).step_by(step) {
                    for dz in (-r..=r).step_by(step) {
                        let rel = v3(int(dx), 0, int(dz));
                        if rel.longer_than(outer) || (inner > 0 && !rel.longer_than(inner)) {
                            continue;
                        }
                        spots.push((
                            format!("stone {n} drum top {:.0} m", m(top)),
                            v3(
                                centre.x + int(dx),
                                top + BODY_DROP + ONE,
                                centre.z + int(dz),
                            ),
                        ));
                    }
                }
            }
            Solid::Span { from, to, half, .. } => {
                for k in 0..=10 {
                    for side in [-half + ONE / 2, 0, half - ONE / 2] {
                        let run = to.sub(from);
                        let p = from.add(run.scale(k * ONE / 10));
                        let r = tool_core::geom::right_of(run.flat().norm());
                        let p = p.add(r.scale(side));
                        spots.push((
                            format!("stone {n} span deck"),
                            v3(p.x, p.y + BODY_DROP + ONE, p.z),
                        ));
                    }
                }
            }
            Solid::Pillar { .. } => {}
        }
    }
    // the keep's foot and the sub-towers' feet (as before)
    let k = &t.keep;
    for dx in (-60..=60).step_by(10) {
        for dz in (-60..=60).step_by(10) {
            spots.push((
                "keep foot".into(),
                v3(k.centre.x + int(dx), int(4), k.centre.z + int(dz)),
            ));
        }
    }
    for br in &t.branches {
        let s = br.sub;
        for dx in (-12..=12).step_by(6) {
            for dz in (-12..=12).step_by(6) {
                spots.push((
                    "sub foot".into(),
                    v3(
                        s.centre.x + int(dx),
                        s.centre.y + int(4),
                        s.centre.z + int(dz),
                    ),
                ));
            }
        }
    }
    // (d) the start deck and round it, the banana zone
    let start = &c.platforms[0];
    for dx in (-30..=30).step_by(5) {
        for dz in (-30..=30).step_by(5) {
            spots.push((
                "start deck surroundings".into(),
                v3(
                    start.centre.x + int(dx),
                    start.top + BODY_DROP + int(2),
                    start.centre.z + int(dz),
                ),
            ));
            // (e) the ground under the start deck
            spots.push((
                "ground under start".into(),
                v3(start.centre.x + int(dx), int(3), start.centre.z + int(dz)),
            ));
        }
    }
    let bz = &t.banana_zone;
    for dx in (-60..=60).step_by(10) {
        for dz in (-40..=40).step_by(10) {
            spots.push((
                "banana zone".into(),
                v3(
                    bz.centre.x + int(dx),
                    bz.centre.y + BODY_DROP + int(2),
                    bz.centre.z + int(dz),
                ),
            ));
        }
    }
    // (e) the ground under the room
    for dx in (-20..=20).step_by(5) {
        for dz in (-20..=20).step_by(5) {
            spots.push((
                "ground under room".into(),
                v3(room.centre.x + int(dx), int(3), room.centre.z + int(dz)),
            ));
        }
    }
    let entries = [Entry {
        name: "P".into(),
        kind: Kind::Chimpanzee,
        driver: Driver::Local,
        palette: 0,
        costume: Costume::Own,
    }];
    let mut hard = 0;
    let mut soft = 0;
    let mut n = 0;
    for (what, p) in &spots {
        let mut r = Race::new(c.clone(), &entries, false, 1);
        r.phase = Phase::Running;
        drop_at(&mut r, *p);
        let (resets, bounces) = run(&mut r, Controls::default(), 4 * TICKS_PER_SECOND);
        let a = &r.racers[0].ape;
        let idle_stuck = resets == 0 && bounces == 0 && a.mode == Mode::Grounded;
        if !idle_stuck && !all {
            continue;
        }
        n += 1;
        let rest = a.pos;
        let feet = rest.y - BODY_DROP;
        let ground = r.course.terrain(rest.x, rest.z);
        let support = r.course.support(rest);
        // walking out: 5 s in each of eight directions from the rest
        let mut ways = Vec::new();
        for dir in 0..8 {
            let mut r2 = r.clone();
            let ctl = Controls {
                fwd: 1,
                yaw: dir * TURN / 8,
                ..Controls::default()
            };
            let (rs, bs) = run(&mut r2, ctl, 5 * TICKS_PER_SECOND);
            if rs > 0 || bs > 0 {
                ways.push(format!(
                    "{}{}",
                    dir * 45,
                    if rs > 0 { " reset" } else { " bounce" }
                ));
            }
        }
        let tag = if !idle_stuck {
            "ok"
        } else if ways.is_empty() {
            hard += 1;
            "HARD"
        } else {
            soft += 1;
            "soft"
        };
        println!(
            "{tag} {what}: dropped at ({:.1}, {:.1}, {:.1}) rests at ({:.1}, {:.1}, {:.1}) feet {:.2} ground {:.2} support {:.2} in_stone {:?} out {} walks: {}",
            m(p.x), m(p.y), m(p.z), m(rest.x), m(rest.y), m(rest.z), m(feet), m(ground), m(support),
            t.in_stone(rest), r.course.tower_out(rest, r.racers[0].ape.hint),
            if ways.is_empty() { "none".into() } else { ways.join(", ") }
        );
    }
    println!(
        "{} spots: {hard} stuck with no way out by walking, {soft} with one (listed {n})",
        spots.len()
    );
}

fn drop_at(r: &mut Race, p: V3) {
    let a = &mut r.racers[0].ape;
    a.pos = p;
    a.vel = V3::ZERO;
    a.mode = Mode::Flying;
    a.hint = r.course.nearest_anywhere(p);
}

/// Steps `ticks` with `ctl` held; how many resets and bounces happened.
fn run(r: &mut Race, ctl: Controls, ticks: i32) -> (usize, usize) {
    let (mut resets, mut bounces) = (0, 0);
    for _ in 0..ticks {
        r.step(&[ctl]);
        for e in &r.events {
            match e {
                Event::Reset { .. } => resets += 1,
                Event::Body {
                    what: Happening::Bounce,
                    ..
                } => bounces += 1,
                _ => {}
            }
        }
    }
    (resets, bounces)
}

fn m(v: i32) -> f64 {
    v as f64 / ONE as f64
}
