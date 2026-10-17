//! Mape's Tower's still scenery, placed from the layout (step 8,
//! SECOND-ORDER TA and TA2 rows). Everything is read from `TowerLayout`
//! and the `Course` (the towers, bridges, doorway, windows, balconies,
//! clouds, portal, tube, tunnel, pads, hangers, vines), never from fixed
//! places, so a retuned course carries its art with it.
//!
//! Chunks are cells of `CELL` (40 m across, 15 m high); each knows the
//! heights it spans (`Scenery::bound_heights`), so on a climb a chunk far
//! above or below the eye is far off (TA2). Both towers are drawn twice:
//! each section in stone courses (`Detail::Near`) in the cell of its
//! height, and in one always-drawn chunk as plain panels (`Detail::Far`)
//! a little inside the wall, so from afar the whole tower stands and close
//! up the near one covers it. Tower 1 is hollow: its sections are
//! `hollow_section`s, the doorway's one drawn the doorway's height so the
//! opening is the contract's, windows cut where the contract's `Window`s
//! are (at most two a section), inner balconies and ledges inside.
//!
//! The void (past the portal's plane) is its own place: the tube in 36 m
//! segments with its portholes and the creatures looking in, the pads
//! drawn where the physics' `PadShape`s cover (`pad_frame`, checked by
//! `the_drawn_pads_cover_what_the_physics_covers`), the tunnel's mouth
//! with the sling bar, the tunnel and the banana sea; drawn only from
//! past the plane, as the climb's parts only from before it (`Chunk.side`).
//! The portal itself turns and breathes each frame (`towerscape::dynamic`,
//! `portal_frame`), not here.

use crate::course::Course;
use crate::fx::{cm, int, mul, ONE, TURN};
use crate::geom::{right_of, v3, yaw_of, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{placed, rgb, Batch, Rgb};
use crate::scene::{self, Scenery, VineLook};
use crate::tower::{Hanger, PadShape, TowerLayout, TubePad, TUBE_PAD_THICK};
use crate::towermodels as m;
use crate::towermodels::{MagePose, Spell};

/// How far (in 3D) a chunk is drawn from the eye.
pub const REACH: i32 = int(90);

/// The tube's wall stops this far under its top (`tower::stone::TUBE_RIM`,
/// not exported): the line comes in over it, so the wall is drawn from
/// there down.
const TUBE_RIM: i32 = int(10);
/// The tube and the tunnel are drawn in segments this long (three of the
/// model's bands), each in its own cell.
const SEGMENT: i32 = int(36);
/// Portholes down the tube: one every this far, the first this far down.
const PORTHOLE_EVERY: i32 = int(30);
const PORTHOLE_FIRST: i32 = int(18);
/// The void wall's colours (`towermodels::tube`'s, which are private): the
/// mouth segment's panels are drawn here so the tunnel's opening is open.
const VOID_WALL: [Rgb; 3] = [rgb(40, 30, 72), rgb(56, 44, 94), rgb(26, 20, 50)];
const VOID_RIB: Rgb = rgb(60, 150, 130);
/// The pads' film and rim (`towermodels::tube`'s), for the ring pads
/// composed here (the model's ring has its hole in the middle; the
/// physics' is offset).
const FILM: Rgb = rgb(140, 236, 216);
const RING: Rgb = rgb(110, 255, 200);
const FILM_DEEP: Rgb = rgb(70, 170, 160);

/// The spell vine `i` is cast with: a balcony's mage by the balcony, a
/// dragon rider, a cloud's rune and the start's by the vine, the sling's
/// the deep spells.
pub fn spell_of(t: &TowerLayout, i: usize) -> Spell {
    match t.hangers.get(i) {
        Some(Hanger::Mage { balcony }) => Spell::nth(*balcony as u32),
        Some(Hanger::Sling) => Spell::DEEP[i % 2],
        Some(Hanger::Cloud { band }) => Spell::nth(*band as u32 + i as u32 * 2),
        _ => Spell::nth(i as u32),
    }
}

/// Vine `i`'s look on this map: a spell beam in its caster's colour with
/// a soft glow (`beam_look`, `BEAM_HALO`); the in-reach and colourblind
/// looks stay the shared ones (the caller keeps those).
pub fn beam_vine(t: &TowerLayout, i: usize, look: VineLook) -> VineLook {
    let (colour, glow) = m::beam_look(spell_of(t, i));
    VineLook {
        colour,
        glow,
        halo: look.width * m::BEAM_HALO,
        ..look
    }
}

/// A frame at `at` turned `yaw`.
fn at(p: V3, yaw: i32) -> Affine {
    m::pose(p, yaw, ONE)
}

/// Where a model drawn by `draw` at the origin, turned `yaw`, puts its
/// spell's start: the model is then moved so that point lands on `tip`.
fn tipped(yaw: i32, tip: V3, draw: impl Fn(&mut Batch, &Affine) -> V3) -> Affine {
    let mut scratch = Batch::new();
    let off = draw(&mut scratch, &at(V3::ZERO, yaw));
    at(tip.sub(off), yaw)
}

/// Whether `p` is within `d` of any of `seen` (and if not, remembers it).
fn fresh(seen: &mut Vec<V3>, p: V3, d: i32) -> bool {
    if seen.iter().any(|q| !q.sub(p).longer_than(d)) {
        return false;
    }
    seen.push(p);
    true
}

/// Which of an octagonal tower's eight faces (face 0 facing +z, then
/// round by an eighth of a turn) faces along `dir`.
fn face_of(dir: V3) -> usize {
    let yaw = yaw_of(dir).wrapping_add(TURN / 16).rem_euclid(TURN);
    (yaw / (TURN / 8)) as usize % 8
}

/// Shapes per part of the scenery (for `examples/scenery_cost.rs` and the
/// budget test).
pub fn parts(c: &Course) -> Vec<(&'static str, usize)> {
    let Some(t) = &c.tower else {
        return Vec::new();
    };
    let mut out = Vec::new();
    build(c, t, &mut |part, _p, b| {
        let n = b.count();
        match out.iter_mut().find(|(k, _)| *k == part) {
            Some((_, m)) => *m += n,
            None => out.push((part, n)),
        }
    });
    out
}

/// What scenery places: each part's name, where it stands (its chunk;
/// `None` always drawn) and its shapes.
type Sink<'a> = dyn FnMut(&'static str, Option<V3>, Batch) + 'a;

