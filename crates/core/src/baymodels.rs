//! Banana Bay Battle's models (2026-10-15, `briefs/banana-bay-battle.md`,
//! step 4 "Models"): stand-alone builders for the pirate map's props, in
//! the faceted low-poly style of `ruinsmodels.rs` and `vehicles.rs`. No
//! course places them yet. Each is built in its own metres, +z forward
//! (a ship's bow) and +y up, the waterline at y 0, then placed by a frame
//! (`pose`) the caller passes. Each takes plain parameters (a ship's kind,
//! side and the state of each sail), so the course can choose where the
//! gaps are; nothing here keeps state: flags, barrels, smoke and the crew
//! move with the tick alone.
//!
//! **The hour: 3 pm in the Caribbean, clear.** A high warm sun (about 50°)
//! a little behind the battle, a bright blue sky, turquoise water. Step 5
//! (sky and lighting) owns the map's real `Look`; `gallery_look` here only
//! lights the model shots the same way.
//!
//! A palette no other map uses: tarred black pirate hulls with oxblood
//! wales and dingy patched canvas; deep blue navy hulls with ochre gun
//! stripes and crisp white sails; honey deck planks; turquoise water;
//! white sand; and the gold of doubloons and the finish's bananas.
//!
//! The navy is nobody's: its ensign is a made-up purple field with a gold
//! chevron and a white ring, its pennant purple and gold.

use crate::bay::{self, Mast, Port, Sail, Ship, ShipKind, Side};
use crate::fx::{self, cm, cos, deg, int, mul, sin, ONE, TURN};
use crate::geom::{facing, v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, rgb, rod, shade, Batch, Rgb};
use crate::ruinsmodels::{banana, bunch, glyph_pulse, pick, wave};
use crate::scene::Look;

pub use crate::ruinsmodels::pose;

// ---------------------------------------------------------------------------
// The gallery's light (step 5 owns the map's).

/// The sun's height over the horizon at 3 pm.
pub const SUN_HIGH: i32 = deg(50);
/// The sun's bearing in the gallery.
pub const SUN_YAW: i32 = deg(35);
pub const SKY: Rgb = rgb(104, 182, 240);
pub const FOG: Rgb = rgb(196, 228, 242);
pub const LIGHT: Rgb = rgb(300, 288, 260);
/// The light bounced up off the turquoise water.
pub const BOUNCE: Rgb = rgb(80, 150, 156);

/// Bright 3 pm light for the model gallery (`?gallery=galleon`).
pub fn gallery_look() -> Look {
    Look {
        sky: SKY,
        fog: FOG,
        fog_range: (int(200), int(600)),
        light_dir: facing(SUN_YAW, SUN_HIGH),
        light: LIGHT,
        ground: BOUNCE,
        rim: [0; 3],
        far: crate::render::FAR,
    }
}

// ---------------------------------------------------------------------------
// The palette.

pub const SEA: Rgb = rgb(34, 170, 186);
const SEA_LIGHT: Rgb = rgb(86, 206, 204);
const FOAM: Rgb = rgb(240, 250, 248);

const PIRATE_HULL: Rgb = rgb(40, 32, 30);
const PIRATE_WALE: Rgb = rgb(120, 34, 30);
/// Weathered cream canvas, stained (BQ1: (198,184,152) unlit read as a
/// teal-grey slab under the sea's bounce).
const PIRATE_SAIL: Rgb = rgb(226, 208, 168);
const PIRATE_PATCH: Rgb = rgb(170, 140, 98);
const NAVY_HULL: Rgb = rgb(30, 50, 98);
const NAVY_STRIPE: Rgb = rgb(226, 178, 60);
const NAVY_SAIL: Rgb = rgb(250, 248, 238);
/// How much the canvas glows: sailcloth is thin, so the 3 pm sun shows
/// through it and its shaded side stays cream instead of taking the
/// sea's teal bounce (BQ1).
const CANVAS_GLOW: i32 = ONE * 11 / 20;
/// A furled roll's folds, a little darker.
const ROLL_GLOW: i32 = ONE * 2 / 5;

/// Decks and masts darkened (BO) so the pale swing ropes stand out
/// against them (were (184,142,92), (140,104,66), (122,86,52)).
const DECK: Rgb = rgb(132, 98, 64);
const DECK_DARK: Rgb = rgb(100, 72, 46);
const BELOW: Rgb = rgb(70, 50, 36);
const MAST: Rgb = rgb(82, 56, 36);
const RIGGING: Rgb = rgb(40, 32, 26);
const IRON: Rgb = rgb(36, 36, 40);
const PORT: Rgb = rgb(22, 16, 14);
const WINDOW: Rgb = rgb(255, 214, 130);
const GOLD: Rgb = rgb(238, 182, 46);
const GOLD_DARK: Rgb = rgb(170, 118, 30);
const BRONZE: Rgb = rgb(112, 118, 96);

const FLAG_BLACK: Rgb = rgb(18, 18, 20);
const BONE: Rgb = rgb(238, 232, 214);
const BANANA: Rgb = rgb(255, 206, 40);
const BANANA_TIP: Rgb = rgb(96, 64, 26);
const RED_FLAG: Rgb = rgb(190, 30, 30);
const ENSIGN: Rgb = rgb(100, 50, 150);
const WHITE: Rgb = rgb(244, 244, 238);

const STAVE: Rgb = rgb(150, 104, 60);
const STAVE_DARK: Rgb = rgb(110, 74, 42);
const STAVE_LIGHT: Rgb = rgb(196, 140, 76);
const MARKER: Rgb = rgb(255, 92, 30);
const BOUNCE_RED: Rgb = rgb(222, 40, 46);
const BOUNCE_BAND: Rgb = rgb(255, 214, 40);
const DRUM: Rgb = rgb(250, 236, 200);

const SAND: Rgb = rgb(226, 202, 148);
const SAND_WET: Rgb = rgb(184, 160, 112);
const ROCK: Rgb = rgb(120, 112, 104);
const PALM_TRUNK: Rgb = rgb(140, 108, 70);
const PALM_LEAF: Rgb = rgb(60, 150, 60);
const PALM_LEAF_DARK: Rgb = rgb(36, 112, 52);
const GEM_RED: Rgb = rgb(220, 30, 60);
const GEM_GREEN: Rgb = rgb(40, 200, 110);
const CHEST: Rgb = rgb(120, 70, 36);
const CHEST_DARK: Rgb = rgb(80, 46, 26);

const FUR: Rgb = rgb(110, 72, 44);
const FACE: Rgb = rgb(206, 166, 120);
const SHIRT_RED: Rgb = rgb(196, 40, 40);
const SHIRT_WHITE: Rgb = rgb(232, 228, 214);
const TROUSERS: Rgb = rgb(70, 60, 50);
const COAT: Rgb = rgb(34, 56, 120);
const BREECHES: Rgb = rgb(236, 234, 224);
const HAT: Rgb = rgb(24, 22, 24);
const STEEL: Rgb = rgb(200, 206, 214);
const MUSKET: Rgb = rgb(92, 60, 36);

const SMOKE: Rgb = rgb(236, 234, 228);
/// A burning ship's smoke at its root.
const SMOKE_DARK: Rgb = rgb(62, 58, 56);
/// Foam round a hull or a barrel: pale, a little of the sea in it.
const WAKE: Rgb = rgb(214, 240, 236);
const FLASH: Rgb = rgb(255, 170, 50);

// ---------------------------------------------------------------------------
// Helpers.

/// A shape of `size` with its middle at `at`, in frame `f`.
fn put(b: &mut Batch, f: &Affine, mesh: Mesh, at: V3, size: V3, c: Rgb, glow: i32) {
    b.push(
        mesh,
        &f.then(&Affine::translate(at)).then(&Affine::scale(size)),
        c,
        glow,
    );
}

/// The same, turned by `turn` about its middle.
fn put_turned(
    b: &mut Batch,
    f: &Affine,
    mesh: Mesh,
    at: V3,
    turn: &Affine,
    size: V3,
    c: Rgb,
    glow: i32,
) {
    b.push(
        mesh,
        &f.then(&Affine::translate(at))
            .then(turn)
            .then(&Affine::scale(size)),
        c,
        glow,
    );
}

/// A rod of `mesh`, `w` across, from `a` to `z` in frame `f`.
fn bar(b: &mut Batch, f: &Affine, mesh: Mesh, a: V3, z: V3, w: i32, c: Rgb, glow: i32) {
    if a.dist(z) > ONE / 100 {
        b.push(mesh, &f.then(&rod(a, z, w)), c, glow);
    }
}

/// Metres, from tenths (`dm(125)` is 12.5 m).
const fn dm(x: i32) -> i32 {
    x * (ONE / 10)
}

// ---------------------------------------------------------------------------
// Ships.
//
// A ship is drawn from the contract's `bay::Ship` (its kind, side, size,
// raised decks, masts with their yards and sails, jib, gaff sails and gun
// ports), so the course chooses every gap; `typical` gives a ship of each
// kind to start from. The drawing is in the ship's own frame (`ship`), +z
// the bow, +x larboard (the contract's starboard, `right_of(+z)`, is -x),
// the waterline at y 0; `ship_at` places it at `Ship.at`, `Ship.yaw`.

/// A ship of `kind` for `side` as the models are built at their typical
/// size, at `at` (the waterline's middle) facing `yaw`, every sail set and
/// no decks indexed yet (the course fills `decks` with its platforms).
/// Sizes: a galleon 44 m stern to stem (55 m with its bowsprit), a
/// frigate 36 m, a brig 26 m, a sloop 20 m.
pub fn typical(kind: ShipKind, side: Side, at: V3, yaw: i32) -> Ship {
    let mast = |along: i32, height: i32, yards: &[(i32, i32)], gaff: bool| Mast {
        along,
        height,
        yards: yards
            .iter()
            .map(|&(height, half_span)| bay::Yard {
                height,
                half_span,
                sail: Sail::Set,
            })
            .collect(),
        gaff: gaff.then_some(Sail::Set),
    };
    let (length, beam, deck, raised, bowsprit, masts, rows, per_row) = match kind {
        ShipKind::Galleon => (
            int(44),
            int(12),
            int(5),
            vec![
                (dm(140), dm(215), dm(75)),
                (-dm(220), -dm(60), int(8)),
                (-dm(220), -dm(150), dm(105)),
            ],
            int(11),
            vec![
                mast(
                    dm(125),
                    dm(285),
                    &[(dm(90), dm(90)), (dm(175), dm(70)), (dm(245), dm(48))],
                    false,
                ),
                mast(
                    0,
                    dm(330),
                    &[(dm(95), dm(105)), (dm(195), dm(82)), (dm(275), dm(55))],
                    false,
                ),
                mast(
                    -dm(130),
                    dm(230),
                    &[(dm(115), dm(65)), (dm(180), dm(48))],
                    true,
                ),
            ],
            2,
            10,
        ),
        ShipKind::Frigate => (
            int(36),
            int(10),
            int(4),
            vec![(dm(115), dm(175), dm(58)), (-dm(180), -dm(70), dm(62))],
            int(9),
            vec![
                mast(
                    dm(100),
                    dm(255),
                    &[(dm(80), dm(80)), (dm(155), dm(62)), (dm(220), dm(44))],
                    false,
                ),
                mast(
                    -dm(5),
                    dm(280),
                    &[(dm(85), dm(90)), (dm(170), dm(70)), (dm(240), dm(48))],
                    false,
                ),
                mast(
                    -dm(110),
                    dm(205),
                    &[(dm(95), dm(60)), (dm(160), dm(45))],
                    true,
                ),
            ],
            1,
            9,
        ),
        ShipKind::Brig => (
            int(26),
            int(8),
            int(3),
            vec![(dm(80), dm(115), dm(42)), (-dm(130), -dm(65), dm(44))],
            int(7),
            vec![
                mast(
                    dm(50),
                    dm(180),
                    &[(dm(65), dm(66)), (dm(130), dm(50))],
                    false,
                ),
                mast(
                    -dm(35),
                    dm(200),
                    &[(dm(70), dm(72)), (dm(140), dm(55))],
                    true,
                ),
            ],
            1,
            6,
        ),
        ShipKind::Sloop => (
            int(20),
            dm(65),
            dm(24),
            vec![(-dm(100), -dm(60), dm(32))],
            int(6),
            vec![mast(dm(20), dm(170), &[], true)],
            1,
            3,
        ),
    };
    let mut ports = Vec::new();
    let span = length * 60 / 100;
    for row in 0..rows {
        let up = (deck * 55 / 100 + dm(8) + row * dm(20)).min(deck - dm(4));
        for i in 0..per_row {
            let along = -span / 2 + span * i / (per_row - 1).max(1) - dm(10);
            for side in [-1, 1] {
                ports.push(Port { along, side, up });
            }
        }
    }
    Ship {
        kind,
        side,
        at,
        yaw,
        length,
        beam,
        deck,
        raised,
        bowsprit,
        masts,
        jib: Some(Sail::Set),
        ports,
        decks: Vec::new(),
    }
}

/// Draws ship `s` where it lies (`Ship.at`, `Ship.yaw`).
pub fn ship_at(b: &mut Batch, s: &Ship, tick: u32, k: u32) {
    ship(b, &pose(s.at, s.yaw, ONE), s, tick, k);
}

/// The hull's half width `along` the ship (the beam narrowing to the bow,
/// less to the stern).
pub fn half_beam(s: &Ship, along: i32) -> i32 {
    let t = fx::div(along, s.length / 2).clamp(-ONE, ONE);
    let narrow = if t > 0 {
        mul(mul(t, t), ONE * 55 / 100)
    } else {
        mul(mul(t, t), ONE * 22 / 100)
    };
    mul(s.beam / 2, ONE - narrow)
}

/// The deck's height over the water `along` the ship: the highest raised
/// deck there, else the main deck.
pub fn deck_at(s: &Ship, along: i32) -> i32 {
    s.raised
        .iter()
        .filter(|r| along >= r.0 && along <= r.1)
        .map(|r| r.2)
        .fold(s.deck, i32::max)
}

/// A yard in the ship's frame: its middle, half span, and how far its
/// square sail hangs when set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Spar {
    at: V3,
    half: i32,
    drop: i32,
}

/// The yards of mast `m`, lowest first, in the ship's frame: each sail
/// hangs to the next yard down, the lowest to just over the rail.
fn spars(s: &Ship, m: usize) -> Vec<Spar> {
    let mast = &s.masts[m];
    let rail = deck_at(s, mast.along) + dm(15);
    let mut out = Vec::new();
    let mut below = rail;
    for y in &mast.yards {
        let up = s.deck + y.height;
        out.push(Spar {
            at: v3(0, up, mast.along + dm(6)),
            half: y.half_span,
            drop: up - below - dm(6),
        });
        below = up;
    }
    out
}

struct Paint {
    hull: Rgb,
    wale: Rgb,
    sail: Rgb,
    trim: Rgb,
}

