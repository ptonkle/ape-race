//! Replays and ghosts (TRIAGE T15).
//!
//! A race is deterministic: the same course, entries, seed and the local
//! player's `Controls` each tick give the same race, AI apes and items
//! included (`ai::think` reads only the race and its brain). So a
//! [`Recording`] is the race's setup plus one `Controls` per tick, and
//! watching it is building the race again and stepping it. Online races
//! cannot be re-run that way (remote apes are moved by messages), so an
//! online recording is a [`Track`] instead: every ape's place, sampled
//! [`TRACK_EVERY`] ticks apart, and played back as if every ape were remote
//! (TRIAGE T24).
//!
//! The [`Library`] is what browser storage keeps: the fastest finished run
//! per course and mode (kept until beaten) and the [`RECENT`] latest others.
//! It is saved as JSON with the controls packed into a short text
//! ([`pack`]). A recording made on another layout of the course will not
//! replay the same, so each carries the course's fingerprint ([`fingerprint`])
//! and the format number [`FORMAT`]; bump that when a physics or AI change
//! would make old recordings play differently.

use serde::{Deserialize, Serialize};

use crate::ai::{self, Brain};
use crate::ape::{Controls, Mode as ApeMode};
use crate::course::Course;
use crate::fx::{int, mul, wrap, ONE};
use crate::geom::V3;
use crate::items::ItemSet;
use crate::race::{Driver, Entry, Item, Phase, Race, Summon};
use crate::rules::TICKS_PER_SECOND;

/// The recording format; recordings of another format are dropped.
pub const FORMAT: u32 = 1;
/// How many recordings besides the bests are kept.
pub const RECENT: usize = 10;
/// A save point for scrubbing every this many ticks.
pub const SAVE_EVERY: usize = TICKS_PER_SECOND as usize;
/// The longest recording kept (10 minutes).
pub const MOST_TICKS: usize = 600 * TICKS_PER_SECOND as usize;
/// Ticks a seek may run per tick of the page clock (seeking past what has
/// been played runs the race forward this fast).
pub const SEEK_PER_TICK: usize = 40;
/// An online recording samples every ape this many ticks apart (30 a
/// second) and draws the ticks between by blending.
pub const TRACK_EVERY: u32 = 4;
/// Online recordings kept among the recent ones (they are larger).
pub const ONLINE_KEPT: usize = 3;
/// Places are kept to 1/1024 m and headings to 1/4096 turn in a track.
const TRACK_POS_SHIFT: u32 = 6;
const TRACK_YAW_SHIFT: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Race,
    Trial,
    /// an online race, kept as a [`Track`]
    Online,
}

impl Mode {
    pub fn key(self) -> &'static str {
        match self {
            Mode::Race => "race",
            Mode::Trial => "trial",
            Mode::Online => "online",
        }
    }
}

/// One race as it was played, from the local player's seat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recording {
    pub course: String,
    pub layout: u32,
    pub mode: Mode,
    pub entries: Vec<Entry>,
    pub local: usize,
    pub equal_stats: bool,
    pub seed: u64,
    /// the items its crates gave (TRIAGE T42)
    pub items: ItemSet,
    /// the AI's difficulty (its brains are rebuilt from it)
    pub ai_level: crate::ai::Difficulty,
    /// the local ape's controls, one per race tick from the first
    pub controls: Vec<Controls>,
    /// the local ape's finish, in race-clock ticks
    pub finish: Option<u32>,
    /// order of keeping: higher is newer
    pub serial: u32,
    /// an online race: every ape's place instead of the controls
    pub track: Option<Track>,
}

impl Recording {
    /// Starts recording `race`, which must not have stepped yet.
    pub fn of(race: &Race, local: usize, mode: Mode) -> Recording {
        Recording {
            course: race.course.name.to_string(),
            layout: fingerprint(&race.course),
            mode,
            entries: race
                .racers
                .iter()
                .map(|r| Entry {
                    name: r.name.clone(),
                    kind: r.ape.kind,
                    driver: r.driver,
                })
                .collect(),
            local,
            equal_stats: race.equal_stats,
            seed: race.dice.0,
            items: race.items,
            ai_level: race.ai_level,
            controls: Vec::new(),
            finish: None,
            serial: 0,
            track: None,
        }
    }

    /// Starts recording an online race (`race` must not have stepped yet):
    /// a track of every ape, its first sample the start.
    pub fn online(race: &Race, local: usize) -> Recording {
        let mut r = Recording::of(race, local, Mode::Online);
        let mut t = Track::default();
        t.sample(race);
        r.track = Some(t);
        r
    }

    /// Whether it was made on this course as it is now.
    pub fn fits(&self, course: &Course) -> bool {
        self.course == course.name && self.layout == fingerprint(course)
    }

    /// The race at its first tick, as it was set up. A track's apes are all
    /// remote: the track moves them.
    pub fn race(&self, course: &Course) -> Race {
        let mut r = Race::new(course.clone(), &self.entries, self.equal_stats, self.seed);
        r.items = self.items;
        r.ai_level = self.ai_level;
        if self.track.is_some() {
            for racer in &mut r.racers {
                racer.driver = Driver::Remote;
            }
        }
        r
    }