/// The scenery's cells: a part goes in the cell of where it stands, so
/// on a climb the cells far above and below the eye are not drawn.
const CELL: (i32, i32) = (int(40), int(15));

/// The parts that belong to the void: drawn only from past the portal's
/// plane (the climb's only from before it), as the sky is.
const VOID_PARTS: [&str; 8] = [
    "tube",
    "void pads",
    "creatures",
    "tunnel mouth",
    "tunnel",
    "zip",
    "banana sea",
    "void platforms",
];

/// Mape's Tower's scenery: the parts in cells of `CELL` (each knowing its
/// box and heights, drawn within `REACH`), and per side one always-drawn
/// chunk (the towers' far panels; the zip and the banana sea). The climb
/// and the void share their space, so each chunk is drawn only from its
/// own side of the portal's plane (`Chunk.side`; the plane is taken as
/// the tear's x: the stand-in's `through` is +x).
pub fn scenery(c: &Course, t: &TowerLayout) -> Scenery {
    type Key = (bool, i32, i32, i32);
    let mut cells: std::collections::BTreeMap<Key, Batch> = Default::default();
    build(c, t, &mut |part, p, b| {
        let past = VOID_PARTS.contains(&part);
        let key = match p {
            Some(p) => (
                past,
                p.x.div_euclid(CELL.0),
                p.y.div_euclid(CELL.1),
                p.z.div_euclid(CELL.0),
            ),
            None => (past, i32::MAX, 0, 0),
        };
        // (`Batch::new`, not `default`: a default batch has no mesh lists)
        #[allow(clippy::unwrap_or_default)]
        cells.entry(key).or_insert_with(Batch::new).append(b);
    });
    let mut values = Vec::new();
    let mut chunks = Vec::new();
    let far = i32::MAX / 4;
    for ((past, x, y, z), b) in cells {
        let draws = b.finish_into(crate::render::STATIC_BUFFER, &mut values);
        let always = x == i32::MAX;
        chunks.push(scene::Chunk {
            centre: if always {
                t.towers[0].centre
            } else {
                v3(
                    x * CELL.0 + CELL.0 / 2,
                    y * CELL.1 + CELL.1 / 2,
                    z * CELL.0 + CELL.0 / 2,
                )
            },
            draws,
            bounds: always.then_some([-far, -far, far, far]),
            heights: always.then_some([-far, far]),
            side: Some((t.portal.tear.x, past)),
        });
    }
    let mut s = Scenery {
        values,
        terrain: Vec::new(),
        chunks,
        reach: REACH,
    };
    let fixed: Vec<bool> = s.chunks.iter().map(|ch| ch.bounds.is_some()).collect();
    let keep: Vec<_> = s.chunks.iter().map(|ch| (ch.bounds, ch.heights)).collect();
    s.bound_chunks();
    s.bound_heights();
    for (k, ch) in s.chunks.iter_mut().enumerate() {
        if fixed[k] {
            (ch.bounds, ch.heights) = keep[k];
        }
    }
    s
}

/// Twilight's glow (TS5): what already glows (windows, spell tips, runes)
/// glows half as much again, so it shines in the dusk; under the sky's
/// marks (`SKY_GLOW`).
fn shine(b: &mut Batch) {
    b.restyle(|c, g| {
        if g > 0 && g < ONE * 3 / 2 {
            (c, (g * 3 / 2).min(ONE * 3 / 2))
        } else {
            (c, g)
        }
    });
}

/// The spans tower 1's wall is drawn in, bottom to top: sections of about
/// `SECTION_H` up to the doorway's sill, one the doorway's height (plus
/// the model's 2 m lintel) holding the doorway, then sections on to the
/// top. Each is (bottom, top, holds the doorway).
fn tower1_spans(t: &TowerLayout) -> Vec<(i32, i32, bool)> {
    let t1 = &t.towers[0];
    let sill = t.doorway.at.y.clamp(t1.centre.y, t1.top() - int(1));
    let door_h = (t.doorway.height + int(2)).min(t1.top() - sill);
    let mut spans = Vec::new();
    let stack = |spans: &mut Vec<(i32, i32, bool)>, y0: i32, y1: i32| {
        if y1 - y0 < int(1) {
            return;
        }
        let n = ((y1 - y0) / m::SECTION_H).max(1);
        let sh = (y1 - y0) / n;
        for s in 0..n {
            let top = if s == n - 1 { y1 } else { y0 + (s + 1) * sh };
            spans.push((y0 + s * sh, top, false));
        }
    };
    stack(&mut spans, t1.centre.y, sill);
    spans.push((sill, sill + door_h, true));
    stack(&mut spans, sill + door_h, t1.top());
    spans
}

