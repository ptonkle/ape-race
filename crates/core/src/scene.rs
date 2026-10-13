//! Drawing the race (PLAN P2, P7, P10): the course's ground and scenery,
//! uploaded once; and every frame the vines, apes, items and the speed
//! effects. Flat colours, with the vines kept bright and clear of
//! clutter (brief section 10: decorations must not hide what an ape grabs).

use crate::course::{
    crown_scale, ground, rise, Course, Mushroom, Prop, TreeKind, GROW, OUTRO, STEP,
};
use crate::fx::{self, cm, deg, int, mul, sin, ONE};
use crate::geom::{right_of, v3, Affine, V3};
use crate::map::Map;
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
const VINE_USED: Rgb = rgb(240, 84, 72);
const BANANA_YELLOW: Rgb = rgb(255, 216, 40);
const BANANA_STALK: Rgb = rgb(92, 62, 30);
const VINE_LEAF: Rgb = rgb(90, 170, 60);
/// The colourblind mode's vines (TRIAGE T33): a vivid green, brighter than
/// anything in the muted world, and in reach near white, thicker and
/// pulsing (brightness and shape, not green to yellow).
const VINE_VIVID: Rgb = rgb(110, 255, 70);
const VINE_REACH_VIVID: Rgb = rgb(255, 255, 240);
const VINE_WIDTH: i32 = cm(20);
const VINE_WIDTH_VIVID: i32 = cm(30);
/// An item vine's flowers (TRIAGE T46): violet petals round a white heart,
/// apart from the in-reach yellow and from red; told apart by shape (five
/// petals) and count (three up the vine) too, for the colourblind mode.
const PETAL: Rgb = rgb(160, 96, 255);
const HEART: Rgb = rgb(255, 252, 236);
const BUD: Rgb = rgb(96, 120, 80);
const FLOWERS: [Rgb; 4] = [
    rgb(240, 90, 120),
    rgb(250, 200, 60),
    rgb(170, 110, 230),
    rgb(250, 140, 60),
];
const ROCK: Rgb = rgb(128, 132, 124);
const MOSS: Rgb = rgb(124, 176, 48);
const LITTER: Rgb = rgb(150, 112, 52);
const MUD: Rgb = rgb(104, 80, 50);
const SLOPE: Rgb = rgb(52, 116, 64);
const HILL: [Rgb; 4] = [
    rgb(40, 118, 76),
    rgb(58, 138, 64),
    rgb(30, 100, 82),
    rgb(84, 148, 58),
];
const EMERGENT: Rgb = rgb(96, 164, 48);
const PALM: Rgb = rgb(118, 192, 58);
const PALM_BARK: Rgb = rgb(150, 126, 88);
const FIG_BARK: Rgb = rgb(160, 148, 116);
const FIG_LEAF: Rgb = rgb(34, 124, 84);
const BAMBOO: Rgb = rgb(176, 202, 76);
const WATER: Rgb = rgb(64, 182, 214);
const WATERFALL: Rgb = rgb(186, 232, 248);
const CAP: Rgb = rgb(238, 58, 96);
const SPOT: Rgb = rgb(255, 246, 236);
const STALK: Rgb = rgb(244, 228, 196);
const GOLD: Rgb = rgb(250, 200, 60);
const SILVER: Rgb = rgb(206, 212, 224);
const BRONZE: Rgb = rgb(206, 128, 70);
const BANANA: Rgb = rgb(255, 214, 56);
const WOOD: Rgb = rgb(150, 104, 62);
const BANNER: Rgb = rgb(240, 236, 220);
const BANNER_DARK: Rgb = rgb(40, 40, 46);
const SHADOW: Rgb = rgb(34, 40, 30);

/// How a map is lit: the sky's and the fog's colours, where the fog
/// starts and ends, the light's direction (toward the sun) and colour, and
/// the ground's bounce. The Jungle at midday; the City at dusk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Look {
    pub sky: Rgb,
    pub fog: Rgb,
    pub fog_range: (i32, i32),
    pub light_dir: V3,
    pub light: Rgb,
    pub ground: Rgb,
}

/// The look of a map.
pub fn look(map: Map) -> Look {
    match map {
        Map::Jungle => Look {
            sky: SKY,
            fog: FOG,
            // PLAN P10: about 150 m
            fog_range: (int(60), int(170)),
            light_dir: v3(cm(55), cm(80), cm(35)).norm(),
            light: rgb(255, 244, 214),
            ground: rgb(70, 110, 50),
        },
        Map::City => crate::cityscape::look(),
    }
}

/// The look of the map a course was built for.
pub fn look_of(c: &Course) -> Look {
    look(Map::of(c))
}

/// The light: its direction (toward the sun), its colour; the sky's and the
/// fog's colours; the ground's bounce. Uploaded with the scene (the
/// Jungle's; each frame then sends its own map's light, `Look`).
pub fn uniforms() -> Vec<i32> {
    let l = look(Map::Jungle);
    let mut v = vec![l.light_dir.x, l.light_dir.y, l.light_dir.z];
    v.extend(l.light);
    v.extend(l.sky);
    v.extend(l.fog);
    v.extend(l.ground);
    v
}

pub fn clear_colour() -> Vec<i32> {
    FOG.to_vec()
}

/// The fog's start and end, in metres from the eye (PLAN P10: about 150 m).
pub fn fog_range() -> (i32, i32) {
    look(Map::Jungle).fog_range
}

/// The ground under the course: a strip of faceted triangles following the
/// centreline, from behind the start tree to the cliff past the finish
/// (TRIAGE T6), out past the trees and up the valley's walls (T16), with a
/// worn track down the middle and patches of moss, leaf litter and mud.
pub fn terrain(c: &Course) -> Vec<i32> {
    terrain_chunked(c, chunk_count(c)).0
}

/// The course's chunks: one per `CHUNK` centreline samples, and one more
/// for the clearing past the finish.
pub(crate) fn chunk_count(c: &Course) -> usize {
    c.path.len().div_ceil(CHUNK) + 1
}

/// The chunk that a point `s` metres along the course falls in.
fn chunk_along(c: &Course, s: i32, n: usize) -> usize {
    if s > c.length {
        n - 1
    } else {
        ((s.max(0) / STEP) as usize / CHUNK).min(n - 2)
    }
}

