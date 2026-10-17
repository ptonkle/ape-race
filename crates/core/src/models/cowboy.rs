//! Rattlesnake Ravine's outfit (`Gear::Cowboy`, 2026-10-17; the person:
//! "heavily inspired by cowboys and the wild west. Each monkey should have
//! a different color pallet ... a Large leather hat, Bandanas, leather
//! chaps, suspenders, holsters or bandoliers containing a banana, boots and
//! spurs"). Every kind wears a big hat, a bandana at the neck, a gun belt,
//! leather chaps and boots with spurs, and carries bananas in a holster or
//! a bandolier; each mixes in its own top (bare with suspenders, a vest, a
//! check shirt) in its own colours (`kit`), as the wizards do. Nothing is
//! held: the hands stay free for the vines.

use super::*;

/// the gorilla's: a sand leather that stands out on its black fur
pub(super) const HAT_SAND: Rgb = rgb(208, 178, 126);
pub(super) const HAT_CREAM: Rgb = rgb(230, 212, 168);
pub(super) const HAT_SLATE: Rgb = rgb(86, 92, 104);
pub(super) const HAT_BROWN: Rgb = rgb(122, 74, 38);
pub(super) const HAT_WHITE: Rgb = rgb(244, 242, 234);
pub(super) const HAT_TAN: Rgb = rgb(198, 150, 92);
pub(super) const BAND_DARK: Rgb = rgb(60, 36, 22);
pub(super) const BAND_RED: Rgb = rgb(150, 30, 30);
pub(super) const KERCHIEF_RED: Rgb = rgb(196, 34, 36);
pub(super) const KERCHIEF_TEAL: Rgb = rgb(34, 166, 168);
pub(super) const KERCHIEF_YELLOW: Rgb = rgb(240, 188, 34);
pub(super) const KERCHIEF_BLUE: Rgb = rgb(36, 82, 172);
pub(super) const KERCHIEF_PURPLE: Rgb = rgb(124, 48, 146);
pub(super) const KERCHIEF_BLACK: Rgb = rgb(30, 28, 32);
pub(super) const PAISLEY: Rgb = rgb(250, 248, 240);
pub(super) const VEST_TAN: Rgb = rgb(184, 134, 82);
pub(super) const VEST_GREEN: Rgb = rgb(40, 92, 66);
pub(super) const SHIRT_RED: Rgb = rgb(172, 40, 44);
pub(super) const CHECK_DARK: Rgb = rgb(40, 22, 24);
pub(super) const STAR_BADGE: Rgb = rgb(246, 200, 64);
pub(super) const BRACES_NAVY: Rgb = rgb(34, 52, 112);
pub(super) const BRACES_RED: Rgb = rgb(186, 44, 40);
pub(super) const BRACES_ORANGE: Rgb = rgb(226, 120, 34);
pub(super) const CHAPS_BLACK: Rgb = rgb(52, 40, 36);
pub(super) const CHAPS_CHOC: Rgb = rgb(78, 46, 26);
pub(super) const CHAPS_TAN: Rgb = rgb(164, 112, 64);
pub(super) const CHAPS_SADDLE: Rgb = rgb(140, 84, 40);
pub(super) const CHAPS_SUEDE: Rgb = rgb(212, 172, 120);
pub(super) const CHAPS_ASH: Rgb = rgb(100, 96, 92);
pub(super) const BOOT_OXBLOOD: Rgb = rgb(98, 30, 28);
pub(super) const BOOT_BROWN: Rgb = rgb(88, 54, 30);
pub(super) const BOOT_BLACK: Rgb = rgb(26, 22, 22);
pub(super) const BOOT_DARK: Rgb = rgb(62, 40, 26);
pub(super) const BOOT_RED: Rgb = rgb(176, 40, 36);
pub(super) const BOOT_TEAL: Rgb = rgb(30, 110, 116);
pub(super) const SPUR: Rgb = rgb(208, 212, 222);
pub(super) const GUNBELT: Rgb = rgb(70, 44, 26);
pub(super) const BUCKLE_GOLD: Rgb = rgb(232, 190, 70);
pub(super) const HOLSTER: Rgb = rgb(110, 66, 34);
pub(super) const BANANA: Rgb = rgb(250, 216, 56);
pub(super) const BANANA_TIP: Rgb = rgb(72, 52, 30);

