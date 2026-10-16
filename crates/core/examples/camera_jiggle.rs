//! How much the chase camera jiggles (SECOND-ORDER BX1): one AI ape raced
//! with the camera on it, as `App::begin` sets it up, and each tick the
//! eye's jump against the ape's own move (|Δeye − Δtarget|) and the turn of
//! the view's direction. Prints the mean, 99th percentile and largest of
//! each, and how many ticks jump over 50 cm or turn over 3 degrees.
//!
//! `cargo run --release -p tool_core --example camera_jiggle -- MAP LEVEL [SEEDS]`
//! (MAP a map key; on `bay` SEEDS layouts: 0 the fixed bay, then seeded ones)
use std::sync::Arc;
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::camera::{Chase, Hulls, Temple};
use tool_core::fx::ONE;
use tool_core::map::Map;
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let map = Map::from_key(&args[0]).expect("a map key");
    let level = Difficulty::from_key(&args[1]).expect("a level key");
    let seeds: u64 = args.get(2).map(|a| a.parse().unwrap()).unwrap_or(1);
    let (mut jumps, mut turns) = (Vec::new(), Vec::new());
    for k in 0..seeds {
        let course = if map == Map::Bay && k > 0 {
            tool_core::bay::bay_seeded(0xBA7_BA77_0001 + k * 7919)
        } else {
            map.course()
        };
        let entries = [Entry {
            name: String::new(),
            kind: Kind::Chimpanzee,
            driver: Driver::Ai,
            palette: 0,
            costume: tool_core::costume::Costume::Own,
        }];
        let mut race = Race::new(course, &entries, true, 7 + k);
        let mut brain = Brain::new(level, 0xA9E5 + k * 101);
        let at = race.racers[0].ape.pos;
        let mut chase = Chase::new(at, race.course.start_yaw, race.course.terrain);
        if let Some(ski) = &race.course.ski {
            let crowns = tool_core::skiscape::crowns(&race.course, ski);
            chase = chase.with_crowns(at, crowns.into());
        }
        if let Some(r) = &race.course.ruins {
            chase = chase.with_temple(at, Some(Arc::new(Temple::of(r))));
        }
        if let Some(h) = Hulls::of(&race.course) {
            chase = chase.with_hulls(at, Some(Arc::new(h)));
        }
        if race.course.sector.is_some() {
            chase.pace = tool_core::camera::SPACE_PACE;
        }
        let last = race.course.heading.len() - 1;
        while race.clock < 150 * TICKS_PER_SECOND as u32 && race.phase != Phase::Done {
            let c = think(&race, 0, &mut brain);
            race.step(&[c]);
            let a = &race.racers[0].ape;
            let (pe, pt) = (chase.eye, chase.target);
            let pf = pt.sub(pe).norm();
            let ahead = race.course.heading[(a.hint + 8).min(last)];
            chase.follow(a.pos, a.vel, ahead, true);
            if !race.running() || race.racers[0].out() {
                continue;
            }
            let moved = chase.target.sub(pt);
            // a reset puts the ape somewhere else: not a jiggle
            if moved.len() > ONE * 3 {
                continue;
            }
            let jump = chase.eye.sub(pe).sub(moved).len() as f64 / ONE as f64;
            let f = chase.target.sub(chase.eye).norm();
            let dot = (f.dot(pf) as f64 / ONE as f64).clamp(-1.0, 1.0);
            jumps.push(jump);
            turns.push(dot.acos().to_degrees());
        }
    }
    let report = |name: &str, v: &mut Vec<f64>, over: f64| {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = v.len().max(1);
        let mean = v.iter().sum::<f64>() / n as f64;
        let p99 = v[(n * 99 / 100).min(v.len() - 1)];
        let max = v.last().copied().unwrap_or(0.0);
        let big = v.iter().filter(|x| **x > over).count();
        println!("{name}: mean {mean:.3} p99 {p99:.3} max {max:.2} over {over}: {big} of {n}");
    };
    println!("{} {} seeds {seeds}", args[0], args[1]);
    report("eye jump m", &mut jumps, 0.5);
    report("view turn deg", &mut turns, 3.0);
}
