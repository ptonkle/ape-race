//! The tower map's menus (step 9, second design: TA2 rows): the lobby spot
//! on the lower bridge's deck, tower 2 ahead at the deck's end with its
//! spiral of balconies, mages and dragons, and the title's flyover, a
//! closed lap round both towers.
//!
//! **The lobby.** The ape stands in the middle of the lower bridge's deck
//! (about 50 m up, both towers about 190 m off), the camera behind it
//! looking along the deck toward tower 2: the parapets and lanterns lead
//! the eye to tower 2's wall, the spiral's balconies and beams up it and
//! the dragons round it, in the afternoon haze. `app::menu_dock` turns
//! that camera so tower 2's left edge ([`landmark`]) sits between the
//! docked menus and the ape, as Sunset Ruins does with its temple's
//! mouth. (Both towers at once would need the fog's far end past 400 m:
//! the stand-in's bridges are 380 m long and the fog ends at 330 m.)
//!
//! **The flyover** is a stadium in plan: a half circle round tower 2
//! (`FLY_ROUND` out from its axis), straight runs either side of the
//! bridges, and a half circle at tower 1's end that reaches just over its
//! open top (the eye can't go round tower 1: the portal's plane, where
//! the void begins, is 14 m past its wall). Its height is a cosine along
//! the lap: highest (`FLY_OVER` over tower 1's top) over the hollow,
//! looking down into it and on to the tear, lowest (`FLY_LOW` over the
//! lower deck) round tower 2. The look swings smoothly from the tear to
//! tower 2 and back along the straights. The lap is closed, so it wraps
//! without a jump (`sectorscape::the_title_flyover_through_space_glides…`).

use crate::course::Course;
use crate::fx::{cm, deg, int, mul, ONE, TURN};
use crate::geom::{right_of, v3, yaw_of, V3};
use crate::render::Camera;
use crate::rules::TICKS_PER_SECOND;
use crate::tower::TowerLayout;

/// The lobby camera: this far behind the ape, this high over its feet,
/// aimed at this height over its feet (low and nearly level, as Sunset
/// Ruins' and Banana Bay Battle's).
pub const LOBBY_BACK: i32 = cm(480);
const LOBBY_EYE_UP: i32 = cm(85);
const LOBBY_AIM_UP: i32 = cm(75);
/// The landmark (tower 2's edge) this high over the feet.
const MARK_UP: i32 = int(6);

/// The flyover: its radius out from tower 2's axis (past the spiral's
/// dragons at 64 m and the cloud ring), how far past tower 1's axis its
/// near end reaches (over the hollow, short of the portal's plane), its
/// highest eye over tower 1's top and lowest over the lower deck, and
/// its speed in metres a second.
const FLY_ROUND: i32 = int(101);
const FLY_PAST_1: i32 = int(10);
const FLY_OVER: i32 = int(70);
const FLY_LOW: i32 = int(20);
const FLY_SPEED: i32 = 8;
/// The look aims this far under the eye at a tower's axis, and no higher
/// than this over its top.
const LOOK_UNDER: i32 = int(8);
const LOOK_OVER_TOP: i32 = int(10);

/// Vines anchored this near the lobby's feet are left out of the lobby:
/// they would hang between the camera and the view.
const LOBBY_CLEAR: i32 = int(14);

/// The lobby zone: the feet's place and the flat way the camera looks.
/// The middle of the lower bridge's deck, looking along it toward tower
/// 2.
pub fn lobby_spot(c: &Course, t: &TowerLayout) -> (V3, V3) {
    let br = &t.bridges[0];
    let along = br.to.sub(br.from).flat().norm();
    let Some(p) = br.deck.and_then(|d| c.platforms.get(d)) else {
        let mid = br.from.add(br.to).scale(ONE / 2);
        return (mid, along);
    };
    (v3(p.centre.x, p.top, p.centre.z), along)
}

