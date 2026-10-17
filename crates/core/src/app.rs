//! The app (PLAN P11, P15): screens, settings, input, the race loop and
//! what the page shows. The page passes its clock, keys and mouse in, and
//! draws the view model (`ui`) and the 3D frame this returns.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::ai::Brain;
use crate::ape::{Controls, Happening};
use crate::camera::{self, Chase, Pan};
use crate::costume::Costume;
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
mod notes_tests;
#[cfg(test)]
mod setup_tests;
mod tile;

// each domain's part of the app, as `impl App` blocks
mod audio;
mod costumes;
mod hud;
mod items;
mod menus;
mod notes;
mod race;
mod replays;
mod results;
mod setup;
mod spectate;
mod view;
mod weather;
use self::audio::cue_for;
#[cfg(test)]
use self::hud::{colour_of, minimap_shape};
use self::race::{follow_speed, step_chains};
use self::replays::{replay_camera, replay_tick};
use self::results::results_due;
use self::spectate::{ahead_of, play_camera, spectate};

/// Ticks of physics a frame may catch up at most (a stalled tab does not
/// fast-forward the race).
const MAX_CATCH_UP: u32 = 12;
/// Microseconds per tick, times the tick rate: a tick is due for every
/// `1_000_000` accumulated (the clock is kept in µs × ticks per second).
const TICK_COST: i64 = 1_000_000;

