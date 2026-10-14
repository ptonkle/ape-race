//! Sector Ape's layout (2026-10-13): about 2.5 km through open space, from
//! the launch deck of a carrier to the black hole. The centreline (the
//! spine) stays at height 0; there is no floor (`Terrain::Flat(FLOOR)`, far
//! below the boundary), only decks (`Course.platforms`) to stand on.
//!
//! - **Sections** (metres along): the launch deck and a debris field
//!   (0–540), the Hulk (600–800: three routes, see below), a bend to the
//!   wreck (860–1100), the wreck (1230–1430: three routes), the asteroid
//!   belt (1560–2060), the armada of monkey mechs (2250–2490), the finish
//!   at the black hole's banana horizon (2530).
//! - **Vines**: stations every 23–29 m, two or three lanes 13 m apart. About
//!   half the vines hang straight down; the rest hang up from below,
//!   sideways in from outside the lanes, or slanted (down or up) in from
//!   outside. Each hangs from its own piece of debris (a hull chunk, an
//!   asteroid, a girder), placed so the vine's end is in its lane and the
//!   debris outside the lanes. Every other station also has a catch vine
//!   well below the line, for an ape that fell.
//! - **Decks** (pads, `PADS`): every 150–260 m, 14 m below the line, each
//!   with a row of vines off its front edge (as the start). Crossing the
//!   boundary puts an ape back on the last deck behind it.
//! - **The Hulk** (`HULK`): a big hull with a corridor through it along the
//!   spine (the door route: the switch door near its exit, the switch vine
//!   just past the door); over its top (up vines standing off its deck); and
//!   round its left side (horizontal vines off its wall, girders out from
//!   it). A deck on the outside route.
//! - **The wreck** (`WRECK_AT`): the destroyed mothership lying across the
//!   course. The hidden route is the passage through its bow on the spine
//!   (`WRECK_GAP`, 10 × 7 m, 34 m deep): straight on, but miss and you hit
//!   the hull (not bouncy: a dead stop). The visible ways are the gap
//!   between its torn halves (64 m left, vines off the torn faces) and over
//!   the bow (up vines off its top).
//! - **The armada** (`ARMADA`): ten grunts and two brutes at twice life
//!   size, in ranks across the course, drifting or marching side to side;
//!   vines hang from their fists (down) and elbows (slanted out), and move
//!   with them. Mechs are not solid.
//! - **The finish**: the black hole sits just past the line, its banana
//!   horizon (`HORIZON` radii) touching it; a big deck below it catches
//!   the finished apes for their dance.

use super::rules::HORIZON;
use super::*;
use crate::course::{banana_arcs, centreline, Course, Leg, Platform, Vine, START_LEN};
use crate::fx::{cm, mul, Dice};
use crate::geom::{right_of, yaw_of};
use crate::rules::{BODY_DROP, TICKS_PER_SECOND};
use crate::spacecraft::{pose, wreck_halves, wreck_passage, WRECK_GAP};

const SEED: u64 = 0x5EC7_02A9_E000;

/// The spine, metres and degrees (left turns positive): 2,530 m.
const LEGS: &[Leg] = &[
    Leg(60, 0),
    Leg(140, 0),
    Leg(240, 30),
    Leg(320, 0),
    Leg(320, -40),
    Leg(350, 0),
    Leg(320, 35),
    Leg(150, 0),
    Leg(150, -20),
    Leg(280, 0),
];

/// The invisible edge round the spine: wide enough for the routes round
/// the Hulk and through the wreck's halves (64 m out), and well above and
/// below the vines.
pub const BOUNDARY: Boundary = Boundary {
    side: int(90),
    below: -int(70),
    above: int(80),
};

/// The "ground", far below the boundary: nothing ever stands on it.
pub const FLOOR: i32 = -int(400);

/// Lanes are this far apart across the course, metres.
const LANE: i32 = 13;

/// Decks to be put back on, on the spine, metres along, each 14 m below
/// the line (and the start's); the features place three more
/// (`FEATURE_PADS`).
pub const PADS: &[i32] = &[220, 400, 890, 1020, 1440, 1640, 1820, 1960];
/// Decks the features place: on the route round the Hulk, before the
/// wreck, between the armada's ranks (metres along, metres right).
pub const FEATURE_PADS: &[(i32, i32)] = &[(600, -52), (1162, 0), (2188, 0)];
const PAD_TOP: i32 = -int(14);
/// A deck's row of vines (and the start's) are this far apart: with this
/// map's longer reach, closer ones were caught one after another.
const ROW_GAP: i32 = int(7);
const PAD_HALF: (i32, i32) = (int(11), int(12));

/// The Hulk: its corridor's ends (metres along), its middle's height, its
/// outer half sizes (across, up) and its corridor's (across, up).
pub const HULK: (i32, i32) = (500, 700);
const HULK_Y: i32 = int(2);
const HULK_HALF: (i32, i32) = (int(32), int(25));
const CORRIDOR: (i32, i32) = (int(15), int(17));
/// The switch door across the corridor, and the switch vine past it.
pub const DOOR_AT: i32 = 676;
pub const SWITCH_AT: i32 = 688;
/// The outside route round the Hulk, metres right (left: negative).
const OUTSIDE: i32 = -52;

/// The wreck: the middle of the passage through its bow, metres along, and
/// its height.
pub const WRECK_AT: i32 = 1230;
const WRECK_Y: i32 = int(2);

