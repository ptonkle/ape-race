//! The descent's platforms (barnacled wreck planks, coral shelves,
//! bubble discs: what the deep creatures' magic leaves to land on) and
//! the finish, the sea of infinite bananas where the apes dance.

use super::{bar, halo_ball, pick, pulse, put, put_turned, round};
use crate::fx::{cm, deg, div, int, mul, ONE, QUARTER, TURN};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, rgb, Batch, Rgb};

const WRECK: Rgb = rgb(92, 78, 66);
const WRECK_DARK: Rgb = rgb(62, 54, 50);
const RUST: Rgb = rgb(130, 72, 44);
const BARNACLE: Rgb = rgb(196, 192, 170);
const KELP: Rgb = rgb(58, 120, 76);
const CORAL_PLATE: Rgb = rgb(196, 104, 162);
const CORAL_UNDER: Rgb = rgb(120, 60, 120);
const POLYP: [Rgb; 2] = [rgb(255, 150, 90), rgb(90, 240, 200)];
const STAGHORN: Rgb = rgb(170, 110, 220);
const FILM: Rgb = rgb(140, 236, 216);
const BUBBLE: Rgb = rgb(190, 255, 240);
const GOLD_DEEP: Rgb = rgb(214, 156, 30);
const BANANA: Rgb = rgb(250, 214, 56);
const BANANA_RIPE: Rgb = rgb(236, 180, 40);
const FLOOR: Rgb = rgb(246, 206, 92);
const GLINT: Rgb = rgb(255, 250, 200);
/// What the far heaps fade to: the void, dusk-gold.
const FADE: Rgb = rgb(110, 84, 96);

/// A barnacled wreck plank deck, `w` long (x) and `d` deep (z), its top
/// at y 0: five weathered planks of uneven length with splintered ends
/// on two cross beams, iron bands, barnacle crusts and kelp hanging
/// below. Still (no tick). About 36 shapes.
pub fn wreck_plank(b: &mut Batch, f: &Affine, w: i32, d: i32, k: u32) {
    let n = 5;
    for i in 0..n {
        let kk = k * 11 + i as u32;
        let z = -d / 2 + d * (2 * i + 1) / (2 * n);
        let lw = w * pick(kk, 1, 80, 100) / 100;
        let dx = pick(kk, 2, -8, 8) * w / 100;
        put_turned(
            b,
            f,
            Mesh::Cube,
            v3(dx, -cm(15), z),
            &Affine::rot_z(deg(pick(kk, 3, -2, 2))),
            v3(lw, cm(30), d / n - cm(6)),
            if i % 2 == 0 { WRECK } else { WRECK_DARK },
            0,
        );
        // a splintered end
        let side = if i % 2 == 0 { 1 } else { -1 };
        put_turned(
            b,
            f,
            Mesh::Pyramid,
            v3(dx + side * (lw / 2 + cm(25)), -cm(15), z),
            &Affine::rot_z(-side * QUARTER),
            v3(d / n - cm(10), cm(50), cm(28)),
            WRECK,
            0,
        );
    }
    for s in [-1, 1] {
        let x = s * w / 3;
        bar(
            b,
            f,
            Mesh::Cube,
            v3(x, -cm(50), -d / 2),
            v3(x, -cm(50), d / 2),
            cm(36),
            WRECK_DARK,
            0,
        );
        bar(
            b,
            f,
            Mesh::Cube,
            v3(x + cm(40), cm(1), -d / 2),
            v3(x + cm(40), cm(1), d / 2),
            cm(10),
            RUST,
            0,
        );
    }
    // barnacle crusts along the edges and underneath
    for j in 0..12u32 {
        let x = w * pick(k * 13 + j, 4, -45, 45) / 100;
        let z = if j % 2 == 0 { d / 2 } else { -d / 2 } * pick(k * 13 + j, 5, 70, 100) / 100;
        let s = cm(pick(k * 13 + j, 6, 14, 30));
        put(
            b,
            f,
            Mesh::Cone,
            v3(x, -cm(30) - s / 3, z),
            v3(s, s, s),
            BARNACLE,
            0,
        );
    }
    // kelp hanging below in still bends
    for j in 0..4 {
        let mut p = v3(-w / 3 + w * 2 * j / 9, -cm(60), d * (j % 2 * 2 - 1) / 4);
        let mut wd = cm(24);
        for s in 1..=3 {
            let q = p.add(v3(cm(20) * (s % 2 * 2 - 1), -cm(90), cm(15)));
            bar(b, f, Mesh::Taper, p, q, wd, KELP, 0);
            p = q;
            wd = wd * 3 / 4;
        }
    }
}

