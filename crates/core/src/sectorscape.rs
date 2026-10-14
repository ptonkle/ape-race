//! Drawing Sector Ape (2026-10-13, `briefs/space-map.md`, the art agent's
//! SECOND-ORDER SA rows).
//!
//! **The light first** (MAPS "Art lessons"): there is no time of day. A
//! blazing orange-white sun hangs far below the track, a little ahead and
//! to the right of the way the course runs, and lights everything from
//! underneath: hull bellies glow warm orange-white, their tops fall into
//! a cool indigo from the far stars and the gas giant, and every outline
//! catches a blue-violet edge light (`Look.rim`, `scene.frag`: SECOND-ORDER
//! SM4), so a hull's top reads against black space. The sky is deep
//! black-blue with stars; a violet ringed gas giant hangs high off to the
//! left; the purple and green haze of Sector X drifts far off round the
//! sides. Nothing is fogged much: space is clear, and only far things sink
//! into a dark violet haze. No weather. The glows are neon: engines in
//! ice blue, lasers green and red, windows pale cyan, the vines (energy
//! tethers) mint.
//!
//! A palette no other map uses: black-blue, sun-orange from below, violet
//! and mint. Drawn from the course's `SectorLayout` (`sector.rs`): the
//! decks and bodies the vines hang from, faint gates showing the boundary,
//! drifting wreckage and asteroids, the wreck of the mothership, the door's
//! hulk and its door, the mech armada, the mothership's pass overhead,
//! battles far off, and the banana black hole at the finish; and round the
//! eye each frame the sun below, the planet, the stars and the haze.
//!
//! The scene (`scene.rs`) calls in here for a course with a `SectorLayout`;
//! the other maps' drawing does not change.

use crate::course::{Course, Platform, STEP};
use crate::fx::{self, cm, deg, int, mul, ratio, sin, Dice, ONE, TURN};
use crate::geom::{facing, right_of, v3, yaw_of, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, placed, rgb, rod, shade, Batch, Camera, Rgb};
use crate::scene::{self, Look, Scenery};
use crate::sector::{Body, Model, SectorLayout, SetPiece};
use crate::spacecraft as sc;

// ---- the light: a sun far below ----

/// Toward the sun: its bearing from the way the course runs (start to
/// finish) and its height (below the horizon). Ahead and to the right, so
/// its glow rises over the bottom of the chase view, and steep, so the
/// light comes from underneath.
const SUN_BEARING: i32 = deg(28);
const SUN_LOW: i32 = -deg(56);

/// The sun's light: a blazing orange-white, brighter than white.
const SUN_LIGHT: Rgb = rgb(300, 236, 178);
/// The ambient from above (the tops of things): a cool, dim indigo.
const SKY: Rgb = rgb(46, 50, 104);
/// The ambient from below (the bellies): the sun's own orange.
const BOUNCE: Rgb = rgb(206, 112, 52);
/// The edge light round every outline: a blue-violet.
const RIM: Rgb = rgb(78, 90, 210);
/// The clear colour behind everything, and what far things sink into: a
/// deep violet-black.
pub const FOG: Rgb = rgb(9, 9, 26);

/// Sector Ape: no day, a sun below.
pub fn look() -> Look {
    Look {
        sky: SKY,
        fog: FOG,
        // clear space: far things sink into the haze only at the edge of
        // what is drawn (`REACH`)
        fog_range: (int(230), int(560)),
        light_dir: facing(SUN_BEARING, SUN_LOW),
        light: SUN_LIGHT,
        ground: BOUNCE,
        rim: RIM,
    }
}

/// Sector Ape's look for its course: the sun stands below, ahead and to
/// the right of the way the course runs.
pub fn look_for(c: &Course) -> Look {
    Look {
        light_dir: sun_dir(c),
        ..look()
    }
}

/// Toward the sun.
pub fn sun_dir(c: &Course) -> V3 {
    facing(course_yaw(c) + SUN_BEARING, SUN_LOW)
}

/// The heading of the course as a whole, start to finish (flat).
fn course_yaw(c: &Course) -> i32 {
    let last = c.path[c.path.len() - 1];
    yaw_of(last.sub(c.path[0]).flat())
}

// ---- the palette ----

