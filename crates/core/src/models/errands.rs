//! Aisle Ape's second outfit, Grocery Run (`Gear::Errands`, 2026-10-19;
//! Sam, GR-T11: "the new outfits should be what modern people wear, so
//! graphic tees, jeans, cargo pants, a beanie, pajamas, the sort of fits
//! you would see people actually go to the grocery store in"). As with the
//! store uniforms, each kind wears its own fit, so the kinds read apart:
//! the chimpanzee a graphic tee and baggy jeans, the gorilla a hoodie and
//! cargo pants, the orangutan its pajamas, the gibbon a beanie and a puffer
//! vest, the proboscis monkey a college crewneck and sweatpants, the lemur
//! an oversized hoodie and a tote. Each palette is one of four colourways.
//! The graphics are made up: no real brand. Nothing is held: the bags hang
//! at the hip, so the hands stay free for the vines.

use super::*;

/// the shoes' white and the socks'
const SNEAKER_WHITE: Rgb = rgb(244, 244, 240);
const SOLE_GREY: Rgb = rgb(196, 196, 200);
const SOCK_WHITE: Rgb = rgb(236, 236, 232);
const SLIDE_BLACK: Rgb = rgb(30, 30, 34);
/// the chimpanzee's tee graphic: a banana in sunglasses over a line of
/// lettering
const PRINT_BANANA: Rgb = rgb(246, 210, 60);
const PRINT_SHADES: Rgb = rgb(22, 22, 24);
const PRINT_TIP: Rgb = rgb(92, 64, 34);
const STRING: Rgb = rgb(240, 240, 236);
const BAG_BLACK: Rgb = rgb(44, 44, 48);
const MOON: Rgb = rgb(242, 230, 166);
const LEEK: Rgb = rgb(78, 154, 58);
const LEEK_WHITE: Rgb = rgb(240, 238, 226);
const BAGUETTE: Rgb = rgb(216, 162, 90);
const CANVAS: Rgb = rgb(230, 220, 196);

/// What covers the head.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Hat {
    None,
    /// a ball cap worn backwards (the gorilla)
    Backwards(Rgb),
    /// a ribbed beanie with a folded cuff (the gibbon)
    Beanie(Rgb),
    /// a bucket hat (the proboscis monkey)
    Bucket(Rgb),
    /// a sleep mask pushed up on the forehead (the orangutan)
    Mask(Rgb),
}

impl Hat {
    /// Whether it covers the top of the head, so the hair there lies flat
    /// (the sleep mask leaves the orangutan's hair standing up).
    pub(super) fn covers_head(self) -> bool {
        matches!(self, Hat::Backwards(_) | Hat::Beanie(_) | Hat::Bucket(_))
    }
}

