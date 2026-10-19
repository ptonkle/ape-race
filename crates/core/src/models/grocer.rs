//! Aisle Ape's outfit (`Gear::Grocer`, 2026-10-18; the person,
//! GR-T1: "make the ape costumes to be workers uniforms of the store, with
//! a big banana in a circle as a logo"). The apes keep their own bodies at
//! their own size and each kind works a different job in the store, so the
//! kinds read apart: the chimpanzee the cashier, the gibbon the stocker,
//! the proboscis monkey the deli worker, the orangutan the produce clerk,
//! the gorilla the store manager and the lemur the cart wrangler. Every
//! one wears the store's logo (`grocerymodels::banana_logo`, a yellow
//! banana in a navy circle) on the chest or apron and big on the back, and
//! most on a cap or visor too; each logo sits on a white patch, so the navy
//! ring reads on a navy vest. Each palette changes the shirt or the apron
//! (`kit`). Nothing is held: the hands stay free for the vines. No real
//! store: the name, the colours and the logo are the map's own.

use super::*;
use crate::grocerymodels::{BANNER_RED, CREAM, NAVY, WHITE};

/// polos and tees
pub(super) const POLO_TEAL: Rgb = rgb(22, 138, 140);
pub(super) const POLO_ROYAL: Rgb = rgb(42, 84, 192);
pub(super) const POLO_PURPLE: Rgb = rgb(112, 52, 142);
pub(super) const TEE_HEATHER: Rgb = rgb(150, 154, 162);
pub(super) const TEE_FOREST: Rgb = rgb(52, 108, 70);
pub(super) const TEE_MAROON: Rgb = rgb(124, 38, 52);
pub(super) const TEE_ORANGE: Rgb = rgb(214, 118, 38);
pub(super) const TEE_CHARCOAL: Rgb = rgb(62, 62, 68);
pub(super) const TEE_TEAL: Rgb = rgb(30, 110, 116);
/// the manager's dress shirts
pub(super) const SHIRT_SKY: Rgb = rgb(186, 212, 240);
pub(super) const SHIRT_LILAC: Rgb = rgb(206, 194, 232);
pub(super) const SHIRT_BUTTER: Rgb = rgb(244, 232, 176);
/// aprons
pub(super) const CANVAS: Rgb = rgb(160, 120, 76);
pub(super) const APRON_BLACK: Rgb = rgb(34, 32, 36);
pub(super) const APRON_BURGUNDY: Rgb = rgb(122, 30, 50);
pub(super) const LEAF: Rgb = rgb(44, 140, 62);
pub(super) const LIME: Rgb = rgb(118, 176, 42);
pub(super) const FOREST: Rgb = rgb(26, 88, 50);
pub(super) const OLIVE: Rgb = rgb(104, 112, 40);
/// the cart wrangler's safety vest and its reflective bands
pub(super) const HI_VIS: Rgb = rgb(206, 242, 30);
pub(super) const REFLECT: Rgb = rgb(214, 220, 228);
/// the caps' and the visor's navy, a little lighter than the logo's ring
pub(super) const CAP_NAVY: Rgb = rgb(40, 56, 112);
pub(super) const HAIRNET: Rgb = rgb(220, 222, 230);
pub(super) const TROUSER_BLACK: Rgb = rgb(30, 30, 34);
pub(super) const TROUSER_DENIM: Rgb = rgb(50, 70, 110);
pub(super) const TROUSER_GREY: Rgb = rgb(78, 80, 88);
pub(super) const TROUSER_KHAKI: Rgb = rgb(170, 148, 106);
pub(super) const TROUSER_NAVY: Rgb = rgb(28, 34, 58);
/// the manager's suit: the logo ring's navy (the person, GR-T9: "the same
/// color as the company logos"), and the banana's yellow for the tie
pub(super) const BLAZER: Rgb = rgb(31, 44, 92);
pub(super) const TIE_YELLOW: Rgb = rgb(246, 212, 40);
pub(super) const BELT: Rgb = rgb(36, 28, 24);
pub(super) const BUCKLE: Rgb = rgb(200, 204, 212);
pub(super) const CUTTER: Rgb = rgb(248, 196, 30);
pub(super) const TIE_STRIPE: Rgb = rgb(236, 190, 52);
/// the patch each logo is sewn on
pub(super) const PATCH: Rgb = rgb(244, 246, 252);

