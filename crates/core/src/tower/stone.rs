//! Mape's Tower's stonework is solid (TW8): the keep, the sub-towers and
//! their roofs, the trampoline room's shell (its doorway and roof hole
//! open), the bridges' masonry under their decks and the topsy towers
//! hanging in the zipper. A body that runs into stone is put back out by
//! the way it is least far in, its speed into the stone gone but a little
//! bounce (`WALL_BOUNCE`, as every map's walls); the tops are ground
//! (`Course::support`), so the keep's crown, a sub-tower's roof walk and
//! the room's roof can be stood on, and a bridge's deck is the drawn
//! slope, not a flat slab at its lower end (TL1).
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
    /// a leaning round pillar from `a` to `b` (a topsy tower), radius `r`
    Pillar { a: V3, b: V3, r: i32 },
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
            Solid::Pillar { a, b, r } => {
                // the body as a ball round its middle
                let m = p.sub(v3(0, BODY_DROP / 2, 0));
                let reach = r + BODY_RADIUS + BODY_DROP / 2;
                let lo = v3(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
                let hi = v3(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));
                if m.x < lo.x - reach
                    || m.x > hi.x + reach
                    || m.y < lo.y - reach
                    || m.y > hi.y + reach
                    || m.z < lo.z - reach
                    || m.z > hi.z + reach
                {
                    return None;
                }
                let ab = b.sub(a);
                let len = ab.len().max(1);
                let u = ab.norm();
                let t = m.sub(a).dot(u).clamp(0, len);
                let q = a.add(u.scale(t));
                let off = m.sub(q);
                let d = off.len();
                if d >= reach {
                    return None;
                }
                let n = if d > 0 { off.norm() } else { v3(ONE, 0, 0) };
                Some(Out::Push(n, reach - d))
            }
            Solid::Span {
                from,
                to,
                half,
                rise,
                crown,
            } => {
                let run = to.sub(from).flat();
                let len = run.len().max(1);
                let reach = len / 2 + half + BODY_RADIUS;
                let mid = from.add(to).scale(ONE / 2);
                if (p.x - mid.x).abs() > reach || (p.z - mid.z).abs() > reach {
                    return None;
                }
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
            Solid::Pillar { .. } => None,
            Solid::Span { from, to, half, .. } => {
                let run = to.sub(from).flat();
                let len = run.len().max(1);
                let mid = from.add(to).scale(ONE / 2);
                let reach = len / 2 + half;
                if (p.x - mid.x).abs() > reach || (p.z - mid.z).abs() > reach {
                    return None;
                }
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
        let mut decks = self.branches.iter().filter_map(|b| b.deck);
        match (decks.next(), decks.last()) {
            (Some(a), Some(z)) => a..z + 1,
            (Some(a), None) => a..a + 1,
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
                    pos.y = top + BODY_DROP;
                    vel.y = vel.y.max(0);
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
            Solid::Pillar { a, b, r } => {
                let ab = b.sub(a);
                let len = ab.len().max(1);
                let u = ab.norm();
                let t = p.sub(a).dot(u).clamp(0, len);
                !p.sub(a.add(u.scale(t))).longer_than(r)
            }
            Solid::Span { .. } => s.out(p.add(v3(0, BODY_DROP / 2, 0))).is_some(),
        })
    }
}

/// The stonework of a laid-out tower (its `stone`).
pub(super) fn stonework(t: &TowerLayout) -> Vec<Solid> {
    let mut out = Vec::new();
    let k = &t.keep;
    out.push(Solid::Drum {
        centre: k.centre,
        inner: 0,
        outer: k.radius,
        bottom: 0,
        top: k.height,
        gap: Gap::None,
    });
    let room = &t.room;
    let mut subs: Vec<SubTower> = Vec::new();
    for br in &t.branches {
        if !subs.iter().any(|s| s.centre == br.sub.centre) {
            subs.push(br.sub);
        }
    }
    for (i, s) in subs.iter().enumerate() {
        let is_room = s.centre.flat() == room.centre.flat();
        if is_room {
            let floor = room.centre.y;
            let roof = floor + room.height;
            let out_door = room.door.sub(room.centre).flat().norm();
            out.push(Solid::Drum {
                centre: s.centre,
                inner: 0,
                outer: s.radius,
                bottom: 0,
                top: floor,
                gap: Gap::None,
            });
            out.push(Solid::Drum {
                centre: s.centre,
                inner: s.radius - ROOM_WALL,
                outer: s.radius,
                bottom: floor,
                top: roof - ROOF_THICK,
                gap: Gap::Door {
                    out: out_door,
                    half: crate::towermodels::side_of(s.radius) / 2,
                },
            });
            out.push(Solid::Drum {
                centre: s.centre,
                inner: 0,
                outer: s.radius,
                bottom: roof - ROOF_THICK,
                top: roof,
                gap: Gap::Hole {
                    at: room.roof_exit.0,
                    radius: room.roof_exit.1,
                },
            });
        } else {
            // the drum, the lit room on posts under the roof, and the
            // conical roof in three steps (`towermodels::sub_tower`)
            let base = s.top + crate::fx::int(3);
            out.push(Solid::Drum {
                centre: s.centre,
                inner: 0,
                outer: s.radius,
                bottom: s.centre.y,
                top: base,
                gap: Gap::None,
            });
            let eave = s.radius + crate::fx::int(1);
            let tall = mul(s.radius, crate::fx::ratio(22, 10)) + crate::fx::int(2);
            for n in 0..3 {
                out.push(Solid::Drum {
                    centre: s.centre,
                    inner: 0,
                    outer: eave * (5 - 2 * n) / 6,
                    bottom: base + tall * n / 3,
                    top: base + tall * (n + 1) / 3,
                    gap: Gap::None,
                });
            }
        }
        // its topsy-turvy twin hung in the zipper (`towerscape::art`)
        let hang = t.topsy_hang(s);
        let n = ((s.top - s.centre.y) / crate::towermodels::SECTION_H).clamp(1, 2);
        let g = Affine::rot_y(i as i32 * 11_000)
            .then(&Affine::rot_y(crate::fx::deg(17) * i as i32))
            .then(&Affine::rot_z(crate::fx::HALF + TOPSY_TILT));
        let down = g.apply_dir(v3(0, ONE, 0)).norm();
        let long = crate::towermodels::SECTION_H * n + crate::fx::int(3);
        out.push(Solid::Pillar {
            a: hang,
            b: hang.add(down.scale(long)),
            r: TOPSY_R,
        });
    }
    for br in &t.branches {
        if br.deck.is_none() {
            continue;
        }
        let len = br.to.sub(br.from).flat().len();
        out.push(Solid::Span {
            from: br.from,
            to: br.to,
            half: (br.half_width).max(cm(150)),
            rise: (len / 3).max(crate::fx::int(3)) + cm(40),
            crown: cm(160),
        });
    }
    out
}

use crate::geom::Affine;
/// The room's wall and roof thickness.
pub const ROOM_WALL: i32 = crate::fx::int(1);
pub const ROOF_THICK: i32 = crate::fx::int(1);
/// A topsy tower's radius (as drawn, its crown's parapet).
const TOPSY_R: i32 = crate::fx::int(8) + cm(60);
