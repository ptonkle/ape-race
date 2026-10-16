//! The portal: the wizard's tear made to turn and breathe like Sector
//! Ape's black hole (`spacecraft::black_hole`: the turning bands, the
//! pull, the sparkles drawn in), simpler than that one. The v1 tear's
//! size class is kept (50 m tall by 56 m wide, `TEAR_H`, `TEAR_W`) so the
//! apes fly through it; its frame is the v1 rift's (the xy plane facing
//! +z, its foot at y 0), so `banana_in_rift` shows the eldritch banana
//! through it unchanged.

use super::{bar, halo_ball, pick, pulse, put, wave};
use crate::fx::{cm, cos, div, int, mul, sin, ONE, TURN};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, rgb, Batch, Rgb};

/// The v1 tear's size (TA8), kept.
pub const TEAR_H: i32 = int(50);
pub const TEAR_W: i32 = int(56);

const EDGE: Rgb = rgb(200, 60, 255);
const HOT: Rgb = rgb(255, 214, 255);
const LIP: Rgb = rgb(26, 8, 44);
const VOID: Rgb = rgb(44, 14, 78);
const VOID_DEEP: Rgb = rgb(26, 8, 52);
const STAR: Rgb = rgb(220, 180, 255);
const SEA: Rgb = rgb(110, 255, 200);
/// The turning bands, outer to inner: torn violet, hot pink-white, the
/// void's sea-green glint, deep violet.
const BANDS: [(Rgb, i32); 4] = [
    (EDGE, ONE),
    (mix_const(HOT, EDGE), ONE),
    (SEA, ONE * 3 / 4),
    (VOID_DEEP, ONE / 2),
];
/// Where each band lies, in the opening's radius, outer to inner.
const BAND_AT: [i32; 4] = [
    crate::fx::ratio(96, 100),
    crate::fx::ratio(84, 100),
    crate::fx::ratio(70, 100),
    crate::fx::ratio(54, 100),
];

const fn mix_const(a: Rgb, b: Rgb) -> Rgb {
    [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2, (a[2] + b[2]) / 2]
}

/// How far in front of the portal its void is drawn to fit (as the v1
/// `RIFT_VIEW`).
pub const PORTAL_VIEW: i32 = int(20);
/// Segments round the torn edge.
const EDGE_SEGS: i32 = 24;

/// A point on the opening's ellipse (`w` across, `h` tall, its middle at
/// `h / 2`) at angle `a` (0 the top, a quarter turn the right), scaled
/// `s` toward its middle.
fn on_ellipse(w: i32, h: i32, a: i32, s: i32) -> V3 {
    v3(
        mul(mul(w / 2, s), sin(a)),
        h / 2 + mul(mul(h / 2, s), cos(a)),
        0,
    )
}

/// How far either side of the portal's middle line (`w` across, `h`
/// tall, its foot at y 0) the opening is clear at height `y`: inside the
/// torn edge's lip and the outer band; 0 outside its height.
pub fn portal_clear(w: i32, h: i32, y: i32) -> i32 {
    if y <= 0 || y >= h || h <= 0 {
        return 0;
    }
    // the ellipse's half width at y
    let t = div(y - h / 2, h / 2);
    let s = ONE - mul(t, t);
    if s <= 0 {
        return 0;
    }
    let hw = mul(w / 2, crate::fx::sqrt(s));
    (mul(hw, BAND_AT[0]) - w / 20 - w / 40).max(0)
}

