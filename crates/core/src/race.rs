//! A race (PLAN P11): the course, up to four apes, the countdown, item
//! crates and thrown things, places and finish times. One `step` is one
//! tick. Who controls each ape (the local player, the AI, a remote browser)
//! is the caller's business: `step` takes one `Controls` per ape, and a
//! remote ape is moved by `set_remote` instead.

use serde::{Deserialize, Serialize};

use crate::ape::{Ape, Controls, Happening, Mode};
use crate::course::Course;
use crate::fx::{Dice, ONE};
use crate::geom::V3;
use crate::rules::{self, Kind};

/// The countdown before the start: 3, 2, 1, then go.
pub const COUNTDOWN: i32 = 3 * rules::TICKS_PER_SECOND;

/// The four items (PLAN P9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Item {
    Banana,
    Stick,
    Hook,
    Boost,
}

impl Item {
    pub const ALL: [Item; 4] = [Item::Banana, Item::Stick, Item::Hook, Item::Boost];

    pub fn key(self) -> &'static str {
        match self {
            Item::Banana => "banana",
            Item::Stick => "stick",
            Item::Hook => "hook",
            Item::Boost => "boost",
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
    pub shots: Vec<Shot>,
    pub events: Vec<Event>,
    pub equal_stats: bool,
    pub dice: Dice,
    /// finishers so far, in order
    pub order: Vec<usize>,
}

/// One entrant: a name, a kind of ape and who drives it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub kind: Kind,
    pub driver: Driver,
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
                }
            })
            .collect();
        let crates = vec![0; course.crates.len()];
        Race {
            course,
            racers,
            phase: Phase::Countdown(COUNTDOWN),
            tick: 0,
            clock: 0,
            crates,
            shots: Vec::new(),
            events: Vec::new(),
            equal_stats,
            dice: Dice(seed),
            order: Vec::new(),
        }
    }

    pub fn running(&self) -> bool {
        self.phase == Phase::Running
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
            if self.racers[i].driver == Driver::Remote {
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
            }
            if c.item {
                crate::items::use_item(self, i, &c);
            }
        }
        crate::items::step(self);
        self.update_places();
    }

    /// Moves a remote ape to where its browser says it is.
    pub fn set_remote(&mut self, i: usize, pos: V3, vel: V3, mode: Mode, yaw: i32) {
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
            .all(|r| r.finish.is_some());
        if self.phase == Phase::Running && racing && !self.racers.is_empty() {
            self.phase = Phase::Done;
        }
    }

    /// Records a finish reported from another browser.
    pub fn remote_finish(&mut self, i: usize, clock: u32) {
        if i < self.racers.len() && self.racers[i].finish.is_none() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::int;
    use crate::geom::v3;

    fn entries(n: usize) -> Vec<Entry> {
        (0..n)
            .map(|i| Entry {
                name: format!("P{i}"),
                kind: Kind::ALL[i],
                driver: Driver::Local,
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
