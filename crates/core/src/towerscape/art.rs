//! Mape's Tower's still scenery, placed from the layout (step 8,
//! SECOND-ORDER TA rows). Everything is read from `TowerLayout` and the
//! `Course` (keep, branches, balconies, room, clouds, summit, zipper,
//! pads, hangers, mushrooms, vines), never from fixed places, so a retuned
//! course carries its art with it.
//!
//! Chunks are the course's; each knows the heights it spans
//! (`Scenery::bound_heights`), so on a climb a chunk far above or below the
//! eye is far off. The keep is drawn twice: each section in stone courses
//! (`Detail::Near`) in the chunk nearest it, and in one always-drawn chunk
//! as plain panels (`Detail::Far`) a little inside, so from afar the whole
//! keep stands and close up the near one covers it.

use crate::course::Course;
use crate::fx::{cm, int, ONE};
use crate::geom::{v3, yaw_of, Affine, V3};
use crate::render::Batch;
use crate::scene::{self, Scenery, VineLook};
use crate::tower::{Hanger, TowerLayout};
use crate::towermodels as m;
use crate::towermodels::{MagePose, Spell};

/// How far (in 3D) a chunk is drawn from the eye.
pub const REACH: i32 = int(90);

/// The spell vine `i` is cast with: a balcony's mage by the balcony, a
/// dragon rider, a cloud's rune and the start's by the vine, the orb
/// mage's arcane, the descent's creatures one of the two deep spells.
pub fn spell_of(t: &TowerLayout, i: usize) -> Spell {
    match t.hangers.get(i) {
        Some(Hanger::Mage { balcony }) => Spell::nth(*balcony as u32),
        Some(Hanger::Orb) => Spell::Arcane,
        Some(Hanger::Monster { .. }) => Spell::DEEP[i % 2],
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

/// The parts that belong to the zipper space: drawn only from past the
/// mirror plane (the climb's only from before it), as the sky is.
const ZIPPER_PARTS: [&str; 7] = [
    "sea monsters",
    "blobs",
    "void platforms",
    "zip",
    "topsy towers",
    "banana sea",
    "zipper trampolines",
];

/// Mape's Tower's scenery: the parts in cells of `CELL` (each knowing its
/// box and heights, drawn within `REACH`), and per side one always-drawn
/// chunk (the keep's far panels, the banana; the zip, the banana sea).
/// The climb and the zipper share their space, so each chunk is drawn
/// only from its own side of the mirror plane (`Chunk.side`).
pub fn scenery(c: &Course, t: &TowerLayout) -> Scenery {
    type Key = (bool, i32, i32, i32);
    let mut cells: std::collections::BTreeMap<Key, Batch> = Default::default();
    build(c, t, &mut |part, p, b| {
        let past = ZIPPER_PARTS.contains(&part);
        let key = match p {
            Some(p) => (
                past,
                p.x.div_euclid(CELL.0),
                p.y.div_euclid(CELL.1),
                p.z.div_euclid(CELL.0),
            ),
            None => (past, i32::MAX, 0, 0),
        };
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
                t.keep.centre
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
            side: Some((t.zipper.mirror_x, past)),
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

/// Draws every part into `sink`.
fn build(c: &Course, t: &TowerLayout, sink: &mut Sink) {
    let mut put = |part: &'static str, p: Option<V3>, draw: &mut dyn FnMut(&mut Batch)| {
        let mut b = Batch::new();
        draw(&mut b);
        shine(&mut b);
        sink(part, p, b);
    };
    let tick = 0;

    // the keep: its foot, sections stacked to its height, the crown
    let k = &t.keep;
    let base = at(k.centre, 0);
    put("keep", Some(k.centre.add(v3(0, int(4), 0))), &mut |b| {
        m::keep_base(b, &base, k.radius, int(5), 1)
    });
    let sections = (k.height / m::SECTION_H).max(1);
    let sh = k.height / sections;
    for s in 0..sections {
        let y = k.centre.y + s * sh;
        let f = at(v3(k.centre.x, y, k.centre.z), 0);
        put(
            "keep",
            Some(v3(k.centre.x, y + sh / 2, k.centre.z)),
            &mut |b| m::keep_section(b, &f, k.radius, sh, s as u32, m::Detail::Near),
        );
        put("keep far", None, &mut |b| {
            m::keep_section(b, &f, k.radius - cm(40), sh, s as u32, m::Detail::Far)
        });
    }
    let crown = at(
        v3(k.centre.x, t.summit.deck.y - m::CROWN_FLOOR, k.centre.z),
        0,
    );
    put("keep", Some(t.summit.deck), &mut |b| {
        m::keep_crown(b, &crown, k.radius, 4, 2)
    });

    // the branches: arch bridges out to the sub-towers
    let mut subs = Vec::new();
    for (i, br) in t.branches.iter().enumerate() {
        let mid = br.from.add(br.to).scale(ONE / 2);
        put("bridges", Some(mid), &mut |b| {
            m::arch_bridge(
                b,
                &Affine::IDENTITY,
                br.from,
                br.to,
                (br.half_width * 2).max(int(3)),
                i as u32,
            )
        });
        let s = br.sub;
        if fresh(&mut subs, s.centre, int(2)) {
            let f = at(s.centre, i as i32 * 9_000);
            put(
                "sub-towers",
                Some(v3(s.centre.x, mid.y, s.centre.z)),
                &mut |b| m::sub_tower(b, &f, s.radius, s.top - s.centre.y, i as u32),
            );
        }
    }

    // the balconies, each with its mage casting toward its beams' anchors
    let mut seen = Vec::new();
    for (n, bal) in t.balconies.iter().enumerate() {
        if !fresh(&mut seen, bal.at, int(3)) {
            continue;
        }
        let yaw = yaw_of(bal.out);
        let wall = bal.at.sub(bal.out.scale(bal.depth));
        let f = at(wall, yaw);
        put("balconies", Some(bal.at), &mut |b| {
            m::balcony(b, &f, bal.half_width * 2, bal.depth, n as u32)
        });
        if bal.mage {
            mage_on(c, t, n, wall, yaw, bal.depth, &mut put);
        }
    }

    // the casters away from balconies: dragons, clouds, the start's and
    // the orb's runes, the descent's creatures
    let mut creatures = Vec::new();
    let mut clouds = Vec::new();
    for (i, h) in t.hangers.iter().enumerate() {
        let a = c.anchor(i, 0);
        let spell = spell_of(t, i);
        match *h {
            Hanger::Dragon { at: round } => {
                let out = a.sub(round).flat();
                let yaw = yaw_of(v3(out.z, 0, -out.x));
                let hide = [m::Hide::Emerald, m::Hide::Crimson, m::Hide::Bronze][i % 3];
                let f = tipped(yaw, a, |b, f| {
                    m::dragon(b, f, hide, m::Wings::Glide, spell, tick, i as u32)
                });
                put("dragons", Some(a), &mut |b| {
                    m::dragon(b, &f, hide, m::Wings::Glide, spell, tick, i as u32);
                });
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
            }
            Hanger::Start | Hanger::Orb => {
                put("anchors", Some(a), &mut |b| {
                    m::spell_anchor(b, &Affine::IDENTITY, a, spell, tick, i as u32)
                });
            }
            Hanger::Monster { at: round } => {
                if !fresh(&mut creatures, a, int(5)) {
                    continue;
                }
                let face = t.zipper.axis.sub(round).flat();
                let yaw = yaw_of(if face.longer_than(cm(10)) {
                    face
                } else {
                    V3::UP
                });
                if i % 6 == 0 {
                    let f = tipped(yaw, a, |b, f| m::sea_monster(b, f, spell, tick, i as u32));
                    put("sea monsters", Some(a), &mut |b| {
                        m::sea_monster(b, &f, spell, tick, i as u32);
                    });
                } else {
                    let size = int(3);
                    let f = tipped(yaw, a, |b, f| m::blob(b, f, size, spell, tick, i as u32));
                    put("blobs", Some(a), &mut |b| {
                        m::blob(b, &f, size, spell, tick, i as u32);
                    });
                }
            }
            Hanger::Mage { .. } => {}
        }
    }

    // the trampolines, at the course's bouncers
    let kinds = [m::Pad::Cloth, m::Pad::Rune, m::Pad::Jelly];
    for (i, mu) in c.mushrooms.iter().enumerate() {
        let pad = kinds[i % 3];
        let f = at(mu.top.sub(v3(0, m::pad_top(pad), 0)), 0);
        let spell = Spell::nth(i as u32 + 1);
        let zipper = c.nearest_anywhere(mu.top) > c.path.len() / 2;
        let part = if zipper {
            "zipper trampolines"
        } else {
            "trampolines"
        };
        put(part, Some(mu.top), &mut |b| {
            m::trampoline(b, &f, mu.radius, pad, spell, 0, tick)
        });
    }

    // the trampoline room, the orb mage in the middle
    let r = &t.room;
    let f = at(r.centre, yaw_of(r.door.sub(r.centre).flat()));
    put("room", Some(r.orb.0), &mut |b| {
        m::trampoline_room(b, &f, r.radius, r.height, false, tick, 3)
    });

    // conjured platforms under the reset pads: frost, runes and crystal on
    // the climb; wreck planks, coral and bubbles in the zipper
    let mut placed = Vec::new();
    for (i, pd) in t.pads.iter().enumerate() {
        let pl = &c.platforms[pd.platform];
        if !fresh(&mut placed, pl.centre, cm(50)) {
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
        } else if t.in_zipper(top) || pd.at > crate::tower::split(c.length) {
            put("void platforms", Some(top), &mut |b| match i % 3 {
                0 => m::wreck_plank(b, &f, hw * 2, hl * 2, k),
                1 => m::coral_shelf(b, &f, hw.max(hl), k),
                _ => m::bubble_disc(b, &f, hw.max(hl), tick, k),
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

    // the summit: the mad wizard tearing reality, the eldritch banana
    let s = &t.summit;
    let f = at(s.wizard, s.wizard_yaw);
    put("summit", Some(s.wizard), &mut |b| {
        m::mad_wizard(b, &f, tick)
    });
    let facing = yaw_of(s.through.neg());
    let (half_h, half_w) = s.tear_half;
    let rift = at(s.tear.sub(v3(0, half_h, 0)), facing);
    let depth = s.banana.sub(s.tear).dot(s.through) + s.banana_len;
    put("summit", Some(s.tear), &mut |b| {
        m::torn_rift(b, &rift, half_w * 2, half_h * 2, ONE, depth, tick)
    });
    let f = at(s.banana, facing);
    put("summit", None, &mut |b| {
        m::eldritch_banana(b, &f, s.banana_len, tick)
    });

    // the zipper: the zip down the void, the sub-towers hung topsy-turvy
    let z = &t.zipper;
    let len = z.top - z.bottom;
    let f = at(v3(z.axis.x, z.top, z.axis.z), facing);
    put("zip", None, &mut |b| {
        m::zip(b, &f, len, int(8), ONE * 3 / 5, tick)
    });
    for (i, s) in subs.iter().enumerate() {
        let top = t
            .branches
            .iter()
            .find(|br| br.sub.centre == *s)
            .map_or(int(40), |br| br.sub.top);
        let hang = t.mirror(v3(s.x, top, s.z));
        let f = at(hang, i as i32 * 11_000);
        let n = ((top - s.y) / m::SECTION_H).clamp(1, 2) as u32;
        put(
            "topsy towers",
            Some(hang.sub(v3(0, int(12), 0))),
            &mut |b| m::topsy_keep(b, &f, int(8), n, ONE / 10, i as u32),
        );
        let f = at(hang.sub(v3(0, int(30), 0)), 0);
        put(
            "topsy towers",
            Some(hang.sub(v3(0, int(30), 0))),
            &mut |b| m::debris(b, &f, int(12), tick, i as u32),
        );
    }

    // the banana sea at the bottom, where the apes dance
    let bz = &t.banana_zone;
    let f = at(bz.centre, 0);
    put("banana sea", None, &mut |b| {
        m::banana_sea(b, &f, int(200), tick)
    });
}

/// A balcony's mage, about 1 m out on it, casting a beam from the staff's
/// tip to each of its vines' anchors.
fn mage_on(
    c: &Course,
    t: &TowerLayout,
    n: usize,
    wall: V3,
    yaw: i32,
    depth: i32,
    put: &mut dyn FnMut(&'static str, Option<V3>, &mut dyn FnMut(&mut Batch)),
) {
    let f = at(wall, yaw).then(&Affine::translate(v3(0, 0, depth / 2)));
    let spell = Spell::nth(n as u32);
    let tip = f.apply(m::staff_tip(MagePose::Cast));
    put("mages", Some(tip), &mut |b| {
        m::mage(b, &f, spell, MagePose::Cast, 0, n as u32);
        for (i, h) in t.hangers.iter().enumerate() {
            if *h == (Hanger::Mage { balcony: n }) {
                m::spell_beam(
                    b,
                    &Affine::IDENTITY,
                    tip,
                    c.anchor(i, 0),
                    spell,
                    0,
                    i as u32,
                );
            }
        }
    });
}

/// The vines cast by balcony `n`'s mage.
pub fn cast_by(t: &TowerLayout, n: usize) -> impl Iterator<Item = usize> + '_ {
    t.hangers
        .iter()
        .enumerate()
        .filter(move |(_, h)| **h == Hanger::Mage { balcony: n })
        .map(|(i, _)| i)
}