/// A big leather hat: its colour, its band, the brim's width and how far
/// its sides curl up (percent of the head).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Hat {
    pub colour: Rgb,
    pub band: Rgb,
    pub brim: i32,
    pub curl: i32,
    /// how far it is pushed back on the head (percent of the head)
    pub back: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Top {
    /// bare fur (with suspenders)
    Bare,
    /// a leather vest, a gold star on some
    Vest { colour: Rgb, star: bool },
    /// a check shirt with sleeves to the forearm
    Shirt { colour: Rgb, check: Rgb },
}

/// One kind's cowboy pieces.
#[derive(Debug, Clone, Copy)]
pub(super) struct Kit {
    pub hat: Hat,
    pub bandana: Rgb,
    /// white dots on the bandana
    pub dots: bool,
    pub top: Top,
    pub suspenders: Option<Rgb>,
    pub chaps: Rgb,
    /// fringe down the chaps' outer seams
    pub fringe: bool,
    pub boots: Rgb,
    /// a holster at this side's hip, a banana in it
    pub holster: Option<i32>,
    /// a bandolier of bananas across the chest
    pub bandolier: bool,
}

/// Each kind's mix: the gorilla the sheriff (sand hat, red bandana, tan
/// vest with a gold star, black chaps, oxblood boots, a holster); the
/// orangutan the trail boss (cream hat, teal bandana, navy suspenders,
/// fringed chocolate chaps, a bandolier); the chimpanzee the outlaw (slate
/// hat, yellow dotted bandana, red check shirt, tan chaps, black boots, a
/// holster); the gibbon the ranch hand (brown hat, blue bandana, red
/// suspenders, saddle chaps, a holster); the proboscis monkey the dandy
/// (white hat, purple bandana, green vest, ash chaps, red boots, a
/// bandolier); the lemur the kid (an oversized tan hat, black dotted
/// bandana, orange suspenders, fringed suede chaps, teal boots, a holster).
pub(super) fn kit(kind: Kind) -> Kit {
    let hat = |colour, band| Hat {
        colour,
        band,
        brim: 170,
        curl: 14,
        back: 0,
    };
    let base = Kit {
        hat: hat(HAT_BROWN, BAND_DARK),
        bandana: KERCHIEF_RED,
        dots: false,
        top: Top::Bare,
        suspenders: None,
        chaps: CHAPS_TAN,
        fringe: false,
        boots: BOOT_BROWN,
        holster: None,
        bandolier: false,
    };
    match kind {
        Kind::Gorilla => Kit {
            hat: Hat {
                brim: 176,
                curl: 10,
                ..hat(HAT_SAND, BAND_DARK)
            },
            bandana: KERCHIEF_RED,
            top: Top::Vest {
                colour: VEST_TAN,
                star: true,
            },
            chaps: CHAPS_BLACK,
            boots: BOOT_OXBLOOD,
            holster: Some(1),
            ..base
        },
        Kind::Orangutan => Kit {
            hat: Hat {
                brim: 186,
                curl: 20,
                ..hat(HAT_CREAM, BAND_DARK)
            },
            bandana: KERCHIEF_TEAL,
            suspenders: Some(BRACES_NAVY),
            chaps: CHAPS_CHOC,
            fringe: true,
            boots: BOOT_BROWN,
            bandolier: true,
            ..base
        },
        Kind::Chimpanzee => Kit {
            hat: hat(HAT_SLATE, BAND_RED),
            bandana: KERCHIEF_YELLOW,
            dots: true,
            top: Top::Shirt {
                colour: SHIRT_RED,
                check: CHECK_DARK,
            },
            chaps: CHAPS_TAN,
            boots: BOOT_BLACK,
            holster: Some(-1),
            ..base
        },
        Kind::Gibbon => Kit {
            hat: Hat {
                curl: 24,
                ..hat(HAT_BROWN, BAND_DARK)
            },
            bandana: KERCHIEF_BLUE,
            suspenders: Some(BRACES_RED),
            chaps: CHAPS_SADDLE,
            boots: BOOT_DARK,
            holster: Some(1),
            ..base
        },
        Kind::Proboscis => Kit {
            hat: Hat {
                brim: 180,
                ..hat(HAT_WHITE, KERCHIEF_BLACK)
            },
            bandana: KERCHIEF_PURPLE,
            top: Top::Vest {
                colour: VEST_GREEN,
                star: false,
            },
            chaps: CHAPS_ASH,
            boots: BOOT_RED,
            bandolier: true,
            ..base
        },
        Kind::Lemur => Kit {
            hat: Hat {
                brim: 188,
                curl: 18,
                // (pushed back, clear of the hands on a vine: the lemur's
                // head is right under them)
                back: 34,
                ..hat(HAT_TAN, BAND_RED)
            },
            bandana: KERCHIEF_BLACK,
            dots: true,
            suspenders: Some(BRACES_ORANGE),
            chaps: CHAPS_SUEDE,
            fringe: true,
            boots: BOOT_TEAL,
            holster: Some(-1),
            ..base
        },
    }
}

