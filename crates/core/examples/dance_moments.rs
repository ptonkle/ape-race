//! After the finish (TRIAGE T53): runs a tile's all-AI race and prints,
//! every quarter second after each ape's finish, its mode, speed and
//! distance past the line, for choosing dance shots (`?tile=...,at=`).
//!
//!     cargo run --release -p tool_core --example dance_moments -- "map=jungle,apes=gorilla+lemur"
use tool_core::ape::{Controls, Mode};
use tool_core::app::App;
use tool_core::fx::ONE;
use tool_core::replay;
use tool_core::rules::TICKS_PER_SECOND;

fn main() {
    let spec = std::env::args().nth(1).unwrap_or_default();
    let mut a = App::new(include_str!("../../../data/strings.json"), "", 1).unwrap();
    a.act("screen.tile", &format!("{spec},at=0"));
    let p = a.play.as_mut().unwrap();
    let every = TICKS_PER_SECOND as u32;
    let mut last = 0;
    while p.race.tick < every * 200 {
        replay::drive(&mut p.race, &mut p.brains, p.local, Controls::default());
        let t = p.race.tick;
        if last == 0 && p.race.racers.iter().all(|r| r.out()) {
            last = t + every * 8;
        }
        if last != 0 && t > last {
            break;
        }
        if !t.is_multiple_of(every / 4) {
            continue;
        }
        for (k, r) in p.race.racers.iter().enumerate() {
            let Some(since) = p.race.since_finish(k) else {
                continue;
            };
            if since > every * 8 {
                continue;
            }
            let m = match r.ape.mode {
                Mode::Swinging { .. } => "S",
                Mode::Flying => "F",
                _ => "G",
            };
            println!(
                "{t:>6} {k} {:<10} +{since:>4} {m} {:>5.1} m/s {:>6.1} m past",
                format!("{:?}", r.ape.kind),
                r.ape.vel.len() as f64 / ONE as f64,
                (r.progress - p.race.course.length) as f64 / ONE as f64
            );
        }
    }
}
