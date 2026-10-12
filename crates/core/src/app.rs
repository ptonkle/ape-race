//! The app (PLAN P11, P15): screens, settings, input, the race loop and
//! what the page shows. The page passes its clock, keys and mouse in, and
//! draws the view model (`ui`) and the 3D frame this returns.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::ai::{self, Brain};
use crate::ape::{Controls, Happening};
use crate::camera::Chase;
use crate::course::Course;
use crate::fx::{self, deg, int, ONE};
use crate::geom::{v3, V3};
use crate::models;
use crate::race::{Driver, Entry, Event, Phase, Race};
use crate::render::{self, Batch, Camera};
use crate::rules::{self, Kind, Stats};
use crate::scene;
use crate::sound::{self, Cue};

mod online_play;
pub use online_play::OnlinePlay;

/// Ticks of physics a frame may catch up at most (a stalled tab does not
/// fast-forward the race).
const MAX_CATCH_UP: u32 = 12;
/// Microseconds per tick, times the tick rate: a tick is due for every
/// `1_000_000` accumulated (the clock is kept in µs × ticks per second).
const TICK_COST: i64 = 1_000_000;

/// The person's settings, kept in browser storage as this JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub volume: u8,
    /// 1 to 10
    pub sensitivity: i32,
    pub invert_y: bool,
    pub auto_camera: bool,
    pub kind: Kind,
    pub ai_count: u8,
    pub equal_stats: bool,
    pub name: String,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            volume: sound::DEFAULT_VOLUME,
            sensitivity: 5,
            invert_y: false,
            auto_camera: true,
            kind: Kind::Chimpanzee,
            ai_count: 3,
            equal_stats: false,
            name: String::new(),
        }
    }
}

