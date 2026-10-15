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

use crate::dance::Dance;
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
    /// what the ape wears (the map's: skis on the Ski Slope)
    pub gear: Gear,
    /// past the finish, its kind's dance (TRIAGE T53; `dance::of`)
    pub dance: Option<Dance>,
}

/// What an ape wears (2026-10-13): nothing, as on the Jungle and the City,
/// or on the Ski Slope a pair of skis, a ski helmet and goggles, or for
/// Sector Ape (the space map, TRIAGE T75, not built yet) a Star Fox flight
/// jacket, one metal arm and aviator sunglasses, or on Sunset Ruins
/// (`Explorer`, 2026-10-14) a pith helmet, a khaki explorer jacket with a
/// compass on the chest, a canteen at the hip and tiny round spectacles,
/// or on Banana Bay Battle (`Pirate`, 2026-10-15) an assortment of pirate
/// pieces, each kind its own mix (`pirate_kit`). Only the drawing changes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Gear {
    #[default]
    None,
    Ski,
    Space,
    Explorer,
    Pirate,
}

impl Gear {
    /// The gear named `key` (`none`, `ski`, `space`, `explorer`, `pirate`;
    /// a tile's `gear=`).
    pub fn from_key(key: &str) -> Option<Gear> {
        match key {
            "none" => Some(Gear::None),
            "ski" => Some(Gear::Ski),
            "space" => Some(Gear::Space),
            "explorer" => Some(Gear::Explorer),
            "pirate" => Some(Gear::Pirate),
            _ => None,
        }
    }

    /// What the apes wear on `map`.
    pub fn on(map: crate::map::Map) -> Gear {
        match map {
            crate::map::Map::Ski => Gear::Ski,
            crate::map::Map::Sector => Gear::Space,
            crate::map::Map::Ruins => Gear::Explorer,
            crate::map::Map::Bay => Gear::Pirate,
            _ => Gear::None,
        }
    }
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
            gear: Gear::None,
            dance: None,
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

/// How many colour palettes each kind has (TRIAGE T49): 0 is the classic
/// (`look`'s own colours), then three alternates.
pub const PALETTES: u8 = 4;

/// A palette's colours, as `Look` names them: fur, limbs, chest, face and
/// skin.
type Coat = [Rgb; 5];

/// `kind`'s alternate palette `palette` (1 to 3; the classic and anything
/// past the last are `None`). Each keeps the kind's marks (the gorilla's
/// silver back, the lemur's ringed tail, the gibbon's face ring) and is
/// a coat the kind could have: a darker, a paler and a rarer one.
fn coat(kind: Kind, palette: u8) -> Option<Coat> {
    let l = look(kind);
    let keep = (l.face, l.skin);
    let c = |fur: Rgb, limb: Rgb, chest: Rgb| Some([fur, limb, chest, keep.0, keep.1]);
    match (kind, palette) {
        // copper, strawberry blond, albino
        (Kind::Orangutan, 1) => c(rgb(150, 56, 30), rgb(138, 50, 26), rgb(128, 52, 32)),
        (Kind::Orangutan, 2) => Some([
            rgb(236, 150, 78),
            rgb(224, 138, 66),
            rgb(214, 128, 70),
            rgb(150, 112, 96),
            rgb(110, 84, 72),
        ]),
        (Kind::Orangutan, 3) => Some([
            rgb(238, 232, 222),
            rgb(226, 220, 210),
            rgb(214, 206, 196),
            rgb(236, 170, 160),
            rgb(220, 150, 140),
        ]),
        // silverback grey, mountain brown, Snowflake (the albino)
        (Kind::Gorilla, 1) => c(rgb(128, 128, 138), rgb(100, 100, 110), rgb(90, 90, 98)),
        (Kind::Gorilla, 2) => Some([
            rgb(82, 60, 46),
            rgb(72, 52, 40),
            rgb(96, 74, 58),
            rgb(70, 58, 54),
            rgb(34, 28, 26),
        ]),
        (Kind::Gorilla, 3) => Some([
            rgb(240, 238, 232),
            rgb(228, 226, 220),
            rgb(218, 214, 206),
            rgb(236, 178, 166),
            rgb(222, 160, 150),
        ]),
        // bonobo black with a dark face, russet, golden
        (Kind::Chimpanzee, 1) => Some([
            rgb(26, 24, 28),
            rgb(22, 20, 24),
            rgb(40, 36, 40),
            rgb(60, 48, 46),
            rgb(54, 44, 42),
        ]),
        (Kind::Chimpanzee, 2) => c(rgb(128, 70, 40), rgb(116, 62, 34), rgb(150, 92, 60)),
        (Kind::Chimpanzee, 3) => Some([
            rgb(204, 160, 80),
            rgb(192, 148, 70),
            rgb(222, 184, 110),
            rgb(236, 196, 170),
            rgb(214, 170, 140),
        ]),
        // black with white hands, silver, cinnamon
        (Kind::Gibbon, 1) => c(rgb(30, 28, 32), rgb(26, 24, 28), rgb(42, 40, 44)),
        (Kind::Gibbon, 2) => c(rgb(168, 176, 184), rgb(152, 160, 170), rgb(196, 202, 208)),
        (Kind::Gibbon, 3) => c(rgb(168, 96, 52), rgb(154, 86, 46), rgb(190, 124, 80)),
        // rust, blond, smoke
        (Kind::Proboscis, 1) => c(rgb(160, 64, 36), rgb(186, 178, 168), rgb(226, 200, 168)),
        (Kind::Proboscis, 2) => c(rgb(230, 170, 100), rgb(236, 230, 218), rgb(244, 228, 200)),
        (Kind::Proboscis, 3) => Some([
            rgb(96, 96, 106),
            rgb(176, 176, 184),
            rgb(214, 210, 204),
            rgb(220, 150, 120),
            keep.1,
        ]),
        // red ruffed, black-and-white ruffed, golden
        (Kind::Lemur, 1) => Some([
            rgb(176, 52, 28),
            rgb(36, 30, 30),
            rgb(34, 28, 28),
            rgb(48, 40, 40),
            rgb(30, 26, 26),
        ]),
        (Kind::Lemur, 2) => Some([
            rgb(238, 236, 232),
            rgb(32, 30, 32),
            rgb(36, 34, 36),
            rgb(40, 38, 40),
            rgb(30, 30, 32),
        ]),
        (Kind::Lemur, 3) => c(rgb(214, 160, 70), rgb(226, 180, 100), rgb(244, 222, 170)),
        _ => None,
    }
}

/// `kind`'s build in palette `palette` (TRIAGE T49).
fn look_in(kind: Kind, palette: u8) -> Look {
    let mut l = look(kind);
    if let Some([fur, limb, chest, face, skin]) = coat(kind, palette) {
        l = Look {
            fur,
            limb,
            chest,
            face,
            skin,
            ..l
        };
    }
    l
}

/// How big `kind` is drawn, percent of the common size.
pub fn size_of(kind: Kind) -> i32 {
    look(kind).size
}

#[cfg(test)]
mod palette_tests {
    use super::*;

    fn shapes(kind: Kind, palette: u8) -> (Vec<i32>, Vec<u32>) {
        let mut b = Batch::new();
        draw_in(&mut b, kind, palette, &Pose::standing(V3::ZERO, 0, 7));
        b.finish(crate::render::DYNAMIC_BUFFER)
    }

    /// The instance values without their colours.
    fn placed_only(v: &[i32]) -> Vec<i32> {
        v.chunks(crate::render::INSTANCE_VALUES)
            .flat_map(|s| s[..12].iter().chain(&s[15..]).copied().collect::<Vec<_>>())
            .collect()
    }

