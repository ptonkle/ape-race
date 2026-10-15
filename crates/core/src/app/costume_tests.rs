//! Unlockable costumes (TRIAGE T89): a finish under a map's Author time, in
//! any mode, unlocks its costume; the selector on Race Setup and in the
//! lobby; the race, its recording and the drawing carry it.

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

/// Runs the local ape to a finish at `ticks` and lets the app see it.
fn finish(a: &mut App, ticks: u32) {
    let p = a.play.as_mut().unwrap();
    p.race.phase = Phase::Running;
    p.race.clock = ticks;
    p.race.racers[0].finish = Some(ticks);
    a.tick();
}

/// Starts a race (`trial` false: against AIs) on `map`.
fn start(a: &mut App, map: Map, trial: bool) {
    a.act("menu.main", "");
    a.act(if trial { "menu.trial" } else { "menu.race" }, "");
    a.act("setup.map", map.key());
    a.act("menu.start", "");
}

fn author(map: Map) -> u32 {
    crate::trial::target_on(map, Medal::Author)
}

#[test]
fn a_finish_under_the_author_time_in_any_mode_unlocks_the_maps_costume() {
    let mut a = app();
    // a race against AIs with items, just over the Author time: nothing
    start(&mut a, Map::Bay, false);
    finish(&mut a, author(Map::Bay) + 1);
    assert!(a.settings.costumes.is_empty());
    assert_eq!(a.play.as_ref().unwrap().new_costume, None);
    // on it: the pirate's, said under the results
    start(&mut a, Map::Bay, false);
    finish(&mut a, author(Map::Bay));
    assert!(a.settings.wearable(Costume::Pirate));
    assert_eq!(a.play.as_ref().unwrap().new_costume, Some(Costume::Pirate));
    let rows = a.best_row(a.play.as_ref().unwrap()).to_string();
    assert!(rows.contains("[\"Costume unlocked\",\"Pirate\"]"), "{rows}");
    // a Time Trial (Speed Boosts only) does it too
    start(&mut a, Map::Sector, true);
    finish(&mut a, author(Map::Sector) - 10);
    assert!(a.settings.wearable(Costume::Space));
    // a second time is not news
    start(&mut a, Map::Sector, false);
    finish(&mut a, author(Map::Sector));
    assert_eq!(a.play.as_ref().unwrap().new_costume, None);
    // a map with no costume unlocks nothing
    start(&mut a, Map::Jungle, false);
    finish(&mut a, author(Map::Jungle) - 100);
    assert_eq!(a.settings.costumes.len(), 2);
    assert!(!a.settings.wearable(Costume::Ski));
    assert!(!a.settings.wearable(Costume::Explorer));
    // kept with the settings
    let saved = ui_of(&mut a)["save"].as_str().unwrap().to_string();
    let b = Settings::parse(&saved);
    assert!(b.wearable(Costume::Pirate) && b.wearable(Costume::Space));
    assert!(!b.wearable(Costume::Ski));
}

#[test]
fn author_times_beaten_before_costumes_unlock_them_and_nonsense_is_dropped() {
    let ski = author(Map::Ski);
    let old = Settings::parse(&format!("{{\"bests\":{{\"ski\":{{\"trial\":{ski}}}}}}}"));
    assert!(old.wearable(Costume::Ski));
    assert!(!old.wearable(Costume::Pirate));
    let odd =
        Settings::parse("{\"costumes\":[\"pirate\",\"moon\",\"own\"],\"costume\":\"explorer\"}");
    assert_eq!(odd.costumes.iter().collect::<Vec<_>>(), ["pirate"]);
    assert_eq!(odd.costume, Costume::Own, "a locked costume is not worn");
    let worn = Settings::parse("{\"costumes\":[\"pirate\"],\"costume\":\"pirate\"}");
    assert_eq!(worn.costume, Costume::Pirate);
}