/// The ground's colour for a facet at `p`, `a` metres across the course.
fn ground_tint(p: V3, a: i32, s: i32) -> Rgb {
    let m = (sin(mul(p.x, 2000)) + sin(mul(p.z, 1700)) + 2 * ONE) / 4;
    let mut t = mix(GROUND_A, GROUND_B, m);
    // patches of moss, leaf litter and mud
    let patch = mul(sin(mul(p.x, 610) + 9000), fx::cos(mul(p.z, 530)));
    if patch > ONE * 11 / 20 {
        t = mix(t, MOSS, ONE * 2 / 3);
    } else if patch < -ONE * 3 / 5 {
        t = mix(t, LITTER, ONE / 2);
    }
    if sin(mul(p.x, 900) + mul(p.z, 330)) > ONE * 17 / 20 && a.abs() < int(40) {
        t = mix(t, MUD, ONE * 3 / 5);
    }
    let a = a.abs();
    if a <= int(9) {
        t = mix(t, PATH, ONE / 4);
    }
    // the valley's walls: forest green, bare rock near the top
    if a > int(45) {
        let up = rise(s, int(120));
        let h = p.y - ground(p.x, p.z);
        t = mix(t, SLOPE, fx::div(a - int(45), int(40)).min(ONE));
        if up > 0 && h > int(18) {
            t = mix(t, ROCK, fx::div(h - int(18), int(20)).min(ONE) * 3 / 4);
        }
    }
    t
}

/// The terrain, and for each of `n` chunks the vertex ranges that lie in
/// it: (chunk, first vertex, vertex count).
fn terrain_chunked(c: &Course, n: usize) -> (Vec<i32>, Vec<(usize, u32, u32)>) {
    let mut o = Vec::new();
    let mut ranges: Vec<(usize, u32, u32)> = Vec::new();
    let verts = |o: &Vec<i32>| (o.len() / VERTEX_VALUES) as u32;
    // columns across the course, metres from the centreline: the floor,
    // then up the valley's walls
    let across: [i32; 15] = [
        -120, -95, -70, -45, -30, -20, -10, 0, 10, 20, 30, 45, 70, 95, 120,
    ];
    let point = |s: i32, k: usize| {
        let a = int(across[k]);
        let q = c.point(s).add(right_of(c.heading_at(s)).scale(a));
        v3(q.x, ground(q.x, q.z) + rise(s, a), q.z)
    };
    let every = int(6);
    // from behind the start tree to under the cliff past the finish
    let mut s = -int(60);
    let mut row = 0;
    while s < c.length + OUTRO + int(30) {
        let s1 = s + every;
        let chunk = chunk_along(c, s, n);
        match ranges.last() {
            Some((k, _, _)) if *k == chunk => {}
            _ => ranges.push((chunk, verts(&o), 0)),
        }
        for k in 0..across.len() - 1 {
            let a = point(s, k);
            let b = point(s, k + 1);
            let cc = point(s1, k + 1);
            let d = point(s1, k);
            let mid_a = int(across[k] + across[k + 1]) / 2;
            for (h, tri) in [[a, d, cc], [a, cc, b]].into_iter().enumerate() {
                let centre = tri[0].add(tri[1]).add(tri[2]).scale(ONE / 3);
                // each facet a touch lighter or darker, for the low-poly look
                let wobble = 95 + ((row * 7 + k * 13 + h * 5) % 9) as i32;
                let t = shade(ground_tint(centre, mid_a, s), wobble);
                push_ground(&mut o, tri, t);
            }
        }
        if let Some(last) = ranges.last_mut() {
            last.2 = verts(&o) - last.1;
        }
        s = s1;
        row += 1;
    }
    debug_assert_eq!(o.len() % (VERTEX_VALUES * 3), 0);
    (o, ranges)
}

/// One flat-shaded ground facet, turned to face up.
pub(crate) fn push_ground(o: &mut Vec<i32>, p: [V3; 3], t: Rgb) {
    let up = p[1].sub(p[0]).cross(p[2].sub(p[0]));
    let p = if up.y < 0 { [p[0], p[2], p[1]] } else { p };
    let n = p[1].sub(p[0]).cross(p[2].sub(p[0])).norm();
    for q in p {
        o.extend_from_slice(&[q.x, q.y, q.z, n.x, n.y, n.z, t[0], t[1], t[2]]);
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

/// The chunk a scenery shape at `p` goes in, of `n`.
pub(crate) fn chunk_of(c: &Course, p: V3, n: usize) -> usize {
    let last = c.path.len() - 1;
    let i = c.nearest_anywhere(p);
    if i == last && p.flat().sub(c.path[last]).dot(c.heading[last]) > 0 {
        n - 1
    } else {
        (i / CHUNK).min(n - 2)
    }
}

/// Builds the course's scenery and ground, chunked. The City has its own
/// (`cityscape.rs`).
pub fn scenery(c: &Course) -> Scenery {
    if let Some(city) = &c.city {
        return crate::cityscape::scenery(c, city);
    }
    let n = chunk_count(c);
    let mut batches: Vec<Batch> = (0..n).map(|_| Batch::new()).collect();
    let chunk_of = |p: V3| chunk_of(c, p, n);
    for p in &c.props {
        let at = match *p {
            Prop::Tree { at, .. } | Prop::Bush { at, .. } | Prop::Fern { at, .. } => at,
            Prop::Flower { at, .. } | Prop::Rock { at, .. } | Prop::Outcrop { at, .. } => at,
            Prop::Branch { to, .. } => to,
            Prop::Fallen { from, to, .. } | Prop::Water { from, to, .. } => from.lerp(to, ONE / 2),
        };
        prop(&mut batches[chunk_of(at)], p);
    }
    for m in &c.mushrooms {
        mushroom(&mut batches[chunk_of(m.top)], m);
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
    finish_arch(&mut batches[n - 2], c);
    outro(&mut batches[n - 1], c);
    let (terrain, ranges) = terrain_chunked(c, n);
    assemble(c, batches, terrain, ranges)
}

/// The scenery from its chunks' shapes and the ground with each chunk's
/// vertex ranges: the terrain's one instance first, each chunk's draws
/// naming its own ranges.
pub(crate) fn assemble(
    c: &Course,
    batches: Vec<Batch>,
    terrain: Vec<i32>,
    ranges: Vec<(usize, u32, u32)>,
) -> Scenery {
    let last = c.path.len() - 1;
    let n = batches.len();
    let mut values = Vec::new();
    let mut identity = Batch::new();
    identity.push(Mesh::Terrain, &Affine::IDENTITY, [ONE; 3], 0);
    identity.finish_into(crate::render::STATIC_BUFFER, &mut values);
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
                    crate::render::GROUND,
                ]);
            }
        }
        let centre = if k == n - 1 {
            c.point(c.length + OUTRO / 2)
        } else {
            c.path[(k * CHUNK + CHUNK / 2).min(last)]
        };
        chunks.push(Chunk { centre, draws });
    }
    Scenery {
        values,
        terrain,
        chunks,
    }
}

