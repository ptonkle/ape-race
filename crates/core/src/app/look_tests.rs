//! The racers' looks: each ape's colour palette (T49), and no aura round
//! them (T59's auras, taken out again by T73).

use serde_json::Value;

use super::*;
use crate::models::PALETTES;
use crate::render::Rgb;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

fn ui_of(a: &mut App) -> Value {
    serde_json::from_str(&a.ui()).unwrap()
}

fn item(ui: &Value, id: &str) -> Option<Value> {
    ui["panels"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|p| p["items"].as_array().cloned().unwrap_or_default())
        .find(|i| i["id"] == id)
}

/// The frame's auras, as T59 drew them: rim glows (a negative strength in
/// the halo pass) round a racer, as (colour, strength). (Sector Ape's black
/// hole has one too, far off.)
fn auras(a: &App) -> Vec<(Rgb, i32)> {
    let (_, values, draws) = a.frame();
    let racers: Vec<V3> = a
        .play
        .as_ref()
        .unwrap()
        .race
        .racers
        .iter()
        .map(|r| r.ape.pos)
        .collect();
    let mut out = Vec::new();
    for d in draws.chunks(render::DRAW_VALUES) {
        if d[6] != render::HALO {
            continue;
        }
        let from = d[2] as usize / render::BYTES_PER_VALUE;
        for k in 0..d[3] as usize {
            let s = &values[from + k * render::INSTANCE_VALUES..][..render::INSTANCE_VALUES];
            let at = v3(s[3], s[7], s[11]);
            if s[15] < 0 && racers.iter().any(|r| r.dist(at) < int(10)) {
                out.push(([s[12], s[13], s[14]], -s[15]));
            }
        }
    }
    out
}

/// `frames` page frames of about 1/60 s.
fn run(a: &mut App, frames: i64) {
    for f in 1..=frames {
        a.advance(f * 16_667);
    }
}

fn race_on(map: &str, opponents: &str) -> App {
    let mut a = app();
    a.act("setup.map", map);
    a.act("menu.race", "");
    a.act("setup.opponents", opponents);
    a.act("menu.start", "");
    a
}

#[test]
fn no_racer_wears_an_aura_and_the_minimap_dots_are_the_kinds() {
    // TRIAGE T73: the auras (T59) are gone, on every map, in either mode,
    // and with them the setting
    for map in ["jungle", "city", "ski", "sector"] {
        for blind in ["off", "on"] {
            let mut a = race_on(map, "3");
            a.act("settings.colourblind", blind);
            run(&mut a, 30);
            assert_eq!(auras(&a), [], "{map}, colourblind {blind}");
        }
    }
    let mut a = race_on("jungle", "3");
    let ui = ui_of(&mut a);
    let p = a.play.as_ref().unwrap();
    for (k, r) in p.race.racers.iter().enumerate() {
        assert_eq!(
            ui["hud"]["minimap"]["apes"][k]["colour"],
            super::colour_of(r.ape.kind),
            "dot {k}"
        );
    }
    a.act("menu.quit", "");
    a.act("menu.settings", "");
    let ui = ui_of(&mut a);
    assert!(item(&ui, "settings.aura").is_none());
    // settings saved with the auras still read
    let old = Settings::parse("{\"volume\":40,\"aura\":true}");
    assert_eq!(old.volume, 40);
}