/// The armada's ranks: metres along, and its mechs' places across
/// (metres right) and whether each is a brute.
pub const ARMADA: &[(i32, &[(i32, bool)])] = &[
    (2055, &[(-26, false), (0, false), (26, false)]),
    (2110, &[(-28, false), (0, true), (28, false)]),
    (2165, &[(-26, false), (0, false), (26, false)]),
    (2220, &[(-24, false), (24, false)]),
    (2272, &[(0, true)]),
];
/// Mechs are drawn at this many times as built.
const MECH_SCALE: i32 = ONE * 2;

/// The black hole's radius.
pub const BLACK_HOLE_R: i32 = int(40);

/// Rows of item crates, metres along.
const CRATES: [i32; 8] = [150, 380, 860, 1080, 1360, 1550, 1800, 1990];

/// The Sector Ape course.
pub fn sector() -> Course {
    sector_seeded(SEED)
}

/// What a vine is like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Way {
    Down,
    Up,
    /// level, in from outside the lanes
    Side,
    /// in from above and outside
    SlantDown,
    /// in from below and outside
    SlantUp,
}

struct B {
    c: Course,
    lay: SectorLayout,
    dice: Dice,
    /// each station's vines, for the banana arcs (down vines only)
    stations: Vec<Vec<usize>>,
    /// the stations now are over where a deck will be: no vine from below
    no_up: bool,
}

impl B {
    fn right(&self, s: i32) -> V3 {
        right_of(self.c.heading_at(s))
    }

    fn yaw(&self, s: i32) -> i32 {
        yaw_of(self.c.heading_at(s))
    }

    /// The point `off` right of the spine at `s`, at height `h`.
    fn pt(&self, s: i32, off: i32, h: i32) -> V3 {
        let p = self.c.point(s).add(self.right(s).scale(off));
        v3(p.x, h, p.z)
    }

    /// A body; returns its index.
    fn body(&mut self, model: Model, frame: Affine, half: V3, bouncy: bool, pull: V3) -> usize {
        self.lay.bodies.push(Body {
            model,
            frame,
            half,
            bouncy,
            pull,
        });
        self.lay.bodies.len() - 1
    }

    /// A body square to the course at `s`, its middle at `at`: its axes
    /// across (x), up (y) and along (z).
    fn boxed(&mut self, s: i32, at: V3, half: V3, model: Model, bouncy: bool) -> usize {
        let frame = pose(at, self.yaw(s), 0, 0, ONE);
        self.body(model, frame, half, bouncy, DOWN)
    }

    /// A vine from its end `tip`, hanging `dir` (a unit vector) from its
    /// anchor, `len` long.
    fn vine(&mut self, tip: V3, dir: V3, len: i32, hanger: Hanger) -> usize {
        let anchor = tip.sub(dir.scale(len));
        let at = self.c.progress(tip, self.c.nearest_anywhere(tip)).0;
        self.c.vines.push(Vine {
            anchor,
            length: len,
            slide: None,
            at,
        });
        self.lay.hangs.push(dir);
        self.lay.hangers.push(hanger);
        self.c.vines.len() - 1
    }

    /// A piece of debris carrying a vine that leaves it at `anchor` going
    /// `dir`: a hull chunk, an asteroid or (for a down vine) a girder
    /// across, placed beyond the anchor so the anchor is on its face.
    fn debris(&mut self, s: i32, anchor: V3, dir: V3, way: Way) -> usize {
        let roll = self.dice.below(6);
        let k = self.dice.below(4);
        let (model, half, turn) = match (way, roll) {
            (Way::Down, 0..=2) => {
                let len = self.dice.range(10, 16);
                (Model::Girder(int(len)), v3(ONE, ONE, int(len) / 2), true)
            }
            (Way::Down, _) | (_, 0..=2) => {
                let sc = ONE + mul(ONE / 2, self.dice.range(0, 100) * ONE / 100);
                (
                    Model::Chunk(k),
                    v3(mul(int(6), sc), mul(cm(180), sc), mul(int(5), sc)),
                    false,
                )
            }
            _ => {
                let sc = ONE + mul(ONE / 2, self.dice.range(0, 100) * ONE / 100);
                (
                    Model::Asteroid(k),
                    v3(mul(int(6), sc), mul(int(5), sc), mul(int(5), sc)),
                    false,
                )
            }
        };
        let yaw = self.yaw(s) + if turn { fx::QUARTER } else { 0 };
        let scale = match model {
            Model::Chunk(_) => fx::div(half.x, int(6)),
            Model::Asteroid(_) => fx::div(half.x, int(6)),
            _ => ONE,
        };
        let axes = pose(V3::ZERO, yaw, 0, 0, ONE);
        let ax = [axes.c0, axes.c1, axes.c2];
        let hs = [half.x, half.y, half.z];
        // from the middle out along -dir to the face: the nearest face the
        // ray leaves through
        let mut t = i32::MAX;
        for k in 0..3 {
            let d = dir.dot(ax[k]).abs();
            if d > ONE / 50 {
                t = t.min(fx::div(hs[k], d));
            }
        }
        let centre = anchor.sub(dir.scale(t));
        let frame = pose(centre, yaw, 0, 0, scale);
        self.body(model, frame, half, true, dir)
    }

    /// A lane's vine at station `s`: its end `off` right of the spine at
    /// height `h`, hanging `way`; `from` is the side its anchor is on (for
    /// the ways in from outside: -1 left, 1 right).
    fn lane_vine(&mut self, s: i32, off: i32, h: i32, way: Way, from: i32) -> usize {
        // (a vine through another's debris, or debris across another vine,
        // is laid again another way, a little higher or lower)
        let mut way = way;
        let mut h = h;
        for _ in 0..8 {
            let v = self.lane_vine_once(s, off, h, way, from);
            if !self.clashes(v) {
                return v;
            }
            self.unlay(v);
            way = self.way();
            h += int(self.dice.range(-3, 3));
        }
        self.lane_vine_once(s, off, h, Way::Down, from)
    }

