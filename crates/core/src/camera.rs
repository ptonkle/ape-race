//! The chase camera (PLAN P10): third person, behind the ape. The mouse
//! turns it; when the mouse rests it eases back behind the direction of
//! travel. Its field of view and distance grow with speed, for the sense of
//! speed. It is updated once a tick, and drawn between ticks by blending the
//! last two.

use crate::course::ground;
use crate::fx::{self, cm, deg, int, mul, ONE};
use crate::geom::{facing, yaw_of, V3};
use crate::render::Camera;

/// The field of view at rest and at full speed.
const FOV_REST: i32 = deg(68);
const FOV_FAST: i32 = deg(92);
/// The speed at which the view is widest.
const FAST: i32 = int(42);
/// Distance behind and height above the ape.
const BACK_REST: i32 = cm(650);
const BACK_FAST: i32 = cm(900);
const RISE: i32 = cm(180);
/// Pitch limits.
const PITCH_MIN: i32 = -deg(70);
const PITCH_MAX: i32 = deg(40);
/// The default pitch: looking a little down.
const PITCH_REST: i32 = -deg(12);
/// After this many ticks without the mouse, the camera eases behind.
const IDLE_TICKS: u32 = 90;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chase {
    pub yaw: i32,
    pub pitch: i32,
    /// ticks since the mouse last moved
    pub idle: u32,
    pub fov: i32,
    pub back: i32,
    pub eye: V3,
    pub target: V3,
    /// last tick's eye and target, for drawing between ticks
    pub prev_eye: V3,
    pub prev_target: V3,
    pub prev_fov: i32,
}

impl Chase {
    pub fn new(at: V3, yaw: i32) -> Chase {
        let mut c = Chase {
            yaw,
            pitch: PITCH_REST,
            idle: IDLE_TICKS,
            fov: FOV_REST,
            back: BACK_REST,
            eye: at,
            target: at,
            prev_eye: at,
            prev_target: at,
            prev_fov: FOV_REST,
        };
        c.place(at);
        c.prev_eye = c.eye;
        c.prev_target = c.target;
        c
    }

    /// Mouse movement, already scaled to angle units.
    pub fn turn(&mut self, dyaw: i32, dpitch: i32) {
        if dyaw != 0 || dpitch != 0 {
            self.idle = 0;
        }
        self.yaw = fx::wrap(self.yaw + dyaw);
        self.pitch = (self.pitch + dpitch).clamp(PITCH_MIN, PITCH_MAX);
    }

    /// One tick: follow the ape at `at` moving at `vel`.
    pub fn follow(&mut self, at: V3, vel: V3, follow: bool) {
        self.prev_eye = self.eye;
        self.prev_target = self.target;
        self.prev_fov = self.fov;
        self.idle = self.idle.saturating_add(1);
        let speed = vel.len();
        let flat = vel.len_xz();
        // ease back behind the travel, faster when going fast
        if follow && self.idle > IDLE_TICKS && flat > int(3) {
            let want = yaw_of(vel);
            let diff = fx::wrap(want - self.yaw);
            let rate = (ONE / 120) + mul(ONE / 40, fx::div(speed.min(FAST), FAST));
            self.yaw = fx::wrap(self.yaw + mul(diff, rate));
            self.pitch += mul(PITCH_REST - self.pitch, ONE / 90);
        }
        let k = fx::div(speed.min(FAST), FAST);
        // ease in the speed so a short burst does not jolt the view
        let fov = fx::lerp(FOV_REST, FOV_FAST, mul(k, k));
        self.fov += (fov - self.fov) / 20;
        let back = fx::lerp(BACK_REST, BACK_FAST, k);
        self.back += (back - self.back) / 20;
        self.place(at);
    }

    fn place(&mut self, at: V3) {
        let look = facing(self.yaw, self.pitch);
        let target = at.sub(V3::UP.scale(cm(40)));
        let mut eye = target.sub(look.scale(self.back)).add(V3::UP.scale(RISE));
        let floor = ground(eye.x, eye.z) + cm(80);
        if eye.y < floor {
            eye.y = floor;
        }
        // the eye trails a little, so speed shows as the ape pulling away
        self.eye = if self.eye == self.target {
            eye
        } else {
            self.eye.lerp(eye, ONE * 2 / 5)
        };
        self.target = target;
    }

    /// The camera to draw with, `alpha` (Q16) of the way from the last tick
    /// to this one.
    pub fn camera(&self, alpha: i32) -> Camera {
        let eye = self.prev_eye.lerp(self.eye, alpha);
        let target = self.prev_target.lerp(self.target, alpha);
        let mut fwd = target.sub(eye);
        if fwd.len() < ONE / 10 {
            fwd = facing(self.yaw, self.pitch);
        }
        Camera {
            eye,
            fwd: fwd.norm(),
            fov: fx::lerp(self.prev_fov, self.fov, alpha),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::v3;

    #[test]
    fn the_ape_is_near_the_middle_of_the_view() {
        let at = v3(0, int(20), 0);
        let mut c = Chase::new(at, 0);
        for _ in 0..200 {
            c.follow(at, v3(0, 0, int(10)), true);
        }
        let cam = c.camera(ONE);
        let (x, y) = cam.screen(ONE * 16 / 9, at).unwrap();
        assert!((x - 50 * ONE).abs() < 10 * ONE, "{}", x / ONE);
        assert!((y - 50 * ONE).abs() < 25 * ONE, "{}", y / ONE);
    }

    #[test]
    fn speed_widens_the_view() {
        let at = v3(0, int(20), 0);
        let mut slow = Chase::new(at, 0);
        let mut fast = Chase::new(at, 0);
        for _ in 0..300 {
            slow.follow(at, v3(0, 0, int(2)), true);
            fast.follow(at, v3(0, 0, int(45)), true);
        }
        assert!(fast.fov > slow.fov + deg(15));
        assert!(fast.back > slow.back);
    }

    #[test]
    fn a_resting_mouse_lets_the_camera_swing_behind() {
        let at = v3(0, int(20), 0);
        let mut c = Chase::new(at, deg(90));
        for _ in 0..600 {
            c.follow(at, v3(0, 0, int(20)), true);
        }
        assert!(fx::wrap(c.yaw).abs() < deg(5), "{}", c.yaw);
        // but a moving mouse holds it
        let mut c = Chase::new(at, deg(90));
        for _ in 0..600 {
            c.turn(1, 0);
            c.follow(at, v3(0, 0, int(20)), true);
        }
        assert!(fx::wrap(c.yaw - deg(90)).abs() < deg(5));
    }

    #[test]
    fn the_eye_stays_above_the_ground() {
        let at = v3(0, int(2), 0);
        let mut c = Chase::new(at, 0);
        c.turn(0, deg(40));
        for _ in 0..50 {
            c.follow(at, V3::ZERO, true);
        }
        assert!(c.eye.y >= ground(c.eye.x, c.eye.z) + cm(80) - 2);
    }
}
