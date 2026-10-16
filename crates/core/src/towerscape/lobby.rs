//! Mape's Tower's menus (step 9): the lobby spot on a high bridge, the
//! keep's wall beside the ape and the sky, the cloud band and the
//! sub-towers behind it, and the title's flyover rising round the tower to
//! the summit and the tear.
//!
//! The lobby ape stands on the highest branch bridge under the cloud band,
//! the camera behind it looking along the keep's wall (the keep on the
//! left). `app::menu_dock` turns that camera so the keep's edge
//! ([`landmark`]) sits between the docked menus and the ape, as Sunset
//! Ruins does with its temple's mouth: the wall fills the space beside the
//! menus and the ape stands against the open sky.
//!
//! The flyover is a slow helix round the keep, out past every sub-tower,
//! from near the base to over the summit, looking in at the keep; over the
//! last stretch its look swings onto the tear.

use crate::course::Course;
use crate::fx::{cm, deg, int, mul, ONE, TURN};
use crate::geom::{facing, right_of, v3, yaw_of, V3};
use crate::render::Camera;
use crate::rules::TICKS_PER_SECOND;
use crate::tower::TowerLayout;

/// The lobby camera: this far behind the ape, this high over its feet,
/// aimed at this height over its feet (low and nearly level, as Sunset
/// Ruins' and Banana Bay Battle's).
pub const LOBBY_BACK: i32 = cm(480);
const LOBBY_EYE_UP: i32 = cm(85);
const LOBBY_AIM_UP: i32 = cm(75);
/// The landmark (the keep's edge) this high over the feet.
const MARK_UP: i32 = int(6);
/// The lobby view turned this far in from along the wall.
const LOOK_IN: i32 = deg(35);

/// The flyover: its radius out from the keep's wall (past every
/// sub-tower: they stand at most 66 m from the axis), its lowest and
/// highest eye over the keep's base and top, how many turns it takes
/// (from the west side round to the west again, half a turn over), and its
/// speed in metres a second.
const FLY_OUT: i32 = int(60);
const FLY_LOW: i32 = int(12);
const FLY_OVER: i32 = int(25);
const FLY_TURNS: i32 = 3 * TURN / 2;
const FLY_SPEED: i32 = 6;
/// The share of the rise over which the look swings from the keep onto
/// the tear (the last part).
const FLY_SWING: i32 = ONE * 3 / 10;

/// The lobby's feet stand this far in from the start deck's far edge.
const LOBBY_FROM_EDGE: i32 = cm(180);
/// Vines anchored this near the lobby's feet (the start's) are left out
/// of the lobby: they would hang between the camera and the view.
const LOBBY_CLEAR: i32 = int(14);

/// The lobby zone: the feet's place and the flat way the camera looks.
/// The start deck (outside the keep, 16 m up), the camera looking along
/// the keep's wall and a little in toward it (the keep on the left), the
/// ape near the deck's far edge so little of the deck shows ahead: the
/// wall, the sub-towers and the haze beyond (TL1).
pub fn lobby_spot(c: &Course, t: &TowerLayout) -> (V3, V3) {
    let Some(p) = c.platforms.first() else {
        return (c.starts[0], c.heading_at(0).flat().norm());
    };
    let mid = v3(p.centre.x, p.top, p.centre.z);
    let k = t.keep.centre;
    let out = mid.sub(v3(k.x, mid.y, k.z)).flat().norm();
    // the two ways along the wall: the one with the keep on the left
    let side = right_of(out);
    let along = if right_of(side).dot(out) > 0 {
        side
    } else {
        side.scale(-ONE)
    };
    // turned a little in toward the keep (a yaw up turns left), so the
    // menus' turn to put its edge beside the ape stays small
    let look = facing(yaw_of(along) + LOOK_IN, 0);
    // to the deck's edge straight ahead, then back off
    let f = p.fwd.flat().norm();
    let reach = |half: i32, d: i32| {
        if d.abs() < 16 {
            i32::MAX
        } else {
            crate::fx::div(half, d.abs())
        }
    };
    let edge = reach(p.half_len, look.dot(f)).min(reach(p.half_width, look.dot(right_of(f))));
    let feet = mid.add(look.scale((edge - LOBBY_FROM_EDGE).max(0)));
    (feet, look)
}

/// Whether vine `i` is left out of the lobby: anchored near its feet.
pub fn hidden_in_lobby(c: &Course, t: &TowerLayout, i: usize) -> bool {
    let (feet, _) = lobby_spot(c, t);
    c.vines
        .get(i)
        .is_some_and(|v| !v.anchor.sub(feet).flat().longer_than(LOBBY_CLEAR))
}


