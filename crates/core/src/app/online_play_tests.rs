//! Two apps joined by an instant transport: the Online screen, the Lobby,
//! an online race, Rematch and leaving.

use serde_json::Value;

use super::super::{App, Screen};

fn app(id: &str) -> App {
    let mut a = App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap();
    a.act("flag.client", id);
    a
}

fn requests(a: &mut App) -> Vec<Value> {
    let ui: Value = serde_json::from_str(&a.ui()).unwrap();
    ui["net_requests"].as_array().cloned().unwrap_or_default()
}

/// Host `H` (app 0) and joiner `J` (app 1).
struct Pair {
    apps: [App; 2],
    now_us: i64,
}

impl Pair {
    fn lobby() -> Pair {
        let mut h = app("h");
        h.act("menu.online", "");
        h.act("online.host", "");
        assert_eq!(h.screen, Screen::Lobby);
        assert_eq!(requests(&mut h)[0]["op"], "host_room");
        let code = h.online.room.clone();
        let mut j = app("j");
        j.act("menu.online", "");
        j.act("online.room", &code.to_lowercase());
        j.act("online.join", "");
        assert_eq!(requests(&mut j)[0]["op"], "join_room");
        h.net_event("connected", "J", "", false);
        j.net_event("connected", "H", "", true);
        let mut pair = Pair {
            apps: [h, j],
            now_us: 0,
        };
        pair.run(1000);
        pair
    }

    /// Runs both apps in 10 ms frames, carrying every message at once.
    fn run(&mut self, ms: i64) {
        for _ in 0..ms / 10 {
            self.now_us += 10_000;
            for k in 0..2 {
                self.apps[k].advance(self.now_us);
                let out: Vec<Value> = serde_json::from_str(&self.apps[k].take_outbox()).unwrap();
                let from = if k == 0 { "H" } else { "J" };
                for o in out {
                    let text = o["text"].as_str().unwrap();
                    self.apps[1 - k].net_event("message", from, text, k == 0);
                }
            }
        }
    }

    fn race(&mut self) {
        self.apps[1].act("lobby.ready", "");
        self.run(200);
        self.apps[0].act("lobby.start", "");
        self.run(500);
    }
}

#[test]
fn two_apps_meet_in_a_lobby_and_race() {
    let mut p = Pair::lobby();
    for a in &p.apps {
        assert_eq!(a.screen, Screen::Lobby);
        let v = a.online.session.as_ref().unwrap().lobby();
        assert_eq!(v.seats.iter().filter(|s| !s.ai).count(), 2);
    }
    // the host's Start waits for the joiner's Ready
    p.apps[0].act("lobby.start", "");
    p.run(200);
    assert_eq!(p.apps[0].screen, Screen::Lobby);
    p.race();
    for (k, a) in p.apps.iter().enumerate() {
        assert_eq!(a.screen, Screen::Race, "app {k}");
        let play = a.play.as_ref().unwrap();
        assert!(play.online);
        assert_eq!(play.local, k);
    }
    let start = p.apps[1].play.as_ref().unwrap().race.racers[0].ape.pos;
    p.apps[0].key("KeyW", true);
    // Esc on the joiner opens the menu, but its race runs on
    p.apps[1].key("Escape", true);
    let tick = p.apps[1].play.as_ref().unwrap().race.tick;
    p.run(5000);
    let jp = p.apps[1].play.as_ref().unwrap();
    assert!(jp.paused && jp.race.tick > tick, "no pause online");
    assert_ne!(jp.race.racers[0].ape.pos, start, "the host's ape moves");
    let ui: Value = serde_json::from_str(&p.apps[1].ui()).unwrap();
    assert_eq!(ui["remote_moved"], true);
}

#[test]
fn rematch_and_leaving() {
    let mut p = Pair::lobby();
    p.race();
    p.run(500);
    p.apps[0].act("menu.rematch", "");
    p.run(1500);
    assert_eq!(p.apps[0].screen, Screen::Lobby);
    assert_eq!(p.apps[1].screen, Screen::Lobby, "the joiner follows");
    // the host leaves: the joiner is told and goes back to Online
    p.apps[0].act("lobby.leave", "");
    assert_eq!(requests(&mut p.apps[0])[0]["op"], "leave");
    assert_eq!(p.apps[0].screen, Screen::Online);
    p.run(300);
    assert_eq!(p.apps[1].screen, Screen::Online);
    assert_eq!(p.apps[1].online.status, "online.dissolved");
}

#[test]
fn a_bad_room_code_is_refused_before_joining() {
    let mut j = app("j");
    j.act("menu.online", "");
    j.act("online.room", "AB0");
    j.act("online.join", "");
    assert_eq!(j.screen, Screen::Online);
    assert_eq!(j.online.status, "online.bad_room");
    assert!(requests(&mut j).is_empty());
}

