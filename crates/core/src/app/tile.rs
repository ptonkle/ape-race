//! The home page's tiles (2026-10-12): `?tile=<spec>` on the page holds an
//! all-AI race still at a chosen tick, seen by a chosen camera, with no
//! menus over it, for `scripts/tiles.py` to screenshot. The race is the
//! same on every run (the core is deterministic), so a tile is reproduced
//! by its spec alone.
//!
//! The spec is `key=value` pairs joined by commas, each optional:
//! `map` (a map key, `jungle`), `seed` (the race's dice, 1), `apes` (kinds
//! joined by `+`, else four picked by the seed), `level` (the AI's
//! difficulty key), `lava` (`on`), `at` (race ticks to run, 0), `focus`
//! (the racer watched, 0), `view` (`chase`, `orbit` or `trackside`),
//! `yaw` and `pitch` (the orbit's, in degrees; the yaw from straight
//! behind, along the course), `dist` (the orbit's distance, in decimetres)
//! `hud` (`on` shows the HUD), and `blind`, `jojo` and `snow` (`on`: the
//! colourblind mode, JOJO MODE, the Ski Slope's snow; `snow=blizzard` a
//! blizzard), `aura` (`off`
//! hides the racers' auras, TRIAGE T59) and `pal` (each racer's palette,
//! joined by `+` (a space, as a URL may decode it, works too), TRIAGE
//! T49; classic for the rest) and `gear` (`none`, `ski` or `space`: what
//! the racers wear instead of the map's own; `space` previews the Sector
//! Ape costumes before their map exists), and for Sector Ape's pictures
//! `door` (ticks since the switch door shut, held at `at`) and `eat`
//! (the watched ape eaten by the jaguar this many ticks before `at`).

use crate::ai::Difficulty;
use crate::camera::Orbit;
use crate::focus::Focus;
use crate::fx::{self, deg, int};
use crate::map::Map;
use crate::race::{Driver, Entry, Race};
use crate::replay::{Mode, Playback, Recording, View};
use crate::rules::Kind;

use super::App;

/// How many apes a tile races when its spec names none.
const DEFAULT_APES: usize = 4;

/// A tile's spec, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    pub map: Map,
    pub seed: u64,
    pub apes: Vec<Kind>,
    pub level: Difficulty,
    pub lava: bool,
    pub at: u32,
    pub focus: usize,
    pub view: View,
    /// the orbit: yaw from behind and pitch (Q16 angles), distance (Q16 m)
    pub yaw: i32,
    pub pitch: i32,
    pub dist: i32,
    pub hud: bool,
    pub blind: bool,
    pub jojo: bool,
    pub snow: crate::snow::Weather,
    pub aura: bool,
    pub palettes: Vec<u8>,
    /// what the racers wear instead of the map's gear (`gear=space`: the
    /// Sector Ape costumes on any map)
    pub gear: Option<crate::models::Gear>,
    /// Sector Ape: the door shut this many ticks before `at`
    pub door: Option<u32>,
    /// the watched ape eaten this many ticks before `at` (the jaguar)
    pub eat: Option<u32>,
}