/// The vines: energy tethers in a bright mint, apart from everything in
/// space (black-blue, orange, violet), from the in-reach yellow, the
/// catch-up coral and the item vines' violet crystals.
pub const TETHER: Rgb = rgb(80, 255, 176);
/// The colourblind mode's tethers: brighter still.
pub const TETHER_VIVID: Rgb = rgb(150, 255, 210);
/// How many times a tether's width its glow is across.
pub const TETHER_HALO: i32 = 4;
/// The item vines' marker: violet crystals (the asteroids' own).
pub const CRYSTAL: Rgb = rgb(186, 110, 255);
const CRYSTAL_DULL: Rgb = rgb(70, 66, 86);

const PLATE: Rgb = rgb(70, 76, 92);
const PLATE_LIGHT: Rgb = rgb(112, 120, 138);
const PLATE_DARK: Rgb = rgb(40, 44, 56);
const KEEL: Rgb = rgb(150, 156, 170);
const HAZARD: Rgb = rgb(240, 180, 30);
const NEON: Rgb = rgb(90, 200, 255);
const NEON_WARM: Rgb = rgb(255, 120, 60);
const ENGINE: Rgb = rgb(120, 200, 255);
const GATE: Rgb = rgb(70, 110, 200);
const SHIELD: Rgb = rgb(140, 220, 255);
const HAZE_VIOLET: Rgb = rgb(140, 64, 230);
const HAZE_GREEN: Rgb = rgb(40, 190, 130);
const HAZE_BLUE: Rgb = rgb(60, 90, 220);

/// A glow drawn full colour and unfogged (`scene.frag`: over 1.5 is sky).
const SKY_GLOW: i32 = ONE * 2;
/// A soft glow in the sky round the eye (`scene.frag`: a halo strength of
/// 3 or more adds the part over 3, unfogged).
const SKY_HALO: i32 = ONE * 3;

// ---- the backdrop: round the eye each frame ----

/// How far from the eye the sun, the planet, the stars and the haze are
/// drawn (inside the far plane, `render::FAR`).
const SUN_AT: i32 = int(470);
const SUN_R: i32 = int(64);
const PLANET_AT: i32 = int(470);
const PLANET_R: i32 = int(46);
/// The planet: off to the left of the way the course runs, high.
const PLANET_BEARING: i32 = -deg(58);
const PLANET_HIGH: i32 = deg(17);
const STAR_COUNT: u32 = 650;
const HAZE_AT: i32 = int(540);

/// Round the eye: the sun far below, the gas giant, the stars, and the
/// haze of Sector X, all at a fixed distance from the eye, as if
/// infinitely far.
pub fn backdrop(b: &mut Batch, c: &Course, eye: V3, tick: u32) {
    let base = course_yaw(c);
    sc::stars(b, eye, STAR_COUNT);
    sun(b, eye.add(sun_dir(c).scale(SUN_AT)), SUN_R, tick);
    sc::planet(
        b,
        eye.add(facing(base + PLANET_BEARING, PLANET_HIGH).scale(PLANET_AT)),
        PLANET_R,
        tick,
    );
    haze(b, eye, base);
}

/// The sun: a white-hot ball, pulsing flares, and rings of orange corona
/// spreading far round it (sky halos, unfogged).
fn sun(b: &mut Batch, at: V3, r: i32, tick: u32) {
    let t = tick as i32;
    let d = r * 2;
    b.push(
        Mesh::Geo,
        &Affine::translate(at).then(&Affine::scale(v3(d, d, d))),
        rgb(255, 250, 236),
        SKY_GLOW,
    );
    let mut dice = Dice(0x5_0A2);
    for k in 0..14 {
        let dir = v3(
            dice.range(-ONE, ONE),
            dice.range(-ONE, ONE),
            dice.range(-ONE, ONE),
        )
        .norm();
        if dir == V3::ZERO {
            continue;
        }
        let reach = mul(
            r,
            ratio(128, 100) + mul(sin(t.wrapping_mul(50) + k * 9_000), ONE / 7),
        );
        b.push(
            Mesh::Spike,
            &rod(
                at.add(dir.scale(r * 9 / 10)),
                at.add(dir.scale(reach)),
                r / 3,
            ),
            rgb(255, 196, 110),
            SKY_GLOW,
        );
    }
    for (size, c, s) in [
        (ratio(15, 10), rgb(255, 236, 190), ONE * 7 / 10),
        (ratio(24, 10), rgb(255, 170, 80), ONE * 2 / 5),
        (ratio(38, 10), rgb(255, 110, 40), ONE / 5),
        (ratio(56, 10), rgb(200, 70, 60), ONE / 10),
    ] {
        let g = mul(r, size) * 2;
        b.push_halo(Mesh::Sphere, &placed(at, 0, v3(g, g, g)), c, SKY_HALO + s);
    }
}

