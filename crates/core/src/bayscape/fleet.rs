//! Banana Bay Battle's art placed on the course (step 8): the fleet, the
//! start pier and its gantry, the treasure island, the barrels, the deck
//! fights and the cannon fire, drawn from the layout (`bay.rs` is the
//! contract, `baymodels.rs` the models). The sky, the light and the water
//! are `bayscape`'s own (the sky-and-weather work); this module draws none
//! of them.
//!
//! **Still (chunked scenery, built once):** every ship without its flags
//! (`baymodels::ship_still`), a rope's tie on each spar a rope hangs from,
//! the tall pier on piles 12 m over the water with a rocky headland behind
//! it, the start gantry the first row of ropes hangs from, and the island:
//! its beach (the walkable part is exactly the beach platform), shoulders
//! sloping into the sea, palms (the three rope palms reach their ropes),
//! a dune behind, and two flag posts at the finish.
//!
//! **Each frame, near the eye only (the models' cost, BM2):** the flags
//! streaming, the deck fighters (2× their built size, BM4), the cannon
//! fire (balls in flight from `BayLayout::ball_at`, muzzle smoke, splashes
//! and hull hits), the barrels bobbing on the swell (`Water::height`; a
//! bouncy barrel's drum stays within `BOB_MAX` of its bouncer's fixed top)
//! and the treasure chest.

