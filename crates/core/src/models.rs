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
