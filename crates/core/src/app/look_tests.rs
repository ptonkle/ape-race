//! The racers' looks: an aura round each in its slot's colour (TRIAGE
//! T59) and each ape's colour palette (T49).

use serde_json::Value;

use super::*;
use crate::mesh::Mesh;
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

/// The frame's auras: each glow drawn in the halo pass with a negative
/// strength, as (colour, strength, centre).
fn auras(a: &App) -> Vec<(Rgb, i32, V3)> {
    let (_, values, draws) = a.frame();
    let mut out = Vec::new();
    for d in draws.chunks(render::DRAW_VALUES) {
        if d[6] != render::HALO || d[1] != Mesh::Sphere as u32 {
            continue;
        }
        let from = d[2] as usize / render::BYTES_PER_VALUE;
        for k in 0..d[3] as usize {
            let s = &values[from + k * render::INSTANCE_VALUES..][..render::INSTANCE_VALUES];
            if s[15] < 0 {
                out.push(([s[12], s[13], s[14]], -s[15], v3(s[3], s[7], s[11])));
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

// the player's own aura is the subtler
const _: () = assert!(scene::AURA_OWN < scene::AURA_GLOW);

#[test]
fn every_racer_glows_in_its_slots_colour_on_every_map() {
    for map in ["jungle", "city", "ski"] {
        let mut a = race_on(map, "3");
        // a little into the countdown, everyone on the platform
        run(&mut a, 30);
        let got = auras(&a);
        assert_eq!(got.len(), 4, "{map}: one aura a racer");
        let p = a.play.as_ref().unwrap();
        for (k, r) in p.race.racers.iter().enumerate() {
            let (colour, strength, at) = got
                .iter()
                .copied()
                .find(|g| g.0 == scene::AURA[k])
                .unwrap_or_else(|| panic!("{map}: no aura in slot {k}'s colour"));
            // round its own ape
            assert!(
                at.dist(r.ape.pos) < int(3),
                "{map} {k}: {at:?} far from the ape"
            );
            let want = if k == p.local {
                scene::AURA_OWN
            } else {
                scene::AURA_GLOW
            };
            assert_eq!(strength, want, "{map} {k}: {colour:?}");
        }
    }
}

#[test]
fn the_auras_are_blue_red_green_purple_and_change_for_colourblind_eyes() {
    let hue = |c: Rgb| {
        let m = *c.iter().max().unwrap();
        c.map(|v| v * 4 / m.max(1))
    };
    // blue, red, green, purple: the strongest channel (and purple's red
    // and blue both strong)
    assert_eq!(hue(scene::AURA[0])[2], 4);
    assert_eq!(hue(scene::AURA[1])[0], 4);
    assert_eq!(hue(scene::AURA[2])[1], 4);
    let purple = hue(scene::AURA[3]);
    assert!(
        purple[2] == 4 && purple[0] >= 2 && purple[1] < 2,
        "{purple:?}"
    );
    let mut a = race_on("jungle", "3");
    a.act("settings.colourblind", "on");
    let colours: Vec<Rgb> = auras(&a).into_iter().map(|g| g.0).collect();
    for c in scene::AURA_BLIND {
        assert!(colours.contains(&c), "{c:?}");
    }
    for c in scene::AURA {
        assert!(!colours.contains(&c), "{c:?}");
    }
}

#[test]
fn the_aura_wraps_the_ape_whatever_it_is_doing() {
    // the shell holds most parts of the drawn ape (three in four: a tail,
    // a finger or a gibbon's raised arm may poke out where the glow has
    // faded anyway): swinging, flying
    // and on the ground, for every kind, going the way it faces
    use crate::models::{draw, Pose};
    let yaw = deg(30);
    let ahead = crate::geom::facing(yaw, 0);
    let poses = [
        Pose {
            rope: Some(V3::UP.sub(ahead.scale(ONE / 5)).norm()),
            grounded: false,
            vel: ahead.scale(int(12)),
            ..Pose::standing(v3(0, int(5), 0), yaw, 3)
        },
        Pose {
            grounded: false,
            vel: ahead.scale(int(10)).add(v3(0, int(3), 0)),
            ..Pose::standing(v3(0, int(5), 0), yaw, 3)
        },
        Pose {
            vel: ahead.scale(int(6)),
            ..Pose::standing(V3::ZERO, yaw, 3)
        },
    ];
    for kind in Kind::ALL {
        for pose in poses {
            let pose = Pose {
                scale: scene::RACE_SCALE,
                ..pose
            };
            let mut b = Batch::new();
            scene::aura(&mut b, kind, &pose, scene::AURA[0], scene::AURA_GLOW);
            let (shell, _) = b.take_halos().finish(render::DYNAMIC_BUFFER);
            let m = crate::geom::Affine {
                c0: v3(shell[0], shell[4], shell[8]),
                c1: v3(shell[1], shell[5], shell[9]),
                c2: v3(shell[2], shell[6], shell[10]),
                t: v3(shell[3], shell[7], shell[11]),
            };
            let mut ape = Batch::new();
            draw(&mut ape, kind, &pose);
            let (v, _) = ape.finish(render::DYNAMIC_BUFFER);
            let parts: Vec<V3> = v
                .chunks(render::INSTANCE_VALUES)
                .map(|s| v3(s[3], s[7], s[11]))
                .collect();
            let inside = parts
                .iter()
                .filter(|p| {
                    // in the unit sphere's space (radius a half)
                    let d = p.sub(m.t);
                    let local = v3(
                        crate::fx::div(d.dot(m.c0), m.c0.dot(m.c0)),
                        crate::fx::div(d.dot(m.c1), m.c1.dot(m.c1)),
                        crate::fx::div(d.dot(m.c2), m.c2.dot(m.c2)),
                    );
                    local.len() <= ONE / 2
                })
                .count();
            assert!(
                inside * 100 >= parts.len() * 75,
                "{kind:?} {:?}: {inside} of {} parts inside",
                (pose.rope.is_some(), pose.grounded),
                parts.len()
            );
            // and it is not much bigger than the ape: the glow stays round it
            assert!(m.c1.len() < int(5) && m.c0.len() < int(5));
        }
    }
}

#[test]
fn the_auras_can_be_turned_off_and_the_setting_is_kept() {
    let mut a = race_on("jungle", "2");
    assert!(Settings::default().aura, "on by default");
    a.act("settings.aura", "off");
    assert!(auras(&a).is_empty());
    let kept = Settings::parse(&serde_json::to_string(&a.settings).unwrap());
    assert!(!kept.aura);
    // settings saved before auras read as on
    assert!(Settings::parse("{\"volume\":40}").aura);
    a.act("menu.quit", "");
    a.act("menu.settings", "");
    let ui = ui_of(&mut a);
    assert_eq!(item(&ui, "settings.aura").unwrap()["value"], false);
}

#[test]
fn the_minimap_dots_wear_the_aura_colours() {
    let mut a = race_on("jungle", "3");
    let ui = ui_of(&mut a);
    let dots: Vec<String> = ui["hud"]["minimap"]["apes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["colour"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(dots, ["#3c8cff", "#ff3228", "#32eb46", "#b946ff"]);
    a.act("settings.aura", "off");
    let ui = ui_of(&mut a);
    assert_eq!(ui["hud"]["minimap"]["apes"][0]["colour"], "#6b4a30");
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
