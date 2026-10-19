//! The tower map: each level over layouts 0..N (default 16), the second it
//! reached each leg and its finish, as `tests/tower_ai.rs`'s races (one
//! Chimpanzee, Equal Stats, limit 200 s).
//! `cargo run --release -p tool_core --example tower_legs -- [N [LEVEL]]`
//! (with `TOWER_LEGS_SEED_K=1` each layout's brain seed is offset by the
//! layout, as `every_level_flies_through_the_tear`'s races)
use tool_core::ai::{Brain, Difficulty};
use tool_core::race::{Driver, Entry, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};
use tool_core::tower::{tower, tower_seeded, Leg};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let n: u64 = args.first().map(|a| a.parse().unwrap()).unwrap_or(16);
    let only = args.get(1).and_then(|k| Difficulty::from_key(k));
    let limit = 200 * TICKS_PER_SECOND as u32;
    let seed_k = std::env::var("TOWER_LEGS_SEED_K").is_ok();
    let runs: Vec<_> = Difficulty::ALL
        .into_iter()
        .filter(|l| only.is_none_or(|o| o == *l))
        .flat_map(|level| (0..n).map(move |k| (level, k)))
        .map(|(level, k)| {
            std::thread::spawn(move || {
                let c = if k == 0 {
                    tower()
                } else {
                    tower_seeded(0x70E5_0000_0001 + k * 7919)
                };
                let entry = Entry {
                    name: String::new(),
                    kind: Kind::Chimpanzee,
                    driver: Driver::Ai,
                    palette: 0,
                    costume: tool_core::costume::Costume::Own,
                };
                let mut race = Race::new(c, &[entry], true, 7);
                let mut brains = vec![Brain::new(level, 0xA9E5 + if seed_k { k } else { 0 })];
                let mut legs: Vec<(Leg, u32)> = Vec::new();
                while race.tick < limit && race.racers[0].finish.is_none() {
                    tool_core::replay::drive(
                        &mut race,
                        &mut brains,
                        usize::MAX,
                        Default::default(),
                    );
                    let leg = race
                        .course
                        .tower
                        .as_ref()
                        .unwrap()
                        .leg_at(race.racers[0].progress);
                    if legs.last().is_none_or(|l| l.0 < leg) {
                        legs.push((leg, race.clock));
                    }
                }
                (level, k, race.racers[0].finish, legs)
            })
        })
        .collect();
    let s = |t: u32| t as f64 / TICKS_PER_SECOND as f64;
    for r in runs {
        let (level, k, finish, legs) = r.join().unwrap();
        let mut row = String::new();
        for w in legs.windows(2) {
            row += &format!(" {:?} {:.0}", w[0].0, s(w[1].1 - w[0].1));
        }
        if let Some(l) = legs.last() {
            row += &format!(" {:?}", l.0);
        }
        println!(
            "{:<9} {k:2} {:>6}  {row}",
            level.key(),
            finish.map_or("-".into(), |f| format!("{:.1}", s(f)))
        );
    }
}