impl Top {
    /// whether the upper arms are in sleeves (the orangutan's arm hair hides)
    pub(super) fn sleeved(self) -> bool {
        matches!(self, Top::Shirt { .. })
    }
}

/// How far round the torso the clothes sit (percent of its width and
/// depth): the gorilla's chest slabs and the proboscis monkey's belly need
/// more room.
fn fit(kind: Kind) -> (i32, i32) {
    match kind {
        Kind::Gorilla => (120, 142),
        Kind::Proboscis => (128, 160),
        _ => (112, 114),
    }
}

/// Everything a cowboy wears but the boots (drawn in `limbs`, in place of
/// the feet), over the ape `kind` posed as `s`.
pub(super) fn cowboy(rig: &mut Rig, kind: Kind, l: &Look, m: &Dims, s: &Skel) {
    let k = kit(kind);
    let d = cm(l.head);
    let h = &s.head;
    hat(rig, h, d, &k.hat);
    bandana(rig, l, h, d, k.bandana, k.dots);
    let (wide, deep) = fit(kind);
    match k.top {
        Top::Bare => {}
        Top::Vest { colour, star } => vest(rig, m, s, wide, deep, colour, star),
        Top::Shirt { colour, check } => shirt(rig, m, s, wide, deep, colour, check),
    }
    if let Some(c) = k.suspenders {
        suspenders(rig, m, s, wide, deep, c);
    }
    belt(rig, m, s, wide, deep);
    chaps(rig, m, s, k.chaps, k.fringe);
    if let Some(side) = k.holster {
        holster(rig, m, s, wide, side);
    }
    if k.bandolier {
        bandolier(rig, m, s, deep);
    }
}

/// The hat on the head `h` (`d` across), level over the ears: a wide,
/// flat oval brim whose two side edges curl up (inside the brim's edge,
/// nothing hanging below it), a band, and a tall crown with a dented top
/// and a pinched front.
fn hat(rig: &mut Rig, h: &Frame, d: i32, hat: &Hat) {
    let h = &Frame {
        o: h.at(0, -pct(d, hat.back / 4), -pct(d, hat.back)),
        ..*h
    };
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    let (brim, low) = (hat.brim, 40);
    rig.put(
        Mesh::Hex,
        h.shape(p(0, low, -4), p(brim * 66 / 100, 4, brim * 74 / 100)),
        hat.colour,
    );
    for s in SIDES {
        // the brim's side edges, curling up from the flat middle
        let inner = h.at(s * pct(d, brim * 22 / 100), pct(d, low + 1), -pct(d, 4));
        let outer = h.at(
            s * pct(d, brim * 34 / 100),
            pct(d, low + 4 + hat.curl),
            -pct(d, 4),
        );
        rig.put(
            Mesh::Bevel,
            Affine {
                c0: outer.sub(inner),
                c1: h.y.scale(pct(d, 4)),
                c2: h.z.scale(pct(d, brim * 52 / 100)),
                t: inner.lerp(outer, ONE / 2),
            },
            shade(hat.colour, 94),
        );
    }
    rig.put(Mesh::Hex, h.shape(p(0, 47, -5), p(68, 10, 76)), hat.band);
    rig.put(
        Mesh::Bevel,
        h.shape(p(0, 60, -6), p(62, 28, 72)),
        hat.colour,
    );
    // the crown's top: two rounded lobes either side of the dent, and the
    // pinch at the front
    for s in SIDES {
        rig.put(
            Mesh::Geo,
            h.shape(p(s * 12, 70, -10), p(32, 12, 64)),
            hat.colour,
        );
        rig.put(
            Mesh::Bevel,
            h.shape(p(s * 12, 66, 29), p(10, 18, 8)),
            shade(hat.colour, 84),
        );
    }
}

