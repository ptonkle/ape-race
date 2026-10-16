//! The magic: spell beams (this map's vines) and their anchors,
//! trampolines, conjured platforms, clouds with their anchors, and the
//! trampoline room with the orb mage.

use super::keep::{section, side_of, slab, Detail};
use super::{bar, halo_ball, halo_bar, mage, pick, pulse, put, put_turned, round, MagePose, Spell};
use super::{GOLD, SLATE, SLATE_DARK, STONE, STONE_DARK, WINDOW, WOOD};
use crate::fx::{cm, cos, deg, int, mul, sin, HALF, ONE, QUARTER, TURN};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, rgb, shade, Batch, Rgb};

const WHITE: Rgb = [ONE; 3];

// ---------------------------------------------------------------------------
// Spell beams.

/// A beam's soft glow, as a multiple of the drawn vine's width (as the
/// City's `WIRE_HALO`): for the course's `VineLook.halo` on this map.
pub const BEAM_HALO: i32 = 3;

/// The colour a spell-beam vine is drawn in (`VineLook.colour`): the
/// spell's colour paled a little toward white so it reads as light, and
/// its glow. The in-reach and colourblind looks stay the shared ones.
pub fn beam_look(spell: Spell) -> (Rgb, i32) {
    (mix(spell.colour(), WHITE, ONE / 5), ONE * 4 / 5)
}

/// Two directions square to `d` and to each other.
fn across(d: V3) -> (V3, V3) {
    let n = d.norm();
    let mut s = n.cross(V3::UP).norm();
    if s == V3::ZERO {
        s = n.cross(v3(ONE, 0, 0)).norm();
    }
    (s, s.cross(n).norm())
}

/// A spell beam from `a` (the staff tip) to `z` (its end), the look of
/// a vine on this map: a thin bright core, one strand of the spell's
/// colour twisting round it, three motes running down it, and a soft
/// glow. Thin, so a few in view never make a wall. About 3 shapes a
/// 3 m, plus 3 motes.
pub fn spell_beam(b: &mut Batch, f: &Affine, a: V3, z: V3, spell: Spell, tick: u32, k: u32) {
    let d = z.sub(a);
    let len = d.len();
    if len < cm(10) {
        return;
    }
    let (s, u) = across(d);
    let (core, glow) = beam_look(spell);
    let c = spell.colour();
    let n = (len / int(3)).clamp(2, 8);
    let spin = (tick as i32).wrapping_mul(TURN / 90) + k as i32 * 9_000;
    let at = |i: i32, r: i32, turn: i32| {
        let t = ONE * i / n;
        let p = a.lerp(z, t);
        let ang = spin + turn * i;
        p.add(s.scale(mul(r, cos(ang))))
            .add(u.scale(mul(r, sin(ang))))
    };
    for i in 0..n {
        bar(
            b,
            f,
            Mesh::Hex,
            at(i, 0, 0),
            at(i + 1, 0, 0),
            cm(12),
            core,
            glow,
        );
        // the strand, twisting a third of a turn a segment
        let r = cm(22);
        let m = |i2: i32| {
            let t = ONE * i2 / (2 * n);
            let ang = spin + TURN / 6 * i2;
            a.lerp(z, t)
                .add(s.scale(mul(r, cos(ang))))
                .add(u.scale(mul(r, sin(ang))))
        };
        bar(b, f, Mesh::Cube, m(2 * i), m(2 * i + 1), cm(5), c, ONE);
        bar(b, f, Mesh::Cube, m(2 * i + 1), m(2 * i + 2), cm(5), c, ONE);
    }
    for j in 0..3u32 {
        let t = ((tick.wrapping_mul(ONE as u32 / 90)).wrapping_add(j * ONE as u32 / 3 + k * 7_777))
            % ONE as u32;
        put(
            b,
            f,
            Mesh::Octa,
            a.lerp(z, t as i32),
            v3(cm(26), cm(26), cm(26)),
            mix(c, WHITE, ONE / 2),
            ONE,
        );
    }
    halo_bar(b, f, a, z, cm(70), c, ONE / 2);
}

