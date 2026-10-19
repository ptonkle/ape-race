//! Grocery Run (GR-T11): unlocked for every map by a finish on Aisle Ape
//! within its Author time with every item on the shopping list; lent to
//! everyone in an online lobby on Aisle Ape, and worn on other maps only
//! once unlocked.

use serde_json::Value;

use super::*;
use crate::costume::Costume;
use crate::trial::Medal;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

fn ui_of(a: &mut App) -> Value {
    serde_json::from_str(&a.ui()).unwrap()
}

fn item(ui: &Value, id: &str) -> Option<Value> {
    ui["panels"]
        .as_array()?
        .iter()
        .flat_map(|p| p["items"].as_array().cloned().unwrap_or_default())
        .find(|i| i["id"] == id)
}

fn options(ui: &Value, id: &str) -> Vec<String> {
    item(ui, id).unwrap()["options"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["value"].as_str().unwrap().to_string())
        .collect()
}

/// Starts a race (`trial` false: against AIs) on `map`.
fn start(a: &mut App, map: Map, trial: bool) {
    a.act("menu.main", "");
    a.act(if trial { "menu.trial" } else { "menu.race" }, "");
    a.act("setup.map", map.key());
    a.act("menu.start", "");
}

/// Runs the local ape to a finish at `ticks` holding the list items
/// `list` (a bit each) and lets the app see it.
fn finish(a: &mut App, ticks: u32, list: u8) {
    let p = a.play.as_mut().unwrap();
    p.race.phase = Phase::Running;
    p.race.clock = ticks;
    p.race.racers[0].ape.list = list;
    p.race.racers[0].finish = Some(ticks);
    a.tick();
}

fn author(map: Map) -> u32 {
    crate::trial::target_on(map, Medal::Author)
}

fn new_costume(a: &App) -> Option<Costume> {
    a.play.as_ref().unwrap().new_costume
}

const ALL_FOUR: u8 = 0b1111;

#[test]
fn the_author_time_on_aisle_ape_with_all_four_items_unlocks_grocery_run() {
    let mut a = app();
    let map = Map::Grocery;
    // three items: the store uniform, not Grocery Run
    start(&mut a, map, false);
    finish(&mut a, author(map), 0b0111);
    assert!(a.settings.wearable(Costume::Grocer));
    assert!(!a.settings.wearable(Costume::Errands));
    assert_eq!(new_costume(&a), Some(Costume::Grocer));
    // all four, a tick over the Author time: still not
    start(&mut a, map, false);
    finish(&mut a, author(map) + 1, ALL_FOUR);
    assert!(!a.settings.wearable(Costume::Errands));
    // all four within it: unlocked, and said under the results
    start(&mut a, map, false);
    finish(&mut a, author(map), ALL_FOUR);
    assert!(a.settings.wearable(Costume::Errands));
    assert_eq!(new_costume(&a), Some(Costume::Errands));
    let rows = a.best_row(a.play.as_ref().unwrap()).to_string();
    assert!(
        rows.contains("[\"Costume unlocked\",\"Grocery Run\"]"),
        "{rows}"
    );
    // a second time is not news
    start(&mut a, map, true);
    finish(&mut a, author(map) - 10, ALL_FOUR);
    assert_eq!(new_costume(&a), None);
    // kept with the settings, and worn on any map
    let saved = ui_of(&mut a)["save"].as_str().unwrap().to_string();
    let b = Settings::parse(&saved);
    assert!(b.wearable(Costume::Errands));
    a.act("menu.main", "");
    a.act("menu.race", "");
    a.act("setup.map", Map::Jungle.key());
    assert!(options(&ui_of(&mut a), "setup.costume").contains(&"errands".to_string()));
    a.act("setup.costume", "errands");
    assert_eq!(a.settings.costume, Costume::Errands);
}

#[test]
fn only_aisle_ape_unlocks_grocery_run() {
    for map in Map::ALL {
        let mut a = app();
        start(&mut a, map, false);
        finish(&mut a, author(map), ALL_FOUR);
        assert_eq!(
            a.settings.wearable(Costume::Errands),
            map == Map::Grocery,
            "{map:?}"
        );
    }
}

#[test]
fn a_locked_grocery_run_is_not_offered_on_race_setup() {
    let mut a = app();
    a.act("menu.race", "");
    for map in [Map::Grocery, Map::Jungle] {
        a.act("setup.map", map.key());
        let ui = ui_of(&mut a);
        assert!(!options(&ui, "setup.costume").contains(&"errands".to_string()));
    }
    a.act("setup.costume", "errands");
    assert_eq!(a.settings.costume, Costume::Own);
}

