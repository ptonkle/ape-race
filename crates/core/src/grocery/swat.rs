//! Shoppers swat (GR-T10): an ape that swings or flies within arm's reach
//! of a shopper's body, below their head, is swatted across the aisle (one
//! on the floor is let be: swatted where it lands, it was juggled from
//! swat to swat): let go of
//! any vine it holds and thrown sideways to the route, away from the
//! shopper, and a little up. (Their head's top stays a bouncer, as before.)
//!
//! Who decides: the browser that runs the ape (`Race::step` calls this for
//! Local and Ai apes only), as for the list.

use super::{GroceryLayout, SHOPPER_HEIGHT};
use crate::ape::Ape;
use crate::course::Course;
use crate::fx::{cm, int};
use crate::geom::{right_of, v3};
use crate::race::{Event, Race, Reward};
use crate::ravine::RavineLayout;

/// A shopper's arm reaches this far round the middle of their body ...
pub const SWAT_REACH: i32 = int(7);
/// ... and throws an ape this fast across the route, and this fast up ...
pub const SWAT_PUSH: i32 = int(22);
pub const SWAT_LIFT: i32 = int(6);
/// ... and not again for this many ticks (one swat, not one a tick).
pub const SWAT_COOL: u8 = 90;

/// The shopper (lizard) swatting ape `a` at `tick`, if one is in reach.
pub fn swatter(g: &GroceryLayout, r: &RavineLayout, a: &Ape, tick: u32) -> Option<usize> {
    if a.swat_cool > 0 || a.mode == crate::ape::Mode::Grounded {
        return None;
    }
    let at = a.middle();
    g.shoppers.iter().map(|s| s.lizard).find(|&l| {
        let (feet, _) = r.lizard_at(l, tick);
        at.y < feet.y + SHOPPER_HEIGHT - cm(200)
            && !at.flat().sub(feet.flat()).longer_than(SWAT_REACH)
    })
}

/// Ape `a` swatted by the shopper with feet at `feet`: off its vine,
/// thrown across `course` (sideways to its way there) away from them.
pub fn swat(a: &mut Ape, course: &Course, feet: crate::geom::V3) {
    if let Some(v) = a.swinging() {
        a.release(v);
    }
    let (s, _) = course.progress(a.pos, a.hint);
    let across = right_of(course.heading_at(s));
    let side = if a.pos.sub(feet).dot(across) >= 0 {
        1
    } else {
        -1
    };
    let along = a.vel.sub(across.scale(a.vel.dot(across)));
    a.vel = v3(along.x, along.y.max(0) + SWAT_LIFT, along.z).add(across.scale(side * SWAT_PUSH));
    a.swat_cool = SWAT_COOL;
}

impl Race {
    /// Aisle Ape after ape `i`'s step: a shopper in reach swats it.
    pub(crate) fn swats(&mut self, i: usize) {
        let (Some(g), Some(r)) = (
            self.course.grocery.as_deref(),
            self.course.ravine.as_deref(),
        ) else {
            return;
        };
        let tick = self.tick;
        if self.racers[i].out() {
            return;
        }
        let Some(l) = swatter(g, r, &self.racers[i].ape, tick) else {
            let a = &mut self.racers[i].ape;
            a.swat_cool = a.swat_cool.saturating_sub(1);
            return;
        };
        let (feet, _) = r.lizard_at(l, tick);
        swat(&mut self.racers[i].ape, &self.course, feet);
        self.racers[i].reward = Some((Reward::Swatted, self.tick));
        self.events.push(Event::Reward {
            who: i,
            what: Reward::Swatted,
        });
    }
}
