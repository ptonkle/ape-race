//! The app (PLAN P11, P15): screens, settings, input, the race loop and
//! what the page shows. The page passes its clock, keys and mouse in, and
//! draws the view model (`ui`) and the 3D frame this returns.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::ai::Brain;
use crate::ape::{Controls, Happening};
use crate::camera::{self, Chase, Pan};
use crate::course::Course;
use crate::focus::{self, Focus};
use crate::fx::{self, deg, int, ONE};
use crate::geom::{v3, V3};
use crate::map::{Map, RANDOM_MAP, RANDOM_PICTURE};
use crate::models;
use crate::race::{Driver, Entry, Event, Phase, Race};
use crate::render::{self, Batch, Camera};
use crate::replay::{self, Ghost, Library, Playback, Recording};
use crate::rules::{self, Kind, Stats};
use crate::scene;
use crate::sound::{self, Cue};
use crate::trial;

mod online_play;
pub use online_play::OnlinePlay;
mod gallery;
#[cfg(test)]
mod map_tests;
mod medals;
mod menu_dock;
#[cfg(test)]
mod setup_tests;
mod tile;

/// Ticks of physics a frame may catch up at most (a stalled tab does not
/// fast-forward the race).
const MAX_CATCH_UP: u32 = 12;
/// Microseconds per tick, times the tick rate: a tick is due for every
/// `1_000_000` accumulated (the clock is kept in µs × ticks per second).
const TICK_COST: i64 = 1_000_000;

/// The left mouse button, held as if it were a key.
const GRAB_BUTTON: &str = "Mouse0";

/// Two presses of R within this many ticks restart the race (0.4 s,
/// TRIAGE T7).
const DOUBLE_TAP: u32 = rules::TICKS_PER_SECOND as u32 * 2 / 5;
/// A finished run is kept in the replay library this long after the finish
/// (or on leaving, whichever comes first), so the coasting is in it too.
const KEEP_AFTER_FINISH: u32 = rules::TICKS_PER_SECOND as u32 * 3;
/// After that, a kept run is brought up to date this often while the
/// rest race on (TRIAGE T34), and once more on leaving.
const KEEP_AGAIN: u32 = rules::TICKS_PER_SECOND as u32 * 10;
/// The followed ape has been out of the race (finished) this long when the
/// camera pans to one still racing (TRIAGE T34).
const SPECTATE_AFTER: u32 = rules::TICKS_PER_SECOND as u32 * 2;
/// A finisher's dance (TRIAGE T53) is watched this long before the camera
/// pans on, and the camera stays at most this long for one to start.
const DANCE_SHOWN: u32 = rules::TICKS_PER_SECOND as u32 * 3;
const FINISH_WATCH_MOST: u32 = rules::TICKS_PER_SECOND as u32 * 12;
/// An eaten ape (TRIAGE T37): the camera stays for the jaguar, and pans
/// on a quarter of a second after it has eaten.
const EATEN_SPECTATE_AFTER: u32 = crate::jaguar::EATEN + rules::TICKS_PER_SECOND as u32 / 4;

/// Clicks on the title that unlock Speed Colours (TRIAGE row 26).
const TITLE_CLICKS: u32 = 10;

/// The person's settings, kept in browser storage as this JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub volume: u8,
    /// 1 to 10
    pub sensitivity: i32,
    pub invert_y: bool,
    pub auto_camera: bool,
    /// the sky and the jungle change colour at speed (TRIAGE rows 21, 26:
    /// off until unlocked)
    pub speed_colours: bool,
    /// the Speed Colours setting is unlocked (the title clicked ten times)
    pub colours_unlocked: bool,
    pub kind: Kind,
    pub ai_count: u8,
    /// the AI's difficulty (TRIAGE T14)
    pub ai_level: crate::ai::Difficulty,
    pub equal_stats: bool,
    pub name: String,
    /// the fastest finish on the jungle course, in race ticks
    pub best: Option<u32>,
    /// the fastest Time Trial on the jungle course (TRIAGE T1)
    pub trial_best: Option<u32>,
    /// race the best Time Trial's ghost
    pub ghost: bool,
    /// the items crates may give in a race (TRIAGE T42)
    pub items: crate::items::ItemSet,
    /// the colourblind mode (TRIAGE T33): the world muted, the vines vivid
    pub colourblind: bool,
    /// the map last chosen for a race, a Time Trial or a hosted lobby (the
    /// title flies over it); settings saved before the City read as Jungle.
    /// On Random, the map last raced (the preview stands on it)
    pub map: Map,
    /// Random is chosen (TRIAGE T64): each race picks a map other than the
    /// last at its start
    pub random_map: bool,
    /// the bests on every map but the Jungle, by the map's key (the
    /// Jungle's stay in `best` and `trial_best`, as saved before the City)
    pub bests: BTreeMap<String, Bests>,
    /// Floor is Lava (TRIAGE T37): touch the ground and a jaguar eats you
    pub lava: bool,
    /// the Ski Slope's weather (2026-10-13): Clear or Snowing, this
    /// player's own view only (no effect on play, not sent online)
    pub weather: crate::snow::Weather,
    /// each kind's chosen colours (TRIAGE T49), by the kind's key; a kind
    /// not here wears the classic
    pub palettes: BTreeMap<String, u8>,
    /// a soft glow round every racer in its start slot's colour (TRIAGE
    /// T59: blue, red, green, purple)
    pub aura: bool,
}

/// One map's personal bests, in race ticks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Bests {
    pub race: Option<u32>,
    pub trial: Option<u32>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            volume: sound::DEFAULT_VOLUME,
            sensitivity: 5,
            invert_y: false,
            auto_camera: true,
            speed_colours: false,
            colours_unlocked: false,
            kind: Kind::Chimpanzee,
            ai_count: 3,
            ai_level: crate::ai::Difficulty::default(),
            equal_stats: false,
            name: String::new(),
            best: None,
            trial_best: None,
            ghost: true,
            items: crate::items::ItemSet::ALL,
            colourblind: false,
            map: Map::default(),
            random_map: false,
            bests: BTreeMap::new(),
            lava: false,
            weather: crate::snow::Weather::Clear,
            palettes: BTreeMap::new(),
            aura: true,
        }
    }
}

impl Settings {
    pub fn parse(text: &str) -> Settings {
        let mut s: Settings = serde_json::from_str(text).unwrap_or_default();
        s.volume = s.volume.min(100);
        s.sensitivity = s.sensitivity.clamp(1, 10);
        // a race has one to three AI opponents (TRIAGE T26: Time Trial is
        // the solo run); a saved 0 reads as 1
        s.ai_count = s.ai_count.clamp(1, 3);
        s.items = s.items.clean();
        // Speed Colours is a secret until unlocked (TRIAGE row 26), even for
        // settings saved while it was on by default
        s.speed_colours &= s.colours_unlocked;
        s.palettes
            .retain(|k, p| Kind::from_key(k).is_some() && *p > 0 && *p < models::PALETTES);
        s
    }

    /// The colours `kind` wears (TRIAGE T49).
    pub fn palette_of(&self, kind: Kind) -> u8 {
        self.palettes.get(kind.key()).copied().unwrap_or(0)
    }

    /// The colours of the chosen ape.
    pub fn palette(&self) -> u8 {
        self.palette_of(self.kind)
    }

    /// Chooses the colours of the chosen ape (out of range: the classic).
    pub fn set_palette(&mut self, palette: u8) {
        let key = self.kind.key().to_string();
        if palette == 0 || palette >= models::PALETTES {
            self.palettes.remove(&key);
        } else {
            self.palettes.insert(key, palette);
        }
    }

    /// The personal best on `map`, of a Time Trial (`trial`) or a race.
    pub fn best(&self, map: Map, trial: bool) -> Option<u32> {
        match (map, trial) {
            (Map::Jungle, false) => self.best,
            (Map::Jungle, true) => self.trial_best,
            _ => self
                .bests
                .get(map.key())
                .and_then(|b| if trial { b.trial } else { b.race }),
        }
    }