fn paint(side: Side) -> Paint {
    match side {
        Side::Pirate => Paint {
            hull: PIRATE_HULL,
            wale: PIRATE_WALE,
            sail: PIRATE_SAIL,
            trim: PIRATE_WALE,
        },
        Side::Navy => Paint {
            hull: NAVY_HULL,
            wale: NAVY_STRIPE,
            sail: NAVY_SAIL,
            trim: NAVY_STRIPE,
        },
    }
}

/// Ship `s` in frame `f` (its own frame: waterline at y 0, bow +z): the
/// hull, raised decks, gun ports with muzzles run out, stern and bow,
/// masts with tops, yards and each sail as its `Sail` says, gaff sails,
/// the jib, shrouds and stays, and its flags streaming with `tick`. `k`
/// varies the patches and the tatters.
pub fn ship(b: &mut Batch, f: &Affine, s: &Ship, tick: u32, k: u32) {
    ship_parts(b, f, s, tick, k, true, true);
}

/// Ship `s` without its flags (the art places it once, as still scenery):
/// everything `ship` draws but the flags and pennants.
pub fn ship_still(b: &mut Batch, f: &Affine, s: &Ship, k: u32) {
    ship_parts(b, f, s, 0, k, true, false);
}

/// Only ship `s`'s flags and pennants, streaming with `tick` (the art
/// draws them each frame near the eye, over `ship_still`).
pub fn ship_flags(b: &mut Batch, f: &Affine, s: &Ship, tick: u32, k: u32) {
    ship_parts(b, f, s, tick, k, false, true);
}

/// The parts of `ship`: the body (hull, rigging, sails) and the flags.
fn ship_parts(b: &mut Batch, f: &Affine, s: &Ship, tick: u32, k: u32, body: bool, flags: bool) {
    let p = paint(s.side);
    if body {
        hull_body(b, f, s, &p);
        castles(b, f, s, &p);
        guns(b, f, s);
    }
    stern(b, f, s, &p, tick, body, flags);
    if body {
        bow(b, f, s, &p);
    }
    let tallest = s.masts.iter().map(|m| m.height).max().unwrap_or(0);
    for (m, mast) in s.masts.iter().enumerate() {
        if !body {
            if flags && mast.height == tallest {
                let head = v3(0, s.deck + mast.height, mast.along);
                match s.side {
                    Side::Pirate => jolly_roger(b, f, head, s.beam * 45 / 100, tick, k),
                    Side::Navy => pennant(b, f, head, s.beam * 80 / 100, tick),
                }
            }
            continue;
        }
        let foot = deck_at(s, mast.along) - dm(5);
        let top = s.deck + mast.height;
        let w = s.beam * 8 / 100;
        let mut from = v3(0, foot, mast.along);
        let ys = spars(s, m);
        for (i, y) in ys.iter().enumerate() {
            let to = v3(0, y.at.y + dm(20), mast.along);
            let wi = w * (10 - i as i32 * 3).max(3) / 10;
            bar(b, f, Mesh::Taper, from, to, wi, MAST, 0);
            put(
                b,
                f,
                Mesh::Hex,
                v3(0, y.at.y + dm(16), mast.along),
                v3(wi * 4, dm(2), wi * 4),
                BELOW,
                0,
            );
            from = to;
        }
        bar(
            b,
            f,
            Mesh::Taper,
            from,
            v3(0, top, mast.along),
            w * 4 / 10,
            MAST,
            0,
        );
        put(
            b,
            f,
            Mesh::Ico,
            v3(0, top, mast.along),
            v3(w / 2, w / 2, w / 2),
            MAST,
            0,
        );
        shrouds(b, f, s, m);
        for (i, y) in ys.iter().enumerate() {
            let state = mast.yards[i].sail;
            yard_and_sail(
                b,
                f,
                y,
                state,
                &p,
                s.side,
                k.wrapping_add((m * 4 + i) as u32),
            );
        }
        if let Some(state) = mast.gaff {
            gaff_sail(b, f, s, m, state, &p, k.wrapping_add(m as u32 * 7));
        }
        if flags && mast.height == tallest {
            let head = v3(0, top, mast.along);
            match s.side {
                Side::Pirate => jolly_roger(b, f, head, s.beam * 45 / 100, tick, k),
                Side::Navy => pennant(b, f, head, s.beam * 80 / 100, tick),
            }
        }
    }
    if body {
        stays(b, f, s, &p, k);
    }
}

/// The hull: stations along the keel (about every 4 m), each a bevelled
/// block for the body and the keel, a coloured wale, the topsides up to
/// that station's deck, a two-tone deck plank and bulwarks with a cap.
fn hull_body(b: &mut Batch, f: &Affine, s: &Ship, p: &Paint) {
    let n = (s.length / int(4)).clamp(5, 11);
    let step = s.length / n;
    let keel = s.deck * 6 / 10;
    for i in 0..n {
        let z = -s.length / 2 + step / 2 + step * i;
        let hw = half_beam(s, z);
        let len = step * 115 / 100;
        let top = deck_at(s, z);
        let wale = s.deck * 55 / 100;
        put(
            b,
            f,
            Mesh::Bevel,
            v3(0, (wale - keel * 6 / 10) / 2, z),
            v3(hw * 2, wale + keel * 6 / 10, len),
            p.hull,
            0,
        );
        put(
            b,
            f,
            Mesh::Bevel,
            v3(0, -keel * 7 / 10, z),
            v3(hw, keel * 6 / 10, len),
            p.hull,
            0,
        );
        put(
            b,
            f,
            Mesh::Cube,
            v3(0, wale - dm(2), z),
            v3(hw * 2 + dm(2), dm(5), len),
            p.wale,
            0,
        );
        put(
            b,
            f,
            Mesh::Cube,
            v3(0, (wale + top) / 2, z),
            v3(hw * 2 * 95 / 100, top - wale, len),
            p.hull,
            0,
        );
        put(
            b,
            f,
            Mesh::Cube,
            v3(0, top + cm(2), z),
            v3(hw * 2 * 88 / 100, cm(6), len),
            if i % 2 == 0 { DECK } else { DECK_DARK },
            0,
        );
        for side in [-1, 1] {
            let x = side * (hw * 95 / 100 - dm(1));
            put(
                b,
                f,
                Mesh::Cube,
                v3(x, top + dm(5), z),
                v3(dm(2), dm(10), len),
                p.hull,
                0,
            );
            put(
                b,
                f,
                Mesh::Cube,
                v3(x, top + dm(10), z),
                v3(dm(3), cm(10), len),
                p.trim,
                0,
            );
        }
    }
}

/// The raised decks' bulkheads where they step up from a lower deck (lit
/// windows and a door, a rail along the edge, ladders up from the main
/// deck), the wheel on the quarterdeck, a hatch and the capstan amidships,
/// and on a galleon a ship's boat lashed in the waist.
fn castles(b: &mut Batch, f: &Affine, s: &Ship, p: &Paint) {
    // a raised deck's end at the stem or the stern needs no bulkhead
    let end = s.length / 2 - int(1);
    for r in &s.raised {
        for (z, dir) in [(r.0, -1), (r.1, 1)] {
            if z.abs() >= end {
                continue;
            }
            let lower = deck_at(s, z + dir * dm(3));
            if lower >= r.2 {
                continue;
            }
            let hw = half_beam(s, z) * 90 / 100;
            let face = z + dir * dm(2);
            put(
                b,
                f,
                Mesh::Cube,
                v3(0, (lower + r.2) / 2, z),
                v3(hw * 2, r.2 - lower, dm(2)),
                shade(p.hull, 130),
                0,
            );
            if r.2 - lower > dm(18) {
                put(
                    b,
                    f,
                    Mesh::Cube,
                    v3(0, lower + dm(9), face),
                    v3(dm(9), dm(18), cm(8)),
                    PORT,
                    0,
                );
            }
            for x in [-hw / 2, hw / 2] {
                put(
                    b,
                    f,
                    Mesh::Cube,
                    v3(x, lower + (r.2 - lower) * 6 / 10, face),
                    v3(dm(8), dm(6).min((r.2 - lower) / 3), cm(8)),
                    WINDOW,
                    ONE / 3,
                );
            }
            if lower == s.deck {
                // the stairs: a wedge whose tall end stands against the
                // bulkhead (the mesh's tall end is at −z), sloping down to
                // the main deck away from it. A raised deck aft of its
                // bulkhead (`dir` +1, the quarterdeck) leaves the wedge as
                // it is; one forward of it (the forecastle) turns it round.
                // (It was the other way round: the steps rose away from
                // the deck they led to, BO.)
                for side in [-1, 1] {
                    let x = side * hw * 70 / 100;
                    let h = r.2 - lower;
                    put_turned(
                        b,
                        f,
                        Mesh::Wedge,
                        v3(x, lower + h / 2, z + dir * h * 6 / 10),
                        &Affine::rot_y(if dir > 0 { 0 } else { TURN / 2 }),
                        v3(dm(12), h, h * 12 / 10),
                        DECK_DARK,
                        0,
                    );
                }
            }
            put(
                b,
                f,
                Mesh::Cube,
                v3(0, r.2 + dm(8), z - dir * dm(1)),
                v3(hw * 2, cm(8), cm(8)),
                p.trim,
                0,
            );
        }
    }
    // the wheel, on the foremost after raised deck (or the main deck's
    // after end)
    let (wz, wy) = s
        .raised
        .iter()
        .filter(|r| r.0 < 0 && r.1 < s.length / 4)
        .map(|r| (r.1 - dm(25), r.2))
        .max_by_key(|&(z, _)| z)
        .unwrap_or((-s.length * 4 / 10, s.deck));
    put_turned(
        b,
        f,
        Mesh::Hex,
        v3(0, wy + dm(10), wz),
        &Affine::rot_x(TURN / 4),
        v3(dm(13), dm(1), dm(13)),
        MAST,
        0,
    );
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, wy + dm(5), wz),
        v3(dm(3), dm(10), dm(4)),
        BELOW,
        0,
    );
    // a hatch grating and the capstan amidships
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, s.deck + cm(8), -dm(30)),
        v3(s.beam * 3 / 10, cm(16), dm(30)),
        BELOW,
        0,
    );
    put(
        b,
        f,
        Mesh::Hex,
        v3(0, s.deck + dm(5), dm(35)),
        v3(dm(10), dm(10), dm(10)),
        MAST,
        0,
    );
    if s.kind == ShipKind::Galleon {
        put(
            b,
            f,
            Mesh::Barrel,
            v3(0, s.deck + dm(7), dm(65)),
            v3(dm(18), dm(8), dm(40)),
            shade(DECK, 80),
            0,
        );
    }
}

/// The gun ports (`Ship.ports`): a dark port in the side with a muzzle run
/// out of it, and a painted strip along each row.
fn guns(b: &mut Batch, f: &Affine, s: &Ship) {
    let p = paint(s.side);
    let mut rows: Vec<i32> = s.ports.iter().map(|q| q.up).collect();
    rows.sort_unstable();
    rows.dedup();
    let span = s.length * 56 / 100;
    for (i, &up) in rows.iter().enumerate() {
        for side in [-1, 1] {
            put(
                b,
                f,
                Mesh::Cube,
                v3(-side * half_beam(s, 0) * 95 / 100, up, -dm(10)),
                v3(cm(10), dm(9), span),
                if i % 2 == 0 { p.wale } else { p.hull },
                0,
            );
        }
    }
    for q in &s.ports {
        // the contract's starboard (+1) is this frame's -x
        let x = -q.side * half_beam(s, q.along) * 95 / 100;
        put(
            b,
            f,
            Mesh::Cube,
            v3(x, q.up, q.along),
            v3(cm(12), dm(6), dm(7)),
            PORT,
            0,
        );
        bar(
            b,
            f,
            Mesh::Hex,
            v3(x, q.up, q.along),
            v3(x - q.side * dm(9), q.up + cm(4), q.along),
            dm(3),
            IRON,
            0,
        );
    }
}

/// The stern: a transom with rows of lit windows and gilded bands,
/// lanterns on the taffrail, and the ensign (navy) or a red flag (pirate)
/// on a staff.
#[allow(clippy::too_many_arguments)]
fn stern(b: &mut Batch, f: &Affine, s: &Ship, p: &Paint, tick: u32, body: bool, flags: bool) {
    if !body {
        if flags {
            stern_flag(b, f, s, tick);
        }
        return;
    }
    let z = -s.length / 2 - dm(3);
    let top = deck_at(s, -s.length / 2 + dm(10));
    let hw = half_beam(s, -s.length / 2) * 90 / 100;
    put(
        b,
        f,
        Mesh::Bevel,
        v3(0, (top + dm(10) + s.deck * 3 / 10) / 2, z),
        v3(hw * 2, top + dm(10) - s.deck * 3 / 10, dm(8)),
        shade(p.hull, 120),
        0,
    );
    let rows = ((top - s.deck) / int(2) + 1).clamp(1, 3);
    for r in 0..rows {
        let y = s.deck - dm(10) + r * (top - s.deck + dm(10)) / rows;
        let n = if hw > int(4) { 5 } else { 3 };
        for i in 0..n {
            let x = (i * 2 - (n - 1)) * hw / (n + 1);
            put(
                b,
                f,
                Mesh::Cube,
                v3(x, y, z - dm(4)),
                v3(dm(7), dm(8), cm(8)),
                WINDOW,
                ONE / 2,
            );
        }
        put(
            b,
            f,
            Mesh::Cube,
            v3(0, y - dm(7), z - dm(5)),
            v3(hw * 2, cm(14), cm(10)),
            GOLD_DARK,
            0,
        );
    }
    for x in [-hw * 8 / 10, 0, hw * 8 / 10] {
        let at = v3(x, top + dm(18), z);
        bar(
            b,
            f,
            Mesh::Hex,
            at.add(v3(0, -dm(8), 0)),
            at,
            cm(10),
            IRON,
            0,
        );
        put(
            b,
            f,
            Mesh::Octa,
            at,
            v3(dm(5), dm(7), dm(5)),
            WINDOW,
            ONE * 3 / 4,
        );
    }
    let foot = v3(0, top, z + dm(3));
    let head = foot.add(v3(0, dm(55), -dm(20)));
    bar(b, f, Mesh::Hex, foot, head, cm(12), MAST, 0);
    if flags {
        stern_flag(b, f, s, tick);
    }
}

/// The ensign (navy) or red flag (pirate) on the stern's staff.
fn stern_flag(b: &mut Batch, f: &Affine, s: &Ship, tick: u32) {
    let z = -s.length / 2 - dm(3);
    let top = deck_at(s, -s.length / 2 + dm(10));
    let head = v3(0, top, z + dm(3)).add(v3(0, dm(55), -dm(20)));
    let size = s.beam * 35 / 100;
    match s.side {
        Side::Navy => ensign(b, f, head, size, tick),
        Side::Pirate => plain_flag(b, f, head, size * 8 / 10, RED_FLAG, tick),
    }
}

