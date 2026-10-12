//! Items (PLAN P9): crates, pickups, the Banana, Stick, Hook and Speed Boost.
//!
//! Who decides a hit: the browser that runs the user. Apes driven here
//! (Local, Ai) are changed here; a Remote ape is moved by its own browser,
//! so a hit on it only emits `Event::Hit`, which the online layer forwards,
//! and the victim's browser calls [`apply_hit`]. A shot thrown by a Remote
//! ape (if the online layer replays one with `use_item`) is drawn but decides
//! nothing here: its owner's browser decides its hits.

use crate::ape::{Controls, Happening};
use crate::fx::{self, mul, ONE};
use crate::geom::{facing, v3, yaw_of, Affine, V3};
use crate::mesh::Mesh;
use crate::race::{Driver, Event, Item, Race, Shot};
use crate::render::{self, placed, rgb, rod, Batch};
use crate::rules::*;

/// Ape `who` pressed the item key: use and consume the held item.
pub fn use_item(race: &mut Race, who: usize, c: &Controls) {
    let Some(item) = race.racers.get(who).and_then(|r| r.item) else {
        return;
    };
    race.racers[who].item = None;
    race.events.push(Event::Used { who, item });
    let owned = race.racers[who].driver != Driver::Remote;
    let ape = &race.racers[who].ape;
    let (pos, vel) = (ape.middle(), ape.vel);
    match item {
        Item::Banana => {
            let dir = facing(c.yaw, c.pitch + BANANA_LOB);
            race.shots.push(Shot {
                item,
                by: who,
                pos: pos.add(dir.scale(BODY_RADIUS)),
                vel: vel.add(dir.scale(BANANA_SPEED)),
                life: BANANA_LIFE,
                target: None,
            });
        }
        Item::Stick => {
            for i in 0..race.racers.len() {
                if owned && i != who && race.racers[i].ape.middle().dist(pos) <= STICK_RANGE {
                    strike(race, i, who, Item::Stick);
                }
            }
        }
        Item::Hook => {
            let target = hook_target(race, who);
            let dir = match target {
                Some(t) => race.racers[t].ape.middle().sub(pos).norm(),
                None => facing(c.yaw, c.pitch),
            };
            race.shots.push(Shot {
                item,
                by: who,
                pos,
                vel: dir.scale(HOOK_SPEED),
                life: HOOK_LIFE,
                target,
            });
        }
        Item::Boost => {
            if owned {
                let dir = if vel.len() > ONE / 2 {
                    vel.norm()
                } else {
                    facing(c.yaw, 0)
                };
                race.racers[who].ape.vel = vel.add(dir.scale(BOOST_GAIN));
            }
        }
    }
}

/// The Hook's target: the nearest ape ahead (further along the course)
/// within `HOOK_RANGE` that has not finished.
pub fn hook_target(race: &Race, who: usize) -> Option<usize> {
    let me = &race.racers[who];
    let at = me.ape.middle();
    let mut best: Option<(i32, usize)> = None;
    for (i, r) in race.racers.iter().enumerate() {
        if i == who || r.finish.is_some() || r.progress <= me.progress {
            continue;
        }
        let d = r.ape.middle().dist(at);
        if d <= HOOK_RANGE && best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, i));
        }
    }
    best.map(|(_, i)| i)
}

/// A random item for an ape in `place` of `n`, Mario Kart style: the
/// leaders get more Bananas and Sticks, those behind more Hooks and Boosts.
pub fn roll(dice: &mut fx::Dice, place: usize, n: usize) -> Item {
    let behind = if n > 1 {
        ((place.clamp(1, n) - 1) * 100 / (n - 1)) as i32
    } else {
        0
    };
    let weights = [
        (Item::Banana, 40 - behind / 4),
        (Item::Stick, 30 - behind / 5),
        (Item::Hook, 15 + behind / 4),
        (Item::Boost, 15 + behind / 5),
    ];
    let total: i32 = weights.iter().map(|w| w.1).sum();
    let mut pick = dice.below(total as u32) as i32;
    for (item, w) in weights {
        if pick < w {
            return item;
        }
        pick -= w;
    }
    Item::Boost
}

/// Crates, pickups and thrown things, once a tick after the apes move.
pub fn step(race: &mut Race) {
    pickups(race);
    fly(race);
}

fn pickups(race: &mut Race) {
    let n = race.racers.len();
    for k in 0..race.crates.len() {
        if race.crates[k] > 0 {
            race.crates[k] -= 1;
            if race.crates[k] > 0 {
                continue;
            }
        }
        let at = race.course.crates[k];
        for i in 0..n {
            let r = &race.racers[i];
            // a Remote ape picks up in its own browser
            if r.item.is_some() || r.finish.is_some() || r.driver == Driver::Remote {
                continue;
            }
            if r.ape.middle().dist(at) <= CRATE_RADIUS + BODY_RADIUS {
                let item = roll(&mut race.dice, r.place, n);
                race.racers[i].item = Some(item);
                race.crates[k] = CRATE_RESPAWN;
                race.events.push(Event::Pickup { who: i, item });
                break;
            }
        }
    }
}

