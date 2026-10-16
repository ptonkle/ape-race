//! The stone: the main keep in stackable sections, its base and crown,
//! turrets, sub-towers, the bridges that join them (stone arch, rope and
//! plank, conjured tiles) and balconies. All still: no tick, except the
//! conjured bridge's bobbing tiles.

use super::{bar, halo_bar, pick, put, put_turned, round, wave, Spell};
use super::{BANNER, GOLD, MORTAR, ROPE, SLATE, SLATE_DARK, SLIT, STONE, STONE_DARK, STONE_LIGHT};
use super::{STONE_WARM, WINDOW, WOOD, WOOD_DARK};
use crate::fx::{cm, int, mul, ratio, HALF, ONE, TURN};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, rod, shade, Batch, Rgb};

/// How finely a keep section is drawn: `Near` in stone courses with slits
/// and ledges, `Far` (from about 150 m) one panel a face and the lit
/// windows only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detail {
    Near,
    Far,
}

/// A keep section's usual height: the keep is a stack of these.
pub const SECTION_H: i32 = int(12);
/// The wall's thickness.
const WALL: i32 = int(1);
/// tan 22.5°: an octagon's side is `2 · TAN8 · apothem`.
const TAN8: i32 = ratio(4142, 10_000);
/// An octagon's corner over its apothem (1 / cos 22.5°).
const CORNER: i32 = ratio(10_824, 10_000);
/// The keep's crown: its roof walk stands this far over the crown's foot.
pub const CROWN_FLOOR: i32 = cm(140);

/// An octagon's side for apothem `r`.
pub fn side_of(r: i32) -> i32 {
    2 * mul(r, TAN8)
}

/// Stone colour for block `n`.
fn stone(n: u32) -> Rgb {
    [STONE, STONE_LIGHT, STONE_WARM][(super::pick(n, 11, 0, 2)) as usize]
}

/// A slab of `mesh` from `a` to `z`, `wide` across (+x of the rod's frame)
/// and `thick` deep.
pub(crate) fn slab(
    b: &mut Batch,
    f: &Affine,
    mesh: Mesh,
    a: V3,
    z: V3,
    wide: i32,
    thick: i32,
    c: Rgb,
    glow: i32,
) {
    if a.dist(z) > ONE / 100 {
        b.push(
            mesh,
            &f.then(&rod(a, z, ONE))
                .then(&Affine::scale(v3(wide, ONE, thick))),
            c,
            glow,
        );
    }
}

/// One section of the main keep: an octagonal drum `h` tall whose faces
/// stand `r` from the axis (the apothem), from y 0 to `h`, in courses of
/// stone about 3 m high, each course's blocks set in or out a little so
/// the courses read, a ledge round its foot, and on each face (by `k`) an
/// arrow slit, a pair of slits, a lit window or nothing. Face 0 faces +z.
/// About 55 shapes `Near`, 12 `Far`.
pub fn keep_section(b: &mut Batch, f: &Affine, r: i32, h: i32, k: u32, detail: Detail) {
    section(b, f, r, h, k, detail, 0xFF);
}

