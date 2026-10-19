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
mod tournament;
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
    /// the Hook has been ticked once since it came back (1.17.0): settings
    /// saved while it was not offered get it ticked when read (missing
    /// from a save reads false; new settings start true)
    #[serde(default)]
    pub hook_back: bool,
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
    /// Race Setup's (and the host's lobby's) tournament choice (TRIAGE
    /// T103): the next race starts a series of `tour_races` races
    pub tournament: bool,
    /// how many races a tournament has (2..=16, default 4)
    pub tour_races: u8,
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
            hook_back: true,
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
            tournament: false,
            tour_races: crate::tournament::DEFAULT_RACES,
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
        s.tour_races = crate::tournament::clamp_races(s.tour_races);
        if !s.hook_back {
            s.items = s.items.with(crate::race::Item::Hook, true);
            s.hook_back = true;
        }
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
    /// a solo tournament's standings between races, and the final ones
    /// after the last (TRIAGE T103)
    Standings,
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
    /// a tournament's final standings: every ape dancing on the podium
    /// (TRIAGE T103)
    Podium,
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
pub const VERSION: &str = "1.18.1";

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
    /// The browser checks' fast mode (`?fast` on the page, `flag.fast`):
    /// an offline race's start countdown is run through in one frame, so
    /// a check need not sit through it. The same ticks run with the same
    /// inputs, so the race and its replay are unchanged; only the wait
    /// goes. Never on unless the page's address asks.
    pub fast_mode: bool,
    /// the solo tournament in progress (TRIAGE T103); online, the session
    /// holds the host's
    pub tour: Option<crate::tournament::Tour>,
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
            fast_mode: false,
            tour: None,
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
            // the podium stands on the small jungle stage, in its light
            (_, Stage::Podium) => Map::Jungle,
            _ => self.map(),
        }
    }

    fn wanted_scene(&self) -> Shown {
        let map = self.shown_map();
        match self.stage {
            Stage::Lineup | Stage::Podium => Shown::StageSet,
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
                .is_some_and(|p| !p.online && p.replay.is_none())
            // a tournament race is not restarted (TRIAGE T103)
            && self.tour.is_none();
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
            Screen::Setup | Screen::Lobby | Screen::Standings => Stage::Showcase,
            Screen::Lineup => Stage::Lineup,
            Screen::Race => Stage::Race,
        };
        self.stage = stage;
        self.podium_stage();
        // leaving the races and their standings drops a solo tournament
        // (TRIAGE T103: quitting returns to the setup)
        if !matches!(screen, Screen::Race | Screen::Standings) {
            self.tour = None;
        }
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
        if id == "flag.fast" {
            self.fast_mode = true;
            return;
        }
        if id == "flag.podium" {
            self.demo_podium();
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
            _ if id.starts_with("tour.") => self.tour_act(id, value),
            "menu.start" => {
                self.tour = None;
                self.start_race()
            }
            "menu.restart" | "menu.rematch" if self.tour.is_none() => self.start_race(),
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
        if self.fast_mode {
            self.run_through_countdown();
        }
        self.now_ms = now_us / 1000;
        self.online_frame();
        self.alpha = ((self.acc * ONE as i64) / TICK_COST) as i32;
        // a joiner's lobby follows the host's map, and its tournament's
        // end shows the podium
        self.podium_stage();
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

    /// Fast mode: the rest of an offline race's countdown runs now, tick by
    /// tick as it would have in real time (an online race keeps its shared
    /// start, a replay and a paused race their own pace).
    fn run_through_countdown(&mut self) {
        let counting = |a: &App| {
            a.play.as_ref().is_some_and(|p| {
                matches!(p.race.phase, crate::race::Phase::Countdown(_))
                    && p.replay.is_none()
                    && !p.online
                    && !p.paused
                    && p.still.is_none()
            })
        };
        let mut n = 0;
        while counting(self) && n <= crate::race::COUNTDOWN {
            self.tick();
            n += 1;
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
            Screen::Standings => self.standings_panels(),
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
        // T103 part 2's hook: the final order, for the podium
        if let Some(podium) = self.podium() {
            v["podium"] = podium;
        }
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
mod hud_tests;
#[cfg(test)]
mod race_tests;
#[cfg(test)]
mod replay_tests;
#[cfg(test)]
mod settings_tests;
#[cfg(test)]
mod sound_tests;
#[cfg(test)]
mod tournament_tests;
#[cfg(test)]
mod trial_tests;
#[cfg(test)]
mod view_tests;
