//! A race (PLAN P11): the course, up to four apes, the countdown, item
//! crates and thrown things, places and finish times. One `step` is one
//! tick. Who controls each ape (the local player, the AI, a remote browser)
//! is the caller's business: `step` takes one `Controls` per ape, and a
//! remote ape is moved by `set_remote` instead.

use serde::{Deserialize, Serialize};

use crate::ape::{Ape, Controls, Happening, Mode};
use crate::course::{Course, Vine};
use crate::fx::{int, Dice, ONE};
use crate::geom::{v3, V3};
use crate::jaguar::{self, Eaten};
use crate::rules::{self, Kind};

/// The countdown before the start: 3, 2, 1, then go.
pub const COUNTDOWN: i32 = 3 * rules::TICKS_PER_SECOND;

/// The five items (PLAN P9; the Vine is TRIAGE T12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Item {
    Banana,
    Stick,
    Hook,
    Boost,
    /// summons a vine ahead of and above the user
    Vine,
}

impl Item {
    pub const ALL: [Item; 5] = [
        Item::Banana,
        Item::Stick,
        Item::Hook,
        Item::Boost,
        Item::Vine,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Item::Banana => "banana",
            Item::Stick => "stick",
            Item::Hook => "hook",
            Item::Boost => "boost",
            Item::Vine => "vine",
        }
    }
}

/// Who drives an ape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Driver {
    /// this browser's player
    Local,
    /// the AI, run by this browser
    Ai,
    /// another browser; moved by `set_remote`
    Remote,
}

/// Something that happened in the race this tick, for sounds, effects and
/// the online messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    /// ape `who` had a body happening (grab, release, land ...)
    Body {
        who: usize,
        what: Happening,
    },
    Countdown {
        left: i32,
    },
    Go,
    Pickup {
        who: usize,
        item: Item,
    },
    Used {
        who: usize,
        item: Item,
    },
    /// ape `who` was hit by `by` with `item`
    Hit {
        who: usize,
        by: usize,
        item: Item,
    },
    Finish {
        who: usize,
        place: usize,
    },
    /// ape `who`'s Vine item put a vine at `course.vines[vine]` (here or,
    /// online, in its owner's browser)
    Summoned {
        who: usize,
        vine: usize,
    },
    /// ape `who` had a reward from the course (group K)
    Reward {
        who: usize,
        what: Reward,
    },
    /// Floor is Lava (TRIAGE T37): ape `who` touched the ground and is out
    Eaten {
        who: usize,
    },
    /// the jaguar leaps on ape `who`, roaring
    Pounce {
        who: usize,
    },
}

/// One ape in the race.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Racer {
    pub name: String,
    pub driver: Driver,
    pub ape: Ape,
    pub item: Option<Item>,
    /// metres along the course (Q16)
    pub progress: i32,
    /// race ticks at the finish
    pub finish: Option<u32>,
    /// 1-based place: finish order, then progress
    pub place: usize,
    /// the last course reward it had, and the race tick it came (for the
    /// HUD's flash)
    pub reward: Option<(Reward, u32)>,
    /// for each of the course's bananas: whether this ape has taken it
    /// (TRIAGE T43: each ape collects its own)
    pub took: Vec<bool>,
    /// Floor is Lava (TRIAGE T37): it touched the ground and the jaguar
    /// came for it
    pub eaten: Option<Eaten>,
    /// its colours (TRIAGE T49), from its entry
    pub palette: u8,
}

impl Racer {
    /// Out of the race: finished, or eaten.
    pub fn out(&self) -> bool {
        self.finish.is_some() || self.eaten.is_some()
    }
}

/// A reward from the course itself (group K), for the HUD's flash and the
/// sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reward {
    /// a bounce mushroom's push (TRIAGE T48)
    Mushroom,
    /// a vine an opponent swung on first (TRIAGE T45)
    Catchup,
    /// a banana on the course (TRIAGE T43); no flash, the count shows it
    Banana,
}

impl Reward {
    pub fn key(self) -> &'static str {
        match self {
            Reward::Mushroom => "mushroom",
            Reward::Catchup => "catchup",
            Reward::Banana => "banana",
        }
    }
}

/// A thrown Banana, or a Hook on its way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shot {
    pub item: Item,
    pub by: usize,
    pub pos: V3,
    pub vel: V3,
    pub life: i32,
    /// a Hook flies at this ape
    pub target: Option<usize>,
}