    /// The ticks it covers.
    pub fn length(&self) -> u32 {
        match &self.track {
            Some(t) => t.ticks(),
            None => self.controls.len() as u32,
        }
    }

    /// Whether more can be recorded (the 10-minute cap).
    pub fn has_room(&self) -> bool {
        (self.length() as usize) < MOST_TICKS
    }

    /// One tick of watching it: the recorded controls, or the track.
    pub fn play(&self, race: &mut Race, brains: &mut [Brain]) {
        match &self.track {
            Some(t) => t.step(race),
            None => {
                let c = self.at(race.tick);
                drive(race, brains, self.local, c);
            }
        }
    }

    /// The controls for the race's next tick (none left: hands off).
    pub fn at(&self, tick: u32) -> Controls {
        self.controls
            .get(tick as usize)
            .copied()
            .unwrap_or_default()
    }

    /// The ape it follows, for the menu.
    pub fn kind(&self) -> crate::rules::Kind {
        self.entries
            .get(self.local)
            .map(|e| e.kind)
            .unwrap_or(crate::rules::Kind::Chimpanzee)
    }
}

/// One tick of a race: `local` takes `mine`, the AI thinks for its apes.
/// The same code runs a live race and a replay, so they cannot drift.
pub fn drive(race: &mut Race, brains: &mut [Brain], local: usize, mine: Controls) {
    let mut controls = Vec::with_capacity(race.racers.len());
    for i in 0..race.racers.len() {
        let c = match race.racers[i].driver {
            Driver::Local if i == local => mine,
            Driver::Ai => match brains.get_mut(i) {
                Some(b) => ai::think(race, i, b),
                None => Controls::default(),
            },
            _ => Controls::default(),
        };
        controls.push(c);
    }
    race.step(&controls);
}

/// A fingerprint of a course's layout: its length, vines and crates.
pub fn fingerprint(c: &Course) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    let mut mix = |v: i32| {
        h ^= v as u32;
        h = h.wrapping_mul(0x0100_0193);
    };
    mix(c.length);
    mix(c.vines.len() as i32);
    for v in &c.vines {
        mix(v.anchor.x);
        mix(v.anchor.y);
        mix(v.anchor.z);
        mix(v.length);
    }
    for k in &c.crates {
        mix(k.x);
        mix(k.y);
        mix(k.z);
    }
    h
}

// ---- packing the controls into text ----

const ABC: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn put(out: &mut String, mut v: u32) {
    loop {
        let low = (v & 31) as usize;
        v >>= 5;
        if v == 0 {
            out.push(ABC[low] as char);
            return;
        }
        out.push(ABC[low | 32] as char);
    }
}

fn zig(v: i32) -> u32 {
    (v.wrapping_shl(1) ^ (v >> 31)) as u32
}

fn unzig(u: u32) -> i32 {
    ((u >> 1) as i32) ^ -((u & 1) as i32)
}

fn buttons(c: &Controls) -> u32 {
    (c.fwd.clamp(-1, 1) + 1) as u32
        + 3 * (c.side.clamp(-1, 1) + 1) as u32
        + 9 * c.grab as u32
        + 18 * c.jump as u32
        + 36 * c.item as u32
}

/// The controls as text: runs of ticks with the same buttons and the same
/// change of yaw and pitch, each run four numbers (buttons, Δyaw, Δpitch,
/// length − 1) in a base-64 varint, 5 bits a letter.
pub fn pack(controls: &[Controls]) -> String {
    let mut out = String::new();
    let (mut yaw, mut pitch) = (0i32, 0i32);
    let mut i = 0;
    while i < controls.len() {
        let c = controls[i];
        let key = (
            buttons(&c),
            c.yaw.wrapping_sub(yaw),
            c.pitch.wrapping_sub(pitch),
        );
        let mut run = 1;
        let (mut y, mut p) = (c.yaw, c.pitch);
        while let Some(n) = controls.get(i + run) {
            if (buttons(n), n.yaw.wrapping_sub(y), n.pitch.wrapping_sub(p)) != key {
                break;
            }
            y = n.yaw;
            p = n.pitch;
            run += 1;
        }
        put(&mut out, key.0);
        put(&mut out, zig(key.1));
        put(&mut out, zig(key.2));
        put(&mut out, run as u32 - 1);
        yaw = y;
        pitch = p;
        i += run;
    }
    out
}

/// The numbers in a text of [`put`]s; `None` when it is not one.
fn numbers(text: &str) -> Option<Vec<u32>> {
    let mut nums = Vec::new();
    let mut v: u32 = 0;
    let mut shift = 0;
    for ch in text.bytes() {
        let d = ABC.iter().position(|a| *a == ch)? as u32;
        if shift > 30 {
            return None;
        }
        v |= (d & 31) << shift;
        if d & 32 == 0 {
            nums.push(v);
            v = 0;
            shift = 0;
        } else {
            shift += 5;
        }
    }
    if shift != 0 {
        return None;
    }
    Some(nums)
}

