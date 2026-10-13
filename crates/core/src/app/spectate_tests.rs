//! Spectating after the finish (TRIAGE T34), the followed racer and the
//! steerable orbit in replays (T24, T23), and the sound in replays (T30).

use serde_json::Value;

use super::*;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

fn ui_of(a: &mut App) -> Value {
    serde_json::from_str(&a.ui()).unwrap()
}

/// An offline race against `n` AI apes, run `ticks` with W held.
fn race(n: u8, ticks: u32) -> App {
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.opponents", &n.to_string());
    a.act("menu.start", "");
    a.key("KeyW", true);
    for _ in 0..ticks {
        a.tick();
    }
    a.key("KeyW", false);
    a
}

fn play(a: &App) -> &Play {
    a.play.as_ref().unwrap()
}

fn panel_kind(a: &mut App) -> String {
    ui_of(a)["panels"][0]["kind"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

/// The followed ape is in the middle of the view, near enough.
fn looks_at(a: &App, i: usize) -> bool {
    let cam = a.camera();
    let at = play(a).race.racers[i].ape.pos;
    cam.screen(ONE * 16 / 9, at)
        .is_some_and(|(x, y)| (x - 50 * ONE).abs() < 20 * ONE && (y - 50 * ONE).abs() < 30 * ONE)
        && cam.eye.dist(at) < int(20)
}

#[test]
fn after_the_finish_the_camera_pans_to_the_leader_still_racing() {
    let mut a = race(3, 900);
    // racing: A and D steer, they do not switch the view
    a.key("KeyD", true);
    a.key("KeyD", false);
    assert_eq!(play(&a).focus.racer, 0);
    assert!(ui_of(&mut a)["want_pointer"] == true);
    let p = a.play.as_mut().unwrap();
    p.race.racers[0].finish = Some(p.race.clock);
    for _ in 0..SPECTATE_AFTER - 1 {
        a.tick();
    }
    assert_eq!(play(&a).focus.racer, 0, "a moment on the finisher first");
    a.tick();
    a.tick();
    let p = play(&a);
    let leader = Focus::leader_racing(&p.race).unwrap();
    assert_eq!(p.focus.racer, leader);
    assert!(p.pan.is_some(), "it pans, not cuts");
    for _ in 0..camera::PAN_TICKS + 10 {
        a.tick();
    }
    assert!(play(&a).pan.is_none());
    assert!(looks_at(&a, leader), "the camera is on the leader");
    // a smaller results panel at the side, the pointer free, the racer named
    let ui = ui_of(&mut a);
    assert_eq!(ui["want_pointer"], false);
    assert_eq!(ui["panels"][0]["kind"], "spectate");
    let shown = ui["panels"].to_string();
    assert!(
        shown.contains("focus") && shown.contains("Watching"),
        "{shown}"
    );
    assert!(shown.contains("menu.rematch") && shown.contains("menu.replay"));
    assert_eq!(ui["hud"]["hint"], a.t("hud.spectate"));
    // D and the panel's buttons switch among those still racing
    a.key("KeyD", true);
    a.key("KeyD", false);
    let other = play(&a).focus.racer;
    assert!(other != leader && other != 0, "{other}");
    a.act("focus.prev", "");
    assert_eq!(play(&a).focus.racer, leader);
    // when the followed one finishes too, on to the next still racing
    let p = a.play.as_mut().unwrap();
    p.race.racers[leader].finish = Some(p.race.clock);
    for _ in 0..SPECTATE_AFTER + 2 {
        a.tick();
    }
    let f = play(&a).focus.racer;
    assert!(f != leader && f != 0 && focus::racing(&play(&a).race.racers[f]));
    // all in: the full results
    let p = a.play.as_mut().unwrap();
    for r in &mut p.race.racers {
        r.finish = r.finish.or(Some(p.race.clock));
    }
    a.tick();
    assert_eq!(panel_kind(&mut a), "overlay");
    // the kept replay runs on past the finish, to the leaving
    let ticks = play(&a).race.tick;
    a.act("menu.main", "");
    let kept = a.library.listing()[0].0.length();
    assert_eq!(kept, ticks, "kept {kept} of {ticks} ticks");
}

#[test]
fn a_time_trial_has_no_one_to_spectate() {
    let mut a = app();
    a.act("menu.trial", "");
    a.act("menu.start", "");
    for _ in 0..400 {
        a.tick();
    }
    let p = a.play.as_mut().unwrap();
    p.race.racers[0].finish = Some(p.race.clock);
    for _ in 0..SPECTATE_AFTER * 2 {
        a.tick();
    }
    assert_eq!(play(&a).focus.racer, 0);
    assert!(play(&a).pan.is_none());
    assert_eq!(panel_kind(&mut a), "overlay");
}

#[test]
fn a_replay_can_follow_any_racer_and_steer_the_orbit() {
    let mut a = race(2, 900);
    a.act("menu.replay", "");
    assert_eq!(play(&a).focus.racer, 0);
    let shown = ui_of(&mut a)["panels"].to_string();
    assert!(
        shown.contains("\"focus\""),
        "the replay panel names the racer"
    );
    a.act("focus.next", "");
    assert_eq!(play(&a).focus.racer, 1);
    a.key("ArrowRight", true);
    a.key("ArrowRight", false);
    assert_eq!(play(&a).focus.racer, 2, "the arrow keys too");
    a.key("KeyA", true);
    a.key("KeyA", false);
    assert_eq!(play(&a).focus.racer, 1);
    for _ in 0..600 {
        a.tick();
    }
    assert!(looks_at(&a, 1), "the chase camera is on racer 1");
    for v in ["orbit", "trackside"] {
        a.act("replay.camera", v);
        let cam = a.camera();
        let at = play(&a).race.racers[1].ape.pos;
        assert!(cam.screen(ONE, at).is_some(), "{v} looks at racer 1");
    }
    // a drag in another view turns it into a steered orbit, from that view
    a.act("replay.camera", "chase");
    let before = a.camera().fwd;
    a.drag(0, 0);
    a.drag(1, 0);
    let pb = play(&a).replay.as_ref().unwrap();
    assert_eq!(pb.view, replay::View::Orbit);
    assert!(pb.orbit.steered);
    assert!(a.camera().fwd.dot(before) > ONE * 9 / 10, "no jump");
    let yaw = play(&a).replay.as_ref().unwrap().orbit.yaw;
    for _ in 0..100 {
        a.tick();
    }
    assert_eq!(play(&a).replay.as_ref().unwrap().orbit.yaw, yaw);
    let d = play(&a).replay.as_ref().unwrap().orbit.dist;
    a.wheel(-120);
    assert!(
        play(&a).replay.as_ref().unwrap().orbit.dist < d,
        "the wheel zooms"
    );
    // outside a replay a drag does nothing
    let mut b = race(1, 10);
    b.drag(50, 50);
    b.wheel(100);
    assert!(b.play.as_ref().unwrap().replay.is_none());
}

/// The sounds the app asks for over `ticks` ticks.
fn sounds_over(a: &mut App, ticks: u32) -> Vec<String> {
    let mut heard = Vec::new();
    for _ in 0..ticks {
        a.tick();
        if let Some(s) = ui_of(a)["sounds"].as_array() {
            heard.extend(s.iter().map(|v| v.as_str().unwrap().to_string()));
        }
    }
    heard
}

#[test]
fn replays_sound_like_the_race_and_hush_when_stopped() {
    let mut a = race(2, 1500);
    a.act("menu.replay", "");
    ui_of(&mut a);
    // seeking runs the race fast and quietly: through the countdown, Go
    // and on to the end
    a.act("replay.seek", "1000");
    ui_of(&mut a);
    a.tick();
    a.play.as_mut().unwrap().hype_speed = int(45);
    assert_eq!(a.wind_gain(), 0, "no wind while seeking");
    let heard = sounds_over(&mut a, 60);
    assert_eq!(play(&a).race.tick, 1500);
    assert!(heard.is_empty(), "{heard:?}");
    // played, the countdown and the race make their sounds again
    a.act("replay.play", "");
    let heard = sounds_over(&mut a, 1400);
    assert!(heard.iter().any(|s| s == "beep"), "{heard:?}");
    assert!(heard.iter().any(|s| s == "go" || s == "land"), "{heard:?}");
    // the wind is the smoothed flat speed, as the dial shows it
    let p = a.play.as_mut().unwrap();
    p.hype_speed = int(45);
    assert!(a.wind_gain() > 90);
    a.key("Space", true);
    a.key("Space", false);
    assert_eq!(a.wind_gain(), 0, "paused: no wind");
}

#[test]
fn the_wind_ignores_a_fall() {
    let mut a = race(0, 400);
    let p = a.play.as_mut().unwrap();
    // falling straight down fast, not moving along
    p.race.racers[0].ape.vel = v3(0, -int(40), 0);
    p.hype_speed = 0;
    assert_eq!(a.wind_gain(), 0);
}
