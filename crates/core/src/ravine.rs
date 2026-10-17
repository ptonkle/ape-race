//! Rattlesnake Ravine (2026-10-17, `briefs/rattlesnake-ravine.md`): a Utah
//! slot canyon at night, ending on a barren mesa. Pitfalls every agent
//! checks: `notes/ravine-pitfalls.md`.
//!
//! **The light:** night under a starry sky, soft; torches, firefly swarms
//! and glowing mushrooms (`Glow`) lead toward the finish.
//!
//! The types below are the contract between the course (placement,
//! physics, the AI) and the scene (drawing): the course fills a
//! `RavineLayout` on `Course.ravine` (`None` on every other map); the scene
//! draws from it. **One set of numbers gives both the solids and the
//! drawing** (MAPS "Common gotchas" 1–2).
//!
//! **What is solid (the person, 2026-10-17):** only things big enough to be
//! obstacles: the canyon walls (`Wall`), arch and cave and tunnel roofs
//! (`Room`), the wall holes the mine tracks come out of (`Hole`), the T-rex
//! skull and its teeth (`Skull`), the sphinx (`Finish.sphinx`) and large
//! rocks (`Rock`). Dressing is never solid: vines, ropes, torches,
//! mushrooms, fireflies, mining props, the bones and fossils set in the
//! walls, the tents and the campfire. Bridge and log decks (`Deck`) are
//! floors you land, stand and walk on, not walls.
//!
//! **The agreed rules** (written in `rules.rs` and `solids.rs`):
//! - **Snake vines** (`Hanger.snake`, about 15% of the vines, from a fixed
//!   list, more on the upper passage): grabbing one snaps it in half; the
//!   ape drops with its momentum killed (velocity zeroed, falling). The
//!   snap is shared and deterministic; when it happened is
//!   `Hanger.snapped_at`, kept in the save points (`snake_state` /
//!   `put_snake_state`, already wired into `replay::SavePoint`). A snapped
//!   vine hangs whole again `SNAKE_REGROW` (10 s) after (SECOND-ORDER RR1).
//! - **The T-rex teeth** (`Skull::brushes`): brushing a tooth puts the ape
//!   on the canyon floor right under the jaw (`Skull.floor_spot`) with zero
//!   speed; it carries on the long way round. With Floor is Lava it is the
//!   kill. The reset is instant, so nothing about it lasts in the save
//!   points.
//! - **Lizards** (`Lizard`, `RavineLayout::lizard_at`) are moving bouncers
//!   walking the walls, as Sunset Ruins' turtles; **mine carts** (`Cart`,
//!   `RavineLayout::cart_at`) are moving bouncers on their tracks, as the
//!   City's cars. Both bouncers already move with them
//!   (`Course::bouncer`, `Course::bouncer_under`).
//!
//! Everything is a pure function of the tick and the layout: nothing is
//! random at runtime.

pub mod layout;
pub mod lobby;
mod rules;
#[cfg(test)]
mod rules_tests;
pub mod solids;
#[cfg(test)]
mod tests;

pub use rules::{SNAKE_NO_GRAB, SNAKE_REGROW};
pub use solids::{Solid, SolidKind, Solids, SolidsCache, ROOF_THICK};

pub use layout::{ravine, ravine_seeded, ravine_upper_seeded};

use crate::course::{Course, Mushroom};
use crate::fx::{self, cm, ONE};
use crate::geom::{v3, V3};
use crate::rules::{BODY_DROP, BODY_RADIUS};

/// The tallest ape as drawn in a race (the gorilla at `scene::RACE_SCALE`),
/// standing. ASSUMPTION (SECOND-ORDER RV3): an estimate from
/// `ruins::GORILLA_WIDTH` (2.86 m wide); the art agent measures the model
/// and changes this one number.
pub const TALLEST_APE: i32 = cm(430);

/// The gap between the T-rex's upper and lower teeth: twice the tallest
/// ape (the brief).
pub const MOUTH_GAP: i32 = 2 * TALLEST_APE;

/// Which side of the way on a wall's rock lies, looking along its points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// One point of a canyon wall's foot line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallPoint {
    /// the wall's face at its foot (on the canyon floor)
    pub at: V3,
    /// how high its rim stands above `at.y`
    pub height: i32,
}

