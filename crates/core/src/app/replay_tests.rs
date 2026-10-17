//! Bests, replays and the ghost (moved from app.rs's tests).

use super::*;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

#[test]
fn a_faster_finish_becomes_the_saved_best() {
    let mut a = app();
    a.settings.best = Some(120 * 100);
    a.act("menu.race", "");
    a.act("setup.opponents", "1");
    a.act("menu.start", "");
    let p = a.play.as_mut().unwrap();
    p.race.phase = Phase::Running;
    p.race.clock = 120 * 90;
    p.race.racers[0].finish = Some(120 * 90);
    a.tick();
    assert_eq!(a.settings.best, Some(120 * 90));
    assert!(a.play.as_ref().unwrap().new_best);
    let ui: Value = serde_json::from_str(&a.ui()).unwrap();
    assert!(ui["save"].as_str().unwrap().contains("10800"));
    // a slower one does not replace it
    let mut b = app();
    b.settings.best = Some(120 * 80);
    b.act("menu.race", "");
    b.act("menu.start", "");
    let p = b.play.as_mut().unwrap();
    p.race.phase = Phase::Running;
    p.race.racers[0].finish = Some(120 * 90);
    b.tick();
    assert_eq!(b.settings.best, Some(120 * 80));
}

fn ui_of(a: &mut App) -> Value {
    serde_json::from_str(&a.ui()).unwrap()
}

fn texts(v: &Value) -> String {
    v.to_string()
}

/// Plays a race with W held and a grab now and then; the apes' places
/// at each of `marks` ticks.
fn play_for(a: &mut App, n: u32, marks: &[u32]) -> Vec<Vec<V3>> {
    let mut seen = Vec::new();
    a.key("KeyW", true);
    for t in 1..=n {
        if t % 150 == 0 {
            a.mouse_button(0, true);
        }
        if t % 150 == 90 {
            a.mouse_button(0, false);
        }
        if t % 40 == 0 {
            a.play.as_mut().unwrap().chase.turn(deg(3), 0);
        }
        a.tick();
        if marks.contains(&t) {
            let p = a.play.as_ref().unwrap();
            seen.push(p.race.racers.iter().map(|r| r.ape.pos).collect());
        }
    }
    a.key("KeyW", false);
    a.mouse_button(0, false);
    seen
}

#[test]
fn watching_the_replay_shows_the_same_race_and_scrubs() {
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.opponents", "2");
    a.act("menu.start", "");
    let seen = play_for(&mut a, 900, &[450, 900]);
    a.act("menu.replay", "");
    let p = a.play.as_ref().unwrap();
    assert!(p.replay.is_some());
    assert_eq!(p.race.tick, 0);
    // keys do nothing to a replay
    a.key("KeyS", true);
    for _ in 0..1000 {
        a.tick();
    }
    let p = a.play.as_ref().unwrap();
    assert_eq!(p.race.tick, 900, "it stops at the end");
    assert!(!p.replay.as_ref().unwrap().playing);
    let now: Vec<V3> = p.race.racers.iter().map(|r| r.ape.pos).collect();
    assert_eq!(now, seen[1]);
    // scrub back halfway: a save point, then run forward
    a.act("replay.seek", "500");
    for _ in 0..20 {
        a.tick();
    }
    let p = a.play.as_ref().unwrap();
    let back: Vec<V3> = p.race.racers.iter().map(|r| r.ape.pos).collect();
    assert_eq!(p.race.tick, 450);
    assert_eq!(back, seen[0]);
    // Play at the end starts it over
    a.act("replay.seek", "1000");
    for _ in 0..20 {
        a.tick();
    }
    a.act("replay.play", "");
    a.tick();
    assert!(a.play.as_ref().unwrap().race.tick < 60);
    // the cameras all give a view
    for v in ["orbit", "trackside", "chase"] {
        a.act("replay.camera", v);
        assert_eq!(a.frame().0.len(), render::VIEW_VALUES);
    }
    // leaving goes back to the race it came from
    a.act("replay.exit", "");
    let p = a.play.as_ref().unwrap();
    assert!(p.replay.is_none());
    assert_eq!(p.race.tick, 900);
}