impl Tile {
    /// Reads a spec; an unknown key or a bad value keeps the default.
    pub fn parse(text: &str) -> Tile {
        let orbit = Orbit::default();
        let mut t = Tile {
            map: Map::default(),
            seed: 1,
            apes: Vec::new(),
            level: Difficulty::default(),
            lava: false,
            at: 0,
            focus: 0,
            view: View::Chase,
            yaw: 0,
            pitch: orbit.pitch,
            dist: orbit.dist,
            hud: false,
            blind: false,
            jojo: false,
            snow: crate::snow::Weather::Clear,
            aura: true,
            palettes: Vec::new(),
            gear: None,
            door: None,
            eat: None,
        };
        for pair in text.split(',') {
            let Some((k, v)) = pair.split_once('=') else {
                continue;
            };
            let n = v.parse::<i32>().ok();
            match k {
                "map" => t.map = Map::from_key(v).unwrap_or(t.map),
                "seed" => t.seed = v.parse().unwrap_or(t.seed),
                "apes" => t.apes = v.split('+').filter_map(Kind::from_key).collect(),
                "level" => t.level = Difficulty::from_key(v).unwrap_or(t.level),
                "lava" => t.lava = v == "on",
                "at" => t.at = v.parse().unwrap_or(t.at),
                "focus" => t.focus = v.parse().unwrap_or(t.focus),
                "view" => {
                    t.view = View::ALL
                        .into_iter()
                        .find(|w| w.key() == v)
                        .unwrap_or(t.view)
                }
                "yaw" => t.yaw = n.map(deg).unwrap_or(t.yaw),
                "pitch" => t.pitch = n.map(deg).unwrap_or(t.pitch),
                "dist" => t.dist = n.map(|d| int(d) / 10).unwrap_or(t.dist),
                "hud" => t.hud = v == "on",
                "blind" => t.blind = v == "on",
                "jojo" => t.jojo = v == "on",
                "snow" => {
                    t.snow = match v {
                        "on" => crate::snow::Weather::Snowing,
                        w => crate::snow::Weather::from_key(w).unwrap_or_default(),
                    }
                }
                "aura" => t.aura = v != "off",
                "gear" => t.gear = crate::models::Gear::from_key(v).or(t.gear),
                "door" => t.door = v.parse().ok(),
                "eat" => t.eat = v.parse().ok(),
                "pal" => {
                    t.palettes = v
                        .split(['+', ' '])
                        .map(|p| p.parse().unwrap_or(0))
                        .collect()
                }
                _ => {}
            }
        }
        t
    }

    /// The racers' kinds: the spec's, else four different ones from the
    /// seed.
    fn kinds(&self) -> Vec<Kind> {
        if !self.apes.is_empty() {
            return self.apes.clone();
        }
        let mut d = fx::Dice(self.seed);
        let mut pool = Kind::ALL.to_vec();
        (0..DEFAULT_APES)
            .map(|_| pool.remove(d.below(pool.len() as u32) as usize))
            .collect()
    }
}