/// The haze of Sector X: broad soft clouds of violet, green and blue far
/// round the sides and low down, added over the stars.
fn haze(b: &mut Batch, eye: V3, base: i32) {
    let mut dice = Dice(0x5EC7_4A2E);
    for k in 0..16 {
        // a band rising from low on the right, round behind, to the left
        let yaw = base + deg(20) + k * deg(19) + dice.range(-deg(8), deg(8));
        let high = dice.range(-deg(26), deg(30));
        let dir = facing(yaw, high);
        let at = eye.add(dir.scale(HAZE_AT));
        let r = int(dice.range(80, 170));
        let c = match k % 3 {
            0 => HAZE_VIOLET,
            1 => HAZE_GREEN,
            _ => HAZE_BLUE,
        };
        let s = ONE / 14 + dice.range(0, ONE / 12);
        // a cloud stretched across: wider than tall, tilted
        let x = right_of(dir);
        let y = dir.cross(x).norm();
        let tilt = dice.range(-deg(30), deg(30));
        let (cx, sx) = (fx::cos(tilt), sin(tilt));
        let ax = x.scale(cx).add(y.scale(sx));
        let ay = y.scale(cx).sub(x.scale(sx));
        let f = Affine {
            c0: ax.scale(r * 3),
            c1: ay.scale(r * 6 / 5),
            c2: dir.scale(r * 2),
            t: at,
        };
        b.push_halo(Mesh::Sphere, &f, c, SKY_HALO + s);
    }
}

// ---- the static scenery ----

/// Chunks whose middle is further than this from the eye are not drawn
/// (the other maps: `scene::DRAW_DISTANCE`, 260 m): space is clear, so
/// things are seen from further.
pub const REACH: i32 = int(420);

/// The output: shapes sorted into the course's chunks.
struct Sink<'a> {
    c: &'a Course,
    n: usize,
    batches: Vec<Batch>,
}

impl Sink<'_> {
    fn at(&mut self, p: V3) -> &mut Batch {
        let k = scene::chunk_of(self.c, p, self.n);
        &mut self.batches[k]
    }
}

/// Builds Sector Ape's scenery, chunked. There is no ground.
pub fn scenery(c: &Course, s: &SectorLayout) -> Scenery {
    build(c, s).0
}

/// How many scenery shapes each part takes (for
/// `examples/scenery_cost.rs`).
pub fn parts(c: &Course) -> Vec<(&'static str, usize)> {
    match &c.sector {
        Some(s) => build(c, s).1,
        None => Vec::new(),
    }
}

fn build(c: &Course, s: &SectorLayout) -> (Scenery, Vec<(&'static str, usize)>) {
    let n = scene::chunk_count(c);
    let mut sink = Sink {
        c,
        n,
        batches: (0..n).map(|_| Batch::new()).collect(),
    };
    let mut parts = Vec::new();
    let mut done = 0;
    let mut tally = |s: &Sink, name: &'static str| {
        let now: usize = s.batches.iter().map(|b| b.count()).sum();
        parts.push((name, now - done));
        done = now;
    };
    for (k, pl) in c.platforms.iter().enumerate() {
        deck(sink.at(pl.centre), pl, k == 0);
    }
    tally(&sink, "decks");
    for (k, body) in s.bodies.iter().enumerate() {
        self::body(sink.at(body.frame.t), body, k);
    }
    tally(&sink, "bodies");
    gates(&mut sink, c, s);
    tally(&sink, "gates");
    debris_field(&mut sink, c, s);
    tally(&sink, "debris");
    let mut scenery = scene::assemble(c, sink.batches, Vec::new(), Vec::new());
    scenery.reach = REACH;
    (scenery, parts)
}