/// A kerchief at the neck: a triangle hanging under the chin, knotted at
/// the back, with white dots on some.
fn bandana(rig: &mut Rig, l: &Look, h: &Frame, d: i32, c: Rgb, dots: bool) {
    // in front of the muzzle's underside
    let z = 30 + l.muzzle / 6;
    let top = h.at(0, -pct(d, 40), pct(d, z));
    let tip = h.at(0, -pct(d, 82), pct(d, z + 8));
    rig.blade(top, tip, h.x, pct(d, 62), pct(d, 8), c);
    rig.put(
        Mesh::Hex,
        h.shape(
            v3(0, -pct(d, 44), -pct(d, 4)),
            v3(pct(d, 74), pct(d, 10), pct(d, 80)),
        ),
        c,
    );
    let knot = h.at(0, -pct(d, 42), -pct(d, 46));
    rig.ball(knot, pct(d, 14), shade(c, 86));
    if dots {
        for (x, y) in [(-12, 52), (12, 52), (0, 66)] {
            rig.ball(
                h.at(pct(d, x), -pct(d, y), pct(d, z + 6 + (y - 52) / 4)),
                pct(d, 9),
                PAISLEY,
            );
        }
    }
}

/// A leather vest from the hips over the shoulders, open down the front,
/// a gold star on the chest of some.
#[allow(clippy::too_many_arguments)]
fn vest(rig: &mut Rig, m: &Dims, s: &Skel, wide: i32, deep: i32, c: Rgb, star: bool) {
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    garment(rig, m, s, 14, 104, wide, deep, c);
    rig.put(
        Mesh::Cube,
        s.body.shape(at(0, 56, deep / 2 - 2), at(10, 52, 3)),
        shade(c, 60),
    );
    if star {
        rig.glow(
            Mesh::Octa,
            s.body.shape(at(-wide / 5, 74, deep / 2 + 2), at(16, 14, 4)),
            STAR_BADGE,
            ONE / 4,
        );
    }
}

/// A check shirt with sleeves to the forearm: dark bands across the front
/// and back, and down the sleeves.
#[allow(clippy::too_many_arguments)]
fn shirt(rig: &mut Rig, m: &Dims, s: &Skel, wide: i32, deep: i32, c: Rgb, check: Rgb) {
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    garment(rig, m, s, 12, 104, wide - 4, deep - 4, c);
    for y in [40, 70] {
        for z in [deep / 2, -deep / 2] {
            rig.put(
                Mesh::Cube,
                s.body.shape(at(0, y, z), at(wide - 20, 6, 3)),
                check,
            );
        }
    }
    for x in [-wide / 5, wide / 5] {
        rig.put(
            Mesh::Cube,
            s.body.shape(at(x, 56, deep / 2), at(6, 80, 3)),
            check,
        );
    }
    sleeves(rig, m, s, 150, 150, c, check);
}

/// Suspenders from the belt up the front, over each shoulder and down the
/// back, with a brass button at each front end.
fn suspenders(rig: &mut Rig, m: &Dims, s: &Skel, wide: i32, deep: i32, c: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let _ = wide;
    for x in [-24, 24] {
        let x = pct(w, x);
        let path = [
            body.at(x, pct(t, 22), pct(d, deep / 2 + 2)),
            body.at(x, pct(t, 80), pct(d, deep / 2 + 4)),
            body.at(x * 5 / 4, pct(t, 108), 0),
            body.at(x, pct(t, 80), -pct(d, deep / 2 + 4)),
            body.at(x, pct(t, 22), -pct(d, deep / 2 + 2)),
        ];
        for k in 0..4 {
            rig.put(Mesh::Hex, rod(path[k], path[k + 1], pct(w, 9)), c);
        }
        rig.ball(path[0], pct(w, 9), BUCKLE_GOLD);
    }
}

/// The gun belt round the hips and its gold buckle.
fn belt(rig: &mut Rig, m: &Dims, s: &Skel, wide: i32, deep: i32) {
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    rig.put(
        Mesh::Barrel,
        s.body.shape(at(0, 20, 0), at(wide + 6, 9, deep + 6)),
        GUNBELT,
    );
    rig.put(
        Mesh::Bevel,
        s.body.shape(at(0, 20, deep / 2 + 3), at(22, 12, 4)),
        BUCKLE_GOLD,
    );
}

