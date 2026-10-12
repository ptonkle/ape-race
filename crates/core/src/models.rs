//! The six apes' models (PLAN P8), built from the unit meshes.
//!
//! Every ape is the same jointed figure (a torso, a head, two two-part arms
//! and two two-part legs) with its own proportions and colours from [`look`],
//! and then the details that make each kind read at racing distance: the
//! orangutan's cheek flanges and shaggy arms, the gorilla's silver back and
//! brow, the chimpanzee's ears and pale face, the gibbon's face ring and very
//! long arms, the proboscis monkey's pot belly and floppy nose, and the
//! lemur's mask, orange eyes and ringed tail.
//!
//! A model is built in its own metres around an anchor point, then scaled
//! about that anchor: the hands when swinging (so the grip stays on the
//! vine), the ground under the feet when running (so the feet stay on the
//! ground), and the body's middle when flying. Nothing here keeps state:
//! every movement (the run cycle, the nose's flop, the tail's sway) comes
//! from the pose's tick and velocity.

use crate::fx::{self, cm, deg, int, mul, HALF, ONE, QUARTER, TURN};
use crate::geom::{facing, v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{rgb, rod, shade, Batch, Camera, Rgb};
use crate::rules::{Kind, BODY_DROP};

/// How an ape is posed this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pose {
    /// the hands: the body's point (the feet are `rules::BODY_DROP` below)
    pub hands: V3,
    /// the way the body faces
    pub yaw: i32,
    /// the rope's direction from the hands up to the anchor, when swinging
    pub rope: Option<V3>,
    pub vel: V3,
    /// on the ground (running) rather than in the air
    pub grounded: bool,
    pub stunned: bool,
    /// a running count for animation (ticks)
    pub tick: u32,
    /// 100 is life size; the selection screen draws bigger
    pub scale: i32,
}

impl Pose {
    /// Standing still on the ground with the feet at `feet`, facing `yaw`.
    pub fn standing(feet: V3, yaw: i32, tick: u32) -> Pose {
        Pose {
            hands: feet.add(v3(0, BODY_DROP, 0)),
            yaw,
            rope: None,
            vel: V3::ZERO,
            grounded: true,
            stunned: false,
            tick,
            scale: 100,
        }
    }
}

// ---------------------------------------------------------------------------
// Each kind's proportions and colours

/// One kind's build. Lengths are centimetres at the kind's own size; `size`
/// then scales the whole ape (percent).
struct Look {
    size: i32,
    fur: Rgb,
    /// arms and legs
    limb: Rgb,
    /// the chest and belly patch
    chest: Rgb,
    /// the bare face
    face: Rgb,
    /// hands, feet and ears
    skin: Rgb,
    /// torso width, length (hips to shoulders) and depth
    torso: [i32; 3],
    /// shoulder joints from the middle
    shoulder: i32,
    arm: i32,
    arm_w: i32,
    leg: i32,
    leg_w: i32,
    /// head diameter
    head: i32,
    /// how far the head sits above the shoulders, percent of its size
    neck: i32,
    /// muzzle size, percent of the head (0: none)
    muzzle: i32,
    /// ear size, percent of the head (0: hidden in the fur)
    ear: i32,
    /// brow ridge depth, percent of the head (0: none)
    brow: i32,
    /// eye size, percent of the head
    eye: i32,
    /// degrees the back leans forward when running
    lean: i32,
    /// runs on its legs with its arms up, not on all fours
    upright: bool,
}

fn look(kind: Kind) -> Look {
    match kind {
        Kind::Orangutan => Look {
            size: 108,
            fur: rgb(200, 84, 28),
            limb: rgb(186, 74, 24),
            chest: rgb(170, 70, 30),
            face: rgb(122, 96, 82),
            skin: rgb(84, 64, 56),
            torso: [46, 45, 36],
            shoulder: 22,
            arm: 76,
            arm_w: 11,
            leg: 28,
            leg_w: 11,
            head: 26,
            neck: 30,
            muzzle: 48,
            ear: 0,
            brow: 10,
            eye: 17,
            lean: 30,
            upright: false,
        },
        Kind::Gorilla => Look {
            size: 122,
            fur: rgb(40, 40, 45),
            limb: rgb(34, 34, 38),
            chest: rgb(64, 64, 70),
            face: rgb(84, 80, 88),
            skin: rgb(28, 28, 32),
            torso: [62, 50, 44],
            shoulder: 31,
            arm: 70,
            arm_w: 16,
            leg: 30,
            leg_w: 15,
            head: 28,
            neck: 18,
            muzzle: 56,
            ear: 18,
            brow: 26,
            eye: 18,
            lean: 45,
            upright: false,
        },
        Kind::Chimpanzee => Look {
            size: 100,
            fur: rgb(54, 40, 32),
            limb: rgb(48, 36, 28),
            chest: rgb(70, 54, 44),
            face: rgb(224, 178, 150),
            skin: rgb(206, 158, 128),
            torso: [36, 40, 28],
            shoulder: 18,
            arm: 66,
            arm_w: 9,
            leg: 32,
            leg_w: 10,
            head: 24,
            neck: 32,
            muzzle: 58,
            ear: 46,
            brow: 14,
            eye: 19,
            lean: 40,
            upright: false,
        },
        Kind::Gibbon => Look {
            size: 90,
            fur: rgb(212, 178, 120),
            limb: rgb(200, 164, 108),
            chest: rgb(222, 194, 140),
            face: rgb(34, 30, 30),
            skin: rgb(240, 236, 226),
            torso: [24, 32, 20],
            shoulder: 13,
            arm: 84,
            arm_w: 6,
            leg: 36,
            leg_w: 7,
            head: 20,
            neck: 40,
            muzzle: 36,
            ear: 0,
            brow: 0,
            eye: 21,
            lean: 10,
            upright: true,
        },
        Kind::Proboscis => Look {
            size: 95,
            fur: rgb(196, 110, 56),
            limb: rgb(208, 204, 196),
            chest: rgb(236, 214, 178),
            face: rgb(232, 168, 132),
            skin: rgb(120, 104, 96),
            torso: [34, 40, 30],
            shoulder: 16,
            arm: 48,
            arm_w: 8,
            leg: 44,
            leg_w: 9,
            head: 21,
            neck: 36,
            muzzle: 0,
            ear: 0,
            brow: 0,
            eye: 16,
            lean: 70,
            upright: false,
        },
        Kind::Lemur => Look {
            size: 75,
            fur: rgb(150, 148, 152),
            limb: rgb(170, 168, 172),
            chest: rgb(238, 236, 232),
            face: rgb(244, 244, 242),
            skin: rgb(40, 38, 40),
            torso: [24, 36, 22],
            shoulder: 12,
            arm: 38,
            arm_w: 6,
            leg: 46,
            leg_w: 8,
            head: 18,
            neck: 40,
            muzzle: 0,
            ear: 0,
            brow: 0,
            eye: 26,
            lean: 70,
            upright: false,
        },
    }
}

