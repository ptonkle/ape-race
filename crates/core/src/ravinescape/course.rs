//! Rattlesnake Ravine's course art (step 8, SECOND-ORDER RA-n rows): the
//! canyon drawn from its layout (`RavineLayout`), so what is drawn is what
//! is solid (MAPS "Common gotchas" 1–2).
//!
//! - **The walls** come from the same `Wall` list as `ravine::solids`, each
//!   stretch a stack of sandstone bands from the solid's foot (`WALL_BELOW`
//!   under the face's foot) to its top (`WALL_ABOVE` over the rim), so the
//!   fin's pieces over and under the skull's mouth reach exactly as far as
//!   their solids (RC3). The bands are set back from the face by 0–24 cm,
//!   never proud of it. Moss at the foot, lichen on the face, fossils,
//!   dinosaur bones and boulders set in the rock.
//! - **The roofs** of the arches, caves and the mine tunnel are the solids'
//!   own pieces (`ROOF_PIECE` long, `ROOF_THICK` thick); the caves' and the
//!   tunnel's side walls likewise. Mouths are left open.
//! - **The decks** are drawn straight from end to end (the solid deck is the
//!   straight line, `Deck::top_at`): rope bridges and fallen logs.
//! - **Hangers:** a ledge vine hangs from a gnarled juniper reaching out
//!   from its wall; a roof vine from an iron peg; a deck vine from a knot.
//! - **Moving things** (`dynamic`): snake vines (whole, or snapped from
//!   `Hanger.snapped_at`), lizards (`lizard_at`), carts (`cart_at`), the
//!   glows' torches, lanterns, mushrooms and fireflies, the campfire.
//!
//! Placement randomness has its own dice (`ART_DICE`), so no other map's
//! draws move.

use crate::course::{ground, Course, OUTRO, STEP};
use crate::fx::{self, cm, deg, int, Dice, ONE, TURN};
use crate::geom::{right_of, v3, yaw_of, Affine, V3};
use crate::mesh::Mesh;
use crate::ravine::solids::{ROOM_SIDE, WALL_ABOVE, WALL_BELOW};
use crate::ravine::{
    DeckKind, GlowKind, HangFrom, LizardColour, RavineLayout, RoomKind, Side, ROOF_THICK,
    SNAKE_REGROW,
};
use crate::ravinemodels as m;
use crate::render::{placed, rgb, Batch, Rgb};
use crate::scene::{assemble, chunk_count, chunk_of, push_ground, Scenery, CHUNK};

/// This art's own dice (maps-archive "Layout lessons").
pub const ART_DICE: u64 = 0x5241_5641_5254_0008;
/// A roof is drawn in pieces this long, as its solid (`solids.rs`).
const ROOF_PIECE: i32 = int(8);
/// The floor's columns reach this far past each wall's face, under the rock.
const FLOOR_UNDER: i32 = int(4);
/// A chunk (100 m of canyon) is drawn while its middle is this near the
/// eye: the canyon winds, so beyond it the walls hide most of a chunk, and
/// what shows is deep in the night haze (the fog's far end is 260 m).
pub const REACH: i32 = int(210);
/// Moving things are drawn within this of the eye.
pub const MOVERS_SEEN: i32 = int(200);
/// The glows' models are drawn within this of the eye.
pub const GLOWS_SEEN: i32 = int(150);

/// The rock's bands, foot to rim: wet and dark at the foot, the slot
/// canyon's red-orange, a paler banded middle, the bleached rim.
const ROCK_FOOT: Rgb = rgb(124, 58, 38);
const ROCK_MID: [Rgb; 3] = [rgb(196, 98, 58), rgb(178, 84, 50), rgb(206, 116, 66)];
const ROCK_BAND: Rgb = rgb(222, 142, 88);
const ROCK_RIM: Rgb = rgb(230, 168, 116);
const ROOF: Rgb = rgb(150, 70, 44);
const FLOOR: Rgb = rgb(150, 92, 64);
const FLOOR_DARK: Rgb = rgb(118, 70, 50);
const MOSS: Rgb = rgb(70, 110, 46);
const MOSS_DARK: Rgb = rgb(46, 84, 36);
const LICHEN: [Rgb; 3] = [rgb(196, 206, 120), rgb(226, 170, 70), rgb(160, 196, 160)];
const JUNIPER: Rgb = rgb(104, 84, 66);
const NEEDLES: Rgb = rgb(66, 96, 64);
const PEG: Rgb = rgb(70, 66, 64);
/// The warning glow round a snake vine's head and rattle (normal, then the
/// colourblind mode's: orange reads apart from every vine look).
const SNAKE_GLOW: Rgb = rgb(255, 120, 40);
const SNAKE_GLOW_VIVID: Rgb = rgb(255, 150, 0);
/// The colourblind mode's glow round a wall lizard (a bouncer): white,
/// apart from the snakes' orange and the vines' green by hue and brightness.
pub const LIZARD_GLOW_VIVID: Rgb = rgb(240, 240, 255);

fn into_rock(side: Side, fwd: V3) -> V3 {
    match side {
        Side::Left => right_of(fwd).neg(),
        Side::Right => right_of(fwd),
    }
}

/// A wall stretch's solid extent, bottom to top (as `Solids::of`).
pub fn stretch_heights(a: &crate::ravine::WallPoint, b: &crate::ravine::WallPoint) -> (i32, i32) {
    (
        a.at.y.min(b.at.y) - WALL_BELOW,
        (a.at.y + a.height).max(b.at.y + b.height) + WALL_ABOVE,
    )
}

/// An upright box from its middle, the way along it and its sizes (across,
/// up, along).
fn block(mid: V3, fwd: V3, size: V3) -> Affine {
    placed(mid, yaw_of(fwd), size)
}

