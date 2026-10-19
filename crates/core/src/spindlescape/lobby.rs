//! The Spindle's lobby and title flyover (step 9), in the flat frame (the
//! view bends them, `spindlescape::camera`).

use crate::course::Course;
use crate::fx::{cm, deg, int};
use crate::geom::{v3, yaw_of, V3};
use crate::render::Camera;

/// The lobby: the ape on the start's blocks, the camera ahead of it looking
/// back, so the tube's wall climbs up behind it and over the top.
pub fn preview(c: &Course) -> (V3, i32, Camera) {
    let feet = v3(
        c.starts[1].x,
        c.starts[1].y - crate::rules::BODY_DROP,
        c.starts[1].z,
    );
    let fwd = crate::geom::facing(c.start_yaw, 0);
    let eye = feet.add(fwd.scale(cm(330))).add(v3(0, cm(140), 0));
    let target = feet.add(v3(0, cm(90), 0));
    (
        feet,
        yaw_of(eye.sub(feet).flat()),
        Camera {
            eye,
            fwd: target.sub(eye).norm(),
            fov: deg(40),
        },
    )
}

/// The title's flyover: a slow glide down the tube, 18 m up, slowly
/// rolling round it.
pub fn flyover_camera(c: &Course, ticks: u32, from: i32) -> Camera {
    let lap = (c.length - int(100)).max(int(1)) as i64;
    let s = ((ticks as i64 * int(8) as i64 / crate::rules::TICKS_PER_SECOND as i64 + from as i64)
        % lap) as i32
        + int(40);
    let round = (ticks as i64 * int(2) as i64 / crate::rules::TICKS_PER_SECOND as i64) as i32;
    let p = c.point(s);
    let ahead = c.point(s + int(40));
    let eye = v3(p.x + round, int(18), p.z);
    let look = v3(ahead.x + round, int(14), ahead.z);
    Camera {
        eye,
        fwd: look.sub(eye).norm(),
        fov: deg(70),
    }
}
