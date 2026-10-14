//! Sector Ape's models (2026-10-13, `briefs/space-map.md`): stand-alone
//! builders for the space map's set pieces, in the faceted low-poly style of
//! the City's `vehicles.rs`. No map uses them yet; a later session places
//! them. Each is built in its own metres, +z forward and +y up, about its
//! own middle, then placed by a frame (`pose`) the caller passes, so one
//! builder serves a far landmark and a near vine anchor alike. Nothing here
//! keeps state: engines, walking, dogfights and the black hole's swirl come
//! from the tick alone.
//!
//! The light is the map's (`look`): a blazing orange-white sun far below
//! the track, so the models are dark on top and warm underneath; their
//! engines, lasers, eyes and windows glow (a glow of 2 counts as sky in
//! `scene.frag`, so the fog does not hide them far away).

use crate::fx::{self, cm, deg, int, mul, ratio, sin, HALF, ONE, QUARTER, TURN};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{rgb, rod, shade, Batch, Rgb};
use crate::scene::Look;

// ---------------------------------------------------------------------------
// The palette: deep black-blue space, the sun's under-light, neon glows.

const HULL: Rgb = rgb(214, 220, 228);
const HULL_GREY: Rgb = rgb(150, 158, 172);
const HULL_DARK: Rgb = rgb(62, 68, 82);
const ACCENT: Rgb = rgb(36, 104, 232);
const WINDOW: Rgb = rgb(150, 220, 255);
const ENGINE: Rgb = rgb(110, 200, 255);
const ENGINE_CORE: Rgb = rgb(235, 250, 255);
const BEACON: Rgb = rgb(255, 60, 50);

const SCORCH: Rgb = rgb(26, 22, 24);
const INTERIOR: Rgb = rgb(46, 46, 54);
const RIB: Rgb = rgb(96, 100, 112);
const EMBER: Rgb = rgb(255, 120, 30);
const ALARM: Rgb = rgb(255, 40, 40);

const BANANA: Rgb = rgb(255, 214, 56);
const BANANA_TIP: Rgb = rgb(90, 64, 30);
const LASER_GOOD: Rgb = rgb(80, 255, 120);
const LASER_BAD: Rgb = rgb(255, 50, 70);
const BLAST: Rgb = rgb(255, 190, 90);

const APE_WHITE: Rgb = rgb(232, 234, 238);
const APE_YELLOW: Rgb = rgb(255, 200, 40);
const APE_FUR: Rgb = rgb(132, 86, 52);
const GLASS: Rgb = rgb(60, 150, 255);
const ENEMY_HULL: Rgb = rgb(44, 30, 58);
const ENEMY_CLAW: Rgb = rgb(150, 24, 44);
const ENEMY_GLOW: Rgb = rgb(255, 60, 220);

const ROCK: Rgb = rgb(92, 84, 98);
const CRATER: Rgb = rgb(52, 46, 58);
const CRYSTAL: Rgb = rgb(176, 96, 255);
const HAZARD: Rgb = rgb(232, 172, 30);

/// Shown full colour and unfogged, as the sky is (`scene.frag`).
const SKY_GLOW: i32 = ONE * 2;
/// The black hole's bananas: half lit (so their facets show), half glowing.
const BANANA_GLOW: i32 = ONE / 2;
/// A far backdrop: lit, unfogged, hazed a little toward the fog.
const FAR_GLOW: i32 = ONE * 5 / 2 + ONE / 8;

/// How Sector Ape is lit (`sectorscape::look`): a black-blue sky and a
/// blazing sun far below, so the light comes from underneath (the light's
/// direction points toward the sun), the ground's bounce, which colours
/// down-facing sides, is the sun's own orange, and an edge light outlines
/// the tops.
pub fn look() -> Look {
    crate::sectorscape::look()
}

/// The frame of a model at `at`, turned `yaw` about +y, its nose raised by
/// `pitch` and rolled by `roll` (to its right), `scale` times as built (Q16).
pub fn pose(at: V3, yaw: i32, pitch: i32, roll: i32, scale: i32) -> Affine {
    Affine::translate(at)
        .then(&Affine::rot_y(yaw))
        .then(&Affine::rot_x(-pitch))
        .then(&Affine::rot_z(roll))
        .then(&Affine::scale(v3(scale, scale, scale)))
}

/// A slow tumble for a drifting piece (`k` picks its own), from the tick:
/// compose it into the piece's frame. A piece carrying a vine should not
/// tumble (the vine's anchor is the map's).
pub fn tumble(tick: u32, k: u32) -> Affine {
    let t = tick as i32;
    Affine::rot_y(t.wrapping_mul(9 + (k % 5) as i32 * 4))
        .then(&Affine::rot_x(t.wrapping_mul(6 + (k % 3) as i32 * 5)))
}

/// Metres.
fn m3(x: i32, y: i32, z: i32) -> V3 {
    v3(int(x), int(y), int(z))
}

/// Decimetres.
fn d3(x: i32, y: i32, z: i32) -> V3 {
    v3(ratio(x, 10), ratio(y, 10), ratio(z, 10))
}

/// A shape of `size` with its middle at `at`, in frame `f`.
fn put(b: &mut Batch, f: &Affine, mesh: Mesh, at: V3, size: V3, c: Rgb, glow: i32) {
    b.push(
        mesh,
        &f.then(&Affine::translate(at)).then(&Affine::scale(size)),
        c,
        glow,
    );
}

/// The same, turned by `turn` about its middle.
fn put_turned(
    b: &mut Batch,
    f: &Affine,
    mesh: Mesh,
    at: V3,
    turn: &Affine,
    size: V3,
    c: Rgb,
    glow: i32,
) {
    b.push(
        mesh,
        &f.then(&Affine::translate(at))
            .then(turn)
            .then(&Affine::scale(size)),
        c,
        glow,
    );
}

/// A rod of `mesh`, `w` across, from `a` to `z` in frame `f`.
fn bar(b: &mut Batch, f: &Affine, mesh: Mesh, a: V3, z: V3, w: i32, c: Rgb, glow: i32) {
    if a.dist(z) > ONE / 100 {
        b.push(mesh, &f.then(&rod(a, z, w)), c, glow);
    }
}

/// A flat plate from `a` to `z` (`chord` wide across the line, level with
/// it, and `thick` through it), in frame `f`: wings, peeled hull plates.
fn slab(b: &mut Batch, f: &Affine, a: V3, z: V3, chord: i32, thick: i32, c: Rgb, glow: i32) {
    let d = z.sub(a);
    let a = Affine::translate(a.add(d.scale(ONE / 2)))
        .then(&Affine::along(d))
        .then(&Affine::scale(v3(chord, d.len(), thick)));
    b.push(Mesh::Cube, &f.then(&a), c, glow);
}

/// An affine from three axes (already scaled) and a middle.
fn axes(at: V3, x: V3, y: V3, z: V3) -> Affine {
    Affine {
        c0: x,
        c1: y,
        c2: z,
        t: at,
    }
}

/// A flicker between 3/4 and 1, out of step for each `k`.
fn flicker(tick: u32, k: i32) -> i32 {
    ONE * 7 / 8 + sin((tick as i32).wrapping_mul(2_900) + k * 11_000) / 8
}

// ---------------------------------------------------------------------------
// The mothership and its wreck

/// Where the mothership breaks in two: metres along its length.
const BREAK: i32 = -10;
/// The wreck's open passage runs across the bow half (along x), through
/// the hull's hangar deck, at this many metres along.
const PASSAGE_Z: i32 = 40;
/// The hull's width and height, metres (the passage's length is the width).
const HULL_W: i32 = 34;
const HULL_H: i32 = 22;
/// The mothership's length, nose to engine nozzles, metres.
pub const MOTHERSHIP_LENGTH: i32 = int(242);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tag {
    Plain,
    /// a light: windows, engines and the beacon (out on the wreck)
    Light,
    /// the solid hull section the wreck's passage goes through
    Passage,
    /// the port wing (torn off the wreck)
    PortWing,
}

#[derive(Clone, Copy)]
struct Part {
    mesh: Mesh,
    a: Affine,
    c: Rgb,
    glow: i32,
    bow: bool,
    tag: Tag,
}

struct Parts(Vec<Part>);

impl Parts {
    fn add(&mut self, mesh: Mesh, at: V3, size: V3, c: Rgb, glow: i32, tag: Tag) {
        self.turned(mesh, at, Affine::IDENTITY, size, c, glow, tag);
    }

    fn turned(&mut self, mesh: Mesh, at: V3, turn: Affine, size: V3, c: Rgb, glow: i32, tag: Tag) {
        self.0.push(Part {
            mesh,
            a: Affine::translate(at).then(&turn).then(&Affine::scale(size)),
            c,
            glow,
            bow: at.z > int(BREAK),
            tag,
        });
    }
}