    fn best_mut(&mut self, map: Map, trial: bool) -> &mut Option<u32> {
        match (map, trial) {
            (Map::Jungle, false) => &mut self.best,
            (Map::Jungle, true) => &mut self.trial_best,
            _ => {
                let b = self.bests.entry(map.key().to_string()).or_default();
                if trial {
                    &mut b.trial
                } else {
                    &mut b.race
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Screen {
    Title,
    Setup,
    Settings,
    Race,
    Lineup,
    /// online: name, Host, Join (PLAN P13)
    Online,
    /// online: the seats before a race
    Lobby,
    /// the kept replays (TRIAGE T15)
    Replays,
}

/// The static scenery the page holds: `scene_version` changes with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shown {
    /// the small jungle stage (the Jungle's ape preview, the lineup)
    StageSet,
    /// a map's whole scenery (the flyover, a race, the City's ape preview)
    World(Map),
}

/// How many maps there are: the size of the course and scenery caches.
const MAPS: usize = Map::ALL.len();

/// A map's place in the caches.
fn slot(map: Map) -> usize {
    Map::ALL.iter().position(|m| *m == map).unwrap_or(0)
}

/// What the 3D view shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    /// the chosen map, flown over slowly (behind the title)
    Flyover,
    /// one ape spinning (race setup)
    Showcase,
    /// all six side by side (the thumbnail)
    Lineup,
    Race,
}

/// A race in progress, as this browser runs it.
pub struct Play {
    pub race: Race,
    pub brains: Vec<Brain>,
    pub chase: Chase,
    pub local: usize,
    /// Paused offline; online the menu overlay is open and the race runs on.
    pub paused: bool,
    /// an online race: never pauses, and the session sends and receives
    pub online: bool,
    /// each ape's position last tick, for drawing between ticks
    prev: Vec<V3>,
    /// ticks since the local ape was out of the race (finished or eaten)
    since_out: Option<u32>,
    /// the local ape's horizontal speed, smoothed over about a quarter
    /// second: what the speedometer, the hype and the colours read (TRIAGE
    /// row 19: a downswing's falling speed set them off)
    pub hype_speed: i32,
    /// the same, smoothed over about a second, for the speed colours
    /// (TRIAGE row 22: they followed each swing's rise and fall)
    pub colour_speed: i32,
    /// the hype call-outs and their flash (TRIAGE T22): live racing only,
    /// not stepped in a replay
    pub hype: crate::hype::Hype,
    /// the local finish beat the saved best
    new_best: bool,
    /// each vine drawn as a bending chain (TRIAGE row 12)
    pub chains: Vec<crate::vinesim::Chain>,
    /// Sector Ape: where an ape was just put back on a deck or the door
    /// just shut, and the race tick (drawn as a flash for a moment)
    pub flashes: Vec<crate::sectorscape::Flash>,
    /// a Time Trial (TRIAGE T1)
    pub trial: bool,
    /// what the local player does each tick, for the replay (offline only)
    pub record: Option<Recording>,
    /// the recording has gone into the library, under this serial
    kept: Option<u32>,
    /// the racer the camera follows (TRIAGE T24, T34)
    pub focus: Focus,
    /// ticks the followed racer has been out of the race
    out_for: u32,
    /// ticks of those it has been dancing (TRIAGE T53)
    danced: u32,
    /// the camera gliding to a new racer
    pan: Option<Pan>,
    /// watching a recording instead of racing (TRIAGE T15)
    pub replay: Option<Playback>,
    /// the best Time Trial, raced again beside the player
    pub ghost: Option<Ghost>,
    /// the map was picked at random (TRIAGE T64): the HUD names it at the
    /// start
    pub random_map: bool,
    /// a home-page tile (`tile.rs`): held still, no menus; `true` keeps
    /// the HUD
    pub still: Option<bool>,
}

/// The game's version, shown small in the bottom right corner: major
/// (1, the first release), minor (one more for each release with new
/// features or a new map: 1.1 the todo batch, 1.2 the City, 1.3 the Ski
/// Slope, 1.4 the map grid and kept best replays, 1.5 auras and palettes,
/// 1.6 finish dances),
/// patch (fixes only).
pub const VERSION: &str = "1.7.0";

pub struct App {
    strings: BTreeMap<String, String>,
    pub settings: Settings,
    pub screen: Screen,
    /// clicks on the title text, toward the secret (TRIAGE row 26)
    title_clicks: u32,
    /// where the title's flyover starts, metres along (group Y's test hook,
    /// `?at=800` on the page)
    fly_from: i32,
    /// the title's background is celebrating an unlock, until the screen
    /// changes
    celebrate: bool,
    stage: Stage,
    pub play: Option<Play>,
    keys: BTreeSet<String>,
    /// the item key went down since the last tick
    item_pressed: bool,
    mouse: (i32, i32),
    pointer_locked: bool,
    last_us: Option<i64>,
    acc: i64,
    /// Q16 share of the way to the next tick
    alpha: i32,
    /// a running tick count for menus and animation
    pub ticks: u32,
    /// what the apes wear instead of the map's gear: a tile's `gear=`
    /// preview (the Sector Ape costumes before their map exists); never
    /// saved, `None` everywhere else
    pub dress: Option<models::Gear>,
    sounds: Vec<String>,
    save: bool,
    scene_version: u32,
    view_w: i32,
    view_h: i32,
    /// each map's course and scenery (in `Map::ALL` order), built when
    /// first needed and kept
    courses: [Option<Course>; MAPS],
    worlds: [Option<scene::Scenery>; MAPS],
    /// the static scenery the page was last given
    shown: Shown,
    stage_set: scene::Scenery,
    seed: u64,
    /// the page clock in milliseconds, for online play
    pub now_ms: i64,
    pub online: OnlinePlay,
    /// the setup and the next race are a Time Trial
    pub trial: bool,
    /// the kept replays, saved under their own storage key
    pub library: Library,
    save_replays: bool,
    /// how many recent replays the saved text holds: fewer each time the
    /// page reports that storage would not take it (TRIAGE T55; the bests
    /// are always in it)
    replay_room: usize,
    /// when R last went down (`ticks`), for the double tap
    last_r: Option<u32>,
    /// the finished race whose replay is being watched, to go back to
    stash: Option<Play>,
    /// where leaving a replay from the menus goes
    replay_exit: Screen,
    /// Sector Ape's model gallery (`gallery.rs`), shown instead of all else
    gallery: Option<gallery::Gallery>,
}

fn s(map: &BTreeMap<String, String>, key: &str) -> String {
    map.get(key).cloned().unwrap_or_else(|| key.to_string())
}

impl App {
    pub fn new(strings_json: &str, settings_json: &str, seed: u64) -> Result<App, String> {
        let strings: BTreeMap<String, String> = serde_json::from_str(strings_json)
            .map_err(|e| format!("The strings file could not be read: {e}"))?;
        let settings = Settings::parse(settings_json);
        let map = settings.map;
        let mut app = App {
            strings,
            settings,
            screen: Screen::Title,
            title_clicks: 0,
            fly_from: 0,
            celebrate: false,
            stage: Stage::Flyover,
            play: None,
            keys: BTreeSet::new(),
            item_pressed: false,
            mouse: (0, 0),
            pointer_locked: false,
            last_us: None,
            acc: 0,
            alpha: 0,
            ticks: 0,
            dress: None,
            sounds: Vec::new(),
            save: false,
            scene_version: 1,
            view_w: 1280,
            view_h: 720,
            courses: Default::default(),
            worlds: Default::default(),
            shown: Shown::World(map),
            stage_set: scene::stage_set(),
            seed,
            now_ms: 0,
            online: OnlinePlay::default(),
            trial: false,
            library: Library::default(),
            save_replays: false,
            replay_room: replay::RECENT,
            last_r: None,
            stash: None,
            replay_exit: Screen::Title,
            gallery: None,
        };
        // only the map the title flies over is built now; the others when
        // first chosen
        app.ensure_world(map);
        Ok(app)
    }

    pub fn t(&self, key: &str) -> String {
        s(&self.strings, key)
    }

    // ---- the maps ----

    fn ensure_course(&mut self, map: Map) {
        let k = slot(map);
        if self.courses[k].is_none() {
            self.courses[k] = Some(map.course());
        }
    }

    /// Builds a map's course and scenery, unless they are built already.
    fn ensure_world(&mut self, map: Map) {
        self.ensure_course(map);
        let k = slot(map);
        if self.worlds[k].is_none() {
            self.worlds[k] = self.courses[k].as_ref().map(scene::scenery);
        }
    }

    /// A map's course, once built.
    pub(crate) fn course_of(&self, map: Map) -> Option<&Course> {
        self.courses[slot(map)].as_ref()
    }

    /// Whether a map's scenery has been built (it is built when first shown).
    #[cfg(test)]
    fn built(&self, map: Map) -> bool {
        self.worlds[slot(map)].is_some()
    }

    /// The map the menus are set to: in a lobby the host's choice, else
    /// the last chosen.
    pub(crate) fn map(&self) -> Map {
        match &self.online.session {
            Some(o) if self.screen == Screen::Lobby => o.map(),
            _ => self.settings.map,
        }
    }

    /// Whether snow falls in the 3D view: the Weather setting is Snowing
    /// and the view is on the Ski Slope (a race, the title's flyover, the
    /// setup's preview), never on the lineup.
    pub(crate) fn snowing(&self) -> bool {
        self.settings.weather == crate::snow::Weather::Snowing
            && self.stage != Stage::Lineup
            && self.shown_map() == Map::Ski
    }

    /// The map the 3D view is on: a race's own, else the menus'.
    fn shown_map(&self) -> Map {
        match (&self.play, self.stage) {
            (Some(p), Stage::Race) => Map::of(&p.race.course),
            _ => self.map(),
        }
    }

    fn wanted_scene(&self) -> Shown {
        let map = self.shown_map();
        match self.stage {
            Stage::Lineup => Shown::StageSet,
            Stage::Showcase if map == Map::Jungle => Shown::StageSet,
            _ => Shown::World(map),
        }
    }

    /// Builds what the screen now needs (the shown map's scenery, the
    /// courses the Replays list checks its runs against) and moves the
    /// scene version on when the static scenery changes.
    fn settle_scene(&mut self) {
        let map = self.shown_map();
        self.ensure_course(map);
        let want = self.wanted_scene();
        if let Shown::World(m) = want {
            self.ensure_world(m);
        }
        if self.screen == Screen::Replays {
            let kept: Vec<Map> = self
                .library
                .listing()
                .iter()
                .filter_map(|(r, _)| Map::from_key(&r.course))
                .collect();
            for m in kept {
                self.ensure_course(m);
            }
        }
        if want != self.shown {
            self.shown = want;
            self.scene_version += 1;
        }
    }

    /// The map choice on Race Setup, Time Trial and the lobby (TRIAGE T54):
    /// a grid of cards, one per map with its picture and name, and last
    /// Random (T64, `random`: chosen instead of `chosen`). `read_only` (a
    /// joiner's lobby) shows the host's pick and lets no card be pressed.
    /// A card sends the grid's id with its map's key, or `random`.
    pub(crate) fn map_tiles(&self, id: &str, chosen: Map, random: bool, read_only: bool) -> Value {
        let mut options: Vec<Value> = Map::ALL
            .iter()
            .map(|m| {
                json!({ "id": format!("{id}.{}", m.key()), "value": m.key(),
                  "text": self.map_name(*m), "picture": m.picture(),
                  "chosen": !random && *m == chosen })
            })
            .collect();
        options.push(
            json!({ "id": format!("{id}.{RANDOM_MAP}"), "value": RANDOM_MAP,
          "text": self.t("map.random"), "picture": RANDOM_PICTURE, "chosen": random }),
        );
        let value = if random { RANDOM_MAP } else { chosen.key() };
        json!({ "type": "tiles", "id": id, "label": self.t("setup.map"),
                "value": value, "read_only": read_only, "options": options })
    }

    /// The map a race on Random runs on, its name shown at the start
    /// (TRIAGE T64).
    pub(crate) fn random_map_flash(&self, p: &Play) -> Option<String> {
        let early = match p.race.phase {
            Phase::Countdown(_) => true,
            Phase::Running => p.race.clock < rules::TICKS_PER_SECOND as u32 * 2,
            _ => false,
        };
        (p.random_map && early && p.replay.is_none()).then(|| {
            format!(
                "{}: {}",
                self.t("hud.random_map"),
                self.map_name(Map::of(&p.race.course))
            )
        })
    }

    /// The Ski Slope's Weather select (`settings.weather` in Settings,
    /// `setup.weather` on the Ski Slope's setup), labelled by its id.
    pub(crate) fn weather_select(&self, id: &str) -> Value {
        let options: Vec<Value> = crate::snow::Weather::ALL
            .iter()
            .map(|w| json!({ "value": w.key(), "text": self.t(&format!("weather.{}", w.key())) }))
            .collect();
        json!({ "type": "select", "id": id, "label": self.t(id),
                "value": self.settings.weather.key(), "options": options })
    }

    /// A map's name, as shown.
    pub(crate) fn map_name(&self, map: Map) -> String {
        self.t(&format!("map.{}", map.key()))
    }

    // ---- input ----

    pub fn key(&mut self, code: &str, down: bool) {
        if down {
            if code == "KeyE" && !self.keys.contains(code) {
                self.item_pressed = true;
            }
            if code == "Escape" {
                self.escape();
            }
            if code == "KeyR" && !self.keys.contains(code) {
                self.quick_restart();
            }
            if code == "Space" && !self.keys.contains(code) {
                if let Some(pb) = self.play.as_mut().and_then(|p| p.replay.as_mut()) {
                    pb.playing = !pb.playing;
                }
            }
            // the racer followed in a replay or while spectating
            if self.screen == Screen::Race && !self.keys.contains(code) {
                match code {
                    "KeyD" | "ArrowRight" => self.switch_focus(true),
                    "KeyA" | "ArrowLeft" => self.switch_focus(false),
                    _ => {}
                }
            }
            self.keys.insert(code.to_string());
        } else {
            self.keys.remove(code);
        }
    }

    /// The left button is grab (held, like a key); the right button uses
    /// the item (TRIAGE row 2).
    pub fn mouse_button(&mut self, button: i32, down: bool) {
        match button {
            0 if down && self.screen == Screen::Race => {
                self.keys.insert(GRAB_BUTTON.to_string());
            }
            0 => {
                self.keys.remove(GRAB_BUTTON);
            }
            2 if down && self.screen == Screen::Race => self.item_pressed = true,
            _ => {}
        }
    }

    pub fn mouse_move(&mut self, dx: i32, dy: i32) {
        if self.pointer_locked {
            self.mouse.0 += dx;
            self.mouse.1 += dy;
        }
    }

    pub fn pointer_lock(&mut self, locked: bool) {
        // losing the pointer mid-race (Esc, or a click away) pauses it
        if self.pointer_locked && !locked {
            if let Some(p) = &mut self.play {
                if p.since_out.is_none() && !p.online {
                    p.paused = true;
                }
            }
        }
        self.pointer_locked = locked;
        if !locked {
            self.mouse = (0, 0);
        }
    }

    /// The window lost focus: let go of every key.
    pub fn blur(&mut self) {
        self.keys.clear();
    }

    fn held(&self, codes: &[&str]) -> bool {
        codes.iter().any(|c| self.keys.contains(*c))
    }

    fn controls(&mut self, yaw: i32, pitch: i32) -> Controls {
        let fwd = self.held(&["KeyW", "ArrowUp"]) as i32 - self.held(&["KeyS", "ArrowDown"]) as i32;
        let side =
            self.held(&["KeyD", "ArrowRight"]) as i32 - self.held(&["KeyA", "ArrowLeft"]) as i32;
        Controls {
            fwd,
            side,
            grab: self.held(&[GRAB_BUTTON]),
            jump: self.held(&["Space"]),
            item: std::mem::take(&mut self.item_pressed),
            yaw,
            pitch,
        }
    }

    /// R twice within [`DOUBLE_TAP`] restarts an offline race or a Time
    /// Trial at once (TRIAGE T7).
    fn quick_restart(&mut self) {
        let offline = self.screen == Screen::Race
            && self
                .play
                .as_ref()
                .is_some_and(|p| !p.online && p.replay.is_none());
        match self.last_r {
            Some(t) if offline && self.ticks.wrapping_sub(t) <= DOUBLE_TAP => {
                self.last_r = None;
                self.start_race();
            }
            _ => self.last_r = offline.then_some(self.ticks),
        }
    }

    fn escape(&mut self) {
        match self.screen {
            Screen::Race if self.play.as_ref().is_some_and(|p| p.replay.is_some()) => {
                self.leave_replay()
            }
            Screen::Race => {
                if let Some(p) = &mut self.play {
                    p.paused = !p.paused;
                }
            }
            Screen::Setup | Screen::Settings | Screen::Online | Screen::Replays => {
                self.go(Screen::Title)
            }
            _ => {}
        }
    }

    // ---- screens ----

    fn go(&mut self, screen: Screen) {
        self.screen = screen;
        self.celebrate = false;
        self.title_clicks = 0;
        let stage = match screen {
            Screen::Title | Screen::Settings | Screen::Online | Screen::Replays => Stage::Flyover,
            Screen::Setup | Screen::Lobby => Stage::Showcase,
            Screen::Lineup => Stage::Lineup,
            Screen::Race => Stage::Race,
        };
        self.stage = stage;
        if screen != Screen::Race {
            if let Some(mut p) = self.play.take() {
                self.keep_recording(&mut p);
            }
            self.stash = None;
        }
        self.settle_scene();
    }

    /// Puts a finished run into the replay library, or brings the one kept
    /// up to date (the race goes on being recorded while the rest finish).
    fn keep_recording(&mut self, p: &mut Play) {
        if p.replay.is_some() {
            return;
        }
        let me = &p.race.racers[p.local];
        let (finish, eaten) = (me.finish, me.eaten.is_some());
        if let (Some(rec), true) = (&p.record, finish.is_some() || eaten) {
            let mut rec = rec.clone();
            rec.finish = finish;
            rec.eaten = eaten;
            match p.kept {
                Some(serial) => {
                    rec.serial = serial;
                    self.save_replays |= self.library.refresh(&rec);
                }
                None => {
                    p.kept = Some(self.library.serial_of_next());
                    self.library.add(rec);
                    self.save_replays = true;
                }
            }
        }
    }

    /// Follows the next or previous racer (`next`): in a replay any of
    /// them, live only once the local ape is out of the race, and then
    /// those still racing (TRIAGE T24, T34).
    fn switch_focus(&mut self, next: bool) {
        let alpha = self.alpha;
        let Some(p) = &mut self.play else {
            return;
        };
        let live = p.replay.is_none();
        if live && focus::racing(&p.race.racers[p.local]) {
            return;
        }
        let from = play_camera(p, alpha);
        let moved = if next {
            p.focus.next(&p.race, live)
        } else {
            p.focus.prev(&p.race, live)
        };
        if moved {
            refocus(p, from, camera::PAN_QUICK);
        }
    }

    /// A drag across the 3D view while not captured, in pixels: in a
    /// replay it steers the orbit camera, switching to it (TRIAGE T23).
    pub fn drag(&mut self, dx: i32, dy: i32) {
        let alpha = self.alpha;
        let Some(p) = &mut self.play else {
            return;
        };
        let Some(view) = p.replay.as_ref().map(|pb| pb.view) else {
            return;
        };
        if view != replay::View::Orbit {
            // start from the view as it is
            let cam = play_camera(p, alpha);
            let at = p.race.racers[p.focus.racer].ape.pos;
            if let Some(pb) = &mut p.replay {
                pb.orbit.yaw = crate::geom::yaw_of(cam.fwd);
                pb.orbit.pitch = crate::geom::pitch_of(cam.fwd);
                pb.orbit.dist = at.dist(cam.eye);
                pb.view = replay::View::Orbit;
            }
        }
        if let Some(pb) = &mut p.replay {
            pb.orbit.drag(dx, dy);
        }
    }

    /// The mouse wheel over the 3D view: in a replay's orbit, zoom (only
    /// its direction counts, so pixel and line wheels agree).
    pub fn wheel(&mut self, dy: i32) {
        if let Some(pb) = self.play.as_mut().and_then(|p| p.replay.as_mut()) {
            if pb.view == replay::View::Orbit {
                pb.orbit.zoom(dy.signum());
            }
        }
    }

    /// The replays kept in browser storage, from the page at start.
    pub fn load_replays(&mut self, text: &str) {
        self.library = Library::parse(text);
    }

    /// The page could not store the last `save_replays` text (storage full
    /// or blocked; what was stored before is still there). Asks to save
    /// again with one recent replay fewer, down to the bests alone, which
    /// are never shed (TRIAGE T55). The library itself keeps them all.
    pub fn replays_not_saved(&mut self) {
        let room = self.replay_room.min(self.library.recent.len());
        if room > 0 {
            self.replay_room = room - 1;
            self.save_replays = true;
        }
    }

    /// Starts watching `rec`; leaving goes back to `exit` (or to the
    /// stashed race's results).
    fn watch(&mut self, rec: Recording, exit: Screen) -> bool {
        let Some(map) = Map::from_key(&rec.course) else {
            return false;
        };
        self.ensure_world(map);
        let Some(race) = self
            .course_of(map)
            .filter(|c| rec.watchable(c))
            .map(|c| rec.race(c))
        else {
            return false;
        };
        let local = rec.local;
        self.begin(race, local);
        self.replay_exit = exit;
        if let Some(p) = &mut self.play {
            p.trial = rec.mode == replay::Mode::Trial;
            p.replay = Some(Playback::new(rec));
        }
        true
    }

    /// The replay viewer's controls: play/pause, speed, camera, seek.
    fn replay_act(&mut self, id: &str, value: &str) {
        let Some(p) = &mut self.play else {
            return;
        };
        let Some(pb) = &mut p.replay else {
            return;
        };
        match id {
            "replay.play" if pb.length() <= p.race.tick => {
                // at the end: Play starts again
                pb.playing = true;
                if let Some(save) = pb.seek_to(0, p.race.tick) {
                    restore(p, save);
                }
            }
            "replay.play" => pb.playing = !pb.playing,
            "replay.speed" => {
                if let Some(sp) = replay::Speed::ALL.into_iter().find(|s| s.key() == value) {
                    pb.speed = sp;
                }
            }
            "replay.camera" => {
                if let Some(v) = replay::View::ALL.into_iter().find(|v| v.key() == value) {
                    pb.view = v;
                }
            }
            "replay.seek" => {
                let permille = value.parse::<u32>().unwrap_or(0);
                if let Some(save) = pb.seek_to(permille, p.race.tick) {
                    restore(p, save);
                }
            }
            _ => {}
        }
    }

    fn leave_replay(&mut self) {
        match self.stash.take() {
            Some(p) => self.play = Some(p),
            None => self.go(self.replay_exit),
        }
    }

    fn mode(&self) -> replay::Mode {
        if self.trial {
            replay::Mode::Trial
        } else {
            replay::Mode::Race
        }
    }

    /// The best kept run of the setup's mode on its map, when it fits the
    /// map's course as it is now.
    fn best_replay(&self) -> Option<&Recording> {
        let course = self.course_of(self.settings.map)?;
        self.library.best_for(course, self.mode())
    }

    /// A look at another map before the map selector (group Y's test hook,
    /// `?map=city` on the page).
    fn preview_map(&mut self, key: &str) {
        if let Some(map) = crate::map::Map::from_key(key) {
            self.settings.map = map;
            self.settle_scene();
        }
    }

    /// A button or field: `id` names it, `value` is its new value.
    pub fn act(&mut self, id: &str, value: &str) {
        if id == "flag.map" {
            self.preview_map(value);
            return;
        }
        if id == "flag.fly_at" {
            self.fly_from = int(value.parse::<i32>().unwrap_or(0).clamp(0, 5000));
            return;
        }
        if id.starts_with("flag.") {
            self.online_flag(id, value);
            return;
        }
        self.sounds.push(Cue::Click.key().to_string());
        let in_online_race = self.play.as_ref().is_some_and(|p| p.online);
        match id {
            _ if id.starts_with("online.") || id.starts_with("lobby.") => {
                self.online_act(id, value)
            }
            "menu.rematch" | "menu.main" | "menu.quit" if in_online_race => {
                self.online_race_menu(id)
            }
            "menu.online" => self.go(Screen::Online),
            "menu.race" => {
                self.trial = false;
                self.go(Screen::Setup)
            }
            "menu.trial" => {
                self.trial = true;
                self.go(Screen::Setup)
            }
            "menu.replays" => self.go(Screen::Replays),
            "menu.replay" => {
                // the race just run: watch it, then come back to its results
                if let Some(mut p) = self.play.take() {
                    self.keep_recording(&mut p);
                    let rec = p.record.clone();
                    let exit = self.screen;
                    self.stash = Some(p);
                    if !rec.is_some_and(|r| self.watch(r, exit)) {
                        self.play = self.stash.take();
                    }
                }
            }
            "replay.best" => {
                if let Some(rec) = self.best_replay().cloned() {
                    self.watch(rec, Screen::Setup);
                }
            }
            _ if id.starts_with("replays.watch.") => {
                let k: usize = id["replays.watch.".len()..].parse().unwrap_or(usize::MAX);
                let rec = self.library.listing().get(k).map(|(r, _)| (*r).clone());
                if let Some(rec) = rec {
                    self.watch(rec, Screen::Replays);
                }
            }
            "replay.exit" => self.leave_replay(),
            "focus.next" => self.switch_focus(true),
            "focus.prev" => self.switch_focus(false),
            _ if id.starts_with("replay.") => self.replay_act(id, value),
            "setup.ghost" => {
                self.settings.ghost = value == "on";
                self.save = true;
            }
            "setup.map" => {
                if value == RANDOM_MAP {
                    self.settings.random_map = true;
                    self.save = true;
                } else if let Some(m) = Map::from_key(value) {
                    self.settings.map = m;
                    self.settings.random_map = false;
                    self.save = true;
                }
            }
            "menu.settings" => self.go(Screen::Settings),
            "menu.back" | "menu.main" | "menu.quit" => self.go(Screen::Title),
            "menu.start" | "menu.restart" | "menu.rematch" => self.start_race(),
            "menu.resume" => {
                if let Some(p) = &mut self.play {
                    p.paused = false;
                }
            }
            "screen.lineup" => self.go(Screen::Lineup),
            "screen.tile" => self.tile(value),
            "screen.gallery" => self.gallery(value),
            "setup.ape" => {
                if let Some(k) = Kind::from_key(value) {
                    self.settings.kind = k;
                    self.save = true;
                }
            }
            "setup.ape.next" | "setup.ape.prev" => {
                let n = Kind::ALL.len();
                let i = self.settings.kind.index();
                let j = if id.ends_with("next") {
                    (i + 1) % n
                } else {
                    (i + n - 1) % n
                };
                self.settings.kind = Kind::ALL[j];
                self.save = true;
            }
            "setup.palette" => {
                if let Ok(p) = value.parse::<u8>() {
                    self.settings.set_palette(p);
                    self.save = true;
                }
            }
            "settings.aura" => {
                self.settings.aura = value == "on";
                self.save = true;
            }
            "setup.ai_level" => {
                if let Some(d) = crate::ai::Difficulty::from_key(value) {
                    self.settings.ai_level = d;
                    self.save = true;
                }
            }
            "setup.opponents" => {
                if let Ok(n) = value.parse::<u8>() {
                    self.settings.ai_count = n.clamp(1, 3);
                    self.save = true;
                }
            }
            "setup.equal" => {
                self.settings.equal_stats = value == "on";
                self.save = true;
            }
            "setup.lava" => {
                self.settings.lava = value == "on";
                self.save = true;
            }
            _ if id.starts_with("setup.item.") => self.tick_item(id, value),
            "settings.volume" => {
                if let Ok(v) = value.parse::<u8>() {
                    self.settings.volume = v.min(100);
                    self.save = true;
                }
            }
            "settings.sensitivity" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.settings.sensitivity = v.clamp(1, 10);
                    self.save = true;
                }
            }
            "settings.invert" => {
                self.settings.invert_y = value == "on";
                self.save = true;
            }
            "title.click" => {
                self.title_clicks += 1;
                if self.title_clicks == TITLE_CLICKS && !self.settings.colours_unlocked {
                    self.settings.colours_unlocked = true;
                    self.settings.speed_colours = true;
                    self.celebrate = true;
                    self.save = true;
                    self.sounds.push(Cue::Finish.key().to_string());
                }
            }
            "settings.speed_colours" => {
                self.settings.speed_colours = value == "on";
                self.save = true;
            }
            "settings.colourblind" => {
                self.settings.colourblind = value == "on";
                self.save = true;
            }
            "settings.auto_camera" => {
                self.settings.auto_camera = value == "on";
                self.save = true;
            }
            // the same setting from Settings and from the Ski Slope's setup
            "settings.weather" | "setup.weather" => {
                if let Some(w) = crate::snow::Weather::from_key(value) {
                    self.settings.weather = w;
                    self.save = true;
                }
            }
            _ => {}
        }
        self.settle_scene();
    }

    fn start_race(&mut self) {
        let mut entries = vec![Entry {
            name: self.t("player.you"),
            kind: self.settings.kind,
            driver: Driver::Local,
            palette: self.settings.palette(),
        }];
        let mut d = fx::Dice(self.seed ^ self.ticks as u64);
        // the AI picks kinds other than the player's where it can
        let mut pool: Vec<Kind> = Kind::ALL
            .into_iter()
            .filter(|k| *k != self.settings.kind)
            .collect();
        let opponents = if self.trial {
            0
        } else {
            // the lobby may have set 0 (online, people race people)
            self.settings.ai_count.max(1)
        };
        for _ in 0..opponents {
            let k = pool.remove(d.below(pool.len() as u32) as usize);
            entries.push(Entry {
                name: self.t(&format!("ape.{}", k.key())),
                kind: k,
                driver: Driver::Ai,
                palette: crate::race::palette_apart(crate::race::worn(&entries), k),
            });
        }
        // Random (TRIAGE T64): a map other than the last, from the page's
        // seed and the clock; it becomes the last map (the preview's)
        if self.settings.random_map {
            self.settings.map = Map::random(d.next(), self.settings.map);
            self.save = true;
        }
        let map = self.settings.map;
        self.ensure_world(map);
        let course = self.course_of(map).cloned().unwrap_or_else(|| map.course());
        let mut race = Race::new(course, &entries, self.settings.equal_stats, d.next());
        // a Time Trial: crates give only the Speed Boost (TRIAGE T13);
        // otherwise the items ticked on the setup (T42)
        race.items = if self.trial {
            crate::items::ItemSet::only(crate::race::Item::Boost)
        } else {
            self.settings.items
        };
        race.ai_level = self.settings.ai_level;
        // Floor is Lava (TRIAGE T37); never in a Time Trial
        race.lava = self.settings.lava && !self.trial;
        let record = Recording::of(&race, 0, self.mode());
        // (after `begin`, which keeps the run being left: it may be the best)
        self.begin(race, 0);
        let ghost = self
            .best_replay()
            .filter(|_| self.trial && self.settings.ghost)
            .and_then(|r| self.course_of(map).and_then(|c| Ghost::new(r, c)));
        if let Some(p) = &mut self.play {
            p.trial = self.trial;
            p.record = Some(record);
            p.ghost = ghost;
            p.random_map = self.settings.random_map;
        }
    }

    /// Starts showing a race in which this browser plays ape `local`.
    pub fn begin(&mut self, race: Race, local: usize) {
        let at = race.racers[local].ape.pos;
        let mut chase = Chase::new(at, race.course.start_yaw, race.course.terrain);
        // Sector Ape runs about twice as fast: the view opens wider
        if race.course.sector.is_some() {
            chase.pace = crate::camera::SPACE_PACE;
        }
        let prev = race.racers.iter().map(|r| r.ape.pos).collect();
        // each AI ape thinks at the race's difficulty, with its own dice
        // from the race's seed, so a replay rebuilds the same brains
        let brains = (0..race.racers.len())
            .map(|i| Brain::new(race.ai_level, race.dice.0 ^ (0x9E37_79B9 * (i as u64 + 1))))
            .collect();
        if let Some(mut old) = self.play.take() {
            self.keep_recording(&mut old);
        }
        self.go(Screen::Race);
        self.play = Some(Play {
            race,
            brains,
            chase,
            local,
            paused: false,
            online: false,
            prev,
            since_out: None,
            hype_speed: 0,
            colour_speed: 0,
            hype: crate::hype::Hype::default(),
            new_best: false,
            chains: Vec::new(),
            flashes: Vec::new(),
            trial: false,
            record: None,
            kept: None,
            focus: Focus::on(local),
            out_for: 0,
            danced: 0,
            pan: None,
            replay: None,
            ghost: None,
            random_map: false,
            still: None,
        });
        self.settle_scene();
    }

    // ---- the clock ----

    /// The page's clock, in microseconds. Runs every tick that is due.
    pub fn advance(&mut self, now_us: i64) {
        let dt = match self.last_us {
            None => 0,
            Some(last) => (now_us - last).clamp(0, 100_000),
        };
        self.last_us = Some(now_us);
        self.acc += dt * rules::TICKS_PER_SECOND as i64;
        let mut n = 0;
        while self.acc >= TICK_COST && n < MAX_CATCH_UP {
            self.acc -= TICK_COST;
            self.tick();
            n += 1;
        }
        if n == MAX_CATCH_UP {
            self.acc = 0;
        }
        self.now_ms = now_us / 1000;
        self.online_frame();
        self.alpha = ((self.acc * ONE as i64) / TICK_COST) as i32;
        // a joiner's lobby follows the host's map
        self.settle_scene();
        // mouse movement is applied once a frame
        let (dx, dy) = std::mem::take(&mut self.mouse);
        if let Some(p) = &mut self.play {
            let k = self.settings.sensitivity * 6;
            let dpitch = if self.settings.invert_y {
                dy * k
            } else {
                -dy * k
            };
            p.chase.turn(-dx * k, dpitch);
        }
    }

    fn tick(&mut self) {
        if self.gallery.as_ref().is_some_and(|g| g.hold) {
            return;
        }
        if self.play.as_ref().is_some_and(|p| p.still.is_some()) {
            // a tile: every frame the same, animations too
            return;
        }
        self.ticks = self.ticks.wrapping_add(1);
        let Some(mut p) = self.play.take() else {
            return;
        };
        if p.replay.is_some() {
            for cue in replay_tick(&mut p) {
                self.sounds.push(cue.key().to_string());
            }
        } else if !p.paused || p.online {
            self.step_race(&mut p);
        }
        if let Some(pan) = &mut p.pan {
            if !pan.tick() {
                p.pan = None;
            }
        }
        self.play = Some(p);
    }

    fn step_race(&mut self, p: &mut Play) {
        p.prev = p.race.racers.iter().map(|r| r.ape.pos).collect();
        let mine = self.controls(p.chase.yaw, p.chase.pitch);
        // an online race is recorded as a track of every ape (TRIAGE T24)
        if p.online && p.record.is_none() && p.race.tick == 0 {
            p.record = Some(Recording::online(&p.race, p.local));
        }
        if let Some(rec) = &mut p.record {
            if rec.track.is_none() && rec.controls.len() == p.race.tick as usize && rec.has_room() {
                rec.controls.push(mine);
            }
        }
        replay::drive(&mut p.race, &mut p.brains, p.local, mine);
        if let Some(g) = &mut p.ghost {
            g.step();
        }
        if p.online {
            self.online_step(p);
        }
        if let Some(track) = p.record.as_mut().and_then(|r| r.track.as_mut()) {
            track.record(&p.race);
        }
        follow_speed(p);
        self.hype_tick(p);
        step_chains(p);
        // the camera follows the focused ape; on another's, by itself
        let spectating = p.focus.racer != p.local;
        let f = &p.race.racers[p.focus.racer];
        let ahead = ahead_of(&p.race.course, f.ape.hint);
        p.chase.follow(
            f.ape.pos,
            f.ape.vel,
            ahead,
            self.settings.auto_camera || spectating,
        );
        spectate(p, self.alpha);
        let me = &p.race.racers[p.local];
        if me.out() {
            if let (None, Some(t)) = (p.since_out, me.finish) {
                // a personal best is kept with the settings, one per map
                // and mode
                let best = self.settings.best_mut(Map::of(&p.race.course), p.trial);
                p.new_best = best.map(|b| t < b).unwrap_or(true);
                if p.new_best {
                    *best = Some(t);
                    self.save = true;
                }
            }
            let since = p.since_out.map(|t| t + 1).unwrap_or(0);
            p.since_out = Some(since);
            if since == KEEP_AFTER_FINISH
                || (since > KEEP_AFTER_FINISH && since.is_multiple_of(KEEP_AGAIN))
            {
                self.keep_recording(p);
            }
        }
        // the sounds of the ape being watched
        for e in p.race.events.clone() {
            if let Some(cue) = cue_for(&e, p.focus.racer) {
                self.sounds.push(cue.key().to_string());
            }
        }
        // Sector Ape's flashes: an ape put back on a deck, the door shut
        if let Some(at) = crate::sectorscape::flash_of(&p.race) {
            p.flashes.extend(at);
            let tick = p.race.tick;
            p.flashes
                .retain(|f| tick.saturating_sub(f.tick) < crate::sectorscape::FLASH_TICKS);
        }
    }

    /// The hype call-outs (TRIAGE T22): crossing a hype level upward calls
    /// it out in an ape voice (`hype::Hype` keeps it from repeating); not
    /// once the local ape has finished.
    fn hype_tick(&mut self, p: &mut Play) {
        if p.race.racers[p.local].out() {
            return;
        }
        // on Sector Ape's faster dial, the same share of the way round
        let dial = crate::speedo::Dial::on(Map::of(&p.race.course));
        let kmh = crate::speedo::kmh(dial.standard(p.hype_speed));
        if let Some(level) = p.hype.step(kmh) {
            self.sounds.push(crate::hype::voice_key(level));
        }
    }

    // ---- what the page shows ----

    pub fn resize(&mut self, w: i32, h: i32) {
        self.view_w = w.max(1);
        self.view_h = h.max(1);
    }

    fn aspect(&self) -> i32 {
        fx::div(self.view_w, self.view_h)
    }

    pub fn scene_version(&self) -> u32 {
        self.scene_version
    }

    fn scenery(&self) -> &scene::Scenery {
        match self.shown {
            Shown::StageSet => &self.stage_set,
            Shown::World(m) => self.worlds[slot(m)].as_ref().unwrap_or(&self.stage_set),
        }
    }

    /// The static instances and the terrain for the current stage (its
    /// draws come with each frame, only the chunks near the camera).
    pub fn static_scene(&self) -> (Vec<i32>, Vec<u32>, Vec<i32>) {
        let s = self.scenery();
        (s.values.clone(), Vec::new(), s.terrain.clone())
    }

    fn camera(&self) -> Camera {
        if let Some(g) = &self.gallery {
            return g.camera();
        }
        match self.stage {
            Stage::Race => match &self.play {
                Some(p) => {
                    let cam = play_camera(p, self.alpha);
                    match &p.pan {
                        Some(pan) => pan.blend(cam, self.alpha),
                        None => cam,
                    }
                }
                None => self.flyover(),
            },
            Stage::Flyover => self.flyover(),
            // on the map, framed beside the docked menus (TRIAGE T47)
            Stage::Showcase => self.preview_camera(),
            Stage::Lineup => models::lineup_camera(self.aspect()),
        }
    }

    /// A slow glide down the chosen map's course, for the title.
    fn flyover(&self) -> Camera {
        let Some(c) = self.course_of(self.map()) else {
            return models::showcase_camera();
        };
        scene::flyover_camera(c, self.ticks, self.fly_from)
    }

    /// This frame's view values, dynamic instances and draws.
    pub fn frame(&self) -> (Vec<i32>, Vec<i32>, Vec<u32>) {
        if let Some(g) = &self.gallery {
            return self.gallery_frame(g);
        }
        let cam = self.camera();
        // the sky swings through colours at top speed (TRIAGE T20)
        let (speed, metres) = match (&self.play, self.stage) {
            (Some(p), Stage::Race) if self.settings.speed_colours => {
                let me = &p.race.racers[p.local];
                // Sector Ape's speeds are about double: the colours come
                // at the same share of its dial
                let dial = crate::speedo::Dial::on(Map::of(&p.race.course));
                (dial.standard(p.colour_speed), me.progress)
            }
            // the unlock: the title's jungle goes wild, colours turning about
            // once a second, until another menu opens
            (_, Stage::Flyover) if self.celebrate => (int(60), (self.ticks as i32) * (ONE / 2)),
            _ => (0, 0),
        };
        let vivid = self.settings.colourblind;
        // each map has its own sky, fog and light (the City at dusk)
        let look = match (&self.play, self.stage) {
            (Some(p), Stage::Race) => scene::look_of(&p.race.course),
            _ => self
                .course_of(self.shown_map())
                .map(scene::look_of)
                .unwrap_or_else(|| scene::look(self.shown_map())),
        };
        // the Ski Slope's snow greys the sky and closes the fog in
        let snowing = self.snowing();
        let look = if snowing {
            crate::snow::overcast(look)
        } else {
            look
        };
        // the colourblind mode mutes the sky and the fog as well (TRIAGE T33)
        let (sky, fog) = if vivid {
            (
                crate::sky::grey(look.sky, crate::sky::MUTE),
                crate::sky::grey(look.fog, crate::sky::MUTE),
            )
        } else {
            (look.sky, look.fog)
        };
        let colours = crate::sky::colours(sky, fog, speed, metres);
        let world = crate::sky::world(speed, metres);
        let mute = if vivid { ONE } else { 0 };
        let view = render::view_values(
            &cam,
            self.aspect(),
            look.fog_range,
            colours,
            world,
            mute,
            (look.light_dir, look.light, look.ground),
            look.rim,
            look.far,
        );
        let (vines, batch) = match self.stage {
            Stage::Race => match &self.play {
                Some(p) => {
                    let apes: Vec<V3> = p
                        .race
                        .racers
                        .iter()
                        .zip(&p.prev)
                        .map(|(r, prev)| prev.lerp(r.ape.pos, self.alpha))
                        .collect();
                    scene::dynamic(&scene::Frame {
                        race: &p.race,
                        cam: &cam,
                        apes: &apes,
                        local: Some(p.focus.racer),
                        tick: self.ticks,
                        chains: &p.chains,
                        vivid,
                        aura: self.settings.aura,
                        gear: self.dress,
                        flashes: &p.flashes,
                    })
                }
                None => (Batch::new(), Batch::new()),
            },
            Stage::Flyover => self
                .course_of(self.map())
                .map(|c| scene::flyover_dynamic(c, self.ticks, vivid, cam.eye))
                .unwrap_or_default(),
            Stage::Showcase => self.preview_batches(vivid),
            Stage::Lineup => {
                let mut b = Batch::new();
                let gear = self.dress.unwrap_or(models::Gear::on(self.map()));
                models::lineup(&mut b, self.ticks, gear);
                (Batch::new(), b)
            }
        };
        let (mut vines, mut batch) = (vines, batch);
        if snowing {
            crate::snow::flakes(&mut batch, cam.eye, self.ticks);
        }
        let halos = (vines.take_halos(), batch.take_halos());
        let mut draws = self.scenery().draws_near(cam.eye);
        let mut values = Vec::new();
        draws.extend(vines.finish_as(render::DYNAMIC_BUFFER, Some(render::VINES), &mut values));
        draws.extend(batch.finish_into(render::DYNAMIC_BUFFER, &mut values));
        if let (Stage::Race, Some(p)) = (self.stage, &self.play) {
            draws.extend(ghost_draws(p, self.alpha, self.ticks, &mut values));
        }
        // the soft glows (the City's wires) over everything else
        for h in [halos.0, halos.1] {
            draws.extend(h.finish_as(render::DYNAMIC_BUFFER, Some(render::HALO), &mut values));
        }
        (view, values, draws)
    }

    /// The view model, as JSON.
    pub fn ui(&mut self) -> String {
        let mut v = json!({
            "screen": self.screen,
            "version": format!("v{VERSION}"),
            "title": self.t("title"),
            "volume": self.settings.volume,
            "no_webgl": self.t("no_webgl"),
            "tab_icon": crate::icon::data_url(),
            "speedo_face": crate::speedo::Dial::on(self.shown_map()).face(),
            "speedo_needle": crate::speedo::banana(),
            "want_pointer": self.screen == Screen::Race
                && self
                    .play
                    .as_ref()
                    .map(|p| !p.paused && p.replay.is_none() && p.focus.racer == p.local)
                    .unwrap_or(false),
        });
        let panels = match self.screen {
            Screen::Title => self.title_panels(),
            Screen::Setup => self.setup_panels(),
            Screen::Settings => self.settings_panels(),
            Screen::Race => self.race_panels(),
            Screen::Lineup => Vec::new(),
            Screen::Online => self.online_panels(),
            Screen::Lobby => self.lobby_panels(),
            Screen::Replays => self.replays_panels(),
        };
        let still = self.play.as_ref().and_then(|p| p.still);
        let bare = still.is_some() || self.gallery.is_some();
        v["panels"] = Value::Array(if bare { Vec::new() } else { panels });
        if self.screen == Screen::Race && still != Some(false) {
            v["hud"] = self.hud();
        }
        if still == Some(true) {
            // a tile's HUD: the race's, without the replay's bar and hint
            v["hud"]["replay"] = Value::Null;
            v["hud"]["hint"] = json!("");
        }
        self.online_ui(&mut v);
        self.dock_ui(&mut v);
        if !self.sounds.is_empty() {
            v["sounds"] = json!(std::mem::take(&mut self.sounds));
        }
        if std::mem::take(&mut self.save) {
            v["save"] = json!(serde_json::to_string(&self.settings).unwrap_or_default());
        }
        if std::mem::take(&mut self.save_replays) {
            v["save_replays"] = json!(self.library.text_keeping(self.replay_room));
        }
        let w = self.wind_gain();
        v["wind"] = decimal(&format!("{}.{:02}", w / 100, w % 100));
        v.to_string()
    }

    /// The wind's loudness, 0..=100, from the followed ape's smoothed
    /// horizontal speed, as the speedometer reads it; quiet while a replay
    /// is paused, over or seeking (TRIAGE T30: it read the raw speed, a
    /// swing's fall included, and held on when a replay stopped).
    fn wind_gain(&self) -> i32 {
        match &self.play {
            Some(p) if !p.paused && self.screen == Screen::Race => {
                if p.replay.as_ref().is_some_and(|pb| pb.hushed(p.race.tick)) {
                    return 0;
                }
                fx::mul(sound::wind_gain(p.hype_speed), int(100)) / ONE
            }
            _ => 0,
        }
    }

    pub(crate) fn button(&self, id: &str) -> Value {
        json!({ "type": "button", "id": id, "text": self.t(id) })
    }

    fn title_panels(&self) -> Vec<Value> {
        vec![
            json!({ "kind": "title", "heading": self.t("title"), "sub": self.t("tagline"),
                    "click": "title.click", "items": [] }),
            json!({ "kind": "menu", "items": [
                self.button("menu.race"),
                self.button("menu.trial"),
                self.button("menu.online"),
                self.button("menu.replays"),
                self.button("menu.settings"),
            ]}),
            self.controls_panel(),
        ]
    }

    fn controls_panel(&self) -> Value {
        let rows: Vec<Value> = ["swing", "grab", "jump", "item", "look", "pause", "retry"]
            .iter()
            .map(|k| {
                json!([
                    self.t(&format!("controls.{k}")),
                    self.t(&format!("controls.{k}_keys"))
                ])
            })
            .collect();
        json!({ "kind": "controls", "heading": self.t("controls.heading"), "items": [
            { "type": "list", "rows": rows }
        ]})
    }

    pub(crate) fn stats_rows(&self, stats: &Stats) -> Value {
        let row = |key: &str, v: i32| json!({ "label": self.t(&format!("stat.{key}")), "value": v, "max": rules::STAT_MAX });
        json!([
            row("pump", stats.pump),
            row("top_speed", stats.top_speed),
            row("reach", stats.reach),
            row("air_control", stats.air_control),
            row("weight", stats.weight),
        ])
    }

    /// The chosen ape's colours (TRIAGE T49), as a select with id `id`.
    pub(crate) fn palette_select(&self, id: &str) -> Value {
        let kind = self.settings.kind;
        let options: Vec<Value> = (0..models::PALETTES)
            .map(|p| json!({ "value": p.to_string(), "text": self.palette_name(kind, p) }))
            .collect();
        json!({ "type": "select", "id": id, "label": self.t("setup.palette"),
          "value": self.settings.palette().to_string(), "options": options })
    }

    /// The name of `kind`'s palette `p`.
    pub(crate) fn palette_name(&self, kind: Kind, p: u8) -> String {
        if p == 0 {
            self.t("palette.classic")
        } else {
            self.t(&format!("palette.{}.{p}", kind.key()))
        }
    }

    fn setup_panels(&self) -> Vec<Value> {
        let kinds: Vec<Value> = Kind::ALL
            .iter()
            .map(|k| json!({ "value": k.key(), "text": self.t(&format!("ape.{}", k.key())) }))
            .collect();
        // one to three: Time Trial is the solo run (TRIAGE T26)
        let opponents: Vec<Value> = (1..=3)
            .map(|n| json!({ "value": n.to_string(), "text": n.to_string() }))
            .collect();
        let stats = self.settings.kind.race_stats(self.settings.equal_stats);
        let mut items = vec![
            self.map_tiles(
                "setup.map",
                self.settings.map,
                self.settings.random_map,
                false,
            ),
            json!({ "type": "picker", "id": "setup.ape", "label": self.t("setup.ape"),
              "value": self.settings.kind.key(), "options": kinds,
              "text": self.t(&format!("ape.{}", self.settings.kind.key())) }),
            self.palette_select("setup.palette"),
            json!({ "type": "stats", "label": self.t("setup.stats"), "rows": self.stats_rows(&stats) }),
        ];
        // the Ski Slope's weather (and Random's, which may pick it), beside
        // the colours so the two short selects share a row (TRIAGE T70)
        if self.settings.map == Map::Ski || self.settings.random_map {
            items.insert(3, self.weather_select("setup.weather"));
        }
        if self.trial {
            items.push(self.medal_table(self.settings.map, None));
            items.push(
                json!({ "type": "toggle", "id": "setup.ghost", "label": self.t("setup.ghost"),
              "value": self.settings.ghost }),
            );
        } else {
            items.push(json!({ "type": "select", "id": "setup.opponents", "label": self.t("setup.opponents"),
              "value": self.settings.ai_count.max(1).to_string(), "options": opponents }));
            let levels: Vec<Value> = crate::ai::Difficulty::ALL
                .iter()
                .map(|d| json!({ "value": d.key(), "text": self.t(&format!("ai.{}", d.key())) }))
                .collect();
            items.push(
                json!({ "type": "select", "id": "setup.ai_level", "label": self.t("setup.ai_level"),
              "value": self.settings.ai_level.key(), "options": levels }),
            );
        }
        items.push(
            json!({ "type": "toggle", "id": "setup.equal", "label": self.t("setup.equal"),
          "value": self.settings.equal_stats }),
        );
        if !self.trial {
            items.push(self.lava_toggle("setup.lava"));
            items.extend(self.item_toggles());
        }
        items.push(self.button("menu.start"));
        if self.best_replay().is_some() {
            items.push(self.quiet_button("replay.best"));
        }
        items.push(self.button("menu.back"));
        let heading = if self.trial {
            "trial.heading"
        } else {
            "setup.heading"
        };
        vec![json!({ "kind": "setup", "heading": self.t(heading), "items": items })]
    }

    /// The race setup's item checkboxes (TRIAGE T42): one per item, ticked
    /// when crates may give it, and a note when none is. Online, the host's
    /// go out with the start.
    pub(crate) fn item_toggles(&self) -> Vec<Value> {
        let options: Vec<Value> = crate::race::Item::ALL
            .iter()
            .map(|item| {
                json!({ "id": format!("setup.item.{}", item.key()),
                  "text": self.t(&format!("item.{}", item.key())),
                  "value": self.settings.items.has(*item) })
            })
            .collect();
        let note = if self.settings.items.is_empty() {
            self.t("setup.no_items")
        } else {
            String::new()
        };
        vec![
            json!({ "type": "checks", "label": self.t("setup.items"), "options": options, "note": note }),
        ]
    }

    /// The Floor is Lava checkbox (TRIAGE T37), on the setup (`setup.lava`)
    /// or the host's lobby (`lobby.lava`).
    pub(crate) fn lava_toggle(&self, id: &str) -> Value {
        json!({ "type": "toggle", "id": id, "label": self.t("setup.lava"),
          "value": self.settings.lava })
    }

    /// An item checkbox changed (`setup.item.<key>`).
    pub(crate) fn tick_item(&mut self, id: &str, value: &str) {
        let key = id.trim_start_matches("setup.item.");
        if let Some(item) = crate::race::Item::ALL.into_iter().find(|i| i.key() == key) {
            self.settings.items = self.settings.items.with(item, value == "on");
            self.save = true;
        }
    }

    pub(crate) fn quiet_button(&self, id: &str) -> Value {
        json!({ "type": "button", "id": id, "text": self.t(id), "quiet": true })
    }

    /// The target times: medal, target (the Author's hidden until Gold is
    /// earned), and a mark on each that `finish` beat.
    fn medal_table(&self, map: Map, finish: Option<u32>) -> Value {
        let medals = medals::of(map);
        let shown = medals.author_shown(self.settings.best(map, true));
        let rows: Vec<Value> = trial::Medal::ALL
            .iter()
            .map(|m| {
                let target = medals.target(*m);
                let text = if *m == trial::Medal::Author && !shown {
                    self.t("trial.hidden")
                } else {
                    clock_text(target)
                };
                let beat = finish.map(|t| t <= target).unwrap_or(false);
                json!([
                    self.t(&format!("medal.{}", m.key())),
                    text,
                    if beat {
                        self.t("trial.beaten")
                    } else {
                        String::new()
                    },
                    false
                ])
            })
            .collect();
        json!({ "type": "table", "head": [self.t("trial.medal"), self.t("trial.target"), ""], "rows": rows })
    }

    /// The kept replays, newest first, the best per course and mode marked.
    fn replays_panels(&self) -> Vec<Value> {
        let mut items: Vec<Value> = self
            .library
            .listing()
            .iter()
            .enumerate()
            .map(|(k, (r, best))| {
                let mut words = Vec::new();
                if *best {
                    words.push(self.t("replays.best"));
                }
                let map = Map::from_key(&r.course);
                words.push(match map {
                    Some(m) => self.map_name(m),
                    None => r.course.clone(),
                });
                words.push(self.t(&format!("mode.{}", r.mode.key())));
                words.push(self.t(&format!("ape.{}", r.kind().key())));
                if r.lava {
                    words.push(self.t("setup.lava"));
                }
                words.push(match r.finish {
                    Some(t) => clock_text(t),
                    None if r.eaten => self.t("results.eaten"),
                    None => String::new(),
                });
                let course = map.and_then(|m| self.course_of(m));
                // a run of an older layout or version: watch-only, noted (T55)
                if let Some(note) = course.and_then(|c| r.older(c)) {
                    words.push(self.t(note));
                }
                json!({ "type": "button", "id": format!("replays.watch.{k}"),
                        "text": words.join(" · "), "quiet": !*best,
                        "disabled": !course.is_some_and(|c| r.watchable(c)) })
            })
            .collect();
        if items.is_empty() {
            items.push(json!({ "type": "list", "rows": [[self.t("replays.none"), ""]] }));
        }
        items.push(self.button("menu.back"));
        vec![json!({ "kind": "replays", "heading": self.t("replays.heading"), "items": items })]
    }

    fn settings_panels(&self) -> Vec<Value> {
        let mut panels = vec![
            json!({ "kind": "settings", "heading": self.t("settings.heading"), "items": [
                { "type": "slider", "id": "settings.volume", "label": self.t("settings.volume"),
                  "min": 0, "max": 100, "step": 5, "value": self.settings.volume,
                  "text": format!("{}%", self.settings.volume) },
                { "type": "slider", "id": "settings.sensitivity", "label": self.t("settings.sensitivity"),
                  "min": 1, "max": 10, "step": 1, "value": self.settings.sensitivity,
                  "text": self.settings.sensitivity.to_string() },
                { "type": "toggle", "id": "settings.invert", "label": self.t("settings.invert"),
                  "value": self.settings.invert_y },
                { "type": "toggle", "id": "settings.auto_camera", "label": self.t("settings.auto_camera"),
                  "value": self.settings.auto_camera },
                { "type": "toggle", "id": "settings.colourblind", "label": self.t("settings.colourblind"),
                  "value": self.settings.colourblind },
                { "type": "toggle", "id": "settings.aura", "label": self.t("settings.aura"),
                  "value": self.settings.aura },
            ]}),
        ];
        // Speed Colours shows once unlocked (TRIAGE row 26)
        if let Some(items) = panels[0]["items"].as_array_mut() {
            items.push(self.weather_select("settings.weather"));
            if self.settings.colours_unlocked {
                items.push(json!({ "type": "toggle", "id": "settings.speed_colours",
                  "label": self.t("settings.speed_colours"), "value": self.settings.speed_colours }));
            }
            items.push(self.button("menu.back"));
        }
        panels
    }

    fn race_panels(&self) -> Vec<Value> {
        let Some(p) = &self.play else {
            return Vec::new();
        };
        if p.online {
            return self.online_race_panels(p);
        }
        if let Some(pb) = &p.replay {
            return vec![self.replay_panel(p, pb)];
        }
        let mut out = Vec::new();
        if p.paused {
            out.push(
                json!({ "kind": "overlay", "heading": self.t("menu.paused"), "items": [
                    self.button("menu.resume"),
                    self.button("menu.restart"),
                    self.quiet_button("menu.replay"),
                    self.button("menu.quit"),
                ]}),
            );
        } else if results_due(p) {
            let map = Map::of(&p.race.course);
            if p.trial {
                let finish = p.race.racers[p.local].finish.unwrap_or(0);
                let medal = medals::of(map)
                    .earned(finish)
                    .map(|m| self.t(&format!("medal.{}", m.key())))
                    .unwrap_or_else(|| self.t("item.none"));
                out.push(
                    json!({ "kind": "results", "heading": self.t("trial.heading"), "items": [
                        { "type": "list", "rows": [
                            [self.t("setup.map"), self.map_name(map)],
                            [self.t("results.time"), clock_text(finish)],
                            [self.t("trial.medal"), medal],
                        ]},
                        self.best_row(p),
                        self.medal_table(map, Some(finish)),
                        self.button("menu.restart"),
                        self.quiet_button("menu.replay"),
                        self.button("menu.main"),
                    ]}),
                );
            } else {
                // while others race on, a smaller panel at the side, with
                // the racer followed (TRIAGE T34)
                let racing = Focus::leader_racing(&p.race).is_some();
                let mut items = vec![self.results_table(p)];
                if racing {
                    items.push(self.focus_picker(p));
                } else {
                    items.push(self.best_row(p));
                }
                items.extend([
                    self.button("menu.rematch"),
                    self.quiet_button("menu.replay"),
                    self.button("menu.main"),
                ]);
                let kind = if racing { "spectate" } else { "results" };
                out.push(
                    json!({ "kind": kind, "heading": self.t("results.heading"), "items": items }),
                );
            }
        }
        out
    }

    /// The racer the camera follows, with buttons either side to switch
    /// (A/D or the arrow keys do the same).
    pub(crate) fn focus_picker(&self, p: &Play) -> Value {
        let name = p
            .race
            .racers
            .get(p.focus.racer)
            .map(|r| r.name.clone())
            .unwrap_or_default();
        json!({ "type": "picker", "id": "focus",
                "text": format!("{} {}", self.t("focus.watching"), name) })
    }

    /// The replay viewer's controls; the seek bar is in the HUD.
    fn replay_panel(&self, p: &Play, pb: &Playback) -> Value {
        let play = if pb.playing && p.race.tick < pb.length() {
            "replay.pause"
        } else {
            "replay.play"
        };
        let speeds: Vec<Value> = replay::Speed::ALL
            .iter()
            .map(|s| json!({ "value": s.key(), "text": self.t(&format!("replay.speed.{}", s.key())) }))
            .collect();
        let views: Vec<Value> = replay::View::ALL
            .iter()
            .map(|v| json!({ "value": v.key(), "text": self.t(&format!("replay.camera.{}", v.key())) }))
            .collect();
        json!({ "kind": "replay", "heading": self.t("replay.heading"), "items": [
            { "type": "button", "id": "replay.play", "text": self.t(play) },
            { "type": "select", "id": "replay.speed", "label": self.t("replay.speed"),
              "value": pb.speed.key(), "options": speeds },
            { "type": "select", "id": "replay.camera", "label": self.t("replay.camera"),
              "value": pb.view.key(), "options": views },
            self.focus_picker(p),
            { "type": "button", "id": "replay.exit", "text": self.t("replay.exit"), "quiet": true },
        ]})
    }

    /// The personal best under the results, marked when it was just set.
    fn best_row(&self, p: &Play) -> Value {
        let map = Map::of(&p.race.course);
        let best = self.settings.best(map, p.trial);
        let best = best.map(clock_text).unwrap_or_else(|| self.t("item.none"));
        let label = if p.new_best {
            self.t("results.new_best")
        } else {
            self.t("results.best")
        };
        json!({ "type": "list", "rows": [[label, best], [self.t("setup.map"), self.map_name(map)]] })
    }

    fn results_table(&self, p: &Play) -> Value {
        let mut order: Vec<usize> = (0..p.race.racers.len()).collect();
        order.sort_by_key(|i| p.race.racers[*i].place);
        let rows: Vec<Value> = order
            .iter()
            .map(|i| {
                let r = &p.race.racers[*i];
                let shown = match (r.finish, r.eaten) {
                    (Some(t), _) => clock_text(t),
                    (None, Some(_)) => self.t("results.eaten"),
                    (None, None) => self.t("results.racing"),
                };
                json!([
                    ordinal(r.place),
                    r.name.clone(),
                    self.t(&format!("ape.{}", r.ape.kind.key())),
                    shown,
                    *i == p.local
                ])
            })
            .collect();
        json!({ "type": "table", "head": [
            self.t("results.place"), self.t("results.name"), self.t("results.ape"), self.t("results.time")
        ], "rows": rows })
    }

    fn hud(&self) -> Value {
        let Some(p) = &self.play else {
            return Value::Null;
        };
        let race = &p.race;
        let me = &race.racers[p.local];
        let big = match race.phase {
            Phase::Countdown(left) => {
                let n = (left + rules::TICKS_PER_SECOND - 1) / rules::TICKS_PER_SECOND;
                n.to_string()
            }
            Phase::Running if race.clock < rules::TICKS_PER_SECOND as u32 => self.t("hud.go"),
            _ => match (me.finish, me.eaten) {
                (Some(_), _) if p.since_out.unwrap_or(0) < rules::TICKS_PER_SECOND as u32 * 2 => {
                    self.t("hud.finished")
                }
                // the jaguar (TRIAGE T37): until the camera moves on
                (None, Some(_)) if p.since_out.unwrap_or(0) < EATEN_SPECTATE_AFTER => {
                    self.t("hud.eaten")
                }
                _ => String::new(),
            },
        };
        // place, time and item are the followed ape's (TRIAGE T24, T34)
        let me = &race.racers[p.focus.racer];
        let shown_clock = me.finish.unwrap_or(race.clock);
        let dial = crate::speedo::Dial::on(Map::of(&race.course));
        let item = me
            .item
            .map(|i| self.t(&format!("item.{}", i.key())))
            .unwrap_or_else(|| self.t("item.none"));
        json!({
            "place": [self.t("hud.place"), format!("{}/{}", me.place, race.racers.len())],
            "time": [self.t("hud.time"), clock_text(shown_clock)],
            "best": self.hud_best(p),
            "map": [self.t("setup.map"), self.map_name(Map::of(&p.race.course))],
            "speed": [self.t("hud.speed"), format!("{} {}", p.hype_speed / ONE, self.t("hud.unit"))],
            "speedo": {
                "value": crate::speedo::kmh(p.hype_speed).to_string(),
                "unit": self.t("hud.kmh"),
                "needle": dial.needle(crate::speedo::kmh(p.hype_speed)),
                "hype": dial.hype(crate::speedo::kmh(p.hype_speed)),
                "amount": decimal(&dial.amount_text(crate::speedo::kmh(p.hype_speed))),
                "shout": self.t(&format!("hud.hype{}", dial.hype(crate::speedo::kmh(p.hype_speed)))),
                // the splash of a call-out (TRIAGE T22): how strongly the
                // shout pops (0 to 1), the screen flash's opacity, and the
                // level whose shout pops while it shows
                "flash": decimal(&p.hype.flash_text()),
                "screen": decimal(&p.hype.screen_text()),
                "burst": p.hype.burst(),
            },
            "item": [self.t("hud.item"), item],
            // the course's bananas taken (TRIAGE T43)
            "bananas": [self.t("hud.bananas"), format!("{}/{}", me.ape.bananas, rules::BANANAS_FULL)],
            "item_key": me.item.map(|i| i.key()).unwrap_or(""),
            // the held item as a picture (TRIAGE T19)
            "item_icon": me.item.map(crate::items::icon_svg).unwrap_or_default(),
            "big": big,
            // a whole message (the jaguar, TRIAGE T37) is drawn smaller
            "big_class": if big.chars().count() > 10 { "big message" } else { "big" },
            "reward": self.random_map_flash(p).unwrap_or_else(|| self.reward_flash(me, race.tick)),
            "hint": if p.replay.is_some() {
                self.t("hud.replay")
            } else if p.focus.racer != p.local {
                self.t("hud.spectate")
            } else if self.pointer_locked {
                String::new()
            } else {
                self.t("hud.click")
            },
            "minimap": self.minimap(p),
            "replay": p.replay.as_ref().map(|pb| json!({
                "at": clock_text(race.clock),
                "of": clock_text(pb.length().saturating_sub(crate::race::COUNTDOWN as u32)),
                "value": (race.tick as u64 * 1000 / pb.length().max(1) as u64).min(1000),
                "max": 1000,
            })),
        })
    }

    /// A course reward's name, for a second after it came (group K: the
    /// mushroom's push, a catch-up vine).
    fn reward_flash(&self, me: &crate::race::Racer, tick: u32) -> String {
        match me.reward {
            Some((r, at)) if tick < at + REWARD_FLASH => self.t(&format!("hud.reward.{}", r.key())),
            _ => String::new(),
        }
    }

    /// The HUD's Best: the race best, or in a Time Trial the next medal to
    /// aim for (TRIAGE T1).
    fn hud_best(&self, p: &Play) -> Value {
        let map = Map::of(&p.race.course);
        if !p.trial {
            let best = self.settings.best(map, false).map(clock_text);
            return json!([
                self.t("results.best"),
                best.unwrap_or_else(|| self.t("item.none"))
            ]);
        }
        let b = self.settings.best(map, true);
        let medals = medals::of(map);
        match medals.next_target(b) {
            Some(trial::Medal::Author) if !medals.author_shown(b) => {
                json!([self.t("medal.author"), self.t("trial.hidden")])
            }
            Some(m) => json!([
                self.t(&format!("medal.{}", m.key())),
                clock_text(medals.target(m))
            ]),
            None => json!([
                self.t("results.best"),
                b.map(clock_text).unwrap_or_default()
            ]),
        }
    }

    /// The overhead minimap (decision 4): the course as a line and each ape
    /// as a dot, in a 0–1000 box, north up along the course's own extent.
    fn minimap(&self, p: &Play) -> Value {
        let c = &p.race.course;
        let (mut lo_x, mut hi_x, mut lo_z, mut hi_z) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
        for q in &c.path {
            lo_x = lo_x.min(q.x);
            hi_x = hi_x.max(q.x);
            lo_z = lo_z.min(q.z);
            hi_z = hi_z.max(q.z);
        }
        let span = (hi_x - lo_x).max(hi_z - lo_z).max(ONE);
        let pad = 60;
        let to = |q: V3| {
            // x to the right, z (the way the course runs) up the map
            let x = pad + ((q.x - lo_x) as i64 * (1000 - 2 * pad) as i64 / span as i64) as i32;
            let y =
                1000 - pad - ((q.z - lo_z) as i64 * (1000 - 2 * pad) as i64 / span as i64) as i32;
            (x, y)
        };
        // the run: on the Ski Slope down each fork's left branch, with the
        // right branches drawn beside (`branches`)
        let lane = |s: i32, k: usize| {
            c.point(s)
                .add(crate::geom::right_of(c.heading_at(s)).scale(c.lanes(s)[k].0))
        };
        let line: Vec<i32> = (0..c.path.len())
            .step_by(10)
            .flat_map(|i| {
                let (x, y) = to(lane(i as i32 * crate::course::STEP, 0));
                [x, y]
            })
            .collect();
        let branches: Vec<Vec<i32>> = c
            .ski
            .as_ref()
            .map(|ski| {
                ski.forks
                    .iter()
                    .map(|f| {
                        let step = int(10);
                        (0..=(f.to - f.from) / step)
                            .flat_map(|k| {
                                let (x, y) = to(lane(f.from + k * step, 1));
                                [x, y]
                            })
                            .collect()
                    })
                    .collect()
            })
            .unwrap_or_default();
        // Sector Ape's routes through space: each drawn through its vines'
        // anchors from where it splits to where it joins (the hidden one
        // through the wreck stays hidden)
        let mut branches = branches;
        if let Some(sl) = &c.sector {
            for path in crate::sectorscape::route_lines(c, sl) {
                branches.push(
                    path.into_iter()
                        .flat_map(|q| {
                            let (x, y) = to(q);
                            [x, y]
                        })
                        .collect(),
                );
            }
        }
        let apes: Vec<Value> = p
            .race
            .racers
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let (x, y) = to(r.ape.pos);
                let me = i == p.local;
                // with the auras on, each dot is its ape's aura colour
                // (TRIAGE T59); else its kind's
                let colour = if self.settings.aura {
                    hex(scene::aura_colour(i, self.settings.colourblind))
                } else {
                    colour_of(r.ape.kind).to_string()
                };
                let mut dot = json!({ "x": x, "y": y, "me": me, "colour": colour });
                // the colourblind mode tells the apes apart by shape too
                // (TRIAGE T33)
                if self.settings.colourblind {
                    let size = if me { MINIMAP_ME_DOT } else { MINIMAP_DOT };
                    dot["shape"] = json!(minimap_shape(r.ape.kind, x, y, size * 5 / 4));
                }
                dot
            })
            .collect();
        let mut apes = apes;
        if let Some(g) = &p.ghost {
            let (x, y) = to(g.race.racers[0].ape.pos);
            apes.insert(
                0,
                json!({ "x": x, "y": y, "me": false, "colour": "#96e1ff" }),
            );
        }
        let (fx_, fy) = to(*c.path.last().unwrap_or(&V3::ZERO));
        json!({
            "size": 1000, "stride": 2, "line": line, "branches": branches, "apes": apes, "finish": [fx_, fy],
            "dot": MINIMAP_DOT, "me_dot": MINIMAP_ME_DOT, "width": 22, "ring_width": 8,
            "track": "rgba(240, 230, 190, 0.75)", "finish_colour": "#ffffff",
            "ring": "rgba(0, 0, 0, 0.6)", "me_ring": "#ffe066",
            "circle": decimal("6.2832"),
        })
    }

    pub fn sound(&self, key: &str) -> Vec<u8> {
        if key == "wind" {
            return sound::wind(self.settings.volume);
        }
        if key == sound::SCREECH {
            return sound::screech(self.settings.volume);
        }
        if let Some(level) = sound::voice_level(key) {
            return sound::voice(level, self.settings.volume);
        }
        match Cue::from_key(key) {
            Some(c) => sound::wav(c, self.settings.volume),
            None => Vec::new(),
        }
    }
}

