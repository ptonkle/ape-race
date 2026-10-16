//! Mape's Tower's stonework is solid (TW12): tower 2, tower 1's hollow
//! wall (its doorway open, its top open), the bridges' masonry under
//! their decks and the tube's wall (its mouth into the tunnel open). A
//! body that runs into stone is put back out by the way it is least far
//! in, its speed into the stone gone but a little bounce (`WALL_BOUNCE`,
//! as every map's walls); the tops are ground (`Course::support`), so
//! tower 2's crown and tower 1's rim can be stood on, and a bridge's deck
//! is the drawn slope, not a flat slab at its lower end (TL1).
//!
//! The shapes are built once with the layout (`TowerLayout.stone`): a
//! handful of drums and spans, each with a cheap box reject first.

use super::*;
use crate::fx::{self, cm, mul, ONE};
use crate::geom::right_of;
use crate::rules::{BODY_DROP, BODY_RADIUS, WALL_BOUNCE};

/// How far over the hands the head reaches (a ceiling's touch).
const HEAD: i32 = cm(30);
/// A top is ground for feet this far through it (as a platform's).
const STAND: i32 = cm(60);

/// An opening in a drum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gap {
    None,
    /// a doorway the full height of a ring: on the side `out` faces (unit,
    /// level), `half` either side of it
    Door {
        out: V3,
        half: i32,
    },
    /// a round hole through a slab, its middle (in plan) and radius
    Hole {
        at: V3,
        radius: i32,
    },
}

/// A piece of solid stone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Solid {
    /// an upright round drum (a ring when `inner > 0`) from `bottom` to
    /// `top`, its axis at `centre` (x and z)
    Drum {
        centre: V3,
        inner: i32,
        outer: i32,
        bottom: i32,
        top: i32,
        gap: Gap,
    },
    /// a bridge's masonry: under its deck from `from` to `to` (the deck's
    /// top at each end), `half` across, the arch's crown `rise` under the
    /// deck at its ends and `crown` under it at its middle
    Span {
        from: V3,
        to: V3,
        half: i32,
        rise: i32,
        crown: i32,
    },
}

/// The way out of a solid for a body, and how far.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Out {
    /// out along `n` (unit) by `by`: a wall (or a ceiling)
    Push(V3, i32),
    /// up onto its top at this height
    Onto(i32),
}

impl Solid {
    /// Whether a body with its hands at `p` (feet `BODY_DROP` under, round
    /// `BODY_RADIUS`) is in it, and the shortest way out.
    fn out(&self, p: V3) -> Option<Out> {
        let feet = p.y - BODY_DROP;
        let head = p.y + HEAD;
        match *self {
            Solid::Drum {
                centre,
                inner,
                outer,
                bottom,
                top,
                gap,
            } => {
                let reach = outer + BODY_RADIUS;
                let (dx, dz) = (p.x - centre.x, p.z - centre.z);
                if dx.abs() >= reach || dz.abs() >= reach || feet >= top || head <= bottom {
                    return None;
                }
                let rel = v3(dx, 0, dz);
                let d = rel.len();
                if d >= reach || (inner > 0 && d <= inner - BODY_RADIUS) {
                    return None;
                }
                match gap {
                    Gap::Door { out, half } => {
                        if rel.dot(out) > 0 && rel.dot(right_of(out)).abs() < half {
                            return None;
                        }
                    }
                    Gap::Hole { at, radius } => {
                        if !v3(p.x - at.x, 0, p.z - at.z).longer_than(radius) {
                            return None;
                        }
                    }
                    Gap::None => {}
                }
                let radial = if d > 0 { rel.norm() } else { v3(ONE, 0, 0) };
                let mut best = Out::Push(radial, reach - d);
                let mut least = reach - d;
                if inner > 0 && d - (inner - BODY_RADIUS) < least {
                    least = d - (inner - BODY_RADIUS);
                    best = Out::Push(radial.neg(), least);
                }
                if top - feet < least {
                    least = top - feet;
                    best = Out::Onto(top);
                }
                if bottom > 0 && head - bottom < least {
                    best = Out::Push(v3(0, -ONE, 0), head - bottom);
                }
                Some(best)
            }
            Solid::Span {
                from,
                to,
                half,
                rise,
                crown,
            } => {
                // (a box round it first: no square root for a far body)
                let w = half + BODY_RADIUS;
                if p.x < from.x.min(to.x) - w
                    || p.x > from.x.max(to.x) + w
                    || p.z < from.z.min(to.z) - w
                    || p.z > from.z.max(to.z) + w
                    || feet > from.y.max(to.y)
                    || head < from.y.min(to.y) - rise
                {
                    return None;
                }
                let run = to.sub(from).flat();
                let len = run.len().max(1);
                let fwd = run.norm();
                let rel = p.sub(from).flat();
                let along = rel.dot(fwd);
                let side = rel.dot(right_of(fwd));
                let wide = half + BODY_RADIUS;
                if along <= 0 || along >= len || side.abs() >= wide {
                    return None;
                }
                let u = fx::div(along, len);
                let deck = from.y + mul(to.y - from.y, u);
                // the arch's underside: `rise` under the deck at the ends,
                // `crown` at the middle (a half sine between)
                let s = fx::sin(mul(u, crate::fx::TURN / 2));
                let under = deck - rise + mul(rise - crown, s);
                if feet >= deck || head <= under {
                    return None;
                }
                let mut best = Out::Push(
                    if side >= 0 {
                        right_of(fwd)
                    } else {
                        right_of(fwd).neg()
                    },
                    wide - side.abs(),
                );
                let mut least = wide - side.abs();
                if deck - feet < least {
                    least = deck - feet;
                    best = Out::Onto(deck);
                }
                if head - under < least {
                    best = Out::Push(v3(0, -ONE, 0), head - under);
                }
                Some(best)
            }
        }
    }

