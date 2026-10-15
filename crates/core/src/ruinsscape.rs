//! Sunset Ruins' scenery and light (`ruins.rs` is the contract it draws
//! from, `ruinsmodels.rs` the models; art phase 2, 2026-10-14, SECOND-ORDER
//! RS rows).
//!
//! **The hour: a vibrant sunset, clear.** The sun stands 7° up, 100° to
//! the left of the river's fall (`SUN_OFF`, RA11): it rakes across the
//! temple's face from the side, so its carved depth shows, and it hangs
//! over the left bank, in view whenever an ape looks across the river. Past the temple
//! mouth the light goes dark (`ruinsmodels::look_inside`), lit by the
//! pulsing banana glyphs; in the round chamber the skylight's shaft lights
//! the pile (`chamber_look`). `look_at` blends them by where the eye is.
//!
//! What never moves is chunked along the course (`scenery`): the ground
//! (the river's valley, the ravine, the plateau), the water's sheet, the
//! jungle (trees from `ruinsmodels::jungle_tree`, cleared round the ruins,
//! the bridge's towers and the lobby), the ruins, the bridge, the
//! corridor's stone and the chamber's wall, floor and roof. What moves or
//! changes is drawn each frame near the eye (`dynamic`): the sky, the
//! river's foam, the turtles, the waterfalls and the facade, the glyphs,
//! the trip wires and trapdoors (from the race's trap state), the pits,
//! the pile and the skylight.

use crate::course::Course;
use crate::fx::{self, cm, deg, int, mul, sin, ONE, TURN};
use crate::geom::{right_of, v3, yaw_of, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, rgb, rod, shade, Batch, Camera, Rgb};
use crate::ruins::{RuinsLayout, CORRIDOR_HALF, RIVER_HALF};
use crate::ruinsmodels as m;
use crate::rules::TICKS_PER_SECOND;
use crate::scene::{self, Look, Scenery};

/// The sun's bearing off the river's fall (to the left, +yaw): just in
/// front of the temple's face, so its light grazes it (RA11).
pub const SUN_OFF: i32 = deg(100);

/// Sunset Ruins' vines: jade ropes, glowing faintly, that read against
/// the gold and crimson sky and the dark temple alike.
pub const VINE_JADE: Rgb = rgb(60, 236, 196);
/// The colourblind mode's: vivid cyan.
pub const VINE_JADE_VIVID: Rgb = rgb(20, 255, 236);
/// The jade's soft glow round each rope (× the vine's width).
pub const VINE_HALO: i32 = 4;

const LOAM: Rgb = rgb(78, 58, 40);
const MOSS_FLOOR: Rgb = rgb(66, 92, 40);
const BED: Rgb = rgb(36, 54, 50);
const BANK: Rgb = rgb(104, 92, 70);
const CLIFF: Rgb = rgb(112, 84, 66);
const WATER: Rgb = rgb(84, 150, 150);
const FOAM: Rgb = rgb(255, 244, 226);
const STONE: Rgb = rgb(132, 112, 88);
const STONE_DARK: Rgb = rgb(78, 64, 52);
const FLOOR_STONE: Rgb = rgb(116, 98, 76);
const MOSS: Rgb = rgb(86, 116, 44);
const TRUNK: Rgb = rgb(74, 50, 40);
const GLOW_GOLD: Rgb = rgb(255, 196, 70);

/// The sun's bearing on this course.
pub fn sun_yaw(c: &Course) -> i32 {
    yaw_of(c.heading[0]) + SUN_OFF
}

/// Sunset Ruins' light before a course is built: the sunset.
pub fn look() -> Look {
    m::look()
}

/// The light outside on this course: the sunset from its sun's bearing.
pub fn look_for(c: &Course) -> Look {
    m::look_toward(sun_yaw(c))
}

/// The round chamber's light: dark, but the skylight's shaft falls warm
/// from straight above, and the fog opens so the enormous wall shows.
pub fn chamber_look() -> Look {
    Look {
        light_dir: v3(cm(10), cm(98), cm(12)).norm(),
        light: rgb(190, 138, 84),
        fog_range: (int(50), int(230)),
        ..m::look_inside()
    }
}

/// The light for an eye at `eye`: the sunset outside, dark past the
/// temple's mouth, skylit in the chamber; blended over a few metres.
pub fn look_at(c: &Course, r: &RuinsLayout, eye: V3) -> Look {
    let (s, _) = c.progress(eye, c.nearest_anywhere(eye));
    let outside = look_for(c);
    let mouth = r.marks.mouth;
    let inside = fx::div(s - (mouth - int(2)), int(14)).clamp(0, ONE);
    if inside == 0 {
        return outside;
    }
    let dark = m::look_inside();
    let into = fx::div(s - (r.marks.chamber - int(24)), int(24)).clamp(0, ONE);
    let within = if into > 0 {
        blend(&dark, &chamber_look(), into)
    } else {
        dark
    };
    if inside == ONE {
        within
    } else {
        blend(&outside, &within, inside)
    }
}

fn blend(a: &Look, b: &Look, t: i32) -> Look {
    let rgb3 = |x: Rgb, y: Rgb| [0, 1, 2].map(|k| fx::lerp(x[k], y[k], t));
    Look {
        sky: rgb3(a.sky, b.sky),
        fog: rgb3(a.fog, b.fog),
        fog_range: (
            fx::lerp(a.fog_range.0, b.fog_range.0, t),
            fx::lerp(a.fog_range.1, b.fog_range.1, t),
        ),
        light_dir: a.light_dir.lerp(b.light_dir, t).norm(),
        light: rgb3(a.light, b.light),
        ground: rgb3(a.ground, b.ground),
        rim: rgb3(a.rim, b.rim),
        far: a.far,
    }
}

// ---------------------------------------------------------------------------
// Helpers.

fn hash(k: u32, salt: u32) -> u32 {
    let mut x = k.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^ (x >> 12)
}

fn pick(k: u32, salt: u32, lo: i32, hi: i32) -> i32 {
    lo + (hash(k, salt) % ((hi - lo).max(0) as u32 + 1)) as i32
}

/// A box of `size` (across, up, along) with its middle at `mid`, its
/// length along `fwd` (which may slope).
fn block(mid: V3, fwd: V3, size: V3) -> Affine {
    Affine::translate(mid)
        .then(&Affine::looking(fwd, V3::UP))
        .then(&Affine::scale(size))
}

/// A frame at `at` whose +z points along the flat `fwd`.
fn facing_frame(at: V3, fwd: V3, scale: i32) -> Affine {
    m::pose(at, yaw_of(fwd.flat()), scale)
}

/// The land's heights where the drawing needs them: the lip's and the
/// temple floor's.
fn levels(r: &RuinsLayout) -> (i32, i32) {
    (r.ravine.lip.0.y, r.chamber.centre.y)
}

/// The centreline at `s` and its right.
fn at_s(c: &Course, s: i32, a: i32) -> V3 {
    c.point(s).add(right_of(c.heading_at(s)).scale(a))
}

// ---------------------------------------------------------------------------
// The scenery that never moves.

