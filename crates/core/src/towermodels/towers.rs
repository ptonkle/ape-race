//! The second design's stone (`briefs/mapes-tower-v2.md`, `notes/
//! tower2-plan.md`): tower 1, a hollow drum twice the old keep's height
//! (the v1 keep: 137 m tall, 41 m apothem) with a doorway half way up,
//! windows that let the sky in and ledges and balconies round its inside;
//! tower 2 (60% of tower 1, the keep's own look) with balconies spiralling
//! up its outside and clouds round its top; and the two long straight
//! bridges between them, with vine hangers under their decks. All still
//! (no tick), in the v1 palette. Every builder takes its geometry as plain
//! arguments (an apothem, a height, a bearing) so the layout can place it.

use super::keep::{side_of, slab, Detail, SECTION_H};
use super::{balcony, cloud, keep_crown, keep_section, pick, put, put_turned, round, turret};
use super::{GOLD, MORTAR, STONE, STONE_DARK, STONE_LIGHT, STONE_WARM, WINDOW, WOOD_DARK};
use crate::fx::{cm, int, mul, HALF, ONE, QUARTER, TURN};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{shade, Batch, Rgb};

/// tan 22.5°: an octagon's side is `2 · TAN8 · apothem`.
const TAN8: i32 = crate::fx::ratio(4142, 10_000);
/// An octagon's corner over its apothem (1 / cos 22.5°).
const CORNER: i32 = crate::fx::ratio(10_824, 10_000);

/// The v1 keep, for the sizes stated relative to it: 137 m tall, 41 m
/// apothem.
pub const V1_KEEP_HEIGHT: i32 = int(137);
pub const V1_KEEP_RADIUS: i32 = int(41);

/// Tower 1: twice the keep's height (274 m), an outer apothem 1.25× the
/// keep's (51 m) and an inner one leaving the inside about 90 m across.
pub const TOWER1_HEIGHT: i32 = V1_KEEP_HEIGHT * 2;
pub const TOWER1_OUT: i32 = V1_KEEP_RADIUS * 5 / 4;
pub const TOWER1_IN: i32 = int(45);
/// Tower 2: 60% of tower 1 (164 m, about 1.2× the keep), the keep's apothem.
pub const TOWER2_HEIGHT: i32 = TOWER1_HEIGHT * 3 / 5;
pub const TOWER2_RADIUS: i32 = V1_KEEP_RADIUS;
/// A bridge's length (about 10 s of swinging) and the hangers' spacing.
pub const BRIDGE_LENGTH: i32 = int(250);
pub const HANGER_SPACING: i32 = int(22);
/// A long bridge's deck width.
pub const BRIDGE_WIDTH: i32 = int(8);

/// A section's inner ledge (`Ledge::Walk`): how far it stands in from the
/// inner wall, and how thick it is.
pub const LEDGE_DEPTH: i32 = int(3);
const LEDGE_THICK: i32 = cm(80);

/// Stone colour for block `n`.
fn stone(n: u32) -> Rgb {
    [STONE, STONE_LIGHT, STONE_WARM][(pick(n, 11, 0, 2)) as usize]
}

/// What a face of a hollow section has cut through its wall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cut {
    /// solid stone with the keep's slits, lit windows and banners by `k`
    Wall,
    /// an arched opening through the wall, the sky seen through it
    Window,
    /// the doorway: an opening the section's height, wide enough to swing
    /// through, with a gable and torches outside
    Door,
}

/// What runs round a hollow section's inside at its foot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ledge {
    /// nothing
    None,
    /// a walkable ledge `LEDGE_DEPTH` deep all round, on corbels
    Walk,
}

/// A window opening's width and height, and the doorway's width.
pub const WINDOW_W: i32 = int(9);
pub const WINDOW_H: i32 = int(8);
pub const DOOR_W: i32 = int(16);

