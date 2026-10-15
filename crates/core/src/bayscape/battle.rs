//! Banana Bay Battle's background battle (BQ2): the race runs through the
//! middle of a fight. Pairs of pirate and navy ships lie broadside to
//! broadside 60–250 m either side of the race line, outside the boundary
//! barrels (20 m), some with every sail set, some burning or listing,
//! trading broadsides. Scenery only: none of it is in the contract
//! (`BayLayout`), so no physics, camera or AI sees it; it is placed from
//! the course alone, the same on every run.
//!
//! **Still (chunked):** each ship in low detail (`baymodels::ship_far`).
//! **Each frame, near the eye:** the broadsides (a flash and two puffs at
//! each of three muzzles, the ball, the hit or splash), the fires and their
//! smoke columns, and a foam collar where each hull meets the sea.

use crate::bay::{BayLayout, Sail, Ship, ShipKind, Side};
use crate::baymodels::{self as m};
use crate::course::Course;
use crate::fx::{self, cm, deg, int, mul, ONE, TURN};
use crate::geom::{facing, right_of, v3, yaw_of, Affine, V3};
use crate::render::Batch;

/// No background hull comes nearer the race line than this (the boundary
/// barrels are at 20 m, the soft wall pushes apes back from there).
pub const NEAREST: i32 = int(60);
/// Nor further than this from it (the fog ends at 380 m).
pub const FARTHEST: i32 = int(250);
/// How far apart the battle's groups are along the course, each side.
const STEP: i32 = int(105);
/// The fire, smoke and foam are drawn nearer than this.
const DRAW_NEAR: i32 = int(380);
/// A background ship fires a broadside every this many ticks.
const CYCLE: u32 = 1_020;
/// A ball's flight between the lines (ticks).
const FLIGHT: u32 = 70;
/// A broadside's muzzle smoke, splash and hit (ticks).
const SMOKE_TICKS: u32 = 170;
const AFTER_TICKS: u32 = 120;
/// The guns of one broadside, a few ticks apart.
const GUNS: u32 = 3;
/// The effects' scale (as the course's own fire).
const FX_SCALE: i32 = ONE * 2;

/// One ship of the background battle.
#[derive(Debug, Clone)]
pub struct Far {
    pub ship: Ship,
    /// the ship it trades fire with (an index into the same list)
    pub foe: Option<usize>,
    pub burning: bool,
    /// its list (a roll, Q16 angle; 0 upright)
    pub list: i32,
    /// its broadsides' phase (ticks)
    pub phase: u32,
}

impl Far {
    /// Its frame: at its waterline, along its heading, rolled by its list.
    pub fn frame(&self) -> Affine {
        m::pose(self.ship.at, self.ship.yaw, ONE).then(&Affine::rot_z(self.list))
    }
}

fn hash(k: u32, salt: u32) -> u32 {
    let mut x = k.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^ (x >> 12)
}

fn pick(k: u32, salt: u32, lo: i32, hi: i32) -> i32 {
    lo + (hash(k, salt) % ((hi - lo).max(0) as u32 + 1)) as i32
}

/// The flat distance from `p` to the segment `a`–`z`.
fn to_segment(p: V3, a: V3, z: V3) -> i32 {
    let (p, a, z) = (p.flat(), a.flat(), z.flat());
    let d = z.sub(a);
    let len = d.len().max(1);
    let dir = d.scale(fx::div(ONE, len));
    let s = p.sub(a).dot(dir).clamp(0, len);
    p.sub(a.add(dir.scale(s))).len()
}

/// Whether a hull of half length `half` at `at` keeps clear of the race
/// line (`NEAREST`), the course's own ships, the pier and its headland,
/// the island and the battle's ships already placed.
fn clear(c: &Course, bay: &BayLayout, placed: &[Far], at: V3, half: i32) -> bool {
    let p = at.flat();
    let line = c
        .path
        .iter()
        .step_by(2)
        .map(|q| q.flat().sub(p).len())
        .min()
        .unwrap_or(0);
    let on_line = line >= NEAREST + half && line <= FARTHEST;
    on_line
        && bay
            .ships
            .iter()
            .all(|s| s.at.flat().sub(p).len() >= s.length / 2 + half + int(30))
        && to_segment(p, bay.dock.from, bay.dock.to) >= half + int(80)
        && bay.island.centre.flat().sub(p).len() >= bay.island.radius + half + int(50)
        && placed
            .iter()
            .all(|f| f.ship.at.flat().sub(p).len() >= f.ship.length / 2 + half + int(14))
}