/// What each face of the span from `y0` to `y1` has cut through it: the
/// doorway on its face when the span holds it; windows where the
/// contract's windows fall in the span, at most two faces (the two whose
/// windows are nearest the span's middle, where the model cuts its hole).
fn tower1_cuts(t: &TowerLayout, y0: i32, y1: i32, door: bool) -> [m::Cut; 8] {
    let mut cuts = [m::Cut::Wall; 8];
    let door_face = face_of(t.doorway.out);
    if door {
        cuts[door_face] = m::Cut::Door;
    }
    let mid = (y0 + y1) / 2;
    let mut faces: Vec<(i32, usize)> = t
        .windows
        .iter()
        .filter(|w| {
            let y = w.at.y + w.height / 2;
            y >= y0 && y < y1
        })
        .map(|w| ((w.at.y + w.height / 2 - mid).abs(), face_of(w.into.neg())))
        .filter(|(_, f)| !(door && *f == door_face))
        .collect();
    faces.sort();
    let mut cut = 0;
    for (_, f) in faces {
        if cuts[f] == m::Cut::Wall && cut < 2 {
            cuts[f] = m::Cut::Window;
            cut += 1;
        }
    }
    cuts
}

/// The frame a void pad is drawn in and the model's shape and radius for
/// it, so the model covers what the physics' `PadShape` covers: a slab
/// is the model's half disc turned so its covered half lies against
/// `open`, shifted `edge` along it and drawn `edge` bigger so its arc
/// still reaches the wall (the extra is in the wall); quarters are the
/// model's turned so its covered quadrants lie round ±`axis`; a ring is
/// composed here (`ring_pad`), so no model shape. The frame is at the
/// tube's axis at the pad's middle height.
fn pad_frame(tube_axis: V3, r: i32, pad: &TubePad) -> (Affine, Option<(m::PadShape, i32)>) {
    let mid = v3(tube_axis.x, pad.y - TUBE_PAD_THICK / 2, tube_axis.z);
    match pad.shape {
        PadShape::Slab { open, edge } => (
            at(mid.add(open.scale(edge)), yaw_of(right_of(open).neg())),
            Some((m::PadShape::Half, r + edge)),
        ),
        PadShape::Quarters { axis } => (
            at(mid, yaw_of(axis) - TURN / 8),
            Some((m::PadShape::Quarters, r)),
        ),
        PadShape::Ring { .. } => (at(mid, 0), None),
    }
}

/// Whether the drawing of `pad` (as `pad_frame` places it) covers the
/// plan point `rel` from the tube's axis, for the test against the
/// physics.
#[cfg(test)]
fn drawn_covers(tube_axis: V3, r: i32, pad: &TubePad, rel: V3) -> bool {
    let (f, shape) = pad_frame(tube_axis, r, pad);
    match shape {
        None => {
            let PadShape::Ring { at, hole } = pad.shape else {
                unreachable!()
            };
            !rel.longer_than(r) && rel.sub(at).longer_than(hole)
        }
        Some((shape, rr)) => {
            // the point in the pad's frame: the frame's x and z axes
            let p = tube_axis.add(rel);
            let d = p.sub(f.t);
            let ax = f.apply_dir(v3(ONE, 0, 0));
            let az = f.apply_dir(v3(0, 0, ONE));
            m::pad_covers(shape, rr, d.dot(ax), d.dot(az))
        }
    }
}

/// A ring pad across a tube of radius `r` with its hole of radius `hole`
/// at `hole_at` (in plan, from the axis), in frame `f` (at the axis at
/// the pad's middle): strips of film from the hole's rim to the wall, a
/// glowing rim round both, and a soft halo. About 45 shapes.
fn ring_pad(b: &mut Batch, f: &Affine, r: i32, hole_at: V3, hole: i32, k: u32) {
    let thick = cm(50);
    let n = 24;
    for i in 0..n {
        let a = TURN * i / n;
        let dir = v3(crate::fx::sin(a), 0, crate::fx::cos(a));
        let p = hole_at.add(dir.scale(hole));
        let q = dir.scale(r);
        let c = if (i + k as i32) % 2 == 0 {
            FILM
        } else {
            crate::render::mix(FILM, FILM_DEEP, ONE / 3)
        };
        // (a strip's width covers the wall's end of the fan)
        m::bar(b, f, Mesh::Cube, p, q, int(5), c, ONE / 2);
    }
    let rim = |b: &mut Batch, centre: V3, rr: i32, segs: i32| {
        for i in 0..segs {
            let p = |j: i32| {
                let a = TURN * j / segs;
                centre.add(v3(
                    mul(rr, crate::fx::sin(a)),
                    0,
                    mul(rr, crate::fx::cos(a)),
                ))
            };
            m::bar(b, f, Mesh::Cube, p(i), p(i + 1), cm(60), RING, ONE);
        }
    };
    rim(b, V3::ZERO, r, 12);
    rim(b, hole_at, hole, 8);
    let _ = thick;
    m::halo_ball(b, f, v3(0, cm(20), 0), 2 * r, FILM_DEEP, ONE / 4);
}

/// A caster's beam inside tower 1: a plain glowing rod from `a` to `z`
/// in the spell's colour with a soft halo (2 shapes; the full
/// `spell_beam` is about 20, and the hollow has over a hundred casters
/// in view at once, TA2-3).
fn thin_beam(b: &mut Batch, a: V3, z: V3, spell: Spell) {
    let (c, glow) = m::beam_look(spell);
    m::bar(b, &Affine::IDENTITY, Mesh::Cube, a, z, cm(25), c, glow);
    m::halo_bar(b, &Affine::IDENTITY, a, z, int(1), c, ONE / 2);
}