/// A vine summoned by the Vine item (TRIAGE T12). It lives in
/// `course.vines[slot]` until `life` runs out and no ape holds it. The slot
/// depends only on who summoned it and how many it has summoned, so online
/// every browser puts it at the same index (the online layer sends it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summon {
    pub slot: usize,
    pub by: usize,
    /// the summoner's count of summons, from 0
    pub serial: u32,
    pub anchor: V3,
    pub length: i32,
    /// ticks left
    pub life: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// ticks left before the start
    Countdown(i32),
    Running,
    /// every ape that is going to has finished
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Race {
    pub course: Course,
    pub racers: Vec<Racer>,
    pub phase: Phase,
    /// ticks since the race was set up
    pub tick: u32,
    /// ticks since Go
    pub clock: u32,
    /// for each crate: ticks until it is back (0: there)
    pub crates: Vec<i32>,
    /// for each crate: the vine it hangs on (TRIAGE T9)
    pub crate_vines: Vec<usize>,
    /// the items crates may give (TRIAGE T42; a Time Trial: only the
    /// Speed Boost, T13)
    pub items: crate::items::ItemSet,
    /// summoned vines still standing
    pub summons: Vec<Summon>,
    /// the course's own vines: summoned ones come after
    pub base_vines: usize,
    /// for each ape: how many vines it has summoned
    pub summon_count: Vec<u32>,
    pub shots: Vec<Shot>,
    pub events: Vec<Event>,
    pub equal_stats: bool,
    pub dice: Dice,
    /// finishers so far, in order
    pub order: Vec<usize>,
    /// how well the AI apes race (TRIAGE T14)
    pub ai_level: crate::ai::Difficulty,
    /// for each vine: which apes have swung on it, one bit per ape (TRIAGE
    /// T45). Apes from other browsers count from their reported state.
    pub used: Vec<u32>,
    /// Floor is Lava (TRIAGE T37): touching the ground is the end
    pub lava: bool,
}

/// One entrant: a name, a kind of ape and who drives it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub kind: Kind,
    pub driver: Driver,
    /// its colours (TRIAGE T49): 0 is the classic, as recordings made
    /// before palettes read
    #[serde(default, skip_serializing_if = "is_classic")]
    pub palette: u8,
}

fn is_classic(p: &u8) -> bool {
    *p == 0
}

/// The palette for an AI ape of `kind` joining apes wearing `taken`
/// (kind and palette, TRIAGE T49): the first no earlier ape of that kind
/// wears, so two apes of a kind are told apart; the classic when every
/// one is taken.
pub fn palette_apart(taken: impl Iterator<Item = (Kind, u8)> + Clone, kind: Kind) -> u8 {
    (0..crate::models::PALETTES)
        .find(|p| !taken.clone().any(|t| t == (kind, *p)))
        .unwrap_or(0)
}

/// Each entry's kind and palette, for `palette_apart`.
pub fn worn(entries: &[Entry]) -> impl Iterator<Item = (Kind, u8)> + Clone + '_ {
    entries.iter().map(|e| (e.kind, e.palette))
}

