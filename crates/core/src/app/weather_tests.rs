//! The Ski Slope's Weather setting (2026-10-13): where it lives, that it is
//! kept, and that snow changes only the drawing, only on the Ski Slope.

use serde_json::Value;

use super::*;
use crate::snow::Weather;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

fn ui_of(a: &mut App) -> Value {
    serde_json::from_str(&a.ui()).unwrap()
}

/// The shown menus' items, in order.
fn items(ui: &Value) -> Vec<Value> {
    ui["panels"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|p| p["items"].as_array().cloned().unwrap_or_default())
        .collect()
}

fn item(ui: &Value, id: &str) -> Option<Value> {
    items(ui).into_iter().find(|i| i["id"] == id)
}

#[test]
fn the_weather_is_in_settings_clear_by_default_and_kept_after_a_reload() {
    let mut a = app();
    assert_eq!(a.settings.weather, Weather::Clear);
    a.act("menu.settings", "");
    let ui = ui_of(&mut a);
    let w = item(&ui, "settings.weather").expect("a weather select in Settings");
    assert_eq!(w["label"], "Weather (Ski Slope)");
    assert_eq!(w["value"], "clear");
    let texts: Vec<&str> = w["options"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["text"].as_str().unwrap())
        .collect();
    assert_eq!(texts, ["Clear", "Snowing", "Blizzard"]);
    a.act("settings.weather", "snowing");
    assert_eq!(a.settings.weather, Weather::Snowing);
    let saved = ui_of(&mut a)["save"].as_str().unwrap().to_string();
    let b = App::new(include_str!("../../../../data/strings.json"), &saved, 7).unwrap();
    assert_eq!(b.settings.weather, Weather::Snowing);
    // an unknown word leaves it alone; settings saved before it read Clear
    a.act("settings.weather", "hail");
    assert_eq!(a.settings.weather, Weather::Snowing);
    assert_eq!(Settings::parse("{\"map\":\"ski\"}").weather, Weather::Clear);
    // and a blizzard is kept too (TRIAGE T63)
    a.act("settings.weather", "blizzard");
    assert_eq!(a.settings.weather, Weather::Blizzard);
    let saved = ui_of(&mut a)["save"].as_str().unwrap().to_string();
    let b = App::new(include_str!("../../../../data/strings.json"), &saved, 7).unwrap();
    assert_eq!(b.settings.weather, Weather::Blizzard);
}

#[test]
fn the_ski_slopes_setup_offers_the_weather_beside_the_colours_and_no_other_map_does() {
    for menu in ["menu.race", "menu.trial"] {
        let mut a = app();
        a.act(menu, "");
        assert!(item(&ui_of(&mut a), "setup.weather").is_none(), "{menu}");
        a.act("setup.map", "city");
        assert!(item(&ui_of(&mut a), "setup.weather").is_none(), "{menu}");
        a.act("setup.map", "ski");
        let ui = ui_of(&mut a);
        let all = items(&ui);
        assert_eq!(all[0]["id"], "setup.map");
        assert_eq!(all[2]["id"], "setup.palette");
        assert_eq!(all[3]["id"], "setup.weather", "{menu}: beside the colours");
        assert_eq!(all[3]["label"], "Weather");
        a.act("setup.weather", "snowing");
        assert_eq!(a.settings.weather, Weather::Snowing);
        assert_eq!(
            item(&ui_of(&mut a), "setup.weather").unwrap()["value"],
            "snowing"
        );
    }
}

/// The app on the title over `map`, with `weather`.
fn title_on(map: Map, weather: Weather) -> App {
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.map", map.key());
    a.act("menu.back", "");
    a.settings.weather = weather;
    for _ in 0..30 {
        a.tick();
    }
    a
}

