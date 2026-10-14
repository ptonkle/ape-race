//! Items (PLAN P9): crates on vines, pickups, the Banana, Stick, Hook, Speed
//! Boost and Vine, and their HUD icons.
//!
//! Who decides a hit: the browser that runs the user. Apes driven here
//! (Local, Ai) are changed here; a Remote ape is moved by its own browser,
//! so a hit on it only emits `Event::Hit`, which the online layer forwards,
//! and the victim's browser calls [`apply_hit`]. A shot thrown by a Remote
//! ape (if the online layer replays one with `use_item`) is drawn but decides
//! nothing here: its owner's browser decides its hits.

use crate::ape::{Controls, Happening, Mode};
use crate::course::Course;
use crate::fx::{self, int, mul, ONE};
use crate::geom::{facing, v3, yaw_of, Affine, V3};
use crate::mesh::Mesh;
use crate::race::{Driver, Event, Item, Race, Shot, Summon};
use crate::render::{self, placed, rgb, rod, Batch};
use crate::rules::*;
use serde::{Deserialize, Serialize};

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
            let aim = facing(c.yaw, c.pitch);
            let dir = facing(c.yaw, c.pitch + BANANA_LOB);
            let target = banana_target(race, who, aim);
            race.shots.push(Shot {
                item,
                by: who,
                pos: pos.add(dir.scale(BODY_RADIUS)),
                vel: vel.add(dir.scale(BANANA_SPEED)),
                life: BANANA_LIFE,
                target,
            });
        }
        Item::Stick => {
            if owned {
                for i in stick_targets(race, who) {
                    strike(race, i, who, Item::Stick);
                }
            }
        }
        Item::Vine => {
            if owned {
                summon(race, who, c);
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
        if i == who || r.out() || r.progress <= me.progress {
            continue;
        }
        let d = r.ape.middle().dist(at);
        if d <= HOOK_RANGE && best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, i));
        }
    }
    best.map(|(_, i)| i)
}

/// The Banana's target (TRIAGE T10): the nearest other ape still racing
/// within `BANANA_CONE` of the aim and `BANANA_SEEK` of the thrower.
pub fn banana_target(race: &Race, who: usize, aim: V3) -> Option<usize> {
    let at = race.racers.get(who)?.ape.middle();
    let aim = aim.norm();
    let wide = fx::cos(BANANA_CONE);
    let mut best: Option<(i32, usize)> = None;
    for (i, r) in race.racers.iter().enumerate() {
        if i == who || r.out() {
            continue;
        }
        let to = r.ape.middle().sub(at);
        let d = to.len();
        if d > BANANA_SEEK || d == 0 || to.norm().dot(aim) < wide {
            continue;
        }
        if best.is_none_or(|(bd, _)| d < bd) {
            best = Some((d, i));
        }
    }
    best.map(|(_, i)| i)
}

/// The apes a Stick swung by `who` now would hit (TRIAGE T11): every other
/// ape within `STICK_RANGE`. The drawing lights them up while `who` holds a
/// Stick.
pub fn stick_targets(race: &Race, who: usize) -> Vec<usize> {
    let Some(me) = race.racers.get(who) else {
        return Vec::new();
    };
    let at = me.ape.middle();
    (0..race.racers.len())
        .filter(|&i| i != who && race.racers[i].ape.middle().dist(at) <= STICK_RANGE)
        .collect()
}