/// The bow: a pointed beak, the bowsprit, and a figurehead (a gilded ape
/// on the navy's ships, a bone ape skull on the pirates').
fn bow(b: &mut Batch, f: &Affine, s: &Ship, p: &Paint) {
    let z = s.length / 2;
    let hw = half_beam(s, z);
    let beak = s.beam * 45 / 100;
    let keel = s.deck * 6 / 10;
    let beak_y = (s.deck * 6 / 10 - keel / 2) / 2 + dm(4);
    let beak_h = s.deck * 6 / 10 + keel / 2;
    put_turned(
        b,
        f,
        Mesh::Spike,
        v3(
            0,
            (s.deck * 6 / 10 - keel / 2) / 2 + dm(4),
            z + beak / 2 - dm(5),
        ),
        &Affine::rot_x(TURN / 4),
        v3(hw * 2, beak, beak_h),
        p.hull,
        0,
    );
    if s.bowsprit > 0 {
        let fore = deck_at(s, z - dm(10));
        let root = v3(0, fore - dm(5), z - dm(10));
        let tip = v3(0, fore + s.bowsprit * 4 / 10, z + s.bowsprit);
        bar(b, f, Mesh::Taper, root, tip, s.beam * 5 / 100, MAST, 0);
    }
    // the figurehead rides the beak's upper edge, a third of the way out
    let fig = v3(0, beak_y + beak_h * 30 / 100, z + beak * 30 / 100);
    let (c, glow) = match s.side {
        Side::Navy => (GOLD, ONE / 6),
        Side::Pirate => (BONE, 0),
    };
    let size = s.beam / 10;
    put(
        b,
        f,
        Mesh::Ico,
        fig,
        v3(size, size * 11 / 10, size),
        c,
        glow,
    );
    for x in [-1, 1] {
        put(
            b,
            f,
            Mesh::Ico,
            fig.add(v3(x * size * 6 / 10, size / 4, -size / 6)),
            v3(size * 4 / 10, size * 4 / 10, size / 4),
            c,
            glow,
        );
    }
    if s.side == Side::Pirate {
        for x in [-1, 1] {
            put(
                b,
                f,
                Mesh::Cube,
                fig.add(v3(x * size / 4, size / 6, size * 4 / 10)),
                v3(size / 4, size / 4, cm(6)),
                PORT,
                0,
            );
        }
    }
}

/// The shrouds: two lines a side from the rail up to mast `m`'s lowest
/// top, thin and dark so the vines stand out (MAPS art lessons).
fn shrouds(b: &mut Batch, f: &Affine, s: &Ship, m: usize) {
    let mast = &s.masts[m];
    let deck = deck_at(s, mast.along);
    let top = mast
        .yards
        .first()
        .map(|y| s.deck + y.height + dm(16))
        .unwrap_or(s.deck + mast.height * 7 / 10);
    for x in [-1, 1] {
        for dz in [-dm(18), 0] {
            let z = mast.along + dz;
            let foot = v3(x * half_beam(s, z) * 95 / 100, deck + dm(10), z);
            bar(
                b,
                f,
                Mesh::Cube,
                foot,
                v3(x * dm(5), top, mast.along),
                cm(7),
                RIGGING,
                0,
            );
        }
    }
}

/// The stays (from each mast's head down to the next mast forward's
/// lowest top, the foremost's to the bowsprit's tip) and the jib under
/// the forestay.
fn stays(b: &mut Batch, f: &Affine, s: &Ship, p: &Paint, k: u32) {
    let low = |m: &Mast| {
        m.yards
            .first()
            .map(|y| s.deck + y.height + dm(16))
            .unwrap_or(s.deck + m.height * 6 / 10)
    };
    // `masts` is fore first
    for w in s.masts.windows(2) {
        let (fore, aft) = (&w[0], &w[1]);
        bar(
            b,
            f,
            Mesh::Cube,
            v3(0, s.deck + aft.height - dm(5), aft.along),
            v3(0, low(fore), fore.along),
            cm(7),
            RIGGING,
            0,
        );
    }
    let Some(fore) = s.masts.first() else {
        return;
    };
    let z = s.length / 2;
    let bow_deck = deck_at(s, z - dm(10));
    let tip = if s.bowsprit > 0 {
        v3(0, bow_deck + s.bowsprit * 4 / 10, z + s.bowsprit)
    } else {
        v3(0, bow_deck + dm(10), z)
    };
    let head = v3(0, s.deck + fore.height - dm(5), fore.along);
    bar(b, f, Mesh::Cube, head, tip, cm(7), RIGGING, 0);
    // the jib: a right triangle of canvas, its clew aft at the foot, its
    // slope along the forestay
    let Some(jib) = s.jib else {
        return;
    };
    let foot = bow_deck + dm(25);
    let clew_z = fore.along + (tip.z - fore.along) / 4;
    let along_stay = |zz: i32| {
        let t = fx::div(zz - head.z, tip.z - head.z).clamp(0, ONE);
        head.y + mul(tip.y - head.y, t)
    };
    let top = along_stay(clew_z);
    let tack_z = tip.z - (tip.z - fore.along) / 6;
    if top <= foot + int(1) || tack_z <= clew_z {
        return;
    }
    let (h, l) = (top - foot, tack_z - clew_z);
    match jib {
        Sail::Set => put(
            b,
            f,
            Mesh::Wedge,
            v3(0, foot + h / 2, clew_z + l / 2),
            v3(dm(2), h, l),
            p.sail,
            CANVAS_GLOW,
        ),
        Sail::Torn => {
            put(
                b,
                f,
                Mesh::Wedge,
                v3(0, foot + h * 7 / 10, clew_z + l * 3 / 10),
                v3(dm(2), h * 6 / 10, l * 6 / 10),
                p.sail,
                CANVAS_GLOW,
            );
            put_turned(
                b,
                f,
                Mesh::Shard,
                v3(0, foot + h * 3 / 10, clew_z + l / 6),
                &Affine::rot_x(deg(pick(k, 33, -15, 15))),
                v3(dm(2), h / 4, l / 6),
                shade(p.sail, 90),
                CANVAS_GLOW,
            );
        }
        Sail::Furled => bar(
            b,
            f,
            Mesh::Hex,
            v3(0, foot + dm(3), clew_z + l / 2),
            tip,
            dm(4),
            shade(p.sail, 92),
            CANVAS_GLOW,
        ),
        Sail::Missing => {}
    }
}

/// Mast `m`'s gaff sail (a spanker or a sloop's mainsail): a boom and a
/// gaff reaching aft from the mast, the canvas between them as `state`
/// says, over the deck aft of the mast.
fn gaff_sail(b: &mut Batch, f: &Affine, s: &Ship, m: usize, state: Sail, p: &Paint, k: u32) {
    let mast = &s.masts[m];
    let deck = deck_at(s, mast.along - dm(10));
    let boom_y = deck + dm(18);
    let reach = (mast.height * 45 / 100).min(mast.along + s.length / 2 - dm(5));
    if reach < int(2) {
        return;
    }
    let lowest = mast
        .yards
        .first()
        .map(|y| s.deck + y.height)
        .unwrap_or(s.deck + mast.height * 8 / 10);
    let gaff_y = (lowest - dm(15)).max(boom_y + int(3));
    let (z0, z1) = (mast.along - dm(3), mast.along - reach);
    bar(
        b,
        f,
        Mesh::Hex,
        v3(0, boom_y, z0),
        v3(0, boom_y, z1),
        dm(2),
        MAST,
        0,
    );
    bar(
        b,
        f,
        Mesh::Hex,
        v3(0, gaff_y, z0),
        v3(0, gaff_y + dm(10), z1 - dm(5)),
        dm(2),
        MAST,
        0,
    );
    let h = gaff_y - boom_y;
    let mid = v3(0, (gaff_y + boom_y) / 2, (z0 + z1) / 2);
    match state {
        Sail::Set => put(
            b,
            f,
            Mesh::Cube,
            mid,
            v3(dm(2), h, reach - dm(4)),
            p.sail,
            CANVAS_GLOW,
        ),
        Sail::Furled => bar(
            b,
            f,
            Mesh::Hex,
            v3(0, boom_y + dm(3), z0),
            v3(0, boom_y + dm(3), z1),
            dm(6),
            shade(p.sail, 92),
            CANVAS_GLOW,
        ),
        Sail::Torn => {
            for (i, t) in [15, 85].into_iter().enumerate() {
                let z = z0 - (reach * t / 100);
                let len = h * pick(k, 34 + i as u32, 40, 80) / 100;
                put(
                    b,
                    f,
                    Mesh::Cube,
                    v3(0, gaff_y - len / 2, z),
                    v3(dm(2), len, reach / 9),
                    p.sail,
                    CANVAS_GLOW,
                );
            }
        }
        Sail::Missing => {}
    }
}

/// A yard across its mast and, under it, its sail as `state` says.
fn yard_and_sail(b: &mut Batch, f: &Affine, y: &Spar, state: Sail, p: &Paint, side: Side, k: u32) {
    let w = y.half / 14;
    bar(
        b,
        f,
        Mesh::Hex,
        y.at.add(v3(-y.half, 0, 0)),
        y.at.add(v3(y.half, 0, 0)),
        w.max(dm(2)),
        MAST,
        0,
    );
    let span = y.half * 2 * 92 / 100;
    let top = y.at.y - dm(2);
    let z = y.at.z + dm(3);
    let sail = p.sail;
    match state {
        Sail::Set => {
            // the canvas and a belly bulging forward
            put(
                b,
                f,
                Mesh::Cube,
                v3(0, top - y.drop / 2, z + dm(1)),
                v3(span, y.drop, dm(2)),
                sail,
                CANVAS_GLOW,
            );
            put(
                b,
                f,
                Mesh::Cube,
                v3(0, top - y.drop * 45 / 100, z + dm(5)),
                v3(span * 80 / 100, y.drop * 70 / 100, dm(3)),
                shade(sail, 97),
                CANVAS_GLOW,
            );
            if side == Side::Pirate && !k.is_multiple_of(3) {
                // a patch
                let px = (pick(k, 11, -30, 30)) * span / 100;
                let py = top - y.drop * pick(k, 12, 30, 70) / 100;
                put(
                    b,
                    f,
                    Mesh::Cube,
                    v3(px, py, z + dm(7)),
                    v3(span / 6, y.drop / 5, cm(4)),
                    PIRATE_PATCH,
                    ONE / 4,
                );
            }
            if side == Side::Navy {
                // a reef band across
                put(
                    b,
                    f,
                    Mesh::Cube,
                    v3(0, top - y.drop / 5, z + dm(7)),
                    v3(span * 82 / 100, cm(14), cm(4)),
                    shade(sail, 88),
                    CANVAS_GLOW,
                );
            }
        }
        Sail::Furled => {
            // a rolled bundle along the yard (BQ1: one even bar read as a
            // slab): fat in the bunt at the middle, thinning toward the
            // yardarms, tied with two dark gaskets
            let fat = w * 3 / 2 + dm(4);
            let thin = w + dm(2);
            let y0 = top - fat / 2 + dm(1);
            let roll = shade(sail, 93);
            bar(
                b,
                f,
                Mesh::Hex,
                v3(-y.half / 4, y0, z + dm(1)),
                v3(y.half / 4, y0, z + dm(1)),
                fat,
                roll,
                ROLL_GLOW,
            );
            for s in [-1, 1] {
                bar(
                    b,
                    f,
                    Mesh::Taper,
                    v3(s * y.half / 5, y0 + (fat - thin) / 4, z),
                    v3(s * y.half * 9 / 10, top - thin / 3, z),
                    thin,
                    shade(sail, 88),
                    ROLL_GLOW,
                );
                let x = s * y.half * 2 / 5;
                bar(
                    b,
                    f,
                    Mesh::Hex,
                    v3(x - cm(6), y0, z),
                    v3(x + cm(6), y0, z),
                    thin + cm(6),
                    RIGGING,
                    0,
                );
            }
        }
        Sail::Torn => {
            // a ragged band left along the yard
            put(
                b,
                f,
                Mesh::Cube,
                v3(0, top - y.drop / 20, z + dm(1)),
                v3(span, y.drop / 10, dm(2)),
                sail,
                CANVAS_GLOW,
            );
            // ragged strips hanging at each end, of uneven length and
            // width, a little apart and a little askew, each ending in a
            // tattered point
            for s in [-1, 1] {
                let mut edge = span / 2;
                for j in 0..3 {
                    let salt = 20 + j as u32 + (s + 1) as u32 * 5;
                    let wide = span * pick(k, salt + 50, 4, 6) / 100;
                    let x = s * (edge - wide / 2);
                    edge -= wide + span * pick(k, salt + 60, 1, 2) / 100;
                    let len = y.drop * pick(k, salt, 40, 95) / 100;
                    let lean = Affine::rot_z(deg(pick(k, salt + 70, -4, 4)));
                    put_turned(
                        b,
                        f,
                        Mesh::Cube,
                        v3(x, top - len / 2, z + dm(1)),
                        &lean,
                        v3(wide, len, dm(2)),
                        shade(sail, 100 - j * 4),
                        CANVAS_GLOW,
                    );
                    put_turned(
                        b,
                        f,
                        Mesh::Shard,
                        v3(x, top - len - dm(4), z + dm(1)),
                        &Affine::rot_z(deg(pick(k, 30 + j as u32, -20, 20))),
                        v3(wide * 9 / 10, dm(10), dm(2)),
                        shade(sail, 94),
                        CANVAS_GLOW,
                    );
                }
            }
            // short tatters along the band
            for j in 0..4 {
                let x = (j * 2 - 3) * span / 10 + pick(k, 40 + j as u32, -5, 5) * span / 100;
                put_turned(
                    b,
                    f,
                    Mesh::Shard,
                    v3(x, top - y.drop / 10 - dm(4), z + dm(1)),
                    &Affine::rot_z(deg(pick(k, 43 + j as u32, -18, 18))),
                    v3(
                        span / 16,
                        dm(10) + pick(k, 47 + j as u32, 0, 8) * cm(1),
                        dm(2),
                    ),
                    shade(sail, 95),
                    CANVAS_GLOW,
                );
            }
        }
        Sail::Missing => {
            // a scrap left at one end
            let s = if k.is_multiple_of(2) { 1 } else { -1 };
            put_turned(
                b,
                f,
                Mesh::Shard,
                v3(s * span * 4 / 10, top - dm(8), z),
                &Affine::rot_z(deg(10 * s)),
                v3(span / 10, dm(16), dm(2)),
                shade(sail, 90),
                CANVAS_GLOW,
            );
        }
    }
}

/// A flag on a staff whose head is `head`, `size` tall and half again as
/// long, streaming toward +z (the wind is astern), its fly flapping with
/// `tick`. Returns the frame of the flag's flat part (hoist half): origin
/// at its middle, +z along the flag, +y up, x through it.
fn flag_frame(b: &mut Batch, f: &Affine, head: V3, size: i32, c: Rgb, tick: u32) -> Affine {
    let len = size * 3 / 2;
    let sway = mul(wave(tick, 2, 0), deg(8));
    let g = f
        .then(&Affine::translate(head.add(v3(0, -size / 2, 0))))
        .then(&Affine::rot_y(sway));
    // the hoist part, flat, carrying the device
    put(
        b,
        &g,
        Mesh::Cube,
        v3(0, 0, len * 3 / 10),
        v3(cm(6), size, len * 6 / 10),
        c,
        0,
    );
    // the fly, rippling
    let ripple = mul(wave(tick, 1, deg(90)), deg(18));
    let fly = g
        .then(&Affine::translate(v3(0, 0, len * 6 / 10)))
        .then(&Affine::rot_y(ripple));
    put(
        b,
        &fly,
        Mesh::Cube,
        v3(0, 0, len * 2 / 10),
        v3(cm(6), size * 96 / 100, len * 4 / 10 + cm(4)),
        shade(c, 92),
        0,
    );
    g.then(&Affine::translate(v3(0, 0, len * 3 / 10)))
}