/// The controls back from [`pack`]; `None` when the text is not one.
pub fn unpack(text: &str) -> Option<Vec<Controls>> {
    let nums = numbers(text)?;
    if nums.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::new();
    let (mut yaw, mut pitch) = (0i32, 0i32);
    for q in nums.chunks(4) {
        let b = q[0];
        if b >= 72 || out.len() + q[3] as usize >= MOST_TICKS {
            return None;
        }
        for _ in 0..=q[3] {
            yaw = yaw.wrapping_add(unzig(q[1]));
            pitch = pitch.wrapping_add(unzig(q[2]));
            out.push(Controls {
                fwd: (b % 3) as i32 - 1,
                side: (b / 3 % 3) as i32 - 1,
                grab: b / 9 % 2 == 1,
                jump: b / 18 % 2 == 1,
                item: b / 36 == 1,
                yaw,
                pitch,
            });
        }
    }
    Some(out)
}

// ---- online races: a track of every ape ----

/// One ape in one sample of a [`Track`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spot {
    pub pos: V3,
    pub yaw: i32,
    pub mode: ApeMode,
    pub stunned: bool,
}

impl Spot {
    /// The ape as a track keeps it (to the track's precision).
    fn of(ape: &crate::ape::Ape) -> Spot {
        let q = |v: i32| (v >> TRACK_POS_SHIFT) << TRACK_POS_SHIFT;
        Spot {
            pos: V3 {
                x: q(ape.pos.x),
                y: q(ape.pos.y),
                z: q(ape.pos.z),
            },
            yaw: (ape.yaw >> TRACK_YAW_SHIFT) << TRACK_YAW_SHIFT,
            mode: match ape.mode {
                ApeMode::Swinging { vine, rope } => ApeMode::Swinging {
                    vine,
                    rope: q(rope),
                },
                m => m,
            },
            stunned: ape.stun > 0,
        }
    }
}

/// An online race as it was seen here (TRIAGE T24): every ape's place
/// [`TRACK_EVERY`] ticks apart, and the vines summoned in it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Track {
    /// `samples[k][i]`: ape `i` at tick `k * TRACK_EVERY`
    pub samples: Vec<Vec<Spot>>,
    /// each summoned vine and the tick it stood at
    pub summons: Vec<(u32, Summon)>,
}

/// One ape's last values while packing or unpacking a track: its mode,
/// place, last step and heading (the place and heading in track units).
type Last = (ApeMode, [i32; 3], [i32; 3], i32);

impl Track {
    /// The ticks it covers.
    pub fn ticks(&self) -> u32 {
        self.samples.len().saturating_sub(1) as u32 * TRACK_EVERY
    }

    fn sample(&mut self, race: &Race) {
        self.samples
            .push(race.racers.iter().map(|r| Spot::of(&r.ape)).collect());
    }

    /// After each tick of the live race: a sample when one falls due, and
    /// any vine summoned since the last.
    pub fn record(&mut self, race: &Race) {
        if race.tick.is_multiple_of(TRACK_EVERY)
            && race.tick / TRACK_EVERY == self.samples.len() as u32
            && (self.ticks() as usize) < MOST_TICKS
        {
            self.sample(race);
        }
        for s in &race.summons {
            let known = self
                .summons
                .iter()
                .any(|(_, o)| o.by == s.by && o.serial == s.serial);
            if !known {
                self.summons.push((race.tick, *s));
            }
        }
    }

    /// Ape `i` at `tick`, blended between its samples: place, velocity,
    /// mode, heading and whether it is stunned.
    pub fn at(&self, tick: u32, i: usize) -> Option<(V3, V3, ApeMode, i32, bool)> {
        let k = (tick / TRACK_EVERY) as usize;
        let a = *self.samples.get(k)?.get(i)?;
        let b = self
            .samples
            .get(k + 1)
            .and_then(|s| s.get(i))
            .copied()
            .unwrap_or(a);
        let t = (tick % TRACK_EVERY) as i32 * ONE / TRACK_EVERY as i32;
        let vel = b
            .pos
            .sub(a.pos)
            .scale(int(TICKS_PER_SECOND / TRACK_EVERY as i32));
        let yaw = a.yaw + mul(wrap(b.yaw - a.yaw), t);
        Some((a.pos.lerp(b.pos, t), vel, a.mode, yaw, a.stunned))
    }

    /// One tick of watching: the race steps (the clock, crates, thrown
    /// things), the summoned vines stand, and every ape is put where the
    /// track has it.
    pub fn step(&self, race: &mut Race) {
        race.step(&[]);
        // with every ape remote the race would call itself done at Go;
        // here it is done when every ape has finished
        if race.phase == Phase::Done && race.racers.iter().any(|r| r.finish.is_none()) {
            race.phase = Phase::Running;
        }
        for (t, s) in &self.summons {
            if *t == race.tick {
                race.place_vine(*s);
            }
        }
        for i in 0..race.racers.len() {
            if let Some((pos, vel, mode, yaw, stunned)) = self.at(race.tick, i) {
                race.set_remote(i, pos, vel, mode, yaw);
                race.racers[i].ape.stun = stunned as i32;
            }
        }
    }