/// A canyon wall: a polyline of its face's foot, rock on `side`. Solid from
/// the face back `depth` and from the foot to the rim (and the solids reach
/// past what is drawn). Braided stretches add walls round the islands
/// between the strands. The drawing and the solids both come from this.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wall {
    pub points: Vec<WallPoint>,
    pub side: Side,
    /// how thick the rock is behind the face
    pub depth: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomKind {
    /// an archway: a short roof spanning the canyon
    Arch,
    /// a cave
    Cave,
    /// the lower passage's mine tunnel (old mining gear, fossil and gold
    /// digs as dressing)
    Tunnel,
    /// the inside of the T-rex skull (the camera's keep-inside volume for
    /// it)
    Skull,
}

/// A roofed stretch (arch, cave, tunnel): a box along its floor's
/// centreline from `from` to `to`, `half_width` either side, `height` from
/// the floor to the roof's underside. The roof is solid (and thick past
/// what is drawn); its mouths are left empty (no filler block). The same
/// box is the chase camera's keep-inside volume while the ape is in it
/// (MAPS gotcha 5, `Temple::kept_in` style): `Room::contains`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Room {
    pub kind: RoomKind,
    pub from: V3,
    pub to: V3,
    pub half_width: i32,
    pub height: i32,
}

impl Room {
    /// Whether `p` is inside the room's volume (between its mouths, its
    /// side walls, its floor and its roof).
    pub fn contains(&self, p: V3) -> bool {
        let run = self.to.sub(self.from).flat();
        let len = run.len();
        if len <= 0 {
            return false;
        }
        let along = run.norm();
        let d = p.sub(self.from).flat();
        let a = d.dot(along);
        let side = d.dot(v3(-along.z, 0, along.x)).abs();
        let floor = self.from.y + fx::mul(self.to.y - self.from.y, fx::div(a.clamp(0, len), len));
        (0..=len).contains(&a)
            && side <= self.half_width
            && p.y >= floor
            && p.y <= floor + self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeckKind {
    /// a wooden rope bridge
    Bridge,
    /// a fallen log across the canyon
    Log,
}

/// A bridge's or a log's deck: its top surface's middle line from `from`
/// to `to`, `half_width` either side. A real floor: an ape lands, stands,
/// walks and jumps on it (MAPS gotcha 3). Its ropes are dressing (not
/// solid, and the camera doesn't dodge them).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Deck {
    pub kind: DeckKind,
    pub from: V3,
    pub to: V3,
    pub half_width: i32,
}

impl Deck {
    /// The deck's top under (`p.x`, `p.z`), if `p` is over it.
    pub fn top_at(&self, p: V3) -> Option<i32> {
        // (cheap first: outside the flat box round its ends, widened by its
        // half width and a metre for rounding, it cannot be over it; the
        // course asks every deck many times a tick)
        let r = self.half_width + fx::ONE;
        if p.x < self.from.x.min(self.to.x) - r
            || p.x > self.from.x.max(self.to.x) + r
            || p.z < self.from.z.min(self.to.z) - r
            || p.z > self.from.z.max(self.to.z) + r
        {
            return None;
        }
        let run = self.to.sub(self.from).flat();
        let len = run.len();
        if len <= 0 {
            return None;
        }
        let along = run.norm();
        let d = p.sub(self.from).flat();
        let a = d.dot(along);
        let side = d.dot(v3(-along.z, 0, along.x)).abs();
        ((0..=len).contains(&a) && side <= self.half_width)
            .then(|| self.from.y + fx::mul(self.to.y - self.from.y, fx::div(a, len)))
    }
}

/// What a vine hangs from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HangFrom {
    /// a bridge's or a log's deck (`RavineLayout.decks[deck]`)
    Deck { deck: usize },
    /// a ledge or branch on a canyon wall (`RavineLayout.walls[wall]`)
    Ledge { wall: usize },
    /// an arch's, cave's or the tunnel's roof (`RavineLayout.rooms[room]`)
    Roof { room: usize },
}

/// One per vine (`RavineLayout.hangers[i]` carries `Course.vines[i]`): what
/// it hangs from, where it is tied (`at`, at least 4 m clear of any top,
/// MAPS gotcha 4), and whether it is a snake vine. Snakes hang from the
/// same things ordinary vines do; each is drawn with a clear head, scales
/// and a slow sway so a sharp player sees it coming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hanger {
    pub from: HangFrom,
    pub at: V3,
    pub snake: bool,
    /// a snake vine: the race tick it snapped on (a race's state, kept in
    /// the save points); `None` while whole, and always `None` on an
    /// ordinary vine
    pub snapped_at: Option<u32>,
}