/// The mothership's parts, whole: a long white hull in segments, a split
/// prow round a hangar mouth, swept wings drooping at the tips, a bridge
/// tower, a keel, turrets, blue bands, rows of lit windows and a block of
/// four engines at the stern.
fn ship_parts() -> Vec<Part> {
    let mut p = Parts(Vec::new());
    let (hw, hh) = (HULL_W, HULL_H);
    // the spine, eight segments of 20 m from -90 to 70
    for i in 0..8 {
        let z = -80 + i * 20;
        let tag = if z == PASSAGE_Z {
            Tag::Passage
        } else {
            Tag::Plain
        };
        let c = if i % 2 == 0 { HULL } else { shade(HULL, 96) };
        p.add(Mesh::Bevel, m3(0, 0, z), m3(hw, hh, 21), c, 0, tag);
    }
    // the raised deck along the top and the keel underneath, in halves
    p.add(
        Mesh::Bevel,
        m3(0, 13, -50),
        m3(18, 6, 80),
        HULL_GREY,
        0,
        Tag::Plain,
    );
    p.add(
        Mesh::Bevel,
        m3(0, 12, 26),
        m3(16, 5, 70),
        HULL_GREY,
        0,
        Tag::Plain,
    );
    p.add(
        Mesh::Bevel,
        m3(0, -13, -50),
        m3(22, 6, 80),
        HULL_DARK,
        0,
        Tag::Plain,
    );
    p.add(
        Mesh::Bevel,
        m3(0, -13, 26),
        m3(20, 6, 70),
        HULL_DARK,
        0,
        Tag::Plain,
    );
    for s in [1, -1] {
        let x = s * (hw / 2) * ONE + s * cm(30);
        // the blue band and the window row along each side, broken at the
        // side hangar door (the wreck's passage)
        for (z0, z1) in [(-92, BREAK), (BREAK, 28), (52, 72)] {
            let (mid, len) = ((z0 + z1) * ONE / 2, (z1 - z0) * ONE);
            p.add(
                Mesh::Cube,
                v3(x, int(2), mid),
                v3(cm(80), int(3), len),
                ACCENT,
                0,
                Tag::Plain,
            );
            p.add(
                Mesh::Cube,
                v3(x, int(7), mid),
                v3(cm(50), cm(80), len - int(2)),
                WINDOW,
                ONE,
                Tag::Light,
            );
        }
        // the side hangar door, shut
        p.add(
            Mesh::Cube,
            v3(x, 0, int(PASSAGE_Z)),
            m3(1, 10, 14),
            HULL_GREY,
            0,
            Tag::Passage,
        );
        // the prow's two prongs, banded, with sloping noses
        p.add(
            Mesh::Bevel,
            m3(s * 11, -2, 90),
            m3(12, 16, 40),
            HULL,
            0,
            Tag::Plain,
        );
        p.add(
            Mesh::Cube,
            m3(s * 11, 5, 90),
            m3(13, 2, 38),
            ACCENT,
            0,
            Tag::Plain,
        );
        p.add(
            Mesh::Wedge,
            m3(s * 11, -2, 115),
            m3(12, 16, 12),
            HULL,
            0,
            Tag::Plain,
        );
        // the wings: swept back, drooping toward blue tips, an engine pod
        // under each
        let wing = if s < 0 { Tag::PortWing } else { Tag::Plain };
        p.turned(
            Mesh::Cube,
            m3(s * 40, -6, -30),
            Affine::rot_y(s * deg(14)).then(&Affine::rot_z(-s * deg(12))),
            m3(48, 2, 40),
            HULL,
            0,
            wing,
        );
        p.turned(
            Mesh::Bevel,
            m3(s * 64, -12, -42),
            Affine::rot_y(s * deg(14)).then(&Affine::rot_z(-s * deg(12))),
            m3(6, 4, 34),
            ACCENT,
            0,
            wing,
        );
        p.turned(
            Mesh::Hex,
            m3(s * 36, -12, -36),
            Affine::rot_x(deg(90)),
            m3(7, 30, 7),
            HULL_GREY,
            0,
            wing,
        );
        p.turned(
            Mesh::Hex,
            m3(s * 36, -12, -51),
            Affine::rot_x(deg(90)),
            v3(int(5), cm(40), int(5)),
            ENGINE,
            SKY_GLOW,
            if s < 0 { Tag::PortWing } else { Tag::Light },
        );
        // turrets on the deck, fore and aft
        for z in [26, -44] {
            p.add(
                Mesh::Hex,
                m3(s * 6, 16, z),
                m3(4, 2, 4),
                HULL_GREY,
                0,
                Tag::Plain,
            );
            p.turned(
                Mesh::Cyl,
                v3(s * int(6), int(17), int(z + 3)),
                Affine::rot_x(deg(90)),
                v3(cm(60), int(6), cm(60)),
                HULL_DARK,
                0,
                Tag::Plain,
            );
        }
    }
    // the hangar mouth between the prongs, lit inside
    p.add(
        Mesh::Bevel,
        m3(0, -2, 76),
        m3(10, 12, 6),
        HULL_DARK,
        0,
        Tag::Plain,
    );
    p.add(
        Mesh::Cube,
        m3(0, -2, 79),
        m3(9, 2, 1),
        WINDOW,
        ONE,
        Tag::Light,
    );
    // the bridge tower and its window band, an aerial and a red beacon
    p.add(
        Mesh::Bevel,
        m3(0, 20, -56),
        m3(14, 10, 16),
        HULL,
        0,
        Tag::Plain,
    );
    p.add(
        Mesh::Cube,
        m3(0, 22, -48),
        m3(12, 2, 1),
        WINDOW,
        ONE,
        Tag::Light,
    );
    p.add(
        Mesh::Cyl,
        m3(0, 30, -58),
        v3(cm(60), int(10), cm(60)),
        HULL_GREY,
        0,
        Tag::Plain,
    );
    p.add(
        Mesh::Octa,
        m3(0, 35, -58),
        m3(2, 2, 2),
        BEACON,
        SKY_GLOW,
        Tag::Light,
    );
    // the engine block and its four nozzles, glowing
    p.add(
        Mesh::Bevel,
        m3(0, 0, -100),
        m3(46, 28, 22),
        HULL_GREY,
        0,
        Tag::Plain,
    );
    for (x, y) in [(11, 6), (-11, 6), (11, -6), (-11, -6)] {
        p.turned(
            Mesh::Hex,
            m3(x, y, -113),
            Affine::rot_x(deg(90)),
            m3(10, 6, 10),
            HULL_DARK,
            0,
            Tag::Plain,
        );
        p.turned(
            Mesh::Hex,
            v3(int(x), int(y), -int(116) - cm(20)),
            Affine::rot_x(deg(90)),
            v3(int(8), cm(40), int(8)),
            ENGINE_CORE,
            SKY_GLOW,
            Tag::Light,
        );
    }
    p.0
}

/// The mothership (a Great Fox–type carrier, our own design): 242 m long,
/// 140 m across the wings, about its middle. Its engines pulse and its
/// beacon blinks with the tick. Frame `f` from `pose`. 63 shapes and 4
/// soft glows.
pub fn mothership(b: &mut Batch, f: &Affine, tick: u32) {
    let blink = (tick / 60).is_multiple_of(2);
    for p in ship_parts() {
        if p.c == BEACON && !blink {
            continue;
        }
        b.push(p.mesh, &f.then(&p.a), p.c, p.glow);
    }
    // the exhaust's soft glow behind the nozzles, pulsing
    for (x, y) in [(11, 6), (-11, 6), (11, -6), (-11, -6)] {
        let s = mul(int(16), flicker(tick, x + y * 3));
        b.push_halo(
            Mesh::Sphere,
            &f.then(&Affine::translate(m3(x, y, -122)))
                .then(&Affine::scale(v3(s, s, s * 2))),
            ENGINE,
            ONE * 3 / 4,
        );
    }
}

/// The open way through the wreck: across the bow half, in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Passage {
    /// the passage's middle, its two mouths, and its axis (unit, from
    /// `entry` to `exit`)
    pub middle: V3,
    pub entry: V3,
    pub exit: V3,
    pub axis: V3,
    /// the clear opening: across the axis along the hull (`width`) and up
    /// the hull (`height`), and the passage's length, in world metres
    pub width: i32,
    pub height: i32,
    pub length: i32,
}

/// The hangar deck's recess above and below the wreck's clear opening
/// (ribs, pipes and lining sit in it): the hole in the hull is this much
/// taller each way than the opening.
pub const RECESS: i32 = cm(120);

/// The default opening through the wreck, metres: 10 wide by 7 tall (the
/// map session sizes it; the hull allows up to 17 by 18).
pub const WRECK_GAP: (i32, i32) = (int(10), int(7));

/// The two halves' frames: the stern drifts back, yawing and rolling; the
/// bow drifts on, nose down and rolled the other way. The gap between the
/// torn faces is about 28 m.
fn halves(f: &Affine) -> (Affine, Affine) {
    let brk = int(BREAK);
    let stern = f
        .then(&Affine::translate(v3(0, -int(2), brk - int(12))))
        .then(&Affine::rot_y(-deg(6)))
        .then(&Affine::rot_z(-deg(5)))
        .then(&Affine::translate(v3(0, 0, -brk)));
    let bow = f
        .then(&Affine::translate(v3(0, int(3), brk + int(16))))
        .then(&Affine::rot_x(deg(7)))
        .then(&Affine::rot_z(deg(9)))
        .then(&Affine::translate(v3(0, 0, -brk)));
    (stern, bow)
}

fn clamp_gap(gap: (i32, i32)) -> (i32, i32) {
    (gap.0.clamp(int(2), int(17)), gap.1.clamp(int(2), int(18)))
}

/// Where the wreck's passage is, for a wreck drawn in frame `f` with
/// opening `gap` (width along the hull, height; clamped to 2–17 m by
/// 2–18 m). It runs straight across the bow half's hull, 34 m long.
pub fn wreck_passage(f: &Affine, gap: (i32, i32)) -> Passage {
    let (w, h) = clamp_gap(gap);
    let (_, bow) = halves(f);
    let half = int(HULL_W / 2) + cm(60);
    let entry = bow.apply(v3(-half, 0, int(PASSAGE_Z)));
    let exit = bow.apply(v3(half, 0, int(PASSAGE_Z)));
    Passage {
        middle: bow.apply(m3(0, 0, PASSAGE_Z)),
        entry,
        exit,
        axis: exit.sub(entry).norm(),
        width: bow.apply_dir(v3(0, 0, w)).len(),
        height: bow.apply_dir(v3(0, h, 0)).len(),
        length: entry.dist(exit),
    }
}