    /// The track as text: per sample, per ape, a mode code (and a new
    /// vine), the second differences of its place and the change of
    /// heading; then `.` and the summoned vines.
    pub fn pack(&self, apes: usize) -> String {
        let mut out = String::new();
        let mut last: Vec<Last> = vec![(ApeMode::Grounded, [0; 3], [0; 3], 0); apes];
        for sample in &self.samples {
            for (spot, (mode, q, d, yaw)) in sample.iter().zip(last.iter_mut()) {
                let code = if spot.mode == *mode {
                    0
                } else {
                    match spot.mode {
                        ApeMode::Flying => 1,
                        ApeMode::Grounded => 2,
                        ApeMode::Swinging { .. } => 3,
                    }
                };
                put(&mut out, code * 2 + spot.stunned as u32);
                if let (3, ApeMode::Swinging { vine, rope }) = (code, spot.mode) {
                    put(&mut out, vine as u32);
                    put(&mut out, zig(rope >> TRACK_POS_SHIFT));
                }
                *mode = spot.mode;
                let now = [spot.pos.x, spot.pos.y, spot.pos.z].map(|v| v >> TRACK_POS_SHIFT);
                for a in 0..3 {
                    let step = now[a].wrapping_sub(q[a]);
                    put(&mut out, zig(step.wrapping_sub(d[a])));
                    d[a] = step;
                    q[a] = now[a];
                }
                let y = spot.yaw >> TRACK_YAW_SHIFT;
                put(&mut out, zig(y.wrapping_sub(*yaw)));
                *yaw = y;
            }
        }
        out.push('.');
        for (t, s) in &self.summons {
            for v in [
                *t,
                s.slot as u32,
                s.by as u32,
                s.serial,
                zig(s.anchor.x),
                zig(s.anchor.y),
                zig(s.anchor.z),
                zig(s.length),
                zig(s.life),
            ] {
                put(&mut out, v);
            }
        }
        out
    }

    /// The track back from [`Track::pack`]; `None` when it is not one.
    pub fn unpack(text: &str, apes: usize) -> Option<Track> {
        let (body, vines) = text.split_once('.')?;
        let mut it = numbers(body)?.into_iter().peekable();
        let mut track = Track::default();
        let mut last: Vec<Last> = vec![(ApeMode::Grounded, [0; 3], [0; 3], 0); apes];
        while it.peek().is_some() {
            if apes == 0 || track.samples.len() > MOST_TICKS / TRACK_EVERY as usize {
                return None;
            }
            let mut sample = Vec::with_capacity(apes);
            for (mode, q, d, yaw) in last.iter_mut() {
                let code = it.next()?;
                *mode = match code / 2 {
                    0 => *mode,
                    1 => ApeMode::Flying,
                    2 => ApeMode::Grounded,
                    3 => ApeMode::Swinging {
                        vine: it.next()? as usize,
                        rope: unzig(it.next()?) << TRACK_POS_SHIFT,
                    },
                    _ => return None,
                };
                let mut pos = [0i32; 3];
                for a in 0..3 {
                    d[a] = d[a].wrapping_add(unzig(it.next()?));
                    q[a] = q[a].wrapping_add(d[a]);
                    pos[a] = q[a] << TRACK_POS_SHIFT;
                }
                *yaw = yaw.wrapping_add(unzig(it.next()?));
                sample.push(Spot {
                    pos: V3 {
                        x: pos[0],
                        y: pos[1],
                        z: pos[2],
                    },
                    yaw: *yaw << TRACK_YAW_SHIFT,
                    mode: *mode,
                    stunned: code % 2 == 1,
                });
            }
            track.samples.push(sample);
        }
        let v = numbers(vines)?;
        if v.len() % 9 != 0 {
            return None;
        }
        for q in v.chunks(9) {
            track.summons.push((
                q[0],
                Summon {
                    slot: q[1] as usize,
                    by: q[2] as usize,
                    serial: q[3],
                    anchor: V3 {
                        x: unzig(q[4]),
                        y: unzig(q[5]),
                        z: unzig(q[6]),
                    },
                    length: unzig(q[7]),
                    life: unzig(q[8]),
                },
            ));
        }
        Some(track)
    }
}

/// A recording as browser storage keeps it.
#[derive(Serialize, Deserialize)]
struct Stored {
    course: String,
    layout: u32,
    mode: Mode,
    entries: Vec<Entry>,
    local: usize,
    equal_stats: bool,
    seed: u64,
    /// recordings from before TRIAGE T42 carry only this: a Time Trial
    #[serde(default)]
    boost_only: bool,
    /// the race's item set (TRIAGE T42); missing in older recordings
    #[serde(default)]
    items: Option<ItemSet>,
    #[serde(default)]
    ai_level: crate::ai::Difficulty,
    finish: Option<u32>,
    serial: u32,
    controls: String,
    /// an online recording's packed track
    #[serde(default, skip_serializing_if = "Option::is_none")]
    track: Option<String>,
}