/// The Vine item (TRIAGE T12): a vine ahead of the user along its motion
/// (where the camera looks, at rest), its anchor high above the hands and
/// its end below where the hands will be when they get there.
fn summon(race: &mut Race, who: usize, c: &Controls) {
    let ape = &race.racers[who].ape;
    let flat = ape.vel.flat();
    let speed = flat.len();
    let dir = if speed > ONE {
        flat.norm()
    } else {
        facing(c.yaw, 0)
    };
    let ahead = mul(speed, SUMMON_LEAD).clamp(SUMMON_AHEAD_MIN, SUMMON_AHEAD_MAX);
    // seconds until the hands get there (at most one), and their height then
    let t = if speed > ONE {
        fx::div(ahead, speed).min(ONE)
    } else {
        0
    };
    let then = ape.pos.y + mul(ape.vel.y, t) - mul(race.course.gravity(), mul(t, t)) / 2;
    let spot = ape.pos.add(dir.scale(ahead));
    let floor = race.course.support(spot);
    let bottom = (ape.pos.y.min(then) - SUMMON_BELOW).max(floor + ONE);
    let top = (ape.pos.y.max(bottom) + SUMMON_RISE).min(floor + CEILING - int(2));
    let length = (top - bottom).max(int(4));
    let serial = race.summon_count.get(who).copied().unwrap_or(0);
    let slot = race.summon_slot(who, serial);
    race.place_vine(Summon {
        slot,
        by: who,
        serial,
        anchor: v3(spot.x, top, spot.z),
        length,
        life: SUMMON_LIFE,
    });
}

/// Summoned vines age; one whose time is up goes once no ape holds it.
fn expire(race: &mut Race) {
    let mut gone = Vec::new();
    for k in 0..race.summons.len() {
        let s = &mut race.summons[k];
        s.life = (s.life - 1).max(0);
        let slot = s.slot;
        let held = race.racers.iter().any(|r| r.ape.swinging() == Some(slot));
        if race.summons[k].life == 0 && !held {
            gone.push(slot);
        }
    }
    for slot in gone {
        race.bury_vine(slot);
    }
}

/// The items a race's crates may give (TRIAGE T42): one bit per
/// [`Item`], in the order of `Item::ALL`. Kept in the settings, sent with an
/// online race's start and kept in a recording, as a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemSet(pub u8);

impl Default for ItemSet {
    fn default() -> ItemSet {
        ItemSet::ALL
    }
}

impl ItemSet {
    pub const ALL: ItemSet = ItemSet((1 << Item::ALL.len()) - 1);
    pub const NONE: ItemSet = ItemSet(0);

    fn bit(item: Item) -> u8 {
        1 << (item as u8)
    }

    /// Just this item (a Time Trial: the Speed Boost, TRIAGE T13).
    pub fn only(item: Item) -> ItemSet {
        ItemSet(ItemSet::bit(item))
    }

    pub fn has(self, item: Item) -> bool {
        self.0 & ItemSet::bit(item) != 0
    }

    /// This set with `item` added (`on`) or taken out.
    pub fn with(self, item: Item, on: bool) -> ItemSet {
        if on {
            ItemSet(self.0 | ItemSet::bit(item))
        } else {
            ItemSet(self.0 & !ItemSet::bit(item))
        }
    }

    /// Only bits that name an item (a stored or received set may have more).
    pub fn clean(self) -> ItemSet {
        ItemSet(self.0 & ItemSet::ALL.0)
    }

    /// No item at all: crates give nothing and are not drawn.
    pub fn is_empty(self) -> bool {
        self.clean().0 == 0
    }

    pub fn items(self) -> Vec<Item> {
        Item::ALL.into_iter().filter(|i| self.has(*i)).collect()
    }
}

/// The item a crate gives ape `who`: one of `race.items` (TRIAGE T42; a
/// Time Trial allows only the Boost, T13), drawn by [`roll_among`] by its
/// place and whether it is low (on the ground or near it). `None` when no
/// item is allowed.
pub fn roll_for(race: &mut Race, who: usize) -> Option<Item> {
    let allowed = race.items.clean();
    let n = race.racers.len();
    let r = &race.racers[who];
    let low =
        r.ape.mode == Mode::Grounded || r.ape.pos.y - race.course.support(r.ape.pos) < SUMMON_LOW;
    let place = r.place;
    roll_among(&mut race.dice, place, n, low, allowed)
}

/// [`roll_among`] with every item allowed.
pub fn roll(dice: &mut fx::Dice, place: usize, n: usize, low: bool) -> Item {
    roll_among(dice, place, n, low, ItemSet::ALL).unwrap_or(Item::Boost)
}