    /// Whether vine `v` (the last laid, with its debris the last body)
    /// hangs through a solid body but its own, or its debris lies across
    /// another vine.
    fn clashes(&self, v: usize) -> bool {
        let own = self.lay.bodies.len() - 1;

        let near = |b: &Body, i: usize| {
            b.frame.t.dist(self.c.vines[i].anchor) < b.reach() + self.c.vines[i].length
        };
        self.lay.bodies[..own]
            .iter()
            .any(|b| b.solid() && near(b, v) && self.through(b, v))
            || self.crosses_a_vine(own, v)
    }

    /// Whether body `k` lies across any of the first `upto` vines (that hang
    /// from bodies: a mech's move).
    fn crosses_a_vine(&self, k: usize, upto: usize) -> bool {
        let b = &self.lay.bodies[k];
        (0..upto).any(|i| {
            matches!(
                self.lay.hangers[i],
                Hanger::Body(_) | Hanger::Switch(_) | Hanger::Mech { .. }
            ) && b.frame.t.dist(self.c.vines[i].anchor) < b.reach() + self.c.vines[i].length
                && self.through(b, i)
        })
    }

    /// Whether vine `i` passes through body `b` (but its first 2 m).
    fn through(&self, b: &Body, i: usize) -> bool {
        let (a, h, len) = (
            self.c.vines[i].anchor,
            self.lay.hangs[i],
            self.c.vines[i].length,
        );
        let ax = b.axes();
        let mut t = int(2);
        while t <= len {
            let d = a.add(h.scale(t)).sub(b.frame.t);
            if d.dot(ax[0]).abs() < b.half.x
                && d.dot(ax[1]).abs() < b.half.y
                && d.dot(ax[2]).abs() < b.half.z
            {
                return true;
            }
            t += ONE;
        }
        false
    }

    /// A vine of the way round the Hulk's left at `s`: level off its wall
    /// (`wall` the wall's body, its face `wall_x` right), or every other
    /// one down from a girder out of its side.
    fn outside_vine(&mut self, s: i32, k: i32, wall_x: i32, wall: usize) -> usize {
        if k % 2 == 0 {
            let len = int(OUTSIDE.abs()) - HULK_HALF.0 + int(self.dice.range(-2, 3));
            let h = HULK_Y + int(self.dice.range(0, 6));
            let tip = self.pt(s, wall_x - len, h);
            let dir = self.right(s).scale(-ONE);
            self.vine(tip, dir, len, Hanger::Body(wall))
        } else {
            let glen = int(34);
            let gmid = self.pt(s, wall_x - glen / 2, int(30));
            let frame = pose(gmid, self.yaw(s) + fx::QUARTER, 0, 0, ONE);
            let g = self.body(
                Model::Girder(glen),
                frame,
                v3(ONE, ONE, glen / 2),
                true,
                DOWN,
            );
            let off = OUTSIDE + if self.dice.below(2) == 0 { -6 } else { 6 };
            let tip = self.pt(s, int(off), int(29) - int(26));
            self.vine(tip, DOWN, int(26), Hanger::Body(g))
        }
    }

    /// Whether vine `v` hangs through a solid body other than its own.
    fn hits_body(&self, v: usize) -> bool {
        let own = match self.lay.hangers[v] {
            Hanger::Body(k) | Hanger::Switch(k) => Some(k),
            _ => None,
        };
        self.lay
            .bodies
            .iter()
            .enumerate()
            .any(|(k, b)| Some(k) != own && b.solid() && self.through(b, v))
    }

    /// Takes back vine `v` (the last laid), and its own girder if it was
    /// the last body laid for it.
    fn unlay_vine(&mut self, v: usize) {
        let girder = matches!(self.lay.hangers[v], Hanger::Body(k) if k + 1 == self.lay.bodies.len()
            && matches!(self.lay.bodies[k].model, Model::Girder(_)));
        self.c.vines.pop();
        self.lay.hangs.pop();
        self.lay.hangers.pop();
        if girder {
            self.lay.bodies.pop();
        }
    }

    /// Takes back vine `v` (the last laid) and its debris (the last body).
    fn unlay(&mut self, v: usize) {
        debug_assert_eq!(v + 1, self.c.vines.len());
        self.c.vines.pop();
        self.lay.hangs.pop();
        self.lay.hangers.pop();
        self.lay.bodies.pop();
    }

    fn lane_vine_once(&mut self, s: i32, off: i32, h: i32, way: Way, from: i32) -> usize {
        let r = self.right(s);
        let inward = r.scale(-from * ONE);
        let up = V3::UP;
        let dir = match way {
            Way::Down => DOWN,
            Way::Up => up,
            Way::Side => inward,
            Way::SlantDown => inward.sub(up).norm(),
            Way::SlantUp => inward.add(up).norm(),
        };
        let len = match way {
            Way::Down => int(self.dice.range(23, 30)),
            Way::Up => int(self.dice.range(18, 22)),
            _ => int(self.dice.range(20, 26)),
        };
        let tip = self.pt(s, int(off), h);
        let anchor = tip.sub(dir.scale(len));
        let b = self.debris(s, anchor, dir, way);
        self.vine(tip, dir, len, Hanger::Body(b))
    }