/// A conjured anchor floating in tower 1's hollow: a small crystal in the
/// spell's colour with a wide soft glow, a light in the dusk inside (2
/// shapes).
fn conjured_light(b: &mut Batch, a: V3, spell: Spell) {
    let c = spell.colour();
    m::put(
        b,
        &Affine::IDENTITY,
        Mesh::Octa,
        a,
        v3(cm(70), cm(120), cm(70)),
        crate::render::mix(c, [ONE; 3], ONE / 3),
        ONE,
    );
    m::halo_ball(b, &Affine::IDENTITY, a, int(6), c, ONE / 2);
}

/// The frame the portal is drawn in (the v1 rift's: the xy plane facing
/// back along `through`, its foot at the tear's bottom) and how far
/// behind it the banana sits.
pub fn portal_pose(t: &TowerLayout) -> (Affine, i32) {
    let s = &t.portal;
    let (half_h, _) = s.tear_half;
    let rift = at(s.tear.sub(v3(0, half_h, 0)), yaw_of(s.through.neg()));
    (rift, s.banana.sub(s.tear).dot(s.through))
}

/// The portal, turning and breathing (`towermodels::portal`), drawn each
/// frame from an eye within `PORTAL_SEEN` of it.
pub fn portal_frame(b: &mut Batch, t: &TowerLayout, eye: V3, tick: u32) {
    if t.portal.tear.sub(eye).longer_than(PORTAL_SEEN) {
        return;
    }
    let (rift, behind) = portal_pose(t);
    let (half_h, half_w) = t.portal.tear_half;
    m::portal(b, &rift, half_w * 2, half_h * 2, ONE, behind * 2, tick);
}

/// The portal is drawn from this far (3D).
pub const PORTAL_SEEN: i32 = int(400);

