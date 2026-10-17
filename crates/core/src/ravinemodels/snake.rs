//! The snake vine (the map's new hazard): a rattlesnake hanging by its tail
//! from a branch or ledge where a vine would hang. Read apart from a vine
//! by shape, not only colour (colourblind mode, JOJO MODE): a coil and a
//! rattle round the anchor, dark and pale bands down the body (a light and
//! dark stripe whatever the hue), a body thicker in the middle, and a J hook
//! at the bottom ending in a wide arrow head with glowing eyes and a
//! flicking forked tongue. A vine is thin, even, straight and leafy.
//!
//! About 40 shapes whole (`snake_vine`), 45 snapped (`snake_snapped`).

use super::*;
use crate::fx::{cm, mul, TURN};

const SCALE: Rgb = rgb(206, 164, 92);
const BAND: Rgb = rgb(54, 36, 26);
const BELLY: Rgb = rgb(232, 210, 160);
const RATTLE: Rgb = rgb(236, 220, 186);
const EYE: Rgb = rgb(255, 220, 40);
const PUPIL: Rgb = rgb(10, 8, 6);
const TONGUE: Rgb = rgb(230, 30, 50);
const FANG: Rgb = rgb(250, 248, 236);
const FLESH: Rgb = rgb(210, 90, 96);
/// The head's size share: big enough to read at a glance from a swing away.
const HEAD: i32 = ONE * 2;
const VINE: Rgb = rgb(66, 120, 50);
const LEAF: Rgb = rgb(80, 150, 60);

/// The body's points for a snake hanging `len` from the frame's origin
/// (its tail; +y up, hanging down), swaying with `tick` (seed `k` picks its
/// phase), ending in the J hook that lifts its head toward +x. The last
/// point is the back of the head; the head looks along the last segment.
pub fn snake_points(len: i32, tick: u32, k: u32) -> Vec<V3> {
    let n = (len / (ONE * 7 / 10)).clamp(6, 14);
    let phase = pick(k, 11, 0, TURN);
    // a slow pendulum sway and a slower twist, the slither running down
    let sway = mul(wave(tick, 7, phase), ONE / 12);
    let twist = mul(wave(tick, 9, phase + TURN / 3), ONE / 16);
    let mut pts = Vec::with_capacity(n as usize + 4);
    for i in 0..=n {
        let t = ONE * i / n;
        let d = mul(len, t);
        let slither = mul(mul(cm(28), t), wave(tick, 3, phase - t * 5 / 4));
        pts.push(v3(mul(d, sway) + slither, -d, mul(d, twist)));
    }
    let p = *pts.last().unwrap_or(&V3::ZERO);
    let c1 = p.add(v3(cm(18), -cm(32), cm(4)));
    let c2 = c1.add(v3(cm(40), -cm(8), cm(8)));
    let c3 = c2.add(v3(cm(44), cm(34), cm(8)));
    pts.extend([c1, c2, c3]);
    pts
}

/// The body's width `i` segments from the tail of `n`: thin at the tail,
/// thickest past the middle, narrowing to the neck.
fn width(i: usize, n: usize) -> i32 {
    let t = ONE * i as i32 / n.max(1) as i32;
    let bulge = mul(t, ONE - t) * 4; // 0..ONE..0
    cm(13) + mul(cm(20), bulge.clamp(ONE / 3, ONE)) - if i + 1 == n { cm(4) } else { 0 }
}

/// The body along `pts` (tail first): banded rods, a belly stripe on the
/// pale bands. `from` counts the segments before `pts[0]`, so the bands
/// stay put across a snap.
fn body(b: &mut Batch, f: &Affine, pts: &[V3], from: usize, n: usize) {
    for (j, w) in pts.windows(2).enumerate() {
        let i = from + j;
        let wd = width(i, n);
        let dark = i % 2 == 1;
        bar(
            b,
            f,
            Mesh::Hex,
            w[0],
            w[1],
            wd,
            if dark { BAND } else { SCALE },
            0,
        );
        if dark {
            // the band stands proud: a ridge round the body
            let mid = w[0].lerp(w[1], ONE / 2);
            let half = w[1].sub(w[0]).scale(ONE / 5);
            bar(
                b,
                f,
                Mesh::Hex,
                mid.sub(half),
                mid.add(half),
                wd + cm(5),
                BAND,
                0,
            );
        }
    }
}