    /// A vine straight down to `tip`, `len` long, from debris over it.
    fn down_vine(&mut self, tip: V3, len: i32) -> usize {
        let s = self.c.progress(tip, self.c.nearest_anywhere(tip)).0;
        // (as `lane_vine`: laid again shorter or longer if it clashes)
        for extra in [0, -3, 3, -5, 5] {
            let len = len + int(extra);
            let anchor = tip.add(v3(0, len, 0));
            let b = self.debris(s, anchor, DOWN, Way::Down);
            let v = self.vine(tip, DOWN, len, Hanger::Body(b));
            if !self.clashes(v) {
                return v;
            }
            self.unlay(v);
        }
        // (debris right under it: shorter, ending above it)
        let mut len = len;
        loop {
            let anchor = tip.add(v3(0, len, 0));
            let b = self.debris(s, anchor, DOWN, Way::Down);
            let short = len - int(4);
            let v = self.vine(anchor.sub(v3(0, short, 0)), DOWN, short, Hanger::Body(b));
            if !self.clashes(v) || short < int(10) {
                return v;
            }
            self.unlay(v);
            len -= int(2);
        }
    }

    /// A deck (a platform and its solid slab), `half` (across, along) about
    /// the spot `off` right at `s`, its top at `top`; returns the platform's
    /// index.
    fn deck(&mut self, s: i32, off: i32, top: i32, half: (i32, i32), bouncy: bool) -> usize {
        let at = self.pt(s, off, top);
        self.c.platforms.push(Platform {
            centre: at,
            fwd: self.c.heading_at(s),
            half_len: half.1,
            half_width: half.0,
            top,
        });
        let slab = v3(half.0, int(1), half.1);
        let mid = v3(at.x, top - cm(30) - slab.y, at.z);
        self.boxed(s, mid, slab, Model::Deck, bouncy);
        self.c.platforms.len() - 1
    }

    /// A row of vines across a deck's front edge (the start's, TRIAGE row
    /// 7): `n` of them `ROW_GAP` apart, hanging from a girder to a metre
    /// above the deck. Returns them.
    fn deck_row(&mut self, s: i32, off: i32, top: i32, n: i32) -> Vec<usize> {
        self.deck_row_gap(s, off, top, n, ROW_GAP)
    }

    /// `deck_row` with the vines `gap` apart.
    fn deck_row_gap(&mut self, s: i32, off: i32, top: i32, n: i32, gap: i32) -> Vec<usize> {
        let span = gap * (n - 1);
        let anchor_y = top + int(30);
        let girder = v3(ONE, ONE, span / 2 + int(2));
        let mid = self.pt(s, int(off), anchor_y + ONE);
        let frame = pose(mid, self.yaw(s) + fx::QUARTER, 0, 0, ONE);
        let b = self.body(Model::Girder(span + int(4)), frame, girder, true, DOWN);
        (0..n)
            .map(|k| {
                let o = int(off) - span / 2 + gap * k;
                let tip = self.pt(s, o, top + ONE);
                self.vine(tip, DOWN, int(29), Hanger::Body(b))
            })
            .collect()
    }

    /// The way a station's vine hangs: about half of them straight down.
    fn way(&mut self) -> Way {
        let way = match self.dice.below(20) {
            0..=3 => Way::Down,
            4..=6 => Way::Up,
            7..=11 => Way::Side,
            12..=15 => Way::SlantDown,
            _ => Way::SlantUp,
        };
        // (over where a deck will be, none from below, through it)
        match way {
            Way::Up if self.no_up => Way::Side,
            Way::SlantUp if self.no_up => Way::SlantDown,
            w => w,
        }
    }

    /// A station of `lanes` (metres right) at `s`: two of them (now and
    /// then all), at heights about the line, hanging every way (or, `down`,
    /// all straight down). Returns its vines.
    fn station(&mut self, s: i32, lanes: &[i32], base: i32, down: bool) -> Vec<usize> {
        let n = lanes.len();
        let skip = if n < 3 || self.dice.below(5) == 0 {
            usize::MAX
        } else {
            self.dice.below(n as u32) as usize
        };
        let mut here = Vec::new();
        for (k, &off) in lanes.iter().enumerate() {
            if k == skip {
                continue;
            }
            let along = s + int(self.dice.range(-3, 3));
            let off = off + self.dice.range(-2, 2);
            let h = base + int(self.dice.range(-6, 8));
            let way = if down { Way::Down } else { self.way() };
            let from = match off.signum() {
                0 => {
                    if self.dice.below(2) == 0 {
                        -1
                    } else {
                        1
                    }
                }
                x => x,
            };
            here.push(self.lane_vine(along, off, h, way, from));
        }
        here
    }

    /// Stations from `from` to `to` (metres along) in `lanes`, `base` the
    /// line's height; returns the last station's place.
    fn stations(&mut self, from: i32, to: i32, lanes: &[i32], base: i32) -> i32 {
        let mut s = from;
        let mut after_pad = false;
        while s < to {
            // a deck here? (its row off the front edge is the station)
            if PADS.iter().any(|&p| p >= s && p < s + 29) {
                // (the deck goes where this station would: the row off its
                // front edge is the station, so the vines before it are a
                // station's spacing behind)
                self.pad(s - 12, 0);
                s += 26;
                after_pad = true;
                continue;
            }
            // (off a deck, from a standstill, plain vines to get going: an
            // odd one caught slowly only whirled the ape round; and plain
            // ones over where a deck will be, none hanging up through it)
            let deck_soon = PADS.iter().any(|&p| p >= s && p < s + 58);
            self.no_up = deck_soon;
            let here = self.station(int(s), lanes, base, after_pad);
            self.no_up = false;
            after_pad = false;
            self.stations.push(here);
            s += self.dice.range(23, 29);
        }
        s
    }