/// A plain flag of colour `c`.
fn plain_flag(b: &mut Batch, f: &Affine, head: V3, size: i32, c: Rgb, tick: u32) {
    flag_frame(b, f, head, size, c, tick);
}

/// The pirates' black flag at the masthead: a grinning ape skull over two
/// crossed bananas, on both faces.
fn jolly_roger(b: &mut Batch, f: &Affine, head: V3, size: i32, tick: u32, _k: u32) {
    let pole_top = head.add(v3(0, size * 6 / 10, 0));
    bar(b, f, Mesh::Hex, head, pole_top, cm(12), MAST, 0);
    let g = flag_frame(b, f, pole_top, size, FLAG_BLACK, tick);
    let s = size;
    for side in [-1, 1] {
        let x = side * cm(5);
        // crossed bananas behind the skull
        for t in [-1, 1] {
            let a = v3(x, -s * 32 / 100, -t * s * 28 / 100);
            let z = v3(x, s * 2 / 100, t * s * 28 / 100);
            bar(b, &g, Mesh::Hex, a, z, s * 8 / 100, BANANA, ONE / 4);
            put(
                b,
                &g,
                Mesh::Cube,
                z,
                v3(cm(4), s * 6 / 100, s * 6 / 100),
                BANANA_TIP,
                0,
            );
        }
        // the ape skull: a wide cranium, a muzzle, dark eyes and teeth
        put(
            b,
            &g,
            Mesh::Ico,
            v3(x, s * 12 / 100, 0),
            v3(cm(4), s * 34 / 100, s * 38 / 100),
            BONE,
            ONE / 4,
        );
        put(
            b,
            &g,
            Mesh::Ico,
            v3(x, -s * 6 / 100, 0),
            v3(cm(4), s * 18 / 100, s * 26 / 100),
            BONE,
            ONE / 4,
        );
        for e in [-1, 1] {
            put(
                b,
                &g,
                Mesh::Cube,
                v3(x * 13 / 10, s * 14 / 100, e * s * 8 / 100),
                v3(cm(3), s * 9 / 100, s * 9 / 100),
                FLAG_BLACK,
                0,
            );
        }
        put(
            b,
            &g,
            Mesh::Cube,
            v3(x * 13 / 10, -s * 8 / 100, 0),
            v3(cm(3), cm(4), s * 16 / 100),
            FLAG_BLACK,
            0,
        );
    }
}

/// The navy's ensign: a made-up flag (no real nation's), a purple field
/// with a gold chevron from the hoist and a white ring in the fly half.
fn ensign(b: &mut Batch, f: &Affine, head: V3, size: i32, tick: u32) {
    let g = flag_frame(b, f, head, size, ENSIGN, tick);
    let s = size;
    let len = s * 3 / 2;
    for side in [-1, 1] {
        let x = side * cm(5);
        let hoist = -len * 3 / 10;
        for t in [-1, 1] {
            bar(
                b,
                &g,
                Mesh::Cube,
                v3(x, t * s * 45 / 100, hoist),
                v3(x, 0, hoist + s * 45 / 100),
                s * 10 / 100,
                GOLD,
                0,
            );
        }
        let ring = v3(x, 0, len * 18 / 100);
        put_turned(
            b,
            &g,
            Mesh::Hex,
            ring,
            &Affine::rot_z(TURN / 4),
            v3(s * 38 / 100, cm(4), s * 38 / 100),
            WHITE,
            0,
        );
        put_turned(
            b,
            &g,
            Mesh::Hex,
            ring.add(v3(side * cm(1), 0, 0)),
            &Affine::rot_z(TURN / 4),
            v3(s * 24 / 100, cm(4), s * 24 / 100),
            ENSIGN,
            0,
        );
    }
}

/// The navy's masthead pennant: a long tapering streamer, purple then
/// gold, waving with `tick`.
fn pennant(b: &mut Batch, f: &Affine, head: V3, len: i32, tick: u32) {
    let pole_top = head.add(v3(0, dm(25), 0));
    bar(b, f, Mesh::Hex, head, pole_top, cm(12), MAST, 0);
    let n = 4;
    let mut at = pole_top.add(v3(0, -dm(3), 0));
    for i in 0..n {
        let a = mul(wave(tick, 1, i * deg(70)), deg(20));
        let d = v3(sin(a), -ONE / 10, cos(a)).norm();
        let next = at.add(d.scale(len / n));
        let w = dm(5) * (n - i) / n + cm(6);
        let c = if i % 2 == 0 { ENSIGN } else { GOLD };
        b.push(
            Mesh::Cube,
            &f.then(&rod(at, next, ONE))
                .then(&Affine::scale(v3(cm(5), ONE, w))),
            c,
            0,
        );
        at = next;
    }
}

// ---------------------------------------------------------------------------
// Barrels.

/// How a floating barrel bobs at `tick`: its rise (metres) and its tilt.
pub(crate) fn bob(tick: u32, k: u32) -> (i32, i32) {
    let phase = pick(k, 1, 0, TURN);
    (
        mul(wave(tick, 3, phase), cm(14)),
        mul(wave(tick, 4, phase + TURN / 4), deg(6)),
    )
}

/// A boundary barrel: a tarred wooden barrel floating upright (its middle
/// at the waterline, about 1 m showing), a pole out of its top with a
/// small orange-and-white flag, bobbing with `tick`. A soft wall's marker.
pub fn boundary_barrel(b: &mut Batch, f: &Affine, tick: u32, k: u32) {
    let (rise, tilt) = bob(tick, k);
    let g = f
        .then(&Affine::translate(v3(0, rise, 0)))
        .then(&Affine::rot_z(tilt));
    // the body: honey oak, bellied (BP4: dark and squat it read as a stone)
    let (bot, mid, top) = (-dm(6), dm(4), dm(14));
    let (r_end, r_mid) = (dm(7), dm(8) + cm(4));
    put(
        b,
        &g,
        Mesh::Barrel,
        v3(0, mid, 0),
        v3(r_mid * 2, top - bot, r_mid * 2),
        STAVE_LIGHT,
        ONE / 12,
    );
    // the staves' seams, dark lines up the belly (four, so two or three
    // show from any side)
    for i in 0..4 {
        let d = facing(i * TURN / 4 + TURN / 8, 0);
        let lo = d.scale(r_mid).add(v3(0, mid - dm(5), 0));
        let hi = d.scale(r_mid).add(v3(0, mid + dm(5), 0));
        bar(b, &g, Mesh::Cube, lo, hi, cm(5), STAVE_DARK, 0);
    }
    // two black iron hoops over the water, standing proud of the wood
    for (y, r) in [(top - dm(2), r_end + cm(8)), (mid + dm(5), r_mid + cm(2))] {
        put(
            b,
            &g,
            Mesh::Hex,
            v3(0, y, 0),
            v3(r * 2, cm(14), r * 2),
            IRON,
            0,
        );
    }
    // the head: a paler lid
    put(
        b,
        &g,
        Mesh::Hex,
        v3(0, top, 0),
        v3(r_end * 2 - cm(10), cm(6), r_end * 2 - cm(10)),
        shade(STAVE_LIGHT, 115),
        0,
    );
    let pole = v3(0, top + dm(50), 0);
    bar(b, &g, Mesh::Hex, v3(0, top, 0), pole, cm(10), MAST, 0);
    // the flag: a swallow-tailed pennant, orange with a white bar, big
    // enough to read down the line
    let a = mul(wave(tick, 1, pick(k, 2, 0, TURN)), deg(25));
    let fl = g
        .then(&Affine::translate(pole.add(v3(0, -dm(4), 0))))
        .then(&Affine::rot_y(a));
    put(
        b,
        &fl,
        Mesh::Cube,
        v3(0, 0, dm(5)),
        v3(cm(4), dm(7), dm(10)),
        MARKER,
        ONE / 5,
    );
    put(
        b,
        &fl,
        Mesh::Cube,
        v3(0, 0, dm(5)),
        v3(cm(5), cm(16), dm(10)),
        WHITE,
        0,
    );
    for t in [-1, 1] {
        put_turned(
            b,
            &fl,
            Mesh::Wedge,
            v3(0, t * dm(2), dm(11)),
            &Affine::rot_z(TURN / 4),
            v3(dm(3), cm(4), dm(4)),
            MARKER,
            ONE / 5,
        );
    }
}

/// A bouncy barrel: a big red barrel banded in yellow with a cream drum
/// skin on top, floating upright (its top about 1.6 m over the water),
/// squashed by `squash` (0 at rest, `ONE` flattened by a third) as an ape
/// bounces off it.
pub fn bounce_barrel(b: &mut Batch, f: &Affine, tick: u32, k: u32, squash: i32) {
    let (rise, tilt) = bob(tick, k);
    let sy = ONE - mul(squash.clamp(0, ONE), ONE / 3);
    let sx = ONE + mul(squash.clamp(0, ONE), ONE / 6);
    let g = f
        .then(&Affine::translate(v3(0, rise, 0)))
        .then(&Affine::rot_z(tilt / 2))
        .then(&Affine::scale(v3(sx, sy, sx)));
    put(
        b,
        &g,
        Mesh::Barrel,
        v3(0, dm(3), 0),
        v3(dm(26), dm(28), dm(26)),
        BOUNCE_RED,
        0,
    );
    for y in [-dm(5), dm(3), dm(11)] {
        put(
            b,
            &g,
            Mesh::Hex,
            v3(0, y, 0),
            v3(dm(26), dm(2), dm(26)),
            BOUNCE_BAND,
            ONE / 6,
        );
    }
    // the drum skin, bulging
    put(
        b,
        &g,
        Mesh::Hex,
        v3(0, dm(17), 0),
        v3(dm(22), dm(1), dm(22)),
        BOUNCE_BAND,
        0,
    );
    put(
        b,
        &g,
        Mesh::Geo,
        v3(0, dm(17), 0),
        v3(dm(20), dm(5), dm(20)),
        DRUM,
        ONE / 8,
    );
    // a yellow banana painted on each side
    for s in [-1, 1] {
        let at = v3(s * dm(13), dm(7), 0);
        bar(
            b,
            &g,
            Mesh::Hex,
            at.add(v3(0, -dm(2), -dm(3))),
            at.add(v3(0, dm(1), dm(1))),
            dm(2),
            BANANA,
            ONE / 6,
        );
        bar(
            b,
            &g,
            Mesh::Hex,
            at.add(v3(0, dm(1), dm(1))),
            at.add(v3(0, dm(3), dm(4))),
            dm(2),
            BANANA,
            ONE / 6,
        );
    }
}

// ---------------------------------------------------------------------------
// The start dock.

/// How high the dock's planks stand over the water.
pub const DOCK_TOP: i32 = dm(18);

/// The start dock: a wooden pier `len` long (from z 0 out along +z) and
/// `w` wide, its planks at `DOCK_TOP`, on piles, with bollards, rope coils,
/// a stack of crates and barrels, a lantern post at its end, and a strip
/// of beach and a boathouse behind it (z < 0).
pub fn dock(b: &mut Batch, f: &Affine, len: i32, w: i32, k: u32) {
    // planks across, in pairs of two tones
    let pitch = dm(12);
    let n = len / pitch;
    for i in 0..n {
        let z = pitch / 2 + i * pitch;
        put(
            b,
            f,
            Mesh::Cube,
            v3(cm(pick(k + i as u32, 3, -6, 6)), DOCK_TOP - cm(6), z),
            v3(w, cm(14), pitch - cm(6)),
            if i % 2 == 0 { DECK } else { DECK_DARK },
            0,
        );
    }
    // stringers under the planks
    for s in [-1, 1] {
        put(
            b,
            f,
            Mesh::Cube,
            v3(s * (w / 2 - dm(3)), DOCK_TOP - dm(3), len / 2),
            v3(dm(3), dm(3), len),
            BELOW,
            0,
        );
    }
    // the piles, standing proud at the edges as posts
    let piles = len / int(4);
    for i in 0..=piles {
        let z = i * int(4) + dm(2);
        for s in [-1, 1] {
            let tall = if i % 2 == 0 { dm(9) } else { dm(2) };
            bar(
                b,
                f,
                Mesh::Hex,
                v3(s * (w / 2 + dm(1)), -int(2), z),
                v3(s * (w / 2 + dm(1)), DOCK_TOP + tall, z),
                dm(4),
                STAVE_DARK,
                0,
            );
        }
    }
    // bollards and rope coils near the end
    for s in [-1, 1] {
        let at = v3(s * (w / 2 - dm(6)), DOCK_TOP + dm(3), len - int(3));
        put(b, f, Mesh::Hex, at, v3(dm(4), dm(6), dm(4)), IRON, 0);
        put(
            b,
            f,
            Mesh::Hex,
            at.add(v3(0, dm(3), 0)),
            v3(dm(6), cm(10), dm(6)),
            IRON,
            0,
        );
        let coil = v3(-s * dm(5), DOCK_TOP + cm(10), len - int(6) + s * int(2));
        put(
            b,
            f,
            Mesh::Hex,
            coil,
            v3(dm(10), dm(2), dm(10)),
            rgb(196, 168, 112),
            0,
        );
        put(
            b,
            f,
            Mesh::Hex,
            coil.add(v3(0, cm(4), 0)),
            v3(dm(5), dm(2), dm(5)),
            BELOW,
            0,
        );
    }
    // crates and barrels stacked by the shore end
    let stack = v3(w / 2 - dm(9), DOCK_TOP, int(5));
    for (i, (dx, dy, dz)) in [(0, 0, 0), (0, 0, 12), (0, 11, 5)].into_iter().enumerate() {
        put_turned(
            b,
            f,
            Mesh::Bevel,
            stack.add(v3(dm(dx), dm(dy) + dm(5), dm(dz))),
            &Affine::rot_y(deg(pick(k, 50 + i as u32, -12, 12))),
            v3(dm(10), dm(10), dm(10)),
            if i == 2 { DECK } else { DECK_DARK },
            0,
        );
    }
    for i in 0..3 {
        let at = v3(-w / 2 + dm(8), DOCK_TOP + dm(6), int(4) + i * dm(11));
        put(b, f, Mesh::Barrel, at, v3(dm(9), dm(12), dm(9)), STAVE, 0);
        put(
            b,
            f,
            Mesh::Hex,
            at.add(v3(0, dm(3), 0)),
            v3(dm(9), cm(6), dm(9)),
            IRON,
            0,
        );
    }
    // a lantern post at the end
    let post = v3(w / 2 - dm(3), DOCK_TOP, len - dm(4));
    bar(
        b,
        f,
        Mesh::Hex,
        post,
        post.add(v3(0, int(3), 0)),
        dm(2),
        STAVE_DARK,
        0,
    );
    bar(
        b,
        f,
        Mesh::Cube,
        post.add(v3(0, dm(29), 0)),
        post.add(v3(-dm(6), dm(29), 0)),
        cm(8),
        IRON,
        0,
    );
    put(
        b,
        f,
        Mesh::Octa,
        post.add(v3(-dm(6), dm(24), 0)),
        v3(dm(4), dm(6), dm(4)),
        WINDOW,
        ONE * 3 / 4,
    );
    // a ladder down to the water
    for s in [-1, 1] {
        bar(
            b,
            f,
            Mesh::Cube,
            v3(-w / 2 - dm(2), -dm(5), len - int(8) + s * dm(3)),
            v3(-w / 2 - dm(2), DOCK_TOP, len - int(8) + s * dm(3)),
            cm(8),
            STAVE_DARK,
            0,
        );
    }
    // the beach behind and a boathouse
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, -dm(2), -int(6)),
        v3(w * 5, dm(8), int(12)),
        SAND,
        0,
    );
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, -dm(4), -dm(5)),
        v3(w * 5, dm(6), int(2)),
        SAND_WET,
        0,
    );
    let house = v3(-w * 3 / 2, int(2), -int(7));
    put(
        b,
        f,
        Mesh::Bevel,
        house,
        v3(int(6), int(4), int(5)),
        DECK_DARK,
        0,
    );
    put_turned(
        b,
        f,
        Mesh::Wedge,
        house.add(v3(0, dm(28), dm(14))),
        &Affine::IDENTITY,
        v3(dm(64), dm(16), dm(28)),
        PIRATE_WALE,
        0,
    );
    put_turned(
        b,
        f,
        Mesh::Wedge,
        house.add(v3(0, dm(28), -dm(14))),
        &Affine::rot_y(TURN / 2),
        v3(dm(64), dm(16), dm(28)),
        PIRATE_WALE,
        0,
    );
    put(
        b,
        f,
        Mesh::Cube,
        house.add(v3(0, -dm(5), dm(25))),
        v3(dm(12), dm(22), cm(8)),
        PORT,
        0,
    );
    palm(b, f, v3(w * 3, 0, -int(9)), int(9), k);
}