/// Puts a save point back (scrubbing a replay).
fn restore(p: &mut Play, save: replay::SavePoint) {
    replay::put_back(save, &mut p.race, &mut p.brains);
    p.prev = p.race.racers.iter().map(|r| r.ape.pos).collect();
    p.since_out = None;
    let f = &p.race.racers[p.focus.racer];
    p.chase = Chase::new(f.ape.pos, p.chase.yaw, p.race.course.terrain);
}

/// The course's way forward a little ahead of centreline sample `hint`.
fn ahead_of(c: &Course, hint: usize) -> V3 {
    c.heading[(hint + 8).min(c.heading.len() - 1)]
}

/// The followed ape's horizontal speed, smoothed: the speedometer, the
/// colours and the wind follow it (TRIAGE rows 19 and 22, T30).
fn follow_speed(p: &mut Play) {
    let flat = p.race.racers[p.focus.racer].ape.vel.len_xz();
    p.hype_speed += (flat - p.hype_speed) / 30;
    p.colour_speed += (flat - p.colour_speed) / 120;
}

/// The camera moves to the focused racer, gliding from `from`.
fn refocus(p: &mut Play, from: Camera, ticks: u32) {
    let f = &p.race.racers[p.focus.racer];
    p.chase = Chase::new(f.ape.pos, p.chase.yaw, p.race.course.terrain);
    p.prev[p.focus.racer] = f.ape.pos;
    p.out_for = 0;
    p.danced = 0;
    p.pan = Some(Pan::new(from, ticks));
}

