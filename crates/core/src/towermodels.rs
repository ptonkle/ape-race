//! The tower map's models for the climbing half (2026-10-15,
//! `briefs/mapes-tower.md`, step 4 "Models"): stand-alone builders in the
//! faceted low-poly style of `ruinsmodels.rs` and `baymodels.rs`. No course
//! places them yet. Each is built in its own metres, +y up, +z its front
//! (a balcony's outward side, a dragon's nose, a rift's face), its feet or
//! base at y 0, and placed by a frame (`pose`) the caller passes. Each
//! takes a few plain parameters (a radius, a height, a spell, a seed `k`);
//! the stone is still (no tick, so it can go in the chunked scenery) and
//! only the magic and the creatures move with the tick.
//!
//! **The hour: twilight (and dawn, the second weather).** A low rose sun
//! a few degrees over the horizon, a deep indigo sky overhead, the lit
//! windows warm. Step 5 owns the map's real `Look`; `gallery_look` here
//! only lights the model shots the same way.
//!
//! A palette no other map uses: pale honey limestone in courses, indigo
//! slate roofs, amber lit windows, the school's banners deep violet and
//! gold, and the magic in five vivid spell colours (`Spell`).
//!
//! The parts, by file: `towermodels/keep.rs` the main keep, sub-towers,
//! turrets, bridges and balconies; `folk.rs` the mages and the dragons with
//! their riders; `magic.rs` spell beams, trampolines, platforms, clouds,
//! the trampoline room; `summit.rs` the insane wizard, the tear and the
//! eldritch banana.

use crate::fx::{deg, int, mul, sin, ONE, TURN};
use crate::geom::{facing, v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{rgb, rod, Batch, Rgb};
use crate::scene::Look;

pub use crate::ruinsmodels::pose;
pub(crate) use crate::ruinsmodels::{pick, wave};

mod creatures;
mod deep;
mod folk;
mod keep;
mod magic;
mod portal;
mod summit;
mod towers;
mod tube;
mod zipper;

pub use creatures::*;
pub use deep::*;
pub use folk::*;
pub use keep::*;
pub use magic::*;
pub use portal::*;
pub use summit::*;
pub use towers::*;
pub use tube::*;
pub use zipper::*;

// ---------------------------------------------------------------------------
// The gallery's light (step 5 owns the map's).

/// The sun's height over the horizon at twilight.
pub const SUN_HIGH: i32 = deg(9);
/// The sun's bearing in the gallery.
pub const SUN_YAW: i32 = deg(40);
pub const SKY: Rgb = rgb(62, 56, 122);
pub const FOG: Rgb = rgb(104, 92, 156);
pub const LIGHT: Rgb = rgb(300, 214, 186);
/// The sky's indigo, light from above and the shaded side.
pub const AMBIENT: Rgb = rgb(96, 92, 150);

/// Twilight light for the model gallery (`?gallery=keep`).
pub fn gallery_look() -> Look {
    Look {
        sky: SKY,
        fog: FOG,
        fog_range: (int(300), int(900)),
        light_dir: facing(SUN_YAW, SUN_HIGH),
        light: LIGHT,
        ground: AMBIENT,
        rim: [0; 3],
        far: crate::render::FAR,
    }
}

/// The gallery's light for model `key`: the zipper space's for the
/// descent's models, twilight for the rest.
pub fn gallery_look_of(key: &str) -> Look {
    if ZIPPER_KEYS.contains(&key) {
        zipper_look()
    } else {
        gallery_look()
    }
}

/// The descent's gallery keys, shown in the zipper space's light.
pub const ZIPPER_KEYS: [&str; 11] = [
    "tube",
    "voidtrampolines",
    "tunnelmouth",
    "zip",
    "topsy",
    "debris",
    "blob",
    "seamonster",
    "voidplatforms",
    "bananasea",
    "zipper",
];

// ---------------------------------------------------------------------------
// The palette.

pub(crate) const STONE: Rgb = rgb(206, 192, 168);
pub(crate) const STONE_LIGHT: Rgb = rgb(224, 212, 188);
pub(crate) const STONE_WARM: Rgb = rgb(196, 176, 148);
pub(crate) const STONE_DARK: Rgb = rgb(150, 138, 124);
pub(crate) const MORTAR: Rgb = rgb(84, 76, 78);
pub(crate) const SLATE: Rgb = rgb(62, 70, 112);
pub(crate) const SLATE_DARK: Rgb = rgb(42, 46, 80);
pub(crate) const WINDOW: Rgb = rgb(255, 196, 108);
pub(crate) const SLIT: Rgb = rgb(26, 22, 32);
pub(crate) const WOOD: Rgb = rgb(112, 78, 50);
pub(crate) const WOOD_DARK: Rgb = rgb(78, 54, 36);
pub(crate) const ROPE: Rgb = rgb(184, 156, 108);
pub(crate) const BANNER: Rgb = rgb(70, 38, 140);
pub(crate) const GOLD: Rgb = rgb(240, 190, 60);
pub(crate) const FUR: Rgb = rgb(110, 72, 44);
pub(crate) const FACE: Rgb = rgb(206, 166, 120);
pub(crate) const BEARD: Rgb = rgb(232, 230, 222);

/// The kinds of spell the mages cast, each its own colour. The beams are
/// the climb's vines; the course chooses which spell sets which
/// orientation (the brief: "different magic spells which control the
/// orientation of the vine").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spell {
    /// violet
    Arcane,
    /// ice blue
    Frost,
    /// orange
    Fire,
    /// green
    Verdant,
    /// pink
    Charm,
    /// sea-green (the descent's creatures)
    Brine,
    /// deep violet (the descent's creatures)
    Abyss,
}