    /// A pad: a deck 14 m below the line and the row off its front edge.
    fn pad(&mut self, s: i32, off: i32) -> usize {
        let k = self.deck(int(s), int(off), PAD_TOP, PAD_HALF, true);
        let edge = int(s) + PAD_HALF.1 + cm(150);
        let row = self.deck_row(edge, off, PAD_TOP, 3);
        self.stations.push(row);
        self.lay.pads.push(k);
        k
    }
}

/// Sector Ape laid out from another seed: the same spine, sections, Hulk,
/// wreck, armada and finish, different station vines (for the AI's tests).
pub fn sector_seeded(seed: u64) -> Course {
    let mut c = centreline("sector", LEGS);
    c.half_width = BOUNDARY.side;
    c.terrain = crate::ski::Terrain::Flat(FLOOR);
    let lay = SectorLayout {
        hangs: Vec::new(),
        hangers: Vec::new(),
        bodies: Vec::new(),
        pads: Vec::new(),
        boundary: BOUNDARY,
        door: Door {
            frame: Affine::IDENTITY,
            half: V3::ZERO,
            switch: 0,
            shut_at: None,
        },
        mechs: Vec::new(),
        wreck: Wreck {
            frame: Affine::IDENTITY,
            gap: WRECK_GAP,
        },
        black_hole: (V3::ZERO, BLACK_HOLE_R),
        set_pieces: Vec::new(),
        lobby: (V3::ZERO, 0),
        finish_deck: 0,
        routes: Vec::new(),
        near: Vec::new(),
        by_at: Vec::new(),
    };
    let mut b = B {
        c,
        lay,
        dice: Dice(seed),
        stations: Vec::new(),
        no_up: false,
    };

    start(&mut b);
    let lanes3 = [-LANE, 0, LANE];
    // the debris field, to the bend and the Hulk
    b.stations(57, 465, &lanes3, 0);
    hulk(&mut b);
    b.stations(718, 1140, &lanes3, 0);
    wreck(&mut b);
    b.stations(1332, 1700, &lanes3, 0);
    // the asteroid belt: four lanes
    b.stations(1708, 1880, &[-20, -7, 7, 20], 0);
    b.stations(1880, 2032, &lanes3, 0);
    armada(&mut b);
    finish(&mut b);
    crates(&mut b);
    bumpers(&mut b);
    set_pieces(&mut b);

    // bananas under the down vines of every other station
    let down: Vec<Vec<usize>> = b
        .stations
        .iter()
        .map(|st| {
            st.iter()
                .copied()
                .filter(|&i| b.lay.hangs[i] == DOWN)
                .collect()
        })
        .collect();
    banana_arcs(&mut b.c, &down);

    let mut lay = b.lay;
    lay.index_bodies(&b.c.path);
    lay.index_vines(&b.c.vines.iter().map(|v| v.at).collect::<Vec<_>>());
    let mut c = b.c;
    c.sector = Some(Box::new(lay));
    c
}

/// The launch deck: the starters across it, the row off its front edge,
/// and the first station close.
fn start(b: &mut B) {
    let deck_s = START_LEN / 2 - int(2);
    let k = b.deck(deck_s, 0, 0, (int(9), START_LEN / 2 + int(2)), true);
    b.lay.pads.push(k);
    let right = b.right(0);
    for k in 0..4 {
        let at = b.c.point(int(2)).add(right.scale(int(-6) + k * int(4)));
        b.c.starts[k as usize] = v3(at.x, BODY_DROP, at.z);
    }
    b.c.start_yaw = 0;
    let edge = START_LEN + cm(150);
    // across the deck's front edge (as the Jungle's start): five, 4.5 m
    // apart (closer, with this map's reach, they were caught one after
    // another)
    let row = b.deck_row_gap(edge, 0, 0, 5, cm(450));
    b.stations.push(row);
    // the first station: close, not too high, straight down
    let here: Vec<usize> = [-6, 0, 6]
        .iter()
        .map(|&off| {
            let tip = b.pt(int(32), int(off), 0);
            let anchor = tip.add(v3(0, int(30), 0));
            let d = b.debris(int(32), anchor, DOWN, Way::Down);
            b.vine(tip, DOWN, int(30), Hanger::Body(d))
        })
        .collect();
    b.stations.push(here);
    // the lobby: the deck's middle, facing on down the course (the
    // mothership waits behind, where its pass starts)
    let mid = b.c.platforms[k].centre;
    b.lay.lobby = (v3(mid.x, BODY_DROP, mid.z), b.c.start_yaw);
}

