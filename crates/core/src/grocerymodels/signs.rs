//! The store's signs, all without words: navy aisle numbers, red-and-yellow
//! sale banners with a price and a check mark, and the departments'
//! pictures over the service counters.

use super::{
    digit, part, Face, Fr, BANNER_RED, BANNER_YELLOW, BOARD, INK, NAVY, PANEL, PICTURE, ROD, WHITE,
};
use crate::fx::{cm, deg, int, HALF, QUARTER};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{shade, Batch};

/// The two faces of a sign `depth` thick in frame `fr`, centred `y` up:
/// front and back, each reading the right way round from its side.
fn both_faces(fr: &Fr, y: i32, depth: i32, w: i32, h: i32) -> [Face; 2] {
    let front = Face::front(fr, y, depth / 2, w, h, cm(3));
    let back_fr = fr.sub(V3::ZERO, HALF);
    let back = Face::front(&back_fr, y, depth / 2, w, h, cm(3));
    [front, back]
}

/// The digits of `n` (one or two; a number over 99 shows its last two).
fn digits_of(n: u32) -> Vec<u32> {
    let n = n % 100;
    if n >= 10 {
        vec![n / 10, n % 10]
    } else {
        vec![n]
    }
}

pub(super) fn aisle_sign(b: &mut Batch, at: V3, yaw: i32, number: u32) {
    let fr = Fr::new(at, yaw);
    let (w, h, depth) = (int(12), int(6), cm(50));
    fr.put(b, Mesh::Cube, V3::ZERO, v3(w, h, depth), NAVY, 0);
    // the hanging rods, up 6 m
    for s in [-1, 1] {
        fr.rod(
            b,
            Mesh::Cyl,
            v3(s * part(w, 35, 100), h / 2, 0),
            v3(s * part(w, 35, 100), h / 2 + int(6), 0),
            cm(15),
            ROD,
            0,
        );
    }
    // the category board under it, its lines of (wordless) listing
    let board_h = part(h, 55, 100);
    let board_y = -h / 2 - board_h / 2 - cm(10);
    fr.put(
        b,
        Mesh::Cube,
        v3(0, board_y, 0),
        v3(part(w, 94, 100), board_h, part(depth, 80, 100)),
        BOARD,
        0,
    );
    let ds = digits_of(number);
    let (dw, dh, t) = (cm(260), cm(420), cm(55));
    let pitch = dw + cm(90);
    for face in both_faces(&fr, 0, depth, w, h) {
        let first = -(pitch * (ds.len() as i32 - 1)) / 2;
        for (i, &n) in ds.iter().enumerate() {
            digit(
                b,
                &face,
                (first + pitch * i as i32, 0),
                dw,
                dh,
                t,
                n,
                WHITE,
                0,
            );
        }
    }
    for face in both_faces(&fr, board_y, part(depth, 80, 100), w, board_h) {
        for (i, len) in [70, 52, 62].into_iter().enumerate() {
            face.slab(
                b,
                (
                    -part(w, 40, 100) + part(w, len, 200),
                    part(board_h, 28, 100) - part(board_h, 28, 100) * i as i32,
                ),
                part(w, len, 100),
                cm(32),
                shade(WHITE, 92),
                0,
            );
        }
    }
}

pub(super) fn banner(b: &mut Batch, at: V3, yaw: i32, k: u32) {
    let fr = Fr::new(at, yaw);
    let (w, h, depth) = (int(6), int(8), cm(20));
    fr.put(b, Mesh::Cube, V3::ZERO, v3(w, h, depth), BANNER_RED, 0);
    for s in [-1, 1] {
        fr.rod(
            b,
            Mesh::Cyl,
            v3(s * part(w, 35, 100), h / 2, 0),
            v3(s * part(w, 35, 100), h / 2 + int(4), 0),
            cm(10),
            ROD,
            0,
        );
    }
    let dollars = 1 + k % 9;
    let cents = [99, 49, 79, 29, 88][(k / 9 % 5) as usize];
    for face in both_faces(&fr, 0, depth, w, h) {
        // the yellow price panel
        let (pw, ph) = (part(w, 84, 100), part(h, 46, 100));
        let py = -part(h, 20, 100);
        face.slab(b, (0, py), pw, ph, BANNER_YELLOW, 0);
        // the dollar digit big, the cents small and raised
        let big = part(ph, 78, 100);
        digit(
            b,
            &face,
            (-part(pw, 22, 100), py),
            part(big, 58, 100),
            big,
            part(big, 15, 100),
            dollars,
            INK,
            1,
        );
        let small = part(ph, 40, 100);
        for (i, n) in [cents / 10, cents % 10].into_iter().enumerate() {
            digit(
                b,
                &face,
                (
                    part(pw, 8, 100) + part(pw, 22, 100) * i as i32,
                    py + part(ph, 18, 100),
                ),
                part(small, 60, 100),
                small,
                part(small, 17, 100),
                n,
                INK,
                1,
            );
        }
        // the check mark over it
        let t = part(w, 9, 100);
        let low = (-part(w, 6, 100), part(h, 16, 100));
        face.bar(
            b,
            (-part(w, 24, 100), part(h, 27, 100)),
            low,
            t,
            BANNER_YELLOW,
            0,
        );
        face.bar(
            b,
            low,
            (part(w, 28, 100), part(h, 42, 100)),
            t,
            BANNER_YELLOW,
            0,
        );
    }
}