use crate::bay::{
    BayLayout, Hanger, Side, BOB_MAX, DOCK_TOP, ISLAND_TOP, VOLLEY_CYCLE, WATER_LEVEL,
};
use crate::baymodels::{self as m, Crew, Pose};
use crate::course::Course;
use crate::fx::{self, cm, deg, int, mul, ONE, TURN};
use crate::geom::{right_of, v3, yaw_of, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{placed, rgb, rod, shade, Batch, Rgb};
use crate::scene::{self, Scenery};

/// Flags are drawn streaming on ships nearer than this.
const FLAGS_NEAR: i32 = int(220);
/// Deck fighters are drawn nearer than this (low detail, but 140 of them).
const CREW_NEAR: i32 = int(140);
/// The deck fighters' scale: built about 1.7 m tall, drawn 2× so they read
/// beside the racers' 3× (BM4).
pub const CREW_SCALE: i32 = ONE * 2;
/// Cannon fire is drawn when its muzzle or its target is nearer than this.
const FIRE_NEAR: i32 = int(260);
/// The foam round the course's hulls is drawn nearer than this.
const FOAM_NEAR: i32 = int(260);
/// The barrels are drawn nearer than this.
const BARREL_NEAR: i32 = int(180);
/// The chest is drawn nearer than this.
const CHEST_NEAR: i32 = int(320);
/// The boundary barrels' scale (built about 1 m showing, a 4.5 m pole).
const BOUNDARY_SCALE: i32 = ONE * 3 / 2;
/// A cannonball's scale (built 30 cm across): big enough to see in flight.
const BALL_SCALE: i32 = ONE * 3;
/// The muzzle smoke's and the splashes' scale.
const FX_SCALE: i32 = ONE * 2;
/// How long the muzzle smoke, a splash and a hull hit last (ticks).
const SMOKE_TICKS: u32 = 180;
const SPLASH_TICKS: u32 = 130;
const HIT_TICKS: u32 = 130;
/// The bouncy barrel model's body radius and its drum skin's height
/// (`baymodels::bounce_barrel` at scale 1).
const BOUNCE_BODY_R: i32 = cm(130);
const BOUNCE_SKIN: i32 = cm(180);

/// Banana Bay Battle's hemp ropes: a mid brown, darker than the pale sky
/// and the bright turquoise sea, so they read as rope, not as light (BP3:
/// a pale tan with a glow read as light shafts).
pub const ROPE_HEMP: Rgb = rgb(168, 112, 50);
/// The colourblind mode's ropes: a light straw, far brighter than the
/// strong blue catch-up rope.
pub const ROPE_HEMP_VIVID: Rgb = rgb(232, 196, 120);
/// How much the ropes glow (only enough to keep their shaded side from
/// going black), in either mode.
pub const ROPE_GLOW: i32 = ONE / 16;
pub const ROPE_GLOW_VIVID: i32 = ONE / 6;
/// The catch-up rope (TRIAGE T66): a hot scarlet against the tan hemp,
/// the turquoise sea and the in-reach yellow; in the colourblind mode a
/// strong blue, far darker than the pale ropes.
pub const CATCH_UP: Rgb = rgb(255, 40, 60);
pub const CATCH_UP_VIVID: Rgb = rgb(20, 50, 235);

const PLANK: Rgb = rgb(170, 128, 82);
const PLANK_DARK: Rgb = rgb(132, 96, 62);
const PILE: Rgb = rgb(78, 60, 46);
const PILE_WET: Rgb = rgb(52, 70, 56);
const HEMP: Rgb = rgb(206, 174, 112);
const HEMP_DARK: Rgb = rgb(150, 120, 72);
const IRON: Rgb = rgb(40, 40, 44);
const CLIFF: Rgb = rgb(150, 130, 104);
const CLIFF_DARK: Rgb = rgb(112, 96, 78);
const GRASS: Rgb = rgb(84, 150, 58);
const SAND: Rgb = rgb(218, 190, 132);
const SAND_WET: Rgb = rgb(176, 150, 104);
const SHALLOWS: Rgb = rgb(110, 220, 210);
const TRUNK: Rgb = rgb(132, 100, 70);
const FROND: Rgb = rgb(70, 160, 60);
const FROND_DARK: Rgb = rgb(44, 120, 50);
const FLAG_BLACK: Rgb = rgb(20, 20, 22);
const BONE: Rgb = rgb(238, 232, 214);
const LAMP: Rgb = rgb(255, 214, 130);

/// A point at another height.
trait WithY {
    fn with_y(self, y: i32) -> V3;
}

impl WithY for V3 {
    fn with_y(self, y: i32) -> V3 {
        v3(self.x, y, self.z)
    }
}

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
/// length along the flat `fwd`.
fn block(mid: V3, fwd: V3, size: V3) -> Affine {
    Affine::translate(mid)
        .then(&Affine::looking(fwd, V3::UP))
        .then(&Affine::scale(size))
}

// ---------------------------------------------------------------------------
// The still scenery.

/// Banana Bay Battle's scenery, chunked: the fleet, the pier and gantry,
/// and the island.
pub fn scenery(c: &Course, bay: &BayLayout) -> Scenery {
    let n = scene::chunk_count(c);
    let mut batches: Vec<Batch> = (0..n).map(|_| Batch::new()).collect();
    let chunk = |p: V3| scene::chunk_of(c, p, n);
    for (k, s) in bay.ships.iter().enumerate() {
        m::ship_still(
            &mut batches[chunk(s.at)],
            &m::pose(s.at, s.yaw, ONE),
            s,
            k as u32,
        );
    }
    // a rope's tie on the spar it hangs from: a lashing round the yard
    for (i, h) in bay.hangers.iter().enumerate() {
        if matches!(h, Hanger::Shore { .. }) {
            continue;
        }
        let Some(v) = c.vines.get(i) else { continue };
        let p = v.anchor;
        let b = &mut batches[chunk(p)];
        b.push(
            Mesh::Hex,
            &placed(p, 0, v3(cm(55), cm(45), cm(55))),
            HEMP_DARK,
            0,
        );
        b.push(
            Mesh::Ico,
            &placed(p.sub(v3(0, cm(40), 0)), 0, v3(cm(30), cm(40), cm(30))),
            PILE,
            0,
        );
    }
    let dock_at = bay.dock.from.lerp(bay.dock.to, ONE / 2);
    pier(&mut batches[chunk(dock_at)], bay);
    gantry(&mut batches[chunk(bay.dock.to)], c, bay);
    island(&mut batches, c, bay, &chunk);
    // the battle round the course, outside the barrels (BQ2)
    super::battle::still(&mut batches, &super::battle::cached(c, bay), &chunk);
    scene::assemble(c, batches, Vec::new(), Vec::new())
}

/// The start: a pier of plank decking on tall piles (`DOCK_TOP`, 12 m, over
/// the water, BC3), cross-braced, with bollards, rope coils, a stack of
/// cargo, lanterns at the seaward end, a ladder down, and a rocky headland
/// at its landward end that it runs out from.
fn pier(b: &mut Batch, bay: &BayLayout) {
    let d = &bay.dock;
    let fwd = d.to.sub(d.from).flat().norm();
    let right = right_of(fwd);
    let len = d.to.sub(d.from).flat().len();
    let hw = d.half_width;
    let at = |along: i32, across: i32, up: i32| {
        d.from
            .add(fwd.scale(along))
            .add(right.scale(across))
            .with_y(up)
    };
    // the planks, across, two tones
    let pitch = cm(150);
    let mut i = 0;
    while i * pitch < len {
        let z = i * pitch + pitch / 2;
        b.push(
            Mesh::Cube,
            &block(
                at(z, cm(pick(i as u32, 1, -8, 8)), DOCK_TOP - cm(12)),
                fwd,
                v3(hw * 2, cm(24), pitch - cm(8)),
            ),
            if i % 2 == 0 { PLANK } else { PLANK_DARK },
            0,
        );
        i += 1;
    }
    // the stringers under the planks and a wale along each side
    for s in [-1, 1] {
        for (up, thick) in [(DOCK_TOP - cm(45), cm(50)), (DOCK_TOP / 2, cm(35))] {
            b.push(
                Mesh::Cube,
                &block(
                    at(len / 2, s * (hw - cm(20)), up),
                    fwd,
                    v3(cm(40), thick, len),
                ),
                PILE,
                0,
            );
        }
    }
    // the piles in rows across, every 4.5 m, the outer ones standing proud
    // as posts; X braces across under every other row, diagonals along the
    // sides
    let step = cm(450);
    let rows = len / step;
    for r in 0..=rows {
        let z = (r * step).min(len - cm(30)).max(cm(30));
        for (j, x) in [-hw + cm(30), 0, hw - cm(30)].into_iter().enumerate() {
            let top = if j != 1 && r % 2 == 0 {
                DOCK_TOP + cm(90)
            } else {
                DOCK_TOP - cm(20)
            };
            b.push(
                Mesh::Hex,
                &rod(at(z, x, WATER_LEVEL - int(2)), at(z, x, top), cm(60)),
                PILE,
                0,
            );
            // the weed and wet band at the waterline
            b.push(
                Mesh::Hex,
                &placed(
                    at(z, x, WATER_LEVEL + cm(30)),
                    0,
                    v3(cm(70), cm(120), cm(70)),
                ),
                PILE_WET,
                0,
            );
        }
        if r % 2 == 0 {
            for s in [-1, 1] {
                b.push(
                    Mesh::Cube,
                    &rod(
                        at(z, -s * (hw - cm(30)), cm(150)),
                        at(z, s * (hw - cm(30)), DOCK_TOP - cm(80)),
                        cm(28),
                    ),
                    PILE,
                    0,
                );
            }
        }
        if r < rows {
            for s in [-1, 1] {
                let x = s * (hw - cm(30));
                b.push(
                    Mesh::Cube,
                    &rod(
                        at(z, x, cm(150)),
                        at((z + step).min(len - cm(30)), x, DOCK_TOP - cm(80)),
                        cm(28),
                    ),
                    PILE,
                    0,
                );
            }
        }
    }
    // bollards and rope coils at the seaward end
    for s in [-1, 1] {
        let p = at(len - int(4), s * (hw - cm(80)), DOCK_TOP + cm(35));
        b.push(
            Mesh::Hex,
            &placed(p, 0, v3(cm(50), cm(70), cm(50))),
            IRON,
            0,
        );
        b.push(
            Mesh::Hex,
            &placed(p.add(v3(0, cm(35), 0)), 0, v3(cm(80), cm(14), cm(80))),
            IRON,
            0,
        );
        let coil = at(len - int(8), s * (hw - int(2)), DOCK_TOP + cm(12));
        b.push(
            Mesh::Hex,
            &placed(coil, 0, v3(cm(140), cm(24), cm(140))),
            HEMP,
            0,
        );
        b.push(
            Mesh::Hex,
            &placed(coil.add(v3(0, cm(6), 0)), 0, v3(cm(70), cm(26), cm(70))),
            PILE,
            0,
        );
        // a lantern post on each corner
        let post = at(len - cm(40), s * (hw - cm(40)), DOCK_TOP);
        b.push(
            Mesh::Hex,
            &rod(post, post.add(v3(0, int(4), 0)), cm(24)),
            PILE,
            0,
        );
        b.push(
            Mesh::Octa,
            &placed(post.add(v3(0, cm(420), 0)), 0, v3(cm(50), cm(70), cm(50))),
            LAMP,
            ONE * 3 / 4,
        );
    }
    // cargo stacked by the landward end, off the start line
    for (k, (x, z, up)) in [(-6, 6, 0), (-6, 8, 0), (-6, 7, 1), (6, 10, 0), (7, 12, 0)]
        .into_iter()
        .enumerate()
    {
        let size = int(2);
        b.push(
            Mesh::Bevel,
            &Affine::translate(at(int(z), int(x), DOCK_TOP + size / 2 + up * size))
                .then(&Affine::rot_y(
                    deg(pick(k as u32, 2, -15, 15)) + yaw_of(fwd),
                ))
                .then(&Affine::scale(v3(size, size, size))),
            if k % 2 == 0 { PLANK_DARK } else { PLANK },
            0,
        );
    }
    for k in 0..4 {
        let p = at(int(4) + k * int(2), -(hw - cm(120)), DOCK_TOP + cm(70));
        b.push(
            Mesh::Barrel,
            &placed(p, 0, v3(cm(120), cm(140), cm(120))),
            PLANK_DARK,
            0,
        );
        b.push(
            Mesh::Hex,
            &placed(p.add(v3(0, cm(30), 0)), 0, v3(cm(124), cm(10), cm(124))),
            IRON,
            0,
        );
    }
    // a ladder down to the water off the seaward end's left side
    for s in [-1, 1] {
        let x = -hw - cm(20);
        let z = len - int(10) + s * cm(35);
        b.push(
            Mesh::Cube,
            &rod(
                at(z, x, WATER_LEVEL - cm(50)),
                at(z, x, DOCK_TOP + cm(80)),
                cm(14),
            ),
            PILE,
            0,
        );
    }
    // the headland it runs out from: a rocky bluff level with the deck,
    // grass on top, palms on it
    let back = at(-int(14), 0, 0);
    b.push(
        Mesh::Bevel,
        &block(
            v3(back.x, DOCK_TOP / 2 - int(1), back.z),
            fwd,
            v3(int(80), DOCK_TOP + int(2), int(30)),
        ),
        CLIFF,
        0,
    );
    b.push(
        Mesh::Cube,
        &block(
            back.with_y(DOCK_TOP + cm(10)),
            fwd,
            v3(int(78), cm(40), int(28)),
        ),
        GRASS,
        0,
    );
    for k in 0..9u32 {
        let x = int(pick(k, 3, -40, 40));
        let z = -int(pick(k, 4, 4, 16));
        let p = at(z, x, WATER_LEVEL + int(pick(k, 5, 1, 6)));
        let s = int(pick(k, 6, 5, 10));
        b.push(
            Mesh::Rock,
            &Affine::translate(p)
                .then(&Affine::rot_y(pick(k, 7, 0, TURN)))
                .then(&Affine::scale(v3(s, s * 3 / 2, s))),
            if k % 2 == 0 { CLIFF_DARK } else { CLIFF },
            0,
        );
    }
    for k in 0..5u32 {
        let x = int(pick(k, 8, 14, 36)) * if k % 2 == 0 { 1 } else { -1 };
        let foot = at(-int(pick(k, 9, 4, 22)), x, DOCK_TOP);
        m::palm(
            b,
            &m::pose(foot, 0, ONE * 2),
            V3::ZERO,
            int(pick(k, 10, 6, 9)),
            k,
        );
    }
}

/// The start gantry: the two tall posts (each `Hanger::Shore` foot of the
/// start row) standing in the water past the pier's end, a beam across
/// over the row's anchors, knee braces, a lashing at each rope, and a
/// black flag on each post.
fn gantry(b: &mut Batch, c: &Course, bay: &BayLayout) {
    let mut feet: Vec<V3> = Vec::new();
    let mut top = 0;
    for (i, h) in bay.hangers.iter().enumerate() {
        if let Hanger::Shore { foot } = *h {
            if foot.y != WATER_LEVEL {
                continue;
            }
            let Some(v) = c.vines.get(i) else { continue };
            top = top.max(v.anchor.y);
            b.push(
                Mesh::Hex,
                &placed(v.anchor, 0, v3(cm(70), cm(60), cm(70))),
                HEMP_DARK,
                0,
            );
            if !feet
                .iter()
                .any(|f| f.flat().sub(foot.flat()).len() < int(1))
            {
                feet.push(foot);
            }
        }
    }
    if feet.len() < 2 {
        return;
    }
    let beam_y = top + cm(55);
    let (l, r) = (feet[0], feet[feet.len() - 1]);
    let across = r.sub(l).flat().norm();
    for f in [l, r] {
        let foot = f.with_y(WATER_LEVEL - int(2));
        let head = f.with_y(beam_y + int(2));
        b.push(Mesh::Cube, &rod(foot, head, cm(110)), PILE, 0);
        b.push(
            Mesh::Hex,
            &placed(
                f.with_y(WATER_LEVEL + cm(40)),
                0,
                v3(cm(150), cm(140), cm(150)),
            ),
            PILE_WET,
            0,
        );
        // iron bands up the post
        let mut y = int(4);
        while y < beam_y {
            b.push(
                Mesh::Cube,
                &placed(f.with_y(y), yaw_of(across), v3(cm(124), cm(20), cm(124))),
                IRON,
                0,
            );
            y += int(6);
        }
        // a black flag on top
        let pole = f.with_y(beam_y + int(2));
        let flag_top = pole.add(v3(0, int(5), 0));
        b.push(Mesh::Hex, &rod(pole, flag_top, cm(16)), PILE, 0);
        let out = if f == l { across.neg() } else { across };
        let mid = flag_top.add(out.scale(cm(160))).sub(v3(0, cm(110), 0));
        b.push(
            Mesh::Cube,
            &block(mid, out, v3(cm(10), int(2), int(3))),
            FLAG_BLACK,
            0,
        );
        b.push(
            Mesh::Ico,
            &placed(
                mid.add(v3(0, cm(20), 0)).add(right_of(out).scale(cm(12))),
                0,
                v3(cm(60), cm(55), cm(60)),
            ),
            BONE,
            0,
        );
    }
    // the beam, and knee braces under each end
    let span = r.sub(l).flat().len();
    let mid = l.lerp(r, ONE / 2).with_y(beam_y);
    b.push(
        Mesh::Cube,
        &block(mid, across, v3(cm(110), cm(110), span + int(2))),
        PILE,
        0,
    );
    for (f, s) in [(l, 1), (r, -1)] {
        b.push(
            Mesh::Cube,
            &rod(
                f.with_y(beam_y - int(5)),
                f.add(across.scale(s * int(5))).with_y(beam_y - cm(50)),
                cm(50),
            ),
            PILE,
            0,
        );
    }
}

/// The finish island: the walkable beach exactly over the beach platform
/// (top `ISLAND_TOP`), shoulders and ends sloping under the sea (never
/// above the platform's top, so nothing looks walkable that isn't), a
/// ring of wet sand and shallows, rocks, a jungly dune past the beach's
/// far end, palms along its sides, the three rope palms reaching their
/// ropes' anchors, and two black flags on posts at the finish line.
fn island(batches: &mut [Batch], c: &Course, bay: &BayLayout, chunk: &dyn Fn(V3) -> usize) {
    let Some(beach) = c.platforms.iter().rev().find(|p| p.top == ISLAND_TOP) else {
        return;
    };
    let fwd = beach.fwd.flat().norm();
    let right = right_of(fwd);
    let (hl, hw) = (beach.half_len, beach.half_width);
    let ctr = beach.centre.with_y(0);
    let at = |along: i32, across: i32, up: i32| {
        ctr.add(fwd.scale(along))
            .add(right.scale(across))
            .with_y(up)
    };
    let last = batches.len() - 1;
    {
        let b = &mut batches[last];
        // the beach: a slab whose top is the platform's
        b.push(
            Mesh::Bevel,
            &block(
                at(0, 0, ISLAND_TOP / 2 - int(1)),
                fwd,
                v3(hw * 2, ISLAND_TOP + int(2), hl * 2),
            ),
            SAND,
            0,
        );
        // shoulders down into the sea on both sides and at both ends
        for s in [-1, 1] {
            b.push(
                Mesh::Geo,
                &block(
                    at(0, s * hw, ISLAND_TOP - cm(190)),
                    fwd,
                    v3(int(16), cm(360), hl * 2 + int(10)),
                ),
                SAND,
                0,
            );
            b.push(
                Mesh::Geo,
                &block(
                    at(s * hl, 0, ISLAND_TOP - cm(190)),
                    fwd,
                    v3(hw * 2 + int(12), cm(360), int(18)),
                ),
                SAND,
                0,
            );
        }
        // wet sand and the pale shallows round it
        b.push(
            Mesh::Cube,
            &block(
                at(0, 0, cm(20)),
                fwd,
                v3(hw * 2 + int(30), cm(70), hl * 2 + int(30)),
            ),
            SAND_WET,
            0,
        );
        b.push(
            Mesh::Cube,
            &block(
                at(0, 0, cm(5)),
                fwd,
                v3(hw * 2 + int(70), cm(50), hl * 2 + int(70)),
            ),
            SHALLOWS,
            ONE / 10,
        );
        // rocks round the shore
        for k in 0..14u32 {
            let along = int(pick(k, 20, -(hl / ONE), hl / ONE));
            let across = (hw + int(pick(k, 21, 6, 16))) * if k % 2 == 0 { 1 } else { -1 };
            let s = int(pick(k, 22, 2, 5));
            b.push(
                Mesh::Rock,
                &Affine::translate(at(along, across, -cm(60)))
                    .then(&Affine::rot_y(pick(k, 23, 0, TURN)))
                    .then(&Affine::scale(v3(s, s * 3 / 4, s))),
                if k % 3 == 0 { CLIFF_DARK } else { CLIFF },
                0,
            );
        }
        // the dune past the beach's far end, green on top, palms on it
        let dune = at(hl + int(40), 0, -int(4));
        b.push(
            Mesh::Geo,
            &block(dune, fwd, v3(int(150), int(26), int(80))),
            SAND,
            0,
        );
        b.push(
            Mesh::Blob,
            &block(
                dune.add(v3(0, int(10), 0)),
                fwd,
                v3(int(110), int(10), int(56)),
            ),
            FROND_DARK,
            0,
        );
        for k in 0..8u32 {
            let p = dune
                .add(right.scale(int(pick(k, 24, -50, 50))))
                .add(fwd.scale(int(pick(k, 25, -20, 20))))
                .with_y(int(6));
            m::palm(
                b,
                &m::pose(p, 0, ONE * 2),
                V3::ZERO,
                int(pick(k, 26, 7, 10)),
                k + 40,
            );
        }
    }
    // palms along both sides of the beach, out of the lanes
    for k in 0..16u32 {
        let along = int(pick(k, 27, -(hl / ONE) + 10, hl / ONE - 10));
        let across = int(pick(k, 28, 30, 38)) * if k % 2 == 0 { 1 } else { -1 };
        let foot = at(along, across, ISLAND_TOP);
        m::palm(
            &mut batches[chunk(foot)],
            &m::pose(foot, 0, ONE * 2),
            V3::ZERO,
            int(pick(k, 29, 6, 9)),
            k + 60,
        );
    }
    // the rope palms: a curved trunk from the foot to the rope's anchor
    for (i, h) in bay.hangers.iter().enumerate() {
        let Hanger::Shore { foot } = *h else { continue };
        if foot.y != ISLAND_TOP {
            continue;
        }
        let Some(v) = c.vines.get(i) else { continue };
        rope_palm(&mut batches[chunk(foot)], foot, v.anchor, i as u32);
    }
    // the finish line: two posts with black flags, out past the lanes
    let n = c.path.len() - 1;
    let fin = c.path[n];
    let r = right_of(c.heading[n]);
    for s in [-1, 1] {
        let foot = fin
            .add(r.scale(s * (c.half_width + int(6))))
            .with_y(ISLAND_TOP);
        let b = &mut batches[chunk(foot)];
        let head = foot.add(v3(0, int(14), 0));
        b.push(Mesh::Hex, &rod(foot, head, cm(40)), PILE, 0);
        let mid = head.sub(v3(0, int(2), 0)).add(r.scale(-s * int(2)));
        b.push(
            Mesh::Cube,
            &block(mid, r, v3(cm(12), int(3), int(4))),
            FLAG_BLACK,
            0,
        );
        b.push(
            Mesh::Ico,
            &placed(mid.add(v3(0, cm(30), 0)), 0, v3(cm(90), cm(80), cm(90))),
            BONE,
            0,
        );
    }
}

/// A tall palm whose trunk curves from `foot` up to the rope's `anchor`
/// (the rope hangs from under its crown), and a crown of fronds.
fn rope_palm(b: &mut Batch, foot: V3, anchor: V3, k: u32) {
    let top = anchor.add(v3(0, cm(60), 0));
    let lean = top.sub(foot).flat();
    let n = 7;
    let mut p = foot;
    for i in 1..=n {
        let t = i * ONE / n;
        // the lean grows with the square of the height: a palm's curve
        let q = foot
            .add(lean.scale(mul(t, t)))
            .with_y(foot.y + mul(top.y - foot.y, t));
        // thick enough for a 34 m palm (BP4)
        let w = cm(240) - i * cm(16);
        b.push(
            Mesh::Taper,
            &rod(p, q, w),
            if i % 2 == 0 { TRUNK } else { shade(TRUNK, 85) },
            0,
        );
        p = q;
    }
    for i in 0..11 {
        let yaw = i * TURN / 11 + pick(k, 30, 0, 3000);
        let d = v3(fx::sin(yaw), 0, fx::cos(yaw));
        let mid = top.add(d.scale(int(5))).add(v3(0, cm(120), 0));
        let tip = top.add(d.scale(int(9))).sub(v3(0, int(3), 0));
        for (a, z) in [(top, mid), (mid, tip)] {
            b.push(
                Mesh::Shard,
                &rod(a, z, ONE).then(&Affine::scale(v3(int(3), ONE, cm(18)))),
                if i % 2 == 0 { FROND } else { FROND_DARK },
                0,
            );
        }
    }
    for i in 0..3 {
        let d = v3(fx::sin(i * TURN / 3), 0, fx::cos(i * TURN / 3));
        b.push(
            Mesh::Ico,
            &placed(
                top.add(d.scale(cm(70))).sub(v3(0, cm(50), 0)),
                0,
                v3(cm(60), cm(60), cm(60)),
            ),
            PILE,
            0,
        );
    }
}

// ---------------------------------------------------------------------------
// What moves, each frame.

/// Everything on Banana Bay Battle the art moves, near `eye`: the flags,
/// the deck fights, the cannon fire, the barrels and the chest. `t` is the
/// frame's clock (animation), `tick` the race's (the volleys and the
/// swell, which the contract ties to it).
pub fn dynamic(
    b: &mut Batch,
    c: &Course,
    bay: &BayLayout,
    eye: V3,
    t: u32,
    tick: u32,
    storm: bool,
) {
    let near = |p: V3, d: i32| p.flat().sub(eye.flat()).len() < d;
    for (k, s) in bay.ships.iter().enumerate() {
        if near(s.at, FLAGS_NEAR) {
            m::ship_flags(b, &m::pose(s.at, s.yaw, ONE), s, t, k as u32);
        }
    }
    fighters(b, bay, &near, t);
    // what floats rides the sea as drawn, a squall's rougher swell and
    // chop included (BR4)
    let height = |x: i32, z: i32| super::sea_at(&bay.water, x, z, tick, storm);
    fire(b, bay, &near, tick, &height);
    barrels(b, c, bay, &near, t, tick, &height);
    foam(b, bay, &near, &height);
    super::battle::dynamic(b, &super::battle::cached(c, bay), eye, tick, &height);
    let isl = &bay.island;
    if near(isl.chest, CHEST_NEAR) {
        let scale = fx::div(isl.chest_size.x, m::CHEST_W).max(ONE / 2);
        m::treasure_chest(b, &m::pose(isl.chest, isl.chest_yaw, scale), t);
    }
}

/// How long one round of a deck duel lasts (ticks): now and then a round
/// ends with one fighter down and the other cheering.
const ROUND: u32 = 6 * crate::rules::TICKS_PER_SECOND as u32;

/// What fighter `k` is doing at `t`.
pub fn fighter_pose(bay: &BayLayout, k: usize, t: u32) -> Pose {
    let f = &bay.fighters[k];
    let Some(foe) = f.foe else {
        return Pose::Cheer;
    };
    let pair = k.min(foe) as u32;
    let round = (t + bay.fighters[pair as usize].phase) / ROUND;
    if hash(pair, round).is_multiple_of(4) {
        // one of them is down this round; the other cheers
        let down = if hash(pair, round ^ 0x55).is_multiple_of(2) {
            pair as usize
        } else {
            foe.max(k)
        };
        if down == k {
            Pose::Down
        } else {
            Pose::Cheer
        }
    } else if f.side == Side::Navy && hash(pair, 7).is_multiple_of(3) {
        Pose::Aim
    } else {
        Pose::Duel
    }
}

fn fighters(b: &mut Batch, bay: &BayLayout, near: &dyn Fn(V3, i32) -> bool, t: u32) {
    for (k, f) in bay.fighters.iter().enumerate() {
        if !near(f.at, CREW_NEAR) {
            continue;
        }
        let crew = match f.side {
            Side::Pirate => Crew::Pirate,
            Side::Navy => Crew::Navy,
        };
        let pose = fighter_pose(bay, k, t);
        m::deckhand(
            b,
            &m::pose(f.at, f.yaw, CREW_SCALE),
            crew,
            pose,
            t.wrapping_add(f.phase),
            k as u32,
        );
    }
}

/// The cannon fire: each volley's ball in flight (`BayLayout::ball_at`),
/// the smoke at its muzzle as it fires, and where it lands a splash or a
/// hit on the hull.
fn fire(
    b: &mut Batch,
    bay: &BayLayout,
    near: &dyn Fn(V3, i32) -> bool,
    tick: u32,
    height: &dyn Fn(i32, i32) -> i32,
) {
    for (i, v) in bay.volleys.iter().enumerate() {
        let Some(s) = bay.ships.get(v.from) else {
            continue;
        };
        let Some(p) = s.ports.get(v.port) else {
            continue;
        };
        let muzzle = s.point(p.along, p.side * s.beam / 2, p.up);
        if !near(muzzle, FIRE_NEAR) && !near(v.target, FIRE_NEAR) {
            continue;
        }
        let into = (tick % VOLLEY_CYCLE + VOLLEY_CYCLE - v.fire % VOLLEY_CYCLE) % VOLLEY_CYCLE;
        if let Some(at) = bay.ball_at(i, tick) {
            m::cannonball(b, &m::pose(at, 0, BALL_SCALE), V3::ZERO);
        }
        if into < SMOKE_TICKS {
            let out = s.right().scale(p.side);
            let age = (into * ONE as u32 / SMOKE_TICKS) as i32;
            m::smoke_puff(b, &m::pose(muzzle, yaw_of(out), FX_SCALE), age, i as u32);
        }
        if into >= v.flight {
            let after = into - v.flight;
            if v.hit && after < HIT_TICKS {
                let Some(t) = bay.ships.get(v.at) else {
                    continue;
                };
                // the hull's face toward -z: +z points into the hull
                let side = if v.target.sub(t.at).dot(t.right()) >= 0 {
                    1
                } else {
                    -1
                };
                let inward = t.right().scale(-side);
                let age = (after * ONE as u32 / HIT_TICKS) as i32;
                m::hull_hit(
                    b,
                    &m::pose(v.target, yaw_of(inward), FX_SCALE),
                    age,
                    i as u32,
                );
            } else if !v.hit && after < SPLASH_TICKS {
                let y = height(v.target.x, v.target.z);
                let age = (after * ONE as u32 / SPLASH_TICKS) as i32;
                m::splash(b, &m::pose(v.target.with_y(y), 0, FX_SCALE), age, i as u32);
            }
        }
    }
}

/// Foam collars where the course's hulls meet the sea (BQ3): three along
/// each hull, each at the swell's height there, so the line moves with
/// the sea.
fn foam(
    b: &mut Batch,
    bay: &BayLayout,
    near: &dyn Fn(V3, i32) -> bool,
    height: &dyn Fn(i32, i32) -> i32,
) {
    for (k, s) in bay.ships.iter().enumerate() {
        if !near(s.at, FOAM_NEAR) {
            continue;
        }
        let third = s.length / 3;
        for i in -1..=1 {
            let p = s.point(i * third * 9 / 10, 0, 0);
            let y = height(p.x, p.z);
            let beam = if i == 0 { s.beam } else { s.beam * 4 / 5 };
            m::foam(
                b,
                &m::pose(p.with_y(y), s.yaw, ONE),
                beam * 6 / 5,
                third * 5 / 4,
                (k * 3) as u32 + (i + 1) as u32,
            );
        }
    }
}

/// The barrels on the swell: the flagged boundary barrels ride
/// `Water::height`; a bouncy barrel's drum skin stays within `BOB_MAX` of
/// its bouncer's fixed top (the physics never bobs it).
fn barrels(
    b: &mut Batch,
    c: &Course,
    bay: &BayLayout,
    near: &dyn Fn(V3, i32) -> bool,
    t: u32,
    tick: u32,
    height: &dyn Fn(i32, i32) -> i32,
) {
    for (k, bb) in bay.boundary.iter().enumerate() {
        if !near(bb.at, BARREL_NEAR) {
            continue;
        }
        let y = height(bb.at.x, bb.at.z);
        let on = m::pose(bb.at.with_y(y), 0, BOUNDARY_SCALE);
        m::boundary_barrel(b, &on, t, k as u32);
        // a small foam ring where it sits in the sea (BQ3)
        m::foam(b, &on, cm(230), cm(230), k as u32);
    }
    for (k, bb) in bay.bouncy.iter().enumerate() {
        let Some(mu) = c.mushrooms.get(bb.bouncer) else {
            continue;
        };
        if !near(mu.top, BARREL_NEAR) {
            continue;
        }
        let (foot, scale) = bouncy_frame(bay, mu.top, mu.radius, t, tick, k as u32);
        m::bounce_barrel(b, &m::pose(foot, bb.yaw, scale), t, k as u32, 0);
        let y = height(mu.top.x, mu.top.z);
        let r = mul(BOUNCE_BODY_R, scale) * 2 + cm(80);
        m::foam(
            b,
            &m::pose(mu.top.with_y(y), bb.yaw, ONE),
            r,
            r,
            k as u32 + 500,
        );
    }
}

/// Where a bouncy barrel's model stands (its origin) and its scale: sized
/// so its body's radius is the bouncer's, and lowered so its drum skin
/// (the model's own bob included) is the bouncer's top plus the swell
/// there, held within `BOB_MAX`.
pub fn bouncy_frame(bay: &BayLayout, top: V3, radius: i32, t: u32, tick: u32, k: u32) -> (V3, i32) {
    let scale = fx::div(radius, BOUNCE_BODY_R);
    let (rise, _) = m::bob(t, k);
    let rise = mul(rise, scale);
    let swell = bay.water.swell(top.x, top.z, tick).clamp(-BOB_MAX, BOB_MAX);
    let skin = top.y + swell;
    (top.with_y(skin - mul(BOUNCE_SKIN, scale) - rise), scale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Map;

    #[test]
    fn a_bouncy_barrels_drum_stays_by_its_bouncers_top() {
        let c = Map::Bay.course();
        let bay = c.bay.as_deref().unwrap();
        for (k, bb) in bay.bouncy.iter().enumerate() {
            let mu = c.mushrooms[bb.bouncer];
            for tick in (0..2_000).step_by(37) {
                let (foot, scale) = bouncy_frame(bay, mu.top, mu.radius, tick, tick, k as u32);
                let (rise, _) = m::bob(tick, k as u32);
                let skin = foot.y + mul(BOUNCE_SKIN + rise, scale);
                assert!(
                    (skin - mu.top.y).abs() <= BOB_MAX + cm(1),
                    "barrel {k} tick {tick}: drum {} cm off its top",
                    (skin - mu.top.y) * 100 / ONE
                );
            }
        }
    }

    #[test]
    fn the_island_draws_nothing_over_the_beach_top_off_the_beach() {
        // nothing that looks walkable where the beach platform isn't
        let c = Map::Bay.course();
        let bay = c.bay.as_deref().unwrap();
        let n = scene::chunk_count(&c);
        let mut batches: Vec<Batch> = (0..n).map(|_| Batch::new()).collect();
        let chunk = |p: V3| scene::chunk_of(&c, p, n);
        island(&mut batches, &c, bay, &chunk);
        let (v, _) = batches.pop().unwrap().finish(crate::render::DYNAMIC_BUFFER);
        let beach = c
            .platforms
            .iter()
            .rev()
            .find(|p| p.top == ISLAND_TOP)
            .unwrap();
        let (fwd, right) = (beach.fwd.flat().norm(), right_of(beach.fwd.flat().norm()));
        let mut top_at_beach = 0;
        let mut beside = 0;
        for i in v.chunks(crate::render::INSTANCE_VALUES) {
            // its top (the up column's half length over its middle)
            let top = i[7] + v3(i[1], i[5], i[9]).len() / 2;
            let d = v3(i[3], 0, i[11]).sub(beach.centre.flat());
            let (along, across) = (d.dot(fwd).abs(), d.dot(right).abs());
            // the ground: shapes whose middle is under the beach top (not palms)
            if i[7] < ISLAND_TOP && along <= beach.half_len && across <= beach.half_width + int(20)
            {
                assert!(
                    top <= ISLAND_TOP + cm(1),
                    "a shape {} cm over the beach's top, {} m from its middle across",
                    (top - ISLAND_TOP) * 100 / ONE,
                    across / ONE
                );
                beside += 1;
                if (top - ISLAND_TOP).abs() <= cm(1) && along == 0 && across == 0 {
                    top_at_beach += 1;
                }
            }
        }
        assert!(beside > 3, "{beside}");
        assert_eq!(top_at_beach, 1, "the beach slab's top is the platform's");
    }

    #[test]
    fn a_fight_round_sometimes_ends_with_one_down_and_one_cheering() {
        let c = Map::Bay.course();
        let bay = c.bay.as_deref().unwrap();
        let mut seen = (0, 0);
        for t in (0..20 * ROUND).step_by(ROUND as usize) {
            for k in 0..bay.fighters.len() {
                let Some(foe) = bay.fighters[k].foe else {
                    continue;
                };
                let (a, b) = (fighter_pose(bay, k, t), fighter_pose(bay, foe, t));
                if a == Pose::Down {
                    assert_eq!(b, Pose::Cheer, "fighter {k} down at {t}: its foe");
                    seen.0 += 1;
                }
                if a == Pose::Duel {
                    seen.1 += 1;
                }
            }
        }
        assert!(seen.0 > 0 && seen.1 > seen.0, "{seen:?}");
    }
}