const BLACK: Rgb = rgb(16, 14, 16);
const EYE_WHITE: Rgb = rgb(250, 250, 246);
const LEMUR_EYE: Rgb = rgb(255, 160, 20);
const STAR: Rgb = rgb(255, 230, 60);
const NOSE: Rgb = rgb(228, 136, 104);

// ---------------------------------------------------------------------------
// Small helpers

/// `v` times `p` percent.
fn pct(v: i32, p: i32) -> i32 {
    v * p / 100
}

/// An angle that turns `per_tick` each tick, wrapped to a turn.
fn spin(tick: u32, per_tick: i32) -> i32 {
    ((tick as i64 * per_tick as i64) & (TURN as i64 - 1)) as i32
}

/// A unit vector along `d`, or `or` when `d` is too short to have one.
fn dir_or(d: V3, or: V3) -> V3 {
    if d.len() < ONE / 1000 {
        or
    } else {
        d.norm()
    }
}

/// A position and three axes: x to the side, y up, z forward. The parts of an
/// ape are placed in a frame's own coordinates.
#[derive(Debug, Clone, Copy)]
struct Frame {
    o: V3,
    x: V3,
    y: V3,
    z: V3,
}

impl Frame {
    /// A frame at `o` whose y is exactly `up` and whose z is as near to `fwd`
    /// as it can be.
    fn upright(o: V3, up: V3, fwd: V3) -> Frame {
        let y = dir_or(up, V3::UP);
        let mut x = y.cross(fwd);
        if x.len() < ONE / 100 {
            x = y.cross(v3(0, 0, ONE));
            if x.len() < ONE / 100 {
                x = y.cross(v3(ONE, 0, 0));
            }
        }
        let x = x.norm();
        let z = x.cross(y).norm();
        Frame { o, x, y, z }
    }

    fn at(&self, x: i32, y: i32, z: i32) -> V3 {
        self.o
            .add(self.x.scale(x))
            .add(self.y.scale(y))
            .add(self.z.scale(z))
    }

    /// A unit shape at (x, y, z) in this frame, `size` across on its axes.
    fn shape(&self, at: V3, size: V3) -> Affine {
        Affine {
            c0: self.x.scale(size.x),
            c1: self.y.scale(size.y),
            c2: self.z.scale(size.z),
            t: self.at(at.x, at.y, at.z),
        }
    }

    /// A direction from parts along x, y and z.
    fn dir(&self, x: i32, y: i32, z: i32) -> V3 {
        dir_or(
            self.x.scale(x).add(self.y.scale(y)).add(self.z.scale(z)),
            self.y,
        )
    }
}

/// Where the shapes go: the model's own metres, scaled about the anchor.
struct Rig<'a> {
    b: &'a mut Batch,
    base: Affine,
}

impl Rig<'_> {
    fn put(&mut self, mesh: Mesh, a: Affine, c: Rgb) {
        self.b.push(mesh, &self.base.then(&a), c, 0);
    }

    fn glow(&mut self, mesh: Mesh, a: Affine, c: Rgb, glow: i32) {
        self.b.push(mesh, &self.base.then(&a), c, glow);
    }

    /// A ball `size` across at `at`, along the world's axes.
    fn ball(&mut self, at: V3, size: i32, c: Rgb) {
        self.put(
            Mesh::Sphere,
            Affine::translate(at).then(&Affine::scale(v3(size, size, size))),
            c,
        );
    }

    /// A rounded rod from `a` to `b`, `w` thick, with a ball at `b`.
    fn bone(&mut self, a: V3, b: V3, w: i32, c: Rgb) {
        if a.dist(b) > ONE / 200 {
            self.put(Mesh::Cyl, rod(a, b, w), c);
        }
        self.ball(b, w, c);
    }
}

/// A two-part limb from `root` toward `target`, `len` long in all. The middle
/// joint bends toward `hint` when the target is nearer than the limb's
/// length. Returns the middle joint and the end.
fn bend(root: V3, target: V3, len: i32, hint: V3) -> (V3, V3) {
    let half = len / 2;
    let d = target.sub(root);
    let dist = d.len();
    if dist >= len - ONE / 200 || dist < ONE / 100 {
        let way = dir_or(d, hint);
        return (root.add(way.scale(half)), root.add(way.scale(len)));
    }
    let way = d.norm();
    let mid = root.add(d.scale(ONE / 2));
    let up = fx::sqrt(mul(half, half) - mul(dist / 2, dist / 2));
    let side = dir_or(hint.reject(way), dir_or(v3(0, ONE, 0).reject(way), hint));
    (mid.add(side.scale(up)), target)
}

// ---------------------------------------------------------------------------
// Posing

/// The ape's joints this frame, in the model's metres about the anchor.
struct Skel {
    /// where the model's origin is in the world (before scaling)
    anchor: V3,
    /// at the hips; y along the spine, z out of the belly
    body: Frame,
    /// at the head's middle
    head: Frame,
    /// the axes the hands and feet are drawn along
    feet_frame: Frame,
    hands: [V3; 2],
    feet: [V3; 2],
    arm_hint: [V3; 2],
    leg_hint: [V3; 2],
    /// the tail leaves along `back`, curling toward `curl` by `bend` a segment
    tail_back: V3,
    tail_curl: V3,
    tail_bend: i32,
}

/// Which side a limb is on: -1 or 1.
const SIDES: [i32; 2] = [-1, 1];

/// Model dimensions in Q16 metres.
struct Dims {
    w: i32,
    t: i32,
    d: i32,
    shoulder: i32,
    arm: i32,
    arm_w: i32,
    leg: i32,
    leg_w: i32,
    head: i32,
}

fn dims(l: &Look) -> Dims {
    Dims {
        w: cm(l.torso[0]),
        t: cm(l.torso[1]),
        d: cm(l.torso[2]),
        shoulder: cm(l.shoulder),
        arm: cm(l.arm),
        arm_w: cm(l.arm_w),
        leg: cm(l.leg),
        leg_w: cm(l.leg_w),
        head: cm(l.head),
    }
}

