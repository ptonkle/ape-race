//! The frame and the view model: colourblind looks, the City's glows, each
//! map's light, every screen (moved from app.rs's tests).

use super::*;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

#[test]
fn the_colourblind_mode_mutes_the_world_and_keeps_the_vines_vivid() {
    // TRIAGE T33: a setting, kept; the frame tells the shader to mute
    // and marks the vines' draws so they stay vivid
    let mut a = app();
    let vine_draws = |a: &App| {
        let (view, _, draws) = a.frame();
        let marked = draws
            .chunks(render::DRAW_VALUES)
            .filter(|d| d[6] == render::VINES)
            .count();
        (view[37], marked)
    };
    let (mute, marked) = vine_draws(&a);
    assert_eq!(mute, 0);
    assert!(marked > 0, "the title's vines are drawn as vines");
    a.act("settings.colourblind", "on");
    assert!(a.settings.colourblind);
    let ui: Value = serde_json::from_str(&a.ui()).unwrap();
    assert!(ui["save"]
        .as_str()
        .unwrap()
        .contains("\"colourblind\":true"));
    assert_eq!(vine_draws(&a).0, ONE);
    a.act("menu.race", "");
    a.act("menu.start", "");
    let (mute, marked) = vine_draws(&a);
    assert_eq!(mute, ONE);
    assert!(marked > 0);
    // the minimap's apes are told apart by shape, each kind its own
    let ui: Value = serde_json::from_str(&a.ui()).unwrap();
    let apes = ui["hud"]["minimap"]["apes"].as_array().unwrap();
    let corners: Vec<usize> = apes
        .iter()
        .map(|d| d["shape"].as_array().map_or(0, |s| s.len()))
        .collect();
    assert!(corners.iter().all(|n| *n >= 6), "{corners:?}");
    let shapes: Vec<Vec<i32>> = crate::rules::Kind::ALL
        .iter()
        .map(|k| minimap_shape(*k, 0, 0, 30))
        .collect();
    for (i, s) in shapes.iter().enumerate() {
        assert!(!shapes[..i].contains(s), "kind {i} shares a shape");
    }
    // the sky is greyer
    let (view, _, _) = a.frame();
    let sky = &view[24..27];
    assert!(sky[1] - sky[0] < (scene::SKY[1] - scene::SKY[0]) / 2);
    // and the setting shows on the settings screen
    a.act("menu.quit", "");
    a.screen = Screen::Settings;
    assert!(a.ui().contains("settings.colourblind"));
}

#[test]
fn the_citys_wire_glows_are_drawn_last_and_the_jungle_has_none() {
    // the person (2026-10-12): soft glows round the City's wires, added
    // over everything (so drawn after every solid shape)
    let mut a = app();
    let kinds = |a: &App| -> Vec<u32> {
        a.frame()
            .2
            .chunks(render::DRAW_VALUES)
            .map(|d| d[6])
            .collect()
    };
    assert!(!kinds(&a).contains(&render::HALO), "the Jungle's title");
    a.act("menu.race", "");
    a.act("menu.start", "");
    assert!(!kinds(&a).contains(&render::HALO), "a Jungle race");
    let mut a = app();
    a.act("flag.map", "city");
    for stage in ["title", "race"] {
        if stage == "race" {
            a.act("menu.race", "");
            a.act("menu.start", "");
        }
        let k = kinds(&a);
        let first = k.iter().position(|d| *d == render::HALO);
        let first = first.unwrap_or_else(|| panic!("no glows on the City's {stage}"));
        assert!(
            k[first..].iter().all(|d| *d == render::HALO),
            "{stage}: {k:?}"
        );
    }
}

#[test]
fn the_frame_carries_its_maps_light() {
    // the City at dusk, the Jungle at midday (`scene::look`)
    let mut a = app();
    let light = |a: &App| a.frame().0[38..47].to_vec();
    let j = scene::look(crate::map::Map::Jungle);
    assert_eq!(
        light(&a)[0..3],
        [j.light_dir.x, j.light_dir.y, j.light_dir.z]
    );
    a.act("flag.map", "city");
    let c = scene::look_of(&crate::map::Map::City.course());
    let v = light(&a);
    assert_eq!(v[0..3], [c.light_dir.x, c.light_dir.y, c.light_dir.z]);
    assert_eq!(v[3..6], c.light);
    assert_eq!(v[6..9], c.ground);
    assert_eq!(a.frame().0[21..24], c.fog);
    a.act("menu.race", "");
    a.act("menu.start", "");
    assert_eq!(light(&a)[3..6], c.light);
}

#[test]
fn every_screen_has_a_view_model_and_a_frame() {
    let mut a = app();
    for id in [
        "menu.race",
        "menu.back",
        "menu.settings",
        "menu.back",
        "screen.lineup",
    ] {
        a.act(id, "");
        let ui: Value = serde_json::from_str(&a.ui()).unwrap();
        assert!(ui["screen"].is_string());
        let (view, _, _) = a.frame();
        assert_eq!(view.len(), render::VIEW_VALUES);
        let (statics, _, _) = a.static_scene();
        assert!(!statics.is_empty());
    }
    a.act("menu.back", "");
    a.act("menu.race", "");
    a.act("menu.start", "");
    let ui: Value = serde_json::from_str(&a.ui()).unwrap();
    assert!(ui["hud"]["minimap"]["apes"].is_array());
    let (_, dynamic, _) = a.frame();
    assert!(!dynamic.is_empty());
}