/// The same ship destroyed: broken in two at 10 m aft of its middle, the
/// halves drifting apart (the stern yawed back, the bow nose-down), torn
/// faces showing decks and ribs, hull plates peeled open, scorch marks,
/// embers and fires, the port wing torn off and debris tumbling between
/// the halves. Its lights are out but for red alarm strips.
///
/// The open passage runs across the bow half through its hangar deck:
/// `gap` is the opening (see `wreck_passage` for where it is). 121 shapes
/// and 2 soft glows; about 266 m long overall.
pub fn wreck(b: &mut Batch, f: &Affine, tick: u32, gap: (i32, i32)) {
    let (stern, bow) = halves(f);
    // the port wing, torn off, drifting below and aft
    let wing_off = Affine::translate(m3(-14, -16, -18))
        .then(&Affine::rot_z(-deg(38)))
        .then(&Affine::rot_y(deg(20)));
    for p in ship_parts() {
        if p.tag == Tag::Light || p.tag == Tag::Passage {
            continue;
        }
        let half = if p.bow { &bow } else { &stern };
        let a = if p.tag == Tag::PortWing {
            stern.then(&wing_off).then(&p.a)
        } else {
            half.then(&p.a)
        };
        // the engine pods are dark
        let (c, glow) = if p.glow > ONE {
            (HULL_DARK, 0)
        } else {
            (p.c, p.glow)
        };
        b.push(p.mesh, &a, c, glow);
    }
    hollow(b, &bow, tick, gap);
    torn_face(b, &bow, tick, 1);
    torn_face(b, &stern, tick, -1);
    // scorch marks over the hull, each half
    const BURNS: [(i32, i32, i32, i32, i32); 10] = [
        (1, 17, 4, -30, 9),
        (-1, 17, -3, -64, 12),
        (1, 0, 11, -40, 14),
        (-1, 0, 11, -20, 10),
        (1, 17, -6, 60, 8),
        (-1, 17, 6, 10, 12),
        (1, 0, 11, 6, 9),
        (-1, 0, -11, 18, 16),
        (1, 17, 3, 22, 6),
        (-1, 0, 11, 58, 7),
    ];
    for (k, &(s, x, y, z, size)) in BURNS.iter().enumerate() {
        let half = if z > BREAK { &bow } else { &stern };
        let at = if x != 0 {
            v3(s * (int(x) + cm(40)), int(y), int(z))
        } else {
            v3(s * int(6), int(y) + y.signum() * cm(40), int(z))
        };
        let sz = if x != 0 {
            v3(int(1), int(size) * 2 / 3, int(size))
        } else {
            v3(int(size), int(1), int(size) * 3 / 4)
        };
        put(b, half, Mesh::Bevel, at, sz, SCORCH, 0);
        // a few still smoulder
        if k % 3 == 0 {
            let e = mul(int(2), flicker(tick, k as i32));
            put(b, half, Mesh::Octa, at, v3(e, e, e), EMBER, ONE);
        }
    }
    // debris tumbling between the halves
    for k in 0..8u32 {
        let t = tumble(tick, k);
        let at = m3(
            -14 + (k as i32 * 9) % 28,
            -8 + (k as i32 * 7) % 18,
            BREAK - 8 + (k as i32 * 5) % 18,
        );
        let size = m3(2 + (k % 3) as i32 * 2, 1, 2 + (k % 4) as i32);
        let (mesh, c) = if k % 2 == 0 {
            (Mesh::Wedge, HULL_GREY)
        } else {
            (Mesh::Cube, SCORCH)
        };
        put_turned(b, f, mesh, at, &t, size, c, 0);
    }
}

/// The hangar section of the bow with the passage blown through it:
/// slabs above and below the opening, walls fore and aft of it, a dark
/// lining, ribs, red alarm strips, and hull plates peeled out round both
/// mouths, scorched.
fn hollow(b: &mut Batch, bow: &Affine, tick: u32, gap: (i32, i32)) {
    let (w, h) = clamp_gap(gap);
    let (hw, hh, z0, len) = (int(HULL_W), int(HULL_H) / 2, int(PASSAGE_Z), int(21));
    // the ribs, pipe, lining and alarms sit in a recess above and below
    // the clear opening, so the opening is clear
    let rec = RECESS;
    let (top, side) = ((hh - h / 2 - rec).max(ONE / 4), (len - w) / 2);
    for s in [1, -1] {
        // above and below the opening
        put(
            b,
            bow,
            Mesh::Bevel,
            v3(0, s * (h / 2 + rec + top / 2), z0),
            v3(hw, top, len),
            if s > 0 { HULL } else { shade(HULL, 96) },
            0,
        );
        // fore and aft of it
        if side > ONE / 4 {
            put(
                b,
                bow,
                Mesh::Cube,
                v3(0, 0, z0 + s * (w / 2 + side / 2)),
                v3(hw, h + rec * 2, side),
                HULL,
                0,
            );
        }
        // the lining: deck and ceiling plates, dark
        put(
            b,
            bow,
            Mesh::Cube,
            v3(0, s * (h / 2 + rec - cm(15)), z0),
            v3(hw - int(1), cm(30), w),
            INTERIOR,
            0,
        );
        // the alarm strips along the deck, flashing
        if (tick / 40 + (s > 0) as u32).is_multiple_of(2) {
            put(
                b,
                bow,
                Mesh::Cube,
                v3(0, -h / 2 - cm(20), z0 + s * (w / 2 - cm(30))),
                v3(hw - int(2), cm(30), cm(30)),
                ALARM,
                ONE,
            );
        }
        // the plates peeled out round each mouth, and the scorch
        let x = s * (hw / 2);
        for (y, out) in [(h / 2 + rec, int(3)), (-h / 2 - rec, -int(3))] {
            slab(
                b,
                bow,
                v3(x, y, z0),
                v3(x + s * int(4), y + out, z0 + cm(80)),
                w * 4 / 5,
                cm(50),
                HULL_GREY,
                0,
            );
        }
        for zs in [1, -1] {
            put(
                b,
                bow,
                Mesh::Bevel,
                v3(x + s * cm(30), 0, z0 + zs * (w / 2 + int(2))),
                v3(cm(80), h + rec * 2 + int(4), int(4)),
                SCORCH,
                0,
            );
        }
    }
    // ribs across the ceiling and a pipe run along it
    for x in [-9, 0, 9] {
        put(
            b,
            bow,
            Mesh::Cube,
            v3(int(x), h / 2 + rec / 2, z0),
            v3(int(1), cm(80), w),
            RIB,
            0,
        );
    }
    bar(
        b,
        bow,
        Mesh::Cyl,
        v3(-hw / 2, h / 2 + rec / 2, z0 + w / 4),
        v3(hw / 2, h / 2 + rec / 2, z0 + w / 4),
        cm(70),
        RIB,
        0,
    );
}

/// A torn face where the hull broke (`dir` 1: the bow's, facing aft; -1:
/// the stern's, facing forward): the dark insides, decks and ribs sticking
/// out jagged, hull plates peeled outward, embers and a fire's glow.
fn torn_face(b: &mut Batch, half: &Affine, tick: u32, dir: i32) {
    let z = int(BREAK);
    let out = -dir; // toward the gap
    put(
        b,
        half,
        Mesh::Bevel,
        v3(0, 0, z + out * cm(60)),
        m3(HULL_W - 2, HULL_H - 2, 1),
        INTERIOR,
        0,
    );
    for (k, y) in [-6, 0, 6].into_iter().enumerate() {
        let k = k as i32;
        put(
            b,
            half,
            Mesh::Cube,
            v3(0, int(y), z + out * int(2 + k)),
            v3(int(30 - k * 5), cm(60), int(4 + k * 2)),
            RIB,
            0,
        );
    }
    const JAGS: [(i32, i32, i32, i32, i32); 5] = [
        (-15, 9, -2, 2, 7),
        (-6, 10, 1, 3, 5),
        (8, 9, 2, 2, 8),
        (15, -4, 3, -1, 6),
        (-14, -8, -3, -2, 9),
    ];
    for (x, y, jx, jy, len) in JAGS {
        let a = v3(int(x), int(y), z);
        bar(
            b,
            half,
            Mesh::Cube,
            a,
            a.add(v3(int(jx), int(jy), out * int(len))),
            cm(70),
            RIB,
            0,
        );
    }
    // plates peeled back from the top and the sides
    let peel = [
        (m3(-6, 11, 0), m3(-8, 15, 0)),
        (m3(7, 11, 0), m3(10, 16, 0)),
        (m3(17, 3, 0), m3(22, 5, 0)),
        (m3(-17, -2, 0), m3(-22, -5, 0)),
    ];
    for (a, z1) in peel {
        let (a, z1) = (a.add(v3(0, 0, z)), z1.add(v3(0, 0, z + out * int(5))));
        slab(b, half, a, z1, int(7), cm(50), HULL, 0);
    }
    for k in 0..4 {
        let e = mul(int(3), flicker(tick, k + dir * 7));
        put(
            b,
            half,
            Mesh::Octa,
            m3(-9 + k * 6, -5 + (k * 5) % 9, 0).add(v3(0, 0, z + out * int(1))),
            v3(e, e, e),
            EMBER,
            ONE,
        );
    }
    let g = mul(int(26), flicker(tick, dir * 3));
    b.push_halo(
        Mesh::Sphere,
        &half
            .then(&Affine::translate(v3(0, 0, z + out * int(3))))
            .then(&Affine::scale(v3(g, g * 4 / 5, g / 2))),
        EMBER,
        ONE / 2,
    );
}

// ---------------------------------------------------------------------------
// Monkey mechs

/// A mech's build: the grunt (13 m tall, oxblood) or the brute (half as big
/// again, purple, horned and spiked).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MechKind {
    Grunt,
    Brute,
}

/// How a mech moves: marching (legs striding, long arms swinging, a
/// stride every 2 s) or drifting (legs dangling, arms reaching forward,
/// back thrusters burning).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gait {
    March,
    Drift,
}

/// One mech: its build, its gait and its own phase (an angle, so a line of
/// them is out of step).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mech {
    pub kind: MechKind,
    pub gait: Gait,
    pub phase: i32,
}

/// Where vines can hang from a mech, in the world: the top of its head, its
/// fists, its elbows and its shoulders (index 0 its left, on +x as built,
/// 1 its right).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MechAnchors {
    pub head: V3,
    pub hands: [V3; 2],
    pub elbows: [V3; 2],
    pub shoulders: [V3; 2],
}

/// The grunt's height, feet to crown, as built (metres).
pub const MECH_HEIGHT: i32 = int(13);

struct MechPose {
    bob: i32,
    lean: i32,
    head: V3,
    shoulders: [V3; 2],
    elbows: [V3; 2],
    hands: [V3; 2],
    hips: [V3; 2],
    knees: [V3; 2],
    feet: [V3; 2],
    thrust: i32,
}

/// The elbow or knee between `a` and `t` for bones `l1` and `l2`, bent
/// toward `hint`.
fn joint(a: V3, t: V3, l1: i32, l2: i32, hint: V3) -> V3 {
    let d = t.sub(a);
    let dist = d.len().clamp(ONE / 10, l1 + l2 - ONE / 50);
    let dir = d.norm();
    let along = fx::div(mul(l1, l1) - mul(l2, l2) + mul(dist, dist), dist * 2);
    let up = fx::sqrt(mul(l1, l1) - mul(along, along));
    let side = hint.reject(dir).norm();
    a.add(dir.scale(along)).add(side.scale(up))
}