/// Rattlesnake Ravine's scenery and ground, chunked.
pub fn scenery(c: &Course, r: &RavineLayout) -> Scenery {
    let n = chunk_count(c);
    let mut batches: Vec<Batch> = (0..n).map(|_| Batch::new()).collect();
    let mut dice = Dice(ART_DICE);
    let chunk = |p: V3| chunk_of(c, p, n);
    walls(&mut batches, &chunk, r, &mut dice);
    rooms(&mut batches, &chunk, c, r, &mut dice);
    decks(&mut batches, &chunk, r);
    hangers(&mut batches, &chunk, c, r, &mut dice);
    tracks(&mut batches, &chunk, r);
    rocks(&mut batches, &chunk, r, &mut dice);
    if let Some(s) = &r.skull {
        let k = chunk(s.at);
        let gap = crate::ravine::MOUTH_GAP;
        m::rex_skull(&mut batches[k], &skull_frame(s), gap, &m::rex_teeth(gap));
    }
    finish(&mut batches[n - 1], r);
    let (terrain, ranges) = floor(c, r, n);
    Scenery {
        reach: REACH,
        ..assemble(c, batches, terrain, ranges)
    }
}

/// The skull's model frame (the same as the layout's teeth and the
/// solids': its snout, model +z, back into the in-leg).
pub fn skull_frame(s: &crate::ravine::Skull) -> Affine {
    Affine::rot_y(yaw_of(crate::geom::facing(s.yaw, 0).neg())).at(s.at)
}

/// The sphinx's model frame: its box's middle on the ground, facing
/// `sphinx_yaw` (its front feet at `Finish.sphinx`, as the solid).
pub fn sphinx_frame(r: &RavineLayout) -> Affine {
    let f = &r.finish;
    let fwd = crate::geom::facing(f.sphinx_yaw, 0);
    let mid = f.sphinx.sub(fwd.scale(m::SPHINX_SIZE.z / 2));
    Affine::translate(mid).then(&Affine::rot_y(f.sphinx_yaw))
}

fn walls(b: &mut [Batch], chunk: &dyn Fn(V3) -> usize, r: &RavineLayout, dice: &mut Dice) {
    let mut seg = 0u32;
    for w in &r.walls {
        for k in w.points.windows(2) {
            seg += 1;
            let (a, z) = (k[0], k[1]);
            let run = z.at.sub(a.at).flat();
            let len = run.len();
            if len <= 0 {
                continue;
            }
            let fwd = run.norm();
            let into = into_rock(w.side, fwd);
            let out = into.neg();
            let (bottom, top) = stretch_heights(&a, &z);
            let foot = a.at.y.min(z.at.y);
            // dressing stays under the drawn rim (the mesa's low walls)
            let rim = (top - WALL_ABOVE - int(2)).max(foot + int(2));
            let face = a.at.lerp(z.at, ONE / 2);
            let bk = &mut b[chunk(face)];
            // the bands: each set back from the face a little (neighbours
            // differ, so overlapping ends never share a plane)
            let lo_cut = (foot + int(9) + cm(dice.range(-150, 150))).clamp(bottom, top);
            let hi_cut = (top - WALL_ABOVE - int(8)).clamp(lo_cut, top);
            let bands = [
                (bottom, lo_cut, ROCK_FOOT),
                (lo_cut, hi_cut, ROCK_MID[(seg % 3) as usize]),
                (
                    hi_cut,
                    top,
                    if seg.is_multiple_of(2) {
                        ROCK_RIM
                    } else {
                        ROCK_BAND
                    },
                ),
            ];
            for (j, &(y0, y1, col)) in bands.iter().enumerate() {
                if y1 - y0 < cm(20) {
                    continue;
                }
                let inset = cm(((seg as i32 + j as i32) % 3) * 12);
                let thick = w.depth - inset;
                let mid = face.add(into.scale(inset + thick / 2));
                bk.push(
                    Mesh::Cube,
                    &block(
                        v3(mid.x, (y0 + y1) / 2, mid.z),
                        fwd,
                        v3(thick, y1 - y0, len + int(1)),
                    ),
                    col,
                    0,
                );
            }
            // the canyon mouth (RA-11, RA-12): where the rims fall to the
            // mesa a stretch stands a storey over the next; rock heaped
            // on the step and sloping away outside, so the walls end as a
            // canyon mouth, not as flat slabs
            if (a.height - z.height).abs() > MOUTH_STEP {
                mouth_rock(bk, (a, z), fwd, into, w.depth, (foot, top), seg);
            }
            // only on faces apes see: stretches standing on the floor
            if top - bottom < int(18) || bottom > foot {
                // (the fin's piece over the skull's mouth: a mossy lip)
                if bottom > foot {
                    let lip = face.add(out.scale(cm(10)));
                    bk.push(
                        Mesh::Blob,
                        &block(
                            v3(lip.x, bottom + cm(10), lip.z),
                            fwd,
                            v3(cm(80), cm(40), len),
                        ),
                        MOSS,
                        0,
                    );
                }
                continue;
            }
            // moss in clumps along the foot (not every stretch: a row of
            // equal lumps read as a hedge), and a hanging curtain now and then
            let moss = face.add(out.scale(cm(5)));
            if dice.below(3) != 0 {
                let t = ONE * dice.range(20, 80) / 100;
                let at = a.at.lerp(z.at, t).add(out.scale(cm(5)));
                bk.push(
                    Mesh::Rock,
                    &block(
                        v3(at.x, foot + cm(dice.range(0, 40)), at.z),
                        fwd,
                        v3(
                            cm(dice.range(120, 220)),
                            cm(dice.range(60, 200)),
                            len * dice.range(40, 90) / 100,
                        ),
                    ),
                    if seg.is_multiple_of(2) {
                        MOSS
                    } else {
                        MOSS_DARK
                    },
                    0,
                );
            }
            if dice.below(5) == 0 {
                let y = (foot + int(dice.range(12, 40))).min(rim - int(3));
                bk.push(
                    Mesh::Blob,
                    &block(
                        v3(moss.x, y, moss.z),
                        fwd,
                        v3(cm(50), int(dice.range(3, 7)), len * 2 / 3),
                    ),
                    MOSS_DARK,
                    0,
                );
            }
            // lichen: flat rosettes on the face
            if dice.below(2) == 0 {
                let t = ONE * dice.range(10, 90) / 100;
                let p = a.at.lerp(z.at, t).add(out.scale(cm(3)));
                let y = (foot + int(dice.range(2, 30))).min(rim);
                let d = cm(dice.range(80, 240));
                bk.push(
                    Mesh::Cyl,
                    &Affine::translate(v3(p.x, y, p.z))
                        .then(&Affine::looking(V3::UP, out))
                        .then(&Affine::scale(v3(d, cm(6), d * 2 / 3))),
                    LICHEN[dice.below(3) as usize],
                    0,
                );
            }
            // a fossil, a dinosaur bone or a boulder set in the rock
            match dice.below(14) {
                0 => {
                    let kind = match dice.below(3) {
                        0 => m::Fossil::Ribs,
                        1 => m::Fossil::Spine,
                        _ => m::Fossil::Ammonite,
                    };
                    let y = (foot + int(dice.range(3, 22))).min(rim - int(2));
                    let f = Affine::translate(v3(face.x, y, face.z))
                        .then(&Affine::looking(out, V3::UP))
                        .then(&Affine::scale(v3(ONE * 3 / 2, ONE * 3 / 2, ONE)));
                    m::fossil_inlay(bk, &f, kind, seg);
                }
                1..=2 => {
                    // a long bone (a femur) lying in the strata
                    let y = (foot + int(dice.range(4, 26))).min(rim - int(1));
                    let p = face.add(out.scale(cm(15)));
                    let tilt = cm(dice.range(-150, 150));
                    let half = fwd.scale(int(dice.range(2, 3)));
                    let (p0, p1) = (
                        v3(p.x, y - tilt, p.z).sub(half),
                        v3(p.x, y + tilt, p.z).add(half),
                    );
                    bk.push(Mesh::Hex, &crate::render::rod(p0, p1, cm(55)), m::BONE, 0);
                    for q in [p0, p1] {
                        bk.push(
                            Mesh::Ico,
                            &placed(q, 0, v3(cm(110), cm(90), cm(80))),
                            m::BONE_DARK,
                            0,
                        );
                    }
                }
                3..=4 => {
                    let y = (foot + int(dice.range(1, 30))).min(rim - int(1));
                    let p = face.add(into.scale(cm(40)));
                    let s = cm(dice.range(150, 320));
                    bk.push(
                        Mesh::Rock,
                        &placed(
                            v3(p.x, y, p.z),
                            dice.range(0, 359) * TURN / 360,
                            v3(s, s * 3 / 4, s),
                        ),
                        m::SANDSTONE_DARK,
                        0,
                    );
                }
                _ => {}
            }
        }
    }
}