/// A ship's deck an ape can stand on (a platform): dark plating on a keel
/// lit orange from below, glowing edge strips, hazard chevrons at its
/// ends, and engine pods holding it in place. The start deck has a launch
/// gantry over its front.
fn deck(b: &mut Batch, pl: &Platform, start: bool) {
    let fwd = pl.fwd.flat().norm();
    let right = right_of(fwd);
    let top = v3(pl.centre.x, pl.top, pl.centre.z);
    let frame = Affine {
        c0: right,
        c1: V3::UP,
        c2: fwd,
        t: top,
    };
    let (hw, hl) = (pl.half_width, pl.half_len);
    let part = |b: &mut Batch, mesh: Mesh, at: V3, size: V3, c: Rgb, glow: i32| {
        b.push(
            mesh,
            &frame
                .then(&Affine::translate(at))
                .then(&Affine::scale(size)),
            c,
            glow,
        );
    };
    // the plating: tiles in two shades
    let tiles_x = (hw * 2 / int(4)).clamp(1, 8);
    let tiles_z = (hl * 2 / int(4)).clamp(1, 12);
    let (tw, tl) = (hw * 2 / tiles_x, hl * 2 / tiles_z);
    for i in 0..tiles_x {
        for j in 0..tiles_z {
            let x = -hw + tw * i + tw / 2;
            let z = -hl + tl * j + tl / 2;
            let c = if (i + j) % 2 == 0 {
                PLATE
            } else {
                shade(PLATE, 88)
            };
            part(
                b,
                Mesh::Cube,
                v3(x, -cm(30), z),
                v3(tw - cm(8), cm(60), tl - cm(8)),
                c,
                0,
            );
        }
    }
    // the slab under the tiles, and the keel below, tapering
    part(
        b,
        Mesh::Bevel,
        v3(0, -cm(110), 0),
        v3(hw * 2 + cm(40), cm(120), hl * 2 + cm(40)),
        PLATE_DARK,
        0,
    );
    part(
        b,
        Mesh::Wedge,
        v3(0, -cm(330), -hl / 4),
        v3(hw * 6 / 5, cm(330), hl * 3 / 2),
        KEEL,
        0,
    );
    part(
        b,
        Mesh::Bevel,
        v3(0, -cm(420), hl / 2),
        v3(hw, cm(200), hl / 2),
        PLATE_LIGHT,
        0,
    );
    // glowing edge strips down both sides
    for sx in [-1, 1] {
        part(
            b,
            Mesh::Cube,
            v3(sx * (hw + cm(10)), -cm(10), 0),
            v3(cm(22), cm(22), hl * 2),
            NEON,
            ONE,
        );
    }
    // hazard chevrons across both ends
    for sz in [-1, 1] {
        for k in 0..((hw * 2 / int(2)).clamp(1, 10)) {
            let x = -hw + int(1) + k * int(2);
            part(
                b,
                Mesh::Cube,
                v3(x, cm(2), sz * (hl - cm(60))),
                v3(cm(90), cm(6), cm(60)),
                if k % 2 == 0 { HAZARD } else { PLATE_DARK },
                0,
            );
        }
    }
    // engine pods at the back corners, glowing backward
    for sx in [-1, 1] {
        let at = v3(sx * (hw + cm(60)), -cm(160), -hl + cm(80));
        part(
            b,
            Mesh::Cyl,
            at,
            v3(cm(150), cm(150), cm(240)),
            PLATE_LIGHT,
            0,
        );
        part(
            b,
            Mesh::Cyl,
            at.add(v3(0, 0, -cm(130))),
            v3(cm(110), cm(110), cm(30)),
            ENGINE,
            SKY_GLOW,
        );
    }
    if start {
        // a launch gantry over the front: two posts and a lit bar
        for sx in [-1, 1] {
            part(
                b,
                Mesh::Cube,
                v3(sx * (hw - cm(30)), int(3), hl - cm(40)),
                v3(cm(50), int(6), cm(50)),
                PLATE_LIGHT,
                0,
            );
        }
        part(
            b,
            Mesh::Cube,
            v3(0, int(6), hl - cm(40)),
            v3(hw * 2, cm(60), cm(60)),
            PLATE_LIGHT,
            0,
        );
        for k in 0..5 {
            part(
                b,
                Mesh::Cube,
                v3(-hw + (k * 2 + 1) * hw / 5, cm(560), hl - cm(40)),
                v3(cm(70), cm(30), cm(70)),
                if k % 2 == 0 { NEON } else { HAZARD },
                SKY_GLOW,
            );
        }
    }
}

/// A body the vines hang from, by its model.
fn body(b: &mut Batch, body: &Body, k: usize) {
    let f = &body.frame;
    match body.model {
        Model::Chunk(v) => sc::hull_chunk(b, f, 0, v),
        Model::Girder(len) => sc::girder(b, f, len / ONE),
        Model::Asteroid(v) => sc::asteroid(b, f, v),
        Model::Wing => sc::ape_wing(b, f, 0),
        Model::Deck => slab_body(b, body),
        Model::Hulk => slab_body(b, body),
        // the walls round the wreck's passage and the hulk's corridor:
        // drawn as part of those
        Model::Hidden => {}
    }
    let _ = k;
}