/// `keep_section` drawing only the faces whose bit is set in `faces`
/// (bit i is face i, at bearing i/8 of a turn): a cutaway.
pub(crate) fn section(
    b: &mut Batch,
    f: &Affine,
    r: i32,
    h: i32,
    k: u32,
    detail: Detail,
    faces: u8,
) {
    let near = detail == Detail::Near;
    let side = side_of(r);
    if faces == 0xFF {
        // the mortar behind the courses
        let d = 2 * (r - cm(25));
        put(b, f, Mesh::Cyl, v3(0, h / 2, 0), v3(d, h, d), MORTAR, 0);
    }
    let courses = if near { (h / cm(240)).max(1) } else { 1 };
    let ch = h / courses;
    for i in 0..8u32 {
        if faces & (1 << i) == 0 {
            continue;
        }
        let g = f.then(&Affine::rot_y(TURN / 8 * i as i32));
        for j in 0..courses {
            let inset = if near && (i as i32 + j) % 2 == 1 {
                cm(8)
            } else {
                0
            };
            let gap = if near { cm(10) } else { 0 };
            put(
                b,
                &g,
                Mesh::Cube,
                v3(0, ch * j + ch / 2, r - WALL / 2 - inset),
                v3(side + cm(4), ch - gap, WALL),
                stone(k.wrapping_mul(97) + i * 13 + j as u32),
                0,
            );
        }
        if near {
            // a pilaster on the corner, hiding the courses' ends
            let a = TURN / 16;
            put_turned(
                b,
                &g,
                Mesh::Cube,
                round(mul(r, CORNER) - cm(25), a, h / 2),
                &Affine::rot_y(a),
                v3(cm(120), h, cm(120)),
                shade(STONE, 88),
                0,
            );
            // the ledge round the section's foot
            put(
                b,
                &g,
                Mesh::Cube,
                v3(0, cm(18), r - WALL / 2 + cm(25)),
                v3(side + cm(36), cm(36), WALL),
                STONE_DARK,
                0,
            );
        }
        // what the face has at mid-height
        let y = h / 2 + ch / 6;
        match pick(k.wrapping_mul(8) + i, 3, 0, 6) {
            0 | 1 if near => slit(b, &g, v3(0, y, r)),
            2 | 3 => window(b, &g, v3(0, y, r), near),
            4 if near => {
                slit(b, &g, v3(-side / 4, y, r));
                slit(b, &g, v3(side / 4, y, r));
            }
            5 if near && h >= int(10) => banner(b, &g, v3(0, h - cm(80), r), mul(h, ONE * 11 / 20)),
            _ => {}
        }
    }
}

/// An arrow slit in a wall face at `at` (on the face, facing +z).
fn slit(b: &mut Batch, g: &Affine, at: V3) {
    put(
        b,
        g,
        Mesh::Cube,
        at.add(v3(0, 0, cm(2))),
        v3(cm(34), cm(240), cm(12)),
        SLIT,
        0,
    );
}

/// A lit window on a wall face at `at`: the warm glass, a stone lintel and
/// a sill (just the glass far off).
fn window(b: &mut Batch, g: &Affine, at: V3, near: bool) {
    put(
        b,
        g,
        Mesh::Cube,
        at.add(v3(0, 0, cm(3))),
        v3(cm(160), cm(300), cm(14)),
        WINDOW,
        ONE * 9 / 10,
    );
    if near {
        // the arch over it, a stone gable
        put_turned(
            b,
            g,
            Mesh::Pyramid,
            at.add(v3(0, cm(185), cm(6))),
            &Affine::IDENTITY,
            v3(cm(210), cm(80), cm(40)),
            STONE_DARK,
            0,
        );
        put(
            b,
            g,
            Mesh::Cube,
            at.add(v3(0, -cm(160), cm(12))),
            v3(cm(200), cm(24), cm(40)),
            STONE_DARK,
            0,
        );
    }
}

/// The school's banner hanging from `top` down a wall face, `len` long:
/// deep violet with a gold border at its foot and a gold star.
fn banner(b: &mut Batch, g: &Affine, top: V3, len: i32) {
    let w = int(2) + cm(40);
    put(
        b,
        g,
        Mesh::Cube,
        top.add(v3(0, -len / 2, cm(10))),
        v3(w, len, cm(6)),
        BANNER,
        ONE / 5,
    );
    put(
        b,
        g,
        Mesh::Cube,
        top.add(v3(0, -len + cm(20), cm(12))),
        v3(w + cm(4), cm(30), cm(8)),
        GOLD,
        ONE / 4,
    );
    put(
        b,
        g,
        Mesh::Octa,
        top.add(v3(0, -len * 2 / 5, cm(16))),
        v3(cm(90), cm(110), cm(10)),
        GOLD,
        ONE / 3,
    );
    put(
        b,
        g,
        Mesh::Hex,
        top.add(v3(0, cm(4), cm(16))),
        v3(w + cm(40), cm(12), cm(12)),
        WOOD_DARK,
        0,
    );
}