// ---------------------------------------------------------------------------
// The finish island.

/// A palm tree whose foot is at `at`, `h` tall, leaning and curving, a
/// crown of drooping fronds and a few coconuts.
pub fn palm(b: &mut Batch, f: &Affine, at: V3, h: i32, k: u32) {
    let lean = pick(k, 60, 0, TURN);
    let out = facing(lean, 0);
    let mut p = at;
    let n = 5;
    for i in 0..n {
        let t = (i + 1) * ONE / n;
        let next = at
            .add(v3(0, mul(h, t), 0))
            .add(out.scale(mul(mul(h, mul(t, t)), ONE * 22 / 100)));
        bar(
            b,
            f,
            Mesh::Taper,
            p,
            next,
            dm(5) - i * cm(5),
            if i % 2 == 0 {
                PALM_TRUNK
            } else {
                shade(PALM_TRUNK, 85)
            },
            0,
        );
        p = next;
    }
    for i in 0..7 {
        let yaw = i * TURN / 7 + pick(k, 61, 0, 3000);
        let d = facing(yaw, 0);
        let mid = p.add(d.scale(dm(18))).add(v3(0, dm(4), 0));
        let tip = p.add(d.scale(dm(34))).add(v3(0, -dm(10), 0));
        let c = if i % 2 == 0 {
            PALM_LEAF
        } else {
            PALM_LEAF_DARK
        };
        for (a, z) in [(p, mid), (mid, tip)] {
            let g = f.then(&rod(a, z, ONE));
            b.push(
                Mesh::Shard,
                &g.then(&Affine::scale(v3(dm(10), ONE, cm(6)))),
                c,
                0,
            );
        }
    }
    for i in 0..3 {
        let d = facing(i * TURN / 3 + lean, 0);
        put(
            b,
            f,
            Mesh::Ico,
            p.add(d.scale(dm(3))).add(v3(0, -dm(3), 0)),
            v3(dm(3), dm(3), dm(3)),
            BELOW,
            0,
        );
    }
}

/// The finish island: a low mound of white sand `r` across (its crest
/// about a sixth of `r` over the water), wet sand and pale shallows round
/// it, rocks on its shore and two palms behind where the chest goes (the
/// middle, `island_top(r)` high).
pub fn island(b: &mut Batch, f: &Affine, r: i32, k: u32) {
    put(
        b,
        f,
        Mesh::Hex,
        v3(0, cm(4), 0),
        v3(r * 27 / 10, cm(6), r * 27 / 10),
        SEA_LIGHT,
        0,
    );
    put(
        b,
        f,
        Mesh::Hex,
        v3(0, cm(10), 0),
        v3(r * 22 / 10, cm(10), r * 22 / 10),
        SAND_WET,
        0,
    );
    put(
        b,
        f,
        Mesh::Geo,
        v3(0, -r / 10, 0),
        v3(r * 2, r * 5 / 10, r * 2),
        SAND,
        0,
    );
    put(
        b,
        f,
        Mesh::Hex,
        v3(0, island_top(r) - dm(2), 0),
        v3(r * 9 / 10, dm(4), r * 9 / 10),
        SAND,
        0,
    );
    for i in 0..7u32 {
        let yaw = (i as i32) * TURN / 7 + pick(k + i, 70, 0, 4000);
        let at = facing(yaw, 0).scale(r * pick(k + i, 71, 85, 105) / 100);
        let s = r * pick(k + i, 72, 8, 16) / 100;
        put_turned(
            b,
            f,
            Mesh::Rock,
            at.add(v3(0, s / 4, 0)),
            &Affine::rot_y(yaw),
            v3(s, s * 7 / 10, s),
            ROCK,
            0,
        );
    }
    for (i, (x, z)) in [(-45, -55), (40, -60), (-70, 10)].into_iter().enumerate() {
        let at = v3(r * x / 100, island_top(r) * 6 / 10, r * z / 100);
        palm(b, f, at, int(10) + int(i as i32 * 2), k + i as u32 * 7);
    }
    for i in 0..4u32 {
        let yaw = (i as i32) * TURN / 4 + deg(30);
        let at = facing(yaw, 0)
            .scale(r * 6 / 10)
            .add(v3(0, island_top(r) / 2, 0));
        put(
            b,
            f,
            Mesh::Blob,
            at,
            v3(int(3), dm(14), int(3)),
            PALM_LEAF_DARK,
            0,
        );
    }
}

/// The height of the island's flat crest over the water.
pub fn island_top(r: i32) -> i32 {
    r * 15 / 100
}

/// The chest's size: wide, deep and tall (to the body's rim), metres.
pub const CHEST_W: i32 = int(8);
pub const CHEST_D: i32 = int(5);
pub const CHEST_H: i32 = int(4);

/// The finish: an oversized treasure chest (8 m wide) on its foot at the
/// origin, its lid thrown back, overflowing with giant golden bananas
/// (each 2–4 m, jumbled every way, glowing and breathing with `tick` like
/// the Ruins pile), bunches lodged in the heap, more spilled onto the sand
/// round it with doubloons and a few gems.
pub fn treasure_chest(b: &mut Batch, f: &Affine, tick: u32) {
    let (w, d, h) = (CHEST_W, CHEST_D, CHEST_H);
    // the body, its iron bands and gold corners
    put(b, f, Mesh::Bevel, v3(0, h / 2, 0), v3(w, h, d), CHEST, 0);
    for x in [-w * 3 / 10, w * 3 / 10] {
        put(
            b,
            f,
            Mesh::Cube,
            v3(x, h / 2, 0),
            v3(dm(4), h + cm(4), d + cm(8)),
            CHEST_DARK,
            0,
        );
    }
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, dm(3), 0),
        v3(w + cm(8), dm(4), d + cm(8)),
        CHEST_DARK,
        0,
    );
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, h - dm(2), 0),
        v3(w + cm(8), dm(4), d + cm(8)),
        GOLD_DARK,
        0,
    );
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            put(
                b,
                f,
                Mesh::Bevel,
                v3(sx * w / 2, h / 2, sz * d / 2),
                v3(dm(5), h + cm(10), dm(5)),
                GOLD,
                ONE / 8,
            );
        }
    }
    // the lock plate on the front
    put(
        b,
        f,
        Mesh::Bevel,
        v3(0, h - dm(7), d / 2 + cm(4)),
        v3(dm(10), dm(12), cm(10)),
        GOLD,
        ONE / 6,
    );
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, h - dm(8), d / 2 + cm(10)),
        v3(dm(2), dm(4), cm(4)),
        PORT,
        0,
    );
    // the lid, thrown back over the hinge and leaning past upright
    let lid = f
        .then(&Affine::translate(v3(0, h, -d / 2)))
        .then(&Affine::rot_x(-deg(105)));
    put(
        b,
        &lid,
        Mesh::Bevel,
        v3(0, dm(3), d / 2),
        v3(w, dm(6), d),
        CHEST,
        0,
    );
    bar(
        b,
        &lid,
        Mesh::Hex,
        v3(-w / 2, dm(6), d / 2),
        v3(w / 2, dm(6), d / 2),
        d * 9 / 10,
        CHEST,
        0,
    );
    for x in [-w * 3 / 10, w * 3 / 10] {
        bar(
            b,
            &lid,
            Mesh::Hex,
            v3(x - dm(2), dm(6), d / 2),
            v3(x + dm(2), dm(6), d / 2),
            d * 95 / 100,
            CHEST_DARK,
            0,
        );
    }
    // a golden core heaped over the rim so no gap shows
    let p = glyph_pulse(tick, 3);
    let core = mix(shade(BANANA, 82), BANANA_TIP, ONE / 6);
    let cglow = ONE / 4 + mul(p, ONE / 6);
    put(
        b,
        f,
        Mesh::Geo,
        v3(0, h + dm(2), 0),
        v3(w * 95 / 100, int(3), d * 95 / 100),
        core,
        cglow,
    );
    put(
        b,
        f,
        Mesh::Geo,
        v3(-dm(5), h + dm(12), dm(3)),
        v3(w * 6 / 10, int(3), d * 6 / 10),
        core,
        cglow,
    );
    // heaped high: a mound well over the rim (BP2)
    put(
        b,
        f,
        Mesh::Geo,
        v3(dm(3), h + int(2), -dm(2)),
        v3(w * 7 / 10, int(5), d * 7 / 10),
        core,
        cglow,
    );
    put(
        b,
        f,
        Mesh::Geo,
        v3(-dm(4), h + int(4), dm(2)),
        v3(w * 4 / 10, int(4), d * 45 / 100),
        core,
        cglow,
    );
    // the heap: bananas jumbled every way over the core
    for i in 0..96u32 {
        let x = mul(w * 48 / 100, pick(i, 81, -ONE, ONE));
        let z = mul(d * 46 / 100, pick(i, 82, -ONE, ONE));
        let mid =
            ONE - mul(fx::div(x.abs(), w / 2), ONE / 2) - mul(fx::div(z.abs(), d / 2), ONE / 2);
        let y = h + mul(int(5), mul(mid, mid) + mid / 3) + cm(pick(i, 83, -20, 50));
        let at = v3(x, y, z);
        let pk = glyph_pulse(tick, i);
        let c = mix(BANANA, rgb(255, 236, 120), pk);
        let glow = ONE / 3 + mul(pk, ONE / 3);
        if i % 14 == 5 {
            let g = f
                .then(&Affine::translate(at))
                .then(&Affine::rot_y(pick(i, 84, 0, TURN)))
                .then(&Affine::rot_x(deg(pick(i, 85, 15, 55))));
            bunch(b, &g, V3::ZERO, 5, int(pick(i, 86, 2, 3)), c, glow, i);
            continue;
        }
        let len = cm(pick(i, 87, 240, 420));
        let g = f
            .then(&Affine::translate(at))
            .then(&Affine::rot_y(pick(i, 88, 0, TURN)))
            .then(&Affine::rot_x(deg(pick(i, 89, -50, 50))))
            .then(&Affine::rot_z(deg(pick(i, 90, -60, 60))));
        banana(
            b,
            &g,
            v3(0, 0, -mul(len, ONE * 39 / 100)),
            v3(0, 0, ONE),
            len,
            mul(len, ONE * 27 / 100),
            c,
            glow,
        );
    }
    // spilling over the front and sides onto the sand
    for i in 0..30u32 {
        let side = pick(i, 91, 0, 2);
        let (x, z) = match side {
            0 => (cm(pick(i, 92, -350, 350)), d / 2 + cm(pick(i, 93, 40, 260))),
            1 => (w / 2 + cm(pick(i, 93, 40, 220)), cm(pick(i, 92, -200, 200))),
            _ => (
                -w / 2 - cm(pick(i, 93, 40, 220)),
                cm(pick(i, 92, -200, 200)),
            ),
        };
        // the first few hang over the rim, the rest lie on the sand
        let over = i < 12;
        let y = if over { h - dm(2) } else { dm(4) };
        let (x, z) = if !over {
            (x, z)
        } else if side == 0 {
            (x.clamp(-w / 2, w / 2), d / 2 + dm(3))
        } else {
            (x.signum() * (w / 2 + dm(3)), z)
        };
        let len = cm(pick(i, 94, 240, 360));
        let g = f
            .then(&Affine::translate(v3(x, y, z)))
            .then(&Affine::rot_y(pick(i, 95, 0, TURN)))
            .then(&Affine::rot_x(deg(if over {
                pick(i, 96, 50, 80)
            } else {
                pick(i, 96, -10, 10)
            })))
            .then(&Affine::rot_z(deg(pick(i, 97, -40, 40))));
        banana(
            b,
            &g,
            v3(0, 0, -mul(len, ONE * 39 / 100)),
            v3(0, 0, ONE),
            len,
            mul(len, ONE * 27 / 100),
            BANANA,
            ONE / 3,
        );
    }
    // doubloons and gems strewn round
    for i in 0..18u32 {
        let yaw = pick(i, 98, 0, TURN);
        let r = w * 55 / 100 + cm(pick(i, 99, 0, 300));
        let at = facing(yaw, 0).scale(r).add(v3(0, cm(8), 0));
        let tilt = Affine::rot_x(deg(pick(i, 100, -25, 25)))
            .then(&Affine::rot_z(deg(pick(i, 101, -25, 25))));
        if i % 6 == 0 {
            put_turned(
                b,
                f,
                Mesh::Octa,
                at.add(v3(0, dm(2), 0)),
                &tilt,
                v3(dm(4), dm(5), dm(4)),
                if i % 12 == 0 { GEM_RED } else { GEM_GREEN },
                ONE / 3,
            );
        } else {
            put_turned(
                b,
                f,
                Mesh::Cyl,
                at,
                &tilt,
                v3(dm(7), cm(8), dm(7)),
                GOLD,
                ONE / 4,
            );
        }
    }
    // a warm glow round the heap
    b.push_halo(
        Mesh::Sphere,
        &f.then(&Affine::translate(v3(0, h + int(3), 0)))
            .then(&Affine::scale(v3(w * 2, int(14), d * 3))),
        rgb(255, 210, 80),
        ONE / 4 + mul(p, ONE / 8),
    );
}