/// Draws every part into `sink`.
fn build(c: &Course, t: &TowerLayout, sink: &mut Sink) -> Vec<Option<V3>> {
    let mut tips = vec![None; c.vines.len()];
    let mut put = |part: &'static str, p: Option<V3>, draw: &mut dyn FnMut(&mut Batch)| {
        let mut b = Batch::new();
        draw(&mut b);
        shine(&mut b);
        sink(part, p, b);
    };
    let tick = 0;

    // tower 1: its foot, hollow sections (the doorway's one its height,
    // windows where the contract's are, a ledge round the inside every
    // other section) near in their cells and far in the always-drawn
    // chunk, the open top
    let t1 = &t.towers[0];
    let base1 = at(t1.centre, 0);
    put("tower 1", Some(t1.centre.add(v3(0, int(4), 0))), &mut |b| {
        m::keep_base(b, &base1, t1.radius, int(6), 1)
    });
    for (s, (y0, y1, door)) in tower1_spans(t).into_iter().enumerate() {
        let f = at(v3(t1.centre.x, y0, t1.centre.z), 0);
        let cuts = tower1_cuts(t, y0, y1, door);
        let ledge = if s % 2 == 0 && s > 0 && !door {
            m::Ledge::Walk
        } else {
            m::Ledge::None
        };
        let k = s as u32 * 31 + 1;
        put(
            "tower 1",
            Some(v3(t1.centre.x, (y0 + y1) / 2, t1.centre.z)),
            &mut |b| {
                m::hollow_section(
                    b,
                    &f,
                    t1.radius,
                    t1.inner,
                    y1 - y0,
                    k,
                    cuts,
                    ledge,
                    m::Detail::Near,
                )
            },
        );
        put("towers far", None, &mut |b| {
            m::hollow_section(
                b,
                &f,
                t1.radius - cm(40),
                t1.inner + cm(40),
                y1 - y0,
                k,
                cuts,
                m::Ledge::None,
                m::Detail::Far,
            )
        });
    }
    let top1 = v3(t1.centre.x, t1.top(), t1.centre.z);
    put("tower 1", Some(top1), &mut |b| {
        m::hollow_top(b, &at(top1, 0), t1.radius, t1.inner, 4, 1)
    });

    // tower 2: the old keep's look, its crown's walk at its top, the
    // cloud ring round its top
    let t2 = &t.towers[1];
    let base2 = at(t2.centre, 0);
    put("tower 2", Some(t2.centre.add(v3(0, int(4), 0))), &mut |b| {
        m::keep_base(b, &base2, t2.radius, int(6), 2)
    });
    let crown_foot = t2.top() - m::CROWN_FLOOR;
    let sections = ((crown_foot - t2.centre.y) / m::SECTION_H).max(1);
    let sh = (crown_foot - t2.centre.y) / sections;
    for s in 0..sections {
        let y = t2.centre.y + s * sh;
        let f = at(v3(t2.centre.x, y, t2.centre.z), 0);
        put(
            "tower 2",
            Some(v3(t2.centre.x, y + sh / 2, t2.centre.z)),
            &mut |b| m::keep_section(b, &f, t2.radius, sh, s as u32 + 50, m::Detail::Near),
        );
        put("towers far", None, &mut |b| {
            m::keep_section(b, &f, t2.radius - cm(40), sh, s as u32 + 50, m::Detail::Far)
        });
    }
    let crown = at(v3(t2.centre.x, crown_foot, t2.centre.z), 0);
    put(
        "tower 2",
        Some(v3(t2.centre.x, t2.top(), t2.centre.z)),
        &mut |b| m::keep_crown(b, &crown, t2.radius, 4, 2),
    );
    for (n, band) in t.clouds.iter().enumerate() {
        let y = (band.bottom + band.top) / 2;
        put("clouds", Some(v3(t2.centre.x, y, t2.centre.z)), &mut |b| {
            m::cloud_ring(
                b,
                &base2,
                t2.radius + int(14),
                y - t2.centre.y,
                6,
                n as u32 + 3,
            )
        });
    }

    // the bridges: long straight decks with their arcades
    for (i, br) in t.bridges.iter().enumerate() {
        let mid = br.from.add(br.to).scale(ONE / 2);
        put("bridges", Some(mid), &mut |b| {
            m::long_bridge(
                b,
                &Affine::IDENTITY,
                br.from,
                br.to,
                (br.half_width * 2).max(int(3)),
                i as u32,
            )
        });
    }

    // the balconies, each with its mage casting toward its beams' anchors
    // (a balcony on top of another shares its mage; inside tower 1 one
    // within 20 m of another does, the stand-in casts one a beam): tower
    // 2's on its outside, tower 1's on its inner wall
    let mut seen: Vec<(V3, Option<V3>)> = Vec::new();
    for (n, bal) in t.balconies.iter().enumerate() {
        let share = if bal.inside { int(20) } else { int(3) };
        let tip = match seen.iter().find(|(p, _)| !p.sub(bal.at).longer_than(share)) {
            Some(&(_, tip)) => tip,
            None => {
                let tower = &t.towers[bal.tower.min(1)];
                let y = bal.at.y - tower.centre.y;
                let w = bal.half_width * 2;
                let mut g = Affine::IDENTITY;
                if bal.inside {
                    let a = yaw_of(bal.out.neg());
                    put("inner balconies", Some(bal.at), &mut |b| {
                        g = m::inner_balcony(
                            b,
                            &at(tower.centre, 0),
                            tower.inner,
                            a,
                            y,
                            w,
                            n as u32,
                        )
                    });
                } else {
                    let a = yaw_of(bal.out);
                    put("balconies", Some(bal.at), &mut |b| {
                        g = m::outer_balcony(
                            b,
                            &at(tower.centre, 0),
                            tower.radius,
                            a,
                            y,
                            w,
                            n as u32,
                        )
                    });
                }
                let tip = bal.mage.then(|| {
                    let spell = Spell::nth(n as u32);
                    put("mages", Some(bal.at), &mut |b| {
                        m::mage(b, &g, spell, MagePose::Cast, tick, n as u32)
                    });
                    g.apply(m::staff_tip(MagePose::Cast))
                });
                seen.push((bal.at, tip));
                tip
            }
        };
        if let Some(tip) = tip {
            for i in cast_by(t, n) {
                let z = c.anchor(i, 0);
                put("beams", Some(bal.at), &mut |b| {
                    if bal.inside {
                        thin_beam(b, tip, z, spell_of(t, i))
                    } else {
                        m::spell_beam(b, &Affine::IDENTITY, tip, z, spell_of(t, i), tick, i as u32)
                    }
                });
                tips[i] = Some(tip);
            }
        }
    }

    // the casters away from balconies: dragons, clouds, the bridges'
    // hangers, the windows' runes, the conjured anchors floating in tower
    // 1, the start's runes, the sling bar
    let mut riders: Vec<(V3, V3)> = Vec::new();
    let mut clouds = Vec::new();
    for (i, h) in t.hangers.iter().enumerate() {
        let a = c.anchor(i, 0);
        let spell = spell_of(t, i);
        match *h {
            Hanger::Dragon { at: round } => {
                // (a dragon close to another's beams shares its rider)
                if let Some(&(_, tip)) = riders.iter().find(|(p, _)| !p.sub(a).longer_than(int(10)))
                {
                    put("dragons", Some(a), &mut |b| {
                        m::spell_beam(b, &Affine::IDENTITY, tip, a, spell, tick, i as u32)
                    });
                    tips[i] = Some(tip);
                    continue;
                }
                let out = a.sub(round).flat();
                let yaw = yaw_of(v3(out.z, 0, -out.x));
                let hide = [m::Hide::Emerald, m::Hide::Crimson, m::Hide::Bronze][i % 3];
                let f = tipped(yaw, a, |b, f| {
                    m::dragon(b, f, hide, m::Wings::Glide, spell, tick, i as u32)
                });
                let mut tip = a;
                put("dragons", Some(a), &mut |b| {
                    tip = m::dragon(b, &f, hide, m::Wings::Glide, spell, tick, i as u32);
                });
                riders.push((a, tip));
                tips[i] = Some(tip);
            }
            Hanger::Cloud { .. } => {
                if fresh(&mut clouds, a, int(8)) {
                    let f = at(a.sub(v3(0, int(2), 0)), i as i32 * 7_000);
                    put("clouds", Some(a), &mut |b| {
                        m::cloud(b, &f, int(16), int(5), i as u32)
                    });
                }
                let f = at(a, 0);
                put("clouds", Some(a), &mut |b| {
                    m::cloud_anchor(b, &f, spell, tick, i as u32)
                });
                tips[i] = Some(a);
            }
            Hanger::Bridge { bridge } => {
                // an iron hanger under the deck over the anchor, its hook
                // 4 m under the deck's top; the anchor hangs lower under
                // a rising line, so a beam runs from the hook to it
                let br = &t.bridges[bridge.min(1)];
                let top = v3(a.x, br.from.y.max(br.to.y), a.z);
                let fwd = br.to.sub(br.from).flat().norm();
                let hook = top.sub(v3(0, m::HANGER_DROP, 0));
                put("hangers", Some(a), &mut |b| {
                    m::bridge_hangers(
                        b,
                        &Affine::IDENTITY,
                        top,
                        top.add(fwd),
                        br.half_width * 2,
                        int(1000),
                        i as u32,
                    );
                    if hook.y - a.y > int(1) {
                        m::spell_beam(b, &Affine::IDENTITY, hook, a, spell, tick, i as u32);
                    }
                });
                tips[i] = Some(hook);
            }
            // (a window lets the sky in through the wall, cut with its
            // section; its beam hangs from a conjured light out in the
            // hollow, as the floating ones do)
            Hanger::Window { .. } | Hanger::Floating { .. } => {
                put("conjured", Some(a), &mut |b| conjured_light(b, a, spell));
                tips[i] = Some(a);
            }
            Hanger::Start => {
                put("anchors", Some(a), &mut |b| {
                    m::spell_anchor(b, &Affine::IDENTITY, a, spell, tick, i as u32)
                });
                tips[i] = Some(a);
            }
            // (the sling bar across the tunnel mouth's top casts these:
            // drawn with the mouth below)
            Hanger::Sling => tips[i] = Some(a),
            Hanger::Mage { .. } => {}
        }
    }

    // conjured platforms under the reset pads: frost, runes and crystal on
    // the climb; crystal ledges on tower 1's inner wall; wreck planks,
    // coral and bubbles in the void
    let mut placed_pads = Vec::new();
    for (i, pd) in t.pads.iter().enumerate() {
        let pl = &c.platforms[pd.platform];
        if !fresh(&mut placed_pads, pl.centre, cm(50)) {
            continue;
        }
        let top = v3(pl.centre.x, pl.top, pl.centre.z);
        let f = at(top, yaw_of(pl.fwd));
        let (hw, hl) = (pl.half_width, pl.half_len);
        let k = i as u32;
        if pd.at == 0 {
            put("pads", Some(top), &mut |b| {
                m::rune_stone(b, &f, hw.max(hl) * 2, tick, k)
            });
        } else if t.in_void(top) {
            put("void platforms", Some(top), &mut |b| match i % 3 {
                0 => m::wreck_plank(b, &f, hw * 2, hl * 2, k),
                1 => m::coral_shelf(b, &f, hw.max(hl), k),
                _ => m::bubble_disc(b, &f, hw.max(hl), tick, k),
            });
        } else if t.inside_tower_1(top.add(v3(0, int(1), 0))) {
            put("pads", Some(top), &mut |b| {
                let g = f.then(&Affine::translate(v3(0, 0, -hl)));
                m::crystal_ledge(b, &g, hw * 2, hl * 2, k)
            });
        } else {
            put("pads", Some(top), &mut |b| match i % 3 {
                0 => m::frost_disc(b, &f, hw.max(hl), k),
                1 => m::rune_stone(b, &f, hw.max(hl) * 2, tick, k),
                _ => {
                    let g = f.then(&Affine::translate(v3(0, 0, -hl)));
                    m::crystal_ledge(b, &g, hw * 2, hl * 2, k)
                }
            });
        }
    }

    // the portal: the mad wizard tearing reality beside the tear (the
    // tear itself turns each frame, `portal_frame`), the eldritch banana
    // pressed flat behind it, seen only through it
    let s = &t.portal;
    // (drawn 2.5 times life beside the giant tear)
    let f = m::pose(s.wizard, s.wizard_yaw, ONE * 5 / 2);
    let (half_h, half_w) = s.tear_half;
    let (rift, behind) = portal_pose(t);
    // his hands pull the tear's near edge apart, a beam from each
    let near = if rift.apply(v3(-half_w, half_h, 0)).dist(s.wizard)
        < rift.apply(v3(half_w, half_h, 0)).dist(s.wizard)
    {
        -1
    } else {
        1
    };
    put("summit", Some(s.wizard), &mut |b| {
        m::mad_wizard(b, &f, tick);
        // (he stands on a floating rune stone, not on the air)
        m::rune_stone(b, &f, int(5), tick, 7);
        for (i, hand) in m::WIZARD_HANDS.into_iter().enumerate() {
            let edge = rift.apply(v3(near * (half_w - int(2)), half_h * (2 + i as i32) / 5, 0));
            m::spell_beam(
                b,
                &Affine::IDENTITY,
                f.apply(hand),
                edge,
                Spell::Arcane,
                tick,
                i as u32,
            );
        }
    });
    put("summit", Some(s.tear), &mut |b| {
        m::banana_in_rift(b, &rift, half_h * 2, s.banana_len, behind, tick)
    });

    // the void: the tube down from under its rim in segments, portholes
    // with the creatures looking in, the mouth segment's wall open round
    // the tunnel's opening and the floor at its foot
    let z = &t.tube;
    let tn = &t.tunnel;
    let wall_top = z.top - TUBE_RIM;
    let mouth_seg = (z.bottom + SEGMENT).min(wall_top);
    let mouth_yaw = yaw_of(tn.along);
    let portholes: Vec<m::Porthole> = (0..)
        .map(|j| PORTHOLE_FIRST + PORTHOLE_EVERY * j)
        .take_while(|&d| d + m::PORTHOLE_R < wall_top - mouth_seg)
        .enumerate()
        .map(|(j, depth)| m::Porthole {
            depth,
            // (round the tube by a golden step, never over the tunnel)
            bearing: mouth_yaw + TURN / 2 + (TURN * 55 / 144) * j as i32,
        })
        .collect();
    let mut seg_top = wall_top;
    let mut seg = 0u32;
    while seg_top > mouth_seg {
        let seg_bottom = (seg_top - SEGMENT).max(mouth_seg);
        let len = seg_top - seg_bottom;
        let f = at(v3(z.axis.x, seg_top, z.axis.z), 0);
        let holes: Vec<m::Porthole> = portholes
            .iter()
            .filter(|p| {
                let d = wall_top - p.depth;
                d <= seg_top && d > seg_bottom
            })
            .map(|p| m::Porthole {
                depth: p.depth - (wall_top - seg_top),
                bearing: p.bearing,
            })
            .collect();
        let mid = v3(z.axis.x, (seg_top + seg_bottom) / 2, z.axis.z);
        let mut frames = Vec::new();
        put("tube", Some(mid), &mut |b| {
            frames = m::tube(b, &f, z.radius, len, &holes, seg + 1);
        });
        for (j, g) in frames.iter().enumerate() {
            let k = seg * 8 + j as u32;
            let p = g.apply(V3::ZERO);
            if k % 3 == 2 {
                put("creatures", Some(p), &mut |b| {
                    m::sea_monster(b, g, Spell::DEEP[(k % 2) as usize], tick, k);
                });
            } else {
                put("creatures", Some(p), &mut |b| {
                    m::blob(b, g, int(3), Spell::DEEP[(k % 2) as usize], tick, k);
                });
            }
        }
        seg_top = seg_bottom;
        seg += 1;
    }
    // the mouth segment: the void's panels round the foot, none over the
    // tunnel's opening, and the floor disc the tunnel's floor runs from
    let mouth_mid = v3(z.axis.x, (mouth_seg + z.bottom) / 2, z.axis.z);
    put("tube", Some(mouth_mid), &mut |b| {
        let f = at(v3(z.axis.x, mouth_seg, z.axis.z), 0);
        let len = mouth_seg - z.bottom;
        let bands = (len / m::SECTION_H).max(1);
        let bh = len / bands;
        let panels = 12;
        let panel_w = mul(2 * z.radius, crate::fx::ratio(268, 1000)) + cm(40);
        let half_arc = crate::fx::atan2(tn.half_width, z.radius) + TURN / 24;
        for j in 0..bands {
            let y0 = -bh * j;
            let y1 = -bh * (j + 1);
            for i in 0..panels {
                let a = TURN * i / panels;
                if crate::fx::wrap(a.wrapping_sub(mouth_yaw)).abs() <= half_arc {
                    continue;
                }
                let g = f.then(&Affine::rot_y(a));
                let c = VOID_WALL[((j * panels + i) as u32 * 7 % 3) as usize];
                m::put(
                    b,
                    &g,
                    Mesh::Cube,
                    v3(0, (y0 + y1) / 2, z.radius + int(1)),
                    v3(panel_w, bh - cm(20), int(2)),
                    c,
                    0,
                );
                m::bar(
                    b,
                    &g,
                    Mesh::Cube,
                    v3(-panel_w / 2, y1 + cm(10), z.radius + cm(40)),
                    v3(panel_w / 2, y1 + cm(10), z.radius + cm(40)),
                    cm(30),
                    VOID_RIB,
                    ONE / 3,
                );
            }
        }
        let d = 2 * z.radius + int(2);
        b.push(
            Mesh::Cyl,
            &placed(
                v3(z.axis.x, z.bottom - int(1), z.axis.z),
                0,
                v3(d, int(2), d),
            ),
            VOID_WALL[2],
            0,
        );
    });

    // the pads: where the physics' shapes cover (`pad_frame`)
    for (k, pad) in z.pads.iter().enumerate() {
        let (f, shape) = pad_frame(z.axis, z.radius, pad);
        put(
            "void pads",
            Some(v3(z.axis.x, pad.y, z.axis.z)),
            &mut |b| match shape {
                Some((shape, rr)) => m::void_pad(b, &f, rr, shape, 0, 0, tick, k as u32),
                None => {
                    let PadShape::Ring { at: hole_at, hole } = pad.shape else {
                        unreachable!()
                    };
                    ring_pad(b, &f, z.radius, hole_at, hole, k as u32)
                }
            },
        );
    }

    // the zip down the inside of the tube's wall, opposite the tunnel
    let back = tn.along.neg();
    let zip_at = v3(z.axis.x, wall_top, z.axis.z).add(back.scale(z.radius - int(2)));
    let fz = at(zip_at, yaw_of(back.neg()));
    put("zip", None, &mut |b| {
        m::zip(b, &fz, wall_top - z.bottom, int(2), ONE / 3, tick)
    });

    // the tunnel's mouth with the sling bar across its top (at the sling
    // row's anchors), and the tunnel on in segments: a round tunnel laid
    // with its floor on the contract's
    let sling_y = t
        .hangers
        .iter()
        .position(|h| *h == Hanger::Sling)
        .map_or(tn.mouth.y + tn.height, |i| c.anchor(i, 0).y);
    let bar_up = mul(z.radius, ONE * 4 / 5);
    let fm = at(v3(tn.mouth.x, sling_y - bar_up, tn.mouth.z), mouth_yaw);
    put(
        "tunnel mouth",
        Some(tn.mouth.add(v3(0, z.radius, 0))),
        &mut |b| {
            m::tunnel_mouth(b, &fm, z.radius, tick, 5);
        },
    );
    let ft = at(tn.mouth.add(v3(0, z.radius, 0)), mouth_yaw);
    let mut along = 0;
    let mut seg = 0u32;
    while along < tn.length {
        let len = (tn.length - along).min(SEGMENT);
        let f = ft.then(&Affine::translate(v3(0, 0, along)));
        let mid = f.apply(v3(0, 0, len / 2));
        put("tunnel", Some(mid), &mut |b| {
            m::tunnel(b, &f, z.radius, len, seg + 11)
        });
        along += len;
        seg += 1;
    }

    // the banana sea past the finish, where the apes dance
    let bz = &t.banana_zone;
    let f = at(bz.centre, mouth_yaw);
    let r = bz.radius.max(int(60));
    put("banana sea", None, &mut |b| m::banana_sea(b, &f, r, tick));
    tips
}