/// The portal, in the xy plane facing +z, `w` across and `h` tall, its
/// foot at y 0; `open` 0..ONE is how far it has opened (the bands and the
/// opening scale with it). A torn edge of jagged glowing violet with a
/// dark lip inside (the v1 tear's look), and inside that: four thin bands
/// of streaks turning round the opening, the inner ones faster, the whole
/// breathing (its radius swelling and shrinking a few per cent over a few
/// seconds); sparkles drawn in from outside the edge, spiralling to the
/// middle and shrinking as they go (the pull); sparks streaming off the
/// edge; and the dark violet void with faint stars `depth` behind it,
/// scaled to fill the opening from `PORTAL_VIEW` in front, so the banana
/// (`banana_in_rift`, between) shows only through it. About 190 shapes
/// and 13 halos.
pub fn portal(b: &mut Batch, f: &Affine, w: i32, h: i32, open: i32, depth: i32, tick: u32) {
    let t = tick as i32;
    let open = open.clamp(0, ONE);
    if open == 0 {
        return;
    }
    let breathe = ONE + mul(wave(tick, 5, 0), ONE / 25);
    let jump = tick / 6;
    // the torn edge: jags in and out, shivering
    let edge = |i: i32| {
        let a = TURN * i / EDGE_SEGS;
        let jag = if i % 2 == 0 { ONE / 14 } else { -ONE / 20 };
        let shiver = pick(jump * 13 + i as u32, 4, -10, 10) * (ONE / 400);
        on_ellipse(w, h, a, mul(open, ONE + jag + shiver))
    };
    for i in 0..EDGE_SEGS {
        let (p, q) = (edge(i), edge(i + 1));
        bar(
            b,
            f,
            Mesh::Cube,
            p.add(v3(0, 0, -cm(10))),
            q.add(v3(0, 0, -cm(10))),
            cm(110),
            EDGE,
            ONE,
        );
        bar(b, f, Mesh::Cube, p, q, cm(40), HOT, ONE);
        // the dark lip just inside
        let inward = |v: V3| v.lerp(v3(0, h / 2, cm(5)), ONE / 16);
        bar(b, f, Mesh::Cube, inward(p), inward(q), w / 16, LIP, ONE);
        if i % 2 == 0 {
            super::halo_bar(b, f, p, q, int(3), EDGE, ONE * 3 / 4);
        }
    }
    // the turning bands: streaks round the opening, inner ones faster
    for (k, ((c, glow), at)) in BANDS.into_iter().zip(BAND_AT).enumerate() {
        let k = k as i32;
        let n = 20 - k * 2;
        let speed = 90 * (k + 1);
        let s = mul(mul(at, open), breathe);
        let len = mul(w + h, ONE / 4) / n * 3 / 2;
        let thick = w / 50 + w / 200 * (3 - k);
        for j in 0..n {
            let a = TURN * j / n
                + t.wrapping_mul(speed)
                + pick(j as u32 * 7 + k as u32, 5, 0, 1000) * (TURN / n / 2000);
            let p = on_ellipse(w, h, a, s);
            let q = on_ellipse(w, h, a + TURN / n / 2, s);
            let along = q.sub(p).norm();
            let out = v3(-along.y, along.x, 0);
            let z = -cm(20) - cm(20) * k;
            b.push(
                Mesh::Shard,
                &f.then(&Affine {
                    c0: out.scale(thick),
                    c1: along.scale(len),
                    c2: v3(0, 0, cm(20)),
                    t: p.add(v3(0, 0, z)),
                }),
                c,
                glow,
            );
        }
    }
    // the pull: sparkles drawn in from outside the edge, spiralling to the
    // middle and shrinking as they go
    for j in 0..16 {
        let frac = (j * ONE / 16 + t.wrapping_mul(70)) & (ONE - 1);
        let s = mul(open, ONE + ONE / 4 - mul(frac, ONE + ONE / 4));
        let a = TURN * j / 16 + mul(frac, TURN) + t.wrapping_mul(30);
        let p = on_ellipse(w, h, a, s).add(v3(0, 0, cm(30)));
        let size = mul(w / 40, ONE - mul(frac, ONE * 3 / 4));
        put(
            b,
            f,
            Mesh::Octa,
            p,
            v3(size, size * 2, size),
            if j % 3 == 0 { SEA } else { HOT },
            ONE,
        );
    }
    // the void: a sheet of dark violet filling the opening, scaled with
    // depth to fit it from the front, with faint stars and a soft glow
    let k = ONE + div(depth, PORTAL_VIEW);
    let back = |p: V3| v3(mul(p.x, k), mul(p.y, k), -depth);
    let rows = 12;
    for i in 0..rows {
        let y0 = h * i / rows;
        let y1 = h * (i + 1) / rows;
        let hw =
            portal_clear(w, h, (y0 + y1) / 2).max(portal_clear(w, h, y0 + (y1 - y0) / 4)) + w / 30;
        let hw = mul(hw, open);
        let mid = back(v3(0, (y0 + y1) / 2, 0));
        put(
            b,
            f,
            Mesh::Cube,
            mid,
            v3(mul(2 * hw, k), mul(h / rows + cm(20), k), cm(30)),
            if i % 3 == 1 { VOID_DEEP } else { VOID },
            ONE * 2 / 3,
        );
    }
    for j in 0..14u32 {
        let a = TURN * pick(j, 21, 0, 999) / 1000;
        let s = pick(j, 22, 20, 80) * (ONE / 100);
        let p = back(on_ellipse(w, h, a, mul(s, open))).add(v3(0, 0, cm(40)));
        let tw = pulse(tick, 2, j, ONE / 4);
        let size = mul(cm(50) + mul(k, cm(12)), tw);
        put(b, f, Mesh::Octa, p, v3(size, size, size), STAR, ONE);
    }
    halo_ball(
        b,
        f,
        back(v3(0, h / 2, 0)).add(v3(0, 0, cm(60))),
        mul(w, k),
        VOID_DEEP,
        ONE / 3,
    );
    // sparks streaming off the edge
    for j in 0..10u32 {
        let life = ((tick + j * 11) % 60) as i32;
        let i = pick(j, 5, 0, EDGE_SEGS - 1);
        let from = edge(i);
        let out = from
            .sub(v3(0, h / 2, 0))
            .norm()
            .scale(cm(14))
            .add(v3(0, 0, cm(6)));
        put(
            b,
            f,
            Mesh::Shard,
            from.add(out.scale(int(life) / 10)),
            v3(cm(30), cm(60), cm(6)),
            mix(HOT, EDGE, ONE * life / 60),
            ONE,
        );
    }
}
