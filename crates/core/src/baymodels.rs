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
const PIRATE_SAIL: Rgb = rgb(198, 184, 152);
const PIRATE_PATCH: Rgb = rgb(150, 132, 104);
const NAVY_HULL: Rgb = rgb(30, 50, 98);
const NAVY_STRIPE: Rgb = rgb(226, 178, 60);
const NAVY_SAIL: Rgb = rgb(246, 244, 232);

const DECK: Rgb = rgb(184, 142, 92);
const DECK_DARK: Rgb = rgb(140, 104, 66);
const BELOW: Rgb = rgb(70, 50, 36);
const MAST: Rgb = rgb(122, 86, 52);
const RIGGING: Rgb = rgb(62, 50, 38);
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

/// The three kinds of ship, by size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShipKind {
    /// about 50 m from bowsprit to stern, three masts, a high stern castle
    /// and two gun decks
    Galleon,
    /// about 40 m, three masts, one gun deck, a low quarterdeck
    Frigate,
    /// a brig-rigged sloop of about 25 m: two masts, a few small guns
    Sloop,
}

/// Whose ship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// tarred black hull, oxblood wale, patched canvas, the black flag
    Pirate,
    /// deep blue hull, ochre gun stripes, white sails, a made-up ensign
    Navy,
}

/// The state of the sail on one yard. Only `Set` fills the space under
/// its yard; the course swings through the others.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sail {
    /// full canvas from the yard down to the next yard (or the deck's
    /// rail): a wall
    Set,
    /// rolled up on its yard: the space under it is open
    Furled,
    /// shot through: ragged strips at each end and along the yard, the
    /// middle half of the span open from just under the yard down
    Torn,
    /// shot away: a bare yard, a scrap at one end
    Gone,
}

/// A ship's hull, life size, in its own frame (+z the bow, the waterline
/// at y 0, the keel's middle at x 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hull {
    /// stem to sternpost along the waterline (the bow's beak and the
    /// bowsprit reach further)
    pub length: i32,
    /// widest, at the middle
    pub beam: i32,
    /// the main deck's (the waist's) height over the water
    pub deck: i32,
    /// how far the keel goes under the water
    pub keel: i32,
    /// the forecastle: from `z0` to `z1`, its deck at height `y`
    pub fore: Raised,
    /// the quarterdeck
    pub quarter: Raised,
    /// the poop deck over the quarterdeck's after part (a galleon's high
    /// stern castle)
    pub poop: Option<Raised>,
    /// rows of gun ports per side
    pub gun_rows: i32,
    /// how far the bowsprit reaches past the stem
    pub bowsprit: i32,
}

/// A raised deck from `z0` (aft) to `z1` (forward), its top at `y`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Raised {
    pub z0: i32,
    pub z1: i32,
    pub y: i32,
}

/// One mast: where it stands along the keel, how tall it is over the
/// water, and its yards from the lowest up (height over the water, half
/// span).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mast {
    pub z: i32,
    pub top: i32,
    pub yards: &'static [(i32, i32)],
}

/// One yard, in the ship's frame: the yard's middle (`at`, just forward
/// of its mast), its half span along x, and how far its sail hangs when
/// set (down to the next yard, or to the rail for a course).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Yard {
    pub mast: usize,
    pub at: V3,
    pub half: i32,
    pub drop: i32,
}

/// A deck you can land and run on, in the ship's frame: an axis-aligned
/// rectangle from `z0` to `z1`, `half` either side of x 0, its top at `y`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeckTop {
    pub z0: i32,
    pub z1: i32,
    pub half: i32,
    pub y: i32,
}

const GALLEON_MASTS: [Mast; 3] = [
    Mast {
        z: dm(-130),
        top: dm(280),
        yards: &[(dm(165), dm(65)), (dm(230), dm(48))],
    },
    Mast {
        z: dm(0),
        top: dm(380),
        yards: &[(dm(145), dm(105)), (dm(245), dm(82)), (dm(325), dm(55))],
    },
    Mast {
        z: dm(125),
        top: dm(335),
        yards: &[(dm(140), dm(90)), (dm(225), dm(70)), (dm(295), dm(48))],
    },
];

const FRIGATE_MASTS: [Mast; 3] = [
    Mast {
        z: dm(-110),
        top: dm(245),
        yards: &[(dm(135), dm(60)), (dm(200), dm(45))],
    },
    Mast {
        z: dm(-5),
        top: dm(320),
        yards: &[(dm(125), dm(90)), (dm(210), dm(70)), (dm(280), dm(48))],
    },
    Mast {
        z: dm(100),
        top: dm(295),
        yards: &[(dm(120), dm(80)), (dm(195), dm(62)), (dm(260), dm(44))],
    },
];

const SLOOP_MASTS: [Mast; 2] = [
    Mast {
        z: dm(-30),
        top: dm(215),
        yards: &[(dm(90), dm(65)), (dm(155), dm(50))],
    },
    Mast {
        z: dm(45),
        top: dm(195),
        yards: &[(dm(85), dm(58)), (dm(145), dm(44))],
    },
];