/// A box-shaped body drawn as a slab of hull plating its own size.
fn slab_body(b: &mut Batch, body: &Body) {
    let f = body.frame.at(V3::ZERO);
    let h = body.half;
    // the frame may carry a scale: undo it for the box (its size is in
    // world metres)
    let unit = Affine {
        c0: f.c0.norm(),
        c1: f.c1.norm(),
        c2: f.c2.norm(),
        t: body.frame.t,
    };
    b.push(
        Mesh::Bevel,
        &unit.then(&Affine::scale(v3(h.x * 2, h.y * 2, h.z * 2))),
        PLATE,
        0,
    );
    b.push(
        Mesh::Cube,
        &unit
            .then(&Affine::translate(v3(0, h.y, 0)))
            .then(&Affine::scale(v3(h.x * 2 - cm(60), cm(10), cm(30)))),
        NEON,
        ONE,
    );
}

/// The boundary shown: every `GATE_EVERY` metres along the course, a frame
/// of faint glowing dashes on the edge an ape is reset past (sides, below
/// and above). Far apart and dim, so they read as space lanes and never
/// compete with the vines.
const GATE_EVERY: i32 = int(90);

fn gates(s: &mut Sink, c: &Course, l: &SectorLayout) {
    let bd = l.boundary;
    let mut at = int(60);
    while at < c.length {
        let p = c.point(at);
        let fwd = c.heading_at(at);
        let right = right_of(fwd);
        let b = s.at(p);
        let corner = |sx: i32, y: i32| p.add(right.scale(sx * bd.side)).add(v3(0, y, 0));
        let edges = [
            (corner(-1, bd.below), corner(1, bd.below)),
            (corner(1, bd.below), corner(1, bd.above)),
            (corner(1, bd.above), corner(-1, bd.above)),
            (corner(-1, bd.above), corner(-1, bd.below)),
        ];
        for (a, z) in edges {
            let len = z.sub(a).len();
            let dashes = (len / int(9)).max(1);
            for k in 0..dashes {
                let t0 = fx::div(k * 2, dashes * 2);
                let t1 = fx::div(k * 2 + 1, dashes * 2);
                b.push(
                    Mesh::Cube,
                    &rod(a.lerp(z, t0), a.lerp(z, t1), cm(30)),
                    GATE,
                    ONE,
                );
            }
        }
        // a corner light at each corner
        for (sx, y) in [(-1, bd.below), (1, bd.below), (1, bd.above), (-1, bd.above)] {
            b.push(
                Mesh::Octa,
                &placed(corner(sx, y), 0, v3(cm(120), cm(120), cm(120))),
                NEON_WARM,
                SKY_GLOW,
            );
        }
        at += GATE_EVERY;
    }
}

/// Still wreckage and rocks scattered outside the boundary, all the way
/// along: hull chunks, girders and asteroids, far enough out never to be
/// touched, so the space round the track is never empty.
fn debris_field(s: &mut Sink, c: &Course, l: &SectorLayout) {
    let bd = l.boundary;
    let mut dice = Dice(0x5EC7_DEB2);
    let mut at = int(20);
    while at < c.length + int(60) {
        let p = c.point(at.min(c.length));
        let right = right_of(c.heading_at(at));
        for _ in 0..3 {
            let side = if dice.below(2) == 0 { 1 } else { -1 };
            let out = bd.side + int(dice.range(25, 140));
            let y = int(dice.range(-110, 90));
            let q = p
                .add(right.scale(side * out))
                .add(v3(0, y, int(dice.range(-20, 20))));
            let f = sc::pose(
                q,
                dice.range(0, TURN),
                dice.range(-deg(60), deg(60)),
                dice.range(0, TURN),
                ONE + dice.range(0, ONE),
            );
            let k = dice.below(16);
            let b = s.at(q);
            match k {
                0..=8 => sc::asteroid(b, &f, k),
                9..=12 => sc::hull_chunk(b, &f, 0, k),
                _ => sc::girder(b, &f, 12 + (k as i32 - 13) * 6),
            }
        }
        at += int(dice.range(28, 46));
    }
}

// ---- what moves ----

/// What Sector Ape draws each frame: the backdrop round the eye, the
/// shields of bouncy bodies, the mechs, the door, the drifting wreckage,
/// the set pieces and the black hole.
pub fn dynamic(b: &mut Batch, c: &Course, s: &SectorLayout, eye: V3, tick: u32, race_tick: u32) {
    backdrop(b, c, eye, tick);
    for (k, body) in s.bodies.iter().enumerate() {
        if body.bouncy && body.frame.t.dist(eye) < int(220) {
            shield(b, body, tick, k);
        }
    }
    for m in &s.mechs {
        let f = m.frame(race_tick);
        if f.t.dist(eye) < int(560) {
            sc::mech(b, &f, &m.mech, race_tick);
        }
    }
    let (at, r) = s.black_hole;
    if at.dist(eye) < int(900) {
        sc::black_hole(b, &black_hole_frame(c, at), r, tick);
    }
    for p in &s.set_pieces {
        set_piece(b, p, eye, tick, race_tick);
    }
}

