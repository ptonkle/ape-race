//! The tower map's outfit (`Gear::Wizard`, 2026-10-15; the person: "different
//! wizard and witch outfits, with scepters and wands and pipes potentially,
//! think gandalf"). Each kind wears its own mix (`kit`), as the pirates do:
//! a tall pointed hat (crooked at the tip) or a witch's hat with a buckle, a
//! long robe or a cloak trimmed with stars and moons, a long beard on some,
//! and one or two of a staff on the back, a sceptre or a lantern at the
//! hip, a wand in the belt, a spellbook and a pipe with a puff of smoke.
//! Nothing is held: the hands stay free for the vines.

use super::*;

pub(super) const HAT_GREY: Rgb = rgb(112, 112, 120);
pub(super) const ROBE_GREY: Rgb = rgb(138, 136, 144);
pub(super) const SILVER: Rgb = rgb(224, 228, 240);
pub(super) const MERLIN_BLUE: Rgb = rgb(34, 60, 172);
pub(super) const STAR_GOLD: Rgb = rgb(255, 212, 72);
pub(super) const WITCH_BLACK: Rgb = rgb(28, 22, 34);
pub(super) const WITCH_PURPLE: Rgb = rgb(116, 40, 156);
pub(super) const BUCKLE: Rgb = rgb(234, 198, 84);
pub(super) const EMERALD: Rgb = rgb(22, 126, 70);
pub(super) const CRIMSON_ROBE: Rgb = rgb(158, 22, 54);
pub(super) const MOON: Rgb = rgb(246, 238, 168);
pub(super) const PLUM: Rgb = rgb(72, 24, 94);
pub(super) const PUMPKIN: Rgb = rgb(248, 122, 22);
pub(super) const BEARD_WHITE: Rgb = rgb(238, 238, 232);
pub(super) const BEARD_GREY: Rgb = rgb(172, 172, 168);
pub(super) const STAFF_WOOD: Rgb = rgb(118, 78, 42);
pub(super) const CRYSTAL: Rgb = rgb(150, 230, 255);
pub(super) const SCEPTRE_GOLD: Rgb = rgb(236, 184, 60);
pub(super) const ORB: Rgb = rgb(204, 92, 255);
pub(super) const WAND: Rgb = rgb(58, 34, 22);
pub(super) const WAND_TIP: Rgb = rgb(255, 250, 216);
pub(super) const PIPE_WOOD: Rgb = rgb(98, 58, 28);
pub(super) const SMOKE: Rgb = rgb(206, 206, 212);
pub(super) const LANTERN_IRON: Rgb = rgb(50, 48, 56);
pub(super) const LANTERN_FLAME: Rgb = rgb(255, 204, 86);
pub(super) const BOOK: Rgb = rgb(112, 28, 30);
pub(super) const PAGES: Rgb = rgb(240, 228, 194);
pub(super) const CORD: Rgb = rgb(198, 168, 108);
pub(super) const WIZ_STRAP: Rgb = rgb(72, 46, 28);

/// A pointed hat: its colour, its band, the brim's width and the crown's
/// height (percent of the head), how far the tip flops back and to the
/// side (percent of the head), and on a witch's hat a buckle on the band.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Hat {
    pub colour: Rgb,
    pub band: Rgb,
    pub brim: i32,
    pub tall: i32,
    pub crook: (i32, i32),
    pub witch: bool,
    /// little stars on the crown
    pub stars: Option<Rgb>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Top {
    /// a long robe to the knees, wide sleeves, a trim down the front and
    /// round the hem, stars or moons on it
    Robe { colour: Rgb, trim: Rgb, moons: bool },
    /// a cloak from the shoulders down the back, a clasp at the throat
    Cloak { colour: Rgb, trim: Rgb },
}

/// One kind's wizard pieces.
#[derive(Debug, Clone, Copy)]
pub(super) struct Kit {
    pub hat: Hat,
    pub top: Top,
    /// a beard's colour and length (percent of the head below the chin)
    pub beard: Option<(Rgb, i32)>,
    /// a staff across the back, its crystal's colour
    pub staff: Option<Rgb>,
    pub sceptre: bool,
    pub wand: bool,
    pub pipe: bool,
    pub book: bool,
    pub lantern: bool,
}

