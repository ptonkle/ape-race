//! One ape's body and its controls (PLAN P3–P6): swinging on a vine, flying,
//! running on the ground, and being Stunned. The body's point is the ape's
//! hands; its feet hang `BODY_DROP` below.

use serde::{Deserialize, Serialize};

use crate::course::Course;
use crate::fx::{self, div, isqrt, mul, ONE};
use crate::geom::{facing, right_of, v3, V3};
use crate::rules::*;

/// What one tick of a player's (or the AI's) controls says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Controls {
    /// W minus S: 1, 0 or -1
    pub fwd: i32,
    /// D minus A: 1, 0 or -1
    pub side: i32,
    /// grab is held (the left mouse button)
    pub grab: bool,
    /// jump is held (Space)
    #[serde(default)]
    pub jump: bool,
    /// the item key went down this tick
    pub item: bool,
    /// the camera's heading: WASD are relative to it
    pub yaw: i32,
    /// the camera's pitch, for aiming a Banana
    pub pitch: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Flying,
    /// holding vine `vine` on a rope `rope` long
    Swinging {
        vine: usize,
        rope: i32,
    },
    Grounded,
}

/// Something that happened to an ape this tick, for sounds and the camera.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Happening {
    Grab,
    Release,
    Miss,
    Jump,
    Land,
    Wall,
    Knocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ape {
    pub kind: crate::rules::Kind,
    pub phys: Physics,
    pub pos: V3,
    pub vel: V3,
    pub mode: Mode,
    /// ticks left in a Grab Window
    pub window: i32,
    /// ticks left in a Grab Cooldown
    pub cooldown: i32,
    /// ticks left Stunned
    pub stun: i32,
    /// grab was held last tick
    held: bool,
    /// jump was held last tick
    jump_held: bool,
    /// the vine let go of last, and how many ticks ago
    last_vine: Option<(usize, i32)>,
    /// the nearest centreline sample, kept as a search hint
    pub hint: usize,
    /// where the ape faces, for drawing
    pub yaw: i32,
    pub happened: Vec<Happening>,
}

impl Ape {
    pub fn new(kind: crate::rules::Kind, equal: bool, pos: V3, yaw: i32) -> Ape {
        Ape {
            kind,
            phys: kind.race_stats(equal).physics(),
            pos,
            vel: V3::ZERO,
            mode: Mode::Grounded,
            window: 0,
            jump_held: false,
            cooldown: 0,
            stun: 0,
            held: false,
            last_vine: None,
            hint: 0,
            yaw,
            happened: Vec::new(),
        }
    }

    pub fn speed(&self) -> i32 {
        self.vel.len()
    }

    pub fn feet(&self) -> V3 {
        self.pos.sub(v3(0, BODY_DROP, 0))
    }

    /// The middle of the body, for hits.
    pub fn middle(&self) -> V3 {
        self.pos.sub(v3(0, BODY_DROP / 2, 0))
    }

    pub fn swinging(&self) -> Option<usize> {
        match self.mode {
            Mode::Swinging { vine, .. } => Some(vine),
            _ => None,
        }
    }

    /// The nearest vine in reach of the hands, if any.
    pub fn vine_in_reach(&self, course: &Course, tick: u32) -> Option<usize> {
        let mut best = None;
        let mut best_d = self.phys.reach;
        for i in 0..course.vines.len() {
            if let Some((v, ago)) = self.last_vine {
                if v == i && ago < RELEASE_IGNORE {
                    continue;
                }
            }
            let a = course.anchor(i, tick);
            // quick reject by height and distance
            if self.pos.y > a.y + self.phys.reach {
                continue;
            }
            let flat = self.pos.flat().sub(a.flat());
            if flat.x.abs() > best_d || flat.z.abs() > best_d {
                continue;
            }
            let bottom = a.y - course.vines[i].length;
            let y = self.pos.y.clamp(bottom, a.y);
            let d = self.pos.dist(v3(a.x, y, a.z));
            if d <= best_d {
                best_d = d;
                best = Some(i);
            }
        }
        best
    }