fn shoulder_of(body: &Frame, m: &Dims, s: i32) -> V3 {
    body.at(s * m.shoulder, pct(m.t, 88), 0)
}

fn hip_of(body: &Frame, m: &Dims, s: i32) -> V3 {
    body.at(s * pct(m.w, 28), pct(m.t, 6), 0)
}

fn pose_skel(l: &Look, pose: &Pose) -> Skel {
    let m = dims(l);
    let face = facing(pose.yaw, 0);
    let tick = pose.tick;
    let mut skel = match pose.rope {
        Some(rope) => swinging(&m, pose, face, dir_or(rope, V3::UP)),
        None if pose.grounded => running(l, &m, pose, face),
        None => flying(&m, pose, face),
    };
    if pose.stunned && pose.rope.is_none() {
        // dizzy: the body circles about its hips, the arms hang out limp
        let w = spin(tick, deg(4));
        let tilt = skel
            .body
            .x
            .scale(mul(fx::cos(w), cm(35)))
            .add(skel.body.z.scale(mul(fx::sin(w), cm(35))));
        skel.body = Frame::upright(skel.body.o, skel.body.y.add(tilt), face);
        for (i, s) in SIDES.into_iter().enumerate() {
            let sh = shoulder_of(&skel.body, &m, s);
            let droop = fx::sin(w + s * QUARTER) * 20 / ONE;
            skel.hands[i] = sh.add(skel.body.dir(s * 70, droop - 50, 25).scale(pct(m.arm, 90)));
            skel.arm_hint[i] = skel.body.dir(s * 50, 0, -50);
        }
    }
    let wobble = if pose.stunned {
        mul(fx::sin(spin(tick, deg(4))), cm(40))
    } else {
        0
    };
    // the head stays nearly upright and faces the way the ape faces
    let hup = dir_or(
        skel.body
            .y
            .add(V3::UP.scale(int(3)))
            .add(skel.body.x.scale(wobble)),
        V3::UP,
    );
    let neck = skel.body.at(0, m.t, 0);
    let hfw = Frame::upright(neck, hup, face);
    let centre = hfw.at(0, pct(m.head, l.neck + 20), pct(m.head, 12));
    skel.head = Frame::upright(centre, hup, face);
    skel
}

fn swinging(m: &Dims, pose: &Pose, face: V3, up: V3) -> Skel {
    let tick = pose.tick;
    let grip = cm(7);
    let reach = pct(m.arm, 92);
    let dx = (m.shoulder - grip).max(0);
    let dv = fx::sqrt(mul(reach, reach) - mul(dx, dx)).max(reach / 2);
    let hips = up.scale(-(dv + pct(m.t, 88)));
    let body = Frame::upright(hips, up, face);
    let hands = SIDES.map(|s| body.x.scale(s * grip));
    // the legs trail behind the swing and kick a little
    let across = pose.vel.reject(up);
    let trail = dir_or(across, V3::ZERO).scale(-(mul(across.len(), cm(3)).min(cm(35))));
    let mut feet = [V3::ZERO; 2];
    for (i, s) in SIDES.into_iter().enumerate() {
        let kick = mul(fx::sin(spin(tick, deg(4)) + s * QUARTER), cm(6));
        feet[i] = hip_of(&body, m, s)
            .add(up.scale(-pct(m.leg, 88)))
            .add(trail)
            .add(body.z.scale(kick));
    }
    Skel {
        anchor: pose.hands,
        body,
        head: body,
        feet_frame: body,
        hands,
        feet,
        arm_hint: SIDES.map(|s| body.dir(s * 60, 0, -40)),
        leg_hint: [body.z; 2],
        tail_back: up.neg(),
        tail_curl: body.z.neg(),
        tail_bend: deg(6),
    }
}

fn flying(m: &Dims, pose: &Pose, face: V3) -> Skel {
    let tick = pose.tick;
    // the body leans into its flight, up to about 50 degrees
    let flat = pose.vel.flat();
    let lean = mul(fx::div(flat.len(), int(12)).min(ONE), cm(120));
    let up = dir_or(V3::UP.add(dir_or(flat, V3::ZERO).scale(lean)), V3::UP);
    let hips = up.scale(-(m.t / 2));
    let body = Frame::upright(hips, up, face);
    let mut hands = [V3::ZERO; 2];
    let mut feet = [V3::ZERO; 2];
    for (i, s) in SIDES.into_iter().enumerate() {
        let flap = mul(fx::sin(spin(tick, deg(6)) + s * deg(40)), cm(8));
        hands[i] = shoulder_of(&body, m, s)
            .add(body.dir(s * 55, 35, 60).scale(pct(m.arm, 93)))
            .add(body.y.scale(flap));
        let kick = mul(fx::sin(spin(tick, deg(8)) + s * HALF), cm(7));
        feet[i] = hip_of(&body, m, s)
            .add(body.dir(s * 12, -80, -45).scale(pct(m.leg, 92)))
            .add(body.z.scale(kick));
    }
    let speed = pose.vel.len();
    let back = if speed > ONE {
        pose.vel.norm().neg()
    } else {
        body.z.neg()
    };
    Skel {
        anchor: pose.hands.sub(v3(0, BODY_DROP / 2, 0)),
        body,
        head: body,
        feet_frame: body,
        hands,
        feet,
        arm_hint: SIDES.map(|s| body.dir(s * 30, -60, -40)),
        leg_hint: [body.z; 2],
        tail_back: back,
        tail_curl: V3::UP,
        tail_bend: deg(5),
    }
}