/// A bouncy body's shield: a soft pulsing glow round its box.
fn shield(b: &mut Batch, body: &Body, tick: u32, k: usize) {
    let f = &body.frame;
    let unit = Affine {
        c0: f.c0.norm(),
        c1: f.c1.norm(),
        c2: f.c2.norm(),
        t: f.t,
    };
    let pulse = ONE / 5 + sin((tick as i32).wrapping_mul(300) + k as i32 * 7_000).abs() / 8;
    let h = body.half;
    b.push_halo(
        Mesh::Sphere,
        &unit.then(&Affine::scale(v3(
            h.x * 2 + int(3),
            h.y * 2 + int(3),
            h.z * 2 + int(3),
        ))),
        SHIELD,
        pulse,
    );
}


/// The black hole's frame: its disc tilted toward the start, so an ape
/// coming in sees it from a little above.
fn black_hole_frame(c: &Course, at: V3) -> Affine {
    Affine::translate(at)
        .then(&Affine::rot_y(course_yaw(c)))
        .then(&Affine::rot_x(deg(14)))
}

fn set_piece(b: &mut Batch, p: &SetPiece, eye: V3, tick: u32, race_tick: u32) {
    match *p {
        SetPiece::Pass {
            from,
            to,
            start,
            end,
        } => {
            if race_tick < start || race_tick > end || end <= start {
                return;
            }
            let t = fx::div((race_tick - start) as i32, (end - start) as i32);
            let at = from.lerp(to, t);
            let yaw = yaw_of(to.sub(from).flat());
            if at.dist(eye) < int(700) {
                sc::mothership(b, &sc::pose(at, yaw, 0, 0, ONE), tick);
            }
        }
        SetPiece::Dogfight { frame } => {
            if frame.t.dist(eye) < int(700) {
                sc::dogfight(b, &frame, tick);
            }
        }
        SetPiece::Battle { frame } => {
            if frame.t.dist(eye) < int(900) {
                sc::capital_battle(b, &frame, tick);
            }
        }
    }
}

// ---- the tethers (the vines) ----

/// Tethers are drawn this far from the eye (the other maps' vines: 230 m).
pub const TETHER_REACH: i32 = int(300);

/// What a tether adds to the vine itself (`scene::vine_along`): an emitter
/// where it leaves its body (a dark collar round a glowing core, along
/// the way it hangs), and two pulses of light running down it from the
/// anchor to its end. `pts` are the vine's points as drawn, anchor first.
pub fn tether(b: &mut Batch, pts: &[V3], dir: V3, look: scene::VineLook, t: u32) {
    let Some(&a) = pts.first() else {
        return;
    };
    let collar = Affine::translate(a.add(dir.scale(cm(25))))
        .then(&Affine::along(dir))
        .then(&Affine::scale(v3(cm(110), cm(50), cm(110))));
    b.push(Mesh::Hex, &collar, PLATE_DARK, 0);
    b.push(
        Mesh::Octa,
        &placed(a.add(dir.scale(cm(60))), 0, v3(cm(60), cm(60), cm(60))),
        look.colour,
        ONE,
    );
    let hot = mix(look.colour, rgb(255, 255, 255), ONE * 3 / 5);
    for k in 0..2u32 {
        let period = 150;
        let frac = fx::div(((t + k * period / 2) % period) as i32, period as i32);
        let p = point_along(pts, frac);
        let r = look.width * 2 + cm(10);
        b.push(Mesh::Octa, &placed(p, 0, v3(r, r * 2, r)), hot, ONE);
    }
}

/// The point `t` (0..=ONE) of the way along a polyline, by length.
fn point_along(pts: &[V3], t: i32) -> V3 {
    let total: i32 = pts.windows(2).map(|w| w[1].sub(w[0]).len()).sum();
    let mut left = mul(total, t);
    for w in pts.windows(2) {
        let l = w[1].sub(w[0]).len();
        if left <= l && l > 0 {
            return w[0].lerp(w[1], fx::div(left, l));
        }
        left -= l;
    }
    pts[pts.len() - 1]
}