/// Sunset Ruins' scenery and ground, chunked.
pub fn scenery(c: &Course, r: &RuinsLayout) -> Scenery {
    let n = scene::chunk_count(c);
    let mut batches: Vec<Batch> = (0..n).map(|_| Batch::new()).collect();
    let chunk = |p: V3| scene::chunk_of(c, p, n);
    let mut push = |p: V3, f: &mut dyn FnMut(&mut Batch)| f(&mut batches[chunk(p)]);
    let (lip_h, floor) = levels(r);

    // the start: a mossy stone dais at the river's head
    for pl in &c.platforms {
        let base = c.terrain(pl.centre.x, pl.centre.z) - int(3);
        let h = pl.top - base;
        let mid = v3(pl.centre.x, base + h / 2, pl.centre.z);
        push(pl.centre, &mut |b| {
            b.push(
                Mesh::Cube,
                &block(mid, pl.fwd, v3(pl.half_width * 2, h, pl.half_len * 2)),
                STONE,
                0,
            );
            b.push(
                Mesh::Cube,
                &block(
                    v3(mid.x, pl.top - cm(30), mid.z),
                    pl.fwd,
                    v3(pl.half_width * 2 + int(1), cm(70), pl.half_len * 2 + int(1)),
                ),
                STONE_DARK,
                0,
            );
            b.push(
                Mesh::Blob,
                &block(
                    v3(mid.x, pl.top, mid.z),
                    pl.fwd,
                    v3(pl.half_width * 2, cm(50), pl.half_len),
                ),
                MOSS,
                0,
            );
        });
    }

    // the river's water: a sheet per stretch, just under the banks
    for w in r.river.points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let d = b.at.sub(a.at);
        let mid = a.at.lerp(b.at, ONE / 2);
        push(mid, &mut |bt| {
            bt.push(
                Mesh::Cube,
                &block(
                    mid,
                    d,
                    v3(a.half_width * 2 + int(1), cm(20), d.len() + int(2)),
                ),
                WATER,
                ONE / 6,
            );
        });
    }
    // rocks in the rapids, each with a skirt of white water
    let lip_s = r.marks.lip;
    let mut k = 0u32;
    let mut s = int(30);
    while s < lip_s - int(4) {
        let a = int(pick(k, 1, -11, 11));
        let p = at_s(c, s, a);
        if let Some(wt) = r.river.water_at(p.x, p.z) {
            if wt.rough > ONE / 2 && !near_turtle_lane(r, p) {
                let size = int(pick(k, 2, 2, 4));
                push(p, &mut |b| {
                    b.push(
                        Mesh::Ico,
                        &m::pose(v3(p.x, wt.surface, p.z), pick(k, 3, 0, TURN), ONE)
                            .then(&Affine::scale(v3(size, size * 2 / 3, size))),
                        CLIFF,
                        0,
                    );
                    b.push(
                        Mesh::Cyl,
                        &crate::render::placed(
                            v3(p.x, wt.surface + cm(6), p.z),
                            0,
                            v3(size * 3 / 2, cm(8), size * 2),
                        ),
                        FOAM,
                        ONE / 3,
                    );
                });
            }
        }
        k += 1;
        s += int(pick(k, 4, 9, 17));
    }

    // the cliffs: the lip's face down into the ravine (either side of the
    // river's falls), and the ravine's far wall under the temple: crags of
    // faceted rock in tiers, ledges jutting, moss on them (RS13)
    let lh = c.heading_at(lip_s);
    let lp = c.point(lip_s);
    let depth = crate::ruins::RAVINE_DEPTH;
    {
        // under the falls themselves, wet dark rock
        let face = v3(lp.x, lip_h, lp.z).sub(lh.scale(int(3)));
        push(lp, &mut |b| {
            crag(
                b,
                face,
                lh,
                -(RIVER_HALF + int(4)),
                RIVER_HALF + int(4),
                depth,
                60,
                1,
            )
        });
        for side in [-1, 1] {
            let p = face.add(right_of(lh).scale(side * int(70)));
            push(p, &mut |b| {
                let (a0, a1) = if side < 0 {
                    (-int(130), -(RIVER_HALF + int(4)))
                } else {
                    (RIVER_HALF + int(4), int(130))
                };
                crag(
                    b,
                    face,
                    lh,
                    a0,
                    a1,
                    depth,
                    92,
                    if side < 0 { 10 } else { 12 },
                )
            });
        }
        let mo = r.marks.mouth;
        let mp = c.point(mo);
        let mh = c.heading_at(mo);
        let face = v3(mp.x, floor, mp.z).sub(mh.scale(int(1)));
        push(mp, &mut |b| {
            crag(
                b,
                face,
                mh.neg(),
                -int(120),
                int(120),
                floor - (lip_h - depth),
                84,
                30,
            )
        });
    }
    ravine_mist_rocks(&mut push, c, r);

    // the temple's face: its stone is static (what glows and pulses, and
    // its waterfall, are drawn each frame in `dynamic`)
    {
        let mo = r.mouth.opening;
        let lip = v3(mo.centre.x, mo.centre.y - mo.half.y, mo.centre.z);
        let f = facing_frame(lip, mo.fwd.neg(), ONE);
        let mut face = Batch::new();
        m::temple_face(&mut face, &f, 0);
        drop(face.take_where(|g| g != 0));
        push(lip, &mut |b| b.append(face.clone()));
    }

    // the rope bridge, at its real width, on stone towers
    let br = &r.bridge;
    let span = br.to.sub(br.from).flat();
    let len = span.len();
    let kx = fx::div(br.width, m::BRIDGE_WIDTH);
    let frame = Affine::translate(br.from)
        .then(&Affine::looking(span, V3::UP))
        .then(&Affine::scale(v3(kx, ONE, ONE)));
    push(br.from.lerp(br.to, ONE / 2), &mut |b| {
        m::rope_bridge(b, &frame, len)
    });
    for (end, toward) in [(br.from, br.to), (br.to, br.from)] {
        let out = end.sub(toward).flat().norm();
        let foot = end.add(out.scale(int(2)));
        let land = c.terrain(foot.x, foot.z);
        let h = br.from.y - land;
        push(foot, &mut |b| {
            // a pillar of crags, mossy on its ledges (RS13)
            let tiers = (h / int(7)).max(1);
            let th = h / tiers;
            for t in 0..tiers {
                let k = (t as u32) * 7 + if end == br.from { 0 } else { 500 };
                let y = land + th * t + th / 2 - cm(60);
                let w = br.width + int(pick(k, 141, 1, 5));
                let at = v3(foot.x, y, foot.z).add(right_of(out).scale(cm(pick(k, 142, -80, 80))));
                b.push(
                    if t % 2 == 0 { Mesh::Geo } else { Mesh::Ico },
                    &m::pose(at, yaw_of(out) + pick(k, 143, -deg(12), deg(12)), ONE)
                        .then(&Affine::scale(v3(w, th + int(2), int(pick(k, 144, 9, 12))))),
                    shade(CLIFF, 84 + pick(k, 145, -12, 14)),
                    0,
                );
                if t > 0 && hash(k, 146).is_multiple_of(3) {
                    b.push(
                        Mesh::Blob,
                        &m::pose(
                            v3(at.x, y - th / 2 + int(1), at.z).add(out.scale(int(3))),
                            yaw_of(out),
                            ONE,
                        )
                        .then(&Affine::scale(v3(
                            w * 2 / 3,
                            int(2),
                            int(4),
                        ))),
                        shade(MOSS, 80),
                        0,
                    );
                }
            }
            b.push(
                Mesh::Blob,
                &block(
                    v3(foot.x, br.from.y - cm(40), foot.z),
                    out,
                    v3(br.width + int(3), cm(80), int(8)),
                ),
                MOSS,
                0,
            );
            m::bridge_posts(
                b,
                &Affine::translate(v3(end.x, br.from.y, end.z))
                    .then(&Affine::looking(toward.sub(end).flat(), V3::UP))
                    .then(&Affine::scale(v3(kx, ONE, ONE))),
            );
        });
    }

    // the ruins: banana statues and moai heads on the banks, big
    for (i, l) in r.landmarks.iter().enumerate() {
        let f = m::pose(l.at, l.yaw, mul(l.scale, LANDMARK_SCALE));
        push(l.at, &mut |b| match l.kind {
            crate::ruins::LandmarkKind::BananaBunch => m::banana_statue(b, &f, i as u32, true),
            crate::ruins::LandmarkKind::Moai => m::moai_head(b, &f, i as u32, true),
        });
    }
    // and a few more set back in the forest, for the skyline
    for (k, p) in skyline_ruins(c, r) {
        let yaw = yaw_of(c.point(s).sub(p).flat());
        let f = m::pose(p, yaw, int(pick(k, 12, 3, 4)));
        push(p, &mut |b| {
            if k % 3 == 1 {
                m::banana_statue(b, &f, k + 20, true)
            } else {
                m::moai_head(b, &f, k + 20, true)
            }
        });
    }

    // the trees vines hang from, each with a bough out to its vine
    for (i, h) in r.hangers.iter().enumerate() {
        let anchor = c.vines[i].anchor;
        let trunk = match *h {
            crate::ruins::Hanger::Tree { trunk } => trunk,
            crate::ruins::Hanger::Ruin { landmark } => {
                let l = &r.landmarks[landmark];
                // a kapok rooted behind the ruin
                let back = l.at.sub(anchor).flat();
                let back = if back.len() > 0 {
                    back.norm()
                } else {
                    V3::ZERO
                };
                let t = l.at.add(back.scale(int(9)));
                v3(t.x, c.terrain(t.x, t.z), t.z)
            }
            _ => continue,
        };
        let h = (anchor.y - trunk.y).max(int(10));
        let f = m::pose(
            trunk,
            pick(i as u32, 21, 0, TURN),
            fx::div(h + int(6), int(24)),
        );
        let fork = trunk.add(v3(0, h * 4 / 5, 0));
        push(trunk, &mut |b| {
            m::jungle_tree(b, &f, m::Tree::Kapok, i as u32);
            b.push(
                Mesh::Taper,
                &rod(fork, anchor.add(v3(0, cm(40), 0)), cm(90)),
                TRUNK,
                0,
            );
            b.push(
                Mesh::Blob,
                &crate::render::placed(anchor.add(v3(0, int(1), 0)), 0, v3(int(4), int(2), int(4))),
                shade(MOSS, 80),
                0,
            );
        });
    }

    // the jungle: dense on both banks, cleared round the ruins, the
    // bridge's towers, the lobby and the start camera
    let lobby = lobby_spot(c, r).0;
    let view_to = r.mouth.opening.centre;
    // the lobby's ledge: a mossy ruin block the ape stands on
    push(lobby, &mut |b| {
        b.push(
            Mesh::Bevel,
            &block(
                lobby.sub(v3(0, int(2), 0)),
                lobby.sub(view_to),
                v3(int(5), int(4), int(4)),
            ),
            STONE,
            0,
        );
        b.push(
            Mesh::Blob,
            &block(
                lobby.sub(v3(0, cm(10), 0)),
                lobby.sub(view_to),
                v3(int(4), cm(40), int(3)),
            ),
            MOSS,
            0,
        );
    });
    let mut clear: Vec<(V3, i32)> = r.landmarks.iter().map(|l| (l.at, int(13))).collect();
    clear.push((br.from, int(12)));
    clear.push((br.to, int(12)));
    clear.push((lobby, int(16)));
    let rows: [(i32, i32); 5] = [(18, 23), (26, 34), (37, 50), (55, 75), (82, 110)];
    let mut k = 0u32;
    let mut s = -int(50);
    while s < lip_s - int(6) {
        for side in [-1, 1] {
            for (ri, &(lo, hi)) in rows.iter().enumerate() {
                k += 1;
                // the outer rows thinner
                if ri >= 3 && hash(k, 30).is_multiple_of(2) {
                    continue;
                }
                let along = s + int(pick(k, 31, -3, 3));
                let a = side * int(pick(k, 32, lo, hi));
                let p = at_s(c, along, a);
                // the start camera looks down the river from behind it
                if along < int(16) && a.abs() < int(34) {
                    continue;
                }
                if clear.iter().any(|(q, d)| q.flat().sub(p.flat()).len() < *d) {
                    continue;
                }
                // the lobby's view of the temple's mouth
                if off_line(p, lobby, view_to) < int(20) {
                    continue;
                }
                let p = v3(p.x, c.terrain(p.x, p.z) + hill(a), p.z);
                let kind = match (ri, hash(k, 33) % 6) {
                    (0, 0..=2) => m::Tree::Palm,
                    (0, _) => m::Tree::Blossom,
                    (_, 0) => m::Tree::Palm,
                    (_, 1) => m::Tree::Fig,
                    (_, 2) => m::Tree::Blossom,
                    _ => m::Tree::Kapok,
                };
                let scale = ONE + ONE * pick(k, 34, 20, 90) / 100;
                let f = m::pose(p, pick(k, 35, 0, TURN), scale);
                push(p, &mut |b| m::jungle_tree(b, &f, kind, k));
            }
        }
        s += int(pick(k, 36, 10, 13));
    }
    // the plateau round the temple: a fringe of trees either side of the
    // facade and along the corridor's outside
    let mut s = r.marks.mouth + int(8);
    while s < r.marks.mouth + int(150) {
        for side in [-1, 1] {
            for &(lo, hi) in &[(40, 52), (58, 80)] {
                k += 1;
                let a = side * int(pick(k, 41, lo, hi));
                let p = at_s(c, s, a);
                if p.flat().sub(r.chamber.centre.flat()).len() < r.chamber.radius + int(20) {
                    continue;
                }
                let p = v3(p.x, floor - cm(30), p.z);
                let kind = if hash(k, 42).is_multiple_of(2) {
                    m::Tree::Kapok
                } else {
                    m::Tree::Fig
                };
                let f = m::pose(p, pick(k, 43, 0, TURN), ONE * 3 / 2);
                push(p, &mut |b| m::jungle_tree(b, &f, kind, k));
            }
        }
        s += int(12);
    }

    // the temple corridor: floor tiles (open over the pits), walls with a
    // band of carved panels, the ceiling
    corridor_stone(c, r, &mut push);
    chamber_stone(r, &mut push);

    let (terrain, ranges) = ground(c, r, n);
    scene::assemble(c, batches, terrain, ranges)
}

