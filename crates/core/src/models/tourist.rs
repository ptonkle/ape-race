//! Concrete Jungle's tourist outfits (`Gear::Tourist`, 2026-10-15). The
//! person: "these should be all touristy, so like I Heart NY and such", and
//! "maybe some more different shirts". Each kind its own mix and its own
//! shirt (`kit`): an I ♥ NY tee, a loud Hawaiian shirt, a Liberty-green
//! striped tee, a red-apple tee, an NY hoodie, a taxi-chequer shirt, with
//! caps, a bucket hat, a sun visor, a foam Liberty crown, big sunglasses, a
//! camera on a strap, bum bags, a lanyard, souvenir bags on the forearm and
//! socks with sandals. Nothing is held in the hands, which grip the vines;
//! the hats stay low so the chase camera sees over them.

use super::*;

/// The shirts.
const TEE_WHITE: Rgb = rgb(250, 250, 248);
const PRINT_BLACK: Rgb = rgb(24, 24, 28);
const HEART_RED: Rgb = rgb(226, 24, 48);
const ALOHA_ORANGE: Rgb = rgb(250, 120, 30);
const ALOHA_FLOWER: Rgb = rgb(255, 236, 80);
const LIBERTY_GREEN: Rgb = rgb(92, 186, 152);
const STRIPE_WHITE: Rgb = rgb(236, 248, 242);
const APPLE_TEE: Rgb = rgb(34, 40, 64);
const APPLE_RED: Rgb = rgb(214, 30, 36);
const APPLE_LEAF: Rgb = rgb(70, 170, 50);
const STEM: Rgb = rgb(96, 60, 30);
const HOODIE_RED: Rgb = rgb(190, 26, 40);
const HOODIE_PRINT: Rgb = rgb(255, 255, 255);
const TAXI_YELLOW: Rgb = rgb(255, 204, 0);
const CHECK_BLACK: Rgb = rgb(20, 20, 20);
const CHECK_WHITE: Rgb = rgb(244, 244, 244);
/// The hats.
const CAP_RED: Rgb = rgb(208, 34, 40);
const CAP_NAVY: Rgb = rgb(28, 44, 110);
const BUCKET_CREAM: Rgb = rgb(238, 226, 190);
const VISOR_PINK: Rgb = rgb(255, 80, 170);
const CROWN_GREEN: Rgb = rgb(110, 200, 160);
/// The sunglasses: big round dark lenses in a white or pink frame.
const BIG_LENS: Rgb = rgb(40, 26, 44);
const FRAME_WHITE: Rgb = rgb(252, 252, 252);
const FRAME_PINK: Rgb = rgb(255, 120, 200);
/// The camera, the bum bags, the lanyard and its badge, the bags, the
/// socks and sandals.
const CAMERA_BODY: Rgb = rgb(30, 30, 34);
const CAMERA_LENS: Rgb = rgb(150, 160, 176);
const STRAP: Rgb = rgb(60, 50, 44);
const BUMBAG_BLACK: Rgb = rgb(36, 34, 40);
const BUMBAG_NEON: Rgb = rgb(150, 255, 60);
const LANYARD_BLUE: Rgb = rgb(30, 110, 230);
const BADGE_WHITE: Rgb = rgb(248, 248, 255);
const BAG_WHITE: Rgb = rgb(248, 246, 240);
const BAG_KRAFT: Rgb = rgb(196, 150, 96);
const BAG_HEART: Rgb = rgb(236, 40, 70);
const SOCK_WHITE: Rgb = rgb(245, 245, 245);
const SANDAL: Rgb = rgb(110, 70, 40);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Hat {
    None,
    Cap(Rgb),
    Bucket(Rgb),
    Visor(Rgb),
    /// a foam Statue of Liberty crown, seven green spikes
    Crown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Shirt {
    /// white, I ♥ NY in big shapes on the front and the back
    HeartTee,
    /// orange with yellow flowers all over
    Hawaiian,
    /// Liberty green with white stripes
    LibertyStripe,
    /// navy with a big red apple front and back
    AppleTee,
    /// a red hoodie, NY on the front and the back, the hood down
    Hoodie,
    /// yellow with a band of black and white chequers
    TaxiCheck,
}

/// One kind's tourist pieces.
#[derive(Debug, Clone, Copy)]
pub(super) struct Kit {
    pub(super) hat: Hat,
    pub(super) shirt: Shirt,
    /// the sunglasses' frame colour
    pub(super) shades: Option<Rgb>,
    pub(super) camera: bool,
    pub(super) bumbag: Option<Rgb>,
    pub(super) lanyard: bool,
    /// the forearm with a souvenir bag (-1 or 1) and the bag's colour
    pub(super) bag: Option<(i32, Rgb)>,
    pub(super) socks: bool,
}

/// Each kind's tourist mix: the gorilla in the I ♥ NY tee, a red cap, a
/// black bum bag and an I ♥ NY bag; the chimpanzee in a Hawaiian shirt with
/// white sunglasses and a camera; the orangutan as Liberty (foam crown,
/// green striped tee, socks and sandals); the gibbon in a red-apple tee,
/// a bucket hat, a camera and a paper bag; the proboscis monkey in an NY
/// hoodie, a pink visor and pink sunglasses, a neon bum bag, socks and
/// sandals; the lemur in a taxi-chequer shirt, a navy cap and a lanyard.
pub(super) fn kit(kind: Kind) -> Kit {
    let bare = Kit {
        hat: Hat::None,
        shirt: Shirt::HeartTee,
        shades: None,
        camera: false,
        bumbag: None,
        lanyard: false,
        bag: None,
        socks: false,
    };
    match kind {
        Kind::Gorilla => Kit {
            hat: Hat::Cap(CAP_RED),
            bumbag: Some(BUMBAG_BLACK),
            bag: Some((-1, BAG_WHITE)),
            ..bare
        },
        Kind::Chimpanzee => Kit {
            shirt: Shirt::Hawaiian,
            shades: Some(FRAME_WHITE),
            camera: true,
            ..bare
        },
        Kind::Orangutan => Kit {
            hat: Hat::Crown,
            shirt: Shirt::LibertyStripe,
            socks: true,
            ..bare
        },
        Kind::Gibbon => Kit {
            hat: Hat::Bucket(BUCKET_CREAM),
            shirt: Shirt::AppleTee,
            camera: true,
            bag: Some((1, BAG_KRAFT)),
            ..bare
        },
        Kind::Proboscis => Kit {
            hat: Hat::Visor(VISOR_PINK),
            shirt: Shirt::Hoodie,
            shades: Some(FRAME_PINK),
            bumbag: Some(BUMBAG_NEON),
            socks: true,
            ..bare
        },
        Kind::Lemur => Kit {
            hat: Hat::Cap(CAP_NAVY),
            shirt: Shirt::TaxiCheck,
            lanyard: true,
            ..bare
        },
    }
}

/// How roomy a shirt is for the kind: percent of the torso across and deep
/// (the gorilla's chest slabs, the proboscis monkey's pot belly).
fn fit(kind: Kind) -> (i32, i32) {
    match kind {
        Kind::Gorilla => (120, 142),
        Kind::Proboscis => (128, 160),
        _ => (112, 114),
    }
}

/// Everything a tourist wears but the socks and sandals' place in `limbs`
/// (drawn here too: they only cover).
pub(super) fn tourist(rig: &mut Rig, kind: Kind, l: &Look, m: &Dims, s: &Skel) {
    let k = kit(kind);
    let d = cm(l.head);
    let h = &s.head;
    match k.hat {
        Hat::None => {}
        Hat::Cap(c) => cap(rig, d, h, c),
        Hat::Bucket(c) => bucket(rig, d, h, c),
        Hat::Visor(c) => visor(rig, d, h, c),
        Hat::Crown => crown(rig, d, h),
    }
    if let Some(c) = k.shades {
        big_shades(rig, kind, l, h, c);
    }
    let (wide, deep) = fit(kind);
    shirt(rig, m, s, k.shirt, wide, deep, kind == Kind::Gorilla);
    if let Some(c) = k.bumbag {
        bumbag(rig, m, s, wide, deep, c);
    }
    if k.camera {
        camera(rig, m, s, deep);
    }
    if k.lanyard {
        lanyard(rig, m, s, deep);
    }
    if let Some((side, c)) = k.bag {
        bag(rig, m, s, side, c);
    }
    if k.socks {
        socks_and_sandals(rig, m, s);
    }
}

// ---------------------------------------------------------------------------
// Hats, kept low (the chase camera looks over them)

fn cap(rig: &mut Rig, d: i32, h: &Frame, c: Rgb) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    rig.put(Mesh::Geo, h.shape(p(0, 38, -4), p(102, 62, 102)), c);
    rig.put(
        Mesh::Bevel,
        h.shape(p(0, 36, 54), p(76, 5, 54)),
        shade(c, 86),
    );
    rig.ball(h.at(0, pct(d, 69), -pct(d, 4)), pct(d, 10), shade(c, 80));
}