/// The left mouse button, held as if it were a key.
const GRAB_BUTTON: &str = "Mouse0";
/// The call button (TRIAGE T58): Left Shift, unused until now.
const CALL_KEY: &str = "ShiftLeft";

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
    /// the newest version whose patch notes this player has opened ("" for
    /// none): an older one lights "New" on the menu's What's New
    pub notes_seen: String,
    /// the patch notes list minor versions (x.y.z, z not 0) too
    pub notes_minor: bool,
    /// the costume this player's ape wears on every map (TRIAGE T89):
    /// `Own` (each map's gear), `Bare`, or an unlocked one
    pub costume: Costume,
    /// the costumes unlocked by beating a map's Author time (T89), by key
    pub costumes: BTreeSet<String>,
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
            items: crate::items::ItemSet::ALL.offered(),
            colourblind: false,
            map: Map::default(),
            random_map: false,
            bests: BTreeMap::new(),
            lava: false,
            weather: crate::snow::Weather::Clear,
            palettes: BTreeMap::new(),
            notes_seen: String::new(),
            notes_minor: false,
            costume: Costume::Own,
            costumes: BTreeSet::new(),
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
        s.items = s.items.clean().offered();
        // Speed Colours is a secret until unlocked (TRIAGE row 26), even for
        // settings saved while it was on by default
        s.speed_colours &= s.colours_unlocked;
        s.palettes
            .retain(|k, p| Kind::from_key(k).is_some() && *p > 0 && *p < models::PALETTES);
        // costumes (TRIAGE T89): only real, unlockable ones; an Author time
        // beaten before costumes existed unlocks its map's too
        s.costumes
            .retain(|k| Costume::from_key(k).is_some_and(|c| !c.free()));
        for map in Map::ALL {
            for trial in [false, true] {
                if let Some(c) = s.best(map, trial).and_then(|t| Costume::earned(map, t)) {
                    s.costumes.insert(c.key().to_string());
                }
            }
        }
        if !s.wearable(s.costume) {
            s.costume = Costume::Own;
        }
        s
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
    /// what each version changed (`patch_notes.rs`)
    Notes,
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
    /// the local finish unlocked this costume (TRIAGE T89)
    pub new_costume: Option<Costume>,
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
pub const VERSION: &str = "1.14.2";

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
    /// Ticks until the call button (Left Shift, TRIAGE T58) works again.
    call_clock: sound::CallClock,
    /// Ape calls for the page to play this frame: the sound key and the
    /// loudness, 0..=100 (full for the player's own or the followed ape,
    /// by distance for another's).
    calls: Vec<(String, i32)>,
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
    /// the docked panels' heights added up with the gaps between them, in
    /// CSS pixels, from the page's measures (0 until it has): two lobby
    /// panels stack in one column when this fits the height
    menus_h: i32,
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
            call_clock: sound::CallClock::default(),
            calls: Vec::new(),
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
            menus_h: 0,
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
            if code == CALL_KEY && !self.keys.contains(code) {
                self.call_pressed();
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
            Screen::Setup | Screen::Settings | Screen::Online | Screen::Replays | Screen::Notes => {
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
            Screen::Title | Screen::Settings | Screen::Online | Screen::Replays | Screen::Notes => {
                Stage::Flyover
            }
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
        if id == "dock.menus_h" {
            self.menus_h = menu_dock::stacked_height(value);
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
            "menu.replay" => self.replays_act(id, value),
            _ if id.starts_with("replay") => self.replays_act(id, value),
            "focus.next" => self.switch_focus(true),
            "focus.prev" => self.switch_focus(false),
            _ if id.starts_with("setup.") => self.setup_act(id, value),
            "menu.settings" => self.go(Screen::Settings),
            "menu.notes" => self.open_notes(),
            // the version in the corner opens the notes from the menus only,
            // never mid-race
            "version" if self.version_opens_notes() => self.open_notes(),
            "notes.minor" => self.notes_minor(value),
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
            _ if id.starts_with("settings.") => self.settings_act(id, value),
            "title.click" => self.title_click(),
            _ => {}
        }
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
        self.call_clock.tick();
        let Some(mut p) = self.play.take() else {
            return;
        };
        if p.replay.is_some() {
            let (cues, calls) = replay_tick(&mut p);
            for cue in cues {
                self.sounds.push(cue.key().to_string());
            }
            for who in calls {
                self.hear_call(&p, who);
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

    // ---- what the page shows ----

    /// The view model, as JSON.
    pub fn ui(&mut self) -> String {
        let mut v = json!({
            "screen": self.screen,
            "version": format!("v{VERSION}"),
            "version_opens": self.version_opens_notes(),
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
            Screen::Notes => self.notes_panels(),
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
        if !self.calls.is_empty() {
            let calls: Vec<Value> = std::mem::take(&mut self.calls)
                .into_iter()
                .map(|(key, g)| json!({ "key": key, "gain": decimal(&format!("{}.{:02}", g / 100, g % 100)) }))
                .collect();
            v["calls"] = Value::Array(calls);
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
}

/// A JSON number with a fractional part, written as text: the page needs
/// some (a gain, a full circle in radians), and the core keeps no floats.
fn decimal(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or(Value::Null)
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

#[cfg(test)]
mod call_tests;
#[cfg(test)]
mod costume_tests;
#[cfg(test)]
#[path = "app/items_hype_tests.rs"]
mod items_hype_tests;
#[cfg(test)]
mod lava_tests;
#[cfg(test)]
mod look_tests;
#[cfg(test)]
mod ski_camera_tests;
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

    /// The minimap is the course seen from above, not from below (TRIAGE
    /// T93): a point to the right of the ape's heading in the world lies
    /// to the right of that heading on the map too.
    #[test]
    fn the_minimap_is_seen_from_above_not_mirrored() {
        for key in ["jungle", "city", "sector"] {
            let mut a = app();
            a.act("menu.race", "");
            a.act("setup.map", key);
            a.act("menu.start", "");
            let me = a.play.as_ref().unwrap().local;
            let (pos, fwd) = {
                let p = a.play.as_ref().unwrap();
                let r = &p.race.racers[me];
                (r.ape.pos, p.race.course.heading_at(r.progress))
            };
            let right = crate::geom::right_of(fwd);
            let mut map_at = |q: V3| -> (i64, i64) {
                a.play.as_mut().unwrap().race.racers[me].ape.pos = q;
                let ui: Value = serde_json::from_str(&a.ui()).unwrap();
                let d = &ui["hud"]["minimap"]["apes"];
                let d = d
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|d| d["me"] == true)
                    .unwrap();
                (d["x"].as_i64().unwrap(), d["y"].as_i64().unwrap())
            };
            let o = map_at(pos);
            let f = map_at(pos.add(fwd.scale(int(40))));
            let r = map_at(pos.add(right.scale(int(40))));
            // on the page (y down) the right of a heading (fx, fy) is
            // (-fy, fx): a quarter turn clockwise
            let (fx_, fy) = (f.0 - o.0, f.1 - o.1);
            let (rx, ry) = (r.0 - o.0, r.1 - o.1);
            let side = rx * -fy + ry * fx_;
            assert!(side > 0, "{key}: right of the heading is drawn on its left (fwd {fx_},{fy}, right {rx},{ry})");
        }
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
            assert_eq!(a.frame().0.len(), render::VIEW_VALUES);
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
        assert!(
            shown.contains("★ Best · Canopy Run · Time Trial"),
            "{shown}"
        );
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
            shown
                .contains("★ Best · Canopy Run · Time Trial · Chimpanzee · 1:15.00 · older layout"),
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