/// Each kind's job in the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Job {
    Cashier,
    Stocker,
    Deli,
    Produce,
    Manager,
    Wrangler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Hat {
    /// bareheaded (the manager)
    None,
    /// a ball cap with the logo on its front
    Cap(Rgb),
    /// a visor with the logo on its band
    Visor(Rgb),
    /// a hairnet under a white paper cap (the deli)
    Hairnet,
}

/// One kind's uniform in one palette.
#[derive(Debug, Clone, Copy)]
pub(super) struct Kit {
    /// the job (named for the tests; the pieces below draw it)
    #[cfg_attr(not(test), allow(dead_code))]
    pub job: Job,
    pub shirt: Rgb,
    /// how far the sleeves reach (as `sleeves`: 100 the elbow, 200 the wrist)
    pub sleeve: i32,
    /// the shirt's hem, percent of the torso up from the hips (below 0: a
    /// long coat)
    pub hem: i32,
    /// a collar at the neck (polos, dress shirts, the deli coat)
    pub collar: bool,
    pub hat: Hat,
    /// a jacket over the shirt, open down the front (the manager's)
    pub jacket: Option<Rgb>,
    /// an open vest over the shirt
    pub vest: Option<Rgb>,
    /// the cart wrangler's safety vest
    pub hi_vis: bool,
    /// a bib apron, its colour and hem
    pub apron: Option<(Rgb, i32)>,
    pub tie: Option<Rgb>,
    pub name_tag: bool,
    pub belt: bool,
    /// a box cutter on the belt
    pub cutter: bool,
    pub trousers: Rgb,
}