/// One section of a hollow tower, `h` tall from y 0: an octagonal drum
/// whose outer faces stand `r_out` from the axis and inner faces `r_in`,
/// the wall between them mortar. Outside it is the keep's look (courses
/// of stone, a pilaster on each corner, a ledge round the foot, slits and
/// lit windows by `k`); inside, courses 6 m high and a ledge at the foot
/// when `ledge` says so. `cuts[i]` is what face i (at bearing i/8 of a
/// turn, face 0 facing +z) has cut through it; an opening is a hole in
/// both skins with stone reveals through the wall. `Detail::Far` draws a
/// panel a face and the openings only. About 90 to 140 shapes `Near` (by its cuts and ledge), 30 `Far`.
#[allow(clippy::too_many_arguments)]
pub fn hollow_section(
    b: &mut Batch,
    f: &Affine,
    r_out: i32,
    r_in: i32,
    h: i32,
    k: u32,
    cuts: [Cut; 8],
    ledge: Ledge,
    detail: Detail,
) {
    let near = detail == Detail::Near;
    let wall = r_out - r_in;
    let side_out = side_of(r_out);
    let side_in = side_of(r_in);
    let courses = if near { (h / cm(300)).max(1) } else { 1 };
    let ch = h / courses;
    let in_courses = if near { (h / int(6)).max(1) } else { 1 };
    let ich = h / in_courses;
    for i in 0..8u32 {
        let g = f.then(&Affine::rot_y(TURN / 8 * i as i32));
        let cut = cuts[i as usize];
        // the opening in this face: (width, bottom, top)
        let hole = match cut {
            Cut::Wall => None,
            Cut::Window => Some((WINDOW_W, h / 2 - WINDOW_H / 2, h / 2 + WINDOW_H / 2)),
            Cut::Door => Some((DOOR_W, 0, h - int(2))),
        };
        // the mortar through the wall's thickness
        match hole {
            None => put(
                b,
                &g,
                Mesh::Cube,
                v3(0, h / 2, (r_out + r_in) / 2),
                v3(side_in, h, wall),
                MORTAR,
                0,
            ),
            Some((hw, lo, hi)) => {
                // mortar either side of the hole, and over and under it
                for s in [-1, 1] {
                    let x0 = s * hw / 2;
                    let x1 = s * side_in / 2;
                    put(
                        b,
                        &g,
                        Mesh::Cube,
                        v3((x0 + x1) / 2, h / 2, (r_out + r_in) / 2),
                        v3((x1 - x0).abs(), h, wall),
                        MORTAR,
                        0,
                    );
                }
                if lo > 0 {
                    put(
                        b,
                        &g,
                        Mesh::Cube,
                        v3(0, lo / 2, (r_out + r_in) / 2),
                        v3(hw, lo, wall),
                        MORTAR,
                        0,
                    );
                }
                put(
                    b,
                    &g,
                    Mesh::Cube,
                    v3(0, (hi + h) / 2, (r_out + r_in) / 2),
                    v3(hw, h - hi, wall),
                    MORTAR,
                    0,
                );
                // the reveals: dressed stone round the hole through the wall
                for s in [-1, 1] {
                    put(
                        b,
                        &g,
                        Mesh::Cube,
                        v3(s * (hw / 2 + cm(30)), (lo + hi) / 2, (r_out + r_in) / 2),
                        v3(cm(60), hi - lo, wall + cm(20)),
                        STONE_DARK,
                        0,
                    );
                }
                put(
                    b,
                    &g,
                    Mesh::Cube,
                    v3(0, hi + cm(30), (r_out + r_in) / 2),
                    v3(hw + cm(120), cm(60), wall + cm(20)),
                    STONE_DARK,
                    0,
                );
                if lo > 0 {
                    put(
                        b,
                        &g,
                        Mesh::Cube,
                        v3(0, lo - cm(30), (r_out + r_in) / 2),
                        v3(hw + cm(120), cm(60), wall + cm(20)),
                        STONE_DARK,
                        0,
                    );
                }
            }
        }
        // the outer skin, in courses, each course split round the hole
        for j in 0..courses {
            let y0 = ch * j;
            let y1 = ch * (j + 1);
            let inset = if near && (i as i32 + j) % 2 == 1 {
                cm(8)
            } else {
                0
            };
            let gap = if near { cm(10) } else { 0 };
            let z = r_out - cm(50) - inset;
            let c = stone(k.wrapping_mul(97) + i * 13 + j as u32);
            skin_course(b, &g, hole, side_out + cm(4), y0, y1 - gap, z, c);
        }
        // the inner skin
        for j in 0..in_courses {
            let y0 = ich * j;
            let y1 = ich * (j + 1);
            let inset = if near && (i as i32 + j) % 2 == 0 {
                cm(10)
            } else {
                0
            };
            let gap = if near { cm(12) } else { 0 };
            let z = r_in + cm(50) + inset;
            let c = shade(stone(k.wrapping_mul(53) + i * 7 + j as u32), 92);
            skin_course(b, &g, hole, side_in - cm(4), y0, y1 - gap, z, c);
        }
        if near {
            // a pilaster on the outer corner, hiding the courses' ends
            let a = TURN / 16;
            put_turned(
                b,
                &g,
                Mesh::Cube,
                round(mul(r_out, CORNER) - cm(25), a, h / 2),
                &Affine::rot_y(a),
                v3(cm(140), h, cm(140)),
                shade(STONE, 88),
                0,
            );
            // the ledge round the section's foot, outside
            put(
                b,
                &g,
                Mesh::Cube,
                v3(0, cm(18), r_out - cm(25)),
                v3(side_out + cm(36), cm(36), int(1)),
                STONE_DARK,
                0,
            );
            // what a solid face has at mid-height
            if cut == Cut::Wall {
                let y = h / 2 + ch / 6;
                match pick(k.wrapping_mul(8) + i, 3, 0, 5) {
                    0 | 1 => slit(b, &g, v3(0, y, r_out)),
                    2 | 3 => lit_window(b, &g, v3(0, y, r_out)),
                    _ => {
                        slit(b, &g, v3(-side_out / 4, y, r_out));
                        slit(b, &g, v3(side_out / 4, y, r_out));
                    }
                }
            }
        } else if cut == Cut::Wall && pick(k.wrapping_mul(8) + i, 3, 0, 5) / 2 == 1 {
            lit_window(b, &g, v3(0, h / 2 + ch / 6, r_out));
        }
        if cut == Cut::Door {
            // a gable over the doorway outside, and two torches
            put(
                b,
                &g,
                Mesh::Pyramid,
                v3(0, h - int(1), r_out + cm(30)),
                v3(DOOR_W + int(3), int(3), int(2)),
                STONE,
                0,
            );
            for s in [-1, 1] {
                let x = s * (DOOR_W / 2 + int(2));
                put(
                    b,
                    &g,
                    Mesh::Hex,
                    v3(x, h / 3, r_out + cm(30)),
                    v3(cm(24), int(1), cm(24)),
                    WOOD_DARK,
                    0,
                );
                put(
                    b,
                    &g,
                    Mesh::Spike,
                    v3(x, h / 3 + cm(90), r_out + cm(30)),
                    v3(cm(50), cm(90), cm(50)),
                    WINDOW,
                    ONE,
                );
            }
        }
        // the inner ledge at the foot: a walk on corbels
        if ledge == Ledge::Walk && near {
            put(
                b,
                &g,
                Mesh::Cube,
                v3(0, LEDGE_THICK / 2, r_in - LEDGE_DEPTH / 2),
                v3(
                    side_in - 2 * mul(LEDGE_DEPTH, TAN8),
                    LEDGE_THICK,
                    LEDGE_DEPTH,
                ),
                STONE_DARK,
                0,
            );
            // one corbel under its middle
            put_turned(
                b,
                &g,
                Mesh::Wedge,
                v3(0, -int(1), r_in - LEDGE_DEPTH * 2 / 5),
                &Affine::rot_z(HALF).then(&Affine::rot_y(HALF)),
                v3(cm(80), int(2), LEDGE_DEPTH * 4 / 5),
                STONE,
                0,
            );
            // a low rail along its edge
            put(
                b,
                &g,
                Mesh::Cube,
                v3(0, LEDGE_THICK + cm(45), r_in - LEDGE_DEPTH + cm(10)),
                v3(side_in - 2 * mul(LEDGE_DEPTH, TAN8), cm(14), cm(20)),
                STONE_LIGHT,
                0,
            );
        }
    }
}

