//! Replays and ghosts (TRIAGE T15).
//!
//! A race is deterministic: the same course, entries, seed and the local
//! player's `Controls` each tick give the same race, AI apes and items
//! included (`ai::think` reads only the race and its brain). So a
//! [`Recording`] is the race's setup plus one `Controls` per tick, and
//! watching it is building the race again and stepping it. Online races are
//! not recorded: remote apes are moved by messages, not by anything this
//! browser could re-run.
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
use crate::ape::Controls;
use crate::course::Course;
use crate::items::ItemSet;
use crate::race::{Driver, Entry, Item, Race};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Race,
    Trial,
}

impl Mode {
    pub fn key(self) -> &'static str {
        match self {
            Mode::Race => "race",
            Mode::Trial => "trial",
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
        }
    }

    /// Whether it was made on this course as it is now.
    pub fn fits(&self, course: &Course) -> bool {
        self.course == course.name && self.layout == fingerprint(course)
    }

    /// The race at its first tick, as it was set up.
    pub fn race(&self, course: &Course) -> Race {
        let mut r = Race::new(course.clone(), &self.entries, self.equal_stats, self.seed);
        r.items = self.items;
        r.ai_level = self.ai_level;
        r
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

/// The controls back from [`pack`]; `None` when the text is not one.
pub fn unpack(text: &str) -> Option<Vec<Controls>> {
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
    if shift != 0 || nums.len() % 4 != 0 {
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
        }
    }

    fn recording(self) -> Option<Recording> {
        if self.local >= self.entries.len() || self.entries.len() > 4 {
            return None;
        }
        Some(Recording {
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
        lib.recent.sort_by_key(|r| r.serial);
        let extra = lib.recent.len().saturating_sub(RECENT);
        lib.recent.drain(..extra);
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
        self.recent.sort_by_key(|r| r.serial);
        let extra = self.recent.len().saturating_sub(RECENT);
        self.recent.drain(..extra);
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
            seek: None,
            odd: false,
            blank: Course::test_line(&[]),
        }
    }

    /// The ticks the recording covers.
    pub fn length(&self) -> u32 {
        self.rec.controls.len() as u32
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
        if !rec.fits(course) || rec.entries.len() != 1 {
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
}
