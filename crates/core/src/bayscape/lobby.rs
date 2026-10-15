//! Banana Bay Battle's menus (step 9): the lobby spot on the pier, with the
//! battle fleet behind the ape, and the title's flyover alongside the
//! fleet.
//!
//! The lobby ape stands on the pier's planks (`BayLayout.lobby`, at the
//! dock's landward end), the camera behind it looking out down the course
//! at the fleet. `app::menu_dock` turns that camera so the fleet's
//! landmark ship ([`landmark`]) sits between the docked menus and the
//! ape, as Sunset Ruins does with its temple's mouth.
//!
//! The flyover glides alongside the line of ships, out on the pirates'
//! side (the sunlit one) and above the hulls, looking ahead and in at the
//! fleet: never through a hull, a mast or a sail, never under the sea,
//! and it ends before the island.

use crate::bay::{BayLayout, Ship};
use crate::course::Course;
use crate::fx::{cm, deg, int, mul, sin, ONE};
use crate::geom::{facing, right_of, v3, yaw_of, V3};
use crate::render::Camera;
use crate::rules::TICKS_PER_SECOND;

/// The lobby camera: this far behind the ape, this high over its feet,
/// aimed at this height over its feet (as Sunset Ruins': low and nearly
/// level, the fleet and the sky behind the ape, not the planks). The menus
/// back it further off where the open part beside them is narrow
/// (`app::menu_dock`), so a gorilla's reach stays in the frame.
pub const LOBBY_BACK: i32 = cm(480);
const LOBBY_EYE_UP: i32 = cm(85);
const LOBBY_AIM_UP: i32 = cm(75);

/// The landmark is the biggest ship this far out from the lobby (nearer
/// ones fill the frame; further ones are lost in the haze).
const LANDMARK_NEAR: i32 = int(30);
const LANDMARK_FAR: i32 = int(170);

/// The flyover: how far out to the side of the line (the hulls and yard
/// ends reach about 16 m, BA8), how high over the water (mid-mast), how
/// far ahead it looks, and its speed (metres a second).
const FLY_OUT: i32 = int(27);
const FLY_UP: i32 = int(17);
const FLY_AHEAD: i32 = int(45);
const FLY_LOOK_UP: i32 = int(7);
const FLY_SPEED: i32 = 6;
/// The flyover starts this far down the course (past the start gantry)
/// and ends this far before the island's beach.
const FLY_FROM: i32 = int(40);
const FLY_SHORT: i32 = int(30);

/// The lobby zone: the feet's place (exactly on the pier's planks) and the
/// flat way the camera looks (down the course, out at the fleet, as
/// `BayLayout.lobby` faces). The ape stands near the pier's seaward end,
/// not at the contract's spot by the landward end: the deck is 12 m over
/// the sea, so from a level camera there 40 m of planks hid every hull
/// below its rigging (BL1). From the end the planks hide only the sea
/// near the pier; the first ships show whole.
pub fn lobby_spot(bay: &BayLayout) -> (V3, V3) {
    let d = &bay.dock;
    let fwd = d.to.sub(d.from).flat().norm();
    let feet =
        d.to.sub(fwd.scale(LOBBY_FROM_END))
            .add(right_of(fwd).scale(LOBBY_ACROSS));
    (v3(feet.x, d.from.y, feet.z), facing(bay.lobby.1, 0))
}

/// Whether vine `i` is one of the start row's ropes (hung from the gantry
/// over the pier's end), which the lobby leaves out.
pub fn start_row(c: &Course, bay: &BayLayout, i: usize) -> bool {
    matches!(bay.hangers.get(i), Some(crate::bay::Hanger::Shore { .. }))
        && c.vines[i].anchor.sub(bay.dock.to).flat().len() < int(20)
}

/// The lobby's feet: this far in from the pier's seaward end, and this
/// far right of its middle (between the start row's ropes, clear of the
/// bollards and lanterns at the corners).
const LOBBY_FROM_END: i32 = cm(180);
const LOBBY_ACROSS: i32 = int(3);

