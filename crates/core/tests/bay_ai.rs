//! The AI on Banana Bay Battle (2026-10-15): every difficulty finishes it,
//! from the water and the decks back onto the line, and no ape ever passes
//! through a set sail. (The ordering test and medal times come with the
//! tuning: SECOND-ORDER BC rows.)

use tool_core::ai::{think, trial_at, Brain, Difficulty};
use tool_core::bay::{bay, bay_seeded, BayLayout, Sail};
use tool_core::course::Course;
use tool_core::fx::ONE;
use tool_core::geom::V3;
use tool_core::race::{Driver, Entry, Event, Phase, Race};
use tool_core::rules::{Kind, BODY_DROP, BODY_RADIUS, TICKS_PER_SECOND};

const LIMIT: u32 = 150 * TICKS_PER_SECOND as u32;

/// Layout `k` of Banana Bay Battle (0 is the course everyone races; the
/// seeds are `examples/bay_report.rs`'s).
fn layout(k: u64) -> Course {
    if k == 0 {
        bay()
    } else {
        bay_seeded(0xBA7_BA77_0001 + k * 7919)
    }
}

/// Every level finishes four layouts inside the limit (150 s), as the
/// Chimpanzee (Equal Stats).
#[test]
fn every_level_finishes_four_bay_layouts() {
    let runs: Vec<_> = Difficulty::ALL
        .into_iter()
        .map(|level| {
            std::thread::spawn(move || {
                let mut stuck = Vec::new();
                for k in 0..4u64 {
                    let t = trial_at(layout(k), &[Kind::Chimpanzee], &[level], true, LIMIT);
                    if t[0].finish.is_none() {
                        stuck.push(format!(
                            "{} layout {k} stopped at {} m",
                            level.key(),
                            t[0].progress / ONE
                        ));
                    }
                }
                stuck
            })
        })
        .collect();
    let stuck: Vec<String> = runs
        .into_iter()
        .flat_map(|r| r.join().expect("the races run"))
        .collect();
    assert!(stuck.is_empty(), "not finished in 150 s: {stuck:?}");
}

/// Whether a body at `p` (the hands) is in a set square sail (from its
/// yard down to the next yard, or the deck), as thick as a body.
fn in_a_set_sail(b: &BayLayout, p: V3) -> bool {
    b.ships.iter().any(|s| {
        let d = p.sub(s.at);
        let along = d.dot(s.fwd());
        let across = d.dot(s.right());
        s.masts.iter().any(|m| {
            (along - m.along).abs() <= BODY_RADIUS
                && m.yards.iter().enumerate().any(|(y, yard)| {
                    let low = if y == 0 { 0 } else { m.yards[y - 1].height };
                    yard.sail == Sail::Set
                        && across.abs() <= yard.half_span + BODY_RADIUS
                        && p.y >= s.deck + low - BODY_RADIUS
                        && p.y - BODY_DROP <= s.deck + yard.height
                })
        })
    })
}

/// Four AI apes (one at each level, items on) race Banana Bay Battle: all
/// finish, and none passes through a set sail on the way.
#[test]
fn four_ai_apes_race_the_bay_and_none_passes_through_a_set_sail() {
    let kinds = [Kind::Gibbon, Kind::Gorilla, Kind::Lemur, Kind::Orangutan];
    let entries: Vec<Entry> = kinds
        .iter()
        .map(|&kind| Entry {
            name: String::new(),
            kind,
            driver: Driver::Ai,
            palette: 0,
        })
        .collect();
    let mut race = Race::new(bay(), &entries, false, 7);
    let mut brains: Vec<Brain> = Difficulty::ALL
        .iter()
        .enumerate()
        .map(|(i, &d)| Brain::new(d, 0xA9E5 + i as u64))
        .collect();
    let b = race.course.bay.as_deref().unwrap().clone();
    let mut resets = 0;
    while race.clock < LIMIT && race.phase != Phase::Done {
        let c: Vec<_> = (0..4).map(|i| think(&race, i, &mut brains[i])).collect();
        race.step(&c);
        resets += race
            .events
            .iter()
            .filter(|e| matches!(e, Event::Reset { .. }))
            .count();
        for (i, r) in race.racers.iter().enumerate() {
            assert!(
                !in_a_set_sail(&b, r.ape.pos),
                "ape {i} in a set sail at {:?}, tick {}",
                r.ape.pos,
                race.tick
            );
        }
    }
    for (i, r) in race.racers.iter().enumerate() {
        assert!(
            r.finish.is_some(),
            "ape {i} stopped at {} m",
            r.progress / ONE
        );
    }
    println!("water resets: {resets}");
}