/// What the lobby frames between the menus and the ape: the keep's edge
/// as seen from the ape (where the wall meets the sky ahead), a little
/// over the feet.
pub fn landmark(c: &Course, t: &TowerLayout) -> V3 {
    let (feet, look) = lobby_spot(c, t);
    let k = t.keep.centre;
    let r = t.keep.radius;
    let to = feet.sub(v3(k.x, feet.y, k.z));
    let dist = to.len().max(r + 1);
    let out = to.norm();
    // the tangent point from the feet: r²/d along the radius, and
    // r·√(1 − (r/d)²) across it, on the side the camera looks
    let along = mul(r, crate::fx::div(r, dist));
    let q = crate::fx::div(r, dist);
    let across = mul(r, crate::fx::sqrt(ONE - mul(q, q)));
    let side = right_of(out);
    let side = if side.dot(look) > 0 {
        side
    } else {
        side.scale(-ONE)
    };
    v3(k.x, feet.y + MARK_UP, k.z)
        .add(out.scale(along))
        .add(side.scale(across))
}

/// The lobby camera `back` behind an ape at `feet`, looking `look` (flat).
pub fn lobby_camera_at(feet: V3, look: V3, back: i32) -> Camera {
    let eye = feet.sub(look.scale(back)).add(v3(0, LOBBY_EYE_UP, 0));
    let target = feet.add(v3(0, LOBBY_AIM_UP, 0));
    Camera {
        eye,
        fwd: target.sub(eye).norm(),
        fov: deg(34),
    }
}

/// The lobby camera at its usual distance.
pub fn lobby_camera(feet: V3, look: V3) -> Camera {
    lobby_camera_at(feet, look, LOBBY_BACK)
}

/// The menus' preview on Mape's Tower: the ape on the bridge, the camera
/// behind it looking along the keep's wall.
pub fn preview(c: &Course) -> (V3, i32, Camera) {
    let (feet, look) = match c.tower.as_deref() {
        Some(t) => lobby_spot(c, t),
        None => (c.starts[0], c.heading_at(0).flat().norm()),
    };
    let camera = lobby_camera(feet, look);
    (feet, yaw_of(camera.eye.sub(feet).flat()), camera)
}

/// The flyover helix's length in metres (Q16): round `FLY_TURNS` at the
/// radius, rising from low to over the top.
fn fly_len(t: &TowerLayout) -> i32 {
    let r = t.keep.radius + FLY_OUT;
    let round = (mul(r, FLY_TURNS) as i64 * 2 * 314_159 / 100_000) as i32;
    let rise = t.keep.height + FLY_OVER - FLY_LOW;
    let sq = |v: i32| (v as i64 / 256).pow(2);
    ((sq(round) + sq(rise)).isqrt() * 256) as i32
}

/// The flyover's eye a share `u` (0 to ONE) of the way up: the west side
/// (−x, the climb's side of the keep, away from the mirror) at the top.
fn fly_eye(t: &TowerLayout, u: i32) -> V3 {
    let k = t.keep.centre;
    let r = t.keep.radius + FLY_OUT;
    let yaw = -TURN / 4 - mul(ONE - u, FLY_TURNS);
    let y = k.y + FLY_LOW + mul(t.keep.height + FLY_OVER - FLY_LOW, u);
    v3(
        k.x + mul(r, crate::fx::sin(yaw)),
        y,
        k.z + mul(r, crate::fx::cos(yaw)),
    )
}

/// The title's flyover on Mape's Tower: up round the keep, `from` metres
/// along the helix, looking in at the keep a little below the eye; over
/// the last stretch the look swings onto the tear over the summit.
pub fn flyover_camera(c: &Course, ticks: u32, from: i32) -> Camera {
    let Some(t) = c.tower.as_deref() else {
        return lobby_camera(c.starts[0], c.heading_at(0).flat().norm());
    };
    let len = fly_len(t).max(int(1));
    let s = ((ticks as i64 * int(FLY_SPEED) as i64 / TICKS_PER_SECOND as i64 + from as i64)
        % len as i64) as i32;
    let u = crate::fx::div(s, len).clamp(0, ONE);
    let eye = fly_eye(t, u);
    let k = t.keep.centre;
    let axis = v3(k.x, eye.y - int(8), k.z);
    let w = crate::fx::div(u - (ONE - FLY_SWING), FLY_SWING).clamp(0, ONE);
    // smoothstep: no jolt where the swing starts or ends
    let w = mul(mul(w, w), 3 * ONE - 2 * w);
    let tear = t.summit.tear;
    let look = v3(
        axis.x + mul(tear.x - axis.x, w),
        axis.y + mul(tear.y - axis.y, w),
        axis.z + mul(tear.z - axis.z, w),
    );
    Camera {
        eye,
        fwd: look.sub(eye).norm(),
        fov: deg(60),
    }
}

