//! Rattlesnake Ravine's solids (`notes/ravine-pitfalls.md`): the canyon
//! walls, the arch, cave and tunnel roofs (and the cave and tunnel sides),
//! the wall holes the mine tracks come out of, the T-rex skull's bone, the
//! sphinx and the large rocks. Nothing else is solid (the person: small
//! dressing never is).
//!
//! Every solid comes from the same `RavineLayout` numbers the scene draws
//! from, and reaches well past what is drawn (`WALL_ABOVE`, `WALL_BACK`,
//! `ROOF_THICK`, ...). The body's ball is swept from where it was at the
//! start of the tick (`sector::sweep`, as the temple's face), so nothing
//! passes through however fast. The solids are built once a course
//! (`Course::ravine_solids`) and indexed by place: a grid of `CELL` squares,
//! each listing the solids near it, so a step looks at a handful.

use super::*;
use crate::fx::{int, mul};
use crate::geom::{facing, right_of};
use crate::rules::WALL_BOUNCE;
use crate::sector::{sweep, Touch};
use std::sync::{Arc, OnceLock};

/// A canyon wall's solid stands this far over its rim, ...
pub const WALL_ABOVE: i32 = int(10);
/// ... runs this far under its foot, ...
pub const WALL_BELOW: i32 = int(10);
/// ... this far further back into the rock than its `depth`, ...
pub const WALL_BACK: i32 = int(4);
/// ... and this far past each end of each stretch (so the stretches
/// overlap at the bends).
pub const WALL_RUN: i32 = int(1);
/// A roof is this thick over its underside.
pub const ROOF_THICK: i32 = int(8);
/// A roof reaches this far into the rock either side; a cave's or the
/// tunnel's side walls are this thick.
pub const ROOM_SIDE: i32 = int(6);
/// A roof is cut into pieces this long (each piece's underside at its
/// higher end, so a sloping roof is never lower than drawn).
const ROOF_PIECE: i32 = int(8);
/// A wall hole's box runs this far back into the rock, and its frame this
/// far round (and out in front of) the mouth.
pub const HOLE_BACK: i32 = int(4);
pub const HOLE_RIM: i32 = cm(60);
/// The skull's bone and the sphinx reach this far past what is drawn.
pub const PAST: i32 = cm(40);
/// The grid's square (larger on a course spread wider than `CELLS`
/// squares across, so the grid stays small).
const CELL: i32 = int(16);
const CELLS: i32 = 512;
/// A solid is listed in every square within this of it: a move shorter
/// than `MOVE` (every tick's, under 60 m/s) only needs the square it ends in.
const MOVE: i32 = int(2);
const PAD: i32 = BODY_RADIUS + BODY_DROP + MOVE;
/// A little off a solid, so the next tick starts clear of it.
const SKIN: i32 = cm(2);
/// A top is ground for feet this far through it (as a platform's).
const STAND: i32 = cm(60);