/// A rune ring in `spell`'s colour with a crystal at its middle, `at`:
/// where a beam starts when no mage holds it (a cloud's anchor, a glyph
/// on a wall). About 12 shapes and a halo.
pub fn spell_anchor(b: &mut Batch, f: &Affine, at: V3, spell: Spell, tick: u32, k: u32) {
    let c = spell.colour();
    let spin = (tick as i32).wrapping_mul(TURN / 240) + k as i32 * 5_000;
    let g = f
        .then(&Affine::translate(at))
        .then(&Affine::rot_y(spin))
        .then(&Affine::rot_x(deg(18)));
    let r = cm(90);
    for i in 0..10 {
        let p = round(r, TURN * i / 10, 0);
        let q = round(r, TURN * (i + 1) / 10, 0);
        bar(b, &g, Mesh::Cube, p, q, cm(8), c, ONE);
    }
    let p = pulse(tick, 2, k, ONE * 3 / 4);
    put(
        b,
        &g,
        Mesh::Octa,
        V3::ZERO,
        v3(mul(cm(60), p), cm(90), mul(cm(60), p)),
        mix(c, WHITE, ONE / 3),
        ONE,
    );
    halo_ball(b, f, at, int(3), c, ONE * 7 / 10);
}

// ---------------------------------------------------------------------------
// Trampolines.

/// The kinds of bounce pad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pad {
    /// a glowing rune circle on a stone disc, a column of light over it
    Rune,
    /// a stretched cloth on a brass ring on four legs
    Cloth,
    /// a conjured jelly disc that squashes
    Jelly,
}

/// The height of a pad's bouncing surface over its foot, at rest.
pub fn pad_top(pad: Pad) -> i32 {
    match pad {
        Pad::Rune => cm(55),
        Pad::Cloth | Pad::Jelly => cm(70),
    }
}