fn running(l: &Look, m: &Dims, pose: &Pose, face: V3) -> Skel {
    let tick = pose.tick;
    // a fixed cadence, with a stride that grows with speed (stateless, so a
    // change of speed never jumps the cycle)
    let amp = fx::div(pose.vel.flat().len(), int(6)).min(ONE);
    let phase = spin(tick, deg(5));
    let lean = deg(l.lean);
    let up = V3::UP.scale(fx::cos(lean)).add(face.scale(fx::sin(lean)));
    let rise = mul(m.t, fx::cos(lean));
    let hip_h = if l.upright {
        pct(m.leg, 90)
    } else {
        (pct(m.arm, 90) - rise).clamp(pct(m.leg, 50), pct(m.leg, 85))
    };
    let bob = mul(fx::sin(phase * 2).abs(), mul(cm(4), amp))
        + mul(fx::sin(spin(tick, deg(2))), cm(1) / 2);
    let hips = V3::UP
        .scale(hip_h + bob)
        .sub(face.scale(mul(m.t, fx::sin(lean)) / 2));
    let body = Frame::upright(hips, up, face);
    let flat = Frame::upright(V3::ZERO, V3::UP, face);
    let ground = |p: V3| v3(p.x, 0, p.z);
    let stride = mul(pct(m.leg, 40), amp);
    let lift = mul(cm(10), amp);
    let mut hands = [V3::ZERO; 2];
    let mut feet = [V3::ZERO; 2];
    for (i, s) in SIDES.into_iter().enumerate() {
        let lp = phase + if s > 0 { HALF } else { 0 };
        feet[i] = ground(hip_of(&body, m, s))
            .add(face.scale(mul(fx::sin(lp), stride)))
            .add(V3::UP.scale(m.leg_w / 2 + mul(fx::cos(lp).max(0), lift)));
        let sh = shoulder_of(&body, m, s);
        hands[i] = if l.upright {
            // a gibbon runs on its legs with its arms held high
            let sway = mul(fx::sin(lp), mul(cm(10), amp));
            sh.add(flat.dir(s * 55, 80, 10).scale(pct(m.arm, 90)))
                .add(face.scale(sway))
        } else {
            let ap = phase + if s > 0 { 0 } else { HALF };
            ground(sh)
                .add(flat.x.scale(s * cm(4)))
                .add(face.scale(cm(8) + mul(fx::sin(ap), stride)))
                .add(V3::UP.scale(pct(m.arm_w, 70) + mul(fx::cos(ap).max(0), lift)))
        };
    }
    Skel {
        anchor: pose.hands.sub(v3(0, BODY_DROP, 0)),
        body,
        head: body,
        feet_frame: flat,
        hands,
        feet,
        arm_hint: SIDES.map(|s| flat.dir(s * 40, 0, -60)),
        leg_hint: [face; 2],
        tail_back: body.y.neg(),
        tail_curl: V3::UP,
        tail_bend: deg(11),
    }
}

// ---------------------------------------------------------------------------
// Drawing

/// Draws one ape.
pub fn draw(b: &mut Batch, kind: Kind, pose: &Pose) {
    let l = look(kind);
    let m = dims(&l);
    let skel = pose_skel(&l, pose);
    let k = (ONE as i64 * l.size as i64 * pose.scale.max(1) as i64 / 10_000) as i32;
    let mut rig = Rig {
        b,
        base: Affine::translate(skel.anchor).then(&Affine::scale(v3(k, k, k))),
    };
    torso(&mut rig, kind, &l, &m, &skel);
    limbs(&mut rig, kind, &l, &m, &skel);
    head(&mut rig, kind, &l, &skel.head, pose);
    tail(&mut rig, kind, &m, &skel, pose.tick);
    if pose.stunned {
        stars(&mut rig, &skel.head, m.head, pose.tick);
    }
}

fn torso(rig: &mut Rig, kind: Kind, l: &Look, m: &Dims, s: &Skel) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    rig.put(
        Mesh::Sphere,
        body.shape(v3(0, t / 2, 0), v3(w, pct(t, 118), d)),
        l.fur,
    );
    rig.put(
        Mesh::Sphere,
        body.shape(
            v3(0, pct(t, 52), pct(d, 22)),
            v3(pct(w, 70), pct(t, 76), pct(d, 66)),
        ),
        l.chest,
    );
    match kind {
        Kind::Gorilla => {
            // the huge shoulders and chest
            rig.put(
                Mesh::Sphere,
                body.shape(
                    v3(0, pct(t, 80), pct(d, 4)),
                    v3(pct(w, 120), pct(t, 62), pct(d, 104)),
                ),
                l.fur,
            );
            for side in SIDES {
                rig.put(
                    Mesh::Sphere,
                    body.shape(
                        v3(side * pct(w, 20), pct(t, 72), pct(d, 34)),
                        v3(pct(w, 44), pct(t, 40), pct(d, 40)),
                    ),
                    l.chest,
                );
            }
            // the silver saddle across the back
            rig.put(
                Mesh::Sphere,
                body.shape(
                    v3(0, pct(t, 48), -pct(d, 26)),
                    v3(pct(w, 88), pct(t, 84), pct(d, 58)),
                ),
                rgb(178, 178, 184),
            );
        }
        Kind::Orangutan => {
            // shaggy tufts over the shoulders and back
            for side in SIDES {
                for (y, z) in [(78, -10), (50, -30), (20, -20)] {
                    rig.put(
                        Mesh::Blob,
                        body.shape(
                            v3(side * pct(w, 34), pct(t, y), pct(d, z)),
                            v3(pct(w, 40), pct(t, 44), pct(d, 50)),
                        ),
                        shade(l.fur, 88),
                    );
                }
            }
        }
        Kind::Proboscis => {
            // the pot belly
            rig.put(
                Mesh::Sphere,
                body.shape(
                    v3(0, pct(t, 34), pct(d, 30)),
                    v3(pct(w, 92), pct(t, 72), pct(d, 86)),
                ),
                l.chest,
            );
        }
        _ => {}
    }
}

fn limbs(rig: &mut Rig, kind: Kind, l: &Look, m: &Dims, s: &Skel) {
    let ff = &s.feet_frame;
    for (i, side) in SIDES.into_iter().enumerate() {
        // arm
        let sh = shoulder_of(&s.body, m, side);
        let (elbow, hand) = bend(sh, s.hands[i], m.arm, s.arm_hint[i]);
        rig.ball(sh, pct(m.arm_w, 150), l.limb);
        rig.bone(sh, elbow, m.arm_w, l.limb);
        rig.bone(elbow, hand, pct(m.arm_w, 88), l.limb);
        rig.put(
            Mesh::Sphere,
            ff.shape(
                V3::ZERO,
                v3(pct(m.arm_w, 150), pct(m.arm_w, 120), pct(m.arm_w, 160)),
            )
            .at(hand),
            l.skin,
        );
        if kind == Kind::Orangutan {
            // long hair hanging from the arms
            for (a, b) in [(sh, elbow), (elbow, hand)] {
                let mid = a.lerp(b, ONE / 2).sub(v3(0, m.arm_w / 2, 0));
                rig.put(
                    Mesh::Blob,
                    rod(mid, mid.add(b.sub(a).scale(ONE / 3)), pct(m.arm_w, 200)),
                    shade(l.fur, 92),
                );
            }
        }
        // leg
        let hip = hip_of(&s.body, m, side);
        let (knee, ankle) = bend(hip, s.feet[i], m.leg, s.leg_hint[i]);
        rig.ball(hip, pct(m.leg_w, 140), l.limb);
        rig.bone(hip, knee, m.leg_w, l.limb);
        rig.bone(knee, ankle, pct(m.leg_w, 85), l.limb);
        let foot = v3(0, 0, m.leg_w / 2);
        rig.put(
            Mesh::Sphere,
            Frame { o: ankle, ..*ff }.shape(
                foot,
                v3(pct(m.leg_w, 140), pct(m.leg_w, 80), pct(m.leg_w, 230)),
            ),
            l.skin,
        );
    }
}