/// What a solid is (for the tests, and the camera).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolidKind {
    Wall,
    Roof,
    Hole,
    Bone,
    Sphinx,
    Rock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// an upright box: its middle, the way along it (flat, unit) and its
    /// half sizes across (`right_of(fwd)`), up and along
    Box { mid: V3, fwd: V3, half: V3 },
    /// an upright round post from its foot
    Post { foot: V3, radius: i32, height: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Solid {
    pub kind: SolidKind,
    pub shape: Shape,
}

impl Solid {
    /// Its flat bounds: (min x, min z, max x, max z).
    fn bounds(&self) -> (i32, i32, i32, i32) {
        match self.shape {
            Shape::Box { mid, fwd, half } => {
                let r = right_of(fwd);
                let ex = mul(half.x, r.x.abs()) + mul(half.z, fwd.x.abs());
                let ez = mul(half.x, r.z.abs()) + mul(half.z, fwd.z.abs());
                (mid.x - ex, mid.z - ez, mid.x + ex, mid.z + ez)
            }
            Shape::Post { foot, radius, .. } => (
                foot.x - radius,
                foot.z - radius,
                foot.x + radius,
                foot.z + radius,
            ),
        }
    }

    /// Its top.
    pub fn top(&self) -> i32 {
        match self.shape {
            Shape::Box { mid, half, .. } => mid.y + half.y,
            Shape::Post { foot, height, .. } => foot.y + height,
        }
    }

    /// Whether `p` (flat) is over it.
    fn over(&self, p: V3) -> bool {
        match self.shape {
            Shape::Box { mid, fwd, half } => {
                let d = p.sub(mid).flat();
                d.dot(right_of(fwd)).abs() <= half.x && d.dot(fwd).abs() <= half.z
            }
            Shape::Post { foot, radius, .. } => !p.flat().sub(foot.flat()).longer_than(radius),
        }
    }

    /// Whether the point `p` is in it, or within `margin` of it.
    pub fn holds(&self, p: V3, margin: i32) -> bool {
        match self.shape {
            Shape::Box { mid, fwd, half } => {
                let d = p.sub(mid);
                d.dot(right_of(fwd)).abs() <= half.x + margin
                    && d.y.abs() <= half.y + margin
                    && d.flat().dot(fwd).abs() <= half.z + margin
            }
            Shape::Post {
                foot,
                radius,
                height,
            } => {
                p.y >= foot.y - margin
                    && p.y <= foot.y + height + margin
                    && !p.flat().sub(foot.flat()).longer_than(radius + margin)
            }
        }
    }

    /// The body's ball (the hands, `BODY_RADIUS`, its feet `BODY_DROP`
    /// under) swept from `p0` to `p1` against it: the first touch.
    fn sweep(&self, p0: V3, p1: V3) -> Option<Touch> {
        match self.shape {
            Shape::Box { mid, fwd, half } => {
                // (the feet hang under the hands: a hand over the top by
                // less than the drop has its feet in the box; the ball's
                // radius is added to that, so a body standing on the top
                // has its hands just on the reach's top)
                let up = BODY_DROP - BODY_RADIUS;
                let mid = mid.add(v3(0, up / 2, 0));
                let half = v3(half.x, half.y + up / 2, half.z);
                let axes = [right_of(fwd), v3(0, ONE, 0), fwd];
                sweep(mid, &axes, half, BODY_RADIUS, p0, p1)
            }
            Shape::Post {
                foot,
                radius,
                height,
            } => sweep_post(
                foot,
                radius + BODY_RADIUS,
                foot.y + height + BODY_DROP,
                p0,
                p1,
            ),
        }
    }
}

/// A ball's centre swept from `p0` to `p1` against an upright post (`foot`,
/// reach `r` round its axis, up to `hi`): the first touch on its side or
/// its top.
fn sweep_post(foot: V3, r: i32, hi: i32, p0: V3, p1: V3) -> Option<Touch> {
    let lo = foot.y - BODY_RADIUS;
    let c = foot.flat();
    let in_y = |y: i32| y > lo && y < hi;
    let d0 = p0.flat().sub(c);
    let d1 = p1.flat().sub(c);
    let l0 = d0.len();
    if l0 < r && in_y(p0.y) {
        // inside: out the side or over the top, whichever is nearer
        let side = r - l0;
        let up = hi - p0.y;
        let radial = if l0 > 0 { d0.norm() } else { v3(ONE, 0, 0) };
        return Some(if up < side {
            Touch {
                t: 0,
                normal: v3(0, ONE, 0),
                depth: up,
                end_depth: (hi - p1.y).max(0),
            }
        } else {
            Touch {
                t: 0,
                normal: radial,
                depth: side,
                end_depth: (r - d1.dot(radial)).max(0),
            }
        });
    }
    let mut best: Option<Touch> = None;
    // the top: crossing its height going down, over it
    if p0.y >= hi && p1.y < hi {
        let t = ((p0.y - hi) as i64 * ONE as i64 / (p0.y - p1.y) as i64) as i32;
        let q = p0.lerp(p1, t);
        if !q.flat().sub(c).longer_than(r) {
            best = Some(Touch {
                t,
                normal: v3(0, ONE, 0),
                depth: 0,
                end_depth: 0,
            });
        }
    }
    // the side: where the flat line first comes within `r` of the axis
    let m = d1.sub(d0);
    let a = m.dot64(m) as i128;
    if a > 0 {
        let b = d0.dot64(m) as i128;
        let cc = d0.dot64(d0) as i128 - (r as i128) * (r as i128);
        let disc = b * b - a * cc;
        if disc >= 0 && cc > 0 {
            let root = crate::fx::isqrt(disc as u128) as i128;
            let t = (-b - root) * ONE as i128 / a;
            if (0..=ONE as i128).contains(&t) {
                let t = t as i32;
                let q = p0.lerp(p1, t);
                if in_y(q.y) && best.is_none_or(|bt| t < bt.t) {
                    let n = q.flat().sub(c);
                    best = Some(Touch {
                        t,
                        normal: if n == V3::ZERO {
                            v3(ONE, 0, 0)
                        } else {
                            n.norm()
                        },
                        depth: 0,
                        end_depth: 0,
                    });
                }
            }
        }
    }
    best
}

/// Rattlesnake Ravine's solids, indexed by place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solids {
    pub list: Vec<Solid>,
    /// the grid's corner, its size in squares, and each square's solids
    x0: i32,
    z0: i32,
    cell: i32,
    nx: i32,
    nz: i32,
    cells: Vec<Vec<u32>>,
}

