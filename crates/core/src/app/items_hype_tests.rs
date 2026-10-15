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

/// Ticks `a` `n` times, the local ape pushed along at `kmh(k)` km/h on
/// tick k (flying level), collecting the hype sounds and the largest flash
/// (Q16), which the HUD must carry.
fn speed_up(a: &mut App, n: u32, kmh: impl Fn(u32) -> i32) -> (Vec<String>, i32) {
    let mut sounds = Vec::new();
    let mut flash = 0;
    for k in 0..n {
        let p = a.play.as_mut().unwrap();
        let me = &mut p.race.racers[p.local].ape;
        me.vel = crate::geom::v3(0, 0, crate::fx::int(kmh(k)) * 10 / 36);
        me.mode = crate::ape::Mode::Flying;
        me.pos.y = crate::course::ground(me.pos.x, me.pos.z) + crate::fx::int(30);
        a.tick();
        let ui = ui_of(a);
        for s in ui["sounds"].as_array().cloned().unwrap_or_default() {
            let s = s.as_str().unwrap().to_string();
            if s.starts_with("hype") {
                sounds.push(s);
            }
        }
        let hype = &a.play.as_ref().unwrap().hype;
        let text: Value = serde_json::from_str(&hype.flash_text()).unwrap();
        assert_eq!(ui["hud"]["speedo"]["flash"], text);
        assert_eq!(ui["hud"]["speedo"]["burst"], hype.burst());
        // the screen flash is off (the person, 2026-10-12): the shout pops alone
        assert_eq!(
            ui["hud"]["speedo"]["screen"],
            serde_json::from_str::<Value>("0.00").unwrap(),
            "the screen flashed"
        );
        flash = flash.max(hype.flash());
    }
    (sounds, flash)
}

#[test]
fn speeding_up_calls_out_each_hype_level_once_with_a_flash() {
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.opponents", "0");
    a.act("menu.start", "");
    a.play.as_mut().unwrap().race.phase = Phase::Running;
    let (sounds, flash) = speed_up(&mut a, 30, |_| 0);
    assert!(sounds.is_empty() && flash == 0, "{sounds:?}");
    // up to 110 km/h over four seconds, then held there for four more
    let (sounds, flash) = speed_up(&mut a, 960, |k| (k as i32 * 110 / 480).min(110));
    assert_eq!(sounds, vec!["hype1", "hype2", "hype3"]);
    assert!(
        flash > crate::fx::ONE * 9 / 10,
        "the top call flashes fully: {flash}"
    );
    assert_eq!(a.play.as_ref().unwrap().hype.flash(), 0, "faded");
    // the page can make each call's sound
    for s in sounds {
        assert!(a.sound(&s).len() > 44, "{s}");
    }
}

#[test]
fn a_replay_makes_no_call_outs() {
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.opponents", "0");
    a.act("menu.start", "");
    a.play.as_mut().unwrap().race.phase = Phase::Running;
    speed_up(&mut a, 300, |_| 0);
    a.act("menu.replay", "");
    assert!(a.play.as_ref().unwrap().replay.is_some());
    let (sounds, flash) = speed_up(&mut a, 600, |k| (k as i32).min(110));
    assert!(sounds.is_empty() && flash == 0, "{sounds:?} {flash}");
}

#[test]
fn the_hud_counts_whole_bananas_out_of_twenty() {
    // TRIAGE T60: five pieces on the course make one banana
    let mut a = app();
    a.act("menu.race", "");
    a.act("menu.start", "");
    let p = a.play.as_mut().unwrap();
    let me = p.local;
    p.race.racers[me].ape.bananas = 37;
    let ui = ui_of(&mut a);
    assert_eq!(ui["hud"]["bananas"][1], "7/20");
}