/// Where each vine's caster sends its spell from (a staff's tip, a
/// creature's, a rune's), one per `Course.vines`; None if nothing drawn
/// casts it.
pub fn casters(c: &Course, t: &TowerLayout) -> Vec<Option<V3>> {
    build(c, t, &mut |_, _, _| {})
}

/// The most shapes drawn at once from any eye along the course (a few
/// metres over each tenth sample), and the scenery's total.
pub fn worst_drawn(c: &Course) -> (usize, usize) {
    worst_drawn_at(c).0
}

/// `worst_drawn`, and where along the course (the sample) the worst is.
pub fn worst_drawn_at(c: &Course) -> ((usize, usize), usize) {
    use crate::render::{DRAW_VALUES, INSTANCE_VALUES};
    let s = scene::scenery(c);
    let mut most = (0, 0);
    for k in (0..c.path.len()).step_by(10) {
        let eye = c.path[k].add(v3(0, int(4), 0));
        let n: u32 = s.draws_near(eye).chunks(DRAW_VALUES).map(|d| d[3]).sum();
        most = most.max((n as usize, k));
    }
    ((most.0, s.values.len() / INSTANCE_VALUES), most.1)
}

/// The vines cast by balcony `n`'s mage.
pub fn cast_by(t: &TowerLayout, n: usize) -> impl Iterator<Item = usize> + '_ {
    t.hangers
        .iter()
        .enumerate()
        .filter(move |(_, h)| **h == Hanger::Mage { balcony: n })
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Map;

    /// The scenery budget (MAPS.md): about 20k shapes, and at most about
    /// as many at once as the first design's worst (4,896, TA3).
    const MOST_AT_ONCE: usize = 5_500;
    const MOST: usize = 20_000;

    #[test]
    fn every_hanger_has_its_caster_drawn_at_its_beam() {
        let c = Map::Tower.course();
        let t = c.tower.as_ref().unwrap();
        let tips = casters(&c, t);
        assert_eq!(tips.len(), c.vines.len());
        for (i, tip) in tips.iter().enumerate() {
            let tip = tip.unwrap_or_else(|| panic!("vine {i} ({:?}) has no caster", t.hangers[i]));
            // the caster's beam reaches the anchor (a spell beam from a
            // staff's tip, else the tip is the anchor), none longer than
            // 32 m (TA5: the summit's ran to 54 m); inside tower 1 a
            // balcony's mage casts for its neighbours within 20 m too
            // (TA2-3), to 48 m
            let a = c.anchor(i, 0);
            let inside =
                matches!(t.hangers[i], Hanger::Mage { balcony } if t.balconies[balcony].inside);
            let most = if inside { int(48) } else { int(32) };
            assert!(
                !tip.sub(a).longer_than(most),
                "vine {i} ({:?}): caster {} m from its anchor",
                t.hangers[i],
                tip.sub(a).len() / ONE
            );
        }
    }

    #[test]
    fn the_scenery_stays_in_budget() {
        let c = Map::Tower.course();
        let ((most, all), k) = worst_drawn_at(&c);
        assert!(all <= MOST, "{all} shapes in all");
        assert!(
            most <= MOST_AT_ONCE,
            "{most} shapes drawn at once (from sample {k}, {} m along)",
            k as i32 * crate::course::STEP / ONE
        );
    }

    #[test]
    fn the_drawn_pads_cover_what_the_physics_covers() {
        let c = Map::Tower.course();
        let t = c.tower.as_ref().unwrap();
        // the layout's pads, and one of each shape turned every which
        // way (the stand-in lays no quarters)
        let mut z = t.tube.clone();
        for k in 0..8u32 {
            let dir = crate::geom::facing(TURN * k as i32 / 8, 0);
            let off = dir.scale(int(k as i32) - int(4));
            let y = z.bottom + int(50) + int(k as i32);
            z.pads.push(TubePad {
                y,
                shape: PadShape::Quarters { axis: dir },
                gap: off,
            });
            z.pads.push(TubePad {
                y,
                shape: PadShape::Slab {
                    open: dir,
                    edge: int(k as i32) - int(3),
                },
                gap: off,
            });
            z.pads.push(TubePad {
                y,
                shape: PadShape::Ring {
                    at: off,
                    hole: int(5) + int(k as i32) / 2,
                },
                gap: off,
            });
        }
        let r = z.radius;
        let mut shapes = [0; 3];
        for (k, pad) in z.pads.iter().enumerate() {
            shapes[match pad.shape {
                PadShape::Slab { .. } => 0,
                PadShape::Ring { .. } => 1,
                PadShape::Quarters { .. } => 2,
            }] += 1;
            // every metre across the tube, a metre in from the wall (the
            // drawn edges are a strip's width wide)
            let mut wrong = 0;
            let mut n = 0;
            let step = ONE;
            let mut x = -r;
            while x <= r {
                let mut zz = -r;
                while zz <= r {
                    let rel = v3(x, 0, zz);
                    if !rel.longer_than(r - int(1)) {
                        n += 1;
                        if z.covered(k, rel) != drawn_covers(z.axis, r, pad, rel) {
                            wrong += 1;
                        }
                    }
                    zz += step;
                }
                x += step;
            }
            // (the edges of a slab or a quadrant are drawn to the point;
            // the ring's hole as the physics has it: nothing wrong but a
            // point or two on an edge: a slab turned on a diagonal puts a
            // few of the edge line's points over, in fixed point)
            assert!(
                wrong * 20 < n,
                "pad {k} ({:?}): {wrong} of {n} points drawn against the physics",
                pad.shape
            );
        }
        assert!(
            shapes.iter().all(|&n| n > 0),
            "every shape placed: {shapes:?}"
        );
    }
}
