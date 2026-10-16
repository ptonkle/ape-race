//! The AI (PLAN P12): one driver per AI ape, using exactly the controls a
//! player has (WASD relative to a heading it chooses, grab held to hang on,
//! jump, the item key). It never moves the ape itself.
//!
//! How it drives:
//!
//! - **On a vine** it holds W toward the vine it means to reach next, and
//!   plans when to let go. The physics is deterministic, so the plan is
//!   exact: it copies its ape, swings the copy ahead with the same controls
//!   it will use itself, and at each release moment flies a second copy from
//!   there (with the same air steering and grab) to see whether, when, how
//!   fast and onto which vine it would land. It lets go at the best moment,
//!   once a short look from that landing shows a way on from there too. The
//!   search runs one release moment a tick, ahead of the real swing (a burst
//!   when it first grabs), so its cost is spread out.
//! - **In the air** it steers across its motion toward the target vine, lined
//!   up for the vine after, and grabs once the target (or a vine beyond it)
//!   is within reach.
//! - **On the ground** (or the start platform) it runs at the nearest vine it
//!   can jump to, and jumps when a copy of itself jumping now would grab one.
//! - **Items**: simple moments for each (see [`use_item`]).
//! - **Difficulty**: [`Difficulty`] handicaps all of the above for the
//!   easier levels; [`Difficulty::KingKong`] (the default) is the full AI.
//!
//! Call [`think`] for ape `who` just before `Race::step`. [`trial`] runs AI
//! apes alone on a course, for the tests and for tuning (finish times,
//! speeds).

use crate::ape::{Ape, Controls, Happening, Mode};
use crate::course::Course;
use crate::fx::{cm, int, Dice, ONE};
use crate::geom::{pitch_of, v3, yaw_of, V3};
use crate::race::{Driver, Entry, Item, Race};
use crate::rules::{
    Kind, BODY_DROP, BOOST_GAIN, HOOK_RANGE, JUMP_SPEED, STICK_RANGE, TICKS_PER_SECOND,
};

/// A predicted flight counts a grab only this much inside the reach, so the
/// real flight (which grabs at [`GRAB_MARGIN`]) never falls just short.
const MARGIN: i32 = cm(15);
/// In the air it grabs this much inside its reach.
const GRAB_MARGIN: i32 = cm(5);
/// A flight that crosses the finish line "reaches" this.
const FINISH: usize = usize::MAX;
/// Banana Bay Battle: a flight that comes down on a deck ahead "reaches"
/// this (see [`BAY_DECK_AFTER`]).
const DECK: usize = usize::MAX - 1;
/// Banana Bay Battle (BT4): with the fleet's two battle lines the next rope
/// is often across the channel or behind a yard, and nearly every release
/// from a rope comes down on a deck, so a search that counted only ropes
/// found nothing and the ape whirled 7-10 s before it let go. Held this
/// long, it counts a landing on a deck ahead too (it runs off it at a rope),
/// at this many metres less than a rope as far along.
const BAY_DECK_AFTER: u32 = TICKS_PER_SECOND as u32;
const DECK_COST: i32 = int(10);
/// It grabs a vine only this far below its anchor or more.
const GOOD_ROPE: i32 = int(5);
/// A predicted flight is followed for at most this many ticks.
const FLIGHT_TICKS: u32 = 2 * TICKS_PER_SECOND as u32;
/// A plan looks this far ahead along the swing before it tries another
/// target.
const HORIZON: u32 = 3 * TICKS_PER_SECOND as u32;
/// Release moments are this many ticks apart.
const MOMENT: u32 = 3;
/// Release moments tried per tick (the cost budget: one predicted flight each).
const BUDGET: u32 = 1;
/// A release moment is taken only once the search has looked this far past
/// it for a better one (or has run to its horizon).
const COVER: u32 = 60;
/// Vines this far ahead are candidates.
const LOOK: i32 = int(70);
const MAX_CANDS: usize = 10;
/// On the Ski Slope the swings run faster downhill (30–40 m/s), and a
/// release flies further: it looks this far ahead, at this many vines (two
/// branches at a fork).
const SKI_LOOK: i32 = int(110);
const SKI_CANDS: usize = 16;
/// A search tries steering for this many of them in turn.
const STEERS: usize = 3;
/// After this long on one vine with no way on, it lets go forward.
const GIVE_UP: u32 = 7 * TICKS_PER_SECOND as u32;
/// Before letting go, it checks the landing has a way on: a second of
/// swinging there, a release tried every few ticks. A dead end costs this
/// much score.
const ONWARD_TICKS: u32 = TICKS_PER_SECOND as u32;
const ONWARD_STEP: u32 = 6;
const DEAD_END: i64 = 60 * ONE as i64;
/// Scoring a landing: metres along per m/s kept, per second waited, per
/// metre of rope.
const SPEED_WORTH: i64 = 1;
const WAIT_COST: i64 = 20;
const ROPE_WORTH: i64 = 3;
/// An ape on the ground with no vine this far ahead runs for the finish.
const LAST_GAP: i32 = int(10);
/// This close to the finish, an ape on the ground runs for it.
const FINISH_RUN: i32 = int(40);
/// In the air it aims short of the target vine, back along the next leg, by
/// the distance still to go over this.
const LEAD: i32 = 2;
/// Within this distance it aims at the vine itself.
const LEAD_NEAR: i32 = int(8);

/// How well an AI drives (TRIAGE T14). The top level is the AI at its best;
/// each level below handicaps it through the same controls: weaker arms (on
/// a vine it holds W only part of the time, and plans with that, so its
/// plans stay exact), a reaction time on each vine before it may let go
/// (with jitter), a liking for shorter hops (more vines on the way), a
/// random error on how good each release looks (so it sometimes takes a
/// worse one, and a worse vine), and a wait before it uses an item. Stuck
/// on one vine for two seconds, any level swings with all it has, and with
/// no new progress for three seconds it drives at its best for four, so even
/// the easiest finishes. All of it is deterministic, from the brain's seed.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Sloth,
    Monkey,
    Ape,
    /// the AI at its best (and the default)
    #[default]
    KingKong,
}

impl Difficulty {
    /// Easiest first.
    pub const ALL: [Difficulty; 4] = [
        Difficulty::Sloth,
        Difficulty::Monkey,
        Difficulty::Ape,
        Difficulty::KingKong,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Difficulty::Sloth => "sloth",
            Difficulty::Monkey => "monkey",
            Difficulty::Ape => "ape",
            Difficulty::KingKong => "king_kong",
        }
    }

    pub fn from_key(k: &str) -> Option<Difficulty> {
        Difficulty::ALL.into_iter().find(|d| d.key() == k)
    }

    /// A level's handicaps on a map: the Jungle's for every map, except on
    /// the City: there, with the Jungle's handicaps, the Monkey was no
    /// faster than the Sloth and the Ape barely faster than the Monkey
    /// (sharp corners and sparser vines spread the levels differently), so
    /// the City's Monkey reacts a little sooner and misjudges less, and its
    /// Ape has stronger arms and reacts sooner. On the Ski Slope the swings
    /// run fast downhill, so a level's reaction time would let the best
    /// release go by (stuck whirling on a vine): it reacts sooner there
    /// (`SKI_HESITATE`), and looks further ahead. Sunset Ruins and Banana
    /// Bay Battle have their own reaction times, arms and misjudging
    /// (`RUINS_*`, `BAY_*`).
    fn tuning_on(self, on: On) -> Tuning {
        let On {
            city,
            ski,
            sector,
            ruins,
            bay,
        } = on;
        let t = self.tuning();
        if bay {
            let k = self as usize;
            return Tuning {
                jitter: BAY_JITTER[k],
                drive: BAY_DRIVE[k],
                hesitate: BAY_HESITATE[k],
                noise: BAY_NOISE[k],
                ..t
            };
        }
        if ruins {
            let k = self as usize;
            return Tuning {
                hesitate: RUINS_HESITATE[k],
                jitter: RUINS_JITTER[k],
                drive: RUINS_DRIVE[k],
                noise: RUINS_NOISE[k],
                ..t
            };
        }
        if sector {
            // Sector Ape: low gravity and high speed make long, floaty
            // hops (see `SECTOR_*`)
            let k = self as usize;
            return Tuning {
                look: SECTOR_LOOK,
                cands: SECTOR_CANDS,
                flight: SECTOR_FLIGHT,
                hesitate: SECTOR_HESITATE[k],
                jitter: SECTOR_HESITATE[k],
                drive: SECTOR_DRIVE[k],
                noise: SECTOR_NOISE[k],
                ideal_hop: SECTOR_HOP[k],
                hidden: SECTOR_HIDDEN[k],
                ..t
            };
        }
        if ski {
            let k = self as usize;
            return Tuning {
                look: SKI_LOOK,
                cands: SKI_CANDS,
                hesitate: SKI_HESITATE[k],
                jitter: SKI_HESITATE[k],
                drive: SKI_DRIVE[k],
                noise: SKI_NOISE[k],
                ideal_hop: SKI_HOP[k],
                ..t
            };
        }
        match self {
            Difficulty::Monkey if city => Tuning {
                hesitate: CITY_MONKEY_HESITATE,
                jitter: CITY_MONKEY_HESITATE,
                noise: CITY_MONKEY_NOISE,
                ..t
            },
            Difficulty::Ape if city => Tuning {
                hesitate: CITY_APE_HESITATE,
                jitter: CITY_APE_HESITATE,
                drive: CITY_APE_DRIVE,
                ..t
            },
            _ => t,
        }
    }

    fn tuning(self) -> Tuning {
        let second = TICKS_PER_SECOND as u32;
        match self {
            Difficulty::Sloth => Tuning {
                ideal_hop: int(22),
                look: LOOK,
                cands: MAX_CANDS,
                hesitate: 14,
                jitter: 14,
                noise: int(6),
                drive: 2,
                item_wait: 6 * second,
                flight: FLIGHT_TICKS,
                hidden: true,
            },
            Difficulty::Monkey => Tuning {
                ideal_hop: int(24),
                look: LOOK,
                cands: MAX_CANDS,
                hesitate: 18,
                jitter: 14,
                noise: int(5),
                drive: 3,
                item_wait: 3 * second,
                flight: FLIGHT_TICKS,
                hidden: true,
            },
            Difficulty::Ape => Tuning {
                ideal_hop: int(26),
                look: LOOK,
                cands: MAX_CANDS,
                hesitate: 4,
                jitter: 4,
                noise: int(2),
                drive: 4,
                item_wait: second,
                flight: FLIGHT_TICKS,
                hidden: true,
            },
            Difficulty::KingKong => Tuning {
                ideal_hop: int(28),
                look: LOOK,
                cands: MAX_CANDS,
                hesitate: 0,
                jitter: 0,
                noise: 0,
                drive: DUTY,
                item_wait: 0,
                flight: FLIGHT_TICKS,
                hidden: true,
            },
        }
    }
}