/// The proboscis monkey's nose: four points from its root on the face to its
/// tip. It droops over the mouth; falling blows it up, running forward
/// presses it back, sideways speed swings it the other way, and it wobbles
/// more the faster the monkey goes.
fn nose_points(h: &Frame, d: i32, vel: V3, tick: u32) -> [V3; 4] {
    // one degree (182 angle units) for each metre per second
    let per = |v: i32| v / 360;
    let up_speed = vel.dot(h.y);
    let fwd_speed = vel.dot(h.z);
    let side_speed = vel.dot(h.x);
    let speed = vel.len();
    let shake = deg(6) + per(speed).min(deg(16));
    let flop = (-per(up_speed) * 3).clamp(-deg(20), deg(70))
        - (per(fwd_speed) * 3).clamp(0, deg(45))
        + mul(fx::sin(spin(tick, deg(9))), shake);
    let sway = (-per(side_speed) * 3).clamp(-deg(35), deg(35))
        + mul(fx::cos(spin(tick, deg(7))), shake / 2);
    // each segment's angle from straight down toward forward
    let base = [deg(75), deg(40), deg(12)];
    let len = pct(d, 27);
    let mut p = [V3::ZERO; 4];
    p[0] = h.at(0, -pct(d, 6), pct(d, 44));
    for i in 0..3 {
        let share = (i as i32 + 1) * ONE / 3;
        let a = (base[i] + mul(flop, share)).clamp(deg(5), deg(160));
        let sw = mul(sway, share);
        let way =
            h.y.scale(-fx::cos(a))
                .add(h.z.scale(fx::sin(a)))
                .add(h.x.scale(fx::sin(sw)));
        p[i + 1] = p[i].add(dir_or(way, h.y.neg()).scale(len));
    }
    p
}

fn eyes(rig: &mut Rig, h: &Frame, d: i32, size: i32, colour: Rgb, y: i32, x: i32) {
    for side in SIDES {
        let e = h.at(side * pct(d, x), pct(d, y), pct(d, 42));
        let es = pct(d, size);
        rig.glow(
            Mesh::Sphere,
            h.shape(V3::ZERO, v3(es, es, es)).at(e),
            colour,
            ONE / 3,
        );
        let ps = pct(es, 55);
        rig.put(
            Mesh::Sphere,
            h.shape(V3::ZERO, v3(ps, ps, ps))
                .at(e.add(h.z.scale(pct(es, 34)))),
            BLACK,
        );
    }
}

fn head(rig: &mut Rig, kind: Kind, l: &Look, h: &Frame, pose: &Pose) {
    let d = cm(l.head);
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    // skull
    rig.put(Mesh::Sphere, h.shape(V3::ZERO, p(100, 96, 96)), l.fur);
    // a white ring or flanges behind the face
    match kind {
        Kind::Orangutan => {
            let c = h.at(0, -pct(d, 4), pct(d, 22));
            rig.put(
                Mesh::Cyl,
                rod(c, c.add(h.z.scale(pct(d, 10))), pct(d, 160)),
                rgb(104, 84, 74),
            );
            // the throat sac and a tuft on top
            rig.put(Mesh::Sphere, h.shape(p(0, -55, 20), p(70, 50, 60)), l.skin);
            rig.put(Mesh::Blob, h.shape(p(0, 46, -6), p(60, 34, 70)), l.fur);
        }
        Kind::Gibbon => {
            let c = h.at(0, -pct(d, 2), pct(d, 26));
            rig.put(
                Mesh::Cyl,
                rod(c, c.add(h.z.scale(pct(d, 10))), pct(d, 94)),
                EYE_WHITE,
            );
        }
        Kind::Gorilla => {
            // the crest on top
            rig.put(Mesh::Sphere, h.shape(p(0, 40, -8), p(36, 46, 76)), l.fur);
        }
        _ => {}
    }
    // the bare face
    let face_size = match kind {
        Kind::Lemur => p(84, 80, 50),
        Kind::Proboscis => p(66, 66, 40),
        _ => p(76, 72, 42),
    };
    rig.put(Mesh::Sphere, h.shape(p(0, -3, 32), face_size), l.face);
    if l.brow > 0 {
        let c = if kind == Kind::Gorilla { l.fur } else { l.face };
        rig.put(
            Mesh::Sphere,
            h.shape(p(0, 22, 40), p(84, 10 + l.brow / 2, l.brow + 14)),
            shade(c, 90),
        );
    }
    if l.ear > 0 {
        for side in SIDES {
            let e = l.ear;
            rig.put(
                Mesh::Sphere,
                h.shape(p(side * 50, 4, 0), p(e / 3, e, pct(e, 80))),
                l.skin,
            );
            rig.put(
                Mesh::Sphere,
                h.shape(p(side * 52, 4, 2), p(e / 4, pct(e, 66), pct(e, 50))),
                shade(l.skin, 70),
            );
        }
    }
    if l.muzzle > 0 {
        let mz = l.muzzle;
        let muzzle = if kind == Kind::Orangutan || kind == Kind::Chimpanzee {
            shade(l.face, 108)
        } else {
            l.face
        };
        rig.put(
            Mesh::Sphere,
            h.shape(p(0, -24, 44), p(mz * 5 / 4, pct(mz, 85), mz)),
            muzzle,
        );
        // nostrils and mouth
        for side in SIDES {
            rig.put(
                Mesh::Sphere,
                h.shape(p(side * 6, -16, 44 + mz / 2), p(8, 6, 6)),
                BLACK,
            );
        }
        rig.put(
            Mesh::Cube,
            h.shape(p(0, -34, 42 + pct(mz, 45)), p(pct(mz, 70), 3, 4)),
            BLACK,
        );
    }
    match kind {
        Kind::Lemur => {
            // the black mask, the pointed snout and the pricked ears
            for side in SIDES {
                rig.put(
                    Mesh::Sphere,
                    h.shape(p(side * 20, 8, 42), p(38, 32, 22)),
                    BLACK,
                );
                let base = h.at(side * pct(d, 30), pct(d, 28), -pct(d, 4));
                let tip = h.at(side * pct(d, 50), pct(d, 66), -pct(d, 8));
                rig.put(Mesh::Cone, rod(base, tip, pct(d, 30)), l.fur);
            }
            let a = h.at(0, -pct(d, 12), pct(d, 34));
            let b = h.at(0, -pct(d, 20), pct(d, 86));
            rig.put(Mesh::Cone, rod(a, b, pct(d, 40)), l.face);
            rig.ball(b.sub(h.z.scale(pct(d, 6))), pct(d, 14), BLACK);
            eyes(rig, h, d, l.eye, LEMUR_EYE, 10, 20);
        }
        Kind::Proboscis => {
            eyes(rig, h, d, l.eye, EYE_WHITE, 14, 15);
            // a small mouth, under the nose
            rig.put(Mesh::Cube, h.shape(p(0, -30, 44), p(26, 4, 6)), BLACK);
            let n = nose_points(h, d, pose.vel, pose.tick);
            let widths = [pct(d, 24), pct(d, 30), pct(d, 38)];
            rig.ball(n[0], pct(d, 22), NOSE);
            for i in 0..3 {
                rig.bone(n[i], n[i + 1], widths[i], NOSE);
            }
            rig.ball(n[3], pct(d, 48), NOSE);
        }
        _ => eyes(rig, h, d, l.eye, EYE_WHITE, 8, 17),
    }
}