/// A coral shelf, a table coral `r` across its radius, its top at y 0:
/// a broad violet-pink plate on a stout stalk, glowing polyps round the
/// rim and clumps of staghorn coral low at the back edge. Still. About
/// 30 shapes.
pub fn coral_shelf(b: &mut Batch, f: &Affine, r: i32, k: u32) {
    put(
        b,
        f,
        Mesh::Hex,
        v3(0, -cm(25), 0),
        v3(r * 2, cm(50), r * 2),
        CORAL_PLATE,
        ONE / 10,
    );
    put(
        b,
        f,
        Mesh::Hex,
        v3(0, -cm(70), 0),
        v3(r * 8 / 5, cm(50), r * 8 / 5),
        CORAL_UNDER,
        0,
    );
    bar(
        b,
        f,
        Mesh::Taper,
        v3(0, -r * 3 / 2, 0),
        v3(0, -cm(80), 0),
        r / 2,
        CORAL_UNDER,
        0,
    );
    for j in 0..10 {
        let a = TURN * j / 10 + k as i32 * 3_000;
        let s = cm(pick(k * 10 + j as u32, 7, 24, 40));
        put(
            b,
            f,
            Mesh::Ico,
            round(r, a, -cm(10)),
            v3(s, s, s),
            POLYP[(j % 2) as usize],
            ONE / 3,
        );
    }
    for c in 0..3 {
        let a = deg(150) + deg(30) * c;
        let root = round(r * 4 / 5, a + k as i32 * 3_000, 0);
        for j in 0..3 {
            let tip = root.add(v3(
                cm(40) * (j - 1),
                cm(70) + cm(20) * (j % 2),
                cm(20) * (c - 1),
            ));
            bar(b, f, Mesh::Spike, root, tip, cm(16), STAGHORN, ONE / 6);
        }
    }
}

/// A bubble disc, `r` across its radius, its top at y 0: a glassy
/// sea-green film held by a ring of bubbles that swell in turn, and
/// bubbles rising through it. About 16 shapes and a halo.
pub fn bubble_disc(b: &mut Batch, f: &Affine, r: i32, tick: u32, k: u32) {
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, -cm(15), 0),
        v3(r * 2, cm(30), r * 2),
        FILM,
        ONE / 3,
    );
    for j in 0..10u32 {
        let a = TURN * j as i32 / 10;
        let s = mul(r / 3, pulse(tick, 2, j + k * 10, ONE * 3 / 4));
        put(
            b,
            f,
            Mesh::Sphere,
            round(r, a, 0),
            v3(s, s, s),
            BUBBLE,
            ONE / 2,
        );
    }
    for j in 0..5u32 {
        let life = ((tick + j * 37 + k * 13) % 180) as i32;
        let a = TURN * pick(k * 5 + j, 8, 0, 999) / 1000;
        let p = round(
            r * pick(k * 5 + j, 9, 10, 70) / 100,
            a,
            -int(3) + int(5) * life / 180,
        );
        let s = cm(20) + cm(10) * (j % 3) as i32;
        put(b, f, Mesh::Sphere, p, v3(s, s, s), BUBBLE, ONE / 2);
    }
    halo_ball(b, f, v3(0, -cm(20), 0), r * 5 / 2, FILM, ONE / 4);
}

/// One banana, `len` long, its middle at the origin of `f` lying along x
/// and curving up at both ends: two tapers from its fat middle. 2 shapes.
pub fn banana(b: &mut Batch, f: &Affine, len: i32, c: Rgb) {
    let mid = V3::ZERO;
    for s in [-1, 1] {
        bar(
            b,
            f,
            Mesh::Taper,
            mid,
            v3(s * len / 2, len / 5, 0),
            len / 5,
            c,
            ONE / 2,
        );
    }
}