/// The Ski Slope's reaction times, easiest level first (see
/// [`Difficulty::tuning_on`]). TRIAGE T84 (fewer low vines) moved the
/// Monkey's 2 → 3 and the Ape's drive 7 → 8 to keep the levels 6 s apart.
const SKI_HESITATE: [u32; 4] = [6, 3, 1, 0];
const SKI_DRIVE: [u32; 4] = [2, 5, 8, DUTY];
const SKI_NOISE: [i32; 4] = [int(6), int(4), int(2), 0];
const SKI_HOP: [i32; 4] = [int(22), int(24), int(26), int(28)];

/// Sunset Ruins' handicaps, easiest level first (see
/// [`Difficulty::tuning_on`]). With the Jungle's, the Monkey was no faster
/// than the Sloth and the Ape barely 6 s behind King Kong: the levels lose
/// their time in the same few places (the rope bridge, the mouth, the
/// turn in), where a long reaction time lets the good release go by. Here
/// the Monkey and the Ape react sooner and misjudge less, and the spacing
/// comes mostly from the arms (RT2). Once the layout's stalls were fixed
/// (RT6) the Monkey and the Ape bunched up again: the Ape got stronger
/// arms (5 of 8) and the Monkey a slower reaction (16 ticks, not 14). The
/// Monkey's misjudging is touchy: 6 m instead of 4 cost it 9 s. Once the
/// temple face was solid (1.11.1) the Sloth came within 1 s of the Monkey
/// and the Ape within 6 s of King Kong: the Sloth's arms are now 1 of 8 and
/// the Ape's 4, with a 6-tick reaction (RT11).
const RUINS_HESITATE: [u32; 4] = [12, 16, 6, 0];
const RUINS_JITTER: [u32; 4] = [6, 5, 5, 0];
const RUINS_DRIVE: [u32; 4] = [1, 3, 4, DUTY];
const RUINS_NOISE: [i32; 4] = [int(8), int(4), int(3), 0];
/// Banana Bay Battle's handicaps, easiest level first (see
/// [`Difficulty::tuning_on`]). With the Jungle's, the Sloth was no slower
/// than the Monkey (72.7 vs 67.1 s over 16 layouts, some Sloth runs 56 s):
/// the ships' swings are fast and long, so the Monkey's longer reaction
/// (18 ticks to the Sloth's 14) lets the good release go by about as often
/// as the Sloth's weak arms cost it. Here the Sloth reacts as late as the
/// Monkey (its arms stay 2 of 8: at 1 it could not get off two decks
/// it was put back on). The Ape then came within 6.6
/// s of King Kong: it reacts later here (8 ticks, not 4). Its misjudging
/// is touchy (3 m cost it 4.5 s; none left it whirling on a rope for 50 s
/// on one layout), so it stays 2 m (BT1).
const BAY_HESITATE: [u32; 4] = [18, 18, 5, 0];
const BAY_JITTER: [u32; 4] = [14, 14, 4, 0];
const BAY_DRIVE: [u32; 4] = [3, 3, 4, DUTY];
const BAY_NOISE: [i32; 4] = [int(6), int(5), int(2), 0];
/// On Sunset Ruins, how near a low vine over the river must be for a
/// swimmer to swim for it and catch it, not for the bank (RT10).
const RUINS_SWIM_TO: i32 = int(16);

/// Which map's tuning a brain uses (at most one is set; none is the Jungle).
#[derive(Debug, Clone, Copy)]
struct On {
    city: bool,
    ski: bool,
    sector: bool,
    ruins: bool,
    bay: bool,
}

/// The City's Monkey (see [`Difficulty::tuning_on`]).
const CITY_MONKEY_HESITATE: u32 = 12;
const CITY_MONKEY_NOISE: i32 = int(4);
/// The City's Ape: stronger arms and a quicker reaction than on the
/// Jungle, to keep the levels spaced there.
const CITY_APE_HESITATE: u32 = 2;
const CITY_APE_DRIVE: u32 = 6;

/// A difficulty's handicaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tuning {
    /// vines this far ahead are candidates, this many at most (by map)
    look: i32,
    cands: usize,
    /// the hop it likes best, metres
    ideal_hop: i32,
    /// on a vine it holds W this many ticks of every [`DUTY`] (weaker arms)
    drive: u32,
    /// it lets go of a vine no sooner than this many ticks after the grab,
    /// plus up to `jitter` more (drawn on each grab)
    hesitate: u32,
    jitter: u32,
    /// each release moment's score is off by up to this much either way,
    /// so it sometimes takes a worse one (and so a worse vine)
    noise: i32,
    /// it holds an item this many ticks before it thinks of using it
    item_wait: u32,
    /// a predicted flight is followed for at most this many ticks (by map)
    flight: u32,
    /// it may take a hidden route (Sector Ape's wreck passage)
    hidden: bool,
}

/// Sector Ape's tuning (see [`Difficulty::tuning_on`]), easiest level
/// first: flights at 25–60 m/s under low gravity carry 50–120 m, so it
/// looks further ahead, at more vines, follows a flight longer, likes
/// longer hops and reacts sooner (a whirl round an anchor at speed takes
/// under two seconds). Only the Ape and King Kong look for the hidden
/// route.
const SECTOR_LOOK: i32 = int(130);
const SECTOR_CANDS: usize = 16;
const SECTOR_FLIGHT: u32 = 4 * TICKS_PER_SECOND as u32;
const SECTOR_HESITATE: [u32; 4] = [4, 2, 0, 0];
const SECTOR_DRIVE: [u32; 4] = [3, 3, 6, DUTY];
const SECTOR_NOISE: [i32; 4] = [cm(1700), cm(600), int(3), 0];
const SECTOR_HOP: [i32; 4] = [int(22), int(28), int(28), int(28)];
const SECTOR_AIM: i32 = int(30);
/// Each reset onto a deck makes it misjudge up to this much more (four at
/// most).
const SHAKEN_NOISE: i32 = int(4);
/// Scoring a landing on Sector Ape (see `score`).
const SECTOR_SPEED_WORTH: i64 = 2;
const SECTOR_WAIT_COST: i64 = 40;
/// On Sector Ape a swing with no way on whirls round its anchor for good
/// (the low gravity): it gives up on it sooner.
const SECTOR_GIVE_UP: u32 = 3 * TICKS_PER_SECOND as u32;
/// Its search for a release on a fresh vine stops for the tick once its
/// flights have run this many ticks (and goes on the next).
const SECTOR_SPEND: u32 = 1_500;
/// Checking a landing has a way on, it tries a release every this many ticks.
const SECTOR_ONWARD_STEP: u32 = 12;

