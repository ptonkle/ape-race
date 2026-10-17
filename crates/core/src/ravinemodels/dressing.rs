//! The canyon's dressing: a rope bridge span and a log across the top (the
//! decks the vines hang from; the skeleton gives them their solid decks),
//! fossils set in the walls, glowing mushrooms, torches and firefly
//! clusters. None of the small pieces has collision.
//!
//! Shape counts: a rope bridge about 4 a metre, a log span about 14, a
//! fossil 10 to 16, mushrooms 11 and a halo, a torch 6 and a halo,
//! fireflies 9 and 3 halos.

use super::*;
use crate::fx::{cm, cos, mul, sin, TURN};

const PLANK: Rgb = rgb(150, 124, 92);
const BARK: Rgb = rgb(92, 70, 52);
const HEARTWOOD: Rgb = rgb(186, 150, 104);
const STEM: Rgb = rgb(220, 226, 210);
const FIREFLY: Rgb = rgb(210, 255, 90);
const RAG: Rgb = rgb(110, 84, 60);

/// The deck's sag at `t` (0..ONE) along a span `len` long: a twentieth of
/// the length at the middle.
pub fn bridge_sag(len: i32, t: i32) -> i32 {
    mul(len / 20, mul(t, ONE - t) * 4)
}

/// A rope bridge from the origin along +z, `len` long and `w` wide, its
/// deck at y 0 at both ends and sagging between (`bridge_sag`), swaying a
/// little with `tick`: planks (one in nine missing), two hand ropes, a
/// rope under each plank end, and verticals every 2 m.
pub fn rope_bridge_span(b: &mut Batch, f: &Affine, len: i32, w: i32, tick: u32, k: u32) {
    let sway = mul(wave(tick, 5, pick(k, 1, 0, TURN)), cm(10));
    let at = |t: i32, x: i32, y: i32| {
        let bulge = mul(t, ONE - t) * 4;
        v3(x + mul(sway, bulge), y - bridge_sag(len, t), mul(len, t))
    };
    let n = (len / (ONE * 6 / 10)).max(2);
    for i in 0..=n {
        let s = k.wrapping_mul(71).wrapping_add(i as u32);
        if i > 0 && i < n && pick(s, 2, 0, 8) == 0 {
            continue;
        }
        let t = ONE * i / n;
        put_turned(
            b,
            f,
            Mesh::Cube,
            at(t, 0, -cm(5)),
            &Affine::rot_y(deg(pick(s, 3, -4, 4))),
            v3(w, cm(8), cm(40)),
            if i % 4 == 0 { TIMBER } else { PLANK },
            0,
        );
    }
    let segs = (len / int(3)).max(2);
    for j in 0..segs {
        let (t0, t1) = (ONE * j / segs, ONE * (j + 1) / segs);
        for sx in [-1, 1] {
            let x = sx * w / 2;
            bar(
                b,
                f,
                Mesh::Hex,
                at(t0, x, cm(110)),
                at(t1, x, cm(110)),
                cm(6),
                ROPE,
                0,
            );
            bar(
                b,
                f,
                Mesh::Hex,
                at(t0, x, -cm(10)),
                at(t1, x, -cm(10)),
                cm(5),
                ROPE,
                0,
            );
        }
    }
    let posts = (len / int(2)).max(1);
    for j in 0..=posts {
        let t = ONE * j / posts;
        for sx in [-1, 1] {
            let x = sx * w / 2;
            let thick = if j == 0 || j == posts { cm(20) } else { cm(4) };
            let c = if thick > cm(10) { TIMBER_DARK } else { ROPE };
            bar(
                b,
                f,
                Mesh::Hex,
                at(t, x, -cm(10)),
                at(t, x, cm(120)),
                thick,
                c,
                0,
            );
        }
    }
}