/// Leather chaps over each leg, hip to the boot's top, fringe down the
/// outer seam on some.
fn chaps(rig: &mut Rig, m: &Dims, s: &Skel, c: Rgb, fringe: bool) {
    let lw = m.leg_w;
    for (i, side) in SIDES.into_iter().enumerate() {
        let hip = hip_of(&s.body, m, side);
        let (knee, ankle) = bend(hip, s.feet[i], m.leg, s.leg_hint[i]);
        let shin = knee.lerp(ankle, ONE * 55 / 100);
        rig.put(Mesh::Taper, rod(hip, knee, pct(lw, 162)), c);
        rig.ball(knee, pct(lw, 140), c);
        rig.put(Mesh::Taper, rod(knee, shin, pct(lw, 140)), c);
        if fringe {
            let out = s.body.x.scale(side);
            for (a, b, wd) in [(hip, knee, 70), (knee, shin, 60)] {
                rig.blade(
                    a.lerp(b, ONE / 8).add(out.scale(pct(lw, 76))),
                    b.add(out.scale(pct(lw, 64))),
                    out,
                    pct(lw, wd),
                    pct(lw, 10),
                    shade(c, 82),
                );
            }
        }
    }
}

/// A leather holster at the `side` hip with a banana for a six-shooter,
/// its curve rising out of the top.
fn holster(rig: &mut Rig, m: &Dims, s: &Skel, wide: i32, side: i32) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let x = side * (wide / 2 + 6);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    rig.put(
        Mesh::Bevel,
        body.shape(at(x, 2, 10), at(16, 34, 26)),
        HOLSTER,
    );
    let a = body.at(pct(w, x), pct(t, 14), pct(d, 12));
    let b = body.at(pct(w, x + side * 4), pct(t, 32), pct(d, 20));
    let c = body.at(pct(w, x + side * 2), pct(t, 42), pct(d, 34));
    rig.put(Mesh::Taper, rod(a, b, pct(w, 12)), BANANA);
    rig.put(Mesh::Taper, rod(b, c, pct(w, 9)), BANANA);
    rig.ball(c, pct(w, 6), BANANA_TIP);
}

/// A bandolier from the right shoulder across the chest to the left hip
/// and round the back, three bananas in its loops.
fn bandolier(rig: &mut Rig, m: &Dims, s: &Skel, deep: i32) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    // (the chest is narrower than the belt: the proboscis monkey's belly)
    let (zt, zl) = (52, deep / 2 + 3);
    let top = body.at(pct(w, 34), pct(t, 98), pct(d, zt));
    let low = body.at(-pct(w, 48), pct(t, 22), pct(d, zl));
    rig.put(Mesh::Hex, rod(top, low, pct(w, 14)), GUNBELT);
    let back_top = body.at(pct(w, 34), pct(t, 98), -pct(d, zt));
    let back_low = body.at(-pct(w, 48), pct(t, 22), -pct(d, zl));
    rig.put(Mesh::Hex, rod(back_top, back_low, pct(w, 14)), GUNBELT);
    let along = dir_or(low.sub(top), body.y);
    let across = dir_or(along.cross(body.z), body.x);
    // bananas big enough to read on the small kinds too
    let u = w.max(cm(44));
    for k in [26, 50, 74] {
        let mid = top.lerp(low, ONE * k / 100).add(body.z.scale(pct(u, 6)));
        let a = mid.sub(across.scale(pct(u, 16)));
        let b = mid.add(body.z.scale(pct(u, 5)));
        let c = mid
            .add(across.scale(pct(u, 16)))
            .add(body.z.scale(pct(u, 3)));
        rig.put(Mesh::Taper, rod(a, b, pct(u, 11)), BANANA);
        rig.put(Mesh::Taper, rod(b, c, pct(u, 9)), BANANA);
        rig.ball(c, pct(u, 5), BANANA_TIP);
    }
}