#[test]
fn the_selector_shows_locked_costumes_and_wears_an_unlocked_one_anywhere() {
    let mut a = app();
    a.act("menu.race", "");
    let ui = ui_of(&mut a);
    let select = item(&ui, "setup.costume").expect("beside the colours");
    assert_eq!(select["value"], "own");
    let options = select["options"].as_array().unwrap();
    let keys: Vec<&str> = options
        .iter()
        .map(|o| o["value"].as_str().unwrap())
        .collect();
    assert_eq!(keys, ["own", "none", "ski", "space", "explorer", "pirate"]);
    let locked: Vec<bool> = options.iter().map(|o| o["disabled"] == true).collect();
    assert_eq!(locked, [false, false, true, true, true, true]);
    assert_eq!(
        options[5]["text"],
        "Pirate (Beat Banana Bay Battle's Author time)"
    );
    // a locked one is refused
    a.act("setup.costume", "pirate");
    assert_eq!(a.settings.costume, Costume::Own);
    a.act("setup.costume", "none");
    assert_eq!(a.settings.costume, Costume::Bare);
    // unlocked: worn on the Jungle, by the player only
    a.settings.unlock(Costume::Pirate);
    a.act("setup.costume", "pirate");
    assert_eq!(a.settings.costume, Costume::Pirate);
    let ui = ui_of(&mut a);
    let select = item(&ui, "setup.costume").unwrap();
    assert_eq!(select["value"], "pirate");
    assert_eq!(select["options"][5]["disabled"], Value::Null);
    a.act("setup.map", "jungle");
    a.act("menu.start", "");
    let p = a.play.as_ref().unwrap();
    assert_eq!(p.race.racers[0].costume, Costume::Pirate);
    assert!(p.race.racers[1..].iter().all(|r| r.costume == Costume::Own));
    assert_eq!(
        p.record.as_ref().unwrap().entries[0].costume,
        Costume::Pirate
    );
    let at = p.race.racers[0].ape.pos;
    assert_eq!(
        scene::racer_pose(&p.race, 0, at, 0).gear,
        models::Gear::Pirate
    );
    assert_eq!(
        scene::racer_pose(&p.race, 1, at, 0).gear,
        models::Gear::None
    );
    // "none" on the Ski Slope: no skis
    a.act("menu.quit", "");
    a.act("menu.main", "");
    a.act("menu.race", "");
    a.act("setup.costume", "none");
    a.act("setup.map", "ski");
    a.act("menu.start", "");
    let p = a.play.as_ref().unwrap();
    let at = p.race.racers[0].ape.pos;
    assert_eq!(
        scene::racer_pose(&p.race, 0, at, 0).gear,
        models::Gear::None
    );
    assert_eq!(scene::racer_pose(&p.race, 1, at, 0).gear, models::Gear::Ski);
}

#[test]
fn recordings_keep_the_costume_and_old_ones_read_as_the_maps_own() {
    let mut a = app();
    a.settings.unlock(Costume::Explorer);
    a.act("menu.race", "");
    a.act("setup.costume", "explorer");
    a.act("menu.start", "");
    for _ in 0..200 {
        a.tick();
    }
    let mut p = a.play.take().unwrap();
    let mut rec = p.record.take().unwrap();
    rec.finish = Some(9_000);
    let mut lib = replay::Library::default();
    lib.add(rec.clone());
    let text = lib.text();
    let back = replay::Library::parse(&text);
    let got = back.best.iter().chain(&back.recent).next().unwrap();
    assert_eq!(got.entries[0].costume, Costume::Explorer);
    assert_eq!(got.entries[1].costume, Costume::Own);
    let old = text
        .replace("\"costume\":\"explorer\",", "")
        .replace(",\"costume\":\"explorer\"", "");
    assert!(!old.contains("costume"), "the map's own is not written");
    let back = replay::Library::parse(&old);
    let got = back.best.iter().chain(&back.recent).next().unwrap();
    assert_eq!(got.entries[0].costume, Costume::Own);
}

#[test]
fn the_lobby_offers_the_costumes_and_shows_each_seats() {
    let mut a = app();
    a.settings.unlock(Costume::Space);
    a.act("menu.online", "");
    a.act("online.host", "");
    a.act("lobby.costume", "space");
    assert_eq!(a.settings.costume, Costume::Space);
    let ui = ui_of(&mut a);
    assert_eq!(item(&ui, "lobby.costume").unwrap()["value"], "space");
    let seat = a.online.session.as_ref().unwrap().lobby().seats[0].clone();
    assert_eq!(seat.costume, Costume::Space);
    let table = ui["panels"][0]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["type"] == "table")
        .cloned()
        .unwrap();
    assert_eq!(table["rows"][0][1], "Chimpanzee (Flight jacket)");
    // a locked one is refused here too
    a.act("lobby.costume", "pirate");
    assert_eq!(
        a.online.session.as_ref().unwrap().lobby().seats[0].costume,
        Costume::Space
    );
}

#[test]
fn every_costume_draws_on_every_map() {
    // the body-changing gear (skis, the metal limb, the helmets) draws on
    // every map's ape, swinging and standing, without a stray part
    for c in Costume::ALL {
        for map in Map::ALL {
            for rope in [None, Some(crate::geom::v3(0, ONE, 0))] {
                let pose = models::Pose {
                    gear: c.gear(map),
                    rope,
                    grounded: rope.is_none(),
                    ..models::Pose::standing(V3::ZERO, 0, 0)
                };
                for kind in Kind::ALL {
                    let mut b = Batch::new();
                    models::draw_in(&mut b, kind, 0, &pose);
                    let (v, _) = b.finish(render::DYNAMIC_BUFFER);
                    assert!(!v.is_empty(), "{c:?} {map:?} {kind:?}");
                    for s in v.chunks(render::INSTANCE_VALUES) {
                        let at = v3(s[3], s[7], s[11]);
                        assert!(at.dist(V3::ZERO) < int(4), "{c:?} {kind:?}: far part");
                    }
                }
            }
        }
    }
}