/// How long it holds a vine with no way on before letting go forward.
fn give_up(course: &Course) -> u32 {
    match course.sector {
        Some(_) => SECTOR_GIVE_UP,
        None => GIVE_UP,
    }
}
const SECTOR_HIDDEN: [bool; 4] = [false, false, false, true];

/// The drive pattern's length, ticks.
const DUTY: u32 = 8;
/// After this long on one vine any AI drives its swing in full.
const STRUGGLE: u32 = 2 * TICKS_PER_SECOND as u32;
/// An AI that has made no new progress along the course for this long is
/// stuck (a vine with no way on, the ground, a mushroom, ...): it drives at
/// its best for [`RESCUE`], then goes back to its own level.
const STUCK: u32 = 3 * TICKS_PER_SECOND as u32;
const RESCUE: u32 = 4 * TICKS_PER_SECOND as u32;

/// What an AI remembers between ticks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Brain {
    /// how well it drives
    pub difficulty: Difficulty,
    /// racing on the City (set by [`think`] each tick)
    city: bool,
    /// racing on the Ski Slope (set by [`think`] each tick)
    ski: bool,
    /// racing on Sector Ape (set by [`think`] each tick)
    sector: bool,
    /// racing on Sunset Ruins (set by [`think`] each tick)
    ruins: bool,
    /// racing on Banana Bay Battle (set by [`think`] each tick)
    bay: bool,
    /// its own dice, for the handicaps
    dice: Dice,
    /// the first tick it may let go of the vine it holds
    ready: u32,
    /// when it grabbed the vine it holds
    grabbed: u32,
    /// its furthest progress and when it got there, and the tick its
    /// rescue (driving at its best) ends
    best_prog: i32,
    best_at: u32,
    rescue_until: u32,
    /// For tuning: how often it was rescued.
    pub rescues: u32,
    /// Sector Ape: how often it has been put back on a deck
    pub shaken: u32,
    /// the tick it got the item it holds
    item_since: Option<u32>,
    /// the vine it is heading for
    pub target: Option<usize>,
    /// the vine after that, to line up for
    pub next: Option<usize>,
    /// the vines ahead it would grab
    pub cands: Vec<usize>,
    plan: Option<Box<Plan>>,
    /// grab was held last tick
    last_grab: bool,
    /// jump was held last tick
    last_jump: bool,
    /// For tuning: releases skipped because the swing was not where the
    /// plan said (the physics is deterministic, so this stays 0 unless
    /// something else moved the ape), the vine a release was planned to
    /// reach, how often the landing was on it, and how often it gave up on
    /// a vine and dropped.
    pub replans: u32,
    pub expect: Option<usize>,
    pub hits: u32,
    pub misses: u32,
    pub give_ups: u32,
    /// the vine it gave up on (not to be run back to), and the vine it let
    /// go of last
    avoid: Option<usize>,
    left: Option<usize>,
}

/// The release search for one swing.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Plan {
    vine: usize,
    /// the swing, simulated ahead of the real one
    sim: Ape,
    /// the tick `sim` steps next
    sim_tick: u32,
    /// where this pass of the search stops
    end: u32,
    /// which candidate it steers for
    pick: usize,
    /// when the ape grabbed this vine
    began: u32,
    /// release moments that reach a vine
    moments: Vec<Moment>,
    /// (Sector Ape) mechs' anchors its flights have asked for
    anchors: Anchors,
}

/// Sector Ape: mechs' vine anchors already worked out, by tick, for one
/// swing's plan. Its flights overlap (one leaves every `MOMENT` ticks and
/// flies seconds), so each is asked for again and again, and a mech's is
/// costly. Only sums of `Course::anchor` are kept: any two are equal.
#[derive(Clone, Default)]
struct Anchors {
    /// (tick, vine, anchor), by tick and vine (`ANCHOR_SLOTS` once used)
    slots: Vec<(u32, u32, V3)>,
}

const ANCHOR_SLOTS: usize = 4096;

impl Anchors {
    /// `course.anchor(i, tick)`.
    fn get(&mut self, course: &Course, i: usize, tick: u32) -> V3 {
        if self.slots.is_empty() {
            self.slots = vec![(u32::MAX, u32::MAX, V3::ZERO); ANCHOR_SLOTS];
        }
        let k = ((tick as usize & (ANCHOR_SLOTS / 8 - 1)) << 3) | (i & 7);
        let s = &mut self.slots[k];
        if s.0 == tick && s.1 == i as u32 {
            return s.2;
        }
        let a = course.anchor(i, tick);
        *s = (tick, i as u32, a);
        a
    }
}

impl PartialEq for Anchors {
    fn eq(&self, _: &Anchors) -> bool {
        true
    }
}

impl Eq for Anchors {}

impl std::fmt::Debug for Anchors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Anchors")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Moment {
    tick: u32,
    score: i64,
    /// the swing's state at that moment, to check the real one matches
    pos: V3,
    vel: V3,
    target: usize,
    next: Option<usize>,
    /// the vine it reaches
    reach: usize,
    /// whether a way on from that vine has been checked: 0 not yet, 1 there
    /// is one, 2 there is none
    onward: u8,
}

impl Brain {
    /// A brain at `difficulty`, its handicaps drawn from `seed` (say, the
    /// race's dice and the ape's index; any seed gives the same AI at the
    /// top level).
    pub fn new(difficulty: Difficulty, seed: u64) -> Brain {
        Brain {
            difficulty,
            city: false,
            ski: false,
            sector: false,
            ruins: false,
            bay: false,
            dice: Dice(seed),
            ready: 0,
            grabbed: 0,
            best_prog: i32::MIN,
            best_at: 0,
            rescue_until: 0,
            rescues: 0,
            shaken: 0,
            item_since: None,
            target: None,
            next: None,
            cands: Vec::new(),
            plan: None,
            last_grab: false,
            last_jump: false,
            replans: 0,
            expect: None,
            hits: 0,
            misses: 0,
            give_ups: 0,
            avoid: None,
            left: None,
        }
    }
}

impl Brain {
    /// The map it races on, for its tuning.
    fn on(&self) -> On {
        On {
            city: self.city,
            ski: self.ski,
            sector: self.sector,
            ruins: self.ruins,
            bay: self.bay,
        }
    }

    /// Its handicaps this tick: its level's, or none while rescued.
    fn tune(&self, tick: u32) -> Tuning {
        let mut t = if tick < self.rescue_until {
            // (a rescue never shows it a hidden route its level does not
            // know)
            let own = self.difficulty.tuning_on(self.on());
            Tuning {
                hidden: own.hidden,
                ..Difficulty::KingKong.tuning_on(self.on())
            }
        } else {
            self.difficulty.tuning_on(self.on())
        };
        // put back on a deck (Sector Ape), it misjudges a little more each
        // time: its plans are exact, so from the same deck it would make
        // the same mistake again
        if self.shaken > 0 {
            let more = SHAKEN_NOISE * self.shaken.min(4) as i32;
            t.noise = t.noise.max(more);
        }
        t
    }

    /// Watches for being stuck, and starts a rescue.
    fn watch(&mut self, tick: u32, progress: i32) {
        if progress > self.best_prog {
            self.best_prog = progress;
            self.best_at = tick;
        } else if tick >= self.best_at + STUCK && tick >= self.rescue_until {
            // (its plan stays: planning afresh would restart the give-up
            // clock on a vine with no way on)
            self.rescue_until = tick + RESCUE;
            self.best_at = tick;
            self.rescues += 1;
        }
    }
}

/// The AI at its best ([`Difficulty::KingKong`]).
impl Default for Brain {
    fn default() -> Brain {
        Brain::new(Difficulty::default(), 0)
    }
}

