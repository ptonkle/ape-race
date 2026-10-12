//! What the page draws (PLAN P2): one 3×4 matrix and a colour per shape,
//! grouped by mesh so each mesh is one instanced draw. The static buffer
//! holds the course's scenery, uploaded once; the dynamic buffer holds what
//! moves, sent every frame. Every number the page needs to read these
//! arrays (strides, offsets, sizes, ids) is in [`numbers_json`], so the page
//! holds none of its own.

use serde_json::json;

use crate::fx::{self, ONE};
use crate::geom::{Affine, Mat4, V3};
use crate::mesh::{self, Mesh};

/// A colour, each channel 0..=ONE.
pub type Rgb = [i32; 3];

/// From 0–255 channels.
pub const fn rgb(r: i32, g: i32, b: i32) -> Rgb {
    [r * ONE / 255, g * ONE / 255, b * ONE / 255]
}

/// Values per instance: three rows (column x, y, z of each axis, then the
/// translation), then r, g, b and glow.
pub const INSTANCE_VALUES: usize = 16;
/// Values per draw: buffer, mesh, byte offset, instance count, then the
/// first vertex and the vertex count of the mesh to draw (0 and 0: all of
/// it; the terrain is drawn a chunk at a time).
pub const DRAW_VALUES: usize = 6;
pub const BYTES_PER_VALUE: usize = 4;
pub const STATIC_BUFFER: u32 = 0;
pub const DYNAMIC_BUFFER: u32 = 1;

pub const NEAR: i32 = ONE / 5;
pub const FAR: i32 = fx::int(600);

/// Shapes to draw, by mesh.
#[derive(Debug, Clone, Default)]
pub struct Batch {
    lists: Vec<Vec<i32>>,
}

impl Batch {
    pub fn new() -> Batch {
        Batch {
            lists: vec![Vec::new(); Mesh::ALL.len()],
        }
    }

    /// One shape. `glow` 0 is lit normally; ONE is drawn at full colour
    /// regardless of light (fruit, effects).
    pub fn push(&mut self, mesh: Mesh, a: &Affine, c: Rgb, glow: i32) {
        self.lists[mesh as usize].extend_from_slice(&[
            a.c0.x, a.c1.x, a.c2.x, a.t.x, a.c0.y, a.c1.y, a.c2.y, a.t.y, a.c0.z, a.c1.z, a.c2.z,
            a.t.z, c[0], c[1], c[2], glow,
        ]);
    }

    pub fn count(&self) -> usize {
        self.lists.iter().map(|m| m.len() / INSTANCE_VALUES).sum()
    }

    pub fn count_of(&self, mesh: Mesh) -> usize {
        self.lists[mesh as usize].len() / INSTANCE_VALUES
    }

    /// The instance values, and one draw per mesh that has any.
    pub fn finish(self, buffer: u32) -> (Vec<i32>, Vec<u32>) {
        let mut values = Vec::new();
        let draws = self.finish_into(buffer, &mut values);
        (values, draws)
    }

    /// Appends the instance values to `values` and returns the draws, whose
    /// offsets count from the start of `values`.
    pub fn finish_into(self, buffer: u32, values: &mut Vec<i32>) -> Vec<u32> {
        let mut draws = Vec::new();
        for m in Mesh::ALL {
            let v = &self.lists[m as usize];
            if v.is_empty() {
                continue;
            }
            draws.extend_from_slice(&[
                buffer,
                m as u32,
                (values.len() * BYTES_PER_VALUE) as u32,
                (v.len() / INSTANCE_VALUES) as u32,
                0,
                0,
            ]);
            values.extend_from_slice(v);
        }
        draws
    }
}

/// A shape of size `size` (x, y, z) at `at`, turned `yaw` about +y.
pub fn placed(at: V3, yaw: i32, size: V3) -> Affine {
    Affine::translate(at)
        .then(&Affine::rot_y(yaw))
        .then(&Affine::scale(size))
}

/// A rod of `width` from `a` to `b` (a cylinder or a cube along the line).
pub fn rod(a: V3, b: V3, width: i32) -> Affine {
    let d = b.sub(a);
    let len = d.len();
    Affine::translate(a.add(d.scale(ONE / 2)))
        .then(&Affine::along(d))
        .then(&Affine::scale(crate::geom::v3(width, len, width)))
}

pub fn shade(c: Rgb, pct: i32) -> Rgb {
    c.map(|v| (v * pct / 100).clamp(0, ONE))
}

pub fn mix(a: Rgb, b: Rgb, t: i32) -> Rgb {
    [
        fx::lerp(a[0], b[0], t),
        fx::lerp(a[1], b[1], t),
        fx::lerp(a[2], b[2], t),
    ]
}

/// A camera: where it is, where it looks, and its vertical field of view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Camera {
    pub eye: V3,
    pub fwd: V3,
    pub fov: i32,
}

impl Camera {
    pub fn matrix(&self, aspect: i32) -> Mat4 {
        Mat4::perspective(self.fov, aspect, NEAR, FAR).mul(&Mat4::look(self.eye, self.fwd, V3::UP))
    }