/// The keep's foot: a battered plinth `h` tall round an octagon of
/// apothem `r`, with the great door and two torches on face 0.
pub fn keep_base(b: &mut Batch, f: &Affine, r: i32, h: i32, k: u32) {
    let d = int(3);
    for i in 0..8u32 {
        let g = f.then(&Affine::rot_y(TURN / 8 * i as i32));
        put(
            b,
            &g,
            Mesh::Wedge,
            v3(0, h / 2, r + d / 2 - cm(10)),
            v3(side_of(r) + 2 * mul(d, TAN8), h, d),
            if i % 2 == 0 {
                STONE_DARK
            } else {
                shade(STONE_DARK, 92)
            },
            0,
        );
        if i == 0 {
            // the door, set into the batter
            let z = r + d - cm(40);
            put(
                b,
                &g,
                Mesh::Cube,
                v3(0, int(3), z),
                v3(int(4), int(6), cm(30)),
                WOOD_DARK,
                0,
            );
            put(
                b,
                &g,
                Mesh::Pyramid,
                v3(0, int(6) + cm(60), z),
                v3(int(5), cm(140), cm(60)),
                STONE,
                0,
            );
            for s in [-1, 1] {
                put(
                    b,
                    &g,
                    Mesh::Hex,
                    v3(s * int(3), int(4), z + cm(20)),
                    v3(cm(20), cm(80), cm(20)),
                    WOOD,
                    0,
                );
                put(
                    b,
                    &g,
                    Mesh::Spike,
                    v3(s * int(3), int(4) + cm(70), z + cm(20)),
                    v3(cm(36), cm(70), cm(36)),
                    WINDOW,
                    ONE,
                );
            }
        }
    }
    let _ = k;
}

/// The keep's crown, its foot at y 0 on a keep of apothem `r`: a corbelled
/// parapet stepping out 60 cm, merlons three a face, the roof walk at
/// `CROWN_FLOOR`, `turrets` turrets at the octagon's corners with slate
/// cones, and the school's flag on a pole in the middle.
pub fn keep_crown(b: &mut Batch, f: &Affine, r: i32, turrets: u32, k: u32) {
    let out = r + cm(60);
    let side = side_of(out);
    for i in 0..8u32 {
        let g = f.then(&Affine::rot_y(TURN / 8 * i as i32));
        // corbels: flipped ramps, flat on top and against the wall
        for s in [-1, 1] {
            put_turned(
                b,
                &g,
                Mesh::Wedge,
                v3(s * side / 4, cm(60), r + cm(30)),
                &Affine::rot_z(HALF),
                v3(cm(60), cm(120), cm(80)),
                STONE_DARK,
                0,
            );
        }
        // the parapet
        put(
            b,
            &g,
            Mesh::Cube,
            v3(0, int(1) + cm(90), out - cm(40)),
            v3(side + cm(10), cm(140), cm(80)),
            stone(k + i),
            0,
        );
        // merlons
        for m in [-1, 0, 1] {
            put(
                b,
                &g,
                Mesh::Cube,
                v3(m * side / 3, int(3) + cm(20), out - cm(40)),
                v3(side / 5, cm(120), cm(80)),
                stone(k + i * 3 + (m + 1) as u32 + 40),
                0,
            );
        }
    }
    // the roof walk
    let d = 2 * r + cm(40);
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, CROWN_FLOOR - cm(20), 0),
        v3(d, cm(40), d),
        STONE_DARK,
        0,
    );
    // turrets on the corners, corbelled out from below
    for t in 0..turrets {
        let a = TURN / 16 + TURN * t as i32 / turrets.max(1) as i32;
        let at = round(r + cm(40), a, -int(5));
        turret(
            b,
            &f.then(&Affine::translate(at)),
            int(2) + r / 10,
            int(11),
            k + t,
        );
    }
    // the flag
    let top = int(9);
    bar(
        b,
        f,
        Mesh::Hex,
        v3(0, CROWN_FLOOR, 0),
        v3(0, top, 0),
        cm(24),
        WOOD_DARK,
        0,
    );
    put(
        b,
        f,
        Mesh::Cube,
        v3(int(2), top - cm(140), 0),
        v3(int(4), int(2) + cm(40), cm(8)),
        BANNER,
        ONE / 5,
    );
    put(
        b,
        f,
        Mesh::Octa,
        v3(int(2), top - cm(140), 0),
        v3(cm(100), cm(120), cm(14)),
        GOLD,
        ONE / 3,
    );
    put(
        b,
        f,
        Mesh::Ico,
        v3(0, top + cm(20), 0),
        v3(cm(50), cm(50), cm(50)),
        GOLD,
        ONE / 3,
    );
}

