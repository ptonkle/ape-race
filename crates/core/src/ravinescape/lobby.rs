//! Rattlesnake Ravine's menus (step 9): the lobby camera on a fallen log
//! over the start straight (`RavineLayout.lobby`, chosen in
//! `ravine::lobby`), and the title's flyover down the canyon.
//!
//! **The lobby.** The ape stands mid-log on a fallen log, some 45 m up
//! the slot; the camera stands behind it, low and level (Sunset Ruins' and the
//! Bay's kind), looking on down the canyon: the next bridges and logs, the
//! vines glowing under them, the first arch beyond, the banded walls and
//! the strip of starry sky. `app::menu_dock` turns that camera so the
//! arch's middle ([`landmark`]) sits between the docked menus and the
//! ape. The vines tied to the lobby's own log are left out
//! ([`hidden_in_lobby`]): from the log they hang a few metres in front
//! of the lens.
//!
//! **The flyover** glides down the canyon `FLY_UP` over the floor: the
//! start straight under the bridges, the arch, the cave, round the
//! hairpin past the T-rex skull (the look turns in to it as the eye goes
//! by), the upper passage at the split (under the open sky, the lower
//! passage's tunnel is too low and dark for it), and on toward the mesa.
//! Its line is the centrelines averaged over `FLY_SMOOTH` either side, so
//! the tight corners round off and nothing shakes; it ends short of the
//! finish and starts again.

use crate::course::{Course, STEP};
use crate::fx::{self, deg, int, mul, ONE};
use crate::geom::{facing, v3, yaw_of, V3};
use crate::ravine::{RavineLayout, RoomKind};
use crate::render::Camera;
use crate::rules::TICKS_PER_SECOND;

/// The lobby camera: this far behind the ape, this high over its feet,
/// aimed at this height over its feet (low and nearly level).
pub const LOBBY_BACK: i32 = fx::cm(480);
const LOBBY_EYE_UP: i32 = fx::cm(85);
const LOBBY_AIM_UP: i32 = fx::cm(80);

/// Vines tied this near the lobby's feet are left out of the lobby.
const LOBBY_CLEAR: i32 = int(26);

/// The flyover's eye over the canyon floor (under every deck, over the
/// braids' ribs), its speed (m/s), how far either side the line is
/// averaged, how far ahead it looks and how far over the eye, where it
/// starts and how far short of the finish it ends.
pub const FLY_UP: i32 = int(16);
const FLY_SPEED: i32 = 7;
const FLY_SMOOTH: i32 = int(26);
const FLY_AHEAD: i32 = int(40);
const FLY_LOOK_UP: i32 = int(5);
const FLY_FROM: i32 = int(30);
const FLY_SHORT: i32 = int(60);
/// The look turns in to the skull's mouth within this far of it.
const SKULL_SEEN: i32 = int(90);

/// The lobby spot: the feet and the flat way the camera looks.
pub fn lobby_spot(r: &RavineLayout) -> (V3, V3) {
    let (feet, yaw) = r.lobby;
    (feet, facing(yaw, 0))
}

/// Whether vine `i` is left out of the lobby: tied near its feet.
pub fn hidden_in_lobby(c: &Course, r: &RavineLayout, i: usize) -> bool {
    let (feet, _) = lobby_spot(r);
    c.vines
        .get(i)
        .is_some_and(|v| !v.anchor.sub(feet).flat().longer_than(LOBBY_CLEAR))
}

