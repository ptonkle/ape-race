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
/// Values per draw: buffer, mesh, byte offset, instance count, the first
/// vertex and the vertex count of the mesh to draw (0 and 0: all of it; the
/// terrain is drawn a chunk at a time), then what it is for the colours at
/// speed: 0 keeps its colour (vines, apes, items), 1 is scenery, 2 the
/// ground (TRIAGE row 20), 3 the vines and their flowers (kept bright in
/// the colourblind mode, TRIAGE T33).
pub const DRAW_VALUES: usize = 7;
pub const KEEPS_COLOUR: u32 = 0;
pub const SCENERY: u32 = 1;
pub const GROUND: u32 = 2;
pub const VINES: u32 = 3;
/// 4: soft glows (`Batch::push_halo`), added over everything drawn before
/// them, so they come last.
pub const HALO: u32 = 4;
pub const BYTES_PER_VALUE: usize = 4;
pub const STATIC_BUFFER: u32 = 0;
pub const DYNAMIC_BUFFER: u32 = 1;

pub const NEAR: i32 = ONE / 5;
pub const FAR: i32 = fx::int(600);

/// Shapes to draw, by mesh.
#[derive(Debug, Clone, Default)]
pub struct Batch {
    lists: Vec<Vec<i32>>,
    /// soft glows, drawn after everything else (`HALO`)
    halos: Option<Box<Batch>>,
}

impl Batch {
    pub fn new() -> Batch {
        Batch {
            lists: vec![Vec::new(); Mesh::ALL.len()],
            halos: None,
        }
    }

    /// A soft glow of colour `c` through the shape, strongest (`strength`)
    /// where it faces the eye and fading to nothing at its outline, added
    /// over what is behind it; drawn last (`take_halos`, `HALO`). A
    /// `strength` over ONE adds evenly, by the part over ONE.
    pub fn push_halo(&mut self, mesh: Mesh, a: &Affine, c: Rgb, strength: i32) {
        self.halos
            .get_or_insert_with(|| Box::new(Batch::new()))
            .push(mesh, a, c, strength);
    }

    /// The halos pushed into this batch, taken out of it.
    pub fn take_halos(&mut self) -> Batch {
        self.halos.take().map_or_else(Batch::new, |b| *b)
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

    /// Where each mesh's list ends now, for `recolour_since`.
    pub fn marks(&self) -> Vec<usize> {
        self.lists.iter().map(|l| l.len()).collect()
    }

    /// Gives every shape pushed since `marks` the colour `f` makes of its
    /// colour and glow.
    pub fn recolour_since(&mut self, marks: &[usize], f: impl Fn(Rgb, i32) -> Rgb) {
        for (list, from) in self.lists.iter_mut().zip(marks) {
            for s in list[*from..].chunks_mut(INSTANCE_VALUES) {
                let c = f([s[12], s[13], s[14]], s[15]);
                s[12..15].copy_from_slice(&c);
            }
        }
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
        self.finish_as(buffer, None, values)
    }

    /// As `finish_into`, every draw marked `kind` when given (else by its
    /// buffer and mesh).
    pub fn finish_as(self, buffer: u32, kind: Option<u32>, values: &mut Vec<i32>) -> Vec<u32> {
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
                if let Some(k) = kind {
                    k
                } else if buffer != STATIC_BUFFER {
                    KEEPS_COLOUR
                } else if m == Mesh::Terrain {
                    GROUND
                } else {
                    SCENERY
                },
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
/// and the sky's (3), which change at top speed (TRIAGE T20); how wild
/// the jungle has turned and its colours (10); and how muted the world is
/// (1: the colourblind mode, TRIAGE T33); then the light's direction and
/// colour and the ground's bounce (9: each map has its own, the City at
/// dusk).
pub fn view_values(
    cam: &Camera,
    aspect: i32,
    fog: (i32, i32),
    colours: (Rgb, Rgb),
    world: (i32, [Rgb; 3]),
    mute: i32,
    light: (V3, Rgb, Rgb),
) -> Vec<i32> {
    let mut v = cam.matrix(aspect).0.to_vec();
    v.extend([cam.eye.x, cam.eye.y, cam.eye.z, fog.0, fog.1]);
    v.extend(colours.1);
    v.extend(colours.0);
    // how far the scenery has turned wild (0..ONE), and to which colour
    // (TRIAGE row 18, a try)
    v.push(world.0);
    for c in world.1 {
        v.extend(c);
    }
    v.push(mute);
    v.extend([light.0.x, light.0.y, light.0.z]);
    v.extend(light.1);
    v.extend(light.2);
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
        "view": { "matrix": [0, 16], "eye": [16, 19], "fog": [19, 21], "fog_colour": [21, 24], "sky": [24, 27], "trip": 27, "wild_ground": [28, 31], "wild_wood": [31, 34], "wild_leaves": [34, 37], "mute": 37, "light_dir": [38, 41], "light": [41, 44], "ground": [44, 47] },
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
                0,
                1,
                1,
                (2 * INSTANCE_VALUES * 4) as u32,
                1,
                0,
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