/// Live, after a tick: once the followed ape has been out of the race for
/// [`SPECTATE_AFTER`], the camera pans to the leading ape still racing
/// (TRIAGE T34).
fn spectate(p: &mut Play, alpha: i32) {
    let followed = &p.race.racers[p.focus.racer];
    if focus::racing(followed) {
        p.out_for = 0;
        p.danced = 0;
        return;
    }
    p.out_for = p.out_for.saturating_add(1);
    if crate::dance::dancing(&p.race, p.focus.racer) {
        p.danced = p.danced.saturating_add(1);
    }
    if p.out_for < spectate_after(followed) {
        return;
    }
    // a finisher's dance is watched first (TRIAGE T53), unless it never
    // gets to dance
    if followed.finish.is_some() && p.danced < DANCE_SHOWN && p.out_for < FINISH_WATCH_MOST {
        return;
    }
    let from = play_camera(p, alpha);
    if p.focus.follow_racing(&p.race) {
        refocus(p, from, camera::PAN_TICKS);
    }
}

/// How long the camera stays on a racer out of the race: a finisher a
/// moment, an eaten one until the jaguar has eaten (TRIAGE T37).
fn spectate_after(r: &crate::race::Racer) -> u32 {
    if r.eaten.is_some() && r.finish.is_none() {
        EATEN_SPECTATE_AFTER
    } else {
        SPECTATE_AFTER
    }
}