/// The hull of `kind`.
pub fn hull(kind: ShipKind) -> Hull {
    match kind {
        ShipKind::Galleon => Hull {
            length: int(44),
            beam: int(12),
            deck: int(5),
            keel: int(3),
            fore: Raised {
                z0: dm(140),
                z1: dm(215),
                y: dm(75),
            },
            quarter: Raised {
                z0: dm(-220),
                z1: dm(-60),
                y: int(8),
            },
            poop: Some(Raised {
                z0: dm(-220),
                z1: dm(-150),
                y: dm(105),
            }),
            gun_rows: 2,
            bowsprit: int(11),
        },
        ShipKind::Frigate => Hull {
            length: int(36),
            beam: int(10),
            deck: int(4),
            keel: dm(25),
            fore: Raised {
                z0: dm(115),
                z1: dm(175),
                y: dm(58),
            },
            quarter: Raised {
                z0: dm(-180),
                z1: dm(-70),
                y: dm(62),
            },
            poop: None,
            gun_rows: 1,
            bowsprit: int(9),
        },
        ShipKind::Sloop => Hull {
            length: int(22),
            beam: int(7),
            deck: dm(26),
            keel: dm(18),
            fore: Raised {
                z0: dm(55),
                z1: dm(90),
                y: dm(36),
            },
            quarter: Raised {
                z0: dm(-110),
                z1: dm(-55),
                y: dm(38),
            },
            poop: None,
            gun_rows: 1,
            bowsprit: int(6),
        },
    }
}

/// The masts of `kind`, aft to forward.
pub fn masts(kind: ShipKind) -> &'static [Mast] {
    match kind {
        ShipKind::Galleon => &GALLEON_MASTS,
        ShipKind::Frigate => &FRIGATE_MASTS,
        ShipKind::Sloop => &SLOOP_MASTS,
    }
}

/// The hull's half width at `z` (the beam narrowing to the bow and less
/// to the stern).
pub fn half_beam(h: &Hull, z: i32) -> i32 {
    let t = fx::div(z, h.length / 2).clamp(-ONE, ONE);
    let narrow = if t > 0 {
        mul(mul(t, t), ONE * 55 / 100)
    } else {
        mul(mul(t, t), ONE * 22 / 100)
    };
    mul(h.beam / 2, ONE - narrow)
}

/// The deck's height at `z`: the forecastle's, the quarterdeck's (or the
/// poop's) where they stand, else the waist's.
pub fn deck_at(h: &Hull, z: i32) -> i32 {
    if let Some(p) = h.poop {
        if z >= p.z0 && z <= p.z1 {
            return p.y;
        }
    }
    for r in [h.fore, h.quarter] {
        if z >= r.z0 && z <= r.z1 {
            return r.y;
        }
    }
    h.deck
}

/// Every yard of `kind` in sail order (mast by mast aft to forward, each
/// from the lowest up): the index a sail state list (`ship`'s `sails`)
/// goes by.
pub fn yards(kind: ShipKind) -> Vec<Yard> {
    let h = hull(kind);
    let mut out = Vec::new();
    for (m, mast) in masts(kind).iter().enumerate() {
        let rail = deck_at(&h, mast.z) + dm(15);
        for (i, &(y, half)) in mast.yards.iter().enumerate() {
            let below = if i == 0 { rail } else { mast.yards[i - 1].0 };
            out.push(Yard {
                mast: m,
                at: v3(0, y, mast.z + dm(6)),
                half,
                drop: y - below - dm(6),
            });
        }
    }
    out
}

/// Where vines can hang from on a ship of `kind`, in its frame: both tips
/// of every yard (a little in from the end), then each mast's top under
/// its cap. The course picks; the scene draws the vine down from it.
pub fn rope_points(kind: ShipKind) -> Vec<V3> {
    let mut out = Vec::new();
    for y in yards(kind) {
        let x = y.half * 92 / 100;
        out.push(y.at.add(v3(-x, -dm(2), 0)));
        out.push(y.at.add(v3(x, -dm(2), 0)));
    }
    for m in masts(kind) {
        out.push(v3(0, m.top - dm(10), m.z + dm(6)));
    }
    out
}