fn tail(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, tick: u32) {
    let (count, seg, w0, w1, bend_by) = match kind {
        Kind::Lemur => (12, cm(8), cm(8), cm(6), s.tail_bend),
        Kind::Proboscis => (8, cm(9), cm(5), cm(4), s.tail_bend / 3),
        _ => return,
    };
    // the lemur carries its tail up in a hook; the proboscis monkey's droops
    let (lift, curl) = match kind {
        Kind::Lemur => (cm(40), s.tail_curl),
        _ => (cm(30), V3::UP.neg()),
    };
    let back = dir_or(s.tail_back.add(V3::UP.scale(lift)), s.body.z.neg());
    let curl = dir_or(curl.reject(back), s.body.z.neg());
    let side = back.cross(curl).norm();
    let mut at = s.body.at(0, pct(m.t, 10), -pct(m.d, 42));
    for i in 0..count {
        let a = bend_by * (i + 1);
        let sway = mul(fx::sin(spin(tick, deg(3)) - i * deg(20)), cm(25));
        let way = dir_or(
            back.scale(fx::cos(a))
                .add(curl.scale(fx::sin(a)))
                .add(side.scale(sway)),
            back,
        );
        let next = at.add(way.scale(seg));
        let w = w0 + (w1 - w0) * i / count;
        let c = match kind {
            Kind::Lemur if i % 2 == 0 => rgb(242, 242, 240),
            Kind::Lemur => BLACK,
            _ => rgb(208, 204, 196),
        };
        rig.bone(at, next, w, c);
        at = next;
    }
}

fn stars(rig: &mut Rig, h: &Frame, d: i32, tick: u32) {
    for i in 0..4 {
        let a = spin(tick, deg(5)) + i * QUARTER;
        let r = pct(d, 75);
        let p =
            h.o.add(V3::UP.scale(pct(d, 80)))
                .add(v3(mul(fx::cos(a), r), 0, mul(fx::sin(a), r)));
        let size = pct(d, 22);
        rig.glow(
            Mesh::Sphere,
            Affine::translate(p).then(&Affine::scale(v3(size, size, size))),
            STAR,
            ONE,
        );
    }
}

// ---------------------------------------------------------------------------
// The lineup (the thumbnail) and the selection screen

/// Gap between apes in the lineup.
const LINEUP_GAP: i32 = cm(150);

/// Where each ape stands in the lineup: in a row along x, feet on y = 0.
fn lineup_spot(i: usize) -> V3 {
    v3((2 * i as i32 - 5) * LINEUP_GAP / 2, 0, 0)
}

/// All six apes standing side by side, turned a little toward the camera's
/// right so their faces, noses and tails all show.
pub fn lineup(b: &mut Batch, tick: u32) {
    for (i, kind) in Kind::ALL.into_iter().enumerate() {
        let pose = Pose::standing(lineup_spot(i), deg(28), tick);
        draw(b, kind, &pose);
    }
}

/// A camera that frames the whole lineup at `aspect` (width / height, Q16).
pub fn lineup_camera(aspect: i32) -> Camera {
    let fov = deg(34);
    let half = fov / 2;
    let tan = fx::div(fx::sin(half), fx::cos(half));
    // the row is about 10 m wide and 2 m tall, plus a margin
    let need_w = fx::div(cm(470), mul(tan, aspect.max(ONE / 4)));
    let need_h = fx::div(cm(110), tan);
    let back = need_w.max(need_h) + cm(60);
    let target = v3(0, cm(70), 0);
    let eye = v3(0, cm(170), back);
    Camera {
        eye,
        fwd: target.sub(eye).norm(),
        fov,
    }
}

/// One ape on the selection screen, turning a full circle every 6 s.
pub fn showcase(b: &mut Batch, kind: Kind, tick: u32) {
    let yaw = ((tick as i64 % 720) * TURN as i64 / 720) as i32;
    let pose = Pose::standing(V3::ZERO, yaw, tick);
    draw(b, kind, &pose);
}