/// Whether the results show yet: a second after the local ape finished,
/// or once the jaguar has eaten it (TRIAGE T37).
pub(crate) fn results_due(p: &Play) -> bool {
    let me = &p.race.racers[p.local];
    let wait = if me.finish.is_none() && me.eaten.is_some() {
        // with the camera's pan on
        EATEN_SPECTATE_AFTER
    } else {
        rules::TICKS_PER_SECOND as u32 + 1
    };
    p.since_out.is_some_and(|t| t >= wait)
}

/// The race camera as it is, before any pan.
fn play_camera(p: &Play, alpha: i32) -> Camera {
    match p.replay.as_ref().map(|pb| pb.view) {
        None | Some(replay::View::Chase) => p.chase.camera(alpha),
        Some(v) => replay_camera(p, v, alpha),
    }
}

/// One page tick of a replay: as many race ticks as the speed (or a seek)
/// wants, the player's recorded camera with them when it follows the
/// player. Returns the sounds of the followed ape, none while seeking.
fn replay_tick(p: &mut Play) -> Vec<Cue> {
    let Some(mut pb) = p.replay.take() else {
        return Vec::new();
    };
    let n = pb.due(p.race.tick);
    let seeking = pb.seek.is_some();
    let mut cues = Vec::new();
    for _ in 0..n {
        pb.keep(&mut p.race, &p.brains);
        let c = pb.rec.at(p.race.tick);
        p.prev = p.race.racers.iter().map(|r| r.ape.pos).collect();
        pb.rec.play(&mut p.race, &mut p.brains);
        pb.orbit.tick();
        follow_speed(p);
        let f = &p.race.racers[p.focus.racer];
        let ahead = ahead_of(&p.race.course, f.ape.hint);
        // the player's own camera when it follows the player; else by itself
        let own = p.focus.racer == p.local && pb.rec.track.is_none();
        if own {
            p.chase.yaw = c.yaw;
            p.chase.pitch = c.pitch;
        }
        p.chase.follow(f.ape.pos, f.ape.vel, ahead, !own);
        if p.race.racers[p.local].out() {
            p.since_out = Some(p.since_out.map(|t| t + 1).unwrap_or(0));
        }
        if !seeking {
            cues.extend(
                p.race
                    .events
                    .iter()
                    .filter_map(|e| cue_for(e, p.focus.racer)),
            );
        }
    }
    p.replay = Some(pb);
    if n > 0 {
        step_chains(p);
    }
    cues
}