#[test]
fn snow_falls_on_the_ski_slope_only_and_greys_its_sky() {
    // the Jungle and the City are untouched by the setting
    for map in [Map::Jungle, Map::City] {
        let clear = title_on(map, Weather::Clear).frame();
        let snow = title_on(map, Weather::Snowing).frame();
        assert!(clear == snow, "{map:?} changed by the weather");
    }
    let clear = title_on(Map::Ski, Weather::Clear);
    let snow = title_on(Map::Ski, Weather::Snowing);
    assert!(snow.snowing() && !clear.snowing());
    let (cv, ci, _) = clear.frame();
    let (sv, si, _) = snow.frame();
    let flakes = (si.len() - ci.len()) / render::INSTANCE_VALUES;
    assert!(flakes > 300, "{flakes} flakes on the title");
    assert_ne!(cv, sv, "the sky and the fog change in the snow");
    // and in the setup's preview of the Ski Slope too
    let mut a = title_on(Map::Ski, Weather::Snowing);
    a.act("menu.race", "");
    assert!(a.snowing());
}

#[test]
fn a_blizzard_is_thicker_whiter_and_hides_the_mountains_on_the_ski_slope_only() {
    // TRIAGE T63: the Jungle and the City are untouched by it too
    for map in [Map::Jungle, Map::City] {
        let clear = title_on(map, Weather::Clear).frame();
        let storm = title_on(map, Weather::Blizzard).frame();
        assert!(clear == storm, "{map:?} changed by a blizzard");
    }
    let (_, ci, _) = title_on(Map::Ski, Weather::Clear).frame();
    let (sv, si, _) = title_on(Map::Ski, Weather::Snowing).frame();
    let storm = title_on(Map::Ski, Weather::Blizzard);
    assert!(storm.snowing());
    let (bv, bi, _) = storm.frame();
    let flakes = |v: &Vec<i32>| (v.len() - ci.len()) / render::INSTANCE_VALUES;
    assert!(
        flakes(&bi) > flakes(&si) * 3 / 2,
        "{} flakes in the blizzard, {} in the snow",
        flakes(&bi),
        flakes(&si)
    );
    assert_ne!(bv, sv, "a blizzard's sky is not the snow's");
    // no far mountain shows: every backdrop glow is hazed into the fog
    let glows = |v: &Vec<i32>| -> Vec<i32> {
        v.chunks(render::INSTANCE_VALUES)
            .map(|s| s[15])
            .filter(|g| (ONE * 5 / 2..ONE * 7 / 2).contains(g))
            .collect()
    };
    assert!(
        glows(&ci).iter().any(|g| *g < ONE * 3),
        "mountains in clear air"
    );
    let buried = glows(&bi);
    assert!(!buried.is_empty());
    assert!(
        buried.iter().all(|g| *g > ONE * 7 / 2 - ONE / 50),
        "a mountain shows through the blizzard"
    );
}

#[test]
fn snow_changes_nothing_in_a_race_but_the_drawing() {
    let run = |weather: Weather| {
        let mut a = app();
        a.act("menu.race", "");
        a.act("setup.map", "ski");
        a.act("setup.weather", weather.key());
        a.act("setup.opponents", "3");
        a.act("menu.start", "");
        for _ in 0..crate::race::COUNTDOWN as u32 + 600 {
            a.tick();
        }
        a
    };
    let clear = run(Weather::Clear);
    let snow = run(Weather::Snowing);
    let blizzard = run(Weather::Blizzard);
    let at = |a: &App| -> Vec<(V3, V3)> {
        let p = a.play.as_ref().unwrap();
        assert_eq!(p.race.course.name, "ski");
        p.race
            .racers
            .iter()
            .map(|r| (r.ape.pos, r.ape.vel))
            .collect()
    };
    assert_eq!(at(&clear), at(&snow), "the same race in any weather");
    assert_eq!(at(&clear), at(&blizzard), "the same race in a blizzard");
    assert!(snow.snowing());
    assert!(snow.frame().1.len() > clear.frame().1.len());
}

#[test]
fn on_the_ski_slope_the_racers_wear_their_gear_and_elsewhere_none() {
    for map in Map::ALL {
        let mut a = app();
        a.act("menu.race", "");
        a.act("setup.map", map.key());
        a.act("menu.start", "");
        a.tick();
        let p = a.play.as_ref().unwrap();
        let pose = scene::racer_pose(&p.race, 0, p.race.racers[0].ape.pos, 0);
        let want = match map {
            Map::Ski => models::Gear::Ski,
            Map::Sector => models::Gear::Space,
            Map::Ruins => models::Gear::Explorer,
            _ => models::Gear::None,
        };
        assert_eq!(pose.gear, want, "{map:?}");
    }
}