#[test]
fn an_invite_link_carries_the_room_and_opening_it_joins() {
    // TRIAGE row 27: the lobby offers the room as a link to copy
    let mut h = app("h");
    h.act("flag.page_url", "https://example.test/ape-race/");
    h.act("menu.online", "");
    h.act("online.host", "");
    let code = h.online.room.clone();
    let ui = h.ui();
    let link = format!("https://example.test/ape-race/#room={code}");
    assert!(ui.contains(&link), "the lobby's Copy Link copies {link}");
    assert!(ui.contains("online.copy_link"));
    // a browser opened from it goes straight to that room
    let mut j = app("j");
    j.act("flag.page_url", "https://example.test/ape-race/");
    j.act("flag.room_link", &code);
    assert_eq!(j.online.room, code);
    assert_eq!(requests(&mut j)[0]["op"], "join_room");
    // and a link with nonsense in it does nothing
    let mut k = app("k");
    k.act("flag.room_link", "!!");
    assert_eq!(k.screen, Screen::Title);
}

/// Every sound the app names in its next view.
fn sounds(a: &mut App) -> Vec<String> {
    let ui: Value = serde_json::from_str(&a.ui()).unwrap();
    ui["sounds"]
        .as_array()
        .map(|s| s.iter().map(|k| k.as_str().unwrap().to_string()).collect())
        .unwrap_or_default()
}

#[test]
fn the_lobby_offers_a_name_field_and_a_rename_reaches_the_other_app() {
    // TRIAGE T35: an invite link lands straight in the lobby, so the
    // lobby has the name field too
    let mut p = Pair::lobby();
    for a in &mut p.apps {
        assert!(a.ui().contains("\"id\":\"lobby.name\""));
    }
    p.apps[1].act("lobby.name", "Bonobo Bob");
    p.apps[0].act("lobby.name", "Queen");
    p.run(500);
    for (k, a) in p.apps.iter().enumerate() {
        let seats = a.online.session.as_ref().unwrap().lobby().seats;
        let names: Vec<String> = seats.into_iter().map(|s| s.name).collect();
        assert_eq!(names[..2], ["Queen", "Bonobo Bob"], "app {k}");
    }
    assert_eq!(p.apps[1].settings.name, "Bonobo Bob", "kept for next time");
    // clearing the field gives the default name
    p.apps[1].act("lobby.name", "");
    p.run(500);
    let seats = p.apps[0].online.session.as_ref().unwrap().lobby().seats;
    assert_eq!(seats[1].name, p.apps[0].t("online.default_name"));
}

#[test]
fn the_screech_button_plays_on_both_apps() {
    let mut p = Pair::lobby();
    for a in &mut p.apps {
        sounds(a);
    }
    p.apps[1].act("lobby.screech", "");
    p.run(100);
    for (k, a) in p.apps.iter_mut().enumerate() {
        let s = sounds(a);
        assert_eq!(
            s.iter().filter(|k| *k == "screech").count(),
            1,
            "app {k}: {s:?}"
        );
    }
    let wav = p.apps[0].sound("screech");
    assert_eq!(&wav[0..4], b"RIFF");
    assert!(
        wav.len() > 22_050,
        "about a second and a quarter of screech"
    );
}

#[test]
fn a_supplied_screech_file_plays_at_the_players_volume() {
    // the swap-in path for the person's recording (SECOND-ORDER L4)
    use crate::sound::{scaled_wav, MASTER};
    let peak = |w: &[u8]| {
        w[44..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| (i16::from_le_bytes(*b) as i64).abs())
            .max()
            .unwrap()
    };
    // a file plays scaled as the synthesised cues are: full scale is MASTER
    // at 100%, half that at 50%
    let mut file = crate::sound::screech(100);
    let loud: Vec<u8> = file[44..]
        .as_chunks::<2>()
        .0
        .iter()
        .flat_map(|b| {
            let s = i16::from_le_bytes(*b) as i64 * i16::MAX as i64 / MASTER;
            (s.clamp(-32767, 32767) as i16).to_le_bytes()
        })
        .collect();
    file.truncate(44);
    file.extend(loud);
    let want = peak(&file) * MASTER / i16::MAX as i64;
    let full = peak(&scaled_wav(&file, 100));
    let half = peak(&scaled_wav(&file, 50));
    assert!((full - want).abs() < MASTER / 50, "{full} for {want}");
    assert!(
        (half - want / 2).abs() < MASTER / 50,
        "{half} for {}",
        want / 2
    );
    assert_eq!(
        scaled_wav(b"RIFF....WAVEjunk", 100).len(),
        44,
        "no data: silence"
    );
}

#[test]
fn the_hosts_item_choice_reaches_the_joiner_with_the_start() {
    // TRIAGE T42: the host's lobby has the item boxes; the joiner's race
    // gets the host's set, whatever its own settings say
    let mut p = Pair::lobby();
    let ui = p.apps[0].ui();
    assert!(
        ui.contains("setup.item.vine"),
        "the host's lobby has the boxes"
    );
    p.apps[0].act("setup.item.vine", "off");
    p.apps[0].act("setup.item.stick", "off");
    p.race();
    let host = p.apps[0].play.as_ref().unwrap().race.items;
    let joiner = p.apps[1].play.as_ref().unwrap().race.items;
    assert!(!host.has(crate::race::Item::Vine) && !host.has(crate::race::Item::Stick));
    assert!(host.has(crate::race::Item::Banana));
    assert_eq!(joiner, host);
    assert!(p.apps[1].settings.items.has(crate::race::Item::Vine));
}