/// The orbit and trackside replay cameras, on the followed ape.
fn replay_camera(p: &Play, view: replay::View, alpha: i32) -> Camera {
    let i = p.focus.racer;
    let me = &p.race.racers[i];
    let at = p
        .prev
        .get(i)
        .copied()
        .unwrap_or(me.ape.pos)
        .lerp(me.ape.pos, alpha);
    match view {
        replay::View::Trackside => {
            // a camera every 60 m, beside the course and a little ahead
            let course = &p.race.course;
            let gap = int(60);
            let spot = ((me.progress / gap + 1) * gap).min(course.length);
            let q = course.point(spot);
            let side = crate::geom::right_of(course.heading_at(spot));
            let flip = if (spot / gap) % 2 == 0 { 1 } else { -1 };
            let foot = q.add(side.scale(int(14) * flip));
            let eye = v3(foot.x, course.terrain(foot.x, foot.z) + int(16), foot.z);
            let d = at.sub(eye);
            // zoom in on a far ape
            let fov =
                fx::mul(deg(70), fx::div(int(12), d.len().max(int(1)))).clamp(deg(10), deg(70));
            Camera {
                eye,
                fwd: d.norm(),
                fov,
            }
        }
        _ => match &p.replay {
            Some(pb) => pb.orbit.camera(at, p.race.course.terrain),
            None => camera::Orbit::default().camera(at, p.race.course.terrain),
        },
    }
}