/// Draws into the batch of the chunk a point falls in.
type Put<'a> = dyn FnMut(V3, &mut dyn FnMut(&mut Batch)) + 'a;

/// The landmarks along the river are drawn this many times their size.
const LANDMARK_SCALE: i32 = ONE * 7 / 2;

/// The valley's sides rise away from the river (drawing only).
fn hill(a: i32) -> i32 {
    let out = (a.abs() - int(45)).max(0);
    mul(out, ONE / 4)
}

/// A cliff face of faceted rock: crags in tiers from `top_at`'s height
/// down `drop` metres, across from `a0` to `a1` (right of `fwd`), their
/// faces toward `fwd` (into the ravine); each crag its own size, turn and
/// shade (`pct` of the cliff's colour, varied), lower tiers jutting
/// further, moss on the ledges.
#[allow(clippy::too_many_arguments)]
fn crag(b: &mut Batch, top_at: V3, fwd: V3, a0: i32, a1: i32, drop: i32, pct: i32, seed: u32) {
    let fwd = fwd.flat().norm();
    let right = right_of(fwd);
    let tiers = 4;
    let tier_h = drop / tiers;
    let mut k = seed * 1000;
    for t in 0..tiers {
        let mut a = a0;
        while a < a1 {
            k += 1;
            let w = int(pick(k, 101, 12, 22));
            let mid_a = (a + w / 2).min(a1 - int(3));
            let h = tier_h + int(pick(k, 102, 2, 9));
            let jut = int(t) * 2 + int(pick(k, 103, -1, 3));
            let y = top_at.y - tier_h * t - tier_h / 2 - int(pick(k, 104, 0, 3));
            let at = top_at.add(right.scale(mid_a)).add(fwd.scale(jut - int(5)));
            let at = v3(at.x, y, at.z);
            let tone = pct + pick(k, 105, -14, 12);
            let col = if hash(k, 106).is_multiple_of(3) {
                shade(STONE_DARK, tone + 10)
            } else {
                shade(CLIFF, tone)
            };
            let mesh = if hash(k, 107).is_multiple_of(2) {
                Mesh::Ico
            } else {
                Mesh::Geo
            };
            b.push(
                mesh,
                &m::pose(at, yaw_of(fwd) + pick(k, 108, -deg(25), deg(25)), ONE)
                    .then(&Affine::scale(v3(w + int(4), h, int(pick(k, 109, 14, 20))))),
                col,
                0,
            );
            // a ledge on top, mossy, every so often
            if hash(k, 110) % 5 < 2 {
                let ledge = at.add(fwd.scale(int(3))).add(v3(0, h / 2 - int(1), 0));
                b.push(
                    Mesh::Blob,
                    &m::pose(ledge, yaw_of(fwd), ONE).then(&Affine::scale(v3(
                        w * 2 / 3,
                        int(2),
                        int(5),
                    ))),
                    shade(MOSS, 70 + pick(k, 111, 0, 30)),
                    0,
                );
            }
            a += w - int(2);
        }
    }
}

