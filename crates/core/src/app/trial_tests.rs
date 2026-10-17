//! A Time Trial from the app (moved from app.rs's tests).

use super::*;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

fn ui_of(a: &mut App) -> Value {
    serde_json::from_str(&a.ui()).unwrap()
}

fn texts(v: &Value) -> String {
    v.to_string()
}

#[test]
fn a_time_trial_is_solo_boosts_only_with_its_own_best() {
    let mut a = app();
    a.settings.best = Some(120 * 90);
    a.act("menu.trial", "");
    assert_eq!(a.screen, Screen::Setup);
    let ui = ui_of(&mut a);
    let shown = texts(&ui["panels"]);
    // the four targets, Bronze 1:40.00 worked out from the constants;
    // the Author time hidden until Gold
    assert!(
        shown.contains("1:40.00") && shown.contains("???"),
        "{shown}"
    );
    assert!(!shown.contains("setup.opponents"));
    a.act("menu.start", "");
    let p = a.play.as_mut().unwrap();
    assert_eq!(p.race.racers.len(), 1);
    assert!(p.race.items == crate::items::ItemSet::only(crate::race::Item::Boost) && p.trial);
    p.race.phase = Phase::Running;
    p.race.clock = 120 * 95;
    p.race.racers[0].finish = Some(120 * 95);
    a.tick();
    // a Time Trial best, separate from the race best
    assert_eq!(a.settings.trial_best, Some(120 * 95));
    assert_eq!(a.settings.best, Some(120 * 90));
    // 95 s earns Bronze (85–100 s); the HUD now aims for Silver
    let ui = ui_of(&mut a);
    assert_eq!(ui["hud"]["best"][1], "1:25.00");
    for _ in 0..200 {
        a.tick();
    }
    let shown = texts(&ui_of(&mut a)["panels"]);
    assert!(
        shown.contains("Bronze") && shown.contains("menu.replay"),
        "{shown}"
    );
}