/// A large, colourful lizard walking up and down a wall: from `from` to
/// `to` and back over `period` ticks, `phase` ticks in. `out` is the wall's
/// outward normal at its path (flat, unit). Its back is bouncer `bouncer`
/// (an index in `Course.mushrooms`), which moves with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lizard {
    pub from: V3,
    pub to: V3,
    pub out: V3,
    pub period: u32,
    pub phase: u32,
    pub bouncer: usize,
    pub colour: LizardColour,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LizardColour {
    Red,
    Yellow,
    Blue,
}

/// A dark hole in a canyon wall a mine track comes out of: solid (a box
/// round the hole's mouth reaching into the rock), so apes can't fly in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hole {
    /// the middle of the hole's mouth, at the rail's height
    pub at: V3,
    /// out of the wall (flat, unit)
    pub out: V3,
    pub half_width: i32,
    pub height: i32,
}

/// A stretch of rickety mine-cart track: the rails' top line from `start`
/// (its first point, at the `start` hole) to its last (at `end`), changing
/// height or crossing the canyon as it likes. The rails and trestles are
/// dressing; the holes are solid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub points: Vec<V3>,
    pub start: Hole,
    pub end: Hole,
}

/// A mine cart running along track `track` from its start hole to its end
/// hole over `period` ticks, `phase` ticks in, then round again (it goes
/// into the end hole and the next comes out of the start). Its bouncer
/// (`Course.mushrooms[bouncer]`, set where the cart is at tick 0) moves with
/// it, the same as the City's cars.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cart {
    pub track: usize,
    pub period: u32,
    pub phase: u32,
    pub bouncer: usize,
}

/// One tooth of the T-rex: a cone from `root` (in the jaw) to `tip`,
/// `radius` at the root. Solids and drawing both come from this list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tooth {
    pub root: V3,
    pub tip: V3,
    pub radius: i32,
}

/// The T-rex fossil skull set halfway up a cave wall: the optional
/// shortcut through its mouth (saving 5–8 s at most; the AI need not use
/// it). `at` is the middle of the mouth's gap; `yaw` the way through. The
/// skull's bone is solid round the mouth; the gap between the upper and
/// lower teeth is `MOUTH_GAP`. `keep` is the camera's keep-inside volume
/// through the mouth. `floor_spot` / `floor_yaw` are where a brushed ape is
/// put: the canyon floor right under the jaw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skull {
    pub at: V3,
    pub yaw: i32,
    pub teeth: Vec<Tooth>,
    pub keep: Room,
    pub floor_spot: V3,
    pub floor_yaw: i32,
}

impl Skull {
    /// Whether a body at `p` (the hands; the body runs from there down
    /// `BODY_DROP` with radius `BODY_RADIUS`) touches any tooth.
    pub fn brushes(&self, p: V3) -> bool {
        let feet = p.sub(v3(0, BODY_DROP, 0));
        self.teeth.iter().any(|t| {
            let reach = t.radius + BODY_RADIUS;
            [p, feet, p.lerp(feet, ONE / 2)]
                .iter()
                .any(|&q| segment_distance(q, t.root, t.tip) <= reach)
        })
    }
}

/// How far `p` is from the segment `a`–`b`.
fn segment_distance(p: V3, a: V3, b: V3) -> i32 {
    let ab = b.sub(a);
    let len2 = ab.dot64(ab);
    if len2 == 0 {
        return p.sub(a).len();
    }
    let t = (p.sub(a).dot64(ab) as i128 * ONE as i128 / len2 as i128).clamp(0, ONE as i128) as i32;
    p.sub(a.lerp(b, t)).len()
}

/// Where the route splits into the upper and lower passages and rejoins:
/// indices into `Course.path`. The lower passage (through the mine tunnel)
/// is the spine, `Course.path` itself, and progress is measured along it;
/// the upper passage (shorter, sparse vines, more snakes) is `upper`, its
/// own centreline from `path[fork]` to `path[rejoin]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Split {
    pub fork: usize,
    pub rejoin: usize,
    pub upper: Vec<V3>,
}

/// A large rock on the canyon floor: solid (a cylinder from its foot).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rock {
    pub at: V3,
    pub radius: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlowKind {
    Torch,
    Fireflies,
    Mushroom,
    Lantern,
}

/// A light source leading toward the finish (dressing, not solid).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glow {
    pub at: V3,
    pub kind: GlowKind,
}