/// The coil round the branch and the rattle above it, at the origin.
fn tail(b: &mut Batch, f: &Affine, tick: u32, k: u32) {
    let r = cm(26);
    let ring: Vec<V3> = (0..=5)
        .map(|i| {
            let a = i * TURN * 5 / 6 / 5 - TURN / 4;
            v3(
                cm(4) * i,
                mul(r, crate::fx::sin(a)),
                mul(r, crate::fx::cos(a)),
            )
        })
        .collect();
    for (i, w) in ring.windows(2).enumerate() {
        bar(
            b,
            f,
            Mesh::Hex,
            w[0],
            w[1],
            cm(10),
            if i % 2 == 0 { SCALE } else { BAND },
            0,
        );
    }
    // the rattle: three beads off the coil's end, shaking a little
    let shake = mul(wave(tick, 1, pick(k, 3, 0, TURN)), cm(3));
    let end = ring[5];
    for i in 0..3 {
        put(
            b,
            f,
            Mesh::Bevel,
            end.add(v3(shake + cm(10) * (i + 1), cm(6) * (i + 1), 0)),
            v3(cm(11) - cm(2) * i, cm(8), cm(12) - cm(2) * i),
            RATTLE,
            0,
        );
    }
}

/// The head at `at`, looking along `dir`: a wide arrow head, a jaw a little
/// open, fangs, glowing eyes with slit pupils, a flicking forked tongue.
fn head(b: &mut Batch, f: &Affine, at: V3, dir: V3, tick: u32, k: u32) {
    let up = if dir.norm().y.abs() > ONE * 9 / 10 {
        v3(0, 0, ONE)
    } else {
        V3::UP
    };
    let g = f
        .then(&Affine::translate(at))
        .then(&Affine::looking(dir, up))
        .then(&Affine::scale(v3(HEAD, HEAD, HEAD)));
    put(
        b,
        &g,
        Mesh::Octa,
        v3(0, cm(4), cm(18)),
        v3(cm(46), cm(20), cm(52)),
        SCALE,
        0,
    );
    put(
        b,
        &g,
        Mesh::Octa,
        v3(0, cm(10), cm(14)),
        v3(cm(30), cm(10), cm(36)),
        BAND,
        0,
    );
    // the jaw, open a crack
    put_turned(
        b,
        &g,
        Mesh::Wedge,
        v3(0, -cm(6), cm(22)),
        &Affine::rot_x(deg(14)),
        v3(cm(28), cm(8), cm(34)),
        BELLY,
        0,
    );
    for sx in [-1, 1] {
        put(
            b,
            &g,
            Mesh::Sphere,
            v3(sx * cm(15), cm(10), cm(24)),
            v3(cm(11), cm(11), cm(11)),
            EYE,
            ONE / 2,
        );
        put(
            b,
            &g,
            Mesh::Cube,
            v3(sx * cm(20), cm(10), cm(25)),
            v3(cm(2), cm(9), cm(3)),
            PUPIL,
            0,
        );
        bar(
            b,
            &g,
            Mesh::Spike,
            v3(sx * cm(6), 0, cm(34)),
            v3(sx * cm(6), -cm(9), cm(36)),
            cm(3),
            FANG,
            0,
        );
    }
    // the tongue: out for a moment, then in
    let flick = wave(tick, 2, pick(k, 5, 0, TURN));
    if flick > 0 {
        let out = cm(18) + mul(flick, cm(22));
        let root = v3(0, -cm(2), cm(42));
        let fork = root.add(v3(0, 0, out));
        bar(b, &g, Mesh::Cube, root, fork, cm(3), TONGUE, ONE / 4);
        for sx in [-1, 1] {
            bar(
                b,
                &g,
                Mesh::Cube,
                fork,
                fork.add(v3(sx * cm(6), -cm(2), cm(9))),
                cm(2),
                TONGUE,
                ONE / 4,
            );
        }
    }
}