/// The whole main keep: its base, `sections` sections of `SECTION_H` and
/// the crown with four turrets; apothem `r`. Its roof walk stands at
/// `keep_height(sections) `. For a very tall keep, place `keep_section`s
/// directly (by chunk, `Detail::Far` far off) instead.
pub fn keep(b: &mut Batch, f: &Affine, r: i32, sections: u32, k: u32) {
    keep_base(b, f, r, int(6), k);
    for s in 0..sections {
        let g = f.then(&Affine::translate(v3(0, SECTION_H * s as i32, 0)));
        keep_section(b, &g, r, SECTION_H, k.wrapping_add(s * 31), Detail::Near);
    }
    let top = f.then(&Affine::translate(v3(0, SECTION_H * sections as i32, 0)));
    keep_crown(b, &top, r, 4, k);
}

/// The height of the roof walk of `keep(_, _, _, sections, _)` over its foot.
pub fn keep_height(sections: u32) -> i32 {
    SECTION_H * sections as i32 + CROWN_FLOOR
}

/// A turret `h` tall of apothem `r` (a hexagonal drum in courses, two
/// slits, a corbelled ring at the top), with a conical slate roof, a gold
/// finial and a pennant. Its foot is at y 0; its tip `turret_tip(r, h)`.
pub fn turret(b: &mut Batch, f: &Affine, r: i32, h: i32, k: u32) {
    // Hex is 1 across the corners; 1.155 across the corners for an
    // apothem of 1/2
    let across = mul(2 * r, ratio(1155, 1000));
    let courses = (h / int(2)).max(1);
    let ch = h / courses;
    put(
        b,
        f,
        Mesh::Hex,
        v3(0, h / 2, 0),
        v3(across - cm(20), h, across - cm(20)),
        MORTAR,
        0,
    );
    for j in 0..courses {
        let w = if j % 2 == 0 { across } else { across - cm(10) };
        put(
            b,
            f,
            Mesh::Hex,
            v3(0, ch * j + ch / 2, 0),
            v3(w, ch - cm(10), w),
            stone(k * 7 + j as u32),
            0,
        );
    }
    for s in 0..2 {
        let g = f.then(&Affine::rot_y(TURN / 6 * (s * 3 + pick(k, 4, 0, 2))));
        // a Hex's faces stand at 30° off its corners
        let g = g.then(&Affine::rot_y(TURN / 12));
        slit(b, &g, v3(0, h / 2 + s * ch, r - cm(2)));
    }
    put(
        b,
        f,
        Mesh::Hex,
        v3(0, h + cm(30), 0),
        v3(across + cm(70), cm(60), across + cm(70)),
        STONE_DARK,
        0,
    );
    cone_roof(
        b,
        f,
        h + cm(60),
        r + cm(70),
        mul(r, ratio(26, 10)) + int(1),
        k,
    );
}

