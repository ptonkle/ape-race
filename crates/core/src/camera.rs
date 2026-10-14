//! The chase camera (PLAN P10): third person, behind the ape. The mouse
//! turns it; when the mouse rests it eases back behind the direction of
//! travel. Its field of view and distance grow with speed, for the sense of
//! speed. It is updated once a tick, and drawn between ticks by blending the
//! last two.

use crate::fx::{self, cm, deg, int, mul, ONE};
use crate::geom::{facing, yaw_of, V3};
use crate::render::Camera;
use crate::ski::Terrain;

/// The field of view at rest and at full speed.
const FOV_REST: i32 = deg(68);
const FOV_FAST: i32 = deg(92);
/// The speed at which the view is widest.
const FAST: i32 = int(42);
/// Distance behind and height above the ape (TRIAGE T40: further back and
/// higher since the apes are drawn at twice life size, so the view still
/// sees past your own ape).
const BACK_REST: i32 = cm(1000);
const BACK_FAST: i32 = cm(1300);
const RISE: i32 = cm(280);
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
    /// the course's ground, which the eye stays above
    pub terrain: Terrain,
}

impl Chase {
    pub fn new(at: V3, yaw: i32, terrain: Terrain) -> Chase {
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
            terrain,
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

    /// One tick: follow the ape at `at` moving at `vel`, on a course that
    /// runs along `ahead` there (a unit vector on the ground).
    pub fn follow(&mut self, at: V3, vel: V3, ahead: V3, follow: bool) {
        self.prev_eye = self.eye;
        self.prev_target = self.target;
        self.prev_fov = self.fov;
        self.idle = self.idle.saturating_add(1);
        let speed = vel.len();
        let flat = vel.len_xz();
        // Ease back to face down the course (TRIAGE row 5: on a backswing
        // the travel points backwards, which is never useful). The travel
        // bends the view only when it heads forward, and only by a third.
        if follow && self.idle > IDLE_TICKS && (flat > int(3) || ahead != V3::ZERO) {
            let course = yaw_of(ahead);
            let forward = vel.flat().dot(ahead) > 0 && flat > int(3);
            let want = if forward {
                course + fx::wrap(yaw_of(vel) - course) / 3
            } else {
                course
            };
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
        let floor = self.terrain.height(eye.x, eye.z) + cm(80);
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

/// The replay's orbit camera (TRIAGE T23): it circles the followed ape on
/// its own until the person drags it; then it stays where it was put. The
/// wheel zooms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Orbit {
    pub yaw: i32,
    pub pitch: i32,
    /// distance from the ape
    pub dist: i32,
    /// dragged by the person: no more circling on its own
    pub steered: bool,
}

/// The orbit's limits and its pace when circling on its own.
pub const ORBIT_NEAR: i32 = cm(250);
pub const ORBIT_FAR: i32 = int(40);
const ORBIT_REST: i32 = int(7);
const ORBIT_PITCH_MIN: i32 = -deg(80);
const ORBIT_PITCH_MAX: i32 = deg(30);
const ORBIT_SPIN: i32 = deg(1) / 4;
/// Angle per pixel dragged.
const ORBIT_DRAG: i32 = deg(1) / 3;

impl Default for Orbit {
    fn default() -> Orbit {
        Orbit {
            yaw: 0,
            pitch: -deg(18),
            dist: ORBIT_REST,
            steered: false,
        }
    }
}

impl Orbit {
    /// One race tick of the replay: circling, unless steered.
    pub fn tick(&mut self) {
        if !self.steered {
            self.yaw = fx::wrap(self.yaw + ORBIT_SPIN);
        }
    }

    /// A drag across the view, in pixels: right turns the view round to
    /// the right, up looks from lower down.
    pub fn drag(&mut self, dx: i32, dy: i32) {
        if dx == 0 && dy == 0 {
            return;
        }
        self.steered = true;
        self.yaw = fx::wrap(self.yaw - dx * ORBIT_DRAG);
        self.pitch = (self.pitch - dy * ORBIT_DRAG).clamp(ORBIT_PITCH_MIN, ORBIT_PITCH_MAX);
    }

    /// The wheel: forward (negative) comes nearer, by a sixth a notch.
    pub fn zoom(&mut self, notches: i32) {
        let n = notches.clamp(-5, 5);
        let mut d = self.dist;
        for _ in 0..n.abs() {
            d = if n < 0 { d - d / 6 } else { d + d / 5 };
        }
        self.dist = d.clamp(ORBIT_NEAR, ORBIT_FAR);
    }

    /// The camera looking at `at`, kept above the ground.
    pub fn camera(&self, at: V3, terrain: Terrain) -> Camera {
        let look = facing(self.yaw, self.pitch);
        let eye = at.sub(look.scale(self.dist));
        let floor = terrain.height(eye.x, eye.z) + int(1);
        let eye = V3 {
            y: eye.y.max(floor),
            ..eye
        };
        Camera {
            eye,
            fwd: at.sub(eye).norm(),
            fov: deg(65),
        }
    }
}

/// A smooth move from one view to another (TRIAGE T34): the camera glides
/// from where it was to the new view over `total` ticks, easing in and out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pan {
    pub from: Camera,
    pub left: u32,
    pub total: u32,
}

/// A pan to the next racer after a finish, and a quick one on a switch.
pub const PAN_TICKS: u32 = crate::rules::TICKS_PER_SECOND as u32 * 6 / 5;
pub const PAN_QUICK: u32 = crate::rules::TICKS_PER_SECOND as u32 / 2;

impl Pan {
    pub fn new(from: Camera, total: u32) -> Pan {
        Pan {
            from,
            left: total.max(1),
            total: total.max(1),
        }
    }

    /// One tick; false once it is over.
    pub fn tick(&mut self) -> bool {
        self.left = self.left.saturating_sub(1);
        self.left > 0
    }

    /// The view `alpha` (Q16) of the way through this tick, between the
    /// start and `to`.
    pub fn blend(&self, to: Camera, alpha: i32) -> Camera {
        let done = int((self.total - self.left) as i32) + alpha;
        let t = fx::div(done, int(self.total as i32)).clamp(0, ONE);
        let s = mul(mul(t, t), 3 * ONE - 2 * t);
        let fwd = self.from.fwd.lerp(to.fwd, s);
        Camera {
            eye: self.from.eye.lerp(to.eye, s),
            fwd: if fwd.len() < ONE / 10 {
                to.fwd
            } else {
                fwd.norm()
            },
            fov: fx::lerp(self.from.fov, to.fov, s),
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
        let mut c = Chase::new(at, 0, Terrain::Hills);
        for _ in 0..200 {
            c.follow(at, v3(0, 0, int(10)), v3(0, 0, ONE), true);
        }
        let cam = c.camera(ONE);
        let (x, y) = cam.screen(ONE * 16 / 9, at).unwrap();
        assert!((x - 50 * ONE).abs() < 10 * ONE, "{}", x / ONE);
        assert!((y - 50 * ONE).abs() < 25 * ONE, "{}", y / ONE);
    }

    #[test]
    fn speed_widens_the_view() {
        let at = v3(0, int(20), 0);
        let mut slow = Chase::new(at, 0, Terrain::Hills);
        let mut fast = Chase::new(at, 0, Terrain::Hills);
        for _ in 0..300 {
            slow.follow(at, v3(0, 0, int(2)), v3(0, 0, ONE), true);
            fast.follow(at, v3(0, 0, int(45)), v3(0, 0, ONE), true);
        }
        assert!(fast.fov > slow.fov + deg(15));
        assert!(fast.back > slow.back);
    }

    #[test]
    fn a_resting_mouse_lets_the_camera_swing_behind() {
        let at = v3(0, int(20), 0);
        let mut c = Chase::new(at, deg(90), Terrain::Hills);
        for _ in 0..600 {
            c.follow(at, v3(0, 0, int(20)), v3(0, 0, ONE), true);
        }
        assert!(fx::wrap(c.yaw).abs() < deg(5), "{}", c.yaw);
        // but a moving mouse holds it
        let mut c = Chase::new(at, deg(90), Terrain::Hills);
        for _ in 0..600 {
            c.turn(1, 0);
            c.follow(at, v3(0, 0, int(20)), v3(0, 0, ONE), true);
        }
        assert!(fx::wrap(c.yaw - deg(90)).abs() < deg(5));
    }

    #[test]
    fn a_backswing_does_not_turn_the_camera_round() {
        let at = v3(0, int(20), 0);
        let mut c = Chase::new(at, 0, Terrain::Hills);
        // swinging back: moving -z on a course that runs +z
        for _ in 0..600 {
            c.follow(at, v3(0, 0, -int(15)), v3(0, 0, ONE), true);
        }
        assert!(fx::wrap(c.yaw).abs() < deg(5), "{}", c.yaw);
        // moving forward and to the side bends it a little that way
        let mut c = Chase::new(at, 0, Terrain::Hills);
        for _ in 0..600 {
            c.follow(at, v3(int(20), 0, int(20)), v3(0, 0, ONE), true);
        }
        assert!(c.yaw > deg(10) && c.yaw < deg(20), "{}", c.yaw);
    }

    #[test]
    fn the_orbit_circles_until_dragged_and_zooms_within_limits() {
        let at = v3(0, int(20), 0);
        let mut o = Orbit::default();
        let y0 = o.yaw;
        o.tick();
        assert_ne!(o.yaw, y0, "circles on its own");
        o.drag(30, 0);
        let y1 = o.yaw;
        for _ in 0..100 {
            o.tick();
        }
        assert_eq!(o.yaw, y1, "steered: holds where it was put");
        // dragging up and down is held to its limits
        o.drag(0, -100_000);
        assert_eq!(o.pitch, ORBIT_PITCH_MAX);
        o.drag(0, 100_000);
        assert_eq!(o.pitch, ORBIT_PITCH_MIN);
        let near = o.dist;
        o.zoom(-1);
        assert!(o.dist < near);
        for _ in 0..40 {
            o.zoom(-1);
        }
        assert_eq!(o.dist, ORBIT_NEAR);
        for _ in 0..40 {
            o.zoom(1);
        }
        assert_eq!(o.dist, ORBIT_FAR);
        // it looks at the ape from its distance
        let o = Orbit::default();
        let cam = o.camera(at, Terrain::Hills);
        assert!((cam.eye.dist(at) - o.dist).abs() < cm(5));
        let (x, y) = cam.screen(ONE * 16 / 9, at).unwrap();
        assert!((x - 50 * ONE).abs() < ONE && (y - 50 * ONE).abs() < ONE);
    }

    #[test]
    fn a_pan_glides_from_the_old_view_to_the_new() {
        let from = Camera {
            eye: v3(0, int(10), 0),
            fwd: v3(0, 0, ONE),
            fov: deg(60),
        };
        let to = Camera {
            eye: v3(int(100), int(10), 0),
            fwd: v3(ONE, 0, 0),
            fov: deg(80),
        };
        let mut pan = Pan::new(from, 10);
        assert_eq!(pan.blend(to, 0).eye, from.eye, "starts where it was");
        let mut xs = Vec::new();
        while pan.tick() {
            xs.push(pan.blend(to, 0).eye.x);
        }
        assert!(xs.windows(2).all(|w| w[0] < w[1]), "{xs:?}");
        assert!(
            xs[0] < int(10) && xs[8] > int(90),
            "eases in and out: {xs:?}"
        );
        assert_eq!(pan.blend(to, 0), to, "ends on the new view");
    }

    #[test]
    fn the_eye_stays_above_the_ground() {
        let at = v3(0, int(2), 0);
        let mut c = Chase::new(at, 0, Terrain::Hills);
        c.turn(0, deg(40));
        for _ in 0..50 {
            c.follow(at, V3::ZERO, v3(0, 0, ONE), true);
        }
        assert!(c.eye.y >= crate::course::ground(c.eye.x, c.eye.z) + cm(80) - 2);
        // and above the ski hill on the Ski Slope, looking up it
        let ski = crate::ski::ski();
        let p = ski.point(int(300));
        let at = v3(p.x, ski.terrain(p.x, p.z) + int(2), p.z);
        let mut c = Chase::new(at, fx::wrap(fx::HALF), ski.terrain);
        c.turn(0, deg(40));
        for _ in 0..50 {
            c.follow(at, V3::ZERO, v3(0, 0, -ONE), true);
        }
        assert!(c.eye.y >= ski.terrain(c.eye.x, c.eye.z) + cm(80) - 2);
    }
}