/// What the lobby frames between the menus and the ape: the biggest ship
/// out ahead (the nearest of the biggest), at its deck (its masts and flags rise
/// over it).
pub fn landmark(bay: &BayLayout) -> V3 {
    let (feet, look) = lobby_spot(bay);
    let ahead = |s: &Ship| s.at.sub(feet).flat().dot(look);
    bay.ships
        .iter()
        .filter(|s| (LANDMARK_NEAR..=LANDMARK_FAR).contains(&ahead(s)))
        .max_by_key(|s| (s.length, -ahead(s)))
        .or(bay.ships.first())
        .map_or(feet.add(look.scale(int(80))), |s| {
            s.at.add(v3(0, s.deck, 0))
        })
}

/// The lobby camera for an ape standing at `feet`, looking `look` (flat).
/// The menus turn it to fit their dock (`app::menu_dock`).
pub fn lobby_camera(feet: V3, look: V3) -> Camera {
    lobby_camera_at(feet, look, LOBBY_BACK)
}

/// The lobby camera `back` behind the ape.
pub fn lobby_camera_at(feet: V3, look: V3, back: i32) -> Camera {
    let eye = feet.sub(look.scale(back)).add(v3(0, LOBBY_EYE_UP, 0));
    let target = feet.add(v3(0, LOBBY_AIM_UP, 0));
    Camera {
        eye,
        fwd: target.sub(eye).norm(),
        fov: deg(34),
    }
}

/// The menus' preview on Banana Bay Battle: the ape on the pier, the
/// camera behind it looking out at the fleet.
pub fn preview(c: &Course) -> (V3, i32, Camera) {
    let (feet, look) = match c.bay.as_deref() {
        Some(b) => lobby_spot(b),
        None => (c.starts[0], c.heading_at(0)),
    };
    let camera = lobby_camera(feet, look);
    (feet, yaw_of(camera.eye.sub(feet).flat()), camera)
}

/// Where the flyover runs along the course: from `FLY_FROM` to
/// `FLY_SHORT` before the island's beach.
fn fly_span(c: &Course) -> (i32, i32) {
    let end = c
        .bay
        .as_deref()
        .map_or(c.length - int(60), |b| c.length + int(50) - b.island.radius)
        - FLY_SHORT;
    (FLY_FROM, end.max(FLY_FROM + int(1)))
}

/// The title's flyover on Banana Bay Battle: a glide alongside the fleet,
/// `from` metres along, out on its sunlit side, looking ahead and in.
pub fn flyover_camera(c: &Course, ticks: u32, from: i32) -> Camera {
    let (start, end) = fly_span(c);
    let lap = (end - start) as i64;
    let s = ((ticks as i64 * int(FLY_SPEED) as i64 / TICKS_PER_SECOND as i64 + from as i64) % lap)
        as i32
        + start;
    let eye = fly_line(c, s);
    let ahead = smooth(c, s + FLY_AHEAD).0;
    let look = v3(ahead.x, FLY_LOOK_UP, ahead.z);
    Camera {
        eye,
        fwd: look.sub(eye).norm(),
        fov: deg(70),
    }
}

/// The centreline and heading averaged over 32 m, so the bends are eased.
fn smooth(c: &Course, s: i32) -> (V3, V3) {
    let (mut p, mut h) = (V3::ZERO, V3::ZERO);
    for k in -8..=8 {
        p = p.add(c.point(s + k * int(2)));
        h = h.add(c.heading_at(s + k * int(2)));
    }
    (p.scale(ONE / 17), h.flat().norm())
}

