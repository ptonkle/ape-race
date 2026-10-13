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
    /// thrown up by a bounce mushroom, with its push forward (TRIAGE T48)
    Bounce,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ape {
    pub kind: crate::rules::Kind,
    pub phys: Physics,
    pub pos: V3,
    pub vel: V3,
    pub mode: Mode,
    /// ticks left Stunned
    pub stun: i32,
    /// jump was held last tick
    jump_held: bool,
    /// in the air from a jump of its own (not a fall from a vine or a
    /// bounce): its landing does not roll on (TRIAGE row 25)
    hopped: bool,
    /// the vine let go of last, and how many ticks ago
    last_vine: Option<(usize, i32)>,
    /// the nearest centreline sample, kept as a search hint
    pub hint: usize,
    /// where the ape faces, for drawing
    pub yaw: i32,
    pub happened: Vec<Happening>,
    /// speed the next let-go adds along the motion (TRIAGE T45: a catch-up
    /// vine's boost, set by the race on the grab; kept in the ape so the
    /// AI's copies plan with it)
    pub release_bonus: i32,
    /// bananas collected (TRIAGE T43), up to `BANANAS_FULL`: they lower
    /// the drag, so the top speed rises
    pub bananas: u32,
    /// the bananas the drag counts: `bananas` as of the last grab. A
    /// banana taken mid-swing or mid-flight counts from the next grab, so
    /// the AI's plan for a swing and its flight (made on the grab) stays
    /// exact.
    pub banana_drag: u32,
}