/// Each kind's mix: the gorilla the grey wanderer (grey crooked hat, grey
/// robe with silver stars, long white beard, staff, pipe); the orangutan a
/// witch (black hat with a purple band and gold buckle, purple robe with
/// moons, wand, lantern); the chimpanzee the court magician (blue hat and
/// robe with gold stars and moons, sceptre, spellbook); the gibbon a hedge
/// wizard (green hat, green cloak, grey beard, staff, pipe); the proboscis
/// monkey the alchemist (crimson hat and robe with moons, white beard
/// under the nose, wand, lantern); the lemur a little witch (plum hat with
/// an orange band, black cloak with orange stars, wand, spellbook).
pub(super) fn kit(kind: Kind) -> Kit {
    let hat = |colour, band, crook| Hat {
        colour,
        band,
        brim: 150,
        tall: 100,
        crook,
        witch: false,
        stars: None,
    };
    let bare = Kit {
        hat: hat(HAT_GREY, HAT_GREY, (0, 0)),
        top: Top::Cloak {
            colour: ROBE_GREY,
            trim: SILVER,
        },
        beard: None,
        staff: None,
        sceptre: false,
        wand: false,
        pipe: false,
        book: false,
        lantern: false,
    };
    match kind {
        Kind::Gorilla => Kit {
            hat: Hat {
                tall: 108,
                brim: 164,
                ..hat(HAT_GREY, shade(HAT_GREY, 70), (34, 18))
            },
            top: Top::Robe {
                colour: ROBE_GREY,
                trim: SILVER,
                moons: false,
            },
            beard: Some((BEARD_WHITE, 96)),
            staff: Some(CRYSTAL),
            pipe: true,
            ..bare
        },
        Kind::Orangutan => Kit {
            hat: Hat {
                witch: true,
                brim: 176,
                ..hat(WITCH_BLACK, WITCH_PURPLE, (30, -20))
            },
            top: Top::Robe {
                colour: WITCH_PURPLE,
                trim: WITCH_BLACK,
                moons: true,
            },
            wand: true,
            lantern: true,
            ..bare
        },
        Kind::Chimpanzee => Kit {
            hat: Hat {
                stars: Some(STAR_GOLD),
                ..hat(MERLIN_BLUE, STAR_GOLD, (12, 0))
            },
            top: Top::Robe {
                colour: MERLIN_BLUE,
                trim: STAR_GOLD,
                moons: true,
            },
            sceptre: true,
            book: true,
            ..bare
        },
        Kind::Gibbon => Kit {
            hat: hat(EMERALD, shade(EMERALD, 60), (40, -24)),
            top: Top::Cloak {
                colour: EMERALD,
                trim: CORD,
            },
            beard: Some((BEARD_GREY, 80)),
            staff: Some(LANTERN_FLAME),
            pipe: true,
            ..bare
        },
        Kind::Proboscis => Kit {
            hat: Hat {
                stars: Some(MOON),
                ..hat(CRIMSON_ROBE, SCEPTRE_GOLD, (26, 26))
            },
            top: Top::Robe {
                colour: CRIMSON_ROBE,
                trim: SCEPTRE_GOLD,
                moons: true,
            },
            beard: Some((BEARD_WHITE, 70)),
            wand: true,
            lantern: true,
            ..bare
        },
        Kind::Lemur => Kit {
            hat: Hat {
                witch: true,
                brim: 170,
                tall: 80,
                ..hat(PLUM, PUMPKIN, (80, 14))
            },
            top: Top::Cloak {
                colour: WITCH_BLACK,
                trim: PUMPKIN,
            },
            wand: true,
            book: true,
            ..bare
        },
    }
}

/// Everything a wizard wears, over the ape `kind` posed as `s`.
pub(super) fn wizard(rig: &mut Rig, kind: Kind, l: &Look, m: &Dims, s: &Skel, tick: u32) {
    let k = kit(kind);
    let d = cm(l.head);
    let h = &s.head;
    if let Some((c, len)) = k.beard {
        beard(rig, h, d, c, len);
    }
    hat(rig, h, d, &k.hat);
    match k.top {
        Top::Robe {
            colour,
            trim,
            moons,
        } => robe(rig, m, s, colour, trim, moons),
        Top::Cloak { colour, trim } => cloak(rig, m, s, colour, trim),
    }
    if k.wand || k.sceptre || k.book || k.lantern {
        let r = matches!(k.top, Top::Robe { .. });
        rig.put(
            Mesh::Barrel,
            s.body.shape(
                v3(0, pct(m.t, 28), 0),
                v3(
                    pct(m.w, if r { 124 } else { 108 }),
                    pct(m.t, 6),
                    pct(m.d, if r { 136 } else { 110 }),
                ),
            ),
            CORD,
        );
    }
    if let Some(c) = k.staff {
        staff(rig, m, s, c);
    }
    if k.sceptre {
        sceptre(rig, m, s);
    }
    if k.wand {
        wand(rig, m, s);
    }
    if k.book {
        book(rig, m, s);
    }
    if k.lantern {
        lantern(rig, m, s);
    }
    if k.pipe {
        pipe(rig, l, h, d, tick);
    }
}