/// The Hulk: the corridor (the door route), over the top, round the left.
fn hulk(b: &mut B) {
    let (s0, s1) = (int(HULK.0), int(HULK.1));
    let mid_s = (s0 + s1) / 2;
    let len = (s1 - s0) / 2;
    let (hx, hy) = HULK_HALF;
    let (cx, cy) = CORRIDOR;
    let mid = b.pt(mid_s, 0, HULK_Y);
    // the hull, drawn whole
    let yaw = b.yaw(mid_s);
    b.body(
        Model::Hulk,
        pose(mid, yaw, 0, 0, ONE),
        v3(hx, hy, len),
        false,
        DOWN,
    );
    // its solid parts round the corridor
    let wall = (hx - cx) / 2;
    let left = b.boxed(
        mid_s,
        b.pt(mid_s, -(cx + wall), HULK_Y),
        v3(wall, hy, len),
        Model::Hidden,
        true,
    );
    b.boxed(
        mid_s,
        b.pt(mid_s, cx + wall, HULK_Y),
        v3(wall, hy, len),
        Model::Hidden,
        true,
    );
    let slab = (hy - cy) / 2;
    let roof = b.boxed(
        mid_s,
        b.pt(mid_s, 0, HULK_Y + cy + slab),
        v3(cx, slab, len),
        Model::Hidden,
        true,
    );
    // the corridor's floor: a deck (an ape that drops in there runs on and
    // jumps for the vines)
    b.deck(mid_s, 0, HULK_Y - cy, (cx, len), false);
    let ceiling = HULK_Y + cy;
    // a deck along its top, to land and run on
    let top = HULK_Y + hy + int(2) + cm(30);
    // (as wide as the hull, over its walls' tops: nothing to get caught
    // between)
    b.deck(mid_s, 0, top, (hx, len), true);
    let top_body = b.lay.bodies.len() - 1;

    // the corridor: from the ceiling and in from the walls, two lanes
    let mut corridor = Vec::new();
    let mut s = HULK.0 + 12;
    let mut k = 0;
    while s < DOOR_AT - 8 {
        let si = int(s);
        let side = if k % 2 == 0 { -1 } else { 1 };
        let mut here = Vec::new();
        // a vine from the ceiling over each lane, to within a jump of
        // the floor (a vine in from a wall only whirled the ape into it)
        for (k, lane) in [side, -side].into_iter().enumerate() {
            let len = int(b.dice.range(24, 26));
            let tip = b.pt(si + int(6) * k as i32, int(6) * lane, ceiling - len);
            here.push(b.vine(tip, DOWN, len, Hanger::Body(roof)));
        }
        corridor.extend(here.iter().copied());
        b.stations.push(here);
        s += b.dice.range(23, 27);
        k += 1;
    }
    // the door, and the switch vine just past it
    let door_mid = b.pt(int(DOOR_AT), 0, HULK_Y);
    b.lay.door = Door {
        frame: pose(door_mid, b.yaw(int(DOOR_AT)), 0, 0, ONE),
        half: v3(cx, cy, ONE),
        switch: 0,
        shut_at: None,
    };
    let sw_len = int(24);
    let tip = b.pt(int(SWITCH_AT), 0, ceiling - sw_len);
    let switch = b.vine(tip, DOWN, sw_len, Hanger::Switch(roof));
    b.lay.door.switch = switch;
    corridor.push(switch);
    b.stations.push(vec![switch]);

    // into the corridor's mouth and out of it: a station either side
    // (on the spine, at the corridor's height)
    for (s, off) in [(HULK.0 - 14, 6), (HULK.1 + 6, -6)] {
        let v = b.lane_vine(int(s), off, 0, Way::Down, 1);
        corridor.push(v);
        b.stations.push(vec![v]);
    }

    // over the top: vines from debris drifting high over it, their swings
    // clear of its deck (vines standing up off the deck whirled apes down
    // onto it, held there)
    let _ = top_body;
    let mut over = Vec::new();
    // up there first: a high vine before the Hulk
    let rise = b.lane_vine(int(HULK.0 - 20), 0, int(30), Way::Down, 1);
    over.push(rise);
    let mut s = HULK.0 + 16;
    let mut k = 0;
    while s < HULK.1 - 6 {
        let si = int(s);
        let off = if k % 2 == 0 { -7 } else { 7 };
        let len = int(b.dice.range(22, 24));
        let h = top + int(10) + int(b.dice.range(0, 3));
        let tip = b.pt(si, int(off), h);
        // (all straight down: a level vine off debris by the hull swung apes
        // down into its walls)
        let v = b.down_vine(tip, len);
        over.push(v);
        b.stations.push(vec![v]);
        s += b.dice.range(24, 28);
        k += 1;
    }

    // round the left: in off its wall, and from girders out of its side
    let mut outside = Vec::new();
    let mut s = HULK.0 + 4;
    let mut k = 0;
    let wall_x = -hx;
    while s < HULK.1 + 10 {
        let si = int(s);
        let v = b.outside_vine(si, k, wall_x, left);
        // (one through a piece of debris is laid again a few metres on)
        let v = if b.hits_body(v) {
            b.unlay_vine(v);
            b.outside_vine(si + int(5), k + 1, wall_x, left)
        } else {
            v
        };
        outside.push(v);
        b.stations.push(vec![v]);
        s += b.dice.range(22, 26);
        k += 1;
    }
    // the way out there from the spine, and a deck on it
    let out_in = b.lane_vine(int(HULK.0 - 30), -42, 0, Way::Down, -1);
    outside.push(out_in);
    let k = b.deck(
        int(FEATURE_PADS[0].0),
        int(OUTSIDE),
        PAD_TOP,
        PAD_HALF,
        true,
    );
    b.lay.pads.push(k);
    let row = b.deck_row(
        int(FEATURE_PADS[0].0) + PAD_HALF.1 + cm(150),
        OUTSIDE,
        PAD_TOP,
        4,
    );
    outside.extend(row.iter().copied());
    b.stations.push(row);

    b.lay.routes.push(Route {
        name: "corridor",
        from: s0 - int(20),
        to: s1 + int(10),
        vines: corridor,
        hidden: false,
        door: true,
    });
    b.lay.routes.push(Route {
        name: "over the hulk",
        from: s0 - int(20),
        to: s1 + int(10),
        vines: over,
        hidden: false,
        door: false,
    });
    b.lay.routes.push(Route {
        name: "round the hulk",
        from: s0 - int(30),
        to: s1 + int(10),
        vines: outside,
        hidden: false,
        door: false,
    });
}