    #[test]
    fn each_palette_recolours_the_same_ape() {
        for kind in Kind::ALL {
            let classic = shapes(kind, 0);
            let mut plain = Batch::new();
            draw(&mut plain, kind, &Pose::standing(V3::ZERO, 0, 7));
            assert_eq!(plain.finish(crate::render::DYNAMIC_BUFFER), classic);
            let mut seen = vec![classic.0.clone()];
            for p in 1..PALETTES {
                let alt = shapes(kind, p);
                assert_eq!(alt.1, classic.1, "{kind:?} {p}: other shapes");
                assert_eq!(placed_only(&alt.0), placed_only(&classic.0));
                assert!(!seen.contains(&alt.0), "{kind:?} {p}: colours not its own");
                seen.push(alt.0);
            }
            // past the last palette: the classic
            assert_eq!(shapes(kind, PALETTES), classic);
        }
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
    /// on skis, each foot's axes: along its ski (`Gear::Ski`)
    skis: Option<[Frame; 2]>,
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

fn pose_skel(kind: Kind, l: &Look, pose: &Pose) -> Skel {
    let m = dims(l);
    let face = facing(pose.yaw, 0);
    let tick = pose.tick;
    let mut skel = match pose.rope {
        Some(rope) => swinging(&m, pose, face, dir_or(rope, V3::UP)),
        None if pose.grounded => {
            let run = running(l, &m, pose, face);
            match pose.dance {
                Some(d) if !pose.stunned => {
                    blend(&run, &dancing(kind, &m, pose, face, d.beat), d.weight)
                }
                _ => run,
            }
        }
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
        skis: None,
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
        skis: None,
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
        skis: None,
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
// Dancing past the finish (TRIAGE T53): one dance per kind

/// One beat of a dance, in ticks (two beats a second).
const BEAT: u32 = 60;

/// How far through a cycle `beats` beats long `t` ticks are, as an angle.
fn cycle(t: u32, beats: u32) -> i32 {
    let n = BEAT * beats;
    ((t % n) as i64 * TURN as i64 / n as i64) as i32
}

/// A smooth wave, one round every `beats` beats, shifted by `off`.
fn wave(t: u32, beats: u32, off: i32) -> i32 {
    fx::sin(cycle(t, beats) + off)
}

/// A hump on every beat, 0 on the beat and ONE between: a hop or a bob.
fn hump(t: u32) -> i32 {
    fx::sin(cycle(t, 1) / 2)
}

/// Which beat of the dance `t` is in.
fn beat_of(t: u32) -> u32 {
    t / BEAT
}

/// The whole body's moves this tick, in the dancer's own frame (x to one
/// side, z the way it faces): what the hands do is in [`dance_hand`].
#[derive(Default)]
struct Moves {
    /// everything up (a hop)
    lift: i32,
    /// everything sideways (a step or a sideways hop)
    slide: i32,
    /// the hips alone sideways (a sway)
    sway: i32,
    /// the hips lowered (bent knees)
    squat: i32,
    /// the back's tilt sideways and forward, as parts of 100 up
    lean: [i32; 2],
    /// the chest turned from the way the ape faces
    twist: i32,
    /// each foot from the middle, sideways
    stance: i32,
    /// each foot lifted
    feet_up: [i32; 2],
}

fn dance_moves(kind: Kind, m: &Dims, t: u32) -> Moves {
    let h = hump(t);
    let beat = beat_of(t);
    match kind {
        // the orangutan sways slowly side to side with its long arms up,
        // rocking onto each foot in turn
        Kind::Orangutan => {
            let sw = wave(t, 2, 0);
            Moves {
                sway: mul(sw, pct(m.w, 24)),
                squat: mul(h, pct(m.leg, 14)),
                lean: [-sw * 14 / ONE, 0],
                twist: mul(sw, deg(10)),
                stance: pct(m.w, 40),
                feet_up: SIDES.map(|s| mul((-s * sw).max(0), pct(m.leg, 18))),
                ..Moves::default()
            }
        }
        // the gorilla stomps and beats its chest for two beats, then leaps
        // and flexes for two
        Kind::Gorilla => {
            let flex = beat % 4 >= 2;
            let stomp = |s: i32| {
                if !flex && beat.is_multiple_of(2) == (s < 0) {
                    mul(h, pct(m.leg, 40))
                } else {
                    0
                }
            };
            let sw = if beat.is_multiple_of(2) { h } else { -h };
            Moves {
                lift: if beat % 4 == 2 {
                    mul(h, pct(m.leg, 70))
                } else {
                    0
                },
                sway: if flex { 0 } else { mul(sw, pct(m.w, 12)) },
                squat: pct(m.leg, 16) + if flex { mul(ONE - h, pct(m.leg, 8)) } else { 0 },
                lean: [
                    if flex { 0 } else { sw * 8 / ONE },
                    if flex { -14 } else { 6 },
                ],
                stance: pct(m.w, 46),
                feet_up: [stomp(-1), stomp(1)],
                ..Moves::default()
            }
        }
        // the chimpanzee does jumping jacks, clapping over its head
        Kind::Chimpanzee => Moves {
            lift: mul(h, pct(m.leg, 70)),
            squat: mul(ONE - h, pct(m.leg, 12)),
            stance: pct(m.w, 22) + mul(h, pct(m.w, 60)),
            ..Moves::default()
        },
        // the gibbon windmills its long arms, twisting, and every eighth
        // beat spins right round (`dance_spin`)
        Kind::Gibbon => Moves {
            squat: mul(h, pct(m.leg, 12)),
            twist: mul(wave(t, 2, 0), deg(18)),
            lean: [wave(t, 2, QUARTER) * 10 / ONE, 0],
            stance: pct(m.w, 55),
            ..Moves::default()
        },
        // the proboscis monkey's disco: hips swinging, belly out, one hand
        // on the hip and the other pointing up to the sky and down
        Kind::Proboscis => {
            let sw = wave(t, 2, 0);
            Moves {
                sway: mul(sw, pct(m.w, 30)),
                squat: mul(h, pct(m.leg, 12)),
                lean: [-sw * 10 / ONE, -8],
                twist: -mul(sw, deg(14)),
                stance: pct(m.w, 34),
                ..Moves::default()
            }
        }
        // the lemur hops sideways, four hops one way and four back, like a
        // dancing sifaka, with its arms up
        Kind::Lemur => {
            let n = t % (BEAT * 8);
            let way = if n < BEAT * 4 { 1 } else { -1 };
            let travel = if n < BEAT * 4 { n } else { BEAT * 8 - n } as i32;
            let span = pct(m.leg, 110);
            Moves {
                lift: mul(h, pct(m.leg, 80)),
                slide: (travel - BEAT as i32 * 2) * span / (BEAT as i32 * 2),
                squat: mul(ONE - h, pct(m.leg, 18)),
                lean: [way * 16, 0],
                twist: way * deg(15),
                stance: pct(m.w, 30),
                feet_up: [mul(h, pct(m.leg, 20)); 2],
                ..Moves::default()
            }
        }
    }
}

/// The whole ape's extra turn this tick, applied to the pose's facing so
/// the skis turn with it: the gibbon's spin every eighth beat.
fn dance_spin(kind: Kind, t: u32) -> i32 {
    match kind {
        Kind::Gibbon if beat_of(t) % 8 == 7 => cycle(t, 1),
        _ => 0,
    }
}

/// A dancing ape's pose: its facing with the dance's spin, blended in.
fn dance_turn(kind: Kind, pose: &Pose) -> Pose {
    match pose.dance {
        Some(d) if pose.grounded && pose.rope.is_none() => Pose {
            yaw: fx::wrap(pose.yaw + mul(dance_spin(kind, d.beat), d.weight)),
            ..*pose
        },
        _ => *pose,
    }
}

/// Hand `s`'s target and its elbow's hint this tick.
fn dance_hand(kind: Kind, m: &Dims, body: &Frame, s: i32, t: u32) -> (V3, V3) {
    let beat = beat_of(t);
    let sh = shoulder_of(body, m, s);
    // out from the shoulder along (x outward, y, z), `r` percent of the arm
    let out = |x: i32, y: i32, z: i32, r: i32| sh.add(body.dir(s * x, y, z).scale(pct(m.arm, r)));
    // a point on the torso: x outward, percent of its width, height, depth
    let on = |x: i32, y: i32, z: i32| body.at(s * pct(m.w, x), pct(m.t, y), pct(m.d, z));
    let hint = |x: i32, y: i32, z: i32| body.dir(s * x, y, z);
    match kind {
        Kind::Orangutan => {
            // both arms up, waving together the way the hips do not and
            // pumping on every beat ("raise the roof")
            let sw = wave(t, 2, 0);
            let reach = 62 + 34 * hump(t) / ONE;
            (out(30 + s * sw * 45 / ONE, 90, 15, reach), hint(70, 0, -30))
        }
        // a flex: fists up by the head, elbows out
        Kind::Gorilla if beat % 4 >= 2 => (out(80, 55, 20, 45), hint(100, -40, -10)),
        Kind::Gorilla => {
            // fists drumming on the chest, one then the other, fast
            let n = (t % 30) as i32 * TURN / 30;
            let hit = fx::sin(n + if s > 0 { HALF } else { 0 }).max(0);
            let wind = on(62, 80, 120);
            let chest = on(24, 72, 66);
            (wind.lerp(chest, hit), hint(100, -30, -40))
        }
        Kind::Chimpanzee => {
            // arms down at the sides, then up to clap over the head
            let down = out(30, -90, 15, 92);
            let clap = body.at(s * cm(3), pct(m.t, 88) + pct(m.arm, 88), pct(m.d, 40));
            (down.lerp(clap, hump(t)), hint(100, 10, 0))
        }
        Kind::Gibbon => {
            // windmills, the arms half a turn apart (flattened at the
            // bottom: the long arms would reach the ground)
            let a = cycle(t, 2) + if s > 0 { HALF } else { 0 };
            let x = 40 + 60 * fx::cos(a) / ONE;
            let y = fx::sin(a).max(-ONE * 3 / 10) * 100 / ONE;
            (out(x, y, 35, 96), hint(0, -30, -100))
        }
        Kind::Proboscis => {
            // the pointing hand swaps every two bars
            let pointer = if (beat / 4).is_multiple_of(2) { 1 } else { -1 };
            if s == pointer {
                let u = (wave(t, 2, -QUARTER) + ONE) / 2;
                let low = out(-35, -55, 50, 92);
                let high = out(55, 85, 25, 98);
                (low.lerp(high, u), hint(60, -30, -40))
            } else {
                (on(70, 28, 20), hint(100, 0, -40))
            }
        }
        Kind::Lemur => {
            // arms up and out, the hands fluttering
            let flutter = 18 * wave(t, 1, s * QUARTER) / ONE;
            (out(60, 70 + flutter, 25, 95), hint(60, -40, -30))
        }
    }
}

/// A dancing ape on the ground, its feet at the anchor's level, facing
/// `face`; `t` ticks into its kind's dance.
fn dancing(kind: Kind, m: &Dims, pose: &Pose, face: V3, t: u32) -> Skel {
    let mv = dance_moves(kind, m, t);
    let flat = Frame::upright(V3::ZERO, V3::UP, face);
    let up = flat.dir(mv.lean[0], 100, mv.lean[1]);
    let chest = flat.dir(fx::sin(mv.twist), 0, fx::cos(mv.twist));
    let hips = flat.at(mv.slide + mv.sway, pct(m.leg, 92) - mv.squat + mv.lift, 0);
    let body = Frame::upright(hips, up, chest);
    let mut hands = [V3::ZERO; 2];
    let mut arm_hint = [V3::ZERO; 2];
    let mut feet = [V3::ZERO; 2];
    for (i, s) in SIDES.into_iter().enumerate() {
        (hands[i], arm_hint[i]) = dance_hand(kind, m, &body, s, t);
        feet[i] = flat.at(
            mv.slide + s * mv.stance,
            m.leg_w / 2 + mv.lift + mv.feet_up[i],
            0,
        );
    }
    // the tail out behind (standing up, a drooping one would reach the
    // ground); the lemur's up, swinging with the hops
    let tail_back = if kind == Kind::Lemur {
        flat.dir(-mv.lean[0], 60, -100)
    } else {
        flat.dir(0, 30, -100)
    };
    Skel {
        anchor: pose.hands.sub(v3(0, BODY_DROP, 0)),
        body,
        head: body,
        feet_frame: flat,
        skis: None,
        hands,
        feet,
        arm_hint,
        leg_hint: SIDES.map(|s| flat.dir(s * 35, 0, 100)),
        tail_back,
        tail_curl: V3::UP,
        tail_bend: deg(9),
    }
}

/// A frame part way from `a` to `b`.
fn mix_frame(a: &Frame, b: &Frame, w: i32) -> Frame {
    Frame::upright(a.o.lerp(b.o, w), a.y.lerp(b.y, w), a.z.lerp(b.z, w))
}

/// One skeleton part way to another (an ape slowing into its dance).
fn blend(a: &Skel, b: &Skel, w: i32) -> Skel {
    let lerp2 = |p: [V3; 2], q: [V3; 2]| [p[0].lerp(q[0], w), p[1].lerp(q[1], w)];
    Skel {
        anchor: a.anchor.lerp(b.anchor, w),
        body: mix_frame(&a.body, &b.body, w),
        head: mix_frame(&a.head, &b.head, w),
        feet_frame: mix_frame(&a.feet_frame, &b.feet_frame, w),
        skis: None,
        hands: lerp2(a.hands, b.hands),
        feet: lerp2(a.feet, b.feet),
        arm_hint: lerp2(a.arm_hint, b.arm_hint),
        leg_hint: lerp2(a.leg_hint, b.leg_hint),
        tail_back: a.tail_back.lerp(b.tail_back, w),
        tail_curl: a.tail_curl.lerp(b.tail_curl, w),
        tail_bend: fx::lerp(a.tail_bend, b.tail_bend, w),
    }
}

// ---------------------------------------------------------------------------
// Drawing

/// Draws one ape, in its classic colours.
pub fn draw(b: &mut Batch, kind: Kind, pose: &Pose) {
    draw_in(b, kind, 0, pose);
}

/// Draws one ape in palette `palette` (TRIAGE T49; 0 is the classic).
pub fn draw_in(b: &mut Batch, kind: Kind, palette: u8, pose: &Pose) {
    let l = look_in(kind, palette);
    let m = dims(&l);
    let pose = &dance_turn(kind, pose);
    let mut skel = pose_skel(kind, &l, pose);
    if pose.gear == Gear::Ski {
        skel.skis = Some(ski_frames(pose));
    }
    let k = (ONE as i64 * l.size as i64 * pose.scale.max(1) as i64 / 10_000) as i32;
    let mut base = Affine::translate(skel.anchor).then(&Affine::scale(v3(k, k, k)));
    if pose.gear == Gear::Ski && pose.grounded && pose.rope.is_none() {
        // on the ground the skis are under the feet, so the ape stands on
        // them, not in the snow
        base = base.then(&Affine::translate(v3(0, SKI_THICK, 0)));
    }
    let mut rig = Rig { b, base };
    let space = pose.gear == Gear::Space;
    let explorer = pose.gear == Gear::Explorer;
    torso(&mut rig, kind, &l, &m, &skel);
    limbs(&mut rig, kind, &l, &m, &skel, pose.gear);
    head(&mut rig, kind, &l, &skel.head, pose);
    tail(&mut rig, kind, &m, &skel, pose.tick);
    if pose.gear == Gear::Ski {
        skis(&mut rig, kind, &m, &skel);
        helmet(&mut rig, kind, &l, &skel.head);
    }
    if space {
        jacket(&mut rig, kind, &m, &skel, pose);
        shades(&mut rig, kind, &l, &skel.head);
    }
    if explorer {
        safari_jacket(&mut rig, kind, &m, &skel);
        pith_helmet(&mut rig, cm(l.head), &skel.head);
        spectacles(&mut rig, kind, &l, &skel.head);
    }
    if pose.gear == Gear::Pirate {
        pirate(&mut rig, kind, &l, &m, &skel);
    }
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

/// `gear`: in the Sector Ape costume, one arm (`metal_side`) is metal from
/// the elbow down, and the jacket's sleeves cover the upper arms.
fn limbs(rig: &mut Rig, kind: Kind, l: &Look, m: &Dims, s: &Skel, gear: Gear) {
    let ff = &s.feet_frame;
    let space = gear == Gear::Space;
    // in a jacket the upper arms are in sleeves
    let sleeved = space || matches!(gear, Gear::Explorer | Gear::Pirate);
    let kit = pirate_kit(kind);
    let pirate = gear == Gear::Pirate;
    for (i, side) in SIDES.into_iter().enumerate() {
        // arm: a round shoulder, a tapering upper arm and forearm
        let sh = shoulder_of(&s.body, m, side);
        let (elbow, wrist) = bend(sh, s.hands[i], m.arm, s.arm_hint[i]);
        rig.ball(sh, pct(m.arm_w, 160), l.limb);
        rig.bone(sh, elbow, pct(m.arm_w, 130), l.limb);
        let metal = space && side == metal_side(kind);
        if metal {
            metal_arm(rig, ff, m, elbow, wrist, side);
        } else if pirate && kit.hook == Some(side) {
            rig.bone(elbow, wrist, pct(m.arm_w, 112), l.limb);
            hook(rig, ff, wrist, m.arm_w);
        } else {
            rig.bone(elbow, wrist, pct(m.arm_w, 112), l.limb);
            hand(rig, ff, wrist, m.arm_w, side, l.skin);
        }
        if kind == Kind::Orangutan {
            // long hair hanging from the arms (not from a sleeve or metal)
            let bare: &[(V3, V3)] = match (sleeved, metal) {
                (false, _) => &[(sh, elbow), (elbow, wrist)],
                (true, false) => &[(elbow, wrist)],
                (true, true) => &[],
            };
            for &(a, b) in bare {
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
        if pirate && kit.peg == Some(side) {
            peg_leg(rig, knee, ankle, m.leg_w);
            continue;
        }
        rig.bone(knee, ankle, pct(m.leg_w, 110), l.limb);
        let on = s.skis.map_or(*ff, |f| f[i]);
        foot(rig, &on, ankle, m.leg_w, side, l.skin);
    }
}

// ---------------------------------------------------------------------------
// The Ski Slope's gear (2026-10-13): skis, a ski helmet and goggles

/// A ski's thickness, length and width (the model's metres, so a bigger
/// kind has longer skis); twice that as drawn in a race.
const SKI_THICK: i32 = cm(3);
const SKI_LEN: i32 = cm(112);
const SKI_WIDE: i32 = cm(9);

/// Each kind's gear colours: the helmet and the skis, the skis' stripe,
/// and the goggles' mirrored lens (gold on the cool helmets, blue on the
/// warm ones), all bright against the snow and the ape's own fur.
fn gear_colours(kind: Kind) -> (Rgb, Rgb, Rgb) {
    const GOLD: Rgb = rgb(255, 176, 30);
    const BLUE: Rgb = rgb(70, 170, 255);
    match kind {
        Kind::Orangutan => (rgb(30, 110, 235), rgb(255, 255, 255), GOLD),
        Kind::Gorilla => (rgb(230, 34, 40), rgb(255, 230, 60), BLUE),
        Kind::Chimpanzee => (rgb(255, 204, 20), rgb(30, 30, 36), BLUE),
        Kind::Gibbon => (rgb(170, 60, 215), rgb(255, 255, 255), GOLD),
        Kind::Proboscis => (rgb(20, 180, 130), rgb(255, 255, 255), GOLD),
        Kind::Lemur => (rgb(255, 110, 20), rgb(30, 30, 36), BLUE),
    }
}

const BINDING: Rgb = rgb(40, 40, 46);
const STRAP: Rgb = rgb(24, 24, 28);

/// Each foot's axes on skis: along the way the ape faces, pitched with its
/// velocity (on the ground that follows the slope; in the air the skis ride
/// the flight, nose a little up), and in the air splayed into a V.
fn ski_frames(pose: &Pose) -> [Frame; 2] {
    let face = facing(pose.yaw, 0);
    let side = V3::UP.cross(face);
    let airborne = !pose.grounded || pose.rope.is_some();
    let pitch = if pose.vel.len() < ONE {
        0
    } else if airborne {
        (crate::geom::pitch_of(pose.vel) * 3 / 5 + deg(6)).clamp(-deg(30), deg(25))
    } else {
        crate::geom::pitch_of(pose.vel).clamp(-deg(20), deg(20))
    };
    SIDES.map(|s| {
        let splay = if airborne { s * deg(8) } else { 0 };
        let fwd = face.scale(fx::cos(splay)).add(side.scale(fx::sin(splay)));
        let z = fwd.scale(fx::cos(pitch)).add(V3::UP.scale(fx::sin(pitch)));
        let y = V3::UP.scale(fx::cos(pitch)).sub(fwd.scale(fx::sin(pitch)));
        Frame::upright(V3::ZERO, y, z)
    })
}

/// A box along a frame's axes, turned up about its x by `tilt` (an `fx`
/// angle): the length runs from `from` along the frame's z, tipped up.
fn board(rig: &mut Rig, f: &Frame, from: V3, tilt: i32, size: V3, c: Rgb) -> V3 {
    let z = f.z.scale(fx::cos(tilt)).add(f.y.scale(fx::sin(tilt)));
    let y = f.y.scale(fx::cos(tilt)).sub(f.z.scale(fx::sin(tilt)));
    let mid = from.add(z.scale(size.z / 2));
    rig.put(
        Mesh::Cube,
        Affine {
            c0: f.x.scale(size.x),
            c1: y.scale(size.y),
            c2: z.scale(size.z),
            t: mid,
        },
        c,
    );
    from.add(z.scale(size.z))
}

/// Two skis, one under each foot, pointing the way the feet point (along
/// the feet's frame); in the air they splay a little.
fn skis(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel) {
    let (shell, stripe, _) = gear_colours(kind);
    for (i, side) in SIDES.into_iter().enumerate() {
        let hip = hip_of(&s.body, m, side);
        let (_, ankle) = bend(hip, s.feet[i], m.leg, s.leg_hint[i]);
        let Some(frames) = s.skis else {
            return;
        };
        let f = Frame {
            o: ankle,
            ..frames[i]
        };
        let w = m.leg_w;
        // the sole of the foot, then the ski under it, its binding a bit
        // behind its middle
        let top = -pct(w, 40);
        let y = top - SKI_THICK / 2;
        let flat = pct(SKI_LEN, 74);
        let back = pct(w, 40) - pct(SKI_LEN, 42);
        let tail = f.at(0, y, back);
        let nose = board(rig, &f, tail, 0, v3(SKI_WIDE, SKI_THICK, flat), shell);
        // the tip curls up in two steps; the tail kicks up a little
        let bend1 = board(
            rig,
            &f,
            nose,
            deg(18),
            v3(SKI_WIDE, SKI_THICK, pct(SKI_LEN, 10)),
            shell,
        );
        board(
            rig,
            &f,
            bend1,
            deg(48),
            v3(pct(SKI_WIDE, 92), SKI_THICK, pct(SKI_LEN, 9)),
            stripe,
        );
        let back_f = Frame {
            x: f.x.neg(),
            z: f.z.neg(),
            ..f
        };
        board(
            rig,
            &back_f,
            tail,
            deg(14),
            v3(SKI_WIDE, SKI_THICK, pct(SKI_LEN, 9)),
            shell,
        );
        // a stripe down the top, and the binding round the foot
        rig.put(
            Mesh::Cube,
            f.shape(
                v3(0, top + SKI_THICK / 10, back + flat / 2),
                v3(pct(SKI_WIDE, 34), SKI_THICK / 4, pct(flat, 96)),
            ),
            stripe,
        );
        rig.put(
            Mesh::Bevel,
            f.shape(
                v3(0, top + pct(w, 12), -pct(w, 10)),
                v3(pct(SKI_WIDE, 110), pct(w, 34), pct(w, 70)),
            ),
            BINDING,
        );
        rig.put(
            Mesh::Bevel,
            f.shape(
                v3(0, top + pct(w, 8), pct(w, 120)),
                v3(pct(SKI_WIDE, 104), pct(w, 22), pct(w, 40)),
            ),
            BINDING,
        );
    }
}

/// A ski helmet over the top of the head in the kind's colour, and goggles:
/// a dark strap round the helmet and a big mirrored lens over the eyes.
fn helmet(rig: &mut Rig, kind: Kind, l: &Look, h: &Frame) {
    let (shell, stripe, lens) = gear_colours(kind);
    let d = cm(l.head);
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    rig.put(Mesh::Geo, h.shape(p(0, 26, -4), p(114, 88, 116)), shell);
    // a raised ridge from front to back, and a short peak over the goggles
    rig.put(Mesh::Bevel, h.shape(p(0, 66, -6), p(22, 12, 90)), stripe);
    rig.put(
        Mesh::Wedge,
        h.shape(p(0, 30, 52), p(70, 8, 16)),
        shade(shell, 86),
    );
    // the strap, round the helmet's rim at the eyes
    let a = h.at(0, pct(d, 4), -pct(d, 4));
    rig.put(
        Mesh::Hex,
        rod(
            a.sub(h.y.scale(pct(d, 9))),
            a.add(h.y.scale(pct(d, 9))),
            pct(d, 120),
        ),
        STRAP,
    );
    // the frame and the lens, glowing a little like a mirror in the sun
    rig.put(Mesh::Bevel, h.shape(p(0, 10, 48), p(102, 48, 18)), STRAP);
    rig.glow(
        Mesh::Bevel,
        h.shape(p(0, 10, 56), p(92, 38, 8)),
        lens,
        ONE / 3,
    );
    // a glint across the lens
    rig.glow(
        Mesh::Cube,
        h.shape(p(-22, 19, 60), p(26, 6, 2)),
        rgb(255, 255, 240),
        ONE / 2,
    );
}

// ---------------------------------------------------------------------------
// Sector Ape's costume (TRIAGE T75, the space map after Star Fox): a cropped
// flight jacket with a team scarf, a winged badge and shoulder patches, a
// pilot's headset, one metal arm and aviator sunglasses. The team colour is
// the kind's own from the Ski Slope (`gear_colours`), so each ape keeps its
// colour from map to map.

/// The jacket: off-white with olive trim, as a Star Fox pilot's.
const JACKET: Rgb = rgb(236, 232, 218);
const OLIVE: Rgb = rgb(92, 112, 52);
const BADGE: Rgb = rgb(246, 194, 56);
/// The metal arm: a gunmetal forearm with chrome bands, joint and hand,
/// and a glowing cyan light.
const GUNMETAL: Rgb = rgb(78, 84, 98);
const CHROME: Rgb = rgb(210, 218, 232);
const LED: Rgb = rgb(80, 236, 255);
/// The sunglasses: near-black lenses in a gold frame, a warm glint on each.
const LENS: Rgb = rgb(18, 20, 30);
const FRAME_GOLD: Rgb = rgb(226, 178, 64);
const GLINT: Rgb = rgb(255, 238, 206);

/// Which arm is metal (-1 or 1): it alternates through the kinds, so a
/// pack of apes shows both.
fn metal_side(kind: Kind) -> i32 {
    match kind {
        Kind::Orangutan | Kind::Chimpanzee | Kind::Proboscis => 1,
        Kind::Gorilla | Kind::Gibbon | Kind::Lemur => -1,
    }
}

/// Where the eyes sit on the face: percent of the head up and to each side.
fn eye_spot(kind: Kind) -> (i32, i32) {
    match kind {
        Kind::Lemur => (10, 20),
        Kind::Proboscis => (14, 15),
        _ => (8, 17),
    }
}

/// A forearm and hand of metal from `elbow` to `wrist`: a chrome joint, a
/// faceted gunmetal forearm with two chrome bands, a cyan light on the
/// outside and a chrome hand.
fn metal_arm(rig: &mut Rig, ff: &Frame, m: &Dims, elbow: V3, wrist: V3, side: i32) {
    let w = m.arm_w;
    let j = pct(w, 150);
    rig.glow(
        Mesh::Octa,
        Affine::translate(elbow).then(&Affine::scale(v3(j, j, j))),
        CHROME,
        ONE / 6,
    );
    rig.put(Mesh::Hex, rod(elbow, wrist, pct(w, 104)), GUNMETAL);
    let at = |p: i32| elbow.lerp(wrist, ONE * p / 100);
    for (a, b) in [(28, 40), (60, 72), (88, 100)] {
        rig.glow(Mesh::Hex, rod(at(a), at(b), pct(w, 126)), CHROME, ONE / 6);
    }
    let out = dir_or(ff.x.scale(side), V3::UP);
    let led = at(50).add(out.scale(pct(w, 52)));
    let k = pct(w, 40);
    rig.glow(
        Mesh::Octa,
        Affine::translate(led).then(&Affine::scale(v3(k, k, k))),
        LED,
        ONE,
    );
    hand(rig, ff, wrist, w, side, CHROME);
}

/// How low each kind's jacket comes (percent of the torso up from the
/// hips), and its colour and trim: the gorilla's is cropped higher and
/// olive with off-white trim, so its silver saddle shows below and stands
/// out against it.
fn jacket_cut(kind: Kind) -> (i32, Rgb, Rgb) {
    match kind {
        Kind::Gorilla => (64, OLIVE, JACKET),
        _ => (46, JACKET, OLIVE),
    }
}

/// The flight jacket over the chest and upper arms, its collar, the team
/// scarf (streaming back with the ape's speed), the badge and the patches.
fn jacket(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, pose: &Pose) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    let team = gear_colours(kind).0;
    let (hem, cloth, trim) = jacket_cut(kind);
    let tall = 104 - hem;
    let mid = hem + tall / 2;
    // the body, a little bigger than the ribcage, gathered at the waist
    // by an olive band; a yoke over the shoulders
    rig.put(
        Mesh::Barrel,
        body.shape(at(0, mid, 0), at(112, tall, 114)),
        cloth,
    );
    rig.put(
        Mesh::Barrel,
        body.shape(at(0, hem + 2, 0), at(104, 10, 106)),
        trim,
    );
    rig.put(
        Mesh::Bevel,
        body.shape(
            at(0, 88, -4),
            v3(2 * m.shoulder + pct(m.arm_w, 90), pct(t, 28), pct(d, 86)),
        ),
        cloth,
    );
    // the zip down the front
    rig.put(
        Mesh::Cube,
        body.shape(at(0, mid, 56), v3(pct(w, 7), pct(t, tall - 10), pct(d, 4))),
        trim,
    );
    // a high olive collar
    rig.put(
        Mesh::Hex,
        rod(
            body.at(0, pct(t, 94), 0),
            body.at(0, pct(t, 108), 0),
            pct(w, 70),
        ),
        trim,
    );
    // the winged gold badge on the chest
    let badge = body.at(-pct(w, 24), pct(t, 80), pct(d, 52));
    let b = pct(w, 13);
    rig.glow(
        Mesh::Octa,
        body.shape(V3::ZERO, v3(b, b, pct(d, 8))).at(badge),
        BADGE,
        ONE / 4,
    );
    for side in SIDES {
        let tip = badge
            .add(body.x.scale(side * pct(w, 15)))
            .add(body.y.scale(pct(t, 5)));
        rig.blade(badge, tip, body.y, pct(t, 8), pct(d, 4), shade(BADGE, 90));
    }
    // the scarf: a knot at the throat and a tail streaming back
    rig.ball(body.at(0, pct(t, 100), pct(d, 44)), pct(w, 16), team);
    let speed = pose.vel.len();
    let back = dir_or(pose.vel.neg(), body.z.neg());
    let droop = ONE - fx::div(speed.min(int(10)), int(10)) * 3 / 4;
    let way = dir_or(back.add(V3::UP.scale(-droop)), body.z.neg());
    let flap = mul(fx::sin(spin(pose.tick, deg(25))), pct(t, 10));
    let root = body.at(0, pct(t, 102), -pct(d, 40));
    let tip = root.add(way.scale(pct(t, 60))).add(body.x.scale(flap));
    rig.blade(root, tip, body.x, pct(w, 26), pct(d, 6), team);
    // the sleeves to the elbows, olive cuffs, a team patch on each shoulder
    for (i, side) in SIDES.into_iter().enumerate() {
        let sh = shoulder_of(body, m, side);
        let (elbow, _) = bend(sh, s.hands[i], m.arm, s.arm_hint[i]);
        let aw = m.arm_w;
        rig.ball(sh, pct(aw, 178), cloth);
        rig.put(Mesh::Taper, rod(sh, elbow, pct(aw, 154)), cloth);
        let along = |p: i32| sh.lerp(elbow, ONE * p / 100);
        rig.put(Mesh::Hex, rod(along(78), along(96), pct(aw, 132)), trim);
        let out = body.x.scale(side * ONE);
        let patch = along(24);
        rig.put(
            Mesh::Hex,
            rod(
                patch.add(out.scale(pct(aw, 66))),
                patch.add(out.scale(pct(aw, 82))),
                pct(aw, 96),
            ),
            team,
        );
    }
}

/// Aviator sunglasses over the eyes (two dark drop-shaped lenses in a thin
/// gold frame with arms back to the ears) and a pilot's headset: an
/// earpiece on the other side from the metal arm and a mic on a boom.
fn shades(rig: &mut Rig, kind: Kind, l: &Look, h: &Frame) {
    let d = cm(l.head);
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    let (ey, ex) = eye_spot(kind);
    let lw = (l.eye + 16).min(2 * ex - 3);
    let lh = l.eye + 4;
    // in front of the eyes, and of the brow where it juts out
    let z = (42 + l.eye / 2 + 5).max(38 + (l.brow + 14) / 2 + 3);
    let top = ey - 1 + lh / 2;
    for side in SIDES {
        let x = side * ex;
        rig.put(Mesh::Geo, h.shape(p(x, ey - 1, z), p(lw, lh, 6)), LENS);
        // the drop of an aviator lens, lower and outward
        rig.put(
            Mesh::Geo,
            h.shape(
                p(x + side * 3, ey - 1 - lh / 4, z - 1),
                p(pct(lw, 82), pct(lh, 70), 5),
            ),
            LENS,
        );
        rig.glow(
            Mesh::Cube,
            h.shape(p(x - side * lw / 6, ey + lh / 5, z + 3), p(lw / 3, 3, 2)),
            GLINT,
            ONE / 2,
        );
        // the arm back to the ear
        let a = h.at(side * pct(d, ex + lw / 2), pct(d, top - 2), pct(d, z - 3));
        let b = h.at(side * pct(d, 52), pct(d, ey + 2), 0);
        rig.put(Mesh::Hex, rod(a, b, pct(d, 4)), FRAME_GOLD);
    }
    // the bar across the top of both lenses
    rig.put(
        Mesh::Cube,
        h.shape(p(0, top, z + 1), p(2 * ex + lw, 3, 3)),
        FRAME_GOLD,
    );
    // the headset
    let side = -metal_side(kind);
    let ex = if l.ear > 0 { 56 } else { 48 };
    let ear = h.at(side * pct(d, ex), -pct(d, 2), pct(d, 2));
    rig.put(
        Mesh::Hex,
        rod(ear, ear.add(h.x.scale(side * pct(d, 10))), pct(d, 30)),
        GUNMETAL,
    );
    let mouth = h.at(side * pct(d, 30), -pct(d, 30), pct(d, 50 + l.muzzle / 4));
    rig.put(Mesh::Hex, rod(ear, mouth, pct(d, 5)), GUNMETAL);
    rig.ball(mouth, pct(d, 10), shade(GUNMETAL, 70));
}

// ---------------------------------------------------------------------------
// Sunset Ruins' explorer outfit (2026-10-14, the person's brief): a pith
// helmet, a khaki explorer jacket with a brass compass on the chest, a
// canteen on a strap hanging at the hip, and tiny round spectacles.

/// The jacket's khaki, its darker trim (collar, pockets, belt), the pith
/// helmet's pale cork and its dark band. Pale, cool and grey-leaning with
/// dark trim (SECOND-ORDER RG8): under the Ruins' gold-orange light a warm
/// khaki read the same orange as the fur, so value carries the contrast.
const KHAKI: Rgb = rgb(200, 204, 190);
const KHAKI_DARK: Rgb = rgb(92, 96, 82);
const PITH: Rgb = rgb(248, 246, 238);
const PITH_BAND: Rgb = rgb(46, 40, 38);
/// The compass: a brass case, a cream face and a red needle.
const BRASS: Rgb = rgb(206, 162, 64);
const DIAL: Rgb = rgb(248, 242, 222);
const NEEDLE: Rgb = rgb(212, 40, 36);
/// The canteen: an olive tin body, a steel cap, a leather strap.
const CANTEEN: Rgb = rgb(58, 104, 62);
const STEEL: Rgb = rgb(184, 188, 194);
const LEATHER: Rgb = rgb(64, 38, 22);
/// The spectacles: thin dark-gold wire round pale glass.
const WIRE: Rgb = rgb(150, 116, 48);
const SPECS_GLASS: Rgb = rgb(214, 236, 246);

/// How low each kind's explorer jacket comes (percent of the torso up from
/// the hips): the gorilla's is cropped higher so its silver saddle shows.
fn safari_hem(kind: Kind) -> i32 {
    match kind {
        Kind::Gorilla => 56,
        _ => 30,
    }
}

/// The khaki jacket over the chest and upper arms: a belt at the hem, a
/// turned-down collar, two flapped pockets, the compass on the chest, and
/// a canteen on a strap across the body, hanging at the hip.
fn safari_jacket(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    let hem = safari_hem(kind);
    let tall = 104 - hem;
    let mid = hem + tall / 2;
    rig.put(
        Mesh::Barrel,
        body.shape(at(0, mid, 0), at(112, tall, 114)),
        KHAKI,
    );
    // the belt, with a brass buckle
    rig.put(
        Mesh::Barrel,
        body.shape(at(0, hem + 4, 0), at(106, 9, 108)),
        KHAKI_DARK,
    );
    rig.put(
        Mesh::Bevel,
        body.shape(at(0, hem + 4, 54), at(14, 8, 4)),
        BRASS,
    );
    rig.put(
        Mesh::Bevel,
        body.shape(
            at(0, 88, -4),
            v3(2 * m.shoulder + pct(m.arm_w, 90), pct(t, 28), pct(d, 86)),
        ),
        KHAKI,
    );
    // the buttoned front
    rig.put(
        Mesh::Cube,
        body.shape(at(0, mid, 56), v3(pct(w, 5), pct(t, tall - 8), pct(d, 4))),
        KHAKI_DARK,
    );
    // a turned-down collar: two flaps meeting at the throat
    for side in SIDES {
        rig.blade(
            body.at(side * pct(w, 18), pct(t, 102), pct(d, 40)),
            body.at(side * pct(w, 6), pct(t, 90), pct(d, 56)),
            body.x,
            pct(w, 16),
            pct(d, 4),
            KHAKI_DARK,
        );
    }
    // the flapped pockets low on the front
    if tall > 40 {
        for side in SIDES {
            rig.put(
                Mesh::Bevel,
                body.shape(at(side * 28, hem + 18, 54), at(26, 18, 6)),
                shade(KHAKI, 94),
            );
            rig.put(
                Mesh::Bevel,
                body.shape(at(side * 28, hem + 27, 57), at(28, 5, 5)),
                KHAKI_DARK,
            );
        }
    }
    // the compass on the chest: a brass case with a ring on top, a cream
    // face and a red needle pointing up
    let c = body.at(-pct(w, 24), pct(t, 76), pct(d, 56));
    let out = body.z;
    let r = pct(w, 19);
    rig.glow(
        Mesh::Hex,
        rod(c, c.add(out.scale(pct(d, 6))), r),
        BRASS,
        ONE / 5,
    );
    let face = c.add(out.scale(pct(d, 6)));
    rig.glow(
        Mesh::Hex,
        rod(face, face.add(out.scale(pct(d, 2))), pct(r, 78)),
        DIAL,
        ONE / 6,
    );
    let front = face.add(out.scale(pct(d, 2)));
    rig.put(
        Mesh::Octa,
        body.shape(V3::ZERO, v3(pct(r, 16), pct(r, 64), pct(d, 3)))
            .at(front.add(body.y.scale(pct(r, 14)))),
        NEEDLE,
    );
    rig.put(
        Mesh::Octa,
        body.shape(V3::ZERO, v3(pct(r, 14), pct(r, 40), pct(d, 3)))
            .at(front.sub(body.y.scale(pct(r, 18)))),
        BLACK,
    );
    rig.put(
        Mesh::Hex,
        rod(
            c.add(body.y.scale(pct(r, 50))),
            c.add(body.y.scale(pct(r, 72))),
            pct(r, 26),
        ),
        BRASS,
    );
    // the canteen's strap from one shoulder across the chest to the other
    // hip, and the canteen hanging outside that hip with a steel cap
    let strap_top = body.at(pct(w, 34), pct(t, 100), pct(d, 40));
    let strap_low = body.at(-pct(w, 52), pct(t, 22), pct(d, 50));
    rig.put(Mesh::Hex, rod(strap_top, strap_low, pct(w, 9)), LEATHER);
    let back_top = body.at(pct(w, 34), pct(t, 100), -pct(d, 40));
    let back_low = body.at(-pct(w, 52), pct(t, 22), -pct(d, 50));
    rig.put(Mesh::Hex, rod(back_top, back_low, pct(w, 9)), LEATHER);
    let can = body.at(-pct(w, 62), pct(t, 10), 0);
    let side_out = body.x.neg();
    let across = pct(w, 44);
    rig.put(
        Mesh::Hex,
        rod(can, can.add(side_out.scale(pct(w, 16))), across),
        CANTEEN,
    );
    rig.put(
        Mesh::Hex,
        rod(
            can.add(side_out.scale(pct(w, 2))),
            can.add(side_out.scale(pct(w, 14))),
            pct(across, 110),
        ),
        shade(CANTEEN, 80),
    );
    let neck = can.add(side_out.scale(pct(w, 8)));
    let up = body.y;
    rig.put(
        Mesh::Hex,
        rod(
            neck.add(up.scale(pct(across, 48))),
            neck.add(up.scale(pct(across, 64))),
            pct(across, 26),
        ),
        STEEL,
    );
    // the sleeves to the elbows, with turned-back cuffs
    for (i, side) in SIDES.into_iter().enumerate() {
        let sh = shoulder_of(body, m, side);
        let (elbow, _) = bend(sh, s.hands[i], m.arm, s.arm_hint[i]);
        let aw = m.arm_w;
        rig.ball(sh, pct(aw, 178), KHAKI);
        rig.put(Mesh::Taper, rod(sh, elbow, pct(aw, 154)), KHAKI);
        let along = |p: i32| sh.lerp(elbow, ONE * p / 100);
        rig.put(
            Mesh::Hex,
            rod(along(80), along(98), pct(aw, 136)),
            KHAKI_DARK,
        );
    }
}

/// The pith helmet: a pale domed shell, a wide flat brim all round, a
/// brown band and a little vent button on top, sized for a head `d` across
/// (an ape's `Look::head`, or the Sunset Ruins jaguar's).
fn pith_helmet(rig: &mut Rig, d: i32, h: &Frame) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    rig.put(Mesh::Geo, h.shape(p(0, 40, -4), p(110, 84, 116)), PITH);
    let band = h.at(0, pct(d, 28), -pct(d, 4));
    rig.put(
        Mesh::Hex,
        rod(band, band.add(h.y.scale(pct(d, 12))), pct(d, 116)),
        PITH_BAND,
    );
    // the brim: a wide hexagonal disc, a little longer front to back
    let brim = h.at(0, pct(d, 26), -pct(d, 2));
    rig.put(
        Mesh::Hex,
        Affine {
            c0: h.x.scale(pct(d, 168)),
            c1: h.y.scale(pct(d, 5)),
            c2: h.z.scale(pct(d, 184)),
            t: brim,
        },
        shade(PITH, 94),
    );
    rig.put(Mesh::Ico, h.shape(p(0, 82, -4), p(14, 10, 14)), PITH_BAND);
}

/// Tiny round spectacles: two small glass discs in thin wire rims, a bridge
/// between them and arms back to the ears.
fn spectacles(rig: &mut Rig, kind: Kind, l: &Look, h: &Frame) {
    let d = cm(l.head);
    let (ey, ex) = eye_spot(kind);
    let lw = l.eye + 1;
    let z = (42 + l.eye / 2 + 4).max(38 + (l.brow + 14) / 2 + 3);
    for side in SIDES {
        let c = h.at(side * pct(d, ex), pct(d, ey), pct(d, z));
        rig.put(
            Mesh::Hex,
            rod(c, c.add(h.z.scale(pct(d, 2))), pct(d, lw + 4)),
            WIRE,
        );
        let g = c.add(h.z.scale(pct(d, 2)));
        rig.glow(
            Mesh::Hex,
            rod(g, g.add(h.z.scale(pct(d, 2))), pct(d, lw)),
            SPECS_GLASS,
            ONE / 3,
        );
        // a glint on each lens
        rig.glow(
            Mesh::Cube,
            h.shape(
                v3(
                    side * pct(d, ex) - pct(d, lw / 5),
                    pct(d, ey + lw / 5),
                    pct(d, z + 5),
                ),
                v3(pct(d, (lw / 3).max(3)), pct(d, 2), pct(d, 2)),
            ),
            rgb(255, 255, 255),
            ONE / 2,
        );
        // the arm back to the ear
        let a = h.at(side * pct(d, ex + lw / 2 + 1), pct(d, ey), pct(d, z));
        let b = h.at(side * pct(d, 50), pct(d, ey + 2), 0);
        rig.put(Mesh::Hex, rod(a, b, pct(d, 3)), WIRE);
    }
    // the bridge over the nose
    let a = h.at(-pct(d, ex - lw / 2), pct(d, ey + 1), pct(d, z + 1));
    let b = h.at(pct(d, ex - lw / 2), pct(d, ey + 1), pct(d, z + 1));
    rig.put(Mesh::Hex, rod(a, b, pct(d, 3)), WIRE);
}

// ---------------------------------------------------------------------------
// Banana Bay Battle's pirates (2026-10-15, the person's brief): an
// assortment of pirate pieces, each kind its own mix so the six read apart
// (`pirate_kit`). Saturated, near-black or near-white pieces, so they stand
// apart from each kind's fur by value under the 3 pm sun.

/// The hats: a black tricorn with gold trim, and bandanas.
const TRICORN: Rgb = rgb(30, 26, 30);
const TRIM_GOLD: Rgb = rgb(238, 190, 52);
const PLUME: Rgb = rgb(250, 250, 246);
const BONE: Rgb = rgb(244, 240, 226);
const BANDANA_RED: Rgb = rgb(208, 30, 40);
const BANDANA_NAVY: Rgb = rgb(26, 42, 104);
const BANDANA_GOLD: Rgb = rgb(244, 192, 30);
const DOT: Rgb = rgb(252, 252, 250);
const PATCH: Rgb = rgb(14, 12, 14);
/// The tops: the captain's crimson coat over a white front, the striped
/// shirt, the loose cream shirt and the navy waistcoat.
const CAPTAIN_RED: Rgb = rgb(196, 30, 42);
const SHIRT_WHITE: Rgb = rgb(246, 246, 240);
const STRIPE_NAVY: Rgb = rgb(30, 40, 92);
const SHIRT_CREAM: Rgb = rgb(242, 234, 212);
const WAISTCOAT: Rgb = rgb(28, 40, 86);
/// Belts, the bandolier and its cartridges, the scabbard and the cutlass's
/// brass hilt.
const BELT: Rgb = rgb(26, 20, 18);
const BANDOLIER_DARK: Rgb = rgb(58, 34, 18);
const CARTRIDGE: Rgb = rgb(222, 178, 70);
const SCABBARD: Rgb = rgb(36, 24, 20);
const HILT: Rgb = rgb(230, 184, 60);
const GRIP: Rgb = rgb(90, 52, 30);
/// The sashes, the peg leg and the hook.
const SASH_MAROON: Rgb = rgb(128, 18, 44);
const SASH_RED: Rgb = rgb(214, 36, 40);
const PEG_WOOD: Rgb = rgb(138, 92, 48);
const PEG_IRON: Rgb = rgb(44, 44, 50);
const HOOK_STEEL: Rgb = rgb(206, 210, 220);
/// A gold hoop earring.
const HOOP: Rgb = rgb(250, 200, 50);
/// The parrots, as body (and head), wings, tail and face accent: a green
/// amazon and a scarlet macaw.
const AMAZON: [Rgb; 4] = [
    rgb(44, 168, 64),
    rgb(36, 90, 210),
    rgb(220, 40, 40),
    rgb(250, 214, 40),
];
const MACAW: [Rgb; 4] = [
    rgb(222, 36, 36),
    rgb(36, 96, 220),
    rgb(250, 200, 30),
    rgb(250, 250, 246),
];
const BEAK: Rgb = rgb(236, 226, 196);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PirateHat {
    /// bare-headed
    None,
    /// the tricorn; `true`: the captain's, with a plume and a skull
    Tricorn(bool),
    /// a bandana, with polka dots of the second colour
    Bandana(Rgb, Option<Rgb>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PirateTop {
    None,
    CaptainsCoat,
    Striped,
    Shirt,
    Waistcoat,
}

/// One kind's pirate pieces.
#[derive(Debug, Clone, Copy)]
struct PirateKit {
    hat: PirateHat,
    top: PirateTop,
    /// the eye under the patch (-1 or 1)
    patch: Option<i32>,
    bandolier: Option<Rgb>,
    sash: Option<Rgb>,
    cutlass: bool,
    /// the leg that is a peg, the hand that is a hook, the ear with a hoop
    peg: Option<i32>,
    hook: Option<i32>,
    earring: Option<i32>,
    /// the shoulder the parrot sits on, and its colours
    parrot: Option<(i32, [Rgb; 4])>,
}

/// Each kind's pirate mix: the gorilla is the captain (tricorn with a
/// plume and a skull, crimson coat, cutlass); the chimpanzee a deckhand
/// (red bandana with white dots, eyepatch, striped shirt, gold earring);
/// the orangutan the parrot's (navy bandana, cream shirt, bandolier, green
/// parrot); the gibbon a swordsman (tricorn, maroon sash, peg leg,
/// cutlass); the proboscis monkey the purser (white bandana with red dots,
/// eyepatch, navy waistcoat buttoned over its pot belly, cutlass); the
/// lemur the lookout (gold bandana, scarlet macaw, red sash, a hook).
fn pirate_kit(kind: Kind) -> PirateKit {
    let bare = PirateKit {
        hat: PirateHat::None,
        top: PirateTop::None,
        patch: None,
        bandolier: None,
        sash: None,
        cutlass: false,
        peg: None,
        hook: None,
        earring: None,
        parrot: None,
    };
    match kind {
        Kind::Gorilla => PirateKit {
            hat: PirateHat::Tricorn(true),
            top: PirateTop::CaptainsCoat,
            cutlass: true,
            ..bare
        },
        Kind::Chimpanzee => PirateKit {
            hat: PirateHat::Bandana(BANDANA_RED, Some(DOT)),
            top: PirateTop::Striped,
            patch: Some(1),
            earring: Some(-1),
            ..bare
        },
        Kind::Orangutan => PirateKit {
            hat: PirateHat::Bandana(BANDANA_NAVY, None),
            top: PirateTop::Shirt,
            bandolier: Some(BANDOLIER_DARK),
            parrot: Some((-1, AMAZON)),
            ..bare
        },
        Kind::Gibbon => PirateKit {
            hat: PirateHat::Tricorn(false),
            sash: Some(SASH_MAROON),
            cutlass: true,
            peg: Some(1),
            ..bare
        },
        Kind::Proboscis => PirateKit {
            hat: PirateHat::Bandana(DOT, Some(BANDANA_RED)),
            top: PirateTop::Waistcoat,
            patch: Some(-1),
            cutlass: true,
            ..bare
        },
        Kind::Lemur => PirateKit {
            hat: PirateHat::Bandana(BANDANA_GOLD, None),
            sash: Some(SASH_RED),
            hook: Some(1),
            parrot: Some((-1, MACAW)),
            ..bare
        },
    }
}

/// Everything a pirate wears but the peg leg and the hook (`limbs` draws
/// those in place of a foot and a hand).
fn pirate(rig: &mut Rig, kind: Kind, l: &Look, m: &Dims, s: &Skel) {
    let kit = pirate_kit(kind);
    let d = cm(l.head);
    let h = &s.head;
    match kit.hat {
        PirateHat::None => {}
        PirateHat::Tricorn(captain) => tricorn(rig, d, h, captain),
        PirateHat::Bandana(c, dots) => bandana(rig, d, h, c, dots),
    }
    if let Some(side) = kit.patch {
        eyepatch(rig, kind, l, h, side);
    }
    if let Some(side) = kit.earring {
        hoop(rig, d, h, side);
    }
    match kit.top {
        PirateTop::None => {}
        PirateTop::CaptainsCoat => captains_coat(rig, m, s),
        PirateTop::Striped => striped_shirt(rig, m, s),
        PirateTop::Shirt => loose_shirt(rig, m, s),
        PirateTop::Waistcoat => waistcoat(rig, m, s),
    }
    if let Some(c) = kit.sash {
        sash(rig, m, s, c);
    }
    if let Some(c) = kit.bandolier {
        bandolier(rig, m, s, c);
    }
    if kit.cutlass {
        cutlass(rig, m, s);
    }
    if let Some((side, colours)) = kit.parrot {
        parrot(rig, m, s, d, side, colours);
    }
}

/// A black tricorn: a crown, a flat brim and three upturned sides meeting
/// in corners (one to the front), gold along their tops; the captain's has
/// a white plume and a white skull and crossbones on the front.
fn tricorn(rig: &mut Rig, d: i32, h: &Frame, captain: bool) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    rig.put(Mesh::Geo, h.shape(p(0, 48, -4), p(86, 54, 88)), TRICORN);
    let floor = h.at(0, pct(d, 34), -pct(d, 4));
    rig.put(
        Mesh::Hex,
        rod(floor, floor.add(h.y.scale(pct(d, 4))), pct(d, 100)),
        TRICORN,
    );
    let r = pct(d, 96);
    let corner = |k: i32| {
        let a = deg(120 * k);
        h.at(
            mul(fx::sin(a), r),
            pct(d, 50),
            mul(fx::cos(a), r) - pct(d, 4),
        )
    };
    for k in 0..3 {
        let (a, b) = (corner(k), corner(k + 1));
        let along = b.sub(a);
        let out = dir_or(along.cross(h.y), h.z);
        // leaning out a little
        let up = h.y.add(out.scale(ONE / 4));
        rig.put(
            Mesh::Bevel,
            Affine {
                c0: along,
                c1: up.scale(pct(d, 30)),
                c2: out.scale(pct(d, 6)),
                t: a.lerp(b, ONE / 2),
            },
            TRICORN,
        );
        let lift = up.scale(pct(d, 15));
        rig.put(
            Mesh::Hex,
            rod(a.add(lift), b.add(lift), pct(d, 6)),
            TRIM_GOLD,
        );
    }
    if captain {
        let base = h.at(pct(d, 30), pct(d, 64), -pct(d, 20));
        rig.blade(
            base,
            base.add(h.dir(30, 50, -70).scale(pct(d, 70))),
            h.y,
            pct(d, 26),
            pct(d, 8),
            PLUME,
        );
        // the skull on the crown's front, over crossed bones
        let at = h.at(0, pct(d, 54), pct(d, 40));
        rig.put(
            Mesh::Ico,
            h.shape(V3::ZERO, p(18, 16, 10))
                .at(at.add(h.z.scale(pct(d, 2)))),
            BONE,
        );
        for s in SIDES {
            let a = at.add(h.x.scale(s * pct(d, 16))).add(h.y.scale(pct(d, 12)));
            let b = at.sub(h.x.scale(s * pct(d, 16))).sub(h.y.scale(pct(d, 12)));
            rig.put(Mesh::Hex, rod(a, b, pct(d, 5)), BONE);
        }
    }
}

/// A bandana over the top of the head, knotted at the back with two tails,
/// with polka `dots` of that colour.
fn bandana(rig: &mut Rig, d: i32, h: &Frame, c: Rgb, dots: Option<Rgb>) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    let (cy, cz) = (36, -6);
    let size = p(108, 60, 108);
    rig.put(Mesh::Geo, h.shape(p(0, cy, cz), size), c);
    let knot = h.at(0, pct(d, 30), -pct(d, 58));
    rig.ball(knot, pct(d, 20), shade(c, 88));
    for s in SIDES {
        rig.blade(
            knot,
            knot.add(h.dir(s * 30, -70, -40).scale(pct(d, 44))),
            h.x,
            pct(d, 20),
            pct(d, 6),
            shade(c, 92),
        );
    }
    if let Some(dot) = dots {
        // on the cap's surface
        for (x, y, z) in [(-40, 50, 60), (40, 50, 60), (0, 90, 40)] {
            let n = v3(int(x), int(y), int(z)).norm();
            let at = h.at(
                mul(n.x, size.x / 2),
                pct(d, cy) + mul(n.y, size.y / 2),
                pct(d, cz) + mul(n.z, size.z / 2),
            );
            rig.ball(at, pct(d, 13), dot);
        }
    }
}

/// A black patch over the eye on `side`, its strap round the head rising
/// over the other ear.
fn eyepatch(rig: &mut Rig, kind: Kind, l: &Look, h: &Frame, side: i32) {
    let d = cm(l.head);
    let (ey, ex) = eye_spot(kind);
    let z = (42 + l.eye / 2 + 4).max(38 + (l.brow + 14) / 2 + 3);
    let c = h.at(side * pct(d, ex), pct(d, ey), pct(d, z));
    rig.put(
        Mesh::Hex,
        rod(c, c.add(h.z.scale(pct(d, 3))), pct(d, l.eye + 10)),
        PATCH,
    );
    // the strap's plane passes through the patch and tips up by a half
    // toward the far side: y = ey + ex / 2 - side * x / 2
    let n = h.dir(side * 50, 100, 0);
    let o = h.at(0, pct(d, ey + ex / 2), 0);
    rig.put(
        Mesh::Hex,
        rod(
            o.sub(n.scale(pct(d, 2))),
            o.add(n.scale(pct(d, 2))),
            pct(d, 116),
        ),
        PATCH,
    );
}

/// A gold hoop hanging from the ear on `side`.
fn hoop(rig: &mut Rig, d: i32, h: &Frame, side: i32) {
    let r = pct(d, 9);
    let c = h.at(side * pct(d, 52), -pct(d, 18) - r, pct(d, 2));
    let point = |k: i32| {
        let a = deg(60 * k);
        c.add(h.y.scale(mul(fx::cos(a), r)))
            .add(h.z.scale(mul(fx::sin(a), r)))
    };
    for k in 0..6 {
        rig.glow(
            Mesh::Hex,
            rod(point(k), point(k + 1), pct(d, 4)),
            HOOP,
            ONE / 6,
        );
    }
}

/// A garment round the torso from `hem` to `top` (percent of the torso up
/// from the hips), `wide` percent of the torso across and `deep` percent
/// of its depth, with a yoke over the shoulders when it reaches them.
#[allow(clippy::too_many_arguments)]
fn garment(rig: &mut Rig, m: &Dims, s: &Skel, hem: i32, top: i32, wide: i32, deep: i32, c: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let tall = top - hem;
    rig.put(
        Mesh::Barrel,
        body.shape(
            v3(0, pct(t, hem + tall / 2), 0),
            v3(pct(w, wide), pct(t, tall), pct(d, deep)),
        ),
        c,
    );
    if top > 96 {
        rig.put(
            Mesh::Bevel,
            body.shape(
                v3(0, pct(t, 88), -pct(d, 4)),
                v3(2 * m.shoulder + pct(m.arm_w, 90), pct(t, 28), pct(d, 86)),
            ),
            c,
        );
    }
}

/// Sleeves over each arm from the shoulder, `to` percent of the way to the
/// elbow (over 100: on down the forearm, 200 the wrist), `puff` percent of
/// the arm's width, ending in a `cuff`.
fn sleeves(rig: &mut Rig, m: &Dims, s: &Skel, to: i32, puff: i32, c: Rgb, cuff: Rgb) {
    for (i, side) in SIDES.into_iter().enumerate() {
        let sh = shoulder_of(&s.body, m, side);
        let (elbow, wrist) = bend(sh, s.hands[i], m.arm, s.arm_hint[i]);
        let aw = m.arm_w;
        rig.ball(sh, pct(aw, puff + 14), c);
        let (from, end) = if to <= 100 {
            (sh, sh.lerp(elbow, ONE * to / 100))
        } else {
            rig.put(Mesh::Taper, rod(sh, elbow, pct(aw, puff)), c);
            rig.ball(elbow, pct(aw, puff - 10), c);
            (elbow, elbow.lerp(wrist, ONE * (to - 100) / 100))
        };
        rig.put(Mesh::Taper, rod(from, end, pct(aw, puff)), c);
        let back = from.lerp(end, ONE * 78 / 100);
        rig.put(Mesh::Hex, rod(back, end, pct(aw, puff - 6)), cuff);
    }
}

/// The captain's long crimson coat over a white front, gold edging and
/// buttons, sleeves to the wrists with gold cuffs, a black belt with a
/// gold buckle.
fn captains_coat(rig: &mut Rig, m: &Dims, s: &Skel) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    // deep enough to cover the gorilla's chest slabs
    garment(rig, m, s, -18, 104, 116, 130, CAPTAIN_RED);
    rig.put(
        Mesh::Cube,
        body.shape(at(0, 66, 65), at(22, 64, 4)),
        SHIRT_WHITE,
    );
    for side in SIDES {
        rig.put(
            Mesh::Cube,
            body.shape(at(side * 13, 52, 66), at(4, 92, 4)),
            TRIM_GOLD,
        );
        for y in [60, 80] {
            let b = body.at(side * pct(w, 24), pct(t, y), pct(d, 64));
            rig.ball(b, pct(w, 8), TRIM_GOLD);
        }
    }
    rig.put(
        Mesh::Barrel,
        body.shape(at(0, 26, 0), at(122, 9, 136)),
        BELT,
    );
    rig.put(
        Mesh::Bevel,
        body.shape(at(0, 26, 68), at(16, 10, 4)),
        TRIM_GOLD,
    );
    sleeves(rig, m, s, 196, 150, CAPTAIN_RED, TRIM_GOLD);
}

/// A white shirt with three navy stripes round it and short sleeves.
fn striped_shirt(rig: &mut Rig, m: &Dims, s: &Skel) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    garment(rig, m, s, 26, 104, 112, 114, SHIRT_WHITE);
    for y in [40, 58, 76] {
        rig.put(
            Mesh::Barrel,
            body.shape(v3(0, pct(t, y), 0), v3(pct(w, 117), pct(t, 7), pct(d, 119))),
            STRIPE_NAVY,
        );
    }
    sleeves(rig, m, s, 40, 150, SHIRT_WHITE, STRIPE_NAVY);
}

/// A loose cream shirt with puffed sleeves gathered at the elbows.
fn loose_shirt(rig: &mut Rig, m: &Dims, s: &Skel) {
    garment(rig, m, s, 24, 104, 116, 118, SHIRT_CREAM);
    sleeves(rig, m, s, 100, 172, SHIRT_CREAM, shade(SHIRT_CREAM, 86));
}

/// A navy waistcoat, roomy enough to button down over a pot belly, with
/// brass buttons and a black belt.
fn waistcoat(rig: &mut Rig, m: &Dims, s: &Skel) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    garment(rig, m, s, 26, 92, 128, 160, WAISTCOAT);
    for y in [42, 56, 70] {
        let b = body.at(0, pct(t, y), pct(d, 78));
        rig.ball(b, pct(w, 9), CARTRIDGE);
    }
    rig.put(
        Mesh::Barrel,
        body.shape(
            v3(0, pct(t, 30), 0),
            v3(pct(w, 132), pct(t, 8), pct(d, 164)),
        ),
        BELT,
    );
}

/// A sash round the waist, knotted on one side with two tails hanging.
fn sash(rig: &mut Rig, m: &Dims, s: &Skel, c: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    rig.put(
        Mesh::Barrel,
        body.shape(
            v3(0, pct(t, 22), 0),
            v3(pct(w, 116), pct(t, 16), pct(d, 118)),
        ),
        c,
    );
    let knot = body.at(pct(w, 48), pct(t, 22), pct(d, 34));
    rig.ball(knot, pct(w, 22), shade(c, 90));
    for (out, len) in [(10, 50), (-6, 40)] {
        rig.blade(
            knot,
            knot.add(body.dir(out, -100, 10).scale(pct(t, len))),
            body.z,
            pct(w, 18),
            pct(d, 6),
            shade(c, 94),
        );
    }
}

/// A bandolier from one shoulder across the chest and back to the other
/// hip, with brass cartridges along the front.
fn bandolier(rig: &mut Rig, m: &Dims, s: &Skel, c: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let top = body.at(pct(w, 36), pct(t, 100), pct(d, 42));
    let low = body.at(-pct(w, 50), pct(t, 22), pct(d, 54));
    rig.put(Mesh::Hex, rod(top, low, pct(w, 13)), c);
    let back_top = body.at(pct(w, 36), pct(t, 100), -pct(d, 42));
    let back_low = body.at(-pct(w, 50), pct(t, 22), -pct(d, 54));
    rig.put(Mesh::Hex, rod(back_top, back_low, pct(w, 13)), c);
    let along = dir_or(low.sub(top), body.y);
    let across = dir_or(along.cross(body.z), body.x);
    for k in [28, 44, 60, 76] {
        let at = top.lerp(low, ONE * k / 100).add(body.z.scale(pct(d, 5)));
        rig.put(
            Mesh::Bevel,
            Affine {
                c0: along.scale(pct(w, 8)),
                c1: across.scale(pct(w, 14)),
                c2: body.z.scale(pct(d, 6)),
                t: at,
            },
            CARTRIDGE,
        );
    }
}

/// A cutlass in a dark scabbard hanging at the hip, pointing down and
/// back, its brass guard, grip and pommel up by the waist.
fn cutlass(rig: &mut Rig, m: &Dims, s: &Skel) {
    let body = &s.body;
    let (w, t) = (m.w, m.t);
    let top = body.at(-pct(w, 60), pct(t, 26), pct(m.d, 12));
    let down = body.dir(-12, -80, -52);
    let len = pct(t, 118);
    let tip = top.add(down.scale(len));
    rig.put(Mesh::Taper, rod(top, tip, pct(w, 11)), SCABBARD);
    rig.ball(tip, pct(w, 7), HILT);
    rig.put(
        Mesh::Hex,
        rod(top, top.add(down.scale(pct(len, 8))), pct(w, 13)),
        HILT,
    );
    let guard = top.sub(down.scale(pct(len, 3)));
    rig.put(
        Mesh::Hex,
        rod(guard, guard.sub(down.scale(pct(len, 4))), pct(w, 24)),
        HILT,
    );
    let grip_end = guard.sub(down.scale(pct(len, 20)));
    rig.put(Mesh::Hex, rod(guard, grip_end, pct(w, 7)), GRIP);
    rig.ball(grip_end, pct(w, 10), HILT);
}

/// A parrot perched on the shoulder on `side`, facing ahead: body, head,
/// hooked beak, eyes, folded wings, a long tail down the back and a face
/// accent, sized by the ape's head `d`. `colours`: body (and head),
/// wings, tail, accent.
fn parrot(rig: &mut Rig, m: &Dims, s: &Skel, d: i32, side: i32, colours: [Rgb; 4]) {
    let body = &s.body;
    let [plumage, wing, tail, accent] = colours;
    // upright and facing the way the head does, however the ape leans
    let perch = shoulder_of(body, m, side)
        .add(V3::UP.scale(pct(m.arm_w, 90)))
        .add(body.x.scale(side * pct(m.arm_w, 20)));
    let f = Frame::upright(perch, V3::UP, s.head.z);
    // (no smaller than a lemur's head would make it, so it reads)
    let k = pct(d, 90).max(cm(21));
    let p = |x: i32, y: i32, z: i32| v3(pct(k, x), pct(k, y), pct(k, z));
    rig.put(Mesh::Geo, f.shape(p(0, 36, 0), p(36, 58, 40)), plumage);
    rig.put(Mesh::Ico, f.shape(p(0, 78, 6), p(32, 30, 32)), plumage);
    rig.put(Mesh::Ico, f.shape(p(0, 76, 18), p(20, 16, 10)), accent);
    rig.put(
        Mesh::Spike,
        rod(
            f.at(0, pct(k, 76), pct(k, 20)),
            f.at(0, pct(k, 64), pct(k, 34)),
            pct(k, 16),
        ),
        BEAK,
    );
    for x in SIDES {
        rig.put(Mesh::Octa, f.shape(p(x * 14, 84, 14), p(5, 7, 5)), BLACK);
        rig.blade(
            f.at(x * pct(k, 18), pct(k, 58), -pct(k, 2)),
            f.at(x * pct(k, 16), pct(k, 4), -pct(k, 16)),
            f.z,
            pct(k, 26),
            pct(k, 8),
            wing,
        );
    }
    rig.blade(
        f.at(0, pct(k, 14), -pct(k, 14)),
        f.at(0, -pct(k, 60), -pct(k, 34)),
        f.x,
        pct(k, 18),
        pct(k, 6),
        tail,
    );
}

/// A steel hook in place of a hand: a leather cup at the wrist and a
/// curved hook ahead along the hand's axes.
fn hook(rig: &mut Rig, f: &Frame, wrist: V3, w: i32) {
    let cup = wrist.add(f.z.scale(pct(w, 70)));
    rig.put(Mesh::Hex, rod(wrist, cup, pct(w, 150)), GRIP);
    let p1 = cup.add(f.z.scale(pct(w, 110)));
    let p2 = p1.add(f.z.scale(pct(w, 50))).add(f.y.scale(pct(w, 70)));
    let p3 = p2.sub(f.z.scale(pct(w, 50))).add(f.y.scale(pct(w, 40)));
    rig.put(Mesh::Hex, rod(cup, p1, pct(w, 34)), HOOK_STEEL);
    rig.put(Mesh::Hex, rod(p1, p2, pct(w, 32)), HOOK_STEEL);
    rig.put(Mesh::Spike, rod(p2, p3, pct(w, 30)), HOOK_STEEL);
}

/// A wooden peg from the knee in place of the lower leg and foot: a
/// darker cup round the knee, the peg tapering down, an iron tip.
fn peg_leg(rig: &mut Rig, knee: V3, ankle: V3, w: i32) {
    let way = dir_or(ankle.sub(knee), V3::UP.neg());
    let tip = ankle.add(way.scale(pct(w, 50)));
    rig.put(
        Mesh::Hex,
        rod(
            knee.sub(way.scale(pct(w, 40))),
            knee.add(way.scale(pct(w, 50))),
            pct(w, 150),
        ),
        shade(PEG_WOOD, 70),
    );
    rig.put(Mesh::Taper, rod(knee, tip, pct(w, 90)), PEG_WOOD);
    rig.put(
        Mesh::Hex,
        rod(tip.sub(way.scale(pct(w, 20))), tip, pct(w, 56)),
        PEG_IRON,
    );
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
    // (a pirate's tricorn or bandana covers the head too)
    let helmeted = matches!(pose.gear, Gear::Ski | Gear::Explorer | Gear::Pirate);
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
            // (under a ski helmet or a pith helmet the hair stays flat)
            let tufts: &[(i32, i32)] = if helmeted {
                &[]
            } else {
                &[(-30, -30), (-10, -10), (12, 12), (30, 32)]
            };
            for &(x, tilt) in tufts {
                let root = h.at(pct(d, x), pct(d, 34), -pct(d, 6));
                let tip = h.at(pct(d, x + tilt), pct(d, 84), -pct(d, 26));
                rig.blade(root, tip, h.x, pct(d, 26), pct(d, 10), shade(l.fur, 104));
            }
        }
        Kind::Gibbon => {
            // the white ring round the black face, and a tufted crown
            disc(rig, h, p(0, -2, 22), pct(d, 100), pct(d, 10), FACE_RING);
            let crown: &[(i32, i32)] = if helmeted {
                &[]
            } else {
                &[(-20, -14), (0, 0), (20, 14)]
            };
            for &(x, tilt) in crown {
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
            let (y, x) = eye_spot(kind);
            eyes(rig, h, d, l.eye, LEMUR_EYE, y, x);
        }
        Kind::Proboscis => {
            let (y, x) = eye_spot(kind);
            eyes(rig, h, d, l.eye, EYE_WHITE, y, x);
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
        _ => {
            let (y, x) = eye_spot(kind);
            eyes(rig, h, d, l.eye, EYE_WHITE, y, x)
        }
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
const HELMET_RIM: Rgb = rgb(190, 196, 210);
const HELMET_GLASS: Rgb = rgb(170, 220, 255);
/// A halo strength that `scene.frag` draws as glass (2 to 3).
const GLASS: i32 = ONE * 2;

/// The big cat's coat and build: the jaguar's, or on the Ski Slope a snow
/// leopard's (the person, 2026-10-13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BigCat {
    pub coat: Rgb,
    pub deep: Rgb,
    pub cream: Rgb,
    pub spot: Rgb,
    pub eye: Rgb,
    /// the tail's thickness, percent of the jaguar's
    pub tail: i32,
    /// a cartoon bubble space helmet (Sector Ape's jaguar, no spacesuit)
    pub helmet: bool,
    /// the explorers' pith helmet (Sunset Ruins' jaguar)
    pub pith: bool,
    /// a navy officer's bicorne, blue coat and gold epaulettes (Banana Bay
    /// Battle's jaguar)
    pub navy: bool,
}

pub const JAGUAR: BigCat = BigCat {
    coat: JAG_COAT,
    deep: JAG_DEEP,
    cream: JAG_CREAM,
    spot: JAG_SPOT,
    eye: JAG_EYE,
    tail: 100,
    helmet: false,
    pith: false,
    navy: false,
};

/// The jaguar in a classic cartoon bubble space helmet and nothing else,
/// for Sector Ape (the person, 2026-10-13). No map switches it on yet: the
/// space map's session makes `BigCat::on` return it there.
pub const SPACE_JAGUAR: BigCat = BigCat {
    helmet: true,
    ..JAGUAR
};

/// The jaguar in the explorers' pith helmet, for Sunset Ruins (the person,
/// 2026-10-14: "give the jaguar a pith helmet too").
pub const RUINS_JAGUAR: BigCat = BigCat {
    pith: true,
    ..JAGUAR
};

/// The jaguar as a navy officer, for Banana Bay Battle (the person,
/// 2026-10-15: a Royal Navy-style jaguar, "every map it's a jaguar"): a
/// bicorne, a blue coat with a white front and gold epaulettes, and no
/// real nation's insignia.
pub const NAVY_JAGUAR: BigCat = BigCat {
    navy: true,
    ..JAGUAR
};

/// Smoky grey-white with charcoal rosettes round grey middles, pale
/// green-grey eyes and a very thick tail.
pub const SNOW_LEOPARD: BigCat = BigCat {
    coat: rgb(214, 214, 206),
    deep: rgb(168, 168, 162),
    cream: rgb(246, 246, 240),
    spot: rgb(54, 56, 60),
    eye: rgb(176, 204, 178),
    tail: 150,
    helmet: false,
    pith: false,
    navy: false,
};

impl BigCat {
    /// The big cat that hunts on a map.
    pub fn on(map: crate::map::Map) -> BigCat {
        match map {
            crate::map::Map::Ski => SNOW_LEOPARD,
            crate::map::Map::Sector => SPACE_JAGUAR,
            crate::map::Map::Ruins => RUINS_JAGUAR,
            crate::map::Map::Bay => NAVY_JAGUAR,
            _ => JAGUAR,
        }
    }
}

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
pub fn jaguar(b: &mut Batch, j: &crate::jaguar::Jaguar, scale: i32, cat: &BigCat) {
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
        cat.coat,
    );
    rig.put(Mesh::Geo, body.shape(p(0, 38, -1), p(46, 46, 52)), cat.coat);
    rig.put(Mesh::Geo, body.shape(p(0, -40, 2), p(44, 46, 50)), cat.coat);
    rig.put(
        Mesh::Geo,
        body.shape(p(0, 0, -14), p(34, 82, 26)),
        cat.cream,
    );
    // the rosettes: dark rings round deeper tawny middles
    for (along, round) in ROSETTES {
        let a = deg(round);
        let n = body.dir(fx::sin(a), 0, fx::cos(a));
        let at = body.at(mul(fx::sin(a), cm(23)), cm(along), mul(fx::cos(a), cm(25)));
        rig.put(
            Mesh::Hex,
            rod(at.sub(n.scale(cm(1))), at.add(n.scale(cm(2))), cm(11)),
            cat.spot,
        );
        rig.put(Mesh::Hex, rod(at, at.add(n.scale(cm(3))), cm(5)), cat.deep);
    }
    if cat.navy {
        navy_coat(&mut rig, &body);
    }
    jaguar_legs(&mut rig, j, &body, fwd, cat);
    jaguar_head(&mut rig, j, &body, fwd, cat);
    jaguar_tail(&mut rig, j, &body, cat);
}

fn jaguar_legs(rig: &mut Rig, j: &crate::jaguar::Jaguar, body: &Frame, fwd: V3, cat: &BigCat) {
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
            (cm(40), cm(62), cm(18))
        } else {
            (-cm(42), cm(66), cm(21))
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
        rig.bone(root, mid, w, cat.coat);
        rig.bone(mid, end, pct(w, 72), shade(cat.coat, 94));
        // a spot or two on the upper leg
        rig.ball(
            root.lerp(mid, ONE / 2).add(flat.x.scale(side * pct(w, 40))),
            cm(5),
            cat.spot,
        );
        // the paw
        let paw = Frame::upright(end, up, fwd);
        rig.put(
            Mesh::Bevel,
            paw.shape(v3(0, 0, cm(3)), p3(12, 7, 15)),
            cat.cream,
        );
    }
}

/// Centimetres as a Q16 vector.
fn p3(x: i32, y: i32, z: i32) -> V3 {
    v3(cm(x), cm(y), cm(z))
}

fn jaguar_head(rig: &mut Rig, j: &crate::jaguar::Jaguar, body: &Frame, fwd: V3, cat: &BigCat) {
    let up = V3::UP;
    // up and ahead of the shoulders; down to the ground, eating
    let high = body.at(0, cm(76), cm(14));
    let low = v3(0, cm(20), 0).add(fwd.scale(cm(98)));
    let at = high.lerp(low, j.head_down);
    let neck = body.at(0, cm(46), cm(6));
    rig.put(Mesh::Taper, rod(neck, at, cm(28)), cat.coat);
    let a = mul(j.head_down, deg(55));
    let head_up = up.scale(fx::cos(a)).add(fwd.scale(fx::sin(a)));
    let head_fwd = fwd.scale(fx::cos(a)).sub(up.scale(fx::sin(a)));
    let h = Frame::upright(at, head_up, head_fwd);
    rig.put(Mesh::Geo, h.shape(V3::ZERO, p3(30, 26, 32)), cat.coat);
    rig.put(
        Mesh::Bevel,
        h.shape(p3(0, -5, 16), p3(17, 12, 14)),
        cat.cream,
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
        cat.cream,
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
        rig.put(Mesh::Ico, h.shape(p3(s * 7, 4, 13), p3(6, 5, 5)), cat.eye);
        rig.put(Mesh::Octa, h.shape(p3(s * 7, 4, 15), p3(2, 4, 2)), BLACK);
        let ear = h.at(s * cm(10), cm(13), -cm(3));
        rig.put(
            Mesh::Hex,
            rod(ear, ear.add(h.z.scale(cm(3))), cm(10)),
            cat.spot,
        );
        let inner = ear.add(h.z.scale(cm(2)));
        rig.put(
            Mesh::Hex,
            rod(inner, inner.add(h.z.scale(cm(2))), cm(6)),
            cat.cream,
        );
        rig.ball(h.at(s * cm(6), cm(11), cm(6)), cm(3), cat.spot);
    }
    rig.ball(h.at(0, cm(13), cm(2)), cm(3), cat.spot);
    if cat.helmet {
        bubble_helmet(rig, &h, at.sub(neck).norm());
    }
    if cat.pith {
        // the head is 30 cm across; the dome sits over the brow, between
        // and over the ears, a little back from the eyes
        let top = Frame {
            o: h.at(0, cm(1), -cm(3)),
            ..h
        };
        pith_helmet(rig, cm(28), &top);
    }
    if cat.navy {
        bicorne(rig, &h);
    }
}

/// The navy officer's colours: a deep blue coat, a white front, gold
/// epaulettes and edging, a black bicorne.
const NAVY_BLUE: Rgb = rgb(24, 38, 96);
const NAVY_WHITE: Rgb = rgb(246, 244, 236);
const EPAULETTE: Rgb = rgb(240, 194, 56);
const BICORNE: Rgb = rgb(22, 22, 28);

/// The officer's coat over the jaguar's shoulders and back (`body`: y
/// along the spine, z up): the blue coat over the front two thirds with a
/// high collar, a white front on the chest with gold buttons either side,
/// gold epaulettes with fringes on the shoulders, and two coat tails over
/// the haunches. Plain: no nation's badge or colours.
fn navy_coat(rig: &mut Rig, body: &Frame) {
    rig.put(
        Mesh::Barrel,
        body.shape(p3(0, 16, 3), p3(58, 92, 62)),
        NAVY_BLUE,
    );
    let collar = body.at(0, cm(54), cm(6));
    rig.put(
        Mesh::Hex,
        rod(collar, collar.add(body.y.scale(cm(7))), cm(36)),
        NAVY_BLUE,
    );
    // the white front, facing ahead under the chin
    rig.put(
        Mesh::Bevel,
        body.shape(p3(0, 60, -6), p3(22, 6, 26)),
        NAVY_WHITE,
    );
    for s in SIDES {
        for z in [2, -12] {
            rig.ball(body.at(s * cm(14), cm(61), cm(z)), cm(4), EPAULETTE);
        }
        let top = body.at(s * cm(20), cm(40), cm(29));
        let out = body.dir(s * 50, 0, 100);
        rig.put(
            Mesh::Hex,
            rod(top, top.add(out.scale(cm(3))), cm(17)),
            EPAULETTE,
        );
        // the fringe hanging down the outside of the shoulder
        let side = body.at(s * cm(28), cm(40), cm(19));
        rig.put(
            Mesh::Hex,
            rod(side, side.add(body.x.scale(s * cm(3))), cm(13)),
            shade(EPAULETTE, 86),
        );
        // the coat tail over the haunch
        rig.blade(
            body.at(s * cm(12), -cm(26), cm(22)),
            body.at(s * cm(18), -cm(56), cm(8)),
            body.x,
            cm(18),
            cm(3),
            NAVY_BLUE,
        );
    }
}

/// A black bicorne worn side to side over the jaguar's head `h` (30 cm
/// across): a crown, a tall brim pointing out over each ear, gold trim up
/// its two top edges, gold tassels at its points and a plain gold cockade.
fn bicorne(rig: &mut Rig, h: &Frame) {
    rig.put(Mesh::Geo, h.shape(p3(0, 14, -2), p3(24, 14, 22)), BICORNE);
    let (l, r) = (h.at(-cm(30), cm(20), -cm(2)), h.at(cm(30), cm(20), -cm(2)));
    rig.blade(l, r, h.y, cm(24), cm(6), BICORNE);
    let apex = h.at(0, cm(32), cm(1));
    for tip in [l, r] {
        rig.put(Mesh::Hex, rod(tip, apex, cm(2)), EPAULETTE);
        rig.ball(tip, cm(5), EPAULETTE);
    }
    rig.ball(h.at(-cm(9), cm(24), cm(2)), cm(6), EPAULETTE);
}

/// A cartoon bubble helmet over the head `h`: a clear glass ball (a soft
/// pale-blue glow over the head, so the face shows through), a metal collar
/// round the neck (`neck` points from the shoulders to the head) and two
/// white glints on the glass.
fn bubble_helmet(rig: &mut Rig, h: &Frame, neck: V3) {
    let middle = h.at(0, cm(3), cm(5));
    let d = cm(66);
    let collar = middle.sub(neck.scale(cm(30)));
    rig.put(
        Mesh::Hex,
        rod(collar, collar.add(neck.scale(cm(6))), cm(46)),
        HELMET_RIM,
    );
    rig.put(
        Mesh::Hex,
        rod(collar.sub(neck.scale(cm(2))), collar, cm(40)),
        shade(HELMET_RIM, 70),
    );
    let glass = rig
        .base
        .then(&Affine::translate(middle))
        .then(&Affine::scale(v3(d, d, d)));
    // glass (`scene.frag`: a glow strength from 2 to 3): faint over the
    // face, a crisp bright rim at its outline (TRIAGE T78)
    rig.b
        .push_halo(Mesh::Sphere, &glass, HELMET_GLASS, GLASS + ONE * 3 / 5);
    // the classic cartoon glints: two hard white streaks curving round
    // the top of the glass, a long one and a short one
    for (from, to, w) in [(-40, 10, 3), (18, 30, 2)] {
        let n = 3;
        for k in 0..n {
            let a0 = deg(from + (to - from) * k / n);
            let a1 = deg(from + (to - from) * (k + 1) / n);
            let on = |a: i32| {
                let r = d / 2 - cm(1);
                h.at(
                    -mul(fx::sin(a), r) * 4 / 10,
                    mul(fx::cos(a), r) * 9 / 10,
                    cm(5),
                )
                .add(h.z.scale(mul(fx::sin(a + deg(40)), r) / 2))
                .add(h.y.scale(cm(3)))
            };
            rig.glow(
                Mesh::Cyl,
                rod(on(a0), on(a1), cm(w)),
                rgb(255, 255, 255),
                ONE,
            );
        }
    }
}

fn jaguar_tail(rig: &mut Rig, j: &crate::jaguar::Jaguar, body: &Frame, cat: &BigCat) {
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
        let w = pct(cm(8) - cm(4) * i / count, cat.tail);
        let c = if i >= 5 && i % 2 == 1 {
            cat.spot
        } else {
            cat.coat
        };
        rig.put(Mesh::Hex, rod(at, next, w), c);
        at = next;
    }
    rig.ball(at, cm(5), cat.spot);
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
pub fn lineup(b: &mut Batch, tick: u32, gear: Gear) {
    for (i, kind) in Kind::ALL.into_iter().enumerate() {
        let pose = Pose {
            gear,
            ..Pose::standing(lineup_spot(i), deg(28), tick)
        };
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
    showcase_in(b, kind, 0, tick);
}

/// As `showcase`, in palette `palette` (TRIAGE T49).
pub fn showcase_in(b: &mut Batch, kind: Kind, palette: u8, tick: u32) {
    let yaw = ((tick as i64 % 720) * TURN as i64 / 720) as i32;
    let pose = Pose::standing(V3::ZERO, yaw, tick);
    draw_in(b, kind, palette, &pose);
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
        // past the finish, mid-hop in its dance (TRIAGE T53)
        let dance = dance_at(rest, 100);
        vec![rest, run, swing, fly, fall, dizzy, dizzy_air, dance]
    }

    #[test]
    fn the_jaguar_is_a_spotted_cat_at_the_racers_scale() {
        // TRIAGE T37: four legs, a tail, rosettes; at the scale the racers
        // are drawn (T73: three times life size), so it reads next to them
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
            jaguar(&mut b, &j, 200, &JAGUAR);
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
            jaguar(&mut b, j, 200, &JAGUAR);
            shapes(b)
                .iter()
                .map(|x| x.at.sub(j.feet).dot(j.fwd()))
                .max()
                .unwrap()
        };
        assert!(reach(&leap) > reach(&base));
    }

