//! Aisle Ape's art (step 8).

use super::course::{dynamic, scenery};
use crate::course::Course;
use crate::fx::int;
use crate::geom::v3;
use crate::map::Map;
use crate::mesh::Mesh;
use crate::render::Batch;

fn store() -> Course {
    Map::Grocery.course()
}

/// The frame's budget (MAPS.md): the scenery drawn at once plus what
/// moves round the eye, along the whole route.
pub const MOST_AT_ONCE: usize = 5_500;

/// The worst frame along the route: (still, moving, sample).
pub fn worst_frame(c: &Course) -> (usize, usize, usize) {
    use crate::render::DRAW_VALUES;
    let g = c.grocery.as_deref().unwrap();
    let s = scenery(c, g);
    let fog = super::look().fog_range.1;
    let mut worst = (0, 0, 0);
    for k in 0..c.path.len() / 5 {
        let eye = c.path[k * 5].add(v3(0, int(22), 0));
        let still: usize = s
            .draws_in(eye, fog)
            .chunks(DRAW_VALUES)
            .filter(|d| d[1] != Mesh::Terrain as u32)
            .map(|d| d[3] as usize)
            .sum();
        let mut b = Batch::new();
        dynamic(&mut b, c, g, eye, 100, 100, false);
        // (and the shopping list's items, as racer 0 sees them, GR-T2)
        super::lot::items_seen(&mut b, g, Some(0), 0, eye, 100);
        let moving = b.count() + b.take_halos().count();
        if still + moving > worst.0 + worst.1 {
            worst = (still, moving, k * 5);
        }
    }
    worst
}

#[test]
fn the_grocery_frame_stays_within_budget() {
    let worst = worst_frame(&store());
    assert!(
        worst.0 + worst.1 <= MOST_AT_ONCE,
        "{} still + {} moving shapes at sample {}",
        worst.0,
        worst.1,
        worst.2
    );
}
