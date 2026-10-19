//! The AI on The Spindle as a Time Trial: King Kong alone, Speed
//! Boosts only, every kind, against the map's medal times (SECOND-ORDER SP7).

use tool_core::ai::{time_trial, Trial};
use tool_core::spindle::spindle;
use tool_core::map::Map;
use tool_core::rules::{Kind, TICKS_PER_SECOND};

const LIMIT: u32 = 150 * TICKS_PER_SECOND as u32;

fn secs(ticks: u32) -> f64 {
    ticks as f64 / TICKS_PER_SECOND as f64
}

fn every_kind(equal: bool) -> Vec<Trial> {
    let mut kinds: Vec<(Kind, bool)> = Kind::ALL.into_iter().map(|k| (k, false)).collect();
    if equal {
        kinds.push((Kind::Chimpanzee, true));
    }
    let runs: Vec<_> = kinds
        .into_iter()
        .map(|(k, eq)| std::thread::spawn(move || time_trial(spindle(), k, eq, LIMIT)))
        .collect();
    runs.into_iter().map(|r| r.join().unwrap()).collect()
}

/// The Spindle's medal report: every kind and Equal Stats.
#[test]
#[ignore]
fn spindle_time_trial_report() {
    println!("spindle time trial (last: equal stats)");
    for t in &every_kind(true) {
        println!(
            "  {:<11} finish {:>9}  grabs {:>3}  landings {:>2}  ground {:>5.1} s",
            format!("{:?}", t.kind),
            t.finish
                .map(|f| format!("{:.2} s", secs(f)))
                .unwrap_or("-".into()),
            t.grabs,
            t.landings,
            secs(t.ground_ticks),
        );
    }
    let medals = tool_core::trial::medals(Map::Spindle).map(secs);
    println!("medals bronze / silver / gold / author: {medals:.2?}");
}

/// The tube's medals follow its time trial: the best kind's run
/// (King Kong alone, Speed Boosts only) beats Gold, and the Author time
/// is just under it (inside 1.5 s).
#[test]
fn the_spindle_medals_follow_its_time_trial() {
    let best = every_kind(false)
        .into_iter()
        .filter_map(|t| t.finish)
        .min()
        .expect("some kind finishes");
    let target = |m| tool_core::trial::target_on(Map::Spindle, m);
    use tool_core::trial::Medal;
    assert!(
        best <= target(Medal::Gold),
        "best run {:.2} s misses Gold",
        secs(best)
    );
    let author = target(Medal::Author);
    assert!(
        author <= best && best - author <= 3 * TICKS_PER_SECOND as u32 / 2,
        "Author {:.2} s is not just under the best run {:.2} s",
        secs(author),
        secs(best)
    );
}
