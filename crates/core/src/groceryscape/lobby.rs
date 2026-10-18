//! Clean-Up on Aisle Ape's lobby and title flyover (step 9).
//!
//! STAND-IN (the skeleton): the ape on the start's stock cart, looking down
//! the back aisle; the flyover glides along the route under the ceiling.

use crate::course::Course;
use crate::fx::{cm, deg, int};
use crate::geom::V3;
use crate::geom::{v3, yaw_of};
use crate::grocery::CEILING_AT;
use crate::render::Camera;

/// The lobby: the ape on the start's stock cart, the back aisle ahead.
pub fn preview(c: &Course) -> (V3, i32, Camera) {
    let (feet, yaw) = c
        .grocery
        .as_deref()
        .map(|g| g.lobby)
        .unwrap_or((c.starts[0], c.start_yaw));
    let feet = v3(feet.x, feet.y - crate::rules::BODY_DROP, feet.z);
    let fwd = crate::geom::facing(yaw, 0);
    let eye = feet.add(fwd.scale(cm(290))).add(v3(0, cm(125), 0));
    let target = feet.add(v3(0, cm(62), 0));
    (
        feet,
        yaw_of(eye.sub(feet).flat()),
        Camera {
            eye,
            fwd: target.sub(eye).norm(),
            fov: deg(34),
        },
    )
}

/// The title's flyover: a slow glide along the route, 24 m up, under the
/// ceiling.
pub fn flyover_camera(c: &Course, ticks: u32, from: i32) -> Camera {
    let lap = (c.length - int(100)).max(int(1)) as i64;
    let s = ((ticks as i64 * int(6) as i64 / crate::rules::TICKS_PER_SECOND as i64 + from as i64)
        % lap) as i32
        + int(40);
    let p = c.point(s);
    let ahead = c.point(s + int(30));
    let eye = v3(p.x, int(24), p.z);
    let look = v3(ahead.x, int(21), ahead.z);
    let _ = CEILING_AT;
    Camera {
        eye,
        fwd: look.sub(eye).norm(),
        fov: deg(70),
    }
}