/// A trampoline of radius `r`, its foot at the origin, in `spell`'s
/// colour; `squash` 0..ONE is how hard it is being bounced on right now
/// (the cloth dips, the jelly flattens, the runes flare).
pub fn trampoline(
    b: &mut Batch,
    f: &Affine,
    r: i32,
    pad: Pad,
    spell: Spell,
    squash: i32,
    tick: u32,
) {
    let c = spell.colour();
    let sq = squash.clamp(0, ONE);
    match pad {
        Pad::Rune => {
            let across = mul(2 * r, ONE * 115 / 100);
            put(
                b,
                f,
                Mesh::Hex,
                v3(0, cm(25), 0),
                v3(across, cm(50), across),
                STONE_DARK,
                0,
            );
            put(
                b,
                f,
                Mesh::Hex,
                v3(0, cm(52), 0),
                v3(across - cm(30), cm(6), across - cm(30)),
                shade(STONE_DARK, 70),
                0,
            );
            let lit = mix(c, WHITE, sq / 2);
            let y = cm(57);
            let rr = mul(r, ONE * 85 / 100);
            for i in 0..12 {
                bar(
                    b,
                    f,
                    Mesh::Cube,
                    round(rr, TURN * i / 12, y),
                    round(rr, TURN * (i + 1) / 12, y),
                    cm(12),
                    lit,
                    ONE,
                );
            }
            // a six-pointed star: two triangles
            let rs = mul(r, ONE * 7 / 10);
            for t in 0..2 {
                for i in 0..3 {
                    let a0 = TURN * (2 * i + t) / 6;
                    let a1 = TURN * (2 * i + 2 + t) / 6;
                    bar(
                        b,
                        f,
                        Mesh::Cube,
                        round(rs, a0, y),
                        round(rs, a1, y),
                        cm(8),
                        lit,
                        ONE,
                    );
                }
            }
            put(
                b,
                f,
                Mesh::Octa,
                v3(0, cm(90), 0),
                v3(cm(40), cm(60), cm(40)),
                lit,
                ONE,
            );
            halo_bar(
                b,
                f,
                v3(0, cm(60), 0),
                v3(0, int(3), 0),
                2 * r,
                c,
                ONE / 4 + sq / 3,
            );
        }
        Pad::Cloth => {
            let y = cm(70);
            for i in 0..12 {
                bar(
                    b,
                    f,
                    Mesh::Cube,
                    round(r, TURN * i / 12, y),
                    round(r, TURN * (i + 1) / 12, y),
                    cm(16),
                    GOLD,
                    ONE / 6,
                );
            }
            for i in 0..4 {
                let p = round(mul(r, ONE * 9 / 10), TURN * i / 4 + TURN / 8, 0);
                bar(b, f, Mesh::Hex, p, p.add(v3(0, y, 0)), cm(14), WOOD, 0);
            }
            let dip = y - cm(4) - mul(sq, cm(35));
            let d = 2 * r - cm(10);
            put(
                b,
                f,
                Mesh::Cyl,
                v3(0, dip, 0),
                v3(d, cm(6), d),
                spell.robe(),
                ONE / 6,
            );
            put(
                b,
                f,
                Mesh::Cyl,
                v3(0, dip + cm(2), 0),
                v3(d * 2 / 3, cm(6), d * 2 / 3),
                c,
                ONE / 3,
            );
            put(
                b,
                f,
                Mesh::Octa,
                v3(0, dip + cm(6), 0),
                v3(r / 2, cm(6), r / 2),
                GOLD,
                ONE / 2,
            );
        }
        Pad::Jelly => {
            let h = mul(cm(70), ONE - sq / 2);
            let wide = mul(2 * r, ONE + sq / 4);
            put(
                b,
                f,
                Mesh::Geo,
                v3(0, h / 2, 0),
                v3(wide, h, wide),
                mix(c, WHITE, ONE / 3),
                ONE * 11 / 20,
            );
            put(
                b,
                f,
                Mesh::Geo,
                v3(0, h / 2, 0),
                v3(wide / 2, h * 2 / 3, wide / 2),
                c,
                ONE,
            );
            for i in 0..5 {
                let a = (tick as i32).wrapping_mul(TURN / 120) + TURN * i / 5;
                put(
                    b,
                    f,
                    Mesh::Octa,
                    round(mul(r, ONE * 7 / 10), a, h - cm(5)),
                    v3(cm(16), cm(16), cm(16)),
                    WHITE,
                    ONE,
                );
            }
            halo_ball(b, f, v3(0, h / 2, 0), wide + int(1), c, ONE / 3);
        }
    }
}

// ---------------------------------------------------------------------------
// Conjured platforms.

const ICE: Rgb = rgb(196, 238, 255);
const ICE_DEEP: Rgb = rgb(110, 190, 240);
const CRYSTAL: Rgb = rgb(206, 160, 255);
const CRYSTAL_DEEP: Rgb = rgb(128, 80, 210);
const RUNE_ROCK: Rgb = rgb(92, 84, 104);

/// A frozen spell: a hexagonal disc of ice of radius `r`, its top at y 0,
/// with icicles hanging under it and frost crystals round its rim.
pub fn frost_disc(b: &mut Batch, f: &Affine, r: i32, k: u32) {
    let across = mul(2 * r, ONE * 115 / 100);
    put(
        b,
        f,
        Mesh::Hex,
        v3(0, -cm(20), 0),
        v3(across, cm(40), across),
        ICE,
        ONE * 3 / 10,
    );
    put(
        b,
        f,
        Mesh::Hex,
        v3(0, -cm(45), 0),
        v3(across * 3 / 4, cm(20), across * 3 / 4),
        ICE_DEEP,
        ONE / 4,
    );
    let flip = Affine::rot_x(HALF);
    put_turned(
        b,
        f,
        Mesh::Spike,
        v3(0, -cm(40) - r / 2, 0),
        &flip,
        v3(r / 2, r, r / 2),
        ICE_DEEP,
        ONE / 4,
    );
    for i in 0..6 {
        let a = TURN * i / 6 + TURN / 12;
        let long = cm(60) + pick(k * 6 + i as u32, 1, 0, 60) * ONE / 100;
        put_turned(
            b,
            f,
            Mesh::Spike,
            round(r * 3 / 4, a, -cm(40) - long / 2),
            &flip,
            v3(cm(26), long, cm(26)),
            ICE,
            ONE / 4,
        );
        if i % 2 == 0 {
            put(
                b,
                f,
                Mesh::Octa,
                round(r, a, cm(10)),
                v3(cm(30), cm(50), cm(30)),
                WHITE,
                ONE / 2,
            );
        }
    }
}

