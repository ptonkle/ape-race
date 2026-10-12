//! Drawing the race (PLAN P2, P7, P10): the course's ground and scenery,
//! uploaded once; and every frame the vines, logs, apes, items and the
//! speed effects. Flat colours, with the vines kept bright and clear of
//! clutter (brief section 10: decorations must not hide what an ape grabs).

use crate::course::{ground, Course, Prop};
use crate::fx::{self, cm, deg, int, mul, sin, ONE};
use crate::geom::{right_of, v3, Affine, V3};
use crate::mesh::{Mesh, VERTEX_VALUES};
use crate::models::{self, Pose};
use crate::race::Race;
use crate::render::{mix, placed, rgb, rod, shade, Batch, Camera, Rgb};
use crate::rules::BODY_DROP;

pub const SKY: Rgb = rgb(150, 206, 196);
pub const FOG: Rgb = rgb(120, 178, 150);
const GROUND_A: Rgb = rgb(78, 128, 52);
const GROUND_B: Rgb = rgb(98, 146, 58);
const PATH: Rgb = rgb(128, 120, 70);
const BARK: [Rgb; 4] = [
    rgb(96, 66, 44),
    rgb(110, 78, 50),
    rgb(84, 62, 46),
    rgb(120, 92, 60),
];
const LEAF: [Rgb; 4] = [
    rgb(46, 112, 50),
    rgb(60, 132, 52),
    rgb(38, 96, 56),
    rgb(76, 140, 60),
];
const VINE: Rgb = rgb(150, 210, 70);
const VINE_REACH: Rgb = rgb(255, 236, 90);
const VINE_LEAF: Rgb = rgb(90, 170, 60);
const FLOWERS: [Rgb; 4] = [
    rgb(240, 90, 120),
    rgb(250, 200, 60),
    rgb(170, 110, 230),
    rgb(250, 140, 60),
];
const ROCK: Rgb = rgb(128, 132, 124);
const WOOD: Rgb = rgb(150, 104, 62);
const BANNER: Rgb = rgb(240, 236, 220);
const BANNER_DARK: Rgb = rgb(40, 40, 46);
const SHADOW: Rgb = rgb(40, 70, 34);

/// The light: its direction (toward the sun), its colour; the sky's and the
/// fog's colours; the ground's bounce. Uploaded with the scene.
pub fn uniforms() -> Vec<i32> {
    let dir = v3(cm(40), ONE, cm(25)).norm();
    let light = rgb(255, 244, 214);
    let mut v = vec![dir.x, dir.y, dir.z];
    v.extend(light);
    v.extend(SKY);
    v.extend(FOG);
    v.extend(rgb(70, 110, 50));
    v
}

pub fn clear_colour() -> Vec<i32> {
    FOG.to_vec()
}

/// The fog's start and end, in metres from the eye (PLAN P10: about 150 m).
pub fn fog_range() -> (i32, i32) {
    (int(60), int(170))
}

/// The ground under the course: a strip of triangles following the
/// centreline, out past the trees, with a worn track down the middle.
pub fn terrain(c: &Course) -> Vec<i32> {
    terrain_chunked(c, c.path.len().div_ceil(CHUNK)).0
}