/// The sails of a background ship: every one set (`set`), or a battle's
/// mix of set, torn, furled and shot away (more torn and gone on fire).
fn rig(mut s: Ship, set: bool, burning: bool, k: u32) -> Ship {
    let n: usize = s
        .masts
        .iter()
        .map(|m| m.yards.len() + m.gaff.is_some() as usize)
        .sum();
    let sails: Vec<Sail> = (0..n as u32)
        .map(|i| {
            let r = hash(k, 300 + i) % 10;
            match (set, burning) {
                (true, _) => Sail::Set,
                (_, true) if r < 4 => Sail::Missing,
                (_, true) => Sail::Torn,
                _ if r < 4 => Sail::Set,
                _ if r < 6 => Sail::Torn,
                _ if r < 9 => Sail::Furled,
                _ => Sail::Missing,
            }
        })
        .collect();
    s = m::with_sails(s, &sails);
    s.jib = None;
    s
}

/// The background battle on course `c`: groups every `STEP` along the
/// race line on each side, each a pirate and a navy ship broadside to
/// broadside (now and then a lone wreck), each at least `NEAREST` from the
/// line and at most `FARTHEST`. The same for the same course.
pub fn fleet(c: &Course, bay: &BayLayout) -> Vec<Far> {
    let mut out: Vec<Far> = Vec::new();
    if c.path.len() < 2 {
        return out;
    }
    let level = bay.water.level;
    let mut run = 0;
    let mut next = STEP / 2;
    let mut group = 0u32;
    for i in 1..c.path.len() {
        run += c.path[i].flat().sub(c.path[i - 1].flat()).len();
        if run < next {
            continue;
        }
        next += STEP;
        let fwd = c.heading[i].flat().norm();
        let right = right_of(fwd);
        for side in [-1, 1] {
            group += 1;
            let k = group;
            // two rows: the near groups and, every other step, the far ones
            let far_row = k.is_multiple_of(2);
            let off = if far_row {
                int(pick(k, 1, 175, 200))
            } else {
                int(pick(k, 2, 110, 135))
            };
            let along = int(pick(k, 3, -30, 30));
            let mid = c.path[i]
                .add(right.scale(side * off))
                .add(fwd.scale(along))
                .with_level(level);
            let yaw = yaw_of(fwd)
                + deg(pick(k, 4, -25, 25))
                + if hash(k, 5).is_multiple_of(2) {
                    TURN / 2
                } else {
                    0
                };
            let across = right_of(facing(yaw, 0));
            let gap = int(pick(k, 6, 40, 52));
            let kinds = [
                ShipKind::Galleon,
                ShipKind::Frigate,
                ShipKind::Frigate,
                ShipKind::Brig,
                ShipKind::Sloop,
            ];
            let lone = hash(k, 7).is_multiple_of(6);
            let first = out.len();
            // the group as drawn, pushed further out until it all clears
            // the line (each ship alone if the group never does)
            let make = |push: i32, j: usize, sign: i32| {
                let kk = k * 2 + j as u32;
                let kind = kinds[(hash(kk, 8) % kinds.len() as u32) as usize];
                let team = if (j as u32 + hash(k, 9)).is_multiple_of(2) {
                    Side::Pirate
                } else {
                    Side::Navy
                };
                let at = if lone {
                    mid
                } else {
                    mid.add(across.scale(sign * gap / 2))
                };
                let at = at
                    .add(right.scale(side * push))
                    .add(facing(yaw, 0).scale(int(pick(kk, 10, -6, 6))));
                (
                    kk,
                    m::typical(kind, team, at, yaw + deg(pick(kk, 11, -6, 6))),
                )
            };
            let n = if lone { 1 } else { 2 };
            let fits = |out: &[Far], s: &Ship| clear(c, bay, out, s.at, s.length / 2 + s.bowsprit);
            let mut group: Vec<(u32, Ship)> = Vec::new();
            for push in [0, 15, 30, 45] {
                let tried: Vec<(u32, Ship)> =
                    (0..n).map(|j| make(int(push), j, [-1, 1][j])).collect();
                let ok = tried.iter().all(|(_, s)| fits(&out, s))
                    && (n == 1 || {
                        let (a, b) = (&tried[0].1, &tried[1].1);
                        a.at.flat().sub(b.at.flat()).len() >= int(30)
                    });
                if ok {
                    group = tried;
                    break;
                }
            }
            for (kk, s) in group {
                let burning = lone || hash(kk, 12).is_multiple_of(5);
                let set = !burning && hash(kk, 13).is_multiple_of(3);
                let list = if burning {
                    deg(pick(kk, 14, 5, 11))
                        * if hash(kk, 15).is_multiple_of(2) {
                            1
                        } else {
                            -1
                        }
                } else {
                    0
                };
                out.push(Far {
                    ship: rig(s, set, burning, kk),
                    foe: None,
                    burning,
                    list,
                    phase: (hash(kk, 16) % CYCLE),
                });
            }
            if out.len() == first + 2 {
                out[first].foe = Some(first + 1);
                out[first + 1].foe = Some(first);
            }
        }
    }
    out
}

