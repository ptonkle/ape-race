//! The shapes everything is drawn from (PLAN P2). Each is a triangle list of
//! (position, normal, tint) in Q16, centred on the origin and one unit
//! across, so an instance's matrix carries its size. The tint is white on
//! every shape but the terrain, whose vertices carry the ground's colours.

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
}

impl Mesh {
    pub const ALL: [Mesh; 7] = [
        Mesh::Cube,
        Mesh::Sphere,
        Mesh::Blob,
        Mesh::Cyl,
        Mesh::Cone,
        Mesh::Quad,
        Mesh::Terrain,
    ];
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

/// Vertex data of a mesh, counter-clockwise from outside.
pub fn vertices(mesh: Mesh) -> Vec<i32> {
    let mut o = Vec::new();
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