/// How far the flyover runs before it starts again (for the map card and
/// the tests).
pub fn fly_length(c: &Course) -> i32 {
    c.tower.as_deref().map_or(int(1), fly_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Map;

    #[test]
    fn the_lobby_ape_stands_on_the_start_deck_near_its_edge_with_the_keep_beside_it() {
        for seed in [1u64, 2, 3] {
            let c = crate::tower::tower_seeded(seed);
            let t = c.tower.as_deref().unwrap();
            let (feet, look) = lobby_spot(&c, t);
            // on the start deck's top, a metre in from every edge, and
            // within 3 m of its edge straight ahead
            let p = &c.platforms[0];
            let inside = |q: V3, by: i32| {
                let d = q.sub(p.centre).flat();
                d.dot(p.fwd).abs() <= p.half_len - by
                    && d.dot(right_of(p.fwd)).abs() <= p.half_width - by
            };
            assert_eq!(feet.y, p.top, "seed {seed}: on the deck's top");
            assert!(inside(feet, int(1)), "seed {seed}: on the deck");
            let ahead = feet.add(look.scale(int(3)));
            assert!(!inside(ahead, 0), "seed {seed}: far from the edge");
            // the start's vines are left out, the rest are not
            let hidden = (0..c.vines.len()).filter(|&i| hidden_in_lobby(&c, t, i));
            let n = hidden.count();
            assert!(n > 0 && n < c.vines.len() / 4, "seed {seed}: {n} left out");
            // the keep on the camera's left
            let k = t.keep.centre;
            let to_keep = v3(k.x, feet.y, k.z).sub(feet);
            assert!(right_of(look).dot(to_keep) < 0, "seed {seed}: keep on the left");
            // the landmark is on the keep's wall, ahead of the ape
            let mark = landmark(&c, t);
            let r = v3(mark.x - k.x, 0, mark.z - k.z).len();
            assert!((r - t.keep.radius).abs() < cm(20), "seed {seed}: mark off the wall");
            assert!(mark.sub(feet).flat().dot(look) > 0, "seed {seed}: mark behind");
        }
        let c = Map::Tower.course();
        assert_eq!(crate::scene::preview(&c).feet, lobby_spot(&c, c.tower.as_deref().unwrap()).0);
    }

    #[test]
    fn the_flyover_rises_round_the_keep_clear_of_the_towers_to_the_tear() {
        for seed in [1u64, 2, 3] {
            let c = crate::tower::tower_seeded(seed);
            let t = c.tower.as_deref().unwrap();
            let len = fly_len(t);
            let tps = TICKS_PER_SECOND as u32;
            let lap = (len as i64 * tps as i64 / int(FLY_SPEED) as i64) as u32;
            let k = t.keep.centre;
            let mut last_y = i32::MIN;
            for tick in (0..lap).step_by(tps as usize / 2) {
                let cam = flyover_camera(&c, tick, 0);
                let e = cam.eye;
                // always rising
                assert!(e.y >= last_y, "seed {seed} tick {tick}: falls");
                last_y = e.y;
                // clear of the keep and every sub-tower by 10 m, on the
                // climb's side of the mirror
                let r = v3(e.x - k.x, 0, e.z - k.z).len();
                assert!(r > t.keep.radius + int(10), "seed {seed} tick {tick}: in the keep");
                for b in &t.branches {
                    let s = b.sub.centre;
                    let d = v3(e.x - s.x, 0, e.z - s.z).len();
                    assert!(d > b.sub.radius + int(10), "seed {seed} tick {tick}: in a sub-tower");
                }
                assert!(e.x < t.zipper.mirror_x - int(5), "seed {seed} tick {tick}: in the zipper");
            }
            // it ends over the summit, looking at the tear
            let end = flyover_camera(&c, lap - 1, 0);
            assert!(end.eye.y > k.y + t.keep.height, "seed {seed}: ends over the top");
            let to_tear = t.summit.tear.sub(end.eye).norm();
            assert!(end.fwd.dot(to_tear) > ONE * 99 / 100, "seed {seed}: ends on the tear");
        }
    }
}