/// A stretch whose ends' rims differ by more than this is a step of the
/// canyon mouth (the slot's rims ease down 1.5 m a stretch at most; past
/// the finish they fall some 9 m a stretch to the mesa's low rims).
pub const MOUTH_STEP: i32 = int(4);

/// The rock of one canyon-mouth step (`a`, `z` the stretch's ends, `foot`
/// and `top` its drawn extent): a talus heaped over the step down onto the
/// next stretch's top, and three shoulders falling away outside it, each
/// lower and further out, so the rock slopes down to the mesa top. Drawing
/// only: all of it is outside the canyon or on the wall's own rock.
fn mouth_rock(
    bk: &mut Batch,
    (a, z): (crate::ravine::WallPoint, crate::ravine::WallPoint),
    fwd: V3,
    into: V3,
    depth: i32,
    (foot, top): (i32, i32),
    seg: u32,
) {
    let drop = (a.height - z.height).abs();
    let (hi, lo) = if a.height > z.height { (a, z) } else { (z, a) };
    let down = lo.at.sub(hi.at).flat().norm();
    let mid_rock = |p: V3| p.add(into.scale(depth / 2));
    // the talus: a big rock over the cut end, a smaller one past it
    for (k, (on, up, size, along)) in [
        (cm(150), drop * 11 / 20, drop * 13 / 10, int(10)),
        (int(6), drop + cm(50), drop * 4 / 5, int(7)),
    ]
    .into_iter()
    .enumerate()
    {
        let p = mid_rock(lo.at).add(down.scale(on));
        bk.push(
            Mesh::Rock,
            &placed(
                v3(p.x, top - up, p.z),
                yaw_of(fwd) + (seg as i32 * 37 + k as i32 * 90) % 40 * TURN / 360,
                v3(depth * 23 / 20, size, along),
            ),
            if k == 0 {
                ROCK_BAND
            } else {
                ROCK_MID[(seg % 3) as usize]
            },
            0,
        );
    }
    // the shoulders outside: lower and further out each time
    let mid = a.at.lerp(z.at, ONE / 2);
    let tall = top - foot;
    let len = z.at.sub(a.at).flat().len();
    for k in 1..=3 {
        let h = tall * (4 - k) / 4;
        let p = mid.add(into.scale(depth + int(7) * k - int(2)));
        bk.push(
            Mesh::Rock,
            &block(
                v3(p.x, foot + h / 2 - int(1), p.z),
                fwd,
                v3(int(10), h, len * 7 / 5),
            ),
            match k {
                1 => ROCK_MID[((seg + 1) % 3) as usize],
                2 => ROCK_BAND,
                _ => m::SANDSTONE_DARK,
            },
            0,
        );
    }
}