/// The selection screen's camera: the ape at the middle, a little from above.
pub fn showcase_camera() -> Camera {
    let target = v3(0, cm(62), 0);
    let eye = v3(0, cm(125), cm(290));
    Camera {
        eye,
        fwd: target.sub(eye).norm(),
        fov: deg(34),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{DYNAMIC_BUFFER, INSTANCE_VALUES};

    /// One drawn shape: its mesh, three axes, translation and colour.
    struct Shape {
        mesh: u32,
        axes: [V3; 3],
        at: V3,
        colour: [i32; 3],
    }

    fn shapes(b: Batch) -> Vec<Shape> {
        let (values, draws) = b.finish(DYNAMIC_BUFFER);
        let mut out = Vec::new();
        for d in draws.chunks(4) {
            let first = d[2] as usize / 4 / INSTANCE_VALUES;
            for i in first..first + d[3] as usize {
                let v = &values[i * INSTANCE_VALUES..(i + 1) * INSTANCE_VALUES];
                out.push(Shape {
                    mesh: d[1],
                    axes: [
                        v3(v[0], v[4], v[8]),
                        v3(v[1], v[5], v[9]),
                        v3(v[2], v[6], v[10]),
                    ],
                    at: v3(v[3], v[7], v[11]),
                    colour: [v[12], v[13], v[14]],
                });
            }
        }
        out
    }

    fn drawn(kind: Kind, pose: &Pose) -> Vec<Shape> {
        let mut b = Batch::new();
        draw(&mut b, kind, pose);
        shapes(b)
    }

    fn at_rest() -> Pose {
        Pose::standing(v3(int(10), int(2), int(-30)), deg(40), 75)
    }

    /// A spread of poses: standing, running, swinging, flying, falling,
    /// stunned on the ground and stunned in the air.
    fn poses() -> Vec<Pose> {
        let rest = at_rest();
        let run = Pose {
            vel: v3(int(5), 0, int(3)),
            tick: 1234,
            ..rest
        };
        let swing = Pose {
            hands: v3(int(10), int(9), int(-30)),
            rope: Some(v3(ONE / 2, ONE, -ONE / 3).norm()),
            vel: v3(int(14), int(-3), int(2)),
            grounded: false,
            tick: 77,
            ..rest
        };
        let fly = Pose {
            rope: None,
            vel: v3(int(-20), int(6), int(25)),
            tick: 4_000_000_000,
            ..swing
        };
        let fall = Pose {
            vel: v3(0, int(-40), 0),
            ..fly
        };
        let dizzy = Pose {
            stunned: true,
            ..rest
        };
        let dizzy_air = Pose {
            stunned: true,
            ..fly
        };
        vec![rest, run, swing, fly, fall, dizzy, dizzy_air]
    }

    #[test]
    fn every_kind_draws_a_reasonable_number_of_shapes() {
        for kind in Kind::ALL {
            for pose in poses() {
                let n = drawn(kind, &pose).len();
                assert!((30..=120).contains(&n), "{kind:?} {n} shapes");
            }
        }
    }

    #[test]
    fn every_kind_looks_different() {
        let sig = |kind: Kind| {
            let s = drawn(kind, &at_rest());
            let mut per_mesh = [0usize; 7];
            let mut colours: Vec<[i32; 3]> = Vec::new();
            for x in &s {
                per_mesh[x.mesh as usize] += 1;
                colours.push(x.colour);
            }
            colours.sort();
            colours.dedup();
            (per_mesh, colours)
        };
        let all: Vec<_> = Kind::ALL.iter().map(|k| sig(*k)).collect();
        for i in 0..all.len() {
            for j in i + 1..all.len() {
                let (a, b) = (Kind::ALL[i], Kind::ALL[j]);
                assert_ne!(all[i].0, all[j].0, "{a:?} and {b:?} use the same shapes");
                assert_ne!(all[i].1, all[j].1, "{a:?} and {b:?} use the same colours");
            }
        }
    }

    #[test]
    fn every_shape_is_a_sensible_size_and_near_the_ape() {
        for kind in Kind::ALL {
            for scale in [100, 350] {
                for pose in poses() {
                    let pose = Pose { scale, ..pose };
                    for s in drawn(kind, &pose) {
                        for a in s.axes {
                            let l = a.len();
                            assert!(l >= ONE / 500, "{kind:?} {pose:?} a flat shape: {l}");
                            assert!(l <= pct(int(2), scale), "{kind:?} a huge shape: {l}");
                        }
                        let off = s.at.dist(pose.hands);
                        assert!(off <= pct(int(3), scale), "{kind:?} {pose:?} {off} away");
                        for c in s.colour {
                            assert!((0..=ONE).contains(&c));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn a_swinging_ape_holds_the_vine_and_hangs_below_it() {
        for kind in Kind::ALL {
            let pose = poses()[2];
            let s = drawn(kind, &pose);
            let rope = pose.rope.unwrap();
            // a hand is at the grip
            let near = s.iter().map(|x| x.at.dist(pose.hands)).min().unwrap();
            assert!(near < cm(12), "{kind:?}: nearest shape {near}");
            // and the body hangs below it, along the rope
            for x in &s {
                let above = x.at.sub(pose.hands).dot(rope);
                assert!(above < cm(12), "{kind:?}: a shape {above} above the hands");
            }
            let middle = s.iter().fold(V3::ZERO, |a, x| a.add(x.at.sub(pose.hands)));
            assert!(middle.dot(rope) < 0, "{kind:?}");
        }
    }

    #[test]
    fn a_running_ape_stands_on_the_ground() {
        for kind in Kind::ALL {
            for pose in [poses()[0], poses()[1]] {
                let ground = pose.hands.y - BODY_DROP;
                let s = drawn(kind, &pose);
                let lowest = s.iter().map(|x| x.at.y).min().unwrap();
                assert!(lowest >= ground - cm(1), "{kind:?}: below the ground");
                assert!(lowest <= ground + cm(10), "{kind:?}: floating at {lowest}");
            }
        }
    }

    #[test]
    fn running_moves_the_feet_and_standing_still_does_not() {
        for kind in Kind::ALL {
            let l = look(kind);
            let feet = |t: u32, vel: V3| {
                let pose = Pose {
                    tick: t,
                    vel,
                    ..at_rest()
                };
                pose_skel(&l, &pose).feet
            };
            let fast = v3(0, 0, int(6));
            // half a stride apart, each foot swings from front to back
            let (a, b) = (feet(18, fast), feet(54, fast));
            for i in 0..2 {
                let moved = a[i].dist(b[i]);
                assert!(moved > cm(15), "{kind:?} foot {i} moved {moved}");
            }
            let (a, b) = (feet(18, V3::ZERO), feet(54, V3::ZERO));
            for i in 0..2 {
                let moved = a[i].dist(b[i]);
                assert!(moved < cm(2), "{kind:?} foot {i} moved {moved} standing");
            }
        }
    }

    #[test]
    fn the_proboscis_nose_flops_with_velocity() {
        let h = Frame::upright(V3::ZERO, V3::UP, v3(0, 0, ONE));
        let d = cm(21);
        let tip = |vel: V3| nose_points(&h, d, vel, 50)[3];
        let still = tip(V3::ZERO);
        // falling blows it up; running forward presses it back; sideways swings it
        let falling = tip(v3(0, int(-15), 0));
        let forward = tip(v3(0, 0, int(15)));
        let sideways = tip(v3(int(15), 0, 0));
        assert!(falling.y > still.y + cm(3), "{still:?} {falling:?}");
        assert!(forward.z < still.z - cm(2), "{still:?} {forward:?}");
        assert!(sideways.x < still.x - cm(3), "{still:?} {sideways:?}");
        // and it wobbles as the ticks pass, more when fast
        let wobble = |vel: V3| nose_points(&h, d, vel, 0)[3].dist(nose_points(&h, d, vel, 10)[3]);
        assert!(wobble(v3(0, 0, int(20))) > wobble(V3::ZERO));
        // the drawn proboscis changes with velocity, all else equal
        let pose = poses()[2];
        let moved = Pose {
            vel: pose.vel.add(v3(0, int(-10), 0)),
            ..pose
        };
        let nose = |s: Vec<Shape>| {
            s.iter()
                .filter(|x| x.colour == NOSE)
                .map(|x| x.at)
                .collect::<Vec<_>>()
        };
        let a = nose(drawn(Kind::Proboscis, &pose));
        let b = nose(drawn(Kind::Proboscis, &moved));
        assert!(!a.is_empty());
        assert_ne!(a, b);
    }

    #[test]
    fn the_lineup_shows_all_six_apes_in_the_camera() {
        let mut b = Batch::new();
        lineup(&mut b, 30);
        let all = shapes(b);
        let one_each: usize = Kind::ALL
            .iter()
            .enumerate()
            .map(|(i, k)| drawn(*k, &Pose::standing(lineup_spot(i), deg(28), 30)).len())
            .sum();
        assert_eq!(all.len(), one_each);
        assert!(all.len() >= 6 * 30);
        for aspect in [ONE * 16 / 9, ONE * 4 / 3, ONE, ONE * 21 / 9] {
            let cam = lineup_camera(aspect);
            for i in 0..6 {
                let middle = lineup_spot(i).add(v3(0, cm(60), 0));
                let (x, y) = cam.screen(aspect, middle).expect("in front");
                assert!((5 * ONE..=95 * ONE).contains(&x), "ape {i} at x {x}");
                assert!((5 * ONE..=95 * ONE).contains(&y), "ape {i} at y {y}");
            }
            for s in &all {
                let (x, y) = cam.screen(aspect, s.at).expect("in front");
                assert!((0..=100 * ONE).contains(&x) && (0..=100 * ONE).contains(&y));
            }
            // the row fills the frame rather than sitting small in the middle
            let (left, _) = cam.screen(aspect, lineup_spot(0)).unwrap();
            let (right, _) = cam.screen(aspect, lineup_spot(5)).unwrap();
            assert!(right - left > 55 * ONE, "{aspect}: {left} to {right}");
        }
    }

    #[test]
    fn the_showcase_turns_once_in_six_seconds_inside_its_camera() {
        let at = |kind, t| {
            let mut b = Batch::new();
            showcase(&mut b, kind, t);
            shapes(b).iter().map(|s| s.at).collect::<Vec<_>>()
        };
        let cam = showcase_camera();
        for kind in Kind::ALL {
            // 720 ticks is 6 s at 120 ticks a second
            let worst = at(kind, 0)
                .iter()
                .zip(&at(kind, 720))
                .map(|(a, b)| a.dist(*b))
                .max()
                .unwrap();
            assert!(worst < cm(2), "{kind:?}");
            // the eyes face the camera (+z) first, +x a quarter of the way round,
            // and away from it half way
            let eyes = |t: u32| {
                let mut b = Batch::new();
                showcase(&mut b, kind, t);
                let e: Vec<V3> = shapes(b)
                    .iter()
                    .filter(|s| s.mesh == Mesh::Sphere as u32)
                    .filter(|s| s.colour == EYE_WHITE || s.colour == LEMUR_EYE)
                    .map(|s| s.at)
                    .collect();
                assert_eq!(e.len(), 2, "{kind:?}");
                e[0].add(e[1]).scale(ONE / 2)
            };
            assert!(eyes(0).z > cm(10), "{kind:?}");
            assert!(eyes(180).x > cm(10), "{kind:?}");
            assert!(eyes(360).z < -cm(10), "{kind:?}");
            assert!(eyes(540).x < -cm(10), "{kind:?}");
            for t in [0, 100, 250, 400, 600] {
                for p in at(kind, t) {
                    for aspect in [ONE * 16 / 9, ONE] {
                        let (x, y) = cam.screen(aspect, p).expect("in front");
                        assert!((5 * ONE..=95 * ONE).contains(&x), "{kind:?} x {x}");
                        assert!((5 * ONE..=95 * ONE).contains(&y), "{kind:?} y {y}");
                    }
                }
            }
        }
    }

    #[test]
    fn a_limb_bends_toward_its_hint_and_keeps_its_length() {
        let root = v3(0, int(1), 0);
        let hint = v3(0, 0, -ONE);
        // within reach: the end is on the target and the joint bends to the hint
        for target in [v3(cm(30), cm(80), 0), v3(0, cm(50), cm(20))] {
            let (mid, end) = bend(root, target, cm(80), hint);
            assert!(end.dist(target) < cm(1), "{target:?}");
            assert!(
                (mid.dist(root) - cm(40)).abs() < cm(1),
                "{target:?} {mid:?}"
            );
            assert!((end.dist(mid) - cm(40)).abs() < cm(1), "{target:?}");
            let straight = root.lerp(target, ONE / 2);
            assert!(mid.sub(straight).dot(hint) > cm(5), "{target:?} {mid:?}");
        }
        // out of reach (or on the root itself): straight, at full length
        for target in [v3(0, int(3), 0), root] {
            let (mid, end) = bend(root, target, cm(80), hint);
            assert!((mid.dist(root) - cm(40)).abs() < cm(1), "{target:?}");
            assert!((end.dist(root) - cm(80)).abs() < cm(1), "{target:?}");
        }
    }
}