/// A course of a wall skin `w` wide from `y0` to `y1` at `z` on face `g`,
/// split round the opening `hole` (width, bottom, top) when it has one.
#[allow(clippy::too_many_arguments)]
fn skin_course(
    b: &mut Batch,
    g: &Affine,
    hole: Option<(i32, i32, i32)>,
    w: i32,
    y0: i32,
    y1: i32,
    z: i32,
    c: Rgb,
) {
    let Some((hw, lo, hi)) = hole else {
        put(
            b,
            g,
            Mesh::Cube,
            v3(0, (y0 + y1) / 2, z),
            v3(w, y1 - y0, int(1)),
            c,
            0,
        );
        return;
    };
    if y1 <= lo || y0 >= hi {
        put(
            b,
            g,
            Mesh::Cube,
            v3(0, (y0 + y1) / 2, z),
            v3(w, y1 - y0, int(1)),
            c,
            0,
        );
        return;
    }
    // the parts of the course under and over the hole
    if y0 < lo {
        put(
            b,
            g,
            Mesh::Cube,
            v3(0, (y0 + lo) / 2, z),
            v3(w, lo - y0, int(1)),
            c,
            0,
        );
    }
    if y1 > hi {
        put(
            b,
            g,
            Mesh::Cube,
            v3(0, (hi + y1) / 2, z),
            v3(w, y1 - hi, int(1)),
            c,
            0,
        );
    }
    // either side of it
    let (a, zz) = (y0.max(lo), y1.min(hi));
    if zz > a {
        for s in [-1, 1] {
            let x0 = s * hw / 2;
            let x1 = s * w / 2;
            put(
                b,
                g,
                Mesh::Cube,
                v3((x0 + x1) / 2, (a + zz) / 2, z),
                v3((x1 - x0).abs(), zz - a, int(1)),
                c,
                0,
            );
        }
    }
}