/// Boulders tumbled on the ravine's floor, half in its mist.
fn ravine_mist_rocks(push: &mut Put, c: &Course, r: &RuinsLayout) {
    let (lip_h, _) = levels(r);
    let bottom = lip_h - crate::ruins::RAVINE_DEPTH;
    for k in 0..14u32 {
        let s = r.marks.lip + int(pick(k, 121, 6, 34));
        let a = int(pick(k, 122, -110, 110));
        let p = at_s(c, s, a);
        let size = int(pick(k, 123, 4, 10));
        let p = v3(p.x, bottom + size / 3, p.z);
        push(p, &mut |b| {
            b.push(
                Mesh::Ico,
                &m::pose(p, pick(k, 124, 0, TURN), ONE).then(&Affine::scale(v3(
                    size,
                    size * 2 / 3,
                    size,
                ))),
                shade(CLIFF, 70 + pick(k, 125, 0, 20)),
                0,
            );
        });
    }
}

/// The ruins set back in the forest for the skyline: their seeds and feet.
fn skyline_ruins(c: &Course, r: &RuinsLayout) -> Vec<(u32, V3)> {
    let mut out = Vec::new();
    for k in 0..10u32 {
        let s = int(40 + k as i32 * 52);
        if s > r.marks.lip - int(20) {
            break;
        }
        let side = if k % 2 == 0 { 1 } else { -1 };
        let a = side * int(pick(k, 11, 48, 70));
        let p = at_s(c, s, a);
        out.push((k, v3(p.x, c.terrain(p.x, p.z), p.z)));
    }
    out
}

/// Where corridor section `k`'s walls (`len` long) carry their 10 m
/// panels, along from its middle: as many as fit whole on the wall,
/// evenly spaced, kept 8 m clear of an end where the corridor bends (the
/// next section's wall cuts across there), so every panel sits flush on
/// its own wall (the person, 1.10.0: flush only up to a point).
fn panel_offsets(r: &RuinsLayout, k: usize, len: i32) -> Vec<i32> {
    let dir = |i: usize| r.corridor.get(i).map(|s| s.to.sub(s.from).flat().norm());
    let here = dir(k);
    let bends = |other: Option<V3>| match (here, other) {
        (Some(a), Some(b)) => a.dot(b) < ONE * 996 / 1000,
        _ => false,
    };
    let m0 = if k > 0 && bends(dir(k - 1)) {
        int(8)
    } else {
        0
    };
    let m1 = if bends(dir(k + 1)) { int(8) } else { 0 };
    let room = len - m0 - m1;
    let n = room / int(13);
    (0..n)
        .map(|i| -len / 2 + m0 + (2 * i + 1) * (room / (2 * n)))
        .collect()
}

/// How far `p` is from the flat segment `a`–`b`.
fn off_line(p: V3, a: V3, b: V3) -> i32 {
    let d = b.sub(a).flat();
    let len = d.len();
    if len == 0 {
        return p.flat().sub(a.flat()).len();
    }
    let along = (p.sub(a).flat().dot(d.norm())).clamp(0, len);
    p.flat().sub(a.flat().add(d.norm().scale(along))).len()
}

/// Whether `p` is on a turtle's swim (rocks keep out of it).
fn near_turtle_lane(r: &RuinsLayout, p: V3) -> bool {
    r.turtles.iter().any(|t| {
        let mid = t.from.lerp(t.to, ONE / 2);
        let d = t.to.sub(t.from).flat();
        let along = d.norm();
        let q = p.sub(mid).flat();
        let off = q.sub(along.scale(q.dot(along))).len();
        off < int(6)
    })
}