fn rooms(
    b: &mut [Batch],
    chunk: &dyn Fn(V3) -> usize,
    c: &Course,
    r: &RavineLayout,
    dice: &mut Dice,
) {
    for room in &r.rooms {
        if room.kind == RoomKind::Skull {
            continue;
        }
        let run = room.to.sub(room.from).flat();
        let len = run.len();
        if len <= 0 {
            continue;
        }
        let fwd = run.norm();
        let pieces = (len + ROOF_PIECE - 1) / ROOF_PIECE;
        let floor = |k: i32| room.from.y + (room.to.y - room.from.y) * k / pieces;
        for k in 0..pieces {
            let under = floor(k).max(floor(k + 1)) + room.height;
            let mid = room.from.lerp(
                room.to,
                ((2 * k + 1) as i64 * ONE as i64 / (2 * pieces) as i64) as i32,
            );
            let bk = &mut b[chunk(mid)];
            let across = 2 * (room.half_width + ROOM_SIDE);
            bk.push(
                Mesh::Cube,
                &block(
                    v3(mid.x, under + ROOF_THICK / 2, mid.z),
                    fwd,
                    v3(across, ROOF_THICK, len / pieces + cm(10)),
                ),
                if k % 2 == 0 { ROOF } else { m::SANDSTONE_DARK },
                0,
            );
            // lumps on the roof's top (never under its underside), moss
            // and hanging roots at its edges
            let lump = mid.add(right_of(fwd).scale(int(dice.range(-12, 12))));
            let s = int(dice.range(4, 8));
            bk.push(
                Mesh::Rock,
                &placed(
                    v3(lump.x, under + ROOF_THICK, lump.z),
                    dice.range(0, 359) * TURN / 360,
                    v3(s * 2, s, s * 2),
                ),
                m::SANDSTONE,
                0,
            );
            if room.kind == RoomKind::Arch && (k == 0 || k == pieces - 1) {
                let edge = room.from.lerp(room.to, if k == 0 { 0 } else { ONE });
                bk.push(
                    Mesh::Blob,
                    &block(
                        v3(edge.x, under + cm(30), edge.z),
                        fwd,
                        v3(across - int(14), cm(70), cm(160)),
                    ),
                    MOSS,
                    0,
                );
            }
        }
        if matches!(room.kind, RoomKind::Cave | RoomKind::Tunnel) {
            let bottom = room.from.y.min(room.to.y) - WALL_BELOW;
            let top = room.from.y.max(room.to.y) + room.height + ROOF_THICK;
            let mid = room.from.lerp(room.to, ONE / 2);
            for s in [-1, 1] {
                let at = mid.add(right_of(fwd).scale(s * (room.half_width + ROOM_SIDE / 2)));
                b[chunk(mid)].push(
                    Mesh::Cube,
                    &block(
                        v3(at.x, (top + bottom) / 2, at.z),
                        fwd,
                        v3(ROOM_SIDE, top - bottom, len),
                    ),
                    ROOF,
                    0,
                );
            }
        }
        if room.kind == RoomKind::Tunnel {
            mine_dressing(b, chunk, c, room, dice);
        }
    }
}

/// The mine tunnel's gear: timber sets every 9 m (never at a vine's
/// anchor), gold and fossil digs on its walls, picks and ore buckets at
/// their feet. Dressing: none of it is solid.
fn mine_dressing(
    b: &mut [Batch],
    chunk: &dyn Fn(V3) -> usize,
    c: &Course,
    room: &crate::ravine::Room,
    dice: &mut Dice,
) {
    let run = room.to.sub(room.from).flat();
    let len = run.len();
    let fwd = run.norm();
    let right = right_of(fwd);
    let half = crate::ravine::layout::HALF;
    let mut a = int(4);
    let mut k = 0u32;
    while a < len - int(2) {
        let p = room.from.add(fwd.scale(a));
        let p = v3(p.x, ground(p.x, p.z), p.z);
        let clear = c.vines.iter().all(|v| {
            v.anchor.flat().dist(p.flat()) > int(3) || v.anchor.y < p.y + room.height - int(2)
        });
        let bk = &mut b[chunk(p)];
        if clear {
            let f = Affine::translate(p).then(&Affine::rot_y(yaw_of(fwd)));
            m::mine_support(bk, &f, 2 * half - int(1), room.height - cm(60), k);
        }
        // a dig on one wall, gear at its foot
        let side = if k.is_multiple_of(2) { 1 } else { -1 };
        let wall = p.add(right.scale(side * (half - cm(5))));
        let out = right.scale(-side);
        let f =
            Affine::translate(v3(wall.x, p.y + int(1), wall.z)).then(&Affine::looking(out, V3::UP));
        m::dig_marks(bk, &f, int(4), int(3), 0, k);
        if k.is_multiple_of(3) {
            let f = Affine::translate(v3(wall.x, p.y + int(5), wall.z))
                .then(&Affine::looking(out, V3::UP))
                .then(&Affine::scale(v3(ONE * 3 / 2, ONE * 3 / 2, ONE)));
            m::fossil_inlay(bk, &f, m::Fossil::Ribs, k);
        }
        let foot = wall.add(out.scale(int(2)));
        let foot = v3(foot.x, ground(foot.x, foot.z), foot.z);
        match dice.below(3) {
            0 => m::pickaxe(
                bk,
                &Affine::translate(foot).then(&Affine::rot_y(yaw_of(out))),
                deg(25),
            ),
            1 => m::ore_bucket(bk, &Affine::translate(foot), k),
            _ => {
                m::ore_bucket(bk, &Affine::translate(foot), k);
                m::pickaxe(
                    bk,
                    &Affine::translate(foot.add(fwd.scale(int(1))))
                        .then(&Affine::rot_y(yaw_of(out))),
                    deg(15),
                );
            }
        }
        a += int(9);
        k += 1;
    }
}

