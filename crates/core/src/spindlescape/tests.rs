//! The Spindle's art (step 8).

use super::{dynamic, scenery, space};
use crate::course::Course;
use crate::fx::{int, ONE};
use crate::geom::v3;
use crate::map::Map;
use crate::mesh::Mesh;
use crate::render::{Batch, DRAW_VALUES, INSTANCE_VALUES};
use crate::spindle::RADIUS;

fn tube() -> Course {
    Map::Spindle.course()
}

/// The frame's budget (MAPS.md): the scenery drawn at once plus what
/// moves round the eye, along the whole route and out into space.
pub const MOST_AT_ONCE: usize = 5_500;

/// The worst frame along the route and past the open end: (still, moving,
/// metres along).
pub fn worst_frame(c: &Course) -> (usize, usize, i32) {
    let s = c.spindle.as_deref().unwrap();
    let sc = scenery(c, s);
    let mut worst = (0, 0, 0);
    let mut z = 0;
    while z < s.mouth + int(160) {
        let eye = v3(0, int(8), z);
        let fog = super::look_at(c, s, eye).fog_range.1;
        let still: usize = sc
            .draws_in(eye, fog)
            .chunks(DRAW_VALUES)
            .filter(|d| d[1] != Mesh::Terrain as u32)
            .map(|d| d[3] as usize)
            .sum();
        let mut b = Batch::new();
        dynamic(&mut b, c, s, eye, 100);
        let moving = b.count() + b.take_halos().count();
        if still + moving > worst.0 + worst.1 {
            worst = (still, moving, z / ONE);
        }
        z += int(10);
    }
    worst
}

#[test]
fn the_spindle_frame_stays_within_budget() {
    let worst = worst_frame(&tube());
    assert!(
        worst.0 + worst.1 <= MOST_AT_ONCE,
        "{} still + {} moving shapes at {} m",
        worst.0,
        worst.1,
        worst.2
    );
}

#[test]
fn the_wall_curves_up_overhead() {
    // the still scenery, bent: some of the wall is overhead, a tube's
    // width up, and none of it further from the axis than the wall's back
    let c = tube();
    let sc = scenery(&c, c.spindle.as_deref().unwrap());
    let mut overhead = 0;
    for s in sc.values.chunks(INSTANCE_VALUES) {
        let (x, y) = (s[3], s[7]);
        let r = v3(x, y - RADIUS, 0).len();
        assert!(r <= RADIUS + int(8), "a shape {} m from the axis", r / ONE);
        if y > RADIUS * 2 - int(3) {
            overhead += 1;
        }
    }
    assert!(overhead > 100, "only {overhead} shapes overhead");
}

#[test]
fn the_light_goes_over_to_space_at_the_open_end() {
    let c = tube();
    let s = c.spindle.as_deref().unwrap();
    let inside = super::look_at(&c, s, v3(0, int(8), int(500)));
    let out = super::look_at(&c, s, v3(0, int(8), s.mouth + int(20)));
    assert_eq!(inside, super::look());
    assert_eq!(out.sky, space::look().sky);
    // and space is drawn only near the open end
    let mut far = Batch::new();
    space::dynamic(&mut far, &c, s, v3(0, int(8), int(500)), 100);
    assert_eq!(far.count(), 0);
    let mut near = Batch::new();
    space::dynamic(&mut near, &c, s, v3(0, int(8), s.mouth), 100);
    assert!(near.count() > 500);
}