/// The wreck across the course: through its bow's passage (hidden), the
/// gap between its halves, or over the bow.
fn wreck(b: &mut B) {
    let at_s = int(WRECK_AT);
    let yaw = b.yaw(at_s) - fx::QUARTER;
    let gap = WRECK_GAP;
    // the wreck's frame: turned so the passage runs along the course, and
    // moved so its middle is on the spine
    // (rolled back by the bow's own 9°, so the passage runs level)
    let f0 = pose(V3::ZERO, yaw, 0, -fx::deg(9), ONE);
    let p0 = wreck_passage(&f0, gap);
    let want = b.pt(at_s, 0, WRECK_Y);
    let f = Affine {
        t: want.sub(p0.middle),
        ..f0
    };
    b.lay.wreck = Wreck { frame: f, gap };
    let pass = wreck_passage(&f, gap);
    let (stern, bow) = wreck_halves(&f);
    // the bow's hull round the passage (ship metres: x across the hull, the
    // passage's way; y up; z along the ship), and the stern's
    let (gw, gh) = (gap.0 / 2, gap.1 / 2);
    let pz = int(40);
    let hull = |b: &mut B, half: &Affine, lo: V3, hi: V3| {
        let mid = lo.add(hi).scale(ONE / 2);
        let h = hi.sub(lo).scale(ONE / 2);
        let frame = half.then(&Affine::translate(mid));
        let frame = Affine {
            c0: frame.c0.norm(),
            c1: frame.c1.norm(),
            c2: frame.c2.norm(),
            ..frame
        };
        b.body(Model::Hidden, frame, h, false, DOWN)
    };
    let (x, y) = (int(17), int(16));
    let aft = hull(b, &bow, v3(-x, -y, -int(10)), v3(x, y, pz - gw));
    hull(b, &bow, v3(-x, -y, pz + gw), v3(x, y, int(110)));
    hull(b, &bow, v3(-x, gh, pz - gw), v3(x, y, pz + gw));
    hull(b, &bow, v3(-x, -y, pz - gw), v3(x, -gh, pz + gw));
    hull(b, &stern, v3(-x, -y, -int(92)), v3(x, y, -int(10)));
    // (no vine's swing reaches the hull, but the one off its torn face: a
    // swing into it is a dead stop, `the_hull_stops_no_swing` in the tests)

    // a deck before it all (on the spine, well short of the hull): its row
    // is where all three ways start
    b.pad(FEATURE_PADS[1].0, FEATURE_PADS[1].1);

    // the hidden route: the snipe vine, on the passage's line 39 m short of
    // its mouth (let go on the level and the ape threads the gap), and two
    // vines on in line beyond the far mouth
    let mut hidden = Vec::new();
    let into = |d: i32| pass.middle.add(pass.axis.scale(d));
    for (d, len) in [(-int(56), int(22)), (int(48), int(24)), (int(74), int(24))] {
        let v = b.down_vine(into(d), len);
        // (only the snipe vine is the hidden route's own: the two past the
        // far mouth are on every route's way back in)
        if d < 0 {
            hidden.push(v);
        }
        b.stations.push(vec![v]);
    }

    // between the halves: out to the gap, a level vine off the bow's torn
    // face across it, and back in
    let mut through = Vec::new();
    let gap_mid = f.apply(v3(0, 0, -int(8)));
    let (gs, goff) = b.c.progress(gap_mid, b.c.nearest_anywhere(gap_mid));
    let goff = goff / ONE;
    for (ds, off, len) in [
        (-int(92), -26, 24),
        (-int(68), goff + 14, 22),
        (-int(44), goff + 2, 20),
    ] {
        let v = b.down_vine(b.pt(gs + ds, int(off), WRECK_Y), int(len));
        through.push(v);
        b.stations.push(vec![v]);
    }
    let face = bow.apply(v3(0, int(2), -int(10)));
    let dir = stern.apply(v3(0, int(2), -int(10))).sub(face).norm();
    let len = int(13);
    let v = b.vine(face.add(dir.scale(len)), dir, len, Hanger::Body(aft));
    through.push(v);
    b.stations.push(vec![v]);
    for (ds, off, len) in [(int(46), goff + 8, 20), (int(70), -30, 24)] {
        let v = b.down_vine(b.pt(gs + ds, int(off), WRECK_Y), int(len));
        through.push(v);
        b.stations.push(vec![v]);
    }

    // over the bow, on its fore side (away from the gap): vines from debris
    // drifting high over it, their swings clear of its top
    let mut over = Vec::new();
    let top = WRECK_Y + y;
    for (ds, off, h, len) in [
        (-int(52), 20, int(24), 24),
        (-int(26), 16, top + int(9), 24),
        (0, 14, top + int(9), 24),
        (int(26), 16, top + int(8), 24),
        (int(52), 18, int(10), 26),
    ] {
        let v = b.down_vine(b.pt(at_s + ds, int(off), h), int(len));
        over.push(v);
        b.stations.push(vec![v]);
    }

    b.lay.routes.push(Route {
        name: "wreck passage",
        from: at_s - int(70),
        to: at_s + int(80),
        vines: hidden,
        hidden: true,
        door: false,
    });
    b.lay.routes.push(Route {
        name: "between the halves",
        from: gs - int(90),
        to: gs + int(60),
        vines: through,
        hidden: false,
        door: false,
    });
    b.lay.routes.push(Route {
        name: "over the bow",
        from: at_s - int(50),
        to: at_s + int(40),
        vines: over,
        hidden: false,
        door: false,
    });
}