fn bucket(rig: &mut Rig, d: i32, h: &Frame, c: Rgb) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    rig.put(Mesh::Geo, h.shape(p(0, 40, -4), p(98, 58, 98)), c);
    let floor = h.at(0, pct(d, 28), -pct(d, 4));
    rig.put(
        Mesh::Taper,
        rod(floor.sub(h.y.scale(pct(d, 6))), floor, pct(d, 142)),
        shade(c, 92),
    );
    rig.put(
        Mesh::Hex,
        rod(floor, floor.add(h.y.scale(pct(d, 8))), pct(d, 102)),
        shade(c, 70),
    );
}

fn visor(rig: &mut Rig, d: i32, h: &Frame, c: Rgb) {
    let p = |x: i32, y: i32, z: i32| v3(pct(d, x), pct(d, y), pct(d, z));
    let band = h.at(0, pct(d, 30), -pct(d, 2));
    rig.put(
        Mesh::Hex,
        rod(band, band.add(h.y.scale(pct(d, 12))), pct(d, 106)),
        c,
    );
    rig.put(Mesh::Bevel, h.shape(p(0, 34, 60), p(86, 5, 52)), c);
}

/// A foam Liberty crown: a band and seven spikes fanned round the front.
fn crown(rig: &mut Rig, d: i32, h: &Frame) {
    let band = h.at(0, pct(d, 28), -pct(d, 4));
    rig.put(
        Mesh::Hex,
        rod(band, band.add(h.y.scale(pct(d, 18))), pct(d, 108)),
        CROWN_GREEN,
    );
    let r = pct(d, 50);
    for k in -3..=3 {
        let a = deg(26 * k);
        let out = h.dir(mul(fx::sin(a), ONE), 0, mul(fx::cos(a), ONE));
        let base = h.at(0, pct(d, 40), -pct(d, 4)).add(out.scale(r));
        let way = dir_or(out.scale(ONE / 2).add(h.y), h.y);
        rig.put(
            Mesh::Spike,
            rod(base, base.add(way.scale(pct(d, 40))), pct(d, 15)),
            shade(CROWN_GREEN, 96 - 3 * k.abs()),
        );
    }
}