pub(super) fn dept_sign(b: &mut Batch, at: V3, yaw: i32, dept: u8, size: i32) {
    let fr = Fr::new(at, yaw);
    let (w, h) = (size, part(size, 62, 100));
    let depth = cm(30);
    fr.put(b, Mesh::Cube, V3::ZERO, v3(w, h, depth), PICTURE, 0);
    fr.put(
        b,
        Mesh::Cube,
        V3::ZERO,
        v3(part(w, 94, 100), part(h, 90, 100), depth + cm(4)),
        PANEL,
        0,
    );
    let face = Face::front(&fr, 0, depth / 2 + cm(2), w, h, cm(3));
    let u = size / 10;
    let pu = |n: i32| part(u, n, 10);
    let ink = PICTURE;
    let light = PANEL;
    let flat = |b: &mut Batch, m: Mesh, at: (i32, i32), sx: i32, sy: i32, ang: i32, layer| {
        face.mark(
            b,
            m,
            at,
            sx,
            sy,
            ang,
            if layer > 0 { light } else { ink },
            layer,
        );
    };
    match dept % 6 {
        0 => {
            // a milk bottle and a wedge of cheese
            flat(b, Mesh::Bevel, (-pu(20), -pu(6)), pu(16), pu(24), 0, 0);
            flat(b, Mesh::Cube, (-pu(20), pu(10)), pu(8), pu(9), 0, 0);
            flat(b, Mesh::Cube, (-pu(20), pu(16)), pu(10), pu(3), 0, 0);
            flat(b, Mesh::Cube, (-pu(20), -pu(5)), pu(10), pu(8), 0, 1);
            let wedge = face
                .a
                .then(&Affine::translate(v3(pu(18), -pu(4), cm(2))))
                .then(&Affine::rot_y(QUARTER))
                .then(&Affine::scale(v3(cm(8), pu(20), pu(30))));
            b.push(Mesh::Wedge, &wedge, ink, 0);
            face.dot(b, (pu(12), -pu(8)), pu(5), light, 1);
            face.dot(b, (pu(20), -pu(10)), pu(4), light, 1);
        }
        1 => {
            // a steak and a fish
            flat(b, Mesh::Sphere, (-pu(19), 0), pu(32), pu(24), deg(-12), 0);
            face.dot(b, (-pu(25), pu(3)), pu(6), light, 1);
            flat(b, Mesh::Sphere, (pu(16), 0), pu(28), pu(14), 0, 0);
            flat(b, Mesh::Octa, (pu(33), 0), pu(12), pu(14), 0, 0);
            face.dot(b, (pu(8), pu(2)), pu(3), light, 1);
        }
        2 => {
            // a ham on the bone
            flat(
                b,
                Mesh::Sphere,
                (-pu(4), -pu(2)),
                pu(34),
                pu(26),
                deg(-10),
                0,
            );
            face.bar(b, (pu(10), pu(4)), (pu(26), pu(12)), pu(5), ink, 0);
            face.dot(b, (pu(27), pu(13)), pu(8), ink, 0);
            face.dot(b, (-pu(10), pu(2)), pu(10), light, 1);
        }
        3 => {
            // a loaf of bread, slashed
            flat(b, Mesh::Bevel, (0, -pu(4)), pu(50), pu(20), 0, 0);
            flat(b, Mesh::Sphere, (0, pu(5)), pu(50), pu(18), 0, 0);
            for i in -1..=1 {
                flat(
                    b,
                    Mesh::Cube,
                    (pu(13) * i, pu(6)),
                    pu(10),
                    pu(2),
                    deg(55),
                    1,
                );
            }
        }
        4 => {
            // a carrot and an apple
            flat(
                b,
                Mesh::Cone,
                (-pu(18), -pu(3)),
                pu(13),
                pu(40),
                HALF + deg(22),
                0,
            );
            flat(b, Mesh::Shard, (-pu(24), pu(19)), pu(7), pu(13), deg(20), 0);
            flat(
                b,
                Mesh::Shard,
                (-pu(16), pu(20)),
                pu(7),
                pu(13),
                -deg(10),
                0,
            );
            flat(b, Mesh::Sphere, (pu(18), -pu(4)), pu(28), pu(26), 0, 0);
            flat(b, Mesh::Cube, (pu(18), pu(11)), pu(2), pu(8), deg(10), 0);
            flat(b, Mesh::Shard, (pu(23), pu(13)), pu(8), pu(5), deg(-20), 0);
            face.dot(b, (pu(11), pu(2)), pu(5), light, 1);
        }
        _ => {
            // a snowflake
            for i in 0..3 {
                flat(
                    b,
                    Mesh::Cube,
                    (0, 0),
                    pu(44),
                    pu(4),
                    deg(30) + deg(60) * i,
                    0,
                );
            }
            for i in 0..6 {
                let a = deg(30) + deg(60) * i;
                let (s, c) = (crate::fx::sin(a), crate::fx::cos(a));
                let r = pu(20);
                let at = (crate::fx::mul(c, r), crate::fx::mul(s, r));
                flat(b, Mesh::Octa, at, pu(7), pu(7), 0, 0);
            }
        }
    }
}