/// `kind`'s job and uniform in `palette` (0 to `PALETTES` - 1; past that
/// it wraps): the chimpanzee the cashier (a polo, a navy vest with the
/// logo badge, a name tag, a visor in the polo's colour); the gibbon the stocker (a tee, a
/// canvas apron with the logo, a box cutter on the belt, a navy cap); the
/// proboscis monkey the deli worker (a white coat, a coloured apron with
/// the logo, a hairnet under a paper cap); the orangutan the produce clerk
/// (a cream shirt, a green apron and cap); the gorilla the manager (a dress
/// shirt and tie under a dark jacket, the logo badge, a name tag, a belt,
/// bareheaded); the lemur the cart wrangler (a tee under a hi-vis vest, a
/// navy cap). A palette picks the polo, the tee, the dress shirt or the
/// apron.
pub(super) fn kit(kind: Kind, palette: u8) -> Kit {
    let p = (palette % PALETTES) as usize;
    let base = Kit {
        job: Job::Cashier,
        shirt: WHITE,
        sleeve: 42,
        hem: 14,
        collar: false,
        hat: Hat::None,
        jacket: None,
        vest: None,
        hi_vis: false,
        apron: None,
        tie: None,
        name_tag: false,
        belt: false,
        cutter: false,
        trousers: TROUSER_BLACK,
    };
    match kind {
        Kind::Chimpanzee => {
            let polo = [BANNER_RED, POLO_TEAL, POLO_ROYAL, POLO_PURPLE][p];
            Kit {
                job: Job::Cashier,
                shirt: polo,
                collar: true,
                // (a visor in the polo's colour: a navy one over the navy vest
                // read as a police officer's)
                hat: Hat::Visor(shade(polo, 96)),
                vest: Some(NAVY),
                name_tag: true,
                ..base
            }
        }
        Kind::Gibbon => Kit {
            job: Job::Stocker,
            shirt: [TEE_HEATHER, TEE_FOREST, TEE_MAROON, TEE_ORANGE][p],
            hat: Hat::Cap(CAP_NAVY),
            apron: Some((CANVAS, 2)),
            belt: true,
            cutter: true,
            trousers: TROUSER_DENIM,
            ..base
        },
        Kind::Proboscis => Kit {
            job: Job::Deli,
            shirt: WHITE,
            sleeve: 120,
            hem: -16,
            collar: true,
            hat: Hat::Hairnet,
            apron: Some(([BANNER_RED, NAVY, APRON_BLACK, APRON_BURGUNDY][p], -6)),
            trousers: TROUSER_GREY,
            ..base
        },
        Kind::Orangutan => {
            let green = [LEAF, LIME, FOREST, OLIVE][p];
            Kit {
                job: Job::Produce,
                shirt: CREAM,
                hat: Hat::Cap(green),
                apron: Some((green, 2)),
                trousers: TROUSER_KHAKI,
                ..base
            }
        }
        Kind::Gorilla => Kit {
            job: Job::Manager,
            shirt: [WHITE, SHIRT_SKY, SHIRT_LILAC, SHIRT_BUTTER][p],
            sleeve: 150,
            collar: true,
            jacket: Some(BLAZER),
            tie: Some(TIE_YELLOW),
            name_tag: true,
            belt: true,
            trousers: BLAZER,
            ..base
        },
        Kind::Lemur => Kit {
            job: Job::Wrangler,
            shirt: [NAVY, TEE_CHARCOAL, BANNER_RED, TEE_TEAL][p],
            hat: Hat::Cap(CAP_NAVY),
            hi_vis: true,
            trousers: TROUSER_NAVY,
            ..base
        },
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

/// The store's logo at `at` on a surface facing `out`, upright toward `up`,
/// `r` round: a white patch a little wider than its ring, and the logo
/// itself (`grocerymodels::banana_logo`) over it.
fn logo(rig: &mut Rig, at: V3, up: V3, out: V3, r: i32) {
    let z = dir_or(out, V3::UP);
    let x = dir_or(up.cross(z), dir_or(V3::UP.cross(z), v3(ONE, 0, 0)));
    let y = z.cross(x).norm();
    let frame = Affine {
        c0: x,
        c1: y,
        c2: z,
        t: at.add(z.scale(r / 10)),
    };
    rig.put(
        Mesh::Cyl,
        frame
            .then(&Affine::translate(v3(0, 0, -r / 20)))
            .then(&Affine::rot_x(QUARTER))
            .then(&Affine::scale(v3(r * 23 / 10, r / 10, r * 23 / 10))),
        PATCH,
    );
    // (the badge is centred on its frame, about a quarter of its radius
    // deep: stood out by half that, all of it is in front of the patch)
    let badge = frame.then(&Affine::translate(v3(0, 0, r / 8)));
    crate::grocerymodels::banana_logo(rig.b, &rig.base.then(&badge), r);
}

/// Everything a store worker wears, over the ape `kind` posed as `s`.
pub(super) fn grocer(rig: &mut Rig, kind: Kind, palette: u8, l: &Look, m: &Dims, s: &Skel) {
    let k = kit(kind, palette);
    let d = cm(l.head);
    let h = &s.head;
    match k.hat {
        Hat::None => {}
        Hat::Cap(c) => cap(rig, d, h, c),
        Hat::Visor(c) => visor(rig, d, h, c),
        Hat::Hairnet => hairnet(rig, d, h),
    }
    let (wide, deep) = fit(kind);
    shirt(rig, kind, m, s, &k, wide, deep);
    // the outermost layer's depth, where the back's logo goes
    let mut outer = deep;
    if let Some(c) = k.jacket {
        jacket(rig, kind, m, s, wide + 6, deep + 6, c, k.shirt);
        outer = deep + 6;
    }
    if let Some(c) = k.vest {
        vest(rig, m, s, wide + 6, deep + 6, c, false);
        outer = deep + 6;
    }
    if k.hi_vis {
        vest(rig, m, s, wide + 8, deep + 8, HI_VIS, true);
        outer = deep + 8;
    }
    if let Some(c) = k.tie {
        tie(rig, m, s, outer, c);
    }
    if k.belt {
        belt(rig, m, s, wide, deep);
    }
    if k.cutter {
        cutter(rig, m, s, wide, deep);
    }
    let (w, t, dd) = (m.w, m.t, m.d);
    let body = &s.body;
    match k.apron {
        Some((c, hem)) => {
            let front = apron(rig, m, s, wide, deep, c, hem);
            logo(
                rig,
                body.at(0, pct(t, 66), pct(dd, front)),
                body.y,
                body.z,
                pct(w, 24),
            );
        }
        None => {
            // the logo badge on the chest, the name tag (if any) across
            // from it
            let z = outer / 2 + 1;
            logo(
                rig,
                body.at(pct(w, wide / 5), pct(t, 72), pct(dd, z)),
                body.y,
                body.z,
                pct(w, 15),
            );
        }
    }
    if k.name_tag {
        name_tag(rig, m, s, -wide / 5, outer);
    }
    // big on the back, where the chase camera sees it
    logo(
        rig,
        body.at(0, pct(t, 56), -pct(dd, outer / 2 + 1)),
        body.y,
        body.z.scale(-ONE),
        pct(w, 28),
    );
    trousers(rig, m, s, k.trousers);
}

/// The shirt or coat, its sleeves and its collar.
fn shirt(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, k: &Kit, wide: i32, deep: i32) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    garment(rig, m, s, k.hem, 104, wide, deep, k.shirt);
    if k.jacket.is_some() {
        // the jacket draws the sleeves (the shirt's cuffs show), the collar
        // and the chest
        return;
    }
    if kind == Kind::Gorilla {
        // over the gorilla's chest slabs, which the round shirt leaves out
        rig.put(
            Mesh::Bevel,
            body.shape(
                v3(0, pct(t, 70), pct(d, 38)),
                v3(pct(w, 98), pct(t, 34), pct(d, 42)),
            ),
            k.shirt,
        );
    }
    let cuff = if k.sleeve > 100 {
        shade(k.shirt, 90)
    } else {
        shade(k.shirt, 82)
    };
    sleeves(rig, m, s, k.sleeve, 150, k.shirt, cuff);
    if k.collar {
        for side in SIDES {
            let a = body.at(side * pct(w, 18), pct(t, 106), pct(d, deep / 2 - 14));
            let b = body.at(side * pct(w, 8), pct(t, 94), pct(d, deep / 2 + 2));
            rig.blade(a, b, body.x, pct(w, 16), pct(w, 4), shade(k.shirt, 90));
        }
    }
}

/// The manager's jacket over the shirt, below the hips to the shoulders,
/// cut full (over the gorilla's chest slabs too, as the captain's coat is:
/// GR-T9, filled in): sleeves nearly to the wrists with the shirt's cuffs
/// showing, a narrow opening down the front where the shirt shows, dark
/// lapels and the shirt's collar at the neck.
#[allow(clippy::too_many_arguments)]
fn jacket(rig: &mut Rig, kind: Kind, m: &Dims, s: &Skel, wide: i32, deep: i32, c: Rgb, shirt: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    let (wide, deep) = (wide.max(116), deep.max(130));
    garment(rig, m, s, -14, 104, wide, deep, c);
    if kind == Kind::Gorilla {
        rig.put(Mesh::Bevel, body.shape(at(0, 70, 44), at(108, 40, 50)), c);
    }
    sleeves(rig, m, s, 185, 170, c, shirt);
    rig.put(
        Mesh::Cube,
        body.shape(at(0, 74, deep / 2 + 1), at(14, 50, 3)),
        shirt,
    );
    for side in SIDES {
        let a = body.at(side * pct(w, 16), pct(t, 102), pct(d, deep / 2 + 1));
        let b = body.at(side * pct(w, 12), pct(t, 62), pct(d, deep / 2 + 2));
        rig.blade(a, b, body.x, pct(w, 14), pct(w, 4), shade(c, 76));
        let a = body.at(side * pct(w, 14), pct(t, 106), pct(d, deep / 2 - 10));
        let b = body.at(side * pct(w, 6), pct(t, 96), pct(d, deep / 2 + 3));
        rig.blade(a, b, body.x, pct(w, 14), pct(w, 4), shade(shirt, 92));
    }
}

/// An open vest over the shirt from the waist to the chest, straps over the
/// shoulders, open down the front; the safety vest with two reflective
/// bands round it.
#[allow(clippy::too_many_arguments)]
fn vest(rig: &mut Rig, m: &Dims, s: &Skel, wide: i32, deep: i32, c: Rgb, bands: bool) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    // (to 94, under the shirt's yoke: the two yokes would fight)
    garment(rig, m, s, 24, 94, wide, deep, c);
    for side in SIDES {
        rig.put(
            Mesh::Cube,
            body.shape(at(side * 22, 96, -4), at(16, 8, deep - 14)),
            c,
        );
    }
    rig.put(
        Mesh::Cube,
        body.shape(at(0, 60, deep / 2), at(10, 66, 3)),
        shade(c, 58),
    );
    if bands {
        for y in [44, 70] {
            rig.put(
                Mesh::Barrel,
                body.shape(at(0, y, 0), at(wide + 3, 7, deep + 3)),
                REFLECT,
            );
        }
    }
}