/// The corridor's stone: per section, the walls (a carved panel band at
/// the apes' height) and the ceiling; the floor in tiles, open over the
/// trapdoors (whose leaves are drawn each frame) down to their pits.
fn corridor_stone(c: &Course, r: &RuinsLayout, push: &mut Put) {
    let floor = r.chamber.centre.y;
    let inside_chamber = |p: V3| p.flat().sub(r.chamber.centre.flat()).len() < r.chamber.radius;
    for (k, sec) in r.corridor.iter().enumerate() {
        let d = sec.to.sub(sec.from).flat();
        let len = d.len();
        if len == 0 {
            continue;
        }
        let fwd = d.norm();
        let right = right_of(fwd);
        let mid = sec.from.lerp(sec.to, ONE / 2);
        let wall_h = sec.height + int(4);
        for side in [-1, 1] {
            let face = mid.add(right.scale(side * sec.half_width));
            if inside_chamber(face) {
                continue;
            }
            let inward = right.scale(-side);
            push(face, &mut |b| {
                b.push(
                    Mesh::Cube,
                    &block(
                        v3(face.x, floor + wall_h / 2 - int(2), face.z)
                            .add(right.scale(side * int(1))),
                        fwd,
                        v3(int(2), wall_h, len + int(4)),
                    ),
                    STONE,
                    0,
                );
                // a dark plinth and a cornice
                for (y, hh) in [(cm(60), cm(120)), (sec.height - int(1), int(2))] {
                    b.push(
                        Mesh::Cube,
                        &block(
                            v3(face.x, floor + y, face.z).add(inward.scale(cm(30))),
                            fwd,
                            v3(cm(80), hh, len + int(2)),
                        ),
                        STONE_DARK,
                        0,
                    );
                }
                for along in panel_offsets(r, k, len) {
                    let at = v3(face.x, floor + int(9), face.z)
                        .add(fwd.scale(along))
                        .add(inward.scale(cm(30)));
                    let f = Affine::translate(at).then(&Affine::looking(inward, V3::UP));
                    m::stone_panel(b, &f, int(10), int(8));
                }
            });
        }
        // the ceiling: a slab and cross beams
        let roof = floor + sec.height;
        push(mid, &mut |b| {
            b.push(
                Mesh::Cube,
                &block(
                    v3(mid.x, roof + int(1), mid.z),
                    fwd,
                    v3(sec.half_width * 2 + int(4), int(2), len + int(2)),
                ),
                STONE_DARK,
                0,
            );
            for q in [-1, 1] {
                let at = v3(mid.x, roof - cm(60), mid.z).add(fwd.scale(q * len / 4));
                b.push(
                    Mesh::Cube,
                    &block(at, fwd, v3(sec.half_width * 2, cm(120), cm(150))),
                    shade(STONE, 80 + (k as i32 % 3) * 6),
                    0,
                );
            }
        });
    }
    // the floor: tiles along the course, none over a trapdoor
    let doors: Vec<(i32, i32)> = r
        .traps
        .iter()
        .map(|t| {
            let (s, _) = c.progress(t.door.centre, c.nearest_anywhere(t.door.centre));
            (s - t.door.half.z, s + t.door.half.z)
        })
        .collect();
    let end = r.corridor.last().map_or(r.marks.chamber, |sec| {
        c.progress(sec.to, c.nearest_anywhere(sec.to)).0
    });
    let step = int(6);
    let mut s = r.marks.mouth - int(2);
    let mut k = 0;
    while s < end {
        let s1 = (s + step).min(end);
        let open: Option<&(i32, i32)> = doors.iter().find(|(a, b)| s1 > *a && s < *b);
        let (a0, a1) = match open {
            // the tiles up to and after the hole
            Some(&(a, b)) => {
                if s < a {
                    (s, a)
                } else if s1 > b {
                    (b, s1)
                } else {
                    (s, s)
                }
            }
            None => (s, s1),
        };
        if a1 > a0 {
            let p = c.point((a0 + a1) / 2);
            let h = c.heading_at((a0 + a1) / 2);
            push(p, &mut |b| {
                b.push(
                    Mesh::Cube,
                    &block(
                        v3(p.x, floor - int(1), p.z),
                        h,
                        v3(CORRIDOR_HALF * 2 + int(2), int(2), a1 - a0 + cm(20)),
                    ),
                    shade(FLOOR_STONE, 92 + (k % 3) * 6),
                    0,
                );
            });
        }
        k += 1;
        s = s1;
    }
    // each pit: dark walls down to its spikes
    for t in &r.traps {
        let dfwd = t.door.fwd;
        let right = right_of(dfwd);
        let depth = floor - t.pit + int(3);
        let mid_y = floor - depth / 2;
        let ctr = t.door.centre;
        push(ctr, &mut |b| {
            for side in [-1, 1] {
                b.push(
                    Mesh::Cube,
                    &block(
                        v3(ctr.x, mid_y, ctr.z).add(dfwd.scale(side * (t.door.half.z + cm(50)))),
                        dfwd,
                        v3(t.door.half.x * 2, depth, int(1)),
                    ),
                    STONE_DARK,
                    0,
                );
                b.push(
                    Mesh::Cube,
                    &block(
                        v3(ctr.x, mid_y, ctr.z).add(right.scale(side * (t.door.half.x + cm(50)))),
                        dfwd,
                        v3(int(1), depth, t.door.half.z * 2 + int(2)),
                    ),
                    STONE_DARK,
                    0,
                );
            }
            b.push(
                Mesh::Cube,
                &block(
                    v3(ctr.x, floor - depth, ctr.z),
                    dfwd,
                    v3(t.door.half.x * 2, int(1), t.door.half.z * 2),
                ),
                rgb(60, 36, 20),
                ONE / 3,
            );
        });
    }
    // the spike traps (always armed)
    for t in &r.spikes {
        let at = v3(t.area.centre.x, floor, t.area.centre.z);
        let f = facing_frame(at, t.area.fwd, fx::div(t.area.half.x * 2, cm(240)));
        push(at, &mut |b| m::spike_trap(b, &f, ONE));
    }
}

/// The round chamber's wall segments: how many round the circle.
const WALL_SEGMENTS: i32 = 60;

/// The chamber's stone: the enormous wall in segments (open where the
/// corridor comes in), a carved panel band, the floor's rings, the low
/// ring of glowing posts marking the invisible wall, the roof round the
/// skylight.
fn chamber_stone(r: &RuinsLayout, push: &mut Put) {
    let ch = &r.chamber;
    let centre = ch.centre;
    let rad = ch.radius;
    let gate = gate_angle(r);
    let wall_h = ch.height + int(6);
    let seg_w = mul(rad, (TURN / WALL_SEGMENTS) * 6283 / 1000) + int(1);
    for k in 0..WALL_SEGMENTS {
        let ang = k * TURN / WALL_SEGMENTS;
        let out = crate::geom::facing(ang, 0);
        // the corridor's way in
        let off = fx::wrap(ang - gate).abs();
        let face = centre.add(out.scale(rad));
        let tangent = right_of(out);
        if off < deg(10) {
            // a lintel over the way in
            push(face, &mut |b| {
                b.push(
                    Mesh::Cube,
                    &block(
                        v3(
                            face.x,
                            centre.y
                                + crate::ruins::CORRIDOR_HEIGHT
                                + (wall_h - crate::ruins::CORRIDOR_HEIGHT) / 2,
                            face.z,
                        )
                        .add(out.scale(int(1))),
                        tangent,
                        v3(int(3), wall_h - crate::ruins::CORRIDOR_HEIGHT, seg_w),
                    ),
                    STONE,
                    0,
                );
            });
            continue;
        }
        push(face, &mut |b| {
            b.push(
                Mesh::Cube,
                &block(
                    v3(face.x, centre.y + wall_h / 2 - int(2), face.z).add(out.scale(cm(150))),
                    tangent,
                    v3(int(3), wall_h, seg_w),
                ),
                if k % 2 == 0 { STONE } else { shade(STONE, 94) },
                0,
            );
            // step bands up the wall, every 12 m
            for y in [cm(60), int(18), int(30), ch.height - int(2)] {
                b.push(
                    Mesh::Cube,
                    &block(
                        v3(face.x, centre.y + y, face.z).sub(out.scale(cm(30))),
                        tangent,
                        v3(cm(80), cm(140), seg_w),
                    ),
                    STONE_DARK,
                    0,
                );
            }
            if k % 2 == 0 {
                let at = v3(face.x, centre.y + int(9), face.z).sub(out.scale(cm(30)));
                let f = Affine::translate(at).then(&Affine::looking(out.neg(), V3::UP));
                m::stone_panel(b, &f, int(10), int(8));
            }
        });
    }
    // the floor: a great disc, rings of darker and lighter stone
    for (rr, c, y) in [
        (rad + int(4), FLOOR_STONE, -int(1)),
        (ch.inner - int(2), shade(FLOOR_STONE, 86), -cm(96)),
        (int(40), shade(FLOOR_STONE, 104), -cm(93)),
        (int(24), shade(STONE_DARK, 110), -cm(90)),
    ] {
        push(centre, &mut |b| {
            b.push(
                Mesh::Cyl,
                &crate::render::placed(centre.add(v3(0, y, 0)), 0, v3(rr * 2, int(2), rr * 2)),
                c,
                0,
            );
        });
    }
    // the invisible wall's line: low posts with glowing caps
    for k in 0..36 {
        let ang = k * TURN / 36;
        let p = centre.add(crate::geom::facing(ang, 0).scale(ch.inner - int(1)));
        push(p, &mut |b| {
            b.push(
                Mesh::Cube,
                &crate::render::placed(p.add(v3(0, cm(60), 0)), ang, v3(cm(90), cm(120), cm(90))),
                STONE_DARK,
                0,
            );
            b.push(
                Mesh::Octa,
                &crate::render::placed(p.add(v3(0, cm(150), 0)), ang, v3(cm(60), cm(80), cm(60))),
                GLOW_GOLD,
                ONE * 3 / 2,
            );
        });
    }
    // the roof: two rings of slabs round the skylight
    let roof = centre.y + ch.height;
    for (r0, r1, n) in [
        (r.skylight.radius - int(1), int(52), 24),
        (int(50), rad + int(4), 60),
    ] {
        let w = mul(r1, (TURN / n) * 6283 / 1000) + int(2);
        for k in 0..n {
            let ang = k * TURN / n;
            let out = crate::geom::facing(ang, 0);
            let p = centre.add(out.scale((r0 + r1) / 2));
            push(p, &mut |b| {
                b.push(
                    Mesh::Cube,
                    &block(v3(p.x, roof + int(2), p.z), out, v3(w, int(4), r1 - r0)),
                    shade(STONE_DARK, 90),
                    0,
                );
            });
        }
    }
}