/// A flat slab from `a` to `b`, `width` across and `thick` through.
fn slab(a: V3, b: V3, width: i32, thick: i32) -> Affine {
    let d = b.sub(a);
    Affine::translate(a.add(d.scale(ONE / 2)))
        .then(&Affine::along(d))
        .then(&Affine::scale(v3(width, d.len(), thick)))
}

/// A point `r` out from `at` at angle `a` round it, `up` above it.
pub(crate) fn around(at: V3, a: i32, r: i32, up: i32) -> V3 {
    at.add(v3(mul(fx::sin(a), r), up, mul(fx::cos(a), r)))
}

fn prop(b: &mut Batch, p: &Prop) {
    match *p {
        Prop::Tree {
            at,
            radius,
            height,
            shade: s,
            kind,
        } => match kind {
            TreeKind::Broadleaf => tree(b, at, radius, height, s, crown_scale(height)),
            TreeKind::Emergent => emergent(b, at, radius, height, s),
            TreeKind::Palm => palm(b, at, radius, height, s),
            TreeKind::Fig => fig(b, at, radius, height, s),
            TreeKind::Bamboo => bamboo(b, at, height, s),
            TreeKind::Hill => hill_tree(b, at, radius, height, s),
        },
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
                let tip = around(at, a, size, size / 2);
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
        Prop::Fallen { from, to, radius } => {
            let lift = v3(0, radius * 7 / 10, 0);
            b.push(
                Mesh::Cyl,
                &rod(from.add(lift), to.add(lift), radius * 2),
                BARK[2],
                0,
            );
            // moss along its top, and a broken branch sticking up
            let mid = from.lerp(to, ONE * 2 / 5).add(lift).add(v3(0, radius, 0));
            b.push(
                Mesh::Blob,
                &placed(mid, radius, v3(radius * 5, radius, radius * 3)),
                MOSS,
                0,
            );
            let knot = from.lerp(to, ONE * 3 / 4).add(lift);
            b.push(
                Mesh::Cone,
                &rod(knot, knot.add(v3(radius / 2, radius * 3, 0)), radius),
                BARK[3],
                0,
            );
        }
        Prop::Water { from, to, width } => {
            b.push(Mesh::Cube, &slab(from, to, width, cm(16)), WATER, ONE / 5);
        }
        Prop::Outcrop { at, size, yaw } => {
            // a few tilted faceted blocks, with green on top
            let tilt = |a: i32| Affine::rot_z(a).then(&Affine::rot_x(a / 2));
            let blocks = [
                (V3::ZERO, v3(size, size * 3 / 2, size * 3 / 4), deg(9)),
                (
                    v3(size * 2 / 3, -size / 4, size / 3),
                    v3(size * 3 / 4, size, size * 3 / 5),
                    -deg(14),
                ),
                (
                    v3(-size / 2, -size / 3, -size / 4),
                    v3(size * 3 / 5, size * 4 / 5, size / 2),
                    deg(20),
                ),
            ];
            for (k, (off, dims, a)) in blocks.into_iter().enumerate() {
                let p = at
                    .add(Affine::rot_y(yaw).apply_dir(off))
                    .add(v3(0, dims.y / 3, 0));
                b.push(
                    Mesh::Cube,
                    &Affine::translate(p)
                        .then(&Affine::rot_y(yaw + k as i32 * 9000))
                        .then(&tilt(a))
                        .then(&Affine::scale(dims)),
                    shade(ROCK, 88 + k as i32 * 8),
                    0,
                );
            }
            b.push(
                Mesh::Blob,
                &placed(
                    at.add(v3(0, size * 13 / 12, 0)),
                    yaw,
                    v3(size, size / 3, size * 4 / 5),
                ),
                SLOPE,
                0,
            );
        }
    }
}

/// A broadleaf: a buttressed trunk, its crown a cluster of leaf blobs,
/// `k` times full size.
fn tree(b: &mut Batch, at: V3, radius: i32, height: i32, s: u8, k: i32) {
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
    for j in 0..3 {
        let a = j * deg(120) + radius;
        let foot = around(at, a, radius * 3, 0);
        b.push(
            Mesh::Cone,
            &rod(foot, at.add(v3(0, radius * 3, 0)), radius),
            shade(bark, 90),
            0,
        );
    }
    // the crown: a cluster of leaf blobs
    let top = at.add(v3(0, height, 0));
    let leaf = LEAF[(s as usize + 1) % 4];
    b.push(
        Mesh::Blob,
        &placed(
            top,
            radius * 7,
            v3(mul(int(14), k), mul(int(7), k), mul(int(14), k)),
        ),
        leaf,
        0,
    );
    for j in 0..3 {
        let a = j * deg(120) + radius * 3;
        let p = around(top, a, mul(int(6), k), -mul(int(2), k));
        b.push(
            Mesh::Blob,
            &placed(p, a, v3(mul(int(9), k), mul(int(5), k), mul(int(9), k))),
            shade(leaf, 85 + j * 8),
            0,
        );
    }
}

/// A giant over the canopy: wide buttresses, a long bare trunk, and a flat
/// umbrella crown on spreading limbs.
fn emergent(b: &mut Batch, at: V3, radius: i32, height: i32, s: u8) {
    let bark = shade(BARK[3], 100 + s as i32 * 4);
    let r = radius * 13 / 10;
    b.push(
        Mesh::Cyl,
        &placed(at.add(v3(0, height / 2, 0)), 0, v3(r * 2, height, r * 2)),
        bark,
        0,
    );
    for j in 0..4 {
        let a = j * deg(90) + radius * 2;
        b.push(
            Mesh::Cone,
            &rod(around(at, a, r * 4, 0), at.add(v3(0, r * 6, 0)), r * 3 / 2),
            shade(bark, 88),
            0,
        );
    }
    let top = at.add(v3(0, height, 0));
    let g = |m: i32| mul(int(m), GROW);
    b.push(
        Mesh::Blob,
        &placed(top, radius * 5, v3(g(24), g(6), g(24))),
        EMERGENT,
        0,
    );
    for j in 0..4 {
        let a = j * deg(90) + deg(45) + radius;
        let p = around(top, a, g(10), -GROW);
        b.push(
            Mesh::Blob,
            &placed(p, a, v3(g(12), g(5), g(12))),
            shade(EMERGENT, 82 + j * 7),
            0,
        );
        if j % 2 == 0 {
            b.push(
                Mesh::Cyl,
                &rod(top.sub(v3(0, g(8), 0)), around(top, a, g(8), -g(2)), r),
                bark,
                0,
            );
        }
    }
}