/// This tick's controls for AI ape `who`. Call it just before `Race::step`.
pub fn think(race: &Race, who: usize, brain: &mut Brain) -> Controls {
    let tick = race.tick + 1;
    let r = &race.racers[who];
    brain.city = race.course.city.is_some();
    brain.ski = race.course.ski.is_some();
    brain.sector = race.course.sector.is_some();
    brain.ruins = race.course.ruins.is_some();
    brain.bay = race.course.bay.is_some();
    // (Sector Ape: put back on a deck, it is shaken: see `Brain::tune`)
    if race
        .events
        .iter()
        .any(|e| matches!(e, crate::race::Event::Reset { who: w, .. } if *w == who))
    {
        brain.shaken += 1;
        // (and off the deck it drives at its best a while: from a standstill
        // a weak swing only whirls)
        brain.rescue_until = race.tick + 1 + RESCUE;
    }
    if race.running() && !r.out() && brain.difficulty != Difficulty::KingKong {
        brain.watch(tick, r.progress);
    }
    let tune = brain.tune(tick);
    let ape = &r.ape;
    let course = &race.course;
    if let Some(e) = brain.expect {
        if ape.happened.contains(&Happening::Grab) || ape.mode == Mode::Grounded {
            if ape.swinging() == Some(e) {
                brain.hits += 1;
            } else {
                brain.misses += 1;
            }
            brain.expect = None;
        }
    }
    let mut c = if !race.running() || r.out() {
        Controls {
            yaw: course.start_yaw,
            ..Controls::default()
        }
    } else if ape.stun > 0 {
        brain.plan = None;
        Controls::default()
    } else {
        match ape.mode {
            Mode::Swinging { vine, .. } => on_vine(ape, course, tick, vine, brain),
            Mode::Flying => {
                brain.plan = None;
                if brain.cands.is_empty() {
                    let at = progress(course, ape) - int(4);
                    brain.cands = ahead(course, ape, at, tick, tune);
                    brain.cands.retain(|&v| Some(v) != brain.left);
                    brain.target = brain.cands.first().copied();
                    brain.next = brain.target.and_then(|t| after(course, ape, t, tick, tune));
                }
                let target = brain.target.unwrap_or(0);
                let next = brain.next;
                flight_controls(
                    ape,
                    course,
                    tick,
                    target,
                    next,
                    &brain.cands,
                    brain.last_grab,
                )
            }
            Mode::Grounded if ape.swimming(course) || ape.climb.is_some() => {
                // in the City's river: no keys, so it swims for the nearest
                // way out (a bank, a ladder, a boat) and climbs out; on
                // Sunset Ruins a swimmer goes only where its keys say, so
                // it holds W toward the nearer bank
                brain.plan = None;
                brain.cands.clear();
                // (Sunset Ruins: a low vine over the river it ran for from
                // a bank is caught from the water; swimming back to the
                // bank it ran from, it ran for the same vine again: RT10)
                let low = brain
                    .target
                    .filter(|_| course.ruins.is_some() && ape.climb.is_none())
                    .filter(|&t| !drawn_up(course, t, tick))
                    .filter(|&t| course.vine_bottom(t, tick) <= ape.pos.y + ape.phys.reach - MARGIN)
                    .map(|t| course.anchor(t, tick).flat().sub(ape.pos.flat()))
                    .filter(|d| d.len() < RUINS_SWIM_TO);
                if let Some(d) = low {
                    Controls {
                        fwd: 1,
                        yaw: yaw_of(d),
                        grab: true,
                        ..Controls::default()
                    }
                } else {
                    let bank = course
                        .ruins
                        .is_some()
                        .then(|| course.water_exit(ape.pos))
                        .flatten()
                        .map(|(to, _)| to.flat().sub(ape.pos.flat()))
                        .filter(|d| d.len() > cm(20));
                    match bank {
                        Some(d) => Controls {
                            fwd: 1,
                            yaw: yaw_of(d),
                            ..Controls::default()
                        },
                        None => Controls {
                            yaw: ape.yaw,
                            ..Controls::default()
                        },
                    }
                }
            }
            Mode::Grounded => {
                brain.plan = None;
                on_ground(ape, course, tick, brain)
            }
        }
    };
    // it holds an item a while (by difficulty) before it thinks of using it
    let since = match r.item {
        Some(_) => *brain.item_since.get_or_insert(tick),
        None => {
            brain.item_since = None;
            tick
        }
    };
    if race.running()
        && !r.out()
        && r.item.is_some()
        && ape.swinging().is_none()
        && tick >= since + tune.item_wait
    {
        use_item(race, who, &mut c);
    }
    brain.last_grab = c.grab;
    brain.last_jump = c.jump;
    c
}

/// How far along the course an ape is.
fn progress(course: &Course, a: &Ape) -> i32 {
    course.progress(a.pos, a.hint).0
}

/// Vines ahead of `from` (metres along), the good ones first (well ahead
/// and not low), best lying first; then the rest, nearest first.
fn ahead(course: &Course, a: &Ape, from: i32, tick: u32, tune: Tuning) -> Vec<usize> {
    let v = a.vel.flat();
    let dir = if v.len() > int(5) {
        v.norm()
    } else {
        course.heading_at(from)
    };
    // how high its speed could carry it
    let top = a.pos.y + (a.vel.dot64(a.vel) / (2 * course.gravity() as i64)) as i32;
    ahead_from(
        course,
        a.swinging(),
        from,
        v3(a.pos.x, top, a.pos.z),
        dir,
        tick,
        tune,
    )
}

/// Vines ahead of `from`, as [`ahead`], seen from `origin` heading `dir`:
/// the good ones in order of how well they lie for a long hop on that line.
fn ahead_from(
    course: &Course,
    held: Option<usize>,
    from: i32,
    origin: V3,
    dir: V3,
    tick: u32,
    tune: Tuning,
) -> Vec<usize> {
    ahead_from_cached(course, held, from, origin, dir, tick, tune, None)
}

/// `ahead_from`, a mech's anchors from `anchors` (and each vine's anchor
/// and sort key worked out once: the same order, as the sort is stable).
#[allow(clippy::too_many_arguments)]
fn ahead_from_cached(
    course: &Course,
    held: Option<usize>,
    from: i32,
    origin: V3,
    dir: V3,
    tick: u32,
    tune: Tuning,
    mut anchors: Option<&mut Anchors>,
) -> Vec<usize> {
    let mut good: Vec<(i64, usize)> = Vec::new();
    let mut rest = Vec::new();
    for (i, v) in course.vines.iter().enumerate() {
        if v.at <= from
            || v.at > from + tune.look
            || Some(i) == held
            || off_limits(course, i, tune)
            || drawn_up(course, i, tick)
        {
            continue;
        }
        let an = anchor_cached(course, i, tick, anchors.as_deref_mut());
        let bottom = bottom_from(course, i, an);
        let low = bottom - course.terrain(an.x, an.z) < int(5);
        if v.at >= from + int(12) && !low {
            // off the line costs three times as much as a hop longer or
            // shorter than the ideal
            let d = an.flat().sub(origin.flat());
            let along = d.dot(dir);
            let lat = d.sub(dir.scale(along)).len();
            // and a vine that hangs out of reach above costs four times over
            let above = (bottom - origin.y).max(0);
            let key = lat as i64 * 3 + (along - tune.ideal_hop).abs() as i64 + above as i64 * 4;
            good.push((key, i));
        } else {
            rest.push(i);
        }
    }
    good.sort_by_key(|g| g.0);
    rest.sort_by_key(|&i| course.vines[i].at);
    let mut out: Vec<usize> = good.into_iter().map(|g| g.1).collect();
    out.extend(rest);
    out.truncate(tune.cands);
    out
}

/// `course.anchor(i, tick)`, from `anchors` when it is a mech's.
fn anchor_cached(course: &Course, i: usize, tick: u32, anchors: Option<&mut Anchors>) -> V3 {
    let mech = course
        .sector
        .as_ref()
        .is_some_and(|s| matches!(s.hangers.get(i), Some(crate::sector::Hanger::Mech { .. })));
    match anchors {
        Some(cache) if mech => cache.get(course, i, tick),
        _ => course.anchor(i, tick),
    }
}

/// `course.vine_bottom(i, tick)`, its anchor that tick `a` already found.
fn bottom_from(course: &Course, i: usize, a: V3) -> i32 {
    let len = course.vines[i].length;
    match course.sector {
        None => a.y - len,
        Some(_) => a.y.min(a.add(course.hang(i).scale(len)).y),
    }
}

/// Where it steers for vine `i`: its anchor, except that on Sector Ape a
/// mech's moving anchor (costly to find, and steered for every tick of
/// every planned flight) is taken where it was at the start.
fn aim_point(course: &Course, i: usize, tick: u32) -> V3 {
    if let Some(s) = &course.sector {
        if matches!(s.hangers.get(i), Some(crate::sector::Hanger::Mech { .. })) {
            return course.vines[i].anchor;
        }
    }
    course.anchor(i, tick)
}

/// Whether vine `i` is on a hidden route this level does not look for
/// (Sector Ape's wreck passage).
fn off_limits(course: &Course, i: usize, tune: Tuning) -> bool {
    // (Sunset Ruins: a pit's vines are for when its trapdoor is open:
    // `drawn_up`)
    !tune.hidden
        && course
            .sector
            .as_ref()
            .is_some_and(|s| s.routes.iter().any(|r| r.hidden && r.vines.contains(&i)))
}