/// A cowboy boot in place of the foot at `ankle` (on the feet's axes `f`):
/// the shaft up the shin toward the `knee`, a pointed toe, a heel and a
/// spur with its rowel at the back.
#[allow(clippy::too_many_arguments)]
pub(super) fn boot(rig: &mut Rig, kind: Kind, f: &Frame, knee: V3, ankle: V3, w: i32, side: i32) {
    let c = kit(kind).boots;
    let _ = side;
    let top = ankle.lerp(knee, ONE * 45 / 100);
    rig.put(Mesh::Taper, rod(top, ankle, pct(w, 132)), c);
    rig.put(
        Mesh::Hex,
        rod(top, top.lerp(ankle, ONE / 8), pct(w, 142)),
        shade(c, 80),
    );
    let f = Frame { o: ankle, ..*f };
    rig.put(
        Mesh::Bevel,
        f.shape(
            v3(0, -pct(w, 12), pct(w, 46)),
            v3(pct(w, 134), pct(w, 66), pct(w, 190)),
        ),
        c,
    );
    rig.put(
        Mesh::Taper,
        rod(
            f.at(0, -pct(w, 18), pct(w, 120)),
            f.at(0, -pct(w, 26), pct(w, 168)),
            pct(w, 104),
        ),
        c,
    );
    rig.put(
        Mesh::Cube,
        f.shape(
            v3(0, -pct(w, 40), -pct(w, 26)),
            v3(pct(w, 90), pct(w, 16), pct(w, 44)),
        ),
        shade(c, 56),
    );
    // the spur: a shank back from the heel, a spiked rowel on its end
    let heel = f.at(0, -pct(w, 22), -pct(w, 48));
    let rowel = f.at(0, -pct(w, 26), -pct(w, 84));
    rig.put(Mesh::Hex, rod(heel, rowel, pct(w, 14)), SPUR);
    rig.glow(
        Mesh::Octa,
        f.shape(
            v3(0, -pct(w, 26), -pct(w, 84)),
            v3(pct(w, 10), pct(w, 50), pct(w, 50)),
        ),
        SPUR,
        ONE / 5,
    );
}

#[cfg(test)]
mod tests {
    use super::super::tests::{at_rest, drawn, poses, triangles};
    use super::*;

    /// The colours that mark each piece of a kind's kit.
    fn kit_colours(kind: Kind) -> Vec<Rgb> {
        let k = kit(kind);
        let mut c = vec![k.hat.colour, k.hat.band, k.bandana, k.chaps, k.boots];
        c.extend([SPUR, GUNBELT, BUCKLE_GOLD, BANANA, BANANA_TIP]);
        if k.dots {
            c.push(PAISLEY);
        }
        match k.top {
            Top::Bare => {}
            Top::Vest { colour, star } => {
                c.push(colour);
                if star {
                    c.push(STAR_BADGE);
                }
            }
            Top::Shirt { colour, check } => c.extend([colour, check]),
        }
        c.extend(k.suspenders);
        if k.holster.is_some() {
            c.push(HOLSTER);
        }
        c
    }