/// A tie from the collar down the chest, a knot at the top and a yellow
/// stripe across it.
fn tie(rig: &mut Rig, m: &Dims, s: &Skel, deep: i32, c: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let z = pct(d, deep / 2 + 3);
    let knot = body.at(0, pct(t, 96), z);
    rig.ball(knot, pct(w, 11), shade(c, 86));
    let top = body.at(0, pct(t, 92), z);
    let tip = body.at(0, pct(t, 44), z);
    rig.blade(top, tip, body.x, pct(w, 16), pct(w, 4), c);
    rig.put(
        Mesh::Cube,
        body.shape(
            v3(0, pct(t, 70), z + pct(d, 2)),
            v3(pct(w, 14), pct(t, 5), pct(d, 2)),
        ),
        TIE_STRIPE,
    );
}

/// A belt round the hips and its buckle.
fn belt(rig: &mut Rig, m: &Dims, s: &Skel, wide: i32, deep: i32) {
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    rig.put(
        Mesh::Barrel,
        s.body.shape(at(0, 18, 0), at(wide + 6, 8, deep + 6)),
        BELT,
    );
    rig.put(
        Mesh::Bevel,
        s.body.shape(at(0, 18, deep / 2 + 3), at(16, 10, 4)),
        BUCKLE,
    );
}