/// A palm: a thin trunk curving up, a star of drooping fronds.
fn palm(b: &mut Batch, at: V3, radius: i32, height: i32, s: u8) {
    let lean = crate::geom::facing(radius * 7, 0).scale((height / 8).min(int(2)));
    let w = radius * 4 / 5;
    let mut prev = at;
    for j in 1..=3 {
        let t = ONE * j / 3;
        let p = at.add(v3(0, mul(height, t), 0)).add(lean.scale(mul(t, t)));
        b.push(Mesh::Cyl, &rod(prev, p, w - j * w / 8), PALM_BARK, 0);
        prev = p;
    }
    let top = prev;
    for j in 0..7 {
        let a = j * deg(51) + radius * 3;
        let tip = around(top, a, int(5), -int(2));
        b.push(
            Mesh::Cube,
            &slab(top, tip, cm(150), cm(14)),
            shade(PALM, 86 + (j + s as i32) % 3 * 9),
            0,
        );
    }
    b.push(
        Mesh::Blob,
        &placed(top.sub(v3(0, cm(50), 0)), 0, v3(cm(140), cm(110), cm(140))),
        rgb(120, 90, 40),
        0,
    );
}

/// A strangler fig: pale roots twisting round a dark core, a dense crown.
fn fig(b: &mut Batch, at: V3, radius: i32, height: i32, s: u8) {
    b.push(
        Mesh::Cyl,
        &placed(
            at.add(v3(0, height * 2 / 5, 0)),
            0,
            v3(radius * 6 / 5, height * 4 / 5, radius * 6 / 5),
        ),
        BARK[0],
        0,
    );
    for j in 0..4 {
        let a = j * deg(90) + radius;
        let p0 = around(at, a, radius * 5 / 2, 0);
        let p1 = around(at, a + deg(70), radius * 6 / 5, height * 2 / 5);
        let p2 = around(at, a + deg(140), radius * 3 / 4, height * 17 / 20);
        b.push(Mesh::Cyl, &rod(p0, p1, radius * 3 / 5), FIG_BARK, 0);
        b.push(Mesh::Cyl, &rod(p1, p2, radius / 2), FIG_BARK, 0);
    }
    let top = at.add(v3(0, height, 0));
    let leaf = shade(FIG_LEAF, 92 + s as i32 * 5);
    let g = |m: i32| mul(int(m), GROW);
    b.push(
        Mesh::Blob,
        &placed(top, radius * 3, v3(g(15), g(8), g(15))),
        leaf,
        0,
    );
    for j in 0..3 {
        let a = j * deg(120) + radius * 2;
        b.push(
            Mesh::Blob,
            &placed(around(top, a, g(6), -g(2)), a, v3(g(10), g(5), g(10))),
            shade(leaf, 86 + j * 6),
            0,
        );
    }
}

/// A stand of bamboo: canes fanning out, leafy at the top.
fn bamboo(b: &mut Batch, at: V3, height: i32, s: u8) {
    for j in 0..5 {
        let a = j * deg(72) + s as i32 * 5000;
        let foot = around(at, a, cm(110), 0);
        let h = height - int((j * 3 + s as i32) % 4);
        let top = around(at, a, cm(220), h);
        let cane = if j % 2 == 0 {
            BAMBOO
        } else {
            shade(BAMBOO, 84)
        };
        b.push(Mesh::Cyl, &rod(foot, top, cm(28)), cane, 0);
        // two sprays of narrow leaves, drooping out from the top
        for (k, turn) in [-deg(35), deg(35)].into_iter().enumerate() {
            let from = top.sub(v3(0, cm(60) + k as i32 * cm(140), 0));
            let tip = around(from, a + turn, cm(260), -cm(90));
            b.push(
                Mesh::Cube,
                &slab(from, tip, cm(70), cm(10)),
                LEAF[(j as usize + k + s as usize) % 4],
                0,
            );
        }
    }
}

/// A far tree on the valley's walls: a faceted cone, or a round blob.
fn hill_tree(b: &mut Batch, at: V3, radius: i32, height: i32, s: u8) {
    let leaf = HILL[s as usize % 4];
    b.push(
        Mesh::Cyl,
        &placed(
            at.add(v3(0, height / 5, 0)),
            0,
            v3(radius, height * 2 / 5, radius),
        ),
        BARK[2],
        0,
    );
    let cr = (height * 7 / 20).min(int(8));
    if s.is_multiple_of(2) {
        b.push(
            Mesh::Cone,
            &placed(
                at.add(v3(0, height * 11 / 20, 0)),
                radius,
                v3(cr * 2, height * 3 / 5, cr * 2),
            ),
            leaf,
            0,
        );
        b.push(
            Mesh::Cone,
            &placed(
                at.add(v3(0, height * 4 / 5, 0)),
                radius * 3,
                v3(cr * 7 / 5, height * 2 / 5, cr * 7 / 5),
            ),
            shade(leaf, 110),
            0,
        );
    } else {
        b.push(
            Mesh::Blob,
            &placed(
                at.add(v3(0, height * 7 / 10, 0)),
                radius * 5,
                v3(cr * 2, height / 2, cr * 2),
            ),
            leaf,
            0,
        );
    }
}