/// `fleet`, kept from the last call for the same course (placing it
/// checks every hull against the race line, too much for each frame).
pub fn cached(c: &Course, bay: &BayLayout) -> std::rc::Rc<Vec<Far>> {
    type Key = (usize, V3, V3, usize);
    thread_local! {
        static LAST: std::cell::RefCell<Option<(Key, std::rc::Rc<Vec<Far>>)>> =
            const { std::cell::RefCell::new(None) };
    }
    let key: Key = (
        c.path.len(),
        c.path.first().copied().unwrap_or(V3::ZERO),
        c.path.last().copied().unwrap_or(V3::ZERO),
        bay.ships.len(),
    );
    LAST.with(|l| {
        let mut l = l.borrow_mut();
        if let Some((k, f)) = l.as_ref() {
            if *k == key {
                return f.clone();
            }
        }
        let f = std::rc::Rc::new(fleet(c, bay));
        *l = Some((key, f.clone()));
        f
    })
}

trait WithLevel {
    fn with_level(self, y: i32) -> V3;
}

impl WithLevel for V3 {
    fn with_level(self, y: i32) -> V3 {
        v3(self.x, y, self.z)
    }
}

/// The background ships, still, into the course's chunks.
pub fn still(batches: &mut [Batch], ships: &[Far], chunk: &dyn Fn(V3) -> usize) {
    for (k, f) in ships.iter().enumerate() {
        m::ship_far(
            &mut batches[chunk(f.ship.at)],
            &f.frame(),
            &f.ship,
            k as u32,
        );
    }
}

/// Where a far ship's broadside toward its foe comes from: the side
/// facing it (+1 starboard).
fn broadside(s: &Ship, foe: &Ship) -> i32 {
    if foe.at.sub(s.at).dot(s.right()) >= 0 {
        1
    } else {
        -1
    }
}