/// The minimap's dots' radii, in its 0–1000 box.
const MINIMAP_DOT: i32 = 24;
const MINIMAP_ME_DOT: i32 = 36;

/// A kind's minimap marker in the colourblind mode (TRIAGE T33), as the
/// corners of a polygon round (`x`, `y`) of radius `r`: a triangle, a
/// square, a pentagon, a hexagon, a diamond or a star, one per kind.
fn minimap_shape(kind: crate::rules::Kind, x: i32, y: i32, r: i32) -> Vec<i32> {
    // (corners, turn of the first corner, inner radius percent for a star)
    let (n, turn, inner) = match kind.index() {
        0 => (3, 0, 100),
        1 => (4, fx::TURN / 8, 100),
        2 => (5, 0, 100),
        3 => (6, 0, 100),
        4 => (4, 0, 70),
        _ => (10, 0, 50),
    };
    (0..n)
        .flat_map(|k| {
            let a = turn + k * fx::TURN / n;
            let rr = if k % 2 == 1 { r * inner / 100 } else { r };
            [x + fx::sin(a) * rr / ONE, y - fx::cos(a) * rr / ONE]
        })
        .collect()
}

/// The ghost ape, drawn into `values` tinted and glowing (the renderer
/// has no transparency); its draws.
fn ghost_draws(p: &Play, alpha: i32, ticks: u32, values: &mut Vec<i32>) -> Vec<u32> {
    let Some(g) = &p.ghost else {
        return Vec::new();
    };
    let r = &g.race.racers[0];
    let at = g.prev.lerp(r.ape.pos, alpha);
    let mut b = Batch::new();
    let pose = scene::racer_pose(&g.race, 0, at, ticks);
    models::draw_in(&mut b, r.ape.kind, r.palette, &pose);
    let start = values.len();
    let draws = b.finish_into(render::DYNAMIC_BUFFER, values);
    for inst in values[start..].chunks_mut(render::INSTANCE_VALUES) {
        let c = render::mix([inst[12], inst[13], inst[14]], GHOST, ONE * 3 / 4);
        inst[12..15].copy_from_slice(&c);
        inst[15] = ONE * 3 / 5;
    }
    draws
}

/// The ghost's colour.
const GHOST: render::Rgb = render::rgb(150, 225, 255);

/// Vines near the local ape are chains that bend (TRIAGE row 12); those
/// further off keep their last shape until the ape comes near.
fn step_chains(p: &mut Play) {
    let c = &p.race.course;
    // a summoned vine (TRIAGE T12) adds a chain or, put where its slot's
    // last one was, starts its own: the other chains keep their shape
    p.chains.truncate(c.vines.len());
    for i in 0..c.vines.len() {
        let a = c.anchor(i, p.race.tick);
        let fresh = || crate::vinesim::Chain::hanging_along(a, c.vines[i].length, c.hang(i));
        match p.chains.get(i) {
            None => p.chains.push(fresh()),
            Some(ch) if i >= p.race.base_vines && ch.pts[0].dist(a) > int(2) => {
                p.chains[i] = fresh()
            }
            Some(_) => {}
        }
    }
    let me = p.race.racers[p.focus.racer].ape.pos;
    for i in 0..c.vines.len() {
        let a = c.anchor(i, p.race.tick);
        if a.flat().sub(me.flat()).len() > int(170) {
            continue;
        }
        let hands = p
            .race
            .racers
            .iter()
            .find(|r| r.ape.swinging() == Some(i))
            .map(|r| r.ape.pos);
        p.chains[i].step(a, c.vines[i].length, hands, p.race.tick, i as u32);
    }
}

/// A JSON number with a fractional part, written as text: the page needs
/// some (a gain, a full circle in radians), and the core keeps no floats.
fn decimal(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or(Value::Null)
}

/// A colour as the page's `#rrggbb`.
fn hex(c: render::Rgb) -> String {
    let byte = |v: i32| (v.clamp(0, ONE) * 255 + ONE / 2) / ONE;
    format!("#{:02x}{:02x}{:02x}", byte(c[0]), byte(c[1]), byte(c[2]))
}

/// An ape kind's colour on the minimap.
fn colour_of(k: Kind) -> &'static str {
    match k {
        Kind::Orangutan => "#e0702c",
        Kind::Gorilla => "#3a3a44",
        Kind::Chimpanzee => "#6b4a30",
        Kind::Gibbon => "#d8c49a",
        Kind::Proboscis => "#c8864a",
        Kind::Lemur => "#9a9aa4",
    }
}