fn slit(b: &mut Batch, g: &Affine, at: V3) {
    put(
        b,
        g,
        Mesh::Cube,
        at.add(v3(0, 0, cm(2))),
        v3(cm(34), cm(240), cm(12)),
        super::SLIT,
        0,
    );
}

fn lit_window(b: &mut Batch, g: &Affine, at: V3) {
    put(
        b,
        g,
        Mesh::Cube,
        at.add(v3(0, 0, cm(3))),
        v3(cm(160), cm(300), cm(14)),
        WINDOW,
        ONE * 9 / 10,
    );
    put(
        b,
        g,
        Mesh::Pyramid,
        at.add(v3(0, cm(185), cm(6))),
        v3(cm(210), cm(80), cm(40)),
        STONE_DARK,
        0,
    );
}

/// The open top of a hollow tower, its foot at y 0 on a tower of outer
/// apothem `r_out` and inner `r_in`: a corbelled parapet with merlons
/// round the outside, a stone walk ring between the skins, a low inner
/// rail, `turrets` turrets on the corners; no roof, the sky over the
/// inside. About 145 shapes with four turrets.
pub fn hollow_top(b: &mut Batch, f: &Affine, r_out: i32, r_in: i32, turrets: u32, k: u32) {
    let out = r_out + cm(60);
    let side = side_of(out);
    let side_in = side_of(r_in);
    for i in 0..8u32 {
        let g = f.then(&Affine::rot_y(TURN / 8 * i as i32));
        for s in [-1, 1] {
            put_turned(
                b,
                &g,
                Mesh::Wedge,
                v3(s * side / 4, cm(60), r_out + cm(30)),
                &Affine::rot_z(HALF),
                v3(cm(60), cm(120), cm(80)),
                STONE_DARK,
                0,
            );
        }
        put(
            b,
            &g,
            Mesh::Cube,
            v3(0, int(1) + cm(90), out - cm(40)),
            v3(side + cm(10), cm(140), cm(80)),
            stone(k + i),
            0,
        );
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
        // the walk ring: a slab a face wide between the skins
        let mid = (r_out + r_in) / 2;
        put(
            b,
            &g,
            Mesh::Cube,
            v3(0, int(1), mid),
            v3(side_of(mid) + cm(20), cm(40), r_out - r_in + cm(60)),
            STONE_DARK,
            0,
        );
        // the inner rail
        put(
            b,
            &g,
            Mesh::Cube,
            v3(0, int(1) + cm(70), r_in - cm(20)),
            v3(side_in + cm(10), cm(16), cm(24)),
            STONE_LIGHT,
            0,
        );
        for m in [-1, 0, 1] {
            put(
                b,
                &g,
                Mesh::Hex,
                v3(m * side_in / 3, int(1) + cm(40), r_in - cm(20)),
                v3(cm(16), cm(60), cm(16)),
                STONE,
                0,
            );
        }
    }
    for t in 0..turrets {
        let a = TURN / 16 + TURN * t as i32 / turrets.max(1) as i32;
        let at = round(r_out + cm(40), a, -int(5));
        turret(
            b,
            &f.then(&Affine::translate(at)),
            int(2) + r_out / 10,
            int(11),
            k + t,
        );
    }
}