// ---------------------------------------------------------------------------
// The item vines' doubloons.

/// An item vine's doubloon: a big gold coin (about 1.1 m across, to read
/// at the apes' 3× size) facing the camera (`right` and `up` its screen
/// axes) and turning on its edge with `spin`, a raised rim and a banana
/// struck on each face, a soft gold glow. While the item is taken
/// (`lit` false): tarnished bronze, smaller, no glow.
pub fn doubloon(b: &mut Batch, at: V3, right: V3, up: V3, spin: i32, lit: bool) {
    let face = up.cross(right).norm();
    let (c, rim, glow, size) = if lit {
        (GOLD, rgb(255, 222, 110), ONE / 2, ONE)
    } else {
        (BRONZE, shade(BRONZE, 80), 0, ONE * 3 / 4)
    };
    let sz = |v: i32| mul(v, size);
    // the coin's axis turns about `up`
    let n = face.scale(cos(spin)).add(right.scale(sin(spin))).norm();
    let across = up.cross(n).norm();
    let coin = |dia: i32, thick: i32| Affine {
        c0: across.scale(sz(dia)),
        c1: n.scale(sz(thick)),
        c2: up.scale(sz(dia)),
        t: at,
    };
    b.push(Mesh::Cyl, &coin(cm(110), cm(14)), rim, glow);
    b.push(Mesh::Cyl, &coin(cm(86), cm(20)), c, glow);
    // a banana struck on both faces
    for s in [-1, 1] {
        let off = n.scale(s * sz(cm(11)));
        let p = |x: i32, y: i32| at.add(off).add(across.scale(sz(x))).add(up.scale(sz(y)));
        let pts = [
            p(-cm(26), cm(10)),
            p(-cm(8), -cm(14)),
            p(cm(16), -cm(12)),
            p(cm(28), cm(12)),
        ];
        for i in 0..3 {
            b.push(
                Mesh::Hex,
                &rod(pts[i], pts[i + 1], sz(cm(12))),
                shade(c, 78),
                glow / 2,
            );
        }
    }
    if lit {
        b.push_halo(
            Mesh::Sphere,
            &crate::render::placed(at, 0, v3(cm(200), cm(200), cm(200))),
            rgb(255, 196, 60),
            ONE * 2 / 5,
        );
    }
}

// ---------------------------------------------------------------------------
// The deck fights' crew.

/// Whose crew.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crew {
    /// striped or red shirts, bandanas or a black tricorn, cutlasses
    Pirate,
    /// blue coats, white breeches, black bicornes, muskets
    Navy,
}

/// What a crew monkey is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pose {
    /// swinging a cutlass (a pirate) or thrusting a bayonet (navy)
    Duel,
    /// both arms up, hopping
    Cheer,
    /// a musket or pistol levelled forward
    Aim,
    /// leaning back on a rope
    Haul,
    /// knocked flat on its back
    Down,
}

/// A low-detail monkey sailor, about 1.7 m tall at scale 1 (short legs,
/// a big head, long arms), its feet at
/// the origin facing +z, posed with `tick` (`k` puts it out of step with
/// its neighbours): ten shapes, about 150 triangles, so dozens can fight
/// on the decks.
pub fn deckhand(b: &mut Batch, f: &Affine, crew: Crew, pose: Pose, tick: u32, k: u32) {
    let phase = pick(k, 5, 0, TURN);
    let beat = wave(tick, 1, phase);
    let g = match pose {
        Pose::Down => f
            .then(&Affine::translate(v3(0, dm(2), 0)))
            .then(&Affine::rot_x(-deg(82))),
        Pose::Cheer => f.then(&Affine::translate(v3(0, mul(beat.abs(), dm(3)), 0))),
        Pose::Haul => f.then(&Affine::rot_x(-deg(18))),
        _ => *f,
    };
    let (shirt, legs) = match crew {
        Crew::Pirate => (
            if k.is_multiple_of(2) {
                SHIRT_RED
            } else {
                SHIRT_WHITE
            },
            TROUSERS,
        ),
        Crew::Navy => (COAT, BREECHES),
    };
    // legs: a lunge for a duel, apart otherwise
    let stride = match pose {
        Pose::Duel => dm(3),
        Pose::Aim => dm(2),
        _ => dm(1),
    };
    for s in [-1, 1] {
        let hip = v3(s * dm(1) + s * cm(5), dm(5), 0);
        let foot = v3(s * dm(2), 0, s * stride);
        bar(b, &g, Mesh::Cube, hip, foot, dm(2), legs, 0);
    }
    // body and head
    put(
        b,
        &g,
        Mesh::Ico,
        v3(0, dm(8), 0),
        v3(dm(6) + cm(5), dm(7), dm(5)),
        shirt,
        0,
    );
    put(
        b,
        &g,
        Mesh::Ico,
        v3(0, dm(13), 0),
        v3(dm(5) + cm(5), dm(5), dm(5)),
        FUR,
        0,
    );
    put(
        b,
        &g,
        Mesh::Cube,
        v3(0, dm(12) + cm(5), dm(2) + cm(5)),
        v3(dm(3) + cm(5), dm(2) + cm(5), cm(10)),
        FACE,
        0,
    );
    // hats
    match crew {
        Crew::Pirate if k.is_multiple_of(3) => {
            put(
                b,
                &g,
                Mesh::Pyramid,
                v3(0, dm(16) + cm(5), 0),
                v3(dm(7), dm(2), dm(6)),
                HAT,
                0,
            );
        }
        Crew::Pirate => {
            put(
                b,
                &g,
                Mesh::Cube,
                v3(0, dm(15) + cm(5), -cm(2)),
                v3(dm(5) + cm(5), cm(14), dm(5) + cm(5)),
                if k.is_multiple_of(2) {
                    FLAG_BLACK
                } else {
                    SHIRT_RED
                },
                0,
            );
        }
        Crew::Navy => {
            put(
                b,
                &g,
                Mesh::Pyramid,
                v3(0, dm(16) + cm(5), 0),
                v3(dm(9), dm(3), dm(3) + cm(5)),
                HAT,
                0,
            );
        }
    }
    // arms and what they hold
    let shoulder = |s: i32| v3(s * dm(3) + s * cm(5), dm(10), 0);
    let (left, right, tool): (V3, V3, Option<(V3, V3, Rgb)>) = match pose {
        Pose::Duel => {
            let a = mul(beat, deg(70));
            let hand = v3(dm(4), dm(11) + mul(sin(a), dm(4)), mul(cos(a), dm(5)));
            let tip = hand.add(v3(
                0,
                mul(sin(a + deg(40)), dm(8)),
                mul(cos(a + deg(40)), dm(8)),
            ));
            let c = if crew == Crew::Pirate { STEEL } else { MUSKET };
            (v3(-dm(5), dm(5), -dm(2)), hand, Some((hand, tip, c)))
        }
        Pose::Cheer => {
            let up = dm(17) + mul(beat, dm(1));
            let sword = v3(dm(4), up, 0);
            (
                v3(-dm(4), up, 0),
                sword,
                Some((sword, sword.add(v3(dm(1), dm(7), 0)), STEEL)),
            )
        }
        Pose::Aim => {
            let hand = v3(dm(1), dm(11), dm(5));
            let gun = if crew == Crew::Navy { dm(14) } else { dm(5) };
            (
                v3(-dm(1), dm(11), dm(4)),
                hand,
                Some((
                    hand.add(v3(0, 0, -dm(4))),
                    hand.add(v3(0, cm(4), gun)),
                    MUSKET,
                )),
            )
        }
        Pose::Haul => {
            let pull = mul(beat, dm(1));
            let l = v3(-dm(1), dm(9), dm(5) + pull);
            let r = v3(dm(1), dm(9), dm(6) + pull);
            (
                l,
                r,
                Some((
                    r.add(v3(0, -dm(2), -dm(4))),
                    r.add(v3(0, dm(12), dm(14))),
                    rgb(196, 168, 112),
                )),
            )
        }
        Pose::Down => (v3(-dm(7), dm(10), dm(1)), v3(dm(7), dm(10), dm(1)), None),
    };
    bar(b, &g, Mesh::Cube, shoulder(-1), left, dm(1) + cm(5), FUR, 0);
    bar(b, &g, Mesh::Cube, shoulder(1), right, dm(1) + cm(5), FUR, 0);
    if let Some((a, z, c)) = tool {
        bar(b, &g, Mesh::Cube, a, z, cm(7), c, 0);
    }
}

// ---------------------------------------------------------------------------
// Cannon fire (scenery).

/// A cannonball, 30 cm across, at `at` in frame `f`.
pub fn cannonball(b: &mut Batch, f: &Affine, at: V3) {
    put(b, f, Mesh::Ico, at, v3(dm(3), dm(3), dm(3)), IRON, 0);
}

/// A muzzle's smoke at `age` (0 the shot, `ONE` gone), from a muzzle at
/// the origin firing along +z: an orange flash at first, then a cloud of
/// puffs billowing out and rising, growing and thinning away.
pub fn smoke_puff(b: &mut Batch, f: &Affine, age: i32, k: u32) {
    let age = age.clamp(0, ONE);
    if age >= ONE {
        return;
    }
    if age < ONE / 8 {
        let s = dm(8) + mul(age, int(10));
        put(
            b,
            f,
            Mesh::Octa,
            v3(0, 0, s / 2),
            v3(s, s, s * 3 / 2),
            FLASH,
            ONE,
        );
    }
    let fade = if age > ONE * 3 / 10 {
        ONE - fx::div(age - ONE * 3 / 10, ONE * 7 / 10)
    } else {
        ONE
    };
    // BP4: soft white puffs that billow out, rise and shrink away, each
    // with a pale haze round it (solid and grey they read as rocks)
    for i in 0..6u32 {
        let reach = mul(fx::sqrt(age), int(pick(k + i, 110, 3, 7)));
        let side = cm(pick(k + i, 111, -150, 150));
        let rise = mul(age, int(pick(k + i, 112, 3, 6)));
        let grow = dm(10) + mul(fx::sqrt(age), int(3));
        let s = mul(grow, fade);
        if s <= 0 {
            continue;
        }
        let c = if age < ONE / 6 {
            mix(FLASH, SMOKE, age * 6)
        } else {
            SMOKE
        };
        let at = v3(side, rise, reach);
        put(
            b,
            f,
            Mesh::Blob,
            at,
            v3(s * 7 / 10, s * 6 / 10, s * 7 / 10),
            c,
            ONE / 2,
        );
        b.push_halo(
            Mesh::Sphere,
            &f.then(&Affine::translate(at)).then(&Affine::scale(v3(
                grow * 3 / 2,
                grow * 5 / 4,
                grow * 3 / 2,
            ))),
            SMOKE,
            mul(ONE / 3, fade),
        );
    }
}

/// A splash where a ball hits the water (its foot at the origin) at `age`:
/// a short burst (BR2), not a standing column: a fan of spray thrown up
/// and out that falls back and breaks apart, a puff of mist round it that
/// fades, and a few broken flecks of foam spreading on the water.
pub fn splash(b: &mut Batch, f: &Affine, age: i32, k: u32) {
    let age = age.clamp(0, ONE);
    if age >= ONE {
        return;
    }
    // the spray: drops flung up and out, falling back by the end (each
    // one up `h` and down again), smaller as they go
    let fall = ONE - age;
    for i in 0..7u32 {
        let yaw = (i as i32) * TURN / 7 + pick(k + i, 120, -3000, 3000);
        let speed = int(pick(k + i, 122, 2, 5));
        let out = facing(yaw, 0).scale(dm(3) + mul(fx::sqrt(age), speed));
        let h = int(pick(k + i, 121, 3, 7));
        let y = mul(h, 4 * mul(age, fall));
        let s = mul(dm(pick(k + i, 123, 4, 8)), fall);
        if s <= cm(2) {
            continue;
        }
        put_turned(
            b,
            f,
            Mesh::Shard,
            out.add(v3(0, y, 0)),
            &Affine::rot_y(yaw).then(&Affine::rot_x(deg(20) + mul(age, deg(60)))),
            v3(s * 2 / 3, s * 2, s / 3),
            FOAM,
            ONE / 3,
        );
    }
    // a narrow spout early on only, breaking up as it rises
    if age < ONE * 2 / 5 {
        let u = fx::div(age, ONE * 2 / 5);
        let tall = mul(sin(mul(u, TURN / 2)), int(3));
        let w = mul(dm(9), ONE - u / 2);
        if tall > cm(20) {
            put(
                b,
                f,
                Mesh::Shard,
                v3(0, tall / 2, 0),
                v3(w, tall, w / 2),
                FOAM,
                ONE / 3,
            );
        }
    }
    // mist: a soft haze that swells and fades
    let mist = dm(20) + mul(fx::sqrt(age), int(5));
    b.push_halo(
        Mesh::Sphere,
        &f.then(&Affine::translate(v3(0, mist / 3, 0)))
            .then(&Affine::scale(v3(mist, mist * 2 / 3, mist))),
        FOAM,
        mul(ONE / 3, fall),
    );
    // the foam left on the water: broken flecks spreading and fading
    let r = dm(10) + mul(fx::sqrt(age), int(3));
    foam_ring(b, f, r, r, mul(ONE * 3 / 5, fall), k, 5);
}

/// A ball striking a hull at the origin (the hull's face toward -z) at
/// `age`: splinters thrown out and a dark puff.
pub fn hull_hit(b: &mut Batch, f: &Affine, age: i32, k: u32) {
    let age = age.clamp(0, ONE);
    if age >= ONE {
        return;
    }
    for i in 0..6u32 {
        let yaw = deg(pick(k + i, 130, -60, 60)) + TURN / 2;
        let d = facing(yaw, deg(pick(k + i, 131, -10, 50)));
        let at = d
            .scale(mul(age, int(5)))
            .add(v3(0, -mul(mul(age, age), int(4)), 0));
        put_turned(
            b,
            f,
            Mesh::Shard,
            at,
            &Affine::rot_z(mul(age, TURN) + (i as i32) * 9000),
            v3(dm(2), dm(6), cm(6)),
            DECK_DARK,
            0,
        );
    }
    // a pale puff rising and shrinking away in a haze (BP4)
    let at = v3(0, mul(age, int(3)), -mul(age, int(2)));
    let grow = dm(8) + mul(fx::sqrt(age), int(2));
    let s = mul(grow, ONE - age);
    put(
        b,
        f,
        Mesh::Blob,
        at,
        v3(s * 7 / 10, s * 6 / 10, s * 7 / 10),
        shade(SMOKE, 88),
        ONE / 3,
    );
    b.push_halo(
        Mesh::Sphere,
        &f.then(&Affine::translate(at)).then(&Affine::scale(v3(
            grow * 3 / 2,
            grow * 5 / 4,
            grow * 3 / 2,
        ))),
        SMOKE,
        mul(ONE / 3, ONE - age),
    );
}