/// Big round sunglasses: a dark disc over each eye in a coloured rim, a
/// bridge and arms back to the ears.
fn big_shades(rig: &mut Rig, kind: Kind, l: &Look, h: &Frame, c: Rgb) {
    let d = cm(l.head);
    let (ey, ex) = eye_spot(kind);
    let z = (42 + l.eye / 2 + 5).max(38 + (l.brow + 14) / 2 + 3);
    let lw = (l.eye + 18).min(2 * ex - 2);
    for side in SIDES {
        let e = h.at(side * pct(d, ex), pct(d, ey), pct(d, z));
        rig.put(
            Mesh::Hex,
            rod(e.sub(h.z.scale(pct(d, 2))), e, pct(d, lw + 6)),
            c,
        );
        rig.put(
            Mesh::Hex,
            rod(e, e.add(h.z.scale(pct(d, 2))), pct(d, lw)),
            BIG_LENS,
        );
        let a = h.at(side * pct(d, ex + lw / 2), pct(d, ey), pct(d, z - 2));
        let b = h.at(side * pct(d, 52), pct(d, ey + 2), 0);
        rig.put(Mesh::Hex, rod(a, b, pct(d, 4)), c);
    }
    rig.put(
        Mesh::Cube,
        h.shape(
            v3(0, pct(d, ey + 2), pct(d, z)),
            v3(pct(d, 2 * ex - lw), pct(d, 4), pct(d, 3)),
        ),
        c,
    );
}