#[test]
fn finished_runs_are_kept_and_watchable_after_a_reload() {
    let mut a = app();
    a.act("menu.trial", "");
    a.act("menu.start", "");
    play_for(&mut a, 400, &[]);
    let p = a.play.as_mut().unwrap();
    p.race.racers[0].finish = Some(9000);
    a.act("menu.main", "");
    let ui = ui_of(&mut a);
    let saved = ui["save_replays"].as_str().unwrap().to_string();
    assert!(saved.len() < 400 * 8, "{} letters", saved.len());
    // a new visit: the best is on the Replays list and on the setup
    let mut b = app();
    b.load_replays(&saved);
    b.act("menu.replays", "");
    let shown = texts(&ui_of(&mut b)["panels"]);
    assert!(
        shown.contains("★ Best · Canopy Run · Time Trial"),
        "{shown}"
    );
    b.act("replays.watch.0", "");
    assert!(b.play.as_ref().unwrap().replay.is_some());
    b.act("replay.exit", "");
    assert_eq!(b.screen, Screen::Replays);
    b.act("menu.trial", "");
    assert!(texts(&ui_of(&mut b)["panels"]).contains("replay.best"));
    b.act("replay.best", "");
    assert!(b.play.as_ref().unwrap().replay.is_some());
    // an unfinished run is not kept
    let mut c = app();
    c.act("menu.race", "");
    c.act("menu.start", "");
    c.act("menu.main", "");
    assert!(ui_of(&mut c)["save_replays"].is_null());
}

/// A Time Trial run on the Jungle finished in `t` ticks, kept.
fn trial_run(a: &mut App, t: u32) {
    a.act("menu.trial", "");
    a.act("menu.start", "");
    play_for(a, 300, &[]);
    a.play.as_mut().unwrap().race.racers[0].finish = Some(t);
    a.act("menu.main", "");
}

#[test]
fn a_best_of_an_older_layout_is_kept_watch_only_and_never_a_ghost() {
    // TRIAGE T55: the map was retuned since the best was made
    let mut a = app();
    trial_run(&mut a, 9000);
    a.library.best[0].layout ^= 1;
    let mut b = app();
    b.load_replays(&a.library.text());
    b.act("menu.replays", "");
    let shown = texts(&ui_of(&mut b)["panels"]);
    assert!(
        shown.contains("★ Best · Canopy Run · Time Trial · Chimpanzee · 1:15.00 · older layout"),
        "{shown}"
    );
    assert!(shown.contains(r#""disabled":false"#), "{shown}");
    b.act("replays.watch.0", "");
    assert!(b.play.as_ref().unwrap().replay.is_some(), "watchable");
    b.act("replay.exit", "");
    // not offered as the best to watch or race on the setup
    b.act("menu.trial", "");
    assert!(!texts(&ui_of(&mut b)["panels"]).contains("replay.best"));
    b.act("menu.start", "");
    assert!(b.play.as_ref().unwrap().ghost.is_none());
    b.act("menu.main", "");
    // a slower run on the layout as it is now: both kept, the new one
    // the ghost
    trial_run(&mut b, 12_000);
    let times: Vec<Option<u32>> = b.library.best.iter().map(|r| r.finish).collect();
    assert_eq!(times, vec![Some(9000), Some(12_000)]);
    b.act("menu.trial", "");
    b.act("menu.start", "");
    assert_eq!(
        b.play.as_ref().unwrap().ghost.as_ref().unwrap().finish,
        Some(12_000)
    );
}

#[test]
fn when_storage_is_full_recent_runs_are_shed_never_a_best() {
    // TRIAGE T55: the page reports each write storage would not take
    let mut a = app();
    trial_run(&mut a, 9000);
    for k in 1..=4 {
        let mut r = a.library.best[0].clone();
        r.finish = Some(9000 + k);
        a.library.add(r);
    }
    let mut recents = Vec::new();
    a.save_replays = true;
    while let Some(text) = ui_of(&mut a)["save_replays"].as_str() {
        let saved = Library::parse(text);
        assert_eq!(saved.best, a.library.best, "a best was shed");
        recents.push(saved.recent.len());
        a.replays_not_saved();
        assert!(recents.len() < 10, "it stops");
    }
    assert_eq!(recents, vec![4, 3, 2, 1, 0]);
    // nothing is lost from this visit's list
    assert_eq!(a.library.recent.len(), 4);
}

#[test]
fn the_best_time_trial_races_as_a_ghost() {
    let mut a = app();
    a.act("menu.trial", "");
    a.act("menu.start", "");
    let seen = play_for(&mut a, 600, &[600]);
    a.play.as_mut().unwrap().race.racers[0].finish = Some(9000);
    a.act("menu.restart", "");
    let p = a.play.as_ref().unwrap();
    assert!(p.ghost.is_some());
    let (_, plain, _) = {
        let mut b = app();
        b.act("menu.trial", "");
        b.act("setup.ghost", "off");
        b.act("menu.start", "");
        assert!(b.play.as_ref().unwrap().ghost.is_none());
        b.frame()
    };
    let (_, with_ghost, _) = a.frame();
    assert!(with_ghost.len() > plain.len());
    // standing still, the player watches the ghost go the same way
    for _ in 0..600 {
        a.tick();
    }
    let g = a.play.as_ref().unwrap().ghost.as_ref().unwrap();
    assert_eq!(g.race.racers[0].ape.pos, seen[0][0]);
}