/// The decks of `kind` you can stand on: the waist, then the forecastle,
/// the quarterdeck and the poop, each a little inside the bulwarks.
pub fn decks(kind: ShipKind) -> Vec<DeckTop> {
    let h = hull(kind);
    let inside = |z0: i32, z1: i32| half_beam(&h, z0).min(half_beam(&h, z1)) - dm(5);
    let mut out = vec![DeckTop {
        z0: h.quarter.z1,
        z1: h.fore.z0,
        half: inside(h.quarter.z1, h.fore.z0),
        y: h.deck,
    }];
    out.push(DeckTop {
        z0: h.fore.z0,
        z1: h.fore.z1,
        half: inside(h.fore.z0, h.fore.z1),
        y: h.fore.y,
    });
    let q0 = h.poop.map(|p| p.z1).unwrap_or(h.quarter.z0);
    out.push(DeckTop {
        z0: q0,
        z1: h.quarter.z1,
        half: inside(q0, h.quarter.z1),
        y: h.quarter.y,
    });
    if let Some(p) = h.poop {
        out.push(DeckTop {
            z0: p.z0,
            z1: p.z1,
            half: inside(p.z0, p.z1),
            y: p.y,
        });
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

/// A still ship of `kind` for `side` in frame `f` (its waterline at y 0,
/// bow +z), each yard's sail as `sails` says (in `yards` order; missing
/// entries are `Set`), its flags streaming with `tick`. `k` varies the
/// patches and wear.
pub fn ship(
    b: &mut Batch,
    f: &Affine,
    kind: ShipKind,
    side: Side,
    sails: &[Sail],
    tick: u32,
    k: u32,
) {
    let h = hull(kind);
    let p = paint(side);
    hull_body(b, f, &h, &p);
    castles(b, f, &h, &p, kind);
    guns(b, f, &h, &p);
    stern(b, f, &h, &p, side, tick);
    bow(b, f, &h, &p, side);
    // masts, tops, yards and sails
    let ys = yards(kind);
    let ms = masts(kind);
    for mast in ms.iter() {
        let foot = deck_at(&h, mast.z) - dm(5);
        let w = h.beam * 8 / 100;
        let mut from = v3(0, foot, mast.z);
        for (i, &(y, _)) in mast.yards.iter().enumerate() {
            let to = v3(0, y + dm(20), mast.z);
            let wi = w * (10 - i as i32 * 3) / 10;
            bar(b, f, Mesh::Taper, from, to, wi, MAST, 0);
            // the top (a platform) under each upper yard
            put(
                b,
                f,
                Mesh::Hex,
                v3(0, y + dm(16), mast.z),
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
            v3(0, mast.top, mast.z),
            w * 3 / 10,
            MAST,
            0,
        );
        put(
            b,
            f,
            Mesh::Ico,
            v3(0, mast.top, mast.z),
            v3(w / 2, w / 2, w / 2),
            MAST,
            0,
        );
        shrouds(b, f, &h, mast);
        // the flag at the tallest mast's head
        let tallest = ms.iter().map(|m| m.top).max().unwrap_or(0);
        if mast.top == tallest {
            let head = v3(0, mast.top, mast.z);
            match side {
                Side::Pirate => jolly_roger(b, f, head, h.beam * 45 / 100, tick, k),
                Side::Navy => pennant(b, f, head, h.beam * 80 / 100, tick),
            }
        }
    }
    for (i, y) in ys.iter().enumerate() {
        let state = sails.get(i).copied().unwrap_or(Sail::Set);
        yard_and_sail(b, f, y, state, &p, side, k.wrapping_add(i as u32));
    }
    stays(b, f, &h, ms);
}

/// The hull: stations along the keel, each a bevelled block for the body
/// and one for the topsides, a narrow one for the keel, a deck plank on
/// top and bulwarks along each side.
fn hull_body(b: &mut Batch, f: &Affine, h: &Hull, p: &Paint) {
    // a station about every 4 m (a sloop 6, a galleon 11)
    let n = (h.length / int(4)).clamp(6, 11);
    let step = h.length / n;
    for i in 0..n {
        let z = -h.length / 2 + step / 2 + step * i;
        let hw = half_beam(h, z);
        let len = step * 115 / 100;
        let top = deck_at(h, z);
        // the body, up to the wale
        let wale = h.deck * 55 / 100;
        put(
            b,
            f,
            Mesh::Bevel,
            v3(0, (wale - h.keel * 6 / 10) / 2, z),
            v3(hw * 2, wale + h.keel * 6 / 10, len),
            p.hull,
            0,
        );
        // the keel
        put(
            b,
            f,
            Mesh::Bevel,
            v3(0, -h.keel * 7 / 10, z),
            v3(hw, h.keel * 6 / 10, len),
            p.hull,
            0,
        );
        // the wale: a band of colour round the hull
        put(
            b,
            f,
            Mesh::Cube,
            v3(0, wale - dm(2), z),
            v3(hw * 2 + dm(2), dm(5), len),
            p.wale,
            0,
        );
        // the topsides, tumbling home a little, up to this station's deck
        put(
            b,
            f,
            Mesh::Cube,
            v3(0, (wale + top) / 2, z),
            v3(hw * 2 * 95 / 100, top - wale, len),
            p.hull,
            0,
        );
        // the deck planks, in two tones
        put(
            b,
            f,
            Mesh::Cube,
            v3(0, top + cm(2), z),
            v3(hw * 2 * 88 / 100, cm(6), len),
            if i % 2 == 0 { DECK } else { DECK_DARK },
            0,
        );
        // bulwarks with a trim cap
        for s in [-1, 1] {
            let x = s * (hw * 95 / 100 - dm(1));
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

/// The forecastle, quarterdeck and poop: their fronts with doors and
/// windows, ladders up from the waist, the wheel and a hatch.
fn castles(b: &mut Batch, f: &Affine, h: &Hull, p: &Paint, kind: ShipKind) {
    let fronts = [(h.fore.z0, h.fore.y, 1), (h.quarter.z1, h.quarter.y, -1)];
    for (z, y, dir) in fronts {
        let hw = half_beam(h, z) * 90 / 100;
        // the bulkhead facing the waist
        put(
            b,
            f,
            Mesh::Cube,
            v3(0, (h.deck + y) / 2, z),
            v3(hw * 2, y - h.deck, dm(2)),
            shade(p.hull, 130),
            0,
        );
        // a door and two windows
        let face = z - dir * dm(2);
        put(
            b,
            f,
            Mesh::Cube,
            v3(0, h.deck + dm(10), face),
            v3(dm(10), dm(20), cm(8)),
            PORT,
            0,
        );
        for s in [-1, 1] {
            put(
                b,
                f,
                Mesh::Cube,
                v3(s * hw / 2, h.deck + dm(14), face),
                v3(dm(8), dm(6), cm(8)),
                WINDOW,
                ONE / 3,
            );
        }
        // ladders up either side
        for s in [-1, 1] {
            let x = s * hw * 70 / 100;
            put_turned(
                b,
                f,
                Mesh::Wedge,
                v3(x, (h.deck + y) / 2, z - dir * (y - h.deck) * 6 / 10),
                &Affine::rot_y(if dir > 0 { 0 } else { TURN / 2 }),
                v3(dm(12), y - h.deck, (y - h.deck) * 12 / 10),
                DECK_DARK,
                0,
            );
        }
        // a rail along the raised deck's edge
        put(
            b,
            f,
            Mesh::Cube,
            v3(0, y + dm(8), z - dir * dm(1)),
            v3(hw * 2, cm(8), cm(8)),
            p.trim,
            0,
        );
    }
    if let Some(poop) = h.poop {
        let hw = half_beam(h, poop.z1) * 88 / 100;
        put(
            b,
            f,
            Mesh::Cube,
            v3(0, (h.quarter.y + poop.y) / 2, poop.z1),
            v3(hw * 2, poop.y - h.quarter.y, dm(2)),
            shade(p.hull, 130),
            0,
        );
        for s in [-2, 0, 2] {
            put(
                b,
                f,
                Mesh::Cube,
                v3(s * hw / 3, h.quarter.y + dm(14), poop.z1 + dm(1)),
                v3(dm(8), dm(6), cm(8)),
                WINDOW,
                ONE / 3,
            );
        }
        put(
            b,
            f,
            Mesh::Cube,
            v3(0, poop.y + dm(8), poop.z1 + dm(1)),
            v3(hw * 2, cm(8), cm(8)),
            p.trim,
            0,
        );
    }
    // the wheel on the quarterdeck
    let wz = h.quarter.z1 - dm(25);
    put_turned(
        b,
        f,
        Mesh::Hex,
        v3(0, h.quarter.y + dm(10), wz),
        &Affine::rot_x(TURN / 4),
        v3(dm(13), dm(1), dm(13)),
        MAST,
        0,
    );
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, h.quarter.y + dm(5), wz),
        v3(dm(3), dm(10), dm(4)),
        BELOW,
        0,
    );
    // a hatch grating and the capstan in the waist
    let mid = (h.quarter.z1 + h.fore.z0) / 2;
    put(
        b,
        f,
        Mesh::Cube,
        v3(0, h.deck + cm(8), mid - dm(30)),
        v3(h.beam * 3 / 10, cm(16), dm(30)),
        BELOW,
        0,
    );
    put(
        b,
        f,
        Mesh::Hex,
        v3(0, h.deck + dm(5), mid + dm(35)),
        v3(dm(10), dm(10), dm(10)),
        MAST,
        0,
    );
    if kind == ShipKind::Galleon {
        // a ship's boat lashed upside down in the waist
        put(
            b,
            f,
            Mesh::Barrel,
            v3(0, h.deck + dm(7), mid + dm(5)),
            v3(dm(18), dm(8), dm(50)),
            shade(DECK, 80),
            0,
        );
    }
}

/// Gun ports down both sides with the cannon's muzzles run out, and a
/// painted strip along each row.
fn guns(b: &mut Batch, f: &Affine, h: &Hull, p: &Paint) {
    let span = h.length * 64 / 100;
    let pitch = if h.length > int(30) { dm(30) } else { dm(28) };
    let n = span / pitch;
    for row in 0..h.gun_rows {
        let y = h.deck * 55 / 100 + dm(8) + row * dm(20);
        let y = y.min(h.deck - dm(4));
        for s in [-1, 1] {
            put(
                b,
                f,
                Mesh::Cube,
                v3(s * (half_beam(h, 0) * 95 / 100), y, -dm(10)),
                v3(cm(10), dm(9), span + pitch),
                if row % 2 == 0 { p.wale } else { p.hull },
                0,
            );
            for i in 0..=n {
                let z = -span / 2 - dm(10) + i * pitch;
                let hw = half_beam(h, z) * 95 / 100;
                let x = s * hw;
                put(
                    b,
                    f,
                    Mesh::Cube,
                    v3(x, y, z),
                    v3(cm(12), dm(6), dm(7)),
                    PORT,
                    0,
                );
                bar(
                    b,
                    f,
                    Mesh::Hex,
                    v3(x, y, z),
                    v3(x + s * dm(9), y + cm(4), z),
                    dm(3),
                    IRON,
                    0,
                );
            }
        }
    }
}

/// The stern: a transom with lit windows in rows, a gallery rail and
/// lanterns; the ensign (navy) or a red flag (pirate) on a staff.
fn stern(b: &mut Batch, f: &Affine, h: &Hull, p: &Paint, side: Side, tick: u32) {
    let z = -h.length / 2 - dm(3);
    let top = h.poop.map(|p| p.y).unwrap_or(h.quarter.y);
    let hw = half_beam(h, -h.length / 2) * 90 / 100;
    put(
        b,
        f,
        Mesh::Bevel,
        v3(0, (top + dm(10) + h.deck * 3 / 10) / 2, z),
        v3(hw * 2, top + dm(10) - h.deck * 3 / 10, dm(8)),
        shade(p.hull, 120),
        0,
    );
    // rows of stern windows, one per raised deck
    let rows = if h.poop.is_some() { 3 } else { 2 };
    for r in 0..rows {
        let y = h.deck - dm(10) + r * (top - h.deck) / rows.max(1);
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
        // a gilded band under each row
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
    // lanterns on the taffrail
    for s in [-1, 0, 1] {
        let at = v3(s * hw * 8 / 10, top + dm(18), z);
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
    // the staff and its flag
    let foot = v3(0, top, z + dm(3));
    let head = foot.add(v3(0, dm(55), -dm(20)));
    bar(b, f, Mesh::Hex, foot, head, cm(12), MAST, 0);
    let size = h.beam * 35 / 100;
    match side {
        Side::Navy => ensign(b, f, head, size, tick),
        Side::Pirate => plain_flag(b, f, head, size * 8 / 10, RED_FLAG, tick),
    }
}

/// The bow: a pointed beak, the bowsprit, a figurehead (gilded on the
/// navy's ships, a bone ape skull on the pirates') and a jib stay.
fn bow(b: &mut Batch, f: &Affine, h: &Hull, p: &Paint, side: Side) {
    let z = h.length / 2;
    let hw = half_beam(h, z);
    let beak = h.beam * 45 / 100;
    put_turned(
        b,
        f,
        Mesh::Spike,
        v3(
            0,
            (h.deck * 6 / 10 - h.keel / 2) / 2 + dm(4),
            z + beak / 2 - dm(5),
        ),
        &Affine::rot_x(TURN / 4),
        v3(hw * 2, beak, h.deck * 6 / 10 + h.keel / 2),
        p.hull,
        0,
    );
    let root = v3(0, h.fore.y - dm(5), z - dm(10));
    let tip = v3(0, h.fore.y + h.bowsprit * 4 / 10, z + h.bowsprit);
    bar(b, f, Mesh::Taper, root, tip, h.beam * 5 / 100, MAST, 0);
    // the figurehead under the bowsprit
    let fig = v3(0, h.deck * 6 / 10, z + beak * 45 / 100);
    let (c, glow) = match side {
        Side::Navy => (GOLD, ONE / 6),
        Side::Pirate => (BONE, 0),
    };
    put(b, f, Mesh::Ico, fig, v3(dm(12), dm(13), dm(12)), c, glow);
    for s in [-1, 1] {
        put(
            b,
            f,
            Mesh::Ico,
            fig.add(v3(s * dm(7), dm(3), -dm(2))),
            v3(dm(5), dm(5), dm(3)),
            c,
            glow,
        );
    }
    if side == Side::Pirate {
        for s in [-1, 1] {
            put(
                b,
                f,
                Mesh::Cube,
                fig.add(v3(s * dm(3), dm(2), dm(5))),
                v3(dm(3), dm(3), cm(6)),
                PORT,
                0,
            );
        }
    }
}

/// The shrouds: two lines a side from the rail up to each mast's lowest
/// top, thin and dark so the vines stand out (MAPS art lessons).
fn shrouds(b: &mut Batch, f: &Affine, h: &Hull, mast: &Mast) {
    let deck = deck_at(h, mast.z);
    let top = mast.yards[0].0 + dm(16);
    for s in [-1, 1] {
        for dz in [-dm(12), dm(6)] {
            let z = mast.z + dz - dm(6);
            let foot = v3(s * half_beam(h, z) * 95 / 100, deck + dm(10), z);
            bar(
                b,
                f,
                Mesh::Cube,
                foot,
                v3(s * dm(5), top, mast.z),
                cm(7),
                RIGGING,
                0,
            );
        }
    }
}

/// The stays: from each mast's head forward and down to the next mast,
/// the foremost to the bowsprit's tip.
fn stays(b: &mut Batch, f: &Affine, h: &Hull, ms: &[Mast]) {
    for w in ms.windows(2) {
        let (a, c) = (w[0], w[1]);
        let low = c.yards[0].0 + dm(16);
        bar(
            b,
            f,
            Mesh::Cube,
            v3(0, a.top - dm(5), a.z),
            v3(0, low, c.z),
            cm(7),
            RIGGING,
            0,
        );
    }
    if let Some(fore) = ms.last() {
        let tip = v3(0, h.fore.y + h.bowsprit * 4 / 10, h.length / 2 + h.bowsprit);
        bar(
            b,
            f,
            Mesh::Cube,
            v3(0, fore.top - dm(5), fore.z),
            tip,
            cm(7),
            RIGGING,
            0,
        );
    }
}

/// A yard across its mast and, under it, its sail as `state` says.
fn yard_and_sail(b: &mut Batch, f: &Affine, y: &Yard, state: Sail, p: &Paint, side: Side, k: u32) {
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
                0,
            );
            put(
                b,
                f,
                Mesh::Cube,
                v3(0, top - y.drop * 45 / 100, z + dm(5)),
                v3(span * 80 / 100, y.drop * 70 / 100, dm(3)),
                shade(sail, 96),
                0,
            );
            if side == Side::Pirate && k % 3 != 0 {
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
                    0,
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
                    0,
                );
            }
        }
        Sail::Furled => {
            bar(
                b,
                f,
                Mesh::Hex,
                v3(-y.half * 9 / 10, top - dm(2), z),
                v3(y.half * 9 / 10, top - dm(2), z),
                dm(6).max(w * 2),
                shade(sail, 92),
                0,
            );
        }
        Sail::Torn => {
            // a strip along the yard
            put(
                b,
                f,
                Mesh::Cube,
                v3(0, top - y.drop / 16, z + dm(1)),
                v3(span, y.drop / 8, dm(2)),
                sail,
                0,
            );
            // ragged strips hanging at each end, of uneven length
            for s in [-1, 1] {
                for j in 0..2 {
                    let wide = span / 9;
                    let x = s * (span / 2 - wide / 2 - j * wide);
                    let len = y.drop * pick(k, 20 + j as u32 + (s + 1) as u32 * 5, 45, 95) / 100;
                    put(
                        b,
                        f,
                        Mesh::Cube,
                        v3(x, top - len / 2, z + dm(1)),
                        v3(wide, len, dm(2)),
                        shade(sail, 100 - j * 6),
                        0,
                    );
                    // a tattered end
                    put_turned(
                        b,
                        f,
                        Mesh::Shard,
                        v3(x, top - len - dm(3), z + dm(1)),
                        &Affine::rot_z(deg(pick(k, 30 + j as u32, -20, 20))),
                        v3(wide * 8 / 10, dm(8), dm(2)),
                        shade(sail, 90),
                        0,
                    );
                }
            }
            // a few short tatters hanging from the middle strip
            for j in 0..3 {
                let x = (j - 1) * span / 6 + pick(k, 40 + j as u32, -6, 6) * span / 100;
                put_turned(
                    b,
                    f,
                    Mesh::Shard,
                    v3(x, top - y.drop / 8 - dm(5), z + dm(1)),
                    &Affine::rot_z(deg(pick(k, 43 + j as u32, -15, 15))),
                    v3(span / 14, dm(12), dm(2)),
                    shade(sail, 92),
                    0,
                );
            }
        }
        Sail::Gone => {
            // a scrap left at one end
            let s = if k.is_multiple_of(2) { 1 } else { -1 };
            put_turned(
                b,
                f,
                Mesh::Shard,
                v3(s * span * 4 / 10, top - dm(8), z),
                &Affine::rot_z(deg(10 * s)),
                v3(span / 10, dm(16), dm(2)),
                shade(sail, 85),
                0,
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
fn bob(tick: u32, k: u32) -> (i32, i32) {
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
    put(
        b,
        &g,
        Mesh::Barrel,
        v3(0, dm(2), 0),
        v3(dm(15), dm(20), dm(15)),
        STAVE,
        0,
    );
    for y in [-dm(3), dm(7)] {
        put(
            b,
            &g,
            Mesh::Hex,
            v3(0, y, 0),
            v3(dm(15), cm(10), dm(15)),
            IRON,
            0,
        );
    }
    put(
        b,
        &g,
        Mesh::Hex,
        v3(0, dm(12), 0),
        v3(dm(11), cm(4), dm(11)),
        STAVE_DARK,
        0,
    );
    let top = v3(0, dm(45), 0);
    bar(b, &g, Mesh::Hex, v3(0, dm(11), 0), top, cm(8), MAST, 0);
    // the flag: a swallow-tailed pennant, orange with a white bar
    let a = mul(wave(tick, 1, pick(k, 2, 0, TURN)), deg(25));
    let fl = g
        .then(&Affine::translate(top.add(v3(0, -dm(3), 0))))
        .then(&Affine::rot_y(a));
    put(
        b,
        &fl,
        Mesh::Cube,
        v3(0, 0, dm(4)),
        v3(cm(4), dm(5), dm(8)),
        MARKER,
        ONE / 5,
    );
    put(
        b,
        &fl,
        Mesh::Cube,
        v3(0, 0, dm(4)),
        v3(cm(5), cm(12), dm(8)),
        WHITE,
        0,
    );
    for t in [-1, 1] {
        put_turned(
            b,
            &fl,
            Mesh::Wedge,
            v3(0, t * dm(1), dm(9)),
            &Affine::rot_z(TURN / 4),
            v3(dm(2), cm(4), dm(3)),
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
    // the heap: bananas jumbled every way over the core
    let mut k = 0u32;
    for i in 0..56u32 {
        let x = mul(w * 46 / 100, pick(i, 81, -ONE, ONE));
        let z = mul(d * 44 / 100, pick(i, 82, -ONE, ONE));
        let mid =
            ONE - mul(fx::div(x.abs(), w / 2), ONE / 2) - mul(fx::div(z.abs(), d / 2), ONE / 2);
        let y = h + mul(int(3), mid) + cm(pick(i, 83, -20, 40));
        let at = v3(x, y, z);
        let pk = glyph_pulse(tick, k);
        k += 1;
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
        let len = cm(pick(i, 87, 220, 380));
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
    for i in 0..14u32 {
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
        let y = if i < 5 { h - dm(2) } else { dm(4) };
        let (x, z) = if i < 5 {
            (x.clamp(-w / 2, w / 2), d / 2 + dm(3))
        } else {
            (x, z)
        };
        let len = cm(pick(i, 94, 240, 360));
        let g = f
            .then(&Affine::translate(v3(x, y, z)))
            .then(&Affine::rot_y(pick(i, 95, 0, TURN)))
            .then(&Affine::rot_x(deg(if i < 5 {
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
        &f.then(&Affine::translate(v3(0, h + int(2), 0)))
            .then(&Affine::scale(v3(w * 2, int(10), d * 3))),
        rgb(255, 210, 80),
        ONE / 5 + mul(p, ONE / 8),
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
        Crew::Pirate => (if k % 2 == 0 { SHIRT_RED } else { SHIRT_WHITE }, TROUSERS),
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
        Crew::Pirate if k % 3 == 0 => {
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
                if k % 2 == 0 { FLAG_BLACK } else { SHIRT_RED },
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
    let fade = if age > ONE * 6 / 10 {
        ONE - fx::div(age - ONE * 6 / 10, ONE * 4 / 10)
    } else {
        ONE
    };
    for i in 0..5u32 {
        let reach = mul(fx::sqrt(age), int(pick(k + i, 110, 3, 7)));
        let side = cm(pick(k + i, 111, -120, 120));
        let rise = mul(age, int(pick(k + i, 112, 1, 3)));
        let s = mul(dm(15) + mul(age, int(4)), fade);
        if s <= 0 {
            continue;
        }
        let c = if age < ONE / 6 {
            mix(FLASH, SMOKE, age * 6)
        } else {
            shade(SMOKE, 100 - (age * 22 / ONE))
        };
        put(
            b,
            f,
            Mesh::Blob,
            v3(side, rise, reach),
            v3(s, s * 85 / 100, s),
            c,
            ONE / 3,
        );
    }
}

/// A splash where a ball hits the water (its foot at the origin) at `age`:
/// a white column rising and falling, a crown of spray thrown out and a
/// ring of foam spreading on the water.
pub fn splash(b: &mut Batch, f: &Affine, age: i32, k: u32) {
    let age = age.clamp(0, ONE);
    if age >= ONE {
        return;
    }
    let up = sin(mul(age, TURN / 2));
    let tall = mul(up, int(7));
    if tall > cm(10) {
        put(
            b,
            f,
            Mesh::Cone,
            v3(0, tall / 2, 0),
            v3(dm(28), tall, dm(28)),
            FOAM,
            ONE / 4,
        );
        put(
            b,
            f,
            Mesh::Blob,
            v3(0, tall * 8 / 10, 0),
            v3(dm(18), tall / 3, dm(18)),
            FOAM,
            ONE / 4,
        );
    }
    for i in 0..6u32 {
        let yaw = (i as i32) * TURN / 6 + pick(k, 120, 0, 6000);
        let out = facing(yaw, 0).scale(dm(5) + mul(age, int(3)));
        let h = mul(up, int(pick(k + i, 121, 2, 4)));
        put_turned(
            b,
            f,
            Mesh::Shard,
            out.add(v3(0, h, 0)),
            &Affine::rot_y(yaw).then(&Affine::rot_x(deg(30))),
            v3(dm(4), dm(10), dm(2)),
            FOAM,
            ONE / 8,
        );
    }
    let r = dm(15) + mul(age, int(6));
    put(
        b,
        f,
        Mesh::Hex,
        v3(0, cm(8), 0),
        v3(r, cm(6), r),
        mix(FOAM, SEA, age),
        0,
    );
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
    let s = mul(dm(10) + mul(age, int(3)), ONE - age);
    put(
        b,
        f,
        Mesh::Blob,
        v3(0, mul(age, int(1)), -mul(age, int(2))),
        v3(s, s, s),
        shade(SMOKE, 60),
        0,
    );
}

// ---------------------------------------------------------------------------
// The model gallery (`?gallery=<key>`, `app/gallery.rs`).

/// The keys `app/gallery.rs` shows from here.
pub const GALLERY: [&str; 16] = [
    "galleon",
    "galleonnavy",
    "frigate",
    "frigatenavy",
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

/// The galleon's sails in the gallery: a gap through the mainmast's lower
/// sails, the rest set (8 yards).
const GALLEON_SAILS: [Sail; 8] = [
    Sail::Set,
    Sail::Set,
    Sail::Gone,
    Sail::Torn,
    Sail::Set,
    Sail::Set,
    Sail::Set,
    Sail::Furled,
];
const FRIGATE_SAILS: [Sail; 8] = [
    Sail::Set,
    Sail::Set,
    Sail::Torn,
    Sail::Gone,
    Sail::Set,
    Sail::Furled,
    Sail::Set,
    Sail::Set,
];
const SLOOP_SAILS: [Sail; 4] = [Sail::Gone, Sail::Set, Sail::Set, Sail::Torn];

/// The middle and rough radius of a gallery model, metres.
pub fn framing(key: &str) -> (V3, i32) {
    match key {
        "galleon" | "galleonnavy" => (v3(0, int(17), int(4)), int(44)),
        "frigate" | "frigatenavy" => (v3(0, int(15), int(3)), int(37)),
        "sloop" | "sloopnavy" => (v3(0, int(10), int(2)), int(23)),
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
    match key {
        "galleon" => ship(
            b,
            f,
            ShipKind::Galleon,
            Side::Pirate,
            &GALLEON_SAILS,
            tick,
            1,
        ),
        "galleonnavy" => ship(b, f, ShipKind::Galleon, Side::Navy, &GALLEON_SAILS, tick, 2),
        "frigate" => ship(
            b,
            f,
            ShipKind::Frigate,
            Side::Pirate,
            &FRIGATE_SAILS,
            tick,
            3,
        ),
        "frigatenavy" => ship(b, f, ShipKind::Frigate, Side::Navy, &FRIGATE_SAILS, tick, 4),
        "sloop" => ship(b, f, ShipKind::Sloop, Side::Pirate, &SLOOP_SAILS, tick, 5),
        "sloopnavy" => ship(b, f, ShipKind::Sloop, Side::Navy, &SLOOP_SAILS, tick, 6),
        "sails" => {
            // every state on a frigate's main topsail yard, pirate above,
            // navy below
            let y = Yard {
                mast: 0,
                at: v3(0, int(9), 0),
                half: int(6),
                drop: int(6),
            };
            let states = [Sail::Set, Sail::Furled, Sail::Torn, Sail::Gone];
            for (i, s) in states.into_iter().enumerate() {
                for (j, side) in [Side::Pirate, Side::Navy].into_iter().enumerate() {
                    let g = f.then(&Affine::translate(v3(
                        int(i as i32 * 13 - 20),
                        -int(j as i32 * 8),
                        0,
                    )));
                    let y = Yard {
                        at: y.at.add(v3(0, 0, 0)),
                        ..y
                    };
                    yard_and_sail(b, &g, &y, s, &paint(side), side, i as u32);
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
            let up = V3::UP;
            doubloon(b, v3(-cm(80), 0, 0), right, up, 0, true);
            doubloon(b, v3(cm(80), 0, 0), right, up, deg(35), false);
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
            // side, a sloop beyond, barrels, smoke and a splash between
            let pg = f.then(&Affine::translate(v3(-int(22), 0, 0)));
            ship(
                b,
                &pg,
                ShipKind::Galleon,
                Side::Pirate,
                &GALLEON_SAILS,
                tick,
                1,
            );
            let nf = f.then(&Affine::translate(v3(int(20), 0, int(8))));
            ship(
                b,
                &nf,
                ShipKind::Frigate,
                Side::Navy,
                &FRIGATE_SAILS,
                tick,
                2,
            );
            let sl = f
                .then(&Affine::translate(v3(int(4), 0, int(60))))
                .then(&Affine::rot_y(deg(20)));
            ship(b, &sl, ShipKind::Sloop, Side::Pirate, &SLOOP_SAILS, tick, 3);
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
            for i in 0..6u32 {
                let deck = hull(ShipKind::Galleon).deck;
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
                deckhand(
                    b,
                    &g,
                    if i % 2 == 0 { Crew::Pirate } else { Crew::Navy },
                    Pose::Duel,
                    tick,
                    i,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// The per-model costs, printed with `--nocapture` (BM rows): kept
    /// under budgets so 20 ships and dozens of crew stay cheap.
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
        for kind in [ShipKind::Galleon, ShipKind::Frigate, ShipKind::Sloop] {
            for side in [Side::Pirate, Side::Navy] {
                let n = yards(kind).len();
                let set = vec![Sail::Set; n];
                let torn = vec![Sail::Torn; n];
                let (_, shapes, tris) = cost(&format!("{kind:?} {side:?} all set"), &|b| {
                    ship(b, &id, kind, side, &set, 0, 1)
                });
                assert!(shapes < 420 && tris < 14_000, "{kind:?} {shapes} {tris}");
                let (_, shapes, _) = cost(&format!("{kind:?} {side:?} all torn"), &|b| {
                    ship(b, &id, kind, side, &torn, 0, 1)
                });
                assert!(shapes < 520, "{kind:?} {shapes}");
            }
        }
        let (_, s, t) = cost("boundary barrel", &|b| boundary_barrel(b, &id, 0, 1));
        assert!(s <= 12 && t < 400, "{s} {t}");
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
        for (name, shapes, tris) in rows {
            println!("{name:32} {shapes:5} shapes {tris:6} triangles");
        }
    }

    #[test]
    fn a_set_sail_fills_its_gap_and_the_others_open_it() {
        // a ray through the middle of the main course: canvas only when set
        let kind = ShipKind::Frigate;
        let ys = yards(kind);
        assert_eq!(ys.len(), 8);
        let main_course = ys.iter().position(|y| y.mast == 1).unwrap();
        let y = ys[main_course];
        assert!(y.drop > int(6), "{}", y.drop);
        let middle_shapes = |state: Sail| {
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
        assert!(middle_shapes(Sail::Set) >= 2);
        assert_eq!(middle_shapes(Sail::Furled), 0);
        assert!(middle_shapes(Sail::Gone) <= 1);
        // the torn sail's strips leave the middle half of the span open
        // below its top strip (tatters reach at most a quarter down)
        let mut b = Batch::new();
        yard_and_sail(
            &mut b,
            &Affine::IDENTITY,
            &y,
            Sail::Torn,
            &paint(Side::Pirate),
            Side::Pirate,
            3,
        );
        assert!(b.count_of(Mesh::Cube) >= 5);
    }

    #[test]
    fn decks_sit_on_the_hull_and_ropes_hang_from_yards() {
        for kind in [ShipKind::Galleon, ShipKind::Frigate, ShipKind::Sloop] {
            let h = hull(kind);
            let ds = decks(kind);
            assert!(ds.len() >= 3, "{kind:?}");
            for d in &ds {
                assert!(d.z0 < d.z1 && d.half > dm(15), "{kind:?} {d:?}");
                assert_eq!(deck_at(&h, (d.z0 + d.z1) / 2), d.y, "{kind:?} {d:?}");
            }
            // every rope point is over the hull or a yard's reach, and
            // above every deck
            for p in rope_points(kind) {
                assert!(p.y > h.deck + int(5), "{kind:?} {p:?}");
                assert!(p.z.abs() < h.length / 2, "{kind:?} {p:?}");
            }
            // walking the waist end to end: about the brief's 10 s for
            // the big ships at a run (5 m/s)
            let walk = ds.iter().map(|d| d.z1 - d.z0).sum::<i32>();
            if kind == ShipKind::Galleon {
                assert!(walk > int(40) && walk < int(55), "{walk}");
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