#[test]
fn an_ape_wears_its_chosen_palette_kept_per_kind() {
    let mut a = app();
    a.act("menu.race", "");
    let ui = ui_of(&mut a);
    let select = item(&ui, "setup.palette").expect("beside the ape picker");
    assert_eq!(select["value"], "0");
    let names: Vec<&str> = select["options"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["text"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Classic", "Bonobo", "Russet", "Golden"]);
    a.act("setup.palette", "2");
    assert_eq!(a.settings.palette(), 2);
    // each kind keeps its own
    a.act("setup.ape", "gorilla");
    assert_eq!(a.settings.palette(), 0);
    a.act("setup.palette", "3");
    a.act("setup.ape", "chimpanzee");
    assert_eq!(a.settings.palette(), 2);
    // kept in the settings, nonsense dropped
    let kept = Settings::parse(&serde_json::to_string(&a.settings).unwrap());
    assert_eq!(kept.palette_of(Kind::Gorilla), 3);
    assert_eq!(kept.palette_of(Kind::Chimpanzee), 2);
    let odd = Settings::parse("{\"palettes\":{\"gorilla\":9,\"yeti\":1}}");
    assert!(odd.palettes.is_empty());
    a.act("setup.palette", "7");
    assert_eq!(a.settings.palette(), 0, "past the last: the classic");
    a.act("setup.palette", "2");
    // the race and its recording carry it
    a.act("menu.start", "");
    let p = a.play.as_ref().unwrap();
    assert_eq!(p.race.racers[0].palette, 2);
    assert_eq!(p.record.as_ref().unwrap().entries[0].palette, 2);
    assert!(p.race.racers[1..].iter().all(|r| r.palette == 0));
    // and the ape is drawn in it
    let fur = |palette| {
        let mut b = Batch::new();
        let pose = scene::racer_pose(&p.race, 0, p.race.racers[0].ape.pos, 0);
        models::draw_in(&mut b, Kind::Chimpanzee, palette, &pose);
        b.finish(render::DYNAMIC_BUFFER).0
    };
    let (_, values, _) = a.frame();
    let has = |v: &[i32], w: &[i32]| {
        let first = &w[..render::INSTANCE_VALUES];
        v.chunks(render::INSTANCE_VALUES)
            .any(|s| s[12..15] == first[12..15])
    };
    assert!(has(&values, &fur(2)) && !has(&values, &fur(1)));
}

#[test]
fn the_lobby_offers_the_palettes_too() {
    let mut a = app();
    a.act("menu.online", "");
    a.act("online.host", "");
    a.act("lobby.palette", "1");
    assert_eq!(a.settings.palette(), 1);
    let ui = ui_of(&mut a);
    assert_eq!(item(&ui, "lobby.palette").unwrap()["value"], "1");
    let seat = a.online.session.as_ref().unwrap().lobby().seats[0].clone();
    assert_eq!(seat.palette, 1);
    let table = ui["panels"][0]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["type"] == "table")
        .cloned()
        .unwrap();
    assert_eq!(table["rows"][0][1], "Chimpanzee (Bonobo)");
    // another ape: its own colours
    a.act("lobby.ape", "lemur");
    assert_eq!(
        a.online.session.as_ref().unwrap().lobby().seats[0].palette,
        0
    );
}

#[test]
fn an_ai_ape_of_a_kind_already_racing_wears_other_colours() {
    let e = |kind, palette| crate::race::Entry {
        name: String::new(),
        kind,
        driver: Driver::Ai,
        palette,
    };
    let taken = [e(Kind::Gorilla, 0), e(Kind::Gorilla, 2), e(Kind::Lemur, 1)];
    let apart = |kind| crate::race::palette_apart(crate::race::worn(&taken), kind);
    assert_eq!(apart(Kind::Gorilla), 1);
    assert_eq!(apart(Kind::Lemur), 0);
    assert_eq!(apart(Kind::Gibbon), 0);
    let all: Vec<_> = (0..PALETTES).map(|p| e(Kind::Gibbon, p)).collect();
    assert_eq!(
        crate::race::palette_apart(crate::race::worn(&all), Kind::Gibbon),
        0,
        "every one taken: the classic"
    );
}

#[test]
fn recordings_keep_the_palettes_and_old_ones_read_as_classic() {
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.palette", "3");
    a.act("menu.start", "");
    run(&mut a, 200);
    let mut p = a.play.take().unwrap();
    let mut rec = p.record.take().unwrap();
    rec.finish = Some(9_000);
    let mut lib = replay::Library::default();
    lib.add(rec.clone());
    let text = lib.text();
    let back = replay::Library::parse(&text);
    let got = back.best.iter().chain(&back.recent).next().unwrap();
    assert_eq!(got.entries[0].palette, 3);
    // the classic is not written, so an old recording reads the same
    let old = text
        .replace("\"palette\":3,", "")
        .replace(",\"palette\":3", "");
    assert!(!old.contains("palette"), "{old}");
    let back = replay::Library::parse(&old);
    let got = back.best.iter().chain(&back.recent).next().unwrap();
    assert_eq!(got.entries[0].palette, 0);
    // a replay races the same apes in the same colours
    let course = a.course_of(Map::Jungle).unwrap();
    assert_eq!(rec.race(course).racers[0].palette, 3);
}