    #[test]
    fn on_the_ski_slope_the_big_cat_is_a_snow_leopard() {
        // the person, 2026-10-13: a snow leopard on the snow, the jaguar
        // everywhere else
        use crate::map::Map;
        assert_eq!(BigCat::on(Map::Ski), SNOW_LEOPARD);
        assert_eq!(BigCat::on(Map::Jungle), JAGUAR);
        assert_eq!(BigCat::on(Map::City), JAGUAR);
        let j = crate::jaguar::Jaguar {
            feet: V3::ZERO,
            yaw: 0,
            gait: 0,
            stride: ONE,
            leap: 0,
            head_down: 0,
            jaw: 0,
        };
        let draw = |cat: &BigCat| {
            let mut b = Batch::new();
            jaguar(&mut b, &j, 200, cat);
            shapes(b)
        };
        let (leopard, jag) = (draw(&SNOW_LEOPARD), draw(&JAGUAR));
        assert!(
            leopard.iter().all(|x| x.colour != JAG_COAT),
            "no tawny on the snow"
        );
        assert!(
            leopard
                .iter()
                .filter(|x| x.colour == SNOW_LEOPARD.spot)
                .count()
                >= 20
        );
        // the same cat, in another coat
        assert_eq!(leopard.len(), jag.len());
    }