/// The bearing from the chamber's centre to where the corridor comes in.
fn gate_angle(r: &RuinsLayout) -> i32 {
    let last = r.corridor.last().map_or(r.chamber.centre, |s| s.to);
    yaw_of(last.sub(r.chamber.centre).flat())
}

/// The ground: the river's valley from behind the start to the lip, the
/// ravine's floor, and the plateau round the temple; its vertex ranges by
/// chunk.
fn ground(c: &Course, r: &RuinsLayout, n: usize) -> (Vec<i32>, Vec<(usize, u32, u32)>) {
    use crate::mesh::VERTEX_VALUES;
    let (_, floor) = levels(r);
    let depth = crate::ruins::RAVINE_DEPTH;
    let lip_h = r.ravine.lip.0.y;
    let mut o = Vec::new();
    let mut ranges: Vec<(usize, u32, u32)> = Vec::new();
    let across: [i32; 19] = [
        -140, -100, -70, -48, -32, -22, -17, -15, -12, 0, 12, 15, 17, 22, 32, 48, 70, 100, 140,
    ];
    let lip_s = r.marks.lip;
    let mouth_s = r.marks.mouth;
    let height = |s: i32, a: i32| -> i32 {
        let p = at_s(c, s, a);
        let base = if s < lip_s {
            c.terrain(p.x, p.z)
        } else if s < mouth_s {
            lip_h - depth
        } else {
            floor - cm(30)
        };
        let bed = if s < lip_s && a.abs() < RIVER_HALF {
            -int(2)
        } else {
            0
        };
        let rise = if s < lip_s || s >= mouth_s {
            hill(a)
        } else {
            0
        };
        base + bed + rise
    };
    let tint = |p: V3, a: i32, s: i32| -> Rgb {
        if s >= lip_s && s < mouth_s {
            return shade(CLIFF, 70);
        }
        let aa = a.abs();
        if aa < RIVER_HALF && s < lip_s {
            return BED;
        }
        if aa < int(18) && s < lip_s {
            return BANK;
        }
        let m = (sin(mul(p.x, 1900)) + sin(mul(p.z, 1600)) + 2 * ONE) / 4;
        let t = mix(LOAM, MOSS_FLOOR, m);
        if aa > int(60) {
            mix(t, shade(MOSS_FLOOR, 80), ONE / 2)
        } else {
            t
        }
    };
    let verts = |o: &Vec<i32>| (o.len() / VERTEX_VALUES) as u32;
    let mut row = 0;
    for (from, to) in [
        (-int(70), lip_s),
        (lip_s, mouth_s),
        (mouth_s, mouth_s + int(160)),
    ] {
        let mut s = from;
        while s < to {
            let s1 = (s + int(6)).min(to);
            // (the river's last row ends a hair short of the lip, the
            // ravine starts at it)
            let (sa, sb) = (s, if s1 == lip_s { s1 - cm(1) } else { s1 });
            let ch = scene::chunk_of(c, c.point((sa + sb) / 2), n);
            match ranges.last() {
                Some((k, _, _)) if *k == ch => {}
                _ => ranges.push((ch, verts(&o), 0)),
            }
            for k in 0..across.len() - 1 {
                let (x0, x1) = (int(across[k]), int(across[k + 1]));
                let pt = |s: i32, a: i32| {
                    let p = at_s(c, s, a);
                    v3(p.x, height(s, a), p.z)
                };
                let (a, b, cc, d) = (pt(sa, x0), pt(sa, x1), pt(sb, x1), pt(sb, x0));
                for (h, tri) in [[a, d, cc], [a, cc, b]].into_iter().enumerate() {
                    let centre = tri[0].add(tri[1]).add(tri[2]).scale(ONE / 3);
                    let wobble = 94 + ((row * 7 + k * 13 + h * 5) % 10) as i32;
                    let t = shade(tint(centre, (x0 + x1) / 2, sa), wobble);
                    scene::push_ground(&mut o, tri, t);
                }
            }
            if let Some(last) = ranges.last_mut() {
                last.2 = verts(&o) - last.1;
            }
            s = s1;
            row += 1;
        }
    }
    (o, ranges)
}

// ---------------------------------------------------------------------------
// What moves, each frame.

/// How far from the eye the moving things are drawn.
const NEAR: i32 = int(240);

/// Everything on Sunset Ruins that moves or changes, near `eye`: the sky,
/// the river's foam, the turtles (from the race's `tick`), the
/// waterfalls, the facade, the glyphs, the trip wires and trapdoors, the
/// pits, the pile and the skylight. `t` is the frame's clock (animation).
pub fn dynamic(b: &mut Batch, c: &Course, r: &RuinsLayout, eye: V3, t: u32, tick: u32) {
    let near = |p: V3, d: i32| p.flat().sub(eye.flat()).len() < d;
    let (_, floor) = levels(r);
    let (eye_s, _) = c.progress(eye, c.nearest_anywhere(eye));
    let deep_inside = eye_s > r.marks.mouth + int(60) && eye.y < floor + int(50);
    if !deep_inside {
        m::sky(b, eye, sun_yaw(c));
    }
    // the river: foam drifting down, glints
    if eye_s < r.marks.lip + int(150) {
        foam(b, c, r, eye, eye_s, t);
    }
    for i in 0..r.turtles.len() {
        let (at, yaw) = r.turtle_at(i, tick);
        if near(at, NEAR) {
            let f = m::pose(at.sub(v3(0, cm(40), 0)), yaw, ONE * 7 / 5);
            m::turtle(b, &f, m::turtle_phase(t, i as u32));
        }
    }
    // the river's falls over the lip
    if let Some(fl) = r.falls.first() {
        let mid = fl.from.lerp(fl.to, ONE / 2);
        if near(mid, int(420)) {
            let f = facing_frame(mid, c.heading_at(r.marks.lip), ONE);
            m::waterfall(b, &f, fl.from.dist(fl.to) + int(2), fl.drop, t);
        }
    }
    // mist rising off the ravine's floor, drifting slowly
    if eye_s < r.marks.mouth + int(80) && near(c.point(r.marks.lip), int(400)) {
        let (lip_h, _) = levels(r);
        let bottom = lip_h - crate::ruins::RAVINE_DEPTH;
        for k in 0..10u32 {
            let s = r.marks.lip + int(pick(k, 131, 4, 36));
            let drift = mul(sin(t as i32 * 40 + k as i32 * 6_500), int(6));
            let a = int(pick(k, 132, -100, 100)) + drift;
            let p = at_s(c, s, a);
            let rise = int(pick(k, 133, 10, 50));
            let size = int(pick(k, 134, 30, 55));
            b.push_halo(
                Mesh::Sphere,
                &crate::render::placed(v3(p.x, bottom + rise, p.z), 0, v3(size, size / 3, size)),
                rgb(255, 226, 190),
                ONE / 3,
            );
        }
    }
    // the temple's face, its mouth over the opening (its own waterfall
    // pours out of it)
    let mo = r.mouth.opening;
    let lip = v3(mo.centre.x, mo.centre.y - mo.half.y, mo.centre.z);
    if near(lip, int(520)) && eye_s < r.marks.mouth + int(80) {
        // its stone is static scenery (so JOJO MODE recolours it, RS9);
        // here only what glows and pulses, and the falls
        let f = facing_frame(lip, mo.fwd.neg(), ONE);
        let mut face = Batch::new();
        m::temple_face(&mut face, &f, t);
        b.append(face.take_where(|g| g != 0));
        m::mouth_falls(b, &f, t);
    }
    // braziers lining the way in, both sides, over the corridor's first
    // 70 m (the person, 1.10.0: the way in was just black)
    if eye_s > r.marks.lip - int(160) && eye_s < r.marks.mouth + int(160) {
        let mut k = 0u32;
        let mut s = r.marks.mouth + int(4);
        while s < r.marks.mouth + int(74) {
            for side in [-1, 1] {
                k += 1;
                let p = at_s(c, s, side * (CORRIDOR_HALF - int(2)));
                let f = m::pose(v3(p.x, floor, p.z), 0, ONE);
                m::brazier(b, &f, t, k);
            }
            s += int(10);
        }
        // a glow in the throat, so the way in reads from the bridge
        let mo = r.mouth.opening;
        b.push_halo(
            Mesh::Sphere,
            &crate::render::placed(
                v3(mo.centre.x, floor + int(4), mo.centre.z).add(mo.fwd.scale(int(10))),
                0,
                v3(int(26), int(12), int(26)),
            ),
            rgb(255, 160, 60),
            ONE / 3,
        );
    }
    // the glyphs: in the corridor's panels and up its walls, round the
    // chamber; pulsing
    if eye_s > r.marks.mouth - int(40) {
        glyphs(b, r, eye, t);
    }
    // trip wires, trapdoors and the pits under them
    for (k, tr) in r.traps.iter().enumerate() {
        let ctr = tr.door.centre;
        if !near(ctr, int(200)) {
            continue;
        }
        let (w0, w1) = tr.wire;
        let d = w1.sub(w0).flat();
        let f = Affine::translate(v3(w0.x, floor, w0.z))
            .then(&Affine::looking(d.cross(V3::UP), V3::UP));
        m::trip_wire(b, &f, d.len(), !tr.armed(tick));
        let open = door_open(tr, tick);
        let f = facing_frame(v3(ctr.x, floor, ctr.z), tr.door.fwd, ONE);
        m::trapdoor(b, &f, tr.door.half.x * 2, tr.door.half.z * 2, open);
        if open > 0 {
            let f = facing_frame(v3(ctr.x, tr.pit - int(2), ctr.z), tr.door.fwd, ONE * 2);
            m::pit_spikes(b, &f, tr.door.half.x, tr.door.half.z, t + k as u32 * 97);
        }
    }
    // the chamber's heart: the pile and the skylight over it
    let ch = &r.chamber;
    if near(ch.centre, ch.radius + int(160)) && eye_s > r.marks.mouth + int(200) {
        let pile = &r.pile;
        let f = m::pose(pile.centre, 0, ONE * 4 / 5);
        m::banana_pile(b, &f, t);
        let sk = &r.skylight;
        let f = m::pose(sk.centre, 0, ONE);
        m::skylight(b, &f, sk.radius, int(16), ch.height - int(4), t);
    }
}