/// What the background battle moves near `eye` at `tick`: the
/// broadsides, the fires and the foam round each hull. `level` is the
/// sea's height there (`height(x, z)`).
pub fn dynamic(b: &mut Batch, ships: &[Far], eye: V3, tick: u32, height: &dyn Fn(i32, i32) -> i32) {
    let near = |p: V3| p.flat().sub(eye.flat()).len() < DRAW_NEAR;
    for (k, f) in ships.iter().enumerate() {
        let s = &f.ship;
        if !near(s.at) {
            continue;
        }
        let k = k as u32;
        // the foam where the hull meets the sea
        let y = height(s.at.x, s.at.z);
        m::foam(
            b,
            &m::pose(s.at.with_level(y), s.yaw, ONE),
            s.beam * 13 / 10,
            s.length * 105 / 100,
            k,
        );
        if f.burning {
            let deck = f
                .frame()
                .apply(v3(0, s.deck, s.length * pick(k, 20, -20, 20) / 100));
            m::fire_far(b, &m::pose(deck, 0, ONE * 3 / 2), tick, k);
        }
        let Some(foe) = f.foe.and_then(|i| ships.get(i)) else {
            continue;
        };
        if f.burning {
            continue;
        }
        let side = broadside(s, &foe.ship);
        let into = (tick + f.phase) % CYCLE;
        let round = (tick + f.phase) / CYCLE;
        for g in 0..GUNS {
            let Some(t) = into.checked_sub(g * 9) else {
                continue;
            };
            let along = s.length * (g as i32 - 1) / 4;
            let muzzle = s.point(along, side * (s.beam / 2 + cm(30)), s.deck * 55 / 100);
            let out = s.right().scale(side);
            if t < SMOKE_TICKS {
                let age = (t * ONE as u32 / SMOKE_TICKS) as i32;
                m::smoke_far(b, &m::pose(muzzle, yaw_of(out), FX_SCALE), age, k + g);
            }
            let them = &foe.ship;
            let hit = !hash(k * 8 + g, round).is_multiple_of(3);
            let target = if hit {
                them.point(
                    along * 8 / 10,
                    -broadside(them, s) * them.beam / 2,
                    them.deck * 45 / 100,
                )
            } else {
                // short, into the sea between the lines
                let p = them.point(along, -broadside(them, s) * (them.beam / 2 + int(7)), 0);
                p.with_level(height(p.x, p.z))
            };
            if t < FLIGHT {
                let u = (t * ONE as u32 / FLIGHT) as i32;
                let arc = mul(4 * mul(u, ONE - u), int(5));
                let at = muzzle.lerp(target, u).add(v3(0, arc, 0));
                m::cannonball(b, &m::pose(at, 0, ONE * 3), V3::ZERO);
            } else if t < FLIGHT + AFTER_TICKS {
                let age = ((t - FLIGHT) * ONE as u32 / AFTER_TICKS) as i32;
                if hit {
                    m::hull_hit(b, &m::pose(target, yaw_of(out), FX_SCALE), age, k + g);
                } else {
                    m::splash(b, &m::pose(target, 0, FX_SCALE), age, k + g);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Map;

    /// BQ2: the background battle is there on both sides, and every hull
    /// in it (its bowsprit included) keeps `NEAREST` from the race line and
    /// within `FARTHEST`, clear of the course's ships, the pier and the
    /// island.
    #[test]
    fn the_background_battle_keeps_clear_of_the_course() {
        let c = Map::Bay.course();
        let bay = c.bay.as_ref().unwrap();
        let ships = fleet(&c, bay);
        let foes = ships.iter().filter(|f| f.foe.is_some()).count();
        assert!(
            ships.len() >= 24 && foes >= 12,
            "{} background ships, {foes} in duels",
            ships.len()
        );
        let mut sides = [0, 0];
        for (k, f) in ships.iter().enumerate() {
            let s = &f.ship;
            let reach = s.length / 2 + s.bowsprit;
            let line = c
                .path
                .iter()
                .map(|q| q.flat().sub(s.at.flat()).len())
                .min()
                .unwrap();
            assert!(
                line - reach >= NEAREST - int(1),
                "ship {k}: hull {} m off the race line",
                (line - reach) / ONE
            );
            assert!(line <= FARTHEST, "ship {k}: {} m out", line / ONE);
            for o in &bay.ships {
                assert!(o.at.flat().sub(s.at.flat()).len() > o.length / 2 + reach);
            }
            let i = c
                .path
                .iter()
                .enumerate()
                .min_by_key(|(_, q)| q.flat().sub(s.at.flat()).len())
                .unwrap()
                .0;
            let right = right_of(c.heading[i].flat().norm());
            sides[(s.at.sub(c.path[i]).dot(right) > 0) as usize] += 1;
        }
        assert!(sides[0] >= 8 && sides[1] >= 8, "{sides:?}");
        assert!(ships.iter().any(|f| f.burning));
        assert!(ships.iter().any(|f| f.foe.is_some()));
        assert!(ships.iter().any(|f| f
            .ship
            .masts
            .iter()
            .all(|m| m.yards.iter().all(|y| y.sail == Sail::Set))));
    }

    /// BQ2: the background ships trade fire over a cycle: balls fly and
    /// muzzles smoke somewhere in it.
    #[test]
    fn the_background_ships_fire_at_each_other() {
        let c = Map::Bay.course();
        let bay = c.bay.as_ref().unwrap();
        let ships = fleet(&c, bay);
        let eye = ships[0].ship.at;
        let mut most = 0;
        let mut least = usize::MAX;
        for tick in (0..CYCLE).step_by(30) {
            let mut b = Batch::new();
            dynamic(&mut b, &ships, eye, tick, &|_, _| bay.water.level);
            let n = b.count_of(crate::mesh::Mesh::Octa) + b.count_of(crate::mesh::Mesh::Ico);
            most = most.max(n);
            least = least.min(n);
        }
        assert!(
            most > least,
            "no flash or ball changes over a cycle: {most} {least} {} ships, {} with foes",
            ships.len(),
            ships
                .iter()
                .filter(|f| f.foe.is_some() && !f.burning)
                .count()
        );
    }
}
