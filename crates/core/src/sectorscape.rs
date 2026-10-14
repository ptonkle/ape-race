//! Drawing Sector Ape (2026-10-13, `briefs/space-map.md`, the art agent's
//! SECOND-ORDER SA rows).
//!
//! **The light first** (MAPS "Art lessons"): there is no time of day. A
//! blazing orange-white sun hangs far below the track, a little ahead and
//! to the left of the way the course runs, and lights everything from
//! underneath: hull bellies glow warm orange-white, their tops fall into
//! a cool indigo from the far stars and the gas giant, and every outline
//! catches a blue-violet edge light (`Look.rim`, `scene.frag`: SECOND-ORDER
//! SM4), so a hull's top reads against black space. The sky is deep
//! black-blue with stars; a violet ringed gas giant hangs high off to the
//! right; the purple and green haze of Sector X drifts far off round the
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

use crate::course::{Course, Platform};
use crate::fx::{self, cm, deg, int, mul, ratio, sin, Dice, ONE, TURN};
use crate::geom::{facing, right_of, v3, yaw_of, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, placed, rgb, rod, shade, Batch, Camera, Rgb};
use crate::scene::{self, Look, Scenery};
use crate::sector::{Body, Model, SectorLayout, SetPiece};
use crate::spacecraft as sc;

// ---- the light: a sun far below ----

/// Toward the sun: its bearing from the way the course runs (start to
/// finish; a yaw grows to the left) and its height (below the horizon).
/// Ahead and to the left, so its glow rises over the bottom of the chase
/// view, and steep, so the light comes from underneath.
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
/// the left of the way the course runs.
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
pub const TETHER_HALO: i32 = 3;
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
/// The planet: off to the right of the way the course runs, high.
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
        // a band rising from low on the left, round behind, to the right
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
        let kind = if k == 0 {
            DeckKind::Start
        } else if k == s.finish_deck {
            DeckKind::Finish
        } else if s.pads.contains(&k) {
            DeckKind::Pad
        } else {
            DeckKind::Plain
        };
        deck(sink.at(pl.centre), pl, kind);
    }
    tally(&sink, "decks");
    for (k, body) in s.bodies.iter().enumerate() {
        if body.model == Model::Hulk {
            hulk(sink.at(body.frame.t), body, &s.door);
            continue;
        }
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
fn deck(b: &mut Batch, pl: &Platform, kind: DeckKind) {
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
    let ring = |b: &mut Batch, r: i32, n: i32, colour: Rgb, glow: i32| {
        for k in 0..n {
            let a = k * TURN / n;
            let out = v3(sin(a), 0, fx::cos(a));
            let along = v3(fx::cos(a), 0, -sin(a));
            let seg = mul(r, ratio(6_283, 1_000)) / n * 7 / 10;
            b.push(
                Mesh::Cube,
                &frame.then(&Affine {
                    c0: out.scale(cm(60)),
                    c1: V3::UP.scale(cm(8)),
                    c2: along.scale(seg),
                    t: out.scale(r).add(v3(0, cm(6), 0)),
                }),
                colour,
                glow,
            );
        }
    };
    match kind {
        DeckKind::Start => {
            // launch arrows painted down the middle, pointing on
            for k in 0..3 {
                let z = -hl / 2 + k * hl / 2;
                for sx in [-1, 1] {
                    b.push(
                        Mesh::Cube,
                        &frame
                            .then(&Affine::translate(v3(sx * cm(90), cm(4), z)))
                            .then(&Affine::rot_y(sx * deg(40)))
                            .then(&Affine::scale(v3(cm(50), cm(6), int(3)))),
                        HAZARD,
                        ONE / 2,
                    );
                }
            }
            // start lights set in both edges
            for sx in [-1, 1] {
                for k in 0..4 {
                    let z = -hl + (k * 2 + 1) * hl / 4;
                    part(
                        b,
                        Mesh::Octa,
                        v3(sx * (hw - cm(30)), cm(10), z),
                        v3(cm(22), cm(22), cm(40)),
                        if k % 2 == 0 { NEON } else { NEON_WARM },
                        SKY_GLOW,
                    );
                }
            }
        }
        // a deck an ape is put back on: a glowing ring where it lands
        DeckKind::Pad => ring(b, hw.min(hl) * 3 / 5, 16, TETHER, ONE * 3 / 4),
        // the finish deck, where the apes dance: a banana-yellow ring and a
        // checkered band across each end
        DeckKind::Finish => {
            ring(b, hw.min(hl) * 3 / 5, 36, rgb(255, 214, 56), ONE);
            ring(b, hw.min(hl) * 2 / 5, 28, rgb(255, 246, 236), ONE / 2);
            for sz in [-1, 1] {
                let n = (hw * 2 / int(3)).clamp(4, 40);
                for k in 0..n {
                    for row in 0..2 {
                        let x = -hw + (k * 2 + 1) * hw / n;
                        let z = sz * (hl - int(3) - row * int(3) / 2);
                        part(
                            b,
                            Mesh::Cube,
                            v3(x, cm(5), z),
                            v3(hw * 2 / n, cm(6), int(3) / 2),
                            if (k + row) % 2 == 0 {
                                rgb(250, 250, 250)
                            } else {
                                rgb(20, 20, 26)
                            },
                            ONE / 3,
                        );
                    }
                }
            }
        }
        DeckKind::Plain => {}
    }
}

/// What a deck is for, which sets its markings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeckKind {
    Start,
    /// a deck an ape is put back on past the boundary
    Pad,
    /// past the finish, under the black hole: the dance floor
    Finish,
    Plain,
}

/// A body the vines hang from, by its model.
fn body(b: &mut Batch, body: &Body, k: usize) {
    let f = &body.frame;
    match body.model {
        Model::Chunk(v) => sc::hull_chunk(b, f, 0, v),
        Model::Girder(len) => sc::girder(b, f, len / ONE),
        Model::Asteroid(v) => sc::asteroid(b, f, v),
        Model::Wing => sc::ape_wing(b, f, 0),
        // a deck's slab: drawn with its platform (`deck`)
        Model::Deck => {}
        Model::Hulk => slab_body(b, body),
        // the walls round the wreck's passage and the hulk's corridor:
        // drawn as part of those
        Model::Hidden => {}
    }
    let _ = k;
}