/// A balcony on a hollow tower's inner wall (inner apothem `r_in`), at
/// bearing `a` and height `y` (its floor), `w` wide, facing the axis: the
/// keep's balcony with its lit doorway. Returns the frame a mage stands
/// in (its origin on the balcony floor, 1 m out from the wall, facing the
/// axis). About 22 shapes.
pub fn inner_balcony(
    b: &mut Batch,
    f: &Affine,
    r_in: i32,
    a: i32,
    y: i32,
    w: i32,
    k: u32,
) -> Affine {
    let g = f
        .then(&Affine::rot_y(a))
        .then(&Affine::translate(v3(0, y, r_in)))
        .then(&Affine::rot_y(HALF));
    balcony(b, &g, w, int(2), k);
    g.then(&Affine::translate(v3(0, 0, int(1))))
}

/// A balcony on a tower's outer wall (apothem `r`) at bearing `a` and
/// height `y`, `w` wide, facing out. Returns the mage's frame as
/// `inner_balcony` does.
pub fn outer_balcony(b: &mut Batch, f: &Affine, r: i32, a: i32, y: i32, w: i32, k: u32) -> Affine {
    let g = f
        .then(&Affine::rot_y(a))
        .then(&Affine::translate(v3(0, y, r)));
    balcony(b, &g, w, int(2), k);
    g.then(&Affine::translate(v3(0, 0, int(1))))
}

/// Which faces of tower 1 are windows in section `s` of `n`, with the
/// doorway (face `door_face`) in section `door_section`: two windows a
/// section on alternating faces from the fourth section up, none on the
/// doorway's section.
pub fn tower1_cuts(s: u32, door_section: u32, door_face: u32) -> [Cut; 8] {
    let mut cuts = [Cut::Wall; 8];
    if s == door_section {
        cuts[(door_face % 8) as usize] = Cut::Door;
        return cuts;
    }
    if s >= 3 {
        let a = (s * 3) % 8;
        cuts[a as usize] = Cut::Window;
        cuts[((a + 4) % 8) as usize] = Cut::Window;
    }
    cuts
}