/// A fallen log across the top from the origin along +z, `len` long and
/// `r` thick (radius), its top at y 0: a slightly bent trunk, the root
/// plate at the start, a broken end, branch stubs, moss on top.
pub fn log_span(b: &mut Batch, f: &Affine, len: i32, r: i32, k: u32) {
    let bend = pick(k, 1, -cm(40), cm(40));
    let p = |t: i32| v3(mul(bend, mul(t, ONE - t) * 4), -r, mul(len, t));
    let pts = [p(0), p(ONE / 3), p(ONE * 2 / 3), p(ONE)];
    for (i, w) in pts.windows(2).enumerate() {
        bar(
            b,
            f,
            Mesh::Taper,
            w[0],
            w[1],
            r * 2 - (r / 6) * i as i32,
            BARK,
            0,
        );
    }
    put_turned(
        b,
        f,
        Mesh::Rock,
        pts[0].sub(v3(0, 0, r / 2)),
        &Affine::rot_x(crate::fx::QUARTER),
        v3(r * 4, r * 4, r),
        BARK,
        0,
    );
    put(
        b,
        f,
        Mesh::Hex,
        pts[3].add(v3(0, 0, cm(4))),
        v3(r * 3 / 2, r * 3 / 2, cm(10)),
        HEARTWOOD,
        0,
    );
    for i in 0..3 {
        let t = ONE * (i * 2 + 1) / 7;
        let sx = if i % 2 == 0 { 1 } else { -1 };
        let base = p(t).add(v3(sx * r * 3 / 4, 0, 0));
        bar(
            b,
            f,
            Mesh::Taper,
            base,
            base.add(v3(sx * r * 2, r, cm(40))),
            r / 2,
            BARK,
            0,
        );
    }
    for i in 0..4 {
        let t = ONE * (i * 2 + 1) / 9;
        put(
            b,
            f,
            Mesh::Blob,
            p(t).add(v3(0, r * 9 / 10, 0)),
            v3(r * 3 / 2, r / 3, len / 7),
            MOSS,
            0,
        );
    }
}

/// Which fossil `fossil_inlay` draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fossil {
    /// a rib cage on a spine, about 4 m across
    Ribs,
    /// a curving spine with spines, about 5 m long
    Spine,
    /// an ammonite's spiral, about 1.5 m across
    Ammonite,
}

/// A fossil set in a wall (the wall's face the frame's z = 0 plane, toward
/// +z), standing a hand proud of it, centred on the origin.
pub fn fossil_inlay(b: &mut Batch, f: &Affine, kind: Fossil, k: u32) {
    let z = cm(8);
    match kind {
        Fossil::Ribs => {
            bar(
                b,
                f,
                Mesh::Hex,
                v3(-int(2), 0, z),
                v3(int(2), cm(20), z),
                cm(26),
                BONE,
                0,
            );
            for i in 0..6 {
                let x = -cm(150) + cm(60) * i;
                let reach = cm(150) - (i - 2).abs() * cm(18);
                let mid = v3(x + cm(20), -reach / 2, z);
                let end = v3(x + cm(50), -reach, z);
                bar(b, f, Mesh::Taper, v3(x, 0, z), mid, cm(14), BONE, 0);
                bar(b, f, Mesh::Taper, mid, end, cm(10), BONE_DARK, 0);
            }
        }
        Fossil::Spine => {
            let tilt = pick(k, 1, -10, 10);
            for i in 0..8 {
                let a = deg(tilt) + i * TURN / 40;
                let at = v3(-cm(240) + cm(62) * i, mul(cm(60), sin(a * 2)), z);
                put(b, f, Mesh::Bevel, at, v3(cm(44), cm(40), cm(20)), BONE, 0);
                put(
                    b,
                    f,
                    Mesh::Spike,
                    at.add(v3(0, cm(40), 0)),
                    v3(cm(14), cm(44), cm(10)),
                    BONE_DARK,
                    0,
                );
            }
        }
        Fossil::Ammonite => {
            let mut prev = V3::ZERO;
            for i in 1..=10 {
                let a = i * TURN * 3 / 20;
                let r = cm(8) * i;
                let p = v3(mul(r, cos(a)), mul(r, sin(a)), z);
                if i > 1 {
                    bar(
                        b,
                        f,
                        Mesh::Hex,
                        prev,
                        p,
                        cm(6) + cm(3) * i,
                        if i % 2 == 0 { BONE } else { BONE_DARK },
                        0,
                    );
                }
                prev = p;
            }
        }
    }
}