/// A slate cone roof whose eave (radius `r`) is at height `y`, `tall`
/// high, with a gold finial and the school's pennant.
fn cone_roof(b: &mut Batch, f: &Affine, y: i32, r: i32, tall: i32, k: u32) {
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, y + cm(10), 0),
        v3(2 * r + cm(30), cm(20), 2 * r + cm(30)),
        SLATE_DARK,
        0,
    );
    put(
        b,
        f,
        Mesh::Cone,
        v3(0, y + cm(20) + tall / 2, 0),
        v3(2 * r, tall, 2 * r),
        SLATE,
        0,
    );
    // a band of darker slate a third of the way up
    let band = y + cm(20) + tall / 3;
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, band, 0),
        v3(r * 4 / 3 + cm(10), cm(16), r * 4 / 3 + cm(10)),
        SLATE_DARK,
        0,
    );
    let tip = y + cm(20) + tall;
    put(
        b,
        f,
        Mesh::Spike,
        v3(0, tip + cm(40), 0),
        v3(cm(24), cm(110), cm(24)),
        GOLD,
        ONE / 4,
    );
    let flap = pick(k, 9, -20, 20);
    put_turned(
        b,
        f,
        Mesh::Shard,
        v3(cm(90), tip + cm(60), 0),
        &Affine::rot_y(crate::fx::deg(flap)),
        v3(cm(180), cm(50), cm(4)),
        BANNER,
        ONE / 5,
    );
}

/// The tip of `turret(_, _, r, h, _)` over its foot.
pub fn turret_tip(r: i32, h: i32) -> i32 {
    h + cm(80) + mul(r, ratio(26, 10)) + int(1) + cm(95)
}

/// A sub-tower: an octagonal drum of apothem `r`, `h` tall (in sections
/// of about 12 m), a corbelled ring of merlons, and a conical slate roof
/// overhanging it. Its foot is at y 0; its top floor (under the roof, at
/// the merlons' foot) at `h`.
pub fn sub_tower(b: &mut Batch, f: &Affine, r: i32, h: i32, k: u32) {
    let n = (h / SECTION_H).max(1);
    let sh = h / n;
    for s in 0..n {
        let g = f.then(&Affine::translate(v3(0, sh * s, 0)));
        keep_section(b, &g, r, sh, k.wrapping_mul(5) + s as u32, Detail::Near);
    }
    let top = f.then(&Affine::translate(v3(0, h, 0)));
    let out = r + cm(40);
    let side = side_of(out);
    for i in 0..8u32 {
        let g = top.then(&Affine::rot_y(TURN / 8 * i as i32));
        put(
            b,
            &g,
            Mesh::Cube,
            v3(0, cm(50), out - cm(40)),
            v3(side + cm(10), int(1), cm(80)),
            STONE_DARK,
            0,
        );
        for m in [-1, 1] {
            put(
                b,
                &g,
                Mesh::Cube,
                v3(m * side / 4, int(1) + cm(50), out - cm(40)),
                v3(side / 4, int(1), cm(70)),
                stone(k.wrapping_add(i).wrapping_add(m as u32)),
                0,
            );
        }
    }
    // the roof on four short posts, a lit room under it
    let d = 2 * r - cm(60);
    put(
        b,
        &top,
        Mesh::Cyl,
        v3(0, int(2) + cm(10), 0),
        v3(d, int(2), d),
        WINDOW,
        ONE * 3 / 5,
    );
    cone_roof(
        b,
        &top,
        int(3),
        r + int(1),
        mul(r, ratio(22, 10)) + int(2),
        k,
    );
}