/// The terrain, and for each of `n` chunks the vertex ranges that lie in
/// it: (chunk, first vertex, vertex count).
fn terrain_chunked(c: &Course, n: usize) -> (Vec<i32>, Vec<(usize, u32, u32)>) {
    let mut o = Vec::new();
    let mut ranges: Vec<(usize, u32, u32)> = Vec::new();
    let verts = |o: &Vec<i32>| (o.len() / VERTEX_VALUES) as u32;
    // columns across the course, metres from the centreline
    let across: [i32; 11] = [-70, -45, -30, -20, -10, 0, 10, 20, 30, 45, 70];
    let every = 3usize;
    let rows: Vec<usize> = (0..c.path.len()).step_by(every).collect();
    let point = |i: usize, k: usize| {
        let p = c.path[i];
        let r = right_of(c.heading[i]);
        let q = p.add(r.scale(int(across[k])));
        v3(q.x, ground(q.x, q.z), q.z)
    };
    let tint = |p: V3, k: usize, row: usize| {
        let m = (sin(mul(p.x, 2000)) + sin(mul(p.z, 1700)) + 2 * ONE) / 4;
        let mut t = mix(GROUND_A, GROUND_B, m);
        if across[k].abs() <= 10 && (row.is_multiple_of(2) || across[k] == 0) {
            t = mix(t, PATH, ONE / 3);
        }
        t
    };
    let normal = |p: V3| {
        let e = ONE / 2;
        let dx = ground(p.x + e, p.z) - ground(p.x - e, p.z);
        let dz = ground(p.x, p.z + e) - ground(p.x, p.z - e);
        v3(-dx, ONE, -dz).norm()
    };
    // the start of the strip reaches back behind the start tree
    for w in 0..rows.len() - 1 {
        let (i0, i1) = (rows[w], rows[w + 1]);
        let chunk = (i0 / CHUNK).min(n - 1);
        match ranges.last() {
            Some((k, _, _)) if *k == chunk => {}
            _ => ranges.push((chunk, verts(&o), 0)),
        }
        for k in 0..across.len() - 1 {
            let a = point(i0, k);
            let b = point(i0, k + 1);
            let cc = point(i1, k + 1);
            let d = point(i1, k);
            let ta = tint(a, k, w);
            let tb = tint(b, k + 1, w);
            let tc = tint(cc, k + 1, w + 1);
            let td = tint(d, k, w + 1);
            push_ground(&mut o, [a, d, cc], [ta, td, tc], normal);
            push_ground(&mut o, [a, cc, b], [ta, tc, tb], normal);
        }
        if let Some(last) = ranges.last_mut() {
            last.2 = verts(&o) - last.1;
        }
    }
    // a skirt of ground behind the start
    let skirt = verts(&o);
    let back = c.path[0].sub(c.heading[0].scale(int(60)));
    let r = right_of(c.heading[0]);
    let s0 = c.path[0].add(r.scale(int(-70)));
    let s1 = c.path[0].add(r.scale(int(70)));
    let b0 = back.add(r.scale(int(-70)));
    let b1 = back.add(r.scale(int(70)));
    let lift = |p: V3| v3(p.x, ground(p.x, p.z), p.z);
    push_ground(
        &mut o,
        [lift(b0), lift(s0), lift(s1)],
        [GROUND_A; 3],
        normal,
    );
    push_ground(
        &mut o,
        [lift(b0), lift(s1), lift(b1)],
        [GROUND_A; 3],
        normal,
    );
    ranges.push((0, skirt, verts(&o) - skirt));
    // and past the finish
    let finish = verts(&o);
    let last = c.path.len() - 1;
    let end = c.path[last];
    let past = end.add(c.heading[last].scale(int(80)));
    let r = right_of(c.heading[last]);
    let e0 = end.add(r.scale(int(-70)));
    let e1 = end.add(r.scale(int(70)));
    let f0 = past.add(r.scale(int(-70)));
    let f1 = past.add(r.scale(int(70)));
    push_ground(
        &mut o,
        [lift(e0), lift(f0), lift(f1)],
        [GROUND_B; 3],
        normal,
    );
    push_ground(
        &mut o,
        [lift(e0), lift(f1), lift(e1)],
        [GROUND_B; 3],
        normal,
    );
    ranges.push((n - 1, finish, verts(&o) - finish));
    debug_assert_eq!(o.len() % (VERTEX_VALUES * 3), 0);
    (o, ranges)
}

fn push_ground(o: &mut Vec<i32>, p: [V3; 3], t: [Rgb; 3], normal: impl Fn(V3) -> V3) {
    // make sure the triangle faces up
    let up = p[1].sub(p[0]).cross(p[2].sub(p[0]));
    let (p, t) = if up.y < 0 {
        ([p[0], p[2], p[1]], [t[0], t[2], t[1]])
    } else {
        (p, t)
    };
    for k in 0..3 {
        let n = normal(p[k]);
        o.extend_from_slice(&[
            p[k].x, p[k].y, p[k].z, n.x, n.y, n.z, t[k][0], t[k][1], t[k][2],
        ]);
    }
}