    /// One tick (PLAN P3–P6).
    pub fn step(&mut self, c: &Controls, course: &Course, tick: u32) {
        self.happened.clear();
        if self.cooldown > 0 {
            self.cooldown -= 1;
        }
        if self.stun > 0 {
            self.stun -= 1;
        }
        if let Some((v, ago)) = self.last_vine {
            self.last_vine = Some((v, ago + 1));
        }
        let stunned = self.stun > 0;
        let pressed = c.grab && !self.held;
        self.held = c.grab;

        // let go
        if let Mode::Swinging { vine, .. } = self.mode {
            if !c.grab || stunned {
                self.release(vine);
            }
        }

        // Space jumps from the ground (TRIAGE row 2: grabbing has its own
        // button, the left mouse button)
        let jumped = c.jump && !self.jump_held;
        self.jump_held = c.jump;
        if jumped && !stunned && self.mode == Mode::Grounded {
            self.vel.y = JUMP_SPEED;
            self.mode = Mode::Flying;
            self.happened.push(Happening::Jump);
        }

        // a fresh press of grab opens a Grab Window
        if pressed && !stunned && self.cooldown == 0 && self.swinging().is_none() {
            self.window = GRAB_WINDOW;
        }

        // the Grab Window: grab a vine in reach, or run out into a miss
        if self.window > 0 {
            if !c.grab || stunned {
                self.close_window();
            } else if let Some(v) = self.vine_in_reach(course, tick) {
                self.grab(v, course, tick);
            } else {
                self.window -= 1;
                if self.window == 0 {
                    self.close_window();
                }
            }
        }

        let wish = if stunned { V3::ZERO } else { wish(c) };
        match self.mode {
            Mode::Swinging { vine, rope } => self.swing(vine, rope, wish, course, tick),
            Mode::Flying => self.fly(wish),
            Mode::Grounded => self.run(wish),
        }
        self.collide(course);
        if self.vel.len_xz() > ONE {
            let target = crate::geom::yaw_of(self.vel);
            let turn = fx::wrap(target - self.yaw);
            self.yaw = fx::wrap(self.yaw + turn / 6);
        }
    }

    fn close_window(&mut self) {
        self.cooldown = GRAB_COOLDOWN;
        self.happened.push(Happening::Miss);
        self.window = 0;
    }

    fn grab(&mut self, vine: usize, course: &Course, tick: u32) {
        let a = course.anchor(vine, tick);
        let length = course.vines[vine].length;
        let to = self.pos.sub(a);
        let rope = to.len().clamp(MIN_ROPE, length);
        let n = to.norm();
        if to.len() > rope {
            self.pos = a.add(n.scale(rope));
        }
        // the speed is redirected along the swing; Grab Keep holds most of it
        let speed = self.vel.len();
        let along = self.vel.reject(n);
        let keep = mul(speed, GRAB_KEEP);
        if along.len() > ONE / 20 {
            self.vel = along.with_len(along.len().max(keep));
        } else {
            self.vel = along;
        }
        self.mode = Mode::Swinging { vine, rope };
        self.window = 0;
        self.happened.push(Happening::Grab);
    }

    pub fn release(&mut self, vine: usize) {
        // the boost is for letting go on the way up, not at the bottom
        if self.vel.y > self.vel.len() / 6 {
            self.vel = self.vel.add(self.vel.norm().scale(RELEASE_BOOST));
        }
        self.mode = Mode::Flying;
        self.last_vine = Some((vine, 0));
        self.happened.push(Happening::Release);
    }

    /// The swing's drive now: full at rest, fading to nothing at Drive
    /// Fade Speed (TRIAGE row 8).
    fn drive_now(&self) -> i32 {
        let speed = self.vel.len().min(DRIVE_FADE_SPEED);
        mul(
            self.phys.pump,
            fx::div(DRIVE_FADE_SPEED - speed, DRIVE_FADE_SPEED),
        )
    }

    fn drag(&mut self) {
        let speed = self.vel.len();
        // per second: drag / 100 000 × speed, as a share of the velocity
        let k = (self.phys.drag as i64 * speed as i64 / 100_000) as i32;
        self.vel = self.vel.sub(self.vel.scale(mul(k, DT)));
    }

    fn swing(&mut self, vine: usize, rope: i32, wish: V3, course: &Course, tick: u32) {
        let a = course.anchor(vine, tick);
        let n = self.pos.sub(a).norm();
        // WASD on a vine (TRIAGE rows 6 and 8): no pumping back and forth.
        // The wish drives the ape along the swing whichever way it is
        // moving (so W on a backswing turns it round), strongest when slow
        // and fading to nothing at Drive Fade Speed, so at speed it is the
        // line and the release that count. The part across the motion also
        // steers the swing's plane, without adding speed.
        let t = wish.reject(n);
        if t.len() > ONE / 10 {
            let drive = self.drive_now();
            self.vel = self.vel.add(t.norm().scale(mul(mul(drive, t.len()), DT)));
            let speed = self.vel.len();
            if speed > ONE {
                let dir = self.vel.norm();
                let across = t.sub(dir.scale(t.dot(dir))).reject(n);
                if across.len() > ONE / 20 {
                    let turn = across.with_len(mul(SWING_STEER, across.len().min(ONE)));
                    // turn the velocity, keeping its size
                    self.vel = self.vel.add(turn.scale(DT)).with_len(speed);
                }
            }
        }
        self.drag();
        let before = self.pos;
        let speed2 = self.vel.dot64(self.vel);
        self.vel.y -= mul(GRAVITY, DT);
        let mut p = self.pos.add(self.vel.scale(DT));
        let d = p.sub(a);
        let taut = d.len() >= rope;
        if taut {
            let n = d.norm();
            p = a.add(n.scale(rope));
            let out = self.vel.dot(n);
            if out > 0 {
                self.vel = self.vel.sub(n.scale(out));
            }
        }
        self.pos = p;
        // A taut rope does no work: the speed is what gravity alone gives
        // over the height change, so a free swing neither gains nor loses.
        if taut {
            let dh = self.pos.y as i64 - before.y as i64;
            let target = speed2 - 2 * GRAVITY as i64 * dh;
            if target > 0 && self.vel != V3::ZERO {
                let s = isqrt(target as u128) as i32;
                self.vel = self.vel.with_len(s);
            }
        }
    }