    /// Where a world point lands on screen, in hundredths of the width and
    /// height from the top left (Q16); `None` behind the camera.
    pub fn screen(&self, aspect: i32, p: V3) -> Option<(i32, i32)> {
        let [x, y, _, w] = self.matrix(aspect).project(p);
        if w <= ONE / 100 {
            return None;
        }
        let nx = fx::div(x, w);
        let ny = fx::div(y, w);
        Some(((nx + ONE) * 50, (ONE - ny) * 50))
    }
}

/// The view values the page uploads each frame: the matrix (16), the eye
/// (3), the fog's start and end distances (2), then the fog's colour (3)
/// and the sky's (3), which change at top speed (TRIAGE T20).
pub fn view_values(cam: &Camera, aspect: i32, fog: (i32, i32), colours: (Rgb, Rgb)) -> Vec<i32> {
    let mut v = cam.matrix(aspect).0.to_vec();
    v.extend([cam.eye.x, cam.eye.y, cam.eye.z, fog.0, fog.1]);
    v.extend(colours.1);
    v.extend(colours.0);
    v
}

/// Every number the page's renderer needs, as JSON.
pub fn numbers_json() -> String {
    let b = BYTES_PER_VALUE as u32;
    let meshes: Vec<_> = Mesh::ALL
        .iter()
        .map(|m| {
            json!({
                "id": *m as u32,
                "vertices": mesh::vertices(*m).len() / mesh::VERTEX_VALUES,
            })
        })
        .collect();
    json!({
        "meshes": meshes,
        "terrain_mesh": Mesh::Terrain as u32,
        "vertex": {
            "values": mesh::VERTEX_VALUES,
            "stride": mesh::VERTEX_VALUES as u32 * b,
            "attributes": [["aPos", 3, 0], ["aNormal", 3, 3 * b], ["aTint", 3, 6 * b]],
        },
        "instance": {
            "values": INSTANCE_VALUES,
            "stride": INSTANCE_VALUES as u32 * b,
            "attributes": [["iA", 4, 0], ["iB", 4, 4 * b], ["iC", 4, 8 * b], ["iD", 4, 12 * b]],
        },
        "draw_values": DRAW_VALUES,
        "buffers": { "static": STATIC_BUFFER, "dynamic": DYNAMIC_BUFFER },
        "view": { "matrix": [0, 16], "eye": [16, 19], "fog": [19, 21], "fog_colour": [21, 24], "sky": [24, 27] },
        "uniforms": [["uLightDir", 3], ["uLight", 3], ["uSky", 3], ["uFog", 3], ["uGround", 3]],
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::{deg, int};
    use crate::geom::{facing, v3};

    #[test]
    fn draws_cover_every_instance_in_order() {
        let mut b = Batch::new();
        let a = placed(V3::ZERO, 0, v3(ONE, ONE, ONE));
        b.push(Mesh::Sphere, &a, [ONE; 3], 0);
        b.push(Mesh::Cube, &a, [ONE; 3], 0);
        b.push(Mesh::Cube, &a, [ONE; 3], 0);
        let (values, draws) = b.finish(DYNAMIC_BUFFER);
        assert_eq!(values.len(), 3 * INSTANCE_VALUES);
        // cube first (mesh 0), two of them, then the sphere after them
        assert_eq!(
            draws,
            vec![
                1,
                0,
                0,
                2,
                0,
                0,
                1,
                1,
                (2 * INSTANCE_VALUES * 4) as u32,
                1,
                0,
                0
            ]
        );
    }

    #[test]
    fn a_rod_reaches_from_end_to_end() {
        let a = v3(int(1), int(2), int(3));
        let b = v3(int(4), int(-2), int(3));
        let r = rod(a, b, ONE / 10);
        let top = r.apply(v3(0, ONE / 2, 0));
        let bottom = r.apply(v3(0, -ONE / 2, 0));
        assert!(top.dist(b) < ONE / 100, "{top:?}");
        assert!(bottom.dist(a) < ONE / 100);
    }

    #[test]
    fn the_point_looked_at_is_the_middle_of_the_screen() {
        let c = Camera {
            eye: v3(0, int(2), int(10)),
            fwd: facing(0, -deg(5)),
            fov: deg(70),
        };
        let p = c.eye.add(c.fwd.scale(int(20)));
        let (x, y) = c.screen(ONE, p).unwrap();
        assert!((x - 50 * ONE).abs() < ONE / 4 && (y - 50 * ONE).abs() < ONE / 4);
        assert!(c.screen(ONE, c.eye.sub(c.fwd.scale(int(5)))).is_none());
    }

    #[test]
    fn numbers_name_every_mesh() {
        let n: serde_json::Value = serde_json::from_str(&numbers_json()).unwrap();
        assert_eq!(n["meshes"].as_array().unwrap().len(), Mesh::ALL.len());
        assert_eq!(n["instance"]["stride"], 64);
    }
}