/// Scenery chunks are this many centreline samples long (100 m).
pub const CHUNK: usize = 50;

/// The course's scenery that never moves, in chunks along the course so
/// the frame draws only those near the camera (the fog hides the rest).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scenery {
    pub values: Vec<i32>,
    pub terrain: Vec<i32>,
    pub chunks: Vec<Chunk>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Chunk {
    pub centre: V3,
    pub draws: Vec<u32>,
}

/// Chunks whose middle is further than this from the eye are not drawn.
pub const DRAW_DISTANCE: i32 = int(260);

impl Scenery {
    /// The draws of every chunk near `eye`.
    pub fn draws_near(&self, eye: V3) -> Vec<u32> {
        let mut out = Vec::new();
        for ch in &self.chunks {
            if ch.centre.flat().sub(eye.flat()).len() <= DRAW_DISTANCE {
                out.extend_from_slice(&ch.draws);
            }
        }
        out
    }

    /// A scenery of one chunk that is always drawn.
    pub fn single(b: Batch) -> Scenery {
        let mut values = Vec::new();
        let draws = b.finish_into(crate::render::STATIC_BUFFER, &mut values);
        Scenery {
            values,
            terrain: Vec::new(),
            chunks: vec![Chunk {
                centre: V3::ZERO,
                draws,
            }],
        }
    }
}

/// Builds the course's scenery and ground, chunked.
pub fn scenery(c: &Course) -> Scenery {
    let n = c.path.len().div_ceil(CHUNK);
    let mut batches: Vec<Batch> = (0..n).map(|_| Batch::new()).collect();
    let chunk_of = |p: V3| (c.nearest_anywhere(p) / CHUNK).min(n - 1);
    for p in &c.props {
        let at = match *p {
            Prop::Tree { at, .. } | Prop::Bush { at, .. } | Prop::Fern { at, .. } => at,
            Prop::Flower { at, .. } | Prop::Rock { at, .. } => at,
            Prop::Branch { to, .. } => to,
        };
        prop(&mut batches[chunk_of(at)], p);
    }
    for pl in &c.platforms {
        start_tree(
            &mut batches[chunk_of(pl.centre)],
            pl.centre,
            pl.fwd,
            pl.half_len,
            pl.half_width,
        );
    }
    finish_arch(&mut batches[n - 1], c);
    // the terrain's one instance comes first; its draws name vertex ranges
    let mut values = Vec::new();
    let mut identity = Batch::new();
    identity.push(Mesh::Terrain, &Affine::IDENTITY, [ONE; 3], 0);
    identity.finish_into(crate::render::STATIC_BUFFER, &mut values);
    let (terrain, ranges) = terrain_chunked(c, n);
    let mut chunks = Vec::new();
    for (k, b) in batches.into_iter().enumerate() {
        let mut draws = b.finish_into(crate::render::STATIC_BUFFER, &mut values);
        for (chunk, first, count) in &ranges {
            if *chunk == k && *count > 0 {
                draws.extend_from_slice(&[
                    crate::render::STATIC_BUFFER,
                    Mesh::Terrain as u32,
                    0,
                    1,
                    *first,
                    *count,
                ]);
            }
        }
        let mid = (k * CHUNK + CHUNK / 2).min(c.path.len() - 1);
        chunks.push(Chunk {
            centre: c.path[mid],
            draws,
        });
    }
    Scenery {
        values,
        terrain,
        chunks,
    }
}