/// A heap of `n` bananas on a golden mound `r` across its radius, its
/// foot at y 0 and its top about `r` / 2 up, colours fading `fade` of
/// the way into the void. 1 + 2`n` shapes.
pub fn banana_heap(b: &mut Batch, f: &Affine, r: i32, n: u32, fade: i32, k: u32) {
    let far = |c: Rgb| mix(c, FADE, fade);
    put(
        b,
        f,
        Mesh::Blob,
        v3(0, r / 6, 0),
        v3(r * 2, r * 3 / 4, r * 2),
        far(GOLD_DEEP),
        ONE * 2 / 5,
    );
    for j in 0..n {
        let kk = k * 41 + j;
        let a = TURN * pick(kk, 1, 0, 999) / 1000;
        let d = r * pick(kk, 2, 0, 75) / 100;
        let t = div(d, r);
        let y = mul(r / 2, ONE - mul(t, t)) + r / 12;
        let g = f
            .then(&Affine::translate(round(d, a, y)))
            .then(&Affine::rot_y(TURN * pick(kk, 3, 0, 999) / 1000))
            .then(&Affine::rot_x(deg(pick(kk, 4, -25, 25))));
        let c = if j % 3 == 0 { BANANA_RIPE } else { BANANA };
        banana(b, &g, r * pick(kk, 5, 45, 65) / 100, far(c));
    }
}

/// The finish: the sea of infinite bananas at the bottom of the zipper,
/// out to `r` round the origin, where the apes dance. A clear golden
/// dance floor at y 0 ringed by strewn bananas; banana heaps round it,
/// rising into hills and fading into the void farther out; a golden
/// plain to the horizon; bananas raining down from above and glints
/// twinkling. About 360 shapes.
pub fn banana_sea(b: &mut Batch, f: &Affine, r: i32, tick: u32) {
    // the plain to the horizon, and the dance floor
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, -int(2) - cm(30), 0),
        v3(r * 2, int(4), r * 2),
        mix(GOLD_DEEP, FADE, ONE / 4),
        ONE / 3,
    );
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, -cm(20), 0),
        v3(int(26), cm(40), int(26)),
        FLOOR,
        ONE / 2,
    );
    for j in 0..16u32 {
        let a = TURN * j as i32 / 16 + pick(j, 11, -900, 900);
        let g = f
            .then(&Affine::translate(round(
                int(13) + cm(pick(j, 12, -40, 80)),
                a,
                cm(20),
            )))
            .then(&Affine::rot_y(a + pick(j, 13, -9_000, 9_000)));
        banana(
            b,
            &g,
            int(1) + cm(40),
            if j % 2 == 0 { BANANA } else { BANANA_RIPE },
        );
    }
    // rings of heaps: near and full, then fewer bananas, then bare hills
    let rings: [(i32, i32, u32, i32, i32); 3] = [
        (8, 22, 12, int(5), 0),
        (12, 46, 4, int(8), ONE / 4),
        (18, 100, 0, int(16), ONE / 2),
    ];
    for (ri, (count, dist, n, size, fade)) in rings.into_iter().enumerate() {
        for i in 0..count {
            let kk = (ri as i32 * 50 + i) as u32;
            let a = TURN * i / count + pick(kk, 14, -1_500, 1_500);
            let d = mul(int(dist), ONE + pick(kk, 15, -15, 15) * ONE / 100);
            let s = mul(size, ONE + pick(kk, 16, -20, 20) * ONE / 100);
            let g = f.then(&Affine::translate(round(d, a, 0)));
            banana_heap(b, &g, s, n, fade, kk);
        }
    }
    // bananas raining down without end
    for j in 0..10u32 {
        let life = ((tick + j * 47) % 240) as i32;
        let a = TURN * pick(j, 17, 0, 999) / 1000;
        let p = round(int(pick(j, 18, 8, 40)), a, int(60) - int(60) * life / 240);
        let g = f
            .then(&Affine::translate(p))
            .then(&Affine::rot_y(a))
            .then(&Affine::rot_z(
                (tick as i32 + j as i32 * 40).wrapping_mul(TURN / 200),
            ));
        banana(b, &g, int(2), BANANA);
    }
    // glints twinkling over the gold
    for j in 0..12u32 {
        let a = TURN * pick(j, 19, 0, 999) / 1000;
        let p = round(int(pick(j, 20, 6, 50)), a, int(pick(j, 21, 1, 6)));
        let s = mul(cm(50), pulse(tick, 1, j * 3, 0));
        put(b, f, Mesh::Octa, p, v3(s / 3, s, s / 3), GLINT, ONE);
    }
    halo_ball(b, f, v3(0, int(2), 0), int(60), BANANA, ONE / 4);
}
