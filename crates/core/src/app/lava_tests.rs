//! Floor is Lava (TRIAGE T37) in the app: the setup's checkbox, the jaguar's
//! moment before the camera moves on, the results, the roar and replays.

use serde_json::Value;

use super::*;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

fn ui_of(a: &mut App) -> Value {
    serde_json::from_str(&a.ui()).unwrap()
}

fn play(a: &App) -> &Play {
    a.play.as_ref().unwrap()
}

/// A race against three AI apes, with Floor is Lava `lava`, past Go.
fn race(lava: bool) -> App {
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.lava", if lava { "on" } else { "off" });
    a.act("setup.opponents", "3");
    a.act("menu.start", "");
    for _ in 0..crate::race::COUNTDOWN as u32 + 60 {
        a.tick();
    }
    a
}

#[test]
fn the_setup_offers_floor_is_lava_but_a_time_trial_never_has_it() {
    let mut a = app();
    a.act("menu.race", "");
    let shown = ui_of(&mut a)["panels"].to_string();
    assert!(shown.contains("setup.lava") && shown.contains("Floor is Lava"));
    a.act("setup.lava", "on");
    assert!(a.settings.lava);
    assert!(ui_of(&mut a)["save"]
        .as_str()
        .unwrap()
        .contains("\"lava\":true"));
    a.act("menu.start", "");
    assert!(play(&a).race.lava);
    let rec = play(&a).record.as_ref().unwrap();
    assert!(rec.lava, "the replay knows the mode");
    // a Time Trial: no checkbox, and never the mode
    a.act("menu.main", "");
    a.act("menu.trial", "");
    let shown = ui_of(&mut a)["panels"].to_string();
    assert!(!shown.contains("setup.lava"));
    a.act("menu.start", "");
    assert!(!play(&a).race.lava);
}

#[test]
fn eaten_the_camera_stays_for_the_jaguar_then_spectates() {
    // the person: "after a short delay it cuts away to spectator mode for
    // the remaining players", a moment after the jaguar has eaten
    let mut a = race(false);
    let p = a.play.as_mut().unwrap();
    p.race.eat(0);
    let mut sounds = Vec::new();
    for t in 0..EATEN_SPECTATE_AFTER - 1 {
        a.tick();
        let ui = ui_of(&mut a);
        if let Some(s) = ui["sounds"].as_array() {
            sounds.extend(s.iter().map(|v| v.as_str().unwrap().to_string()));
        }
        if t == crate::jaguar::BITE {
            assert_eq!(ui["hud"]["big"], a.t("hud.eaten"));
            assert!(
                ui["panels"].as_array().unwrap().is_empty(),
                "no results yet"
            );
        }
    }
    assert!(sounds.iter().any(|s| s == "roar"), "{sounds:?}");
    assert_eq!(play(&a).focus.racer, 0, "the camera stays for the jaguar");
    a.tick();
    a.tick();
    let p = play(&a);
    let leader = Focus::leader_racing(&p.race).unwrap();
    assert_eq!(p.focus.racer, leader);
    assert!(p.pan.is_some(), "it pans, as after a finish");
    let ui = ui_of(&mut a);
    assert_eq!(ui["panels"][0]["kind"], "spectate");
    assert_eq!(ui["hud"]["big"], "");
    assert!(ui["panels"].to_string().contains(&a.t("results.eaten")));
}

#[test]
fn eaten_racers_are_listed_after_the_finishers_and_the_race_ends() {
    let mut a = race(false);
    let p = a.play.as_mut().unwrap();
    p.race.eat(0);
    let clock = p.race.clock;
    for i in 1..4 {
        p.race.racers[i].finish = Some(clock + i as u32);
        p.race.order.push(i);
    }
    for _ in 0..EATEN_SPECTATE_AFTER + 2 {
        a.tick();
    }
    let ui = ui_of(&mut a);
    assert_eq!(
        ui["panels"][0]["kind"], "overlay",
        "all out: the full results"
    );
    let rows = ui["panels"][0]["items"][0]["rows"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[3][3], a.t("results.eaten"));
    assert_eq!(rows[3][4], true, "the eaten local ape last");
    // an eaten run is kept among the replays (never a best), marked
    a.act("menu.main", "");
    let listed = a.library.listing();
    assert_eq!(listed.len(), 1);
    assert!(listed[0].0.eaten && !listed[0].1);
    assert!(a.settings.best.is_none(), "no best time from being eaten");
}

#[test]
fn a_lava_race_replays_the_jaguar_and_the_death() {
    let mut a = race(true);
    // run straight off the start platform onto the jungle floor
    a.key("KeyW", true);
    for _ in 0..600 {
        a.tick();
    }
    let e = play(&a).race.racers[0]
        .eaten
        .expect("eaten off the platform");
    for _ in 0..crate::jaguar::GONE {
        a.tick();
    }
    let others: Vec<_> = play(&a).race.racers.iter().map(|r| r.eaten).collect();
    a.act("menu.replay", "");
    let length = play(&a).replay.as_ref().unwrap().length();
    for _ in 0..length + 10 {
        a.tick();
    }
    let p = play(&a);
    assert!(p.race.lava);
    assert_eq!(p.race.racers[0].eaten, Some(e), "the same death");
    let again: Vec<_> = p.race.racers.iter().map(|r| r.eaten).collect();
    assert_eq!(again, others, "and the AI apes' too");
    // kept and reloaded, it still has the mode
    a.act("replay.exit", "");
    a.act("menu.main", "");
    let text = a.library.text();
    assert!(text.contains("\"lava\":true") && text.contains("\"eaten\":true"));
    let back = crate::replay::Library::parse(&text);
    assert!(back.recent[0].lava && back.recent[0].eaten);
}

#[test]
fn a_race_without_the_mode_is_kept_as_before() {
    // a replay with the mode off is stored and plays back unchanged: no
    // new fields in its text, so older builds read it the same
    let mut a = race(false);
    let p = a.play.as_mut().unwrap();
    p.race.racers[0].finish = Some(p.race.clock);
    for _ in 0..KEEP_AFTER_FINISH + 2 {
        a.tick();
    }
    a.act("menu.main", "");
    let text = a.library.text();
    assert!(!text.contains("lava") && !text.contains("eaten"), "{text}");
}