fn mech_pose(m: &Mech, tick: u32) -> MechPose {
    let t = tick as i32;
    let tps = crate::rules::TICKS_PER_SECOND;
    let (march, w) = match m.gait {
        Gait::March => (true, t.wrapping_mul(TURN / (tps * 2)) + m.phase),
        Gait::Drift => (false, t.wrapping_mul(TURN / (tps * 6)) + m.phase),
    };
    let bob = if march {
        mul(sin(w * 2).abs(), cm(35))
    } else {
        mul(sin(w), cm(60))
    };
    let mut p = MechPose {
        bob,
        lean: deg(18),
        head: v3(0, cm(590) + bob, cm(310)),
        shoulders: [V3::ZERO; 2],
        elbows: [V3::ZERO; 2],
        hands: [V3::ZERO; 2],
        hips: [V3::ZERO; 2],
        knees: [V3::ZERO; 2],
        feet: [V3::ZERO; 2],
        thrust: if march { ONE / 4 } else { ONE },
    };
    for i in 0..2 {
        let sx = if i == 0 { 1 } else { -1 };
        let lw = w + i as i32 * HALF;
        p.hips[i] = v3(sx * cm(170), -cm(60) + bob, 0);
        p.shoulders[i] = v3(sx * cm(370), cm(480) + bob, cm(120));
        let (foot, hand) = if march {
            let swing = sin(lw);
            let lift = mul(fx::cos(lw).max(0), cm(90));
            (
                v3(sx * cm(190), -cm(660) + lift, mul(swing, cm(180))),
                v3(
                    sx * cm(440),
                    -cm(260) + bob + mul(swing.abs(), cm(60)),
                    cm(140) - mul(swing, cm(220)),
                ),
            )
        } else {
            (
                v3(
                    sx * cm(220),
                    -cm(620),
                    -cm(100) + mul(sin(lw + QUARTER), cm(80)),
                ),
                v3(sx * cm(460), cm(120) + mul(sin(lw), cm(60)) + bob, cm(440)),
            )
        };
        p.feet[i] = foot;
        p.hands[i] = hand;
        p.knees[i] = joint(p.hips[i], foot, cm(340), cm(360), v3(0, 0, ONE));
        p.elbows[i] = joint(
            p.shoulders[i],
            hand,
            cm(390),
            cm(410),
            v3(sx * ONE / 2, 0, -ONE),
        );
    }
    p
}

/// The frame a mech is drawn in: the caller's, grown for a brute.
fn mech_frame(f: &Affine, kind: MechKind) -> Affine {
    match kind {
        MechKind::Grunt => *f,
        MechKind::Brute => f.then(&Affine::scale(v3(ONE * 3 / 2, ONE * 3 / 2, ONE * 3 / 2))),
    }
}

/// Where a mech's vine anchors are this tick (frame `f` from `pose`, as
/// `mech` is drawn).
pub fn mech_anchors(f: &Affine, m: &Mech, tick: u32) -> MechAnchors {
    let g = mech_frame(f, m.kind);
    let p = mech_pose(m, tick);
    MechAnchors {
        head: g.apply(p.head.add(v3(0, cm(130), -cm(20)))),
        hands: p.hands.map(|h| g.apply(h.sub(v3(0, cm(95), 0)))),
        elbows: p.elbows.map(|e| g.apply(e)),
        shoulders: p.shoulders.map(|s| g.apply(s.add(v3(0, cm(120), 0)))),
    }
}

/// A monkey mech of the enemy armada (after Sector Y, our own design):
/// bipedal, hunched, with an ape's head (brow ridge, muzzle, round ears,
/// glowing red eyes), a barrel chest, huge shoulders and long arms ending
/// in fists down by its knees, a thruster pack on its back. Built about
/// its pelvis: the grunt stands 13 m (feet 6.6 m below, crown 7 m above);
/// the brute half as big again (23 m with its spikes). Frame `f` from
/// `pose`. 37 shapes (41 for a brute).
pub fn mech(b: &mut Batch, f: &Affine, m: &Mech, tick: u32) {
    let g = mech_frame(f, m.kind);
    let p = mech_pose(m, tick);
    let brute = m.kind == MechKind::Brute;
    let plate = if brute {
        rgb(70, 26, 92)
    } else {
        rgb(122, 30, 36)
    };
    let metal = rgb(66, 64, 74);
    let dark = rgb(30, 28, 36);
    let face = if brute {
        rgb(96, 84, 76)
    } else {
        rgb(130, 98, 72)
    };
    let eye = rgb(255, 44, 30);
    let lean = Affine::rot_x(p.lean);
    let bob = p.bob;
    // pelvis, belly, the barrel chest and its plate, a glowing core
    put(b, &g, Mesh::Bevel, v3(0, bob, 0), d3(36, 14, 22), metal, 0);
    put(
        b,
        &g,
        Mesh::Bevel,
        v3(0, cm(120) + bob, cm(60)),
        d3(28, 16, 24),
        metal,
        0,
    );
    put_turned(
        b,
        &g,
        Mesh::Barrel,
        v3(0, cm(340) + bob, cm(90)),
        &lean,
        d3(56, 52, 42),
        plate,
        0,
    );
    put_turned(
        b,
        &g,
        Mesh::Bevel,
        v3(0, cm(380) + bob, cm(270)),
        &lean,
        d3(34, 24, 8),
        dark,
        0,
    );
    put(
        b,
        &g,
        Mesh::Octa,
        v3(0, cm(370) + bob, cm(318)),
        d3(10, 10, 6),
        eye,
        SKY_GLOW,
    );
    // the back's thruster pack, two nozzles and their flames
    put_turned(
        b,
        &g,
        Mesh::Bevel,
        v3(0, cm(380) + bob, -cm(150)),
        &lean,
        d3(40, 32, 18),
        dark,
        0,
    );
    for sx in [1, -1] {
        let n = v3(sx * cm(110), cm(190) + bob, -cm(200));
        put(b, &g, Mesh::Hex, n, d3(10, 10, 10), metal, 0);
        let len = cm(80) + mul(p.thrust, mul(int(3), flicker(tick, sx)));
        bar(
            b,
            &g,
            Mesh::Cone,
            n.sub(v3(0, cm(40) + len, 0)),
            n.sub(v3(0, cm(40), 0)),
            cm(80),
            rgb(255, 150, 60),
            SKY_GLOW,
        );
    }
    // the shoulders: huge pauldrons (spiked on a brute)
    for i in 0..2 {
        let s = p.shoulders[i];
        put(
            b,
            &g,
            Mesh::Geo,
            s.add(d3(0, 4, 0)),
            d3(30, 26, 30),
            plate,
            0,
        );
        if brute {
            let sx = if i == 0 { 1 } else { -1 };
            bar(
                b,
                &g,
                Mesh::Spike,
                s.add(d3(0, 14, 0)),
                s.add(d3(sx * 8, 34, -4)),
                cm(90),
                dark,
                0,
            );
        }
    }
    // the head: skull, brow ridge, muzzle and mouth grille, eyes, ears
    let h = p.head;
    bar(
        b,
        &g,
        Mesh::Hex,
        v3(0, cm(500) + bob, cm(180)),
        h,
        cm(140),
        dark,
        0,
    );
    put(b, &g, Mesh::Bevel, h, d3(26, 22, 24), metal, 0);
    put_turned(
        b,
        &g,
        Mesh::Bevel,
        h.add(d3(0, 6, 11)),
        &Affine::rot_x(-deg(12)),
        d3(31, 7, 11),
        dark,
        0,
    );
    put(
        b,
        &g,
        Mesh::Bevel,
        h.add(d3(0, -6, 13)),
        d3(19, 12, 12),
        face,
        0,
    );
    put(
        b,
        &g,
        Mesh::Cube,
        h.add(d3(0, -9, 19)),
        d3(13, 2, 2),
        dark,
        0,
    );
    for sx in [1, -1] {
        put(
            b,
            &g,
            Mesh::Octa,
            h.add(d3(sx * 6, 1, 12)),
            d3(6, 3, 3),
            eye,
            SKY_GLOW,
        );
        bar(
            b,
            &g,
            Mesh::Hex,
            h.add(d3(sx * 12, 2, 0)),
            h.add(d3(sx * 16, 2, 0)),
            cm(130),
            plate,
            0,
        );
        if brute {
            bar(
                b,
                &g,
                Mesh::Spike,
                h.add(d3(sx * 7, 10, -2)),
                h.add(d3(sx * 14, 24, -8)),
                cm(60),
                rgb(220, 210, 190),
                0,
            );
        }
    }
    // arms: upper arm, elbow, a great forearm widening to the fist
    for i in 0..2 {
        let (s, e, hand) = (p.shoulders[i], p.elbows[i], p.hands[i]);
        bar(b, &g, Mesh::Taper, s, e, cm(140), metal, 0);
        put(b, &g, Mesh::Ico, e, d3(14, 14, 14), dark, 0);
        bar(b, &g, Mesh::Taper, hand, e, cm(190), plate, 0);
        put(
            b,
            &g,
            Mesh::Bevel,
            hand.sub(d3(0, 2, 0)),
            d3(18, 19, 20),
            dark,
            0,
        );
    }
    // legs: thigh, knee, shin, foot
    for i in 0..2 {
        let (hip, k, ft) = (p.hips[i], p.knees[i], p.feet[i]);
        bar(b, &g, Mesh::Taper, hip, k, cm(170), metal, 0);
        put(b, &g, Mesh::Ico, k, d3(15, 15, 15), plate, 0);
        bar(
            b,
            &g,
            Mesh::Taper,
            k,
            ft.add(d3(0, 4, 0)),
            cm(150),
            metal,
            0,
        );
        put(
            b,
            &g,
            Mesh::Bevel,
            ft.add(d3(0, 0, 5)),
            d3(16, 8, 26),
            dark,
            0,
        );
    }
}

// ---------------------------------------------------------------------------
// Fighters, lasers and the distant battles

