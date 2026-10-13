//! The six apes' models (PLAN P8), built from the faceted unit meshes in a
//! low-poly, flat-shaded style (TRIAGE T3): barrel torsos, geodesic heads,
//! six-sided tapering limbs, bevelled hands and feet with fingers and toes,
//! hexagonal discs for ears and face rings, and kite-shaped tufts of hair.
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
            fur: rgb(50, 50, 58),
            limb: rgb(44, 44, 52),
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
const NOSE: Rgb = rgb(238, 122, 98);
/// the gibbon's face ring (not quite the eyes' white)
const FACE_RING: Rgb = rgb(246, 244, 236);
const TAIL_GREY: Rgb = rgb(208, 204, 196);

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

    /// A faceted ball `size` across at `at`, along the world's axes.
    fn ball(&mut self, at: V3, size: i32, c: Rgb) {
        self.put(
            Mesh::Ico,
            Affine::translate(at).then(&Affine::scale(v3(size, size, size))),
            c,
        );
    }

    /// A six-sided segment from `a` (`w` across) tapering toward `b`, with a
    /// faceted knuckle at `b`.
    fn bone(&mut self, a: V3, b: V3, w: i32, c: Rgb) {
        if a.dist(b) > ONE / 200 {
            self.put(Mesh::Taper, rod(a, b, w), c);
        }
        self.ball(b, pct(w, 66), c);
    }

    /// A flat kite from `a` to `b`, `w` wide along `side` and `thick` through
    /// it: a tuft of hair, an ear, a patch of the lemur's mask.
    fn blade(&mut self, a: V3, b: V3, side: V3, w: i32, thick: i32, c: Rgb) {
        let d = b.sub(a);
        let along = dir_or(d, V3::UP);
        let x = dir_or(
            side.reject(along),
            dir_or(V3::UP.cross(along), v3(ONE, 0, 0)),
        );
        let z = x.cross(along).norm();
        self.put(
            Mesh::Shard,
            Affine {
                c0: x.scale(w),
                c1: d,
                c2: z.scale(thick),
                t: a.lerp(b, ONE / 2),
            },
            c,
        );
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
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    // the hips, the ribcage and a yoke across the shoulders
    rig.put(
        Mesh::Barrel,
        body.shape(at(0, 18, -4), at(86, 52, 90)),
        l.fur,
    );
    rig.put(
        Mesh::Barrel,
        body.shape(at(0, 58, 0), at(100, 84, 100)),
        l.fur,
    );
    rig.put(
        Mesh::Bevel,
        body.shape(
            at(0, 86, -4),
            v3(2 * m.shoulder + pct(m.arm_w, 60), pct(t, 26), pct(d, 76)),
        ),
        shade(l.fur, 94),
    );
    // the chest and belly patch
    rig.put(
        Mesh::Geo,
        body.shape(at(0, 50, 30), at(62, 72, 50)),
        l.chest,
    );
    match kind {
        Kind::Gorilla => {
            // the slabs of the chest, and the silver saddle across the back
            for side in SIDES {
                rig.put(
                    Mesh::Bevel,
                    body.shape(at(side * 22, 70, 38), at(44, 30, 34)),
                    l.chest,
                );
            }
            rig.put(
                Mesh::Geo,
                body.shape(at(0, 46, -26), at(84, 72, 54)),
                rgb(158, 158, 166),
            );
        }
        Kind::Orangutan => {
            // long shaggy hair hanging off the shoulders and back
            for side in SIDES {
                for (x, y, z, len) in [(40, 90, -10, 60), (30, 70, -40, 70), (14, 46, -46, 66)] {
                    let top = body.at(side * pct(w, x), pct(t, y), pct(d, z));
                    let bottom = top
                        .add(V3::UP.scale(-pct(t, len)))
                        .add(body.x.scale(side * pct(w, 12)))
                        .add(body.z.scale(-pct(d, 14)));
                    rig.blade(
                        top,
                        bottom,
                        body.x,
                        pct(w, 30),
                        pct(d, 12),
                        shade(l.fur, 86),
                    );
                }
            }
            rig.blade(
                body.at(0, pct(t, 70), -pct(d, 48)),
                body.at(0, pct(t, 10), -pct(d, 60)),
                body.x,
                pct(w, 40),
                pct(d, 12),
                shade(l.fur, 80),
            );
        }
        Kind::Proboscis => {
            // the pot belly, and the pale cape round the shoulders
            rig.put(
                Mesh::Geo,
                body.shape(at(0, 34, 30), at(94, 74, 88)),
                l.chest,
            );
            rig.put(
                Mesh::Barrel,
                body.shape(at(0, 90, 0), at(118, 30, 110)),
                rgb(236, 220, 190),
            );
        }
        Kind::Chimpanzee => {
            // a ruff of dark hair on the shoulders
            for side in SIDES {
                let top = body.at(side * pct(w, 34), pct(t, 96), -pct(d, 24));
                let bottom = body.at(side * pct(w, 40), pct(t, 66), -pct(d, 40));
                rig.blade(
                    top,
                    bottom,
                    body.x,
                    pct(w, 30),
                    pct(d, 12),
                    shade(l.fur, 84),
                );
            }
        }
        Kind::Lemur => {
            // a darker saddle down the back
            rig.put(
                Mesh::Bevel,
                body.shape(at(0, 56, -30), at(76, 70, 40)),
                shade(l.fur, 82),
            );
        }
        Kind::Gibbon => {}
    }
}