/// A cluster of five glowing mushrooms on the ground at the origin, their
/// teal caps pulsing slowly with `tick`, a soft halo over them.
pub fn mushrooms(b: &mut Batch, f: &Affine, tick: u32, k: u32) {
    let glow = ONE / 2 + mul(wave(tick, 4, pick(k, 1, 0, TURN)), ONE / 5);
    for i in 0..5 {
        let s = k.wrapping_mul(17).wrapping_add(i);
        let a = (i as i32) * TURN / 5 + pick(s, 2, 0, TURN / 8);
        let r = if i == 0 { 0 } else { cm(40) };
        let h = pick(s, 3, cm(25), cm(70));
        let cap = pick(s, 4, cm(25), cm(50));
        let base = v3(mul(r, sin(a)), 0, mul(r, cos(a)));
        let lean = v3(pick(s, 5, -cm(8), cm(8)), h, pick(s, 6, -cm(8), cm(8)));
        bar(
            b,
            f,
            Mesh::Cyl,
            base,
            base.add(lean),
            cap / 4,
            STEM,
            ONE / 4,
        );
        let c = if i % 2 == 0 {
            GLOW_TEAL
        } else {
            rgb(110, 170, 255)
        };
        put(
            b,
            f,
            Mesh::Sphere,
            base.add(lean),
            v3(cap, cap / 2, cap),
            c,
            glow,
        );
    }
    halo_ball(b, f, v3(0, cm(40), 0), cm(220), GLOW_TEAL, ONE / 3);
}

/// A torch pushed into the ground at the origin (or, turned, into a wall):
/// a pole, a rag-wrapped head, two flames flickering with `tick`, a halo.
pub fn torch(b: &mut Batch, f: &Affine, tick: u32, k: u32) {
    let top = v3(0, cm(160), 0);
    bar(b, f, Mesh::Hex, V3::ZERO, top, cm(8), TIMBER_DARK, 0);
    put(b, f, Mesh::Ico, top, v3(cm(20), cm(26), cm(20)), RAG, 0);
    for i in 0..2 {
        let h = cm(40) + mul(wave(tick, 1, pick(k, 2 + i as u32, 0, TURN)), cm(10));
        put_turned(
            b,
            f,
            Mesh::Shard,
            top.add(v3(0, cm(10) + h / 2, 0)),
            &Affine::rot_y(i * TURN / 4),
            v3(cm(22) - cm(6) * i, h, cm(10)),
            if i == 0 { FLAME } else { FLAME_CORE },
            ONE * 9 / 10,
        );
    }
    halo_ball(b, f, top.add(v3(0, cm(30), 0)), int(2), FLAME, ONE / 2);
}

/// A cluster of nine fireflies drifting about the point `r` over the
/// origin within `r`, each blinking on its own beat with `tick`, three
/// soft halos among them.
pub fn fireflies(b: &mut Batch, f: &Affine, r: i32, tick: u32, k: u32) {
    let mid = v3(0, r * 2 / 3, 0);
    let mut spots = Vec::with_capacity(9);
    for i in 0..9 {
        let s = k.wrapping_mul(29).wrapping_add(i);
        let home = v3(
            pick(s, 1, -r, r),
            pick(s, 2, -r / 2, r / 2),
            pick(s, 3, -r, r),
        );
        let drift = v3(
            mul(wave(tick, 5, pick(s, 4, 0, TURN)), cm(40)),
            mul(wave(tick, 3, pick(s, 5, 0, TURN)), cm(25)),
            mul(wave(tick, 7, pick(s, 6, 0, TURN)), cm(40)),
        );
        let at = mid.add(home).add(drift);
        let blink = wave(tick, 2, pick(s, 7, 0, TURN)).max(0);
        put(
            b,
            f,
            Mesh::Sphere,
            at,
            v3(cm(9), cm(9), cm(9)),
            FIREFLY,
            ONE / 4 + mul(blink, ONE * 3 / 4),
        );
        spots.push(at);
    }
    for at in spots.iter().step_by(3) {
        halo_ball(b, f, *at, cm(80), FIREFLY, ONE / 3);
    }
}