/// A stone arch bridge from `a` to `z` (the deck's top, in the middle of
/// its width, at each end), `w` wide: the deck, parapets with coping, an
/// arch under it with solid spandrels, and two lanterns. A branch from
/// the keep to a sub-tower. About 25 shapes.
pub fn arch_bridge(b: &mut Batch, f: &Affine, a: V3, z: V3, w: i32, k: u32) {
    let d = z.sub(a);
    let len = d.len();
    let g = f
        .then(&Affine::translate(a))
        .then(&Affine::looking(d, V3::UP));
    put(
        b,
        &g,
        Mesh::Bevel,
        v3(0, -cm(30), len / 2),
        v3(w, cm(60), len),
        STONE_DARK,
        0,
    );
    for s in [-1, 1] {
        put(
            b,
            &g,
            Mesh::Cube,
            v3(s * (w / 2 - cm(20)), cm(50), len / 2),
            v3(cm(40), int(1), len),
            STONE,
            0,
        );
        put(
            b,
            &g,
            Mesh::Cube,
            v3(s * (w / 2 - cm(20)), int(1) + cm(7), len / 2),
            v3(cm(56), cm(16), len + cm(10)),
            STONE_LIGHT,
            0,
        );
    }
    // the arch: springing `rise` under the deck at both ends
    let rise = (len / 3).max(int(3));
    let n = 8;
    let arch = |i: i32| {
        let t = TURN / 2 * i / n;
        v3(
            0,
            -rise + mul(rise - int(1) - cm(20), crate::fx::sin(t)),
            len * i / n,
        )
    };
    for i in 0..n {
        let (p, q) = (arch(i), arch(i + 1));
        slab(
            b,
            &g,
            Mesh::Cube,
            p,
            q,
            w + cm(20),
            cm(80),
            if i % 2 == 0 { STONE_DARK } else { STONE },
            0,
        );
        // the spandrel over it, up to the deck
        let top = p.y.max(q.y) + cm(30);
        let h = -cm(60) - top;
        if h > cm(10) {
            put(
                b,
                &g,
                Mesh::Cube,
                v3(0, top + h / 2, (p.z + q.z) / 2),
                v3(w - cm(20), h, len / n + cm(2)),
                stone(k + i as u32),
                0,
            );
        }
    }
    for (i, t) in [(0, 1), (1, 2)] {
        let s = if (k + i).is_multiple_of(2) { 1 } else { -1 };
        let at = v3(s * (w / 2 - cm(20)), int(1), len * t / 3);
        bar(
            b,
            &g,
            Mesh::Hex,
            at,
            at.add(v3(0, int(1) + cm(20), 0)),
            cm(16),
            WOOD_DARK,
            0,
        );
        put(
            b,
            &g,
            Mesh::Ico,
            at.add(v3(0, int(1) + cm(40), 0)),
            v3(cm(40), cm(50), cm(40)),
            WINDOW,
            ONE,
        );
    }
}

/// A rope-and-plank bridge from `a` to `z` (the deck's middle at each
/// end), `w` wide, sagging `sag` at its middle: planks (a few missing),
/// rope hand rails and hangers, and posts at both ends. About 70 shapes
/// for 24 m.
pub fn rope_bridge(b: &mut Batch, f: &Affine, a: V3, z: V3, w: i32, sag: i32, k: u32) {
    let d = z.sub(a);
    let len = d.len();
    let g = f
        .then(&Affine::translate(a))
        .then(&Affine::looking(d, V3::UP));
    let y = |t: i32| -4 * mul(sag, mul(t, ONE - t));
    let planks = (len / cm(80)).max(2);
    for i in 0..planks {
        if pick(k * 131 + i as u32, 6, 0, 13) == 0 && i > 1 && i < planks - 2 {
            continue;
        }
        let t = (ONE * 2 * i + ONE) / (2 * planks);
        let c = if i % 3 == 0 { WOOD_DARK } else { WOOD };
        put(
            b,
            &g,
            Mesh::Cube,
            v3(0, y(t) - cm(5), mul(len, t)),
            v3(w, cm(8), cm(60)),
            c,
            0,
        );
    }
    let segs = 8;
    for s in [-1, 1] {
        let x = s * (w / 2);
        let rail = |i: i32| {
            let t = ONE * i / segs;
            v3(x, y(t) + int(1) + cm(10), mul(len, t))
        };
        for i in 0..segs {
            bar(b, &g, Mesh::Cube, rail(i), rail(i + 1), cm(6), ROPE, 0);
        }
        for i in 1..segs {
            let p = rail(i);
            bar(
                b,
                &g,
                Mesh::Cube,
                p,
                v3(x, p.y - int(1) - cm(10), p.z),
                cm(4),
                ROPE,
                0,
            );
        }
        for end in [0, len] {
            let at = v3(x, 0, end);
            bar(
                b,
                &g,
                Mesh::Hex,
                at.add(v3(0, -cm(30), 0)),
                at.add(v3(0, int(1) + cm(40), 0)),
                cm(22),
                WOOD_DARK,
                0,
            );
        }
    }
}