/// A hand: a palm, four fingers ahead along the frame's z and a thumb on the
/// inside (`side` is the hand's side, -1 or 1).
fn hand(rig: &mut Rig, f: &Frame, at: V3, w: i32, side: i32, c: Rgb) {
    let f = Frame { o: at, ..*f };
    rig.put(
        Mesh::Bevel,
        f.shape(V3::ZERO, v3(pct(w, 140), pct(w, 84), pct(w, 120))),
        c,
    );
    for i in 0..4 {
        let x = pct(w, (2 * i - 3) * 17);
        rig.put(
            Mesh::Bevel,
            f.shape(
                v3(x, -pct(w, 12), pct(w, 70)),
                v3(pct(w, 30), pct(w, 36), pct(w, 56)),
            ),
            shade(c, 92),
        );
    }
    // the thumb, to the inside
    rig.put(
        Mesh::Pyramid,
        f.shape(
            v3(-side * pct(w, 76), -pct(w, 6), pct(w, 30)),
            v3(pct(w, 34), pct(w, 30), pct(w, 34)),
        ),
        shade(c, 86),
    );
}

/// A foot: a sole, three toes and a big toe set off to the inside, as apes have.
fn foot(rig: &mut Rig, f: &Frame, ankle: V3, w: i32, side: i32, c: Rgb) {
    let f = Frame { o: ankle, ..*f };
    rig.put(
        Mesh::Bevel,
        f.shape(
            v3(0, -pct(w, 6), pct(w, 40)),
            v3(pct(w, 130), pct(w, 64), pct(w, 180)),
        ),
        c,
    );
    for i in 0..3 {
        let x = side * pct(w, (i - 1) * 30 + 16);
        rig.put(
            Mesh::Bevel,
            f.shape(
                v3(x, -pct(w, 12), pct(w, 134)),
                v3(pct(w, 28), pct(w, 34), pct(w, 44)),
            ),
            shade(c, 90),
        );
    }
    let toe = f.at(-side * pct(w, 66), -pct(w, 8), pct(w, 104));
    let root = f.at(-side * pct(w, 36), -pct(w, 4), pct(w, 76));
    rig.put(Mesh::Taper, rod(root, toe, pct(w, 44)), shade(c, 90));
}