impl Solids {
    /// The solids of `r`.
    pub fn of(r: &RavineLayout) -> Solids {
        let mut list = Vec::new();
        let up = v3(0, ONE, 0);
        let upright = |mid: V3, fwd: V3, half: V3| Shape::Box {
            mid,
            fwd: fwd.flat().norm(),
            half,
        };
        // the canyon walls: a box behind each stretch of the face
        for w in &r.walls {
            for k in w.points.windows(2) {
                let (a, b) = (k[0], k[1]);
                let run = b.at.sub(a.at).flat();
                let len = run.len();
                if len <= 0 {
                    continue;
                }
                let fwd = run.norm();
                let into = match w.side {
                    Side::Left => right_of(fwd).neg(),
                    Side::Right => right_of(fwd),
                };
                let thick = w.depth + WALL_BACK;
                let bottom = a.at.y.min(b.at.y) - WALL_BELOW;
                let top = (a.at.y + a.height).max(b.at.y + b.height) + WALL_ABOVE;
                let mid = a.at.lerp(b.at, ONE / 2).add(into.scale(thick / 2));
                list.push(Solid {
                    kind: SolidKind::Wall,
                    shape: upright(
                        v3(mid.x, (top + bottom) / 2, mid.z),
                        fwd,
                        v3(thick / 2, (top - bottom) / 2, len / 2 + WALL_RUN),
                    ),
                });
            }
        }
        // the arches', caves' and the tunnel's roofs, and the caves' and
        // the tunnel's sides (their mouths stay open)
        for room in &r.rooms {
            if room.kind == RoomKind::Skull {
                continue;
            }
            let run = room.to.sub(room.from).flat();
            let len = run.len();
            if len <= 0 {
                continue;
            }
            let fwd = run.norm();
            let pieces = (len + ROOF_PIECE - 1) / ROOF_PIECE;
            let floor = |k: i32| room.from.y + (room.to.y - room.from.y) * k / pieces;
            for k in 0..pieces {
                let (f0, f1) = (floor(k), floor(k + 1));
                let under = f0.max(f1) + room.height;
                let mid = room.from.lerp(
                    room.to,
                    ((2 * k + 1) as i64 * ONE as i64 / (2 * pieces) as i64) as i32,
                );
                list.push(Solid {
                    kind: SolidKind::Roof,
                    shape: upright(
                        v3(mid.x, under + ROOF_THICK / 2, mid.z),
                        fwd,
                        v3(
                            room.half_width + ROOM_SIDE,
                            ROOF_THICK / 2,
                            len / pieces / 2 + cm(5),
                        ),
                    ),
                });
            }
            if matches!(room.kind, RoomKind::Cave | RoomKind::Tunnel) {
                let bottom = room.from.y.min(room.to.y) - WALL_BELOW;
                let top = room.from.y.max(room.to.y) + room.height + ROOF_THICK;
                let mid = room.from.lerp(room.to, ONE / 2);
                for s in [-1, 1] {
                    let at = mid.add(right_of(fwd).scale(s * (room.half_width + ROOM_SIDE / 2)));
                    list.push(Solid {
                        kind: SolidKind::Roof,
                        shape: upright(
                            v3(at.x, (top + bottom) / 2, at.z),
                            fwd,
                            v3(ROOM_SIDE / 2, (top - bottom) / 2, len / 2),
                        ),
                    });
                }
            }
        }
        // the holes the mine tracks come out of
        for t in &r.tracks {
            for h in [t.start, t.end] {
                let out = h.out.flat().norm();
                let bottom = h.at.y - int(1);
                let top = h.at.y + h.height + HOLE_RIM;
                let at = h.at.add(out.scale((HOLE_RIM - HOLE_BACK) / 2));
                list.push(Solid {
                    kind: SolidKind::Hole,
                    shape: upright(
                        v3(at.x, (top + bottom) / 2, at.z),
                        out,
                        v3(
                            h.half_width + HOLE_RIM,
                            (top - bottom) / 2,
                            (HOLE_RIM + HOLE_BACK) / 2,
                        ),
                    ),
                });
            }
        }
        // the T-rex skull's bone, from the model's own boxes (its snout
        // toward +z, the way through along -z)
        if let Some(s) = &r.skull {
            let along = facing(s.yaw, 0).neg();
            let across = right_of(along);
            for (c, size) in crate::ravinemodels::rex_bone_boxes(MOUTH_GAP) {
                let mid =
                    s.at.add(across.scale(c.x))
                        .add(up.scale(c.y))
                        .add(along.scale(c.z));
                list.push(Solid {
                    kind: SolidKind::Bone,
                    shape: upright(
                        mid,
                        along,
                        v3(size.x / 2 + PAST, size.y / 2 + PAST, size.z / 2 + PAST),
                    ),
                });
            }
        }
        // the sphinx, boxed (its front feet at `sphinx`)
        {
            let f = &r.finish;
            let fwd = facing(f.sphinx_yaw, 0);
            let size = crate::ravinemodels::SPHINX_SIZE;
            let mid = f
                .sphinx
                .sub(fwd.scale(size.z / 2))
                .add(up.scale(size.y / 2));
            list.push(Solid {
                kind: SolidKind::Sphinx,
                shape: upright(
                    mid,
                    fwd,
                    v3(size.x / 2 + PAST, size.y / 2 + PAST, size.z / 2 + PAST),
                ),
            });
        }
        for k in &r.rocks {
            list.push(Solid {
                kind: SolidKind::Rock,
                shape: Shape::Post {
                    foot: k.at,
                    radius: k.radius,
                    height: k.height,
                },
            });
        }
        Solids::indexed(list)
    }