/// A yellow box cutter clipped to the belt at the right hip, its blade's
/// tip showing.
fn cutter(rig: &mut Rig, m: &Dims, s: &Skel, wide: i32, deep: i32) {
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    let x = wide / 2 - 4;
    rig.put(
        Mesh::Bevel,
        s.body.shape(at(x, 20, deep / 4), at(12, 30, 18)),
        CUTTER,
    );
    rig.put(
        Mesh::Cube,
        s.body.shape(at(x, 37, deep / 4 + 4), at(6, 6, 6)),
        BUCKLE,
    );
}

/// A bib apron down the front from the chest to `hem` (percent of the
/// torso up from the hips), a strap round the neck, ties round the waist
/// and a pocket. Returns its front's depth (percent of the torso's).
#[allow(clippy::too_many_arguments)]
fn apron(rig: &mut Rig, m: &Dims, s: &Skel, wide: i32, deep: i32, c: Rgb, hem: i32) -> i32 {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    let front = deep / 2 + 3;
    let (top, waist) = (92, 42);
    // the bib, and the skirt below the waist tie nearer in (the torso
    // narrows to the hips; one flat panel stood off them like a board)
    let skirt = front - 7;
    rig.put(
        Mesh::Cube,
        body.shape(
            at(0, (top + waist) / 2, front),
            at(wide * 7 / 10, top - waist, 4),
        ),
        c,
    );
    rig.put(
        Mesh::Cube,
        body.shape(
            at(0, (waist + hem) / 2, skirt),
            at(wide * 3 / 4, waist - hem + 4, 4),
        ),
        c,
    );
    rig.put(
        Mesh::Cube,
        body.shape(at(0, 24, skirt + 3), at(wide / 2, 14, 3)),
        shade(c, 80),
    );
    rig.put(
        Mesh::Barrel,
        body.shape(at(0, waist, 0), at(wide + 4, 6, deep + 4)),
        shade(c, 90),
    );
    for side in SIDES {
        let a = body.at(side * pct(w, wide * 3 / 10), pct(t, top), pct(d, front));
        let b = body.at(side * pct(w, 14), pct(t, 108), -pct(d, deep / 6));
        rig.put(Mesh::Hex, rod(a, b, pct(w, 6)), shade(c, 90));
    }
    front + 2
}

/// A white name tag with a navy stripe on the chest at `x` (percent of the
/// torso across), on a layer `z` percent of the torso deep.
fn name_tag(rig: &mut Rig, m: &Dims, s: &Skel, x: i32, z: i32) {
    let (w, t, d) = (m.w, m.t, m.d);
    let at = |x: i32, y: i32, z: i32| v3(pct(w, x), pct(t, y), pct(d, z));
    rig.put(
        Mesh::Cube,
        s.body.shape(at(x, 76, z / 2 + 1), at(20, 11, 3)),
        WHITE,
    );
    rig.put(
        Mesh::Cube,
        s.body.shape(at(x, 80, z / 2 + 2), at(20, 3, 3)),
        NAVY,
    );
}

