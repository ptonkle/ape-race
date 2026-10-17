//! The race flow from the app: starting, the clock, pausing, restarting,
//! the AI's difficulty and the vines' chains (moved from app.rs's tests).

use super::*;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

#[test]
fn a_race_starts_from_setup_with_the_chosen_opponents() {
    let mut a = app();
    a.act("menu.race", "");
    assert_eq!(a.screen, Screen::Setup);
    a.act("setup.ape", "gibbon");
    a.act("setup.opponents", "2");
    a.act("menu.start", "");
    assert_eq!(a.screen, Screen::Race);
    let p = a.play.as_ref().unwrap();
    assert_eq!(p.race.racers.len(), 3);
    assert_eq!(p.race.racers[0].ape.kind, Kind::Gibbon);
    assert_eq!(p.race.racers[0].driver, Driver::Local);
    assert!(p.race.racers[1..]
        .iter()
        .all(|r| r.driver == Driver::Ai && r.ape.kind != Kind::Gibbon));
}

#[test]
fn the_clock_runs_ticks_at_the_tick_rate() {
    let mut a = app();
    a.act("menu.race", "");
    a.act("menu.start", "");
    a.advance(0);
    // one second in 16.667 ms frames
    for k in 1..=60 {
        a.advance(k * 16_667);
    }
    let t = a.play.as_ref().unwrap().race.tick;
    assert!((119..=121).contains(&t), "{t}");
    // and the same second in two frames gives the same ticks
    let mut b = app();
    b.act("menu.race", "");
    b.act("menu.start", "");
    b.advance(0);
    b.advance(500_000);
    b.advance(1_000_020);
    assert!(b.play.as_ref().unwrap().race.tick >= 24);
}

#[test]
fn a_summoned_vine_adds_a_chain_and_leaves_the_others_bent() {
    let mut a = app();
    a.act("menu.race", "");
    a.act("menu.start", "");
    let p = a.play.as_mut().unwrap();
    step_chains(p);
    let n = p.race.course.vines.len();
    assert_eq!(p.chains.len(), n);
    // a far vine's chain, bent: it is not stepped from here
    let me = p.race.racers[p.local].ape.pos;
    let far = (0..n)
        .find(|i| p.race.course.vines[*i].anchor.dist(me) > int(300))
        .unwrap();
    p.chains[far].pts[4].x += int(2);
    let bent = p.chains[far];
    let local = p.local;
    p.race.racers[local].item = Some(crate::race::Item::Vine);
    crate::items::use_item(&mut p.race, local, &Controls::default());
    step_chains(p);
    assert_eq!(p.chains.len(), n + 1);
    assert_eq!(p.chains[far], bent);
    assert_eq!(p.chains[n].pts[0], p.race.course.vines[n].anchor);
    // the slot's next vine elsewhere starts its own chain
    let mut s = p.race.summons[0];
    s.serial += crate::rules::SUMMON_SLOTS as u32;
    s.anchor = s.anchor.add(v3(int(30), 0, 0));
    p.race.place_vine(s);
    step_chains(p);
    assert_eq!(p.chains[n].pts[0], s.anchor);
}

#[test]
fn escape_pauses_and_resumes_the_race() {
    let mut a = app();
    a.act("menu.race", "");
    a.act("menu.start", "");
    a.key("Escape", true);
    assert!(a.play.as_ref().unwrap().paused);
    let t = a.play.as_ref().unwrap().race.tick;
    a.advance(0);
    a.advance(100_000);
    assert_eq!(a.play.as_ref().unwrap().race.tick, t);
    a.act("menu.resume", "");
    assert!(!a.play.as_ref().unwrap().paused);
}

#[test]
fn the_chosen_ai_difficulty_drives_the_ai_apes() {
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.ai_level", "sloth");
    a.act("setup.opponents", "2");
    a.act("menu.start", "");
    let p = a.play.as_ref().unwrap();
    assert_eq!(p.race.ai_level, crate::ai::Difficulty::Sloth);
    assert!(p
        .brains
        .iter()
        .all(|b| b.difficulty == crate::ai::Difficulty::Sloth));
    // and it is kept with the settings
    assert_eq!(a.settings.ai_level, crate::ai::Difficulty::Sloth);
}

#[test]
fn r_twice_quickly_restarts_but_once_or_slowly_does_not() {
    let mut a = app();
    a.act("menu.race", "");
    a.act("menu.start", "");
    for _ in 0..100 {
        a.tick();
    }
    let tick = |a: &App| a.play.as_ref().unwrap().race.tick;
    a.key("KeyR", true);
    a.key("KeyR", false);
    for _ in 0..DOUBLE_TAP + 1 {
        a.tick();
    }
    a.key("KeyR", true);
    a.key("KeyR", false);
    assert!(tick(&a) > 100, "two slow presses do not restart");
    for _ in 0..DOUBLE_TAP - 1 {
        a.tick();
    }
    a.key("KeyR", true);
    a.key("KeyR", false);
    assert_eq!(tick(&a), 0, "two quick presses restart");
    assert_eq!(a.screen, Screen::Race);
    // and in a Time Trial
    a.act("menu.trial", "");
    a.act("menu.start", "");
    for _ in 0..50 {
        a.tick();
    }
    a.key("KeyR", true);
    a.key("KeyR", false);
    a.tick();
    a.key("KeyR", true);
    let p = a.play.as_ref().unwrap();
    assert!(p.trial && p.race.tick == 0);
}
