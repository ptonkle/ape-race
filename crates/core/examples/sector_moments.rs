//! When the moments worth a picture happen in a tile's race on Sector Ape
//! (`?tile=map=sector,...`, `app/tile.rs`): the same four AI apes from the
//! same seed, stepped as the tile steps them, and for each ape the first
//! tick it swings on a sideways vine, on one hanging up, is in the Hulk's
//! corridor, passes the wreck, is in the armada, and finishes. Shoot a
//! tile at that tick with `focus=` the ape.
//!
//! `cargo run --release -p tool_core --example sector_moments [-- SEED [TICKS]]`
use tool_core::ai::{Brain, Difficulty};
use tool_core::ape::Mode;
use tool_core::fx::{self, ONE};
use tool_core::map::Map;
use tool_core::race::{Driver, Entry, Race};
use tool_core::rules::Kind;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let seed: u64 = args.first().and_then(|a| a.parse().ok()).unwrap_or(1);
    let ticks: u32 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(14_000);
    // the tile's four kinds from its seed (`Tile::kinds`)
    let mut d = fx::Dice(seed);
    let mut pool = Kind::ALL.to_vec();
    let kinds: Vec<Kind> = (0..4)
        .map(|_| pool.remove(d.below(pool.len() as u32) as usize))
        .collect();
    let entries: Vec<Entry> = kinds
        .iter()
        .map(|&kind| Entry {
            name: String::new(),
            kind,
            driver: Driver::Ai,
            palette: 0,
        })
        .collect();
    let course = Map::Sector.course();
    let mut race = Race::new(course, &entries, false, seed);
    race.ai_level = Difficulty::default();
    let mut brains: Vec<Brain> = (0..race.racers.len())
        .map(|i| Brain::new(race.ai_level, race.dice.0 ^ (0x9E37_79B9 * (i as u64 + 1))))
        .collect();
    let names = [
        "sideways vine",
        "up vine",
        "slanted vine",
        "in the corridor",
        "at the wreck",
        "in the armada",
        "finished",
    ];
    let mut seen = vec![[None::<u32>; 7]; race.racers.len()];
    for _ in 0..ticks {
        tool_core::replay::drive(&mut race, &mut brains, 0, Default::default());
        let c = &race.course;
        for (k, r) in race.racers.iter().enumerate() {
            let p = r.progress / ONE;
            let (_, side) = c.progress(r.ape.pos, r.ape.hint);
            let mut mark = |m: usize| {
                if seen[k][m].is_none() {
                    seen[k][m] = Some(race.tick);
                }
            };
            if let Mode::Swinging { vine, .. } = r.ape.mode {
                let h = c.hang(vine);
                if h.y.abs() < ONE / 4 {
                    mark(0);
                } else if h.y > ONE * 9 / 10 {
                    mark(1);
                } else if c.odd_vine(vine) {
                    mark(2);
                }
            }
            if (520..680).contains(&p) && side.abs() < 15 * ONE && r.ape.pos.y.abs() < 18 * ONE {
                mark(3);
            }
            if (1180..1260).contains(&p) {
                mark(4);
            }
            if (2060..2260).contains(&p) {
                mark(5);
            }
            if r.finish.is_some() {
                mark(6);
            }
        }
    }
    for (k, s) in seen.iter().enumerate() {
        println!("focus={k} ({:?})", kinds[k]);
        for (m, name) in names.iter().enumerate() {
            match s[m] {
                Some(t) => println!("  {name}: at={t}"),
                None => println!("  {name}: -"),
            }
        }
    }
}
