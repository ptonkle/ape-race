//! The tournament's podium (TRIAGE T103 part 2): after the final standings,
//! every ape of the tournament dances its finish dance by overall place.
//! The top three stand on a three-step podium (1st in the middle and
//! tallest, 2nd on the left, 3rd on the right, as the camera sees it); the
//! rest stand on the ground in front, in place order from left to right.
//! It stands on the small jungle stage (the lineup's), at the origin.

use crate::dance::Dance;
use crate::fx::{self, cm, deg, mul, ONE};
use crate::geom::{v3, V3};
use crate::mesh::Mesh;
use crate::models::{self, Gear, Pose};
use crate::render::{placed, rgb, Batch, Camera, Rgb};
use crate::rules::Kind;

/// A step's width (x) and depth (z).
pub const STEP_W: i32 = cm(150);
pub const STEP_D: i32 = cm(130);
/// The steps' heights, 1st, 2nd, 3rd.
pub const HEIGHTS: [i32; 3] = [cm(90), cm(60), cm(35)];
/// The ground row in front: how far in front of the steps' faces, and
/// the gap between apes.
const ROW_IN_FRONT: i32 = cm(110);
const ROW_GAP: i32 = cm(130);

/// The steps' stone, and the trim round each top: gold, silver, bronze.
const STONE: Rgb = rgb(222, 214, 196);
const TRIM: [Rgb; 3] = [rgb(240, 190, 40), rgb(196, 204, 214), rgb(196, 120, 60)];

/// A step's middle on x, by place (0: 1st).
fn step_x(place: usize) -> i32 {
    match place {
        0 => 0,
        1 => -STEP_W,
        _ => STEP_W,
    }
}

/// Where each of `n` apes stands, in overall place order: its feet. The
/// top three on their steps' tops, the rest on the ground in front.
pub fn spots(n: usize) -> Vec<V3> {
    let rest = n.saturating_sub(3) as i32;
    (0..n)
        .map(|i| {
            if i < 3 {
                v3(step_x(i), HEIGHTS[i], 0)
            } else {
                let k = i as i32 - 3;
                let x = (2 * k - (rest - 1)) * ROW_GAP / 2;
                v3(x, 0, STEP_D / 2 + ROW_IN_FRONT)
            }
        })
        .collect()
}

/// The steps (as many as there are apes, at most three), then each ape
/// dancing on its spot, facing the camera. `apes`: by overall place, each
/// one's kind, colours and gear.
pub fn draw(b: &mut Batch, apes: &[(Kind, u8, Gear)], tick: u32) {
    for (place, h) in HEIGHTS.iter().enumerate().take(apes.len()) {
        let x = step_x(place);
        b.push(
            Mesh::Cube,
            &placed(v3(x, h / 2, 0), 0, v3(STEP_W, *h, STEP_D)),
            STONE,
            0,
        );
        // a coloured band round the top
        let band = cm(8);
        b.push(
            Mesh::Cube,
            &placed(
                v3(x, h - band / 2, 0),
                0,
                v3(STEP_W + cm(4), band, STEP_D + cm(4)),
            ),
            TRIM[place],
            0,
        );
    }
    for (i, (feet, (kind, palette, gear))) in spots(apes.len()).into_iter().zip(apes).enumerate() {
        // each its own beat, so they do not move as one
        let beat = tick + i as u32 * 17;
        let pose = Pose {
            gear: *gear,
            dance: Some(Dance { beat, weight: ONE }),
            ..Pose::standing(feet, 0, tick)
        };
        models::draw_in(b, *kind, *palette, &pose);
    }
}

/// A camera that frames the podium and the row in front at `aspect`
/// (width / height, Q16), the podium moved right by `right` (Q16, a
/// share of the half width: the docked panels cover the left).
pub fn camera(aspect: i32, right: i32) -> Camera {
    let fov = deg(34);
    let half = fov / 2;
    let tan = fx::div(fx::sin(half), fx::cos(half));
    // about 6 m wide and 3 m tall to fit, in the part of the view the
    // panels leave open
    let open = (ONE - right).max(ONE / 4);
    let need_w = fx::div(fx::div(cm(380), open), mul(tan, aspect.max(ONE / 4)));
    let need_h = fx::div(cm(170), tan);
    let back = need_w.max(need_h) + cm(100);
    // looking left of the podium puts it right of the middle
    let shift = mul(mul(mul(tan, aspect.max(ONE / 4)), back), right);
    let target = v3(-shift, cm(110), 0);
    let eye = v3(-shift, cm(230), back);
    Camera {
        eye,
        fwd: target.sub(eye).norm(),
        fov,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_top_three_stand_on_the_steps_and_the_rest_in_front_in_order() {
        let s = spots(5);
        // 1st in the middle and highest, 2nd left and lower, 3rd right and
        // lowest of the three
        assert_eq!(s[0].x, 0);
        assert!(s[1].x < 0 && s[2].x > 0);
        assert!(s[0].y > s[1].y && s[1].y > s[2].y && s[2].y > 0);
        // the rest on the ground in front of the steps, left to right
        for r in &s[3..] {
            assert_eq!(r.y, 0);
            assert!(r.z > STEP_D / 2);
        }
        assert!(s[3].x < s[4].x);
        // fewer apes: only the steps' spots
        assert_eq!(spots(2), s[..2].to_vec());
    }
}