/// How far a trapdoor's leaves have swung down at `tick` (0 shut, ONE
/// open): quick on opening, slower closing.
pub fn door_open(tr: &crate::ruins::Trapdoor, tick: u32) -> i32 {
    let Some(at) = tr.tripped_at else {
        return 0;
    };
    let open_at = at + crate::ruins::TRAP_DELAY;
    let shut_at = open_at + crate::ruins::TRAP_OPEN;
    let tps = TICKS_PER_SECOND as u32;
    if tick < open_at {
        0
    } else if tick < shut_at {
        (((tick - open_at) as i64 * ONE as i64 / (tps as i64 * 3 / 10)) as i32).min(ONE)
    } else {
        let gone = (tick - shut_at) as i64 * ONE as i64 / (tps as i64 / 2);
        (ONE - gone as i32).max(0)
    }
}

/// Foam on the river near the eye: white flecks drifting downstream with
/// the current, growing and fading so none pops, more where it is rough;
/// small near the eye and larger further off (bands, MAPS water lesson).
fn foam(b: &mut Batch, c: &Course, r: &RuinsLayout, eye: V3, eye_s: i32, t: u32) {
    let cell = int(4);
    let from = (eye_s - int(40)).max(-int(14)) / cell;
    let to = (eye_s + int(150)).min(r.marks.lip) / cell;
    let tps = TICKS_PER_SECOND as i64;
    for k in from..to {
        let s0 = k * cell;
        let p0 = c.point(s0);
        let Some(w) = r.river.water_at(p0.x, p0.z) else {
            continue;
        };
        let dist = p0.flat().sub(eye.flat()).len();
        let band = if dist < int(50) {
            ONE
        } else if dist < int(100) {
            ONE * 2
        } else {
            ONE * 4
        };
        let many = 2 + (w.rough * 4 / ONE);
        let many = if band > ONE { many / 2 + 1 } else { many };
        for j in 0..many {
            let id = (k as u32).wrapping_mul(31).wrapping_add(j as u32);
            let life = 3 + pick(id, 51, 0, 2);
            let period = life as i64 * tps;
            let age = ((t as i64 + pick(id, 52, 0, 400) as i64) % period) as i32;
            let share = (age as i64 * ONE as i64 / period) as i32;
            // drifts with the current over its life
            let drift = mul(w.current.len() + int(2), int(life)) / ONE * share / ONE;
            let s = s0 + drift;
            let a = int(pick(id, 53, -12, 12)) + mul(sin(age * 80 + id as i32 * 700), cm(60));
            let p = at_s(c, s, a);
            let swell = sin(share / 2);
            let size = mul(mul(cm(pick(id, 54, 50, 120)), band), swell);
            if size < cm(5) {
                continue;
            }
            b.push(
                Mesh::Blob,
                &crate::render::placed(
                    v3(p.x, w.surface + cm(14), p.z),
                    pick(id, 55, 0, TURN),
                    v3(size, size / 5, size * 2),
                ),
                FOAM,
                ONE / 3,
            );
            // standing waves in the rapids, bobbing
            if w.rough > ONE * 3 / 4 && j == 0 {
                let bob = mul(sin(t as i32 * 300 + id as i32 * 900), cm(25));
                b.push(
                    Mesh::Wedge,
                    &crate::render::placed(
                        v3(p.x, w.surface + cm(30) + bob, p.z),
                        yaw_of(c.heading_at(s)),
                        v3(int(3), cm(70), int(2)),
                    ),
                    mix(WATER, FOAM, ONE * 3 / 5),
                    ONE / 5,
                );
            }
        }
        // a sun glint on the water, toward the sun
        if band == ONE && k % 2 == 0 {
            let id = k as u32;
            let a = int(pick(id, 56, -10, 10));
            let p = at_s(c, s0 + int(pick(id, 57, 0, 3)), a);
            let flick = (sin(t as i32 * 500 + id as i32 * 1300) + ONE) / 2;
            if flick > ONE / 2 {
                b.push(
                    Mesh::Quad,
                    &crate::render::placed(
                        v3(p.x, w.surface + cm(16), p.z),
                        deg(45),
                        v3(cm(40), ONE, cm(40)),
                    ),
                    rgb(255, 230, 160),
                    ONE * 2,
                );
            }
        }
    }
}

