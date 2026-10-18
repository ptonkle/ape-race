//! The shopping list (GR-T2, GR-T3): four items, one per zone, each hung
//! in a side aisle the route passes, a spot for each racer. An ape takes
//! its own spot's item by touching it or grabbing its string; the first and
//! third it takes add `ITEM_GAIN` of its top speed, the second and fourth
//! `ITEM_GAIN` of its air steering, for the rest of the race.
//!
//! Who decides: the browser that runs the ape (as for the course's
//! bananas): `Race::step` calls this for Local and Ai apes only.

use super::{GroceryLayout, ITEM_GAIN, ITEM_REACH, LIST_ZONES};
use crate::ape::{Ape, Happening, Mode};
use crate::fx::ONE;
use crate::race::{Event, Race, Reward};
use crate::rules::BODY_RADIUS;

/// The zone of the item `a` takes this tick, if any: one of its own spot's
/// not yet taken, within `ITEM_REACH` of its middle or on the vine it has
/// just grabbed.
pub fn taken(g: &GroceryLayout, a: &Ape) -> Option<usize> {
    let at = a.middle();
    let near = ITEM_REACH + BODY_RADIUS;
    let grabbed = match a.mode {
        Mode::Swinging { vine, .. } if a.happened.contains(&Happening::Grab) => Some(vine),
        _ => None,
    };
    g.items
        .iter()
        .filter(|it| it.slot == a.slot && a.list & (1 << it.zone) == 0)
        .find(|it| grabbed == Some(it.vine) || it.at.dist(at) <= near)
        .map(|it| it.zone as usize)
}

/// The `n`th item's gain (1-based): the share over the kind's own top
/// speed (odd `n`) or air steering (even `n`) before it and after it.
fn levels(n: u32) -> (i64, i64) {
    let k = n.div_ceil(2) as i64;
    let at = |k: i64| ONE as i64 + k * ITEM_GAIN as i64;
    (at(k - 1), at(k))
}

/// Ape `a` takes zone `zone`'s item: marked on its list, and its physics
/// moved on a level. Top speed under quadratic drag goes as one over the
/// square root of the drag, so the drag falls by the square of the
/// speed's gain; the swing's drive and the run go up by the gain itself.
/// Returns what it gave.
pub fn take(a: &mut Ape, zone: usize) -> Reward {
    a.list |= 1 << zone;
    let n = a.list.count_ones();
    let (was, now) = levels(n);
    let p = &mut a.phys;
    let up = |v: i32| (v as i64 * now / was) as i32;
    if n % 2 == 1 {
        p.drag = (p.drag as i64 * was * was / (now * now)) as i32;
        p.drive_fade = up(p.drive_fade);
        p.run_speed = up(p.run_speed);
        Reward::ListSpeed
    } else {
        p.air = up(p.air);
        Reward::ListAir
    }
}

/// Whether ape `a` has every item on its list.
pub fn full(a: &Ape) -> bool {
    a.list.count_ones() as usize >= LIST_ZONES
}

impl Race {
    /// Clean-Up on Aisle Ape after ape `i`'s step: the item it touched or
    /// caught, if any.
    pub(crate) fn shopping(&mut self, i: usize) {
        let Some(g) = self.course.grocery.as_deref() else {
            return;
        };
        let r = &self.racers[i];
        if r.out() {
            return;
        }
        let Some(zone) = taken(g, &r.ape) else {
            return;
        };
        let what = take(&mut self.racers[i].ape, zone);
        self.racers[i].reward = Some((what, self.tick));
        self.events.push(Event::Reward { who: i, what });
    }
}