/// An item vine's marker on Sector Ape: a violet crystal turning beside
/// the tether, glowing while its item is there, dull grey while taken.
pub fn crystal(b: &mut Batch, at: V3, spin: i32, open: bool) {
    let f = Affine::translate(at)
        .then(&Affine::rot_y(spin))
        .then(&Affine::rot_z(deg(20)))
        .then(&Affine::scale(v3(cm(44), cm(95), cm(44))));
    if open {
        b.push(Mesh::Octa, &f, CRYSTAL, ONE);
        b.push_halo(
            Mesh::Sphere,
            &placed(at, 0, v3(cm(150), cm(150), cm(150))),
            CRYSTAL,
            ONE / 2,
        );
    } else {
        b.push(Mesh::Octa, &f, CRYSTAL_DULL, 0);
    }
}

// ---- the space jaguar ----

/// Floor is Lava on Sector Ape: the space jaguar (in its bubble helmet)
/// pops into being beside the ape with a flash and a burst of sparkles,
/// leaps, bites, eats, and pops out again (`jaguar::Eaten::pop`). No
/// shadow: there is no floor.
pub fn jaguar_pop(b: &mut Batch, e: &crate::jaguar::Eaten, c: &Course, race_tick: u32, tick: u32) {
    let ago = e.ago(race_tick);
    let lift = v3(0, cm(150), 0);
    if ago < POP_FLASH {
        pop_burst(b, e.from.add(lift), ago, tick);
    }
    if let Some(out) = ago.checked_sub(crate::jaguar::POP_OUT_AT) {
        if out < POP_FLASH {
            pop_burst(b, e.feet(c).add(lift), out, tick);
        }
    }
    let size = e.size(race_tick);
    if size < ONE / 20 {
        return;
    }
    if let Some(j) = e.jaguar(c, race_tick, 0) {
        let scale = mul(crate::scene::RACE_SCALE, size).max(1);
        crate::models::jaguar(
            b,
            &j,
            scale,
            &crate::models::BigCat::on(crate::map::Map::Sector),
        );
    }
}

/// How long a pop's flash and sparkles last (ticks).
const POP_FLASH: u32 = 54;

/// A pop: a white flash swelling and fading, a ring of light spreading,
/// and sparkles of white, mint, violet and banana yellow flying out.
fn pop_burst(b: &mut Batch, at: V3, age: u32, tick: u32) {
    let t = fx::div(age as i32, POP_FLASH as i32);
    let fade = ONE - t;
    let r = cm(80) + mul(t, int(4));
    b.push_halo(
        Mesh::Sphere,
        &placed(at, 0, v3(r * 2, r * 2, r * 2)),
        rgb(255, 255, 255),
        mul(fade, fade),
    );
    let ring = cm(60) + mul(t, int(6));
    b.push_halo(
        Mesh::Sphere,
        &placed(at, 0, v3(ring * 2, cm(20), ring * 2)),
        CRYSTAL,
        fade / 2,
    );
    let colours = [rgb(255, 255, 255), TETHER, CRYSTAL, rgb(255, 220, 60)];
    let mut dice = Dice(0x9_0B);
    for k in 0..16 {
        let dir = v3(
            dice.range(-ONE, ONE),
            dice.range(-ONE, ONE),
            dice.range(-ONE, ONE),
        )
        .norm();
        let out = mul(t, int(3) + dice.range(0, int(3)));
        let size = mul(cm(35), fade) + cm(4);
        let spin = (tick as i32).wrapping_mul(900) + k * 4_000;
        b.push(
            Mesh::Octa,
            &Affine::translate(at.add(dir.scale(out)))
                .then(&Affine::rot_y(spin))
                .then(&Affine::scale(v3(size, size * 2, size))),
            colours[k as usize % 4],
            SKY_GLOW,
        );
    }
}

// ---- the menus' ape preview and the title's flyover ----

/// Round the lobby deck behind the menus: the backdrop, and the mothership
/// riding at anchor behind the ape, broadside on, its engines glowing,
/// with a pair of ape wings circling it.
pub fn lobby_backdrop(b: &mut Batch, c: &Course, s: &SectorLayout, eye: V3, tick: u32) {
    backdrop(b, c, eye, tick);
    let (feet, yaw) = s.lobby;
    let fwd = facing(yaw, 0);
    let at = feet.add(fwd.scale(LOBBY_SHIP_AT)).add(v3(0, int(14), 0));
    sc::mothership(b, &sc::pose(at, yaw + deg(78), 0, deg(4), ONE), tick);
    for k in 0..2 {
        let a = (tick as i32).wrapping_mul(110) + k * fx::HALF;
        let p = at.add(v3(
            mul(sin(a), int(150)),
            int(40) + k * int(12),
            mul(fx::cos(a), int(70)),
        ));
        let heading = yaw_of(v3(fx::cos(a), 0, -mul(sin(a), ONE * 7 / 15)));
        sc::ape_wing(b, &sc::pose(p, heading, 0, -deg(30), ONE * 2), tick);
    }
}

