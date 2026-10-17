//! The ape call button (TRIAGE T58): Left Shift makes the player's ape
//! give its kind's call, in a race, in the setup screens and in the lobby;
//! a 2 s cooldown; another racer's call heard by distance; played back from
//! a replay.

use serde_json::Value;

use super::{App, Screen};
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

/// In a race the key calls at full loudness once per cooldown, each call
/// is in the recording at its tick, and the replay plays them back. (The
/// cooldown's own count is `sound::CallClock`'s test, the loudness by
/// distance `sound::heard_call`'s.)
#[test]
fn a_call_in_a_race_is_recorded_and_the_replay_plays_it_back() {
    let mut a = app();
    race(&mut a, "gibbon");
    a.key("KeyW", true);
    for _ in 0..30 {
        a.tick();
    }
    press(&mut a);
    assert_eq!(calls(&mut a), vec![("call-gibbon".to_string(), 100)]);
    press(&mut a);
    assert!(
        calls(&mut a).is_empty(),
        "a second press inside the cooldown"
    );
    for _ in 0..sound::CALL_COOLDOWN {
        a.tick();
    }
    press(&mut a);
    assert_eq!(calls(&mut a).len(), 1, "after 2 s it calls again");
    let rec = a.play.as_ref().unwrap().record.as_ref().unwrap();
    assert_eq!(rec.calls, vec![(30, 0), (30 + sound::CALL_COOLDOWN, 0)]);
    assert_eq!(&a.sound("call-gibbon")[0..4], b"RIFF");
    for _ in 0..300 {
        a.tick();
    }
    a.act("menu.replay", "");
    assert!(a.play.as_ref().unwrap().replay.is_some());
    let mut played = Vec::new();
    for _ in 0..400 {
        a.tick();
        played.extend(calls(&mut a));
    }
    assert_eq!(played, vec![("call-gibbon".to_string(), 100); 2]);
}
