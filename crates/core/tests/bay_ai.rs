//! The AI on Banana Bay Battle (2026-10-15): every difficulty finishes it,
//! from the water and the decks back onto the line, no ape ever passes
//! through a set sail, and the difficulties stay ordered and spaced, as on
//! the Jungle (`tests/ai.rs`; SECOND-ORDER BT rows).

use tool_core::ai::{think, time_trial, trial_at, Brain, Difficulty, Trial};
use tool_core::bay::{bay, bay_seeded, BayLayout, Sail};
use tool_core::course::Course;
use tool_core::fx::ONE;
use tool_core::geom::V3;
use tool_core::map::Map;
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

fn secs(ticks: u32) -> f64 {
    ticks as f64 / TICKS_PER_SECOND as f64
}

/// As `difficulties_are_ordered_and_spaced` on the Jungle: over sixteen
/// layouts of Banana Bay Battle (one Chimpanzee, Equal Stats), each easier
/// level's mean finish is at least `GAP` slower than the next harder's,
/// and every race at every level finishes inside the limit (150 s).
/// Sixteen, not eight: one layout's whirl on a rope can move a level's
/// mean by 3 s (BT1).
#[test]
fn bay_difficulties_are_ordered_and_spaced() {
    const GAP: f64 = 6.0;
    let runs: Vec<_> = Difficulty::ALL
        .into_iter()
        .map(|level| {
            std::thread::spawn(move || {
                (0..16u64)
                    .map(|k| {
                        let t = trial_at(layout(k), &[Kind::Chimpanzee], &[level], true, LIMIT);
                        let f = t[0].finish.unwrap_or_else(|| {
                            panic!(
                                "{} finishes bay layout {k} (at {} m)",
                                level.key(),
                                t[0].progress / ONE
                            )
                        });
                        secs(f)
                    })
                    .collect::<Vec<f64>>()
            })
        })
        .collect();
    let mut means = Vec::new();
    for (level, run) in Difficulty::ALL.into_iter().zip(runs) {
        let row = run.join().expect("every race finishes");
        let mean = row.iter().sum::<f64>() / row.len() as f64;
        println!("{:<10} {row:.1?} mean {mean:.1} s", level.key());
        means.push(mean);
    }
    for (i, w) in means.windows(2).enumerate() {
        assert!(
            w[0] >= w[1] + GAP,
            "{} ({:.1} s) is not {GAP} s slower than {} ({:.1} s)",
            Difficulty::ALL[i].key(),
            w[0],
            Difficulty::ALL[i + 1].key(),
            w[1]
        );
    }
}

/// Banana Bay Battle's medal report: King Kong alone as a Time Trial runs,
/// every kind and Equal Stats, against its medal times.
#[test]
#[ignore]
fn bay_time_trial_report() {
    let mut row: Vec<Trial> = Kind::ALL
        .into_iter()
        .map(|k| time_trial(bay(), k, false, LIMIT))
        .collect();
    row.push(time_trial(bay(), Kind::Chimpanzee, true, LIMIT));
    println!("bay time trial (last: equal stats)");
    for t in &row {
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
    let medals = tool_core::trial::medals(Map::Bay).map(secs);
    println!("medals bronze / silver / gold / author: {medals:.2?}");
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
            costume: tool_core::costume::Costume::Own,
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

/// Put back on any ship's deck by the water (its reset spot, aft of the
/// foremast), every level gets off it and 25 m on along the course within
/// 12 s (the ships of the two lines overlap, so "the next ship's stern" may
/// lie behind the spot).
#[test]
fn every_level_gets_off_every_deck_back_onto_the_line() {
    let runs: Vec<_> = [Difficulty::Sloth, Difficulty::KingKong]
        .into_iter()
        .map(|level| {
            std::thread::spawn(move || {
                let mut stuck = Vec::new();
                let entries = [Entry {
                    name: String::new(),
                    kind: Kind::Chimpanzee,
                    driver: Driver::Ai,
                    palette: 0,
                    costume: tool_core::costume::Costume::Own,
                }];
                let course = bay();
                let b = course.bay.as_deref().unwrap().clone();
                for k in 0..b.ships.len() - 1 {
                    let mut race = Race::new(course.clone(), &entries, true, 3);
                    while race.phase != Phase::Running {
                        race.step(&[Default::default()]);
                    }
                    let s = &b.ships[k];
                    let spot = s.reset_spot().expect("a deck");
                    race.racers[0].ape.put_on(spot, s.yaw, &race.course);
                    let goal = race
                        .course
                        .progress(spot, race.course.nearest_anywhere(spot))
                        .0
                        + 25 * ONE;
                    let mut brain = Brain::new(level, 0xA9E5);
                    let end = race.tick + 12 * TICKS_PER_SECOND as u32;
                    while race.tick < end && race.racers[0].progress < goal {
                        let c = think(&race, 0, &mut brain);
                        race.step(&[c]);
                    }
                    if race.racers[0].progress < goal {
                        stuck.push(format!(
                            "{} on ship {k}: at {} m, the goal at {} m",
                            level.key(),
                            race.racers[0].progress / ONE,
                            goal / ONE
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
    assert!(stuck.is_empty(), "stuck on a deck: {stuck:?}");
}