impl App {
    /// Shows the tile `spec` describes: the race run to its tick, then held
    /// there (no ticks, no menus).
    pub(super) fn tile(&mut self, spec: &str) {
        let t = Tile::parse(spec);
        self.settings.map = t.map;
        self.settings.colourblind = t.blind;
        self.settings.speed_colours = t.jojo;
        self.settings.aura = t.aura;
        self.dress = t.gear;
        self.settings.weather = t.snow;
        self.ensure_course(t.map);
        let Some(course) = self.course_of(t.map).cloned() else {
            return;
        };
        let entries: Vec<Entry> = t
            .kinds()
            .into_iter()
            .enumerate()
            .map(|(i, k)| Entry {
                name: self.t(&format!("ape.{}", k.key())),
                kind: k,
                driver: Driver::Ai,
                palette: t.palettes.get(i).copied().unwrap_or(0),
            })
            .collect();
        let focus = t.focus.min(entries.len() - 1);
        let mut race = Race::new(course, &entries, false, t.seed);
        race.ai_level = t.level;
        race.lava = t.lava;
        self.begin(race, focus);
        let Some(mut p) = self.play.take() else {
            return;
        };
        for k in 0..t.at {
            if t.eat.is_some_and(|e| k + e == t.at) {
                p.race.eat(focus);
            }
            self.step_race(&mut p);
        }
        // Sector Ape's door, shut a moment ago (for its picture)
        let now = p.race.tick;
        if let (Some(ago), Some(s)) = (t.door, p.race.course.sector.as_deref_mut()) {
            s.door.shut_at = Some(now.saturating_sub(ago));
        }
        // held: nothing between ticks, the camera where it is
        p.prev = p.race.racers.iter().map(|r| r.ape.pos).collect();
        p.chase.prev_eye = p.chase.eye;
        p.chase.prev_target = p.chase.target;
        p.chase.prev_fov = p.chase.fov;
        p.focus = Focus::on(focus);
        p.pan = None;
        let me = &p.race.racers[focus];
        let along = crate::geom::yaw_of(p.race.course.heading_at(me.progress));
        let mut pb = Playback::new(Recording::of(&p.race, focus, Mode::Race));
        pb.playing = false;
        pb.view = t.view;
        pb.orbit = Orbit {
            yaw: fx::wrap(along + t.yaw),
            pitch: t.pitch,
            dist: t.dist,
            steered: true,
        };
        p.replay = Some(pb);
        p.still = Some(t.hud);
        self.play = Some(p);
        // the animations' clock, held with the race
        self.ticks = t.at;
        self.sounds.clear();
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn app() -> App {
        App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
    }

    fn ui(a: &mut App) -> Value {
        serde_json::from_str(&a.ui()).unwrap()
    }

    #[test]
    fn a_spec_reads_its_keys_and_keeps_defaults_for_the_rest() {
        let t = Tile::parse("seed=9,apes=gorilla+lemur,at=600,focus=1,view=orbit,yaw=90,pitch=-20,dist=85,hud=on,bogus=3,lava=on");
        assert_eq!(t.seed, 9);
        assert_eq!(t.apes, [Kind::Gorilla, Kind::Lemur]);
        assert!(t.aura && t.palettes.is_empty());
        let p = Tile::parse("pal=1+3,aura=off");
        assert_eq!((p.palettes.as_slice(), p.aura), ([1, 3].as_slice(), false));
        assert_eq!(Tile::parse("pal=2 0 1").palettes, [2, 0, 1]);
        assert_eq!(
            Tile::parse("gear=space").gear,
            Some(crate::models::Gear::Space)
        );
        assert_eq!(Tile::parse("gear=bogus").gear, None);
        let d = Tile::parse("door=30,eat=50");
        assert_eq!((d.door, d.eat), (Some(30), Some(50)));
        assert_eq!((t.door, t.eat), (None, None));
        assert_eq!((t.at, t.focus, t.view), (600, 1, View::Orbit));
        assert_eq!((t.yaw, t.pitch, t.dist), (deg(90), -deg(20), int(85) / 10));
        assert!(t.hud && t.lava);
        assert_eq!(t.map, Map::Jungle);
        let d = Tile::parse("");
        assert_eq!((d.seed, d.at, d.view, d.hud), (1, 0, View::Chase, false));
        // four different apes from the seed
        let k = d.kinds();
        assert_eq!(k.len(), DEFAULT_APES);
        assert!(k.iter().all(|a| k.iter().filter(|b| *b == a).count() == 1));
    }

    #[test]
    fn a_tile_holds_an_ai_race_at_its_tick_with_no_menus_or_hud() {
        let mut a = app();
        a.act("screen.tile", "seed=3,at=900,focus=2,view=trackside");
        let v = ui(&mut a);
        assert_eq!(v["screen"], "race");
        assert_eq!(v["panels"], Value::Array(Vec::new()));
        assert!(v.get("hud").is_none());
        let p = a.play.as_ref().unwrap();
        assert_eq!(p.race.tick, 900);
        assert_eq!(p.focus.racer, 2);
        assert!(p.race.racers.iter().all(|r| r.driver == Driver::Ai));
        let cam = a.camera();
        // held: the page's clock runs on and nothing moves
        let mut t = 0;
        for _ in 0..30 {
            t += 16_667;
            a.advance(t);
        }
        assert_eq!(a.play.as_ref().unwrap().race.tick, 900);
        assert_eq!(a.camera(), cam);
        assert_eq!(a.ticks, 900);
        // the same spec makes the same race
        let mut b = app();
        b.act("screen.tile", "seed=3,at=900,focus=2,view=trackside");
        let (pa, pb) = (a.play.as_ref().unwrap(), b.play.as_ref().unwrap());
        for (x, y) in pa.race.racers.iter().zip(&pb.race.racers) {
            assert_eq!(x.ape.pos, y.ape.pos);
        }
        // `hud=on` shows the HUD, without the replay's bar and hint
        let mut c = app();
        c.act("screen.tile", "at=10,hud=on");
        let v = ui(&mut c);
        assert!(v["hud"].is_object());
        assert!(v["hud"]["replay"].is_null());
        assert_eq!(v["hud"]["hint"], "");
    }

    #[test]
    fn a_tile_can_dress_the_racers_in_another_maps_gear() {
        // the Sector Ape costumes, before their map exists, on the Jungle
        let shapes = |spec: &str| {
            let mut a = app();
            a.act("screen.tile", spec);
            let n = a.frame().1.len();
            (a.dress, n)
        };
        let (none, plain) = shapes("apes=gorilla+lemur,at=60");
        let (dress, space) = shapes("apes=gorilla+lemur,at=60,gear=space");
        assert_eq!((none, dress), (None, Some(crate::models::Gear::Space)));
        assert!(space > plain, "{space} vs {plain}");
    }
}