/// An ape-wing (our own fighter, after the Arwing's silhouette): a white
/// dart with a glass canopy, two round ape-ear intakes behind it, swept
/// wings with banana-yellow tip fins, and twin engines. 11 m long, 12 m
/// across, about its middle. 15 shapes.
pub fn ape_wing(b: &mut Batch, f: &Affine, tick: u32) {
    put(b, f, Mesh::Bevel, V3::ZERO, d3(18, 13, 60), APE_WHITE, 0);
    put_turned(
        b,
        f,
        Mesh::Pyramid,
        d3(0, 0, 47),
        &Affine::rot_x(deg(90)),
        d3(16, 34, 11),
        APE_WHITE,
        0,
    );
    put(b, f, Mesh::Bevel, d3(0, 5, 5), d3(19, 3, 40), APE_YELLOW, 0);
    put(b, f, Mesh::Geo, d3(0, 8, 12), d3(11, 9, 24), GLASS, ONE / 4);
    put_turned(
        b,
        f,
        Mesh::Shard,
        d3(0, 10, -23),
        &Affine::rot_y(deg(90)),
        d3(18, 16, 2),
        APE_WHITE,
        0,
    );
    let flame = mul(cm(120), flicker(tick, 1));
    for sx in [1, -1] {
        bar(
            b,
            f,
            Mesh::Hex,
            d3(sx * 8, 9, -9),
            d3(sx * 8, 9, -5),
            cm(90),
            APE_FUR,
            0,
        );
        slab(
            b,
            f,
            d3(sx * 8, -1, -6),
            d3(sx * 56, -9, -26),
            cm(260),
            cm(25),
            APE_WHITE,
            0,
        );
        put_turned(
            b,
            f,
            Mesh::Shard,
            d3(sx * 56, -4, -26),
            &Affine::rot_z(-sx * deg(15)).then(&Affine::rot_y(deg(90))),
            d3(22, 20, 3),
            APE_YELLOW,
            ONE / 4,
        );
        bar(
            b,
            f,
            Mesh::Hex,
            d3(sx * 6, 0, -28),
            d3(sx * 6, 0, -34),
            cm(80),
            HULL_DARK,
            0,
        );
        bar(
            b,
            f,
            Mesh::Cone,
            d3(sx * 6, 0, -34),
            d3(sx * 6, 0, -34).sub(v3(0, 0, cm(50) + flame)),
            cm(55),
            ENGINE,
            SKY_GLOW,
        );
    }
}

/// An enemy fighter: a dark pod with a magenta eye, three red claws swept
/// forward round it, a spine, and a magenta engine. 9 m long. 7 shapes.
pub fn enemy_fighter(b: &mut Batch, f: &Affine, tick: u32) {
    put(b, f, Mesh::Ico, V3::ZERO, d3(22, 18, 30), ENEMY_HULL, 0);
    put(
        b,
        f,
        Mesh::Octa,
        d3(0, 2, 15),
        d3(8, 6, 5),
        ENEMY_GLOW,
        SKY_GLOW,
    );
    for k in 0..3 {
        let a = k * (TURN / 3) + QUARTER;
        let r = v3(fx::cos(a), sin(a), 0);
        bar(
            b,
            f,
            Mesh::Shard,
            r.scale(int(1)).sub(v3(0, 0, cm(50))),
            r.scale(cm(380)).add(v3(0, 0, cm(280))),
            cm(130),
            ENEMY_CLAW,
            0,
        );
    }
    bar(
        b,
        f,
        Mesh::Spike,
        d3(0, 6, -6),
        d3(0, 22, -26),
        cm(90),
        ENEMY_CLAW,
        0,
    );
    let flame = mul(cm(140), flicker(tick, 5));
    bar(
        b,
        f,
        Mesh::Cone,
        d3(0, 0, -14),
        d3(0, 0, -14).sub(v3(0, 0, cm(60) + flame)),
        cm(110),
        ENEMY_GLOW,
        SKY_GLOW,
    );
}

/// A laser bolt from `a` to `z` (world), `w` across: a bright rod and a
/// soft glow round it. 1 shape and 1 soft glow.
pub fn laser(b: &mut Batch, a: V3, z: V3, w: i32, c: Rgb) {
    if a.dist(z) < ONE / 100 {
        return;
    }
    b.push(Mesh::Cyl, &rod(a, z, w), c, SKY_GLOW);
    b.push_halo(Mesh::Cyl, &rod(a, z, w * 4), c, ONE / 2);
}

/// A flash where a bolt hit, `age` ticks after (gone after 30).
fn blast(b: &mut Batch, at: V3, age: u32, size: i32) {
    if age >= 30 {
        return;
    }
    let s = mul(size, ONE / 3 + ratio(age as i32, 20));
    b.push(
        Mesh::Ico,
        &Affine::translate(at).then(&Affine::scale(v3(s, s, s))),
        BLAST,
        SKY_GLOW,
    );
}

/// A point on a dogfight's loop: a figure of eight 120 m across and 60 m
/// deep, rising and falling, at angle `a` (`g` tilts each pair's own).
fn loop_at(a: i32, g: i32) -> V3 {
    v3(
        mul(sin(a), int(60)),
        mul(sin(a * 2 + g), int(14)) + g / 1000,
        mul(sin(a * 2), int(30)),
    )
}

/// A dogfight in the distance, about the middle of frame `f`: two pairs
/// chasing each other round crossing loops 120 m across (an ape-wing
/// chasing an enemy and its wingman, and an enemy chasing an ape-wing), the
/// chaser firing twin bolts every 0.3 s that fly a fifth of a second, every
/// fourth volley hitting. A loop takes 14 s. 55 shapes plus up to 5 for
/// bolts and flashes, and up to 4 soft glows; for far away, draw at 2–4
/// times scale.
pub fn dogfight(b: &mut Batch, f: &Affine, tick: u32) {
    let tps = crate::rules::TICKS_PER_SECOND;
    let lag = deg(26);
    for g in 0..2i32 {
        let a = (tick as i32).wrapping_mul(TURN / (tps * 14)) + g * HALF;
        let tilt = g * deg(70);
        let at = |a: i32| f.apply(loop_at(a, tilt));
        let heading = |a: i32| at(a + deg(2)).sub(at(a));
        let (lead, chase) = (at(a), at(a - lag));
        let place = |p: V3, d: V3| {
            Affine::translate(p)
                .then(&Affine::looking(d, V3::UP))
                .then(&Affine::scale(v3(f.c0.len(), f.c0.len(), f.c0.len())))
        };
        let (fl, fc) = (place(lead, heading(a)), place(chase, heading(a - lag)));
        // pair 0: an enemy fleeing an ape-wing (and an enemy wingman);
        // pair 1: an ape-wing fleeing an enemy
        if g == 0 {
            enemy_fighter(b, &fl, tick);
            ape_wing(b, &fc, tick);
            let wing = at(a + deg(8)).add(fl.apply_dir(m3(4, 1, 0)));
            enemy_fighter(b, &place(wing, heading(a + deg(8))), tick + 17);
        } else {
            ape_wing(b, &fl, tick);
            enemy_fighter(b, &fc, tick);
        }
        let colour = if g == 0 { LASER_GOOD } else { LASER_BAD };
        let period = 36u32;
        let t = tick + g as u32 * 13;
        let age = t % period;
        if age < 24 {
            let to = lead.sub(chase).norm();
            let from = fc.apply(m3(0, 0, 6));
            let speed = fc.apply_dir(m3(0, 0, 3)).len();
            for sx in [1, -1] {
                let start = from
                    .add(fc.apply_dir(d3(sx * 6, 0, 0)))
                    .add(to.scale(speed * age as i32));
                laser(
                    b,
                    start,
                    start.add(to.scale(speed * 2)),
                    fc.c0.len() / 3,
                    colour,
                );
            }
        }
        // every fourth volley hits
        if (t / period) % 4 == 3 && age >= 20 {
            blast(b, lead, age - 20, f.c0.len() * 4);
        }
    }
}

/// A friendly cruiser, 90 m: a white hull with a sloping prow, a blue
/// band, a bridge, two turrets and three engines. 12 shapes.
fn cruiser(b: &mut Batch, f: &Affine, tick: u32) {
    put(b, f, Mesh::Bevel, V3::ZERO, m3(14, 10, 66), HULL, 0);
    put(b, f, Mesh::Wedge, m3(0, 0, 41), m3(14, 10, 16), HULL, 0);
    put(b, f, Mesh::Cube, m3(0, 1, 0), m3(15, 2, 60), ACCENT, 0);
    put(b, f, Mesh::Bevel, m3(0, 8, -18), m3(7, 7, 12), HULL_GREY, 0);
    put(b, f, Mesh::Cube, m3(0, 10, -12), m3(6, 1, 1), WINDOW, ONE);
    for z in [18, 4] {
        put(b, f, Mesh::Hex, m3(0, 6, z), m3(5, 2, 5), HULL_GREY, 0);
    }
    for x in [-4, 0, 4] {
        let s = mul(int(4), flicker(tick, x));
        put_turned(
            b,
            f,
            Mesh::Hex,
            m3(x, 0, -34),
            &Affine::rot_x(deg(90)),
            v3(s, cm(60), s),
            ENGINE_CORE,
            SKY_GLOW,
        );
    }
}

/// An enemy dreadnought, 100 m: a dark purple barrel with two red prongs
/// forward, spines along its back, glowing magenta slits and engines. 14
/// shapes.
fn dreadnought(b: &mut Batch, f: &Affine, tick: u32) {
    put_turned(
        b,
        f,
        Mesh::Barrel,
        V3::ZERO,
        &Affine::rot_x(deg(90)),
        m3(20, 80, 16),
        ENEMY_HULL,
        0,
    );
    for sx in [1, -1] {
        bar(
            b,
            f,
            Mesh::Spike,
            m3(sx * 7, 0, 30),
            m3(sx * 4, -2, 62),
            int(6),
            ENEMY_CLAW,
            0,
        );
        put(
            b,
            f,
            Mesh::Cube,
            m3(sx * 9, 2, 0),
            m3(1, 2, 40),
            ENEMY_GLOW,
            ONE,
        );
        let s = mul(int(6), flicker(tick, sx * 3));
        put_turned(
            b,
            f,
            Mesh::Hex,
            m3(sx * 5, 0, -41),
            &Affine::rot_x(deg(90)),
            v3(s, int(1), s),
            ENEMY_GLOW,
            SKY_GLOW,
        );
    }
    for k in 0..4 {
        let z = -24 + k * 14;
        bar(
            b,
            f,
            Mesh::Spike,
            m3(0, 7, z),
            m3(0, 16 - k, z - 6),
            int(4),
            ENEMY_CLAW,
            0,
        );
    }
    put(
        b,
        f,
        Mesh::Octa,
        m3(0, 4, 36),
        m3(5, 3, 3),
        ENEMY_GLOW,
        SKY_GLOW,
    );
}