/// The armada: ranks of mechs, vines from their fists and elbows, and a
/// station of debris between ranks.
fn armada(b: &mut B) {
    use crate::spacecraft::{Gait, Mech, MechKind};
    let mut n = 0;
    for (r, &(s, mechs)) in ARMADA.iter().enumerate() {
        let si = int(s);
        let mut here = Vec::new();
        for &(off, brute) in mechs {
            let kind = if brute {
                MechKind::Brute
            } else {
                MechKind::Grunt
            };
            let gait = if r % 2 == 0 { Gait::Drift } else { Gait::March };
            // the pelvis high enough that a fist's vine ends about the line
            let (y, len) = if brute {
                (int(36), int(24))
            } else {
                (int(29), int(21))
            };
            let sway = int(7);
            let m = MechPath {
                mech: Mech {
                    kind,
                    gait,
                    phase: (n * 13_001) % fx::TURN,
                },
                from: b.pt(si, int(off) - sway, y),
                to: b.pt(si, int(off) + sway, y),
                period: (20 + (n % 3) as u32 * 4) * TICKS_PER_SECOND as u32,
                // facing the apes coming
                yaw: b.yaw(si) + fx::HALF,
                scale: MECH_SCALE,
            };
            b.lay.mechs.push(m);
            let mi = b.lay.mechs.len() - 1;
            // one fist's vine straight down, the other fist's and an
            // elbow's slanted out and down (the mech's own pull: outward)
            let e = (n % 2) as u8;
            let mid = m.frame(0).t;
            for (at, slant) in [
                (MechPoint::Hand(e), false),
                (MechPoint::Hand(1 - e), true),
                (MechPoint::Elbow(e), true),
            ] {
                let anchor = m.point(at, 0);
                let dir = if slant {
                    anchor.sub(mid).flat().norm().sub(V3::UP).norm()
                } else {
                    DOWN
                };
                // (a slanted vine's end about as low as the straight one's)
                let l = if slant { mul(len, ONE * 4 / 3) } else { len };
                here.push(b.vine(
                    anchor.add(dir.scale(l)),
                    dir,
                    l,
                    Hanger::Mech { mech: mi, at },
                ));
            }
            n += 1;
        }
        b.stations.push(here);
        // debris between this rank and the next
        let next = ARMADA.get(r + 1).map_or(2330 - 2, |x| x.0);
        let mid = (s + next) / 2;
        if next - s > 30 {
            let here = b.station(int(mid), &[-LANE, 0, LANE], 0, r % 2 == 0);
            b.stations.push(here);
        }
    }
    // a deck between the ranks
    b.pad(FEATURE_PADS[2].0, FEATURE_PADS[2].1);
}

/// The black hole past the line, its horizon touching it, and the deck
/// under it for the dance.
fn finish(b: &mut B) {
    let l = b.c.length;
    let r = BLACK_HOLE_R;
    let rim = mul(r, HORIZON);
    let centre = b.pt(l + rim, 0, int(15));
    b.lay.black_hole = (centre, r);
    let top = int(15) - rim - int(10);
    let k = b.deck(l + rim, 0, top, (int(55), int(66)), false);
    b.lay.finish_deck = k;
}

/// Item crates: rows across the course about every 240 m, on the vines of
/// the station nearest each place.
fn crates(b: &mut B) {
    for at in CRATES {
        let si = int(at);
        let best = b
            .stations
            .iter()
            .filter(|st| st.len() >= 2)
            .min_by_key(|st| (b.c.vines[st[0]].at - si).abs())
            .cloned();
        if let Some(st) = best {
            for &v in st.iter().take(3) {
                b.c.crates.push(b.c.vines[v].anchor);
            }
        }
    }
}

/// Bouncy asteroids with no vines, between the lanes of the asteroid belt
/// and the debris field: pinball bumpers.
fn bumpers(b: &mut B) {
    for (s, off, h) in [
        (130, -20, 6),
        (330, 22, -4),
        (1560, -24, 2),
        (1740, 0, -12),
        (1790, -28, 10),
        (1850, 26, -2),
        (1980, 20, 8),
    ] {
        let si = int(s);
        // (moved up or down off any vine it would lie across, else left out)
        for dh in [0, 8, -8, 14, -14] {
            let at = b.pt(si, int(off), int(h + dh));
            let half = v3(int(6), int(5), int(5));
            let frame = pose(at, b.yaw(si) + fx::deg(s), fx::deg(off), 0, ONE);
            let k = b.body(Model::Asteroid((s % 4) as u32), frame, half, true, DOWN);
            if !b.crosses_a_vine(k, b.c.vines.len()) {
                break;
            }
            b.lay.bodies.pop();
        }
    }
}

/// The mothership's pass overhead, dogfights and a capital battle far off.
fn set_pieces(b: &mut B) {
    let tps = TICKS_PER_SECOND as u32;
    let go = crate::race::COUNTDOWN as u32;
    let from = b.pt(-int(260), 0, int(95));
    let to = b.pt(int(1300), int(40), int(120));
    b.lay.set_pieces.push(SetPiece::Pass {
        from,
        to,
        start: go + 14 * tps,
        end: go + 32 * tps,
    });
    for (s, off, h) in [
        (300, -260, 70),
        (900, 280, 110),
        (1600, -300, 40),
        (2150, 260, 90),
    ] {
        let si = int(s);
        let at = b.pt(si, int(off), int(h));
        b.lay.set_pieces.push(SetPiece::Dogfight {
            frame: pose(at, b.yaw(si), 0, 0, ONE * 3),
        });
    }
    for (s, off, h) in [(700, -460, 60), (1900, 450, -40)] {
        let si = int(s);
        let at = b.pt(si, int(off), int(h));
        b.lay.set_pieces.push(SetPiece::Battle {
            frame: pose(at, b.yaw(si) + fx::QUARTER, 0, 0, ONE),
        });
    }
}

#[cfg(test)]
mod tests;
