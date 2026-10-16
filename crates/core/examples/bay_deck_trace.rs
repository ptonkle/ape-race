//! One AI ape put on a Banana Bay Battle ship's reset spot (as the water
//! puts it back), traced every half second for 12 s: progress, side, mode
//! and height. For `every_level_gets_off_every_deck_back_onto_the_line`.
//!
//! `cargo run --release -p tool_core --example bay_deck_trace -- LEVEL SHIP`
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::bay::bay;
use tool_core::fx::ONE;
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let level = Difficulty::ALL
        .into_iter()
        .find(|d| d.key() == args[1])
        .expect("a level");
    let k: usize = args[2].parse().unwrap();
    let course = bay();
    let b = course.bay.as_deref().unwrap().clone();
    let entries = [Entry {
        name: String::new(),
        kind: Kind::Chimpanzee,
        driver: Driver::Ai,
        palette: 0,
        costume: tool_core::costume::Costume::Own,
    }];
    let mut race = Race::new(course.clone(), &entries, true, 3);
    while race.phase != Phase::Running {
        race.step(&[Default::default()]);
    }
    let s = &b.ships[k];
    let spot = s.reset_spot().expect("a deck");
    race.racers[0].ape.put_on(spot, s.yaw, &race.course);
    let mut brain = Brain::new(level, 0xA9E5);
    let start = race.tick;
    while race.tick < start + 12 * TICKS_PER_SECOND as u32 {
        let c = think(&race, 0, &mut brain);
        race.step(&[c]);
        if (race.tick - start).is_multiple_of(TICKS_PER_SECOND as u32 / 2) {
            let a = &race.racers[0].ape;
            let (along, side) = race.course.progress(a.pos, a.hint);
            println!(
                "{:5.1}s {:6.1} m side {:+6.1} y {:5.1} {:?}",
                (race.tick - start) as f64 / TICKS_PER_SECOND as f64,
                along as f64 / ONE as f64,
                side as f64 / ONE as f64,
                a.pos.y as f64 / ONE as f64,
                a.mode
            );
        }
    }
}
