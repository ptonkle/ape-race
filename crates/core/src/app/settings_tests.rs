//! The settings: kept in storage, the title's secret and the speed colours
//! (moved from app.rs's tests).

use super::*;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

#[test]
fn settings_round_trip_through_storage() {
    let mut a = app();
    a.act("settings.volume", "35");
    a.act("settings.invert", "on");
    let ui: Value = serde_json::from_str(&a.ui()).unwrap();
    let saved = ui["save"].as_str().unwrap().to_string();
    let b = App::new(include_str!("../../../../data/strings.json"), &saved, 1).unwrap();
    assert_eq!(b.settings.volume, 35);
    assert!(b.settings.invert_y);
    // nonsense in storage gives the defaults
    assert_eq!(Settings::parse("{oops"), Settings::default());
}

#[test]
fn ten_clicks_on_the_title_unlock_speed_colours() {
    let mut a = app();
    assert!(!a.settings.speed_colours && !a.settings.colours_unlocked);
    let shown = |a: &mut App| {
        a.act("menu.settings", "");
        let ui = a.ui();
        a.act("menu.back", "");
        ui.contains("settings.speed_colours")
    };
    assert!(!shown(&mut a));
    for _ in 0..9 {
        a.act("title.click", "");
    }
    assert!(!a.settings.colours_unlocked);
    a.act("title.click", "");
    assert!(a.settings.colours_unlocked && a.settings.speed_colours);
    // a fanfare, and the title's jungle goes wild at once
    let ui: Value = serde_json::from_str(&a.ui()).unwrap();
    assert!(ui["sounds"].to_string().contains("finish"));
    assert!(ui["save"]
        .as_str()
        .unwrap()
        .contains("\"colours_unlocked\":true"));
    assert!(a.frame().0[27] > 0);
    // until another menu opens; the setting stays from then on
    assert!(shown(&mut a));
    assert_eq!(a.frame().0[27], 0);
    // clicks on another screen visit do not count toward it twice
    let saved = serde_json::to_string(&a.settings).unwrap();
    let b = App::new(include_str!("../../../../data/strings.json"), &saved, 1).unwrap();
    assert!(b.settings.colours_unlocked && b.settings.speed_colours);
    // a setting saved "on" before the secret existed reads as off
    let old = Settings::parse("{\"speed_colours\":true}");
    assert!(!old.speed_colours);
}

#[test]
fn speed_colours_can_be_turned_off() {
    let mut a = app();
    a.settings.speed_colours = true;
    a.act("menu.race", "");
    a.act("menu.start", "");
    let fast = |a: &mut App| {
        let p = a.play.as_mut().unwrap();
        p.colour_speed = crate::fx::int(40);
        let (view, _, _) = a.frame();
        view[27]
    };
    assert!(fast(&mut a) > 0);
    a.act("settings.speed_colours", "off");
    assert_eq!(fast(&mut a), 0);
}

/// The Monkey level was removed (1.14.4, SECOND-ORDER DM-2): settings
/// saved with it load with the Sloth and keep everything else, and a
/// "monkey" key (a picture-tile query) gives the Sloth too.
#[test]
fn a_saved_monkey_level_loads_as_the_sloth() {
    let s = Settings::parse(r#"{"ai_level":"monkey","ai_count":2}"#);
    assert_eq!(s.ai_level, crate::ai::Difficulty::Sloth);
    assert_eq!(s.ai_count, 2);
    assert_eq!(
        crate::ai::Difficulty::from_key("monkey"),
        Some(crate::ai::Difficulty::Sloth)
    );
}