fn prop(b: &mut Batch, p: &Prop) {
    match *p {
        Prop::Tree {
            at,
            radius,
            height,
            shade: s,
        } => tree(b, at, radius, height, s),
        Prop::Branch { from, to } => {
            b.push(Mesh::Cyl, &rod(from, to, cm(55)), BARK[1], 0);
            // a few leaves along it
            for k in 1..4 {
                let p = from.lerp(to, ONE * k / 4).add(v3(0, cm(40), 0));
                b.push(
                    Mesh::Blob,
                    &placed(p, k * 9000, v3(cm(180), cm(90), cm(160))),
                    LEAF[(k as usize) % 4],
                    0,
                );
            }
        }
        Prop::Bush { at, size, shade: s } => {
            b.push(
                Mesh::Blob,
                &placed(
                    at.add(v3(0, size / 3, 0)),
                    size,
                    v3(size * 2, size, size * 2),
                ),
                LEAF[s as usize % 4],
                0,
            );
        }
        Prop::Fern { at, size, yaw } => {
            for k in 0..5 {
                let a = yaw + k * deg(72);
                let tip = at.add(v3(mul(fx::sin(a), size), size / 2, mul(fx::cos(a), size)));
                b.push(
                    Mesh::Cube,
                    &rod(at, tip, size / 5),
                    LEAF[(k as usize + 1) % 4],
                    0,
                );
            }
        }
        Prop::Flower { at, colour } => {
            b.push(
                Mesh::Cyl,
                &placed(at.add(v3(0, cm(25), 0)), 0, v3(cm(5), cm(50), cm(5))),
                LEAF[1],
                0,
            );
            b.push(
                Mesh::Blob,
                &placed(at.add(v3(0, cm(55), 0)), 0, v3(cm(30), cm(18), cm(30))),
                FLOWERS[colour as usize % 4],
                ONE / 3,
            );
        }
        Prop::Rock { at, size, yaw } => {
            b.push(
                Mesh::Blob,
                &placed(
                    at.add(v3(0, size / 4, 0)),
                    yaw,
                    v3(size * 3 / 2, size, size),
                ),
                ROCK,
                0,
            );
        }
    }
}

fn tree(b: &mut Batch, at: V3, radius: i32, height: i32, s: u8) {
    let bark = BARK[s as usize % 4];
    b.push(
        Mesh::Cyl,
        &placed(
            at.add(v3(0, height / 2, 0)),
            0,
            v3(radius * 2, height, radius * 2),
        ),
        bark,
        0,
    );
    // buttress roots
    for k in 0..3 {
        let a = k * deg(120) + radius;
        let foot = at.add(v3(
            mul(fx::sin(a), radius * 3),
            0,
            mul(fx::cos(a), radius * 3),
        ));
        b.push(
            Mesh::Cone,
            &rod(foot, at.add(v3(0, radius * 3, 0)), radius),
            shade(bark, 90),
            0,
        );
    }
    // the crown: a cluster of leaf blobs, high above the vines
    let top = at.add(v3(0, height, 0));
    let leaf = LEAF[(s as usize + 1) % 4];
    b.push(
        Mesh::Blob,
        &placed(top, radius * 7, v3(int(14), int(7), int(14))),
        leaf,
        0,
    );
    for k in 0..3 {
        let a = k * deg(120) + radius * 3;
        let p = top.add(v3(
            mul(fx::sin(a), int(6)),
            -int(2),
            mul(fx::cos(a), int(6)),
        ));
        b.push(
            Mesh::Blob,
            &placed(p, a, v3(int(9), int(5), int(9))),
            shade(leaf, 85 + k * 8),
            0,
        );
    }
}

fn start_tree(b: &mut Batch, centre: V3, fwd: V3, half_len: i32, half_width: i32) {
    let right = right_of(fwd);
    let yaw = crate::geom::yaw_of(fwd);
    // the deck
    b.push(
        Mesh::Cube,
        &placed(
            centre.sub(v3(0, cm(40), 0)),
            yaw,
            v3(half_width * 2, cm(80), half_len * 2),
        ),
        WOOD,
        0,
    );
    // planks
    let mut k = -half_len + cm(60);
    while k < half_len {
        let p = centre.add(fwd.scale(k)).add(v3(0, cm(2), 0));
        b.push(
            Mesh::Cube,
            &placed(p, yaw, v3(half_width * 2, cm(4), cm(12))),
            shade(WOOD, 75),
            0,
        );
        k += cm(120);
    }
    // the trunk under it, and posts
    let gy = ground(centre.x, centre.z);
    b.push(
        Mesh::Cyl,
        &placed(
            v3(centre.x, (gy + centre.y) / 2, centre.z),
            0,
            v3(int(4), centre.y - gy, int(4)),
        ),
        BARK[0],
        0,
    );
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            let foot = centre
                .add(right.scale(sx * (half_width - cm(50))))
                .add(fwd.scale(sz * (half_len - cm(50))));
            b.push(
                Mesh::Cyl,
                &rod(v3(foot.x, gy, foot.z), v3(foot.x, centre.y, foot.z), cm(60)),
                BARK[2],
                0,
            );
        }
    }
}