/// A snake vine hanging `len` from the frame's origin (its anchor, the
/// branch along x), swaying with `tick`; `k` picks its phase.
pub fn snake_vine(b: &mut Batch, f: &Affine, len: i32, tick: u32, k: u32) {
    let pts = snake_points(len, tick, k);
    snake_along(b, f, &pts, tick, k);
}

/// A snake drawn along any chain of points (tail first, the anchor at
/// `pts[0]`), for a scene that bends it with the vine simulation; the head
/// looks along the last segment.
pub fn snake_along(b: &mut Batch, f: &Affine, pts: &[V3], tick: u32, k: u32) {
    if pts.len() < 2 {
        return;
    }
    let g = f.then(&Affine::translate(pts[0]));
    tail(b, &g, tick, k);
    let n = pts.len() - 1;
    body(b, f, pts, 0, n);
    let (a, z) = (pts[n - 1], pts[n]);
    head(b, f, z, z.sub(a), tick, k);
}

/// The torn end at `at`: raw flesh and two frayed scraps.
fn torn(b: &mut Batch, f: &Affine, at: V3, w: i32) {
    put(b, f, Mesh::Ico, at, v3(w, w * 2 / 3, w), FLESH, 0);
    for sx in [-1, 1] {
        put_turned(
            b,
            f,
            Mesh::Shard,
            at.add(v3(sx * w / 3, -w / 3, 0)),
            &Affine::rot_z(sx * deg(25)),
            v3(w / 2, w, cm(3)),
            SCALE,
            0,
        );
    }
}

/// A snake vine snapped in half (an ape grabbed it): the tail half still
/// hangs from the anchor, short and limp, a raw end at its foot; the head
/// half has fallen `fall` below where it hung, tipping over as it goes
/// (flat on its side by a fall of half its length). The bands stay where
/// they were on the whole snake.
pub fn snake_snapped(b: &mut Batch, f: &Affine, len: i32, fall: i32, tick: u32, k: u32) {
    // the halves hang still: no sway once broken
    let pts = snake_points(len, 0, k);
    let n = pts.len() - 1;
    let m = n / 2;
    let g = f.then(&Affine::translate(pts[0]));
    tail(b, &g, tick, k);
    body(b, f, &pts[..=m], 0, n);
    torn(b, f, pts[m], width(m, n) + cm(4));
    let tilt = mul(
        deg(90),
        crate::fx::div(fall.max(0), (len / 2).max(1)).min(ONE),
    );
    let pivot = pts[m];
    let h = f
        .then(&Affine::translate(pivot.sub(v3(0, fall, 0))))
        .then(&Affine::rot_z(tilt))
        .then(&Affine::translate(pivot.neg()));
    torn(b, &h, pivot.sub(v3(0, cm(6), 0)), width(m, n) + cm(4));
    body(b, &h, &pts[m..], m, n);
    let (a, z) = (pts[n - 1], pts[n]);
    head(b, &h, z, z.sub(a), 0, k);
}

/// A plain hanging vine `len` long (for the gallery's side-by-side check,
/// the map draws its own): thin, even, straight, a few leaves.
pub fn plain_vine(b: &mut Batch, f: &Affine, len: i32) {
    bar(b, f, Mesh::Hex, V3::ZERO, v3(0, -len, 0), cm(9), VINE, 0);
    let mut d = ONE;
    let mut side = 1;
    while d < len - ONE / 2 {
        put_turned(
            b,
            f,
            Mesh::Shard,
            v3(side * cm(14), -d, 0),
            &Affine::rot_z(side * deg(40)),
            v3(cm(16), cm(30), cm(3)),
            LEAF,
            0,
        );
        side = -side;
        d += ONE * 3 / 2;
    }
}