/// A floating rune stone `w` across, its flat top at y 0: a boulder under
/// a dressed slab, glowing runes on its sides and a soft levitation glow
/// under it; `tick` pulses the runes.
pub fn rune_stone(b: &mut Batch, f: &Affine, w: i32, tick: u32, k: u32) {
    put(
        b,
        f,
        Mesh::Bevel,
        v3(0, -cm(15), 0),
        v3(w, cm(30), w),
        STONE,
        0,
    );
    put(
        b,
        f,
        Mesh::Rock,
        v3(0, -cm(30) - w * 3 / 10, 0),
        v3(w, w * 7 / 10, w),
        RUNE_ROCK,
        0,
    );
    let spell = Spell::nth(k);
    let lit = mix(spell.colour(), WHITE, ONE / 4);
    let p = pulse(tick, 2, k, ONE * 3 / 5);
    for i in 0..4 {
        let g = f.then(&Affine::rot_y(QUARTER * i));
        let z = w / 2 + cm(1);
        bar(
            b,
            &g,
            Mesh::Cube,
            v3(-cm(20), -cm(25), z),
            v3(cm(20), -cm(5), z),
            cm(6),
            lit,
            p,
        );
        bar(
            b,
            &g,
            Mesh::Cube,
            v3(0, -cm(30), z),
            v3(0, cm(0), z),
            cm(6),
            lit,
            p,
        );
    }
    halo_ball(b, f, v3(0, -w * 7 / 10, 0), w, spell.colour(), ONE * 2 / 5);
}

/// A conjured crystal ledge on a wall: the wall at z 0, +z out, `w` wide,
/// `d` deep, its top at y 0; a glassy slab over a cluster of crystals
/// growing down and out of the wall.
pub fn crystal_ledge(b: &mut Batch, f: &Affine, w: i32, d: i32, k: u32) {
    put(
        b,
        f,
        Mesh::Bevel,
        v3(0, -cm(15), d / 2),
        v3(w, cm(30), d),
        CRYSTAL,
        ONE * 2 / 5,
    );
    for i in 0..6u32 {
        let x = -w / 2 + w * (2 * i as i32 + 1) / 12;
        let reach = cm(80) + pick(k * 9 + i, 2, 0, 80) * ONE / 100;
        let root = v3(x, -cm(20), d / 3);
        let tip = v3(
            x + pick(k + i, 3, -30, 30) * ONE / 100,
            -cm(20) - reach,
            d / 6,
        );
        bar(
            b,
            f,
            Mesh::Spike,
            root,
            tip,
            cm(50),
            if i % 2 == 0 { CRYSTAL_DEEP } else { CRYSTAL },
            ONE / 3,
        );
    }
    for i in 0..3 {
        put(
            b,
            f,
            Mesh::Octa,
            v3(-w / 3 + w * i / 3, cm(10), d - cm(20)),
            v3(cm(24), cm(40), cm(24)),
            WHITE,
            ONE / 2,
        );
    }
}

// ---------------------------------------------------------------------------
// Clouds.

const CLOUD_TOP: Rgb = rgb(255, 224, 230);
const CLOUD_MID: Rgb = rgb(222, 184, 210);
const CLOUD_BASE: Rgb = rgb(152, 132, 186);