impl Stored {
    fn of(r: &Recording) -> Stored {
        Stored {
            course: r.course.clone(),
            layout: r.layout,
            mode: r.mode,
            entries: r.entries.clone(),
            local: r.local,
            equal_stats: r.equal_stats,
            seed: r.seed,
            boost_only: r.items == ItemSet::only(Item::Boost),
            items: Some(r.items),
            ai_level: r.ai_level,
            finish: r.finish,
            serial: r.serial,
            controls: pack(&r.controls),
            track: r.track.as_ref().map(|t| t.pack(r.entries.len())),
        }
    }

    fn recording(self) -> Option<Recording> {
        if self.local >= self.entries.len() || self.entries.len() > 4 {
            return None;
        }
        let track = match &self.track {
            Some(text) => Some(Track::unpack(text, self.entries.len())?),
            None => None,
        };
        if (track.is_some()) != (self.mode == Mode::Online) {
            return None;
        }
        Some(Recording {
            track,
            controls: unpack(&self.controls)?,
            course: self.course,
            layout: self.layout,
            mode: self.mode,
            entries: self.entries,
            local: self.local,
            equal_stats: self.equal_stats,
            seed: self.seed,
            items: self.items.unwrap_or(if self.boost_only {
                ItemSet::only(Item::Boost)
            } else {
                ItemSet::ALL
            }),
            ai_level: self.ai_level,
            finish: self.finish,
            serial: self.serial,
        })
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Shelf {
    format: u32,
    best: Vec<Stored>,
    recent: Vec<Stored>,
}

/// The kept recordings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Library {
    /// the fastest finish per course and mode
    pub best: Vec<Recording>,
    /// the latest others, oldest first, at most [`RECENT`]
    pub recent: Vec<Recording>,
}

impl Library {
    /// From browser storage; nonsense or another format gives an empty one.
    pub fn parse(text: &str) -> Library {
        let shelf: Shelf = serde_json::from_str(text).unwrap_or_default();
        if shelf.format != FORMAT {
            return Library::default();
        }
        let take = |v: Vec<Stored>| -> Vec<Recording> {
            v.into_iter().filter_map(Stored::recording).collect()
        };
        let mut lib = Library {
            best: take(shelf.best),
            recent: take(shelf.recent),
        };
        lib.trim();
        lib
    }

    pub fn text(&self) -> String {
        let shelf = Shelf {
            format: FORMAT,
            best: self.best.iter().map(Stored::of).collect(),
            recent: self.recent.iter().map(Stored::of).collect(),
        };
        serde_json::to_string(&shelf).unwrap_or_default()
    }

    pub fn best_of(&self, course: &str, mode: Mode) -> Option<&Recording> {
        self.best
            .iter()
            .find(|r| r.course == course && r.mode == mode)
    }

    fn next_serial(&self) -> u32 {
        self.best
            .iter()
            .chain(&self.recent)
            .map(|r| r.serial + 1)
            .max()
            .unwrap_or(1)
    }

    /// Keeps a finished run: the best for its course and mode if it beats
    /// the one kept (which then joins the recent ones), else a recent one.
    /// An unfinished run is not kept.
    pub fn add(&mut self, mut rec: Recording) {
        let Some(t) = rec.finish else {
            return;
        };
        rec.serial = self.next_serial();
        if rec.mode == Mode::Online {
            // never a best: kept among the recent, at most ONLINE_KEPT
            self.recent.push(rec);
            self.trim();
            return;
        }
        let old = self
            .best
            .iter()
            .position(|r| r.course == rec.course && r.mode == rec.mode);
        match old {
            Some(k) if self.best[k].finish.map(|b| b <= t).unwrap_or(false) => {
                self.recent.push(rec)
            }
            Some(k) => {
                let was = std::mem::replace(&mut self.best[k], rec);
                self.recent.push(was);
            }
            None => self.best.push(rec),
        }
        self.trim();
    }

    /// Oldest first, at most [`RECENT`], at most [`ONLINE_KEPT`] online.
    fn trim(&mut self) {
        self.recent.sort_by_key(|r| r.serial);
        let online = self
            .recent
            .iter()
            .filter(|r| r.mode == Mode::Online)
            .count();
        let mut drop = online.saturating_sub(ONLINE_KEPT);
        self.recent.retain(|r| {
            let go = drop > 0 && r.mode == Mode::Online;
            drop -= go as usize;
            !go
        });
        let extra = self.recent.len().saturating_sub(RECENT);
        self.recent.drain(..extra);
    }

    /// Puts the newer state of a recording already kept (the same serial)
    /// in its place: a race goes on being recorded after the finish.
    /// Returns whether it changed anything.
    pub fn refresh(&mut self, rec: &Recording) -> bool {
        let slot = self
            .best
            .iter_mut()
            .chain(self.recent.iter_mut())
            .find(|r| r.serial == rec.serial);
        match slot {
            Some(r) if rec.length() > r.length() => {
                let finish = r.finish;
                *r = rec.clone();
                r.finish = finish;
                true
            }
            _ => false,
        }
    }

    /// The serial the next kept recording gets.
    pub fn serial_of_next(&self) -> u32 {
        self.next_serial()
    }

