//! The ape call button (TRIAGE T58): Left Shift makes the player's ape
//! give its kind's call, in a race, in the setup screens and in the lobby;
//! a 2 s cooldown; another racer's call heard by distance; played back from
//! a replay.

use serde_json::Value;

use super::{App, Screen};
use crate::fx::int;
use crate::geom::v3;
use crate::rules::Kind;
use crate::sound;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

/// A gain the core wrote as a decimal (`0.42`), in hundredths (no floats
/// in the core, its tests included).
fn hundredths(v: &Value) -> i64 {
    let text = v.to_string();
    let (whole, frac) = text.split_once('.').unwrap_or((&text, ""));
    let frac = format!("{frac:0<2}");
    whole.parse::<i64>().unwrap() * 100 + frac[..2].parse::<i64>().unwrap()
}

/// The calls the page is told to play: (sound key, loudness in hundredths,
/// 100 = full).
fn calls(a: &mut App) -> Vec<(String, i64)> {
    let ui: Value = serde_json::from_str(&a.ui()).unwrap();
    ui["calls"]
        .as_array()
        .map(|cs| {
            cs.iter()
                .map(|c| {
                    (
                        c["key"].as_str().unwrap().to_string(),
                        hundredths(&c["gain"]),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn press(a: &mut App) {
    a.key("ShiftLeft", true);
    a.key("ShiftLeft", false);
}

fn race(a: &mut App, kind: &str) {
    a.act("menu.race", "");
    a.act("setup.ape", kind);
    a.act("setup.opponents", "2");
    a.act("menu.start", "");
    assert_eq!(a.screen, Screen::Race);
}

#[test]
fn the_key_calls_once_and_not_again_inside_two_seconds() {
    let mut a = app();
    race(&mut a, "gorilla");
    for _ in 0..10 {
        a.tick();
    }
    press(&mut a);
    assert_eq!(calls(&mut a), vec![("call-gorilla".to_string(), 100)]);
    // held down or pressed again at once: nothing
    press(&mut a);
    assert!(
        calls(&mut a).is_empty(),
        "a second press inside the cooldown"
    );
    for _ in 0..sound::CALL_COOLDOWN - 1 {
        a.tick();
    }
    press(&mut a);
    assert!(calls(&mut a).is_empty(), "just short of 2 s");
    a.tick();
    press(&mut a);
    assert_eq!(calls(&mut a).len(), 1, "after 2 s it calls again");
    // both calls are in the recording, at their ticks
    let rec = a.play.as_ref().unwrap().record.as_ref().unwrap();
    assert_eq!(rec.calls, vec![(10, 0), (10 + sound::CALL_COOLDOWN, 0)]);
    assert_eq!(&a.sound("call-gorilla")[0..4], b"RIFF");
}

#[test]
fn the_setup_screens_play_the_chosen_kind() {
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.ape", "lemur");
    press(&mut a);
    assert_eq!(calls(&mut a), vec![("call-lemur".to_string(), 100)]);
    a.act("menu.back", "");
    a.act("menu.trial", "");
    assert_eq!(a.screen, Screen::Setup);
    a.act("setup.ape", "proboscis");
    press(&mut a);
    assert!(
        calls(&mut a).is_empty(),
        "the cooldown holds across screens"
    );
    for _ in 0..sound::CALL_COOLDOWN {
        a.tick();
    }
    press(&mut a);
    assert_eq!(calls(&mut a), vec![("call-proboscis".to_string(), 100)]);
    assert_eq!(a.settings.kind, Kind::Proboscis);
    // the title has no ape to call with
    a.act("menu.back", "");
    for _ in 0..sound::CALL_COOLDOWN {
        a.tick();
    }
    press(&mut a);
    assert!(calls(&mut a).is_empty());
}

#[test]
fn another_racers_call_is_quieter_when_farther() {
    let mut a = app();
    race(&mut a, "gorilla");
    let mut p = a.play.take().unwrap();
    let me = p.race.racers[0].ape.pos;
    let other = p.race.racers[1].ape.kind;
    let mut heard = Vec::new();
    for m in [5, 20, 60] {
        p.race.racers[1].ape.pos = me.add(v3(int(m), 0, 0));
        a.hear_call(&p, 1);
        heard.push(calls(&mut a));
    }
    a.play = Some(p);
    assert_eq!(heard[0].len(), 1, "5 m: {:?}", heard[0]);
    assert_eq!(heard[0][0].0, sound::call_key(other));
    let (near, far) = (heard[0][0].1, heard[1][0].1);
    assert!(near < 100 && near > 30, "5 m: {near}");
    assert!(far < near / 5 && far > 0, "20 m: {far} vs 5 m: {near}");
    assert!(heard[2].is_empty(), "60 m: nothing: {:?}", heard[2]);
}

#[test]
fn a_replay_plays_the_call_back() {
    let mut a = app();
    race(&mut a, "gibbon");
    a.key("KeyW", true);
    for _ in 0..30 {
        a.tick();
    }
    press(&mut a);
    assert_eq!(calls(&mut a).len(), 1);
    for _ in 0..300 {
        a.tick();
    }
    a.act("menu.replay", "");
    assert!(a.play.as_ref().unwrap().replay.is_some());
    let mut played = Vec::new();
    for _ in 0..200 {
        a.tick();
        played.extend(calls(&mut a));
    }
    assert_eq!(played, vec![("call-gibbon".to_string(), 100)]);
}