impl Race {
    pub fn new(course: Course, entries: &[Entry], equal_stats: bool, seed: u64) -> Race {
        let racers = entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let start = course.starts[i % 4];
                Racer {
                    name: e.name.clone(),
                    driver: e.driver,
                    ape: Ape::new(e.kind, equal_stats, start, course.start_yaw),
                    item: None,
                    progress: 0,
                    finish: None,
                    place: i + 1,
                    reward: None,
                    took: vec![false; course.bananas.len()],
                    eaten: None,
                    palette: e.palette,
                }
            })
            .collect();
        let crate_vines = crate::items::crate_vines(&course);
        let crates = vec![0; crate_vines.len()];
        let n = entries.len();
        Race {
            base_vines: course.vines.len(),
            course,
            racers,
            phase: Phase::Countdown(COUNTDOWN),
            tick: 0,
            clock: 0,
            crates,
            crate_vines,
            items: crate::items::ItemSet::ALL,
            summons: Vec::new(),
            summon_count: vec![0; n],
            used: Vec::new(),
            shots: Vec::new(),
            events: Vec::new(),
            equal_stats,
            dice: Dice(seed),
            order: Vec::new(),
            ai_level: crate::ai::Difficulty::default(),
            lava: false,
        }
    }

    pub fn running(&self) -> bool {
        self.phase == Phase::Running
    }

    /// Race ticks since ape `i` finished (TRIAGE T53, the dance), or None
    /// before it has. The finish is on the race clock, which starts at Go
    /// (after the fixed countdown) and stops once the race is done, so
    /// this counts on the race's own tick instead. It comes from shared
    /// state alone: every browser, ghost and replay sees the same.
    pub fn since_finish(&self, i: usize) -> Option<u32> {
        let finish = self.racers.get(i)?.finish?;
        Some(self.tick.saturating_sub(finish + COUNTDOWN as u32))
    }

    /// One tick. `controls[i]` drives ape `i` unless it is Remote.
    pub fn step(&mut self, controls: &[Controls]) {
        self.events.clear();
        self.tick += 1;
        match self.phase {
            Phase::Countdown(left) => {
                let left = left - 1;
                if left % rules::TICKS_PER_SECOND == 0 && left > 0 {
                    self.events.push(Event::Countdown {
                        left: left / rules::TICKS_PER_SECOND,
                    });
                }
                if left <= 0 {
                    self.phase = Phase::Running;
                    self.events.push(Event::Go);
                } else {
                    self.phase = Phase::Countdown(left);
                }
                // before Go the apes stand still (they may turn the camera)
                return;
            }
            Phase::Done => {}
            Phase::Running => self.clock += 1,
        }
        let tick = self.tick;
        for i in 0..self.racers.len() {
            // an eaten ape stays where it touched down
            if self.racers[i].driver == Driver::Remote || self.racers[i].eaten.is_some() {
                continue;
            }
            let c = controls.get(i).copied().unwrap_or_default();
            // a finished ape coasts with no one at the keys
            let c = if self.racers[i].finish.is_some() {
                Controls {
                    grab: c.grab && self.racers[i].ape.swinging().is_some(),
                    ..Controls::default()
                }
            } else {
                c
            };
            self.racers[i].ape.step(&c, &self.course, tick);
            for what in self.racers[i].ape.happened.clone() {
                self.events.push(Event::Body { who: i, what });
                match what {
                    Happening::Bounce => self.reward(i, Reward::Mushroom),
                    Happening::Grab => self.catch_up(i),
                    _ => {}
                }
            }
            if self.floor_is_lava(i) {
                self.eat(i);
                continue;
            }
            if c.item {
                crate::items::use_item(self, i, &c);
            }
            self.collect_bananas(i);
        }
        self.jaguars();
        crate::items::step(self);
        self.mark_used();
        self.update_places();
    }

    /// Floor is Lava (TRIAGE T37): ape `i` (still racing) is on the ground.
    /// The start platform and the mushroom caps are not the ground.
    fn floor_is_lava(&self, i: usize) -> bool {
        let r = &self.racers[i];
        self.lava
            && self.phase == Phase::Running
            && !r.out()
            && r.ape.mode == Mode::Grounded
            && self.course.on_floor(r.ape.pos)
    }

    /// Ape `i` is eaten where it is: it stops, facing the jaguar coming.
    pub fn eat(&mut self, i: usize) {
        let tick = self.tick;
        let Some(r) = self.racers.get_mut(i) else {
            return;
        };
        if r.out() {
            return;
        }
        let e = Eaten::new(&self.course, r.ape.pos, r.ape.hint, tick);
        r.ape.vel = V3::ZERO;
        r.ape.stun = 0;
        r.ape.yaw = crate::geom::yaw_of(e.from.sub(e.at));
        r.item = None;
        r.eaten = Some(e);
        self.events.push(Event::Eaten { who: i });
    }

    /// Records ape `i` eaten at `at` in another browser (online, or a
    /// replay's track).
    pub fn remote_eaten(&mut self, i: usize, at: V3) {
        if let Some(r) = self.racers.get_mut(i) {
            if !r.out() {
                r.ape.pos = at;
                r.ape.hint = self.course.nearest(at, r.ape.hint);
                self.eat(i);
            }
        }
    }

    /// The jaguars' moments that make a sound.
    fn jaguars(&mut self) {
        for (who, r) in self.racers.iter().enumerate() {
            if let Some(e) = r.eaten {
                if e.ago(self.tick) == jaguar::RUN {
                    self.events.push(Event::Pounce { who });
                }
            }
        }
    }

    /// Moves a remote ape to where its browser says it is (an eaten one
    /// stays where it was eaten).
    pub fn set_remote(&mut self, i: usize, pos: V3, vel: V3, mode: Mode, yaw: i32) {
        if self.racers.get(i).is_some_and(|r| r.eaten.is_some()) {
            return;
        }
        // a vine this browser has not heard of yet: drawn as flying
        let mode = match mode {
            Mode::Swinging { vine, .. } if vine >= self.course.vines.len() => Mode::Flying,
            m => m,
        };
        if let Some(r) = self.racers.get_mut(i) {
            r.ape.pos = pos;
            r.ape.vel = vel;
            r.ape.mode = mode;
            r.ape.yaw = yaw;
            r.ape.hint = self.course.nearest(pos, r.ape.hint);
        }
    }

    fn update_places(&mut self) {
        let length = self.course.length;
        for i in 0..self.racers.len() {
            let r = &mut self.racers[i];
            let (s, _) = self.course.progress(r.ape.pos, r.ape.hint);
            r.progress = s;
            if r.finish.is_none() && self.phase == Phase::Running && s >= length {
                r.finish = Some(self.clock);
                self.order.push(i);
                let place = self.order.len();
                self.events.push(Event::Finish { who: i, place });
            }
        }
        // places: finishers in order, then the rest by progress
        let mut rest: Vec<usize> = (0..self.racers.len())
            .filter(|i| !self.order.contains(i))
            .collect();
        rest.sort_by(|a, b| self.racers[*b].progress.cmp(&self.racers[*a].progress));
        for (k, i) in self.order.iter().chain(rest.iter()).enumerate() {
            self.racers[*i].place = k + 1;
        }
        let racing = self
            .racers
            .iter()
            .filter(|r| r.driver != Driver::Remote)
            .all(|r| r.out());
        if self.phase == Phase::Running && racing && !self.racers.is_empty() {
            self.phase = Phase::Done;
        }
    }

    /// Ape `who` has a course reward: its flash and its event.
    fn reward(&mut self, who: usize, what: Reward) {
        self.racers[who].reward = Some((what, self.tick));
        self.events.push(Event::Reward { who, what });
    }

    /// Ape `who` takes the bananas its body passes through (TRIAGE T43).
    fn collect_bananas(&mut self, who: usize) {
        let r = &mut self.racers[who];
        if r.out() {
            return;
        }
        let at = r.ape.middle();
        let near = rules::BANANA_PICKUP + rules::BODY_RADIUS;
        let mut got = 0;
        for (k, b) in self.course.bananas.iter().enumerate() {
            if (b.x - at.x).abs() > near || (b.z - at.z).abs() > near || r.took[k] {
                continue;
            }
            if b.dist(at) <= near {
                got += 1;
                r.took[k] = true;
            }
        }
        if got > 0 {
            r.ape.bananas = (r.ape.bananas + got).min(rules::BANANAS_FULL);
            self.events.push(Event::Reward {
                who,
                what: Reward::Banana,
            });
        }
    }

    /// Which apes have swung on vine `v`, one bit each.
    pub fn used_by(&self, v: usize) -> u32 {
        self.used.get(v).copied().unwrap_or(0)
    }

    /// Whether vine `v` is a catch-up vine for ape `who` (TRIAGE T45): an
    /// opponent has swung on it and `who` has not. Each ape sees its own.
    pub fn catch_up_vine(&self, who: usize, v: usize) -> bool {
        let me = 1u32 << (who % 32);
        let used = self.used_by(v);
        used & !me != 0 && used & me == 0
    }

    /// Ape `who` just grabbed: a catch-up vine gives it a little speed when
    /// it lets go, once (its own grab then marks the vine as used by it).
    /// On the grab itself the speed would set a fast ape whirling round
    /// the anchor.
    fn catch_up(&mut self, who: usize) {
        let Some(v) = self.racers[who].ape.swinging() else {
            return;
        };
        if !self.catch_up_vine(who, v) {
            return;
        }
        self.racers[who].ape.release_bonus = rules::CATCHUP_GAIN;
        self.mark(who, v);
        self.reward(who, Reward::Catchup);
    }

    fn mark(&mut self, who: usize, v: usize) {
        if self.used.len() <= v {
            self.used.resize(v + 1, 0);
        }
        self.used[v] |= 1u32 << (who % 32);
    }

    /// Every ape on a vine marks it as used by it.
    fn mark_used(&mut self) {
        for i in 0..self.racers.len() {
            if let Some(v) = self.racers[i].ape.swinging() {
                if v < self.course.vines.len() {
                    self.mark(i, v);
                }
            }
        }
    }

    /// A summoned vine's slot starts unused.
    fn forget_used(&mut self, slot: usize) {
        if let Some(u) = self.used.get_mut(slot) {
            *u = 0;
        }
    }

    /// The slot ape `by`'s summon number `serial` goes in.
    pub fn summon_slot(&self, by: usize, serial: u32) -> usize {
        self.base_vines + by * rules::SUMMON_SLOTS + serial as usize % rules::SUMMON_SLOTS
    }

    /// Puts a summoned vine in its slot (growing the vine list with buried
    /// placeholders as needed) and raises `Event::Summoned`. The same summon
    /// twice is ignored. An ape this browser moves that holds the vine the
    /// slot had is let go first. Online, the layer calls this for summons
    /// from other browsers.
    pub fn place_vine(&mut self, s: Summon) {
        let slots = self.base_vines + self.racers.len() * rules::SUMMON_SLOTS;
        if s.slot < self.base_vines || s.slot >= slots || s.by >= self.racers.len() {
            return;
        }
        let seen = self
            .summons
            .iter()
            .any(|o| o.slot == s.slot && o.serial == s.serial && o.by == s.by);
        if seen {
            return;
        }
        while self.course.vines.len() <= s.slot {
            self.course.vines.push(buried());
        }
        self.let_go_of(s.slot);
        self.summons.retain(|o| o.slot != s.slot);
        self.forget_used(s.slot);
        let hint = self.course.nearest_anywhere(s.anchor);
        let (at, _) = self.course.progress(s.anchor, hint);
        self.course.vines[s.slot] = Vine {
            anchor: s.anchor,
            length: s.length,
            slide: None,
            at,
        };
        if let Some(c) = self.summon_count.get_mut(s.by) {
            *c = (*c).max(s.serial + 1);
        }
        self.summons.push(s);
        self.events.push(Event::Summoned {
            who: s.by,
            vine: s.slot,
        });
    }

    /// Takes a summoned vine away: its slot is buried (kept, so no other
    /// vine's index moves).
    pub fn bury_vine(&mut self, slot: usize) {
        if slot >= self.base_vines && slot < self.course.vines.len() {
            self.let_go_of(slot);
            self.course.vines[slot] = buried();
            self.forget_used(slot);
            self.summons.retain(|o| o.slot != slot);
        }
    }

    /// Apes this browser moves let go of vine `slot`.
    fn let_go_of(&mut self, slot: usize) {
        for r in &mut self.racers {
            if r.driver != Driver::Remote && r.ape.swinging() == Some(slot) {
                r.ape.release(slot);
            }
        }
    }

    /// Records a finish reported from another browser.
    pub fn remote_finish(&mut self, i: usize, clock: u32) {
        if i < self.racers.len() && !self.racers[i].out() {
            self.racers[i].finish = Some(clock);
            self.order.push(i);
            let place = self.order.len();
            self.events.push(Event::Finish { who: i, place });
        }
    }

    /// Seconds as Q16, for the HUD.
    pub fn seconds(ticks: u32) -> i32 {
        (ticks as i64 * ONE as i64 / rules::TICKS_PER_SECOND as i64) as i32
    }
}