/// The flyover's eye at `s`: out to the left of the line (the pirates'
/// side, lit by the sun), rising and falling a little.
fn fly_line(c: &Course, s: i32) -> V3 {
    let (p, h) = smooth(c, s);
    let q = p.sub(right_of(h).scale(FLY_OUT));
    v3(q.x, FLY_UP + mul(sin(s / 120), int(2)), q.z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bay::bay_seeded;
    use crate::fx;

    /// The space a ship fills (with its masts, yards, sails and bowsprit),
    /// with `room` round it: whether `p` is inside.
    fn inside(s: &Ship, p: V3, room: i32) -> bool {
        let d = p.sub(s.at);
        let along = d.flat().dot(s.fwd());
        let across = d.flat().dot(s.right());
        let span = s
            .masts
            .iter()
            .flat_map(|m| m.yards.iter().map(|y| y.half_span))
            .max()
            .unwrap_or(0)
            .max(s.beam / 2);
        let top = s.deck + s.masts.iter().map(|m| m.height).max().unwrap_or(0);
        along.abs() <= s.length / 2 + s.bowsprit + room
            && across.abs() <= span + room
            && d.y <= top + room
    }

    /// The flyover never goes through a hull, a mast, a yard or a sail
    /// (with 3 m to spare), never near the water, and it glides: no jump
    /// in one frame bigger than its speed allows, but at the loop's end.
    #[test]
    fn the_flyover_glides_alongside_the_fleet_clear_of_every_ship() {
        for seed in [1, 7, 1234] {
            let c = bay_seeded(seed);
            let bay = c.bay.as_deref().unwrap();
            let (start, end) = fly_span(&c);
            let lap =
                ((end - start) as i64 * TICKS_PER_SECOND as i64 / int(FLY_SPEED) as i64) as u32;
            let mut last: Option<Camera> = None;
            let mut seen = 0;
            for t in (0..lap).step_by(4) {
                let cam = flyover_camera(&c, t, 0);
                for (k, s) in bay.ships.iter().enumerate() {
                    assert!(
                        !inside(s, cam.eye, int(3)),
                        "seed {seed} tick {t}: the eye {:?} inside ship {k}",
                        cam.eye
                    );
                    if s.at.sub(cam.eye).flat().len() < int(80) {
                        seen += 1;
                    }
                }
                assert!(
                    cam.eye.y > int(12),
                    "seed {seed} tick {t}: low over the sea"
                );
                if let Some(l) = last {
                    // 4 ticks at 6 m/s is 20 cm; the weave adds a little
                    assert!(
                        cam.eye.sub(l.eye).len() < cm(40),
                        "seed {seed} tick {t}: jumped {} cm",
                        cam.eye.sub(l.eye).len() * 100 / ONE
                    );
                    assert!(
                        cam.fwd.dot(l.fwd) > fx::cos(deg(1)),
                        "seed {seed} tick {t}: turned sharply"
                    );
                }
                last = Some(cam);
            }
            assert!(seen > 0, "seed {seed}: the fleet is alongside");
            // it ends before the island's beach
            let beach = c.length + int(50) - bay.island.radius;
            assert!(end < beach - int(20), "seed {seed}");
        }
    }

    /// The lobby ape stands on the pier's planks, well inside its edges,
    /// clear of the cargo; the landmark is a ship out ahead.
    #[test]
    fn the_lobby_ape_stands_on_the_piers_planks_with_the_fleet_ahead() {
        for seed in [1, 7, 1234] {
            let c = bay_seeded(seed);
            let bay = c.bay.as_deref().unwrap();
            let (feet, look) = lobby_spot(bay);
            let d = &bay.dock;
            assert_eq!(feet.y, d.from.y, "seed {seed}: on the planks' top");
            let fwd = d.to.sub(d.from).flat().norm();
            let along = feet.sub(d.from).flat().dot(fwd);
            let across = feet.sub(d.from).flat().dot(right_of(fwd));
            assert!(
                along > int(1) && along < d.to.sub(d.from).flat().len() - int(1),
                "seed {seed}: {along}"
            );
            assert!(
                across.abs() < d.half_width - int(3),
                "seed {seed}: {across}"
            );
            // the camera's eye is over the pier too, not out over the water
            let eye = lobby_camera(feet, look).eye;
            let e_along = eye.sub(d.from).flat().dot(fwd);
            assert!(e_along > 0, "seed {seed}: the eye behind the pier");
            let mark = landmark(bay);
            let out = mark.sub(feet).flat().dot(look);
            assert!(
                (LANDMARK_NEAR..=LANDMARK_FAR).contains(&out),
                "seed {seed}: the landmark {} m out",
                out / ONE
            );
            assert!(mark.y > bay.water.level, "seed {seed}: on a deck");
        }
    }

    /// The lobby leaves out the start row's seven ropes and no other.
    #[test]
    fn the_lobby_leaves_out_the_start_row_only() {
        for seed in [1, 7, 1234] {
            let c = bay_seeded(seed);
            let bay = c.bay.as_deref().unwrap();
            let left: Vec<usize> = (0..c.vines.len())
                .filter(|&i| start_row(&c, bay, i))
                .collect();
            assert_eq!(left, (0..7).collect::<Vec<_>>(), "seed {seed}");
        }
    }
}