    /// Its top under a body's hands at `p`, if it stands there (the feet
    /// above it or just through it).
    fn top_under(&self, p: V3) -> Option<i32> {
        let feet = p.y - BODY_DROP;
        match *self {
            Solid::Drum {
                centre,
                inner,
                outer,
                top,
                gap,
                ..
            } => {
                let (dx, dz) = (p.x - centre.x, p.z - centre.z);
                if dx.abs() > outer || dz.abs() > outer || feet < top - STAND {
                    return None;
                }
                let rel = v3(dx, 0, dz);
                if rel.longer_than(outer) || (inner > 0 && !rel.longer_than(inner)) {
                    return None;
                }
                if let Gap::Hole { at, radius } = gap {
                    if !v3(p.x - at.x, 0, p.z - at.z).longer_than(radius) {
                        return None;
                    }
                }
                Some(top)
            }
            Solid::Span { from, to, half, .. } => {
                if p.x < from.x.min(to.x) - half
                    || p.x > from.x.max(to.x) + half
                    || p.z < from.z.min(to.z) - half
                    || p.z > from.z.max(to.z) + half
                    || feet < from.y.min(to.y) - STAND
                {
                    return None;
                }
                let run = to.sub(from).flat();
                let len = run.len().max(1);
                let fwd = run.norm();
                let rel = p.sub(from).flat();
                let along = rel.dot(fwd);
                if along < 0 || along > len || rel.dot(right_of(fwd)).abs() > half {
                    return None;
                }
                let deck = from.y + mul(to.y - from.y, fx::div(along, len));
                (feet >= deck - STAND).then_some(deck)
            }
        }
    }
}

impl TowerLayout {
    /// The bridges' decks' indices in `Course.platforms` (a run: they are
    /// pushed together); the ground there is the drawn slope instead.
    pub fn deck_platforms(&self) -> std::ops::Range<usize> {
        let decks = || self.bridges.iter().filter_map(|b| b.deck);
        match (decks().min(), decks().max()) {
            (Some(a), Some(z)) => a..z + 1,
            _ => 0..0,
        }
    }

    /// A body at `pos` (the hands) moving at `vel` that has run into the
    /// stone: put back out of it, its speed into it gone but a little
    /// bounce. Whether it hit a wall (not only stepped onto a top).
    /// `None` if it is clear of every piece.
    pub fn stone_push(&self, pos: V3, vel: V3) -> Option<(V3, V3, bool)> {
        let mut pos = pos;
        let mut vel = vel;
        let mut touched = false;
        let mut wall = false;
        for s in &self.stone {
            let Some(out) = s.out(pos) else {
                continue;
            };
            touched = true;
            match out {
                Out::Push(n, by) => {
                    pos = pos.add(n.scale(by));
                    let into = -vel.dot(n);
                    if into > 0 {
                        vel = vel.add(n.scale(into + mul(into, WALL_BOUNCE)));
                    }
                    wall = true;
                }
                Out::Onto(top) => {
                    // (the fall is kept: the landing, `Ape::collide`'s
                    // support check, stops it and grounds the ape. Zeroed
                    // here, the ape rested on the top still "flying",
                    // unable to walk or jump: stuck on the room's floor,
                    // TW13)
                    pos.y = top + BODY_DROP;
                }
            }
        }
        touched.then_some((pos, vel, wall))
    }