/// A random item of `allowed` for an ape in `place` of `n`, Mario Kart
/// style: the leaders get more Bananas and Sticks, those behind more Hooks,
/// Boosts and Vines; an ape that is `low` (on or near the ground) gets many
/// more Vines. Items left out are not drawn: the others keep their weights
/// (TRIAGE T42). One allowed item comes out without a throw of the dice
/// (as the Time Trial's Boost always did), none gives `None`.
pub fn roll_among(
    dice: &mut fx::Dice,
    place: usize,
    n: usize,
    low: bool,
    allowed: ItemSet,
) -> Option<Item> {
    let left = allowed.items();
    if left.len() <= 1 {
        return left.first().copied();
    }
    let behind = if n > 1 {
        ((place.clamp(1, n) - 1) * 100 / (n - 1)) as i32
    } else {
        0
    };
    let weights: Vec<(Item, i32)> = [
        (Item::Banana, 40 - behind / 4),
        (Item::Stick, 30 - behind / 5),
        (Item::Hook, 15 + behind / 4),
        (Item::Boost, 15 + behind / 5),
        (Item::Vine, 4 + behind / 10 + if low { 30 } else { 0 }),
    ]
    .into_iter()
    .filter(|(i, _)| allowed.has(*i))
    .collect();
    let total: i32 = weights.iter().map(|w| w.1).sum();
    let mut pick = dice.below(total as u32) as i32;
    for (item, w) in weights.iter().copied() {
        if pick < w {
            return Some(item);
        }
        pick -= w;
    }
    left.last().copied()
}

/// Crates, pickups, thrown things and summoned vines, once a tick after the
/// apes move.
pub fn step(race: &mut Race) {
    pickups(race);
    fly(race);
    expire(race);
}

/// The vines that carry a crate (TRIAGE T9): for each of the course's crate
/// spots, the nearest vine within `CRATE_VINE_NEAR` not already carrying
/// one; a course with no crate spots gets one on every
/// `CRATE_VINE_EVERY`-th vine.
pub fn crate_vines(course: &Course) -> Vec<usize> {
    if course.crates.is_empty() {
        return (0..course.vines.len()).step_by(CRATE_VINE_EVERY).collect();
    }
    let mut out: Vec<usize> = Vec::new();
    for spot in &course.crates {
        let best = course
            .vines
            .iter()
            .enumerate()
            .filter(|(i, _)| !out.contains(i))
            .map(|(i, v)| (v.anchor.flat().sub(spot.flat()).len(), i))
            .filter(|(d, _)| *d <= CRATE_VINE_NEAR)
            .min();
        if let Some((_, i)) = best {
            out.push(i);
        }
    }
    out
}

/// Where crate `k` hangs: just under the end of its vine (toward the hands
/// of an ape swinging on it).
pub fn crate_at(race: &Race, k: usize) -> V3 {
    let v = race.crate_vines[k];
    let a = race.course.anchor(v, race.tick);
    let length = race.course.vines[v].length;
    let down = race
        .racers
        .iter()
        .find(|r| r.ape.swinging() == Some(v))
        .map(|r| r.ape.pos.sub(a))
        .filter(|d| d.len() > ONE)
        .map_or(race.course.hang(v), |d| d.norm());
    a.add(down.scale(length + CRATE_HANG))
}

/// From a vine's end to the middle of the crate on it.
const CRATE_HANG: i32 = cm(90);