fn fly(race: &mut Race) {
    let shots = std::mem::take(&mut race.shots);
    let mut kept = Vec::with_capacity(shots.len());
    for mut s in shots {
        s.life -= 1;
        let owned = race.racers.get(s.by).map(|r| r.driver) != Some(Driver::Remote);
        let done = match s.item {
            Item::Banana => {
                s.vel.y -= mul(GRAVITY, DT);
                s.pos = s.pos.add(s.vel.scale(DT));
                let hit = (0..race.racers.len()).find(|&i| {
                    i != s.by
                        && race.racers[i].ape.middle().dist(s.pos) <= BANANA_RADIUS + BODY_RADIUS
                });
                if let Some(i) = hit {
                    if owned {
                        strike(race, i, s.by, Item::Banana);
                    }
                    true
                } else {
                    s.pos.y <= race.course.support(s.pos)
                }
            }
            Item::Hook => match s.target.filter(|t| *t < race.racers.len()) {
                Some(t) => {
                    let to = race.racers[t].ape.middle().sub(s.pos);
                    let step = mul(HOOK_SPEED, DT);
                    if to.len() <= step + BODY_RADIUS {
                        s.pos = s.pos.add(to);
                        if owned {
                            hooked(race, t, s.by);
                        }
                        true
                    } else {
                        s.vel = to.with_len(HOOK_SPEED);
                        s.pos = s.pos.add(s.vel.scale(DT));
                        false
                    }
                }
                None => {
                    s.pos = s.pos.add(s.vel.scale(DT));
                    false
                }
            },
            Item::Stick | Item::Boost => true,
        };
        if !done && s.life > 0 {
            kept.push(s);
        }
    }
    race.shots = kept;
}

/// The Hook arrives: the target is hit and pulled back, and the user is
/// pulled toward it.
fn hooked(race: &mut Race, target: usize, by: usize) {
    strike(race, target, by, Item::Hook);
    if race.racers[by].driver != Driver::Remote {
        let dir = race.racers[target]
            .ape
            .pos
            .sub(race.racers[by].ape.pos)
            .norm();
        let me = &mut race.racers[by].ape;
        me.vel = me.vel.add(dir.scale(HOOK_GAIN));
    }
}

/// A hit decided in this browser: always an event; the effect only on an
/// ape this browser moves.
fn strike(race: &mut Race, who: usize, by: usize, item: Item) {
    race.events.push(Event::Hit { who, by, item });
    if race.racers[who].driver != Driver::Remote {
        effect(race, who, by, item);
    }
}

/// Applies a hit decided by another browser to an ape this browser moves,
/// and emits its `Event::Hit` (for sounds; do not forward a hit whose `who`
/// is local). Ignored for a Remote ape or a bad index. `race.step` clears
/// events, so read them before the next step.
pub fn apply_hit(race: &mut Race, who: usize, by: usize, item: Item) {
    let n = race.racers.len();
    if who >= n || by >= n || who == by || race.racers[who].driver == Driver::Remote {
        return;
    }
    race.events.push(Event::Hit { who, by, item });
    effect(race, who, by, item);
}

/// What a hit does to ape `who`, struck by `by`. Pushes are scaled by
/// Weight like the rest of the hit.
fn effect(race: &mut Race, who: usize, by: usize, item: Item) {
    let from = race.racers[by].ape.pos;
    let ape = &mut race.racers[who].ape;
    let scale = ape.phys.hit_scale;
    match item {
        Item::Banana => ape.hit(BANANA_KEEP, BANANA_STUN, V3::ZERO),
        Item::Stick => {
            let push = ape.pos.sub(from).flat().with_len(mul(STICK_PUSH, scale));
            ape.hit(STICK_KEEP, STICK_STUN, push);
        }
        Item::Hook => {
            let pull = from.sub(ape.pos).with_len(mul(HOOK_PULL, scale));
            ape.hit(HOOK_KEEP, HOOK_STUN, pull);
        }
        Item::Boost => return,
    }
    race.events.push(Event::Body {
        who,
        what: Happening::Knocked,
    });
}

// ---- drawing ----

const WOOD: render::Rgb = rgb(178, 114, 54);
const PLANK: render::Rgb = rgb(112, 64, 30);
const MARK: render::Rgb = rgb(255, 214, 64);
const BANANA: render::Rgb = rgb(250, 214, 50);
const TIP: render::Rgb = rgb(92, 62, 30);
const ROPE: render::Rgb = rgb(150, 112, 62);
const METAL: render::Rgb = rgb(170, 176, 186);