/// An empty vine slot: far underground, out of everyone's reach, and
/// behind the start for the AI.
fn buried() -> Vine {
    Vine {
        anchor: v3(0, -int(2000), 0),
        length: ONE,
        slide: None,
        at: -int(10_000),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(n: usize) -> Vec<Entry> {
        (0..n)
            .map(|i| Entry {
                name: format!("P{i}"),
                kind: Kind::ALL[i],
                driver: Driver::Local,
                palette: 0,
            })
            .collect()
    }

    #[test]
    fn the_countdown_holds_everyone_then_goes() {
        let c = Course::jungle();
        let mut r = Race::new(c, &entries(2), false, 1);
        let start = r.racers[0].ape.pos;
        let push = Controls {
            fwd: 1,
            ..Controls::default()
        };
        let mut counted = Vec::new();
        let mut went = false;
        for _ in 0..COUNTDOWN {
            r.step(&[push, push]);
            for e in &r.events {
                if let Event::Countdown { left } = e {
                    counted.push(*left);
                }
                went |= *e == Event::Go;
            }
        }
        assert_eq!(counted, vec![2, 1]);
        assert!(went);
        assert_eq!(r.racers[0].ape.pos, start);
        assert!(r.running());
        for _ in 0..60 {
            r.step(&[push, push]);
        }
        assert_ne!(r.racers[0].ape.pos, start);
    }

    #[test]
    fn finishers_are_placed_in_order() {
        let c = Course::jungle();
        let mut r = Race::new(c, &entries(3), false, 1);
        r.phase = Phase::Running;
        let end = r.course.point(r.course.length);
        let near_end = r.course.point(r.course.length - int(1));
        let samples = r.course.samples();
        // ape 1 crosses first, then ape 0; ape 2 is still out on the course
        r.racers[1].ape.hint = samples - 1;
        r.racers[1].ape.pos =
            v3(end.x, int(20), end.z).add(r.course.heading_at(r.course.length).scale(int(1)));
        r.racers[1].ape.mode = Mode::Flying;
        r.step(&[Controls::default(); 3]);
        assert_eq!(r.racers[1].finish, Some(r.clock));
        r.racers[0].ape.hint = samples - 1;
        r.racers[0].ape.pos = v3(near_end.x, int(20), near_end.z)
            .add(r.course.heading_at(r.course.length).scale(int(3)));
        r.racers[0].ape.mode = Mode::Flying;
        r.step(&[Controls::default(); 3]);
        assert_eq!(r.order, vec![1, 0]);
        assert_eq!(r.racers[1].place, 1);
        assert_eq!(r.racers[0].place, 2);
        assert_eq!(r.racers[2].place, 3);
        assert_eq!(r.phase, Phase::Running);
    }

    /// Two apes on a one-vine course, running; ape 1 on the vine first if
    /// `opponent_first`, then ape 0 flies into it holding grab, and lets go
    /// the tick after. Returns the race after the grab, and ape 0's speed
    /// just after the grab and its level speed just after letting go.
    fn catch_up_race(opponent_first: bool) -> (Race, i32, i32) {
        catch_up_race_at(opponent_first, int(10))
    }

    fn catch_up_race_at(opponent_first: bool, speed: i32) -> (Race, i32, i32) {
        let c = Course::test_line(&[(int(50), int(30), int(12))]);
        let mut r = Race::new(c, &entries(2), false, 1);
        r.phase = Phase::Running;
        let a = r.course.vines[0].anchor;
        let hold = Controls {
            grab: true,
            ..Controls::default()
        };
        if opponent_first {
            r.racers[1].ape.pos = a.sub(v3(0, int(10), 0));
            r.racers[1].ape.mode = Mode::Swinging {
                vine: 0,
                rope: int(10),
            };
            r.step(&[Controls::default(), hold]);
        }
        r.racers[0].ape.pos = a.add(v3(0, -int(10), -int(3)));
        r.racers[0].ape.vel = v3(0, 0, speed);
        r.racers[0].ape.mode = Mode::Flying;
        r.step(&[hold, hold]);
        assert_eq!(r.racers[0].ape.swinging(), Some(0));
        let grabbed = r.racers[0].ape.speed();
        let mut gone = r.clone();
        gone.step(&[Controls::default(), hold]);
        assert_eq!(gone.racers[0].ape.swinging(), None);
        (r, grabbed, gone.racers[0].ape.vel.len_xz())
    }

    #[test]
    fn a_vine_an_opponent_swung_on_gives_a_catch_up_boost_once() {
        // TRIAGE T45
        let (plain, plain_grab, plain_after) = catch_up_race(false);
        let (r, grab, after) = catch_up_race(true);
        // the grab is the same; letting go adds the boost (forward, level)
        assert_eq!(grab, plain_grab);
        assert!(
            after >= plain_after + rules::CATCHUP_GAIN - ONE / 10,
            "{} vs {} cm/s",
            after / 655,
            plain_after / 655
        );
        assert!(r.events.contains(&Event::Reward {
            who: 0,
            what: Reward::Catchup
        }));
        assert!(!plain
            .events
            .iter()
            .any(|e| matches!(e, Event::Reward { .. })));
        // never past Catch-up Top: caught at 45 m/s it gains nothing
        let (_, _, fast) = catch_up_race_at(true, int(45));
        let (_, _, fast_plain) = catch_up_race_at(false, int(45));
        assert!(
            fast <= fast_plain,
            "{} vs {} cm/s",
            fast / 655,
            fast_plain / 655
        );
        // each ape sees its own: ape 1's vine was never a catch-up vine for
        // ape 1, and once ape 0 has had its boost it is not one for ape 0
        assert!(!r.catch_up_vine(1, 0));
        assert!(!r.catch_up_vine(0, 0));
        // but ape 0 swinging marked it for ape 1 in the plain race
        assert!(plain.catch_up_vine(1, 0));
    }

    #[test]
    fn an_ape_from_another_browser_marks_the_vines_it_swings_on() {
        // online (TRIAGE T45): no message of its own; the reported state
        // is enough
        let c = Course::test_line(&[(int(50), int(30), int(12))]);
        let mut es = entries(2);
        es[1].driver = Driver::Remote;
        let mut r = Race::new(c, &es, false, 1);
        r.phase = Phase::Running;
        let a = r.course.vines[0].anchor;
        let swing = Mode::Swinging {
            vine: 0,
            rope: int(10),
        };
        r.set_remote(1, a.sub(v3(0, int(10), 0)), V3::ZERO, swing, 0);
        assert!(!r.catch_up_vine(0, 0));
        r.step(&[Controls::default(); 2]);
        assert!(r.catch_up_vine(0, 0));
    }

    #[test]
    fn each_ape_collects_its_own_bananas_once() {
        // TRIAGE T43
        let mut c = Course::test_line(&[]);
        let p = c.point(int(100));
        c.bananas = (0..3).map(|k| v3(p.x, int(10), p.z + int(k * 2))).collect();
        let mut r = Race::new(c, &entries(2), false, 1);
        r.phase = Phase::Running;
        let start = r.course.bananas[0].add(v3(0, rules::BODY_DROP / 2, -int(3)));
        r.racers[0].ape.pos = start;
        r.racers[0].ape.vel = v3(0, 0, int(20));
        r.racers[0].ape.mode = Mode::Flying;
        let mut rewards = 0;
        for _ in 0..60 {
            r.step(&[Controls::default(); 2]);
            rewards += r
                .events
                .iter()
                .filter(|e| {
                    matches!(
                        e,
                        Event::Reward {
                            who: 0,
                            what: Reward::Banana
                        }
                    )
                })
                .count();
        }
        assert_eq!(r.racers[0].ape.bananas, 3);
        assert!((1..=3).contains(&rewards), "{rewards}");
        assert!(r.racers[0].took.iter().all(|t| *t));
        // the other ape still has all three to take
        assert!(r.racers[1].took.iter().all(|t| !*t));
        assert_eq!(r.racers[1].ape.bananas, 0);
        // flying through again takes nothing more
        r.racers[0].ape.pos = start;
        r.racers[0].ape.vel = v3(0, 0, int(20));
        for _ in 0..60 {
            r.step(&[Controls::default(); 2]);
        }
        assert_eq!(r.racers[0].ape.bananas, 3);
    }

    #[test]
    fn a_summoned_vine_starts_unused() {
        let c = Course::test_line(&[(int(50), int(30), int(12))]);
        let mut r = Race::new(c, &entries(2), false, 1);
        let slot = r.summon_slot(1, 0);
        let s = Summon {
            slot,
            by: 1,
            serial: 0,
            anchor: v3(0, int(30), int(80)),
            length: int(12),
            life: 100,
        };
        r.place_vine(s);
        r.mark(1, slot);
        assert!(r.catch_up_vine(0, slot));
        r.place_vine(Summon { serial: 3, ..s });
        assert!(!r.catch_up_vine(0, slot));
    }

    /// A running lava race on the jungle with ape 0 dropped from `height`
    /// over the floor at `s` metres along, stepped until it lands.
    fn dropped(lava: bool, s: i32, height: i32) -> Race {
        let mut r = Race::new(Course::jungle(), &entries(2), false, 1);
        r.lava = lava;
        r.phase = Phase::Running;
        let p = r.course.point(int(s));
        let floor = crate::course::ground(p.x, p.z);
        r.racers[0].ape.pos = v3(p.x, floor + rules::BODY_DROP + height, p.z);
        r.racers[0].ape.hint = r.course.nearest_anywhere(r.racers[0].ape.pos);
        r.racers[0].ape.mode = Mode::Flying;
        r.racers[0].ape.vel = v3(0, 0, int(8));
        r
    }

    #[test]
    fn floor_is_lava_eats_an_ape_that_touches_the_ground() {
        // TRIAGE T37
        let mut r = dropped(true, 300, int(3));
        let mut eaten_at = None;
        for _ in 0..240 {
            r.step(&[Controls::default(); 2]);
            if r.events.contains(&Event::Eaten { who: 0 }) {
                eaten_at = Some(r.tick);
            }
        }
        let at = eaten_at.expect("eaten on landing");
        let e = r.racers[0].eaten.expect("eaten");
        assert_eq!(e.tick, at);
        assert!(r.racers[0].out() && r.racers[0].finish.is_none());
        // it stays where it touched down, and only ape 0 was eaten
        assert_eq!(r.racers[0].ape.pos, e.at);
        assert_eq!(r.racers[0].ape.vel, V3::ZERO);
        assert!(r.racers[1].eaten.is_none(), "on the start platform");
        // the jaguar leaps, roaring, at the end of its run
        let mut r2 = dropped(true, 300, int(3));
        let mut pounce = None;
        for _ in 0..(jaguar::RUN + 300) {
            r2.step(&[Controls::default(); 2]);
            if r2.events.contains(&Event::Pounce { who: 0 }) {
                pounce = Some(r2.tick);
            }
        }
        assert_eq!(pounce, Some(at + jaguar::RUN));
        // without the mode, the same landing is only a landing
        let mut off = dropped(false, 300, int(3));
        for _ in 0..240 {
            off.step(&[Controls::default(); 2]);
        }
        assert!(off.racers[0].eaten.is_none());
        assert_eq!(off.racers[0].ape.mode, Mode::Grounded);
    }

    #[test]
    fn the_start_platform_and_the_countdown_are_safe() {
        let mut r = Race::new(Course::jungle(), &entries(2), false, 1);
        r.lava = true;
        // standing on the platform through the countdown and after Go
        for _ in 0..COUNTDOWN as u32 + 120 {
            r.step(&[Controls::default(); 2]);
        }
        assert!(r.running());
        assert!(r.racers.iter().all(|x| x.eaten.is_none()));
        assert!(r.racers.iter().all(|x| x.ape.mode == Mode::Grounded));
    }

    #[test]
    fn a_mushroom_bounce_is_not_the_ground() {
        let mut r = dropped(true, 300, int(3));
        let m = r.course.mushrooms[0];
        let a = &mut r.racers[0].ape;
        a.pos = v3(m.top.x, m.top.y + rules::BODY_DROP + int(2), m.top.z);
        a.hint = r.course.nearest_anywhere(a.pos);
        a.vel = v3(0, -int(5), 0);
        let mut bounced = false;
        for _ in 0..120 {
            if bounced {
                break;
            }
            r.step(&[Controls::default(); 2]);
            bounced |= r.events.contains(&Event::Body {
                who: 0,
                what: Happening::Bounce,
            });
        }
        assert!(bounced);
        assert!(r.racers[0].eaten.is_none());
    }

    #[test]
    fn eaten_apes_place_after_finishers_by_progress_and_end_the_race() {
        let mut r = Race::new(Course::jungle(), &entries(3), false, 1);
        r.lava = true;
        r.phase = Phase::Running;
        let place = |r: &mut Race, i: usize, s: i32| {
            let p = r.course.point(int(s));
            let a = &mut r.racers[i].ape;
            a.pos = v3(p.x, crate::course::ground(p.x, p.z) + rules::BODY_DROP, p.z);
            a.hint = r.course.nearest_anywhere(a.pos);
            a.mode = Mode::Grounded;
        };
        // ape 2 eaten far along, ape 1 eaten early, ape 0 finishes last
        place(&mut r, 2, 900);
        place(&mut r, 1, 200);
        r.racers[0].ape.pos = v3(0, int(40), 0);
        r.racers[0].ape.mode = Mode::Flying;
        r.step(&[Controls::default(); 3]);
        assert!(r.racers[1].eaten.is_some() && r.racers[2].eaten.is_some());
        assert_eq!(r.phase, Phase::Running, "ape 0 still racing");
        r.remote_finish(0, r.clock);
        r.step(&[Controls::default(); 3]);
        assert_eq!(r.phase, Phase::Done);
        assert_eq!(r.racers[0].place, 1);
        assert_eq!(r.racers[2].place, 2, "the furthest eaten next");
        assert_eq!(r.racers[1].place, 3);
    }

    #[test]
    fn places_follow_progress_during_the_race() {
        let c = Course::jungle();
        let mut r = Race::new(c, &entries(2), false, 1);
        r.phase = Phase::Running;
        let p = r.course.point(int(300));
        r.racers[1].ape.pos = v3(p.x, int(20), p.z);
        r.racers[1].ape.hint = r.course.nearest_anywhere(r.racers[1].ape.pos);
        r.racers[1].ape.mode = Mode::Flying;
        r.step(&[Controls::default(); 2]);
        assert_eq!(r.racers[1].place, 1);
        assert_eq!(r.racers[0].place, 2);
    }
}