    /// The highest stone top a body at `p` (the hands) stands on.
    pub fn stone_top(&self, p: V3) -> Option<i32> {
        self.stone.iter().filter_map(|s| s.top_under(p)).max()
    }

    /// Which piece of stone (its index in `stone`) a body at `p` (the
    /// hands) is in, if any (the audit, the tests).
    pub fn in_stone(&self, p: V3) -> Option<usize> {
        self.stone.iter().position(|s| s.out(p).is_some())
    }

    /// Whether the point `p` itself (no body round it) is in stone.
    pub fn point_in_stone(&self, p: V3) -> Option<usize> {
        // (a body whose hands are `p` with no radius: lift it so the
        // feet are at `p` and test the head too)
        self.stone.iter().position(|s| match *s {
            Solid::Drum {
                centre,
                inner,
                outer,
                bottom,
                top,
                gap,
            } => {
                let rel = v3(p.x - centre.x, 0, p.z - centre.z);
                let d = rel.len();
                let open = match gap {
                    Gap::Door { out, half } => {
                        rel.dot(out) > 0 && rel.dot(right_of(out)).abs() < half
                    }
                    Gap::Hole { at, radius } => !v3(p.x - at.x, 0, p.z - at.z).longer_than(radius),
                    Gap::None => false,
                };
                p.y > bottom && p.y < top && d < outer && d > inner && !open
            }
            Solid::Span { .. } => s.out(p.add(v3(0, BODY_DROP / 2, 0))).is_some(),
        })
    }
}

/// The stonework of a laid-out tower (its `stone`).
pub(super) fn stonework(t: &TowerLayout) -> Vec<Solid> {
    let mut out = Vec::new();
    // tower 1: a hollow ring, the doorway's storey open on its side, the
    // top open (the ring stops at the rim)
    let t1 = &t.towers[0];
    let d = &t.doorway;
    let (sill, lintel) = (d.at.y, d.at.y + d.height);
    for (bottom, top, gap) in [
        (t1.centre.y, sill, Gap::None),
        (
            sill,
            lintel,
            Gap::Door {
                out: d.out,
                half: d.half_width,
            },
        ),
        (lintel, t1.top(), Gap::None),
    ] {
        if top > bottom {
            out.push(Solid::Drum {
                centre: t1.centre,
                inner: t1.inner,
                outer: t1.radius,
                bottom,
                top,
                gap,
            });
        }
    }
    // tower 2: solid to its crown
    let t2 = &t.towers[1];
    out.push(Solid::Drum {
        centre: t2.centre,
        inner: 0,
        outer: t2.radius,
        bottom: t2.centre.y,
        top: t2.top(),
        gap: Gap::None,
    });
    // the bridges' masonry under their decks
    for br in &t.bridges {
        if br.deck.is_none() {
            continue;
        }
        // (a long flat deck on piers (drawing), not the old short arch:
        // the race runs under it)
        out.push(Solid::Span {
            from: br.from,
            to: br.to,
            half: (br.half_width).max(cm(150)),
            rise: br.thick + cm(40),
            crown: br.thick,
        });
    }
    // the tube's wall: its bottom ring open on the tunnel's side (the
    // mouth), the rest closed
    let tube = &t.tube;
    let mouth_top = tube.bottom + t.tunnel.height;
    out.push(Solid::Drum {
        centre: v3(tube.axis.x, 0, tube.axis.z),
        inner: tube.radius,
        outer: tube.radius + tube.wall,
        bottom: tube.bottom - crate::fx::int(2),
        top: mouth_top,
        gap: Gap::Door {
            out: t.tunnel.along,
            half: t.tunnel.half_width,
        },
    });
    out.push(Solid::Drum {
        centre: v3(tube.axis.x, 0, tube.axis.z),
        inner: tube.radius,
        outer: tube.radius + tube.wall,
        bottom: mouth_top,
        // (open at the top: the race comes in over the rim)
        top: tube.top - TUBE_RIM,
        gap: Gap::None,
    });
    out
}

/// The tube's wall stops this far under its top: the race comes in over
/// the rim.
pub const TUBE_RIM: i32 = crate::fx::int(10);
