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
    /// (`SKI_HESITATE`), and looks further ahead.
    fn tuning_on(self, city: bool, ski: bool) -> Tuning {
        let t = self.tuning();
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
            },
        }
    }
}

/// The Ski Slope's reaction times, easiest level first (see
/// [`Difficulty::tuning_on`]).
const SKI_HESITATE: [u32; 4] = [6, 2, 1, 0];
const SKI_DRIVE: [u32; 4] = [2, 5, 7, DUTY];
const SKI_NOISE: [i32; 4] = [int(6), int(4), int(2), 0];
const SKI_HOP: [i32; 4] = [int(22), int(24), int(26), int(28)];

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
}

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
            dice: Dice(seed),
            ready: 0,
            grabbed: 0,
            best_prog: i32::MIN,
            best_at: 0,
            rescue_until: 0,
            rescues: 0,
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
    /// Its handicaps this tick: its level's, or none while rescued.
    fn tune(&self, tick: u32) -> Tuning {
        if tick < self.rescue_until {
            Difficulty::KingKong.tuning_on(self.city, self.ski)
        } else {
            self.difficulty.tuning_on(self.city, self.ski)
        }
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
                // way out (a bank, a ladder, a boat) and climbs out
                brain.plan = None;
                brain.cands.clear();
                Controls {
                    yaw: ape.yaw,
                    ..Controls::default()
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
    let mut good = Vec::new();
    let mut rest = Vec::new();
    for (i, v) in course.vines.iter().enumerate() {
        if v.at <= from || v.at > from + tune.look || Some(i) == held {
            continue;
        }
        let an = course.anchor(i, tick);
        let low = course.vine_bottom(i, tick) - course.terrain(an.x, an.z) < int(5);
        if v.at >= from + int(12) && !low {
            good.push(i);
        } else {
            rest.push(i);
        }
    }
    // off the line costs three times as much as a hop longer or shorter
    // than the ideal
    good.sort_by_key(|&i| {
        let d = course.anchor(i, tick).flat().sub(origin.flat());
        let along = d.dot(dir);
        let lat = d.sub(dir.scale(along)).len();
        // and a vine that hangs out of reach above costs four times over
        let above = (course.vine_bottom(i, tick) - origin.y).max(0);
        lat as i64 * 3 + (along - tune.ideal_hop).abs() as i64 + above as i64 * 4
    });
    rest.sort_by_key(|&i| course.vines[i].at);
    good.extend(rest);
    good.truncate(tune.cands);
    good
}

/// The vine to make for after `target`.
fn after(course: &Course, a: &Ape, target: usize, tick: u32, tune: Tuning) -> Option<usize> {
    let at = course.anchor(target, tick);
    let dir = at.flat().sub(a.pos.flat());
    let dir = if dir.len() > int(2) {
        dir.norm()
    } else {
        course.heading_at(course.vines[target].at)
    };
    ahead_from(
        course,
        Some(target),
        course.vines[target].at,
        at,
        dir,
        tick,
        tune,
    )
    .first()
    .copied()
}

/// How far the hands at `p` are from vine `i`.
fn vine_dist(course: &Course, i: usize, tick: u32, p: V3) -> i32 {
    p.dist(course.vine_point(i, tick, p))
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
    let reach = a.phys.reach - margin;
    for &i in cands.iter().filter(|&&i| course.vines[i].at >= min_at) {
        let an = course.anchor(i, tick);
        // (Sector Ape: a vine may reach out sideways from its anchor)
        let box_ = match course.sector {
            Some(_) => reach + course.vines[i].length,
            None => reach,
        };
        if (a.pos.x - an.x).abs() > box_ || (a.pos.z - an.z).abs() > box_ {
            continue;
        }
        // not right up by the anchor: a short rope only whirls round
        if vine_dist(course, i, tick, a.pos) <= reach && a.pos.dist(an) >= GOOD_ROPE {
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
    let to = course.anchor(target, tick).flat().sub(a.pos.flat());
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
        let at = course.anchor(target, tick).flat();
        let mut to = at.sub(a.pos.flat());
        if let Some(n) = next {
            // far out, aim short of the vine, back along the next leg, so the
            // ape comes at it heading on; close in, aim at the vine itself
            let leg = course.anchor(n, tick).flat().sub(at).norm();
            let d = to.len();
            if d > LEAD_NEAR {
                to = to.sub(leg.scale((d - LEAD_NEAR) / LEAD));
            }
        }
        let v = a.vel.flat();
        if v.len() < ONE {
            c.fwd = 1;
            c.yaw = yaw_of(to);
        } else {
            let vn = v.norm();
            let along = to.dot(vn);
            let lat = to.sub(vn.scale(along));
            if along > 0 && lat.len() > cm(25) {
                c.fwd = 1;
                c.yaw = yaw_of(lat);
            }
        }
    }
    c
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
) -> Option<(usize, u32, i32, i32)> {
    let mut far = cands.iter().map(|&i| course.vines[i].at).max().unwrap_or(0) + int(8);
    if course.length - far < int(50) {
        // the finish line is in range too
        far = course.length + int(1);
    }
    for _ in 0..FLIGHT_TICKS {
        if progress(course, a) >= course.length {
            // over the line
            return Some((FINISH, tick, a.vel.flat().len(), int(12)));
        }
        if a.swinging().is_none() {
            if let Some(v) = in_reach(a, course, tick, cands, MARGIN, aim_at(course, target)) {
                if !last_grab {
                    let rope = a.pos.dist(course.anchor(v, tick));
                    // the speed that will carry on toward the vine after
                    // (a swing keeps its heading)
                    let on = match after(course, a, v, tick, tune) {
                        Some(n) => course.anchor(n, tick).flat().sub(a.pos.flat()),
                        None => course.heading_at(course.vines[v].at),
                    };
                    let useful = a.vel.flat().dot(on.norm()).max(0);
                    return Some((v, tick, useful, rope));
                }
            }
        }
        let mut c = flight_controls(a, course, tick, target, next, cands, true);
        c.grab = false;
        a.step(&c, course, tick);
        last_grab = false;
        if a.mode == Mode::Grounded || progress(course, a) > far {
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
fn score(course: &Course, now: u32, (v, at_tick, speed, rope): (usize, u32, i32, i32)) -> i64 {
    let at = if v == FINISH {
        // the finish beats any vine
        (course.length + int(100)) as i64
    } else {
        course.vines[v].at as i64
    };
    let wait = (at_tick - now) as i64 * ONE as i64 / TICKS_PER_SECOND as i64;
    // a metre along is worth a metre per second of speed kept toward the
    // vine after, and a second of waiting costs 20 m; a longer rope makes a
    // bigger swing
    at + speed as i64 * SPEED_WORTH - wait * WAIT_COST + rope.min(int(12)) as i64 * ROPE_WORTH
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
    let budget = if plan.sim_tick == plan.began {
        COVER / MOMENT + BUDGET
    } else {
        BUDGET
    };
    for _ in 0..budget {
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
                    let wide = near(course, a, tick);
                    if !wide.is_empty() {
                        brain.cands = wide;
                    }
                    plan.pick = 0;
                }
            }
            break;
        }
        let (steer, coast) = pick(&brain.cands, plan.pick);
        let next = after(course, a, steer, tick, tune);
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
                brain.expect = Some(m.reach);
                brain.left = Some(vine);
                brain.plan = None;
            } else {
                // the swing went another way than planned: plan again
                brain.replans += 1;
                brain.plan = None;
            }
        }
    } else if tick - plan.began > GIVE_UP {
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
    if m.reach == FINISH {
        return true;
    }
    let mut s = a.clone();
    let Some((_, t, _, _)) = fly_ahead(&mut s, course, tick, m.target, m.next, cands, true, tune)
    else {
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
        if k % ONWARD_STEP == 0 {
            let mut f = s.clone();
            if fly_ahead(&mut f, course, t, steer, next, &cands, true, tune).is_some() {
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
fn near(course: &Course, a: &Ape, tick: u32) -> Vec<usize> {
    let at = progress(course, a);
    let held_at = a.swinging().map_or(at, |v| course.vines[v].at);
    let mut out: Vec<(i32, usize)> = Vec::new();
    for (i, v) in course.vines.iter().enumerate() {
        if Some(i) == a.swinging() || v.at < held_at || v.at > at + LOOK {
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
        if v.at < prog - int(15) || v.at > prog + int(120) || Some(i) == brain.avoid {
            continue;
        }
        let an = course.anchor(i, tick);
        if !raised && course.vine_bottom(i, tick) > top {
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
    // would a jump now catch a vine?
    let mut j = a.clone();
    let jump = Controls { jump: true, ..c };
    j.step(&jump, course, tick);
    let next = after(course, a, t, tick, tune);
    brain.next = next;
    if fly_ahead(&mut j, course, tick + 1, t, next, &brain.cands, false, tune).is_some() {
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
    /// bananas held at the end (TRIAGE T43)
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
        t.bananas = r.ape.bananas;
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
    course.vines.get(target).map_or(i32::MIN, |v| v.at - int(2))
}
