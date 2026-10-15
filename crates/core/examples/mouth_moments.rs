//! When each ape of a Sunset Ruins tile race passes the temple mouth
//! (for the entrance shots): prints each ape's progress and mode near the
//! mouth every 1/8 s.
//!
//!     cargo run --release -p tool_core --example mouth_moments -- "map=ruins,seed=1"
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
    let mouth = p.race.course.ruins.as_ref().unwrap().marks.mouth / ONE;
    println!("mouth at {mouth} m");
    let q = TICKS_PER_SECOND as u32 / 8;
    while p.race.tick < TICKS_PER_SECOND as u32 * 150 {
        replay::drive(&mut p.race, &mut p.brains, p.local, Controls::default());
        let t = p.race.tick;
        if !t.is_multiple_of(q) {
            continue;
        }
        for (i, r) in p.race.racers.iter().enumerate() {
            let s = r.progress / ONE;
            if (s - mouth).abs() < 60 {
                let m = match r.ape.mode {
                    Mode::Swinging { .. } => "S",
                    Mode::Flying => "F",
                    _ => "G",
                };
                println!("{t:>6} ape {i} {s}m {m}");
            }
        }
        if p.race.racers.iter().all(|r| r.progress / ONE > mouth + 60) {
            break;
        }
    }
}