/// Whether vine `i` is left out of the lobby: anchored near its feet.
pub fn hidden_in_lobby(c: &Course, t: &TowerLayout, i: usize) -> bool {
    let (feet, _) = lobby_spot(c, t);
    c.vines
        .get(i)
        .is_some_and(|v| !v.anchor.sub(feet).flat().longer_than(LOBBY_CLEAR))
}

/// What the lobby frames between the menus and the ape: tower 2's left
/// edge as seen from the ape (where its wall meets the sky, on the left
/// of the look), a little over the feet.
pub fn landmark(c: &Course, t: &TowerLayout) -> V3 {
    let (feet, look) = lobby_spot(c, t);
    let k = t.towers[1].centre;
    let r = t.towers[1].radius;
    let to = feet.sub(v3(k.x, feet.y, k.z));
    let dist = to.len().max(r + 1);
    let out = to.norm();
    // the tangent point from the feet: r²/d along the radius, and
    // r·√(1 − (r/d)²) across it, on the left of the look
    let along = mul(r, crate::fx::div(r, dist));
    let q = crate::fx::div(r, dist);
    let across = mul(r, crate::fx::sqrt(ONE - mul(q, q)));
    let side = right_of(out);
    let side = if side.dot(right_of(look)) < 0 {
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

/// The menus' preview on the tower map: the ape on the lower bridge's
/// deck, the camera behind it looking along the deck to tower 2.
pub fn preview(c: &Course) -> (V3, i32, Camera) {
    let (feet, look) = match c.tower.as_deref() {
        Some(t) => lobby_spot(c, t),
        None => (c.starts[0], c.heading_at(0).flat().norm()),
    };
    let camera = lobby_camera(feet, look);
    (feet, yaw_of(camera.eye.sub(feet).flat()), camera)
}

/// The flyover's stadium: the way from tower 1 to tower 2 (`e`, level,
/// unit) and across it (`n`), the two half circles' centres (`a` at tower
/// 1's end, `b` at tower 2's), the half circle's length and the straights'
/// length, all in metres (Q16).
struct Lap {
    e: V3,
    n: V3,
    a: V3,
    b: V3,
    half: i32,
    straight: i32,
}

impl Lap {
    fn of(t: &TowerLayout) -> Lap {
        let t1 = t.towers[0].centre.flat();
        let t2 = t.towers[1].centre.flat();
        let e = t2.sub(t1).norm();
        let n = right_of(e);
        let a = t1.add(e.scale(FLY_ROUND + FLY_PAST_1));
        let b = t2;
        let half = (FLY_ROUND as i64 * 314_159 / 100_000) as i32;
        let straight = b.sub(a).len();
        Lap {
            e,
            n,
            a,
            b,
            half,
            straight,
        }
    }

    /// The lap's length.
    fn len(&self) -> i32 {
        2 * (self.half + self.straight)
    }

    /// Where `s` metres round the lap is in plan (from the start of
    /// tower 1's half circle, on the right of the way to tower 2, round
    /// tower 1's end, along, round tower 2, and back), and how far round
    /// the lap toward tower 2 it is (0 on tower 1's half circle, `ONE` on
    /// tower 2's, smoothly between along the straights).
    fn at(&self, s: i32) -> (V3, i32) {
        let s = s.rem_euclid(self.len().max(1));
        let (half, straight) = (self.half, self.straight);
        // the angle round a half circle `d` metres into it: a half turn
        // over `half` metres
        let angle = |d: i32| (d as i64 * (TURN / 2) as i64 / half.max(1) as i64) as i32;
        let smooth = |w: i32| {
            let w = w.clamp(0, ONE);
            mul(mul(w, w), 3 * ONE - 2 * w)
        };
        if s < half {
            // round tower 1's end: from the right side (+n) over the
            // far point (−e) to the left side (−n)
            let th = angle(s);
            let p = self
                .a
                .add(self.n.scale(mul(FLY_ROUND, crate::fx::cos(th))))
                .sub(self.e.scale(mul(FLY_ROUND, crate::fx::sin(th))));
            (p, 0)
        } else if s < half + straight {
            let d = s - half;
            let p = self.a.sub(self.n.scale(FLY_ROUND)).add(self.e.scale(d));
            (p, smooth(crate::fx::div(d, straight.max(1))))
        } else if s < 2 * half + straight {
            // round tower 2: from the left side (−n) over the far point
            // (+e) to the right side (+n)
            let th = angle(s - half - straight);
            let p = self
                .b
                .sub(self.n.scale(mul(FLY_ROUND, crate::fx::cos(th))))
                .add(self.e.scale(mul(FLY_ROUND, crate::fx::sin(th))));
            (p, ONE)
        } else {
            let d = s - 2 * half - straight;
            let p = self.b.add(self.n.scale(FLY_ROUND)).sub(self.e.scale(d));
            (p, ONE - smooth(crate::fx::div(d, straight.max(1))))
        }
    }
}

/// The flyover's lap in metres (Q16).
fn fly_len(t: &TowerLayout) -> i32 {
    Lap::of(t).len()
}

/// The flyover's eye `s` metres round the lap: the stadium in plan, the
/// height a cosine along the lap, highest at the middle of tower 1's
/// half circle (over the hollow) and lowest at the middle of tower 2's.
fn fly_eye(t: &TowerLayout, s: i32) -> V3 {
    let lap = Lap::of(t);
    let (p, _) = lap.at(s);
    let high = t.towers[0].top() + FLY_OVER;
    let low = t.bridges[0].from.y.max(t.bridges[0].to.y) + FLY_LOW;
    let (mid, amp) = ((high + low) / 2, (high - low) / 2);
    let s = s.rem_euclid(lap.len().max(1));
    let phase = ((s - lap.half / 2) as i64 * TURN as i64 / lap.len().max(1) as i64) as i32;
    v3(p.x, mid + mul(amp, crate::fx::cos(phase)), p.z)
}

/// The title's flyover on the tower map: `from` metres round the lap at
/// `ticks`, looking at the tear over tower 1's end, at tower 2's axis
/// round tower 2, and smoothly between along the straights.
pub fn flyover_camera(c: &Course, ticks: u32, from: i32) -> Camera {
    let Some(t) = c.tower.as_deref() else {
        return lobby_camera(c.starts[0], c.heading_at(0).flat().norm());
    };
    let lap = Lap::of(t);
    let len = lap.len().max(int(1));
    let s = ((ticks as i64 * int(FLY_SPEED) as i64 / TICKS_PER_SECOND as i64 + from as i64)
        .rem_euclid(len as i64)) as i32;
    let eye = fly_eye(t, s);
    let (_, w) = lap.at(s);
    let t2 = t.towers[1].centre;
    let tower2 = v3(
        t2.x,
        (eye.y - LOOK_UNDER).min(t.towers[1].top() + LOOK_OVER_TOP),
        t2.z,
    );
    let tear = t.portal.tear;
    let look = tear.add(tower2.sub(tear).scale(w));
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
    fn the_lobby_ape_stands_mid_deck_on_the_lower_bridge_looking_at_tower_2() {
        for seed in [1u64, 2, 3] {
            let c = crate::tower::tower_seeded(seed);
            let t = c.tower.as_deref().unwrap();
            let (feet, look) = lobby_spot(&c, t);
            // on the lower bridge's deck, well in from its edges, not in
            // stone, looking along the deck toward tower 2
            let br = &t.bridges[0];
            let p = &c.platforms[br.deck.unwrap()];
            let d = feet.sub(p.centre).flat();
            assert_eq!(feet.y, p.top, "seed {seed}: on the deck's top");
            assert!(
                d.dot(p.fwd).abs() <= p.half_len - int(20)
                    && d.dot(right_of(p.fwd)).abs() <= p.half_width - int(3),
                "seed {seed}: on the deck"
            );
            assert_eq!(
                t.point_in_stone(feet.add(v3(0, int(1), 0))),
                None,
                "seed {seed}: in stone"
            );
            let e = t.towers[1].centre.sub(t.towers[0].centre).flat().norm();
            assert!(
                look.dot(e) > ONE * 99 / 100,
                "seed {seed}: looks at tower 2"
            );
            // tower 2's edge is ahead and on the left of the look
            let mark = landmark(&c, t).sub(feet);
            assert!(mark.dot(look) > 0, "seed {seed}: the landmark ahead");
            assert!(mark.dot(right_of(look)) < 0, "seed {seed}: on the left");
            // few vines are left out
            let hidden = (0..c.vines.len()).filter(|&i| hidden_in_lobby(&c, t, i));
            let n = hidden.count();
            assert!(n < c.vines.len() / 4, "seed {seed}: {n} left out");
        }
        let c = Map::Tower.course();
        assert_eq!(
            crate::scene::preview(&c).feet,
            lobby_spot(&c, c.tower.as_deref().unwrap()).0
        );
    }

    #[test]
    fn the_flyover_laps_both_towers_without_a_jump_clear_of_the_stone() {
        for seed in [1u64, 2, 3] {
            let c = crate::tower::tower_seeded(seed);
            let t = c.tower.as_deref().unwrap();
            let tps = TICKS_PER_SECOND as u32;
            let lap = (fly_len(t) as i64 * tps as i64 / int(FLY_SPEED) as i64) as u32;
            let step = tps / 4;
            // a quarter second's flight (the lap's speed along, and the
            // height's cosine up to three quarters of it), and a little
            let hop = int(FLY_SPEED) * 5 / 16 + cm(30);
            let mut last = flyover_camera(&c, 0, 0).eye;
            let (mut over_hollow, mut at_tear, mut at_tower2) = (false, false, false);
            // (a lap and a half: the wrap is crossed)
            for tick in (step..lap * 3 / 2).step_by(step as usize) {
                let cam = flyover_camera(&c, tick, 0);
                let e = cam.eye;
                assert!(
                    !e.sub(last).longer_than(hop),
                    "seed {seed} tick {tick}: the eye jumps {} m",
                    e.sub(last).len() / ONE
                );
                last = e;
                // never in the void, in stone, in a tower or through a
                // deck
                assert!(!t.in_void(e), "seed {seed} tick {tick}: in the void");
                assert_eq!(
                    t.point_in_stone(e),
                    None,
                    "seed {seed} tick {tick}: in stone"
                );
                for (n, tower) in t.towers.iter().enumerate() {
                    let r = e.sub(tower.centre).flat().len();
                    assert!(
                        r > tower.radius + int(10) || e.y > tower.top() + int(10),
                        "seed {seed} tick {tick}: in tower {n}"
                    );
                }
                for (n, br) in t.bridges.iter().enumerate() {
                    let run = br.to.sub(br.from).flat();
                    let rel = e.sub(br.from).flat();
                    let along = rel.dot(run.norm());
                    let aside = rel.dot(right_of(run.norm())).abs();
                    let near = along > -int(10)
                        && along < run.len() + int(10)
                        && aside < br.half_width + int(10);
                    assert!(
                        !near || (e.y - br.from.y).abs() > int(10),
                        "seed {seed} tick {tick}: through bridge {n}"
                    );
                }
                let t1 = &t.towers[0];
                if !e.sub(t1.centre).flat().longer_than(t1.inner) && e.y > t1.top() {
                    over_hollow = true;
                    let to_tear = t.portal.tear.sub(e).norm();
                    at_tear |= cam.fwd.dot(to_tear) > ONE * 99 / 100;
                }
                let t2 = &t.towers[1];
                let to_t2 = v3(t2.centre.x, e.y, t2.centre.z).sub(e).norm();
                at_tower2 |= e.y < t2.top() && cam.fwd.dot(to_t2) > ONE * 95 / 100;
            }
            assert!(over_hollow, "seed {seed}: over tower 1's open top");
            assert!(at_tear, "seed {seed}: looks down at the tear from there");
            assert!(at_tower2, "seed {seed}: looks at tower 2 from beside it");
        }
    }
}
