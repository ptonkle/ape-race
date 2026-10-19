//! Past the open end: Sector Ape's space (Sam: "you get blasted out into
//! space into something that looks like the sector ape level, feel free to
//! re-use those assets"). Its backdrop (stars, the sun far below, the gas
//! giant, the haze), a banana black hole far ahead on the tube's axis, the
//! mothership passing over, a dogfight and a capital-ship battle off to
//! the sides, and hull chunks and asteroids drifting round the open end,
//! all drawn by Sector Ape's own models (`spacecraft`, `sectorscape`).
//!
//! These are placed in the world, not on the tube, and the world turns
//! against the tube (`spindle::spin_at`): the apes ride with the tube, so
//! the stars and the ships wheel round them. Each shape is set through
//! `Warp::affine_of`, so the frame's bend puts it back in the world.

use crate::course::Course;
use crate::fx::{self, deg, int, ONE};
use crate::geom::{v3, Affine, V3};
use crate::render::{mix, Batch};
use crate::scene::Look;
use crate::spacecraft as sc;
use crate::spindle::{spin_at, SpindleLayout, RADIUS};

/// Space is drawn for an eye this near the open end (inside the tube, it
/// shows only through the opening).
pub const SEEN_FROM: i32 = int(240);
/// The black hole, on the axis this far past the open end, this big.
pub const HOLE_AHEAD: i32 = int(1050);
pub const HOLE_R: i32 = int(55);

/// Sector Ape's look, for an eye out in space.
pub fn look() -> Look {
    crate::sectorscape::look()
}

/// `a` going over to `b` (`t` 0..=ONE).
pub fn blend(a: &Look, b: &Look, t: i32) -> Look {
    let lerp = |x: i32, y: i32| fx::lerp(x, y, t);
    Look {
        sky: mix(a.sky, b.sky, t),
        fog: mix(a.fog, b.fog, t),
        fog_range: (
            lerp(a.fog_range.0, b.fog_range.0),
            lerp(a.fog_range.1, b.fog_range.1),
        ),
        light_dir: v3(
            lerp(a.light_dir.x, b.light_dir.x),
            lerp(a.light_dir.y, b.light_dir.y),
            lerp(a.light_dir.z, b.light_dir.z),
        )
        .norm(),
        light: mix(a.light, b.light, t),
        ground: mix(a.ground, b.ground, t),
        rim: mix(a.rim, b.rim, t),
        far: if t > 0 { b.far } else { a.far },
    }
}

/// The point on the tube's axis `along` past the open end.
fn on_axis(s: &SpindleLayout, along: i32) -> V3 {
    v3(0, RADIUS, s.mouth + along)
}

/// Space round an eye near the open end (`eye` in the flat frame), turned
/// with the tick against the tube.
pub fn dynamic(b: &mut Batch, c: &Course, s: &SpindleLayout, eye: V3, tick: u32) {
    if eye.z < s.mouth - SEEN_FROM {
        return;
    }
    let w = &s.warp;
    let at = w.point(eye);
    let turn = spin_at(tick);
    let mut own = Batch::new();
    // the backdrop round the eye: turning about the eye
    crate::sectorscape::backdrop(&mut own, c, at, tick);
    let about_eye = Affine::translate(at)
        .then(&Affine::rot_z(turn))
        .then(&Affine::translate(at.neg()));
    own.transform(|a| about_eye.then(&a));
    // the set pieces: turning about the axis
    let mut set = Batch::new();
    sc::black_hole(
        &mut set,
        &Affine::translate(on_axis(s, HOLE_AHEAD)).then(&Affine::rot_x(deg(90))),
        HOLE_R,
        tick,
    );
    let pass = (tick % (120 * 60)) as i32 * int(1) / 12;
    sc::mothership(
        &mut set,
        &sc::pose(
            v3(-int(160) + pass, RADIUS + int(170), s.mouth + int(420)),
            deg(90),
            0,
            0,
            ONE,
        ),
        tick,
    );
    sc::dogfight(
        &mut set,
        &sc::pose(
            v3(int(260), RADIUS - int(80), s.mouth + int(520)),
            deg(20),
            0,
            0,
            ONE * 2,
        ),
        tick,
    );
    sc::capital_battle(
        &mut set,
        &sc::pose(
            v3(-int(380), RADIUS + int(60), s.mouth + int(760)),
            deg(-15),
            0,
            0,
            ONE * 2,
        ),
        tick,
    );
    // drifting wreckage round the open end, clear of the way out
    let mut dice = fx::Dice(0x5_D1E5_0BAD);
    for k in 0..18u32 {
        let a = dice.range(0, fx::TURN);
        let r = RADIUS + int(40) + dice.range(0, int(160));
        let along = int(40) + dice.range(0, int(420));
        let p = v3(
            fx::mul(fx::sin(a), r),
            RADIUS - fx::mul(fx::cos(a), r),
            s.mouth + along,
        );
        let f = sc::pose(p, 0, 0, 0, ONE * 2).then(&sc::tumble(tick, k));
        if k % 2 == 0 {
            sc::asteroid(&mut set, &f, k);
        } else {
            sc::hull_chunk(&mut set, &f, tick, k);
        }
    }
    let axis = v3(0, RADIUS, 0);
    let about_axis = Affine::translate(axis)
        .then(&Affine::rot_z(turn))
        .then(&Affine::translate(axis.neg()));
    set.transform(|a| about_axis.then(&a));
    own.append(set);
    // set in the world: placed so the frame's bend puts them back
    own.transform(|a| w.affine_of(&a));
    b.append(own);
}