pub(crate) fn ordinal(n: usize) -> String {
    let suffix = match (n % 10, n % 100) {
        (1, 11) | (2, 12) | (3, 13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

/// Race ticks as m:ss.hh.
pub fn clock_text(ticks: u32) -> String {
    let hundredths = ticks as u64 * 100 / rules::TICKS_PER_SECOND as u64;
    let m = hundredths / 6000;
    let sec = (hundredths / 100) % 60;
    let h = hundredths % 100;
    format!("{m}:{sec:02}.{h:02}")
}

/// How long a course reward's name shows on the HUD, ticks.
const REWARD_FLASH: u32 = rules::TICKS_PER_SECOND as u32;

/// The sound a race event makes, for the player at `local`.
fn cue_for(e: &Event, local: usize) -> Option<Cue> {
    match *e {
        Event::Body { who, what } if who == local => Some(match what {
            Happening::Grab => Cue::Grab,
            Happening::Release => Cue::Release,
            Happening::Miss => Cue::Miss,
            Happening::Jump => Cue::Jump,
            Happening::Land => Cue::Land,
            Happening::Knocked => Cue::Hit,
            // its sound comes with the reward
            Happening::Bounce => return None,
            // (Sector Ape; the art may give it its own sound)
            Happening::Rebound => Cue::Bounce,
            Happening::Wall => return None,
        }),
        Event::Countdown { .. } => Some(Cue::Beep),
        Event::Go => Some(Cue::Go),
        Event::Pickup { who, .. } if who == local => Some(Cue::Pickup),
        Event::Used { who, item } if who == local => Some(match item {
            crate::race::Item::Boost => Cue::Boost,
            _ => Cue::Throw,
        }),
        Event::Hit { who, by, .. } if who == local || by == local => Some(Cue::Hit),
        Event::Finish { who, .. } if who == local => Some(Cue::Finish),
        Event::Pounce { who } if who == local => Some(Cue::Roar),
        Event::Reset { who, .. } if who == local => Some(Cue::Reset),
        // (the door shuts for everyone: every player hears it)
        Event::Door { .. } => Some(Cue::Door),
        Event::Reward { who, what } if who == local => Some(match what {
            crate::race::Reward::Mushroom => Cue::Bounce,
            crate::race::Reward::Catchup => Cue::Catchup,
            crate::race::Reward::Banana => Cue::Banana,
        }),
        _ => None,
    }
}

#[cfg(test)]
#[path = "app/items_hype_tests.rs"]
mod items_hype_tests;
#[cfg(test)]
mod lava_tests;
#[cfg(test)]
mod look_tests;
#[cfg(test)]
mod spectate_tests;
#[cfg(test)]
mod weather_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App::new(include_str!("../../../data/strings.json"), "", 7).unwrap()
    }

    #[test]
    fn a_race_starts_from_setup_with_the_chosen_opponents() {
        let mut a = app();
        a.act("menu.race", "");
        assert_eq!(a.screen, Screen::Setup);
        a.act("setup.ape", "gibbon");
        a.act("setup.opponents", "2");
        a.act("menu.start", "");
        assert_eq!(a.screen, Screen::Race);
        let p = a.play.as_ref().unwrap();
        assert_eq!(p.race.racers.len(), 3);
        assert_eq!(p.race.racers[0].ape.kind, Kind::Gibbon);
        assert_eq!(p.race.racers[0].driver, Driver::Local);
        assert!(p.race.racers[1..]
            .iter()
            .all(|r| r.driver == Driver::Ai && r.ape.kind != Kind::Gibbon));
    }

    #[test]
    fn a_reset_and_the_door_have_their_own_sounds() {
        // (Sector Ape, the course agent's SC1: a warp back onto a deck for
        // the ape put back, and the door's slam for everyone)
        let reset = Event::Reset { who: 0, pad: 3 };
        assert_eq!(cue_for(&reset, 0), Some(Cue::Reset));
        assert_eq!(cue_for(&reset, 1), None);
        let door = Event::Door { who: 2 };
        assert_eq!(cue_for(&door, 0), Some(Cue::Door));
        assert_eq!(cue_for(&door, 2), Some(Cue::Door));
        assert_ne!(sound::wav(Cue::Reset, 50), sound::wav(Cue::Door, 50));
    }

    #[test]
    fn the_clock_runs_ticks_at_the_tick_rate() {
        let mut a = app();
        a.act("menu.race", "");
        a.act("menu.start", "");
        a.advance(0);
        // one second in 16.667 ms frames
        for k in 1..=60 {
            a.advance(k * 16_667);
        }
        let t = a.play.as_ref().unwrap().race.tick;
        assert!((119..=121).contains(&t), "{t}");
        // and the same second in two frames gives the same ticks
        let mut b = app();
        b.act("menu.race", "");
        b.act("menu.start", "");
        b.advance(0);
        b.advance(500_000);
        b.advance(1_000_020);
        assert!(b.play.as_ref().unwrap().race.tick >= 24);
    }

    #[test]
    fn a_summoned_vine_adds_a_chain_and_leaves_the_others_bent() {
        let mut a = app();
        a.act("menu.race", "");
        a.act("menu.start", "");
        let p = a.play.as_mut().unwrap();
        step_chains(p);
        let n = p.race.course.vines.len();
        assert_eq!(p.chains.len(), n);
        // a far vine's chain, bent: it is not stepped from here
        let me = p.race.racers[p.local].ape.pos;
        let far = (0..n)
            .find(|i| p.race.course.vines[*i].anchor.dist(me) > int(300))
            .unwrap();
        p.chains[far].pts[4].x += int(2);
        let bent = p.chains[far];
        let local = p.local;
        p.race.racers[local].item = Some(crate::race::Item::Vine);
        crate::items::use_item(&mut p.race, local, &Controls::default());
        step_chains(p);
        assert_eq!(p.chains.len(), n + 1);
        assert_eq!(p.chains[far], bent);
        assert_eq!(p.chains[n].pts[0], p.race.course.vines[n].anchor);
        // the slot's next vine elsewhere starts its own chain
        let mut s = p.race.summons[0];
        s.serial += crate::rules::SUMMON_SLOTS as u32;
        s.anchor = s.anchor.add(v3(int(30), 0, 0));
        p.race.place_vine(s);
        step_chains(p);
        assert_eq!(p.chains[n].pts[0], s.anchor);
    }

    #[test]
    fn escape_pauses_and_resumes_the_race() {
        let mut a = app();
        a.act("menu.race", "");
        a.act("menu.start", "");
        a.key("Escape", true);
        assert!(a.play.as_ref().unwrap().paused);
        let t = a.play.as_ref().unwrap().race.tick;
        a.advance(0);
        a.advance(100_000);
        assert_eq!(a.play.as_ref().unwrap().race.tick, t);
        a.act("menu.resume", "");
        assert!(!a.play.as_ref().unwrap().paused);
    }

    #[test]
    fn settings_round_trip_through_storage() {
        let mut a = app();
        a.act("settings.volume", "35");
        a.act("settings.invert", "on");
        let ui: Value = serde_json::from_str(&a.ui()).unwrap();
        let saved = ui["save"].as_str().unwrap().to_string();
        let b = App::new(include_str!("../../../data/strings.json"), &saved, 1).unwrap();
        assert_eq!(b.settings.volume, 35);
        assert!(b.settings.invert_y);
        // nonsense in storage gives the defaults
        assert_eq!(Settings::parse("{oops"), Settings::default());
    }

    #[test]
    fn a_faster_finish_becomes_the_saved_best() {
        let mut a = app();
        a.settings.best = Some(120 * 100);
        a.act("menu.race", "");
        a.act("setup.opponents", "1");
        a.act("menu.start", "");
        let p = a.play.as_mut().unwrap();
        p.race.phase = Phase::Running;
        p.race.clock = 120 * 90;
        p.race.racers[0].finish = Some(120 * 90);
        a.tick();
        assert_eq!(a.settings.best, Some(120 * 90));
        assert!(a.play.as_ref().unwrap().new_best);
        let ui: Value = serde_json::from_str(&a.ui()).unwrap();
        assert!(ui["save"].as_str().unwrap().contains("10800"));
        // a slower one does not replace it
        let mut b = app();
        b.settings.best = Some(120 * 80);
        b.act("menu.race", "");
        b.act("menu.start", "");
        let p = b.play.as_mut().unwrap();
        p.race.phase = Phase::Running;
        p.race.racers[0].finish = Some(120 * 90);
        b.tick();
        assert_eq!(b.settings.best, Some(120 * 80));
    }

    #[test]
    fn the_chosen_ai_difficulty_drives_the_ai_apes() {
        let mut a = app();
        a.act("menu.race", "");
        a.act("setup.ai_level", "sloth");
        a.act("setup.opponents", "2");
        a.act("menu.start", "");
        let p = a.play.as_ref().unwrap();
        assert_eq!(p.race.ai_level, crate::ai::Difficulty::Sloth);
        assert!(p
            .brains
            .iter()
            .all(|b| b.difficulty == crate::ai::Difficulty::Sloth));
        // and it is kept with the settings
        assert_eq!(a.settings.ai_level, crate::ai::Difficulty::Sloth);
    }

    #[test]
    fn ten_clicks_on_the_title_unlock_speed_colours() {
        let mut a = app();
        assert!(!a.settings.speed_colours && !a.settings.colours_unlocked);
        let shown = |a: &mut App| {
            a.act("menu.settings", "");
            let ui = a.ui();
            a.act("menu.back", "");
            ui.contains("settings.speed_colours")
        };
        assert!(!shown(&mut a));
        for _ in 0..9 {
            a.act("title.click", "");
        }
        assert!(!a.settings.colours_unlocked);
        a.act("title.click", "");
        assert!(a.settings.colours_unlocked && a.settings.speed_colours);
        // a fanfare, and the title's jungle goes wild at once
        let ui: Value = serde_json::from_str(&a.ui()).unwrap();
        assert!(ui["sounds"].to_string().contains("finish"));
        assert!(ui["save"]
            .as_str()
            .unwrap()
            .contains("\"colours_unlocked\":true"));
        assert!(a.frame().0[27] > 0);
        // until another menu opens; the setting stays from then on
        assert!(shown(&mut a));
        assert_eq!(a.frame().0[27], 0);
        // clicks on another screen visit do not count toward it twice
        let saved = serde_json::to_string(&a.settings).unwrap();
        let b = App::new(include_str!("../../../data/strings.json"), &saved, 1).unwrap();
        assert!(b.settings.colours_unlocked && b.settings.speed_colours);
        // a setting saved "on" before the secret existed reads as off
        let old = Settings::parse("{\"speed_colours\":true}");
        assert!(!old.speed_colours);
    }

    #[test]
    fn speed_colours_can_be_turned_off() {
        let mut a = app();
        a.settings.speed_colours = true;
        a.act("menu.race", "");
        a.act("menu.start", "");
        let fast = |a: &mut App| {
            let p = a.play.as_mut().unwrap();
            p.colour_speed = crate::fx::int(40);
            let (view, _, _) = a.frame();
            view[27]
        };
        assert!(fast(&mut a) > 0);
        a.act("settings.speed_colours", "off");
        assert_eq!(fast(&mut a), 0);
    }

    #[test]
    fn clock_text_is_minutes_seconds_hundredths() {
        assert_eq!(clock_text(0), "0:00.00");
        assert_eq!(clock_text(120 * 75 + 60), "1:15.50");
    }

    #[test]
    fn ordinals() {
        assert_eq!(ordinal(1), "1st");
        assert_eq!(ordinal(2), "2nd");
        assert_eq!(ordinal(3), "3rd");
        assert_eq!(ordinal(4), "4th");
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
        // (the racers' auras, TRIAGE T59, are glows too)
        a.act("settings.aura", "off");
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
            assert_eq!(view.len(), 50);
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

    #[test]
    fn the_ski_slopes_minimap_shows_both_branches_of_each_fork() {
        let mut a = app();
        a.act("menu.race", "");
        a.act("setup.map", "ski");
        a.act("menu.start", "");
        let ui: Value = serde_json::from_str(&a.ui()).unwrap();
        let m = &ui["hud"]["minimap"];
        let branches = m["branches"].as_array().unwrap();
        assert_eq!(branches.len(), 2, "one more branch per fork");
        let line: Vec<i64> = m["line"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_i64())
            .collect();
        for b in branches {
            let b: Vec<i64> = b
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_i64())
                .collect();
            assert!(b.len() >= 8);
            // the branch's middle is apart from every point of the run's line
            let (x, y) = (b[(b.len() / 2) & !1], b[((b.len() / 2) & !1) + 1]);
            let near = line
                .chunks(2)
                .map(|p| (p[0] - x).abs() + (p[1] - y).abs())
                .min()
                .unwrap();
            assert!(near > 4, "the branch's middle is {near} from the line");
        }
        // the other maps have none
        let mut j = app();
        j.act("menu.race", "");
        j.act("menu.start", "");
        let ui: Value = serde_json::from_str(&j.ui()).unwrap();
        assert!(ui["hud"]["minimap"]["branches"]
            .as_array()
            .unwrap()
            .is_empty());
    }

    fn ui_of(a: &mut App) -> Value {
        serde_json::from_str(&a.ui()).unwrap()
    }

    fn texts(v: &Value) -> String {
        v.to_string()
    }

    #[test]
    fn a_time_trial_is_solo_boosts_only_with_its_own_best() {
        let mut a = app();
        a.settings.best = Some(120 * 90);
        a.act("menu.trial", "");
        assert_eq!(a.screen, Screen::Setup);
        let ui = ui_of(&mut a);
        let shown = texts(&ui["panels"]);
        // the four targets, Bronze 1:40.00 worked out from the constants;
        // the Author time hidden until Gold
        assert!(
            shown.contains("1:40.00") && shown.contains("???"),
            "{shown}"
        );
        assert!(!shown.contains("setup.opponents"));
        a.act("menu.start", "");
        let p = a.play.as_mut().unwrap();
        assert_eq!(p.race.racers.len(), 1);
        assert!(p.race.items == crate::items::ItemSet::only(crate::race::Item::Boost) && p.trial);
        p.race.phase = Phase::Running;
        p.race.clock = 120 * 95;
        p.race.racers[0].finish = Some(120 * 95);
        a.tick();
        // a Time Trial best, separate from the race best
        assert_eq!(a.settings.trial_best, Some(120 * 95));
        assert_eq!(a.settings.best, Some(120 * 90));
        // 95 s earns Bronze (85–100 s); the HUD now aims for Silver
        let ui = ui_of(&mut a);
        assert_eq!(ui["hud"]["best"][1], "1:25.00");
        for _ in 0..200 {
            a.tick();
        }
        let shown = texts(&ui_of(&mut a)["panels"]);
        assert!(
            shown.contains("Bronze") && shown.contains("menu.replay"),
            "{shown}"
        );
    }

    #[test]
    fn r_twice_quickly_restarts_but_once_or_slowly_does_not() {
        let mut a = app();
        a.act("menu.race", "");
        a.act("menu.start", "");
        for _ in 0..100 {
            a.tick();
        }
        let tick = |a: &App| a.play.as_ref().unwrap().race.tick;
        a.key("KeyR", true);
        a.key("KeyR", false);
        for _ in 0..DOUBLE_TAP + 1 {
            a.tick();
        }
        a.key("KeyR", true);
        a.key("KeyR", false);
        assert!(tick(&a) > 100, "two slow presses do not restart");
        for _ in 0..DOUBLE_TAP - 1 {
            a.tick();
        }
        a.key("KeyR", true);
        a.key("KeyR", false);
        assert_eq!(tick(&a), 0, "two quick presses restart");
        assert_eq!(a.screen, Screen::Race);
        // and in a Time Trial
        a.act("menu.trial", "");
        a.act("menu.start", "");
        for _ in 0..50 {
            a.tick();
        }
        a.key("KeyR", true);
        a.key("KeyR", false);
        a.tick();
        a.key("KeyR", true);
        let p = a.play.as_ref().unwrap();
        assert!(p.trial && p.race.tick == 0);
    }

    /// Plays a race with W held and a grab now and then; the apes' places
    /// at each of `marks` ticks.
    fn play_for(a: &mut App, n: u32, marks: &[u32]) -> Vec<Vec<V3>> {
        let mut seen = Vec::new();
        a.key("KeyW", true);
        for t in 1..=n {
            if t % 150 == 0 {
                a.mouse_button(0, true);
            }
            if t % 150 == 90 {
                a.mouse_button(0, false);
            }
            if t % 40 == 0 {
                a.play.as_mut().unwrap().chase.turn(deg(3), 0);
            }
            a.tick();
            if marks.contains(&t) {
                let p = a.play.as_ref().unwrap();
                seen.push(p.race.racers.iter().map(|r| r.ape.pos).collect());
            }
        }
        a.key("KeyW", false);
        a.mouse_button(0, false);
        seen
    }

    #[test]
    fn watching_the_replay_shows_the_same_race_and_scrubs() {
        let mut a = app();
        a.act("menu.race", "");
        a.act("setup.opponents", "2");
        a.act("menu.start", "");
        let seen = play_for(&mut a, 900, &[450, 900]);
        a.act("menu.replay", "");
        let p = a.play.as_ref().unwrap();
        assert!(p.replay.is_some());
        assert_eq!(p.race.tick, 0);
        // keys do nothing to a replay
        a.key("KeyS", true);
        for _ in 0..1000 {
            a.tick();
        }
        let p = a.play.as_ref().unwrap();
        assert_eq!(p.race.tick, 900, "it stops at the end");
        assert!(!p.replay.as_ref().unwrap().playing);
        let now: Vec<V3> = p.race.racers.iter().map(|r| r.ape.pos).collect();
        assert_eq!(now, seen[1]);
        // scrub back halfway: a save point, then run forward
        a.act("replay.seek", "500");
        for _ in 0..20 {
            a.tick();
        }
        let p = a.play.as_ref().unwrap();
        let back: Vec<V3> = p.race.racers.iter().map(|r| r.ape.pos).collect();
        assert_eq!(p.race.tick, 450);
        assert_eq!(back, seen[0]);
        // Play at the end starts it over
        a.act("replay.seek", "1000");
        for _ in 0..20 {
            a.tick();
        }
        a.act("replay.play", "");
        a.tick();
        assert!(a.play.as_ref().unwrap().race.tick < 60);
        // the cameras all give a view
        for v in ["orbit", "trackside", "chase"] {
            a.act("replay.camera", v);
            assert_eq!(a.frame().0.len(), 50);
        }
        // leaving goes back to the race it came from
        a.act("replay.exit", "");
        let p = a.play.as_ref().unwrap();
        assert!(p.replay.is_none());
        assert_eq!(p.race.tick, 900);
    }

    #[test]
    fn finished_runs_are_kept_and_watchable_after_a_reload() {
        let mut a = app();
        a.act("menu.trial", "");
        a.act("menu.start", "");
        play_for(&mut a, 400, &[]);
        let p = a.play.as_mut().unwrap();
        p.race.racers[0].finish = Some(9000);
        a.act("menu.main", "");
        let ui = ui_of(&mut a);
        let saved = ui["save_replays"].as_str().unwrap().to_string();
        assert!(saved.len() < 400 * 8, "{} letters", saved.len());
        // a new visit: the best is on the Replays list and on the setup
        let mut b = app();
        b.load_replays(&saved);
        b.act("menu.replays", "");
        let shown = texts(&ui_of(&mut b)["panels"]);
        assert!(shown.contains("★ Best · Jungle · Time Trial"), "{shown}");
        b.act("replays.watch.0", "");
        assert!(b.play.as_ref().unwrap().replay.is_some());
        b.act("replay.exit", "");
        assert_eq!(b.screen, Screen::Replays);
        b.act("menu.trial", "");
        assert!(texts(&ui_of(&mut b)["panels"]).contains("replay.best"));
        b.act("replay.best", "");
        assert!(b.play.as_ref().unwrap().replay.is_some());
        // an unfinished run is not kept
        let mut c = app();
        c.act("menu.race", "");
        c.act("menu.start", "");
        c.act("menu.main", "");
        assert!(ui_of(&mut c)["save_replays"].is_null());
    }

    /// A Time Trial run on the Jungle finished in `t` ticks, kept.
    fn trial_run(a: &mut App, t: u32) {
        a.act("menu.trial", "");
        a.act("menu.start", "");
        play_for(a, 300, &[]);
        a.play.as_mut().unwrap().race.racers[0].finish = Some(t);
        a.act("menu.main", "");
    }

    #[test]
    fn a_best_of_an_older_layout_is_kept_watch_only_and_never_a_ghost() {
        // TRIAGE T55: the map was retuned since the best was made
        let mut a = app();
        trial_run(&mut a, 9000);
        a.library.best[0].layout ^= 1;
        let mut b = app();
        b.load_replays(&a.library.text());
        b.act("menu.replays", "");
        let shown = texts(&ui_of(&mut b)["panels"]);
        assert!(
            shown.contains("★ Best · Jungle · Time Trial · Chimpanzee · 1:15.00 · older layout"),
            "{shown}"
        );
        assert!(shown.contains(r#""disabled":false"#), "{shown}");
        b.act("replays.watch.0", "");
        assert!(b.play.as_ref().unwrap().replay.is_some(), "watchable");
        b.act("replay.exit", "");
        // not offered as the best to watch or race on the setup
        b.act("menu.trial", "");
        assert!(!texts(&ui_of(&mut b)["panels"]).contains("replay.best"));
        b.act("menu.start", "");
        assert!(b.play.as_ref().unwrap().ghost.is_none());
        b.act("menu.main", "");
        // a slower run on the layout as it is now: both kept, the new one
        // the ghost
        trial_run(&mut b, 12_000);
        let times: Vec<Option<u32>> = b.library.best.iter().map(|r| r.finish).collect();
        assert_eq!(times, vec![Some(9000), Some(12_000)]);
        b.act("menu.trial", "");
        b.act("menu.start", "");
        assert_eq!(
            b.play.as_ref().unwrap().ghost.as_ref().unwrap().finish,
            Some(12_000)
        );
    }

    #[test]
    fn when_storage_is_full_recent_runs_are_shed_never_a_best() {
        // TRIAGE T55: the page reports each write storage would not take
        let mut a = app();
        trial_run(&mut a, 9000);
        for k in 1..=4 {
            let mut r = a.library.best[0].clone();
            r.finish = Some(9000 + k);
            a.library.add(r);
        }
        let mut recents = Vec::new();
        a.save_replays = true;
        while let Some(text) = ui_of(&mut a)["save_replays"].as_str() {
            let saved = Library::parse(text);
            assert_eq!(saved.best, a.library.best, "a best was shed");
            recents.push(saved.recent.len());
            a.replays_not_saved();
            assert!(recents.len() < 10, "it stops");
        }
        assert_eq!(recents, vec![4, 3, 2, 1, 0]);
        // nothing is lost from this visit's list
        assert_eq!(a.library.recent.len(), 4);
    }

    #[test]
    fn the_best_time_trial_races_as_a_ghost() {
        let mut a = app();
        a.act("menu.trial", "");
        a.act("menu.start", "");
        let seen = play_for(&mut a, 600, &[600]);
        a.play.as_mut().unwrap().race.racers[0].finish = Some(9000);
        a.act("menu.restart", "");
        let p = a.play.as_ref().unwrap();
        assert!(p.ghost.is_some());
        let (_, plain, _) = {
            let mut b = app();
            b.act("menu.trial", "");
            b.act("setup.ghost", "off");
            b.act("menu.start", "");
            assert!(b.play.as_ref().unwrap().ghost.is_none());
            b.frame()
        };
        let (_, with_ghost, _) = a.frame();
        assert!(with_ghost.len() > plain.len());
        // standing still, the player watches the ghost go the same way
        for _ in 0..600 {
            a.tick();
        }
        let g = a.play.as_ref().unwrap().ghost.as_ref().unwrap();
        assert_eq!(g.race.racers[0].ape.pos, seen[0][0]);
    }
}