fn finish_arch(b: &mut Batch, c: &Course) {
    let n = c.path.len() - 1;
    let at = c.path[n];
    let fwd = c.heading[n];
    let right = right_of(fwd);
    let yaw = crate::geom::yaw_of(fwd);
    let gy = ground(at.x, at.z);
    let span = c.half_width;
    for s in [-1, 1] {
        let foot = at.add(right.scale(s * span));
        b.push(
            Mesh::Cyl,
            &placed(
                v3(foot.x, gy + int(20), foot.z),
                0,
                v3(int(2), int(40), int(2)),
            ),
            BARK[3],
            0,
        );
    }
    // a chequered banner across the top, and a glowing line on the ground
    let cells = 16;
    for k in 0..cells {
        for row in 0..2 {
            let x = -span + (2 * span) * k / cells + span / cells;
            let p = at
                .add(right.scale(x))
                .add(v3(0, gy + int(34) + row * int(2), 0));
            let colour = if (k + row) % 2 == 0 {
                BANNER
            } else {
                BANNER_DARK
            };
            b.push(
                Mesh::Cube,
                &placed(p, yaw, v3(2 * span / cells, int(2), cm(40))),
                colour,
                ONE / 3,
            );
        }
    }
    b.push(
        Mesh::Cube,
        &placed(
            v3(at.x, gy + cm(5), at.z),
            yaw,
            v3(span * 2, cm(10), cm(80)),
        ),
        BANNER,
        ONE / 2,
    );
}

/// What the frame draws, besides the scenery.
pub struct Frame<'a> {
    pub race: &'a Race,
    pub cam: &'a Camera,
    /// each ape's drawn position (blended between ticks)
    pub apes: &'a [V3],
    /// the local ape, whose vine in reach is lit up
    pub local: Option<usize>,
    /// ticks, for animation
    pub tick: u32,
}

/// Everything that moves.
pub fn dynamic(f: &Frame) -> Batch {
    let mut b = Batch::new();
    let race = f.race;
    let c = &race.course;
    let tick = race.tick;
    let eye = f.cam.eye;
    let reach = f.local.and_then(|i| {
        let a = &race.racers[i].ape;
        if a.swinging().is_none() && a.cooldown == 0 && a.stun == 0 {
            a.vine_in_reach(c, tick)
        } else {
            None
        }
    });
    // who holds which vine, and where its hands are
    let mut held: Vec<Option<V3>> = vec![None; c.vines.len()];
    for (k, r) in race.racers.iter().enumerate() {
        if let Some(v) = r.ape.swinging() {
            held[v] = Some(f.apes[k]);
        }
    }
    for i in 0..c.vines.len() {
        let a = c.anchor(i, tick);
        if a.flat().sub(eye.flat()).len() > int(230) {
            continue;
        }
        let colour = if Some(i) == reach { VINE_REACH } else { VINE };
        let glow = if Some(i) == reach {
            ONE * 7 / 10
        } else {
            ONE / 5
        };
        vine(
            &mut b,
            a,
            c.vines[i].length,
            held[i],
            colour,
            glow,
            f.tick + i as u32 * 37,
        );
    }
    for i in 0..c.logs.len() {
        let l = &c.logs[i];
        let m = c.log_centre(i, tick);
        let yaw = crate::geom::yaw_of(l.along);
        b.push(
            Mesh::Cyl,
            &Affine::translate(m)
                .then(&Affine::rot_y(yaw))
                .then(&Affine::rot_x(deg(90)))
                .then(&Affine::scale(v3(
                    l.radius * 2,
                    l.half_len * 2,
                    l.radius * 2,
                ))),
            BARK[1],
            0,
        );
        for s in [-1, 1] {
            let end = m.add(l.along.scale(s * (l.half_len - cm(50))));
            let top = l.pivot.add(l.along.scale(s * (l.half_len - cm(50))));
            b.push(Mesh::Cyl, &rod(end, top, cm(12)), VINE, ONE / 5);
        }
    }
    for (k, r) in race.racers.iter().enumerate() {
        let at = f.apes[k];
        let rope = r.ape.swinging().map(|v| c.anchor(v, tick).sub(at).norm());
        let pose = Pose {
            hands: at,
            yaw: r.ape.yaw,
            rope,
            vel: r.ape.vel,
            grounded: r.ape.mode == crate::ape::Mode::Grounded,
            stunned: r.ape.stun > 0,
            tick: f.tick + k as u32 * 53,
            scale: 100,
        };
        models::draw(&mut b, r.ape.kind, &pose);
        // a soft shadow on the ground, so height reads
        let support = c.support(at);
        let feet = at.y - BODY_DROP;
        let high = (feet - support).clamp(0, int(30));
        let size = cm(140) - high * 2 / 60;
        b.push(
            Mesh::Cyl,
            &placed(
                v3(at.x, support + cm(4), at.z),
                0,
                v3(size.max(cm(60)), cm(2), size.max(cm(60))),
            ),
            SHADOW,
            0,
        );
    }
    crate::items::draw(&mut b, race);
    speed_lines(&mut b, f);
    b
}