/// Two capital ships trading fire in the distance, in frame `f`: a
/// friendly cruiser at -120 m on x and an enemy dreadnought at +120 m,
/// broadside on, bobbing; each fires a bolt from each turret every
/// second or so that crosses in 0.4 s and flashes where it hits. 25 shapes
/// for the ships, up to 6 more for bolts and flashes.
pub fn capital_battle(b: &mut Batch, f: &Affine, tick: u32) {
    let t = tick as i32;
    let bob = |k: i32| mul(sin(t.wrapping_mul(150) + k * 20_000), int(2));
    let good = f
        .then(&Affine::translate(v3(-int(120), bob(0), 0)))
        .then(&Affine::rot_y(deg(4)));
    let bad = f
        .then(&Affine::translate(v3(int(120), int(10) + bob(1), int(20))))
        .then(&Affine::rot_y(HALF - deg(6)));
    cruiser(b, &good, tick);
    dreadnought(b, &bad, tick);
    let scale = f.c0.len();
    let flight = 48u32;
    for k in 0..3u32 {
        let (period, offset) = (110 + k * 17, k * 37);
        let shots = [
            (&good, m3(0, 6, 18 - k as i32 * 14), &bad, LASER_GOOD),
            (&bad, m3(0, 7, -20 + k as i32 * 18), &good, LASER_BAD),
        ];
        for (j, (from, at, to, c)) in shots.into_iter().enumerate() {
            let age = (tick + offset + j as u32 * 50) % period;
            let a = from.apply(at);
            let z = to.apply(m3(0, 0, 20 - k as i32 * 20));
            if age < flight {
                let d = z.sub(a);
                let s = ratio(age as i32, flight as i32);
                let head = a.add(d.scale(s));
                let tail = a.add(d.scale((s - ONE / 6).max(0)));
                laser(b, tail, head, scale, c);
            } else {
                blast(b, z, age - flight, scale * 10);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Debris

/// A floating chunk of hull (variant `k`): a curved run of plates with the
/// mothership's blue band, ribs underneath, a scorch, torn spikes at one
/// end and a blinking light. About 12 m by 10 m. 12 shapes.
pub fn hull_chunk(b: &mut Batch, f: &Affine, tick: u32, k: u32) {
    let k = (k % 3) as i32;
    put(b, f, Mesh::Bevel, V3::ZERO, m3(8, 1, 10), HULL, 0);
    for sx in [1, -1] {
        put_turned(
            b,
            f,
            Mesh::Bevel,
            d3(sx * 56, -12, 0),
            &Affine::rot_z(sx * deg(25 + k * 8)),
            v3(int(4), int(1), int(10 - k * 2)),
            shade(HULL, 94),
            0,
        );
    }
    for z in [-3, 0, 3] {
        put(b, f, Mesh::Cube, m3(0, -1, z), d3(110, 8, 6), RIB, 0);
    }
    put(b, f, Mesh::Cube, d3(0, 5, 20), d3(81, 2, 12), ACCENT, 0);
    put(
        b,
        f,
        Mesh::Bevel,
        d3(15 - k * 10, 4, -30),
        d3(40, 4, 30),
        SCORCH,
        0,
    );
    for x in [-2, 2] {
        bar(
            b,
            f,
            Mesh::Spike,
            m3(x, 0, -5),
            m3(x + k - 1, 0, -8),
            cm(90),
            HULL_GREY,
            0,
        );
    }
    if (tick / 60 + k as u32).is_multiple_of(2) {
        put(
            b,
            f,
            Mesh::Octa,
            d3(-30, 7, 40),
            d3(5, 5, 5),
            BEACON,
            SKY_GLOW,
        );
    }
}

/// A broken girder along z, `len` metres long (at least 6), 2 m square: four
/// rails, a diagonal brace on two faces in each 3 m bay, a hazard band and
/// one rail bent out at the broken end. 4 + 2 per bay + 2 shapes.
pub fn girder(b: &mut Batch, f: &Affine, len: i32) {
    let len = len.max(6);
    let half = int(len) / 2;
    for (x, y) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let end = if (x, y) == (1, 1) {
            half - int(3)
        } else {
            half
        };
        put(
            b,
            f,
            Mesh::Cube,
            v3(int(x), int(y), (end - half) / 2),
            v3(cm(35), cm(35), end + half),
            RIB,
            0,
        );
    }
    bar(
        b,
        f,
        Mesh::Cube,
        v3(int(1), int(1), half - int(3)),
        v3(int(3), int(2), half),
        cm(35),
        RIB,
        0,
    );
    for i in 0..len / 3 {
        let (z0, z1) = (-half + int(i * 3), -half + int(i * 3 + 3));
        bar(
            b,
            f,
            Mesh::Cube,
            v3(-int(1), int(1), z0),
            v3(int(1), int(1), z1),
            cm(25),
            RIB,
            0,
        );
        bar(
            b,
            f,
            Mesh::Cube,
            v3(int(1), -int(1), z0),
            v3(int(1), int(1), z1),
            cm(25),
            RIB,
            0,
        );
    }
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, 0, -half + int(1)),
        v3(cm(230), cm(230), cm(60)),
        HAZARD,
        0,
    );
}

/// An asteroid (variant `k`): a big lumpy rock with two smaller ones fused
/// to it, two craters, and violet crystals glowing in its cracks (Sector
/// X's haze). About 10 m across. 8 shapes.
pub fn asteroid(b: &mut Batch, f: &Affine, k: u32) {
    let k = (k % 4) as i32;
    let turn = Affine::rot_y(k * deg(70)).then(&Affine::rot_x(k * deg(40)));
    let g = f.then(&turn);
    put(
        b,
        &g,
        Mesh::Rock,
        V3::ZERO,
        m3(10 + k, 8, 9 - k / 2),
        ROCK,
        0,
    );
    put(
        b,
        &g,
        Mesh::Rock,
        m3(4, 2, -2),
        m3(5, 4, 5),
        shade(ROCK, 90),
        0,
    );
    put(
        b,
        &g,
        Mesh::Rock,
        m3(-3, -3, 2),
        m3(4, 3, 4),
        shade(ROCK, 110),
        0,
    );
    put(b, &g, Mesh::Hex, d3(-15, 38, 5), d3(26, 4, 26), CRATER, 0);
    put(b, &g, Mesh::Hex, d3(20, 30, 18), d3(16, 4, 16), CRATER, 0);
    for (x, y, z) in [(30, -20, 30), (-40, 0, 25), (0, -30, -38)] {
        put(b, &g, Mesh::Octa, d3(x, y, z), d3(12, 26, 12), CRYSTAL, ONE);
    }
}

// ---------------------------------------------------------------------------
// The black hole, the sun and the planet

/// A banana in the black hole's swirl: its middle at `p`, lying along unit
/// `t`, curving toward `-n` at both ends, `len` long. 3 shapes.
fn banana(b: &mut Batch, f: &Affine, p: V3, t: V3, n: V3, len: i32) {
    let bend = n.scale(len / 5);
    let (e1, e2) = (
        p.add(t.scale(len / 2)).sub(bend),
        p.sub(t.scale(len / 2)).sub(bend),
    );
    let w = len / 4;
    bar(b, f, Mesh::Taper, p, e1, w, BANANA, BANANA_GLOW);
    bar(b, f, Mesh::Taper, p, e2, w, BANANA, BANANA_GLOW);
    bar(
        b,
        f,
        Mesh::Cyl,
        e2,
        e2.add(e2.sub(p).with_len(len / 8)),
        w / 3,
        BANANA_TIP,
        BANANA_GLOW,
    );
}

/// The black hole at the finish, radius `r` (its dark ball's), in frame `f`
/// (the accretion disc lies in the frame's xz plane): a black ball, its
/// event horizon a ring of 24 bananas round it with two more rings of 18
/// tilted across it, two spiral arms of bananas falling in
/// (each shrinking as it nears the ring), and three bands of glowing
/// streaks swirling round, the inner ones faster. Everything turns with
/// the tick. 349 shapes (240 of them in 80 bananas) and 1 soft glow.
pub fn black_hole(b: &mut Batch, f: &Affine, r: i32, tick: u32) {
    let t = tick as i32;
    let core = r * 2;
    b.push(
        Mesh::Sphere,
        &f.then(&Affine::scale(v3(core, core, core))),
        rgb(0, 0, 0),
        SKY_GLOW,
    );
    let flat = |a: i32| (v3(sin(a), 0, fx::cos(a)), v3(fx::cos(a), 0, -sin(a)));
    // the event horizon: one ring in the disc's plane, two tilted across
    let rings: [(Affine, i32, i32, i32); 3] = [
        (Affine::IDENTITY, 24, ratio(114, 100), 120),
        (Affine::rot_x(deg(62)), 18, ratio(122, 100), -90),
        (
            Affine::rot_x(-deg(62)).then(&Affine::rot_y(deg(90))),
            18,
            ratio(122, 100),
            75,
        ),
    ];
    for (tilt, n, at, speed) in rings {
        let g = f.then(&tilt);
        let len = mul(r, ratio(24, 100));
        for k in 0..n {
            let a = k * TURN / n + t.wrapping_mul(speed);
            let (out, along) = flat(a);
            banana(b, &g, out.scale(mul(r, at)), along, out, len);
        }
    }
    // two arms of bananas spiralling in
    for arm in 0..2 {
        for k in 0..10 {
            let frac = (k * ONE / 10 + t.wrapping_mul(40)) & (ONE - 1);
            let rad = mul(r, ratio(115, 100) + mul(ratio(19, 10), ONE - frac));
            let a = arm * HALF + mul(frac, TURN) + t.wrapping_mul(60);
            let (out, along) = flat(a);
            let len = mul(
                mul(r, ratio(3, 10)),
                ONE * 3 / 5 + mul(ONE - frac, ONE * 2 / 5),
            );
            banana(
                b,
                f,
                out.scale(rad).add(v3(0, mul(sin(a * 3), r / 10), 0)),
                along,
                out,
                len,
            );
        }
    }
    // the accretion disc: white-hot, orange and violet bands of streaks
    let bands = [
        (28, ratio(150, 100), rgb(255, 244, 214), 260),
        (36, ratio(210, 100), rgb(255, 140, 40), 150),
        (44, ratio(290, 100), rgb(150, 70, 230), 80),
    ];
    for (n, at, c, speed) in bands {
        let rad = mul(r, at);
        let len = mul(rad, ratio(6, 10)) * 10 / n;
        for k in 0..n {
            let a = k * TURN / n + t.wrapping_mul(speed) + (k % 3) * 600;
            let (out, along) = flat(a);
            let p = out.scale(rad + (k % 4 - 2) * (r / 12));
            b.push(
                Mesh::Shard,
                &f.then(&axes(
                    p,
                    out.scale(len / 4),
                    along.scale(len),
                    V3::UP.scale(r / 30),
                )),
                c,
                SKY_GLOW,
            );
        }
    }
    // a faint violet glow round the whole
    let glow = mul(r, ratio(40, 10));
    b.push_halo(
        Mesh::Sphere,
        &f.then(&Affine::scale(v3(glow, glow / 3, glow))),
        rgb(120, 60, 200),
        ONE / 6,
    );
}

/// Unit directions for the sun's flares.
const FLARES: [(i32, i32, i32); 10] = [
    (100, 20, 0),
    (-90, 30, 30),
    (20, 100, -10),
    (-30, -100, 20),
    (40, -20, 100),
    (-10, 40, -100),
    (70, 70, 40),
    (-70, -50, -50),
    (60, -70, -40),
    (-60, 60, 60),
];

/// The sun far below the track, radius `r`, at `at` (world): a white-hot
/// ball with an orange corona and ten flares licking out, pulsing. Draw it
/// round the eye each frame inside the far plane (as the Ski Slope's
/// mountains are), as if infinitely far. 11 shapes and 2 soft glows.
pub fn sun(b: &mut Batch, at: V3, r: i32, tick: u32) {
    let t = tick as i32;
    let d = r * 2;
    b.push(
        Mesh::Geo,
        &Affine::translate(at).then(&Affine::scale(v3(d, d, d))),
        rgb(255, 246, 224),
        SKY_GLOW,
    );
    for (k, (x, y, z)) in FLARES.into_iter().enumerate() {
        let dir = v3(x * ONE / 100, y * ONE / 100, z * ONE / 100).norm();
        let reach = mul(
            r,
            ratio(130, 100) + mul(sin(t.wrapping_mul(70) + k as i32 * 9_000), ONE / 6),
        );
        b.push(
            Mesh::Spike,
            &rod(
                at.add(dir.scale(r * 9 / 10)),
                at.add(dir.scale(reach)),
                r / 4,
            ),
            rgb(255, 170, 70),
            SKY_GLOW,
        );
    }
    for (size, c, s) in [
        (ratio(26, 10), rgb(255, 200, 120), ONE * 3 / 4),
        (ratio(38, 10), rgb(255, 120, 40), ONE * 3 / 10),
    ] {
        let g = mul(r, size);
        b.push_halo(
            Mesh::Sphere,
            &Affine::translate(at).then(&Affine::scale(v3(g, g, g))),
            c,
            s,
        );
    }
}

/// A ringed gas giant, radius `r`, at `at` (world), tilted 18°: violet with
/// teal and lilac bands, a ring of 44 pale segments turning slowly, and a
/// small moon. A far backdrop: lit by the sun below, unfogged. Draw it
/// round the eye inside the far plane, as the sun. 50 shapes and 1 soft
/// glow.
pub fn planet(b: &mut Batch, at: V3, r: i32, tick: u32) {
    let f = Affine::translate(at).then(&Affine::rot_z(-deg(18)));
    let d = r * 2;
    put(
        b,
        &f,
        Mesh::Geo,
        V3::ZERO,
        v3(d, d, d),
        rgb(96, 78, 150),
        FAR_GLOW,
    );
    for (k, y) in [-50, -15, 30, 62].into_iter().enumerate() {
        let yk = ratio(y, 100);
        let w = mul(d, fx::sqrt(ONE - mul(yk, yk))) * 102 / 100;
        let c = if k % 2 == 0 {
            rgb(70, 150, 160)
        } else {
            rgb(170, 130, 190)
        };
        put(
            b,
            &f,
            Mesh::Cyl,
            v3(0, mul(r, yk), 0),
            v3(w, r / 9, w),
            c,
            FAR_GLOW,
        );
    }
    let spin = (tick as i32).wrapping_mul(6);
    let n = 44;
    let rad = mul(r, ratio(19, 10));
    let seg = mul(rad, ratio(6_283 * 106 / 100, 1_000)) / n;
    for k in 0..n {
        let a = k * TURN / n + spin;
        let (out, along) = (v3(sin(a), 0, fx::cos(a)), v3(fx::cos(a), 0, -sin(a)));
        let c = if k % 2 == 0 {
            rgb(214, 200, 230)
        } else {
            rgb(176, 160, 200)
        };
        b.push(
            Mesh::Cube,
            &f.then(&axes(
                out.scale(rad),
                out.scale(mul(r, ratio(7, 10))),
                V3::UP.scale(r / 60),
                along.scale(seg),
            )),
            c,
            FAR_GLOW,
        );
    }
    let moon = r / 2;
    put(
        b,
        &f,
        Mesh::Geo,
        v3(mul(r, ratio(28, 10)), r / 2, mul(r, ratio(12, 10))),
        v3(moon, moon, moon),
        rgb(150, 146, 160),
        FAR_GLOW,
    );
    let halo = mul(r, ratio(23, 10));
    b.push_halo(
        Mesh::Sphere,
        &Affine::translate(at).then(&Affine::scale(v3(halo, halo, halo))),
        rgb(120, 110, 220),
        ONE / 4,
    );
}

/// A starfield round the eye: `n` small glowing points scattered over a
/// ball 520 m out (inside the far plane), the same every frame.
pub fn stars(b: &mut Batch, eye: V3, n: u32) {
    let mut dice = fx::Dice(0x5EC7_0A9E);
    for _ in 0..n {
        let d = v3(
            dice.range(-ONE, ONE),
            dice.range(-ONE, ONE),
            dice.range(-ONE, ONE),
        );
        if d.len() < ONE / 8 {
            continue;
        }
        let s = cm(60) + dice.range(0, cm(160));
        let c = match dice.below(4) {
            0 => rgb(255, 226, 190),
            1 => rgb(180, 200, 255),
            _ => rgb(236, 240, 255),
        };
        b.push(
            Mesh::Octa,
            &Affine::translate(eye.add(d.with_len(int(520)))).then(&Affine::scale(v3(s, s, s))),
            c,
            SKY_GLOW,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{DRAW_VALUES, DYNAMIC_BUFFER, INSTANCE_VALUES};

    /// One drawn shape: its three axes, middle, colour and glow.
    struct Shape {
        axes: [V3; 3],
        at: V3,
        colour: Rgb,
        glow: i32,
    }

    fn drawn(b: Batch) -> Vec<Shape> {
        let (values, draws) = b.finish(DYNAMIC_BUFFER);
        let mut out = Vec::new();
        for d in draws.chunks(DRAW_VALUES) {
            let first = d[2] as usize / 4 / INSTANCE_VALUES;
            for i in first..first + d[3] as usize {
                let v = &values[i * INSTANCE_VALUES..(i + 1) * INSTANCE_VALUES];
                out.push(Shape {
                    axes: [
                        v3(v[0], v[4], v[8]),
                        v3(v[1], v[5], v[9]),
                        v3(v[2], v[6], v[10]),
                    ],
                    at: v3(v[3], v[7], v[11]),
                    colour: [v[12], v[13], v[14]],
                    glow: v[15],
                });
            }
        }
        out
    }

    /// Shapes and soft glows a builder draws.
    fn count(draw: impl Fn(&mut Batch)) -> (usize, usize) {
        let mut b = Batch::new();
        draw(&mut b);
        let halos = b.take_halos().count();
        (b.count(), halos)
    }

    fn draw(f: impl Fn(&mut Batch)) -> Vec<Shape> {
        let mut b = Batch::new();
        f(&mut b);
        drawn(b)
    }

    fn values(f: impl Fn(&mut Batch)) -> Vec<i32> {
        let mut b = Batch::new();
        f(&mut b);
        let halos = b.take_halos();
        let mut v = b.finish(DYNAMIC_BUFFER).0;
        v.extend(halos.finish(DYNAMIC_BUFFER).0);
        v
    }

    /// The box round every shape's corners: (min, max).
    fn bounds(s: &[Shape]) -> (V3, V3) {
        let mut lo = v3(i32::MAX, i32::MAX, i32::MAX);
        let mut hi = v3(i32::MIN, i32::MIN, i32::MIN);
        for x in s {
            for k in 0..8 {
                let sign = |bit: i32| if k & bit == 0 { ONE / 2 } else { -ONE / 2 };
                let p =
                    x.at.add(x.axes[0].scale(sign(1)))
                        .add(x.axes[1].scale(sign(2)))
                        .add(x.axes[2].scale(sign(4)));
                lo = v3(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
                hi = v3(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
            }
        }
        (lo, hi)
    }

    /// Whether `p` is inside a shape's unit box (its axes are square to
    /// each other: every builder's are a turn times a scale).
    fn inside(s: &Shape, p: V3) -> bool {
        let d = p.sub(s.at);
        s.axes.iter().all(|a| {
            let len2 = a.dot64(*a);
            len2 > 0 && (d.dot64(*a) as i128 * 2).abs() < len2 as i128
        })
    }

    fn in_range(n: usize, lo: usize, hi: usize, what: &str) {
        assert!((lo..=hi).contains(&n), "{what}: {n} shapes");
    }

    #[test]
    fn the_mothership_is_long_white_and_its_engines_pulse() {
        let f = Affine::IDENTITY;
        let (n, halos) = count(|b| mothership(b, &f, 0));
        in_range(n, 60, 90, "mothership");
        assert_eq!(halos, 4);
        let s = draw(|b| mothership(b, &f, 0));
        let (lo, hi) = bounds(&s);
        let len = hi.z - lo.z;
        assert!((len - MOTHERSHIP_LENGTH).abs() < int(6), "{}", len / ONE);
        let wide = hi.x - lo.x;
        assert!(wide > int(130) && wide < int(150), "{}", wide / ONE);
        assert!(s.iter().filter(|x| x.colour == HULL).count() >= 8);
        assert!(s.iter().any(|x| x.colour == ACCENT));
        assert_ne!(
            values(|b| mothership(b, &f, 0)),
            values(|b| mothership(b, &f, 7))
        );
    }

    #[test]
    fn the_wreck_is_broken_dark_and_open_through_its_passage() {
        let f = Affine::IDENTITY;
        let (n, halos) = count(|b| wreck(b, &f, 0, WRECK_GAP));
        in_range(n, 100, 160, "wreck");
        assert_eq!(halos, 2);
        let s = draw(|b| wreck(b, &f, 0, WRECK_GAP));
        // its engines are out and it is scorched
        assert!(!s.iter().any(|x| x.colour == ENGINE_CORE));
        assert!(s.iter().filter(|x| x.colour == SCORCH).count() >= 10);
        // a clear way through: nothing in the opening, mouth to mouth
        let p = wreck_passage(&f, WRECK_GAP);
        assert!((p.width - int(10)).abs() < ONE / 10);
        assert!((p.height - int(7)).abs() < ONE / 10);
        assert!((p.length - int(HULL_W) - cm(120)).abs() < ONE / 10);
        let (_, bow) = halves(&f);
        let up = bow.apply_dir(v3(0, ONE, 0));
        let along = bow.apply_dir(v3(0, 0, ONE));
        for i in 0..=12 {
            let c = p.entry.lerp(p.exit, i * ONE / 12);
            for (a, b) in [(-4, -4), (-4, 4), (4, -4), (4, 4), (0, 0)] {
                let q = c
                    .add(along.scale(mul(p.width, ratio(a, 10))))
                    .add(up.scale(mul(p.height, ratio(b, 10))));
                assert!(!s.iter().any(|x| inside(x, q)), "blocked at {i} {a} {b}");
            }
        }
        // ... and the hull is solid round it: a passage height up is blocked
        let blocked = p.middle.add(up.scale(p.height));
        assert!(s.iter().any(|x| inside(x, blocked)));
        // the embers and alarms flicker
        assert_ne!(
            values(|b| wreck(b, &f, 0, WRECK_GAP)),
            values(|b| wreck(b, &f, 41, WRECK_GAP))
        );
    }

    #[test]
    fn the_wreck_passage_follows_its_gap_and_frame() {
        let f = pose(m3(100, 20, -50), deg(90), 0, 0, ONE * 2);
        let p = wreck_passage(&f, (int(6), int(30)));
        assert!((p.width - int(12)).abs() < ONE / 5);
        assert!((p.height - int(36)).abs() < ONE / 5, "clamped to 18 m");
        assert!(p.middle.dist(m3(100, 20, -50)) > int(60));
    }

    fn grunt(gait: Gait) -> Mech {
        Mech {
            kind: MechKind::Grunt,
            gait,
            phase: 0,
        }
    }

    #[test]
    fn a_mech_stands_13_m_and_walks_with_the_tick() {
        let f = Affine::IDENTITY;
        for gait in [Gait::March, Gait::Drift] {
            let m = grunt(gait);
            let (n, _) = count(|b| mech(b, &f, &m, 0));
            in_range(n, 35, 55, "grunt");
            assert_ne!(
                values(|b| mech(b, &f, &m, 0)),
                values(|b| mech(b, &f, &m, 30))
            );
            let a0 = mech_anchors(&f, &m, 0);
            let a1 = mech_anchors(&f, &m, 60);
            assert!(a0.hands[0].dist(a1.hands[0]) > cm(20), "{gait:?}");
        }
        let (lo, hi) = bounds(&draw(|b| mech(b, &f, &grunt(Gait::March), 0)));
        let tall = hi.y - lo.y;
        assert!((tall - MECH_HEIGHT).abs() < int(1), "{}", tall / ONE);
        // the brute: half as big again, spiked
        let brute = Mech {
            kind: MechKind::Brute,
            ..grunt(Gait::March)
        };
        let (n, _) = count(|b| mech(b, &f, &brute, 0));
        in_range(n, 40, 60, "brute");
        let (blo, bhi) = bounds(&draw(|b| mech(b, &f, &brute, 0)));
        assert!(bhi.y - blo.y > tall * 7 / 5);
    }

    #[test]
    fn a_mechs_vine_anchors_sit_on_its_head_and_fists() {
        let f = pose(m3(5, 40, 9), deg(30), 0, 0, ONE);
        let m = grunt(Gait::March);
        let a = mech_anchors(&f, &m, 77);
        let s = draw(|b| mech(b, &f, &m, 77));
        let (_, hi) = bounds(&s);
        // the head's anchor is at the crown; each fist's under its fist
        assert!((a.head.y - hi.y).abs() < int(1), "{} {}", a.head.y, hi.y);
        for h in a.hands {
            assert!(s.iter().any(|x| x.at.dist(h) < int(1)), "a fist over {h:?}");
        }
        assert!(a.hands[0].dist(a.hands[1]) > int(6));
        assert!(a.elbows.iter().zip(&a.shoulders).all(|(e, s)| e.y < s.y));
    }

    #[test]
    fn fighters_read_from_afar_and_dogfight_with_lasers() {
        let f = Affine::IDENTITY;
        in_range(count(|b| ape_wing(b, &f, 0)).0, 14, 20, "ape-wing");
        in_range(count(|b| enemy_fighter(b, &f, 0)).0, 5, 10, "enemy");
        let s = draw(|b| ape_wing(b, &f, 0));
        let (lo, hi) = bounds(&s);
        assert!(hi.z - lo.z > int(9) && hi.x - lo.x > int(10));
        assert!(s.iter().any(|x| x.glow >= SKY_GLOW), "glowing engines");
        // the dogfight: ships, bolts in flight, shots that hit
        let (n, _) = count(|b| dogfight(b, &f, 0));
        in_range(n, 50, 80, "dogfight");
        let (mut good, mut bad, mut hits) = (false, false, false);
        for t in (0..400).step_by(5) {
            let s = draw(|b| dogfight(b, &f, t));
            good |= s.iter().any(|x| x.colour == LASER_GOOD);
            bad |= s.iter().any(|x| x.colour == LASER_BAD);
            hits |= s.iter().any(|x| x.colour == BLAST);
        }
        assert!(good && bad && hits);
        assert_ne!(
            values(|b| dogfight(b, &f, 0)),
            values(|b| dogfight(b, &f, 9))
        );
        // two ape-wings and three enemies are always flying
        let s = draw(|b| dogfight(b, &f, 123));
        assert_eq!(s.iter().filter(|x| x.colour == GLASS).count(), 2);
        assert_eq!(s.iter().filter(|x| x.colour == ENEMY_HULL).count(), 3);
    }

    #[test]
    fn capital_ships_trade_fire() {
        let f = Affine::IDENTITY;
        let (n, _) = count(|b| capital_battle(b, &f, 0));
        in_range(n, 25, 45, "battle");
        let mut colours = Vec::new();
        for t in (0..300).step_by(7) {
            for x in draw(|b| capital_battle(b, &f, t)) {
                colours.push(x.colour);
            }
        }
        for c in [LASER_GOOD, LASER_BAD, BLAST] {
            assert!(colours.contains(&c));
        }
        assert_ne!(
            values(|b| capital_battle(b, &f, 0)),
            values(|b| capital_battle(b, &f, 20))
        );
    }

    #[test]
    fn the_debris_kit_is_cheap_and_tumbles() {
        let f = Affine::IDENTITY;
        for k in 0..3 {
            in_range(count(|b| hull_chunk(b, &f, 0, k)).0, 10, 14, "chunk");
            in_range(count(|b| asteroid(b, &f, k)).0, 8, 8, "asteroid");
        }
        assert_eq!(count(|b| girder(b, &f, 24)).0, 4 + 1 + 2 * 8 + 1);
        let (lo, hi) = bounds(&draw(|b| girder(b, &f, 24)));
        assert!((hi.z - lo.z - int(24)).abs() < int(1));
        assert_ne!(tumble(0, 1), tumble(60, 1));
        assert_ne!(tumble(60, 1), tumble(60, 2));
    }

    #[test]
    fn the_black_holes_event_horizon_is_made_of_bananas() {
        let f = Affine::IDENTITY;
        let r = int(20);
        let (n, halos) = count(|b| black_hole(b, &f, r, 0));
        in_range(n, 300, 400, "black hole");
        assert_eq!(halos, 1);
        let s = draw(|b| black_hole(b, &f, r, 0));
        let bananas: Vec<&Shape> = s.iter().filter(|x| x.colour == BANANA).collect();
        // 80 bananas, two shapes each, plus a stalk
        assert_eq!(bananas.len(), 160);
        assert_eq!(s.iter().filter(|x| x.colour == BANANA_TIP).count(), 80);
        // the horizon: 60 of them in a shell just outside the dark ball
        let on_horizon = bananas
            .iter()
            .filter(|x| {
                let d = x.at.len();
                d > r && d < mul(r, ratio(125, 100))
            })
            .count();
        assert!(on_horizon >= 120, "{on_horizon}");
        // the ball is black
        assert!(s
            .iter()
            .any(|x| x.colour == [0, 0, 0] && x.axes[0].len() == r * 2));
        // and it swirls
        let moved = draw(|b| black_hole(b, &f, r, 30));
        assert!(s
            .iter()
            .zip(&moved)
            .any(|(a, b)| a.colour == BANANA && a.at != b.at));
    }

    #[test]
    fn the_sun_planet_and_stars_are_backdrops_that_glow() {
        let (n, halos) = count(|b| sun(b, V3::ZERO, int(40), 0));
        assert_eq!((n, halos), (11, 2));
        let (n, halos) = count(|b| planet(b, V3::ZERO, int(40), 0));
        assert_eq!((n, halos), (50, 1));
        let s = draw(|b| planet(b, V3::ZERO, int(40), 0));
        assert!(s.iter().all(|x| x.glow == FAR_GLOW), "lit, unfogged");
        let s = draw(|b| sun(b, V3::ZERO, int(40), 0));
        assert!(s.iter().all(|x| x.glow == SKY_GLOW));
        assert_ne!(
            values(|b| sun(b, V3::ZERO, int(40), 0)),
            values(|b| sun(b, V3::ZERO, int(40), 50))
        );
        let s = draw(|b| stars(b, m3(10, 0, 0), 300));
        assert!(s.len() > 280);
        assert!(s
            .iter()
            .all(|x| (x.at.dist(m3(10, 0, 0)) - int(520)).abs() < ONE));
        // the light comes from below
        assert!(look().light_dir.y < -ONE / 2);
    }
}
