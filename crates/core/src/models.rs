//! The six apes' models (PLAN P8), built from the unit meshes. Placeholder:
//! every kind is a brown ball until the models work lands.

use crate::fx::{cm, ONE};
use crate::geom::{v3, V3};
use crate::mesh::Mesh;
use crate::render::{placed, rgb, Batch};
use crate::rules::Kind;

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
}

/// Draws one ape.
pub fn draw(b: &mut Batch, kind: Kind, pose: &Pose) {
    let _ = kind;
    let body = pose.hands.sub(v3(0, cm(70), 0));
    b.push(
        Mesh::Sphere,
        &placed(body, pose.yaw, v3(ONE, ONE * 3 / 2, ONE)),
        rgb(120, 80, 50),
        0,
    );
}

/// All six side by side (placeholder until the models work lands).
pub fn lineup(b: &mut Batch, tick: u32) {
    for (i, k) in Kind::ALL.iter().enumerate() {
        let pose = Pose {
            hands: v3((i as i32 - 3) * cm(180) + cm(90), cm(250), 0),
            yaw: 0,
            rope: None,
            vel: V3::ZERO,
            grounded: true,
            stunned: false,
            tick,
            scale: 100,
        };
        draw(b, *k, &pose);
    }
}

pub fn lineup_camera(aspect: i32) -> crate::render::Camera {
    let _ = aspect;
    crate::render::Camera {
        eye: v3(0, cm(200), crate::fx::int(9)),
        fwd: v3(0, -cm(10), -ONE).norm(),
        fov: crate::fx::deg(60),
    }
}

pub fn showcase(b: &mut Batch, kind: Kind, tick: u32) {
    let pose = Pose {
        hands: v3(0, cm(250), 0),
        yaw: (tick as i32) * (crate::fx::TURN / 720),
        rope: None,
        vel: V3::ZERO,
        grounded: true,
        stunned: false,
        tick,
        scale: 100,
    };
    draw(b, kind, &pose);
}

pub fn showcase_camera(aspect: i32) -> crate::render::Camera {
    let _ = aspect;
    crate::render::Camera {
        eye: v3(-cm(150), cm(220), crate::fx::int(5)),
        fwd: v3(cm(30), -cm(15), -ONE).norm(),
        fov: crate::fx::deg(50),
    }
}