/// A vine from its anchor: straight down and swaying a little; or, when
/// held, taut from the anchor to the hands with the rest dangling below.
fn vine(b: &mut Batch, a: V3, length: i32, hands: Option<V3>, colour: Rgb, glow: i32, t: u32) {
    let width = cm(14);
    match hands {
        Some(h) => {
            let rope = h.sub(a).len().min(length);
            b.push(Mesh::Cyl, &rod(a, h, width), colour, glow);
            let rest = length - rope;
            if rest > cm(30) {
                let end = h.sub(v3(0, rest, 0));
                b.push(Mesh::Cyl, &rod(h, end, width), colour, glow);
            }
            leaves(b, a, h, t);
        }
        None => {
            // sway: the bottom drifts a little, in two directions
            let s = sin((t as i32) * 90);
            let s2 = sin((t as i32) * 70 + 9000);
            let bottom = a.add(v3(mul(s, cm(50)), -length, mul(s2, cm(40))));
            let mid = a.lerp(bottom, ONE / 2).add(v3(mul(s, cm(15)), 0, 0));
            b.push(Mesh::Cyl, &rod(a, mid, width), colour, glow);
            b.push(Mesh::Cyl, &rod(mid, bottom, width), colour, glow);
            // a knot at the bottom to grab
            b.push(
                Mesh::Sphere,
                &placed(bottom, 0, v3(cm(34), cm(34), cm(34))),
                colour,
                glow,
            );
            leaves(b, a, bottom, t);
        }
    }
}

fn leaves(b: &mut Batch, a: V3, end: V3, t: u32) {
    let n = 4;
    for k in 1..n {
        let p = a.lerp(end, ONE * k / n);
        let side = if k % 2 == 0 { cm(18) } else { -cm(18) };
        b.push(
            Mesh::Blob,
            &placed(
                p.add(v3(side, 0, 0)),
                (t as i32) * 40 + k * 7000,
                v3(cm(40), cm(14), cm(26)),
            ),
            VINE_LEAF,
            0,
        );
    }
}

