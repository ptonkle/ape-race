//! One ape's body and its controls (PLAN P3–P6): swinging on a vine, flying,
//! running on the ground, and being Stunned. The body's point is the ape's
//! hands; its feet hang `BODY_DROP` below.

use serde::{Deserialize, Serialize};

use crate::course::Course;
use crate::fx::{self, cm, div, isqrt, mul, ONE};
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
    /// Sector Ape: thrown off a ship or a piece of debris, elastically,
    /// with a boost
    Rebound,
    /// Sunset Ruins: flung up by a spike trap or the spikes in a pit
    Spiked,
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
    /// banana pieces collected (TRIAGE T43; T60: `BANANA_PIECES` to a
    /// banana, `whole_bananas`), up to `BANANAS_FULL` bananas: they lower the
    /// drag, so the top speed rises
    pub bananas: u32,
    /// the bananas the drag counts: `bananas` as of the last grab. A
    /// banana taken mid-swing or mid-flight counts from the next grab, so
    /// the AI's plan for a swing and its flight (made on the grab) stays
    /// exact.
    pub banana_drag: u32,
    /// climbing out of the City's river: where the hands end up (on the
    /// bank or the bridge deck)
    pub climb: Option<V3>,
    /// Sector Ape: ticks before a rebound off debris boosts again (a
    /// rebound with the boost comes once, not every tick of a touch)
    pub rebound_cool: u8,
    /// Sunset Ruins: ticks left before a spiked ape can grab again
    pub no_grab: i32,
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
            climb: None,
            rebound_cool: 0,
            no_grab: 0,
        }
    }

    /// An ape for a race on `course`: as `new`, with the map's own reach
    /// (Sector Ape's is longer: grabs at its speeds would need precision).
    pub fn new_on(
        course: &Course,
        kind: crate::rules::Kind,
        equal: bool,
        pos: V3,
        yaw: i32,
    ) -> Ape {
        let mut a = Ape::new(kind, equal, pos, yaw);
        if course.sector.is_some() {
            a.phys.reach = mul(a.phys.reach, crate::sector::SECTOR_REACH);
            a.phys.air = mul(a.phys.air, crate::sector::SECTOR_AIR);
        }
        a
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
        // (Sector Ape: vines hang every way)
        if course.sector.is_some() {
            return self.reach_any_way(course, tick);
        }
        let mut best = None;
        let mut best_d = self.phys.reach;
        let ruins = course.ruins.as_deref();
        for i in 0..course.vines.len() {
            if let Some((v, ago)) = self.last_vine {
                if v == i && ago < RELEASE_IGNORE {
                    continue;
                }
            }
            // (Sunset Ruins: a pit vine is up out of reach while its
            // trapdoor is shut)
            if ruins.is_some_and(|r| r.vine_up(i, tick)) {
                continue;
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
        if self.no_grab > 0 {
            self.no_grab -= 1;
        }

        // climbing out of the river (the City): nothing else until out
        if let Some(top) = self.climb {
            self.climb_out(top, course);
            return;
        }

        // Sunset Ruins: a pit vine drawn up with its trapdoor shut lets go
        if let (Mode::Swinging { vine, .. }, Some(r)) = (self.mode, course.ruins.as_deref()) {
            if r.vine_up(vine, tick) {
                self.release(vine);
            }
        }

        // let go
        if let Mode::Swinging { vine, .. } = self.mode {
            if !c.grab || stunned {
                self.release(vine);
                // (Sunset Ruins: the rope bridge's boost)
                if course.ruins.is_some() {
                    self.bridge_boost(vine, course);
                }
                // (Sector Ape: an odd vine's slingshot)
                if course.sector.is_some() {
                    self.sling(vine, course);
                }
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
            // (on skis a hop keeps it all: the glide is the ground's speed)
            if flat.len() > run && !self.on_snow(course) {
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
        if c.grab && !stunned && self.no_grab == 0 && self.swinging().is_none() {
            if let Some(v) = self.vine_in_reach(course, tick) {
                self.grab(v, course, tick);
            }
        }

        let wish = if stunned { V3::ZERO } else { wish(c) };
        let before = self.pos;
        match self.mode {
            Mode::Swinging { vine, rope } => self.swing(vine, rope, wish, course, tick),
            Mode::Flying => self.fly(wish, course),
            Mode::Grounded => self.run(wish, course),
        }
        self.collide(course, tick, before);
        if course.sector.is_some() {
            self.vel = self.vel.clamp_len(crate::sector::SECTOR_TOP);
        }
        if self.vel.flat().longer_than(ONE) {
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
    fn drive_now(&self, odd: bool) -> i32 {
        let fade = self.phys.drive_fade.max(ONE);
        // (Sector Ape: an odd vine's drive fades much later)
        let fade = if odd {
            mul(fade, crate::sector::ODD_FADE)
        } else {
            fade
        };
        let speed = self.vel.len().min(fade);
        mul(self.phys.pump, fx::div(fade - speed, fade))
    }

    /// The drag now, in hundred-millionths per metre: the kind's, less up
    /// to `BANANAS_DRAG_CUT` thousandths of it for the bananas collected.
    pub fn drag_now(&self) -> i64 {
        let full = (BANANAS_FULL * BANANA_PIECES) as i64;
        let n = (self.banana_drag as i64).min(full);
        self.phys.drag as i64 * (1000 - BANANAS_DRAG_CUT * n / full)
    }

    /// The whole bananas collected (TRIAGE T60), as the HUD counts them.
    pub fn whole_bananas(&self) -> u32 {
        self.bananas / BANANA_PIECES
    }

    fn drag(&mut self, course: &Course) {
        let speed = self.vel.len();
        // per second: drag / 100 000 × speed, as a share of the velocity
        let k = (self.drag_now() * speed as i64 / 100_000_000) as i32;
        // (Sector Ape: thinner in open space)
        let k = match course.sector {
            Some(_) => mul(k, crate::sector::SPACE_DRAG),
            None => k,
        };
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
            let drive = self.drive_now(course.odd_vine(vine));
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
        self.drag(course);
        let before = self.pos;
        let speed2 = self.vel.dot64(self.vel);
        let gravity = course.gravity();
        self.vel.y -= mul(gravity, DT);
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
            let target = speed2 - 2 * gravity as i64 * dh;
            if target > 0 && self.vel != V3::ZERO {
                let s = isqrt(target as u128) as i32;
                self.vel = self.vel.with_len(s);
            }
        }
    }

    fn fly(&mut self, wish: V3, course: &Course) {
        // steering in the air: across the motion, or braking, never faster
        let mut acc = wish.scale(self.phys.air);
        let flat = self.vel.flat();
        // (no wish, nothing to take off: `acc` is zero)
        if wish != V3::ZERO && flat.longer_than(ONE) {
            let dir = flat.norm();
            let along = acc.dot(dir);
            if along > 0 {
                acc = acc.sub(dir.scale(along));
            }
        }
        self.vel = self.vel.add(acc.scale(DT));
        self.drag(course);
        self.vel.y -= mul(course.gravity(), DT);
        self.pos = self.pos.add(self.vel.scale(DT));
    }

    /// Whether the ape is swimming in the City's river.
    pub fn swimming(&self, course: &Course) -> bool {
        self.mode == Mode::Grounded
            && self.climb.is_none()
            && course
                .water(self.pos)
                .is_some_and(|level| self.feet().y < level)
    }

    /// In the river: swim (the river's running) toward the nearest way out
    /// unless WASD say otherwise, and climb out on reaching a bank or a
    /// ladder; a boat bounces the swimmer up (as a mushroom). On Sunset
    /// Ruins only WASD steer: with no keys the current alone carries the
    /// swimmer, downstream (the AI swims for a bank with keys of its own).
    fn swim(&mut self, wish: V3, course: &Course) {
        let exit = course.water_exit(self.pos);
        let dir = match exit {
            _ if wish != V3::ZERO => wish,
            _ if course.ruins.is_some() => V3::ZERO,
            Some((to, _)) if to.flat().dist(self.pos.flat()) > cm(20) => {
                to.flat().sub(self.pos.flat()).norm()
            }
            _ => V3::ZERO,
        };
        let flat = self.vel.flat();
        let target = dir.scale(SWIM_SPEED);
        // (Sunset Ruins: the current carries the swimmer downstream)
        let target = match course.ruins.as_deref() {
            Some(r) => target.add(
                r.river
                    .water_at(self.pos.x, self.pos.z)
                    .map_or(V3::ZERO, |w| w.current),
            ),
            None => target,
        };
        let diff = target.sub(flat);
        let step = mul(WATER_DRAG, DT);
        let next = if diff.len() <= step {
            target
        } else {
            flat.add(diff.with_len(step))
        };
        self.vel = v3(next.x, 0, next.z);
        self.pos = self.pos.add(self.vel.scale(DT));
        if let Some((to, Some(top))) = exit {
            if to.flat().dist(self.pos.flat()) <= CLIMB_REACH + BODY_RADIUS {
                self.climb = Some(top);
            }
        }
    }

    /// Up the bank or the ladder, then a step onto the top.
    fn climb_out(&mut self, top: V3, course: &Course) {
        let step = mul(CLIMB_SPEED, DT);
        if self.pos.y < top.y {
            self.pos.y = (self.pos.y + step).min(top.y);
            self.vel = v3(0, CLIMB_SPEED, 0);
        } else {
            let d = top.flat().sub(self.pos.flat());
            if d.len() <= step {
                self.pos = top;
                self.vel = V3::ZERO;
                self.climb = None;
            } else {
                let m = d.with_len(step);
                self.pos = self.pos.add(m);
                self.vel = m.scale(int_speed(TICKS_PER_SECOND));
            }
        }
        self.mode = Mode::Grounded;
        self.hint = course.nearest(self.pos, self.hint);
    }

    /// Whether the ape stands on the Ski Slope's snow (on skis), not on a
    /// platform or another map's ground.
    pub fn on_snow(&self, course: &Course) -> bool {
        course.ski.is_some() && course.support(self.pos) <= course.terrain(self.pos.x, self.pos.z)
    }

    fn run(&mut self, wish: V3, course: &Course) {
        if self.swimming(course) {
            self.swim(wish, course);
            return;
        }
        if self.on_snow(course) {
            self.ski(wish, course);
            return;
        }
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

    /// On skis (the Ski Slope): gravity pulls the ape down the hill, the
    /// snow and the air hold it back, WASD across the motion carve it
    /// round, W pushes a little and S brakes. It glides on the snow.
    fn ski(&mut self, wish: V3, course: &Course) {
        let terrain = course.terrain;
        let down = terrain.downhill(self.pos.x, self.pos.z);
        // gravity along the slope, as a pull on the ground: g·grade/(1+grade²)
        let grade = down.len();
        let pull = down.scale(div(GRAVITY, ONE + mul(grade, grade)));
        let mut v = self.vel.flat();
        let speed = v.len();
        // (a wish against the motion brakes first, down to a near stop)
        let against = speed > cm(50) && wish.dot(v) < -mul(speed, ONE / 2);
        if speed < SKI_SKATE && wish != V3::ZERO && !against {
            // skating off from a stop, as running
            let target = wish.scale(SKI_SKATE);
            let diff = target.sub(v);
            let step = mul(RUN_ACCEL, DT);
            v = if diff.len() <= step {
                target
            } else {
                v.add(diff.with_len(step))
            };
        } else if speed > ONE / 10 && wish != V3::ZERO {
            let dir = v.norm();
            let along = wish.dot(dir);
            let across = wish.sub(dir.scale(along));
            if across.len() > ONE / 20 {
                let turn = across.with_len(mul(SKI_STEER, across.len().min(ONE)));
                v = v.add(turn.scale(DT)).with_len(speed);
            }
            if along > 0 {
                let push = mul(SKI_PUSH, div((SKI_PUSH_FADE - speed).max(0), SKI_PUSH_FADE));
                v = v.add(dir.scale(mul(mul(push, along), DT)));
            } else if along < 0 {
                let s = v.len();
                v = v.with_len(fx::approach(s, 0, mul(mul(SKI_BRAKE, -along), DT)));
            }
        }
        v = v.add(pull.scale(DT));
        let s = v.len();
        let loss = mul(SKI_FRICTION + mul(SKI_DRAG, s), DT);
        v = if s > loss {
            v.with_len(s - loss)
        } else {
            V3::ZERO
        };
        self.vel = v3(v.x, 0, v.z);
        self.drag(course);
        self.vel.y = 0;
        let p = self.pos.add(self.vel.scale(DT));
        // glued to the snow
        self.pos = v3(p.x, course.terrain(p.x, p.z) + BODY_DROP, p.z);
    }

    fn collide(&mut self, course: &Course, tick: u32, before: V3) {
        self.hint = course.nearest(self.pos, self.hint);
        let (p, v, wall) = course.contain(self.pos, self.vel, self.hint);
        self.pos = p;
        self.vel = v;
        if wall {
            self.happened.push(Happening::Wall);
        }
        // Sector Ape: the ships, the debris and the door
        if course.sector.is_some() {
            self.hit_bodies(course, tick, before);
        }
        // a bounce mushroom throws a falling or running ape back up
        // toward the vines (TRIAGE T5)
        if self.swinging().is_none() && self.vel.y <= 0 {
            if let Some(m) = course.bouncer_under(self.pos, tick) {
                let top = course.bouncer(m, tick).top.y;
                self.pos.y = self.pos.y.max(top + BODY_DROP);
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
        // (Sunset Ruins: in the air, well over anything that could hold
        // it, there is nothing to land on, RT7)
        if course.ruins.is_some() && self.mode != Mode::Grounded && course.ruins_clear(self.pos) {
            self.spikes(course, tick);
            return;
        }
        let support = match course.ruins {
            Some(_) => course.support_at(self.pos, tick),
            None => course.support(self.pos),
        };
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
                // ... or a splash into the City's river, which takes the
                // speed instead (see `swim`)
                let splash = course.water(self.pos).is_some();
                if self.mode == Mode::Flying && self.on_snow(course) {
                    // on skis a landing keeps the speed along the slope
                    // (only the part into the snow is lost)
                    let n = course.terrain.normal(self.pos.x, self.pos.z);
                    let along = self.vel.reject(n);
                    let f = along.flat();
                    if f.len() > ONE / 20 {
                        self.vel = f.with_len(along.len());
                    }
                } else if self.mode == Mode::Flying && flat.len() > ONE && !self.hopped && !splash {
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
        // Sunset Ruins: the spike traps and the pits' spikes
        if course.ruins.is_some() {
            self.spikes(course, tick);
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
        self.bananas = self.bananas.saturating_sub(BANANAS_DROPPED * BANANA_PIECES);
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
    // (no keys, no wish; and with no sideways key no need of the right: the
    // same sums, cheaper, for the AI's copies of the ape)
    if c.fwd == 0 && c.side == 0 {
        return V3::ZERO;
    }
    let f = facing(c.yaw, 0);
    let r = if c.side == 0 { V3::ZERO } else { right_of(f) };
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

mod space;
pub use space::{HIT_RADIUS, REBOUND_BACK};

#[cfg(test)]
mod ski_tests;
#[cfg(test)]
mod tests;