/// The hat on the head `h` (`d` across): a wide flat brim, a band, and a
/// crown of three narrowing pieces rising and flopping back (and aside) to
/// a point; a witch's hat has a buckle on the band.
fn hat(rig: &mut Rig, h: &Frame, d: i32, hat: &Hat) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    let brim = h.at(0, pct(d, 32), -pct(d, 4));
    rig.put(
        Mesh::Hex,
        rod(brim, brim.add(h.y.scale(pct(d, 5))), pct(d, hat.brim)),
        hat.colour,
    );
    let (back, aside) = hat.crook;
    let c0 = h.at(0, pct(d, 34), -pct(d, 4));
    let c1 = h.at(
        pct(d, aside / 8),
        pct(d, 34 + hat.tall * 45 / 100),
        -pct(d, 7 + back / 4),
    );
    let c2 = h.at(
        pct(d, aside / 3),
        pct(d, 34 + hat.tall * 78 / 100),
        -pct(d, 8 + back * 3 / 5),
    );
    let tip = h.at(pct(d, aside), pct(d, 34 + hat.tall), -pct(d, 8 + back));
    rig.put(Mesh::Taper, rod(c0, c1, pct(d, 84)), hat.colour);
    rig.put(Mesh::Taper, rod(c1, c2, pct(d, 52)), hat.colour);
    rig.put(Mesh::Spike, rod(c2, tip, pct(d, 32)), hat.colour);
    rig.put(Mesh::Hex, h.shape(p(0, 41, -4), p(88, 10, 88)), hat.band);
    if hat.witch {
        rig.put(Mesh::Bevel, h.shape(p(0, 41, 40), p(20, 13, 6)), BUCKLE);
        rig.put(
            Mesh::Bevel,
            h.shape(p(0, 41, 42), p(9, 6, 4)),
            shade(hat.colour, 120),
        );
    }
    if let Some(c) = hat.stars {
        // on the crown's front: two low, one higher
        for (x, y, z) in [(-14, 52, 33), (16, 62, 29), (2, 82, 20)] {
            rig.glow(Mesh::Octa, h.shape(p(x, y, z), p(12, 12, 5)), c, ONE / 3);
        }
    }
}

/// A beard from the chin down the chest, `len` percent of the head long,
/// and a moustache over it.
fn beard(rig: &mut Rig, h: &Frame, d: i32, c: Rgb, len: i32) {
    let chin = h.at(0, -pct(d, 36), pct(d, 34));
    let tip = h.at(0, -pct(d, 36 + len), pct(d, 30));
    rig.put(
        Mesh::Geo,
        h.shape(
            v3(0, -pct(d, 36), pct(d, 34)),
            v3(pct(d, 60), pct(d, 30), pct(d, 34)),
        ),
        c,
    );
    rig.put(Mesh::Spike, rod(chin, tip, pct(d, 72)), c);
    for s in SIDES {
        rig.blade(
            h.at(s * pct(d, 4), -pct(d, 26), pct(d, 50)),
            h.at(s * pct(d, 30), -pct(d, 40), pct(d, 42)),
            h.y,
            pct(d, 14),
            pct(d, 8),
            shade(c, 92),
        );
    }
}

/// A long robe to the knees with wide sleeves halfway down the forearms, a trim down
/// the front and round the hem and cuffs, and glinting stars (or moons).
fn robe(rig: &mut Rig, m: &Dims, s: &Skel, c: Rgb, trim: Rgb, moons: bool) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    garment(rig, m, s, -34, 104, 118, 130, c);
    // the hem's trim: a band of rods round the robe's edge. (A filled flat
    // barrel there hung under the belly of an ape on all fours like a
    // saucer: the person, 2026-10-16, "weird discs on the bottom of the
    // apes", T94)
    let (rw, rd) = (pct(w, 120) / 2, pct(d, 132) / 2);
    let hem = |k: i32| {
        let a = deg(36 * k);
        body.at(mul(fx::cos(a), rw), -pct(t, 34), mul(fx::sin(a), rd))
    };
    for k in 0..10 {
        rig.put(Mesh::Hex, rod(hem(k), hem(k + 1), pct(w, 7)), trim);
    }
    rig.put(Mesh::Cube, body.shape(at(0, 34, 65), at(10, 128, 4)), trim);
    let mark = if moons { MOON } else { trim };
    for (x, y) in [(-30, 8), (32, 54), (-26, 76), (28, -14)] {
        let a = body.at(pct(w, x), pct(t, y), pct(d, 64));
        rig.glow(
            if moons { Mesh::Ico } else { Mesh::Octa },
            body.shape(V3::ZERO, at(14, 10, 4)).at(a),
            mark,
            ONE / 4,
        );
    }
    sleeves(rig, m, s, 150, 176, c, trim);
}