    /// Every kept recording, newest first, each with whether it is a best.
    pub fn listing(&self) -> Vec<(&Recording, bool)> {
        let mut all: Vec<(&Recording, bool)> = self
            .best
            .iter()
            .map(|r| (r, true))
            .chain(self.recent.iter().map(|r| (r, false)))
            .collect();
        all.sort_by_key(|r| std::cmp::Reverse(r.0.serial));
        all
    }
}

// ---- watching ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speed {
    Half,
    One,
    Double,
}

impl Speed {
    pub const ALL: [Speed; 3] = [Speed::Half, Speed::One, Speed::Double];

    pub fn key(self) -> &'static str {
        match self {
            Speed::Half => "half",
            Speed::One => "one",
            Speed::Double => "double",
        }
    }
}

/// Where the replay camera is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// the player's own chase camera, as it was
    Chase,
    /// circling the ape
    Orbit,
    /// fixed cameras along the course, cutting as the ape passes
    Trackside,
}

impl View {
    pub const ALL: [View; 3] = [View::Chase, View::Orbit, View::Trackside];

    pub fn key(self) -> &'static str {
        match self {
            View::Chase => "chase",
            View::Orbit => "orbit",
            View::Trackside => "trackside",
        }
    }
}

/// A race state to scrub back to. The course never changes in a race, so
/// a save point holds a small stand-in instead of a copy (the jungle's
/// scenery alone is some 90 KB) and gets the real one back in [`put_back`].
#[derive(Debug, Clone)]
pub struct SavePoint {
    pub race: Race,
    pub brains: Vec<Brain>,
}

/// Makes `race` the save point's state, keeping its course.
pub fn put_back(save: SavePoint, race: &mut Race, brains: &mut Vec<Brain>) {
    let mut r = save.race;
    std::mem::swap(&mut r.course, &mut race.course);
    *race = r;
    *brains = save.brains;
}

/// A recording being watched. The race itself is the `Play`'s.
#[derive(Debug, Clone)]
pub struct Playback {
    pub rec: Recording,
    /// `saves[k]`: the race before tick `k * SAVE_EVERY`
    pub saves: Vec<SavePoint>,
    pub playing: bool,
    pub speed: Speed,
    pub view: View,
    /// the steerable orbit camera (TRIAGE T23)
    pub orbit: crate::camera::Orbit,
    /// a seek in progress: run forward to this tick
    pub seek: Option<u32>,
    /// half speed steps every other tick
    odd: bool,
    /// the save points' stand-in course
    blank: Course,
}

impl Playback {
    pub fn new(rec: Recording) -> Playback {
        Playback {
            rec,
            saves: Vec::new(),
            playing: true,
            speed: Speed::One,
            view: View::Chase,
            orbit: crate::camera::Orbit::default(),
            seek: None,
            odd: false,
            blank: Course::test_line(&[]),
        }
    }

    /// The ticks the recording covers.
    pub fn length(&self) -> u32 {
        self.rec.length()
    }

    /// Paused, at the end, or seeking: the time the sound should be quiet
    /// (TRIAGE T30).
    pub fn hushed(&self, tick: u32) -> bool {
        !self.playing || self.seek.is_some() || tick >= self.length()
    }

    /// How many race ticks to run this page tick (0: hold the frame).
    pub fn due(&mut self, tick: u32) -> usize {
        if let Some(to) = self.seek {
            if tick >= to {
                self.seek = None;
                return 0;
            }
            return ((to - tick) as usize).min(SEEK_PER_TICK);
        }
        if tick >= self.length() {
            // the end: hold there, paused
            self.playing = false;
        }
        if !self.playing {
            return 0;
        }
        match self.speed {
            Speed::Half => {
                self.odd = !self.odd;
                self.odd as usize
            }
            Speed::One => 1,
            Speed::Double => 2,
        }
    }

    /// Before a tick: keeps a save point when one falls due.
    pub fn keep(&mut self, race: &mut Race, brains: &[Brain]) {
        let t = race.tick as usize;
        if t.is_multiple_of(SAVE_EVERY) && t / SAVE_EVERY == self.saves.len() {
            let course = std::mem::replace(&mut race.course, self.blank.clone());
            let copy = race.clone();
            race.course = course;
            self.saves.push(SavePoint {
                race: copy,
                brains: brains.to_vec(),
            });
        }
    }

    /// Seeks to `permille` of the way through: returns the save point to
    /// go back to, when the target is behind `now` or far ahead.
    pub fn seek_to(&mut self, permille: u32, now: u32) -> Option<SavePoint> {
        let to = (self.length() as u64 * permille.min(1000) as u64 / 1000) as u32;
        self.seek = Some(to);
        let k = (to as usize / SAVE_EVERY).min(self.saves.len().saturating_sub(1));
        let from = (k * SAVE_EVERY) as u32;
        if to < now || (from > now && !self.saves.is_empty()) {
            self.saves.get(k).cloned()
        } else {
            None
        }
    }
}

/// The best Time Trial run, raced again beside the player (drawn tinted).
#[derive(Debug, Clone)]
pub struct Ghost {
    pub race: Race,
    pub controls: Vec<Controls>,
    pub prev: crate::geom::V3,
    /// its finish, for the HUD
    pub finish: Option<u32>,
}