impl Ape {
    pub fn new(kind: crate::rules::Kind, equal: bool, pos: V3, yaw: i32) -> Ape {
        Ape {
            kind,
            phys: kind.race_stats(equal).physics(),
            pos,
            vel: V3::ZERO,
            mode: Mode::Grounded,
            jump_held: false,
            hopped: false,
            stun: 0,
            last_vine: None,
            hint: 0,
            yaw,
            happened: Vec::new(),
            release_bonus: 0,
            bananas: 0,
            banana_drag: 0,
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
        if self.stun > 0 {
            self.stun -= 1;
        }
        if let Some((v, ago)) = self.last_vine {
            self.last_vine = Some((v, ago + 1));
        }
        let stunned = self.stun > 0;

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
            // a hop out of a fast slide keeps only part of the extra speed,
            // so hopping along is no way to keep it (TRIAGE row 25)
            let flat = self.vel.flat();
            let run = self.phys.run_speed;
            if flat.len() > run {
                let kept = run + mul(flat.len() - run, HOP_KEEP);
                let f = flat.with_len(kept);
                self.vel = v3(f.x, self.vel.y, f.z);
            }
            self.hopped = true;
            self.vel.y = JUMP_SPEED;
            self.mode = Mode::Flying;
            self.happened.push(Happening::Jump);
        }

        // Holding grab catches the first vine in reach (TRIAGE row 9: no
        // timing, no Missed Grab, no cooldown).
        if c.grab && !stunned && self.swinging().is_none() {
            if let Some(v) = self.vine_in_reach(course, tick) {
                self.grab(v, course, tick);
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

    fn grab(&mut self, vine: usize, course: &Course, tick: u32) {
        let a = course.anchor(vine, tick);
        let length = course.vines[vine].length;
        let to = self.pos.sub(a);
        // The vine is taut from the moment it is caught, so the swing starts
        // at once (TRIAGE row 14: a slack catch dropped, then snapped). It
        // stretches to hands below its end rather than yanking the ape up;
        // a catch high on it slides down while swinging (see `swing`).
        let rope = to.len().clamp(ONE, length + self.phys.reach);
        let n = to.norm();
        if to.len() > rope {
            self.pos = a.add(n.scale(rope));
        }
        // the speed is redirected along the swing: all of the part across
        // the rope, and Grab Keep of the part along it, so a grab caught
        // moving across the vine keeps its speed and a mistimed one does
        // not (TRIAGE row 23)
        let speed = self.vel.len();
        let along = self.vel.reject(n);
        let lost = speed - along.len();
        let keep = along.len() + mul(lost, GRAB_KEEP);
        if along.len() > ONE / 20 {
            self.vel = along.with_len(keep);
        } else {
            self.vel = along;
        }
        self.mode = Mode::Swinging { vine, rope };
        self.hopped = false;
        self.banana_drag = self.bananas;
        self.happened.push(Happening::Grab);
    }

    pub fn release(&mut self, vine: usize) {
        // the boost is for letting go on the way up, not at the bottom
        if self.vel.y > self.vel.len() / 6 {
            self.vel = self.vel.add(self.vel.norm().scale(RELEASE_BOOST));
        }
        // a catch-up vine's boost, never past Catch-up Top (TRIAGE T45):
        // forward, level, so it carries the ape on rather than up over the
        // next vines
        let flat = self.vel.flat();
        let gain = self.release_bonus.min(CATCHUP_TOP - self.vel.len());
        if gain > 0 && flat.len() > ONE {
            self.vel = self.vel.add(flat.norm().scale(gain));
        }
        self.release_bonus = 0;
        self.mode = Mode::Flying;
        self.last_vine = Some((vine, 0));
        self.happened.push(Happening::Release);
    }

    /// The swing's drive now: full at rest (the same for every ape),
    /// fading to nothing at the ape's drive fade speed (TRIAGE rows 8, 10).
    fn drive_now(&self) -> i32 {
        let fade = self.phys.drive_fade.max(ONE);
        let speed = self.vel.len().min(fade);
        mul(self.phys.pump, fx::div(fade - speed, fade))
    }

    /// The drag now, in hundred-millionths per metre: the kind's, less up
    /// to `BANANAS_DRAG_CUT` thousandths of it for the bananas collected.
    pub fn drag_now(&self) -> i64 {
        let n = self.banana_drag.min(BANANAS_FULL) as i64;
        self.phys.drag as i64 * (1000 - BANANAS_DRAG_CUT * n / BANANAS_FULL as i64)
    }

    fn drag(&mut self) {
        let speed = self.vel.len();
        // per second: drag / 100 000 × speed, as a share of the velocity
        let k = (self.drag_now() * speed as i64 / 100_000_000) as i32;
        self.vel = self.vel.sub(self.vel.scale(mul(k, DT)));
    }

    fn swing(&mut self, vine: usize, rope: i32, wish: V3, course: &Course, tick: u32) {
        let a = course.anchor(vine, tick);
        // a short rope slides out to Min Rope while swinging: the ape slips
        // down the vine into a longer, swingier arc (TRIAGE row 14)
        let longest = MIN_ROPE.min(course.vines[vine].length);
        let sliding = rope < longest;
        let rope = if sliding {
            let r = (rope + mul(VINE_SLIDE, DT)).min(longest);
            self.mode = Mode::Swinging { vine, rope: r };
            r
        } else {
            rope
        };
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
        // while sliding the ape is held at the rope's length, so it never
        // goes slack
        let taut = d.len() >= rope || sliding;
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
            // too fast for legs: the ground takes the extra, slowly (TRIAGE
            // T5), and WASD across the slide turn it
            let dir = flat.norm();
            let across = wish.sub(dir.scale(wish.dot(dir)));
            let flat = if across.len() > ONE / 20 {
                let turn = across.with_len(mul(SLIDE_STEER, across.len().min(ONE)));
                flat.add(turn.scale(DT)).with_len(speed)
            } else {
                flat
            };
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
        // a bounce mushroom throws a falling or running ape back up
        // toward the vines (TRIAGE T5)
        if self.swinging().is_none() && self.vel.y <= 0 {
            if let Some(m) = course.mushroom_under(self.pos) {
                self.pos.y = self.pos.y.max(course.mushrooms[m].top.y + BODY_DROP);
                self.vel.y = MUSHROOM_BOUNCE;
                // and a push forward along the heading, as a reward (T48)
                let flat = self.vel.flat();
                let dir = if flat.len() > ONE {
                    flat.norm()
                } else {
                    facing(self.yaw, 0)
                };
                self.vel = self.vel.add(dir.scale(MUSHROOM_PUSH));
                self.mode = Mode::Flying;
                self.happened.push(Happening::Bounce);
                return;
            }
        }
        let support = course.support(self.pos);
        let feet = self.pos.y - BODY_DROP;
        if feet <= support {
            self.pos.y = support + BODY_DROP;
            if self.vel.y < 0 {
                if self.mode == Mode::Flying && self.vel.y < -int_speed(4) {
                    self.happened.push(Happening::Land);
                }
                // a landing rolls some of its fall on along the ground
                // rather than stopping dead (TRIAGE T5)
                // ... unless it is the end of the ape's own hop (TRIAGE row 25)
                let flat = self.vel.flat();
                if self.mode == Mode::Flying && flat.len() > ONE && !self.hopped {
                    let roll = mul(-self.vel.y, LANDING_ROLL);
                    self.vel = flat.with_len(flat.len() + roll);
                }
                self.vel.y = 0;
                if self.mode == Mode::Flying {
                    self.mode = Mode::Grounded;
                }
                self.hopped = false;
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
        self.release_bonus = 0;
        // and drops some bananas (TRIAGE T43)
        self.bananas = self.bananas.saturating_sub(BANANAS_DROPPED);
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
