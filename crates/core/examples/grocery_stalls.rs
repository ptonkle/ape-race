//! Where the AI stalls on Aisle Ape: every level over N
//! layouts, each stall (3 s with under 25 m gained) by where it began, in
//! 50 m bins, with the time lost; and the finish times.
//!
//! `cargo run --release -p tool_core --example grocery_stalls -- [LAYOUTS [APES]]`
//! (`APES`: a race of that many, 1 to 4, each at its own item spot; every
//! ape's stalls are counted, its finish shown)
use tool_core::ai::{think, Brain, Difficulty};
use tool_core::fx::ONE;
use tool_core::grocery::{grocery, grocery_seeded};
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn run(level: Difficulty, k: u64, apes: usize) -> (Vec<Option<f64>>, Vec<(i32, f64)>) {
    let course = match k {
        0 => grocery(),
        _ => grocery_seeded(0x6E0C_E2E5_0001 + k * 7919),
    };
    let entries: Vec<Entry> = (0..apes)
        .map(|_| Entry {
            name: String::new(),
            kind: Kind::Chimpanzee,
            driver: Driver::Ai,
            palette: 0,
            costume: tool_core::costume::Costume::Own,
        })
        .collect();
    let mut race = Race::new(course, &entries, true, 7);
    let mut brains: Vec<Brain> = (0..apes)
        .map(|i| Brain::new(level, 0xA9E5 + i as u64))
        .collect();
    let half = TICKS_PER_SECOND as u32 / 2;
    let mut hist: Vec<Vec<i32>> = vec![Vec::new(); apes];
    let mut stalls: Vec<(i32, f64)> = Vec::new();
    let mut open: Vec<Option<(i32, u32)>> = vec![None; apes];
    while race.clock < 180 * TICKS_PER_SECOND as u32 && race.phase != Phase::Done {
        let cs: Vec<_> = (0..apes).map(|i| think(&race, i, &mut brains[i])).collect();
        race.step(&cs);
        if race.clock.is_multiple_of(half) && race.running() {
            for i in 0..apes {
                if race.racers[i].finish.is_some() {
                    continue;
                }
                let p = race.racers[i].progress / ONE;
                let h = &mut hist[i];
                h.push(p);
                let n = h.len();
                let stalled = n > 6 && p - h[n - 7] < 25;
                match (stalled, open[i]) {
                    (true, None) => open[i] = Some((h[n - 7], race.clock)),
                    (false, Some((at, from))) => {
                        stalls.push((
                            at,
                            (race.clock - from) as f64 / TICKS_PER_SECOND as f64 + 3.0,
                        ));
                        open[i] = None;
                    }
                    _ => {}
                }
            }
        }
    }
    for (at, from) in open.into_iter().flatten() {
        stalls.push((
            at,
            (race.clock - from) as f64 / TICKS_PER_SECOND as f64 + 3.0,
        ));
    }
    let fin = race
        .racers
        .iter()
        .map(|r| r.finish.map(|f| f as f64 / TICKS_PER_SECOND as f64))
        .collect();
    (fin, stalls)
}

fn main() {
    let n: u64 = std::env::args()
        .nth(1)
        .map(|a| a.parse().unwrap())
        .unwrap_or(6);
    let apes: usize = std::env::args()
        .nth(2)
        .map(|a| a.parse().unwrap())
        .unwrap_or(1);
    let jobs: Vec<_> = Difficulty::ALL
        .into_iter()
        .flat_map(|l| (0..n).map(move |k| (l, k)))
        .map(|(l, k)| std::thread::spawn(move || (l, k, run(l, k, apes))))
        .collect();
    let mut bins = std::collections::BTreeMap::new();
    let mut rows: Vec<String> = Vec::new();
    for j in jobs {
        let (l, k, (fin, stalls)) = j.join().unwrap();
        rows.push(format!(
            "{:<10} layout {k}: {}",
            l.key(),
            fin.iter()
                .map(|f| f.map(|f| format!("{f:.1} s")).unwrap_or("DNF".into()))
                .collect::<Vec<_>>()
                .join("  ")
        ));
        for (at, lost) in stalls {
            let e = bins.entry(at / 50 * 50).or_insert((0, 0.0, String::new()));
            e.0 += 1;
            e.1 += lost;
            e.2 += &format!("{}{} ", &l.key()[..1], k);
        }
    }
    rows.sort();
    for r in rows {
        println!("{r}");
    }
    println!("stalls by where they began (50 m bins): count, seconds lost, who");
    for (at, (c, s, who)) in bins {
        println!("  {at:>5} m  {c:>3}  {s:>6.1} s  {who}");
    }
}