/// How far behind the lobby deck the mothership rides.
const LOBBY_SHIP_AT: i32 = int(240);

/// Where an ape stands behind the menus (`SectorLayout.lobby`, a ship's
/// deck), the way it faces and the camera: the sun's glow below, the
/// mothership behind.
pub fn preview(c: &Course) -> (V3, i32, Camera) {
    let (feet, yaw) = c
        .sector
        .as_ref()
        .map(|s| s.lobby)
        .unwrap_or((c.starts[0], c.start_yaw));
    let fwd = facing(yaw, 0);
    let right = right_of(fwd);
    let eye = feet
        .add(fwd.scale(-cm(290)))
        .add(right.scale(-cm(40)))
        .add(v3(0, cm(110), 0));
    let target = feet.add(v3(0, cm(62), 0));
    (
        feet,
        yaw_of(eye.sub(feet).flat()),
        Camera {
            eye,
            fwd: target.sub(eye).norm(),
            fov: deg(34),
        },
    )
}

/// The title's flyover on Sector Ape: a glide through space along the
/// course, weaving gently up and down and side to side between the vines
/// and looking a little ahead, so the bodies drift past on every side.
pub fn flyover_camera(c: &Course, ticks: u32, from: i32) -> Camera {
    let lap = (c.length - int(100)).max(int(1)) as i64;
    let s = ((ticks as i64 * int(9) as i64 / crate::rules::TICKS_PER_SECOND as i64 + from as i64)
        % lap) as i32
        + int(40);
    let at = |s: i32| {
        let p = c.point(s);
        let right = right_of(c.heading_at(s));
        let w = sin(s / 3);
        let h = sin(s / 4 + 9_000);
        p.add(right.scale(mul(w, int(14))))
            .add(v3(0, int(6) + mul(h, int(8)), 0))
    };
    let eye = at(s);
    let look = at(s + int(36)).sub(v3(0, int(4), 0));
    Camera {
        eye,
        fwd: look.sub(eye).norm(),
        fov: deg(74),
    }
}

/// How far apart the flyover's samples are, for tests.
#[allow(dead_code)]
const FLY_STEP: i32 = STEP;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Map;

    #[test]
    fn the_sun_is_far_below_and_the_light_comes_from_underneath() {
        let c = Map::Sector.course();
        let l = scene::look_of(&c);
        assert!(l.light_dir.y < -fx::sin(deg(50)), "the sun is below");
        // a blazing orange-white: red over green over blue
        assert!(l.light[0] > ONE && l.light[0] > l.light[1] && l.light[1] > l.light[2]);
        // the bellies glow orange, the tops a cool indigo
        assert!(l.ground[0] > l.ground[2] * 3);
        assert!(l.sky[2] > l.sky[0] * 2);
        // an edge light, and no other map has one
        assert!(l.rim[2] > ONE / 2);
        for m in [Map::Jungle, Map::City, Map::Ski] {
            assert_eq!(scene::look(m).rim, [0; 3], "{m:?}");
        }
        // deep black-blue space behind everything
        assert!(l.fog.iter().all(|v| *v < ONE / 8));
    }

    #[test]
    fn the_backdrop_stays_inside_the_far_plane() {
        let c = Map::Sector.course();
        let eye = c.point(int(300)).add(v3(0, int(20), 0));
        let mut b = Batch::new();
        backdrop(&mut b, &c, eye, 77);
        let mut halos = b.take_halos();
        let (v, _) = std::mem::take(&mut b).finish(crate::render::DYNAMIC_BUFFER);
        let (h, _) = std::mem::take(&mut halos).finish(crate::render::DYNAMIC_BUFFER);
        assert!(v.len() / crate::render::INSTANCE_VALUES > STAR_COUNT as usize / 2);
        for i in v.chunks(16).chain(h.chunks(16)) {
            let at = v3(i[3], i[7], i[11]);
            assert!(
                at.dist(eye) < crate::render::FAR - int(40),
                "{}",
                at.dist(eye) / ONE
            );
        }
        // the sun is below the eye
        let sun = eye.add(sun_dir(&c).scale(SUN_AT));
        assert!(sun.y < eye.y - int(300));
    }
}
