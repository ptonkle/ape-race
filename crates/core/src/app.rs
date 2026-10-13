//! The app (PLAN P11, P15): screens, settings, input, the race loop and
//! what the page shows. The page passes its clock, keys and mouse in, and
//! draws the view model (`ui`) and the 3D frame this returns.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::ai::Brain;
use crate::ape::{Controls, Happening};
use crate::camera::Chase;
use crate::course::Course;
use crate::fx::{self, deg, int, ONE};
use crate::geom::{v3, V3};
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
mod menu_dock;
#[cfg(test)]
mod setup_tests;

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
        s
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

/// What the 3D view shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    /// the jungle, flown over slowly (behind the title)
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
    /// ticks since the local ape finished
    since_finish: Option<u32>,
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
    /// a Time Trial (TRIAGE T1)
    pub trial: bool,
    /// what the local player does each tick, for the replay (offline only)
    pub record: Option<Recording>,
    /// the recording has gone into the library
    kept: bool,
    /// watching a recording instead of racing (TRIAGE T15)
    pub replay: Option<Playback>,
    /// the best Time Trial, raced again beside the player
    pub ghost: Option<Ghost>,
}

pub struct App {
    strings: BTreeMap<String, String>,
    pub settings: Settings,
    pub screen: Screen,
    /// clicks on the title text, toward the secret (TRIAGE row 26)
    title_clicks: u32,
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
    sounds: Vec<String>,
    save: bool,
    scene_version: u32,
    view_w: i32,
    view_h: i32,
    /// the course and its scenery, built once
    course: Course,
    jungle: scene::Scenery,
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
    /// when R last went down (`ticks`), for the double tap
    last_r: Option<u32>,
    /// the finished race whose replay is being watched, to go back to
    stash: Option<Play>,
    /// where leaving a replay from the menus goes
    replay_exit: Screen,
}

fn s(map: &BTreeMap<String, String>, key: &str) -> String {
    map.get(key).cloned().unwrap_or_else(|| key.to_string())
}

impl App {
    pub fn new(strings_json: &str, settings_json: &str, seed: u64) -> Result<App, String> {
        let course = Course::jungle();
        let strings: BTreeMap<String, String> = serde_json::from_str(strings_json)
            .map_err(|e| format!("The strings file could not be read: {e}"))?;
        Ok(App {
            strings,
            settings: Settings::parse(settings_json),
            screen: Screen::Title,
            title_clicks: 0,
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
            sounds: Vec::new(),
            save: false,
            scene_version: 1,
            view_w: 1280,
            view_h: 720,
            jungle: scene::scenery(&course),
            stage_set: scene::stage_set(),
            course,
            seed,
            now_ms: 0,
            online: OnlinePlay::default(),
            trial: false,
            library: Library::default(),
            save_replays: false,
            last_r: None,
            stash: None,
            replay_exit: Screen::Title,
        })
    }