/// Whether vine `i` is drawn up out of reach at `tick` (Sunset Ruins: a
/// pit vine over a shut trapdoor).
fn drawn_up(course: &Course, i: usize, tick: u32) -> bool {
    course.ruins.as_ref().is_some_and(|r| r.vine_up(i, tick))
}

/// The vine to make for after `target`.
fn after(course: &Course, a: &Ape, target: usize, tick: u32, tune: Tuning) -> Option<usize> {
    after_cached(course, a, target, tick, tune, None)
}

/// `after`, a mech's anchors from `anchors`.
fn after_cached(
    course: &Course,
    a: &Ape,
    target: usize,
    tick: u32,
    tune: Tuning,
    anchors: Option<&mut Anchors>,
) -> Option<usize> {
    let at = aim_point(course, target, tick);
    let dir = at.flat().sub(a.pos.flat());
    let dir = if dir.len() > int(2) {
        dir.norm()
    } else {
        course.heading_at(course.vines[target].at)
    };
    ahead_from_cached(
        course,
        Some(target),
        course.vines[target].at,
        at,
        dir,
        tick,
        tune,
        anchors,
    )
    .first()
    .copied()
}

/// How far the hands at `p` are from vine `i`.
fn vine_dist_from(course: &Course, i: usize, a: V3, p: V3) -> i32 {
    p.dist(vine_point_from(course, i, a, p))
}

/// The point of vine `i` nearest `p`, its anchor this tick `a` already
/// found (as `Course::vine_point`).
fn vine_point_from(course: &Course, i: usize, a: V3, p: V3) -> V3 {
    let len = course.vines[i].length;
    match course.sector {
        None => v3(a.x, p.y.clamp(a.y - len, a.y), a.z),
        Some(_) => {
            let h = course.hang(i);
            let t = p.sub(a).dot(h).clamp(0, len);
            a.add(h.scale(t))
        }
    }
}

/// The first candidate `margin` inside reach, at or beyond `min_at` along
/// the course, if any.
fn in_reach(
    a: &Ape,
    course: &Course,
    tick: u32,
    cands: &[usize],
    margin: i32,
    min_at: i32,
) -> Option<usize> {
    in_reach_watched(a, course, tick, cands, margin, min_at, None, None)
}

/// Sector Ape, over one planned flight: when each candidate is worth
/// looking at again. A candidate outside the box round where its anchor
/// can be (`Course::anchor_bound`) by some distance stays outside until the
/// ape has moved that far along x or z, so until then `in_reach` skips it:
/// the same answer, without the sums (the plans fly thousands of ticks a
/// tick).
struct Watch {
    /// how far the ape has moved, tick by tick, along x or z (the larger)
    moved: i64,
    last: V3,
    /// for each candidate, `moved` before which it cannot be in its box
    clear: Vec<i64>,
}

impl Watch {
    fn new(a: &Ape, cands: &[usize]) -> Watch {
        Watch {
            moved: 0,
            last: a.pos,
            clear: vec![0; cands.len()],
        }
    }

    /// After a step: count the move.
    fn follow(&mut self, a: &Ape) {
        let d = a.pos.sub(self.last);
        self.moved += (d.x as i64).abs().max((d.z as i64).abs());
        self.last = a.pos;
    }
}

/// `in_reach`, skipping the candidates `watch` says cannot be in reach.
fn in_reach_watched(
    a: &Ape,
    course: &Course,
    tick: u32,
    cands: &[usize],
    margin: i32,
    min_at: i32,
    mut watch: Option<&mut Watch>,
    mut anchors: Option<&mut Anchors>,
) -> Option<usize> {
    let reach = a.phys.reach - margin;
    for (k, &i) in cands
        .iter()
        .enumerate()
        .filter(|&(_, &i)| course.vines[i].at >= min_at)
    {
        if let Some(w) = &watch {
            if w.clear[k] > w.moved {
                continue;
            }
        }
        // (Sector Ape: a vine may reach out sideways from its anchor)
        let box_ = match course.sector {
            Some(_) => reach + course.vines[i].length,
            None => reach,
        };
        // (a moving anchor, a mech's, is costly to find: first a test
        // against where it can be at all)
        if course.sector.is_some() {
            let (c0, slack) = course.anchor_bound(i);
            let lim = box_ + slack;
            if (a.pos.x - c0.x).abs() > lim || (a.pos.z - c0.z).abs() > lim {
                if let Some(w) = watch.as_deref_mut() {
                    let out = (a.pos.x as i64 - c0.x as i64)
                        .abs()
                        .max((a.pos.z as i64 - c0.z as i64).abs());
                    w.clear[k] = w.moved + out - lim as i64;
                }
                continue;
            }
        }
        let an = anchor_cached(course, i, tick, anchors.as_deref_mut());
        if (a.pos.x - an.x).abs() > box_ || (a.pos.z - an.z).abs() > box_ {
            continue;
        }
        // not right up by the anchor: a short rope only whirls round
        if course.sector.is_some() {
            // (Sector Ape: by squared lengths, no square roots: the plans
            // fly long, and this runs for every candidate every tick)
            let near = vine_point_from(course, i, an, a.pos).sub(a.pos);
            let r = reach as i64;
            let off = a.pos.sub(an);
            if near.dot64(near) <= r * r && off.dot64(off) >= GOOD_ROPE as i64 * GOOD_ROPE as i64 {
                return Some(i);
            }
            continue;
        }
        if vine_dist_from(course, i, an, a.pos) <= reach && a.pos.dist(an) >= GOOD_ROPE {
            return Some(i);
        }
    }
    None
}

/// On a vine: hold W toward the target. The drive pushes along the swing
/// whichever way the ape moves, so this both speeds the swing toward the
/// target and turns a backswing round, and its part across the motion
/// steers the swing's plane onto the line. `coast` holds nothing (a search
/// pass that lets the swing run free).
fn swing_controls(
    a: &Ape,
    course: &Course,
    tick: u32,
    target: usize,
    coast: bool,
    tune: Tuning,
) -> Controls {
    let to = aim_point(course, target, tick).flat().sub(a.pos.flat());
    let drive = !coast && tick % DUTY < tune.drive;
    Controls {
        fwd: if drive { 1 } else { 0 },
        yaw: yaw_of(to),
        grab: true,
        ..Controls::default()
    }
}

/// In the air: steer across the motion toward the target vine, coming at
/// it along the line to the vine after it (a swing keeps the heading it was
/// grabbed with), and grab when the target or a vine beyond is in reach.
fn flight_controls(
    a: &Ape,
    course: &Course,
    tick: u32,
    target: usize,
    next: Option<usize>,
    cands: &[usize],
    last_grab: bool,
) -> Controls {
    let grab = !last_grab
        && in_reach(a, course, tick, cands, GRAB_MARGIN, aim_at(course, target)).is_some();
    let mut c = Controls {
        grab,
        ..Controls::default()
    };
    if target < course.vines.len() {
        let (at, leg) = aim_and_leg(course, tick, target, next);
        steer(a, at, leg, &mut c);
    }
    c
}

/// Where `flight_controls` aims (flat) and the unit direction of the leg
/// on to `next`.
fn aim_and_leg(course: &Course, tick: u32, target: usize, next: Option<usize>) -> (V3, Option<V3>) {
    let at = aim_point(course, target, tick).flat();
    let leg = next.map(|n| aim_point(course, n, tick).flat().sub(at).norm());
    (at, leg)
}

/// Whether `aim_and_leg` is the same at every tick: on Sector Ape the AI
/// aims at every vine's anchor where it starts (`aim_point`), so unless
/// a vine slides it is.
fn aim_is_fixed(course: &Course, target: usize, next: Option<usize>) -> bool {
    course.sector.is_some()
        && target < course.vines.len()
        && course.vines[target].slide.is_none()
        && next.is_none_or(|n| course.vines[n].slide.is_none())
}

/// The steering part of `flight_controls`, aiming at `at` (flat).
fn steer(a: &Ape, at: V3, leg: Option<V3>, c: &mut Controls) {
    let mut to = at.sub(a.pos.flat());
    if let Some(leg) = leg {
        // far out, aim short of the vine, back along the next leg, so the
        // ape comes at it heading on; close in, aim at the vine itself
        let d = to.len();
        if d > LEAD_NEAR {
            to = to.sub(leg.scale((d - LEAD_NEAR) / LEAD));
        }
    }
    let v = a.vel.flat();
    if !v.longer_than(ONE - 1) {
        c.fwd = 1;
        c.yaw = yaw_of(to);
    } else {
        let vn = v.norm();
        let along = to.dot(vn);
        let lat = to.sub(vn.scale(along));
        if along > 0 && lat.longer_than(cm(25)) {
            c.fwd = 1;
            c.yaw = yaw_of(lat);
        }
    }
}