/// Work trousers over each leg, hip to ankle (the feet stay bare).
fn trousers(rig: &mut Rig, m: &Dims, s: &Skel, c: Rgb) {
    let lw = m.leg_w;
    for (i, side) in SIDES.into_iter().enumerate() {
        let hip = hip_of(&s.body, m, side);
        let (knee, ankle) = bend(hip, s.feet[i], m.leg, s.leg_hint[i]);
        let cuff = knee.lerp(ankle, ONE * 80 / 100);
        rig.put(Mesh::Taper, rod(hip, knee, pct(lw, 156)), c);
        rig.ball(knee, pct(lw, 136), c);
        rig.put(Mesh::Taper, rod(knee, cuff, pct(lw, 134)), c);
    }
}

// ---------------------------------------------------------------------------
// Hats, kept low (the chase camera looks over them)

/// A ball cap with the logo on its crown's front.
fn cap(rig: &mut Rig, d: i32, h: &Frame, c: Rgb) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    rig.put(Mesh::Geo, h.shape(p(0, 38, -4), p(102, 62, 102)), c);
    rig.put(
        Mesh::Bevel,
        h.shape(p(0, 36, 54), p(76, 5, 54)),
        shade(c, 86),
    );
    rig.ball(h.at(0, pct(d, 69), -pct(d, 4)), pct(d, 10), shade(c, 80));
    logo(
        rig,
        h.at(0, pct(d, 52), pct(d, 40)),
        h.y,
        h.dir(0, 40, 100),
        pct(d, 18),
    );
}

/// A visor with the logo on the front of its band.
fn visor(rig: &mut Rig, d: i32, h: &Frame, c: Rgb) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    let band = h.at(0, pct(d, 30), -pct(d, 2));
    rig.put(
        Mesh::Hex,
        rod(band, band.add(h.y.scale(pct(d, 14))), pct(d, 106)),
        c,
    );
    rig.put(Mesh::Bevel, h.shape(p(0, 34, 60), p(86, 5, 52)), c);
    logo(rig, h.at(0, pct(d, 44), pct(d, 50)), h.y, h.z, pct(d, 16));
}

/// A hairnet over the top of the head and a white paper cap on it, the
/// logo on the cap's front.
fn hairnet(rig: &mut Rig, d: i32, h: &Frame) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    rig.put(Mesh::Geo, h.shape(p(0, 28, -10), p(106, 72, 100)), HAIRNET);
    rig.put(Mesh::Bevel, h.shape(p(0, 64, -8), p(46, 26, 92)), WHITE);
    logo(rig, h.at(0, pct(d, 64), pct(d, 38)), h.y, h.z, pct(d, 11));
    rig.put(
        Mesh::Hex,
        h.shape(p(0, 54, -8), p(58, 4, 100)),
        shade(WHITE, 88),
    );
}

#[cfg(test)]
mod tests {
    use super::super::food::tests::read;
    use super::super::tests::{at_rest, poses, triangles, Shape};
    use super::*;
    use crate::costume::Costume;
    use crate::map::Map;

    fn drawn_in(kind: Kind, palette: u8, pose: &Pose) -> Vec<Shape> {
        let mut b = Batch::new();
        draw_in(&mut b, kind, palette, pose);
        read(b)
    }

    /// One logo drawn alone (`grocerymodels::banana_logo`, the models
    /// agent's): its shapes, its colours and its triangles.
    fn one_logo() -> (usize, Vec<Rgb>, usize) {
        let mut b = Batch::new();
        crate::grocerymodels::banana_logo(&mut b, &Affine::translate(V3::ZERO), ONE);
        let s = read(b);
        let mut c: Vec<Rgb> = s.iter().map(|x| x.colour).collect();
        c.dedup();
        (s.len(), c, triangles(&s))
    }

    /// The logos `k` wears: the back's, the chest's or apron's, the hat's.
    fn logos(k: &Kit) -> usize {
        2 + (k.hat != Hat::None) as usize
    }

    /// The colours that mark each piece of a kit.
    fn kit_colours(k: &Kit) -> Vec<Rgb> {
        let mut c = vec![k.shirt, k.trousers, PATCH];
        match k.hat {
            Hat::None => {}
            Hat::Cap(x) => c.push(x),
            Hat::Visor(x) => c.push(x),
            Hat::Hairnet => c.extend([HAIRNET, WHITE]),
        }
        c.extend(k.vest);
        c.extend(k.jacket);
        if k.hi_vis {
            c.extend([HI_VIS, REFLECT]);
        }
        if let Some((a, _)) = k.apron {
            c.push(a);
        }
        if let Some(x) = k.tie {
            c.extend([x, TIE_STRIPE]);
        }
        if k.name_tag {
            c.extend([WHITE, NAVY]);
        }
        if k.belt {
            c.extend([BELT, BUCKLE]);
        }
        if k.cutter {
            c.push(CUTTER);
        }
        c
    }