fn decks(b: &mut [Batch], chunk: &dyn Fn(V3) -> usize, r: &RavineLayout) {
    for (k, d) in r.decks.iter().enumerate() {
        let run = d.to.sub(d.from);
        let len = run.len();
        if len <= 0 {
            continue;
        }
        let f = Affine::looking(run, V3::UP).at(d.from);
        let bk = &mut b[chunk(d.from.lerp(d.to, ONE / 2))];
        match d.kind {
            DeckKind::Bridge => m::rope_bridge_level(bk, &f, len, 2 * d.half_width, k as u32),
            DeckKind::Log => m::log_span(bk, &f, len, d.half_width, k as u32),
        }
    }
}

/// What each vine hangs from, drawn: a juniper reaching out of its wall
/// for a ledge vine, an iron peg under a roof, a knot under a deck.
fn hangers(
    b: &mut [Batch],
    chunk: &dyn Fn(V3) -> usize,
    c: &Course,
    r: &RavineLayout,
    dice: &mut Dice,
) {
    for (i, h) in r.hangers.iter().enumerate() {
        let a = c.vines.get(i).map(|v| v.anchor).unwrap_or(h.at);
        let bk = &mut b[chunk(a)];
        match h.from {
            HangFrom::Ledge { wall } => {
                let Some(w) = r.walls.get(wall) else { continue };
                let root = nearest_face(w, a);
                let root = v3(root.x, a.y - int(dice.range(2, 5)), root.z);
                let tip = a
                    .add(a.sub(root).flat().norm().scale(cm(120)))
                    .add(v3(0, cm(40), 0));
                bk.push(
                    Mesh::Taper,
                    &crate::render::rod(root, tip, cm(70)),
                    JUNIPER,
                    0,
                );
                // a twig and a tuft of needles at the tip
                let mid = root.lerp(tip, ONE * 2 / 3);
                bk.push(
                    Mesh::Taper,
                    &crate::render::rod(
                        mid,
                        mid.add(v3(
                            cm(dice.range(-150, 150)),
                            int(2),
                            cm(dice.range(-150, 150)),
                        )),
                        cm(25),
                    ),
                    JUNIPER,
                    0,
                );
                bk.push(
                    Mesh::Blob,
                    &placed(tip.add(v3(0, cm(50), 0)), 0, v3(int(2), cm(110), int(2))),
                    NEEDLES,
                    0,
                );
            }
            HangFrom::Roof { .. } => {
                bk.push(
                    Mesh::Hex,
                    &placed(a.add(v3(0, -cm(15), 0)), 0, v3(cm(40), cm(30), cm(40))),
                    PEG,
                    0,
                );
            }
            HangFrom::Deck { .. } => {
                bk.push(
                    Mesh::Ico,
                    &placed(a, 0, v3(cm(45), cm(35), cm(45))),
                    m::ROPE,
                    0,
                );
            }
        }
    }
}

/// The point of wall `w`'s face nearest `p` (flat), at `p`'s height.
fn nearest_face(w: &crate::ravine::Wall, p: V3) -> V3 {
    let mut best = (i64::MAX, p);
    for k in w.points.windows(2) {
        let (a, z) = (k[0].at, k[1].at);
        let ab = z.sub(a).flat();
        let l2 = ab.dot64(ab).max(1);
        let t = (p.sub(a).flat().dot64(ab) as i128 * ONE as i128 / l2 as i128).clamp(0, ONE as i128)
            as i32;
        let q = a.lerp(z, t);
        let d = p.sub(q).flat().dot64(p.sub(q).flat());
        if d < best.0 {
            best = (d, q);
        }
    }
    let q = best.1;
    v3(q.x, p.y, q.z)
}

fn tracks(b: &mut [Batch], chunk: &dyn Fn(V3) -> usize, r: &RavineLayout) {
    for (k, t) in r.tracks.iter().enumerate() {
        for (j, w) in t.points.windows(2).enumerate() {
            let (a, z) = (
                w[0].sub(v3(0, m::RAIL_TOP, 0)),
                w[1].sub(v3(0, m::RAIL_TOP, 0)),
            );
            let mid = a.lerp(z, ONE / 2);
            let floor = ground(mid.x, mid.z);
            m::mine_track(
                &mut b[chunk(mid)],
                &Affine::IDENTITY,
                a,
                z,
                Some(floor),
                (k * 16 + j) as u32,
            );
        }
        for (j, h) in [t.start, t.end].iter().enumerate() {
            let floor = h.at.sub(v3(0, m::RAIL_TOP, 0));
            let f = Affine::looking(h.out.flat(), V3::UP).at(floor);
            m::wall_hole(
                &mut b[chunk(h.at)],
                &f,
                2 * h.half_width,
                h.height,
                (k * 2 + j) as u32,
            );
        }
    }
}

fn rocks(b: &mut [Batch], chunk: &dyn Fn(V3) -> usize, r: &RavineLayout, dice: &mut Dice) {
    for k in &r.rocks {
        // the Rock mesh lies inside the unit ball, pulled in by up to a
        // quarter: drawn a little over the solid post so it reads as big
        let d = k.radius * 2 + k.radius / 5;
        b[chunk(k.at)].push(
            Mesh::Rock,
            &placed(
                k.at.add(v3(0, k.height / 2, 0)),
                dice.range(0, 359) * TURN / 360,
                v3(d, k.height + cm(40), d),
            ),
            if dice.below(2) == 0 {
                m::SANDSTONE
            } else {
                m::SANDSTONE_DARK
            },
            0,
        );
        b[chunk(k.at)].push(
            Mesh::Blob,
            &placed(
                k.at.add(v3(0, k.height, 0)),
                0,
                v3(k.radius, cm(40), k.radius),
            ),
            MOSS,
            0,
        );
    }
}