/// A cloak from the shoulders down the back to the knees, ragged at the
/// hem, its edge trimmed, a clasp at the throat and stars on the back.
fn cloak(rig: &mut Rig, m: &Dims, s: &Skel, c: Rgb, trim: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    rig.put(
        Mesh::Barrel,
        body.shape(
            v3(0, pct(t, 96), -pct(d, 6)),
            v3(pct(w, 96), pct(t, 16), pct(d, 100)),
        ),
        c,
    );
    for x in [-50, 0, 50] {
        let top = body.at(pct(w, x * 3 / 4), pct(t, 94), -pct(d, 52));
        let low = body.at(pct(w, x), -pct(t, 36), -pct(d, 66));
        rig.blade(top, low, body.x, pct(w, 74), pct(d, 8), c);
    }
    for x in [-56, 56] {
        let top = body.at(pct(w, x * 3 / 4), pct(t, 94), -pct(d, 54));
        let low = body.at(pct(w, x * 21 / 20), -pct(t, 38), -pct(d, 68));
        rig.put(Mesh::Hex, rod(top, low, pct(w, 8)), trim);
    }
    rig.ball(body.at(0, pct(t, 98), pct(d, 50)), pct(w, 16), trim);
    for (x, y) in [(-22, 50), (18, 20), (4, 72)] {
        let a = body.at(pct(w, x), pct(t, y), -pct(d, 72));
        rig.glow(
            Mesh::Octa,
            body.shape(V3::ZERO, v3(pct(w, 16), pct(w, 16), pct(d, 6)))
                .at(a),
            trim,
            ONE / 4,
        );
    }
}

/// A gnarled staff slung across the back on a strap, its head curling round
/// a glowing crystal above one shoulder (clear of the head and the hands).
fn staff(rig: &mut Rig, m: &Dims, s: &Skel, crystal: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let low = body.at(-pct(w, 50), -pct(t, 34), -pct(d, 70));
    let top = body.at(pct(w, 56), pct(t, 124), -pct(d, 70));
    rig.put(Mesh::Hex, rod(low, top, pct(w, 10)), STAFF_WOOD);
    let along = dir_or(top.sub(low), body.y);
    let gem = top.add(along.scale(pct(t, 14)));
    for x in SIDES {
        let out = top
            .add(body.x.scale(x * pct(w, 8)))
            .add(along.scale(pct(t, 10)));
        rig.put(Mesh::Hex, rod(top, out, pct(w, 6)), STAFF_WOOD);
        rig.put(
            Mesh::Spike,
            rod(out, gem.add(along.scale(pct(t, 6))), pct(w, 5)),
            STAFF_WOOD,
        );
    }
    rig.glow(
        Mesh::Octa,
        Affine {
            c0: body.x.scale(pct(w, 14)),
            c1: along.scale(pct(t, 16)),
            c2: body.z.scale(pct(w, 14)),
            t: gem,
        },
        crystal,
        ONE,
    );
    // the strap from the far shoulder across the chest
    let a = body.at(-pct(w, 34), pct(t, 98), pct(d, 40));
    let b = body.at(pct(w, 50), pct(t, 20), pct(d, 56));
    rig.put(Mesh::Hex, rod(a, b, pct(w, 10)), WIZ_STRAP);
}

/// A short golden sceptre hanging from the belt at the right hip, a
/// glowing violet orb on its head by the waist.
fn sceptre(rig: &mut Rig, m: &Dims, s: &Skel) {
    let body = &s.body;
    let (w, t) = (m.w, m.t);
    let head = body.at(pct(w, 62), pct(t, 32), pct(m.d, 22));
    let end = head.add(body.dir(8, -80, -30).scale(pct(t, 66)));
    rig.put(Mesh::Hex, rod(head, end, pct(w, 8)), SCEPTRE_GOLD);
    rig.ball(end, pct(w, 10), SCEPTRE_GOLD);
    rig.put(
        Mesh::Hex,
        rod(head, head.add(body.y.scale(pct(t, 4))), pct(w, 18)),
        SCEPTRE_GOLD,
    );
    rig.glow(
        Mesh::Geo,
        Affine::translate(head.add(body.y.scale(pct(t, 12)))).then(&Affine::scale(v3(
            pct(w, 20),
            pct(w, 20),
            pct(w, 20),
        ))),
        ORB,
        ONE * 3 / 4,
    );
}