    #[test]
    fn only_the_space_jaguar_wears_a_bubble_helmet() {
        // the person, 2026-10-13: Sector Ape's jaguar wears a cartoon
        // bubble space helmet and no spacesuit; the others are unchanged
        use crate::map::Map;
        for map in Map::ALL {
            assert_eq!(BigCat::on(map).helmet, map == Map::Sector, "{map:?}");
        }
        const { assert!(!JAGUAR.helmet && !SNOW_LEOPARD.helmet && SPACE_JAGUAR.helmet) };
        let j = crate::jaguar::Jaguar {
            feet: V3::ZERO,
            yaw: 0,
            gait: 0,
            stride: ONE,
            leap: 0,
            head_down: 0,
            jaw: 0,
        };
        let draw = |cat: &BigCat| {
            let mut b = Batch::new();
            jaguar(&mut b, &j, 100, cat);
            let glass = b.take_halos().count();
            (shapes(b), glass)
        };
        let ((space, glass), (plain, none)) = (draw(&SPACE_JAGUAR), draw(&JAGUAR));
        // the glass dome over the head, the collar and two glints; nothing
        // else changes
        assert_eq!((glass, none), (1, 0));
        // (two collar rings and two glints of three strokes each)
        assert_eq!(space.len(), plain.len() + 8);
        assert!(space.iter().any(|x| x.colour == HELMET_RIM));
        assert!(plain.iter().all(|x| x.colour != HELMET_RIM));
        // no spacesuit: the same coat
        let coat = |s: &[Shape]| s.iter().filter(|x| x.colour == JAG_COAT).count();
        assert_eq!(coat(&space), coat(&plain));
    }