/// The top: a tee, a hoodie, a crewneck or a thermal under a vest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Top {
    /// an oversized tee with the banana graphic (the chimpanzee)
    Graphic,
    /// a faded tee with a moon and stars (the orangutan)
    Moon,
    /// a pullover hoodie: the hood down, drawstrings and a front pocket
    Hoodie,
    /// a college crewneck lettered APE U
    Crewneck,
    /// a long-sleeve thermal under a puffer vest
    Thermal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Legs {
    /// baggy jeans, cuffed
    Jeans,
    /// cargo pants with a pocket on each thigh
    Cargo,
    /// flannel pajama pants, checked by bands
    Pajamas,
    /// joggers, tapered to cuffs at the ankle
    Joggers,
    /// sweatpants, gathered at the ankle
    Sweats,
    /// bike shorts, above the knee
    Shorts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Shoes {
    /// white low-tops
    Sneakers,
    /// slides over white crew socks
    Slides,
    /// fuzzy slippers
    Slippers,
    /// running shoes, their own colour on a white sole
    Runners,
    /// foam clogs over white socks
    Clogs,
    /// chunky sneakers on a coloured sole
    Chunky,
}

/// One kind's fit in one colourway.
#[derive(Debug, Clone, Copy)]
pub(super) struct Kit {
    pub top: Top,
    pub legs: Legs,
    pub shoes: Shoes,
    pub hat: Hat,
    /// the top's colour (the tee, the hoodie, the crewneck, the thermal)
    pub shirt: Rgb,
    /// the trousers' or the shorts'
    pub trousers: Rgb,
    /// the second colour: the jeans' cuffs, the vest, the crewneck's
    /// letters, the pajamas' check, the shoes or soles
    pub second: Rgb,
    /// the shoes' own colour where they have one (slides' strap, runners,
    /// clogs, slippers, the chunky soles)
    pub shoe: Rgb,
    /// a crossbody bag (the chimpanzee) or a tote (the lemur)
    pub bag: Bag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Bag {
    None,
    Crossbody,
    /// a tote on one shoulder, a leek and a baguette standing out of it
    Tote(Rgb),
}

/// `kind`'s fit in colourway `palette` (0 to `PALETTES` - 1; past that it
/// wraps), the four of each as on the concept sheet.
pub(super) fn kit(kind: Kind, palette: u8) -> Kit {
    let p = (palette % PALETTES) as usize;
    let pick = |a: [Rgb; 4]| a[p];
    match kind {
        Kind::Chimpanzee => Kit {
            top: Top::Graphic,
            legs: Legs::Jeans,
            shoes: Shoes::Sneakers,
            hat: Hat::None,
            shirt: pick([
                rgb(242, 240, 234),
                rgb(39, 39, 44),
                rgb(169, 184, 154),
                rgb(239, 227, 200),
            ]),
            trousers: pick([
                rgb(143, 176, 208),
                rgb(46, 68, 102),
                rgb(90, 126, 166),
                rgb(43, 43, 48),
            ]),
            second: pick([
                rgb(168, 196, 222),
                rgb(62, 86, 128),
                rgb(112, 146, 184),
                rgb(60, 60, 66),
            ]),
            shoe: SNEAKER_WHITE,
            bag: Bag::Crossbody,
        },
        Kind::Gorilla => Kit {
            top: Top::Hoodie,
            legs: Legs::Cargo,
            shoes: Shoes::Slides,
            hat: Hat::Backwards(pick([
                rgb(42, 60, 106),
                rgb(184, 50, 46),
                rgb(232, 228, 218),
                rgb(212, 154, 42),
            ])),
            shirt: pick([
                rgb(154, 158, 166),
                rgb(44, 58, 98),
                rgb(42, 42, 46),
                rgb(47, 90, 62),
            ]),
            trousers: pick([
                rgb(107, 107, 62),
                rgb(184, 164, 122),
                rgb(168, 139, 96),
                rgb(122, 124, 130),
            ]),
            second: SOCK_WHITE,
            shoe: SLIDE_BLACK,
            bag: Bag::None,
        },
        Kind::Orangutan => Kit {
            top: Top::Moon,
            legs: Legs::Pajamas,
            shoes: Shoes::Slippers,
            hat: Hat::Mask(pick([
                rgb(156, 196, 232),
                rgb(242, 184, 198),
                rgb(250, 214, 120),
                rgb(192, 168, 224),
            ])),
            shirt: pick([
                rgb(185, 180, 200),
                rgb(216, 208, 192),
                rgb(200, 200, 204),
                rgb(168, 200, 232),
            ]),
            trousers: pick([
                rgb(184, 50, 58),
                rgb(60, 90, 154),
                rgb(62, 122, 76),
                rgb(240, 176, 192),
            ]),
            second: pick([
                rgb(58, 26, 32),
                rgb(22, 32, 58),
                rgb(22, 48, 30),
                rgb(252, 252, 252),
            ]),
            shoe: pick([
                rgb(242, 184, 198),
                rgb(232, 226, 214),
                rgb(138, 106, 80),
                rgb(250, 250, 250),
            ]),
            bag: Bag::None,
        },
        Kind::Gibbon => Kit {
            top: Top::Thermal,
            legs: Legs::Joggers,
            shoes: Shoes::Runners,
            hat: Hat::Beanie(pick([
                rgb(212, 154, 42),
                rgb(184, 50, 46),
                rgb(61, 107, 70),
                rgb(166, 144, 200),
            ])),
            shirt: pick([
                rgb(232, 224, 208),
                rgb(200, 204, 212),
                rgb(240, 236, 226),
                rgb(42, 42, 48),
            ]),
            trousers: pick([
                rgb(58, 58, 64),
                rgb(42, 42, 48),
                rgb(74, 74, 80),
                rgb(30, 30, 36),
            ]),
            second: pick([
                rgb(34, 34, 38),
                rgb(34, 48, 90),
                rgb(184, 154, 106),
                rgb(184, 188, 196),
            ]),
            shoe: pick([
                rgb(58, 106, 208),
                rgb(232, 228, 218),
                rgb(208, 106, 42),
                rgb(242, 242, 240),
            ]),
            bag: Bag::None,
        },
        Kind::Proboscis => Kit {
            top: Top::Crewneck,
            legs: Legs::Sweats,
            shoes: Shoes::Clogs,
            hat: Hat::Bucket(pick([
                rgb(200, 180, 138),
                rgb(42, 42, 48),
                rgb(232, 226, 214),
                rgb(90, 106, 58),
            ])),
            shirt: pick([
                rgb(122, 42, 54),
                rgb(46, 90, 60),
                rgb(38, 50, 90),
                rgb(216, 204, 180),
            ]),
            trousers: pick([
                rgb(142, 144, 150),
                rgb(69, 71, 78),
                rgb(164, 166, 172),
                rgb(38, 38, 42),
            ]),
            second: pick([
                rgb(242, 234, 214),
                rgb(242, 210, 60),
                rgb(232, 162, 58),
                rgb(122, 42, 54),
            ]),
            shoe: pick([
                rgb(92, 196, 74),
                rgb(242, 210, 60),
                rgb(58, 166, 224),
                rgb(240, 138, 168),
            ]),
            bag: Bag::None,
        },
        Kind::Lemur => Kit {
            top: Top::Hoodie,
            legs: Legs::Shorts,
            shoes: Shoes::Chunky,
            hat: Hat::None,
            shirt: pick([
                rgb(184, 168, 216),
                rgb(240, 220, 140),
                rgb(168, 192, 160),
                rgb(168, 200, 232),
            ]),
            trousers: pick([
                rgb(32, 32, 36),
                rgb(58, 58, 64),
                rgb(32, 32, 36),
                rgb(106, 108, 114),
            ]),
            second: SNEAKER_WHITE,
            shoe: pick([
                rgb(242, 184, 198),
                rgb(168, 200, 232),
                rgb(242, 230, 166),
                rgb(250, 250, 250),
            ]),
            bag: Bag::Tote(pick([CANVAS, CANVAS, rgb(42, 42, 48), CANVAS])),
        },
    }
}

/// How far round the torso the clothes sit (percent of its width and
/// depth), as the store uniforms: the gorilla's chest slabs and the
/// proboscis monkey's belly need more room.
fn fit(kind: Kind) -> (i32, i32) {
    match kind {
        Kind::Gorilla => (120, 142),
        Kind::Proboscis => (128, 160),
        _ => (112, 114),
    }
}

/// Everything a Grocery Run fit adds, over the ape `kind` posed as `s`.
pub(super) fn errands(rig: &mut Rig, kind: Kind, palette: u8, l: &Look, m: &Dims, s: &Skel) {
    let k = kit(kind, palette);
    let d = cm(l.head);
    let h = &s.head;
    match k.hat {
        Hat::None => {}
        Hat::Backwards(c) => backwards_cap(rig, d, h, c),
        Hat::Beanie(c) => beanie(rig, d, h, c),
        Hat::Bucket(c) => bucket_hat(rig, d, h, c),
        Hat::Mask(c) => sleep_mask(rig, d, h, c),
    }
    let (wide, deep) = fit(kind);
    // the front of the top, where its print or lettering goes (percent of
    // the torso's depth)
    let front = deep / 2 + 1;
    match k.top {
        Top::Graphic => {
            // oversized: a long hem and short, wide sleeves
            garment(rig, m, s, -6, 104, wide + 6, deep + 4, k.shirt);
            chest(rig, kind, m, s, k.shirt);
            sleeves(rig, m, s, 70, 176, k.shirt, shade(k.shirt, 92));
            banana_print(rig, m, s, front + 2, k.shirt);
        }
        Top::Moon => {
            garment(rig, m, s, -4, 104, wide + 6, deep + 4, k.shirt);
            chest(rig, kind, m, s, k.shirt);
            sleeves(rig, m, s, 64, 170, k.shirt, shade(k.shirt, 92));
            moon_print(rig, m, s, front + 2);
        }
        Top::Hoodie => {
            // the lemur's hangs to mid-thigh
            let hem = if kind == Kind::Lemur { -22 } else { -2 };
            garment(rig, m, s, hem, 104, wide + 4, deep + 4, k.shirt);
            chest(rig, kind, m, s, k.shirt);
            sleeves(rig, m, s, 192, 168, k.shirt, shade(k.shirt, 86));
            hoodie(rig, m, s, deep + 4, hem, k.shirt);
        }
        Top::Crewneck => {
            garment(rig, m, s, -2, 104, wide + 4, deep + 4, k.shirt);
            chest(rig, kind, m, s, k.shirt);
            sleeves(rig, m, s, 190, 160, k.shirt, shade(k.shirt, 84));
            hem_band(rig, m, s, -2, wide + 4, deep + 4, shade(k.shirt, 84));
            // on the chest, and big across the back, where the chase
            // camera sees it (a proboscis monkey runs leaning far over, its
            // chest to the floor)
            lettering(rig, m, s, front - 7, 1, k.second);
            lettering(rig, m, s, front - 7, -1, k.second);
        }
        Top::Thermal => {
            garment(rig, m, s, 0, 104, wide, deep, k.shirt);
            chest(rig, kind, m, s, k.shirt);
            sleeves(rig, m, s, 192, 150, k.shirt, shade(k.shirt, 88));
            puffer(rig, m, s, wide + 8, deep + 8, k.second);
        }
    }
    match k.bag {
        Bag::None => {}
        Bag::Crossbody => crossbody(rig, m, s, wide, front + 4),
        Bag::Tote(c) => tote(rig, m, s, wide, c),
    }
    trousers(rig, m, s, &k);
    shoes(rig, m, s, &k);
}

/// Over the gorilla's chest slabs, which the round top leaves out (as the
/// store uniform's shirt).
fn chest(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, c: Rgb) {
    if kind == Kind::Gorilla {
        let (w, t, d) = (m.w, m.t, m.d);
        rig.put(
            Mesh::Bevel,
            s.body.shape(
                v3(0, pct(t, 70), pct(d, 40)),
                v3(pct(w, 100), pct(t, 36), pct(d, 44)),
            ),
            c,
        );
    }
}

/// The chimpanzee's tee graphic on the chest, `z` percent of the torso
/// deep: a banana in sunglasses over a bar of lettering.
fn banana_print(rig: &mut Rig, m: &Dims, s: &Skel, z: i32, shirt: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let zz = pct(d, z);
    // the banana: a fat crescent of kites, fattest in the middle, its
    // tips up and brown
    let pts = [(-30, 90), (-20, 76), (-6, 68), (8, 68), (22, 76), (32, 88)];
    for i in 0..5 {
        let (ax, ay) = pts[i];
        let (bx, by) = pts[i + 1];
        let fat = [12, 20, 24, 20, 12][i];
        rig.blade(
            body.at(pct(w, ax), pct(t, ay), zz),
            body.at(pct(w, bx), pct(t, by), zz),
            body.y,
            pct(w, fat),
            pct(d, 3),
            PRINT_BANANA,
        );
    }
    for (x, y) in [(-31, 92), (33, 90)] {
        rig.ball(body.at(pct(w, x), pct(t, y), zz), pct(w, 7), PRINT_TIP);
    }
    // its sunglasses
    for x in [-8, 8] {
        rig.put(
            Mesh::Cube,
            body.shape(
                v3(pct(w, x), pct(t, 74), zz + pct(d, 2)),
                v3(pct(w, 12), pct(t, 5), pct(d, 3)),
            ),
            PRINT_SHADES,
        );
    }
    // the lettering under it, in the shirt's opposite
    let ink = if (shirt[0] * 3 + shirt[1] * 6 + shirt[2]) / 10 > 140 {
        rgb(42, 42, 48)
    } else {
        rgb(242, 240, 234)
    };
    rig.put(
        Mesh::Cube,
        body.shape(v3(0, pct(t, 54), zz), v3(pct(w, 52), pct(t, 4), pct(d, 3))),
        ink,
    );
}

/// The orangutan's faded tee print: a crescent moon and three stars.
fn moon_print(rig: &mut Rig, m: &Dims, s: &Skel, z: i32) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let zz = pct(d, z);
    // the crescent: kites round most of a circle
    for i in 0..5 {
        let a = deg(110 + i * 34);
        let b = deg(110 + (i + 1) * 34);
        let p = |a: i32| {
            body.at(
                pct(w, 6) + fx::mul(fx::cos(a), pct(w, 16)),
                pct(t, 70) + fx::mul(fx::sin(a), pct(w, 16)),
                zz,
            )
        };
        rig.blade(p(a), p(b), body.z, pct(w, 7), pct(d, 3), MOON);
    }
    for (x, y) in [(-24, 82), (20, 60), (-14, 52)] {
        rig.put(
            Mesh::Cube,
            body.shape(
                v3(pct(w, x), pct(t, y), zz),
                v3(pct(w, 5), pct(w, 5), pct(d, 3)),
            ),
            MOON,
        );
    }
}

/// The hoodie's hood lying down behind the neck, its drawstrings and its
/// front pocket.
fn hoodie(rig: &mut Rig, m: &Dims, s: &Skel, deep: i32, hem: i32, c: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    // (round and lying flat: a box there read as a backpack)
    rig.put(
        Mesh::Geo,
        body.shape(at(0, 96, -deep / 2 + 6), at(52, 18, 26)),
        shade(c, 90),
    );
    // the drawstrings, lying on the chest
    for x in [-8, 8] {
        let a = body.at(pct(w, x), pct(t, 96), pct(d, deep / 2 - 6));
        let b = body.at(pct(w, x), pct(t, 80), pct(d, deep / 2 - 2));
        rig.put(Mesh::Hex, rod(a, b, pct(w, 3)), STRING);
    }
    let y = (hem + 50) / 2;
    rig.put(
        Mesh::Cube,
        body.shape(at(0, y, deep / 2 - 6), at(56, 24, 4)),
        shade(c, 92),
    );
    hem_band(rig, m, s, hem, 0, deep, shade(c, 86));
}

/// A ribbed band round the hem.
fn hem_band(rig: &mut Rig, m: &Dims, s: &Skel, hem: i32, wide: i32, deep: i32, c: Rgb) {
    let (w, t, d) = (m.w, m.t, m.d);
    let wide = if wide == 0 { 112 } else { wide };
    rig.put(
        Mesh::Barrel,
        s.body.shape(
            v3(0, pct(t, hem + 3), 0),
            v3(pct(w, wide + 3), pct(t, 7), pct(d, deep + 3)),
        ),
        c,
    );
}

/// The crewneck's lettering, APE U, in blocks across the chest (`face` 1)
/// or the back (-1, mirrored so it reads from behind).
fn lettering(rig: &mut Rig, m: &Dims, s: &Skel, z: i32, face: i32, c: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let zz = face * pct(d, z);
    let out = body.z.scale(face * ONE);
    let (top, bot) = (80, 60);
    // (x across, percent of the torso; bars as (x0, y0, x1, y1))
    let bars: [(i32, i32, i32, i32); 13] = [
        // A
        (-46, bot, -40, top),
        (-34, bot, -40, top),
        (-44, 68, -36, 68),
        // P
        (-26, bot, -26, top),
        (-26, top, -16, top),
        (-16, top, -16, 70),
        (-26, 70, -16, 70),
        // E
        (-6, bot, -6, top),
        (-6, top, 4, top),
        (-6, 70, 2, 70),
        (-6, bot, 4, bot),
        // U
        (22, bot, 22, top),
        (34, bot, 34, top),
    ];
    for (x0, y0, x1, y1) in bars {
        let a = body.at(face * pct(w, x0), pct(t, y0), zz);
        let b = body.at(face * pct(w, x1), pct(t, y1), zz);
        let b = if a.dist(b) < pct(w, 1) {
            b.add(body.x.scale(pct(w, 1)))
        } else {
            b
        };
        rig.put(Mesh::Cube, bar(a, b, out, pct(w, 5), pct(d, 3)), c);
    }
    rig.put(
        Mesh::Cube,
        bar(
            body.at(face * pct(w, 22), pct(t, bot), zz),
            body.at(face * pct(w, 34), pct(t, bot), zz),
            out,
            pct(w, 5),
            pct(d, 3),
        ),
        c,
    );
}

/// A box from `a` to `b`, `w` wide and `thick` through along `out`.
fn bar(a: V3, b: V3, out: V3, w: i32, thick: i32) -> Affine {
    let along = b.sub(a);
    let z = dir_or(out, V3::UP);
    let x = dir_or(along.reject(z), v3(ONE, 0, 0));
    let y = z.cross(x).norm();
    Affine {
        c0: x.scale(along.len() + w),
        c1: y.scale(w),
        c2: z.scale(thick),
        t: a.lerp(b, ONE / 2),
    }
}

/// The puffer vest over the thermal: from the waist to the chest, straps
/// over the shoulders, quilted in bands, a zip down the front.
fn puffer(rig: &mut Rig, m: &Dims, s: &Skel, wide: i32, deep: i32, c: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    garment(rig, m, s, 10, 94, wide, deep, c);
    for side in SIDES {
        rig.put(
            Mesh::Cube,
            body.shape(at(side * 24, 99, -4), at(18, 10, deep - 12)),
            c,
        );
    }
    for y in [28, 46, 64, 82] {
        rig.put(
            Mesh::Barrel,
            body.shape(at(0, y, 0), at(wide + 4, 5, deep + 4)),
            shade(c, 80),
        );
    }
    rig.put(
        Mesh::Cube,
        body.shape(at(0, 52, deep / 2 + 1), at(4, 80, 3)),
        shade(c, 60),
    );
}

/// The chimpanzee's crossbody bag: a strap from the left shoulder across
/// the chest to a small pouch at the right hip.
fn crossbody(rig: &mut Rig, m: &Dims, s: &Skel, wide: i32, z: i32) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let a = body.at(-pct(w, 30), pct(t, 100), pct(d, z - 10));
    let mid = body.at(pct(w, 4), pct(t, 60), pct(d, z));
    let b = body.at(pct(w, wide / 2 - 8), pct(t, 26), pct(d, z - 4));
    for (p, q) in [(a, mid), (mid, b)] {
        rig.put(Mesh::Hex, rod(p, q, pct(w, 6)), BAG_BLACK);
    }
    rig.put(
        Mesh::Bevel,
        body.shape(
            v3(pct(w, wide / 2 - 6), pct(t, 18), pct(d, z - 4)),
            v3(pct(w, 34), pct(t, 22), pct(d, 18)),
        ),
        BAG_BLACK,
    );
}

