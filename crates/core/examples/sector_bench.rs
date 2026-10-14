//! Micro-timings of Sector Ape's per-tick costs: a mech's anchor, a flying
//! ape's step, the body hits, the support and the boundary test.
//!
//! `cargo run --release -p tool_core --example sector_bench`
use std::time::Instant;
use tool_core::ape::{Ape, Controls, Mode};
use tool_core::fx::{int, ONE};
use tool_core::geom::v3;
use tool_core::rules::Kind;
use tool_core::sector::{sector, MechPoint};

fn main() {
    let c = sector();
    let s = c.sector.as_ref().unwrap();
    let n = 200_000u32;
    let t = Instant::now();
    let mut acc = 0i64;
    for k in 0..n {
        acc += s.mechs[(k % 12) as usize].point(MechPoint::Hand(0), k).x as i64;
    }
    println!("mech point: {:?} each ({acc})", t.elapsed() / n);
    for (label, at) in [("debris", 300), ("belt", 1800), ("armada", 2150)] {
        let p = c.point(int(at));
        let mut a = Ape::new_on(&c, Kind::Chimpanzee, true, v3(p.x, int(5), p.z), 0);
        a.mode = Mode::Flying;
        a.vel = v3(0, 0, int(40));
        a.hint = c.nearest_anywhere(a.pos);
        let t = Instant::now();
        let steps = 20_000u32;
        for k in 0..steps {
            let mut b = a.clone();
            b.step(&Controls::default(), &c, k);
            acc += b.pos.x as i64;
        }
        println!("{label}: a flying step {:?}", t.elapsed() / steps);
        let t = Instant::now();
        for _ in 0..steps {
            acc += c.support(a.pos) as i64;
        }
        println!("{label}: support {:?}", t.elapsed() / steps);
        let t = Instant::now();
        for _ in 0..steps {
            acc += c.out_of_bounds(a.pos, a.hint) as i64;
        }
        println!("{label}: out of bounds {:?}", t.elapsed() / steps);
        let t = Instant::now();
        for _ in 0..steps {
            acc += c.nearest(a.pos, a.hint) as i64;
        }
        println!(
            "{label}: nearest {:?} ({} bodies near)",
            t.elapsed() / steps,
            s.bodies_near(a.hint).len()
        );
    }
    println!("{}", acc % ONE as i64);
}