// ---------------------------------------------------------------------------
// The model gallery (`?gallery=<key>`, `app/gallery.rs`).

/// The keys `app/gallery.rs` shows from here.
pub const GALLERY: [&str; 18] = [
    "galleon",
    "galleonnavy",
    "frigate",
    "frigatenavy",
    "brig",
    "brignavy",
    "sloop",
    "sloopnavy",
    "sails",
    "barrels",
    "dockstart",
    "island",
    "chest",
    "doubloon",
    "crew",
    "cannonfire",
    "flags",
    "bayall",
];

// ---------------------------------------------------------------------------
// The background battle (BQ2): low-detail ships, fire and smoke for the
// fight outside the course's barrels, seen from 60 m and further.

/// Ship `s` in low detail (in its own frame, as `ship`): a hull block with
/// its wale, a bow and a stern castle, the deck, each mast as one taper,
/// each yard as one bar with its sail as `Sail` says (one or two shapes),
/// the gaff sail and a flag at the tallest masthead. About 30 shapes
/// against `ship`'s 300 or more; still, so it goes in the chunked
/// scenery. `k` varies the patches.
pub fn ship_far(b: &mut Batch, f: &Affine, s: &Ship, k: u32) {
    let p = paint(s.side);
    let (len, beam, deck) = (s.length, s.beam, s.deck);
    let body = len * 74 / 100;
    let aft = -len / 2 + body / 2;
    put(
        b,
        f,
        Mesh::Bevel,
        v3(0, deck / 2 - dm(10), aft + len / 20),
        v3(beam, deck + dm(20), body),
        p.hull,
        0,
    );
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, deck * 55 / 100, aft + len / 20),
        v3(beam + cm(12), dm(6), body + cm(10)),
        p.wale,
        0,
    );
    // the bow: a box turned corner-on and drawn out, so the hull comes
    // to a point at the stem with its rail level (a ramp read as a raft's
    // at 60 m, BQ7.3)
    let stem = len * 21 / 100;
    b.push(
        Mesh::Bevel,
        &f.then(&Affine::translate(v3(
            0,
            deck / 2 - dm(10),
            aft + len / 20 + body / 2,
        )))
        .then(&Affine::scale(v3(
            beam * 707 / 1000,
            deck + dm(20),
            stem * 1414 / 1000,
        )))
        .then(&Affine::rot_y(deg(45))),
        p.hull,
        0,
    );
    // a forestay from the foremast's head to the bow
    let bow_tip = v3(0, deck + dm(10), aft + len / 20 + body / 2 + stem);
    if let Some(fore) = s.masts.iter().max_by_key(|m| m.along) {
        bar(
            b,
            f,
            Mesh::Cyl,
            v3(0, deck + fore.height * 9 / 10, fore.along),
            bow_tip,
            cm(9),
            RIGGING,
            0,
        );
    }
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, deck + dm(1), aft + len / 20),
        v3(beam * 85 / 100, dm(2), body * 96 / 100),
        DECK,
        0,
    );
    // the raised decks as blocks
    for &(from, to, h) in s.raised.iter().take(2) {
        let (lo, hi) = (from.min(to), from.max(to));
        put(
            b,
            f,
            Mesh::Cube,
            v3(0, (deck + h) / 2, (lo + hi) / 2),
            v3(beam * 9 / 10, h - deck + dm(4), hi - lo),
            p.hull,
            0,
        );
    }
    let tallest = s.masts.iter().map(|m| m.height).max().unwrap_or(0);
    for (mi, mast) in s.masts.iter().enumerate() {
        let top = s.deck + mast.height;
        bar(
            b,
            f,
            Mesh::Taper,
            v3(0, deck, mast.along),
            v3(0, top, mast.along),
            beam * 7 / 100,
            MAST,
            0,
        );
        // a shroud each side, from the rail to near the masthead
        for side in [-1, 1] {
            bar(
                b,
                f,
                Mesh::Cyl,
                v3(side * beam / 2, deck + dm(8), mast.along - dm(15)),
                v3(0, deck + mast.height * 8 / 10, mast.along),
                cm(9),
                RIGGING,
                0,
            );
        }
        let ys = spars(s, mi);
        for (i, y) in ys.iter().enumerate() {
            put(
                b,
                f,
                Mesh::Cube,
                y.at,
                v3(y.half * 2, dm(3), dm(3)),
                MAST,
                0,
            );
            let span = y.half * 2 * 92 / 100;
            let top = y.at.y - dm(2);
            let z = y.at.z + dm(3);
            match mast.yards[i].sail {
                Sail::Set => {
                    put(
                        b,
                        f,
                        Mesh::Cube,
                        v3(0, top - y.drop / 2, z + dm(2)),
                        v3(span, y.drop, dm(4)),
                        p.sail,
                        CANVAS_GLOW,
                    );
                    if s.side == Side::Pirate && (k + i as u32).is_multiple_of(2) {
                        put(
                            b,
                            f,
                            Mesh::Cube,
                            v3(span / 5, top - y.drop / 2, z + dm(5)),
                            v3(span / 5, y.drop / 4, cm(4)),
                            PIRATE_PATCH,
                            ONE / 4,
                        );
                    }
                }
                Sail::Furled => put(
                    b,
                    f,
                    Mesh::Hex,
                    v3(0, top - dm(3), z),
                    v3(span, dm(7), dm(7)),
                    shade(p.sail, 90),
                    ROLL_GLOW,
                ),
                Sail::Torn => {
                    put(
                        b,
                        f,
                        Mesh::Cube,
                        v3(-span / 4, top - y.drop / 4, z + dm(2)),
                        v3(span / 2, y.drop / 2, dm(3)),
                        p.sail,
                        CANVAS_GLOW,
                    );
                    put(
                        b,
                        f,
                        Mesh::Shard,
                        v3(span / 3, top - y.drop / 3, z + dm(2)),
                        v3(span / 4, y.drop * 2 / 3, dm(3)),
                        shade(p.sail, 94),
                        CANVAS_GLOW,
                    );
                }
                Sail::Missing => {}
            }
        }
        if let Some(Sail::Set) = mast.gaff {
            let reach = mast.height * 4 / 10;
            put(
                b,
                f,
                Mesh::Cube,
                v3(0, deck + mast.height * 35 / 100, mast.along - reach / 2),
                v3(dm(3), mast.height * 4 / 10, reach),
                p.sail,
                CANVAS_GLOW,
            );
        }
        if mast.height == tallest {
            // the colours, hung still at the masthead
            let c = match s.side {
                Side::Pirate => FLAG_BLACK,
                Side::Navy => ENSIGN,
            };
            let size = beam * 45 / 100;
            put(
                b,
                f,
                Mesh::Cube,
                v3(0, top - size / 2, mast.along - size * 3 / 4),
                v3(cm(8), size, size * 3 / 2),
                c,
                ONE / 6,
            );
        }
    }
}

/// A far ship's fire at `tick` (its frame's origin on the burning deck):
/// flickering flames and a dark smoke column leaning downwind (+z), puffs
/// rising and spreading as they go. `k` sets the phase.
pub fn fire_far(b: &mut Batch, f: &Affine, tick: u32, k: u32) {
    for i in 0..3u32 {
        let flick = wave(tick, 5 + i as i32, pick(k + i, 60, 0, TURN));
        let h = dm(30) + mul(flick, dm(10));
        let x = cm(pick(k + i, 61, -250, 250));
        put(
            b,
            f,
            Mesh::Spike,
            v3(x, h / 2, cm(pick(k + i, 62, -300, 300))),
            v3(dm(18) + mul(flick, dm(4)), h, dm(18)),
            if i == 1 { BANANA } else { FLASH },
            ONE,
        );
    }
    // the column (BR2): soft puffs that rise 30 m over 8 s, drifting off
    // with the wind and swaying, a small dark core inside a wide haze
    // that thins and pales as it climbs, then begin again
    let life = 960u32;
    for i in 0..6u32 {
        let age = ((tick + k * 97 + i * life / 6) % life) as i32 * ONE / life as i32;
        let rise = mul(age, int(30)) + dm(20);
        let lean = mul(mul(age, age), int(16));
        let sway = mul(sin(mul(age, TURN) + pick(k + i, 65, 0, TURN)), int(2));
        let at = v3(cm(pick(k + i, 63, -100, 100)) + sway, rise, lean);
        let grow = dm(22) + mul(age, int(8));
        // the core shrinks away as the haze takes over
        let core = mul(grow, (ONE - age) * 7 / 10);
        let fade = if age < ONE / 8 { age * 8 } else { ONE - age };
        if core > cm(20) {
            put(
                b,
                f,
                Mesh::Blob,
                at,
                v3(core, core * 3 / 4, core),
                mix(SMOKE_DARK, SMOKE, ONE / 4 + age * 3 / 4),
                ONE / 2,
            );
        }
        b.push_halo(
            Mesh::Sphere,
            &f.then(&Affine::translate(at)).then(&Affine::scale(v3(
                grow * 2,
                grow * 3 / 2,
                grow * 2,
            ))),
            mix(SMOKE_DARK, SMOKE, ONE / 2 + age / 2),
            mul(ONE / 2, fade),
        );
    }
}

/// A far broadside's muzzle smoke (its frame's +z pointing out of the
/// port) at `age` (0 fired, ONE gone): a flash, then two pale puffs
/// drifting out and up. Three shapes at most, against `smoke_puff`'s
/// dozen, for guns 60 m and more away.
pub fn smoke_far(b: &mut Batch, f: &Affine, age: i32, k: u32) {
    let age = age.clamp(0, ONE);
    if age >= ONE {
        return;
    }
    if age < ONE / 10 {
        put(
            b,
            f,
            Mesh::Octa,
            v3(0, 0, dm(10)),
            v3(dm(12), dm(12), dm(20)),
            FLASH,
            ONE,
        );
    }
    let fade = ONE - mul(age, age);
    for i in 0..2u32 {
        let grow = dm(14) + mul(fx::sqrt(age), int(4));
        let s = mul(grow, fade);
        put(
            b,
            f,
            Mesh::Blob,
            v3(
                cm(pick(k + i, 64, -120, 120)),
                mul(age, int(3 + i as i32)),
                mul(fx::sqrt(age), int(4 + i as i32 * 2)),
            ),
            v3(s, s * 7 / 10, s),
            SMOKE,
            ONE / 2,
        );
    }
}

/// A light foam collar where something meets the sea (its frame's origin
/// at the waterline), `w` across and `l` along: broken pale flecks round
/// it, low and part sea-coloured so they read as foam, not a plate (BR2).
pub fn foam(b: &mut Batch, f: &Affine, w: i32, l: i32, k: u32) {
    foam_ring(b, f, w / 2, l / 2, ONE * 3 / 4, k, 6);
}

/// `n` flecks of foam round an ellipse `rx` by `rz` at the origin, `pale`
/// (Q16) of the way from the sea's colour to white foam, each a flat
/// irregular blob turned its own way, with gaps between.
fn foam_ring(b: &mut Batch, f: &Affine, rx: i32, rz: i32, pale: i32, k: u32, n: u32) {
    let c = mix(SEA_LIGHT, WAKE, pale);
    for i in 0..n {
        let a = (i as i32) * TURN / n as i32 + pick(k + i, 130, -2500, 2500);
        let (sa, ca) = (sin(a), cos(a));
        let at = v3(mul(rx, sa), cm(2), mul(rz, ca));
        // along the ring about two thirds of the gap to the next, across
        // a third of that; a hair thick
        let span = rx.max(rz) / 1000 * 6283 / n as i32 * 2 / 3;
        let len = span / 100 * pick(k + i, 131, 70, 110);
        put_turned(
            b,
            f,
            Mesh::Octa,
            at,
            &Affine::rot_y(a + TURN / 4 + pick(k + i, 132, -2000, 2000)),
            v3(len.max(cm(30)), cm(4), (len / 3).max(cm(15))),
            c,
            ONE / 5,
        );
    }
}

/// Ship `s` grown by `f` (Q16) about its waterline middle: every length,
/// height and offset along it, its place and heading kept.
pub fn scaled(mut s: Ship, f: i32) -> Ship {
    let g = |v: i32| mul(v, f);
    s.length = g(s.length);
    s.beam = g(s.beam);
    s.deck = g(s.deck);
    s.bowsprit = g(s.bowsprit);
    for r in &mut s.raised {
        *r = (g(r.0), g(r.1), g(r.2));
    }
    for m in &mut s.masts {
        m.along = g(m.along);
        m.height = g(m.height);
        for y in &mut m.yards {
            y.height = g(y.height);
            y.half_span = g(y.half_span);
        }
    }
    for p in &mut s.ports {
        p.along = g(p.along);
        p.up = g(p.up);
    }
    s
}

/// Sets the sails of `s` as listed, mast by mast (fore first), each
/// mast's yards lowest first, then its gaff; entries past the list stay.
pub fn with_sails(mut s: Ship, sails: &[Sail]) -> Ship {
    let mut it = sails.iter().copied();
    for m in &mut s.masts {
        for y in &mut m.yards {
            if let Some(v) = it.next() {
                y.sail = v;
            }
        }
        if let Some(g) = &mut m.gaff {
            if let Some(v) = it.next() {
                *g = v;
            }
        }
    }
    s
}

/// The gallery's ship of `kind` for `side`, with a gap or two shot
/// through its sails the way the course will want them.
fn gallery_ship(kind: ShipKind, side: Side) -> Ship {
    use Sail::*;
    let s = typical(kind, side, V3::ZERO, 0);
    let sails: &[Sail] = match kind {
        // fore: set, set, furled; main: missing, torn, set; mizzen: set,
        // set, spanker set
        ShipKind::Galleon => &[Set, Set, Furled, Missing, Torn, Set, Set, Set, Set],
        ShipKind::Frigate => &[Set, Furled, Set, Torn, Missing, Set, Set, Set, Torn],
        ShipKind::Brig => &[Missing, Set, Torn, Set, Set],
        ShipKind::Sloop => &[Set],
    };
    with_sails(s, sails)
}

/// The middle and rough radius of a gallery model, metres.
pub fn framing(key: &str) -> (V3, i32) {
    match key {
        "galleon" | "galleonnavy" => (v3(0, int(17), int(4)), int(44)),
        "frigate" | "frigatenavy" => (v3(0, int(15), int(3)), int(37)),
        "brig" | "brignavy" => (v3(0, int(11), int(2)), int(27)),
        "sloop" | "sloopnavy" => (v3(0, int(9), int(2)), int(21)),
        "sails" => (v3(0, int(5), 0), int(16)),
        "barrels" => (v3(0, dm(15), 0), int(5)),
        "dockstart" => (v3(0, int(2), int(10)), int(22)),
        "island" => (v3(0, int(5), 0), int(26)),
        "chest" => (v3(0, int(4), 0), int(9)),
        "doubloon" => (V3::ZERO, int(2)),
        "crew" => (v3(0, dm(9), 0), int(5)),
        "cannonfire" => (v3(0, int(3), 0), int(12)),
        "flags" => (v3(0, int(4), 0), int(9)),
        _ => (v3(0, int(14), 0), int(80)),
    }
}