/// Streaks past the camera at speed (PLAN P10).
fn speed_lines(b: &mut Batch, f: &Frame) {
    let Some(i) = f.local else { return };
    let ape = &f.race.racers[i].ape;
    let speed = ape.speed();
    let from = int(24);
    if speed < from {
        return;
    }
    let k = fx::div((speed - from).min(int(20)), int(20));
    let dir = ape.vel.norm();
    let right = right_of(f.cam.fwd);
    let up = right.cross(f.cam.fwd).norm();
    let count = 4 + mul(k, int(10)) / ONE;
    let mut d = fx::Dice(f.tick as u64 / 3);
    for _ in 0..count {
        let x = int(d.range(-6, 6));
        let y = int(d.range(-4, 4));
        let z = int(d.range(4, 16));
        let p = f
            .cam
            .eye
            .add(f.cam.fwd.scale(z))
            .add(right.scale(x))
            .add(up.scale(y));
        if x.abs() < int(2) && y.abs() < int(2) {
            continue;
        }
        let len = cm(150) + mul(k, int(3));
        b.push(
            Mesh::Cube,
            &rod(p, p.sub(dir.scale(len)), cm(3)),
            rgb(255, 255, 255),
            ONE,
        );
    }
}

/// The little stage the selection screen and the lineup stand on.
pub fn stage_set() -> Scenery {
    Scenery::single(stage_batch())
}

fn stage_batch() -> Batch {
    let mut b = Batch::new();
    b.push(
        Mesh::Cyl,
        &placed(v3(0, -cm(10), 0), 0, v3(int(14), cm(20), int(8))),
        rgb(92, 140, 60),
        0,
    );
    b.push(
        Mesh::Cyl,
        &placed(v3(0, -int(2), 0), 0, v3(int(200), int(3), int(200))),
        GROUND_A,
        0,
    );
    // a backdrop of jungle
    for k in 0..14 {
        let x = int(-26) + k * int(4);
        let z = -int(10) - int((k * 7) % 5);
        tree(&mut b, v3(x, -cm(10), z), cm(90), int(16), (k % 4) as u8);
    }
    for k in 0..9 {
        let x = int(-8) + k * int(2);
        b.push(
            Mesh::Blob,
            &placed(
                v3(x, cm(20), -int(3) - (k % 3) * ONE),
                k * 3000,
                v3(int(2), ONE, int(2)),
            ),
            LEAF[(k % 4) as usize],
            0,
        );
    }
    b
}

/// What moves behind the title: the vines, swaying.
pub fn flyover_dynamic(c: &Course, tick: u32) -> Batch {
    let mut b = Batch::new();
    for i in 0..c.vines.len() {
        let a = c.anchor(i, tick);
        vine(
            &mut b,
            a,
            c.vines[i].length,
            None,
            VINE,
            ONE / 5,
            tick + i as u32 * 37,
        );
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_terrain_is_whole_upward_triangles() {
        let c = Course::jungle();
        let t = terrain(&c);
        assert_eq!(t.len() % (VERTEX_VALUES * 3), 0);
        assert!(t.len() > 1000);
        for tri in t.chunks(VERTEX_VALUES * 3) {
            let p = |k: usize| {
                v3(
                    tri[k * VERTEX_VALUES],
                    tri[k * VERTEX_VALUES + 1],
                    tri[k * VERTEX_VALUES + 2],
                )
            };
            let up = p(1).sub(p(0)).cross(p(2).sub(p(0)));
            assert!(up.y >= 0);
        }
    }

    #[test]
    fn the_scenery_is_a_few_thousand_shapes() {
        let c = Course::jungle();
        let s = scenery(&c);
        let n = s.values.len() / crate::render::INSTANCE_VALUES;
        assert!(n > 1000 && n < 20_000, "{n}");
        assert_eq!(s.chunks.len(), c.path.len().div_ceil(CHUNK));
        // every terrain vertex is drawn by exactly one chunk
        let mut drawn = 0;
        for ch in &s.chunks {
            for d in ch.draws.chunks(crate::render::DRAW_VALUES) {
                if d[1] == Mesh::Terrain as u32 {
                    drawn += d[5];
                }
            }
        }
        assert_eq!(drawn as usize, s.terrain.len() / VERTEX_VALUES);
        // near the start only a few chunks are drawn
        let near = s.draws_near(c.path[0]);
        let all: usize = s.chunks.iter().map(|ch| ch.draws.len()).sum();
        assert!(near.len() * 3 < all, "{} of {all}", near.len());
    }
}