/// The whole of tower 1 (`TOWER1_HEIGHT` tall unless `h` says otherwise,
/// outer apothem `r_out`, inner `r_in`): the keep's base, sections of
/// `SECTION_H` with a ledge round the inside every other section, windows
/// (`tower1_cuts`), the doorway on face `door_face` in the section at
/// height `door_y`, the open top with four turrets. The layout should
/// place the sections itself, by chunk (`hollow_section`, `Detail::Far`
/// far off); this is the whole for the gallery and tests. About 2,850
/// shapes at 274 m.
#[allow(clippy::too_many_arguments)]
pub fn tower1(
    b: &mut Batch,
    f: &Affine,
    r_out: i32,
    r_in: i32,
    h: i32,
    door_y: i32,
    door_face: u32,
    k: u32,
) {
    super::keep_base(b, f, r_out, int(6), k);
    let n = (h / SECTION_H).max(1) as u32;
    let door_section = (door_y / SECTION_H).clamp(0, n as i32 - 1) as u32;
    for s in 0..n {
        let g = f.then(&Affine::translate(v3(0, SECTION_H * s as i32, 0)));
        let ledge = if s % 2 == 0 && s > 0 {
            Ledge::Walk
        } else {
            Ledge::None
        };
        hollow_section(
            b,
            &g,
            r_out,
            r_in,
            SECTION_H,
            k.wrapping_add(s * 31),
            tower1_cuts(s, door_section, door_face),
            ledge,
            Detail::Near,
        );
    }
    let top = f.then(&Affine::translate(v3(0, SECTION_H * n as i32, 0)));
    hollow_top(b, &top, r_out, r_in, 4, k);
}

/// The height of a hollow tower's walk ring over its foot for `h` of
/// sections (`tower1`): the sections' top plus 1 m.
pub fn tower1_top(h: i32) -> i32 {
    SECTION_H * (h / SECTION_H).max(1) + int(1)
}

/// Tower 2: the keep's look at `h` tall (apothem `r`): its base, sections
/// and crown, with `balconies` balconies spiralling `turns` turns up its
/// outside from a third of the way up to just under the crown (starting at
/// bearing `a0`), and `clouds` clouds round its top. Returns the mage
/// frames of the balconies, lowest first. About 1,300 shapes at 164 m.
#[allow(clippy::too_many_arguments)]
pub fn tower2(
    b: &mut Batch,
    f: &Affine,
    r: i32,
    h: i32,
    balconies: u32,
    turns: i32,
    a0: i32,
    clouds: u32,
    k: u32,
) -> Vec<Affine> {
    super::keep_base(b, f, r, int(6), k);
    let n = (h / SECTION_H).max(1) as u32;
    for s in 0..n {
        let g = f.then(&Affine::translate(v3(0, SECTION_H * s as i32, 0)));
        keep_section(b, &g, r, SECTION_H, k.wrapping_add(s * 31), Detail::Near);
    }
    let top = f.then(&Affine::translate(v3(0, SECTION_H * n as i32, 0)));
    keep_crown(b, &top, r, 4, k);
    let frames = spiral_balconies(
        b,
        f,
        r,
        h / 3,
        SECTION_H * n as i32 - int(6),
        balconies,
        turns,
        a0,
        k,
    );
    cloud_ring(b, f, r + int(14), SECTION_H * n as i32 - int(10), clouds, k);
    frames
}