/// A bounce mushroom (TRIAGE T5): a thick pale stalk under a bright domed
/// cap with white spots, lit a little so it reads from the vines.
fn mushroom(b: &mut Batch, m: &Mushroom) {
    let top = m.top;
    let gy = ground(top.x, top.z);
    let r = m.radius;
    // the cap's height grows with its radius (TRIAGE T48)
    let cap_h = mul(r, ONE * 74 / 100);
    let under = top.y - cap_h * 3 / 4;
    b.push(
        Mesh::Cyl,
        &placed(
            v3(top.x, (gy + under) / 2, top.z),
            0,
            v3(r * 3 / 4, under - gy + cm(10), r * 3 / 4),
        ),
        STALK,
        0,
    );
    // the gills under the cap, then the cap
    b.push(
        Mesh::Cyl,
        &placed(v3(top.x, under, top.z), 0, v3(r * 9 / 5, cm(20), r * 9 / 5)),
        shade(STALK, 85),
        0,
    );
    b.push(
        Mesh::Blob,
        &placed(
            top.sub(v3(0, cap_h / 2, 0)),
            deg(15),
            v3(r * 2, cap_h, r * 2),
        ),
        CAP,
        ONE / 4,
    );
    for j in 0..6 {
        let (a, out, down) = if j == 5 {
            (0, 0, -cm(4))
        } else {
            (j * deg(72) + deg(20), r * 3 / 5, cm(22))
        };
        b.push(
            Mesh::Blob,
            &placed(around(top, a, out, -down), a, v3(cm(80), cm(28), cm(80))),
            SPOT,
            ONE / 4,
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
        // its top stops under the deck, so the two never fight for a pixel
        &placed(
            v3(centre.x, (gy + centre.y - int(1)) / 2, centre.z),
            0,
            v3(int(4), centre.y - int(1) - gy, int(4)),
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

/// Past the finish (TRIAGE T6): bunting over the run-out, a podium and a
/// pile of bananas in a clearing, and the valley closed by a cliff with a
/// waterfall into a pool, where a finished ape coasts to a stop.
fn outro(b: &mut Batch, c: &Course) {
    let last = c.path.len() - 1;
    let fwd = c.heading[last];
    let right = right_of(fwd);
    let yaw = crate::geom::yaw_of(fwd);
    let up = |h: i32| v3(0, h, 0);
    let at = |s: i32, a: i32| {
        let q = c.point(c.length + s).add(right.scale(a));
        v3(q.x, ground(q.x, q.z), q.z)
    };
    // two lines of bunting across the run-out
    for s in [int(26), int(66)] {
        let ends = [at(s, -c.half_width), at(s, c.half_width)];
        for e in ends {
            b.push(Mesh::Cyl, &rod(e, e.add(up(int(10))), cm(40)), BARK[3], 0);
        }
        let (l, r) = (ends[0].add(up(int(10))), ends[1].add(up(int(10))));
        let flags = 16;
        let sag = |t: i32| up(-mul(mul(4 * t, ONE - t), cm(180)));
        let mut prev = l;
        for k in 0..flags {
            let t = ONE * (2 * k + 1) / (2 * flags);
            let p = l.lerp(r, t).add(sag(t));
            b.push(Mesh::Cyl, &rod(prev, p, cm(8)), BANNER_DARK, 0);
            prev = p;
            b.push(
                Mesh::Cone,
                &Affine::translate(p.sub(v3(0, cm(70), 0)))
                    .then(&Affine::rot_y(yaw))
                    .then(&Affine::rot_x(deg(180)))
                    .then(&Affine::scale(v3(cm(110), cm(130), cm(14)))),
                if k % 5 == 4 {
                    BANNER
                } else {
                    FLOWERS[k as usize % 4]
                },
                ONE / 4,
            );
        }
        b.push(Mesh::Cyl, &rod(prev, r, cm(8)), BANNER_DARK, 0);
    }
    // the podium: first in the middle, second and third either side
    let p = at(int(48), 0);
    for (x, h, colour) in [
        (0, int(3), GOLD),
        (-cm(420), int(2), SILVER),
        (cm(420), cm(130), BRONZE),
    ] {
        let q = p.add(right.scale(x));
        b.push(
            Mesh::Cube,
            &placed(q.add(up(h / 2)), yaw, v3(int(4), h, int(4))),
            colour,
            ONE / 6,
        );
        b.push(
            Mesh::Cube,
            &placed(q.add(up(h + cm(5))), yaw, v3(cm(420), cm(10), cm(420))),
            BANNER,
            ONE / 6,
        );
    }
    // a pile of bananas beside it: rings of bent bananas, smaller going up
    let q = at(int(46), int(12));
    b.push(
        Mesh::Cone,
        &placed(q.add(up(cm(60))), 0, v3(int(5), cm(120), int(5))),
        shade(BANANA, 70),
        0,
    );
    for (ring, count, r, h) in [
        (0, 9, cm(230), cm(30)),
        (1, 6, cm(150), cm(80)),
        (2, 3, cm(70), cm(130)),
    ] {
        for k in 0..count {
            let a = k * fx::TURN / count + ring * 4000;
            let mid = around(q, a, r, h);
            let along = crate::geom::facing(a + deg(90), 0).scale(cm(60));
            let ends = [
                mid.sub(along).add(up(cm(18))),
                mid.add(along).add(up(cm(18))),
            ];
            b.push(Mesh::Cube, &rod(ends[0], mid, cm(24)), BANANA, ONE / 6);
            b.push(Mesh::Cube, &rod(mid, ends[1], cm(24)), BANANA, ONE / 6);
        }
    }
    // the cliff closing the valley, and its waterfall into a pool
    let cliff = OUTRO + int(10);
    let mut a = -int(126);
    let mut k = 0;
    while a <= int(126) {
        let base = at(cliff + int(k % 3), a);
        let h = int(30) + int((k * 7) % 13) + rise(c.length + cliff, a);
        let w = int(14 + (k * 5) % 7);
        // a broad base, and a narrower, set back, leaning block on it
        b.push(
            Mesh::Cube,
            &Affine::translate(base.add(up(h * 3 / 10 - int(2))))
                .then(&Affine::rot_y(yaw + (k % 5 - 2) * deg(9)))
                .then(&Affine::rot_z((k % 3 - 1) * deg(4)))
                .then(&Affine::scale(v3(w, h * 3 / 5, int(14)))),
            shade(ROCK, 80 + (k * 5) % 18),
            0,
        );
        let upper = base.add(fwd.scale(int(4))).add(up(h * 7 / 10 - int(3)));
        b.push(
            Mesh::Cube,
            &Affine::translate(upper)
                .then(&Affine::rot_y(yaw - (k % 4 - 1) * deg(11)))
                .then(&Affine::rot_x(-deg(6)))
                .then(&Affine::rot_z((1 - k % 3) * deg(7)))
                .then(&Affine::scale(v3(w * 3 / 4, h * 3 / 5, int(10)))),
            shade(ROCK, 92 + (k * 7) % 16),
            0,
        );
        b.push(
            Mesh::Blob,
            &placed(upper.add(up(h * 3 / 10)), k * 3000, v3(w, int(4), int(10))),
            HILL[k as usize % 4],
            0,
        );
        a += int(12);
        k += 1;
    }
    let top = at(cliff - int(5), 0).add(up(int(36)));
    let foot = at(OUTRO - int(4), 0).add(up(cm(30)));
    b.push(
        Mesh::Cube,
        &slab(top, foot, int(8), cm(40)),
        WATERFALL,
        ONE / 3,
    );
    for x in [-int(3), int(3)] {
        let side = right.scale(x);
        b.push(
            Mesh::Cube,
            &slab(
                top.add(side).add(up(-int(4))),
                foot.add(side),
                int(2),
                cm(60),
            ),
            BANNER,
            ONE / 3,
        );
    }
    let pool = at(OUTRO - int(5), 0);
    b.push(
        Mesh::Cyl,
        &placed(pool.add(up(cm(10))), yaw, v3(int(24), cm(20), int(14))),
        WATER,
        ONE / 5,
    );
    for j in 0..5 {
        let p = foot.add(right.scale(int(j * 2 - 4)));
        b.push(
            Mesh::Blob,
            &placed(p, j * 7000, v3(int(3), cm(120), int(2))),
            BANNER,
            ONE / 3,
        );
    }
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
    /// the vines' bending chains, one per vine (empty: draw them straight)
    pub chains: &'a [crate::vinesim::Chain],
    /// the colourblind mode (TRIAGE T33): vivid vines, in reach near white
    pub vivid: bool,
}

/// How a vine is drawn: its colour, glow and width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VineLook {
    pub colour: Rgb,
    pub glow: i32,
    pub width: i32,
}

/// The look of a vine, in reach of the local ape or not, in the ordinary
/// or the colourblind (`vivid`) colours. In reach, a vivid vine is near
/// white, thicker, and its width pulses.
pub fn vine_look(in_reach: bool, vivid: bool, tick: u32) -> VineLook {
    match (vivid, in_reach) {
        (false, false) => VineLook {
            colour: VINE,
            glow: ONE / 5,
            width: VINE_WIDTH,
        },
        (false, true) => VineLook {
            colour: VINE_REACH,
            glow: ONE * 7 / 10,
            width: VINE_WIDTH,
        },
        (true, false) => VineLook {
            colour: VINE_VIVID,
            glow: ONE / 2,
            width: VINE_WIDTH_VIVID,
        },
        (true, true) => VineLook {
            colour: VINE_REACH_VIVID,
            glow: ONE,
            width: VINE_WIDTH_VIVID * 3 / 2 + mul(sin(tick as i32 * 600).abs(), cm(14)),
        },
    }
}

/// Everything that moves: the vines and their flowers (drawn as
/// `render::VINES`), and the rest.
pub fn dynamic(f: &Frame) -> (Batch, Batch) {
    let mut vines = Batch::new();
    let mut b = Batch::new();
    let race = f.race;
    let c = &race.course;
    let tick = race.tick;
    let eye = f.cam.eye;
    let reach = f.local.and_then(|i| {
        let a = &race.racers[i].ape;
        if a.swinging().is_none() && a.stun == 0 {
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
        let look = vine_look(Some(i) == reach, f.vivid, f.tick);
        let look = catch_up_look(race, f.local, i, Some(i) == reach, look);
        match f.chains.get(i) {
            Some(chain) => chain_vine(&mut vines, chain, held[i], look, f.tick + i as u32 * 37),
            None => vine(
                &mut vines,
                a,
                c.vines[i].length,
                held[i],
                look,
                f.tick + i as u32 * 37,
            ),
        }
    }
    item_flowers(&mut vines, f, &held);
    for (k, r) in race.racers.iter().enumerate() {
        let at = f.apes[k];
        let pose = racer_pose(race, k, at, f.tick + k as u32 * 53);
        models::draw(&mut b, r.ape.kind, &pose);
        // a soft shadow on the ground, so height reads (as big as the
        // drawn ape)
        let support = c.support(at);
        let feet = at.y - BODY_DROP;
        let high = (feet - support).clamp(0, int(30));
        let size = (cm(110) - high * 2 / 60).max(cm(60)) * RACE_SCALE / 100;
        b.push(
            Mesh::Cyl,
            &placed(v3(at.x, support + cm(4), at.z), 0, v3(size, cm(2), size)),
            SHADOW,
            0,
        );
    }
    course_bananas(&mut b, race, f.local, eye, f.tick);
    crate::items::draw(&mut b, race);
    speed_lines(&mut b, f);
    (vines, b)
}

/// Where along an item vine its flowers sit, from the anchor (TRIAGE T46,
/// T28): on its upper and middle part, where a swinging ape looks, not at
/// its end.
const FLOWER_AT: [i32; 3] = [ONE * 3 / 10, ONE / 2, ONE * 7 / 10];

/// The points of vine `i` as drawn: its chain, or a straight line down.
fn vine_points(f: &Frame, i: usize, hands: Option<V3>) -> Vec<V3> {
    let c = &f.race.course;
    let a = c.anchor(i, f.race.tick);
    match (f.chains.get(i), hands) {
        (Some(chain), _) => chain.pts.to_vec(),
        (None, Some(h)) => vec![a, h],
        (None, None) => vec![a, a.sub(v3(0, c.vines[i].length, 0))],
    }
}

/// The point `t` (0..=ONE) of the way along a polyline, by length.
fn along(pts: &[V3], t: i32) -> V3 {
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

/// The flowers on each item vine (TRIAGE T46: catching it gives the item,
/// so the flowers mark it). While its item is taken, closed buds.
fn item_flowers(b: &mut Batch, f: &Frame, held: &[Option<V3>]) {
    let race = f.race;
    let right = right_of(f.cam.fwd);
    let up = right.cross(f.cam.fwd).norm();
    // no item allowed (TRIAGE T42): no flowers
    if race.items.is_empty() {
        return;
    }
    for (k, &i) in race.crate_vines.iter().enumerate() {
        let a = race.course.anchor(i, race.tick);
        if a.flat().sub(f.cam.eye.flat()).len() > int(230) {
            continue;
        }
        let open = race.crates.get(k).copied().unwrap_or(0) == 0;
        let pts = vine_points(f, i, held.get(i).copied().flatten());
        for (j, t) in FLOWER_AT.into_iter().enumerate() {
            // beside the vine, on alternate sides as the camera sees it
            let side = if j % 2 == 0 { cm(40) } else { -cm(40) };
            let at = along(&pts, t).add(right.scale(side));
            let spin = f.tick as i32 * 30 + (k * 3 + j) as i32 * 4000;
            if open {
                flower(b, at, right, up, spin);
            } else {
                b.push(
                    Mesh::Sphere,
                    &placed(at, 0, v3(cm(30), cm(40), cm(30))),
                    BUD,
                    0,
                );
            }
        }
    }
}

/// An open flower facing the camera: five petals round a heart.
pub fn flower(b: &mut Batch, at: V3, right: V3, up: V3, spin: i32) {
    let face = up.cross(right).norm();
    for p in 0..5 {
        let a = spin + p * fx::TURN / 5;
        let out = right.scale(fx::cos(a)).add(up.scale(fx::sin(a)));
        let across = face.cross(out).norm();
        let petal = Affine {
            c0: across.scale(cm(42)),
            c1: out.scale(cm(70)),
            c2: face.scale(cm(12)),
            t: at.add(out.scale(cm(42))),
        };
        b.push(Mesh::Blob, &petal, PETAL, ONE / 2);
    }
    b.push(
        Mesh::Sphere,
        &placed(at.add(face.scale(cm(6))), 0, v3(cm(36), cm(36), cm(36))),
        HEART,
        ONE * 3 / 4,
    );
}

/// How big the racers are drawn, percent of life size (TRIAGE T40: twice,
/// so opponents are easier to see). Only the drawing grows: the physics
/// (reach, hit ranges, `BODY_DROP`) keeps its sizes.
pub const RACE_SCALE: i32 = 200;

/// Racer `k`'s pose this frame, drawn at `at` (its hands) at `RACE_SCALE`.
/// A grounded ape is drawn about its feet, so it still stands on the floor.
pub fn racer_pose(race: &Race, k: usize, at: V3, tick: u32) -> Pose {
    let a = &race.racers[k].ape;
    Pose {
        hands: at,
        yaw: a.yaw,
        rope: a
            .swinging()
            .map(|v| race.course.anchor(v, race.tick).sub(at).norm()),
        vel: a.vel,
        grounded: a.mode == crate::ape::Mode::Grounded,
        stunned: a.stun > 0,
        tick,
        scale: RACE_SCALE,
    }
}

/// The course's bananas (TRIAGE T43) the viewer has not taken, near the
/// eye: each turning slowly and bobbing, lit so an arc reads from afar.
fn course_bananas(b: &mut Batch, race: &Race, viewer: Option<usize>, eye: V3, t: u32) {
    let took = viewer.and_then(|v| race.racers.get(v)).map(|r| &r.took);
    for (k, at) in race.course.bananas.iter().enumerate() {
        if took.is_some_and(|got| got.get(k).copied().unwrap_or(false)) {
            continue;
        }
        if (at.x - eye.x).abs() > int(150) || (at.z - eye.z).abs() > int(150) {
            continue;
        }
        let spin = (t as i32) * 220 + k as i32 * 5_000;
        let bob = mul(sin((t as i32) * 300 + k as i32 * 7_000), cm(12));
        pickup_banana(b, at.add(v3(0, bob, 0)), spin);
    }
}

/// One course banana: a curved fruit of rods round a vertical axis, stalk
/// up.
fn pickup_banana(b: &mut Batch, at: V3, spin: i32) {
    let frame = Affine::translate(at).then(&Affine::rot_y(spin));
    let r = cm(45);
    let arc = |d: i32| {
        let a = deg(d);
        frame.apply(v3(mul(fx::sin(a), r), r / 2 - mul(fx::cos(a), r), 0))
    };
    let points = [-70, -35, 0, 35, 70].map(arc);
    for w in points.windows(2) {
        b.push(
            Mesh::Cyl,
            &rod(w[0], w[1], cm(17)),
            BANANA_YELLOW,
            ONE * 2 / 5,
        );
    }
    for p in &points[1..4] {
        b.push(
            Mesh::Sphere,
            &placed(*p, 0, v3(cm(17), cm(17), cm(17))),
            BANANA_YELLOW,
            ONE * 2 / 5,
        );
    }
    let tip = points[4].add(points[4].sub(points[3]).with_len(cm(9)));
    b.push(Mesh::Cyl, &rod(points[4], tip, cm(8)), BANANA_STALK, 0);
}

/// A catch-up vine (TRIAGE T45: an opponent swung on it, the viewer has
/// not) glows coral red, apart from the in-reach yellow (which wins) and
/// the item vines' flowers; any other vine keeps its look.
fn catch_up_look(
    race: &Race,
    viewer: Option<usize>,
    i: usize,
    in_reach: bool,
    look: VineLook,
) -> VineLook {
    if !in_reach && viewer.is_some_and(|v| race.catch_up_vine(v, i)) {
        VineLook {
            colour: VINE_USED,
            glow: look.glow.max(ONE * 9 / 20),
            ..look
        }
    } else {
        look
    }
}

/// A vine from its anchor: straight down and swaying a little; or, when
/// held, taut from the anchor to the hands with the rest dangling below.
fn vine(b: &mut Batch, a: V3, length: i32, hands: Option<V3>, look: VineLook, t: u32) {
    let VineLook {
        colour,
        glow,
        width,
    } = look;
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
            let knot = width + cm(14);
            b.push(
                Mesh::Sphere,
                &placed(bottom, 0, v3(knot, knot, knot)),
                colour,
                glow,
            );
            leaves(b, a, bottom, t);
        }
    }
}

/// A vine drawn from its bending chain. A held one's grip point is moved to
/// where the hands are drawn this frame (between ticks).
fn chain_vine(
    b: &mut Batch,
    chain: &crate::vinesim::Chain,
    hands: Option<V3>,
    look: VineLook,
    t: u32,
) {
    let VineLook {
        colour,
        glow,
        width,
    } = look;
    let mut pts = chain.pts;
    if let Some(h) = hands {
        let k = (1..pts.len())
            .min_by_key(|k| pts[*k].sub(h).dot64(pts[*k].sub(h)))
            .unwrap_or(pts.len() - 1);
        pts[k] = h;
    }
    for w in pts.windows(2) {
        b.push(Mesh::Cyl, &rod(w[0], w[1], width), colour, glow);
        // a joint, so the bends read smooth
        b.push(
            Mesh::Sphere,
            &placed(w[1], 0, v3(width, width, width)),
            colour,
            glow,
        );
    }
    let bottom = pts[pts.len() - 1];
    if hands.is_none() {
        b.push(
            Mesh::Sphere,
            &placed(
                bottom,
                0,
                v3(width + cm(14), width + cm(14), width + cm(14)),
            ),
            colour,
            glow,
        );
    }
    for k in (1..pts.len() - 1).step_by(2) {
        let side = if k % 4 == 1 { cm(18) } else { -cm(18) };
        b.push(
            Mesh::Blob,
            &placed(
                pts[k].add(v3(side, 0, 0)),
                (t as i32) * 40 + k as i32 * 7000,
                v3(cm(40), cm(14), cm(26)),
            ),
            VINE_LEAF,
            0,
        );
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
        tree(
            &mut b,
            v3(x, -cm(10), z),
            cm(90),
            int(16),
            (k % 4) as u8,
            ONE,
        );
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

/// What moves behind the title: the vines, swaying (drawn as
/// `render::VINES`).
pub fn flyover_dynamic(c: &Course, tick: u32, vivid: bool) -> Batch {
    let mut b = Batch::new();
    let look = vine_look(false, vivid, tick);
    for i in 0..c.vines.len() {
        let a = c.anchor(i, tick);
        vine(
            &mut b,
            a,
            c.vines[i].length,
            None,
            look,
            tick + i as u32 * 37,
        );
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_racer(kind: crate::rules::Kind) -> Race {
        let e = crate::race::Entry {
            name: "P".into(),
            kind,
            driver: crate::race::Driver::Local,
        };
        Race::new(Course::jungle(), &[e], false, 1)
    }

    /// The lowest and highest instance centres in a batch (y).
    fn heights(b: Batch) -> (i32, i32) {
        let (v, _) = b.finish(crate::render::DYNAMIC_BUFFER);
        let ys = v.chunks(crate::render::INSTANCE_VALUES).map(|i| i[7]);
        (ys.clone().min().unwrap(), ys.max().unwrap())
    }

    #[test]
    fn racers_are_drawn_twice_life_size_and_stand_on_the_ground() {
        // TRIAGE T40
        for kind in crate::rules::Kind::ALL {
            let mut race = one_racer(kind);
            let a = &mut race.racers[0].ape;
            a.mode = crate::ape::Mode::Grounded;
            let floor = race.course.support(a.pos);
            a.pos.y = floor + BODY_DROP;
            let at = race.racers[0].ape.pos;
            let pose = racer_pose(&race, 0, at, 40);
            let mut big = Batch::new();
            models::draw(&mut big, kind, &pose);
            let mut life = Batch::new();
            models::draw(&mut life, kind, &Pose { scale: 100, ..pose });
            let (lo, hi) = heights(big);
            let (lo1, hi1) = heights(life);
            assert!(
                (hi - lo) * 10 >= (hi1 - lo1) * 18,
                "{kind:?}: {} vs {} cm tall",
                (hi - lo) * 100 / ONE,
                (hi1 - lo1) * 100 / ONE
            );
            // the feet stay on the floor, not sunk into it
            assert!(
                lo >= floor - cm(2),
                "{kind:?}: sunk {} cm",
                (floor - lo) * 100 / ONE
            );
            assert!(lo <= floor + cm(30), "{kind:?}: floating");
        }
    }

    /// The vines batch for a frame looking at `at` from 20 m back.
    fn vines_seen(race: &Race, at: V3, vivid: bool) -> Batch {
        let eye = at.add(v3(0, 0, -int(20)));
        let cam = Camera {
            eye,
            fwd: at.sub(eye).norm(),
            fov: deg(70),
        };
        let apes: Vec<V3> = race.racers.iter().map(|r| r.ape.pos).collect();
        dynamic(&Frame {
            race,
            cam: &cam,
            apes: &apes,
            local: Some(0),
            tick: 10,
            chains: &[],
            vivid,
        })
        .0
    }

    /// The centres of the instances of `mesh` coloured `colour`.
    fn coloured(b: Batch, mesh: Mesh, colour: Rgb) -> Vec<V3> {
        let mut out = Vec::new();
        let (v, draws) = b.finish(crate::render::DYNAMIC_BUFFER);
        for d in draws.chunks(crate::render::DRAW_VALUES) {
            if d[1] != mesh as u32 {
                continue;
            }
            let first = d[2] as usize / crate::render::BYTES_PER_VALUE;
            for i in v[first..first + d[3] as usize * crate::render::INSTANCE_VALUES]
                .chunks(crate::render::INSTANCE_VALUES)
            {
                if [i[12], i[13], i[14]] == colour {
                    out.push(v3(i[3], i[7], i[11]));
                }
            }
        }
        out
    }

    #[test]
    fn item_vines_carry_flowers_up_their_middle() {
        // TRIAGE T46 (and T28): no crate at the vine's end; three flowers
        // of five petals on its upper and middle part, buds while taken
        let mut race = one_racer(crate::rules::Kind::Chimpanzee);
        let k = 0;
        let v = race.crate_vines[k];
        let a = race.course.anchor(v, race.tick);
        let len = race.course.vines[v].length;
        let mid = a.sub(v3(0, len / 2, 0));
        let petals = coloured(vines_seen(&race, mid, false), Mesh::Blob, PETAL);
        let near: Vec<&V3> = petals
            .iter()
            .filter(|p| p.flat().sub(a.flat()).len() < int(3))
            .collect();
        assert_eq!(near.len(), 15, "five petals on each of three flowers");
        for p in near {
            let down = a.y - p.y;
            assert!(
                down > len / 5 && down < len * 4 / 5,
                "a petal {} m down a {} m vine",
                down / ONE,
                len / ONE
            );
        }
        // taken: closed buds, no petals, until the item comes back
        race.crates[k] = 5;
        let b = vines_seen(&race, mid, false);
        let petals = coloured(b.clone(), Mesh::Blob, PETAL);
        assert!(petals.iter().all(|p| p.flat().sub(a.flat()).len() > int(3)));
        let buds = coloured(b, Mesh::Sphere, BUD);
        assert_eq!(
            buds.iter()
                .filter(|p| p.flat().sub(a.flat()).len() < int(3))
                .count(),
            3
        );
        // no item allowed in this race (TRIAGE T42): no flowers or buds
        race.items = crate::items::ItemSet::NONE;
        let b = vines_seen(&race, mid, false);
        let marks =
            coloured(b.clone(), Mesh::Blob, PETAL).len() + coloured(b, Mesh::Sphere, BUD).len();
        assert_eq!(marks, 0, "flowers drawn with no items allowed");
    }

    #[test]
    fn colourblind_vines_change_by_brightness_and_width_not_hue() {
        // TRIAGE T33
        let plain = vine_look(false, false, 0);
        let vivid = vine_look(false, true, 0);
        let reach = vine_look(true, true, 0);
        let luma = |c: Rgb| (c[0] * 30 + c[1] * 59 + c[2] * 11) / 100;
        assert!(vivid.width > plain.width && reach.width > vivid.width);
        assert!(luma(reach.colour) > luma(vivid.colour) + ONE / 5);
        // near white: no channel far from the others
        let spread = reach.colour.iter().max().unwrap() - reach.colour.iter().min().unwrap();
        assert!(spread < ONE / 10);
        // and it pulses
        assert_ne!(
            vine_look(true, true, 3).width,
            vine_look(true, true, 40).width
        );
        // the ordinary mode keeps its green and yellow
        assert_eq!(vine_look(true, false, 0).colour, VINE_REACH);
    }

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
        assert!(n > 1000 && n < 30_000, "{n}");
        // one chunk per 100 m, and the clearing past the finish
        assert_eq!(s.chunks.len(), c.path.len().div_ceil(CHUNK) + 1);
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
