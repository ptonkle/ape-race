//! The race setup's item checkboxes (TRIAGE T42) and the hype call-outs
//! (T22), through the app.

use serde_json::Value;

use super::{App, Screen, Settings};
use crate::items::ItemSet;
use crate::race::{Item, Phase};

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

fn ui_of(a: &mut App) -> Value {
    serde_json::from_str(&a.ui()).unwrap()
}

/// The setup's item checkboxes, as (id, ticked).
fn checks(ui: &Value) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for panel in ui["panels"].as_array().unwrap() {
        for item in panel["items"].as_array().unwrap() {
            if item["type"] == "checks" {
                for o in item["options"].as_array().unwrap() {
                    out.push((
                        o["id"].as_str().unwrap().to_string(),
                        o["value"].as_bool().unwrap(),
                    ));
                }
            }
        }
    }
    out
}

#[test]
fn unticked_items_stay_out_of_the_race_and_the_choice_is_kept() {
    let mut a = app();
    a.act("menu.race", "");
    assert_eq!(a.screen, Screen::Setup);
    let ui = ui_of(&mut a);
    let boxes = checks(&ui);
    assert_eq!(boxes.len(), Item::ALL.len(), "{boxes:?}");
    assert!(boxes.iter().all(|(_, on)| *on));
    a.act("setup.item.banana", "off");
    a.act("setup.item.hook", "off");
    let ui = ui_of(&mut a);
    let saved = ui["save"].as_str().expect("settings saved");
    let kept = Settings::parse(saved);
    assert!(!kept.items.has(Item::Banana) && !kept.items.has(Item::Hook));
    assert!(kept.items.has(Item::Stick) && kept.items.has(Item::Vine));
    assert_eq!(checks(&ui).iter().filter(|(_, on)| !on).count(), 2);
    a.act("menu.start", "");
    let p = a.play.as_ref().unwrap();
    assert_eq!(p.race.items, kept.items);
    // the replay keeps the set too
    assert_eq!(p.record.as_ref().unwrap().items, kept.items);
}

#[test]
fn with_every_item_unticked_the_setup_says_so_and_the_race_has_none() {
    let mut a = app();
    a.act("menu.race", "");
    for item in Item::ALL {
        a.act(&format!("setup.item.{}", item.key()), "off");
    }
    let ui = ui_of(&mut a);
    assert!(ui.to_string().contains("No items this race"), "{ui}");
    a.act("menu.start", "");
    let p = a.play.as_ref().unwrap();
    assert!(p.race.items.is_empty());
}

#[test]
fn a_time_trial_shows_no_item_boxes_and_keeps_boost_only() {
    let mut a = app();
    a.settings.items = ItemSet::NONE;
    a.act("menu.trial", "");
    let ui = ui_of(&mut a);
    assert!(checks(&ui).is_empty());
    a.act("menu.start", "");
    let p = a.play.as_mut().unwrap();
    p.race.phase = Phase::Running;
    assert_eq!(p.race.items, ItemSet::only(Item::Boost));
}

#[test]
fn stored_settings_with_stray_bits_are_cleaned() {
    let s = Settings::parse(r#"{"items": 255}"#);
    assert_eq!(s.items, ItemSet::ALL);
    let s = Settings::parse(r#"{"volume": 40}"#);
    assert_eq!(s.items, ItemSet::ALL, "older settings allow every item");
}

#[test]
fn a_kept_replay_keeps_its_item_set_and_old_ones_read_as_before() {
    use crate::replay::Library;
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.item.boost", "off");
    a.act("menu.start", "");
    let mut rec = a.play.as_ref().unwrap().record.clone().unwrap();
    rec.controls = vec![Default::default(); 3];
    rec.finish = Some(3);
    let mut lib = Library::default();
    lib.add(rec.clone());
    let text = lib.text();
    let back = Library::parse(&text);
    let got = back.best.iter().chain(&back.recent).next().expect("kept");
    assert_eq!(got.items, rec.items);
    assert!(!got.items.has(Item::Boost));
    // a recording stored before item sets: "items" missing, a Time Trial
    // known by its old flag
    let old = text.replace(&format!(",\"items\":{}", rec.items.0), "");
    assert_ne!(old, text);
    let back = Library::parse(&old);
    let got = back.best.iter().chain(&back.recent).next().unwrap();
    assert_eq!(got.items, ItemSet::ALL);
    let trial = old.replace("\"boost_only\":false", "\"boost_only\":true");
    let back = Library::parse(&trial);
    let got = back.best.iter().chain(&back.recent).next().unwrap();
    assert_eq!(got.items, ItemSet::only(Item::Boost));
}
