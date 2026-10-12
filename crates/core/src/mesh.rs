//! The shapes everything is drawn from (PLAN P2). Each is a triangle list of
//! (position, normal, tint) in Q16, centred on the origin and one unit
//! across, so an instance's matrix carries its size. The tint is white on
//! every shape but the terrain, whose vertices carry the ground's colours.
//!
//! The faceted shapes after the terrain (TRIAGE T3, a low-poly look) are each
//! the convex hull of a few points, drawn flat: every face has one normal, so
//! each facet takes one shade and its edges stay crisp.

use crate::fx::{self, ONE};
use crate::geom::{v3, V3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mesh {
    /// a box from -0.5 to 0.5 on each axis
    Cube = 0,
    /// a smooth ball of diameter 1
    Sphere = 1,
    /// a coarse ball, for leaves, bushes and fur tufts
    Blob = 2,
    /// an upright cylinder, height 1 (y -0.5 to 0.5), diameter 1
    Cyl = 3,
    /// an upright cone, base at y -0.5 (diameter 1), tip at y 0.5
    Cone = 4,
    /// a flat square in the xz plane facing +y, 1 across
    Quad = 5,
    /// the course's ground, built per course and uploaded with the scene
    Terrain = 6,
    /// an icosahedron (20 triangles), diameter 1: a chunky faceted ball
    Ico = 7,
    /// a geodesic ball (80 triangles), diameter 1: a rounder faceted ball
    Geo = 8,
    /// an octahedron (8 triangles), corners on the axes at ±0.5: a diamond
    Octa = 9,
    /// an upright hexagonal prism, height 1 (y -0.5 to 0.5), 1 across the
    /// corners; as a rod it is a six-sided post, squashed it is a hex disc
    Hex = 10,
    /// a six-sided prism tapering from 1 across at y -0.5 to 0.6 at y 0.5;
    /// as a rod it is wide at the first end (limbs, branches, trunks)
    Taper = 11,
    /// a ramp: flat on the bottom (y -0.5) and the back (z -0.5), sloping
    /// from the back's top edge down to the bottom's front edge; x is across
    Wedge = 12,
    /// a square pyramid, base at y -0.5 (1 across), apex at y 0.5
    Pyramid = 13,
    /// a five-sided cone, base at y -0.5 (diameter 1), tip at y 0.5
    Spike = 14,
    /// an eight-sided barrel: narrow ends, wide middle, staggered triangular
    /// facets (torsos; stacked, a low-poly tree crown)
    Barrel = 15,
    /// an irregular boulder of about 70 facets inside the unit ball
    Rock = 16,
    /// a box with bevelled edges and corners (26 faces), 1 across
    Bevel = 17,
    /// a kite: points at y ±0.5 and x ±0.5 (a little above the middle), a
    /// ridge front and back at z ±0.5; squash z for a leaf, a fur tuft, an
    /// ear, a feather or a flame
    Shard = 18,
}

impl Mesh {
    pub const ALL: [Mesh; 19] = [
        Mesh::Cube,
        Mesh::Sphere,
        Mesh::Blob,
        Mesh::Cyl,
        Mesh::Cone,
        Mesh::Quad,
        Mesh::Terrain,
        Mesh::Ico,
        Mesh::Geo,
        Mesh::Octa,
        Mesh::Hex,
        Mesh::Taper,
        Mesh::Wedge,
        Mesh::Pyramid,
        Mesh::Spike,
        Mesh::Barrel,
        Mesh::Rock,
        Mesh::Bevel,
        Mesh::Shard,
    ];

    /// One of the faceted shapes: the hull of a few points, one normal per
    /// face, so it shades in crisp flat facets.
    pub fn faceted(self) -> bool {
        self as u32 > Mesh::Terrain as u32
    }
}

/// Values per vertex: position xyz, normal xyz, tint rgb.
pub const VERTEX_VALUES: usize = 9;

const H: i32 = ONE / 2;

pub fn push_tri(out: &mut Vec<i32>, a: V3, b: V3, c: V3) {
    let n = b.sub(a).cross(c.sub(a)).norm();
    for p in [a, b, c] {
        out.extend_from_slice(&[p.x, p.y, p.z, n.x, n.y, n.z, ONE, ONE, ONE]);
    }
}

/// A triangle with its own normals per corner (smooth shading) and a tint.
pub fn push_smooth(out: &mut Vec<i32>, p: [V3; 3], n: [V3; 3], tint: [i32; 3]) {
    for k in 0..3 {
        out.extend_from_slice(&[
            p[k].x, p[k].y, p[k].z, n[k].x, n[k].y, n[k].z, tint[0], tint[1], tint[2],
        ]);
    }
}

fn push_quad(out: &mut Vec<i32>, a: V3, b: V3, c: V3, d: V3) {
    push_tri(out, a, b, c);
    push_tri(out, a, c, d);
}

/// A ball of `rings` × `segments`, radius 0.5; smooth normals when `smooth`.
fn ball(out: &mut Vec<i32>, rings: i32, segments: i32, smooth: bool) {
    let point = |ring: i32, seg: i32| {
        let lat = fx::HALF * ring / rings - fx::QUARTER;
        let lon = fx::TURN * seg / segments;
        let c = fx::cos(lat);
        v3(
            fx::mul(fx::mul(c, fx::cos(lon)), H),
            fx::mul(fx::sin(lat), H),
            fx::mul(fx::mul(c, fx::sin(lon)), H),
        )
    };
    for ring in 0..rings {
        for seg in 0..segments {
            let a = point(ring, seg);
            let b = point(ring, seg + 1);
            let c = point(ring + 1, seg + 1);
            let d = point(ring + 1, seg);
            let tris = [[a, c, b], [a, d, c]];
            for t in tris {
                if t[0] == t[1] || t[1] == t[2] || t[0] == t[2] {
                    continue;
                }
                if smooth {
                    let n = t.map(|p| p.norm());
                    push_smooth(out, t, n, [ONE; 3]);
                } else {
                    push_tri(out, t[0], t[1], t[2]);
                }
            }
        }
    }
}

const SIDES: i32 = 12;

fn rim(k: i32, y: i32, r: i32) -> V3 {
    let a = fx::TURN * k / SIDES;
    v3(fx::mul(fx::cos(a), r), y, fx::mul(fx::sin(a), r))
}

/// One flat triangle with the normal `n` at every corner.
fn push_flat(out: &mut Vec<i32>, p: [V3; 3], n: V3) {
    push_smooth(out, p, [n; 3], [ONE; 3]);
}

/// A unit vector along a long (Q32) vector.
fn unit64(x: i64, y: i64, z: i64) -> V3 {
    let s = x as i128 * x as i128 + y as i128 * y as i128 + z as i128 * z as i128;
    if s == 0 {
        return V3::ZERO;
    }
    let l = fx::isqrt(s as u128) as i128;
    let c = |a: i64| ((a as i128 * ONE as i128) / l) as i32;
    v3(c(x), c(y), c(z))
}

/// The difference `b - a` in 64 bits.
fn d64(a: V3, b: V3) -> [i64; 3] {
    [
        b.x as i64 - a.x as i64,
        b.y as i64 - a.y as i64,
        b.z as i64 - a.z as i64,
    ]
}

/// The convex hull of `pts`, flat-shaded and wound outward. Every plane
/// through three of the points with no point beyond it is a face; the points
/// on it are put in order round their middle and fanned. Exact in integers,
/// so points meant to share a face (a square, a hexagon) make one face.
/// Quartic in the number of points: fine for the few dozen used here, built
/// once a load.
fn hull(out: &mut Vec<i32>, pts: &[V3]) {
    let n = pts.len().max(1) as i64;
    let sum = pts.iter().fold([0i64; 3], |a, p| {
        [a[0] + p.x as i64, a[1] + p.y as i64, a[2] + p.z as i64]
    });
    let inside = v3(
        (sum[0] / n) as i32,
        (sum[1] / n) as i32,
        (sum[2] / n) as i32,
    );
    let mut faces: Vec<Vec<usize>> = Vec::new();
    for i in 0..pts.len() {
        for j in i + 1..pts.len() {
            for k in j + 1..pts.len() {
                let u = d64(pts[i], pts[j]);
                let w = d64(pts[i], pts[k]);
                let c = [
                    u[1] * w[2] - u[2] * w[1],
                    u[2] * w[0] - u[0] * w[2],
                    u[0] * w[1] - u[1] * w[0],
                ];
                if c == [0; 3] {
                    continue;
                }
                let side = |p: V3| {
                    let d = d64(pts[i], p);
                    (0..3)
                        .map(|a| c[a] as i128 * d[a] as i128)
                        .sum::<i128>()
                        .signum()
                };
                let (mut above, mut below) = (false, false);
                let mut on = Vec::new();
                for (m, p) in pts.iter().enumerate() {
                    match side(*p) {
                        1 => above = true,
                        -1 => below = true,
                        _ => on.push(m),
                    }
                }
                if (above && below) || faces.contains(&on) {
                    continue;
                }
                // the normal points away from the inside
                let s = if side(inside) > 0 { -1 } else { 1 };
                let normal = unit64(s * c[0], s * c[1], s * c[2]);
                let corners: Vec<V3> = on.iter().map(|m| pts[*m]).collect();
                faces.push(on);
                let mid = corners.iter().fold([0i64; 3], |a, p| {
                    [a[0] + p.x as i64, a[1] + p.y as i64, a[2] + p.z as i64]
                });
                let cn = corners.len() as i64;
                let mid = v3(
                    (mid[0] / cn) as i32,
                    (mid[1] / cn) as i32,
                    (mid[2] / cn) as i32,
                );
                let ax = corners[0].sub(mid).norm();
                let ay = normal.cross(ax);
                let mut ordered: Vec<(i32, V3)> = corners
                    .iter()
                    .map(|p| {
                        let d = p.sub(mid);
                        (fx::atan2(d.dot(ay), d.dot(ax)), *p)
                    })
                    .collect();
                ordered.sort_by_key(|o| o.0);
                for t in 1..ordered.len() - 1 {
                    push_flat(out, [ordered[0].1, ordered[t].1, ordered[t + 1].1], normal);
                }
            }
        }
    }
}

/// `count` points in a level ring at height `y`, radius `r`, the first at
/// angle `phase`.
fn ring(count: i32, y: i32, r: i32, phase: i32) -> Vec<V3> {
    (0..count)
        .map(|k| {
            let a = phase + fx::TURN * k / count;
            v3(fx::mul(fx::cos(a), r), y, fx::mul(fx::sin(a), r))
        })
        .collect()
}

/// The golden ratio, Q16.
const PHI: i32 = 106_039;

/// An icosahedron's twelve corners, radius 0.5.
fn ico_points() -> Vec<V3> {
    let mut p = Vec::new();
    for a in [-ONE, ONE] {
        for b in [-PHI, PHI] {
            for c in [v3(0, a, b), v3(a, b, 0), v3(b, 0, a)] {
                p.push(c.norm().scale(H));
            }
        }
    }
    p
}

/// The icosahedron's corners and its edges' middles pushed out to the ball:
/// 42 points, whose hull is the 80-triangle geodesic ball.
fn geo_points() -> Vec<V3> {
    let ico = ico_points();
    let mut edge = i32::MAX;
    for (i, a) in ico.iter().enumerate() {
        for b in &ico[i + 1..] {
            edge = edge.min(a.dist(*b));
        }
    }
    let mut p = ico.clone();
    for (i, a) in ico.iter().enumerate() {
        for b in &ico[i + 1..] {
            if a.dist(*b) < edge + ONE / 100 {
                p.push(a.add(*b).norm().scale(H));
            }
        }
    }
    p
}

/// The points whose hull is each faceted mesh.
fn facet_points(mesh: Mesh) -> Vec<V3> {
    let q = |n: i32| ONE * n / 100;
    match mesh {
        Mesh::Ico => ico_points(),
        Mesh::Geo => geo_points(),
        Mesh::Octa => vec![
            v3(H, 0, 0),
            v3(-H, 0, 0),
            v3(0, H, 0),
            v3(0, -H, 0),
            v3(0, 0, H),
            v3(0, 0, -H),
        ],
        Mesh::Hex => [ring(6, -H, H, 0), ring(6, H, H, 0)].concat(),
        Mesh::Taper => [ring(6, -H, H, 0), ring(6, H, q(30), 0)].concat(),
        Mesh::Wedge => {
            let mut p = Vec::new();
            for x in [-H, H] {
                p.extend([v3(x, -H, -H), v3(x, -H, H), v3(x, H, -H)]);
            }
            p
        }
        Mesh::Pyramid => vec![
            v3(-H, -H, -H),
            v3(H, -H, -H),
            v3(H, -H, H),
            v3(-H, -H, H),
            v3(0, H, 0),
        ],
        Mesh::Spike => {
            let mut p = ring(5, -H, H, fx::QUARTER);
            p.push(v3(0, H, 0));
            p
        }
        Mesh::Barrel => {
            let s = fx::TURN / 16;
            [
                ring(8, -H, q(30), s),
                ring(8, -q(18), H, 0),
                ring(8, q(18), H, s),
                ring(8, H, q(30), 0),
            ]
            .concat()
        }
        Mesh::Rock => {
            // the geodesic ball's points, each pulled in by its own amount
            let mut dice = fx::Dice(0x0C4_5EED);
            geo_points()
                .into_iter()
                .map(|p| p.scale(ONE - dice.range(0, q(26))))
                .collect()
        }
        Mesh::Bevel => {
            let a = q(33);
            let mut p = Vec::new();
            for x in [-a, a] {
                for y in [-a, a] {
                    for f in [H, -H] {
                        p.extend([v3(f, x, y), v3(x, f, y), v3(x, y, f)]);
                    }
                }
            }
            p
        }
        Mesh::Shard => vec![
            v3(0, H, 0),
            v3(0, -H, 0),
            v3(H, q(10), 0),
            v3(-H, q(10), 0),
            v3(0, q(10), H),
            v3(0, q(10), -H),
        ],
        _ => Vec::new(),
    }
}

/// Vertex data of a mesh, counter-clockwise from outside.
pub fn vertices(mesh: Mesh) -> Vec<i32> {
    let mut o = Vec::new();
    if mesh.faceted() {
        hull(&mut o, &facet_points(mesh));
        return o;
    }
    match mesh {
        Mesh::Terrain => {}
        Mesh::Cube => {
            let p = |x: i32, y: i32, z: i32| v3(x * H, y * H, z * H);
            push_quad(&mut o, p(1, -1, 1), p(1, -1, -1), p(1, 1, -1), p(1, 1, 1));
            push_quad(
                &mut o,
                p(-1, -1, -1),
                p(-1, -1, 1),
                p(-1, 1, 1),
                p(-1, 1, -1),
            );
            push_quad(&mut o, p(-1, 1, 1), p(1, 1, 1), p(1, 1, -1), p(-1, 1, -1));
            push_quad(
                &mut o,
                p(-1, -1, -1),
                p(1, -1, -1),
                p(1, -1, 1),
                p(-1, -1, 1),
            );
            push_quad(&mut o, p(-1, -1, 1), p(1, -1, 1), p(1, 1, 1), p(-1, 1, 1));
            push_quad(
                &mut o,
                p(1, -1, -1),
                p(-1, -1, -1),
                p(-1, 1, -1),
                p(1, 1, -1),
            );
        }
        Mesh::Sphere => ball(&mut o, 8, 12, true),
        Mesh::Blob => ball(&mut o, 4, 6, false),
        Mesh::Cyl => {
            for k in 0..SIDES {
                let (a0, a1) = (rim(k, -H, H), rim(k + 1, -H, H));
                let (b0, b1) = (rim(k, H, H), rim(k + 1, H, H));
                // sides, smooth around
                let n0 = v3(a0.x, 0, a0.z).norm();
                let n1 = v3(a1.x, 0, a1.z).norm();
                push_smooth(&mut o, [a0, b1, a1], [n0, n1, n1], [ONE; 3]);
                push_smooth(&mut o, [a0, b0, b1], [n0, n0, n1], [ONE; 3]);
                push_tri(&mut o, v3(0, H, 0), b1, b0);
                push_tri(&mut o, v3(0, -H, 0), a0, a1);
            }
        }
        Mesh::Cone => {
            let tip = v3(0, H, 0);
            for k in 0..SIDES {
                let a0 = rim(k, -H, H);
                let a1 = rim(k + 1, -H, H);
                push_tri(&mut o, a0, tip, a1);
                push_tri(&mut o, v3(0, -H, 0), a0, a1);
            }
        }
        Mesh::Quad => {
            push_quad(
                &mut o,
                v3(-H, 0, -H),
                v3(-H, 0, H),
                v3(H, 0, H),
                v3(H, 0, -H),
            );
        }
        _ => {}
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mesh_is_whole_triangles_inside_the_unit_box() {
        for m in Mesh::ALL {
            let v = vertices(m);
            assert_eq!(v.len() % (VERTEX_VALUES * 3), 0, "{m:?}");
            if m != Mesh::Terrain {
                assert!(!v.is_empty(), "{m:?}");
            }
            for p in v.chunks(VERTEX_VALUES) {
                for c in &p[0..3] {
                    assert!(c.abs() <= H + 2, "{m:?} {c}");
                }
                let n = v3(p[3], p[4], p[5]);
                assert!((n.len() - ONE).abs() < 64, "{m:?} normal {n:?}");
            }
        }
    }

    fn corners(t: &[i32]) -> [V3; 3] {
        [0, 1, 2].map(|k| {
            let c = &t[k * VERTEX_VALUES..];
            v3(c[0], c[1], c[2])
        })
    }

    fn normals(t: &[i32]) -> [V3; 3] {
        [0, 1, 2].map(|k| {
            let c = &t[k * VERTEX_VALUES..];
            v3(c[3], c[4], c[5])
        })
    }

    #[test]
    fn faceted_meshes_are_flat_shaded_closed_and_face_outward() {
        for m in Mesh::ALL.into_iter().filter(|m| m.faceted()) {
            let v = vertices(m);
            let tris: Vec<&[i32]> = v.chunks(VERTEX_VALUES * 3).collect();
            let all: Vec<V3> = tris.iter().flat_map(|t| corners(t)).collect();
            let mean = all
                .iter()
                .fold(V3::ZERO, |a, p| a.add(*p))
                .scale(ONE / all.len() as i32);
            let mut edges: Vec<(V3, V3)> = Vec::new();
            for t in &tris {
                let p = corners(t);
                let n = normals(t);
                // one normal for the whole triangle: a flat facet
                assert!(n[0] == n[1] && n[1] == n[2], "{m:?} {n:?}");
                // wound counter-clockwise seen from the side it faces
                let wind = p[1].sub(p[0]).cross(p[2].sub(p[0]));
                assert!(wind.dot(n[0]) > 0, "{m:?} {p:?}");
                // and that side is the outside
                let mid = p[0].add(p[1]).add(p[2]).scale(ONE / 3);
                assert!(mid.sub(mean).dot(n[0]) > 0, "{m:?} {p:?}");
                edges.extend([(p[0], p[1]), (p[1], p[2]), (p[2], p[0])]);
            }
            // closed: every edge is met once each way by the next facet
            for (a, b) in &edges {
                let back = edges.iter().filter(|e| e.0 == *b && e.1 == *a).count();
                assert_eq!(back, 1, "{m:?} edge {a:?} {b:?}");
            }
        }
    }

    #[test]
    fn faceted_meshes_have_their_faces() {
        // triangles: a polygon of k corners fans into k - 2
        for (m, tris) in [
            (Mesh::Ico, 20),
            (Mesh::Geo, 80),
            (Mesh::Octa, 8),
            (Mesh::Hex, 6 * 2 + 2 * 4),
            (Mesh::Taper, 6 * 2 + 2 * 4),
            (Mesh::Wedge, 2 + 3 * 2),
            (Mesh::Pyramid, 4 + 2),
            (Mesh::Spike, 5 + 3),
            (Mesh::Barrel, 8 * 6 + 2 * 6),
            (Mesh::Bevel, 6 * 2 + 12 * 2 + 8),
            (Mesh::Shard, 8),
        ] {
            assert_eq!(vertices(m).len() / VERTEX_VALUES / 3, tris, "{m:?}");
        }
        let rock = vertices(Mesh::Rock).len() / VERTEX_VALUES / 3;
        assert!((50..=80).contains(&rock), "Rock {rock}");
        assert_ne!(vertices(Mesh::Rock), vertices(Mesh::Geo));
    }

    #[test]
    fn cube_faces_point_outward() {
        let v = vertices(Mesh::Cube);
        for p in v.chunks(VERTEX_VALUES) {
            let pos = v3(p[0], p[1], p[2]);
            let n = v3(p[3], p[4], p[5]);
            assert!(pos.dot(n) > 0);
        }
    }

    #[test]
    fn ball_normals_point_outward() {
        for m in [Mesh::Sphere, Mesh::Blob, Mesh::Cone, Mesh::Cyl] {
            let v = vertices(m);
            for t in v.chunks(VERTEX_VALUES * 3) {
                let c = v3(
                    (t[0] + t[9] + t[18]) / 3,
                    (t[1] + t[10] + t[19]) / 3,
                    (t[2] + t[11] + t[20]) / 3,
                );
                let n = v3(
                    t[3] + t[12] + t[21],
                    t[4] + t[13] + t[22],
                    t[5] + t[14] + t[23],
                );
                assert!(c.dot(n) > 0, "{m:?}");
            }
        }
    }
}