/// A wand tucked upright into the belt at the front left, a glint at its
/// tip.
fn wand(rig: &mut Rig, m: &Dims, s: &Skel) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let low = body.at(-pct(w, 30), pct(t, 14), pct(d, 66));
    let tip = body.at(-pct(w, 38), pct(t, 54), pct(d, 72));
    rig.put(Mesh::Hex, rod(low, tip, pct(w, 6)), WAND);
    rig.glow(
        Mesh::Octa,
        Affine::translate(tip).then(&Affine::scale(v3(pct(w, 9), pct(w, 9), pct(w, 9)))),
        WAND_TIP,
        ONE,
    );
}

/// A spellbook at the left hip: a red cover, the pages' edge showing and a
/// gold clasp.
fn book(rig: &mut Rig, m: &Dims, s: &Skel) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    rig.put(
        Mesh::Bevel,
        body.shape(at(-68, 16, 4), at(12, 28, 46)),
        BOOK,
    );
    rig.put(
        Mesh::Cube,
        body.shape(at(-68, 16, 10), at(8, 25, 42)),
        PAGES,
    );
    rig.ball(
        body.at(-pct(w, 76), pct(t, 16), pct(d, 26)),
        pct(w, 7),
        BUCKLE,
    );
}

/// A lantern hanging from the belt at the right hip on a short chain: an
/// iron cap and base round a glowing flame.
fn lantern(rig: &mut Rig, m: &Dims, s: &Skel) {
    let (w, t) = (m.w, m.t);
    let hang = s.body.at(pct(w, 62), pct(t, 26), pct(m.d, 6));
    let top = hang.sub(V3::UP.scale(pct(t, 10)));
    let mid = top.sub(V3::UP.scale(pct(t, 10)));
    let low = mid.sub(V3::UP.scale(pct(t, 10)));
    rig.put(Mesh::Hex, rod(hang, top, pct(w, 3)), LANTERN_IRON);
    rig.put(
        Mesh::Pyramid,
        rod(mid.lerp(top, ONE / 2), top, pct(w, 24)),
        LANTERN_IRON,
    );
    rig.glow(
        Mesh::Hex,
        rod(low, mid.lerp(top, ONE / 2), pct(w, 18)),
        LANTERN_FLAME,
        ONE,
    );
    rig.put(
        Mesh::Hex,
        rod(low.sub(V3::UP.scale(pct(t, 3))), low, pct(w, 24)),
        LANTERN_IRON,
    );
}

/// A long pipe from the right corner of the mouth, its bowl held up ahead
/// of the chin with a glowing ember, and three puffs of smoke rising and
/// growing from it (moving with `tick`).
fn pipe(rig: &mut Rig, l: &Look, h: &Frame, d: i32, tick: u32) {
    let z = if l.muzzle > 0 {
        40 + l.muzzle * 44 / 100
    } else {
        44
    };
    let mouth = h.at(pct(d, 14), -pct(d, 34), pct(d, z));
    let bowl = mouth.add(h.dir(30, -26, 100).scale(pct(d, 40)));
    rig.put(Mesh::Hex, rod(mouth, bowl, pct(d, 6)), PIPE_WOOD);
    let up = bowl.add(V3::UP.scale(pct(d, 16)));
    rig.put(Mesh::Taper, rod(bowl, up, pct(d, 16)), PIPE_WOOD);
    rig.glow(
        Mesh::Hex,
        rod(up, up.add(V3::UP.scale(pct(d, 2))), pct(d, 10)),
        PUMPKIN,
        ONE,
    );
    for k in 0..3u32 {
        // 0 at the bowl to ONE at the top, over two seconds
        let age = ((tick + k * 40) % 120) as i32 * ONE / 120;
        let at = up
            .add(V3::UP.scale(pct(d, 10) + mul(age, pct(d, 60))))
            .add(h.x.scale(mul(fx::sin(age * 2), pct(d, 8))));
        let size = pct(d, 10) + mul(age, pct(d, 18));
        rig.put(
            Mesh::Blob,
            Affine::translate(at).then(&Affine::scale(v3(size, size, size))),
            SMOKE,
        );
    }
}