/// The mesa's still set: the sphinx and the tents (the campfire burns in
/// `dynamic`), and far buttes round the valley.
fn finish(b: &mut Batch, r: &RavineLayout) {
    let f = &r.finish;
    m::sphinx(b, &sphinx_frame(r));
    for (k, &(at, yaw)) in f.tents.iter().enumerate() {
        let c = if k % 2 == 0 {
            m::TENT_CANVAS
        } else {
            m::TENT_RED
        };
        let big = crate::ravine::layout::TENT_SCALE;
        m::tent(
            b,
            &Affine::translate(at)
                .then(&Affine::rot_y(yaw))
                .then(&Affine::scale(v3(big, big, big))),
            c,
            k as u32,
        );
    }
    // the mesa's buttes: flat-topped towers of banded rock far round the
    // valley, past the canyon's end
    let mut dice = Dice(ART_DICE ^ 0x4D45_5341);
    for k in 0..9 {
        let a = f.sphinx_yaw + TURN / 2 + (k - 4) * deg(26) + deg(dice.range(-6, 6));
        let d = int(dice.range(200, 300));
        let at = f.sphinx.add(crate::geom::facing(a, 0).scale(d));
        let at = v3(at.x, ground(at.x, at.z) - int(2), at.z);
        let w = int(dice.range(30, 70));
        let h = int(dice.range(30, 60));
        for (j, (share, col)) in [
            (ONE, ROCK_FOOT),
            (ONE * 3 / 4, ROCK_MID[0]),
            (ONE / 2, ROCK_BAND),
        ]
        .into_iter()
        .enumerate()
        {
            let hh = fx::mul(h, share);
            b.push(
                Mesh::Cube,
                &placed(
                    at.add(v3(0, hh / 2 + j as i32 * cm(5), 0)),
                    a,
                    v3(w - j as i32 * int(4), hh, w - j as i32 * int(4)),
                ),
                col,
                0,
            );
        }
        b.push(
            Mesh::Rock,
            &placed(at, a, v3(w * 3 / 2, int(14), w * 3 / 2)),
            ROCK_FOOT,
            0,
        );
    }
}

/// The canyon floor: rows across every `WALL_STEP` m from the back wall to
/// the far end, from the left wall's face to the right's (round the upper
/// passage and out onto the mesa), each a little under the rock.
fn floor(c: &Course, r: &RavineLayout, n: usize) -> (Vec<i32>, Vec<(usize, u32, u32)>) {
    use crate::mesh::VERTEX_VALUES;
    let len = c.length / ONE;
    let step = crate::ravine::layout::WALL_STEP;
    let first = -crate::ravine::layout::START_BACK / ONE;
    // the right wall (the longest on that side) has a point every step
    let right = r
        .walls
        .iter()
        .filter(|w| w.side == Side::Right)
        .max_by_key(|w| w.points.len());
    const COLS: i32 = 8;
    let row = |k: usize| -> Option<[V3; COLS as usize + 1]> {
        let s = first + k as i32 * step;
        let p = c.point(int(s));
        let rt = right_of(c.heading_at(int(s)));
        let left = crate::ravine::layout::open_half(len, s) + FLOOR_UNDER;
        let rw = right
            .and_then(|w| w.points.get(k))
            .map(|q| q.at.sub(p).flat().dot(rt))
            .unwrap_or(crate::ravine::layout::HALF)
            + FLOOR_UNDER;
        let mut out = [V3::ZERO; COLS as usize + 1];
        for (j, o) in out.iter_mut().enumerate() {
            let a = -left + fx::mul(left + rw, ONE * j as i32 / COLS);
            let q = p.add(rt.scale(a));
            *o = v3(q.x, ground(q.x, q.z), q.z);
        }
        Some(out)
    };
    let rows = ((len + OUTRO / ONE - first) / step) as usize + 1;
    let mut o = Vec::new();
    let mut ranges: Vec<(usize, u32, u32)> = Vec::new();
    let verts = |o: &Vec<i32>| (o.len() / VERTEX_VALUES) as u32;
    let chunk_along = |s: i32| {
        if s > c.length {
            n - 1
        } else {
            ((s.max(0) / STEP) as usize / CHUNK).min(n - 2)
        }
    };
    for k in 0..rows.saturating_sub(1) {
        let (Some(a), Some(z)) = (row(k), row(k + 1)) else {
            continue;
        };
        let s = int(first + k as i32 * step);
        let ch = chunk_along(s);
        match ranges.last() {
            Some((x, _, _)) if *x == ch => {}
            _ => ranges.push((ch, verts(&o), 0)),
        }
        for j in 0..COLS as usize {
            for (h, tri) in [[a[j], z[j], z[j + 1]], [a[j], z[j + 1], a[j + 1]]]
                .into_iter()
                .enumerate()
            {
                let edge = j == 0 || j == COLS as usize - 1;
                let base = if edge { FLOOR_DARK } else { FLOOR };
                let wobble = 92 + ((k * 7 + j * 13 + h * 5) % 11) as i32;
                let t = [0, 1, 2].map(|q| base[q] * wobble / 100);
                push_ground(&mut o, tri, t);
            }
        }
        if let Some(last) = ranges.last_mut() {
            last.2 = verts(&o) - last.1;
        }
    }
    // the mesa top round the finish, out to the far buttes and past them,
    // seen over the low rims (flat, a little under the floor's lowest so
    // it never shows through it); in the last chunk with the finish
    let mid = c.point(c.length + int(60));
    let mid = v3(mid.x, MESA_TOP, mid.z);
    let rings = [0, 140, 300, 520, 800].map(int);
    const SPOKES: i32 = 24;
    let at = |ring: usize, k: i32| {
        let a = k * TURN / SPOKES;
        mid.add(v3(
            fx::mul(fx::sin(a), rings[ring]),
            0,
            fx::mul(fx::cos(a), rings[ring]),
        ))
    };
    match ranges.last() {
        Some((x, _, _)) if *x == n - 1 => {}
        _ => ranges.push((n - 1, verts(&o), 0)),
    }
    for ring in 0..rings.len() - 1 {
        for k in 0..SPOKES {
            let shade = 88 + ((ring as i32 * 5 + k * 7) % 9);
            let t = [0, 1, 2].map(|q| FLOOR_DARK[q] * shade / 100);
            push_ground(
                &mut o,
                [at(ring, k), at(ring + 1, k), at(ring + 1, k + 1)],
                t,
            );
            if ring > 0 {
                push_ground(
                    &mut o,
                    [at(ring, k), at(ring + 1, k + 1), at(ring, k + 1)],
                    t,
                );
            }
        }
    }
    if let Some(last) = ranges.last_mut() {
        last.2 = verts(&o) - last.1;
    }
    (o, ranges)
}