/// A conjured bridge from `a` to `z`, `w` wide: hexagonal tiles of
/// `spell`'s light, a little apart, bobbing, over a soft glow. About two
/// shapes a 2 m tile.
pub fn magic_bridge(b: &mut Batch, f: &Affine, a: V3, z: V3, w: i32, spell: Spell, tick: u32) {
    let d = z.sub(a);
    let len = d.len();
    let g = f
        .then(&Affine::translate(a))
        .then(&Affine::looking(d, V3::UP));
    let n = (len / int(2)).max(1);
    let c = spell.colour();
    let pale = mix(c, [ONE; 3], ONE / 2);
    for i in 0..n {
        let bob = mul(wave(tick, 3, i * 4_000), cm(12));
        let at = v3(0, bob - cm(12), len * (2 * i + 1) / (2 * n));
        put(
            b,
            &g,
            Mesh::Hex,
            at,
            v3(w + cm(20), cm(24), int(2) - cm(10)),
            pale,
            ONE * 3 / 5,
        );
        put(
            b,
            &g,
            Mesh::Hex,
            at.add(v3(0, cm(14), 0)),
            v3(w / 2, cm(6), w / 2),
            c,
            ONE,
        );
    }
    halo_bar(
        b,
        &g,
        v3(0, -cm(20), 0),
        v3(0, -cm(20), len),
        w + int(1),
        c,
        ONE / 3,
    );
}

/// A balcony on a wall: the origin at the wall's face, floor level, +z out
/// from the wall; `w` wide and `depth` deep. A stone floor on three
/// corbels, a balustrade, and a lit doorway behind. About 22 shapes; the
/// mage stands about 1 m out (`v3(0, 0, depth / 2)`).
pub fn balcony(b: &mut Batch, f: &Affine, w: i32, depth: i32, k: u32) {
    put(
        b,
        f,
        Mesh::Bevel,
        v3(0, -cm(20), depth / 2),
        v3(w, cm(40), depth),
        STONE_DARK,
        0,
    );
    for x in [-w / 3, 0, w / 3] {
        put_turned(
            b,
            f,
            Mesh::Wedge,
            v3(x, -cm(110), depth * 2 / 5),
            &Affine::rot_z(HALF),
            v3(cm(50), cm(140), mul(depth, ONE * 4 / 5)),
            STONE,
            0,
        );
    }
    let rail = int(1) + cm(5);
    let front = depth - cm(12);
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, rail, front),
        v3(w, cm(16), cm(22)),
        STONE_LIGHT,
        0,
    );
    for s in [-1, 1] {
        put(
            b,
            f,
            Mesh::Cube,
            v3(s * (w / 2 - cm(11)), rail, depth / 2),
            v3(cm(22), cm(16), depth),
            STONE_LIGHT,
            0,
        );
        put(
            b,
            f,
            Mesh::Hex,
            v3(s * (w / 2 - cm(11)), cm(50), depth / 2),
            v3(cm(16), int(1), cm(16)),
            STONE,
            0,
        );
    }
    let n = (w / cm(60)).clamp(2, 9);
    for i in 0..n {
        let x = -w / 2 + cm(30) + (w - cm(60)) * i / (n - 1).max(1);
        put(
            b,
            f,
            Mesh::Hex,
            v3(x, cm(50), front),
            v3(cm(16), int(1), cm(16)),
            STONE,
            0,
        );
    }
    // the doorway
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, int(1) + cm(30), cm(4)),
        v3(cm(150), int(2) + cm(60), cm(10)),
        shade(WINDOW, 70),
        ONE * 3 / 5,
    );
    put(
        b,
        f,
        Mesh::Pyramid,
        v3(0, int(2) + cm(90), cm(8)),
        v3(cm(190), cm(70), cm(24)),
        STONE_DARK,
        0,
    );
    let _ = k;
}
