//! The Spindle's rules, in the flat frame (`spindle.rs`): round the tube x
//! wraps, the axle is the ceiling, blocks are solid and their tops floors,
//! slime throws a landing ape back up, and the spinning rings' columns push
//! an ape round with them.

use super::{Block, BlockKind, SpindleLayout, AXLE_UNDER};
use crate::ape::{Ape, Happening, Mode};
use crate::fx::{cm, mul, ONE};
use crate::geom::{v3, V3};
use crate::rules::{BODY_DROP, BODY_RADIUS, MUSHROOM_BOUNCE, WALL_BOUNCE};

/// A body's feet within this of a block's top stand on it (as a
/// platform's, `Course::support`).
pub const ON_TOP: i32 = cm(60);
/// A slime block's sides throw an ape off this much harder than a wall.
pub const SLIME_SIDE: i32 = ONE * 9 / 10;

impl SpindleLayout {
    /// The top of the highest still block under a body at `p` (the hands)
    /// that holds it: its feet over the block and above (or just into) its
    /// top.
    pub fn top_under(&self, p: V3) -> Option<i32> {
        let feet = p.y - BODY_DROP;
        self.blocks
            .iter()
            .filter(|b| {
                p.x >= b.lo.x
                    && p.x <= b.hi.x
                    && p.z >= b.lo.z
                    && p.z <= b.hi.z
                    && feet >= b.hi.y - ON_TOP
            })
            .map(|b| b.hi.y)
            .max()
    }

    /// The block whose box a body at `p` is in (not on its top), and the way
    /// out of it with the least move: (block, outward normal, depth).
    fn block_hit(&self, p: V3) -> Option<(&Block, V3, i32)> {
        let r = BODY_RADIUS;
        let feet = p.y - BODY_DROP;
        self.blocks.iter().find_map(|b| {
            if p.x <= b.lo.x - r
                || p.x >= b.hi.x + r
                || p.z <= b.lo.z - r
                || p.z >= b.hi.z + r
                || p.y + r <= b.lo.y
                || feet >= b.hi.y - ON_TOP
            {
                return None;
            }
            let ways = [
                (v3(-ONE, 0, 0), p.x - (b.lo.x - r)),
                (v3(ONE, 0, 0), b.hi.x + r - p.x),
                (v3(0, 0, -ONE), p.z - (b.lo.z - r)),
                (v3(0, 0, ONE), b.hi.z + r - p.z),
                (v3(0, -ONE, 0), p.y + r - b.lo.y),
            ];
            let (n, d) = ways.into_iter().min_by_key(|w| w.1)?;
            Some((b, n, d))
        })
    }
}

impl Ape {
    /// The Spindle, after the ape's move (`Ape::collide`).
    pub(crate) fn hit_spindle(&mut self, s: &SpindleLayout, tick: u32) {
        // round the tube, but not while holding a vine: the rope is
        // measured from its anchor, on this side of the seam
        if self.swinging().is_none() {
            self.pos.x = s.warp.wrap_x(self.pos.x);
        }
        if s.in_space(self.pos) {
            return;
        }
        // the axle's underside is the ceiling
        let top = AXLE_UNDER - BODY_RADIUS;
        if self.pos.y > top {
            self.pos.y = top;
            self.vel.y = self.vel.y.min(0);
        }
        // the blocks' sides and undersides
        if let Some((b, n, d)) = s.block_hit(self.pos) {
            let bounce = if b.kind == BlockKind::Slime {
                SLIME_SIDE
            } else {
                WALL_BOUNCE
            };
            self.pos = self.pos.add(n.scale(d));
            let into = -self.vel.dot(n);
            if into > 0 {
                self.vel = self.vel.add(n.scale(into + mul(into, bounce)));
            }
            self.wall_hit();
        }
        // the spinning rings' columns, carrying the ape round with them
        for sp in &s.spinners {
            let r = BODY_RADIUS;
            let dz = self.pos.z - sp.z;
            if dz.abs() >= sp.depth / 2 + r {
                continue;
            }
            for k in 0..sp.count {
                let dx = s.warp.wrap_x(self.pos.x - sp.x_at(k, tick));
                let hx = sp.width / 2 + r;
                if dx.abs() >= hx {
                    continue;
                }
                let out_x = hx - dx.abs();
                let out_z = sp.depth / 2 + r - dz.abs();
                if out_x <= out_z {
                    let side = if dx >= 0 { ONE } else { -ONE };
                    self.pos.x += mul(side, out_x);
                    let carry = sp.speed();
                    // pushed round at the column's speed at least
                    if (carry > 0) == (side > 0) && mul(self.vel.x, side) < carry.abs() {
                        self.vel.x = carry;
                    } else if mul(self.vel.x, side) < 0 {
                        self.vel.x = -mul(self.vel.x, WALL_BOUNCE);
                    }
                } else {
                    let side = if dz >= 0 { ONE } else { -ONE };
                    self.pos.z += mul(side, out_z);
                    if mul(self.vel.z, side) < 0 {
                        self.vel.z = -mul(self.vel.z, WALL_BOUNCE);
                    }
                }
                self.wall_hit();
            }
        }
        // slime: a body falling onto its top is thrown back up
        if self.swinging().is_none() && self.vel.y <= 0 {
            let feet = self.pos.y - BODY_DROP;
            let slime = s.blocks.iter().find(|b| {
                b.kind == BlockKind::Slime
                    && self.pos.x >= b.lo.x
                    && self.pos.x <= b.hi.x
                    && self.pos.z >= b.lo.z
                    && self.pos.z <= b.hi.z
                    && feet <= b.hi.y + cm(20)
                    && feet >= b.hi.y - ON_TOP
            });
            if let Some(b) = slime {
                self.pos.y = b.hi.y + BODY_DROP;
                self.vel.y = MUSHROOM_BOUNCE;
                self.mode = Mode::Flying;
                self.happened.push(Happening::Bounce);
            }
        }
    }

    /// Ran into a block or a column: a swing into it lets go (the rope
    /// would only press the ape in again), as the tower map's stone.
    fn wall_hit(&mut self) {
        self.happened.push(Happening::Wall);
        if let Mode::Swinging { vine, .. } = self.mode {
            self.mode = Mode::Flying;
            self.last_vine = Some((vine, 0));
            self.release_bonus = 0;
        }
    }
}