/// The mesa top's height outside the canyon (under the floor's lowest,
/// `ground`'s 0.9 m dips).
const MESA_TOP: i32 = -cm(120);

fn hue(c: LizardColour) -> m::LizardHue {
    match c {
        LizardColour::Red => m::LizardHue::Red,
        LizardColour::Yellow => m::LizardHue::Yellow,
        LizardColour::Blue => m::LizardHue::Blue,
    }
}

/// How far a snapped snake's head half has fallen `since` ticks after the
/// snap: falling as a body does, until it lies on the floor.
pub fn snake_fall(since: u32, most: i32) -> i32 {
    let t = since as i64 * ONE as i64 / crate::rules::TICKS_PER_SECOND as i64;
    let d = (crate::rules::GRAVITY as i64 * t * t / 2 / ONE as i64 / ONE as i64) as i32;
    d.clamp(0, most.max(0))
}

/// What moves, drawn each frame near the eye: the snake vines (snapped as
/// `Hanger.snapped_at` says), the lizards, the carts, the glows' models
/// and the campfire. `t` is the frame's ticks (sways, flames), `tick` the
/// race's (where the movers are); `vivid` the colourblind mode.
pub fn dynamic(
    b: &mut Batch,
    c: &Course,
    r: &RavineLayout,
    eye: V3,
    t: u32,
    tick: u32,
    vivid: bool,
) {
    let near = |p: V3, d: i32| !p.flat().sub(eye.flat()).longer_than(d);
    let mut snakes = Batch::new();
    for (i, h) in r.hangers.iter().enumerate() {
        if !h.snake {
            continue;
        }
        let Some(v) = c.vines.get(i) else { continue };
        let a = c.anchor(i, tick);
        if !near(a, MOVERS_SEEN) {
            continue;
        }
        let f = Affine::translate(a).then(&Affine::rot_y(i as i32 * deg(47)));
        let k = i as u32;
        let head = match h
            .snapped_at
            .filter(|&at| tick >= at && tick < at.saturating_add(SNAKE_REGROW))
        {
            Some(at) => {
                let most = a.y - v.length - ground(a.x, a.z) - cm(30);
                let fall = snake_fall(tick - at, most);
                m::snake_snapped(&mut snakes, &f, v.length, fall, t, k);
                None
            }
            None => {
                let pts = m::snake_points(v.length, t, k);
                m::snake_along(&mut snakes, &f, &pts, t, k);
                pts.last().map(|&p| f.apply(p))
            }
        };
        // a warning glow round the head and the rattle, so a snake reads
        // at a glance in every mode (RK4)
        let glow = if vivid { SNAKE_GLOW_VIVID } else { SNAKE_GLOW };
        if let Some(p) = head {
            snakes.push_halo(
                Mesh::Sphere,
                &placed(p, 0, v3(int(2), int(2), int(2))),
                glow,
                ONE / 2,
            );
        }
        snakes.push_halo(
            Mesh::Sphere,
            &placed(a.sub(v3(0, cm(60), 0)), 0, v3(cm(150), cm(150), cm(150))),
            glow,
            ONE / 3,
        );
    }
    if vivid {
        // the colourblind mode: the bands go black and white, a stripe
        // that reads whatever the hue
        snakes.restyle(|col, g| {
            let l = (col[0] * 30 + col[1] * 59 + col[2] * 11) / 100;
            if g > 0 {
                (col, g)
            } else if l > ONE / 2 {
                ([ONE; 3], g)
            } else {
                ([ONE / 20; 3], g)
            }
        });
    }
    b.append(snakes);
    for (i, l) in r.lizards.iter().enumerate() {
        let (at, dir) = r.lizard_at(i, tick);
        if !near(at, MOVERS_SEEN) {
            continue;
        }
        let f = m::on_surface(at, l.out, dir).then(&Affine::scale(v3(
            ONE * 3 / 2,
            ONE * 3 / 2,
            ONE * 3 / 2,
        )));
        if vivid {
            // the colourblind mode: the lizard (a bouncer) goes pale and
            // wears a white glow, so it reads off the greyed wall by
            // brightness whatever its hue (step 13, RA-13)
            let mut own = Batch::new();
            m::lizard(&mut own, &f, m::lizard_phase(tick, i as u32), hue(l.colour));
            own.restyle(|col, g| (col.map(|c| c / 2 + ONE / 2), g));
            b.append(own);
            b.push_halo(
                Mesh::Sphere,
                &placed(at.add(l.out.scale(cm(60))), 0, v3(int(3), int(3), int(3))),
                LIZARD_GLOW_VIVID,
                ONE / 3,
            );
        } else {
            m::lizard(b, &f, m::lizard_phase(tick, i as u32), hue(l.colour));
        }
    }
    for (i, _) in r.carts.iter().enumerate() {
        let (at, yaw) = r.cart_at(i, tick);
        if !near(at, MOVERS_SEEN) {
            continue;
        }
        let f = Affine::translate(at).then(&Affine::rot_y(yaw));
        m::mine_cart(b, &f, tick, i as u32, i % 2 == 0);
    }
    for (i, g) in r.glows.iter().enumerate() {
        if !near(g.at, GLOWS_SEEN) {
            continue;
        }
        let k = i as u32;
        match g.kind {
            GlowKind::Torch | GlowKind::Lantern => {
                // on a bracket out of the nearer wall
                let p = c.nearest_anywhere(g.at);
                let side = right_of(c.heading[p]);
                let off = g.at.sub(c.path[p]).flat().dot(side);
                let out = if off > 0 { side.neg() } else { side };
                let wall = g.at.sub(out.scale(cm(60)));
                if g.kind == GlowKind::Torch {
                    let f = Affine::translate(wall)
                        .then(&Affine::looking(out, V3::UP))
                        .then(&Affine::rot_x(-deg(25)));
                    m::torch(b, &f, t, k);
                } else {
                    let hook = g.at.add(v3(0, cm(20), 0));
                    b.push(
                        Mesh::Cube,
                        &crate::render::rod(wall.add(v3(0, cm(20), 0)), hook, cm(10)),
                        PEG,
                        0,
                    );
                    m::lantern(b, &Affine::translate(hook), t, k);
                }
            }
            GlowKind::Mushroom => {
                let at = v3(g.at.x, ground(g.at.x, g.at.z), g.at.z);
                m::mushrooms(
                    b,
                    &Affine::translate(at).then(&Affine::rot_y(i as i32 * deg(61))),
                    t,
                    k,
                );
            }
            GlowKind::Fireflies => {
                let r3 = int(3);
                m::fireflies(
                    b,
                    &Affine::translate(g.at.sub(v3(0, r3 * 2 / 3, 0))),
                    r3,
                    t,
                    k,
                );
            }
        }
    }
    if let Some(s) = r.skull.as_ref().filter(|s| near(s.at, GLOWS_SEEN)) {
        skull_lights(b, s, t);
    }
    if near(r.finish.campfire, MOVERS_SEEN) {
        let at = r.finish.campfire;
        let toward = yaw_of(r.finish.sphinx.sub(at));
        let big = crate::ravine::layout::CAMPFIRE_SCALE;
        let size = Affine::scale(v3(big, big, big));
        let frame = Affine::translate(at)
            .then(&Affine::rot_y(toward))
            .then(&size);
        m::campfire_fire(b, &frame, t, 1);
        // the layout's sticks, each leaning in over the fire to 1.2 m up
        // (the model's lean, at its size)
        let upright = Affine::translate(at).then(&size);
        for (i, &foot) in r.finish.sticks.iter().enumerate() {
            let out = foot.sub(at).flat();
            let foot = v3(fx::div(out.x, big), 0, fx::div(out.z, big));
            let tip = foot.scale(fx::div(cm(55), foot.len().max(1)));
            let end = v3(tip.x, cm(120), tip.z);
            m::roasting_stick(b, &upright, foot, end, i as i32);
        }
    }
}