    fn indexed(list: Vec<Solid>) -> Solids {
        let bounds: Vec<_> = list.iter().map(|s| s.bounds()).collect();
        let x0 = bounds.iter().map(|b| b.0).min().unwrap_or(0) - PAD;
        let z0 = bounds.iter().map(|b| b.1).min().unwrap_or(0) - PAD;
        let x1 = bounds.iter().map(|b| b.2).max().unwrap_or(0) + PAD;
        let z1 = bounds.iter().map(|b| b.3).max().unwrap_or(0) + PAD;
        let cell = CELL.max((x1 - x0).max(z1 - z0) / CELLS + 1);
        let nx = ((x1 - x0) / cell + 1).max(1);
        let nz = ((z1 - z0) / cell + 1).max(1);
        let mut cells = vec![Vec::new(); (nx * nz) as usize];
        for (k, b) in bounds.iter().enumerate() {
            let (cx0, cz0) = ((b.0 - PAD - x0) / cell, (b.1 - PAD - z0) / cell);
            let (cx1, cz1) = ((b.2 + PAD - x0) / cell, (b.3 + PAD - z0) / cell);
            for cz in cz0.max(0)..=cz1.min(nz - 1) {
                for cx in cx0.max(0)..=cx1.min(nx - 1) {
                    cells[(cz * nx + cx) as usize].push(k as u32);
                }
            }
        }
        Solids {
            list,
            x0,
            z0,
            cell,
            nx,
            nz,
            cells,
        }
    }