/// What the lobby frames between the menus and the ape: the first arch's
/// middle, halfway up its span.
pub fn landmark(r: &RavineLayout) -> V3 {
    let (feet, look) = lobby_spot(r);
    match r.rooms.iter().find(|m| m.kind == RoomKind::Arch) {
        Some(a) => a.from.lerp(a.to, ONE / 2).add(v3(0, a.height * 3 / 4, 0)),
        None => feet.add(look.scale(int(100))),
    }
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

/// The menus' preview on Rattlesnake Ravine: the ape on the lobby's
/// log, the camera behind it looking down the canyon.
pub fn preview(c: &Course) -> (V3, i32, Camera) {
    let (feet, look) = match c.ravine.as_deref() {
        Some(r) => lobby_spot(r),
        None => (c.starts[0], c.heading_at(0).flat().norm()),
    };
    let camera = lobby_camera_at(feet, look, LOBBY_BACK);
    (feet, yaw_of(camera.eye.sub(feet).flat()), camera)
}

/// The flyover's route on the floor, a point every `STEP`: the spine to
/// the fork, the upper passage, the spine from the rejoin to short of the
/// finish.
fn route(c: &Course, r: &RavineLayout) -> Vec<V3> {
    let last = (((c.length - FLY_SHORT) / STEP) as usize).min(c.path.len().saturating_sub(1));
    let first = ((FLY_FROM / STEP) as usize).min(last);
    let Some(sp) = r
        .split
        .as_ref()
        .filter(|s| s.fork < s.rejoin && s.rejoin <= last)
    else {
        return c.path[first..=last].to_vec();
    };
    let mut out: Vec<V3> = c.path[first..sp.fork.max(first)].to_vec();
    // the upper passage, a point every STEP along its polyline
    let mut pts = vec![c.path[sp.fork]];
    pts.extend(sp.upper.iter().copied());
    pts.push(c.path[sp.rejoin]);
    for w in pts.windows(2) {
        let len = w[1].sub(w[0]).flat().len();
        let n = (len / STEP).max(1);
        for k in 0..n {
            out.push(w[0].lerp(w[1], fx::div(k, n)));
        }
    }
    out.extend_from_slice(&c.path[sp.rejoin..=last]);
    out
}

/// The route's point `s` metres along, averaged over `FLY_SMOOTH` either
/// side (clamped at its ends), and eased between the route's points so
/// the eye glides rather than steps.
fn smooth_at(route: &[V3], s: i32) -> V3 {
    let at = |mid: i32| {
        let n = route.len() as i32;
        let reach = FLY_SMOOTH / STEP;
        let (mut x, mut y, mut z, mut k) = (0i64, 0i64, 0i64, 0i64);
        for i in mid - reach..=mid + reach {
            let p = route[i.clamp(0, n - 1) as usize];
            x += p.x as i64;
            y += p.y as i64;
            z += p.z as i64;
            k += 1;
        }
        v3((x / k) as i32, (y / k) as i32, (z / k) as i32)
    };
    if route.is_empty() {
        return V3::ZERO;
    }
    let mid = s.div_euclid(STEP);
    at(mid).lerp(at(mid + 1), fx::div(s.rem_euclid(STEP), STEP))
}

/// How far the flyover runs before it starts again (Q16 m).
pub fn fly_length(c: &Course) -> i32 {
    c.ravine
        .as_deref()
        .map_or(int(1), |r| route(c, r).len() as i32 * STEP)
        .max(int(1))
}

/// The title's flyover on Rattlesnake Ravine, `from` metres along its
/// route at `ticks`.
pub fn flyover_camera(c: &Course, ticks: u32, from: i32) -> Camera {
    let Some(r) = c.ravine.as_deref() else {
        let (_, _, cam) = preview(c);
        return cam;
    };
    let route = route(c, r);
    let len = (route.len() as i32 * STEP).max(int(1));
    // (ahead of the end the look would bunch up: the eye stops that far
    // short of it)
    let run = (len - FLY_AHEAD - FLY_SMOOTH).max(int(1));
    let s = ((ticks as i64 * int(FLY_SPEED) as i64 / TICKS_PER_SECOND as i64 + from as i64)
        .rem_euclid(run as i64)) as i32;
    let floor = smooth_at(&route, s);
    let eye = floor.add(v3(0, FLY_UP, 0));
    let ahead = smooth_at(&route, s + FLY_AHEAD).add(v3(0, FLY_UP + FLY_LOOK_UP, 0));
    // past the skull the look turns in to its mouth, smoothly with the
    // distance
    let look = match &r.skull {
        Some(k) => {
            let d = k.at.sub(eye).flat().len();
            let w = fx::div((SKULL_SEEN - d).max(0), SKULL_SEEN);
            let w = mul(mul(w, w), 3 * ONE - 2 * w) * 3 / 5;
            ahead.lerp(k.at, w)
        }
        None => ahead,
    };
    Camera {
        eye,
        fwd: look.sub(eye).norm(),
        fov: deg(70),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Map;
    use crate::ravine::DeckKind;

    #[test]
    fn the_lobby_ape_stands_mid_log_high_in_the_slot_looking_down_the_canyon_at_the_arch() {
        for seed in [1u64, 2, 3] {
            let c = crate::ravine::ravine_seeded(seed);
            let r = c.ravine.as_deref().unwrap();
            let (feet, look) = lobby_spot(r);
            // on a log (its top, in from its ends), high up the
            // slot, not in rock
            let on = r.decks.iter().find(|d| {
                d.kind == DeckKind::Log
                    && d.top_at(feet)
                        .is_some_and(|y| (y - feet.y).abs() < fx::cm(5))
            });
            let d = on.unwrap_or_else(|| panic!("seed {seed}: on no log"));
            let run = d.to.sub(d.from).flat();
            let along = feet.sub(d.from).flat().dot(run.norm());
            assert!(
                along > int(10) && along < run.len() - int(10),
                "seed {seed}: {} m along a {} m log",
                along / ONE,
                run.len() / ONE
            );
            assert!(feet.y > int(30), "seed {seed}: {} m up", feet.y / ONE);
            let solids = c.ravine_solids().unwrap();
            let cam = preview(&c).2;
            assert!(
                !solids.holds(feet.add(v3(0, int(1), 0)), 0),
                "seed {seed}: feet in rock"
            );
            assert!(!solids.holds(cam.eye, int(1)), "seed {seed}: eye in rock");
            // looking on down the canyon, the arch ahead
            let ahead = c.heading_at(int(120)).flat().norm();
            assert!(look.dot(ahead) > ONE * 95 / 100, "seed {seed}: looks back");
            let mark = landmark(r).sub(feet);
            assert!(
                mark.dot(look) > int(80),
                "seed {seed}: the arch {} m ahead",
                mark.dot(look) / ONE
            );
            // few vines are left out
            let n = (0..c.vines.len())
                .filter(|&i| hidden_in_lobby(&c, r, i))
                .count();
            assert!(n <= 8, "seed {seed}: {n} vines left out");
        }
        let c = Map::Ravine.course();
        let r = c.ravine.as_deref().unwrap();
        assert_eq!(crate::scene::preview(&c).feet, lobby_spot(r).0);
    }

    #[test]
    fn the_flyover_glides_down_the_canyon_clear_of_the_rock_past_the_skull_and_the_split() {
        for seed in [1u64, 2, 3] {
            let c = crate::ravine::ravine_seeded(seed);
            let r = c.ravine.as_deref().unwrap();
            let solids = c.ravine_solids().unwrap();
            let tps = TICKS_PER_SECOND as u32;
            let lap = (fly_length(&c) as i64 * tps as i64 / int(FLY_SPEED) as i64) as u32;
            let step = tps / 4;
            let mut last = flyover_camera(&c, 0, 0).eye;
            let (mut at_skull, mut upper) = (false, false);
            let sp = r.split.as_ref().unwrap();
            for tick in (step..lap).step_by(step as usize) {
                let cam = flyover_camera(&c, tick, 0);
                let e = cam.eye;
                let hop = e.sub(last).len();
                last = e;
                // (the wrap back to the start is the one cut)
                if hop > int(FLY_SPEED) {
                    continue;
                }
                assert!(
                    !solids.holds(e, int(2)),
                    "seed {seed} tick {tick}: the eye in rock at {:?}",
                    e
                );
                for (n, d) in r.decks.iter().enumerate() {
                    if let Some(y) = d.top_at(e) {
                        assert!(e.y < y - int(10), "seed {seed} tick {tick}: by deck {n}");
                    }
                }
                if let Some(k) = &r.skull {
                    let to = k.at.sub(e);
                    at_skull |= !to.longer_than(int(60)) && cam.fwd.dot(to.norm()) > ONE * 9 / 10;
                }
                upper |= sp
                    .upper
                    .iter()
                    .any(|p| !p.sub(e).flat().longer_than(int(4)));
            }
            assert!(at_skull, "seed {seed}: never looks at the skull");
            assert!(upper, "seed {seed}: not along the upper passage");
        }
    }
}