/// The torches round the T-rex skull's mouth (the place is dark: the fin's
/// hairpin, far from the moon): `SKULL_TORCHES` big torches on the fin's
/// face either side of the skull, each with its pool of light, and a warm
/// glow deep in the mouth and the hole behind it, so the teeth and the gap
/// read from the approach (`ravinescape::skull_look` lights the bone).
pub fn skull_lights(b: &mut Batch, s: &crate::ravine::Skull, t: u32) {
    let out = crate::geom::facing(s.yaw, 0).neg();
    let across = right_of(out);
    for (k, &(side, up)) in SKULL_TORCHES.iter().enumerate() {
        let wall = s.at.add(across.scale(side)).add(v3(0, up, 0));
        let f = Affine::translate(wall.add(out.scale(cm(40))))
            .then(&Affine::looking(out, V3::UP))
            .then(&Affine::rot_x(-deg(25)))
            .then(&Affine::scale(v3(ONE * 2, ONE * 2, ONE * 2)));
        m::torch(b, &f, t, 900 + k as u32);
        b.push_halo(
            Mesh::Sphere,
            &placed(
                wall.add(out.scale(int(2))).add(v3(0, int(3), 0)),
                0,
                v3(int(8), int(8), int(8)),
            ),
            SKULL_FIRE,
            ONE / 4,
        );
    }
    for (deep, size, strength) in [(int(1), int(6), ONE / 3), (-int(10), int(5), ONE / 3)] {
        b.push_halo(
            Mesh::Sphere,
            &placed(s.at.add(out.scale(deep)), 0, v3(size, size * 3 / 4, size)),
            SKULL_FIRE,
            strength,
        );
    }
}

/// The skull's torches: across the fin's face from the mouth's middle
/// (past the skull's 6.5 m half width) and up from it.
pub const SKULL_TORCHES: [(i32, i32); 4] = [
    (-int(9), int(1)),
    (int(9), int(1)),
    (-int(10), -int(12)),
    (int(10), -int(12)),
];
/// The torchlight's colour round the skull.
const SKULL_FIRE: Rgb = rgb(255, 160, 70);

#[cfg(test)]
mod tests;