    #[test]
    fn only_the_sunset_ruins_jaguar_wears_a_pith_helmet() {
        // the person, 2026-10-14: "give the jaguar a pith helmet too"
        use crate::map::Map;
        for map in Map::ALL {
            assert_eq!(BigCat::on(map).pith, map == Map::Ruins, "{map:?}");
        }
        for head_down in [0, ONE] {
            let j = crate::jaguar::Jaguar {
                feet: V3::ZERO,
                yaw: 0,
                gait: 0,
                stride: ONE,
                leap: 0,
                head_down,
                jaw: ONE / 2,
            };
            let draw = |cat: &BigCat| {
                let mut b = Batch::new();
                jaguar(&mut b, &j, 100, cat);
                shapes(b)
            };
            let ruins = draw(&RUINS_JAGUAR);
            let has = |s: &[Shape], c: Rgb| s.iter().any(|x| x.colour == c);
            assert!(
                has(&ruins, PITH) && has(&ruins, PITH_BAND),
                "no pith helmet"
            );
            for other in [JAGUAR, SPACE_JAGUAR, SNOW_LEOPARD] {
                let s = draw(&other);
                assert!(!has(&s, PITH) && !has(&s, PITH_BAND), "{other:?}");
            }
            // dome, band, brim, button; the same cat underneath
            let plain = draw(&JAGUAR);
            assert_eq!(ruins.len(), plain.len() + 4);
            let coat = |s: &[Shape]| s.iter().filter(|x| x.colour == JAG_COAT).count();
            assert_eq!(coat(&ruins), coat(&plain));
            // on the head: the dome sits over the eyes, not off in space
            let eyes: Vec<V3> = plain
                .iter()
                .filter(|x| x.colour == JAG_EYE)
                .map(|x| x.at)
                .collect();
            let brow = eyes[0].lerp(eyes[1], ONE / 2);
            let dome = ruins
                .iter()
                .find(|x| x.colour == PITH && x.mesh == Mesh::Geo as u32)
                .unwrap()
                .at;
            assert!(
                dome.dist(brow) < cm(22),
                "dome {} from the brow",
                dome.dist(brow)
            );
            let up = dome.sub(brow).dot(V3::UP);
            assert!(head_down > 0 || up > cm(4), "dome only {up} above the eyes");
        }
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
                pose_skel(kind, &l, &pose).feet
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
        lineup(&mut b, 30, Gear::None);
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

    /// The poses on skis.
    fn ski_poses() -> Vec<Pose> {
        poses()
            .into_iter()
            .map(|p| Pose {
                gear: Gear::Ski,
                ..p
            })
            .collect()
    }

    /// A pose's ski boards: the long boxes in the kind's gear colour, each
    /// (its middle, its long axis).
    fn boards(kind: Kind, pose: &Pose) -> Vec<(V3, V3)> {
        let (shell, _, _) = gear_colours(kind);
        drawn(kind, pose)
            .into_iter()
            .filter(|s| s.mesh == Mesh::Cube as u32 && s.colour == shell)
            .map(|s| (s.at, s.axes[2]))
            .filter(|(_, along)| along.len() > cm(40))
            .collect()
    }

    #[test]
    fn on_skis_every_kind_wears_skis_a_helmet_and_goggles_in_every_pose() {
        // the Ski Slope (2026-10-13)
        for kind in Kind::ALL {
            let (shell, _, lens) = gear_colours(kind);
            for pose in ski_poses() {
                let s = drawn(kind, &pose);
                let bare = drawn(
                    kind,
                    &Pose {
                        gear: Gear::None,
                        ..pose
                    },
                );
                assert!(
                    s.len() > bare.len() + 15,
                    "{kind:?} {} vs {}",
                    s.len(),
                    bare.len()
                );
                assert!((50..=280).contains(&s.len()), "{kind:?} {} shapes", s.len());
                // the helmet's shell over the head, and the mirrored lens
                assert!(s
                    .iter()
                    .any(|x| x.mesh == Mesh::Geo as u32 && x.colour == shell));
                assert!(s.iter().any(|x| x.colour == lens), "{kind:?}: no goggles");
                // two long boards, pointing the way the ape faces
                let b = boards(kind, &pose);
                assert_eq!(b.len(), 2, "{kind:?} {pose:?}");
                let face = facing(pose.yaw, 0);
                for (_, along) in &b {
                    let d = along.norm().dot(face);
                    assert!(d > ONE * 4 / 5, "{kind:?} a ski {d} off the facing");
                }
                // sensible sizes, near the ape
                for x in &s {
                    for a in x.axes {
                        assert!(a.len() >= ONE / 500 && a.len() <= int(2));
                    }
                    assert!(x.at.dist(pose.hands) <= int(3));
                }
            }
        }
    }

    #[test]
    fn skis_go_under_the_feet_and_the_ape_stands_on_them() {
        for kind in Kind::ALL {
            let pose = Pose {
                gear: Gear::Ski,
                ..at_rest()
            };
            let feet = pose.hands.y - BODY_DROP;
            let s = drawn(kind, &pose);
            let low = s.iter().map(|x| x.at.y).min().unwrap();
            // the boards rest on the ground, not in it, and nothing is lower
            for (at, _) in boards(kind, &pose) {
                assert!(
                    at.y >= feet && at.y <= feet + cm(8),
                    "{kind:?}: a ski at {}",
                    at.y - feet
                );
            }
            assert!(
                low >= feet,
                "{kind:?}: sunk {} cm",
                (feet - low) * 100 / ONE
            );
            // the two skis side by side, a hip's width apart
            let b = boards(kind, &pose);
            let apart = b[0].0.dist(b[1].0);
            assert!((cm(5)..=cm(60)).contains(&apart), "{kind:?}: {apart}");
        }
    }

    #[test]
    fn in_the_air_the_skis_splay_into_a_v() {
        for kind in Kind::ALL {
            let fly = ski_poses()[3];
            let b = boards(kind, &fly);
            let d = b[0].1.norm().dot(b[1].1.norm());
            assert!(d < ONE - ONE / 100, "{kind:?}: parallel in the air");
            let stand = boards(kind, &ski_poses()[0]);
            let d = stand[0].1.norm().dot(stand[1].1.norm());
            assert!(d > ONE - ONE / 500, "{kind:?}: splayed on the ground");
        }
    }

    #[test]
    fn without_gear_nothing_is_worn() {
        for kind in Kind::ALL {
            for pose in poses() {
                assert!(boards(kind, &pose).is_empty(), "{kind:?}");
            }
        }
        assert_eq!(Gear::on(crate::map::Map::Ski), Gear::Ski);
        assert_eq!(Gear::on(crate::map::Map::Jungle), Gear::None);
        assert_eq!(Gear::on(crate::map::Map::City), Gear::None);
        // the Sector Ape costume, on its map only
        for map in crate::map::Map::ALL {
            assert_eq!(
                Gear::on(map) == Gear::Space,
                map == crate::map::Map::Sector,
                "{map:?}"
            );
        }
    }

    #[test]
    fn on_sunset_ruins_every_kind_wears_the_explorer_outfit_in_every_pose() {
        // a pith helmet, a khaki jacket with a compass, a canteen at the
        // hip and tiny round spectacles (briefs/sunset-ruins.md)
        assert_eq!(Gear::from_key("explorer"), Some(Gear::Explorer));
        for map in crate::map::Map::ALL {
            assert_eq!(
                Gear::on(map) == Gear::Explorer,
                map == crate::map::Map::Ruins,
                "{map:?}"
            );
        }
        for kind in Kind::ALL {
            for p in poses() {
                let pose = Pose {
                    gear: Gear::Explorer,
                    ..p
                };
                let s = drawn(kind, &pose);
                let bare = drawn(kind, &p);
                assert!(
                    s.len() > bare.len() + 25,
                    "{kind:?} {} vs {}",
                    s.len(),
                    bare.len()
                );
                assert!((50..=300).contains(&s.len()), "{kind:?} {} shapes", s.len());
                let has = |c: Rgb| s.iter().any(|x| x.colour == c);
                assert!(has(PITH) && has(PITH_BAND), "{kind:?}: no pith helmet");
                assert!(has(KHAKI) && has(KHAKI_DARK), "{kind:?}: no jacket");
                assert!(
                    has(BRASS) && has(DIAL) && has(NEEDLE),
                    "{kind:?}: no compass"
                );
                assert!(has(CANTEEN) && has(LEATHER), "{kind:?}: no canteen");
                assert!(has(WIRE) && has(SPECS_GLASS), "{kind:?}: no spectacles");
                // none of the other maps' gear
                assert!(!has(JACKET) && !has(LENS) && !has(GUNMETAL), "{kind:?}");
                // two round lenses, each just ahead of an eye
                let face = facing(dance_turn(kind, &pose).yaw, 0);
                let mut whites: Vec<V3> = s
                    .iter()
                    .filter(|x| {
                        x.mesh == Mesh::Ico as u32
                            && (x.colour == EYE_WHITE || x.colour == LEMUR_EYE)
                    })
                    .map(|x| x.at)
                    .collect();
                whites.dedup();
                let lenses: Vec<V3> = s
                    .iter()
                    .filter(|x| x.colour == SPECS_GLASS)
                    .map(|x| x.at)
                    .collect();
                assert_eq!((whites.len(), lenses.len()), (2, 2), "{kind:?}");
                for e in &whites {
                    let ahead = lenses
                        .iter()
                        .filter(|l| l.dist(*e) < cm(12))
                        .map(|l| l.sub(*e).dot(face))
                        .max()
                        .unwrap_or(i32::MIN);
                    assert!(ahead > cm(1), "{kind:?} {pose:?}: lens {ahead} ahead");
                }
                for x in &s {
                    for a in x.axes {
                        assert!(
                            a.len() >= ONE / 500 && a.len() <= int(2),
                            "{kind:?} mesh {} colour {:?} axis {}",
                            x.mesh,
                            x.colour,
                            a.len()
                        );
                    }
                    assert!(x.at.dist(pose.hands) <= int(3));
                }
            }
        }
    }

    /// Triangles drawn for these shapes.
    fn triangles(s: &[Shape]) -> usize {
        s.iter()
            .map(|x| {
                let m = Mesh::ALL.iter().find(|m| **m as u32 == x.mesh).unwrap();
                crate::mesh::vertices(*m).len() / crate::mesh::VERTEX_VALUES / 3
            })
            .sum()
    }

    /// The colours that mark each piece of a kind's pirate kit.
    fn kit_colours(kind: Kind) -> Vec<Rgb> {
        let k = pirate_kit(kind);
        let mut c = Vec::new();
        match k.hat {
            PirateHat::None => {}
            PirateHat::Tricorn(captain) => {
                c.extend([TRICORN, TRIM_GOLD]);
                if captain {
                    c.extend([PLUME, BONE]);
                }
            }
            PirateHat::Bandana(b, dots) => {
                c.push(b);
                c.extend(dots);
            }
        }
        c.extend(k.patch.map(|_| PATCH));
        c.extend(k.earring.map(|_| HOOP));
        c.extend(match k.top {
            PirateTop::None => vec![],
            PirateTop::CaptainsCoat => vec![CAPTAIN_RED, SHIRT_WHITE, BELT],
            PirateTop::Striped => vec![SHIRT_WHITE, STRIPE_NAVY],
            PirateTop::Shirt => vec![SHIRT_CREAM],
            PirateTop::Waistcoat => vec![WAISTCOAT, BELT],
        });
        c.extend(k.sash);
        if let Some(b) = k.bandolier {
            c.extend([b, CARTRIDGE]);
        }
        if k.cutlass {
            c.extend([SCABBARD, HILT, GRIP]);
        }
        c.extend(k.peg.map(|_| PEG_WOOD));
        c.extend(k.hook.map(|_| HOOK_STEEL));
        if let Some((_, p)) = k.parrot {
            c.extend(p);
            c.push(BEAK);
        }
        c
    }

    #[test]
    fn on_banana_bay_every_kind_wears_its_own_pirate_mix_in_every_pose() {
        // the person, 2026-10-15: "an assortment of pirate-related costumes
        // ... cutlasses in the scabbard, eyepatch, bandana, tricorn, pegleg,
        // bandolier, [parrot] on the shoulder" (briefs/banana-bay-battle.md)
        assert_eq!(Gear::from_key("pirate"), Some(Gear::Pirate));
        for map in crate::map::Map::ALL {
            assert_eq!(Gear::on(map) == Gear::Pirate, map.key() == "bay", "{map:?}");
        }
        // every piece the brief names is worn by some kind
        let all: Vec<PirateKit> = Kind::ALL.iter().map(|k| pirate_kit(*k)).collect();
        assert!(all.iter().any(|k| matches!(k.hat, PirateHat::Tricorn(_))));
        assert!(all.iter().any(|k| matches!(k.hat, PirateHat::Bandana(..))));
        assert!(all.iter().any(|k| k.patch.is_some()));
        assert!(all.iter().any(|k| k.cutlass));
        assert!(all.iter().any(|k| k.bandolier.is_some()));
        assert!(all.iter().any(|k| k.peg.is_some()));
        assert!(all.iter().any(|k| k.parrot.is_some()));
        // each kind its own mix, so they read apart
        for (i, a) in Kind::ALL.iter().enumerate() {
            for b in &Kind::ALL[i + 1..] {
                assert_ne!(kit_colours(*a), kit_colours(*b), "{a:?} {b:?}");
            }
        }
        for kind in Kind::ALL {
            let kit = pirate_kit(kind);
            for p in poses() {
                let pose = Pose {
                    gear: Gear::Pirate,
                    ..p
                };
                let s = drawn(kind, &pose);
                let bare = drawn(kind, &p);
                assert!(
                    s.len() > bare.len() + 8,
                    "{kind:?} {} vs {}",
                    s.len(),
                    bare.len()
                );
                assert!((50..=300).contains(&s.len()), "{kind:?} {} shapes", s.len());
                let has = |c: Rgb| s.iter().any(|x| x.colour == c);
                for c in kit_colours(kind) {
                    assert!(has(c), "{kind:?} {pose:?}: no {c:?}");
                }
                // none of the other maps' gear
                assert!(
                    !has(JACKET) && !has(LENS) && !has(KHAKI) && !has(PITH),
                    "{kind:?}"
                );
                // a peg leg: one foot fewer; a hook: one hand fewer
                let count = |c: Rgb| s.iter().filter(|x| x.colour == c).count();
                let bare_count = |c: Rgb| bare.iter().filter(|x| x.colour == c).count();
                let skin = look(kind).skin;
                let (feet, hands) = (kit.peg.is_some(), kit.hook.is_some());
                if feet || hands {
                    assert!(count(skin) < bare_count(skin), "{kind:?}");
                }
                for x in &s {
                    for a in x.axes {
                        assert!(
                            a.len() >= ONE / 500 && a.len() <= int(2),
                            "{kind:?} mesh {} colour {:?} axis {}",
                            x.mesh,
                            x.colour,
                            a.len()
                        );
                    }
                    assert!(x.at.dist(pose.hands) <= int(3));
                }
            }
        }
    }

    #[test]
    fn the_pirate_outfits_cost_about_what_the_explorer_outfit_does() {
        // SECOND-ORDER BG2: the triangles a pirate adds over the bare ape,
        // against the explorer outfit's, kind by kind
        let mut lines = Vec::new();
        let (mut pirate_all, mut explorer_all) = (0, 0);
        for kind in Kind::ALL {
            let p = at_rest();
            let tris = |g: Gear| triangles(&drawn(kind, &Pose { gear: g, ..p }));
            let bare = tris(Gear::None);
            let (pirate, explorer) = (tris(Gear::Pirate) - bare, tris(Gear::Explorer) - bare);
            lines.push(format!(
                "{kind:?}: bare {bare}, explorer +{explorer}, pirate +{pirate}"
            ));
            pirate_all += pirate;
            explorer_all += explorer;
            assert!(pirate <= explorer * 3 / 2, "{}", lines.join("; "));
        }
        assert!(
            pirate_all <= explorer_all * 5 / 4,
            "pirate {pirate_all} vs explorer {explorer_all}: {}",
            lines.join("; ")
        );
    }

    #[test]
    fn only_the_banana_bay_jaguar_dresses_as_a_navy_officer() {
        // the person, 2026-10-15: a Royal Navy-style jaguar (bicorne, blue
        // coat, gold epaulettes), no real nation's insignia
        use crate::map::Map;
        for map in Map::ALL {
            assert_eq!(BigCat::on(map).navy, map.key() == "bay", "{map:?}");
        }
        for head_down in [0, ONE] {
            let j = crate::jaguar::Jaguar {
                feet: V3::ZERO,
                yaw: 0,
                gait: 0,
                stride: ONE,
                leap: 0,
                head_down,
                jaw: ONE / 2,
            };
            let draw = |cat: &BigCat| {
                let mut b = Batch::new();
                jaguar(&mut b, &j, 100, cat);
                shapes(b)
            };
            let navy = draw(&NAVY_JAGUAR);
            let has = |s: &[Shape], c: Rgb| s.iter().any(|x| x.colour == c);
            for c in [NAVY_BLUE, NAVY_WHITE, EPAULETTE, BICORNE] {
                assert!(has(&navy, c), "no {c:?}");
            }
            for other in [JAGUAR, SPACE_JAGUAR, RUINS_JAGUAR, SNOW_LEOPARD] {
                let s = draw(&other);
                assert!(!has(&s, NAVY_BLUE) && !has(&s, BICORNE), "{other:?}");
            }
            // the same cat underneath
            let plain = draw(&JAGUAR);
            let coat = |s: &[Shape]| s.iter().filter(|x| x.colour == JAG_COAT).count();
            assert_eq!(coat(&navy), coat(&plain));
            // the bicorne's crown sits on the head, over the eyes
            let eyes: Vec<V3> = plain
                .iter()
                .filter(|x| x.colour == JAG_EYE)
                .map(|x| x.at)
                .collect();
            let brow = eyes[0].lerp(eyes[1], ONE / 2);
            let crown = navy
                .iter()
                .find(|x| x.colour == BICORNE && x.mesh == Mesh::Geo as u32)
                .unwrap()
                .at;
            assert!(
                crown.dist(brow) < cm(22),
                "crown {} from the brow",
                crown.dist(brow)
            );
            // the gold (epaulettes, collar, buttons) up off the ground
            for x in navy.iter().filter(|x| x.colour == EPAULETTE) {
                assert!(x.at.y > cm(15), "gold at {}", x.at.y);
            }
        }
    }

    /// The poses in the Sector Ape costume.
    fn space_poses() -> Vec<Pose> {
        poses()
            .into_iter()
            .map(|p| Pose {
                gear: Gear::Space,
                ..p
            })
            .collect()
    }

    #[test]
    fn in_space_every_kind_wears_a_jacket_a_metal_arm_and_sunglasses_in_every_pose() {
        // Sector Ape (TRIAGE T75), previewed with a tile's `gear=space`
        for kind in Kind::ALL {
            let team = gear_colours(kind).0;
            for pose in space_poses() {
                let s = drawn(kind, &pose);
                let bare = drawn(
                    kind,
                    &Pose {
                        gear: Gear::None,
                        ..pose
                    },
                );
                assert!(
                    s.len() > bare.len() + 20,
                    "{kind:?} {} vs {}",
                    s.len(),
                    bare.len()
                );
                assert!((50..=300).contains(&s.len()), "{kind:?} {} shapes", s.len());
                let has = |c: Rgb| s.iter().any(|x| x.colour == c);
                assert!(has(JACKET) && has(OLIVE), "{kind:?}: no jacket");
                assert!(has(team), "{kind:?}: no scarf or patches");
                assert!(has(LENS) && has(FRAME_GOLD), "{kind:?}: no sunglasses");
                assert!(has(GUNMETAL) && has(LED), "{kind:?}: no metal arm");
                // one chrome hand: a palm and four fingers
                let bevels = |c: Rgb| {
                    s.iter()
                        .filter(|x| x.mesh == Mesh::Bevel as u32 && x.colour == c)
                        .count()
                };
                assert_eq!(
                    (bevels(CHROME), bevels(shade(CHROME, 92))),
                    (1, 4),
                    "{kind:?} {pose:?}"
                );
                for x in &s {
                    for a in x.axes {
                        assert!(
                            a.len() >= ONE / 500 && a.len() <= int(2),
                            "{kind:?} mesh {} colour {:?} axis {}",
                            x.mesh,
                            x.colour,
                            a.len()
                        );
                    }
                    assert!(x.at.dist(pose.hands) <= int(3));
                }
            }
        }
    }

    #[test]
    fn the_sunglasses_sit_in_front_of_the_eyes() {
        for kind in Kind::ALL {
            for pose in space_poses() {
                let face = facing(dance_turn(kind, &pose).yaw, 0);
                let s = drawn(kind, &pose);
                let eye = |c: Rgb| {
                    s.iter()
                        .filter(|x| x.mesh == Mesh::Ico as u32 && x.colour == c)
                        .map(|x| x.at)
                        .collect::<Vec<_>>()
                };
                let mut whites = eye(EYE_WHITE);
                whites.extend(eye(LEMUR_EYE));
                let lenses: Vec<V3> = s
                    .iter()
                    .filter(|x| x.mesh == Mesh::Geo as u32 && x.colour == LENS)
                    .map(|x| x.at)
                    .collect();
                assert_eq!((whites.len(), lenses.len()), (2, 4), "{kind:?}");
                // each eye has a lens just ahead of it
                for e in &whites {
                    let ahead = lenses
                        .iter()
                        .filter(|l| l.dist(*e) < cm(12))
                        .map(|l| l.sub(*e).dot(face))
                        .max()
                        .unwrap_or(i32::MIN);
                    assert!(ahead > cm(1), "{kind:?} {pose:?}: lens {ahead} ahead");
                }
            }
        }
    }

    #[test]
    fn in_space_the_fur_and_markings_still_show() {
        for kind in Kind::ALL {
            for palette in 0..PALETTES {
                let l = look_in(kind, palette);
                let pose = Pose {
                    gear: Gear::Space,
                    ..at_rest()
                };
                let mut b = Batch::new();
                draw_in(&mut b, kind, palette, &pose);
                let s = shapes(b);
                let count = |c: Rgb| s.iter().filter(|x| x.colour == c).count();
                // the bare arm, the legs and the head keep the kind's colours
                assert!(count(l.limb) >= 4, "{kind:?} {palette}: limbs hidden");
                assert!(count(l.fur) >= 2, "{kind:?} {palette}: fur hidden");
                assert!(count(l.skin) >= 3, "{kind:?} {palette}: no bare hand");
            }
        }
        // the gorilla's silver saddle and the lemur's ringed tail, as bare
        let pose = Pose {
            gear: Gear::Space,
            ..at_rest()
        };
        let silver = rgb(158, 158, 166);
        assert!(drawn(Kind::Gorilla, &pose)
            .iter()
            .any(|x| x.colour == silver));
        let rings = |p: &Pose| {
            drawn(Kind::Lemur, p)
                .iter()
                .filter(|x| x.colour == rgb(242, 242, 240))
                .count()
        };
        assert_eq!(rings(&pose), rings(&at_rest()));
        assert!(rings(&pose) > 0);
    }

    #[test]
    fn the_metal_arm_alternates_through_the_kinds() {
        let sides: Vec<i32> = Kind::ALL.into_iter().map(metal_side).collect();
        assert!(sides.contains(&1) && sides.contains(&-1), "{sides:?}");
        // and it is the arm on that side
        for kind in Kind::ALL {
            let pose = Pose {
                gear: Gear::Space,
                ..at_rest()
            };
            let face = facing(pose.yaw, 0);
            let side = V3::UP.cross(face).norm();
            let s = drawn(kind, &pose);
            let led = s.iter().find(|x| x.colour == LED).unwrap().at;
            let off = led.sub(pose.hands).dot(side);
            assert!(off * metal_side(kind) > cm(5), "{kind:?}: the LED at {off}");
        }
    }

    /// `pose` dancing, `beat` ticks into its dance.
    fn dance_at(pose: Pose, beat: u32) -> Pose {
        Pose {
            dance: Some(Dance { beat, weight: ONE }),
            ..pose
        }
    }

    /// A kind's dance over eight beats, a sample every eighth of a beat:
    /// the hands and the hips from the feet, in tenths of its arm.
    fn dance_path(kind: Kind) -> Vec<[i32; 7]> {
        let l = look(kind);
        let m = dims(&l);
        (0..64)
            .map(|i| {
                let pose = dance_turn(kind, &dance_at(at_rest(), i * BEAT / 8));
                let s = pose_skel(kind, &l, &pose);
                let t = |v: i32| v * 10 / m.arm;
                let [a, b] = s.hands;
                let o = s.body.o;
                [
                    t(a.x - o.x),
                    t(a.y - o.y),
                    t(a.z - o.z),
                    t(b.x - o.x),
                    t(b.y - o.y),
                    t(b.z - o.z),
                    t(o.y),
                ]
            })
            .collect()
    }

    #[test]
    fn every_kind_dances_its_own_dance_and_the_dance_moves() {
        // the person (2026-10-12): a unique dance for each of the six
        let paths: Vec<_> = Kind::ALL.into_iter().map(dance_path).collect();
        for (i, a) in paths.iter().enumerate() {
            let k = Kind::ALL[i];
            // the hands go up and down by a good part of an arm
            for h in [1, 4] {
                let ys: Vec<i32> = a.iter().map(|p| p[h]).collect();
                let range = ys.iter().max().unwrap() - ys.iter().min().unwrap();
                assert!(range >= 3, "{k:?}: a hand moves {range} tenths of an arm");
            }
            for (j, b) in paths.iter().enumerate().skip(i + 1) {
                let apart: i32 = a
                    .iter()
                    .zip(b)
                    .map(|(p, q)| p.iter().zip(q).map(|(x, y)| (x - y).abs()).sum::<i32>())
                    .sum();
                assert!(
                    apart > 200,
                    "{k:?} and {:?} dance alike: {apart}",
                    Kind::ALL[j]
                );
            }
        }
    }

    #[test]
    fn a_dancer_lands_on_every_beat_and_never_sinks() {
        for kind in Kind::ALL {
            let rest = at_rest();
            let ground = rest.hands.y - BODY_DROP;
            for t in (0..BEAT * 8).step_by(5) {
                let s = drawn(kind, &dance_at(rest, t));
                let low = s.iter().map(|x| x.at.y).min().unwrap();
                let lowest = s.iter().min_by_key(|x| x.at.y).unwrap();
                assert!(
                    low >= ground - cm(1),
                    "{kind:?} at {t}: in the ground {} cm, mesh {} colour {:?}",
                    (low - ground) * 100 / ONE,
                    lowest.mesh,
                    lowest.colour
                );
                if t % BEAT == 0 {
                    assert!(
                        low <= ground + cm(10),
                        "{kind:?} at {t}: in the air on the beat"
                    );
                }
                for x in &s {
                    assert!(x.at.dist(rest.hands) <= int(3), "{kind:?} at {t}");
                }
            }
        }
    }

    #[test]
    fn on_skis_a_dancer_keeps_its_skis_under_its_feet_turning_with_it() {
        for kind in Kind::ALL {
            for t in [0, 20, BEAT * 7 + BEAT / 4, BEAT * 7 + BEAT / 2] {
                let pose = dance_at(
                    Pose {
                        gear: Gear::Ski,
                        ..at_rest()
                    },
                    t,
                );
                let b = boards(kind, &pose);
                assert_eq!(b.len(), 2, "{kind:?} at {t}");
                let face = facing(dance_turn(kind, &pose).yaw, 0);
                for (_, along) in &b {
                    let d = along.norm().dot(face);
                    assert!(
                        d > ONE * 9 / 10,
                        "{kind:?} at {t}: a ski {d} off the facing"
                    );
                }
            }
        }
        // the gibbon's spin turns it, skis and all
        let mid = dance_at(at_rest(), BEAT * 7 + BEAT / 2);
        let spun = dance_turn(Kind::Gibbon, &mid).yaw;
        assert!(fx::wrap(spun - mid.yaw).abs() > deg(150));
    }

    #[test]
    fn slowing_into_the_dance_blends_from_the_run() {
        let l = look(Kind::Chimpanzee);
        let run = pose_skel(Kind::Chimpanzee, &l, &at_rest());
        let full = pose_skel(Kind::Chimpanzee, &l, &dance_at(at_rest(), 30));
        let half = Pose {
            dance: Some(Dance {
                beat: 30,
                weight: ONE / 2,
            }),
            ..at_rest()
        };
        let mid = pose_skel(Kind::Chimpanzee, &l, &half);
        for i in 0..2 {
            let want = run.hands[i].lerp(full.hands[i], ONE / 2);
            assert!(mid.hands[i].dist(want) < cm(1));
            assert!(mid.hands[i].dist(run.hands[i]) > cm(10));
        }
    }
}