    pub fn t(&self, key: &str) -> String {
        s(&self.strings, key)
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
                if p.since_finish.is_none() && !p.online {
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
        if stage != self.stage {
            // the jungle's scenery serves the flyover and the race alike
            let jungle = |s: Stage| matches!(s, Stage::Flyover | Stage::Race);
            if !(jungle(stage) && jungle(self.stage)) {
                self.scene_version += 1;
            }
            self.stage = stage;
        }
        if screen != Screen::Race {
            if let Some(mut p) = self.play.take() {
                self.keep_recording(&mut p);
            }
            self.stash = None;
        }
    }

    /// Puts a finished offline run into the replay library (once).
    fn keep_recording(&mut self, p: &mut Play) {
        if p.kept || p.online || p.replay.is_some() {
            return;
        }
        let finish = p.race.racers[p.local].finish;
        if let (Some(rec), Some(_)) = (&p.record, finish) {
            let mut rec = rec.clone();
            rec.finish = finish;
            self.library.add(rec);
            self.save_replays = true;
            p.kept = true;
        }
    }

    /// The replays kept in browser storage, from the page at start.
    pub fn load_replays(&mut self, text: &str) {
        self.library = Library::parse(text);
    }

    /// Starts watching `rec`; leaving goes back to `exit` (or to the
    /// stashed race's results).
    fn watch(&mut self, rec: Recording, exit: Screen) -> bool {
        if !rec.fits(&self.course) {
            return false;
        }
        let race = rec.race(&self.course);
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

    /// The best kept run of the setup's mode, when it fits the course.
    fn best_replay(&self) -> Option<&Recording> {
        self.library
            .best_of(self.course.name, self.mode())
            .filter(|r| r.fits(&self.course))
    }

    /// A button or field: `id` names it, `value` is its new value.
    pub fn act(&mut self, id: &str, value: &str) {
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
                    let rec = p.record.clone().filter(|_| !p.online);
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
            _ if id.starts_with("replay.") => self.replay_act(id, value),
            "setup.ghost" => {
                self.settings.ghost = value == "on";
                self.save = true;
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
            _ => {}
        }
    }

    fn start_race(&mut self) {
        let mut entries = vec![Entry {
            name: self.t("player.you"),
            kind: self.settings.kind,
            driver: Driver::Local,
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
            });
        }
        let mut race = Race::new(
            self.course.clone(),
            &entries,
            self.settings.equal_stats,
            d.next(),
        );
        // a Time Trial: crates give only the Speed Boost (TRIAGE T13);
        // otherwise the items ticked on the setup (T42)
        race.items = if self.trial {
            crate::items::ItemSet::only(crate::race::Item::Boost)
        } else {
            self.settings.items
        };
        race.ai_level = self.settings.ai_level;
        let record = Recording::of(&race, 0, self.mode());
        // (after `begin`, which keeps the run being left: it may be the best)
        self.begin(race, 0);
        let ghost = self
            .best_replay()
            .filter(|_| self.trial && self.settings.ghost)
            .and_then(|r| Ghost::new(r, &self.course));
        if let Some(p) = &mut self.play {
            p.trial = self.trial;
            p.record = Some(record);
            p.ghost = ghost;
        }
    }

    /// Starts showing a race in which this browser plays ape `local`.
    pub fn begin(&mut self, race: Race, local: usize) {
        let at = race.racers[local].ape.pos;
        let chase = Chase::new(at, race.course.start_yaw);
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
            since_finish: None,
            hype_speed: 0,
            colour_speed: 0,
            hype: crate::hype::Hype::default(),
            new_best: false,
            chains: Vec::new(),
            trial: false,
            record: None,
            kept: false,
            replay: None,
            ghost: None,
        });
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
        self.ticks = self.ticks.wrapping_add(1);
        let Some(mut p) = self.play.take() else {
            return;
        };
        if p.replay.is_some() {
            replay_tick(&mut p);
        } else if !p.paused || p.online {
            self.step_race(&mut p);
        }
        self.play = Some(p);
    }

    fn step_race(&mut self, p: &mut Play) {
        p.prev = p.race.racers.iter().map(|r| r.ape.pos).collect();
        let mine = self.controls(p.chase.yaw, p.chase.pitch);
        if let Some(rec) = &mut p.record {
            if rec.controls.len() == p.race.tick as usize && rec.controls.len() < replay::MOST_TICKS
            {
                rec.controls.push(mine);
            }
        }
        replay::drive(&mut p.race, &mut p.brains, p.local, mine);
        if let Some(g) = &mut p.ghost {
            g.step();
        }
        let flat = p.race.racers[p.local].ape.vel.len_xz();
        p.hype_speed += (flat - p.hype_speed) / 30;
        p.colour_speed += (flat - p.colour_speed) / 120;
        self.hype_tick(p);
        if p.online {
            self.online_step(p);
        }
        step_chains(p);
        let me = &p.race.racers[p.local];
        // the course's way forward, a little ahead of the ape
        let c = &p.race.course;
        let ahead = c.heading[(me.ape.hint + 8).min(c.heading.len() - 1)];
        p.chase
            .follow(me.ape.pos, me.ape.vel, ahead, self.settings.auto_camera);
        if let Some(t) = me.finish {
            if p.since_finish.is_none() {
                // a personal best is kept with the settings, one per mode
                let best = if p.trial {
                    &mut self.settings.trial_best
                } else {
                    &mut self.settings.best
                };
                p.new_best = best.map(|b| t < b).unwrap_or(true);
                if p.new_best {
                    *best = Some(t);
                    self.save = true;
                }
            }
            let since = p.since_finish.map(|t| t + 1).unwrap_or(0);
            p.since_finish = Some(since);
            if since == KEEP_AFTER_FINISH {
                self.keep_recording(p);
            }
        }
        for e in p.race.events.clone() {
            if let Some(cue) = cue_for(&e, p.local) {
                self.sounds.push(cue.key().to_string());
            }
        }
    }

    /// The hype call-outs (TRIAGE T22): crossing a hype level upward calls
    /// it out in an ape voice (`hype::Hype` keeps it from repeating); not
    /// once the local ape has finished.
    fn hype_tick(&mut self, p: &mut Play) {
        if p.race.racers[p.local].finish.is_some() {
            return;
        }
        let kmh = crate::speedo::kmh(p.hype_speed);
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
        match self.stage {
            Stage::Flyover | Stage::Race => &self.jungle,
            Stage::Showcase | Stage::Lineup => &self.stage_set,
        }
    }

    /// The static instances and the terrain for the current stage (its
    /// draws come with each frame, only the chunks near the camera).
    pub fn static_scene(&self) -> (Vec<i32>, Vec<u32>, Vec<i32>) {
        let s = self.scenery();
        (s.values.clone(), Vec::new(), s.terrain.clone())
    }

    fn camera(&self) -> Camera {
        match self.stage {
            Stage::Race => match &self.play {
                Some(p) => match p.replay.as_ref().map(|pb| pb.view) {
                    None | Some(replay::View::Chase) => p.chase.camera(self.alpha),
                    Some(v) => replay_camera(p, v, self.alpha),
                },
                None => self.flyover(),
            },
            Stage::Flyover => self.flyover(),
            // framed beside the docked menus (TRIAGE T47)
            Stage::Showcase => self.framed(models::showcase_camera()),
            Stage::Lineup => models::lineup_camera(self.aspect()),
        }
    }

    /// A slow glide down the course, for the title.
    fn flyover(&self) -> Camera {
        let c = &self.course;
        let s = (self.ticks as i64 * int(6) as i64 / rules::TICKS_PER_SECOND as i64
            % (c.length - int(100)) as i64) as i32
            + int(40);
        let p = c.point(s);
        let ahead = c.point(s + int(30));
        let eye = v3(p.x, crate::course::ground(p.x, p.z) + int(14), p.z);
        let look = v3(ahead.x, eye.y - int(3), ahead.z);
        Camera {
            eye,
            fwd: look.sub(eye).norm(),
            fov: deg(70),
        }
    }

    /// This frame's view values, dynamic instances and draws.
    pub fn frame(&self) -> (Vec<i32>, Vec<i32>, Vec<u32>) {
        let cam = self.camera();
        // the sky swings through colours at top speed (TRIAGE T20)
        let (speed, metres) = match (&self.play, self.stage) {
            (Some(p), Stage::Race) if self.settings.speed_colours => {
                let me = &p.race.racers[p.local];
                (p.colour_speed, me.progress)
            }
            // the unlock: the title's jungle goes wild, colours turning about
            // once a second, until another menu opens
            (_, Stage::Flyover) if self.celebrate => (int(60), (self.ticks as i32) * (ONE / 2)),
            _ => (0, 0),
        };
        let vivid = self.settings.colourblind;
        // the colourblind mode mutes the sky and the fog as well (TRIAGE T33)
        let (sky, fog) = if vivid {
            (
                crate::sky::grey(scene::SKY, crate::sky::MUTE),
                crate::sky::grey(scene::FOG, crate::sky::MUTE),
            )
        } else {
            (scene::SKY, scene::FOG)
        };
        let colours = crate::sky::colours(sky, fog, speed, metres);
        let world = crate::sky::world(speed, metres);
        let mute = if vivid { ONE } else { 0 };
        let view = render::view_values(
            &cam,
            self.aspect(),
            scene::fog_range(),
            colours,
            world,
            mute,
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
                        local: Some(p.local),
                        tick: self.ticks,
                        chains: &p.chains,
                        vivid,
                    })
                }
                None => (Batch::new(), Batch::new()),
            },
            Stage::Flyover => (
                scene::flyover_dynamic(&self.course, self.ticks, vivid),
                Batch::new(),
            ),
            Stage::Showcase => {
                let mut b = Batch::new();
                models::showcase(&mut b, self.settings.kind, self.ticks);
                (Batch::new(), b)
            }
            Stage::Lineup => {
                let mut b = Batch::new();
                models::lineup(&mut b, self.ticks);
                (Batch::new(), b)
            }
        };
        let mut draws = self.scenery().draws_near(cam.eye);
        let mut values = Vec::new();
        draws.extend(vines.finish_as(render::DYNAMIC_BUFFER, Some(render::VINES), &mut values));
        draws.extend(batch.finish_into(render::DYNAMIC_BUFFER, &mut values));
        if let (Stage::Race, Some(p)) = (self.stage, &self.play) {
            draws.extend(ghost_draws(p, self.alpha, self.ticks, &mut values));
        }
        (view, values, draws)
    }

    /// The view model, as JSON.
    pub fn ui(&mut self) -> String {
        let mut v = json!({
            "screen": self.screen,
            "title": self.t("title"),
            "volume": self.settings.volume,
            "no_webgl": self.t("no_webgl"),
            "tab_icon": crate::icon::data_url(),
            "speedo_face": crate::speedo::face(),
            "speedo_needle": crate::speedo::banana(),
            "want_pointer": self.screen == Screen::Race
                && self.play.as_ref().map(|p| !p.paused && p.replay.is_none()).unwrap_or(false),
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
        v["panels"] = Value::Array(panels);
        if self.screen == Screen::Race {
            v["hud"] = self.hud();
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
            v["save_replays"] = json!(self.library.text());
        }
        let w = self.wind_gain();
        v["wind"] = decimal(&format!("{}.{:02}", w / 100, w % 100));
        v.to_string()
    }

    /// The wind's loudness, 0..=100, from the local ape's speed.
    fn wind_gain(&self) -> i32 {
        match &self.play {
            Some(p) if !p.paused && self.screen == Screen::Race => {
                let speed = p.race.racers[p.local].ape.speed();
                fx::mul(sound::wind_gain(speed), int(100)) / ONE
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
            json!({ "type": "picker", "id": "setup.ape", "label": self.t("setup.ape"),
              "value": self.settings.kind.key(), "options": kinds,
              "text": self.t(&format!("ape.{}", self.settings.kind.key())) }),
            json!({ "type": "stats", "label": self.t("setup.stats"), "rows": self.stats_rows(&stats) }),
        ];
        if self.trial {
            items.push(self.medal_table(None));
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

    /// An item checkbox changed (`setup.item.<key>`).
    pub(crate) fn tick_item(&mut self, id: &str, value: &str) {
        let key = id.trim_start_matches("setup.item.");
        if let Some(item) = crate::race::Item::ALL.into_iter().find(|i| i.key() == key) {
            self.settings.items = self.settings.items.with(item, value == "on");
            self.save = true;
        }
    }

    fn quiet_button(&self, id: &str) -> Value {
        json!({ "type": "button", "id": id, "text": self.t(id), "quiet": true })
    }

    /// The target times: medal, target (the Author's hidden until Gold is
    /// earned), and a mark on each that `finish` beat.
    fn medal_table(&self, finish: Option<u32>) -> Value {
        let shown = trial::author_shown(self.settings.trial_best);
        let rows: Vec<Value> = trial::Medal::ALL
            .iter()
            .map(|m| {
                let target = trial::target(*m);
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
                words.push(self.t(&format!("course.{}", r.course)));
                words.push(self.t(&format!("mode.{}", r.mode.key())));
                words.push(self.t(&format!("ape.{}", r.kind().key())));
                words.push(r.finish.map(clock_text).unwrap_or_default());
                json!({ "type": "button", "id": format!("replays.watch.{k}"),
                        "text": words.join(" · "), "quiet": !*best,
                        "disabled": !r.fits(&self.course) })
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
            ]}),
        ];
        // Speed Colours shows once unlocked (TRIAGE row 26)
        if let Some(items) = panels[0]["items"].as_array_mut() {
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
        } else if p
            .since_finish
            .map(|t| t > rules::TICKS_PER_SECOND as u32)
            .unwrap_or(false)
        {
            if p.trial {
                let finish = p.race.racers[p.local].finish.unwrap_or(0);
                let medal = trial::earned(finish)
                    .map(|m| self.t(&format!("medal.{}", m.key())))
                    .unwrap_or_else(|| self.t("item.none"));
                out.push(
                    json!({ "kind": "overlay", "heading": self.t("trial.heading"), "items": [
                        { "type": "list", "rows": [
                            [self.t("results.time"), clock_text(finish)],
                            [self.t("trial.medal"), medal],
                        ]},
                        self.best_row(p),
                        self.medal_table(Some(finish)),
                        self.button("menu.restart"),
                        self.quiet_button("menu.replay"),
                        self.button("menu.main"),
                    ]}),
                );
            } else {
                out.push(
                    json!({ "kind": "overlay", "heading": self.t("results.heading"), "items": [
                        self.results_table(p),
                        self.best_row(p),
                        self.button("menu.rematch"),
                        self.quiet_button("menu.replay"),
                        self.button("menu.main"),
                    ]}),
                );
            }
        }
        out
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
            { "type": "button", "id": "replay.exit", "text": self.t("replay.exit"), "quiet": true },
        ]})
    }

    /// The personal best under the results, marked when it was just set.
    fn best_row(&self, p: &Play) -> Value {
        let best = if p.trial {
            self.settings.trial_best
        } else {
            self.settings.best
        };
        let best = best.map(clock_text).unwrap_or_else(|| self.t("item.none"));
        let label = if p.new_best {
            self.t("results.new_best")
        } else {
            self.t("results.best")
        };
        json!({ "type": "list", "rows": [[label, best]] })
    }

    fn results_table(&self, p: &Play) -> Value {
        let mut order: Vec<usize> = (0..p.race.racers.len()).collect();
        order.sort_by_key(|i| p.race.racers[*i].place);
        let rows: Vec<Value> = order
            .iter()
            .map(|i| {
                let r = &p.race.racers[*i];
                let shown = match r.finish {
                    Some(t) => clock_text(t),
                    None => self.t("results.racing"),
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
            _ => match me.finish {
                Some(_) if p.since_finish.unwrap_or(0) < rules::TICKS_PER_SECOND as u32 * 2 => {
                    self.t("hud.finished")
                }
                _ => String::new(),
            },
        };
        let shown_clock = me.finish.unwrap_or(race.clock);
        let item = me
            .item
            .map(|i| self.t(&format!("item.{}", i.key())))
            .unwrap_or_else(|| self.t("item.none"));
        json!({
            "place": [self.t("hud.place"), format!("{}/{}", me.place, race.racers.len())],
            "time": [self.t("hud.time"), clock_text(shown_clock)],
            "best": self.hud_best(p),
            "speed": [self.t("hud.speed"), format!("{} {}", p.hype_speed / ONE, self.t("hud.unit"))],
            "speedo": {
                "value": crate::speedo::kmh(p.hype_speed).to_string(),
                "unit": self.t("hud.kmh"),
                "needle": crate::speedo::needle(crate::speedo::kmh(p.hype_speed)),
                "hype": crate::speedo::hype(crate::speedo::kmh(p.hype_speed)),
                "amount": decimal(&crate::speedo::amount_text(crate::speedo::kmh(p.hype_speed))),
                "shout": self.t(&format!("hud.hype{}", crate::speedo::hype(crate::speedo::kmh(p.hype_speed)))),
                // the splash of a call-out (TRIAGE T22): the screen's flash,
                // 0 to 1, and the level whose shout pops while it shows
                "flash": decimal(&p.hype.flash_text()),
                "burst": p.hype.burst(),
            },
            "item": [self.t("hud.item"), item],
            "item_key": me.item.map(|i| i.key()).unwrap_or(""),
            // the held item as a picture (TRIAGE T19)
            "item_icon": me.item.map(crate::items::icon_svg).unwrap_or_default(),
            "big": big,
            "hint": if p.replay.is_some() || self.pointer_locked {
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

    /// The HUD's Best: the race best, or in a Time Trial the next medal to
    /// aim for (TRIAGE T1).
    fn hud_best(&self, p: &Play) -> Value {
        if !p.trial {
            let best = self.settings.best.map(clock_text);
            return json!([
                self.t("results.best"),
                best.unwrap_or_else(|| self.t("item.none"))
            ]);
        }
        let b = self.settings.trial_best;
        match trial::next_target(b) {
            Some(trial::Medal::Author) if !trial::author_shown(b) => {
                json!([self.t("medal.author"), self.t("trial.hidden")])
            }
            Some(m) => json!([
                self.t(&format!("medal.{}", m.key())),
                clock_text(trial::target(m))
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
        let line: Vec<i32> = c
            .path
            .iter()
            .step_by(10)
            .flat_map(|q| {
                let (x, y) = to(*q);
                [x, y]
            })
            .collect();
        let apes: Vec<Value> = p
            .race
            .racers
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let (x, y) = to(r.ape.pos);
                let me = i == p.local;
                let mut dot = json!({ "x": x, "y": y, "me": me, "colour": colour_of(r.ape.kind) });
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
            "size": 1000, "stride": 2, "line": line, "apes": apes, "finish": [fx_, fy],
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
    p.since_finish = None;
    let me = &p.race.racers[p.local];
    p.chase = Chase::new(me.ape.pos, p.chase.yaw);
}

/// One page tick of a replay: as many race ticks as the speed (or a seek)
/// wants, the player's recorded camera with them.
fn replay_tick(p: &mut Play) {
    let Some(mut pb) = p.replay.take() else {
        return;
    };
    let n = pb.due(p.race.tick);
    for _ in 0..n {
        pb.keep(&mut p.race, &p.brains);
        let c = pb.rec.at(p.race.tick);
        p.prev = p.race.racers.iter().map(|r| r.ape.pos).collect();
        replay::drive(&mut p.race, &mut p.brains, p.local, c);
        let flat = p.race.racers[p.local].ape.vel.len_xz();
        p.hype_speed += (flat - p.hype_speed) / 30;
        p.colour_speed += (flat - p.colour_speed) / 120;
        let me = &p.race.racers[p.local];
        let course = &p.race.course;
        let ahead = course.heading[(me.ape.hint + 8).min(course.heading.len() - 1)];
        p.chase.yaw = c.yaw;
        p.chase.pitch = c.pitch;
        p.chase.follow(me.ape.pos, me.ape.vel, ahead, false);
        if me.finish.is_some() {
            p.since_finish = Some(p.since_finish.map(|t| t + 1).unwrap_or(0));
        }
    }
    p.replay = Some(pb);
    if n > 0 {
        step_chains(p);
    }
}

/// The orbit and trackside replay cameras.
fn replay_camera(p: &Play, view: replay::View, alpha: i32) -> Camera {
    let me = &p.race.racers[p.local];
    let at = p
        .prev
        .get(p.local)
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
            let eye = v3(
                foot.x,
                crate::course::ground(foot.x, foot.z) + int(16),
                foot.z,
            );
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
        _ => {
            let a = (p.race.tick as i32).wrapping_mul(deg(1) / 4);
            let look = crate::geom::facing(a, -deg(18));
            let eye = at.sub(look.scale(int(7)));
            let floor = crate::course::ground(eye.x, eye.z) + int(1);
            let eye = v3(eye.x, eye.y.max(floor), eye.z);
            Camera {
                eye,
                fwd: at.sub(eye).norm(),
                fov: deg(65),
            }
        }
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
    models::draw(&mut b, r.ape.kind, &pose);
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
        let fresh = || crate::vinesim::Chain::hanging(a, c.vines[i].length);
        match p.chains.get(i) {
            None => p.chains.push(fresh()),
            Some(ch) if i >= p.race.base_vines && ch.pts[0].dist(a) > int(2) => {
                p.chains[i] = fresh()
            }
            Some(_) => {}
        }
    }
    let me = p.race.racers[p.local].ape.pos;
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
        _ => None,
    }
}

#[cfg(test)]
#[path = "app/items_hype_tests.rs"]
mod items_hype_tests;

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
            assert_eq!(view.len(), 38);
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
        // the four targets, Bronze 1:45.00 worked out from the constants;
        // the Author time hidden until Gold
        assert!(
            shown.contains("1:45.00") && shown.contains("???"),
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
        // 95 s earns Bronze (90–105 s); the HUD now aims for Silver
        let ui = ui_of(&mut a);
        assert_eq!(ui["hud"]["best"][1], "1:30.00");
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
            assert_eq!(a.frame().0.len(), 38);
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