#[test]
fn an_online_lobby_on_aisle_ape_lends_grocery_run_to_everyone() {
    // Sam, GR-T11: "you can select between costumes in a dropdown online
    // when you play aisle ape, and is only selectable on other maps if
    // you've unlocked it"
    let mut a = app();
    a.act("menu.online", "");
    a.act("online.host", "");
    let seat = |a: &App| a.online.session.as_ref().unwrap().lobby().seats[0].costume;
    a.act("lobby.map", Map::Jungle.key());
    assert!(!options(&ui_of(&mut a), "lobby.costume").contains(&"errands".to_string()));
    a.act("lobby.costume", "errands");
    assert_eq!(seat(&a), Costume::Own);
    // on Aisle Ape it is offered and worn, though locked, and not kept
    a.act("lobby.map", Map::Grocery.key());
    let ui = ui_of(&mut a);
    assert!(options(&ui, "lobby.costume").contains(&"errands".to_string()));
    a.act("lobby.costume", "errands");
    assert_eq!(seat(&a), Costume::Errands);
    assert_eq!(
        item(&ui_of(&mut a), "lobby.costume").unwrap()["value"],
        "errands"
    );
    assert_eq!(a.settings.costume, Costume::Own);
    assert!(!a.settings.wearable(Costume::Errands));
    // the host moves the lobby off Aisle Ape: it is given back
    a.act("lobby.map", Map::City.key());
    assert_eq!(seat(&a), Costume::Own);
    assert!(!options(&ui_of(&mut a), "lobby.costume").contains(&"errands".to_string()));
    // Random is not Aisle Ape until it is picked at the start
    a.act("lobby.map", Map::Grocery.key());
    a.act("lobby.costume", "errands");
    a.act("lobby.map", crate::map::RANDOM_MAP);
    assert_eq!(seat(&a), Costume::Own);
    // once unlocked, it is worn on any map and kept
    a.settings.unlock(Costume::Errands);
    a.act("lobby.map", Map::City.key());
    a.act("lobby.costume", "errands");
    assert_eq!(seat(&a), Costume::Errands);
    assert_eq!(a.settings.costume, Costume::Errands);
    a.act("lobby.map", Map::Jungle.key());
    assert_eq!(seat(&a), Costume::Errands);
}

#[test]
fn a_lent_grocery_run_is_given_back_when_the_hosts_lobby_leaves_aisle_ape() {
    // a joiner hears of the host's new map in the roster: it gives the lent
    // costume back too
    let mut host = app();
    // (each browser's own player id, as the page gives it)
    host.act("flag.client", "browser-h");
    host.act("menu.online", "");
    host.act("online.host", "");
    host.act("lobby.map", Map::Grocery.key());
    let mut joiner = app();
    joiner.act("flag.client", "browser-j");
    joiner.act("menu.online", "");
    let room = host.online.room.clone();
    joiner.act("online.room", &room);
    joiner.act("online.join", "");
    let pass = |from: &mut App, to: &mut App, from_peer: &str, host_side: bool| {
        let out: Value = serde_json::from_str(&from.take_outbox()).unwrap();
        for m in out.as_array().unwrap() {
            to.net_event("message", from_peer, m["text"].as_str().unwrap(), host_side);
        }
    };
    joiner.net_event("connected", "h", "", true);
    host.net_event("connected", "j", "", false);
    for _ in 0..4 {
        pass(&mut joiner, &mut host, "j", false);
        pass(&mut host, &mut joiner, "h", true);
    }
    let mine = |a: &App| {
        let v = a.online.session.as_ref().unwrap().lobby();
        v.seats.iter().find(|s| s.me).unwrap().costume
    };
    assert_eq!(
        joiner.online.session.as_ref().unwrap().lobby().map,
        Map::Grocery
    );
    joiner.act("lobby.costume", "errands");
    for _ in 0..4 {
        pass(&mut joiner, &mut host, "j", false);
        pass(&mut host, &mut joiner, "h", true);
    }
    assert_eq!(mine(&joiner), Costume::Errands);
    host.act("lobby.map", Map::Bay.key());
    for _ in 0..4 {
        pass(&mut host, &mut joiner, "h", true);
        pass(&mut joiner, &mut host, "j", false);
    }
    assert_eq!(
        joiner.online.session.as_ref().unwrap().lobby().map,
        Map::Bay
    );
    assert_eq!(mine(&joiner), Costume::Own);
}