#[test]
fn an_online_race_is_recorded_as_a_track_and_replays() {
    // TRIAGE T24: every ape's place, kept and watched like any replay
    let mut p = Pair::lobby();
    p.race();
    p.apps[0].key("KeyW", true);
    p.run(6000);
    let j = &mut p.apps[1];
    let play = j.play.as_ref().unwrap();
    let rec = play.record.as_ref().unwrap();
    let track = rec.track.as_ref().expect("an online race keeps a track");
    assert_eq!(rec.mode, crate::replay::Mode::Online);
    assert!(rec.length() + crate::replay::TRACK_EVERY > play.race.tick);
    let seen = play.race.racers[0].ape.pos;
    assert!(track.samples.len() > 100);
    // watch it: the host's ape (remote here) goes where it went
    j.act("menu.replay", "");
    let rp = j.play.as_ref().unwrap();
    assert!(rp.replay.is_some() && !rp.online);
    let start = rp.race.racers[0].ape.pos;
    let length = rp.replay.as_ref().unwrap().length();
    for _ in 0..length + 10 {
        j.tick();
    }
    let rp = j.play.as_ref().unwrap();
    assert_ne!(rp.race.racers[0].ape.pos, start);
    assert!(rp.race.racers[0].ape.pos.dist(seen) < crate::fx::int(3));
    // leaving goes back to the online race; a finished one is kept
    j.act("replay.exit", "");
    let back = j.play.as_mut().unwrap();
    assert!(back.online);
    back.race.racers[1].finish = Some(back.race.clock);
    j.act("menu.main", "");
    let listed = j.library.listing();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].0.mode, crate::replay::Mode::Online);
}

#[test]
fn the_hosts_map_shows_in_every_lobby_and_the_race_runs_on_it() {
    use crate::map::Map;
    let mut p = Pair::lobby();
    // the joiner's own last map does not count
    p.apps[1].settings.map = Map::Jungle;
    let v = p.apps[1].scene_version();
    p.apps[0].act("lobby.map", "city");
    assert_eq!(p.apps[0].settings.map, Map::City);
    p.run(200);
    let jv = p.apps[1].online.session.as_ref().unwrap().lobby();
    assert_eq!(jv.map, Map::City);
    assert_eq!(p.apps[1].map(), Map::City);
    // the joiner's ape preview moves onto the City at once
    assert!(p.apps[1].scene_version() > v);
    let ui: Value = serde_json::from_str(&p.apps[1].ui()).unwrap();
    let shown = ui["panels"].to_string();
    assert!(shown.contains("City"), "{shown}");
    // a joiner cannot choose
    p.apps[1].act("lobby.map", "jungle");
    p.run(200);
    assert_eq!(p.apps[0].map(), Map::City);
    p.race();
    for (k, a) in p.apps.iter().enumerate() {
        let race = &a.play.as_ref().unwrap().race;
        assert_eq!(race.course.name, "city", "app {k}");
    }
    assert_eq!(p.apps[1].settings.map, Map::Jungle);
}

#[test]
fn floor_is_lava_online_the_jaguar_eats_on_every_browser() {
    // TRIAGE T37: the host's checkbox reaches the joiner (shown in its
    // lobby, and in its race); an ape eaten in its owner's browser is
    // eaten in the other's, where it was, and the results say so
    let mut p = Pair::lobby();
    assert!(
        p.apps[0].ui().contains("lobby.lava"),
        "the host's lobby has the box"
    );
    p.apps[0].act("lobby.lava", "on");
    p.run(1500);
    let shown = p.apps[1].ui();
    assert!(!shown.contains("lobby.lava"), "only the host sets it");
    assert!(shown.contains("Floor is Lava"));
    assert!(p.apps[1].online.session.as_ref().unwrap().lobby().lava);
    p.race();
    for a in &p.apps {
        assert!(a.play.as_ref().unwrap().race.lava);
    }
    // the host runs off the start platform onto the floor
    p.apps[0].key("KeyW", true);
    p.run(12000);
    let host = p.apps[0].play.as_ref().unwrap().race.racers[0].eaten;
    let seen = p.apps[1].play.as_ref().unwrap().race.racers[0].eaten;
    let (host, seen) = (host.expect("eaten at home"), seen.expect("and here"));
    assert_eq!(seen.at, host.at, "where its owner said");
    assert_eq!(seen.from, host.from, "the same jaguar");
    let ui: Value = serde_json::from_str(&p.apps[0].ui()).unwrap();
    assert!(
        ui["panels"].to_string().contains("Eaten"),
        "{}",
        ui["panels"]
    );
    let rows = p.apps[1].online.session.as_ref().unwrap().results();
    let host_row = rows.iter().find(|r| r.index == 0).unwrap();
    assert!(
        matches!(host_row.fin, crate::online::Fin::Eaten(..)),
        "{rows:?}"
    );
}