impl Settings {
    pub fn parse(text: &str) -> Settings {
        let mut s: Settings = serde_json::from_str(text).unwrap_or_default();
        s.volume = s.volume.min(100);
        s.sensitivity = s.sensitivity.clamp(1, 10);
        s.ai_count = s.ai_count.min(3);
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
}

pub struct App {
    strings: BTreeMap<String, String>,
    pub settings: Settings,
    pub screen: Screen,
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
            self.keys.insert(code.to_string());
        } else {
            self.keys.remove(code);
        }
    }

    pub fn mouse_button(&mut self, button: i32, down: bool) {
        if button == 0 && down && self.pointer_locked {
            self.item_pressed = true;
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
            grab: self.held(&["Space"]),
            item: std::mem::take(&mut self.item_pressed),
            yaw,
            pitch,
        }
    }

    fn escape(&mut self) {
        match self.screen {
            Screen::Race => {
                if let Some(p) = &mut self.play {
                    p.paused = !p.paused;
                }
            }
            Screen::Setup | Screen::Settings | Screen::Online => self.go(Screen::Title),
            _ => {}
        }
    }

    // ---- screens ----

    fn go(&mut self, screen: Screen) {
        self.screen = screen;
        let stage = match screen {
            Screen::Title | Screen::Settings | Screen::Online => Stage::Flyover,
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
            self.play = None;
        }
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
            "menu.race" => self.go(Screen::Setup),
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
            "setup.opponents" => {
                if let Ok(n) = value.parse::<u8>() {
                    self.settings.ai_count = n.min(3);
                    self.save = true;
                }
            }
            "setup.equal" => {
                self.settings.equal_stats = value == "on";
                self.save = true;
            }
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
        for _ in 0..self.settings.ai_count {
            let k = pool.remove(d.below(pool.len() as u32) as usize);
            entries.push(Entry {
                name: self.t(&format!("ape.{}", k.key())),
                kind: k,
                driver: Driver::Ai,
            });
        }
        let race = Race::new(
            self.course.clone(),
            &entries,
            self.settings.equal_stats,
            d.next(),
        );
        self.begin(race, 0);
    }

    /// Starts showing a race in which this browser plays ape `local`.
    pub fn begin(&mut self, race: Race, local: usize) {
        let at = race.racers[local].ape.pos;
        let chase = Chase::new(at, race.course.start_yaw);
        let prev = race.racers.iter().map(|r| r.ape.pos).collect();
        let brains = vec![Brain::default(); race.racers.len()];
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
        if !p.paused || p.online {
            self.step_race(&mut p);
        }
        self.play = Some(p);
    }

    fn step_race(&mut self, p: &mut Play) {
        p.prev = p.race.racers.iter().map(|r| r.ape.pos).collect();
        let mine = self.controls(p.chase.yaw, p.chase.pitch);
        let mut controls = Vec::with_capacity(p.race.racers.len());
        for i in 0..p.race.racers.len() {
            let c = match p.race.racers[i].driver {
                Driver::Local => mine,
                Driver::Ai => ai::think(&p.race, i, &mut p.brains[i]),
                Driver::Remote => Controls::default(),
            };
            controls.push(c);
        }
        p.race.step(&controls);
        if p.online {
            self.online_step(p);
        }
        let me = &p.race.racers[p.local];
        p.chase
            .follow(me.ape.pos, me.ape.vel, self.settings.auto_camera);
        if me.finish.is_some() {
            p.since_finish = Some(p.since_finish.map(|t| t + 1).unwrap_or(0));
        }
        for e in p.race.events.clone() {
            if let Some(cue) = cue_for(&e, p.local) {
                self.sounds.push(cue.key().to_string());
            }
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
                Some(p) => p.chase.camera(self.alpha),
                None => self.flyover(),
            },
            Stage::Flyover => self.flyover(),
            Stage::Showcase => models::showcase_camera(),
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
        let view = render::view_values(&cam, self.aspect(), scene::fog_range());
        let batch = match self.stage {
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
                    })
                }
                None => Batch::new(),
            },
            Stage::Flyover => scene::flyover_dynamic(&self.course, self.ticks),
            Stage::Showcase => {
                let mut b = Batch::new();
                models::showcase(&mut b, self.settings.kind, self.ticks);
                b
            }
            Stage::Lineup => {
                let mut b = Batch::new();
                models::lineup(&mut b, self.ticks);
                b
            }
        };
        let mut draws = self.scenery().draws_near(cam.eye);
        let mut values = Vec::new();
        draws.extend(batch.finish_into(render::DYNAMIC_BUFFER, &mut values));
        (view, values, draws)
    }

    /// The view model, as JSON.
    pub fn ui(&mut self) -> String {
        let mut v = json!({
            "screen": self.screen,
            "title": self.t("title"),
            "volume": self.settings.volume,
            "no_webgl": self.t("no_webgl"),
            "want_pointer": self.screen == Screen::Race
                && self.play.as_ref().map(|p| !p.paused).unwrap_or(false),
        });
        let panels = match self.screen {
            Screen::Title => self.title_panels(),
            Screen::Setup => self.setup_panels(),
            Screen::Settings => self.settings_panels(),
            Screen::Race => self.race_panels(),
            Screen::Lineup => Vec::new(),
            Screen::Online => self.online_panels(),
            Screen::Lobby => self.lobby_panels(),
        };
        v["panels"] = Value::Array(panels);
        if self.screen == Screen::Race {
            v["hud"] = self.hud();
        }
        self.online_ui(&mut v);
        if !self.sounds.is_empty() {
            v["sounds"] = json!(std::mem::take(&mut self.sounds));
        }
        if std::mem::take(&mut self.save) {
            v["save"] = json!(serde_json::to_string(&self.settings).unwrap_or_default());
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
            json!({ "kind": "title", "heading": self.t("title"), "sub": self.t("tagline"), "items": [] }),
            json!({ "kind": "menu", "items": [
                self.button("menu.race"),
                self.button("menu.online"),
                self.button("menu.settings"),
            ]}),
            self.controls_panel(),
        ]
    }

    fn controls_panel(&self) -> Value {
        let rows: Vec<Value> = ["swing", "grab", "item", "look", "pause"]
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
        let opponents: Vec<Value> = (0..=3)
            .map(|n| json!({ "value": n.to_string(), "text": n.to_string() }))
            .collect();
        let stats = self.settings.kind.race_stats(self.settings.equal_stats);
        vec![
            json!({ "kind": "setup", "heading": self.t("setup.heading"), "items": [
                { "type": "picker", "id": "setup.ape", "label": self.t("setup.ape"),
                  "value": self.settings.kind.key(), "options": kinds,
                  "text": self.t(&format!("ape.{}", self.settings.kind.key())) },
                { "type": "stats", "label": self.t("setup.stats"), "rows": self.stats_rows(&stats) },
                { "type": "select", "id": "setup.opponents", "label": self.t("setup.opponents"),
                  "value": self.settings.ai_count.to_string(), "options": opponents },
                { "type": "toggle", "id": "setup.equal", "label": self.t("setup.equal"),
                  "value": self.settings.equal_stats },
                self.button("menu.start"),
                self.button("menu.back"),
            ]}),
        ]
    }

    fn settings_panels(&self) -> Vec<Value> {
        vec![
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
                self.button("menu.back"),
            ]}),
        ]
    }

    fn race_panels(&self) -> Vec<Value> {
        let Some(p) = &self.play else {
            return Vec::new();
        };
        if p.online {
            return self.online_race_panels(p);
        }
        let mut out = Vec::new();
        if p.paused {
            out.push(
                json!({ "kind": "overlay", "heading": self.t("menu.paused"), "items": [
                    self.button("menu.resume"),
                    self.button("menu.restart"),
                    self.button("menu.quit"),
                ]}),
            );
        } else if p
            .since_finish
            .map(|t| t > rules::TICKS_PER_SECOND as u32)
            .unwrap_or(false)
        {
            out.push(
                json!({ "kind": "overlay", "heading": self.t("results.heading"), "items": [
                    self.results_table(p),
                    self.button("menu.rematch"),
                    self.button("menu.main"),
                ]}),
            );
        }
        out
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
            "speed": [self.t("hud.speed"), format!("{} {}", me.ape.speed() / ONE, self.t("hud.unit"))],
            "item": [self.t("hud.item"), item],
            "item_key": me.item.map(|i| i.key()).unwrap_or(""),
            "big": big,
            "hint": if self.pointer_locked { String::new() } else { self.t("hud.click") },
            "minimap": self.minimap(p),
            "cooldown": me.ape.cooldown > 0,
        })
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
                json!({ "x": x, "y": y, "me": i == p.local, "colour": colour_of(r.ape.kind) })
            })
            .collect();
        let (fx_, fy) = to(*c.path.last().unwrap_or(&V3::ZERO));
        json!({
            "size": 1000, "stride": 2, "line": line, "apes": apes, "finish": [fx_, fy],
            "dot": 24, "me_dot": 36, "width": 22, "ring_width": 8,
            "track": "rgba(240, 230, 190, 0.75)", "finish_colour": "#ffffff",
            "ring": "rgba(0, 0, 0, 0.6)", "me_ring": "#ffe066",
            "circle": decimal("6.2832"),
        })
    }

    pub fn sound(&self, key: &str) -> Vec<u8> {
        if key == "wind" {
            return sound::wind(self.settings.volume);
        }
        match Cue::from_key(key) {
            Some(c) => sound::wav(c, self.settings.volume),
            None => Vec::new(),
        }
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
            assert_eq!(view.len(), 21);
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
}