// ---------------------------------------------------------------------------
// The shirts

/// A flat stroke on the shirt's front (`face` 1) or back (-1) from (`u0`,
/// `v0`) to (`u1`, `v1`): u percent of the torso to the viewer's right, v
/// percent up; `width` percent of the torso across, on a shirt `wide` and
/// `deep` (it follows the curve round the sides).
#[allow(clippy::too_many_arguments)]
fn stroke(
    rig: &mut Rig,
    m: &Dims,
    s: &Skel,
    (wide, deep, face): (i32, i32, i32),
    (u0, v0): (i32, i32),
    (u1, v1): (i32, i32),
    width: i32,
    c: Rgb,
) {
    let at = |u: i32, v: i32| on_shirt(m, s, wide, deep, face, u, v);
    let (a, b) = (at(u0, v0), at(u1, v1));
    let along = b.sub(a);
    let normal = s.body.z.scale(face * ONE);
    let across = dir_or(normal.cross(along), s.body.x);
    let len = if along.len() < pct(m.w, width) {
        // a dot: a square
        across.cross(normal).scale(pct(m.w, width))
    } else {
        along
    };
    rig.put(
        Mesh::Cube,
        Affine {
            c0: across.scale(pct(m.w, width)),
            c1: len,
            c2: normal.scale(pct(m.d, 3)),
            t: a.lerp(b, ONE / 2),
        },
        c,
    );
}

/// The point on a shirt's surface at (`u`, `v`) seen from the front
/// (`face` 1) or the back (-1).
fn on_shirt(m: &Dims, s: &Skel, wide: i32, deep: i32, face: i32, u: i32, v: i32) -> V3 {
    // the viewer's right is the ape's +x from the front, -x from behind
    let x = face * u;
    let half = wide / 2;
    let z = deep / 2 - deep * x * x / (4 * half * half) + 1;
    s.body
        .at(pct(m.w, x), pct(m.t, v), face * pct(m.d, z.max(0)))
}

/// A heart at (`u`, `v`), `size` percent of the torso across.
#[allow(clippy::too_many_arguments)]
fn heart(
    rig: &mut Rig,
    m: &Dims,
    s: &Skel,
    shape: (i32, i32, i32),
    u: i32,
    v: i32,
    size: i32,
    c: Rgb,
) {
    let (wide, deep, face) = shape;
    let normal = s.body.z.scale(face * ONE);
    let k = pct(m.w, size);
    for x in SIDES {
        let at = on_shirt(m, s, wide, deep, face, u + x * size / 4, v + size / 5);
        rig.put(
            Mesh::Geo,
            Affine {
                c0: s.body.x.scale(pct(k, 58)),
                c1: s.body.y.scale(pct(k, 58)),
                c2: normal.scale(pct(m.d, 4)),
                t: at,
            },
            c,
        );
    }
    let at = on_shirt(m, s, wide, deep, face, u, v - size / 8);
    // a diamond, wider than tall, its point down
    rig.put(
        Mesh::Octa,
        Affine {
            c0: s.body.x.scale(pct(k, 104)),
            c1: s.body.y.scale(pct(k, 90)),
            c2: normal.scale(pct(m.d, 4)),
            t: at,
        },
        c,
    );
}

/// The letters N and Y side by side, centred at (0, `v`), `tall` percent
/// of the torso high.
fn ny(rig: &mut Rig, m: &Dims, s: &Skel, shape: (i32, i32, i32), v: i32, tall: i32, c: Rgb) {
    let (top, low) = (v + tall / 2, v - tall / 2);
    let w = tall * 3 / 4;
    let st = 6;
    // N
    let (l, r) = (-w - 3, -3);
    stroke(rig, m, s, shape, (l, low), (l, top), st, c);
    stroke(rig, m, s, shape, (r, low), (r, top), st, c);
    stroke(rig, m, s, shape, (l, top), (r, low), st, c);
    // Y
    let (l, r) = (3, w + 3);
    let mid = (l + r) / 2;
    stroke(rig, m, s, shape, (l, top), (mid, v), st, c);
    stroke(rig, m, s, shape, (r, top), (mid, v), st, c);
    stroke(rig, m, s, shape, (mid, v), (mid, low), st, c);
}