    fn fly(&mut self, wish: V3) {
        // steering in the air: across the motion, or braking, never faster
        let mut acc = wish.scale(self.phys.air);
        let flat = self.vel.flat();
        if flat.len() > ONE {
            let dir = flat.norm();
            let along = acc.dot(dir);
            if along > 0 {
                acc = acc.sub(dir.scale(along));
            }
        }
        self.vel = self.vel.add(acc.scale(DT));
        self.drag();
        self.vel.y -= mul(GRAVITY, DT);
        self.pos = self.pos.add(self.vel.scale(DT));
    }

    fn run(&mut self, wish: V3) {
        let flat = self.vel.flat();
        let speed = flat.len();
        let run = self.phys.run_speed;
        let next = if speed > run {
            // too fast for legs: the ground takes the extra
            flat.with_len(fx::approach(speed, run, mul(GROUND_FRICTION, DT)))
        } else {
            let target = wish.scale(run);
            let diff = target.sub(flat);
            let step = mul(RUN_ACCEL, DT);
            if diff.len() <= step {
                target
            } else {
                flat.add(diff.with_len(step))
            }
        };
        self.vel = v3(next.x, 0, next.z);
        self.pos = self.pos.add(self.vel.scale(DT));
    }

    fn collide(&mut self, course: &Course) {
        self.hint = course.nearest(self.pos, self.hint);
        let (p, v, wall) = course.contain(self.pos, self.vel, self.hint);
        self.pos = p;
        self.vel = v;
        if wall {
            self.happened.push(Happening::Wall);
        }
        let support = course.support(self.pos);
        let feet = self.pos.y - BODY_DROP;
        if feet <= support {
            self.pos.y = support + BODY_DROP;
            if self.vel.y < 0 {
                if self.mode == Mode::Flying && self.vel.y < -int_speed(4) {
                    self.happened.push(Happening::Land);
                }
                self.vel.y = 0;
                if self.mode == Mode::Flying {
                    self.mode = Mode::Grounded;
                }
            }
        } else if self.mode == Mode::Grounded && feet > support + ONE / 4 {
            // walked off an edge
            self.mode = Mode::Flying;
        }
    }

    /// An item's hit: keep this share of the speed, be Stunned this long,
    /// both scaled by Weight. A hit on a vine knocks the ape off it.
    pub fn hit(&mut self, keep: i32, stun: i32, push: V3) {
        let lose = mul(ONE - keep, self.phys.hit_scale).clamp(0, ONE);
        self.vel = self.vel.scale(ONE - lose).add(push);
        self.stun = self.stun.max(mul(stun, self.phys.hit_scale));
        if let Mode::Swinging { vine, .. } = self.mode {
            self.mode = Mode::Flying;
            self.last_vine = Some((vine, 0));
        }
        self.window = 0;
        self.happened.push(Happening::Knocked);
    }
}

/// Ignore the vine just let go of for this many ticks.
const RELEASE_IGNORE: i32 = secs_to_ticks(30);

const fn int_speed(n: i32) -> i32 {
    n * ONE
}

/// The direction WASD ask for, on the ground plane, from the camera's
/// heading: zero, or a unit vector.
pub fn wish(c: &Controls) -> V3 {
    let f = facing(c.yaw, 0);
    let r = right_of(f);
    let w = f
        .scale(c.fwd.signum() * ONE)
        .add(r.scale(c.side.signum() * ONE));
    if w == V3::ZERO {
        return w;
    }
    w.norm()
}

/// Kinetic plus potential energy per unit mass, Q16 (for tests).
pub fn energy(a: &Ape) -> i64 {
    (a.vel.dot64(a.vel) >> 17) + ((GRAVITY as i64 * a.pos.y as i64) >> 16)
}

/// How long a straight drop from height `h` takes, in ticks (for tests).
pub fn fall_ticks(h: i32) -> i32 {
    // t = sqrt(2h/g)
    let t = fx::sqrt(div(2 * h, GRAVITY));
    mul(t, crate::fx::int(TICKS_PER_SECOND))
}

#[cfg(test)]
mod tests;