/// The door's ship: a rust-red freighter hull its box's size, a corridor
/// straight through it along its length where the door is (the door's
/// opening, `Door.half`), lit inside with strips, hazard frames round
/// both mouths, rows of windows, a bridge tower and engines at the stern.
/// The door itself is drawn each frame (`door`).
fn hulk(b: &mut Batch, body: &Body, door: &crate::sector::Door) {
    let f = &body.frame;
    let axes = [f.c0.norm(), f.c1.norm(), f.c2.norm()];
    let unit = Affine {
        c0: axes[0],
        c1: axes[1],
        c2: axes[2],
        t: f.t,
    };
    let h = body.half;
    // the corridor: through the door's middle, along the hull, as wide and
    // tall as the door's opening (a 12 m by 9 m bore if there is none)
    let (ox, oy, dw, dh) = if door.half == V3::ZERO {
        (0, 0, int(6), cm(450))
    } else {
        let o = door.frame.t.sub(f.t);
        (
            o.dot(axes[0]),
            o.dot(axes[1]),
            door.half.x + cm(20),
            door.half.y + cm(20),
        )
    };
    let part = |b: &mut Batch, mesh: Mesh, at: V3, size: V3, c: Rgb, glow: i32| {
        b.push(
            mesh,
            &unit.then(&Affine::translate(at)).then(&Affine::scale(size)),
            c,
            glow,
        );
    };
    let slab = |b: &mut Batch, x0: i32, x1: i32, y0: i32, y1: i32, c: Rgb| {
        if x1 - x0 > cm(10) && y1 - y0 > cm(10) {
            part(
                b,
                Mesh::Cube,
                v3((x0 + x1) / 2, (y0 + y1) / 2, 0),
                v3(x1 - x0, y1 - y0, h.z * 2),
                c,
                0,
            );
        }
    };
    // the walls round the bore
    slab(b, -h.x, ox - dw, -h.y, h.y, HULK);
    slab(b, ox + dw, h.x, -h.y, h.y, HULK);
    slab(b, ox - dw, ox + dw, -h.y, oy - dh, shade(HULK, 90));
    slab(b, ox - dw, ox + dw, oy + dh, h.y, HULK);
    // a grey armour band along each side and the keel
    for sx in [-1, 1] {
        part(
            b,
            Mesh::Bevel,
            v3(sx * (h.x + cm(40)), -h.y / 3, 0),
            v3(cm(120), h.y / 2, h.z * 19 / 10),
            HULK_GREY,
            0,
        );
        // rows of windows
        for row in 0..2 {
            let y = h.y / 4 + row * int(3);
            if y > h.y - int(1) {
                continue;
            }
            let n = (h.z * 2 / int(6)).clamp(2, 14);
            for k in 0..n {
                let z = -h.z + (k * 2 + 1) * h.z / n;
                part(
                    b,
                    Mesh::Cube,
                    v3(sx * (h.x + cm(10)), y, z),
                    v3(cm(20), cm(80), cm(160)),
                    if (k + row) % 4 == 0 {
                        rgb(255, 200, 120)
                    } else {
                        WINDOW
                    },
                    ONE,
                );
            }
        }
    }
    part(
        b,
        Mesh::Wedge,
        v3(0, -h.y - cm(150), h.z / 4),
        v3(h.x * 3 / 2, cm(300), h.z * 3 / 2),
        HULK_GREY,
        0,
    );
    // the bore's lining: dark plates (the sun below cannot reach inside),
    // ribbed, with a lit strip down the roof
    part(
        b,
        Mesh::Cube,
        v3(ox, oy + dh - cm(15), 0),
        v3(dw * 2, cm(30), h.z * 2),
        BORE,
        0,
    );
    for sx in [-1, 1] {
        part(
            b,
            Mesh::Cube,
            v3(ox + sx * (dw - cm(15)), oy, 0),
            v3(cm(30), dh * 2, h.z * 2),
            shade(BORE, 130),
            0,
        );
    }
    for sx in [-1, 0, 1] {
        part(
            b,
            Mesh::Cube,
            v3(ox + sx * dw / 2, oy + dh - cm(35), 0),
            v3(cm(40), cm(10), h.z * 2),
            rgb(220, 240, 255),
            SKY_GLOW,
        );
    }
    let ribs = (h.z * 2 / int(8)).clamp(2, 20);
    for k in 1..ribs {
        let z = -h.z + k * h.z * 2 / ribs;
        part(
            b,
            Mesh::Cube,
            v3(ox, oy + dh - cm(50), z),
            v3(dw * 2, cm(40), cm(50)),
            shade(BORE, 160),
            0,
        );
        for sx in [-1, 1] {
            part(
                b,
                Mesh::Cube,
                v3(ox + sx * (dw - cm(45)), oy, z),
                v3(cm(40), dh * 2, cm(50)),
                shade(BORE, 160),
                0,
            );
        }
    }
    // the bore's lights: strips along its walls, and hazard frames round
    // both mouths
    for sx in [-1, 1] {
        part(
            b,
            Mesh::Cube,
            v3(ox + sx * (dw - cm(40)), oy + dh - cm(60), 0),
            v3(cm(25), cm(25), h.z * 2),
            NEON,
            SKY_GLOW,
        );
        part(
            b,
            Mesh::Cube,
            v3(ox + sx * (dw - cm(40)), oy - dh + cm(40), 0),
            v3(cm(25), cm(25), h.z * 2),
            NEON_WARM,
            ONE,
        );
    }
    for sz in [-1, 1] {
        let z = sz * (h.z + cm(30));
        let n = 8;
        for k in 0..n {
            let c = if k % 2 == 0 { HAZARD } else { PLATE_DARK };
            let t = -dw + (k * 2 + 1) * dw / n;
            part(
                b,
                Mesh::Cube,
                v3(ox + t, oy + dh + cm(40), z),
                v3(dw / n * 2, cm(80), cm(60)),
                c,
                0,
            );
            part(
                b,
                Mesh::Cube,
                v3(ox + t, oy - dh - cm(40), z),
                v3(dw / n * 2, cm(80), cm(60)),
                c,
                0,
            );
        }
        for sx in [-1, 1] {
            part(
                b,
                Mesh::Cube,
                v3(ox + sx * (dw + cm(40)), oy, z),
                v3(cm(80), dh * 2 + cm(160), cm(60)),
                HAZARD,
                0,
            );
        }
    }
    // ribs round the hull every 10 m, standing proud of the plating
    let ribs = (h.z * 2 / int(10)).clamp(2, 16);
    for k in 1..ribs {
        let z = -h.z + k * h.z * 2 / ribs;
        for (at, size) in [
            (v3(0, h.y + cm(20), z), v3(h.x * 2 + cm(60), cm(40), cm(70))),
            (
                v3(0, -h.y - cm(20), z),
                v3(h.x * 2 + cm(60), cm(40), cm(70)),
            ),
            (v3(h.x + cm(20), 0, z), v3(cm(40), h.y * 2, cm(70))),
            (v3(-h.x - cm(20), 0, z), v3(cm(40), h.y * 2, cm(70))),
        ] {
            part(b, Mesh::Cube, at, size, HULK_GREY, 0);
        }
    }
    // a white band down each side
    for sx in [-1, 1] {
        part(
            b,
            Mesh::Cube,
            v3(sx * (h.x + cm(5)), h.y * 2 / 3, 0),
            v3(cm(20), cm(90), h.z * 2 - int(2)),
            rgb(226, 222, 214),
            0,
        );
    }
    // cargo containers stacked along the top, ahead of the bridge
    let cargo = [
        rgb(232, 120, 40),
        rgb(40, 150, 160),
        rgb(220, 214, 200),
        rgb(170, 40, 50),
    ];
    let mut k = 0;
    let mut z = -h.z / 4 + int(8);
    while z < h.z - int(5) {
        for sx in [-1, 1] {
            if h.x > int(6) {
                part(
                    b,
                    Mesh::Cube,
                    v3(sx * h.x / 2, h.y + int(2) + cm(40), z),
                    v3(h.x * 8 / 10, int(4), int(9)),
                    cargo[k % cargo.len()],
                    0,
                );
            }
            k += 1;
        }
        z += int(10);
    }
    // the bridge tower toward the stern, its windows lit
    part(
        b,
        Mesh::Bevel,
        v3(0, h.y + int(4), -h.z / 2),
        v3(h.x, int(8), int(12)),
        HULK_GREY,
        0,
    );
    part(
        b,
        Mesh::Cube,
        v3(0, h.y + int(6), -h.z / 2 + int(6) + cm(10)),
        v3(h.x * 8 / 10, cm(90), cm(20)),
        WINDOW,
        ONE,
    );
    part(
        b,
        Mesh::Cyl,
        v3(h.x / 3, h.y + int(14), -h.z / 2),
        v3(cm(40), int(12), cm(40)),
        PLATE_LIGHT,
        0,
    );
    // engines at the stern, glowing
    for sx in [-1, 1] {
        let at = v3(sx * h.x / 2, -h.y / 3, -h.z - int(2));
        part(b, Mesh::Cyl, at, v3(int(6), int(5), int(6)), HULK_GREY, 0);
        part(
            b,
            Mesh::Hex,
            at.add(v3(0, 0, -int(2) - cm(60))),
            v3(int(5), cm(30), int(5)),
            ENGINE,
            SKY_GLOW,
        );
    }
}