fn limbs(rig: &mut Rig, kind: Kind, l: &Look, m: &Dims, s: &Skel) {
    let ff = &s.feet_frame;
    for (i, side) in SIDES.into_iter().enumerate() {
        // arm: a round shoulder, a tapering upper arm and forearm
        let sh = shoulder_of(&s.body, m, side);
        let (elbow, wrist) = bend(sh, s.hands[i], m.arm, s.arm_hint[i]);
        rig.ball(sh, pct(m.arm_w, 160), l.limb);
        rig.bone(sh, elbow, pct(m.arm_w, 130), l.limb);
        rig.bone(elbow, wrist, pct(m.arm_w, 112), l.limb);
        hand(rig, ff, wrist, m.arm_w, side, l.skin);
        if kind == Kind::Orangutan {
            // long hair hanging from the arms
            for (a, b) in [(sh, elbow), (elbow, wrist)] {
                for j in 1..4 {
                    let top = a.lerp(b, ONE * j / 4);
                    let along = b.sub(a);
                    let bottom = top
                        .sub(v3(0, pct(m.arm_w, 260), 0))
                        .sub(along.scale(ONE / 6));
                    rig.blade(
                        top,
                        bottom,
                        along,
                        pct(m.arm_w, 150),
                        pct(m.arm_w, 50),
                        shade(l.fur, 92 - 4 * j),
                    );
                }
            }
        }
        // leg
        let hip = hip_of(&s.body, m, side);
        let (knee, ankle) = bend(hip, s.feet[i], m.leg, s.leg_hint[i]);
        rig.ball(hip, pct(m.leg_w, 150), l.limb);
        rig.bone(hip, knee, pct(m.leg_w, 130), l.limb);
        rig.bone(knee, ankle, pct(m.leg_w, 110), l.limb);
        foot(rig, ff, ankle, m.leg_w, side, l.skin);
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

/// Two eyes: a faceted white (or orange) ball with a black diamond pupil.
fn eyes(rig: &mut Rig, h: &Frame, d: i32, size: i32, colour: Rgb, y: i32, x: i32) {
    for side in SIDES {
        let e = h.at(side * pct(d, x), pct(d, y), pct(d, 42));
        let es = pct(d, size);
        rig.glow(
            Mesh::Ico,
            h.shape(V3::ZERO, v3(es, es, pct(es, 80))).at(e),
            colour,
            ONE / 3,
        );
        let ps = pct(es, 56);
        rig.put(
            Mesh::Octa,
            h.shape(V3::ZERO, v3(ps, ps, pct(ps, 70)))
                .at(e.add(h.z.scale(pct(es, 30)))),
            BLACK,
        );
    }
}

/// A flat hexagon in the face's plane, `across` wide and `thick` deep (the
/// cheek flanges, the face ring).
fn disc(rig: &mut Rig, h: &Frame, at: V3, across: i32, thick: i32, c: Rgb) {
    let a = h.at(at.x, at.y, at.z);
    rig.put(Mesh::Hex, rod(a, a.add(h.z.scale(thick)), across), c);
}

fn head(rig: &mut Rig, kind: Kind, l: &Look, h: &Frame, pose: &Pose) {
    let d = cm(l.head);
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    // the skull, and the bare face on its front
    rig.put(Mesh::Geo, h.shape(V3::ZERO, p(100, 94, 96)), l.fur);
    let face_size = match kind {
        Kind::Lemur => p(84, 80, 52),
        Kind::Proboscis => p(70, 70, 44),
        Kind::Gibbon => p(68, 70, 44),
        _ => p(78, 74, 44),
    };
    rig.put(Mesh::Geo, h.shape(p(0, -4, 30), face_size), l.face);
    match kind {
        Kind::Orangutan => {
            // the broad cheek flanges, a throat sac and hair standing up
            disc(
                rig,
                h,
                p(0, -6, 14),
                pct(d, 176),
                pct(d, 14),
                rgb(110, 86, 74),
            );
            disc(
                rig,
                h,
                p(0, -6, 12),
                pct(d, 196),
                pct(d, 8),
                shade(l.fur, 90),
            );
            rig.put(Mesh::Geo, h.shape(p(0, -58, 16), p(74, 54, 64)), l.skin);
            for (x, tilt) in [(-30, -30), (-10, -10), (12, 12), (30, 32)] {
                let root = h.at(pct(d, x), pct(d, 34), -pct(d, 6));
                let tip = h.at(pct(d, x + tilt), pct(d, 84), -pct(d, 26));
                rig.blade(root, tip, h.x, pct(d, 26), pct(d, 10), shade(l.fur, 104));
            }
        }
        Kind::Gibbon => {
            // the white ring round the black face, and a tufted crown
            disc(rig, h, p(0, -2, 22), pct(d, 100), pct(d, 10), FACE_RING);
            for (x, tilt) in [(-20, -14), (0, 0), (20, 14)] {
                let root = h.at(pct(d, x), pct(d, 36), -pct(d, 4));
                let tip = h.at(pct(d, x + tilt), pct(d, 66), -pct(d, 18));
                rig.blade(root, tip, h.x, pct(d, 26), pct(d, 10), shade(l.fur, 96));
            }
        }
        Kind::Gorilla => {
            // the crest on top
            rig.put(Mesh::Pyramid, h.shape(p(0, 40, -10), p(40, 40, 74)), l.fur);
        }
        Kind::Chimpanzee => {
            // sideburns
            for side in SIDES {
                let root = h.at(side * pct(d, 40), -pct(d, 10), pct(d, 10));
                let tip = h.at(side * pct(d, 54), -pct(d, 46), pct(d, 6));
                rig.blade(root, tip, h.z, pct(d, 30), pct(d, 10), shade(l.fur, 110));
            }
        }
        Kind::Proboscis => {
            // a russet cap
            rig.put(
                Mesh::Geo,
                h.shape(p(0, 22, -6), p(96, 62, 92)),
                shade(l.fur, 88),
            );
        }
        Kind::Lemur => {}
    }
    if l.brow > 0 {
        let c = if kind == Kind::Gorilla { l.fur } else { l.face };
        rig.put(
            Mesh::Wedge,
            h.shape(p(0, 20, 38), p(86, 10 + l.brow / 2, l.brow + 14)),
            shade(c, 90),
        );
    }
    if l.ear > 0 {
        // flat hexagonal ears, a darker hollow in each
        for side in SIDES {
            let e = l.ear;
            let at = h.at(side * pct(d, 46), pct(d, 4), 0);
            let out = h.x.scale(side * pct(d, 8));
            rig.put(Mesh::Hex, rod(at, at.add(out), pct(d, e)), l.skin);
            let at = at.add(out);
            rig.put(
                Mesh::Hex,
                rod(at, at.add(out.scale(ONE / 3)), pct(d, pct(e, 60))),
                shade(l.skin, 70),
            );
        }
    }
    if l.muzzle > 0 {
        let mz = l.muzzle;
        let muzzle = if kind == Kind::Orangutan || kind == Kind::Chimpanzee {
            shade(l.face, 110)
        } else {
            l.face
        };
        rig.put(
            Mesh::Bevel,
            h.shape(p(0, -26, 42), p(pct(mz, 110), pct(mz, 74), pct(mz, 90))),
            muzzle,
        );
        // nostrils and mouth
        for side in SIDES {
            rig.put(
                Mesh::Octa,
                h.shape(p(side * 7, -16, 40 + pct(mz, 46)), p(9, 6, 6)),
                BLACK,
            );
        }
        rig.put(
            Mesh::Wedge,
            h.shape(p(0, -36, 40 + pct(mz, 44)), p(pct(mz, 74), 4, 5)),
            BLACK,
        );
    }
    match kind {
        Kind::Lemur => {
            // the black mask, the pointed snout and the pricked ears
            for side in SIDES {
                let top = h.at(side * pct(d, 16), pct(d, 22), pct(d, 50));
                let bottom = h.at(side * pct(d, 28), -pct(d, 12), pct(d, 42));
                rig.blade(top, bottom, h.x, pct(d, 34), pct(d, 12), BLACK);
                let base = h.at(side * pct(d, 30), pct(d, 28), -pct(d, 4));
                let tip = h.at(side * pct(d, 54), pct(d, 72), -pct(d, 10));
                rig.blade(base, tip, h.x, pct(d, 36), pct(d, 12), l.fur);
                rig.blade(
                    base.add(h.z.scale(pct(d, 4))),
                    base.lerp(tip, ONE * 3 / 4).add(h.z.scale(pct(d, 4))),
                    h.x,
                    pct(d, 20),
                    pct(d, 8),
                    l.face,
                );
            }
            rig.put(
                Mesh::Geo,
                h.shape(p(0, 30, -8), p(80, 50, 80)),
                shade(l.fur, 80),
            );
            let a = h.at(0, -pct(d, 14), pct(d, 34));
            let b = h.at(0, -pct(d, 20), pct(d, 90));
            rig.put(Mesh::Spike, rod(a, b, pct(d, 44)), l.face);
            rig.put(
                Mesh::Octa,
                h.shape(V3::ZERO, p(16, 12, 14))
                    .at(b.sub(h.z.scale(pct(d, 4)))),
                BLACK,
            );
            eyes(rig, h, d, l.eye, LEMUR_EYE, 10, 20);
        }
        Kind::Proboscis => {
            eyes(rig, h, d, l.eye, EYE_WHITE, 14, 15);
            // a small mouth, under the nose
            rig.put(Mesh::Wedge, h.shape(p(0, -30, 44), p(26, 4, 6)), BLACK);
            // the nose widens to a heavy bulb that hangs over the mouth
            let n = nose_points(h, d, pose.vel, pose.tick);
            let widths = [pct(d, 26), pct(d, 32), pct(d, 42)];
            rig.put(Mesh::Ico, h.shape(V3::ZERO, p(24, 24, 24)).at(n[0]), NOSE);
            for i in 0..3 {
                rig.put(Mesh::Taper, rod(n[i + 1], n[i], widths[i]), NOSE);
            }
            rig.put(Mesh::Geo, h.shape(V3::ZERO, p(60, 54, 56)).at(n[3]), NOSE);
        }
        _ => eyes(rig, h, d, l.eye, EYE_WHITE, 8, 17),
    }
}

fn tail(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, tick: u32) {
    let (count, seg, w0, w1, bend_by) = match kind {
        Kind::Lemur => (12, cm(8), cm(9), cm(7), s.tail_bend),
        Kind::Proboscis => (8, cm(9), cm(6), cm(4), s.tail_bend / 3),
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
    let mut way = back;
    for i in 0..count {
        let a = bend_by * (i + 1);
        let sway = mul(fx::sin(spin(tick, deg(3)) - i * deg(20)), cm(25));
        way = dir_or(
            back.scale(fx::cos(a))
                .add(curl.scale(fx::sin(a)))
                .add(side.scale(sway)),
            back,
        );
        let next = at.add(way.scale(seg));
        let w = w0 + (w1 - w0) * i / count;
        if kind == Kind::Lemur {
            // crisp rings, white and black
            let c = if i % 2 == 0 {
                rgb(242, 242, 240)
            } else {
                BLACK
            };
            rig.put(Mesh::Hex, rod(at, next, w), c);
        } else {
            rig.put(Mesh::Taper, rod(at, next, pct(w, 120)), TAIL_GREY);
            rig.ball(next, pct(w, 80), TAIL_GREY);
        }
        at = next;
    }
    // the tip: a black point, or a pale tuft
    if kind == Kind::Lemur {
        rig.put(Mesh::Spike, rod(at, at.add(way.scale(seg)), w1), BLACK);
    } else {
        let tip = at.add(way.scale(seg / 2));
        rig.blade(at, tip, side, w1 * 3, w1, rgb(232, 228, 220));
    }
}

fn stars(rig: &mut Rig, h: &Frame, d: i32, tick: u32) {
    for i in 0..4 {
        let a = spin(tick, deg(5)) + i * QUARTER;
        let r = pct(d, 75);
        let p =
            h.o.add(V3::UP.scale(pct(d, 80)))
                .add(v3(mul(fx::cos(a), r), 0, mul(fx::sin(a), r)));
        let size = pct(d, 26);
        rig.glow(
            Mesh::Octa,
            Affine::translate(p)
                .then(&Affine::rot_y(a * 2))
                .then(&Affine::scale(v3(size, size, size / 2))),
            STAR,
            ONE,
        );
    }
}

// ---------------------------------------------------------------------------
// The jaguar (Floor is Lava, TRIAGE T37)

/// The jaguar's coat: tawny, a cream belly and muzzle, dark rosettes with
/// deeper tawny middles.
const JAG_COAT: Rgb = rgb(214, 156, 64);
const JAG_DEEP: Rgb = rgb(170, 108, 40);
const JAG_CREAM: Rgb = rgb(240, 224, 186);
const JAG_SPOT: Rgb = rgb(38, 28, 22);
const JAG_EYE: Rgb = rgb(206, 196, 58);
const JAG_NOSE: Rgb = rgb(120, 70, 60);
const JAG_MOUTH: Rgb = rgb(150, 40, 46);
const JAG_FANG: Rgb = rgb(250, 248, 236);

/// Where the rosettes sit on the body: centimetres along the spine from its
/// middle, and degrees round it from the top toward the side.
const ROSETTES: [(i32, i32); 22] = [
    (40, 0),
    (40, 55),
    (40, -55),
    (22, 30),
    (22, -30),
    (22, 85),
    (22, -85),
    (4, 0),
    (4, 58),
    (4, -58),
    (4, 105),
    (4, -105),
    (-14, 30),
    (-14, -30),
    (-14, 85),
    (-14, -85),
    (-32, 0),
    (-32, 56),
    (-32, -56),
    (-46, 30),
    (-46, -30),
    (-48, 85),
];

/// Draws the jaguar posed as `j`, at `scale` percent of life size (the
/// racers' `scene::RACE_SCALE`, so it reads beside them): the same faceted
/// low-poly style as the apes, on four legs with a long ringed tail.
pub fn jaguar(b: &mut Batch, j: &crate::jaguar::Jaguar, scale: i32) {
    let k = (ONE as i64 * scale.max(1) as i64 / 100) as i32;
    let mut rig = Rig {
        b,
        base: Affine::translate(j.feet).then(&Affine::scale(v3(k, k, k))),
    };
    let fwd = j.fwd();
    let up = V3::UP;
    // eating: the shoulders drop and the spine tips forward
    let tip = mul(j.head_down, deg(12));
    let spine = fwd.scale(fx::cos(tip)).sub(up.scale(fx::sin(tip)));
    let bob = mul(fx::sin(j.gait * 2).abs(), mul(cm(4), j.stride));
    let middle = up.scale(cm(60) + bob - mul(j.head_down, cm(6)));
    let body = Frame::upright(middle, spine, up);
    let p = |x: i32, y: i32, z: i32| v3(cm(x), cm(y), cm(z));
    // the torso: a barrel along the spine, the chest and haunches, the belly
    rig.put(
        Mesh::Barrel,
        body.shape(p(0, 0, 0), p(44, 112, 48)),
        JAG_COAT,
    );
    rig.put(Mesh::Geo, body.shape(p(0, 38, -1), p(46, 46, 52)), JAG_COAT);
    rig.put(Mesh::Geo, body.shape(p(0, -40, 2), p(44, 46, 50)), JAG_COAT);
    rig.put(
        Mesh::Geo,
        body.shape(p(0, 0, -14), p(34, 82, 26)),
        JAG_CREAM,
    );
    // the rosettes: dark rings round deeper tawny middles
    for (along, round) in ROSETTES {
        let a = deg(round);
        let n = body.dir(fx::sin(a), 0, fx::cos(a));
        let at = body.at(mul(fx::sin(a), cm(23)), cm(along), mul(fx::cos(a), cm(25)));
        rig.put(
            Mesh::Hex,
            rod(at.sub(n.scale(cm(1))), at.add(n.scale(cm(2))), cm(11)),
            JAG_SPOT,
        );
        rig.put(Mesh::Hex, rod(at, at.add(n.scale(cm(3))), cm(5)), JAG_DEEP);
    }
    jaguar_legs(&mut rig, j, &body, fwd);
    jaguar_head(&mut rig, j, &body, fwd);
    jaguar_tail(&mut rig, j, &body);
}

fn jaguar_legs(rig: &mut Rig, j: &crate::jaguar::Jaguar, body: &Frame, fwd: V3) {
    let up = V3::UP;
    let flat = Frame::upright(V3::ZERO, up, fwd);
    // front left and right, hind left and right: a gallop's order
    let legs: [(i32, bool, i32); 4] = [
        (-1, true, 0),
        (1, true, TURN / 10),
        (-1, false, HALF),
        (1, false, HALF + TURN / 10),
    ];
    for (side, front, offset) in legs {
        let (along, len, w) = if front {
            (cm(40), cm(62), cm(15))
        } else {
            (-cm(42), cm(66), cm(17))
        };
        let root = body.at(-side * cm(13), along, -cm(8));
        let under = v3(root.x, cm(4), root.z);
        let lp = j.gait + offset;
        let stride = mul(j.stride, cm(30));
        let lift = mul(j.stride, cm(14));
        let run = under
            .add(fwd.scale(mul(fx::sin(lp), stride)))
            .add(up.scale(mul(fx::cos(lp).max(0), lift)));
        // the leap: forelegs reach out ahead, hind legs push back
        let reach = if front {
            root.add(fwd.scale(cm(58))).sub(up.scale(cm(22)))
        } else {
            root.sub(fwd.scale(cm(58))).sub(up.scale(cm(28)))
        };
        // eating: the forepaws a little ahead, holding on
        let eat = if front {
            under.add(fwd.scale(cm(16)))
        } else {
            under
        };
        let target = run
            .lerp(reach, j.leap)
            .lerp(eat, if front { j.head_down } else { 0 });
        let hint = if front { fwd.neg() } else { fwd };
        let (mid, end) = bend(root, target, len, hint);
        rig.bone(root, mid, w, JAG_COAT);
        rig.bone(mid, end, pct(w, 72), shade(JAG_COAT, 94));
        // a spot or two on the upper leg
        rig.ball(
            root.lerp(mid, ONE / 2).add(flat.x.scale(side * pct(w, 40))),
            cm(5),
            JAG_SPOT,
        );
        // the paw
        let paw = Frame::upright(end, up, fwd);
        rig.put(
            Mesh::Bevel,
            paw.shape(v3(0, 0, cm(3)), p3(12, 7, 15)),
            JAG_CREAM,
        );
    }
}

/// Centimetres as a Q16 vector.
fn p3(x: i32, y: i32, z: i32) -> V3 {
    v3(cm(x), cm(y), cm(z))
}

fn jaguar_head(rig: &mut Rig, j: &crate::jaguar::Jaguar, body: &Frame, fwd: V3) {
    let up = V3::UP;
    // up and ahead of the shoulders; down to the ground, eating
    let high = body.at(0, cm(76), cm(14));
    let low = v3(0, cm(20), 0).add(fwd.scale(cm(98)));
    let at = high.lerp(low, j.head_down);
    let neck = body.at(0, cm(46), cm(6));
    rig.put(Mesh::Taper, rod(neck, at, cm(28)), JAG_COAT);
    let a = mul(j.head_down, deg(55));
    let head_up = up.scale(fx::cos(a)).add(fwd.scale(fx::sin(a)));
    let head_fwd = fwd.scale(fx::cos(a)).sub(up.scale(fx::sin(a)));
    let h = Frame::upright(at, head_up, head_fwd);
    rig.put(Mesh::Geo, h.shape(V3::ZERO, p3(30, 26, 32)), JAG_COAT);
    rig.put(
        Mesh::Bevel,
        h.shape(p3(0, -5, 16), p3(17, 12, 14)),
        JAG_CREAM,
    );
    rig.put(Mesh::Octa, h.shape(p3(0, -1, 23), p3(7, 4, 5)), JAG_NOSE);
    // the lower jaw, hinged under the cheeks, open by `jaw`
    let open = mul(j.jaw, deg(42));
    let jaw_fwd = h.z.scale(fx::cos(open)).sub(h.y.scale(fx::sin(open)));
    let jaw_up = h.y.scale(fx::cos(open)).add(h.z.scale(fx::sin(open)));
    let hinge = h.at(0, -cm(9), cm(4));
    rig.put(
        Mesh::Bevel,
        Affine {
            c0: h.x.scale(cm(14)),
            c1: jaw_up.scale(cm(5)),
            c2: jaw_fwd.scale(cm(15)),
            t: hinge.add(jaw_fwd.scale(cm(8))),
        },
        JAG_CREAM,
    );
    if j.jaw > ONE / 5 {
        // the open mouth and its fangs
        rig.put(
            Mesh::Bevel,
            h.shape(p3(0, -10, 12), p3(12, 4, 12)),
            JAG_MOUTH,
        );
        for s in SIDES {
            let top = h.at(s * cm(4), -cm(10), cm(20));
            rig.put(
                Mesh::Spike,
                rod(top, top.sub(h.y.scale(cm(5))), cm(2)),
                JAG_FANG,
            );
            let low = hinge.add(jaw_fwd.scale(cm(13))).add(h.x.scale(s * cm(4)));
            rig.put(
                Mesh::Spike,
                rod(low, low.add(jaw_up.scale(cm(4))), cm(2)),
                JAG_FANG,
            );
        }
    }
    for s in SIDES {
        // eyes, round ears with pale insides, spots on the brow
        rig.put(Mesh::Ico, h.shape(p3(s * 7, 4, 13), p3(6, 5, 5)), JAG_EYE);
        rig.put(Mesh::Octa, h.shape(p3(s * 7, 4, 15), p3(2, 4, 2)), BLACK);
        let ear = h.at(s * cm(10), cm(13), -cm(3));
        rig.put(
            Mesh::Hex,
            rod(ear, ear.add(h.z.scale(cm(3))), cm(10)),
            JAG_SPOT,
        );
        let inner = ear.add(h.z.scale(cm(2)));
        rig.put(
            Mesh::Hex,
            rod(inner, inner.add(h.z.scale(cm(2))), cm(6)),
            JAG_CREAM,
        );
        rig.ball(h.at(s * cm(6), cm(11), cm(6)), cm(3), JAG_SPOT);
    }
    rig.ball(h.at(0, cm(13), cm(2)), cm(3), JAG_SPOT);
}

fn jaguar_tail(rig: &mut Rig, j: &crate::jaguar::Jaguar, body: &Frame) {
    let back = body.y.neg();
    let side = body.x;
    let mut at = body.at(0, -cm(60), cm(8));
    let count = 10;
    for i in 0..count {
        // out and down, the end curling up; swaying with the stride
        let droop = deg(-15 + i * 12).min(deg(70)) - if i >= 7 { deg((i - 6) * 28) } else { 0 };
        let sway = mul(fx::sin(j.gait - i * deg(25)), ONE / 6);
        let way = dir_or(
            back.scale(fx::cos(droop))
                .sub(V3::UP.scale(fx::sin(droop)))
                .add(side.scale(sway)),
            back,
        );
        let next = at.add(way.scale(cm(9)));
        let w = cm(8) - cm(4) * i / count;
        let c = if i >= 5 && i % 2 == 1 {
            JAG_SPOT
        } else {
            JAG_COAT
        };
        rig.put(Mesh::Hex, rod(at, next, w), c);
        at = next;
    }
    rig.ball(at, cm(5), JAG_SPOT);
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
    let need_w = fx::div(cm(420), mul(tan, aspect.max(ONE / 4)));
    let need_h = fx::div(cm(110), tan);
    let back = need_w.max(need_h) + cm(60);
    let target = v3(0, cm(80), 0);
    let eye = v3(0, cm(120), back);
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
        for d in draws.chunks(crate::render::DRAW_VALUES) {
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
    fn the_jaguar_is_a_spotted_cat_at_the_racers_scale() {
        // TRIAGE T37: four legs, a tail, rosettes; twice life size, as the
        // racers are drawn, so it reads next to them
        let base = crate::jaguar::Jaguar {
            feet: v3(int(5), int(1), int(7)),
            yaw: deg(30),
            gait: deg(70),
            stride: ONE,
            leap: 0,
            head_down: 0,
            jaw: 0,
        };
        let leap = crate::jaguar::Jaguar {
            leap: ONE,
            jaw: ONE,
            stride: 0,
            ..base
        };
        let eat = crate::jaguar::Jaguar {
            head_down: ONE,
            jaw: ONE / 2,
            stride: 0,
            ..base
        };
        for j in [base, leap, eat] {
            let mut b = Batch::new();
            jaguar(&mut b, &j, 200);
            let s = shapes(b);
            assert!((80..=250).contains(&s.len()), "{} shapes", s.len());
            let spots = s.iter().filter(|x| x.colour == JAG_SPOT).count();
            assert!(spots >= 20, "{spots} rosettes and spots");
            let top = s.iter().map(|x| x.at.y).max().unwrap() - j.feet.y;
            let fwd = j.fwd();
            let along: Vec<i32> = s.iter().map(|x| x.at.sub(j.feet).dot(fwd)).collect();
            let long = along.iter().max().unwrap() - along.iter().min().unwrap();
            assert!(
                (cm(110)..=cm(260)).contains(&top),
                "{} cm tall",
                top * 100 / ONE
            );
            assert!(long >= cm(300), "{} cm long", long * 100 / ONE);
        }
        // the leap reaches out further than the run
        let reach = |j: &crate::jaguar::Jaguar| {
            let mut b = Batch::new();
            jaguar(&mut b, j, 200);
            shapes(b)
                .iter()
                .map(|x| x.at.sub(j.feet).dot(j.fwd()))
                .max()
                .unwrap()
        };
        assert!(reach(&leap) > reach(&base));
    }

    #[test]
    fn every_kind_draws_a_reasonable_number_of_shapes() {
        for kind in Kind::ALL {
            for pose in poses() {
                let n = drawn(kind, &pose).len();
                assert!((50..=250).contains(&n), "{kind:?} {n} shapes");
            }
        }
    }

    #[test]
    fn every_kind_looks_different() {
        let sig = |kind: Kind| {
            let s = drawn(kind, &at_rest());
            let mut per_mesh = [0usize; Mesh::ALL.len()];
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
                    .filter(|s| s.mesh == Mesh::Ico as u32)
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