/// `n` balconies on a tower's outside (apothem `r`), from height `y0` to
/// `y1`, spiralling `turns` turns (in `TURN` units, so `TURN * 3 / 2` is a
/// turn and a half) from bearing `a0`. Returns the mage frames.
#[allow(clippy::too_many_arguments)]
pub fn spiral_balconies(
    b: &mut Batch,
    f: &Affine,
    r: i32,
    y0: i32,
    y1: i32,
    n: u32,
    turns: i32,
    a0: i32,
    k: u32,
) -> Vec<Affine> {
    let mut out = Vec::new();
    for i in 0..n {
        let t = if n > 1 {
            ONE * i as i32 / (n as i32 - 1)
        } else {
            0
        };
        let a = a0.wrapping_add(mul(turns, t));
        let y = y0 + mul(y1 - y0, t);
        out.push(outer_balcony(b, f, r, a, y, int(5), k + i));
    }
    out
}

/// `n` clouds round a tower at radius `r` (their middles) and height `y`,
/// each about 24 m across, bobbing up and down a few metres by `k`.
pub fn cloud_ring(b: &mut Batch, f: &Affine, r: i32, y: i32, n: u32, k: u32) {
    for i in 0..n {
        let kk = k.wrapping_mul(17) + i;
        let a = TURN * i as i32 / n.max(1) as i32 + pick(kk, 4, -8, 8) * (TURN / 100);
        let dy = pick(kk, 5, -6, 6) * ONE;
        let at = round(r + pick(kk, 6, -4, 6) * ONE, a, y + dy);
        let w = int(20) + pick(kk, 7, 0, 8) * ONE;
        cloud(b, &f.then(&Affine::translate(at)), w, w / 3, kk);
    }
}

