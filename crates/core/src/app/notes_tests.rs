//! What's New (the patch notes panel): the menu entry and its "New"
//! marker, the version in the corner, and the minor-patches tick.

use serde_json::Value;

use super::{App, Screen, VERSION};

fn app_with(saved: &str) -> App {
    App::new(include_str!("../../../../data/strings.json"), saved, 7).unwrap()
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

fn versions(ui: &Value) -> Vec<String> {
    item(ui, "notes.list").unwrap()["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["version"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn new_marker_until_this_version_is_opened() {
    let mut a = app_with("");
    let b = item(&ui_of(&mut a), "menu.notes").unwrap();
    assert_eq!(b["badge"], "New", "a player who never opened the notes");
    a.act("menu.notes", "");
    assert_eq!(a.screen, Screen::Notes);
    let ui = ui_of(&mut a);
    let saved: Value = serde_json::from_str(ui["save"].as_str().unwrap()).unwrap();
    assert_eq!(
        saved["notes_seen"], VERSION,
        "opening the notes is remembered"
    );
    a.act("menu.back", "");
    let b = item(&ui_of(&mut a), "menu.notes").unwrap();
    assert!(b.get("badge").is_none(), "seen: no marker");
}

#[test]
fn a_minor_update_lights_new_too() {
    // seen the major version before this one's patch
    let mut a = app_with(r#"{"notes_seen":"1.10.0"}"#);
    let b = item(&ui_of(&mut a), "menu.notes").unwrap();
    assert_eq!(b["badge"], "New");
    let mut a = app_with(&format!(r#"{{"notes_seen":"{VERSION}"}}"#));
    assert!(item(&ui_of(&mut a), "menu.notes")
        .unwrap()
        .get("badge")
        .is_none());
}

#[test]
fn minor_patches_hidden_until_ticked_and_remembered() {
    let mut a = app_with("");
    a.act("menu.notes", "");
    let ui = ui_of(&mut a);
    assert_eq!(item(&ui, "notes.minor").unwrap()["value"], false);
    let shown = versions(&ui);
    assert!(shown.contains(&"v1.10.0".to_string()));
    assert!(
        shown.iter().all(|v| v.ends_with(".0")),
        "only major versions by default: {shown:?}"
    );
    a.act("notes.minor", "on");
    let ui = ui_of(&mut a);
    let all = versions(&ui);
    assert_eq!(all[0], format!("v{VERSION}"), "newest first");
    assert!(all.len() > shown.len());
    let saved: Value = serde_json::from_str(ui["save"].as_str().unwrap()).unwrap();
    assert_eq!(saved["notes_minor"], true);
    let mut b = app_with(ui["save"].as_str().unwrap());
    b.act("menu.notes", "");
    assert_eq!(versions(&ui_of(&mut b)), all, "the tick is kept");
}

#[test]
fn the_version_opens_the_notes_from_the_menus_only() {
    let mut a = app_with("");
    assert_eq!(ui_of(&mut a)["version_opens"], true);
    a.act("version", "");
    assert_eq!(a.screen, Screen::Notes);
    a.act("menu.back", "");
    a.act("menu.race", "");
    assert_eq!(ui_of(&mut a)["version_opens"], false);
    a.act("version", "");
    assert_eq!(a.screen, Screen::Setup, "not from race setup");
}