/// The mesa finish's set: the sphinx with a proboscis monkey's head (solid;
/// `sphinx` is its front feet's middle on the ground, `sphinx_yaw` the way
/// it faces), the campfire with bananas roasting on sticks pushed into the
/// dirt (`sticks`: each stick's foot), and tents (foot, yaw). Only the
/// sphinx is solid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finish {
    pub sphinx: V3,
    pub sphinx_yaw: i32,
    pub campfire: V3,
    pub sticks: Vec<V3>,
    pub tents: Vec<(V3, i32)>,
}

/// Rattlesnake Ravine's layout: everything the course fills and the scene
/// draws (see the module's notes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RavineLayout {
    pub walls: Vec<Wall>,
    /// arches, caves and the mine tunnel
    pub rooms: Vec<Room>,
    /// bridges and logs
    pub decks: Vec<Deck>,
    /// one per vine, in `Course.vines` order
    pub hangers: Vec<Hanger>,
    pub lizards: Vec<Lizard>,
    pub tracks: Vec<Track>,
    pub carts: Vec<Cart>,
    pub skull: Option<Skull>,
    pub split: Option<Split>,
    pub rocks: Vec<Rock>,
    pub glows: Vec<Glow>,
    pub finish: Finish,
    /// the lobby spot (where the ape stands behind the menus) and its yaw
    pub lobby: (V3, i32),
}

impl RavineLayout {
    /// Lizard `i` at race tick `tick`: where its body's middle is (on the
    /// wall) and the way it walks (unit). Eased at each end, so it turns
    /// round there. Shared by the physics, the AI and the drawing.
    pub fn lizard_at(&self, i: usize, tick: u32) -> (V3, V3) {
        let Some(l) = self.lizards.get(i) else {
            return (V3::ZERO, v3(0, ONE, 0));
        };
        let period = l.period.max(1);
        let k = tick.wrapping_add(l.phase) % period;
        let phase = (k as i64 * fx::TURN as i64 / period as i64) as i32;
        let s = (ONE - fx::cos(phase)) / 2;
        let at = l.from.lerp(l.to, s);
        let dir = if k < period / 2 {
            l.to.sub(l.from)
        } else {
            l.from.sub(l.to)
        };
        (
            at,
            if dir == V3::ZERO {
                v3(0, ONE, 0)
            } else {
                dir.norm()
            },
        )
    }

    /// Cart `i` at race tick `tick`: where it sits on its rails and which
    /// way it faces (yaw), at an even speed along the track from the start
    /// hole to the end hole, then round again. Shared by the physics, the
    /// AI and the drawing.
    pub fn cart_at(&self, i: usize, tick: u32) -> (V3, i32) {
        let Some(c) = self.carts.get(i) else {
            return (V3::ZERO, 0);
        };
        let Some(t) = self.tracks.get(c.track).filter(|t| !t.points.is_empty()) else {
            return (V3::ZERO, 0);
        };
        let pts = &t.points;
        if pts.len() == 1 {
            return (pts[0], 0);
        }
        let lens: Vec<i64> = pts
            .windows(2)
            .map(|w| w[1].sub(w[0]).len() as i64)
            .collect();
        let total: i64 = lens.iter().sum::<i64>().max(1);
        let period = c.period.max(1) as i64;
        let k = (tick as i64 + c.phase as i64).rem_euclid(period);
        let mut left = total * k / period;
        for (j, &l) in lens.iter().enumerate() {
            if left <= l || j == lens.len() - 1 {
                let f = if l == 0 {
                    0
                } else {
                    (left.min(l) * ONE as i64 / l) as i32
                };
                let a = pts[j];
                let b = pts[j + 1];
                return (a.lerp(b, f), crate::geom::yaw_of(b.sub(a).flat()));
            }
            left -= l;
        }
        (pts[pts.len() - 1], 0)
    }

    /// The lizard whose bouncer is `m`, if any.
    fn lizard_of(&self, m: usize) -> Option<usize> {
        self.lizards.iter().position(|l| l.bouncer == m)
    }

    /// The cart whose bouncer is `m`, if any.
    fn cart_of(&self, m: usize) -> Option<usize> {
        self.carts.iter().position(|c| c.bouncer == m)
    }

    /// Whether vine `i` is a snake vine (drawn as a snake, not a vine).
    pub fn is_snake(&self, i: usize) -> bool {
        self.hangers.get(i).is_some_and(|h| h.snake)
    }