fn pickups(race: &mut Race) {
    // no item allowed (TRIAGE T42): the crates are not there
    if race.items.is_empty() {
        return;
    }
    let n = race.racers.len();
    for k in 0..race.crates.len() {
        if race.crates[k] > 0 {
            race.crates[k] -= 1;
            if race.crates[k] > 0 {
                continue;
            }
        }
        let vine = race.crate_vines[k];
        let at = crate_at(race, k);
        for i in 0..n {
            let r = &race.racers[i];
            // a Remote ape picks up in its own browser
            if r.item.is_some() || r.out() || r.driver == Driver::Remote {
                continue;
            }
            // catching the crate's vine takes it, and so does flying into it
            let caught = r.ape.swinging() == Some(vine);
            if caught || r.ape.middle().dist(at) <= CRATE_RADIUS + BODY_RADIUS {
                let Some(item) = roll_for(race, i) else {
                    break;
                };
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
                let target = s
                    .target
                    .filter(|t| race.racers.get(*t).is_some_and(|r| !r.out()));
                match target {
                    // homing (TRIAGE T10): turn toward the target at most
                    // `BANANA_TURN`, at least `BANANA_CHASE` faster than it
                    Some(t) => {
                        let ape = &race.racers[t].ape;
                        let speed = s.vel.len().max(ape.speed() + BANANA_CHASE);
                        let to = ape.middle().sub(s.pos);
                        s.vel = turn_toward(s.vel, to, mul(BANANA_TURN, DT)).scale(speed);
                    }
                    None => s.vel.y -= mul(race.course.gravity(), DT),
                }
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
            Item::Stick | Item::Boost | Item::Vine => true,
        };
        if !done && s.life > 0 {
            kept.push(s);
        }
    }
    race.shots = kept;
}

/// The unit direction `from` turned toward `to` by at most `max` (an
/// angle).
pub fn turn_toward(from: V3, to: V3, max: i32) -> V3 {
    if to == V3::ZERO {
        return from.norm();
    }
    let w = to.norm();
    if from == V3::ZERO {
        return w;
    }
    let u = from.norm();
    let cos = u.dot(w);
    if cos >= fx::cos(max) {
        return w;
    }
    let mut side = w.sub(u.scale(cos));
    if side.len() < ONE / 100 {
        // straight behind: turn either way, here over the top
        side = V3::UP.reject(u);
        if side.len() < ONE / 100 {
            side = v3(ONE, 0, 0);
        }
    }
    u.scale(fx::cos(max))
        .add(side.norm().scale(fx::sin(max)))
        .norm()
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
    if race.racers[who].eaten.is_some() {
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
        Item::Boost | Item::Vine => return,
    }
    race.events.push(Event::Body {
        who,
        what: Happening::Knocked,
    });
}

// ---- drawing ----

const BANANA: render::Rgb = rgb(250, 214, 50);
const TIP: render::Rgb = rgb(92, 62, 30);
const ROPE: render::Rgb = rgb(150, 112, 62);
const METAL: render::Rgb = rgb(170, 176, 186);

const LEAFY: render::Rgb = rgb(90, 180, 70);
const REACH: render::Rgb = rgb(255, 120, 60);

/// Flying Bananas and Hooks with their ropes (an item vine's flowers are
/// drawn with the vines, `scene::item_flowers`, TRIAGE T46), the knot of leaves a summoned vine hangs from, and the
/// Stick's reach for the player (this browser's `Driver::Local` ape) holding
/// one: a ring round it, brighter with an ape inside, and a marker over each
/// ape in reach.
pub fn draw(b: &mut Batch, race: &Race) {
    let tick = race.tick as i32;
    for s in &race.summons {
        if let Some(v) = race.course.vines.get(s.slot) {
            b.push(
                Mesh::Blob,
                &placed(v.anchor, tick * 60, v3(cm(180), cm(90), cm(180))),
                LEAFY,
                ONE / 2,
            );
        }
    }
    for s in &race.shots {
        match s.item {
            Item::Banana => draw_banana(b, s.pos, s.vel, tick),
            Item::Hook => {
                let from = race.racers.get(s.by).map(|r| r.ape.pos).unwrap_or(s.pos);
                draw_hook(b, from, s.pos, s.vel);
            }
            Item::Stick | Item::Boost | Item::Vine => {}
        }
    }
    for (i, r) in race.racers.iter().enumerate() {
        if r.driver == Driver::Local && r.item == Some(Item::Stick) {
            draw_stick_reach(b, race, i, tick);
        }
    }
}

/// The Stick's reach round ape `who`, and a marker over each ape in it.
fn draw_stick_reach(b: &mut Batch, race: &Race, who: usize, tick: i32) {
    let centre = race.racers[who].ape.middle();
    let targets = stick_targets(race, who);
    let glow = if targets.is_empty() { ONE / 4 } else { ONE };
    let parts = 32;
    let point = |k: i32| {
        let a = k * fx::TURN / parts + tick * 40;
        centre.add(v3(
            mul(fx::cos(a), STICK_RANGE),
            0,
            mul(fx::sin(a), STICK_RANGE),
        ))
    };
    // a dashed ring: every other piece
    for k in (0..parts).step_by(2) {
        b.push(Mesh::Cyl, &rod(point(k), point(k + 1), cm(9)), REACH, glow);
    }
    for t in targets {
        let head = race.racers[t].ape.pos.add(v3(0, cm(120), 0));
        let bob = mul(fx::sin(tick * 500), cm(15));
        b.push(
            Mesh::Cone,
            &rod(
                head.add(v3(0, cm(70) + bob, 0)),
                head.add(v3(0, bob, 0)),
                cm(45),
            ),
            REACH,
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

// ---- icons (TRIAGE T19) ----

/// The item's icon for the HUD's item slot: inline SVG markup, flat
/// geometric shapes on a 64 × 64 view box, no background (so it sits on the
/// HUD's own). The page sets it as the slot's `innerHTML`, or as an `<img>`
/// via a `data:image/svg+xml,` URL.
pub fn icon_svg(item: Item) -> String {
    let body = match item {
        Item::Banana => concat!(
            r##"<path d="M16 10 L24 10 L25 22 Q30 40 50 46 L58 46 Q56 56 42 56 Q14 52 14 22 Z" "##,
            r##"fill="#fad632" stroke="#5c3e1e" stroke-width="3" stroke-linejoin="round"/>"##,
            r##"<path d="M20 24 Q22 42 40 50" fill="none" stroke="#e0b020" stroke-width="3" stroke-linecap="round"/>"##,
            r##"<rect x="15" y="5" width="10" height="7" rx="2" fill="#5c3e1e"/>"##,
        ),
        Item::Stick => concat!(
            r##"<polygon points="12,58 6,52 46,12 52,18" fill="#b27236" stroke="#5c3e1e" stroke-width="3" stroke-linejoin="round"/>"##,
            r##"<polygon points="46,12 52,6 58,12 52,18" fill="#8a5428" stroke="#5c3e1e" stroke-width="3" stroke-linejoin="round"/>"##,
            r##"<path d="M30 26 L36 20" stroke="#5c3e1e" stroke-width="3" stroke-linecap="round"/>"##,
            r##"<path d="M34 6 L38 2 M58 30 L62 26 M44 4 L44 0 M60 20 L64 20" stroke="#ffd640" stroke-width="4" stroke-linecap="round"/>"##,
        ),
        Item::Hook => concat!(
            r##"<path d="M40 4 L40 40 Q40 56 26 56 Q12 56 12 42 L12 34" fill="none" stroke="#aab0ba" stroke-width="7" stroke-linejoin="round"/>"##,
            r##"<polygon points="4,36 12,22 20,36" fill="#aab0ba"/>"##,
            r##"<rect x="34" y="2" width="12" height="8" rx="2" fill="#96703e"/>"##,
        ),
        Item::Boost => concat!(
            r##"<polygon points="38,2 10,36 28,36 22,62 54,24 36,24" fill="#ff8c1a" stroke="#7a3a00" stroke-width="3" stroke-linejoin="round"/>"##,
            r##"<polygon points="34,12 20,30 32,30" fill="#ffd640"/>"##,
        ),
        Item::Vine => concat!(
            r##"<rect x="6" y="2" width="36" height="8" rx="4" fill="#6a4a26"/>"##,
            r##"<path d="M24 8 Q34 20 22 30 Q12 40 24 50 L26 60" fill="none" stroke="#3c8c32" stroke-width="6" stroke-linecap="round"/>"##,
            r##"<polygon points="28,18 40,14 34,24" fill="#5ab446"/>"##,
            r##"<polygon points="18,34 6,32 14,42" fill="#5ab446"/>"##,
            r##"<path d="M50 34 L50 54 M40 44 L60 44" stroke="#ffd640" stroke-width="6" stroke-linecap="round"/>"##,
        ),
    };
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64">{body}</svg>"#
    )
}

const fn cm(n: i32) -> i32 {
    fx::cm(n)
}

#[cfg(test)]
mod tests;
