//! A solo tournament from Race Setup (TRIAGE T103): the choice, the fixed
//! lineup, the standings between races, the final order for the podium.

use serde_json::Value;

use super::{App, Screen};
use crate::map::Map;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

fn ui(a: &mut App) -> Value {
    serde_json::from_str(&a.ui()).unwrap()
}

/// The item with this id, in any panel (the dock moves the map grid to a
/// strip of its own).
fn item(a: &mut App, id: &str) -> Option<Value> {
    ui(a)["panels"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|p| p["items"].as_array().cloned().unwrap_or_default())
        .find(|i| i["id"] == id)
}

/// A tournament of `races` races started from Race Setup, three AI apes.
fn started(races: u8, random: bool) -> App {
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.opponents", "3");
    a.act("setup.map", if random { "random" } else { "jungle" });
    a.act("setup.tournament", "on");
    a.settings.tour_races = races;
    a.act("menu.start", "");
    a
}

/// Ends the race with these finishes (by racer) and opens the standings.
fn finish(a: &mut App, fin: [Option<u32>; 4]) {
    let p = a.play.as_mut().unwrap();
    for (r, f) in p.race.racers.iter_mut().zip(fin) {
        r.finish = f;
    }
    p.race.clock = 1000;
    a.act("tour.standings", "");
}

#[test]
fn race_setup_offers_a_tournament_of_two_to_sixteen_races() {
    let mut a = app();
    a.act("menu.race", "");
    assert!(item(&mut a, "setup.races").is_none(), "off: no count");
    a.act("setup.tournament", "on");
    let s = item(&mut a, "setup.races").unwrap();
    assert_eq!(s["type"], "stepper");
    assert_eq!(s["text"], "4");
    for _ in 0..5 {
        a.act("setup.races.dec", "");
    }
    assert_eq!(a.settings.tour_races, 2);
    assert_eq!(item(&mut a, "setup.races").unwrap()["dec_disabled"], true);
    for _ in 0..20 {
        a.act("setup.races.inc", "");
    }
    assert_eq!(a.settings.tour_races, 16);
}

#[test]
fn a_solo_tournament_keeps_its_lineup_and_ends_on_the_final_order() {
    let mut a = started(2, false);
    let kinds = |a: &App| -> Vec<_> {
        a.play
            .as_ref()
            .unwrap()
            .race
            .racers
            .iter()
            .map(|r| r.ape.kind)
            .collect()
    };
    let first = kinds(&a);
    // the player wins, ai1 second, ai2 eaten or out there, ai3 third
    finish(&mut a, [Some(100), Some(200), None, Some(300)]);
    assert_eq!(a.screen, Screen::Standings);
    assert!(ui(&mut a).get("podium").is_none(), "not over yet");
    // a later start: fresh dice would draw other AI apes
    a.ticks += 12_345;
    a.act("tour.next", "");
    assert_eq!(a.screen, Screen::Race);
    assert_eq!(kinds(&a), first, "the same apes every race");
    // ai2 wins, the player second
    finish(&mut a, [Some(150), None, Some(90), None]);
    let v = ui(&mut a);
    let podium = v["podium"].as_array().unwrap();
    let order: Vec<(u64, u64)> = podium
        .iter()
        .map(|p| (p["place"].as_u64().unwrap(), p["points"].as_u64().unwrap()))
        .collect();
    // you 10+7, ai2 0+10, ai1 7+0, ai3 5+0
    assert_eq!(order, [(1, 17), (2, 10), (3, 7), (4, 5)]);
    assert_eq!(podium[0]["me"], true);
    assert_eq!(podium[1]["kind"], first[2].key());
    a.act("tour.next", "");
    assert_eq!(a.screen, Screen::Standings, "no race after the last");
    a.act("tour.done", "");
    assert_eq!(a.screen, Screen::Setup);
    assert!(a.tour.is_none());
}

#[test]
fn the_player_picks_each_next_map_unless_the_tournament_is_random() {
    let mut a = started(3, false);
    finish(&mut a, [Some(100), None, None, None]);
    let tiles = item(&mut a, "tour.map").expect("the next map's grid");
    assert!(tiles["options"]
        .as_array()
        .unwrap()
        .iter()
        .all(|o| o["value"] != "random"));
    a.act("tour.map", "city");
    a.act("tour.next", "");
    assert_eq!(Map::of(&a.play.as_ref().unwrap().race.course), Map::City);
    let mut r = started(3, true);
    finish(&mut r, [Some(100), None, None, None]);
    assert!(item(&mut r, "tour.map").is_none(), "Random: no grid");
}

#[test]
fn quitting_a_tournament_race_drops_it_and_returns_to_the_setup() {
    let mut a = started(4, false);
    a.act("tour.quit", "");
    assert_eq!(a.screen, Screen::Setup);
    assert!(a.tour.is_none());
}