impl Spell {
    pub const ALL: [Spell; 5] = [
        Spell::Arcane,
        Spell::Frost,
        Spell::Fire,
        Spell::Verdant,
        Spell::Charm,
    ];

    /// The spell's bright colour (a beam's core, a staff's orb).
    pub const fn colour(self) -> Rgb {
        match self {
            Spell::Arcane => rgb(196, 110, 255),
            Spell::Frost => rgb(120, 226, 255),
            Spell::Fire => rgb(255, 140, 44),
            Spell::Verdant => rgb(96, 244, 128),
            Spell::Charm => rgb(255, 118, 196),
            Spell::Brine => rgb(70, 255, 190),
            Spell::Abyss => rgb(150, 96, 255),
        }
    }

    /// The robes of a mage of this school: the spell's colour, deep.
    pub const fn robe(self) -> Rgb {
        match self {
            Spell::Arcane => rgb(78, 40, 132),
            Spell::Frost => rgb(36, 82, 132),
            Spell::Fire => rgb(150, 44, 30),
            Spell::Verdant => rgb(34, 100, 58),
            Spell::Charm => rgb(132, 40, 96),
            Spell::Brine => rgb(20, 90, 80),
            Spell::Abyss => rgb(60, 24, 100),
        }
    }

    /// The descent's two spells, the sea monsters' and blobs' vines.
    pub const DEEP: [Spell; 2] = [Spell::Brine, Spell::Abyss];

    /// Spell `k` of the five, round and round.
    pub fn nth(k: u32) -> Spell {
        Spell::ALL[(k % 5) as usize]
    }
}

// ---------------------------------------------------------------------------
// Helpers.

/// A shape of `size` with its middle at `at`, in frame `f`.
pub(crate) fn put(b: &mut Batch, f: &Affine, mesh: Mesh, at: V3, size: V3, c: Rgb, glow: i32) {
    b.push(
        mesh,
        &f.then(&Affine::translate(at)).then(&Affine::scale(size)),
        c,
        glow,
    );
}