/// Draws gallery model `key` in frame `f` (turning) at `tick`.
pub fn gallery(b: &mut Batch, key: &str, f: &Affine, tick: u32) {
    // the sea under everything
    let (_, r) = framing(key);
    put(
        b,
        &Affine::IDENTITY,
        Mesh::Hex,
        v3(0, -cm(5), 0),
        v3(r * 8, cm(10), r * 8),
        SEA,
        0,
    );
    let kinds = [
        ("galleon", ShipKind::Galleon),
        ("frigate", ShipKind::Frigate),
        ("brig", ShipKind::Brig),
        ("sloop", ShipKind::Sloop),
    ];
    for (i, (name, kind)) in kinds.into_iter().enumerate() {
        if key == name {
            return ship(b, f, &gallery_ship(kind, Side::Pirate), tick, i as u32);
        }
        if key.strip_suffix("navy") == Some(name) {
            return ship(b, f, &gallery_ship(kind, Side::Navy), tick, i as u32 + 10);
        }
    }
    match key {
        "sails" => {
            // every state on a yard, pirate above, navy below
            let states = [Sail::Set, Sail::Furled, Sail::Torn, Sail::Missing];
            for (i, st) in states.into_iter().enumerate() {
                for (j, side) in [Side::Pirate, Side::Navy].into_iter().enumerate() {
                    let g = f.then(&Affine::translate(v3(
                        int(i as i32 * 13 - 20),
                        -int(j as i32 * 8),
                        0,
                    )));
                    let y = Spar {
                        at: v3(0, int(9), 0),
                        half: int(6),
                        drop: int(6),
                    };
                    yard_and_sail(b, &g, &y, st, &paint(side), side, i as u32);
                }
            }
        }
        "barrels" => {
            boundary_barrel(b, &f.then(&Affine::translate(v3(-int(2), 0, 0))), tick, 1);
            bounce_barrel(b, &f.then(&Affine::translate(v3(int(2), 0, 0))), tick, 2, 0);
        }
        "dockstart" => dock(b, f, int(28), int(5), 1),
        "island" => {
            island(b, f, int(22), 1);
            treasure_chest(
                b,
                &f.then(&Affine::translate(v3(0, island_top(int(22)), 0))),
                tick,
            );
        }
        "chest" => treasure_chest(b, f, tick),
        "doubloon" => {
            let right = v3(ONE, 0, 0);
            doubloon(b, v3(-cm(80), 0, 0), right, V3::UP, 0, true);
            doubloon(b, v3(cm(80), 0, 0), right, V3::UP, deg(35), false);
        }
        "crew" => {
            let poses = [Pose::Duel, Pose::Cheer, Pose::Aim, Pose::Haul, Pose::Down];
            for (i, p) in poses.into_iter().enumerate() {
                for (j, c) in [Crew::Pirate, Crew::Navy].into_iter().enumerate() {
                    let at = v3(int(i as i32 * 2 - 4), 0, int(j as i32 * 2 - 1));
                    let g = f
                        .then(&Affine::translate(at))
                        .then(&Affine::rot_y(if j == 0 { 0 } else { TURN / 2 }));
                    deckhand(b, &g, c, p, tick, (i * 2 + j) as u32);
                }
            }
        }
        "cannonfire" => {
            let ages = [ONE / 20, ONE / 4, ONE / 2, ONE * 8 / 10];
            for (i, a) in ages.into_iter().enumerate() {
                let x = int(i as i32 * 6 - 9);
                smoke_puff(
                    b,
                    &f.then(&Affine::translate(v3(x, int(4), -int(6)))),
                    a,
                    i as u32,
                );
                splash(
                    b,
                    &f.then(&Affine::translate(v3(x, 0, int(4)))),
                    a,
                    i as u32,
                );
                hull_hit(
                    b,
                    &f.then(&Affine::translate(v3(x, int(8), int(2)))),
                    a,
                    i as u32,
                );
            }
            cannonball(b, f, v3(0, int(6), 0));
            cannonball(b, f, v3(int(2), int(7), 0));
        }
        "flags" => {
            let h = int(3);
            jolly_roger(b, f, v3(0, h, -int(8)), int(3), tick, 0);
            ensign(b, f, v3(0, h + int(2), -int(1)), int(3), tick);
            pennant(b, f, v3(0, h + int(3), int(5)), int(5), tick);
        }
        _ => {
            // a little battle: a pirate galleon and a navy frigate side by
            // side, a sloop and a brig beyond, barrels, smoke and a splash
            let ships = [
                (ShipKind::Galleon, Side::Pirate, v3(-int(22), 0, 0), 0),
                (ShipKind::Frigate, Side::Navy, v3(int(20), 0, int(8)), 0),
                (
                    ShipKind::Sloop,
                    Side::Pirate,
                    v3(int(4), 0, int(60)),
                    deg(20),
                ),
                (
                    ShipKind::Brig,
                    Side::Navy,
                    v3(-int(40), 0, int(60)),
                    -deg(15),
                ),
            ];
            for (i, (kind, side, at, yaw)) in ships.into_iter().enumerate() {
                let g = f.then(&Affine::translate(at)).then(&Affine::rot_y(yaw));
                ship(b, &g, &gallery_ship(kind, side), tick, i as u32);
            }
            for i in 0..4 {
                smoke_puff(
                    b,
                    &f.then(&Affine::translate(v3(-int(16), int(3), int(i * 7 - 10))))
                        .then(&Affine::rot_y(TURN / 4)),
                    ONE / 5 + i * ONE / 6,
                    i as u32,
                );
            }
            splash(b, &f.then(&Affine::translate(v3(0, 0, int(4)))), ONE / 3, 1);
            for i in 0..6 {
                boundary_barrel(
                    b,
                    &f.then(&Affine::translate(v3(int(i * 12 - 30), 0, -int(26)))),
                    tick,
                    i as u32,
                );
            }
            bounce_barrel(
                b,
                &f.then(&Affine::translate(v3(0, 0, -int(14)))),
                tick,
                9,
                0,
            );
            let deck = typical(ShipKind::Galleon, Side::Pirate, V3::ZERO, 0).deck;
            for i in 0..6u32 {
                let at = v3(
                    -int(22) + int(i as i32 % 3 - 1) * 2,
                    deck,
                    int(i as i32 * 2 - 5),
                );
                let g = f
                    .then(&Affine::translate(at))
                    .then(&Affine::rot_y(if i % 2 == 0 {
                        TURN / 4
                    } else {
                        -TURN / 4
                    }));
                let crew = if i % 2 == 0 { Crew::Pirate } else { Crew::Navy };
                deckhand(b, &g, crew, Pose::Duel, tick, i);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KINDS: [ShipKind; 4] = [
        ShipKind::Galleon,
        ShipKind::Frigate,
        ShipKind::Brig,
        ShipKind::Sloop,
    ];

    /// Triangles in a batch (each instance's mesh's).
    fn triangles(b: &Batch) -> usize {
        Mesh::ALL
            .iter()
            .map(|&m| {
                b.count_of(m) * crate::mesh::vertices(m).len() / crate::mesh::VERTEX_VALUES / 3
            })
            .sum()
    }

    #[test]
    fn every_bay_model_draws_something() {
        for key in GALLERY {
            let mut b = Batch::new();
            gallery(&mut b, key, &Affine::IDENTITY, 240);
            assert!(b.count() > 10, "{key}");
        }
    }

    fn all(kind: ShipKind, side: Side, sail: Sail) -> Ship {
        let s = typical(kind, side, V3::ZERO, 0);
        let n = s
            .masts
            .iter()
            .map(|m| m.yards.len() + m.gaff.is_some() as usize)
            .sum::<usize>();
        let mut s = with_sails(s, &vec![sail; n]);
        s.jib = Some(sail);
        s
    }

    /// The per-model costs (SECOND-ORDER BM2): kept under budgets so 20
    /// ships and dozens of crew stay cheap.
    #[test]
    fn the_models_stay_cheap() {
        let mut rows: Vec<(String, usize, usize)> = Vec::new();
        let mut cost = |name: &str, draw: &dyn Fn(&mut Batch)| {
            let mut b = Batch::new();
            draw(&mut b);
            let row = (name.to_string(), b.count(), triangles(&b));
            rows.push(row.clone());
            row
        };
        let id = Affine::IDENTITY;
        for kind in KINDS {
            for side in [Side::Pirate, Side::Navy] {
                let set = all(kind, side, Sail::Set);
                let torn = all(kind, side, Sail::Torn);
                let (_, shapes, tris) = cost(&format!("{kind:?} {side:?} all set"), &|b| {
                    ship(b, &id, &set, 0, 1)
                });
                assert!(shapes < 420 && tris < 14_000, "{kind:?} {shapes} {tris}");
                let (_, shapes, _) = cost(&format!("{kind:?} {side:?} all torn"), &|b| {
                    ship(b, &id, &torn, 0, 1)
                });
                assert!(shapes < 520, "{kind:?} {shapes}");
            }
        }
        // 13: staves and hoops so it reads as a barrel, not a stone (BP4)
        let (_, s, t) = cost("boundary barrel", &|b| boundary_barrel(b, &id, 0, 1));
        assert!(s <= 13 && t < 400, "{s} {t}");
        let (_, s, t) = cost("bounce barrel", &|b| bounce_barrel(b, &id, 0, 1, 0));
        assert!(s <= 12 && t < 500, "{s} {t}");
        for p in [Pose::Duel, Pose::Cheer, Pose::Aim, Pose::Haul, Pose::Down] {
            for c in [Crew::Pirate, Crew::Navy] {
                let (_, s, t) = cost(&format!("deckhand {c:?} {p:?}"), &|b| {
                    deckhand(b, &id, c, p, 30, 1)
                });
                assert!(s <= 10 && t <= 200, "{p:?} {s} {t}");
            }
        }
        cost("dock 28 m", &|b| dock(b, &id, int(28), int(5), 1));
        cost("island r 22 m", &|b| island(b, &id, int(22), 1));
        cost("treasure chest", &|b| treasure_chest(b, &id, 0));
        let (_, s, _) = cost("doubloon", &|b| {
            doubloon(b, V3::ZERO, v3(ONE, 0, 0), V3::UP, 0, true)
        });
        assert!(s <= 12);
        cost("cannonball", &|b| cannonball(b, &id, V3::ZERO));
        cost("smoke puff", &|b| smoke_puff(b, &id, ONE / 3, 1));
        cost("splash", &|b| splash(b, &id, ONE / 3, 1));
        cost("hull hit", &|b| hull_hit(b, &id, ONE / 3, 1));
        // the core may not print (tests/boundary.rs): set `SHOW` to see
        // the table (name, shapes, triangles) in this assertion's message
        const SHOW: bool = false;
        assert!(!SHOW && rows.len() > 30, "{rows:#?}");
    }

    #[test]
    fn a_set_sail_fills_its_gap_and_the_others_open_it() {
        let s = typical(ShipKind::Frigate, Side::Navy, V3::ZERO, 0);
        // the main course (mast 1, the lowest yard) hangs to the rail
        let y = spars(&s, 1)[0];
        assert!(y.drop > int(6), "{}", y.drop);
        let canvas = |state: Sail| {
            let mut b = Batch::new();
            yard_and_sail(
                &mut b,
                &Affine::IDENTITY,
                &y,
                state,
                &paint(Side::Navy),
                Side::Navy,
                0,
            );
            b.count_of(Mesh::Cube) + b.count_of(Mesh::Shard)
        };
        assert!(canvas(Sail::Set) >= 2);
        assert_eq!(canvas(Sail::Furled), 0);
        assert!(canvas(Sail::Missing) <= 1);
        // a whole ship draws more canvas with every sail set than with
        // every sail gone
        let mut set = Batch::new();
        let mut gone = Batch::new();
        ship(
            &mut set,
            &Affine::IDENTITY,
            &all(ShipKind::Galleon, Side::Pirate, Sail::Set),
            0,
            1,
        );
        ship(
            &mut gone,
            &Affine::IDENTITY,
            &all(ShipKind::Galleon, Side::Pirate, Sail::Missing),
            0,
            1,
        );
        assert!(set.count_of(Mesh::Cube) > gone.count_of(Mesh::Cube) + 15);
    }

    #[test]
    fn typical_ships_fit_the_contract() {
        for kind in KINDS {
            let s = typical(kind, Side::Pirate, v3(int(10), 0, int(20)), deg(30));
            // the brief's ~10 s end to end for the big ones; the smaller
            // fill the gaps
            let want = match kind {
                ShipKind::Galleon => (int(40), int(50)),
                ShipKind::Frigate => (int(32), int(40)),
                ShipKind::Brig => (int(22), int(30)),
                ShipKind::Sloop => (int(16), int(24)),
            };
            assert!(s.length >= want.0 && s.length <= want.1, "{kind:?}");
            // masts fore first, every yard above the deck and inside the
            // hull's length, the sloop's mast gaff-rigged
            for w in s.masts.windows(2) {
                assert!(w[0].along > w[1].along, "{kind:?}");
            }
            for (m, mast) in s.masts.iter().enumerate() {
                assert!(mast.along.abs() < s.length / 2, "{kind:?}");
                for (i, y) in mast.yards.iter().enumerate() {
                    assert!(y.height > int(5) && y.height < mast.height, "{kind:?}");
                    let (l, r) = s.yard_ends(m, i);
                    assert_eq!(l.dist(r) / ONE, y.half_span * 2 / ONE, "{kind:?}");
                }
            }
            assert_eq!(
                kind == ShipKind::Sloop,
                s.masts.iter().all(|m| m.yards.is_empty())
            );
            // ports on both sides, under the main deck
            assert!(s.ports.iter().any(|p| p.side > 0) && s.ports.iter().any(|p| p.side < 0));
            assert!(s.ports.iter().all(|p| p.up < s.deck), "{kind:?}");
            // the raised decks stand over the main deck inside the hull
            for r in &s.raised {
                assert!(r.0 < r.1 && r.2 > s.deck, "{kind:?}");
                assert!(r.0 >= -s.length / 2 && r.1 <= s.length / 2, "{kind:?}");
            }
        }
    }

    #[test]
    fn the_fire_and_the_barrels_move_with_the_tick() {
        let mut one = Batch::new();
        let mut two = Batch::new();
        bounce_barrel(&mut one, &Affine::IDENTITY, 0, 1, 0);
        bounce_barrel(&mut two, &Affine::IDENTITY, 50, 1, 0);
        assert_ne!(one.finish(1).0, two.finish(1).0);
        // smoke and splashes are gone at the end of their age
        let mut b = Batch::new();
        smoke_puff(&mut b, &Affine::IDENTITY, ONE, 0);
        splash(&mut b, &Affine::IDENTITY, ONE, 0);
        hull_hit(&mut b, &Affine::IDENTITY, ONE, 0);
        assert_eq!(b.count(), 0);
    }
}
