//! The chase camera (PLAN P10): third person, behind the ape. The mouse
//! turns it; when the mouse rests it eases back behind the direction of
//! travel. Its field of view and distance grow with speed, for the sense of
//! speed. It is updated once a tick, and drawn between ticks by blending the
//! last two.

use crate::fx::{self, cm, deg, int, mul, ONE};
use crate::geom::{facing, yaw_of, V3};
use crate::render::Camera;
use crate::ski::Terrain;
use crate::skiscape::Crown;
use std::sync::Arc;

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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// how fast the map runs (Q16; ONE on every map but Sector Ape, about
    /// twice as fast: `SPACE_PACE`): the view widens up to `FAST` times
    /// this, and further than elsewhere
    pub pace: i32,
    /// the trees the eye keeps out of (the Ski Slope's conifers; none on
    /// the other maps), SECOND-ORDER SW9
    pub crowns: Arc<[Crown]>,
    /// how far from the ape the trees let the eye be: eased in as a tree
    /// comes between them and out again after (`FREE` while none does)
    pub reach: i32,
    /// how fast the reach is changing (a tick)
    pub reach_speed: i32,
}

/// The reach while no tree is in the way.
const FREE: i32 = i32::MAX;
/// How near a tree the eye may come: a little, so the view's near plane
/// does not cut into the needles; and the berth it eases in to keep.
const TREE_GAP: i32 = cm(60);
const TREE_BERTH: i32 = cm(300);
/// A tree this near the ape is not kept out of.
pub const NEAR_APE: i32 = cm(200);
/// How far ahead (s) the eye looks for trees coming.
const LOOK_AHEAD: i32 = ONE * 3 / 10;
/// How the reach glides in ahead of a tree and out after it: the most its
/// speed changes a tick, and its top speed (a tick).
const PULL_IN: (i32, i32) = (cm(6), cm(40));
const LET_OUT: (i32, i32) = (cm(1), cm(8));
/// The nearest the eye eases in to the ape for a tree.
const NEAREST: i32 = cm(300);
/// The step along the line from the ape to the eye when looking for trees.
const TREE_STEP: i32 = cm(20);