/// Flies a copy of the ape from `tick` with the air policy and no hold on
/// any vine. Returns the candidate it would grab, the tick, and its speed
/// and rope there.
fn fly_ahead(
    a: &mut Ape,
    course: &Course,
    mut tick: u32,
    target: usize,
    next: Option<usize>,
    cands: &[usize],
    mut last_grab: bool,
    tune: Tuning,
    spent: &mut u32,
    mut anchors: Option<&mut Anchors>,
    deck: bool,
) -> Option<(usize, u32, i32, i32)> {
    let mut far = cands.iter().map(|&i| course.vines[i].at).max().unwrap_or(0) + int(8);
    if course.length - far < int(50) {
        // the finish line is in range too
        far = course.length + int(1);
    }
    // (Sector Ape: a long floaty flight that is below every candidate's
    // lowest point and falling can never come up to one again: stop
    // following it, as on the ground)
    let lowest = match course.sector {
        Some(_) => cands
            .iter()
            .map(|&i| course.vine_bottom(i, tick) - int(10))
            .min()
            .unwrap_or(i32::MIN),
        None => i32::MIN,
    };
    // (how far along it is: worked out once a tick, after each step)
    let mut along = progress(course, a);
    let from = along;
    // (Sector Ape: the candidates not worth looking at yet)
    let mut watch = course.sector.as_ref().map(|_| Watch::new(a, cands));
    // (and where it steers for, worked out once when that never moves)
    let fixed = aim_is_fixed(course, target, next).then(|| aim_and_leg(course, tick, target, next));
    for _ in 0..tune.flight {
        *spent += 1;
        if along >= course.length {
            // over the line
            return Some((FINISH, tick, a.vel.flat().len(), int(12)));
        }
        if a.pos.y < lowest && a.vel.y < 0 && far <= course.length {
            return None;
        }
        if a.swinging().is_none() {
            let hit = in_reach_watched(
                a,
                course,
                tick,
                cands,
                MARGIN,
                aim_at(course, target),
                watch.as_mut(),
                anchors.as_deref_mut(),
            );
            if let Some(v) = hit {
                if !last_grab {
                    let rope = a.pos.dist(course.anchor(v, tick));
                    // the speed that will carry on toward the vine after
                    // (a swing keeps its heading)
                    let on = match after_cached(course, a, v, tick, tune, anchors.as_deref_mut()) {
                        Some(n) => aim_point(course, n, tick).flat().sub(a.pos.flat()),
                        None => course.heading_at(course.vines[v].at),
                    };
                    let useful = a.vel.flat().dot(on.norm()).max(0);
                    return Some((v, tick, useful, rope));
                }
            }
        }
        let c = match fixed {
            Some((at, leg)) => {
                // (`flight_controls` with no grab, as below)
                let mut c = Controls::default();
                steer(a, at, leg, &mut c);
                c
            }
            None => {
                let mut c = flight_controls(a, course, tick, target, next, cands, true);
                c.grab = false;
                c
            }
        };
        a.step(&c, course, tick);
        last_grab = false;
        if let Some(w) = watch.as_mut() {
            w.follow(a);
        }
        along = progress(course, a);
        if deck && a.mode == Mode::Grounded && along > from {
            // (Banana Bay Battle: down on a deck ahead, not in the sea)
            let raised = course.support(a.pos) > course.terrain(a.pos.x, a.pos.z) + int(3);
            if raised {
                return Some((DECK, tick, 0, along));
            }
        }
        if a.mode == Mode::Grounded || along > far {
            return None;
        }
        // (Sector Ape: past the boundary is a reset, a flight lost)
        if course.sector.is_some() && course.out_of_bounds(a.pos, a.hint) {
            return None;
        }
        tick += 1;
    }
    None
}

/// How good a landing is, in metres-ish: further along, faster, and sooner.
/// (A landing on a deck, [`DECK`], carries how far along it is in place of
/// the rope's length.)
fn score(course: &Course, now: u32, (v, at_tick, speed, rope): (usize, u32, i32, i32)) -> i64 {
    let wait = (at_tick - now) as i64 * ONE as i64 / TICKS_PER_SECOND as i64;
    if v == DECK {
        return (rope - DECK_COST) as i64 - wait * WAIT_COST;
    }
    let at = if v == FINISH {
        // the finish beats any vine
        (course.length + int(100)) as i64
    } else {
        course.vines[v].at as i64
    };
    // a metre along is worth a metre per second of speed kept toward the
    // vine after, and a second of waiting costs 20 m; a longer rope makes a
    // bigger swing
    // (Sector Ape: at twice the speed a second and a metre per second are
    // worth more)
    let (speed_worth, wait_cost) = match course.sector {
        Some(_) => (SECTOR_SPEED_WORTH, SECTOR_WAIT_COST),
        None => (SPEED_WORTH, WAIT_COST),
    };
    at + speed as i64 * speed_worth - wait * wait_cost + rope.min(int(12)) as i64 * ROPE_WORTH
}

