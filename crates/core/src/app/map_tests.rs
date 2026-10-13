//! Choosing a map (the City, 2026-10-12): the selector, the lazily built
//! courses and scenery, the background, and bests, medals, the ghost and
//! the replays kept per map.

use serde_json::Value;

use super::{App, Phase, Screen, Settings, Shown};
use crate::map::Map;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

fn ui_of(a: &mut App) -> Value {
    serde_json::from_str(&a.ui()).unwrap()
}

/// The setup's or lobby's item with this id.
fn item(ui: &Value, id: &str) -> Option<Value> {
    ui["panels"]
        .as_array()?
        .iter()
        .flat_map(|p| p["items"].as_array().cloned().unwrap_or_default())
        .find(|i| i["id"] == id)
}

/// Runs the local ape to a finish at `ticks` and lets the app see it.
fn finish(a: &mut App, ticks: u32) {
    let p = a.play.as_mut().unwrap();
    p.race.phase = Phase::Running;
    p.race.clock = ticks;
    p.race.racers[0].finish = Some(ticks);
    a.tick();
}

#[test]
fn settings_from_before_the_city_read_as_the_jungle_and_the_map_is_kept() {
    let old = Settings::parse("{\"best\":12000,\"trial_best\":9000}");
    assert_eq!(old.map, Map::Jungle);
    assert_eq!(old.best(Map::Jungle, false), Some(12000));
    assert_eq!(old.best(Map::Jungle, true), Some(9000));
    assert_eq!(old.best(Map::City, false), None);
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.map", "city");
    assert_eq!(a.settings.map, Map::City);
    let saved = ui_of(&mut a)["save"].as_str().unwrap().to_string();
    let b = App::new(include_str!("../../../../data/strings.json"), &saved, 7).unwrap();
    assert_eq!(b.settings.map, Map::City);
    // an unknown map leaves the choice alone
    a.act("setup.map", "moon");
    assert_eq!(a.settings.map, Map::City);
}

#[test]
fn race_setup_and_time_trial_offer_every_map() {
    for menu in ["menu.race", "menu.trial"] {
        let mut a = app();
        a.act(menu, "");
        let ui = ui_of(&mut a);
        let select = item(&ui, "setup.map").expect("a map select");
        let values: Vec<&str> = select["options"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o["value"].as_str().unwrap())
            .collect();
        assert_eq!(values, ["jungle", "city"]);
        assert_eq!(select["value"], "jungle");
        assert_eq!(select["options"][1]["text"], "City");
    }
}

#[test]
fn only_the_chosen_map_is_built_at_start_and_the_city_when_first_chosen() {
    let mut a = app();
    assert!(a.built(Map::Jungle));
    assert!(!a.built(Map::City));
    let saved = "{\"map\":\"city\"}";
    let b = App::new(include_str!("../../../../data/strings.json"), saved, 7).unwrap();
    assert!(b.built(Map::City));
    assert!(!b.built(Map::Jungle));
    a.act("menu.race", "");
    assert!(!a.built(Map::City));
    a.act("setup.map", "city");
    assert!(a.built(Map::City));
}

#[test]
fn choosing_the_city_moves_the_preview_onto_it_and_races_there() {
    let mut a = app();
    a.act("menu.race", "");
    assert_eq!(a.shown, Shown::StageSet);
    let v = a.scene_version();
    a.act("setup.map", "city");
    // the background changes at once: the City's scenery behind the ape
    assert_eq!(a.shown, Shown::World(Map::City));
    assert!(a.scene_version() > v);
    let (_, _, draws) = a.frame();
    assert!(!draws.is_empty());
    a.act("menu.start", "");
    assert_eq!(a.play.as_ref().unwrap().race.course.name, "city");
    assert_eq!(a.shown, Shown::World(Map::City));
    assert_eq!(ui_of(&mut a)["hud"]["map"][1], "City");
    // the title flies over the last chosen map
    a.act("menu.main", "");
    assert_eq!(a.shown, Shown::World(Map::City));
    // and back on the Jungle the ape stands in its stage again
    a.act("menu.race", "");
    a.act("setup.map", "jungle");
    assert_eq!(a.shown, Shown::StageSet);
    a.act("menu.start", "");
    assert_eq!(a.play.as_ref().unwrap().race.course.name, "jungle");
}

#[test]
fn bests_are_kept_per_map() {
    let mut a = app();
    a.settings.best = Some(120 * 100);
    a.act("menu.race", "");
    a.act("setup.map", "city");
    a.act("menu.start", "");
    finish(&mut a, 120 * 110);
    // a first City finish is the City's best, and the Jungle's is untouched
    assert!(a.play.as_ref().unwrap().new_best);
    assert_eq!(a.settings.best(Map::City, false), Some(120 * 110));
    assert_eq!(a.settings.best, Some(120 * 100));
    assert_eq!(a.settings.best(Map::City, true), None);
    let saved = ui_of(&mut a)["save"].as_str().unwrap().to_string();
    let b = Settings::parse(&saved);
    assert_eq!(b.best(Map::City, false), Some(120 * 110));
    // a Time Trial on the City
    a.act("menu.main", "");
    a.act("menu.trial", "");
    a.act("menu.start", "");
    finish(&mut a, 120 * 95);
    assert_eq!(a.settings.best(Map::City, true), Some(120 * 95));
    assert_eq!(a.settings.trial_best, None);
}

#[test]
fn the_ghost_and_the_best_replay_are_the_maps_own() {
    let mut a = app();
    a.act("menu.trial", "");
    a.act("menu.start", "");
    for _ in 0..400 {
        a.tick();
    }
    finish(&mut a, 9000);
    a.act("menu.main", "");
    // the Jungle's best: a ghost and a Watch Best on the Jungle only
    a.act("menu.trial", "");
    assert!(item(&ui_of(&mut a), "replay.best").is_some());
    a.act("setup.map", "city");
    assert!(item(&ui_of(&mut a), "replay.best").is_none());
    a.act("menu.start", "");
    assert!(a.play.as_ref().unwrap().ghost.is_none());
    a.act("menu.main", "");
    a.act("menu.trial", "");
    a.act("setup.map", "jungle");
    a.act("menu.start", "");
    assert!(a.play.as_ref().unwrap().ghost.is_some());
}

#[test]
fn the_replays_list_names_each_runs_map_and_plays_it_there() {
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.map", "city");
    a.act("menu.start", "");
    for _ in 0..400 {
        a.tick();
    }
    finish(&mut a, 9000);
    a.act("menu.main", "");
    let saved = ui_of(&mut a)["save_replays"].as_str().unwrap().to_string();
    // a new visit on the Jungle: the City's run is listed as the City's
    let mut b = app();
    b.load_replays(&saved);
    b.act("menu.replays", "");
    let ui = ui_of(&mut b);
    let run = item(&ui, "replays.watch.0").unwrap();
    assert!(run["text"].as_str().unwrap().contains("City"), "{run}");
    assert_eq!(run["disabled"], false);
    b.act("replays.watch.0", "");
    let p = b.play.as_ref().unwrap();
    assert!(p.replay.is_some());
    assert_eq!(p.race.course.name, "city");
    assert_eq!(b.shown, Shown::World(Map::City));
    // and back on the list the background is the title's map again
    b.act("replay.exit", "");
    assert_eq!(b.screen, Screen::Replays);
    assert_eq!(b.shown, Shown::World(Map::Jungle));
}