    #[test]
    fn on_rattlesnake_ravine_every_kind_wears_its_own_cowboy_mix_in_every_pose() {
        // the person, 2026-10-17 (briefs/rattlesnake-ravine.md): a large
        // leather hat, bandanas, leather chaps, suspenders, holsters or
        // bandoliers containing a banana, boots and spurs; each ape its own
        // colours
        assert_eq!(Gear::from_key("cowboy"), Some(Gear::Cowboy));
        for map in crate::map::Map::ALL {
            assert_eq!(
                Gear::on(map) == Gear::Cowboy,
                map.key() == "ravine",
                "{map:?}"
            );
        }
        let all: Vec<Kit> = Kind::ALL.iter().map(|k| kit(*k)).collect();
        assert!(all.iter().any(|k| k.suspenders.is_some()));
        assert!(all.iter().any(|k| k.bandolier));
        assert!(all.iter().any(|k| k.holster.is_some()));
        assert!(all.iter().any(|k| k.fringe));
        // every kind carries a banana, in a holster or a bandolier
        for k in &all {
            assert!(k.holster.is_some() || k.bandolier);
        }
        // a palette each: no two kinds share a hat, bandana, chaps or boots
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a.hat.colour, b.hat.colour);
                assert_ne!(a.bandana, b.bandana);
                assert_ne!(a.chaps, b.chaps);
                assert_ne!(a.boots, b.boots);
            }
        }
        for kind in Kind::ALL {
            let l = look(kind);
            let k = kit(kind);
            for p in poses() {
                let pose = Pose {
                    gear: Gear::Cowboy,
                    ..p
                };
                let s = drawn(kind, &pose);
                let bare = drawn(kind, &p);
                assert!(s.len() > bare.len() + 8, "{kind:?}");
                assert!((50..=300).contains(&s.len()), "{kind:?} {} shapes", s.len());
                let has = |c: Rgb| s.iter().any(|x| x.colour == c);
                for c in kit_colours(kind) {
                    assert!(has(c), "{kind:?} {pose:?}: no {c:?}");
                }
                // the boots replace the bare feet: two spurs, no toes
                let spurs = s.iter().filter(|x| x.colour == SPUR).count();
                assert_eq!(spurs, 4, "{kind:?}: {spurs} spur pieces");
                assert!(!has(shade(l.skin, 90)), "{kind:?}: bare toes");
                for c in [
                    JACKET,
                    LENS,
                    KHAKI,
                    PITH,
                    TRICORN,
                    CAPTAIN_RED,
                    SCABBARD,
                    wizard::HAT_GREY,
                ] {
                    assert!(!has(c), "{kind:?}: {c:?}");
                }
                for x in &s {
                    for a in x.axes {
                        assert!(
                            a.len() >= ONE / 500 && a.len() <= int(2),
                            "{kind:?} mesh {} colour {:?} axis {}",
                            x.mesh,
                            x.colour,
                            a.len()
                        );
                    }
                    assert!(x.at.dist(pose.hands) <= int(3));
                }
                let d = cm(l.head) * l.size / 100;
                if pose.rope.is_none() && pose.grounded && !pose.stunned {
                    // the hat sits on the head: every piece over the face,
                    // within a head of it, and the crown over the skull
                    let face = s.iter().find(|x| x.colour == l.face).unwrap().at;
                    let hat: Vec<&_> = s.iter().filter(|x| x.colour == k.hat.colour).collect();
                    assert!(hat.len() >= 2, "{kind:?}: no hat");
                    for x in &hat {
                        let up = x.at.y - face.y;
                        assert!(up > d / 5 && up < d, "{kind:?}: hat {up} over the face");
                        assert!(x.at.dist(face) < d, "{kind:?}: hat off the head");
                    }
                }
                if pose.rope.is_none() && pose.grounded && pose.dance.is_none() {
                    // the boots stand on the ground under the ape
                    let ground = pose.hands.y - BODY_DROP;
                    let boots = s.iter().filter(|x| x.colour == k.boots);
                    let low = boots.map(|x| x.at.y).min().unwrap();
                    assert!(
                        low - ground >= -cm(1) && low - ground < cm(10),
                        "{kind:?}: boot {} over the ground",
                        low - ground
                    );
                }
                // swinging, the hands stay free: no piece in the grip
                if pose.rope.is_some() {
                    let own: Vec<Rgb> = kit_colours(kind)
                        .into_iter()
                        .filter(|c| *c != l.skin && *c != l.fur && *c != l.limb)
                        .collect();
                    for x in s.iter().filter(|x| own.contains(&x.colour)) {
                        assert!(
                            x.at.dist(pose.hands) > cm(10),
                            "{kind:?}: {:?} mesh {} at the grip ({})",
                            x.colour,
                            x.mesh,
                            x.at.dist(pose.hands)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn the_cowboy_outfits_cost_about_what_the_explorer_outfit_does() {
        let mut lines = Vec::new();
        let (mut cowboy_all, mut explorer_all) = (0, 0);
        for kind in Kind::ALL {
            let p = at_rest();
            let tris = |g: Gear| triangles(&drawn(kind, &Pose { gear: g, ..p }));
            let bare = tris(Gear::None);
            let (cowboy, explorer) = (tris(Gear::Cowboy) - bare, tris(Gear::Explorer) - bare);
            lines.push(format!(
                "{kind:?}: bare {bare}, explorer +{explorer}, cowboy +{cowboy}"
            ));
            cowboy_all += cowboy;
            explorer_all += explorer;
        }
        let table = lines.join("; ");
        assert!(cowboy_all <= explorer_all * 5 / 4, "{table}");
        for l in &lines {
            let n: Vec<usize> = l
                .split('+')
                .skip(1)
                .map(|x| {
                    x.split(|c: char| !c.is_ascii_digit())
                        .next()
                        .unwrap()
                        .parse()
                        .unwrap()
                })
                .collect();
            assert!(n[1] <= n[0] * 3 / 2, "{table}");
        }
    }
}
