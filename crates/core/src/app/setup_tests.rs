//! The race setup's choices (TRIAGE T26).

use serde_json::Value;

use super::{App, Settings};

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

#[test]
fn a_race_offers_one_to_three_opponents_never_none() {
    let mut a = app();
    a.act("menu.race", "");
    let ui: Value = serde_json::from_str(&a.ui()).unwrap();
    let select = ui["panels"][0]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == "setup.opponents")
        .cloned()
        .unwrap();
    let values: Vec<&str> = select["options"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["value"].as_str().unwrap())
        .collect();
    assert_eq!(values, ["1", "2", "3"]);
    // a 0 sent anyway races one opponent
    a.act("setup.opponents", "0");
    assert_eq!(a.settings.ai_count, 1);
    a.act("menu.start", "");
    assert_eq!(a.play.as_ref().unwrap().race.racers.len(), 2);
}

#[test]
fn a_saved_zero_opponents_reads_as_one() {
    assert_eq!(Settings::parse("{\"ai_count\":0}").ai_count, 1);
    assert_eq!(Settings::parse("{\"ai_count\":2}").ai_count, 2);
    // the lobby may set none (people race people); an offline race still has one
    let mut a = app();
    a.settings.ai_count = 0;
    a.act("menu.race", "");
    a.act("menu.start", "");
    assert_eq!(a.play.as_ref().unwrap().race.racers.len(), 2);
}