/// A long straight stone bridge from `a` to `z` (the deck's top at each
/// end, in the middle of its width), `w` wide: a deck, parapets with
/// coping and a lantern every 25 m, and an arcade of arches hanging under
/// it (a corbelled arch between pendant piers every 25 m). No vine
/// hangers: `bridge_hangers` adds those. About 130 shapes for 250 m.
pub fn long_bridge(b: &mut Batch, f: &Affine, a: V3, z: V3, w: i32, k: u32) {
    let d = z.sub(a);
    let len = d.len();
    let g = f
        .then(&Affine::translate(a))
        .then(&Affine::looking(d, V3::UP));
    put(
        b,
        &g,
        Mesh::Bevel,
        v3(0, -cm(40), len / 2),
        v3(w, cm(80), len),
        STONE_DARK,
        0,
    );
    for s in [-1, 1] {
        put(
            b,
            &g,
            Mesh::Cube,
            v3(s * (w / 2 - cm(25)), cm(55), len / 2),
            v3(cm(50), int(1) + cm(10), len),
            STONE,
            0,
        );
        put(
            b,
            &g,
            Mesh::Cube,
            v3(s * (w / 2 - cm(25)), int(1) + cm(18), len / 2),
            v3(cm(70), cm(16), len + cm(10)),
            STONE_LIGHT,
            0,
        );
    }
    // the arcade under the deck
    let span = int(25);
    let bays = (len / span).max(1);
    let bay = len / bays;
    let drop = int(7);
    for i in 0..=bays {
        let zz = bay * i;
        // a pendant pier under each arch's springing
        put(
            b,
            &g,
            Mesh::Cube,
            v3(0, -cm(80) - drop / 2, zz),
            v3(w + cm(40), drop, int(2)),
            stone(k + i as u32),
            0,
        );
        put(
            b,
            &g,
            Mesh::Pyramid,
            v3(0, -cm(80) - drop - cm(70), zz),
            v3(w + cm(40), cm(140), int(2)),
            STONE_DARK,
            0,
        );
    }
    let n = 6;
    for i in 0..bays {
        let z0 = bay * i;
        let arch = |j: i32| {
            let t = HALF * j / n;
            v3(
                0,
                -cm(80) - drop + mul(drop - int(1), crate::fx::sin(t)),
                z0 + bay * j / n,
            )
        };
        for j in 0..n {
            let (p, q) = (arch(j), arch(j + 1));
            slab(
                b,
                &g,
                Mesh::Cube,
                p,
                q,
                w + cm(20),
                cm(80),
                if j % 2 == 0 { STONE_DARK } else { STONE },
                0,
            );
        }
    }
    // lanterns along the parapet
    let lamps = (len / span).max(1);
    for i in 0..lamps {
        let s = if (k + i as u32).is_multiple_of(2) {
            1
        } else {
            -1
        };
        let at = v3(s * (w / 2 - cm(25)), int(1) + cm(20), bay * i + bay / 2);
        put(
            b,
            &g,
            Mesh::Hex,
            at.add(v3(0, cm(60), 0)),
            v3(cm(16), int(1) + cm(20), cm(16)),
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
    // a small gate turret at each end's left parapet
    for (zz, kk) in [(int(3), k), (len - int(3), k + 1)] {
        let at = v3(-(w / 2 + cm(60)), -int(1), zz);
        turret(b, &g.then(&Affine::translate(at)), cm(140), int(4), kk);
    }
}

/// Where a bridge's vine hangers hang, in the frame of the bridge from
/// `a` to `z` (the deck's top): one every `spacing` from the first end
/// (the last within half a spacing of the far end is left out), each
/// `HANGER_DROP` under the deck. The points are the hooks' tips, where
/// the vine's anchor goes.
pub fn hanger_points(a: V3, z: V3, spacing: i32) -> Vec<V3> {
    let d = z.sub(a);
    let len = d.len();
    let n = (len / spacing.max(int(1))).max(1);
    (0..=n)
        .filter(|i| len - spacing * i >= spacing / 2 || *i == 0)
        .map(|i| {
            a.lerp(z, crate::fx::div(spacing * i, len))
                .add(v3(0, -HANGER_DROP, 0))
        })
        .collect()
}

/// How far under the deck a hanger's hook tip hangs.
pub const HANGER_DROP: i32 = int(4);

/// The vine hangers under a bridge from `a` to `z` (`w` wide): at each of
/// `hanger_points`, an iron bracket bolted under the deck's middle, a
/// chain down to a hook with a gold ring at its tip. Returns the hook
/// tips (in `f`'s frame, as `hanger_points`). About 5 shapes a hanger.
pub fn bridge_hangers(
    b: &mut Batch,
    f: &Affine,
    a: V3,
    z: V3,
    w: i32,
    spacing: i32,
    k: u32,
) -> Vec<V3> {
    let points = hanger_points(a, z, spacing);
    let d = z.sub(a);
    let g = f
        .then(&Affine::translate(a))
        .then(&Affine::looking(d, V3::UP));
    let iron = super::SLATE_DARK;
    for (i, p) in points.iter().enumerate() {
        let zz = p.sub(a).len();
        // the bracket under the deck
        put(
            b,
            &g,
            Mesh::Cube,
            v3(0, -cm(80) - cm(30), zz),
            v3(w / 2, cm(60), cm(80)),
            iron,
            0,
        );
        put_turned(
            b,
            &g,
            Mesh::Wedge,
            v3(0, -cm(80) - cm(90), zz),
            &Affine::rot_z(HALF),
            v3(cm(80), cm(60), cm(80)),
            STONE_DARK,
            0,
        );
        // the chain down to the hook
        super::bar(
            b,
            &g,
            Mesh::Hex,
            v3(0, -cm(80) - cm(60), zz),
            v3(0, -HANGER_DROP + cm(70), zz),
            cm(14),
            iron,
            0,
        );
        // the hook: a half ring, open toward the near end
        let tip = v3(0, -HANGER_DROP, zz);
        put_turned(
            b,
            &g,
            Mesh::Cube,
            tip.add(v3(0, cm(35), 0)),
            &Affine::rot_x(QUARTER / 2),
            v3(cm(18), cm(80), cm(18)),
            iron,
            0,
        );
        put(
            b,
            &g,
            Mesh::Hex,
            tip,
            v3(cm(50), cm(10), cm(50)),
            GOLD,
            ONE / 2 + ONE * ((i as u32 + k) % 2) as i32 / 4,
        );
    }
    points
}