/// Crates (not while they respawn), flying Bananas and Hooks with their
/// ropes.
pub fn draw(b: &mut Batch, race: &Race) {
    let tick = race.tick as i32;
    for (k, at) in race.course.crates.iter().enumerate() {
        if race.crates.get(k).copied().unwrap_or(0) == 0 {
            draw_crate(b, *at, tick, k as i32);
        }
    }
    for s in &race.shots {
        match s.item {
            Item::Banana => draw_banana(b, s.pos, s.vel, tick),
            Item::Hook => {
                let from = race.racers.get(s.by).map(|r| r.ape.pos).unwrap_or(s.pos);
                draw_hook(b, from, s.pos, s.vel);
            }
            Item::Stick | Item::Boost => {}
        }
    }
}

fn draw_crate(b: &mut Batch, at: V3, tick: i32, k: i32) {
    let bob = mul(fx::sin(tick * 300 + k * 7_919), cm(25));
    let yaw = tick * 180 + k * 5_003;
    let c = at.add(v3(0, bob, 0));
    let size = cm(130);
    let frame = Affine::translate(c).then(&Affine::rot_y(yaw));
    let glow = ONE / 5;
    b.push(
        Mesh::Cube,
        &placed(c, yaw, v3(size, size, size)),
        WOOD,
        glow,
    );
    // dark planks round the top and bottom edges
    for y in [size / 2 - cm(8), cm(8) - size / 2] {
        b.push(
            Mesh::Cube,
            &placed(
                c.add(v3(0, y, 0)),
                yaw,
                v3(size + cm(6), cm(16), size + cm(6)),
            ),
            PLANK,
            glow,
        );
    }
    // a glowing "?" on each side face
    let half = size / 2 + cm(4);
    for face in 0..4 {
        let f = frame.then(&Affine::rot_y(face * fx::QUARTER));
        let at = |x: i32, y: i32| f.apply(v3(x, y, half));
        let bar = |b: &mut Batch, p: V3, q: V3| b.push(Mesh::Cube, &rod(p, q, cm(10)), MARK, ONE);
        // the hook of the "?": up the left, over the top, down the right
        bar(b, at(-cm(22), cm(22)), at(-cm(22), cm(38)));
        bar(b, at(-cm(22), cm(38)), at(cm(22), cm(38)));
        bar(b, at(cm(22), cm(38)), at(cm(22), cm(14)));
        bar(b, at(cm(22), cm(14)), at(0, cm(6)));
        bar(b, at(0, cm(6)), at(0, -cm(14)));
        // the dot
        b.push(
            Mesh::Cube,
            &placed(
                at(0, -cm(34)),
                yaw + face * fx::QUARTER,
                v3(cm(12), cm(12), cm(4)),
            ),
            MARK,
            ONE,
        );
    }
}

fn draw_banana(b: &mut Batch, pos: V3, vel: V3, tick: i32) {
    // a curved banana: rods along an arc, turning end over end as it flies
    let frame = Affine::translate(pos)
        .then(&Affine::rot_y(yaw_of(vel)))
        .then(&Affine::rot_x(tick * 1_200));
    let r = cm(35);
    let arc = |deg: i32| {
        let a = fx::deg(deg);
        frame.apply(v3(0, mul(fx::cos(a), r) - r / 2, mul(fx::sin(a), r)))
    };
    let points = [-60, -30, 0, 30, 60].map(arc);
    for w in points.windows(2) {
        b.push(Mesh::Cyl, &rod(w[0], w[1], cm(14)), BANANA, ONE / 4);
    }
    for p in &points[1..4] {
        b.push(
            Mesh::Sphere,
            &placed(*p, 0, v3(cm(14), cm(14), cm(14))),
            BANANA,
            ONE / 4,
        );
    }
    for (end, next) in [(points[0], points[1]), (points[4], points[3])] {
        let tip = end.add(end.sub(next).with_len(cm(7)));
        b.push(Mesh::Cyl, &rod(end, tip, cm(7)), TIP, 0);
    }
}

fn draw_hook(b: &mut Batch, from: V3, at: V3, vel: V3) {
    b.push(Mesh::Cyl, &rod(from, at, cm(5)), ROPE, 0);
    let dir = if vel == V3::ZERO { at.sub(from) } else { vel };
    let f = Affine::translate(at).then(&Affine::looking(dir, V3::UP));
    let p = |x: i32, y: i32, z: i32| f.apply(v3(x, y, z));
    // the shank, a point, and a barb curling back
    b.push(
        Mesh::Cyl,
        &rod(p(0, 0, -cm(30)), p(0, 0, cm(10)), cm(8)),
        METAL,
        ONE / 6,
    );
    b.push(
        Mesh::Cone,
        &rod(p(0, 0, cm(10)), p(0, 0, cm(30)), cm(14)),
        METAL,
        ONE / 6,
    );
    b.push(
        Mesh::Cyl,
        &rod(p(0, 0, -cm(30)), p(0, cm(18), -cm(20)), cm(6)),
        METAL,
        ONE / 6,
    );
    b.push(
        Mesh::Cone,
        &rod(p(0, cm(18), -cm(20)), p(0, cm(22), -cm(4)), cm(9)),
        METAL,
        ONE / 6,
    );
}

const fn cm(n: i32) -> i32 {
    fx::cm(n)
}

#[cfg(test)]
mod tests;