/// The inside of the Hulk's bore: dark plates.
const BORE: Rgb = rgb(30, 30, 38);
/// The freighter's hull: a rusty red, its armour a dark grey.
const HULK: Rgb = rgb(138, 70, 58);
const HULK_GREY: Rgb = rgb(84, 88, 98);
const WINDOW: Rgb = rgb(150, 220, 255);

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

/// The boundary shown: every `GATE_EVERY` metres along the course, a
/// glowing bracket at each corner of the edge an ape is reset past (the
/// sides, below and above), with a warm light at its point. Far apart and
/// dim, they read as the lane through space and never compete with the
/// vines.
const GATE_EVERY: i32 = int(110);
/// How far each bracket's arms run along the edges.
const BRACKET: i32 = int(7);

fn gates(s: &mut Sink, c: &Course, l: &SectorLayout) {
    let bd = l.boundary;
    let mut at = int(60);
    while at < c.length {
        let p = c.point(at);
        let fwd = c.heading_at(at);
        let right = right_of(fwd);
        let b = s.at(p);
        let corner = |sx: i32, y: i32| p.add(right.scale(sx * bd.side)).add(v3(0, y, 0));
        for (sx, y, sy) in [
            (-1, bd.below, 1),
            (1, bd.below, 1),
            (1, bd.above, -1),
            (-1, bd.above, -1),
        ] {
            let k = corner(sx, y);
            let along_side = k.add(right.scale(-sx * BRACKET));
            let along_up = k.add(v3(0, sy * BRACKET, 0));
            for end in [along_side, along_up] {
                b.push(Mesh::Cube, &rod(k, end, cm(40)), GATE, ONE);
            }
            b.push(
                Mesh::Octa,
                &placed(k, 0, v3(cm(130), cm(130), cm(130))),
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
    sc::black_hole(
        b,
        &seen_from(eye, &black_hole_frame(c, at), HOLE_NEAR),
        r,
        tick,
    );
    for p in &s.set_pieces {
        set_piece(b, p, eye, tick, race_tick);
    }
    if s.wreck.frame.t.dist(eye) < int(700) && s.wreck.frame != Affine::IDENTITY {
        sc::wreck(b, &s.wreck.frame, tick, s.wreck.gap);
    }
    door(b, c, s, eye, race_tick);
    for (k, (kind, f)) in landmarks(c, s).into_iter().enumerate() {
        landmark(b, kind, &f, eye, tick, k);
    }
    drifting(b, c, s, eye, tick);
}

// ---- the door ----

/// How long the door takes to slide shut or open (ticks): quick, so what
/// is drawn stays close to what is solid (`Door::shut`).
const DOOR_SLIDE: u32 = 30;

/// The shared door in the hulk's corridor: a heavy slab with hazard
/// stripes that slams down from the corridor's roof when the switch is
/// swung (`Door::shut`) and slides back up as it reopens, with a red
/// light over the corridor while it is shut and a green one while open.
fn door(b: &mut Batch, c: &Course, s: &SectorLayout, eye: V3, tick: u32) {
    let _ = c;
    door_alone(b, &s.door, eye, tick);
}

fn door_alone(b: &mut Batch, d: &crate::sector::Door, eye: V3, tick: u32) {
    if d.half == V3::ZERO || d.frame.t.dist(eye) > int(400) {
        return;
    }
    // how far down it is, 0 (open, up in the roof) to ONE (shut)
    let down = match d.shut_at {
        Some(at) if tick >= at => {
            let since = tick - at;
            let open_at = crate::sector::DOOR_SHUT;
            if since < open_at {
                fx::div(since.min(DOOR_SLIDE) as i32, DOOR_SLIDE as i32)
            } else {
                ONE - fx::div((since - open_at).min(DOOR_SLIDE) as i32, DOOR_SLIDE as i32)
            }
        }
        _ => 0,
    };
    let f = &d.frame;
    let axes = [f.c0.norm(), f.c1.norm(), f.c2.norm()];
    let h = d.half;
    let up = axes[1].scale(mul(ONE - down, h.y * 2));
    let at = f.t.add(up);
    let unit = Affine {
        c0: axes[0],
        c1: axes[1],
        c2: axes[2],
        t: at,
    };
    let part = |b: &mut Batch, mesh: Mesh, p: V3, size: V3, colour: Rgb, glow: i32| {
        b.push(
            mesh,
            &unit.then(&Affine::translate(p)).then(&Affine::scale(size)),
            colour,
            glow,
        );
    };
    if down > 0 {
        part(
            b,
            Mesh::Bevel,
            V3::ZERO,
            v3(h.x * 2, h.y * 2, h.z * 2),
            PLATE_LIGHT,
            0,
        );
        // hazard stripes across both faces
        let n = (h.x * 2 / int(2)).clamp(2, 12);
        for k in 0..n {
            let x = -h.x + (k * 2 + 1) * h.x / n;
            for sz in [-1, 1] {
                part(
                    b,
                    Mesh::Cube,
                    v3(x, -h.y + cm(80), sz * (h.z + cm(3))),
                    v3(h.x / n, cm(120), cm(8)),
                    if k % 2 == 0 { HAZARD } else { PLATE_DARK },
                    ONE / 2,
                );
            }
        }
        // a glowing seam along its bottom edge
        part(
            b,
            Mesh::Cube,
            v3(0, -h.y, 0),
            v3(h.x * 2, cm(20), h.z * 2 + cm(10)),
            NEON_WARM,
            SKY_GLOW,
        );
    }
    // the warning lights over the corridor's mouths: red while shut
    let shut = d.shut(tick);
    let light = if shut {
        rgb(255, 50, 40)
    } else {
        rgb(60, 255, 120)
    };
    for sz in [-1, 1] {
        let p =
            f.t.add(axes[1].scale(h.y + cm(90)))
                .add(axes[2].scale(sz * (h.z + int(1))));
        b.push(
            Mesh::Octa,
            &placed(
                p,
                (tick as i32).wrapping_mul(400),
                v3(cm(90), cm(90), cm(90)),
            ),
            light,
            SKY_GLOW,
        );
        if shut {
            b.push_halo(
                Mesh::Sphere,
                &placed(p, 0, v3(int(4), int(4), int(4))),
                light,
                ONE / 2,
            );
        }
    }
}

// ---- landmarks of the art's own: battles, a station, drifting wrecks ----

/// A landmark the art adds round the course (scenery only, outside the
/// boundary): the brief asks for plentiful landmarks along the way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Landmark {
    /// two capital ships trading broadsides (`spacecraft::capital_battle`)
    Battle,
    /// fighters chasing in loops (`spacecraft::dogfight`), drawn large
    Dogfight,
    /// a ring station turning slowly
    Station,
    /// a slow convoy of ape wings in formation
    Wings,
}

/// The art's landmarks along the course: every ~260 m, alternating sides,
/// well outside the boundary, each with its frame.
fn landmarks(c: &Course, s: &SectorLayout) -> Vec<(Landmark, Affine)> {
    let bd = s.boundary;
    let mut out = Vec::new();
    let mut dice = Dice(0x1A4D_3A2C);
    let kinds = [
        Landmark::Battle,
        Landmark::Station,
        Landmark::Dogfight,
        Landmark::Wings,
        Landmark::Battle,
        Landmark::Dogfight,
        Landmark::Station,
    ];
    let mut at = int(150);
    let mut k = 0;
    while at < c.length - int(80) {
        let kind = kinds[k % kinds.len()];
        let side = if k % 2 == 0 { 1 } else { -1 };
        let p = c.point(at);
        let fwd = c.heading_at(at);
        let right = right_of(fwd);
        let (out_by, up) = match kind {
            Landmark::Battle => (bd.side + int(220), int(dice.range(-60, 40))),
            Landmark::Station => (bd.side + int(170), int(dice.range(-80, 60))),
            Landmark::Dogfight => (bd.side + int(60), bd.above + int(dice.range(30, 70))),
            Landmark::Wings => (bd.side + int(40), bd.above + int(20)),
        };
        let q = p.add(right.scale(side * out_by)).add(v3(0, up, 0));
        let yaw = yaw_of(fwd) + if kind == Landmark::Battle { deg(90) } else { 0 };
        let scale = match kind {
            Landmark::Dogfight => ONE * 2,
            Landmark::Wings => ONE * 2,
            _ => ONE,
        };
        out.push((kind, sc::pose(q, yaw, 0, 0, scale)));
        at += int(dice.range(230, 300));
        k += 1;
    }
    out
}

fn landmark(b: &mut Batch, kind: Landmark, f: &Affine, eye: V3, tick: u32, k: usize) {
    let d = f.t.dist(eye);
    match kind {
        Landmark::Battle if d < int(700) => sc::capital_battle(b, f, tick + k as u32 * 97),
        Landmark::Dogfight if d < int(500) => sc::dogfight(b, f, tick + k as u32 * 131),
        Landmark::Station if d < int(650) => station(b, f, tick),
        Landmark::Wings if d < int(450) => wings(b, f, tick),
        _ => {}
    }
}

/// A ring station: a wheel of hull segments with lit windows, four spokes
/// to a hub, a docking spire and blinking beacons, turning slowly about
/// its axle. About 70 shapes.
fn station(b: &mut Batch, f: &Affine, tick: u32) {
    let spin = (tick as i32).wrapping_mul(10);
    let g = f
        .then(&Affine::rot_x(deg(90) - deg(20)))
        .then(&Affine::rot_y(spin));
    let n = 20;
    let rad = int(48);
    for k in 0..n {
        let a = k * TURN / n;
        let out = v3(sin(a), 0, fx::cos(a));
        let along = v3(fx::cos(a), 0, -sin(a));
        let seg = mul(rad, ratio(6_283, 1_000)) / n + int(1);
        b.push(
            Mesh::Bevel,
            &g.then(&Affine {
                c0: out.scale(int(9)),
                c1: V3::UP.scale(int(8)),
                c2: along.scale(seg),
                t: out.scale(rad),
            }),
            if k % 5 == 0 { PLATE_LIGHT } else { KEEL },
            0,
        );
        // a row of lit windows on the outer face
        b.push(
            Mesh::Cube,
            &g.then(&Affine {
                c0: out.scale(cm(30)),
                c1: V3::UP.scale(int(2)),
                c2: along.scale(seg * 7 / 10),
                t: out.scale(rad + int(4) + cm(20)),
            }),
            if k % 3 == 0 {
                rgb(255, 210, 140)
            } else {
                rgb(150, 220, 255)
            },
            ONE,
        );
    }
    for k in 0..4 {
        let a = k * TURN / 4 + TURN / 8;
        let out = v3(sin(a), 0, fx::cos(a));
        b.push(
            Mesh::Cyl,
            &g.then(&rod(out.scale(int(8)), out.scale(rad - int(4)), int(3))),
            PLATE,
            0,
        );
    }
    b.push(
        Mesh::Barrel,
        &g.then(&Affine::scale(v3(int(16), int(22), int(16)))),
        KEEL,
        0,
    );
    b.push(
        Mesh::Spike,
        &g.then(&rod(v3(0, int(10), 0), v3(0, int(40), 0), int(4))),
        PLATE_LIGHT,
        0,
    );
    if (tick / 50).is_multiple_of(2) {
        b.push(
            Mesh::Octa,
            &g.then(&Affine::translate(v3(0, int(41), 0)))
                .then(&Affine::scale(v3(int(2), int(2), int(2)))),
            rgb(255, 60, 50),
            SKY_GLOW,
        );
    }
    b.push(
        Mesh::Hex,
        &g.then(&Affine::translate(v3(0, -int(12), 0)))
            .then(&Affine::scale(v3(int(10), int(2), int(10)))),
        ENGINE,
        SKY_GLOW,
    );
}

/// Five ape wings flying in a V, slowly overtaking along the course.
fn wings(b: &mut Batch, f: &Affine, tick: u32) {
    let ahead = mul(sin((tick as i32).wrapping_mul(20)), int(20));
    for (x, z) in [(0, 0), (-12, -10), (12, -10), (-24, -20), (24, -20)] {
        let p = v3(
            int(x),
            mul(sin((tick as i32).wrapping_mul(60) + x * 900), int(1)),
            int(z) + ahead,
        );
        sc::ape_wing(b, &f.then(&Affine::translate(p)), tick);
    }
}

/// Wreckage and rocks drifting and tumbling just outside the boundary near
/// the eye: they carry no vine, so they may turn.
fn drifting(b: &mut Batch, c: &Course, s: &SectorLayout, eye: V3, tick: u32) {
    let bd = s.boundary;
    let (i, _) = c.progress(eye, c.nearest_anywhere(eye));
    let mut dice = Dice(0xD21F7);
    let mut at = int(30);
    let mut k = 0u32;
    while at < c.length {
        let step = int(dice.range(40, 70));
        let side = if dice.below(2) == 0 { 1 } else { -1 };
        let out = bd.side + int(dice.range(6, 30));
        let up = int(dice.range(-30, 60));
        let kind = dice.below(10);
        let size = ONE + dice.range(0, ONE / 2);
        if (at - i).abs() < int(260) {
            let p = c.point(at);
            let right = right_of(c.heading_at(at));
            let drift = v3(
                mul(
                    sin((tick as i32).wrapping_mul(7) + k as i32 * 5_000),
                    int(3),
                ),
                mul(
                    sin((tick as i32).wrapping_mul(5) + k as i32 * 7_000),
                    int(2),
                ),
                0,
            );
            let q = p.add(right.scale(side * out)).add(v3(0, up, 0)).add(drift);
            let f = Affine::translate(q)
                .then(&sc::tumble(tick, k))
                .then(&Affine::scale(v3(size, size, size)));
            match kind {
                0..=5 => sc::asteroid(b, &f, k),
                6..=8 => sc::hull_chunk(b, &f, tick, k),
                _ => sc::girder(b, &f, 14),
            }
        }
        at += step;
        k += 1;
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

/// The black hole's frame: its disc stood up 60° to face the apes coming
/// in (seen from a little above, as in the person's example), so its wide
/// disc rises round the finish instead of lying across the course before
/// it (SECOND-ORDER SA10).
pub fn black_hole_frame(c: &Course, at: V3) -> Affine {
    Affine::translate(at)
        .then(&Affine::rot_y(yaw_of(c.heading_at(c.length))))
        .then(&Affine::rot_x(-BLACK_HOLE_TILT))
}

/// Beyond this distance the black hole is drawn shrunk toward the eye
/// (`seen_from`), so it shows on the horizon from anywhere on the course
/// without the far plane cutting it.
const HOLE_NEAR: i32 = int(300);

/// A frame `f` as seen from `eye`, brought in to `near` if it is further:
/// moved along the line of sight and shrunk by the same share, so it looks
/// exactly as big and in the same place, but stays inside the far plane.
fn seen_from(eye: V3, f: &Affine, near: i32) -> Affine {
    let d = f.t.dist(eye);
    if d <= near {
        return *f;
    }
    let k = fx::div(near, d);
    Affine {
        c0: f.c0.scale(k),
        c1: f.c1.scale(k),
        c2: f.c2.scale(k),
        t: eye.add(f.t.sub(eye).scale(k)),
    }
}

/// How far the disc is stood up from lying flat.
const BLACK_HOLE_TILT: i32 = deg(60);

fn set_piece(b: &mut Batch, p: &SetPiece, eye: V3, tick: u32, race_tick: u32) {
    match *p {
        SetPiece::Pass {
            from,
            to,
            start,
            end,
        } => {
            // waiting behind the start until its pass, then on past its
            // end and away
            if end <= start {
                return;
            }
            let t = fx::div(
                race_tick.saturating_sub(start).min((end - start) * 3) as i32,
                (end - start) as i32,
            );
            let at = from.add(to.sub(from).scale(t));
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

// ---- flashes: an ape put back on a deck, the door shutting ----

/// How long a flash shows (race ticks).
pub const FLASH_TICKS: u32 = 60;

/// This tick's flashes on Sector Ape (`None` on every other map): where
/// each ape put back on a deck now stands, and the door when it shuts.
pub fn flash_of(race: &crate::race::Race) -> Option<Vec<(V3, u32)>> {
    let s = race.course.sector.as_deref()?;
    let tick = race.tick;
    Some(
        race.events
            .iter()
            .filter_map(|e| match *e {
                crate::race::Event::Reset { who, .. } => {
                    race.racers.get(who).map(|r| (r.ape.pos, tick))
                }
                crate::race::Event::Door { .. } => Some((s.door.frame.t, tick)),
                _ => None,
            })
            .collect(),
    )
}

/// The flashes: an ape warps back onto its deck in a burst of mint light
/// and sparkles; the door's shutting flashes red-white.
pub fn flashes(b: &mut Batch, flashes: &[(V3, u32)], race_tick: u32, tick: u32) {
    for &(at, when) in flashes {
        let age = race_tick.saturating_sub(when);
        if age < POP_FLASH {
            pop_burst(b, at, age, tick);
            // a column of light where it lands
            let t = fx::div(age as i32, POP_FLASH as i32);
            b.push_halo(
                Mesh::Cyl,
                &rod(
                    at.sub(v3(0, int(2), 0)),
                    at.add(v3(0, int(6), 0)),
                    cm(140) + mul(t, int(2)),
                ),
                TETHER,
                ONE - t,
            );
        }
    }
}

// ---- the finish: apes drawn into the black hole ----

/// How a finished ape near the black hole is drawn (`None`: as ever): it
/// stretches toward the hole and thins across as it nears the banana
/// horizon, as if being drawn in. The transform is about the ape's hands
/// (`at`).
pub fn stretch(s: &SectorLayout, r: &crate::race::Racer, at: V3) -> Option<Affine> {
    r.finish?;
    let (hole, rad) = s.black_hole;
    let to = hole.sub(at);
    let d = to.len();
    let from = mul(rad, int(3));
    let rim = mul(rad, crate::sector::rules::HORIZON);
    if d >= from || d == 0 {
        return None;
    }
    let k = fx::div(from - d, (from - rim).max(ONE)).clamp(0, ONE);
    let dir = to.norm();
    let along = ONE + mul(k, ONE * 4 / 5);
    let across = ONE - mul(k, ONE * 2 / 5);
    // M = across·I + (along − across)·d dᵀ, about `at`
    let col = |e: V3, di: i32| e.scale(across).add(dir.scale(mul(along - across, di)));
    let m = Affine {
        c0: col(v3(ONE, 0, 0), dir.x),
        c1: col(v3(0, ONE, 0), dir.y),
        c2: col(v3(0, 0, ONE), dir.z),
        t: V3::ZERO,
    };
    let t = at.sub(m.apply_dir(at));
    Some(Affine { t, ..m })
}

/// Every shape in `own`, moved by `warp` into `b`, with banana-yellow
/// sparkles swirling round the hands (`at`).
pub fn warped(b: &mut Batch, own: Batch, warp: &Affine, at: V3, tick: u32) {
    let (values, draws) = own.finish(crate::render::DYNAMIC_BUFFER);
    for d in draws.chunks(crate::render::DRAW_VALUES) {
        let mesh = Mesh::ALL[d[1] as usize];
        let first = d[2] as usize / crate::render::BYTES_PER_VALUE;
        for i in values[first..first + d[3] as usize * crate::render::INSTANCE_VALUES]
            .chunks(crate::render::INSTANCE_VALUES)
        {
            let a = Affine {
                c0: v3(i[0], i[4], i[8]),
                c1: v3(i[1], i[5], i[9]),
                c2: v3(i[2], i[6], i[10]),
                t: v3(i[3], i[7], i[11]),
            };
            b.push(mesh, &warp.then(&a), [i[12], i[13], i[14]], i[15]);
        }
    }
    for k in 0..6 {
        let a = (tick as i32).wrapping_mul(900) + k * TURN / 6;
        let p = at.add(v3(
            mul(sin(a), int(2)),
            mul(sin(a * 2), cm(80)),
            mul(fx::cos(a), int(2)),
        ));
        b.push(
            Mesh::Octa,
            &placed(p, a, v3(cm(18), cm(40), cm(18))),
            rgb(255, 220, 70),
            SKY_GLOW,
        );
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

// ---- the model gallery (`?gallery=hulk`, `deck`, `station`, `tether`) ----

/// Sector Ape's own pieces for the model gallery (`app/gallery.rs`), turned
/// by `f` about the origin.
pub fn gallery(b: &mut Batch, key: &str, f: &Affine, tick: u32) {
    match key {
        "hulk" => {
            let body = Body {
                model: Model::Hulk,
                frame: f.then(&Affine::IDENTITY),
                half: v3(int(16), int(12), int(40)),
                bouncy: false,
                pull: crate::sector::DOWN,
            };
            let door = crate::sector::Door {
                frame: f.then(&Affine::translate(v3(int(2), -int(2), int(5)))),
                half: v3(int(6), cm(450), cm(60)),
                switch: 0,
                shut_at: Some(tick.saturating_sub(DOOR_SLIDE * 2) / 600 * 600),
            };
            hulk(b, &body, &door);
            door_alone(b, &door, V3::ZERO, tick);
        }
        "deck" => {
            let pl = Platform {
                centre: f.t,
                fwd: f.c2.norm(),
                half_len: int(9),
                half_width: int(6),
                top: 0,
            };
            deck(b, &pl, DeckKind::Start);
        }
        "station" => station(b, f, tick),
        "tether" => {
            for (k, dir) in [
                crate::sector::DOWN,
                v3(ONE, 0, 0),
                v3(0, ONE, 0),
                v3(ONE, -ONE, 0).norm(),
            ]
            .into_iter()
            .enumerate()
            {
                let a = v3(int(k as i32 * 8 - 12), int(10), 0);
                let look = scene::vine_look_on(crate::map::Map::Sector, k == 3, false, tick);
                let end = a.add(dir.scale(int(9)));
                b.push(Mesh::Cyl, &rod(a, end, look.width), look.colour, look.glow);
                b.push_halo(
                    Mesh::Cyl,
                    &rod(a, end, look.halo),
                    look.colour,
                    scene::HALO_GLOW,
                );
                tether(b, &[a, end], dir, look, tick + k as u32 * 37);
            }
        }
        _ => {}
    }
}

// ---- the minimap ----

/// The routes the track splits into, for the minimap: each one's line from
/// where it splits (on the centreline) through its vines' anchors, in the
/// order they come along the course, to where it joins. The hidden route
/// (through the wreck) is left off: it is a secret to find.
pub fn route_lines(c: &Course, s: &SectorLayout) -> Vec<Vec<V3>> {
    s.routes
        .iter()
        .filter(|r| !r.hidden && !r.vines.is_empty())
        .map(|r| {
            let mut vines: Vec<usize> = r
                .vines
                .iter()
                .copied()
                .filter(|&v| v < c.vines.len())
                .collect();
            vines.sort_by_key(|&v| c.vines[v].at);
            let mut line = vec![c.point(r.from.clamp(0, c.length))];
            line.extend(vines.iter().map(|&v| c.vines[v].anchor));
            line.push(c.point(r.to.clamp(0, c.length)));
            line
        })
        .collect()
}

// ---- the menus' ape preview and the title's flyover ----

/// Round the lobby behind the menus: the backdrop, and the mothership
/// riding at anchor out in the open beyond the ape, broadside on, its
/// engines glowing, with a pair of ape wings circling it.
pub fn lobby_backdrop(b: &mut Batch, c: &Course, s: &SectorLayout, eye: V3, tick: u32) {
    backdrop(b, c, eye, tick);
    let (feet, look) = lobby_spot(c, s);
    let yaw = yaw_of(look);
    let at = feet.add(look.scale(LOBBY_SHIP_AT)).sub(v3(0, int(20), 0));
    sc::mothership(b, &sc::pose(at, yaw + deg(70), 0, deg(4), ONE), tick);
    for k in 0..2 {
        let a = (tick as i32).wrapping_mul(110) + k * fx::HALF;
        let p = at.add(v3(
            mul(sin(a), int(150)),
            int(36) + k * int(12),
            mul(fx::cos(a), int(70)),
        ));
        let heading = yaw_of(v3(fx::cos(a), 0, -mul(sin(a), ONE * 7 / 15)));
        sc::ape_wing(b, &sc::pose(p, heading, 0, -deg(30), ONE * 2), tick);
    }
}

/// How far out beyond the lobby's deck the mothership rides.
const LOBBY_SHIP_AT: i32 = int(260);

/// Where the ape stands behind the menus and the way the camera looks at
/// it (flat): on the launch deck (`SectorLayout.lobby`'s deck) out at its
/// left-hand edge, seen from the deck's middle looking out over the edge
/// into open space, on the sun's side, so behind the ape there is only
/// space, the mothership and the haze. (A heading's yaw grows to the left:
/// `geom::facing`.) With no deck, at the lobby itself,
/// looking down the course.
fn lobby_spot(c: &Course, s: &SectorLayout) -> (V3, V3) {
    let (hands, yaw) = s.lobby;
    let feet = hands.sub(v3(0, crate::rules::BODY_DROP, 0));
    let deck = c.platforms.iter().find(|pl| {
        let d = feet.sub(pl.centre);
        d.dot(pl.fwd).abs() <= pl.half_len
            && d.dot(right_of(pl.fwd)).abs() <= pl.half_width
            && (feet.y - pl.top).abs() < int(1)
    });
    match deck {
        Some(pl) => {
            let fwd = pl.fwd.flat().norm();
            let right = right_of(fwd);
            let edge = v3(pl.centre.x, pl.top, pl.centre.z)
                .sub(right.scale(pl.half_width - cm(250)))
                .sub(fwd.scale(pl.half_len / 3));
            (edge, facing(yaw_of(fwd) + LOBBY_LOOK, 0))
        }
        None => (feet, facing(yaw, 0)),
    }
}

/// The lobby's view: this far round to the left of the course's way (a
/// little back toward the start, away from the start row's girder).
const LOBBY_LOOK: i32 = deg(105);

/// Where an ape stands behind the menus (`lobby_spot`), the way it faces
/// (toward the camera) and the camera.
pub fn preview(c: &Course) -> (V3, i32, Camera) {
    let (feet, look) = match c.sector.as_deref() {
        Some(s) => lobby_spot(c, s),
        None => (c.starts[0], facing(c.start_yaw, 0)),
    };
    let eye = feet.sub(look.scale(cm(290))).add(v3(0, cm(125), 0));
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

/// The title's flyover on Sector Ape: a flight through space along the
/// course, not a flat glide. Between the features it weaves gently a few
/// metres over the spine; where the track splits it takes a route through
/// the debris (the Hulk's corridor, between the wreck's halves), flying
/// along that route's vines a little above their ends, easing in and out.
pub fn flyover_camera(c: &Course, ticks: u32, from: i32) -> Camera {
    let lap = (c.length - int(100)).max(int(1)) as i64;
    let s = ((ticks as i64 * int(9) as i64 / crate::rules::TICKS_PER_SECOND as i64 + from as i64)
        % lap) as i32
        + int(40);
    let eye = fly_line(c, s);
    let look = fly_line(c, s + int(30)).sub(v3(0, int(2), 0));
    Camera {
        eye,
        fwd: look.sub(eye).norm(),
        fov: deg(74),
    }
}

/// The flyover's line at `s` metres along: the spine, a weave, and the
/// route it takes where the track splits.
fn fly_line(c: &Course, s: i32) -> V3 {
    let p = c.point(s);
    let right = right_of(c.heading_at(s));
    let weave = (
        mul(sin(s / 3), int(5)),
        int(6) + mul(sin(s / 4 + 9_000), int(3)),
    );
    let (across, up) = match c.sector.as_deref().and_then(|l| fly_route(c, l, s)) {
        Some(((a, u), k)) => (fx::lerp(weave.0, a, k), fx::lerp(weave.1, u, k)),
        None => weave,
    };
    let q = p.add(right.scale(across));
    v3(q.x, up, q.z)
}

/// Where the flyover goes at `s` on a route through a split, if it is on
/// one: (across, height) a little above that route's vines' ends, and how
/// far it has eased onto it (0..=ONE). It takes the door's route (through
/// the Hulk's corridor), else the one furthest out (between the wreck's
/// halves); never the hidden one.
fn fly_route(c: &Course, l: &SectorLayout, s: i32) -> Option<((i32, i32), i32)> {
    let ease = int(45);
    let r = l
        .routes
        .iter()
        .filter(|r| !r.hidden && r.vines.len() >= 2 && s > r.from - ease && s < r.to + ease)
        .max_by_key(|r| {
            let out: i64 = r
                .vines
                .iter()
                .filter_map(|&v| c.vines.get(v))
                .map(|v| (v.anchor.flat().sub(c.point(v.at)).len() / ONE) as i64)
                .sum();
            (r.door, out / r.vines.len() as i64)
        })?;
    let mut pts: Vec<(i32, i32, i32)> = r
        .vines
        .iter()
        .filter(|&&v| v < c.vines.len())
        .map(|&v| {
            let tip = c.tip(v, 0);
            let (at, across) = c.progress(tip, c.nearest_anywhere(tip));
            (at, across, tip.y + int(3))
        })
        .collect();
    pts.sort_by_key(|p| p.0);
    let (first, last) = (pts[0], pts[pts.len() - 1]);
    let (a, u) = if s <= first.0 {
        (first.1, first.2)
    } else if s >= last.0 {
        (last.1, last.2)
    } else {
        let k = pts
            .windows(2)
            .position(|w| s >= w[0].0 && s < w[1].0)
            .unwrap_or(0);
        let (p0, p1) = (pts[k], pts[k + 1]);
        let t = fx::div(s - p0.0, (p1.0 - p0.0).max(1));
        (fx::lerp(p0.1, p1.1, t), fx::lerp(p0.2, p1.2, t))
    };
    // eased on before the first vine and off after the last
    let k = if s < first.0 {
        fx::div((s - (first.0 - ease)).max(0), ease)
    } else if s > last.0 {
        ONE - fx::div((s - last.0).min(ease), ease)
    } else {
        ONE
    };
    let k = mul(mul(k, k), int(3) - 2 * k);
    Some(((a, u), k))
}

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

    /// Every instance's middle in a batch.
    fn middles(b: Batch) -> Vec<V3> {
        let (v, _) = b.finish(crate::render::DYNAMIC_BUFFER);
        v.chunks(crate::render::INSTANCE_VALUES)
            .map(|i| v3(i[3], i[7], i[11]))
            .collect()
    }

    #[test]
    fn a_tether_is_drawn_the_way_its_vine_hangs() {
        // the person (2026-10-13): vines sideways, up from below, at angles
        let mut c = Map::Sector.course();
        let i = c.vines.len() / 2;
        for dir in [v3(ONE, 0, 0), v3(0, ONE, 0), v3(-ONE, ONE, 0).norm()] {
            c.sector.as_mut().unwrap().hangs[i] = dir;
            let a = c.anchor(i, 0);
            let len = c.vines[i].length;
            let (vines, _) = scene::flyover_dynamic(&c, 0, false, a);
            let far = a.add(dir.scale(len * 3 / 4));
            let near = middles(vines)
                .into_iter()
                .map(|m| m.dist(far))
                .min()
                .unwrap();
            assert!(near < int(1), "{dir:?}: {} m off", near / ONE);
        }
    }

    #[test]
    fn the_lobby_ape_stands_on_its_deck_with_the_mothership_behind() {
        let c = Map::Sector.course();
        let s = c.sector.as_ref().unwrap();
        let (feet, _, cam) = preview(&c);
        // on the launch deck, standing on its top, out near its edge
        let deck = &c.platforms[0];
        assert_eq!(feet.y, deck.top);
        let side = feet.sub(deck.centre).dot(right_of(deck.fwd)).abs();
        assert!(
            side > deck.half_width / 2 && side < deck.half_width,
            "{}",
            side / ONE
        );
        // the menus' preview uses it (`scene::preview`)
        assert_eq!(scene::preview(&c).feet, feet);
        // the mothership is in the lobby's backdrop, beyond the ape
        let mut b = Batch::new();
        lobby_backdrop(&mut b, &c, s, cam.eye, 0);
        let ahead = cam.fwd.flat().norm();
        let beyond = middles(b)
            .into_iter()
            .filter(|m| m.sub(feet).dot(ahead) > int(150) && m.dist(feet) < int(400))
            .count();
        assert!(beyond > 40, "{beyond}");
    }

    #[test]
    fn a_finished_ape_near_the_black_hole_is_drawn_stretching_into_it() {
        let c = Map::Sector.course();
        let s = c.sector.as_ref().unwrap();
        let e = crate::race::Entry {
            name: "P".into(),
            kind: crate::rules::Kind::Gorilla,
            driver: crate::race::Driver::Local,
            palette: 0,
        };
        let mut race = crate::race::Race::new(c.clone(), &[e], false, 1);
        let (hole, r) = s.black_hole;
        let at = hole.add(v3(0, 0, -mul(r, int(2))));
        assert_eq!(stretch(s, &race.racers[0], at), None, "not finished");
        race.racers[0].finish = Some(100);
        let w = stretch(s, &race.racers[0], at).expect("drawn in");
        // the hands stay put; a point toward the hole moves further toward
        // it; one across moves in
        assert!(w.apply(at).dist(at) < cm(2));
        let toward = at.add(hole.sub(at).with_len(ONE));
        assert!(w.apply(toward).dist(at) > ONE + ONE / 10);
        let side = at.add(v3(ONE, 0, 0));
        assert!(w.apply(side).dist(at) < ONE - ONE / 20);
        // far from it, as ever
        assert_eq!(
            stretch(s, &race.racers[0], hole.add(v3(0, 0, -mul(r, int(5))))),
            None
        );
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