#[allow(clippy::too_many_arguments)]
fn shirt(rig: &mut Rig, m: &Dims, s: &Skel, kind: Shirt, wide: i32, deep: i32, slabs: bool) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let colour = match kind {
        Shirt::HeartTee => TEE_WHITE,
        Shirt::Hawaiian => ALOHA_ORANGE,
        Shirt::LibertyStripe => LIBERTY_GREEN,
        Shirt::AppleTee => APPLE_TEE,
        Shirt::Hoodie => HOODIE_RED,
        Shirt::TaxiCheck => TAXI_YELLOW,
    };
    garment(rig, m, s, 24, 104, wide, deep, colour);
    if slabs {
        // over the gorilla's chest slabs, which the round shirt leaves out
        rig.put(
            Mesh::Bevel,
            body.shape(
                v3(0, pct(t, 70), pct(d, 38)),
                v3(pct(w, 98), pct(t, 34), pct(d, 42)),
            ),
            colour,
        );
    }
    let (to, puff) = match kind {
        Shirt::Hoodie => (196, 150),
        Shirt::Hawaiian => (60, 166),
        _ => (42, 150),
    };
    sleeves(rig, m, s, to, puff, colour, shade(colour, 86));
    for face in SIDES {
        let shape = (wide, deep, face);
        match kind {
            Shirt::HeartTee => {
                // I ♥ over NY, as the souvenir shirt has it
                stroke(rig, m, s, shape, (-16, 66), (-16, 90), 7, PRINT_BLACK);
                heart(rig, m, s, shape, 10, 78, 28, HEART_RED);
                ny(rig, m, s, shape, 50, 22, PRINT_BLACK);
            }
            Shirt::Hawaiian => {
                for (u, v) in [(-28, 80), (4, 88), (26, 70), (-12, 58), (18, 44), (-26, 38)] {
                    let at = on_shirt(m, s, wide, deep, face, u, v);
                    rig.ball(at, pct(w, 15), ALOHA_FLOWER);
                    rig.ball(at.add(body.z.scale(face * pct(d, 4))), pct(w, 6), HEART_RED);
                }
            }
            Shirt::LibertyStripe => {}
            Shirt::AppleTee => {
                let at = on_shirt(m, s, wide, deep, face, 0, 68);
                rig.put(
                    Mesh::Geo,
                    Affine {
                        c0: body.x.scale(pct(w, 44)),
                        c1: body.y.scale(pct(t, 34)),
                        c2: body.z.scale(pct(d, 8)),
                        t: at,
                    },
                    APPLE_RED,
                );
                stroke(rig, m, s, shape, (0, 82), (2, 92), 5, STEM);
                stroke(rig, m, s, shape, (4, 88), (16, 94), 9, APPLE_LEAF);
            }
            Shirt::Hoodie => {
                ny(rig, m, s, shape, 66, 26, HOODIE_PRINT);
            }
            Shirt::TaxiCheck => {
                for (row, v) in [(0, 62), (1, 70)] {
                    for i in -3..3 {
                        let c = if (i + row) % 2 == 0 {
                            CHECK_BLACK
                        } else {
                            CHECK_WHITE
                        };
                        let u = i * 10 + 5;
                        stroke(rig, m, s, shape, (u - 4, v), (u + 4, v), 8, c);
                    }
                }
            }
        }
    }
    match kind {
        Shirt::LibertyStripe => {
            for y in [42, 60, 78] {
                rig.put(
                    Mesh::Barrel,
                    body.shape(
                        v3(0, pct(t, y), 0),
                        v3(pct(w, wide + 5), pct(t, 7), pct(d, deep + 5)),
                    ),
                    STRIPE_WHITE,
                );
            }
        }
        Shirt::Hoodie => {
            // the hood down behind the neck, and the drawstrings
            rig.put(
                Mesh::Geo,
                body.shape(
                    v3(0, pct(t, 98), -pct(d, deep / 2 - 6)),
                    v3(pct(w, 70), pct(t, 26), pct(d, 44)),
                ),
                shade(HOODIE_RED, 84),
            );
            for side in SIDES {
                let a = body.at(side * pct(w, 10), pct(t, 98), pct(d, deep / 2 - 6));
                let b = body.at(side * pct(w, 12), pct(t, 80), pct(d, deep / 2 + 2));
                rig.put(Mesh::Hex, rod(a, b, pct(w, 3)), HOODIE_PRINT);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Things carried, none in the hands

/// A bum bag on a belt round the hips, the pouch in front.
fn bumbag(rig: &mut Rig, m: &Dims, s: &Skel, wide: i32, deep: i32, c: Rgb) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    rig.put(
        Mesh::Barrel,
        body.shape(
            v3(0, pct(t, 24), 0),
            v3(pct(w, wide + 6), pct(t, 6), pct(d, deep + 6)),
        ),
        BUMBAG_BLACK,
    );
    rig.put(
        Mesh::Bevel,
        body.shape(
            v3(0, pct(t, 22), pct(d, deep / 2 + 8)),
            v3(pct(w, 52), pct(t, 18), pct(d, 22)),
        ),
        c,
    );
}

/// A camera on a strap round the neck, low on the belly.
fn camera(rig: &mut Rig, m: &Dims, s: &Skel, deep: i32) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let at = body.at(0, pct(t, 44), pct(d, deep / 2 + 10));
    for side in SIDES {
        let neck = body.at(side * pct(w, 22), pct(t, 102), 0);
        let corner = at.add(body.x.scale(side * pct(w, 18)));
        rig.put(Mesh::Hex, rod(neck, corner, pct(w, 4)), STRAP);
    }
    rig.put(
        Mesh::Bevel,
        body.shape(
            v3(0, pct(t, 44), pct(d, deep / 2 + 10)),
            v3(pct(w, 40), pct(t, 18), pct(d, 16)),
        ),
        CAMERA_BODY,
    );
    rig.put(
        Mesh::Hex,
        rod(at, at.add(body.z.scale(pct(d, 22))), pct(w, 20)),
        CAMERA_LENS,
    );
}

/// A blue lanyard round the neck with a white badge on the chest.
fn lanyard(rig: &mut Rig, m: &Dims, s: &Skel, deep: i32) {
    let body = &s.body;
    let (w, t, d) = (m.w, m.t, m.d);
    let badge = body.at(0, pct(t, 82), pct(d, deep / 2 + 3));
    for side in SIDES {
        let neck = body.at(side * pct(w, 18), pct(t, 104), pct(d, 10));
        rig.put(Mesh::Hex, rod(neck, badge, pct(w, 4)), LANYARD_BLUE);
    }
    rig.put(
        Mesh::Cube,
        body.shape(
            v3(0, pct(t, 76), pct(d, deep / 2 + 3)),
            v3(pct(w, 20), pct(t, 14), pct(d, 3)),
        ),
        BADGE_WHITE,
    );
}

/// A souvenir bag hanging from the forearm on `side` (the hand stays free),
/// with a red heart on its sides.
fn bag(rig: &mut Rig, m: &Dims, s: &Skel, side: i32, c: Rgb) {
    let i = if side < 0 { 0 } else { 1 };
    let sh = shoulder_of(&s.body, m, side);
    let (elbow, wrist) = bend(sh, s.hands[i], m.arm, s.arm_hint[i]);
    let hang = elbow.lerp(wrist, ONE * 30 / 100);
    let k = m.w;
    let f = Frame::upright(hang.sub(V3::UP.scale(pct(k, 30))), V3::UP, s.body.z);
    for x in SIDES {
        rig.put(
            Mesh::Hex,
            rod(hang, f.at(x * pct(k, 12), 0, 0), pct(k, 3)),
            shade(c, 70),
        );
    }
    let size = v3(pct(k, 42), pct(k, 46), pct(k, 18));
    rig.put(Mesh::Cube, f.shape(v3(0, -pct(k, 23), 0), size), c);
    for face in SIDES {
        let mid = f.at(0, -pct(k, 22), face * pct(k, 10));
        for x in SIDES {
            rig.put(
                Mesh::Ico,
                Affine {
                    c0: f.x.scale(pct(k, 13)),
                    c1: f.y.scale(pct(k, 13)),
                    c2: f.z.scale(pct(k, 2)),
                    t: mid.add(f.x.scale(x * pct(k, 6))).add(f.y.scale(pct(k, 4))),
                },
                BAG_HEART,
            );
        }
        rig.put(
            Mesh::Octa,
            Affine {
                c0: f.x.scale(pct(k, 24)),
                c1: f.y.scale(pct(k, 20)),
                c2: f.z.scale(pct(k, 2)),
                t: mid.sub(f.y.scale(pct(k, 3))),
            },
            BAG_HEART,
        );
    }
}

/// White socks pulled up the shins and brown sandals under the feet.
fn socks_and_sandals(rig: &mut Rig, m: &Dims, s: &Skel) {
    let w = m.leg_w;
    for (i, side) in SIDES.into_iter().enumerate() {
        let hip = hip_of(&s.body, m, side);
        let (knee, ankle) = bend(hip, s.feet[i], m.leg, s.leg_hint[i]);
        rig.put(
            Mesh::Hex,
            rod(ankle, ankle.lerp(knee, ONE * 45 / 100), pct(w, 122)),
            SOCK_WHITE,
        );
        let f = Frame {
            o: ankle,
            ..s.feet_frame
        };
        rig.put(
            Mesh::Cube,
            f.shape(
                v3(0, -pct(w, 40), pct(w, 44)),
                v3(pct(w, 150), pct(w, 14), pct(w, 210)),
            ),
            SANDAL,
        );
        rig.put(
            Mesh::Cube,
            f.shape(
                v3(0, pct(w, 22), pct(w, 70)),
                v3(pct(w, 140), pct(w, 14), pct(w, 26)),
            ),
            SANDAL,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{at_rest, drawn, poses, triangles};
    use super::*;

    /// The colours that mark each piece of a kind's kit.
    fn kit_colours(kind: Kind) -> Vec<Rgb> {
        let k = kit(kind);
        let mut c = Vec::new();
        match k.hat {
            Hat::None => {}
            Hat::Cap(x) | Hat::Bucket(x) | Hat::Visor(x) => c.push(x),
            Hat::Crown => c.push(CROWN_GREEN),
        }
        c.extend(match k.shirt {
            Shirt::HeartTee => vec![TEE_WHITE, PRINT_BLACK, HEART_RED],
            Shirt::Hawaiian => vec![ALOHA_ORANGE, ALOHA_FLOWER],
            Shirt::LibertyStripe => vec![LIBERTY_GREEN, STRIPE_WHITE],
            Shirt::AppleTee => vec![APPLE_TEE, APPLE_RED, APPLE_LEAF],
            Shirt::Hoodie => vec![HOODIE_RED, HOODIE_PRINT],
            Shirt::TaxiCheck => vec![TAXI_YELLOW, CHECK_BLACK, CHECK_WHITE],
        });
        if let Some(f) = k.shades {
            c.extend([f, BIG_LENS]);
        }
        if k.camera {
            c.extend([CAMERA_BODY, CAMERA_LENS, STRAP]);
        }
        c.extend(k.bumbag);
        if k.lanyard {
            c.extend([LANYARD_BLUE, BADGE_WHITE]);
        }
        if let Some((_, b)) = k.bag {
            c.extend([b, BAG_HEART]);
        }
        if k.socks {
            c.extend([SOCK_WHITE, SANDAL]);
        }
        c
    }

    #[test]
    fn on_the_city_every_kind_wears_its_own_tourist_mix_in_every_pose() {
        // the person, 2026-10-15: "these should be all touristy, so like I
        // Heart NY and such" and "maybe some more different shirts"
        assert_eq!(Gear::from_key("tourist"), Some(Gear::Tourist));
        for map in crate::map::Map::ALL {
            assert_eq!(
                Gear::on(map) == Gear::Tourist,
                map.key() == "city",
                "{map:?}"
            );
        }
        // no two kinds in the same shirt, and each its own mix
        for (i, a) in Kind::ALL.iter().enumerate() {
            for b in &Kind::ALL[i + 1..] {
                assert_ne!(kit(*a).shirt, kit(*b).shirt, "{a:?} {b:?}");
                assert_ne!(kit_colours(*a), kit_colours(*b), "{a:?} {b:?}");
            }
        }
        let all: Vec<Kit> = Kind::ALL.iter().map(|k| kit(*k)).collect();
        assert!(all.iter().any(|k| k.shirt == Shirt::HeartTee));
        assert!(all.iter().any(|k| k.hat == Hat::Crown));
        assert!(all.iter().any(|k| k.camera));
        assert!(all.iter().any(|k| k.bumbag.is_some()));
        assert!(all.iter().any(|k| k.socks));
        for kind in Kind::ALL {
            for p in poses() {
                let pose = Pose {
                    gear: Gear::Tourist,
                    ..p
                };
                let s = drawn(kind, &pose);
                let bare = drawn(kind, &p);
                assert!(
                    s.len() > bare.len() + 8,
                    "{kind:?} {} vs {}",
                    s.len(),
                    bare.len()
                );
                assert!((50..=300).contains(&s.len()), "{kind:?} {} shapes", s.len());
                let has = |c: Rgb| s.iter().any(|x| x.colour == c);
                for c in kit_colours(kind) {
                    assert!(has(c), "{kind:?} {pose:?}: no {c:?}");
                }
                // none of the other maps' gear
                assert!(
                    !has(JACKET) && !has(LENS) && !has(KHAKI) && !has(PITH) && !has(TRICORN),
                    "{kind:?}"
                );
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
            }
        }
    }

    #[test]
    fn i_love_ny_is_on_the_front_and_the_back_for_the_chase_camera() {
        let p = Pose {
            gear: Gear::Tourist,
            ..at_rest()
        };
        let s = drawn(Kind::Gorilla, &p);
        let ahead = crate::geom::facing(p.yaw, 0);
        let mid = s
            .iter()
            .fold(V3::ZERO, |a, x| a.add(x.at))
            .scale(ONE / s.len() as i32);
        let side = |c: Rgb| -> (usize, usize) {
            let on = s.iter().filter(|x| x.colour == c);
            let front = on.clone().filter(|x| x.at.sub(mid).dot(ahead) > 0).count();
            (front, on.count() - front)
        };
        // the heart's three pieces and the I, N and Y's seven strokes, each side
        assert_eq!(side(HEART_RED), (3, 3));
        let (front, back) = side(PRINT_BLACK);
        assert!(front >= 7 && back >= 7, "{front} {back}");
    }

    #[test]
    fn the_hands_stay_free_to_grip_the_vine() {
        // nothing a tourist carries is in or at the hands when swinging
        let swing = poses().into_iter().find(|p| p.rope.is_some()).unwrap();
        for kind in Kind::ALL {
            let pose = Pose {
                gear: Gear::Tourist,
                ..swing
            };
            let s = drawn(kind, &pose);
            let carried = [
                CAMERA_BODY,
                BAG_WHITE,
                BAG_KRAFT,
                BUMBAG_BLACK,
                BUMBAG_NEON,
                BADGE_WHITE,
            ];
            for x in s.iter().filter(|x| carried.contains(&x.colour)) {
                let d = x.at.dist(pose.hands);
                assert!(d >= cm(20), "{kind:?}: {:?} {d} from the hands", x.colour);
            }
        }
    }

    #[test]
    fn the_tourist_outfits_cost_about_what_the_explorer_outfit_does() {
        let mut lines = Vec::new();
        let (mut tourist_all, mut explorer_all) = (0, 0);
        for kind in Kind::ALL {
            let p = at_rest();
            let tris = |g: Gear| triangles(&drawn(kind, &Pose { gear: g, ..p }));
            let bare = tris(Gear::None);
            let (tourist, explorer) = (tris(Gear::Tourist) - bare, tris(Gear::Explorer) - bare);
            lines.push(format!(
                "{kind:?}: bare {bare}, explorer +{explorer}, tourist +{tourist}"
            ));
            tourist_all += tourist;
            explorer_all += explorer;
            assert!(tourist <= explorer * 3 / 2, "{}", lines.join("; "));
        }
        assert!(
            tourist_all <= explorer_all * 5 / 4,
            "tourist {tourist_all} vs explorer {explorer_all}: {}",
            lines.join("; ")
        );
    }
}