impl Ghost {
    pub fn new(rec: &Recording, course: &Course) -> Option<Ghost> {
        if !rec.fits(course) || rec.entries.len() != 1 || rec.track.is_some() {
            return None;
        }
        let race = rec.race(course);
        let prev = race.racers[0].ape.pos;
        Some(Ghost {
            race,
            controls: rec.controls.clone(),
            prev,
            finish: rec.finish,
        })
    }

    pub fn step(&mut self) {
        self.prev = self.race.racers[0].ape.pos;
        let c = self
            .controls
            .get(self.race.tick as usize)
            .copied()
            .unwrap_or_default();
        self.race.step(&[c]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::deg;
    use crate::rules::Kind;

    fn entries(n: usize) -> Vec<Entry> {
        (0..n)
            .map(|i| Entry {
                name: format!("P{i}"),
                kind: Kind::ALL[i],
                driver: if i == 0 { Driver::Local } else { Driver::Ai },
            })
            .collect()
    }

    /// A made-up player: holds W and grab, sometimes lets go, turns.
    fn player(t: u32) -> Controls {
        Controls {
            fwd: 1,
            side: if t % 300 < 40 { 1 } else { 0 },
            grab: t % 170 > 30,
            jump: t.is_multiple_of(400),
            item: t % 900 == 450,
            yaw: deg((t as i32 / 7) % 40 - 20),
            pitch: -deg(12),
        }
    }

    #[test]
    fn controls_pack_into_text_and_back_exactly() {
        let mut cs: Vec<Controls> = (0..3000).map(player).collect();
        // a still stretch and a wrap past the extremes
        cs.extend(std::iter::repeat_n(Controls::default(), 500));
        cs.push(Controls {
            yaw: i32::MAX,
            pitch: i32::MIN,
            fwd: -1,
            side: -1,
            ..Controls::default()
        });
        let text = pack(&cs);
        assert_eq!(unpack(&text), Some(cs.clone()));
        // compact: well under 8 letters a tick for this player
        assert!(text.len() < cs.len() * 4, "{} for {}", text.len(), cs.len());
        assert_eq!(unpack("not packed!"), None);
        assert_eq!(unpack("AAA"), None);
    }

    #[test]
    fn a_recorded_race_plays_back_the_same() {
        let course = Course::jungle();
        let mut live = Race::new(course.clone(), &entries(3), false, 99);
        let mut rec = Recording::of(&live, 0, Mode::Race);
        let mut brains = vec![Brain::default(); 3];
        for _ in 0..2400 {
            let c = player(live.tick);
            rec.controls.push(c);
            drive(&mut live, &mut brains, 0, c);
        }
        // through storage, then raced again
        let mut lib = Library::default();
        rec.finish = Some(1);
        lib.add(rec);
        let back = Library::parse(&lib.text());
        let rec = back.best[0].clone();
        let mut again = rec.race(&course);
        let mut brains = vec![Brain::default(); 3];
        while (again.tick as usize) < rec.controls.len() {
            let c = rec.at(again.tick);
            drive(&mut again, &mut brains, 0, c);
        }
        for i in 0..3 {
            assert_eq!(again.racers[i].ape.pos, live.racers[i].ape.pos, "ape {i}");
            assert_eq!(again.racers[i].item, live.racers[i].item);
        }
        assert!(again.racers[1].progress > 0);
    }

    fn run(serial_finish: u32, mode: Mode) -> Recording {
        let race = Race::new(Course::jungle(), &entries(1), false, 1);
        let mut r = Recording::of(&race, 0, mode);
        r.controls = vec![Controls::default(); 5];
        r.finish = Some(serial_finish);
        r
    }

    #[test]
    fn the_library_keeps_the_best_per_mode_and_ten_others() {
        let mut lib = Library::default();
        lib.add(run(9000, Mode::Trial));
        lib.add(run(9500, Mode::Trial));
        lib.add(run(8800, Mode::Trial));
        lib.add(run(9900, Mode::Race));
        let best = lib.best_of("jungle", Mode::Trial).unwrap();
        assert_eq!(best.finish, Some(8800));
        assert_eq!(
            lib.best_of("jungle", Mode::Race).unwrap().finish,
            Some(9900)
        );
        // the beaten best joined the others
        let others: Vec<Option<u32>> = lib.recent.iter().map(|r| r.finish).collect();
        assert_eq!(others, vec![Some(9000), Some(9500)]);
        for k in 0..15 {
            lib.add(run(20_000 + k, Mode::Trial));
        }
        assert_eq!(lib.recent.len(), RECENT);
        assert_eq!(lib.recent[0].finish, Some(20_005));
        assert_eq!(
            lib.best_of("jungle", Mode::Trial).unwrap().finish,
            Some(8800)
        );
        // newest first, the bests marked
        let list = lib.listing();
        assert_eq!(list[0].0.finish, Some(20_014));
        assert_eq!(list.iter().filter(|(_, b)| *b).count(), 2);
        // unfinished runs are not kept; storage nonsense is an empty shelf
        let mut r = run(1, Mode::Race);
        r.finish = None;
        lib.add(r);
        assert_eq!(lib.recent.len(), RECENT);
        assert_eq!(Library::parse("{oops"), Library::default());
        assert_eq!(Library::parse(&lib.text()), lib);
    }

    #[test]
    fn a_recording_from_another_layout_does_not_fit() {
        let race = Race::new(Course::jungle(), &entries(1), false, 1);
        let rec = Recording::of(&race, 0, Mode::Trial);
        assert!(rec.fits(&Course::jungle()));
        assert!(!rec.fits(&Course::jungle_seeded(5)));
        assert!(Ghost::new(&rec, &Course::jungle_seeded(5)).is_none());
    }

    /// A race of `n` apes run live for `ticks`, recorded as a track (as an
    /// online race is); the live race too.
    fn tracked(n: usize, ticks: u32) -> (Race, Recording) {
        let mut live = Race::new(Course::jungle(), &entries(n), false, 7);
        let mut rec = Recording::online(&live, 0);
        let mut brains = vec![Brain::default(); n];
        for _ in 0..ticks {
            let c = player(live.tick);
            drive(&mut live, &mut brains, 0, c);
            rec.track.as_mut().unwrap().record(&live);
        }
        (live, rec)
    }

    #[test]
    fn a_track_packs_into_text_and_back_exactly() {
        let (_, mut rec) = tracked(4, 3000);
        let t = rec.track.as_mut().unwrap();
        assert_eq!(t.ticks(), 3000);
        t.summons.push((
            700,
            Summon {
                slot: 400,
                by: 2,
                serial: 0,
                anchor: crate::geom::v3(-5, 9_000_000, 77),
                length: 1_500_000,
                life: 900,
            },
        ));
        let text = t.pack(4);
        assert_eq!(Track::unpack(&text, 4).as_ref(), Some(&*t));
        // some 6 letters an ape a sample: a 75 s race of four is ~55 KB
        let per = text.len() / (t.samples.len() * 4);
        assert!(per <= 9, "{per} letters an ape a sample");
        assert_eq!(Track::unpack("AAA.", 4), None);
        assert_eq!(Track::unpack(&text[..text.len() - 3], 4), None);
        // through the library
        rec.finish = Some(1);
        let mut lib = Library::default();
        lib.add(rec.clone());
        let back = Library::parse(&lib.text());
        assert_eq!(back.recent[0].track, rec.track);
    }

    #[test]
    fn a_track_plays_back_where_the_apes_were() {
        let (live, rec) = tracked(3, 2400);
        let course = Course::jungle();
        let mut again = rec.race(&course);
        assert!(again.racers.iter().all(|r| r.driver == Driver::Remote));
        let mut brains = Vec::new();
        let mut clock_ran = false;
        while again.tick < rec.length() {
            rec.play(&mut again, &mut brains);
            if again.tick == 1200 {
                // a sample tick: exactly the place kept
                for i in 0..3 {
                    let s = rec.track.as_ref().unwrap().samples[300][i];
                    assert_eq!(again.racers[i].ape.pos, s.pos, "ape {i}");
                }
            }
            clock_ran |= again.clock > 0 && again.phase == Phase::Running;
        }
        assert!(clock_ran, "the race clock runs");
        for i in 0..3 {
            // to the track's precision (1/1024 m)
            let d = again.racers[i].ape.pos.dist(live.racers[i].ape.pos);
            assert!(d < 200, "ape {i} off by {d}");
            assert_eq!(again.racers[i].ape.mode, live.racers[i].ape.mode);
        }
        assert!(again.racers[1].progress > crate::fx::int(50));
        // between samples it blends, and moves
        let t = rec.track.as_ref().unwrap();
        let (a, va, ..) = t.at(1201, 1).unwrap();
        let (b, ..) = t.at(1203, 1).unwrap();
        assert_ne!(a, b);
        assert_ne!(va, V3::ZERO);
    }

    #[test]
    fn online_runs_are_kept_only_among_the_recent_and_few() {
        let mut lib = Library::default();
        lib.add(run(9000, Mode::Race));
        for k in 0..5 {
            let (_, mut rec) = tracked(2, 40);
            rec.finish = Some(100 + k);
            lib.add(rec);
        }
        assert!(
            lib.best_of("jungle", Mode::Online).is_none(),
            "never a best"
        );
        let online: Vec<Option<u32>> = lib
            .recent
            .iter()
            .filter(|r| r.mode == Mode::Online)
            .map(|r| r.finish)
            .collect();
        assert_eq!(online, vec![Some(102), Some(103), Some(104)]);
        assert_eq!(lib.best.len(), 1);
        // a kept one brought up to date keeps its serial and its finish
        let mut newer = lib.recent[2].clone();
        newer.finish = Some(1);
        newer.track.as_mut().unwrap().samples.push(Vec::new());
        assert!(lib.refresh(&newer));
        assert_eq!(lib.recent[2].length(), newer.length());
        assert_eq!(lib.recent[2].finish, Some(104));
        assert!(!lib.refresh(&newer), "nothing new");
    }
}