fn on_vine(a: &Ape, course: &Course, tick: u32, vine: usize, brain: &mut Brain) -> Controls {
    let mut tune = brain.tune(tick);
    let fresh = a.happened.contains(&Happening::Grab);
    if fresh && brain.avoid != Some(vine) {
        brain.avoid = None;
    }
    if fresh {
        // its reaction time on this vine
        brain.grabbed = tick;
        brain.ready = tick + tune.hesitate + brain.dice.below(tune.jitter + 1);
    }
    if tick.saturating_sub(brain.grabbed) > STRUGGLE {
        // stuck on this vine: a weak swing may never get it out, so it
        // swings with all it has
        tune.drive = DUTY;
    }
    let stale = match &brain.plan {
        Some(p) => p.vine != vine || fresh,
        None => true,
    };
    if stale {
        let from = course.vines[vine].at;
        brain.cands = ahead(course, a, from, tick, tune);
        brain.target = brain.cands.first().copied();
        brain.plan = Some(Box::new(Plan {
            vine,
            sim: a.clone(),
            sim_tick: tick,
            end: tick + HORIZON,
            pick: 0,
            began: tick,
            moments: Vec::new(),
            anchors: Anchors::default(),
        }));
    }
    if brain.cands.is_empty() {
        // nothing ahead: the finish. Swing on and let go forward.
        let h = course.heading_at(progress(course, a));
        let mut c = swing_controls(a, course, tick, vine, false, tune);
        c.yaw = yaw_of(h);
        c.grab = !(a.vel.y > 0 && a.vel.flat().dot(h) > 0);
        return c;
    }
    let Some(plan) = brain.plan.as_mut() else {
        return Controls::default();
    };

    // search ahead; a fresh plan looks far enough ahead at once to take a
    // moment right away: the best release is often the first upswing
    // (Sector Ape: its flights are long, so a burst that size cost tens of
    // milliseconds on one tick: it looks a third as far at once, then two
    // moments a tick, catching up within a fifth of a second)
    // (on Sector Ape it goes on searching that hard until it has looked
    // that far ahead, but stops for the tick once its flights have run
    // `SECTOR_SPEND` ticks)
    let budget = match course.sector {
        Some(_) if plan.sim_tick < plan.began + COVER => COVER / MOMENT + BUDGET,
        None if plan.sim_tick == plan.began => COVER / MOMENT + BUDGET,
        _ => BUDGET,
    };
    let cap = match course.sector {
        Some(_) => SECTOR_SPEND,
        None => u32::MAX,
    };
    let mut spent = 0;
    let mut after_steer: Option<(usize, Option<usize>)> = None;
    for _ in 0..budget {
        if spent >= cap {
            break;
        }
        if plan.sim_tick >= plan.end || plan.sim.swinging() != Some(vine) {
            // this pass is done: if nothing reached a vine, try the next
            // target and way of swinging, from now
            if plan.moments.iter().all(|m| m.tick < tick) {
                plan.pick += 1;
                plan.sim = a.clone();
                plan.sim_tick = tick;
                plan.end = tick + HORIZON;
                if plan.pick == 2 * brain.cands.len().min(STEERS) {
                    // nothing ahead will do: any vine near will, even one
                    // across the course at this station, to come at the next
                    // from a better line
                    let wide = near(course, a, tick, tune);
                    if !wide.is_empty() {
                        brain.cands = wide;
                    }
                    plan.pick = 0;
                }
            }
            break;
        }
        let (steer, coast) = pick(&brain.cands, plan.pick);
        // (the vine after the one it steers for, from where the real ape
        // is: the same all this tick, so worked out once)
        let next = match after_steer {
            Some((s, n)) if s == steer => n,
            _ => {
                let n = after(course, a, steer, tick, tune);
                after_steer = Some((steer, n));
                n
            }
        };
        let mut fl = plan.sim.clone();
        let hit = fly_ahead(
            &mut fl,
            course,
            plan.sim_tick,
            steer,
            next,
            &brain.cands,
            true,
            tune,
            &mut spent,
            Some(&mut plan.anchors),
            course.bay.is_some() && plan.sim_tick >= plan.began + BAY_DECK_AFTER,
        );
        if let Some(hit) = hit.filter(|_| plan.sim_tick >= brain.ready) {
            // a weaker AI misjudges how good a moment is
            let error = brain.dice.range(-tune.noise, tune.noise) as i64;
            plan.moments.push(Moment {
                tick: plan.sim_tick,
                score: score(course, plan.began, hit) + error,
                pos: plan.sim.pos,
                vel: plan.sim.vel,
                target: steer,
                next,
                reach: hit.0,
                onward: 0,
            });
        }
        for _ in 0..MOMENT {
            let c = swing_controls(&plan.sim, course, plan.sim_tick, steer, coast, tune);
            plan.sim.step(&c, course, plan.sim_tick);
            plan.sim_tick += 1;
        }
    }
    plan.moments.retain(|m| m.tick >= tick);
    let (steer, coast) = pick(&brain.cands, plan.pick);
    brain.target = Some(steer);
    let mut c = swing_controls(a, course, tick, steer, coast, tune);

    // the best moment still to come; let go if it is now
    let best = plan.moments.iter().copied().max_by_key(|m| m.score);
    if let Some(m) = best {
        let covered = plan.sim_tick >= m.tick + COVER || plan.sim_tick >= plan.end;
        let m = &m;
        if m.tick == tick && covered && m.onward == 0 && m.pos == a.pos && m.vel == a.vel {
            // the best so far: is there a way on from where it lands? If
            // not, count it down and wait for the next best
            let ok = onward(a, course, tick, m, &brain.cands, tune);
            for x in plan.moments.iter_mut().filter(|x| x.tick == tick) {
                x.onward = if ok { 1 } else { 2 };
                if !ok {
                    x.score -= DEAD_END;
                }
            }
        }
        let best = plan.moments.iter().copied().max_by_key(|m| m.score);
        if let Some(m) = best.filter(|m| m.tick == tick && covered && m.onward != 0) {
            if m.pos == a.pos && m.vel == a.vel {
                c = flight_controls(a, course, tick, m.target, m.next, &brain.cands, true);
                brain.target = Some(m.target);
                brain.next = m.next;
                brain.expect = (m.reach != DECK).then_some(m.reach);
                brain.left = Some(vine);
                brain.plan = None;
            } else {
                // the swing went another way than planned: plan again
                brain.replans += 1;
                brain.plan = None;
            }
        }
    } else if tick - plan.began > give_up(course) {
        // no way on from here: let go moving forward, and run for it
        let h = course.heading_at(progress(course, a));
        if a.vel.flat().dot(h) > a.vel.len() / 2 && a.vel.y >= 0 {
            c.grab = false;
            brain.plan = None;
            brain.cands.clear();
            brain.give_ups += 1;
            brain.avoid = Some(vine);
            brain.left = Some(vine);
        }
    }
    c
}

/// Whether the vine the release at moment `m` lands on has a way on: a
/// copy flies there, grabs it, and swings for a second, trying releases
/// toward the vine after.
fn onward(a: &Ape, course: &Course, tick: u32, m: &Moment, cands: &[usize], tune: Tuning) -> bool {
    if m.reach == FINISH || m.reach == DECK {
        return true;
    }
    let mut s = a.clone();
    let Some((_, t, _, _)) = fly_ahead(
        &mut s, course, tick, m.target, m.next, cands, true, tune, &mut 0, None, false,
    ) else {
        return false;
    };
    let press = Controls {
        grab: true,
        ..Controls::default()
    };
    s.step(&press, course, t);
    let Some(held) = s.swinging() else {
        return false;
    };
    let cands = ahead(course, &s, course.vines[held].at, t, tune);
    let Some(&steer) = cands.first() else {
        return true;
    };
    let next = after(course, &s, steer, t, tune);
    for (t, k) in (t + 1..).zip(0..ONWARD_TICKS) {
        // (Sector Ape: its flights are long; it tries half as often)
        let every = if course.sector.is_some() {
            SECTOR_ONWARD_STEP
        } else {
            ONWARD_STEP
        };
        if k % every == 0 {
            let mut f = s.clone();
            if fly_ahead(
                &mut f, course, t, steer, next, &cands, true, tune, &mut 0, None, false,
            )
            .is_some()
            {
                return true;
            }
        }
        let c = swing_controls(&s, course, t, steer, false, tune);
        s.step(&c, course, t);
        if s.swinging().is_none() {
            return false;
        }
    }
    false
}

/// A search pass's target and whether it lets the swing run free.
fn pick(cands: &[usize], pick: usize) -> (usize, bool) {
    let n = cands.len().min(STEERS);
    (cands[pick % n], (pick / n) % 2 == 1)
}

/// Every vine within 45 m but the one held, from its station on: the one
/// across the corridor, say.
fn near(course: &Course, a: &Ape, tick: u32, tune: Tuning) -> Vec<usize> {
    let at = progress(course, a);
    let held_at = a.swinging().map_or(at, |v| course.vines[v].at);
    let mut out: Vec<(i32, usize)> = Vec::new();
    for (i, v) in course.vines.iter().enumerate() {
        if Some(i) == a.swinging()
            || v.at < held_at
            || v.at > at + LOOK
            || off_limits(course, i, tune)
        {
            continue;
        }
        let d = course.anchor(i, tick).flat().sub(a.pos.flat()).len();
        if d < int(45) {
            out.push((v.at, i));
        }
    }
    out.sort();
    out.reverse();
    out.truncate(MAX_CANDS);
    out.into_iter().map(|(_, i)| i).collect()
}

/// On the ground or the start platform: run at a vine that can be jumped
/// to, and jump when a copy of the ape jumping now would grab one.
fn on_ground(a: &Ape, course: &Course, tick: u32, brain: &mut Brain) -> Controls {
    let tune = brain.tune(tick);
    let prog = progress(course, a);
    let support = course.support(a.pos);
    let raised = support > course.terrain(a.pos.x, a.pos.z) + int(3);
    // the highest a jump can reach
    let rise = (JUMP_SPEED as i64 * JUMP_SPEED as i64 / (2 * course.gravity() as i64)) as i32;
    let top = support + BODY_DROP + rise + a.phys.reach - MARGIN;
    let mut best: Option<(i64, usize)> = None;
    for (i, v) in course.vines.iter().enumerate() {
        if v.at < prog - int(15)
            || v.at > prog + int(120)
            || Some(i) == brain.avoid
            || off_limits(course, i, tune)
            || drawn_up(course, i, tick)
        {
            continue;
        }
        let an = course.anchor(i, tick);
        // (Sector Ape: a deck may be under vines out of a jump's reach, or
        // over vines hanging below it)
        // (Banana Bay Battle: a deck too: running off it at a rope out of
        // reach only drops the ape into the water and back on the deck)
        if (!raised || course.sector.is_some() || course.bay.is_some())
            && course.vine_bottom(i, tick) > top
        {
            continue;
        }
        // (... or behind it: running back off a deck only drops the ape off
        // it)
        if course.sector.is_some() && raised && v.at < prog {
            continue;
        }
        if course.sector.is_some() && an.y.max(course.tip(i, tick).y) < support {
            continue;
        }
        let d = an.flat().sub(a.pos.flat()).len() as i64;
        let behind = if v.at < prog { int(25) as i64 } else { 0 };
        // up on the start platform: the nearest of the first vines, so the
        // starters fan out onto the start row's vine in front of each
        // (TRIAGE T32: with `at` alone they all took the same one)
        let cost = if raised { v.at as i64 + d } else { d + behind };
        if best.is_none_or(|(b, _)| cost < b) {
            best = Some((cost, i));
        }
    }
    // near the finish there is nothing left to climb, and past the last
    // vine climbing back is no use: run for the line
    let last = course.vines.iter().map(|v| v.at).max().unwrap_or(0);
    let target = best
        .map(|(_, i)| i)
        .filter(|_| prog < course.length - FINISH_RUN && prog + LAST_GAP < last);
    brain.target = target;
    let mut c = Controls {
        fwd: 1,
        yaw: yaw_of(course.heading_at(prog)),
        ..Controls::default()
    };
    let Some(t) = target else {
        return c;
    };
    let to = course.anchor(t, tick).flat().sub(a.pos.flat());
    c.yaw = yaw_of(to);
    brain.cands = ahead(course, a, prog - int(20), tick, tune);
    if !brain.cands.contains(&t) {
        brain.cands.insert(0, t);
    }
    if brain.last_jump {
        return c;
    }
    // (Sector Ape: a jump floats for seconds in the low gravity; a vine in
    // reach from the deck is caught running off its edge)
    let hands = support + BODY_DROP + a.phys.reach - MARGIN;
    if course.sector.is_some() && raised && course.vine_bottom(t, tick) <= hands {
        return c;
    }
    // would a jump now catch a vine?
    let mut j = a.clone();
    let jump = Controls { jump: true, ..c };
    j.step(&jump, course, tick);
    let next = after(course, a, t, tick, tune);
    brain.next = next;
    if fly_ahead(
        &mut j,
        course,
        tick + 1,
        t,
        next,
        &brain.cands,
        false,
        tune,
        &mut 0,
        None,
        false,
    )
    .is_some()
    {
        c.jump = true;
    } else if !raised && to.len() < cm(40) {
        // right under it and still no: jump anyway
        c.jump = true;
    }
    c
}