/// A cloud `w` across and about `h` thick, its base at y 0: a flat dark
/// base under puffs that are rose-lit on top. Still; 6 to 15 shapes.
pub fn cloud(b: &mut Batch, f: &Affine, w: i32, h: i32, k: u32) {
    put(
        b,
        f,
        Mesh::Geo,
        v3(0, h / 5, 0),
        v3(w, h * 2 / 5, w * 3 / 5),
        CLOUD_BASE,
        ONE / 3,
    );
    let n = (w / int(3)).clamp(5, 14);
    for i in 0..n {
        let kk = k.wrapping_mul(37) + i as u32;
        let x = mul(w / 2, ONE * (2 * i + 1 - n) / n) + pick(kk, 1, -40, 40) * ONE / 100;
        let z = pick(kk, 2, -100, 100) * (w / 5) / 100;
        let edge = ONE - mul(crate::fx::div(x.abs(), w / 2), ONE * 4 / 5);
        let s = mul(h, ONE * 2 / 5 + mul(edge.max(0), ONE * 7 / 10));
        let y = h / 4 + s / 3;
        let c = if s > h * 3 / 4 { CLOUD_TOP } else { CLOUD_MID };
        put(
            b,
            f,
            Mesh::Geo,
            v3(x, y, z),
            v3(s * 5 / 4, s, s),
            c,
            ONE * 2 / 5,
        );
    }
}

/// A cloud's vine anchor at `f`'s origin: a rune ring in `spell`'s colour
/// half sunk in the cloud, with a glow seen from far below.
pub fn cloud_anchor(b: &mut Batch, f: &Affine, spell: Spell, tick: u32, k: u32) {
    spell_anchor(b, f, V3::ZERO, spell, tick, k);
}

// ---------------------------------------------------------------------------
// The trampoline room.

/// Where the orb mage stands in `trampoline_room(_, _, r, h, …)`.
pub fn orb_mage_at(h: i32) -> V3 {
    v3(0, h * 7 / 20, 0)
}

/// The orb of vines round the orb mage: four great circles of `spell`'s
/// beams, radius `r`, round `f`'s origin, turning slowly. About 50
/// shapes.
pub fn vine_orb(b: &mut Batch, f: &Affine, r: i32, spell: Spell, tick: u32) {
    let (core, glow) = beam_look(spell);
    let spin = (tick as i32).wrapping_mul(TURN / 600);
    let n = 12;
    for j in 0..4 {
        let g = if j == 3 {
            f.then(&Affine::rot_y(-spin)).then(&Affine::rot_x(QUARTER))
        } else {
            f.then(&Affine::rot_y(spin + TURN / 3 * j))
                .then(&Affine::rot_z(deg(12)))
        };
        let p = |i: i32| v3(mul(r, cos(TURN * i / n)), mul(r, sin(TURN * i / n)), 0);
        for i in 0..n {
            bar(b, &g, Mesh::Hex, p(i), p(i + 1), cm(14), core, glow);
        }
    }
    halo_ball(b, f, V3::ZERO, 2 * r, spell.colour(), ONE / 5);
}