/// The lemur's tote on its left shoulder, hanging at the hip, a leek and a
/// baguette standing out of it.
fn tote(rig: &mut Rig, m: &Dims, s: &Skel, wide: i32, c: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let x = -pct(w, wide / 2 + 14);
    let top = body.at(x, pct(t, 40), 0);
    for z in [-14, 14] {
        let sh = body.at(-pct(w, 30), pct(t, 104), pct(d, z));
        rig.put(
            Mesh::Hex,
            rod(sh, top.add(body.z.scale(pct(d, z))), pct(w, 5)),
            c,
        );
    }
    rig.put(
        Mesh::Bevel,
        body.shape(v3(x, pct(t, 20), 0), v3(pct(w, 18), pct(t, 44), pct(d, 64))),
        c,
    );
    // the leek, its white end in the bag, and the baguette beside it
    let leek = body.at(x, pct(t, 30), pct(d, 14));
    let tip = leek.add(body.y.scale(pct(t, 62)));
    rig.put(Mesh::Hex, rod(leek, tip, pct(w, 9)), LEEK);
    rig.put(
        Mesh::Hex,
        rod(leek, leek.add(body.y.scale(pct(t, 20))), pct(w, 10)),
        LEEK_WHITE,
    );
    for side in SIDES {
        rig.blade(
            tip,
            tip.add(body.y.scale(pct(t, 20)))
                .add(body.z.scale(side * pct(d, 12))),
            body.x,
            pct(w, 8),
            pct(w, 2),
            LEEK,
        );
    }
    let loaf = body.at(x, pct(t, 26), -pct(d, 16));
    rig.put(
        Mesh::Taper,
        rod(loaf, loaf.add(body.y.scale(pct(t, 58))), pct(w, 12)),
        BAGUETTE,
    );
}