/// Uses the item held, at a simple moment for each.
fn use_item(race: &Race, who: usize, c: &mut Controls) {
    let me = &race.racers[who];
    let a = &me.ape;
    let Some(item) = me.item else {
        return;
    };
    let course = &race.course;
    // the nearest other ape ahead, and the nearest at all
    let mut ahead: Option<(i32, usize)> = None;
    let mut close = i32::MAX;
    for (i, r) in race.racers.iter().enumerate() {
        if i == who || r.out() {
            continue;
        }
        let d = r.ape.pos.dist(a.pos);
        close = close.min(d);
        if r.progress > me.progress && ahead.is_none_or(|(b, _)| d < b) {
            ahead = Some((d, i));
        }
    }
    let fire = match item {
        Item::Boost => {
            // on a straight, moving along it
            let p = me.progress;
            let h0 = course.heading_at(p);
            let h1 = course.heading_at(p + int(60));
            let v = a.vel.flat();
            h0.dot(h1) > ONE * 95 / 100
                && v.len() > int(10)
                && v.norm().dot(h0) > ONE * 9 / 10
                && a.speed() + BOOST_GAIN < int(60)
        }
        Item::Hook => ahead.is_some_and(|(d, _)| d < HOOK_RANGE),
        Item::Banana => {
            if let Some((d, i)) = ahead {
                if d < int(40) {
                    let to = race.racers[i].ape.middle().sub(a.pos);
                    c.yaw = yaw_of(to);
                    c.pitch = pitch_of(to);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        Item::Stick => close < STICK_RANGE,
        // with no vine in reach, on the ground or falling
        Item::Vine => {
            a.swinging().is_none()
                && a.vine_in_reach(course, race.tick).is_none()
                && (a.mode == Mode::Grounded || a.vel.y < 0)
        }
    };
    c.item = fire;
}

/// How one AI ape did in a trial race.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trial {
    pub kind: Kind,
    /// race ticks at the finish, if it finished
    pub finish: Option<u32>,
    /// metres along the course at the end
    pub progress: i32,
    /// mean and top speed over the race, m/s (Q16)
    pub mean_speed: i32,
    pub top_speed: i32,
    /// how many vines it grabbed, and how often it landed on the ground
    pub grabs: u32,
    pub landings: u32,
    /// ticks spent on the ground
    pub ground_ticks: u32,
    /// whole bananas held at the end (TRIAGE T43, T60)
    pub bananas: u32,
    /// Floor is Lava (TRIAGE T37): the race clock when the jaguar ate it
    pub eaten: Option<u32>,
}

/// Runs a race of AI apes alone on `course` for at most `limit` race ticks,
/// for tests and tuning: finish times and typical speeds.
pub fn trial(course: Course, kinds: &[Kind], equal: bool, limit: u32) -> Vec<Trial> {
    let levels = vec![Difficulty::default(); kinds.len()];
    trial_at(course, kinds, &levels, equal, limit)
}

/// [`trial`] with each ape at its own difficulty (`levels[i]` for
/// `kinds[i]`), each brain seeded from its index.
pub fn trial_at(
    course: Course,
    kinds: &[Kind],
    levels: &[Difficulty],
    equal: bool,
    limit: u32,
) -> Vec<Trial> {
    trial_with(
        course,
        kinds,
        levels,
        equal,
        limit,
        crate::items::ItemSet::ALL,
        false,
    )
}

/// [`trial_at`] with Floor is Lava on (TRIAGE T37): for measuring how
/// often the AI touches the ground and is eaten.
pub fn lava_trial(
    course: Course,
    kinds: &[Kind],
    levels: &[Difficulty],
    equal: bool,
    limit: u32,
) -> Vec<Trial> {
    trial_with(
        course,
        kinds,
        levels,
        equal,
        limit,
        crate::items::ItemSet::ALL,
        true,
    )
}

/// One King Kong alone as a Time Trial runs (TRIAGE T1, T29: crates give
/// only the Speed Boost), for setting the medal times.
pub fn time_trial(course: Course, kind: Kind, equal: bool, limit: u32) -> Trial {
    let boost = crate::items::ItemSet::only(Item::Boost);
    trial_with(
        course,
        &[kind],
        &[Difficulty::KingKong],
        equal,
        limit,
        boost,
        false,
    )[0]
}

fn trial_with(
    course: Course,
    kinds: &[Kind],
    levels: &[Difficulty],
    equal: bool,
    limit: u32,
    items: crate::items::ItemSet,
    lava: bool,
) -> Vec<Trial> {
    let entries: Vec<Entry> = kinds
        .iter()
        .map(|&kind| Entry {
            name: String::new(),
            kind,
            driver: Driver::Ai,
            palette: 0,
            costume: crate::costume::Costume::Own,
        })
        .collect();
    let mut race = Race::new(course, &entries, equal, 7);
    race.items = items;
    race.lava = lava;
    let mut brains: Vec<Brain> = (0..kinds.len())
        .map(|i| Brain::new(levels[i], 0xA9E5 + i as u64))
        .collect();
    let mut out: Vec<Trial> = kinds
        .iter()
        .map(|&kind| Trial {
            kind,
            finish: None,
            progress: 0,
            mean_speed: 0,
            top_speed: 0,
            grabs: 0,
            landings: 0,
            ground_ticks: 0,
            bananas: 0,
            eaten: None,
        })
        .collect();
    let mut sum = vec![0i64; kinds.len()];
    while race.clock < limit && race.phase != crate::race::Phase::Done {
        let controls: Vec<Controls> = (0..kinds.len())
            .map(|i| think(&race, i, &mut brains[i]))
            .collect();
        race.step(&controls);
        if !race.running() {
            continue;
        }
        for (i, r) in race.racers.iter().enumerate() {
            if r.out() {
                if r.finish.is_none() && out[i].eaten.is_none() {
                    out[i].eaten = Some(race.clock);
                }
                continue;
            }
            let s = r.ape.speed();
            sum[i] += s as i64;
            let t = &mut out[i];
            t.top_speed = t.top_speed.max(s);
            if r.ape.mode == Mode::Grounded {
                t.ground_ticks += 1;
            }
            for h in &r.ape.happened {
                match h {
                    Happening::Grab => t.grabs += 1,
                    Happening::Land => t.landings += 1,
                    _ => {}
                }
            }
        }
    }
    for (i, r) in race.racers.iter().enumerate() {
        let t = &mut out[i];
        t.finish = r.finish;
        t.progress = r.progress;
        t.bananas = r.ape.whole_bananas();
        if r.finish.is_none() && r.eaten.is_some() && t.eaten.is_none() {
            t.eaten = Some(race.clock);
        }
        let ticks = r.finish.unwrap_or(race.clock).max(1);
        t.mean_speed = (sum[i] / ticks as i64) as i32;
    }
    out
}

/// In a flight aimed at `target` it grabs that vine or one beyond, not one
/// it passes on the way (a dense stretch would catch it at every station).
fn aim_at(course: &Course, target: usize) -> i32 {
    // (Sector Ape: a floaty flight may come down on a vine short of the
    // target; any within `SECTOR_AIM` of it will do)
    let slack = if course.sector.is_some() {
        SECTOR_AIM
    } else {
        int(2)
    };
    course.vines.get(target).map_or(i32::MIN, |v| v.at - slack)
}
