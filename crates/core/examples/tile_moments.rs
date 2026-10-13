//! The moments of a tile's race (`scripts/tiles.py`), for choosing shots:
//! runs the all-AI race a tile spec describes and prints its events
//! (bounces, items, hits, finishes) and, every second, each ape's place.
//!
//!     cargo run --release -p tool_core --example tile_moments -- "map=jungle,seed=3"
use tool_core::ape::{Controls, Happening, Mode};
use tool_core::app::App;
use tool_core::fx::{int, ONE};
use tool_core::race::Event;
use tool_core::replay;
use tool_core::rules::TICKS_PER_SECOND;
use tool_core::speedo;

fn main() {
    let spec = std::env::args().nth(1).unwrap_or_default();
    let mut a = App::new(include_str!("../../../data/strings.json"), "", 1).unwrap();
    a.act("screen.tile", &format!("{spec},at=0"));
    let p = a.play.as_mut().unwrap();
    let names: Vec<String> = p
        .race
        .racers
        .iter()
        .map(|r| format!("{:?}", r.ape.kind))
        .collect();
    println!("apes: {names:?}");
    let every = TICKS_PER_SECOND as u32;
    while p.race.racers.iter().any(|r| !r.out()) && p.race.tick < every * 150 {
        replay::drive(&mut p.race, &mut p.brains, p.local, Controls::default());
        let t = p.race.tick;
        for e in &p.race.events {
            let line = match e {
                Event::Body {
                    who,
                    what: Happening::Bounce,
                } => format!("bounce {who}"),
                Event::Body {
                    who,
                    what: Happening::Knocked,
                } => format!("knocked {who}"),
                Event::Used { who, item } => format!("used {who} {item:?}"),
                Event::Hit { who, by, item } => format!("hit {who} by {by} {item:?}"),
                Event::Finish { who, place } => format!("finish {who} place {place}"),
                Event::Eaten { who, .. } => format!("eaten {who}"),
                _ => continue,
            };
            println!("{t:>6} {line}");
        }
        // two apes in the air within 6 m: a crowded shot
        if t.is_multiple_of(every / 4) {
            let r = &p.race.racers;
            for i in 0..r.len() {
                for j in i + 1..r.len() {
                    let d = r[i].ape.pos.dist(r[j].ape.pos);
                    let up = |k: usize| r[k].ape.mode != Mode::Grounded;
                    if d < int(6) && up(i) && up(j) && !r[i].out() {
                        println!("{t:>6} close {i} {j} {}dm", d * 10 / ONE);
                    }
                }
            }
        }
        if t.is_multiple_of(every) {
            let row: Vec<String> = p
                .race
                .racers
                .iter()
                .map(|r| {
                    let m = match r.ape.mode {
                        Mode::Swinging { .. } => "S",
                        Mode::Flying => "F",
                        _ => "G",
                    };
                    let kmh = speedo::kmh(r.ape.vel.len_xz());
                    format!("{:>5}m {m} {kmh:>3}", r.progress / ONE)
                })
                .collect();
            let ps: Vec<i32> = p.race.racers.iter().map(|r| r.progress / ONE).collect();
            let spread = ps.iter().max().unwrap() - ps.iter().min().unwrap();
            println!("{t:>6} | {} | spread {spread}", row.join(" | "));
        }
    }
}