/// The banana glyphs near the eye, pulsing: two in each wall panel, and
/// up the walls above them.
fn glyphs(b: &mut Batch, r: &RuinsLayout, eye: V3, t: u32) {
    let floor = r.chamber.centre.y;
    let reach = int(110);
    let near = |p: V3| p.flat().sub(eye.flat()).len() < reach;
    let mut k = 0u32;
    let glyph = |b: &mut Batch, at: V3, facing: V3, k: u32| {
        let f = Affine::translate(at).then(&Affine::looking(facing, V3::UP));
        m::banana_glyph(b, &f, t, k);
    };
    let spots = m::glyph_spots(int(10), int(8));
    for (si, sec) in r.corridor.iter().enumerate() {
        let d = sec.to.sub(sec.from).flat();
        if d.len() == 0 {
            continue;
        }
        let fwd = d.norm();
        let right = right_of(fwd);
        let mid = sec.from.lerp(sec.to, ONE / 2);
        if !near(mid) {
            k += 20;
            continue;
        }
        for side in [-1, 1] {
            let face = mid.add(right.scale(side * sec.half_width));
            if face.flat().sub(r.chamber.centre.flat()).len() < r.chamber.radius {
                continue;
            }
            let inward = right.scale(-side);
            let across = right_of(inward);
            for along in panel_offsets(r, si, d.len()) {
                let centre = v3(face.x, floor + int(9), face.z)
                    .add(fwd.scale(along))
                    .add(inward.scale(cm(70)));
                for sp in &spots {
                    k += 1;
                    let at = centre.add(across.scale(sp.x)).add(v3(0, sp.y, 0));
                    glyph(b, at, inward, k);
                }
                // higher up, one over each panel
                k += 1;
                let at = v3(centre.x, floor + int(21), centre.z);
                glyph(b, at, inward, k);
            }
        }
    }
    // round the chamber
    let ch = &r.chamber;
    let gate = gate_angle(r);
    for s in 0..WALL_SEGMENTS {
        let ang = s * TURN / WALL_SEGMENTS;
        if fx::wrap(ang - gate).abs() < deg(10) {
            continue;
        }
        let out = crate::geom::facing(ang, 0);
        let face = ch.centre.add(out.scale(ch.radius - cm(50)));
        if face.flat().sub(eye.flat()).len() > int(150) {
            continue;
        }
        let across = right_of(out.neg());
        for sp in spots.iter().filter(|_| s % 2 == 0) {
            let at = v3(face.x, floor + int(9) + sp.y, face.z).add(across.scale(sp.x));
            glyph(b, at, out.neg(), 400 + s as u32 * 7);
        }
        for (j, y) in [int(24), int(36)].into_iter().enumerate() {
            if (s + j as i32) % 2 == 0 {
                let at = v3(face.x, floor + y, face.z);
                glyph(b, at, out.neg(), 900 + s as u32 * 3 + j as u32);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The lobby and the title's flyover.

/// Where the lobby may stand, from the temple's mouth: metres back up the
/// river and to its right, the first whose view of the mouth no ruin
/// blocks (RS7, RS12).
const LOBBY_TRIES: [(i32, i32); 8] = [
    (125, 42),
    (110, 40),
    (140, 42),
    (95, 38),
    (155, 40),
    (80, 34),
    (70, 30),
    (60, 26),
];
/// How far right of the mouth the camera turns, so the mouth sits between
/// the menus and the ape, clear of both.
const LOBBY_TURN: i32 = -deg(13);

/// The lobby zone: a mossy ruin block on the river's right bank, back
/// from the lip, the temple's gaping face across the ravine with its
/// waterfall, the river pouring over the lip, the sunset sky round it.
/// The feet's place and the flat way the camera looks.
pub fn lobby_spot(c: &Course, r: &RuinsLayout) -> (V3, V3) {
    let mo = r.marks.mouth;
    let mh = c.heading_at(mo);
    let mouth = r.mouth.opening.centre;
    let at = |(back, right): (i32, i32)| {
        let p = c
            .point(mo)
            .sub(mh.scale(int(back)))
            .add(right_of(mh).scale(int(right)));
        // on a ruin block a metre up (`scenery` draws it)
        v3(p.x, c.terrain(p.x, p.z) + int(1), p.z)
    };
    let clear = |feet: V3| {
        r.landmarks
            .iter()
            .all(|l| off_line(l.at, feet, mouth) > int(32))
            && skyline_ruins(c, r)
                .iter()
                .all(|&(_, p)| off_line(p, feet, mouth) > int(24))
            && r.hangers.iter().all(|h| match *h {
                crate::ruins::Hanger::Tree { trunk } => off_line(trunk, feet, mouth) > int(9),
                _ => true,
            })
    };
    let feet = LOBBY_TRIES
        .iter()
        .map(|&t| at(t))
        .find(|&f| clear(f))
        .unwrap_or_else(|| at(LOBBY_TRIES[0]));
    let look = yaw_of(mouth.sub(feet).flat()) + LOBBY_TURN;
    (feet, crate::geom::facing(look, 0))
}

/// The menus' preview on Sunset Ruins: the ape on the lobby ledge, the
/// camera behind it looking toward the temple.
pub fn preview(c: &Course) -> (V3, i32, Camera) {
    let (feet, look) = match c.ruins.as_deref() {
        Some(r) => lobby_spot(c, r),
        None => (c.starts[0], c.heading_at(0)),
    };
    let eye = feet.sub(look.scale(cm(290))).add(v3(0, cm(125), 0));
    let target = feet.add(v3(0, cm(62), 0));
    (
        feet,
        yaw_of(eye.sub(feet).flat()),
        Camera {
            eye,
            fwd: target.sub(eye).norm(),
            fov: deg(34),
        },
    )
}

/// The title's flyover on Sunset Ruins: a glide down the river, over the
/// lip and the ravine and in through the temple's mouth, along the
/// corridor and round the chamber. Its height is a smooth line (eased
/// between set heights), never the ground's drop into the ravine, and it
/// weaves gently across.
pub fn flyover_camera(c: &Course, ticks: u32, from: i32) -> Camera {
    let lap = (c.length - int(100)).max(int(1)) as i64;
    let s = ((ticks as i64 * int(6) as i64 / TICKS_PER_SECOND as i64 + from as i64) % lap) as i32
        + int(40);
    let eye = fly_line(c, s);
    let look = fly_line(c, s + int(30)).sub(v3(0, int(3), 0));
    Camera {
        eye,
        fwd: look.sub(eye).norm(),
        fov: deg(70),
    }
}

/// The flyover's line at `s`.
fn fly_line(c: &Course, s: i32) -> V3 {
    // the centreline averaged over 32 m, so its tight bends (into the
    // chamber, in to the pile) are eased
    let mut sum = V3::ZERO;
    for k in -8..=8 {
        sum = sum.add(c.point(s + k * int(2)));
    }
    let p = sum.scale(ONE / 17);
    let Some(r) = c.ruins.as_deref() else {
        return v3(p.x, int(14), p.z);
    };
    let (lip_h, floor) = levels(r);
    let (lip, mouth) = (r.marks.lip, r.marks.mouth);
    let river = |s: i32| {
        let q = c.point(s);
        c.terrain(q.x, q.z) + int(14)
    };
    // the set heights: down the river, over the lip, into the mouth's
    // middle, up to the corridor's
    let knots = [
        (lip - int(80), river(lip - int(80))),
        (lip, lip_h + int(10)),
        (mouth, floor + int(7)),
        (mouth + int(50), floor + int(13)),
    ];
    let y = if s <= knots[0].0 {
        river(s)
    } else if s >= knots[3].0 {
        floor + int(13)
    } else {
        let k = (0..3).find(|&k| s < knots[k + 1].0).unwrap_or(2);
        let (a, b) = (knots[k], knots[k + 1]);
        let share = fx::div(s - a.0, b.0 - a.0);
        let ease = (ONE - fx::cos(share / 2)) / 2;
        fx::lerp(a.1, b.1, ease)
    };
    // a slow weave across, none at the mouth
    let calm = fx::div((s - mouth).abs(), int(60)).min(ONE);
    let across = mul(mul(sin(s / 160), int(4)), calm);
    let q = p.add(right_of(c.heading_at(s)).scale(across));
    v3(q.x, y, q.z)
}