    #[test]
    fn on_clean_up_on_aisle_ape_every_kind_works_its_own_job_with_the_logo_in_every_pose() {
        // the person, GR-T1: "make the ape costumes to be workers uniforms
        // of the store, with a big banana in a circle as a logo"
        assert_eq!(Gear::from_key("grocer"), Some(Gear::Grocer));
        for map in Map::ALL {
            assert_eq!(
                Gear::on(map) == Gear::Grocer,
                map.key() == "grocery",
                "{map:?}"
            );
        }
        // six jobs, one each
        let jobs: Vec<Job> = Kind::ALL.iter().map(|k| kit(*k, 0).job).collect();
        for (i, a) in jobs.iter().enumerate() {
            assert!(!jobs[i + 1..].contains(a), "{a:?} twice");
        }
        // most wear the logo on a cap or visor
        let hats = Kind::ALL
            .iter()
            .filter(|k| logos(&kit(**k, 0)) == 3)
            .count();
        assert!(hats >= 4, "{hats} logo hats");
        assert!(Kind::ALL.iter().all(|k| logos(&kit(*k, 0)) >= 2));
        let (per_logo, logo_colours, _) = one_logo();
        for kind in Kind::ALL {
            let l = look(kind);
            // four palettes, each its own shirt or apron
            let mut seen: Vec<(Rgb, Option<(Rgb, i32)>)> = Vec::new();
            for palette in 0..PALETTES {
                let k = kit(kind, palette);
                assert!(!seen.contains(&(k.shirt, k.apron)), "{kind:?} {palette}");
                seen.push((k.shirt, k.apron));
            }
            for palette in 0..PALETTES {
                let k = kit(kind, palette);
                for p in poses() {
                    let pose = Pose {
                        gear: Gear::Grocer,
                        ..p
                    };
                    let s = drawn_in(kind, palette, &pose);
                    let bare = drawn_in(kind, palette, &p);
                    assert!(s.len() > bare.len() + 20, "{kind:?}");
                    assert!((50..=300).contains(&s.len()), "{kind:?} {} shapes", s.len());
                    let has = |c: Rgb| s.iter().any(|x| x.colour == c);
                    for c in kit_colours(&k).into_iter().chain(logo_colours.clone()) {
                        assert!(has(c), "{kind:?} {palette} {pose:?}: no {c:?}");
                    }
                    // the logos: a patch each, and every logo's shapes
                    let patches = s.iter().filter(|x| x.colour == PATCH).count();
                    assert_eq!(patches, logos(&k), "{kind:?}: {patches} patches");
                    let extra = s.len() - bare.len();
                    assert!(extra > logos(&k) * (per_logo + 1), "{kind:?}");
                    // the bare ape's own body still drawn: its own size and
                    // fur (no food recolouring)
                    let lp = look_in(kind, palette);
                    assert!(has(lp.fur) && has(lp.face), "{kind:?}: not the ape");
                    for c in [
                        cowboy::HAT_SAND,
                        cowboy::GUNBELT,
                        JACKET,
                        KHAKI,
                        PITH,
                        TRICORN,
                    ] {
                        assert!(!has(c), "{kind:?}: {c:?}");
                    }
                    for x in &s {
                        for a in x.axes {
                            assert!(
                                // (the logo's ring is a twentieth of a
                                // small badge's radius thick)
                                a.len() >= ONE / 1000 && a.len() <= int(2),
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
                        // the hat sits on the head: over the face, within
                        // a head of it
                        let face = s.iter().find(|x| x.colour == lp.face).unwrap().at;
                        let hat = match k.hat {
                            Hat::Cap(colour) | Hat::Visor(colour) => Some(colour),
                            Hat::Hairnet => Some(HAIRNET),
                            Hat::None => None,
                        };
                        if let Some(c) = hat {
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
                    if pose.rope.is_some() {
                        let own: Vec<Rgb> = kit_colours(&k)
                            .into_iter()
                            .chain(logo_colours.clone())
                            .filter(|c| *c != lp.skin && *c != lp.fur && *c != lp.limb)
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
    }

    #[test]
    fn the_logo_on_the_back_faces_out_behind_the_ape() {
        // the chase camera sees the back: the biggest logo is there,
        // behind the body's middle and facing away from the belly
        let (_, logo_colours, _) = one_logo();
        for kind in Kind::ALL {
            let pose = Pose {
                gear: Gear::Grocer,
                ..at_rest()
            };
            let s = drawn_in(kind, 0, &pose);
            let bare = drawn_in(kind, 0, &at_rest());
            let patches: Vec<&Shape> = s.iter().filter(|x| x.colour == PATCH).collect();
            let biggest = patches.iter().max_by_key(|x| x.axes[0].len()).unwrap();
            // the ape faces its yaw: its back is the other way
            let fwd = facing(pose.yaw, 0);
            let middle = bare
                .iter()
                .fold(V3::ZERO, |a, x| a.add(x.at.scale(ONE / bare.len() as i32)));
            assert!(
                biggest.at.sub(middle).dot(fwd) < 0,
                "{kind:?}: the big logo is not on the back"
            );
            // and it faces out: the logo's own shapes on the patch (in the
            // colours only the logo uses) stand behind it, away from the ape
            let r = biggest.axes[0].len();
            let kit = kit_colours(&kit(kind, 0));
            let own: Vec<Rgb> = logo_colours
                .iter()
                .filter(|c| !kit.contains(c) && !bare.iter().any(|x| x.colour == **c))
                .copied()
                .collect();
            let on: Vec<&Shape> = s
                .iter()
                .filter(|x| x.colour != PATCH && x.at.dist(biggest.at) < r)
                .filter(|x| own.contains(&x.colour))
                .collect();
            assert!(!on.is_empty(), "{kind:?}: no logo on the back's patch");
            // (out along the patch's own face, away from the body: on a
            // sloping back the logo's parts stand off the patch that way)
            let n = biggest.axes[1].norm();
            let n = if biggest.at.sub(middle).dot(n) < 0 {
                n.neg()
            } else {
                n
            };
            for x in on {
                assert!(
                    x.at.sub(biggest.at).dot(n) > 0,
                    "{kind:?}: the back's logo faces into the ape"
                );
            }
        }
    }

    #[test]
    fn the_store_uniforms_cost_about_what_the_cowboy_outfits_do() {
        // the clothes (and the logos' patches) are held to the cowboys'
        // budget; the logo itself is the models agent's, counted apart,
        // at most three to an ape
        let (_, _, logo_tris) = one_logo();
        let mut lines = Vec::new();
        let (mut grocer_all, mut cowboy_all) = (0, 0);
        for kind in Kind::ALL {
            let p = at_rest();
            let tris = |g: Gear| triangles(&drawn_in(kind, 0, &Pose { gear: g, ..p }));
            let bare = tris(Gear::None);
            let k = kit(kind, 0);
            assert!(logos(&k) <= 3);
            let logo = logos(&k) * logo_tris;
            let grocer = tris(Gear::Grocer) - bare - logo;
            let cowboy = tris(Gear::Cowboy) - bare;
            lines.push(format!(
                "{kind:?}: bare {bare}, cowboy +{cowboy}, grocer +{grocer} (+{logo} logos)"
            ));
            grocer_all += grocer;
            cowboy_all += cowboy;
        }
        let table = lines.join("; ");
        assert!(grocer_all <= cowboy_all * 5 / 4, "{table}");
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

    #[test]
    fn the_store_uniform_is_a_costume_unlocked_on_its_map() {
        assert!(Costume::ALL.contains(&Costume::Grocer));
        assert_eq!(Costume::from_key("grocer"), Some(Costume::Grocer));
        assert_eq!(Costume::Grocer.key(), "grocer");
        assert_eq!(Gear::from_key(Costume::Grocer.key()), Some(Gear::Grocer));
        assert_eq!(Costume::of_map(Map::Grocery), Some(Costume::Grocer));
        assert_eq!(Costume::Grocer.map(), Some(Map::Grocery));
        assert!(!Costume::Grocer.free());
        for map in Map::ALL {
            assert_eq!(Costume::Grocer.gear(map), Gear::Grocer);
        }
    }
}