    /// The solids listed in the square `p` is in (every solid within
    /// `PAD` of `p` is).
    pub fn near(&self, p: V3) -> &[u32] {
        let (cx, cz) = ((p.x - self.x0) / self.cell, (p.z - self.z0) / self.cell);
        if p.x < self.x0 || p.z < self.z0 || cx >= self.nx || cz >= self.nz {
            return &[];
        }
        &self.cells[(cz * self.nx + cx) as usize]
    }

    /// The first touch of a body moving from `p0` to `p1`, and which solid.
    fn first(&self, p0: V3, p1: V3) -> Option<(Touch, usize)> {
        let d = p1.sub(p0);
        let mut best: Option<(Touch, usize)> = None;
        let mut try_one = |k: usize| {
            if let Some(t) = self.list[k].sweep(p0, p1) {
                if best.is_none_or(|(b, _)| t.t < b.t) {
                    best = Some((t, k));
                }
            }
        };
        if d.x.abs() < MOVE && d.z.abs() < MOVE {
            for &k in self.near(p1) {
                try_one(k as usize);
            }
        } else {
            // (a long move, a put-back: every solid)
            for k in 0..self.list.len() {
                try_one(k);
            }
        }
        best
    }

    /// Whether the point `p` is in a solid, or within `margin` of one
    /// (`margin` under `PAD`).
    pub fn holds(&self, p: V3, margin: i32) -> bool {
        self.near(p)
            .iter()
            .any(|&k| self.list[k as usize].holds(p, margin))
    }

    /// The highest solid top under a body's hands at `p`, if it stands
    /// there (its feet over it or just through it).
    pub fn top_under(&self, p: V3) -> Option<i32> {
        let feet = p.y - BODY_DROP;
        self.near(p)
            .iter()
            .map(|&k| &self.list[k as usize])
            .filter(|s| feet >= s.top() - STAND && s.over(p))
            .map(|s| s.top())
            .max()
    }

    /// A body moved from `before` to `pos` at `vel`, put back out of the
    /// solids it ran into: against the face, its speed into it gone but a
    /// little bounce (as every map's walls), or onto a top it came down
    /// on. Its new place and speed, and whether it hit a wall (not a top);
    /// `None` if it touched nothing.
    pub fn push(&self, before: V3, pos: V3, vel: V3) -> Option<(V3, V3, bool)> {
        let (mut p, mut v, mut wall, mut hit) = (pos, vel, false, false);
        // (a bend, two solids: a few passes)
        for _ in 0..3 {
            let Some((t, k)) = self.first(before, p) else {
                break;
            };
            hit = true;
            let n = t.normal;
            let s = &self.list[k];
            let at = if t.depth > 0 {
                p.add(n.scale(t.end_depth + SKIN))
            } else {
                before.add(p.sub(before).scale(t.t)).add(n.scale(SKIN))
            };
            if n.y > ONE / 2 && s.over(at) {
                // onto the top: the feet on it, where the landing finds
                // ground (`top_under`) and takes the fall's speed
                p = v3(at.x, s.top() + BODY_DROP, at.z);
            } else {
                p = at;
                let into = v.dot(n);
                if into < 0 {
                    v = v.sub(n.scale(into + mul(into, WALL_BOUNCE)));
                }
                wall = true;
            }
        }
        hit.then_some((p, v, wall))
    }
}

/// `Course.ravine_solids`: the solids, built from the layout the first
/// time they are asked for. Always equal to another (a cache: the layout
/// it comes from is compared already).
#[derive(Debug, Clone, Default)]
pub struct SolidsCache(OnceLock<Arc<Solids>>);

impl PartialEq for SolidsCache {
    fn eq(&self, _: &SolidsCache) -> bool {
        true
    }
}

impl Eq for SolidsCache {}

impl Course {
    /// Rattlesnake Ravine's solids (`None` on every other map). Built once
    /// a course, on first use: a layout changed after that (a test's) needs
    /// `ravine_solids = Default::default()` again.
    pub fn ravine_solids(&self) -> Option<&Arc<Solids>> {
        let r = self.ravine.as_deref()?;
        Some(self.ravine_solids.0.get_or_init(|| Arc::new(Solids::of(r))))
    }
}