    /// Every room the chase camera keeps inside (the arches, caves, the
    /// tunnel and the skull's mouth).
    pub fn keeps(&self) -> impl Iterator<Item = &Room> {
        self.rooms.iter().chain(self.skull.iter().map(|s| &s.keep))
    }
}

impl Course {
    /// Rattlesnake Ravine's bouncer `m` at `tick`: a lizard's or a cart's
    /// moves with it (offset from where it was at tick 0); every other one
    /// stands still.
    pub(crate) fn ravine_bouncer(&self, r: &RavineLayout, m: usize, tick: u32) -> Mushroom {
        let b = self.mushrooms[m];
        let moved = |from: V3, now: V3| Mushroom {
            top: b.top.add(now.sub(from)),
            radius: b.radius,
        };
        if let Some(l) = r.lizard_of(m) {
            return moved(r.lizard_at(l, 0).0, r.lizard_at(l, tick).0);
        }
        if let Some(c) = r.cart_of(m) {
            return moved(r.cart_at(c, 0).0, r.cart_at(c, tick).0);
        }
        b
    }

    /// The bouncer a body at `p` (the hands) is on at `tick` on Rattlesnake
    /// Ravine: a still one exactly as `mushroom_under`, else a lizard or a
    /// cart where it is now.
    pub(crate) fn ravine_bouncer_under(&self, r: &RavineLayout, p: V3, tick: u32) -> Option<usize> {
        if let Some(m) = self.mushroom_under(p) {
            if r.lizard_of(m).is_none() && r.cart_of(m).is_none() {
                return Some(m);
            }
        }
        let feet = p.y - BODY_DROP;
        let on = |m: &Mushroom| {
            let d = p.flat().sub(m.top.flat());
            let rr = m.radius + BODY_RADIUS / 2;
            d.x.abs() <= rr
                && d.z.abs() <= rr
                && d.len() <= rr
                && feet <= m.top.y
                && feet >= m.top.y - crate::rules::MUSHROOM_HEIGHT
        };
        // (cheap first: a mover's top is always within its path's span of
        // where it starts, so a body further off than that, or the top's
        // reach, is on none; the AI's planned flights ask every tick)
        let near = |m: usize, (lo, hi): (V3, V3)| {
            let b = self.mushrooms[m].top;
            let rr = self.mushrooms[m].radius + BODY_RADIUS / 2;
            let span = hi.sub(lo);
            (p.x - b.x).abs() <= span.x + rr
                && (p.z - b.z).abs() <= span.z + rr
                && feet <= b.y + span.y
                && feet >= b.y - span.y - crate::rules::MUSHROOM_HEIGHT
        };
        let lizards = r
            .lizards
            .iter()
            .map(|l| (l.bouncer, span_of(&[l.from, l.to])));
        let carts = r.carts.iter().map(|c| {
            let pts = r.tracks.get(c.track).map_or(&[][..], |t| &t.points[..]);
            (c.bouncer, span_of(pts))
        });
        lizards
            .chain(carts)
            .find(|&(m, span)| {
                m < self.mushrooms.len() && near(m, span) && on(&self.ravine_bouncer(r, m, tick))
            })
            .map(|(m, _)| m)
    }
}

/// The box round `pts` (lowest corner, highest corner); zero for none.
fn span_of(pts: &[V3]) -> (V3, V3) {
    let Some(&first) = pts.first() else {
        return (V3::ZERO, V3::ZERO);
    };
    pts.iter().fold((first, first), |(lo, hi), q| {
        (
            v3(lo.x.min(q.x), lo.y.min(q.y), lo.z.min(q.z)),
            v3(hi.x.max(q.x), hi.y.max(q.y), hi.z.max(q.z)),
        )
    })
}

/// The snake vines' snap state, for a save point: when each hanger snapped
/// (empty off this map).
pub fn snake_state(c: &Course) -> Vec<Option<u32>> {
    c.ravine
        .as_deref()
        .map(|r| r.hangers.iter().map(|h| h.snapped_at).collect())
        .unwrap_or_default()
}

/// Put back a save point's snake-vine snap state.
pub fn put_snake_state(c: &mut Course, state: &[Option<u32>]) {
    if let Some(r) = c.ravine.as_deref_mut() {
        for (h, s) in r.hangers.iter_mut().zip(state) {
            h.snapped_at = *s;
        }
    }
}