/// Sector Ape's pace (the person: about double the other maps), and how
/// much wider its view opens at full speed.
pub const SPACE_PACE: i32 = ONE * 2;
const FOV_SPACE: i32 = deg(104);
const BACK_SPACE: i32 = cm(1500);

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
            pace: ONE,
            crowns: Arc::from(Vec::new()),
            reach: FREE,
            reach_speed: 0,
        };
        c.place(at, V3::ZERO);
        c.prev_eye = c.eye;
        c.prev_target = c.target;
        c
    }

    /// The same camera, keeping out of `crowns` (placed again at `at`, so
    /// it starts clear of them).
    pub fn with_crowns(mut self, at: V3, crowns: Arc<[Crown]>) -> Chase {
        self.crowns = crowns;
        self.eye = self.target;
        self.place(at, V3::ZERO);
        self.prev_eye = self.eye;
        self.prev_target = self.target;
        self
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
        let (fast, fov_fast, back_fast) = if self.pace == ONE {
            (FAST, FOV_FAST, BACK_FAST)
        } else {
            (mul(FAST, self.pace), FOV_SPACE, BACK_SPACE)
        };
        let k = fx::div(speed.min(fast), fast);
        // ease in the speed so a short burst does not jolt the view
        let fov = fx::lerp(FOV_REST, fov_fast, mul(k, k));
        self.fov += (fov - self.fov) / 20;
        let back = fx::lerp(BACK_REST, back_fast, k);
        self.back += (back - self.back) / 20;
        self.place(at, vel);
    }

    fn place(&mut self, at: V3, vel: V3) {
        let look = facing(self.yaw, self.pitch);
        let target = at.sub(V3::UP.scale(cm(40)));
        let behind = look.scale(self.back).neg().add(V3::UP.scale(RISE));
        let eye = self.above_ground(target.add(behind));
        // the eye trails a little, so speed shows as the ape pulling away
        let eye = if self.eye == self.target {
            eye
        } else {
            self.eye.lerp(eye, ONE * 2 / 5)
        };
        self.eye = if self.crowns.is_empty() {
            eye
        } else {
            self.clear_of_trees(target, eye, vel.scale(LOOK_AHEAD))
        };
        self.target = target;
    }

    fn above_ground(&self, mut eye: V3) -> V3 {
        let floor = self.terrain.height(eye.x, eye.z) + cm(80);
        if eye.y < floor {
            eye.y = floor;
        }
        eye
    }

    /// The eye `eye` brought in toward the ape's `target` so it stays out
    /// of the trees (SECOND-ORDER SW9): to the furthest point of the line
    /// between them outside every tree. A tree between the eye and the ape
    /// is left be. The eye glides in ahead of a tree, keeping a berth from
    /// it where it is and where it will be a moment from now (moved on by
    /// `lead`, as the ape goes), and after it glides back out; only a tree
    /// that comes faster than that brings the eye straight in.
    fn clear_of_trees(&mut self, target: V3, eye: V3, lead: V3) -> V3 {
        let line = eye.sub(target);
        let len = line.len();
        if len < TREE_STEP {
            return eye;
        }
        let dir = line.norm();
        let xs = [target.x, eye.x, target.x + lead.x, eye.x + lead.x];
        let zs = [target.z, eye.z, target.z + lead.z, eye.z + lead.z];
        let (x0, x1) = (*xs.iter().min().unwrap(), *xs.iter().max().unwrap());
        let (z0, z1) = (*zs.iter().min().unwrap(), *zs.iter().max().unwrap());
        // the trees near the lines, but not one the ape is inside
        let crowns = Arc::clone(&self.crowns);
        let near: Vec<&Crown> = crowns
            .iter()
            .filter(|c| {
                let r = c.radius * 108 / 100 + TREE_BERTH;
                c.foot.x > x0 - r
                    && c.foot.x < x1 + r
                    && c.foot.z > z0 - r
                    && c.foot.z < z1 + r
                    && !c.holds(target, 0)
            })
            .collect();
        // the gliding reach keeps a berth from them all, now and a moment
        // ahead, though never nearer the ape than `NEAREST`
        let at = len.min(self.reach);
        let later = clear_reach(target.add(lead), dir, at, &near, TREE_BERTH);
        let soft = clear_reach(target, dir, at, &near, TREE_BERTH)
            .min(later)
            .max(NEAREST.min(len));
        if soft < at {
            self.glide(at, soft, PULL_IN);
        } else {
            // clear where the eye is: back out as far as the berth allows
            let room = open_reach(target, dir, at, len, &near, TREE_BERTH);
            if room >= len && self.reach >= len - cm(5) {
                self.reach = FREE;
                self.reach_speed = 0;
            } else {
                self.glide(at, room, LET_OUT);
            }
        }
        // and the eye never goes into one, unless the ape is brushing past
        // it (the eye could only keep out of it by coming right up to the
        // ape); one the ape drew it into, it eases out of
        let was = self.eye;
        let solid: Vec<&Crown> = near
            .into_iter()
            .filter(|c| !c.holds(target, NEAR_APE) && !c.holds(was, 0))
            .collect();
        let d = clear_reach(target, dir, len.min(self.reach), &solid, TREE_GAP);
        if d < len.min(self.reach) {
            self.reach = d;
            self.reach_speed = 0;
        }
        if d >= len {
            return eye;
        }
        self.above_ground(target.add(dir.scale(d)))
    }

    /// Moves the reach from `at` toward `goal`, speeding up and slowing
    /// down within `pace` (the most it speeds up a tick, the fastest it
    /// goes), so it glides there and stops.
    fn glide(&mut self, at: i32, goal: i32, pace: (i32, i32)) {
        let (accel, top) = pace;
        let gap = goal - at;
        // the speed from which it can still stop at the goal
        let stop = fx::sqrt(mul(2 * accel, gap.abs())).min(top);
        let want = if gap < 0 { -stop } else { stop };
        // heading the other way it brakes as hard as it ever pulls in
        let grip = if (gap < 0) == (self.reach_speed < 0) {
            accel
        } else {
            accel.max(PULL_IN.0)
        };
        let v = self.reach_speed + (want - self.reach_speed).clamp(-grip, grip);
        let next = (at + v).max(0);
        if (gap < 0 && next <= goal) || (gap >= 0 && next >= goal) {
            self.reach = goal;
            self.reach_speed = 0;
        } else {
            self.reach = next;
            self.reach_speed = v;
        }
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

/// The furthest point from `from` along `dir` (a unit vector), up to
/// `len`, that is `gap` clear of every tree in `near`: its distance.
fn clear_reach(from: V3, dir: V3, len: i32, near: &[&Crown], gap: i32) -> i32 {
    let mut d = len;
    while d > 0 && near.iter().any(|c| c.holds(from.add(dir.scale(d)), gap)) {
        d = (d - TREE_STEP).max(0);
    }
    d
}

/// How far from `from` along `dir` (a unit vector) the line stays `gap`
/// clear of every tree in `near` going on out from `at`, up to `len`.
fn open_reach(from: V3, dir: V3, at: i32, len: i32, near: &[&Crown], gap: i32) -> i32 {
    let mut d = at;
    while d < len {
        let next = (d + TREE_STEP).min(len);
        if near.iter().any(|c| c.holds(from.add(dir.scale(next)), gap)) {
            return d;
        }
        d = next;
    }
    len
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
    fn on_sector_ape_the_view_keeps_widening_to_twice_the_speed() {
        // the person (2026-10-13): about double the speed, so the camera
        // widens further; every other map stops at `FAST`
        let at = v3(0, int(20), 0);
        let run = |pace: i32, speed: i32| {
            let mut c = Chase::new(at, 0, Terrain::Hills);
            c.pace = pace;
            for _ in 0..400 {
                c.follow(at, v3(0, 0, speed), v3(0, 0, ONE), true);
            }
            (c.fov, c.back)
        };
        let (jungle, _) = run(ONE, int(84));
        let (half, _) = run(SPACE_PACE, int(42));
        let (full, back) = run(SPACE_PACE, int(84));
        assert!(full > jungle + deg(8), "{} vs {}", full / ONE, jungle / ONE);
        assert!(full > half + deg(15));
        assert!(back > BACK_FAST);
        assert_eq!(run(ONE, int(84)).0, run(ONE, int(42)).0);
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
        // and above the ski hill on the Ski Slope: behind the start, where
        // the hill rises above the Jungle's ground, looking down it
        let ski = crate::ski::ski();
        let p = ski.point(-int(20));
        let at = v3(p.x, ski.terrain(p.x, p.z) + int(1), p.z);
        let mut c = Chase::new(at, 0, ski.terrain);
        c.turn(0, deg(40));
        for _ in 0..50 {
            c.follow(at, V3::ZERO, v3(0, 0, ONE), true);
        }
        assert!(c.eye.y >= ski.terrain(c.eye.x, c.eye.z) + cm(80) - 2);
    }
}
