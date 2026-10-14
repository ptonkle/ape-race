//! Skiing (the Ski Slope, 2026-10-13): on the snow an ape glides down the
//! hill, carves with WASD, and a landing keeps its speed along the slope.

use super::*;
use crate::fx::int;
use crate::rules::Kind;

/// An ape standing on the Ski Slope's snow `s` metres along its run (on its
/// line), at rest.
fn on_the_snow(c: &Course, s: i32) -> Ape {
    let p = c.point(int(s));
    let mut a = Ape::new(
        Kind::Chimpanzee,
        true,
        v3(p.x, c.terrain(p.x, p.z) + BODY_DROP, p.z),
        0,
    );
    a.hint = c.nearest_anywhere(a.pos);
    a
}

fn run(a: &mut Ape, c: &Course, ctl: Controls, ticks: u32) {
    for t in 0..ticks {
        a.step(&ctl, c, t);
    }
}

#[test]
fn on_the_snow_an_ape_glides_downhill_and_tops_out_below_a_swing() {
    let c = crate::ski::ski();
    // the first straight runs down the fall line (steep from 100 m)
    let mut a = on_the_snow(&c, 120);
    assert!(a.on_snow(&c));
    let y0 = a.pos.y;
    run(&mut a, &c, Controls::default(), 4 * TICKS_PER_SECOND as u32);
    assert_eq!(a.mode, Mode::Grounded, "glued to the snow");
    assert!(a.speed() > int(3), "it slid off: {} m/s", a.speed() / ONE);
    assert!(a.pos.y < y0 - cm(50), "and down the hill");
    assert_eq!(a.pos.y, c.terrain(a.pos.x, a.pos.z) + BODY_DROP);
    // however steep, skiing tops out well under a good swing (25–35 m/s;
    // under 26 m/s even at `SLOPE_DEG` 10)
    let mut fast = on_the_snow(&c, 40);
    let mut top = 0;
    for t in 0..12 * TICKS_PER_SECOND as u32 {
        let ctl = Controls {
            fwd: 1,
            yaw: crate::geom::yaw_of(c.heading_at(int(150))),
            ..Controls::default()
        };
        fast.step(&ctl, &c, t);
        top = top.max(fast.speed());
    }
    assert!(top < int(26), "skiing reached {} m/s", top / ONE);
}

#[test]
fn the_jungle_floor_does_not_slide() {
    let c = Course::jungle();
    let p = c.point(int(300));
    let mut a = Ape::new(
        Kind::Chimpanzee,
        true,
        v3(p.x, c.terrain(p.x, p.z) + BODY_DROP, p.z),
        0,
    );
    assert!(!a.on_snow(&c));
    run(&mut a, &c, Controls::default(), 240);
    assert_eq!(a.vel, V3::ZERO);
}

#[test]
fn wasd_carve_the_skis_round_and_s_brakes() {
    let c = crate::ski::ski();
    let fwd = c.heading_at(int(60));
    let mut a = on_the_snow(&c, 60);
    a.vel = fwd.scale(int(15));
    let yaw = crate::geom::yaw_of(fwd);
    // D: carve to the right, keeping the speed
    let right = Controls {
        side: 1,
        yaw,
        ..Controls::default()
    };
    run(&mut a, &c, right, 60);
    let turned = a.vel.flat().norm().dot(right_of(fwd));
    assert!(turned > ONE / 4, "turned {}", turned * 100 / ONE);
    assert!(a.speed() > int(13), "kept {} m/s", a.speed() / ONE);
    // S: a snowplough slows it far faster than gliding on
    let ski_on = |fwd: i32| {
        let mut b = on_the_snow(&c, 20);
        b.vel = c.heading_at(int(20)).scale(int(10));
        let ctl = Controls {
            fwd,
            yaw: crate::geom::yaw_of(c.heading_at(int(20))),
            ..Controls::default()
        };
        run(&mut b, &c, ctl, TICKS_PER_SECOND as u32);
        b.speed()
    };
    let (braked, glided) = (ski_on(-1), ski_on(0));
    assert!(
        braked < int(3) && glided > int(8),
        "S {} m/s, none {} m/s",
        braked / ONE,
        glided / ONE
    );
}

#[test]
fn a_landing_on_the_snow_keeps_the_speed_along_the_slope() {
    let c = crate::ski::ski();
    let s = int(150);
    let p = c.point(s);
    let down = c.terrain.downhill(p.x, p.z);
    let fwd = down.norm();
    // falling at 20 m/s along the fall line, steeper than the slope
    let mut a = Ape::new(
        Kind::Chimpanzee,
        true,
        v3(p.x, c.terrain(p.x, p.z) + BODY_DROP + int(3), p.z),
        0,
    );
    a.hint = c.nearest_anywhere(a.pos);
    a.mode = Mode::Flying;
    a.vel = fwd.scale(int(20)).add(v3(0, -int(6), 0));
    let mut landed = None;
    for t in 0..120 {
        let before = a.vel;
        a.step(&Controls::default(), &c, t);
        if a.mode == Mode::Grounded {
            landed = Some((before, a.speed()));
            break;
        }
    }
    let (before, kept) = landed.expect("it lands");
    // the part along the slope of the speed it came down with (about
    // 21 m/s here), not the Jungle's roll (20 + 30% of the fall, then
    // friction down to Run Speed)
    let n = c.terrain.normal(a.pos.x, a.pos.z);
    let want = before.reject(n).len();
    assert!(
        (kept - want).abs() < int(1),
        "kept {} of {}",
        kept / ONE,
        want / ONE
    );
    assert!(kept > int(19));
}