/// Each leg's trousers: their cut by `k.legs`.
fn trousers(rig: &mut Rig, m: &Dims, s: &Skel, k: &Kit) {
    let lw = m.leg_w;
    let c = k.trousers;
    for (i, side) in SIDES.into_iter().enumerate() {
        let hip = hip_of(&s.body, m, side);
        let (knee, ankle) = bend(hip, s.feet[i], m.leg, s.leg_hint[i]);
        let at = |p: i32| knee.lerp(ankle, ONE * p / 100);
        match k.legs {
            Legs::Shorts => {
                let end = hip.lerp(knee, ONE * 80 / 100);
                rig.put(Mesh::Taper, rod(hip, end, pct(lw, 150)), c);
                rig.put(
                    Mesh::Hex,
                    rod(hip.lerp(end, ONE * 84 / 100), end, pct(lw, 142)),
                    shade(c, 80),
                );
                continue;
            }
            Legs::Jeans => {
                rig.put(Mesh::Taper, rod(hip, knee, pct(lw, 174)), c);
                rig.ball(knee, pct(lw, 150), c);
                rig.put(Mesh::Taper, rod(knee, at(84), pct(lw, 164)), c);
                // the turned-up cuff, the denim's paler inside
                rig.put(Mesh::Hex, rod(at(80), at(96), pct(lw, 172)), k.second);
            }
            Legs::Cargo => {
                rig.put(Mesh::Taper, rod(hip, knee, pct(lw, 162)), c);
                rig.ball(knee, pct(lw, 138), c);
                rig.put(Mesh::Taper, rod(knee, at(92), pct(lw, 144)), c);
                // a pocket on the outside of the thigh, its flap darker
                let f = Frame::upright(hip.lerp(knee, ONE * 58 / 100), knee.sub(hip), s.body.z);
                let out = side * pct(lw, 80);
                rig.put(
                    Mesh::Bevel,
                    f.shape(v3(out, 0, 0), v3(pct(lw, 30), pct(lw, 96), pct(lw, 84))),
                    shade(c, 94),
                );
                rig.put(
                    Mesh::Cube,
                    f.shape(
                        v3(out + side * pct(lw, 6), pct(lw, 46), 0),
                        v3(pct(lw, 30), pct(lw, 18), pct(lw, 96)),
                    ),
                    shade(c, 80),
                );
            }
            Legs::Pajamas => {
                rig.put(Mesh::Taper, rod(hip, knee, pct(lw, 166)), c);
                rig.ball(knee, pct(lw, 142), c);
                rig.put(Mesh::Taper, rod(knee, at(96), pct(lw, 156)), c);
                // the flannel's check: dark bands round the leg, a pale one
                // between
                for (a, b, w) in [
                    (hip.lerp(knee, ONE / 5), hip.lerp(knee, ONE * 3 / 10), 172),
                    (
                        hip.lerp(knee, ONE * 6 / 10),
                        hip.lerp(knee, ONE * 7 / 10),
                        168,
                    ),
                    (at(24), at(34), 160),
                    (at(62), at(72), 160),
                ] {
                    rig.put(Mesh::Hex, rod(a, b, pct(lw, w)), k.second);
                }
                for (a, b, w) in [
                    (
                        hip.lerp(knee, ONE * 4 / 10),
                        hip.lerp(knee, ONE * 45 / 100),
                        170,
                    ),
                    (at(46), at(50), 160),
                ] {
                    rig.put(Mesh::Hex, rod(a, b, pct(lw, w)), shade(c, 130));
                }
            }
            Legs::Joggers => {
                rig.put(Mesh::Taper, rod(hip, knee, pct(lw, 150)), c);
                rig.ball(knee, pct(lw, 134), c);
                rig.put(Mesh::Taper, rod(knee, at(86), pct(lw, 128)), c);
                rig.put(Mesh::Hex, rod(at(84), at(100), pct(lw, 124)), shade(c, 80));
            }
            Legs::Sweats => {
                rig.put(Mesh::Taper, rod(hip, knee, pct(lw, 164)), c);
                rig.ball(knee, pct(lw, 140), c);
                rig.put(Mesh::Taper, rod(knee, at(86), pct(lw, 150)), c);
                rig.put(Mesh::Hex, rod(at(84), at(100), pct(lw, 130)), shade(c, 88));
            }
        }
    }
}

