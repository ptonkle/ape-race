//! What moves in the store, every frame: the shoppers (each where its
//! lizard walks, `RavineLayout::lizard_at`, the same as its bouncer), their
//! carts and balloon bunches (each at its vine's anchor, `Course::anchor`,
//! the same as the physics'), and the misters' puffs over the produce.

use crate::course::Course;
use crate::fx::{int, ONE, TURN};
use crate::geom::{yaw_of, V3};
use crate::grocery::GroceryLayout;
use crate::grocerymodels as m;
use crate::ravine::RavineLayout;
use crate::render::Batch;

/// Shoppers and balloons are drawn within this of the eye.
pub const SEEN: i32 = int(300);
/// A shopper's stride: one step cycle every this many metres walked.
const STRIDE: i32 = int(12);

pub fn dynamic(
    b: &mut Batch,
    c: &Course,
    r: &RavineLayout,
    g: &GroceryLayout,
    eye: V3,
    _t: u32,
    tick: u32,
    _vivid: bool,
) {
    let near = |p: V3| !p.flat().sub(eye.flat()).longer_than(SEEN);
    for s in &g.shoppers {
        let Some(l) = r.lizards.get(s.lizard) else {
            continue;
        };
        let (feet, way) = r.lizard_at(s.lizard, tick);
        if !near(feet) {
            continue;
        }
        // (the stride from how far along the walk: the lizard eases to a
        // stop at each end, so the legs do too)
        let walked = feet.sub(l.from).flat().len();
        let stride = ((walked % STRIDE) as i64 * ONE as i64 / STRIDE as i64) as i32;
        let string = s.balloons.map(|i| c.tip(i, tick));
        m::shopper(b, feet, yaw_of(way), s.look, stride, string, s.cart);
        if let Some(i) = s.balloons {
            m::balloons(b, c.anchor(i, tick), tick, i as u32);
        }
    }
    for k in &g.misters {
        if near(k.at) {
            m::mister(b, k.at, tick, k.period, k.phase);
        }
    }
    let _ = TURN;
}