/// The trampoline room: a round chamber of apothem `r` and `h` tall, its
/// floor at y 0, inside a tower. Bounce pads on the floor and on ledges
/// climbing the wall, glowing crystals round the wall, the orb mage on a
/// pillar in the middle inside his orb of vines, and a slate roof with an
/// opening at the top (the way out), rimmed in light. `cutaway` leaves
/// out the three wall faces toward +z, to see in. About 400 shapes.
pub fn trampoline_room(
    b: &mut Batch,
    f: &Affine,
    r: i32,
    h: i32,
    cutaway: bool,
    tick: u32,
    k: u32,
) {
    let faces = if cutaway { 0b0111_1100 } else { 0xFF };
    section(b, f, r, h, k, Detail::Near, faces);
    let d = 2 * r + cm(40);
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, -cm(30), 0),
        v3(d, cm(60), d),
        STONE_DARK,
        0,
    );
    put(
        b,
        f,
        Mesh::Cyl,
        v3(0, cm(2), 0),
        v3(r, cm(6), r),
        shade(STONE, 80),
        0,
    );
    let inner = r - int(1);
    // crystals round the wall, lighting the room
    for i in 0..8 {
        let a = TURN * i / 8 + TURN / 16;
        let s = Spell::nth(i as u32 + k);
        put(
            b,
            f,
            Mesh::Octa,
            round(inner - cm(30), a, h / 2),
            v3(cm(50), cm(90), cm(50)),
            mix(s.colour(), WHITE, ONE / 3),
            ONE,
        );
    }
    // ledges with pads, spiralling up the wall
    let pads = [Pad::Rune, Pad::Jelly, Pad::Rune, Pad::Jelly, Pad::Rune];
    for (i, pad) in pads.into_iter().enumerate() {
        let i = i as i32;
        let a = TURN * i * 3 / 10 + TURN / 2;
        let y = h * (2 + i * 3) / 20;
        let g = f
            .then(&Affine::translate(round(inner - cm(160), a, y)))
            .then(&Affine::rot_y(a + HALF));
        put(
            b,
            &g,
            Mesh::Bevel,
            v3(0, -cm(20), 0),
            v3(cm(380), cm(40), cm(320)),
            STONE,
            0,
        );
        put_turned(
            b,
            &g,
            Mesh::Wedge,
            v3(0, -cm(110), -cm(40)),
            &Affine::rot_z(HALF),
            v3(cm(200), cm(140), cm(200)),
            STONE_DARK,
            0,
        );
        let sq = (crate::ruinsmodels::wave(tick, 2, i * 13_000)).max(0);
        trampoline(b, &g, cm(120), pad, Spell::nth(i as u32), sq, tick);
    }
    // floor pads
    for i in 0..3 {
        let at = round(r / 2, TURN * i / 3 + TURN / 6, 0);
        let sq = crate::ruinsmodels::wave(tick, 2, i * 21_000).max(0);
        trampoline(
            b,
            &f.then(&Affine::translate(at)),
            cm(150),
            Pad::Cloth,
            Spell::nth(i as u32 + 2),
            sq,
            tick,
        );
    }
    // the orb mage on his pillar
    let top = orb_mage_at(h);
    bar(b, f, Mesh::Hex, V3::ZERO, top, cm(160), STONE, 0);
    put(
        b,
        f,
        Mesh::Hex,
        top.add(v3(0, -cm(15), 0)),
        v3(cm(220), cm(30), cm(220)),
        STONE_DARK,
        0,
    );
    let mf = f.then(&Affine::translate(top));
    mage(b, &mf, Spell::Arcane, MagePose::Cast, tick, k + 1);
    vine_orb(
        b,
        &f.then(&Affine::translate(top.add(v3(0, cm(130), 0)))),
        int(4),
        Spell::Arcane,
        tick,
    );
    // the roof: slate sloping up to an opening rimmed in light
    let eave = r + cm(80);
    let hole = r * 3 / 10;
    let rise = (eave - hole) * 3 / 5;
    let side = side_of((eave + hole) / 2) + cm(30);
    for i in 0..8 {
        let a = TURN * i / 8;
        slab(
            b,
            f,
            Mesh::Cube,
            round(eave, a, h),
            round(hole, a, h + rise),
            side,
            cm(40),
            if i % 2 == 0 { SLATE } else { SLATE_DARK },
            0,
        );
    }
    for i in 0..8 {
        let p = round(hole + cm(10), TURN * i / 8 + TURN / 16, h + rise + cm(20));
        let q = round(
            hole + cm(10),
            TURN * (i + 1) / 8 + TURN / 16,
            h + rise + cm(20),
        );
        bar(b, f, Mesh::Cube, p, q, cm(30), WINDOW, ONE);
    }
    halo_bar(
        b,
        f,
        v3(0, h * 3 / 5, 0),
        v3(0, h + rise + int(4), 0),
        hole * 2,
        WINDOW,
        ONE / 5,
    );
}