/// Each foot's shoes: their kind by `k.shoes`.
fn shoes(rig: &mut Rig, m: &Dims, s: &Skel, k: &Kit) {
    let lw = m.leg_w;
    let p = |x: i32, y: i32, z: i32| v3(pct(lw, x), pct(lw, y), pct(lw, z));
    for (i, side) in SIDES.into_iter().enumerate() {
        let hip = hip_of(&s.body, m, side);
        let (knee, ankle) = bend(hip, s.feet[i], m.leg, s.leg_hint[i]);
        let f = Frame {
            o: ankle,
            ..s.feet_frame
        };
        let sock = |rig: &mut Rig| {
            rig.put(
                Mesh::Hex,
                rod(ankle, ankle.lerp(knee, ONE * 22 / 100), pct(lw, 124)),
                SOCK_WHITE,
            );
        };
        match k.shoes {
            Shoes::Sneakers => {
                // (the upper's underside inside the sole, clear of the toes')
                rig.put(Mesh::Bevel, f.shape(p(0, 13, 58), p(176, 70, 226)), k.shoe);
                rig.put(
                    Mesh::Cube,
                    f.shape(p(0, -33, 58), p(184, 24, 236)),
                    SOLE_GREY,
                );
            }
            Shoes::Runners => {
                rig.put(Mesh::Bevel, f.shape(p(0, 13, 58), p(172, 70, 224)), k.shoe);
                rig.put(
                    Mesh::Cube,
                    f.shape(p(0, -33, 58), p(180, 24, 234)),
                    SNEAKER_WHITE,
                );
                // a white stripe down the outside
                rig.blade(
                    f.at(side * pct(lw, 90), pct(lw, 20), pct(lw, 10)),
                    f.at(side * pct(lw, 90), -pct(lw, 14), pct(lw, 104)),
                    f.y,
                    pct(lw, 18),
                    pct(lw, 6),
                    SNEAKER_WHITE,
                );
            }
            Shoes::Chunky => {
                rig.put(
                    Mesh::Bevel,
                    f.shape(p(0, 14, 58), p(180, 70, 230)),
                    k.second,
                );
                rig.put(Mesh::Cube, f.shape(p(0, -40, 58), p(192, 40, 242)), k.shoe);
            }
            Shoes::Slides => {
                sock(rig);
                rig.put(Mesh::Cube, f.shape(p(0, -40, 54), p(168, 14, 216)), k.shoe);
                rig.put(Mesh::Bevel, f.shape(p(0, 4, 70), p(170, 52, 70)), k.shoe);
            }
            Shoes::Slippers => {
                rig.put(Mesh::Bevel, f.shape(p(0, 4, 56), p(196, 96, 240)), k.shoe);
                // the fuzz round the opening
                for x in [-60, -20, 20, 60] {
                    rig.ball(
                        f.at(pct(lw, x), pct(lw, 44), pct(lw, 4)),
                        pct(lw, 46),
                        k.shoe,
                    );
                }
            }
            Shoes::Clogs => {
                sock(rig);
                rig.put(Mesh::Bevel, f.shape(p(0, 4, 60), p(186, 90, 232)), k.shoe);
                for x in [-40, 0, 40] {
                    rig.ball(
                        f.at(pct(lw, x), pct(lw, 36), pct(lw, 116)),
                        pct(lw, 16),
                        shade(k.shoe, 60),
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Hats, kept low (the chase camera looks over them)

/// A ball cap worn backwards: the crown, the brim over the back of the
/// neck, and the strap's gap at the forehead.
fn backwards_cap(rig: &mut Rig, d: i32, h: &Frame, c: Rgb) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    rig.put(Mesh::Geo, h.shape(p(0, 38, -4), p(102, 62, 102)), c);
    rig.put(
        Mesh::Bevel,
        h.shape(p(0, 36, -62), p(76, 5, 50)),
        shade(c, 86),
    );
    rig.ball(h.at(0, pct(d, 69), -pct(d, 4)), pct(d, 10), shade(c, 80));
    rig.put(
        Mesh::Cube,
        h.shape(p(0, 36, 50), p(22, 10, 6)),
        shade(c, 60),
    );
}

/// A ribbed beanie, its cuff turned up.
fn beanie(rig: &mut Rig, d: i32, h: &Frame, c: Rgb) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    rig.put(Mesh::Geo, h.shape(p(0, 42, -4), p(104, 74, 104)), c);
    let band = h.at(0, pct(d, 26), -pct(d, 4));
    rig.put(
        Mesh::Hex,
        rod(band, band.add(h.y.scale(pct(d, 18))), pct(d, 110)),
        shade(c, 88),
    );
}

/// A bucket hat: a round crown and a brim sloping down all round.
fn bucket_hat(rig: &mut Rig, d: i32, h: &Frame, c: Rgb) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    rig.put(Mesh::Geo, h.shape(p(0, 44, -4), p(100, 66, 100)), c);
    let rim = h.at(0, pct(d, 24), -pct(d, 4));
    rig.put(
        Mesh::Taper,
        rod(rim.add(h.y.scale(pct(d, 12))), rim, pct(d, 150)),
        shade(c, 90),
    );
}

/// A sleep mask pushed up on the forehead: its band round the head and the
/// mask over the brow, two closed eyes stitched on it.
fn sleep_mask(rig: &mut Rig, d: i32, h: &Frame, c: Rgb) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    let band = h.at(0, pct(d, 36), -pct(d, 4));
    rig.put(
        Mesh::Hex,
        rod(band, band.add(h.y.scale(pct(d, 8))), pct(d, 104)),
        shade(c, 80),
    );
    rig.put(Mesh::Bevel, h.shape(p(0, 42, 44), p(80, 22, 16)), c);
    for x in [-18, 18] {
        rig.put(Mesh::Cube, h.shape(p(x, 40, 53), p(18, 3, 3)), shade(c, 50));
    }
}

#[cfg(test)]
mod tests {
    use super::super::food::tests::read;
    use super::super::mech::tests::{faces, fight, same};
    use super::super::tests::{at_rest, poses, triangles, Shape};
    use super::*;
    use crate::costume::Costume;
    use crate::map::Map;

    fn drawn_in(kind: Kind, palette: u8, pose: &Pose) -> Vec<Shape> {
        let mut b = Batch::new();
        draw_in(&mut b, kind, palette, pose);
        read(b)
    }

    fn errands_pose(p: Pose) -> Pose {
        Pose {
            gear: Gear::Errands,
            ..p
        }
    }

    /// The colours that mark a kit's own pieces.
    fn kit_colours(k: &Kit) -> Vec<Rgb> {
        let mut c = vec![k.shirt, k.trousers];
        match k.hat {
            Hat::None => {}
            Hat::Backwards(x) | Hat::Beanie(x) | Hat::Bucket(x) | Hat::Mask(x) => c.push(x),
        }
        match k.shoes {
            Shoes::Sneakers => c.extend([k.shoe, SOLE_GREY]),
            Shoes::Runners => c.extend([k.shoe, SNEAKER_WHITE]),
            Shoes::Chunky => c.extend([k.shoe, k.second]),
            Shoes::Slides | Shoes::Clogs => c.extend([k.shoe, SOCK_WHITE]),
            Shoes::Slippers => c.push(k.shoe),
        }
        match k.bag {
            Bag::None => {}
            Bag::Crossbody => c.push(BAG_BLACK),
            Bag::Tote(x) => c.extend([x, LEEK, BAGUETTE]),
        }
        match k.top {
            Top::Graphic => c.extend([PRINT_BANANA, PRINT_SHADES]),
            Top::Moon => c.push(MOON),
            Top::Hoodie => c.push(STRING),
            Top::Crewneck | Top::Thermal => c.push(k.second),
        }
        if k.legs == Legs::Jeans || k.legs == Legs::Pajamas {
            c.push(k.second);
        }
        c
    }

    #[test]
    fn grocery_run_dresses_every_kind_in_its_own_everyday_fit() {
        // Sam, GR-T11: "graphic tees, jeans, cargo pants, a beanie,
        // pajamas, the sort of fits you would see people actually go to the
        // grocery store in"
        assert_eq!(Gear::from_key("errands"), Some(Gear::Errands));
        // no map dresses its apes in it: it is worn as a costume
        for map in Map::ALL {
            assert_ne!(Gear::on(map), Gear::Errands, "{map:?}");
        }
        let kits: Vec<Kit> = Kind::ALL.iter().map(|k| kit(*k, 0)).collect();
        // each of the pieces Sam named is worn by some kind
        assert!(kits.iter().any(|k| k.top == Top::Graphic));
        assert!(kits.iter().any(|k| k.legs == Legs::Jeans));
        assert!(kits.iter().any(|k| k.legs == Legs::Cargo));
        assert!(kits.iter().any(|k| matches!(k.hat, Hat::Beanie(_))));
        assert!(kits.iter().any(|k| k.legs == Legs::Pajamas));
        // and no two kinds wear the same fit
        for (i, a) in kits.iter().enumerate() {
            for b in &kits[i + 1..] {
                assert!(
                    (a.top, a.legs, a.shoes) != (b.top, b.legs, b.shoes),
                    "{:?} twice",
                    (a.top, a.legs, a.shoes)
                );
            }
        }
        for kind in Kind::ALL {
            // four colourways, each its own top or trousers
            let mut seen: Vec<(Rgb, Rgb)> = Vec::new();
            for palette in 0..PALETTES {
                let k = kit(kind, palette);
                assert!(!seen.contains(&(k.shirt, k.trousers)), "{kind:?} {palette}");
                seen.push((k.shirt, k.trousers));
            }
            for palette in 0..PALETTES {
                let k = kit(kind, palette);
                let lp = look_in(kind, palette);
                for p in poses() {
                    let s = drawn_in(kind, palette, &errands_pose(p));
                    let bare = drawn_in(kind, palette, &p);
                    assert!(s.len() > bare.len() + 15, "{kind:?}");
                    assert!((50..=300).contains(&s.len()), "{kind:?} {} shapes", s.len());
                    let has = |c: Rgb| s.iter().any(|x| x.colour == c);
                    for c in kit_colours(&k) {
                        assert!(has(c), "{kind:?} {palette} {p:?}: no {c:?}");
                    }
                    // the ape itself still drawn: its own fur and face
                    assert!(has(lp.fur) && has(lp.face), "{kind:?}: not the ape");
                    for x in &s {
                        for a in x.axes {
                            assert!(
                                a.len() >= ONE / 1000 && a.len() <= int(2),
                                "{kind:?} mesh {} colour {:?} axis {}",
                                x.mesh,
                                x.colour,
                                a.len()
                            );
                        }
                        assert!(x.at.dist(p.hands) <= int(3));
                    }
                    // a hat sits on the head: over the face, within a head
                    // of it
                    let d = cm(look(kind).head) * look(kind).size / 100;
                    if p.rope.is_none() && p.grounded && !p.stunned {
                        if let Hat::Backwards(c) | Hat::Beanie(c) | Hat::Bucket(c) | Hat::Mask(c) =
                            k.hat
                        {
                            let face = s.iter().find(|x| x.colour == lp.face).unwrap().at;
                            let on: Vec<&Shape> = s
                                .iter()
                                .filter(|x| x.colour == c && x.at.dist(face) < d)
                                .collect();
                            assert!(!on.is_empty(), "{kind:?}: no hat on the head");
                            for x in &on {
                                assert!(x.at.y > face.y, "{kind:?}: hat under the face");
                            }
                        }
                    }
                    // swinging, the hands stay free: no piece in the grip
                    if p.rope.is_some() {
                        let own: Vec<Rgb> = kit_colours(&k)
                            .into_iter()
                            .filter(|c| *c != lp.skin && *c != lp.fur && *c != lp.limb)
                            .collect();
                        for x in s.iter().filter(|x| own.contains(&x.colour)) {
                            assert!(
                                x.at.dist(p.hands) > cm(10),
                                "{kind:?}: {:?} mesh {} at the grip ({})",
                                x.colour,
                                x.mesh,
                                x.at.dist(p.hands)
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn the_crewneck_is_lettered_on_the_back_where_the_chase_camera_sees_it() {
        // a proboscis monkey runs leaning far over (its chest to the floor,
        // as the mech armour's chest plate, SP-M), so APE U goes on the
        // back as well as the chest
        let kind = Kind::Proboscis;
        let k = kit(kind, 0);
        assert_eq!(k.top, Top::Crewneck);
        let pose = errands_pose(at_rest());
        let s = drawn_in(kind, 0, &pose);
        let bare = drawn_in(kind, 0, &at_rest());
        let middle = bare
            .iter()
            .fold(V3::ZERO, |a, x| a.add(x.at.scale(ONE / bare.len() as i32)));
        let letters: Vec<&Shape> = s
            .iter()
            .filter(|x| x.colour == k.second && x.mesh == Mesh::Cube as u32)
            .collect();
        // (leaning over, its back is up, toward a camera behind and above)
        let up = letters.iter().filter(|x| x.at.y > middle.y).count();
        let down = letters.len() - up;
        assert!(
            up >= 10 && down >= 10,
            "{up} on the back, {down} on the chest"
        );
    }

    #[test]
    fn grocery_run_costs_about_what_the_cowboy_outfits_do() {
        let mut lines = Vec::new();
        let mut over = Vec::new();
        for kind in Kind::ALL {
            let p = at_rest();
            let tris = |g: Gear| triangles(&drawn_in(kind, 0, &Pose { gear: g, ..p }));
            let bare = tris(Gear::None);
            let errands = tris(Gear::Errands) - bare;
            let cowboy = tris(Gear::Cowboy) - bare;
            lines.push(format!("{kind:?}: errands +{errands}, cowboy +{cowboy}"));
            if errands * 2 > cowboy * 3 {
                over.push(kind);
            }
        }
        assert!(over.is_empty(), "{}", lines.join("; "));
    }

    /// No piece the outfit adds has a face in the plane of another shape's
    /// face, facing the same way and overlapping it, in another colour: the
    /// two would flicker (the mech armour's test, SP-M10).
    #[test]
    fn no_piece_of_grocery_run_shares_a_face_with_another_colour() {
        let gap = cm(1) / 5;
        let mut bad = Vec::new();
        for kind in Kind::ALL {
            for palette in 0..PALETTES {
                for p in poses() {
                    let bare = drawn_in(kind, palette, &p);
                    let s = drawn_in(kind, palette, &errands_pose(p));
                    let new: Vec<bool> =
                        s.iter().map(|y| !bare.iter().any(|x| same(x, y))).collect();
                    let f: Vec<_> = s.iter().map(faces).collect();
                    for i in 0..s.len() {
                        for j in i + 1..s.len() {
                            if !(new[i] || new[j])
                                || (s[i].colour == s[j].colour && s[i].glow == s[j].glow)
                            {
                                continue;
                            }
                            if f[i].iter().any(|a| f[j].iter().any(|b| fight(a, b, gap))) {
                                bad.push(format!(
                                    "{kind:?} {palette}: mesh {} {:?} at {:?} / mesh {} {:?} at {:?}",
                                    s[i].mesh, s[i].colour, s[i].at, s[j].mesh, s[j].colour, s[j].at
                                ));
                            }
                        }
                    }
                }
            }
        }
        assert!(bad.is_empty(), "{} fights:\n{}", bad.len(), bad.join("\n"));
    }

    #[test]
    fn grocery_run_is_a_costume_of_no_map_of_its_own() {
        assert!(Costume::ALL.contains(&Costume::Errands));
        assert_eq!(Costume::from_key("errands"), Some(Costume::Errands));
        assert_eq!(Gear::from_key(Costume::Errands.key()), Some(Gear::Errands));
        // no map's Author time alone unlocks it, and it is not free
        assert_eq!(Costume::Errands.map(), None);
        assert!(!Costume::Errands.free());
        for map in Map::ALL {
            assert_eq!(Costume::Errands.gear(map), Gear::Errands);
            assert_ne!(Costume::of_map(map), Some(Costume::Errands));
        }
    }
}