/// The same, turned by `turn` about its middle.
pub(crate) fn put_turned(
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
pub(crate) fn bar(b: &mut Batch, f: &Affine, mesh: Mesh, a: V3, z: V3, w: i32, c: Rgb, glow: i32) {
    if a.dist(z) > ONE / 100 {
        b.push(mesh, &f.then(&rod(a, z, w)), c, glow);
    }
}

/// A soft glow round a rod from `a` to `z`.
pub(crate) fn halo_bar(b: &mut Batch, f: &Affine, a: V3, z: V3, w: i32, c: Rgb, strength: i32) {
    if a.dist(z) > ONE / 100 {
        b.push_halo(Mesh::Cyl, &f.then(&rod(a, z, w)), c, strength);
    }
}

/// A soft glowing ball of diameter `d` at `at`.
pub(crate) fn halo_ball(b: &mut Batch, f: &Affine, at: V3, d: i32, c: Rgb, strength: i32) {
    b.push_halo(
        Mesh::Sphere,
        &f.then(&Affine::translate(at))
            .then(&Affine::scale(v3(d, d, d))),
        c,
        strength,
    );
}

/// Metres, from tenths (`dm(125)` is 12.5 m).
pub(crate) const fn dm(x: i32) -> i32 {
    x * (ONE / 10)
}

/// The point `r` out from the y axis at bearing `a` (0 is +z, a quarter
/// turn +x), at height `y`.
pub(crate) fn round(r: i32, a: i32, y: i32) -> V3 {
    v3(mul(r, sin(a)), y, mul(r, crate::fx::cos(a)))
}

/// A pulse in `lo..ONE`, period `secs`, phase `k`.
pub(crate) fn pulse(tick: u32, secs: i32, k: u32, lo: i32) -> i32 {
    let w = wave(tick, secs, (k as i32).wrapping_mul(7_919));
    lo + mul((w + ONE) / 2, ONE - lo)
}

// ---------------------------------------------------------------------------
// The gallery (`?gallery=<key>`).

pub const GALLERY: [&str; 38] = [
    "tower1",
    "hollowsection",
    "hollowtop",
    "tower2",
    "longbridge",
    "towers",
    "portal",
    "tube",
    "voidtrampolines",
    "tunnelmouth",
    "keep",
    "keepsection",
    "turret",
    "subtower",
    "bridges",
    "balcony",
    "mages",
    "dragon",
    "dragons",
    "beams",
    "trampolines",
    "trampolineroom",
    "clouds",
    "conjured",
    "madwizard",
    "rift",
    "eldritch",
    "summit",
    "climb",
    "towerall",
    "zip",
    "topsy",
    "debris",
    "blob",
    "seamonster",
    "voidplatforms",
    "bananasea",
    "zipper",
];

/// The middle and rough radius of a gallery model, metres.
pub fn framing(key: &str) -> (V3, i32) {
    match key {
        "tower1" => (v3(0, int(137), 0), int(200)),
        "hollowsection" => (v3(0, int(12), 0), int(70)),
        "hollowtop" => (v3(0, int(4), 0), int(70)),
        "tower2" => (v3(0, int(82), 0), int(120)),
        "longbridge" => (v3(0, -int(2), 0), int(140)),
        // (the pair is drawn centred on the turn, `towers_scene`: both
        // in view all the way round, TA2-4)
        "towers" => (v3(0, int(130), 0), int(250)),
        "portal" => (v3(0, int(25), 0), int(36)),
        "tube" => (v3(0, -int(60), 0), int(110)),
        "voidtrampolines" => (v3(0, 0, 0), int(30)),
        // (the camera 75 m off the mouth: outside the ring and past the
        // 40 m of tunnel whichever way it turns, TA2-4)
        "tunnelmouth" => (V3::ZERO, int(44)),
        "keep" => (v3(0, int(56), 0), int(84)),
        "keepsection" => (v3(0, int(6), 0), int(14)),
        "turret" => (v3(0, int(9), 0), int(9)),
        "subtower" => (v3(0, int(20), 0), int(22)),
        "bridges" => (v3(0, int(2), 0), int(26)),
        "balcony" => (v3(0, int(2), int(1)), int(5)),
        "mages" => (v3(0, int(1), 0), int(5)),
        "dragon" => (v3(0, int(1), 0), int(11)),
        "dragons" => (v3(0, int(4), 0), int(30)),
        "beams" => (v3(0, int(8), 0), int(16)),
        "trampolines" => (v3(0, int(1), 0), int(9)),
        "trampolineroom" => (v3(0, int(10), 0), int(22)),
        "clouds" => (v3(0, int(4), 0), int(26)),
        "conjured" => (v3(0, int(1), 0), int(10)),
        "madwizard" => (v3(0, int(3), 0), int(4)),
        "rift" => (v3(0, int(16), 0), int(22)),
        "eldritch" => (v3(0, 0, 0), int(48)),
        "summit" => (v3(0, int(20), -int(30)), int(70)),
        "climb" => (v3(0, int(30), 0), int(48)),
        "zip" => (v3(0, 0, 0), int(36)),
        "topsy" => (v3(0, 0, 0), int(34)),
        "debris" => (v3(0, 0, 0), int(18)),
        "blob" => (v3(0, int(5), 0), int(12)),
        "seamonster" => (v3(0, int(4), 0), int(7)),
        "voidplatforms" => (v3(0, 0, 0), int(14)),
        "bananasea" => (v3(0, int(3), 0), int(40)),
        "zipper" => (v3(0, 0, 0), int(60)),
        _ => (v3(0, int(70), 0), int(110)),
    }
}

/// Draws gallery model `key` in frame `f` (turning) at `tick`.
pub fn gallery(b: &mut Batch, key: &str, f: &Affine, tick: u32) {
    let at = |x: i32, y: i32, z: i32| f.then(&Affine::translate(v3(x, y, z)));
    match key {
        "tower1" => tower1(
            b,
            f,
            TOWER1_OUT,
            TOWER1_IN,
            TOWER1_HEIGHT,
            TOWER1_HEIGHT / 2,
            2,
            7,
        ),
        "hollowsection" => {
            // two sections, a window and the doorway, a ledge on the upper
            let mut cuts = [Cut::Wall; 8];
            cuts[0] = Cut::Door;
            cuts[3] = Cut::Window;
            hollow_section(
                b,
                f,
                TOWER1_OUT,
                TOWER1_IN,
                SECTION_H,
                1,
                cuts,
                Ledge::None,
                Detail::Near,
            );
            let mut cuts = [Cut::Wall; 8];
            cuts[1] = Cut::Window;
            cuts[5] = Cut::Window;
            hollow_section(
                b,
                &at(0, SECTION_H, 0),
                TOWER1_OUT,
                TOWER1_IN,
                SECTION_H,
                2,
                cuts,
                Ledge::Walk,
                Detail::Near,
            );
            let g = inner_balcony(
                b,
                &at(0, SECTION_H, 0),
                TOWER1_IN,
                deg(90),
                int(6),
                int(5),
                1,
            );
            mage(b, &g, Spell::Frost, MagePose::Cast, tick, 1);
        }
        "hollowtop" => hollow_top(b, f, TOWER1_OUT, TOWER1_IN, 4, 1),
        "tower2" => {
            let frames = tower2(b, f, TOWER2_RADIUS, TOWER2_HEIGHT, 8, TURN * 3 / 2, 0, 6, 3);
            for (i, g) in frames.iter().enumerate() {
                mage(b, g, Spell::nth(i as u32), MagePose::Cast, tick, i as u32);
            }
        }
        "longbridge" => {
            let (a, z) = (v3(-int(125), 0, 0), v3(int(125), 0, 0));
            long_bridge(b, f, a, z, BRIDGE_WIDTH, 1);
            bridge_hangers(b, f, a, z, BRIDGE_WIDTH, HANGER_SPACING, 1);
        }
        "towers" => towers_scene(b, f, tick),
        "portal" => {
            portal(b, f, TEAR_W, TEAR_H, ONE, int(1), tick);
            banana_in_rift(b, f, TEAR_H, int(30), ONE / 2, tick);
        }
        "tube" => {
            let holes = [
                Porthole {
                    depth: int(30),
                    bearing: deg(60),
                },
                Porthole {
                    depth: int(70),
                    bearing: deg(200),
                },
                Porthole {
                    depth: int(110),
                    bearing: deg(320),
                },
            ];
            let frames = tube(b, f, TUBE_RADIUS, int(130), &holes, 1);
            for (i, g) in frames.iter().enumerate() {
                if i % 2 == 0 {
                    blob(b, g, int(3), Spell::Brine, tick, i as u32);
                } else {
                    sea_monster(b, g, Spell::Abyss, tick, i as u32);
                }
            }
            zip(
                b,
                &at(0, -int(5), -(TUBE_RADIUS - int(2))),
                int(120),
                int(2),
                ONE / 3,
                tick,
            );
            for i in 0..9u32 {
                let a = deg(97) * i as i32;
                let lean = Affine::rot_x(deg(i as i32 % 11));
                let f = at(0, -int(15) - int(12) * i as i32, 0)
                    .then(&Affine::translate(round(int(18), a, 0)))
                    .then(&Affine::rot_y(a))
                    .then(&lean);
                void_trampoline(b, &f, crate::tower::DISC_RADIUS, 0, tick, i);
            }
        }
        "voidtrampolines" => {
            for i in 0..3 {
                let f = at(int(i * 14 - 14), 0, 0).then(&Affine::rot_x(deg(5 * i)));
                void_trampoline(
                    b,
                    &f,
                    crate::tower::DISC_RADIUS,
                    wave(tick, 2, i * 9_000).max(0),
                    tick,
                    i as u32,
                );
            }
        }
        "tunnelmouth" => {
            tunnel_mouth(b, f, TUNNEL_RADIUS, tick, 1);
            tunnel(b, f, TUNNEL_RADIUS, int(40), 1);
        }
        "keep" => keep(b, f, int(10), 8, 7),
        "keepsection" => keep_section(b, f, int(10), int(12), 3, Detail::Near),
        "turret" => turret(b, f, int(3), int(12), 2),
        "subtower" => sub_tower(b, f, int(6), int(30), 4),
        "bridges" => {
            arch_bridge(
                b,
                f,
                v3(-int(12), 0, -int(10)),
                v3(int(12), 0, -int(10)),
                int(4),
                1,
            );
            rope_bridge(
                b,
                f,
                v3(-int(12), 0, 0),
                v3(int(12), 0, 0),
                int(2),
                int(2),
                2,
            );
            magic_bridge(
                b,
                f,
                v3(-int(12), 0, int(10)),
                v3(int(12), int(3), int(10)),
                int(3),
                Spell::Frost,
                tick,
            );
        }
        "balcony" => {
            balcony(b, f, int(5), int(2), 1);
            mage(b, &at(0, 0, int(1)), Spell::Arcane, MagePose::Cast, tick, 1);
        }
        "mages" => {
            let poses = [
                MagePose::Cast,
                MagePose::Aim,
                MagePose::Idle,
                MagePose::Ride,
                MagePose::Cast,
            ];
            for (i, s) in Spell::ALL.into_iter().enumerate() {
                let x = int(i as i32 * 2 - 4);
                mage(b, &at(x, 0, 0), s, poses[i], tick, i as u32);
            }
        }
        "dragon" => {
            dragon(b, f, Hide::Emerald, Wings::Flap, Spell::Fire, tick, 1);
        }
        "dragons" => {
            let hides = [Hide::Emerald, Hide::Crimson, Hide::Bronze, Hide::Midnight];
            let wings = [Wings::Flap, Wings::Glide, Wings::Bank, Wings::Dive];
            for i in 0..4 {
                let g = at(
                    int((i % 2) * 26 - 13),
                    int((i / 2) * 14 - 4),
                    int((i % 2) * 6),
                );
                dragon(
                    b,
                    &g,
                    hides[i as usize],
                    wings[i as usize],
                    Spell::nth(i as u32 + 1),
                    tick + i as u32 * 17,
                    i as u32,
                );
            }
        }
        "beams" => {
            for (i, s) in Spell::ALL.into_iter().enumerate() {
                let x = int(i as i32 * 5 - 10);
                spell_beam(
                    b,
                    f,
                    v3(x, int(16), 0),
                    v3(x + int(i as i32 - 2) * 2, 0, int(2)),
                    s,
                    tick,
                    i as u32,
                );
                spell_anchor(b, f, v3(x, int(16), 0), s, tick, i as u32);
            }
        }
        "trampolines" => {
            trampoline(
                b,
                &at(-int(6), 0, 0),
                int(2),
                Pad::Rune,
                Spell::Arcane,
                wave(tick, 2, 0).max(0),
                tick,
            );
            trampoline(
                b,
                f,
                int(2),
                Pad::Cloth,
                Spell::Charm,
                wave(tick, 2, 9_000).max(0),
                tick,
            );
            trampoline(
                b,
                &at(int(6), 0, 0),
                int(2),
                Pad::Jelly,
                Spell::Verdant,
                wave(tick, 2, 20_000).max(0),
                tick,
            );
        }
        "trampolineroom" => trampoline_room(b, f, int(14), int(22), int(5), true, tick, 1),
        "clouds" => {
            cloud(b, &at(-int(10), 0, 0), int(22), int(7), 1);
            cloud(b, &at(int(12), int(4), -int(6)), int(16), int(5), 2);
            cloud_anchor(b, &at(-int(10), int(3), 0), Spell::Frost, tick, 1);
            cloud_anchor(b, &at(int(12), int(7), -int(6)), Spell::Arcane, tick, 2);
        }
        "conjured" => {
            frost_disc(b, &at(-int(6), 0, 0), int(2), 1);
            rune_stone(b, f, int(3), tick, 2);
            crystal_ledge(b, &at(int(6), 0, 0), int(4), int(2), 3);
        }
        "madwizard" => mad_wizard(b, f, tick),
        "rift" => rift(b, f, int(14), int(30), ONE, tick),
        "eldritch" => eldritch_banana(b, f, int(60), tick),
        "summit" => summit(b, f, tick),
        "climb" => climb_scene(b, f, tick),
        "zip" => zip(b, &at(0, int(30), 0), int(60), int(2), ONE * 2 / 5, tick),
        "topsy" => {
            topsy_keep(b, &at(-int(14), int(20), 0), int(8), 2, deg(15), 1);
            topsy(b, &at(int(14), int(16), 0), deg(30), -deg(25), |b, g| {
                turret(b, g, int(3), int(14), 2);
            });
            topsy(b, &at(0, -int(12), int(4)), 0, deg(20), |b, g| {
                arch_bridge(b, g, v3(-int(10), 0, 0), v3(int(10), 0, 0), int(4), 3);
            });
        }
        "debris" => debris(b, f, int(14), tick, 1),
        "blob" => {
            for i in 0..3u32 {
                let x = int(i as i32 * 6 - 6);
                let size = int(2) + dm(5) * i as i32;
                let spell = Spell::DEEP[(i % 2) as usize];
                let tip = blob(b, &at(x, 0, 0), size, spell, tick, i);
                let end = tip.add(v3(int(3) - int(3) * i as i32, int(8), -int(4)));
                sea_beam(b, &Affine::IDENTITY, tip, end, spell, tick, i);
            }
        }
        "seamonster" => {
            let tip = sea_monster(b, f, Spell::Brine, tick, 1);
            let end = f.apply(MONSTER_STAFF_TIP.add(v3(int(6), int(7), -int(4))));
            sea_beam(b, &Affine::IDENTITY, tip, end, Spell::Brine, tick, 1);
        }
        "voidplatforms" => {
            wreck_plank(b, &at(-int(9), int(2), 0), int(8), int(4), 1);
            coral_shelf(b, &at(0, -int(1), int(2)), int(3), 2);
            bubble_disc(b, &at(int(9), int(1), 0), int(3), tick, 3);
        }
        "bananasea" => banana_sea(b, f, int(400), tick),
        "zipper" => zipper_scene(b, f, tick),
        _ => {
            keep(b, f, int(14), 10, 7);
            sub_tower(b, &at(int(44), 0, int(10)), int(6), int(56), 3);
            arch_bridge(
                b,
                f,
                v3(int(14), int(48), int(4)),
                v3(int(38), int(48), int(9)),
                int(4),
                2,
            );
            sub_tower(b, &at(-int(36), 0, -int(20)), int(5), int(84), 5);
            rope_bridge(
                b,
                f,
                v3(-int(13), int(76), -int(6)),
                v3(-int(31), int(76), -int(18)),
                int(2),
                int(2),
                3,
            );
            cloud(b, &at(int(20), int(96), int(20)), int(30), int(8), 3);
            cloud(b, &at(-int(26), int(110), int(14)), int(26), int(7), 4);
            dragon(
                b,
                &at(int(30), int(70), int(30)),
                Hide::Crimson,
                Wings::Flap,
                Spell::Fire,
                tick,
                2,
            );
        }
    }
}

/// The second design's two towers and their bridges together, at the
/// sizes of the plan: tower 1 hollow at the origin with its doorway
/// facing tower 2, tower 2 a bridge's length along +x, the lower bridge a
/// third of the way up tower 2 and the upper at its top, hangers under
/// both, the portal beside tower 1's top on its far side with the mad
/// wizard before it. About 5,400 shapes: for the gallery only.
fn towers_scene(b: &mut Batch, f: &Affine, tick: u32) {
    let gap = BRIDGE_LENGTH + TOWER1_OUT + TOWER2_RADIUS;
    // (tower 1 at -gap/2, tower 2 at +gap/2: the pair turns about its
    // middle)
    let f = &f.then(&Affine::translate(v3(-gap / 2, 0, 0)));
    let at = |x: i32, y: i32, z: i32| f.then(&Affine::translate(v3(x, y, z)));
    // the doorway faces +x (face 2)
    tower1(
        b,
        f,
        TOWER1_OUT,
        TOWER1_IN,
        TOWER1_HEIGHT,
        TOWER1_HEIGHT / 2,
        2,
        7,
    );
    let frames = tower2(
        b,
        &at(gap, 0, 0),
        TOWER2_RADIUS,
        TOWER2_HEIGHT,
        8,
        TURN * 3 / 2,
        0,
        6,
        3,
    );
    for (i, g) in frames.iter().enumerate() {
        mage(b, g, Spell::nth(i as u32), MagePose::Cast, tick, i as u32);
    }
    for (y, k) in [(TOWER2_HEIGHT / 3, 1), (TOWER2_HEIGHT - int(8), 2)] {
        let (a, z) = (v3(TOWER1_OUT, y, 0), v3(gap - TOWER2_RADIUS, y, 0));
        long_bridge(b, f, a, z, BRIDGE_WIDTH, k);
        bridge_hangers(b, f, a, z, BRIDGE_WIDTH, HANGER_SPACING, k);
    }
    // the portal beside the top on the far side, facing the tower
    let top = tower1_top(TOWER1_HEIGHT);
    let pf = at(-TOWER1_OUT - int(30), top - int(10), 0).then(&Affine::rot_y(deg(90)));
    portal(b, &pf, TEAR_W, TEAR_H, ONE, int(1), tick);
    banana_in_rift(b, &pf, TEAR_H, int(30), ONE / 2, tick);
    let wiz = at(-TOWER1_OUT - int(8), top, int(12)).then(&Affine::rot_y(deg(-90)));
    mad_wizard(b, &wiz, tick);
}

/// A stretch of the climb, to see the pieces together: a keep section
/// with two balconies and their mages, a sub-tower joined by an arch
/// bridge, a dragon with its beam, a trampoline, a cloud with an anchor.
fn climb_scene(b: &mut Batch, f: &Affine, tick: u32) {
    let at = |x: i32, y: i32, z: i32| f.then(&Affine::translate(v3(x, y, z)));
    for s in 0..5 {
        keep_section(
            b,
            &at(0, int(s * 12), 0),
            int(12),
            int(12),
            s as u32,
            Detail::Near,
        );
    }
    keep_crown(b, &at(0, int(60), 0), int(12), 4, 1);
    for (i, (yaw, y)) in [(deg(10), 14), (deg(-70), 38)].into_iter().enumerate() {
        let g = at(0, int(y), 0)
            .then(&Affine::rot_y(yaw))
            .then(&Affine::translate(v3(0, 0, int(12))));
        balcony(b, &g, int(5), int(2), i as u32);
        mage(
            b,
            &g.then(&Affine::translate(v3(0, 0, int(1)))),
            Spell::nth(i as u32),
            MagePose::Cast,
            tick,
            i as u32,
        );
        let tip = g.apply(staff_tip(MagePose::Cast).add(v3(0, 0, int(1))));
        let end = tip.add(v3(int(8 - i as i32 * 16), -int(10), int(6)));
        spell_beam(
            b,
            &Affine::IDENTITY,
            tip,
            end,
            Spell::nth(i as u32),
            tick,
            i as u32,
        );
    }
    sub_tower(b, &at(int(34), 0, int(8)), int(5), int(40), 2);
    arch_bridge(
        b,
        f,
        v3(int(12), int(26), int(4)),
        v3(int(29), int(26), int(7)),
        int(4),
        1,
    );
    trampoline(
        b,
        &at(int(20), int(26), int(6)),
        int(2),
        Pad::Rune,
        Spell::Arcane,
        0,
        tick,
    );
    cloud(b, &at(-int(10), int(54), int(20)), int(20), int(6), 2);
    cloud_anchor(b, &at(-int(10), int(57), int(20)), Spell::Frost, tick, 1);
    dragon(
        b,
        &at(-int(26), int(30), int(22)),
        Hide::Bronze,
        Wings::Flap,
        Spell::Verdant,
        tick,
        3,
    );
}

/// A stretch of the descent, to see the pieces together: the zip down
/// the middle, an upside-down keep and sub-tower tilted beside it, a
/// blob on a bubble disc and a sea monster on a wreck plank casting their
/// beams at the zip, a coral shelf below and debris all round.
fn zipper_scene(b: &mut Batch, f: &Affine, tick: u32) {
    let at = |x: i32, y: i32, z: i32| f.then(&Affine::translate(v3(x, y, z)));
    zip(b, &at(0, int(50), -int(6)), int(100), int(3), ONE / 2, tick);
    topsy_keep(b, &at(-int(34), int(44), -int(10)), int(10), 3, deg(18), 2);
    topsy(
        b,
        &at(int(32), int(36), -int(16)),
        deg(40),
        -deg(28),
        |b, g| {
            sub_tower(b, g, int(5), int(34), 3);
        },
    );
    bubble_disc(b, &at(-int(14), int(10), int(10)), int(3), tick, 1);
    let tip = blob(
        b,
        &at(-int(14), int(10), int(10)),
        int(3),
        Spell::Brine,
        tick,
        1,
    );
    let end = f.apply(v3(-int(4), int(24), -int(4)));
    sea_beam(b, &Affine::IDENTITY, tip, end, Spell::Brine, tick, 1);
    wreck_plank(b, &at(int(14), -int(8), int(8)), int(9), int(4), 2);
    let mf = at(int(14), -int(8), int(8)).then(&Affine::rot_y(-deg(40)));
    let tip = sea_monster(b, &mf, Spell::Abyss, tick, 2);
    let end = f.apply(v3(int(4), int(6), -int(4)));
    sea_beam(b, &Affine::IDENTITY, tip, end, Spell::Abyss, tick, 2);
    coral_shelf(b, &at(-int(6), -int(30), int(12)), int(4), 3);
    debris(b, f, int(48), tick, 4);
}

#[cfg(test)]
mod tests;
