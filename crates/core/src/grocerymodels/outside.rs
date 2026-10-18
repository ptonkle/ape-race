//! The second pass's models (GR-T1–T8): the store's banana logo, the
//! shopping list's items and the order bag, the parking lot (cars, light
//! poles, cart corrals, the entrance canopy, the sliding doors), the plane
//! that picks up the order, the drop ceiling and the bagged ice, and the
//! odd vines' fittings (a sign's arm for a sideways rope, a balloon weight
//! for an upward string).
//!
//! Every builder's signature and doc comment is the contract the scene
//! calls. Shape budgets (`grocerymodels/tests.rs` holds each): the logo 12,
//! a list item 20 lit, the plane 60, a car 24, a light pole 10, a corral
//! 40, the canopy 30, the doors 20, the drop ceiling 60, the ice wall 60.

use super::{
    digit, hash, part, Face, Fr, BANNER_RED, BANNER_YELLOW, CHROME, CREAM, HANDLE, INK, KRAFT,
    NAVY, PRICE_YELLOW, STEEL, WHITE, WIRE, WOOD,
};
use crate::fx::{self, cm, deg, int, mul, ONE, QUARTER, TURN};
use crate::geom::{v3, yaw_of, Affine, V3};
use crate::grocery::{CAR_SIZE, CEILING_AT};
use crate::mesh::Mesh;
use crate::render::{mix, placed, rgb, rod, shade, Batch, Rgb};
use crate::rules::TICKS_PER_SECOND as TPS;

const BROWN_STEM: Rgb = rgb(96, 66, 34);
const POLE: Rgb = rgb(96, 100, 106);
const GALVANISED: Rgb = rgb(188, 192, 196);
const GLASS: Rgb = rgb(52, 66, 84);
const DOOR_GLASS: Rgb = rgb(170, 205, 222);
const ALUMINIUM: Rgb = rgb(172, 178, 184);
const LAMP_WARM: Rgb = rgb(255, 244, 214);
const ICE: Rgb = rgb(214, 232, 246);
const ICE_PRINT: Rgb = rgb(40, 110, 200);
const FROST: Rgb = rgb(224, 242, 255);
const EXIT_GREEN: Rgb = rgb(40, 200, 96);
const TILE: Rgb = rgb(230, 228, 222);
const PLENUM: Rgb = rgb(200, 198, 192);
const STRIP: Rgb = rgb(250, 250, 244);
const SPARKLE: Rgb = rgb(255, 222, 96);
const FOIL: Rgb = rgb(214, 64, 150);
const TYRE: Rgb = rgb(34, 34, 38);
const TRIM: Rgb = rgb(44, 44, 48);

/// An angle `per` a tick, `tick` ticks on, folded into one turn.
fn spin(tick: u32, per: i32) -> i32 {
    fx::wrap((tick as i32 % (TPS * 3600)).wrapping_mul(per))
}

/// The transform of a unit shape in `fr` at local `at`, turned `turn`,
/// sized `size` (for a frame that is not a plain yaw, the plane's).
fn frame_at(fr: &Fr, at: V3) -> Affine {
    Affine::translate(fr.pt(at)).then(&fr.rot)
}

/// A flat elliptic disc on `frame`'s x-y plane: its middle at `at`, its
/// long way `long` turned `angle` from +x, `wide` across it, `thick`
/// through (centred on the plane).
#[allow(clippy::too_many_arguments)]
fn disc(
    b: &mut Batch,
    frame: &Affine,
    at: (i32, i32),
    angle: i32,
    long: i32,
    wide: i32,
    thick: i32,
    c: Rgb,
    glow: i32,
) {
    let a = frame
        .then(&Affine::translate(v3(at.0, at.1, 0)))
        .then(&Affine::rot_z(angle))
        .then(&Affine::rot_x(QUARTER))
        .then(&Affine::scale(v3(long, thick, wide)));
    b.push(Mesh::Cyl, &a, c, glow);
}

/// A flat bar on `frame`'s x-y plane (a box), as `disc`.
#[allow(clippy::too_many_arguments)]
fn bar(
    b: &mut Batch,
    frame: &Affine,
    at: (i32, i32),
    angle: i32,
    long: i32,
    wide: i32,
    thick: i32,
    c: Rgb,
    glow: i32,
) {
    let a = frame
        .then(&Affine::translate(v3(at.0, at.1, 0)))
        .then(&Affine::rot_z(angle))
        .then(&Affine::scale(v3(long, wide, thick)));
    b.push(Mesh::Cube, &a, c, glow);
}

/// The logo as a badge `through` thick plus its own layers, centred on
/// `frame`'s plane, so it reads from both faces (the back mirrored): a
/// cream ring, a navy disc, the banana (`full`: with its highlight and
/// tip, 7 shapes; else 5; `stem` false drops the stem).
fn badge(b: &mut Batch, frame: &Affine, radius: i32, through: i32, full: bool, stem: bool) {
    let t = (radius / 20).max(cm(1));
    let u = |n: i32| part(radius, n, 100);
    disc(
        b,
        frame,
        (0, 0),
        0,
        radius * 2,
        radius * 2,
        through + t,
        CREAM,
        0,
    );
    disc(
        b,
        frame,
        (0, 0),
        0,
        u(168),
        u(168),
        through + t * 2,
        NAVY,
        0,
    );
    // the banana: a crescent cut from a yellow ellipse by a navy one laid
    // over its top (smooth edges from two shapes), tipped up to the
    // right; a pale line along its inside edge; the stem and the dark tip
    let tilt = deg(12);
    let (ts, tc) = (fx::sin(tilt), fx::cos(tilt));
    let spot = |x: i32, y: i32| -> (i32, i32) {
        let (x, y) = (u(x), u(y));
        (mul(x, tc) - mul(y, ts), mul(x, ts) + mul(y, tc))
    };
    let deep = through + t * 3;
    disc(
        b,
        frame,
        spot(0, 14),
        tilt,
        u(124),
        u(76),
        deep,
        PRICE_YELLOW,
        ONE / 6,
    );
    if full {
        disc(
            b,
            frame,
            spot(0, 32),
            tilt,
            u(120),
            u(70),
            deep + t / 2,
            rgb(255, 238, 150),
            ONE / 4,
        );
    }
    disc(
        b,
        frame,
        spot(0, 35),
        tilt,
        u(134),
        u(66),
        deep + t,
        NAVY,
        0,
    );
    if stem {
        bar(
            b,
            frame,
            spot(64, 21),
            tilt + deg(55),
            u(18),
            u(9),
            deep + t + t / 2,
            BROWN_STEM,
            0,
        );
    }
    if full {
        disc(
            b,
            frame,
            spot(-60, 14),
            0,
            u(9),
            u(9),
            deep + t + t / 2,
            BROWN_STEM,
            0,
        );
    }
}

/// The store's logo: a big yellow banana in a navy circle with a cream
/// ring (GR-T1), in `frame`'s x-y plane facing its +z, `radius` round (its
/// ring's outside): a flat badge centred on the plane, the ring a
/// twentieth of the radius thick, the navy disc and the banana each that
/// much prouder, so it reads from behind too (mirrored). 7 shapes. The
/// uniforms, the plane, the canopy, the corrals, the bag and the doors
/// wear it.
pub fn banana_logo(b: &mut Batch, frame: &Affine, radius: i32) {
    badge(b, frame, radius, 0, true, true);
}

/// Shopping-list item `zone` (0 a box of spaghetti, 1 a can of coffee, 2 a
/// box of frozen waffles, 3 a chocolate bar), its middle at `at`, turning
/// slowly with `tick`. `lit` (the viewer's own): glowing brightly, a halo
/// round it, sparkles; not lit (another racer's): its plain colours, dim.
/// About 4 m tall, so it reads from a cross aisle away. The string it
/// hangs on is the vine, drawn by the scene. At most 20 shapes (and two
/// halos) lit.
pub fn list_item(b: &mut Batch, at: V3, zone: u8, lit: bool, tick: u32) {
    // a turn every 8 s
    let fr = Fr::new(at, spin(tick, TURN / (TPS * 8)));
    let g = if lit { ONE } else { 0 };
    let col = |c: Rgb| if lit { c } else { shade(c, 66) };
    // print through the body, so both faces show it
    let slab = |b: &mut Batch, at: V3, size: V3, c: Rgb| {
        fr.put(b, Mesh::Cube, at, size, col(c), g);
    };
    match zone % 4 {
        0 => {
            // spaghetti: a tall blue box, a window on the pasta, a white
            // band and a red stripe at the top
            let (w, h, d) = (int(2), cm(420), cm(90));
            slab(b, V3::ZERO, v3(w, h, d), rgb(30, 80, 190));
            slab(
                b,
                v3(0, -cm(40), 0),
                v3(cm(100), cm(230), d + cm(12)),
                rgb(244, 208, 104),
            );
            for x in [-cm(25), cm(25)] {
                slab(
                    b,
                    v3(x, -cm(40), 0),
                    v3(cm(8), cm(220), d + cm(16)),
                    rgb(220, 176, 70),
                );
            }
            slab(
                b,
                v3(0, cm(150), 0),
                v3(w + cm(6), cm(60), d + cm(6)),
                WHITE,
            );
            slab(
                b,
                v3(0, cm(110), 0),
                v3(w + cm(6), cm(16), d + cm(6)),
                BANNER_RED,
            );
        }
        1 => {
            // coffee: a red can, a cream label, a lid, a bean each side
            let (dia, h) = (cm(290), cm(380));
            fr.put(
                b,
                Mesh::Cyl,
                V3::ZERO,
                v3(dia, h, dia),
                col(rgb(150, 32, 30)),
                g,
            );
            fr.put(
                b,
                Mesh::Cyl,
                v3(0, -cm(20), 0),
                v3(dia + cm(8), cm(170), dia + cm(8)),
                col(rgb(240, 226, 196)),
                g,
            );
            fr.put(
                b,
                Mesh::Cyl,
                v3(0, h / 2 + cm(10), 0),
                v3(dia + cm(10), cm(30), dia + cm(10)),
                col(rgb(60, 40, 30)),
                g,
            );
            for s in [-1, 1] {
                let z = s * (dia / 2 + cm(6));
                fr.put(
                    b,
                    Mesh::Sphere,
                    v3(0, -cm(20), z),
                    v3(cm(90), cm(130), cm(30)),
                    col(rgb(92, 52, 26)),
                    g,
                );
                fr.turned(
                    b,
                    Mesh::Cube,
                    v3(0, -cm(20), z + s * cm(10)),
                    Affine::rot_z(deg(12)),
                    v3(cm(10), cm(110), cm(20)),
                    col(rgb(52, 28, 14)),
                    g,
                );
            }
        }
        2 => {
            // frozen waffles: a yellow box, a frosty band, a golden waffle
            // with its grid and a pat of butter, each face
            let (w, h, d) = (cm(330), cm(390), int(1));
            slab(b, V3::ZERO, v3(w, h, d), rgb(250, 200, 40));
            slab(
                b,
                v3(0, cm(150), 0),
                v3(w + cm(6), cm(70), d + cm(6)),
                rgb(130, 190, 236),
            );
            let wy = -cm(35);
            slab(
                b,
                v3(0, wy, 0),
                v3(cm(230), cm(230), d + cm(10)),
                rgb(204, 140, 56),
            );
            for k in [-1, 1] {
                slab(
                    b,
                    v3(k * cm(40), wy, 0),
                    v3(cm(14), cm(220), d + cm(14)),
                    rgb(150, 92, 34),
                );
                slab(
                    b,
                    v3(0, wy + k * cm(40), 0),
                    v3(cm(220), cm(14), d + cm(14)),
                    rgb(150, 92, 34),
                );
            }
            slab(
                b,
                v3(cm(10), wy + cm(10), 0),
                v3(cm(60), cm(44), d + cm(22)),
                rgb(252, 236, 160),
            );
        }
        _ => {
            // a chocolate bar: a brown wrapper with a red band, the foil
            // torn back at the top, the squares showing
            let (w, d) = (cm(290), cm(55));
            slab(b, v3(0, -cm(45), 0), v3(w, cm(310), d), rgb(110, 58, 30));
            slab(
                b,
                v3(0, -cm(30), 0),
                v3(w + cm(6), cm(90), d + cm(6)),
                BANNER_RED,
            );
            slab(
                b,
                v3(0, cm(130), 0),
                v3(w + cm(4), cm(40), d + cm(4)),
                rgb(214, 214, 220),
            );
            slab(
                b,
                v3(0, cm(170), 0),
                v3(w - cm(10), cm(60), d - cm(6)),
                rgb(74, 40, 24),
            );
            slab(
                b,
                v3(0, cm(170), 0),
                v3(cm(10), cm(60), d + cm(4)),
                rgb(46, 24, 14),
            );
            slab(
                b,
                v3(0, cm(170), 0),
                v3(w - cm(10), cm(8), d + cm(4)),
                rgb(46, 24, 14),
            );
        }
    }
    if !lit {
        return;
    }
    // four sparkles round it, twinkling, and the halo
    for k in 0..4 {
        let a = spin(tick, -(TURN / (TPS * 5))) + k * TURN / 4;
        let tw = fx::sin(spin(tick, TURN / (TPS * 2)) + k * TURN * 3 / 8).abs();
        let s = cm(25) + mul(tw, cm(75));
        let p = at.add(v3(
            mul(fx::sin(a), cm(330)),
            [cm(160), -cm(90), cm(60), -cm(180)][k as usize],
            mul(fx::cos(a), cm(330)),
        ));
        b.push(Mesh::Octa, &placed(p, a, v3(s, s * 2, s)), SPARKLE, ONE);
    }
    // rim glows (their far sides only), so the item's own colours show
    // through the middle: a gold one close round it, a pale one wider
    b.push_halo(
        Mesh::Sphere,
        &placed(at, 0, v3(int(7), int(8), int(7))),
        rgb(255, 214, 80),
        -ONE,
    );
    b.push_halo(
        Mesh::Sphere,
        &placed(at, 0, v3(int(11), int(12), int(11))),
        rgb(255, 244, 200),
        -ONE / 2,
    );
}

/// A full paper grocery bag held up (the order, GR-T5): its bottom's middle
/// at `foot`, facing `yaw`, celery and a baguette out of the top, the
/// banana logo on its side. About 4 m tall to the bag's rim, 5.5 m to the
/// baguette's end. 26 shapes.
pub fn order_bag(b: &mut Batch, foot: V3, yaw: i32) {
    let fr = Fr::new(foot, yaw);
    let (w, h, d) = (cm(320), cm(380), int(2));
    fr.put(b, Mesh::Cube, v3(0, h / 2, 0), v3(w, h, d), KRAFT, 0);
    // the rim folded over, the side gussets' creases
    fr.put(
        b,
        Mesh::Cube,
        v3(0, h - cm(13), 0),
        v3(w + cm(8), cm(36), d + cm(8)),
        shade(KRAFT, 86),
        0,
    );
    // the open top: the bag's dark inside
    fr.put(
        b,
        Mesh::Cube,
        v3(0, h + cm(7), 0),
        v3(w - cm(30), cm(4), d - cm(30)),
        shade(KRAFT, 40),
        0,
    );
    for s in [-1, 1] {
        fr.put(
            b,
            Mesh::Cube,
            v3(s * (w / 2), h / 2 - cm(10), 0),
            v3(cm(6), h - cm(60), cm(10)),
            shade(KRAFT, 72),
            0,
        );
    }
    let logo = frame_at(&fr, v3(0, cm(170), d / 2 + cm(1)));
    banana_logo(b, &logo, cm(95));
    // celery: three pale stalks leaning out of the back, leaves on top
    for (i, (x, lean)) in [(-cm(90), -cm(40)), (-cm(50), cm(10)), (-cm(10), cm(50))]
        .into_iter()
        .enumerate()
    {
        let a = v3(x, h - cm(100), -cm(40));
        let z = v3(x + lean, h + cm(100) + (i as i32 % 2) * cm(25), -cm(60));
        fr.rod(b, Mesh::Taper, a, z, cm(44), rgb(170, 214, 120), 0);
        // two leaves fanned out of the top
        for s in [-1, 1] {
            fr.turned(
                b,
                Mesh::Shard,
                z.add(v3(s * cm(22), cm(40), 0)),
                Affine::rot_z(-s * deg(28) - deg(6) * (i as i32 - 1)),
                v3(cm(60), cm(110), cm(14)),
                rgb(110, 176, 64),
                0,
            );
        }
    }
    // a baguette slanting up out of the front corner, slashed
    let a = v3(cm(80), h - cm(130), cm(30));
    let z = v3(cm(150), h + cm(140), -cm(20));
    fr.rod(b, Mesh::Sphere, a, z, cm(60), rgb(214, 160, 80), 0);
    let along = z.sub(a);
    for k in 1..=3 {
        let p = a.add(along.scale(ONE * (k + 2) / 8)).add(v3(0, 0, cm(26)));
        fr.turned(
            b,
            Mesh::Cube,
            p,
            Affine::rot_z(deg(40)),
            v3(cm(40), cm(9), cm(8)),
            rgb(244, 214, 150),
            0,
        );
    }
    // a red cereal box's corner behind the celery
    fr.turned(
        b,
        Mesh::Cube,
        v3(cm(20), h, -cm(30)),
        Affine::rot_y(deg(20)),
        v3(cm(150), cm(80), cm(50)),
        rgb(214, 48, 40),
        0,
    );
}

/// The plane that picks up the order (GR-T5): a small high-wing cargo
/// plane in the store's livery (white, a navy stripe, the banana logo on
/// the tail), 60 m across the wings and 47 m long, its middle at `at`, nose
/// toward `facing(yaw, pitch)`, banked `roll` (a positive roll dips its
/// right wing), its propeller turning with `tick`. With `hook`, a line
/// hangs from its belly to a hook at that point. At most 60 shapes.
pub fn plane(b: &mut Batch, at: V3, yaw: i32, pitch: i32, roll: i32, tick: u32, hook: Option<V3>) {
    let fr = Fr {
        o: at,
        rot: Affine::rot_y(yaw)
            .then(&Affine::rot_x(-pitch))
            .then(&Affine::rot_z(roll)),
    };
    let m = |x: i32| cm(x);
    let nose_up = Affine::rot_x(QUARTER);
    let tail_back = Affine::rot_x(-QUARTER);
    // the fuselage, its stripe and windows
    fr.put(
        b,
        Mesh::Bevel,
        v3(0, 0, m(100)),
        v3(m(640), m(700), m(2400)),
        WHITE,
        0,
    );
    fr.put(
        b,
        Mesh::Cube,
        v3(0, -m(60), m(100)),
        v3(m(664), m(130), m(2360)),
        NAVY,
        0,
    );
    fr.put(
        b,
        Mesh::Cube,
        v3(0, m(150), m(500)),
        v3(m(656), m(110), m(1200)),
        GLASS,
        ONE / 8,
    );
    // the nose, the windscreen, the engine and its spinner
    fr.turned(
        b,
        Mesh::Taper,
        v3(0, -m(20), m(1550)),
        nose_up,
        v3(m(620), m(500), m(620)),
        WHITE,
        0,
    );
    fr.turned(
        b,
        Mesh::Cube,
        v3(0, m(270), m(1330)),
        Affine::rot_x(-deg(40)),
        v3(m(520), m(200), m(30)),
        GLASS,
        ONE / 8,
    );
    let prop = v3(0, -m(40), m(1960));
    fr.turned(
        b,
        Mesh::Cyl,
        v3(0, -m(40), m(1830)),
        nose_up,
        v3(m(400), m(200), m(400)),
        NAVY,
        0,
    );
    fr.turned(
        b,
        Mesh::Cone,
        prop.add(v3(0, 0, m(80))),
        nose_up,
        v3(m(170), m(180), m(170)),
        WHITE,
        0,
    );
    // three blades, turning fast; a blur disc round them
    let turn = spin(tick, deg(47));
    for k in 0..3 {
        let blade = Affine::rot_z(turn + k * TURN / 3).then(&Affine::translate(v3(m(330), 0, 0)));
        fr.turned(
            b,
            Mesh::Cube,
            prop,
            blade,
            v3(m(620), m(60), m(20)),
            TRIM,
            0,
        );
    }
    b.push_halo(
        Mesh::Cyl,
        &fr.aff(prop, &nose_up, v3(m(1300), m(10), m(1300))),
        rgb(220, 224, 230),
        ONE / 5,
    );
    // the wing on top, navy tips, the lights (red to the left), the struts
    fr.put(
        b,
        Mesh::Cube,
        v3(0, m(400), m(600)),
        v3(int(60), m(100), m(840)),
        WHITE,
        0,
    );
    for s in [-1, 1] {
        fr.put(
            b,
            Mesh::Cube,
            v3(s * m(2920), m(400), m(600)),
            v3(m(160), m(110), m(850)),
            NAVY,
            0,
        );
        fr.put(
            b,
            Mesh::Sphere,
            v3(s * m(3000), m(400), m(780)),
            v3(m(70), m(70), m(70)),
            if s > 0 {
                rgb(255, 40, 40)
            } else {
                rgb(40, 255, 90)
            },
            ONE,
        );
        fr.rod(
            b,
            Mesh::Cyl,
            v3(s * m(300), -m(220), m(600)),
            v3(s * m(1600), m(350), m(600)),
            m(40),
            STEEL,
            0,
        );
    }
    // the tail: a tapering boom, a swept fin with the logo, the tailplane
    fr.turned(
        b,
        Mesh::Taper,
        v3(0, m(60), -m(1800)),
        tail_back,
        v3(m(600), m(1400), m(560)),
        WHITE,
        0,
    );
    fr.put(
        b,
        Mesh::Wedge,
        v3(0, m(800), -m(2100)),
        v3(m(60), m(1200), m(1100)),
        WHITE,
        0,
    );
    let fin = frame_at(&fr, v3(0, m(660), -m(2320))).then(&Affine::rot_y(QUARTER));
    badge(b, &fin, m(320), m(60), true, true);
    fr.put(
        b,
        Mesh::Sphere,
        v3(0, m(1410), -m(2610)),
        v3(m(50), m(50), m(50)),
        rgb(255, 40, 40),
        ONE,
    );
    fr.put(
        b,
        Mesh::Cube,
        v3(0, m(200), -m(2350)),
        v3(int(20), m(50), m(500)),
        WHITE,
        0,
    );
    // the fixed gear: two mains and a nose wheel
    for s in [-1, 1] {
        fr.rod(
            b,
            Mesh::Cube,
            v3(s * m(280), -m(300), m(200)),
            v3(s * m(560), -m(620), m(100)),
            m(50),
            STEEL,
            0,
        );
        fr.turned(
            b,
            Mesh::Cyl,
            v3(s * m(580), -m(660), m(100)),
            Affine::rot_z(QUARTER),
            v3(m(220), m(90), m(220)),
            TYRE,
            0,
        );
    }
    fr.rod(
        b,
        Mesh::Cube,
        v3(0, -m(250), m(1600)),
        v3(0, -m(600), m(1650)),
        m(40),
        STEEL,
        0,
    );
    fr.turned(
        b,
        Mesh::Cyl,
        v3(0, -m(660), m(1650)),
        Affine::rot_z(QUARTER),
        v3(m(180), m(70), m(180)),
        TYRE,
        0,
    );
    // the line from the belly to a J hook, its bend (where it catches)
    // at the point
    if let Some(h) = hook {
        let belly = fr.pt(v3(0, -m(350), -m(200)));
        let eye = h.add(v3(0, m(240), 0));
        b.push(Mesh::Cyl, &rod(belly, eye, m(40)), HANDLE, 0);
        let tip = h.add(v3(m(130), m(130), 0));
        b.push(Mesh::Cyl, &rod(eye, h, m(45)), STEEL, 0);
        b.push(Mesh::Cyl, &rod(h, tip, m(45)), STEEL, 0);
        b.push(
            Mesh::Sphere,
            &placed(h, 0, v3(m(60), m(60), m(60))),
            STEEL,
            0,
        );
    }
}

/// The eight cars' paints.
const PAINTS: [Rgb; 8] = [
    rgb(176, 32, 36),
    rgb(196, 200, 204),
    rgb(236, 236, 232),
    rgb(34, 54, 112),
    rgb(30, 31, 35),
    rgb(48, 104, 74),
    rgb(108, 112, 118),
    rgb(196, 176, 136),
];

/// A parked car: its middle on the ground at `centre`, facing `yaw`,
/// `look` (0..8) its body colour and shape (`look % 4`: a sedan, a
/// hatchback, a pickup, a minivan; `look % 8` the paint), filling
/// `grocery::CAR_SIZE` (its solid box) exactly. At most 24 shapes.
pub fn car(b: &mut Batch, centre: V3, yaw: i32, look: u8) {
    let (len, wid, h) = CAR_SIZE;
    let (hl, hw) = (len / 2, wid / 2);
    let fr = Fr::new(centre, yaw);
    let paint = PAINTS[look as usize % 8];
    let kind = look % 4;
    // the body's top; the roof's back and front; the windscreen's foot;
    // the rear glass's foot (at the roof's back: upright)
    let (top, rb, rf, wf, wb) = match kind {
        0 => (int(8) + cm(60), -int(8), int(4), int(12), -int(14)),
        1 => (
            int(8) + cm(60),
            -int(18),
            int(3),
            int(11) + cm(50),
            -int(22),
        ),
        2 => (int(9) + cm(60), -int(5), int(5), int(11), -int(5)),
        _ => (int(8), -int(22), int(8), int(15), -int(24) - cm(50)),
    };
    let roof = h - cm(70);
    let gw = wid - cm(360);
    // the body, low, full length but the bumpers
    fr.put(
        b,
        Mesh::Bevel,
        v3(0, (cm(180) + top) / 2, 0),
        v3(wid - cm(60), top - cm(180), len - cm(120)),
        paint,
        0,
    );
    // the greenhouse: glass, the windscreen and rear glass sloping, the
    // roof and a pillar in the paint
    fr.put(
        b,
        Mesh::Cube,
        v3(0, (top + roof) / 2, (rb + rf) / 2),
        v3(gw, roof - top, rf - rb),
        GLASS,
        ONE / 8,
    );
    fr.put(
        b,
        Mesh::Wedge,
        v3(0, (top + roof) / 2, (rf + wf) / 2),
        v3(gw - cm(10), roof - top, wf - rf),
        GLASS,
        ONE / 8,
    );
    if rb - wb > cm(20) {
        fr.turned(
            b,
            Mesh::Wedge,
            v3(0, (top + roof) / 2, (rb + wb) / 2),
            Affine::rot_y(fx::HALF),
            v3(gw - cm(10), roof - top, rb - wb),
            GLASS,
            ONE / 8,
        );
    }
    fr.put(
        b,
        Mesh::Cube,
        v3(0, roof + (h - roof) / 2, (rb + rf) / 2),
        v3(gw + cm(40), h - roof, rf - rb + cm(60)),
        paint,
        0,
    );
    let pillar = if kind == 3 {
        rf - int(9)
    } else {
        (rb + rf) / 2
    };
    fr.put(
        b,
        Mesh::Cube,
        v3(0, (top + roof) / 2, pillar),
        v3(gw + cm(20), roof - top, cm(90)),
        paint,
        0,
    );
    // the door seam
    fr.put(
        b,
        Mesh::Cube,
        v3(0, (cm(260) + top - cm(60)) / 2, pillar),
        v3(wid - cm(54), top - cm(320), cm(12)),
        shade(paint, 55),
        0,
    );
    // a pickup's bed: its walls and tailgate, a dark liner
    if kind == 2 {
        let (bf, bk) = (-int(6) - cm(50), -hl + cm(60));
        let wall = int(12) + cm(60);
        for s in [-1, 1] {
            fr.put(
                b,
                Mesh::Cube,
                v3(s * (hw - cm(65)), (top + wall) / 2, (bf + bk) / 2),
                v3(cm(70), wall - top + cm(10), bf - bk),
                paint,
                0,
            );
        }
        fr.put(
            b,
            Mesh::Cube,
            v3(0, (top + wall) / 2, bk + cm(35)),
            v3(wid - cm(60), wall - top + cm(10), cm(70)),
            paint,
            0,
        );
        fr.put(
            b,
            Mesh::Cube,
            v3(0, top + cm(5), (bf + bk) / 2),
            v3(wid - cm(200), cm(20), bf - bk - cm(100)),
            TRIM,
            0,
        );
    }
    // bumpers, front and back, to the box's ends
    for s in [-1, 1] {
        fr.put(
            b,
            Mesh::Cube,
            v3(0, cm(380), s * (hl - cm(70))),
            v3(wid - cm(100), cm(220), cm(140)),
            TRIM,
            0,
        );
    }
    // wheels to the box's sides and the ground, hubcaps through each axle
    let axle = if kind == 0 || kind == 1 {
        int(17)
    } else {
        int(18)
    };
    for z in [-axle, axle] {
        for s in [-1, 1] {
            fr.turned(
                b,
                Mesh::Cyl,
                v3(s * (hw - cm(130)), cm(360), z),
                Affine::rot_z(QUARTER),
                v3(cm(720), cm(260), cm(720)),
                TYRE,
                0,
            );
        }
        fr.turned(
            b,
            Mesh::Cyl,
            v3(0, cm(360), z),
            Affine::rot_z(QUARTER),
            v3(cm(380), wid, cm(380)),
            CHROME,
            0,
        );
    }
    // lights and the grille
    let ly = top - cm(170);
    for s in [-1, 1] {
        fr.put(
            b,
            Mesh::Cube,
            v3(s * (hw - cm(280)), ly, hl - cm(60)),
            v3(cm(340), cm(140), cm(60)),
            rgb(250, 246, 220),
            ONE / 2,
        );
        fr.put(
            b,
            Mesh::Cube,
            v3(s * (hw - cm(240)), ly, -hl + cm(60)),
            v3(cm(260), cm(150), cm(60)),
            rgb(220, 30, 30),
            ONE / 3,
        );
    }
    fr.put(
        b,
        Mesh::Cube,
        v3(0, ly - cm(20), hl - cm(60)),
        v3(cm(760), cm(170), cm(50)),
        TRIM,
        0,
    );
}

/// A parking lot light pole on `foot`, `height` tall (to its arm), its arm
/// out to `arm` (a point at the arm's height, where the vine hangs), a
/// lamp head at the arm's end lit from below (a glowing lens, a soft cone
/// of light under it). 7 shapes.
pub fn lamp_post(b: &mut Batch, foot: V3, height: i32, arm: V3) {
    let top = foot.add(v3(0, height + int(1), 0));
    let root = foot.add(v3(0, height, 0));
    b.push(
        Mesh::Cyl,
        &placed(
            foot.add(v3(0, cm(150), 0)),
            0,
            v3(cm(360), cm(300), cm(360)),
        ),
        rgb(172, 170, 162),
        0,
    );
    b.push(Mesh::Taper, &rod(foot, top, cm(140)), POLE, 0);
    b.push(
        Mesh::Sphere,
        &placed(top, 0, v3(cm(100), cm(70), cm(100))),
        POLE,
        0,
    );
    b.push(Mesh::Cyl, &rod(root, arm, cm(60)), POLE, 0);
    let out = arm.sub(root);
    let mid = root.lerp(arm, ONE / 2);
    b.push(
        Mesh::Cyl,
        &rod(root.sub(v3(0, int(3), 0)), mid, cm(30)),
        POLE,
        0,
    );
    let yaw = if out.len_xz() > cm(10) {
        yaw_of(out)
    } else {
        0
    };
    b.push(
        Mesh::Bevel,
        &placed(arm.add(v3(0, cm(20), 0)), yaw, v3(cm(320), cm(120), int(5))),
        rgb(70, 74, 80),
        0,
    );
    b.push(
        Mesh::Cube,
        &placed(arm.sub(v3(0, cm(45), 0)), yaw, v3(cm(260), cm(20), cm(420))),
        LAMP_WARM,
        ONE,
    );
    b.push_halo(
        Mesh::Cone,
        &placed(
            arm.sub(v3(0, int(7), 0)),
            yaw,
            v3(int(14), int(13), int(16)),
        ),
        LAMP_WARM,
        ONE / 6,
    );
}

/// A cart corral: its middle line from `from` to `to` on the ground (open
/// at `from`, closed at `to`), two rails each side, a sign on a post at
/// `from`'s end (the logo), a row of nested carts inside. At most 40.
pub fn corral(b: &mut Batch, from: V3, to: V3) {
    let run = to.sub(from).flat();
    let len = run.len();
    if len < int(2) {
        return;
    }
    let fr = Fr::new(from, yaw_of(run));
    let hw = cm(480);
    let (lo, hi) = (cm(550), cm(1050));
    // rails both sides at two heights, posts at the ends and the middle,
    // the closed end
    for s in [-1, 1] {
        for y in [lo, hi] {
            fr.rod(
                b,
                Mesh::Cyl,
                v3(s * hw, y, 0),
                v3(s * hw, y, len),
                cm(50),
                GALVANISED,
                0,
            );
        }
        for z in [0, len / 2, len] {
            fr.rod(
                b,
                Mesh::Cyl,
                v3(s * hw, 0, z),
                v3(s * hw, hi + cm(30), z),
                cm(60),
                GALVANISED,
                0,
            );
        }
    }
    for y in [lo, hi] {
        fr.rod(
            b,
            Mesh::Cyl,
            v3(-hw, y, len),
            v3(hw, y, len),
            cm(50),
            GALVANISED,
            0,
        );
    }
    // the sign: a navy board on a post, the logo, facing the open end
    let post = v3(hw + cm(80), 0, 0);
    fr.rod(
        b,
        Mesh::Cyl,
        post,
        post.add(v3(0, int(24), 0)),
        cm(50),
        NAVY,
        0,
    );
    let board = post.add(v3(0, int(21), -cm(40)));
    fr.put(b, Mesh::Cube, board, v3(int(6), int(5), cm(30)), NAVY, 0);
    let logo = frame_at(&fr, board).then(&Affine::rot_y(fx::HALF));
    badge(b, &logo, cm(240), cm(30), false, true);
    // nested carts against the closed end, noses toward it: the stack's
    // wire basket as rails and a floor, its wheels, and each cart's back
    // gate and red handle
    let pitch = int(3);
    let cart_len = cm(1080);
    let room = len - cm(150) - cart_len;
    if room < 0 {
        return;
    }
    let n = (room / pitch + 1).min(CORRAL_CARTS);
    let (front, back) = (len - cm(100), len - cm(100) - cart_len - (n - 1) * pitch);
    let (mid, run_len) = ((front + back) / 2, front - back);
    let half = cm(330);
    for s in [-1, 1] {
        for y in [cm(860), cm(1150)] {
            fr.rod(
                b,
                Mesh::Cyl,
                v3(s * half, y, back),
                v3(s * half, y, front),
                cm(30),
                CHROME,
                0,
            );
        }
        fr.put(
            b,
            Mesh::Cube,
            v3(s * cm(250), cm(35), mid),
            v3(cm(40), cm(70), run_len - cm(160)),
            TYRE,
            0,
        );
    }
    fr.put(
        b,
        Mesh::Cube,
        v3(0, cm(580), mid),
        v3(half * 2, cm(24), run_len),
        WIRE,
        0,
    );
    for k in 0..n {
        let z = back + k * pitch;
        fr.turned(
            b,
            Mesh::Cube,
            v3(0, cm(870), z + cm(40)),
            Affine::rot_x(-deg(10)),
            v3(half * 2 - cm(20), cm(560), cm(16)),
            WIRE,
            0,
        );
        fr.rod(
            b,
            Mesh::Cyl,
            v3(-cm(340), int(12), z - cm(40)),
            v3(cm(340), int(12), z - cm(40)),
            cm(48),
            rgb(206, 38, 44),
            0,
        );
    }
}

/// The most carts nested in a corral.
const CORRAL_CARTS: i32 = 6;

/// The entrance canopy over the sidewalk: a flat roof from the store's
/// front out over the lot, its corners `a` and `c` on the ground (least and
/// most), its underside `under` over the ground (`a`'s height), on posts at
/// the outer corners, the banana logo on its front fascia. The store's
/// front is along the most x (the plan's north, `GroceryLayout::outside`):
/// the outer edge, the posts and the logo are at the least. At most 30.
pub fn canopy(b: &mut Batch, a: V3, c: V3, under: i32) {
    let lo = v3(a.x.min(c.x), a.y, a.z.min(c.z));
    let hi = v3(a.x.max(c.x), a.y, a.z.max(c.z));
    let (dx, dz) = (hi.x - lo.x, hi.z - lo.z);
    let (mx, mz) = ((lo.x + hi.x) / 2, (lo.z + hi.z) / 2);
    let y = a.y + under;
    let cream = rgb(226, 224, 218);
    b.push(
        Mesh::Cube,
        &placed(v3(mx, y + cm(120), mz), 0, v3(dx, cm(240), dz)),
        cream,
        0,
    );
    // the fascia: navy round three sides, standing a little proud of the
    // roof's edges (no shared faces), a yellow line along the front
    let fh = cm(560);
    let fy = y + fh / 2 - cm(60);
    b.push(
        Mesh::Cube,
        &placed(v3(lo.x + cm(50), fy, mz), 0, v3(cm(120), fh, dz + cm(10))),
        NAVY,
        0,
    );
    for z in [lo.z + cm(55), hi.z - cm(55)] {
        b.push(
            Mesh::Cube,
            &placed(
                v3((lo.x + cm(110) + hi.x) / 2, fy, z),
                0,
                v3(dx - cm(110), fh, cm(120)),
            ),
            NAVY,
            0,
        );
    }
    b.push(
        Mesh::Cube,
        &placed(
            v3(lo.x + cm(50), y + cm(30), mz),
            0,
            v3(cm(150), cm(80), dz + cm(16)),
        ),
        PRICE_YELLOW,
        0,
    );
    // lights in the underside, two rows of three
    for i in 0..2 {
        for j in 0..3 {
            let x = lo.x + part(dx, 1 + 2 * i, 4);
            let z = lo.z + part(dz, 1 + 2 * j, 6);
            b.push(
                Mesh::Cube,
                &placed(v3(x, y - cm(5), z), 0, v3(cm(300), cm(20), cm(600))),
                STRIP,
                ONE,
            );
        }
    }
    // posts at the outer corners
    for z in [lo.z + cm(150), hi.z - cm(150)] {
        let foot = v3(lo.x + cm(150), a.y, z);
        b.push(
            Mesh::Cyl,
            &rod(foot, foot.add(v3(0, under, 0)), cm(180)),
            NAVY,
            0,
        );
        b.push(
            Mesh::Cyl,
            &placed(foot.add(v3(0, cm(60), 0)), 0, v3(cm(280), cm(120), cm(280))),
            rgb(172, 170, 162),
            0,
        );
    }
    // the logo, standing up from the front fascia's middle
    let r = (dz / 9).clamp(int(3), int(7));
    let logo =
        Affine::translate(v3(lo.x - cm(10), y + fh - cm(60), mz)).then(&Affine::rot_y(-QUARTER));
    banana_logo(b, &logo, r);
}

/// The exit's sliding glass doors in the store's front: the opening's
/// middle on the ground at `mid`, the way across it `along` (flat, unit),
/// `width` across and `height` up; each leaf slid `open` (0 shut .. ONE
/// fully open) into the wall. A frame, a green exit sign over it (both
/// faces), the banana logo on each leaf (through the glass). At most 20.
pub fn doors(b: &mut Batch, mid: V3, along: V3, width: i32, height: i32, open: i32) {
    let fr = Fr::new(mid, yaw_of(along) - QUARTER);
    let open = open.clamp(0, ONE);
    // the jambs and the header over the opening
    for s in [-1, 1] {
        fr.put(
            b,
            Mesh::Cube,
            v3(s * (width / 2 + cm(70)), (height + cm(360)) / 2, 0),
            v3(cm(140), height + cm(360), cm(160)),
            ALUMINIUM,
            0,
        );
    }
    fr.put(
        b,
        Mesh::Cube,
        v3(0, height + cm(180), 0),
        v3(width + cm(280), cm(360), cm(200)),
        ALUMINIUM,
        0,
    );
    // the exit sign
    fr.put(
        b,
        Mesh::Cube,
        v3(0, height + cm(600), 0),
        v3(int(8), cm(300), int(1)),
        EXIT_GREEN,
        ONE,
    );
    fr.put(
        b,
        Mesh::Cube,
        v3(0, height + cm(600), 0),
        v3(cm(560), cm(70), cm(110)),
        WHITE,
        ONE,
    );
    // the leaves: an aluminium frame, the glass inset through it, the logo
    let lw = width / 2 + cm(20);
    let r = (width / 9).clamp(int(1), int(4));
    for s in [-1, 1] {
        let x = s * (width / 4 + mul(open, width / 2));
        fr.put(
            b,
            Mesh::Cube,
            v3(x, height / 2, 0),
            v3(lw, height - cm(20), cm(50)),
            ALUMINIUM,
            0,
        );
        fr.put(
            b,
            Mesh::Cube,
            v3(x, height / 2 + cm(10), 0),
            v3(lw - cm(130), height - cm(160), cm(60)),
            DOOR_GLASS,
            ONE / 4,
        );
        let logo = frame_at(&fr, v3(x, part(height, 56, 100), 0));
        badge(b, &logo, r, cm(60), false, false);
    }
}

/// The ceiling tiles' grid: lines in each direction.
const GRID_MOST: i32 = 9;
/// A ceiling tile (60 cm real).
const TILE_SIZE: i32 = cm(720);

/// The drop ceiling over part of the frozen aisle (GR-T7): a box along
/// `fwd` (flat, unit) centred on `centre` (its height ignored), `half_width`
/// across and `half_len` along, its underside `under` over the floor, up
/// to the store's ceiling: white acoustic tiles on a grid, lit panels, a
/// navy fascia at each end with a frozen snowflake. At most 46 shapes.
pub fn low_ceiling(b: &mut Batch, centre: V3, fwd: V3, half_width: i32, half_len: i32, under: i32) {
    let fr = Fr::new(v3(centre.x, 0, centre.z), yaw_of(fwd));
    let top = CEILING_AT.max(under + cm(80));
    let lit = crate::groceryscape::course::LIT;
    let (hw, hl) = (half_width, half_len);
    // the tile slab and the box over it, short of the fascias
    fr.put(
        b,
        Mesh::Cube,
        v3(0, under + cm(40), 0),
        v3(hw * 2 - cm(10), cm(80), hl * 2 - cm(170)),
        TILE,
        lit,
    );
    if top - under > cm(100) {
        fr.put(
            b,
            Mesh::Cube,
            v3(0, (under + cm(80) + top) / 2, 0),
            v3(hw * 2 - cm(40), top - under - cm(80), hl * 2 - cm(200)),
            PLENUM,
            lit,
        );
    }
    // the T-bar grid under the tiles
    let lines = |half: i32| ((half * 2) / TILE_SIZE - 1).clamp(1, GRID_MOST);
    let (nx, nz) = (lines(hw), lines(hl));
    let (sx, sz) = (hw * 2 / (nx + 1), hl * 2 / (nz + 1));
    for i in 1..=nx {
        fr.put(
            b,
            Mesh::Cube,
            v3(-hw + sx * i, under, 0),
            v3(cm(30), cm(12), hl * 2 - cm(170)),
            WHITE,
            lit,
        );
    }
    for i in 1..=nz {
        fr.put(
            b,
            Mesh::Cube,
            v3(0, under, -hl + sz * i),
            v3(hw * 2 - cm(10), cm(12), cm(30)),
            WHITE,
            lit,
        );
    }
    // the lit panels: two cells along, in two rows of three
    for cx in [(nx + 1) / 4, (nx + 1) * 3 / 4] {
        for k in [(nz + 1) / 6, (nz + 1) / 2, (nz + 1) * 5 / 6] {
            let k = k.clamp(1, nz.max(1));
            fr.put(
                b,
                Mesh::Cube,
                v3(-hw + sx * cx + sx / 2, under - cm(8), -hl + sz * k),
                v3(part(sx, 86, 100), cm(10), part(sz * 2, 92, 100)),
                STRIP,
                ONE,
            );
        }
    }
    // the fascias and their snowflakes
    let fh = (top - under).min(int(9));
    for s in [-1, 1] {
        fr.put(
            b,
            Mesh::Cube,
            v3(0, under + fh / 2, s * (hl - cm(40))),
            v3(hw * 2, fh, cm(80)),
            NAVY,
            0,
        );
        let face = frame_at(
            &fr.sub(
                v3(0, under + fh / 2, s * hl),
                if s > 0 { 0 } else { fx::HALF },
            ),
            V3::ZERO,
        );
        let size = part(fh, 80, 100).min(int(7));
        snowflake(b, &face, size);
    }
}

/// A snowflake on `frame`'s x-y plane, `size` across: three bars and a
/// diamond at each arm's end (9 shapes), proud of the plane by 6 cm.
fn snowflake(b: &mut Batch, frame: &Affine, size: i32) {
    let t = (size / 10).max(cm(20));
    for i in 0..3 {
        bar(
            b,
            frame,
            (0, 0),
            deg(30) + deg(60) * i,
            size,
            t,
            cm(16),
            FROST,
            ONE / 2,
        );
    }
    for i in 0..6 {
        let a = deg(30) + deg(60) * i;
        let r = part(size, 36, 100);
        let at = frame.then(&Affine::translate(v3(
            mul(fx::cos(a), r),
            mul(fx::sin(a), r),
            0,
        )));
        b.push(
            Mesh::Octa,
            &at.then(&Affine::rot_z(a))
                .then(&Affine::scale(v3(t * 3, t * 2, cm(16)))),
            FROST,
            ONE / 2,
        );
    }
}

/// The bagged-ice pallets stacked across the aisle (GR-T7): along `from`
/// to `to` (their middle line on the floor), `half` thick (either side),
/// `top` tall, filling that box exactly (it is solid there): a row of
/// wooden pallets, clear bags of ice cubes stacked four layers on each
/// with a blue print band across each layer, and a navy sign with a
/// snowflake on each face of the top layer. At most 48 shapes.
pub fn ice_wall(b: &mut Batch, from: V3, to: V3, half: i32, top: i32) {
    let run = to.sub(from).flat();
    let len = run.len();
    if len < int(1) {
        return;
    }
    let fr = Fr::new(from.lerp(to, ONE / 2), yaw_of(run) - QUARTER);
    let n = (len / int(15)).clamp(1, 6);
    let pitch = len / n;
    let pallet = cm(170);
    let layers = 4;
    let lh = (top - pallet) / layers;
    let bag_d = half * 2 - cm(60);
    for k in 0..n {
        let x = -len / 2 + pitch * k + pitch / 2;
        fr.put(
            b,
            Mesh::Cube,
            v3(x, pallet / 2, 0),
            v3(pitch - cm(16), pallet, half * 2),
            WOOD,
            0,
        );
        fr.put(
            b,
            Mesh::Cube,
            v3(x, part(pallet, 45, 100), 0),
            v3(
                part(pitch, 70, 100),
                part(pallet, 40, 100),
                half * 2 + cm(4),
            ),
            rgb(30, 24, 20),
            0,
        );
        for j in 0..layers {
            let inset = [cm(10), cm(40), cm(20), cm(50)][((k + j) % 4) as usize];
            let tone = mix(
                ICE,
                WHITE,
                ((hash((k * 7 + j) as u32) % 5) as i32) * ONE / 20,
            );
            fr.put(
                b,
                Mesh::Bevel,
                v3(x, pallet + lh * j + lh / 2, 0),
                v3(pitch - cm(16) - inset, lh, bag_d),
                tone,
                ONE / 4,
            );
        }
    }
    for j in 0..layers {
        fr.put(
            b,
            Mesh::Cube,
            v3(0, pallet + lh * j + part(lh, 58, 100), 0),
            v3(len - cm(160), part(lh, 22, 100), bag_d + cm(20)),
            ICE_PRINT,
            ONE / 6,
        );
    }
    // the signs on the top layer's faces
    let sy = top - lh / 2;
    let sw = (len / 6).min(int(10));
    for s in [-1, 1] {
        fr.put(
            b,
            Mesh::Cube,
            v3(0, sy, s * (half - cm(15))),
            v3(sw, part(lh, 74, 100), cm(30)),
            NAVY,
            0,
        );
        let face = frame_at(
            &fr.sub(
                v3(0, sy, s * (half - cm(6))),
                if s > 0 { 0 } else { fx::HALF },
            ),
            V3::ZERO,
        );
        for i in 0..3 {
            bar(
                b,
                &face,
                (0, 0),
                deg(30) + deg(60) * i,
                part(lh, 62, 100),
                cm(40),
                cm(16),
                FROST,
                ONE / 2,
            );
        }
    }
}

/// A hanging sale sign's arm, for a sideways vine (GR-T8): chains from the
/// ceiling (`CEILING_AT`) down to a red-and-yellow sign beside `anchor`
/// (its middle level with it), and a short arm from the sign's edge along
/// `way` (flat, unit) to `anchor`, an eye there that the rope leaves from.
pub fn sign_arm(b: &mut Batch, anchor: V3, way: V3) {
    let fr = Fr::new(anchor, yaw_of(way) - QUARTER);
    let (w, h, d) = (int(7), cm(550), cm(25));
    let arm = cm(250);
    let mid = v3(-arm - w / 2, 0, 0);
    fr.put(b, Mesh::Cube, mid, v3(w, h, d), BANNER_RED, 0);
    // the arm: a bracket on the sign's edge, a rod, the eye
    fr.put(
        b,
        Mesh::Cube,
        v3(-arm - cm(20), 0, 0),
        v3(cm(50), cm(160), cm(50)),
        STEEL,
        0,
    );
    fr.rod(b, Mesh::Cyl, v3(-arm, 0, 0), V3::ZERO, cm(35), STEEL, 0);
    fr.put(
        b,
        Mesh::Sphere,
        V3::ZERO,
        v3(cm(70), cm(70), cm(70)),
        STEEL,
        0,
    );
    // the chains up to the ceiling, a plate at each
    let up = CEILING_AT.max(anchor.y + int(3)) - anchor.y;
    for s in [-1, 1] {
        let x = mid.x + s * part(w, 38, 100);
        fr.rod(
            b,
            Mesh::Cube,
            v3(x, h / 2, 0),
            v3(x, up, 0),
            cm(14),
            STEEL,
            0,
        );
        fr.put(
            b,
            Mesh::Cube,
            v3(x, up - cm(10), 0),
            v3(cm(80), cm(20), cm(80)),
            STEEL,
            0,
        );
    }
    // each face: a yellow price panel, its digit, a check mark over it
    let n = 1 + hash((anchor.x ^ anchor.z.rotate_left(7)) as u32) % 9;
    for turn in [0, fx::HALF] {
        let side = fr.sub(mid, turn);
        let face = Face::front(&side, 0, d / 2, w, h, cm(3));
        let py = -part(h, 20, 100);
        let (pw, ph) = (part(w, 84, 100), part(h, 48, 100));
        face.slab(b, (0, py), pw, ph, BANNER_YELLOW, 0);
        digit(
            b,
            &face,
            (0, py),
            part(ph, 50, 100),
            part(ph, 80, 100),
            part(ph, 13, 100),
            n,
            INK,
            1,
        );
        let t = part(w, 8, 100);
        let low = (-part(w, 6, 100), part(h, 14, 100));
        face.bar(
            b,
            (-part(w, 24, 100), part(h, 27, 100)),
            low,
            t,
            BANNER_YELLOW,
            0,
        );
        face.bar(
            b,
            low,
            (part(w, 28, 100), part(h, 40, 100)),
            t,
            BANNER_YELLOW,
            0,
        );
    }
}

/// A balloon weight (GR-T8: an upward vine's anchor): a shiny foil-wrapped
/// weight sitting at `at` (its bottom's middle), the foil gathered on top
/// and the string tied there, 2.6 m up. 8 shapes and a glint.
pub fn balloon_weight(b: &mut Batch, at: V3) {
    let fr = Fr::new(at, deg(20));
    // a flat foot, the crinkled body over it, the neck gathered tight
    fr.put(
        b,
        Mesh::Hex,
        v3(0, cm(30), 0),
        v3(cm(210), cm(60), cm(210)),
        shade(FOIL, 86),
        ONE / 4,
    );
    fr.put(
        b,
        Mesh::Ico,
        v3(0, cm(95), 0),
        v3(cm(200), cm(140), cm(200)),
        FOIL,
        ONE / 4,
    );
    fr.turned(
        b,
        Mesh::Ico,
        v3(0, cm(100), 0),
        Affine::rot_y(deg(36)),
        v3(cm(170), cm(150), cm(170)),
        mix(FOIL, WHITE, ONE / 5),
        ONE / 4,
    );
    fr.put(
        b,
        Mesh::Cone,
        v3(0, cm(185), 0),
        v3(cm(90), cm(60), cm(90)),
        shade(FOIL, 90),
        ONE / 4,
    );
    // the foil's ruffle fanned out over the neck
    for s in [-1, 1] {
        fr.turned(
            b,
            Mesh::Shard,
            v3(s * cm(25), cm(230), 0),
            Affine::rot_z(-s * deg(35)).then(&Affine::rot_y(s * deg(30))),
            v3(cm(70), cm(70), cm(16)),
            mix(FOIL, WHITE, ONE / 8),
            ONE / 4,
        );
    }
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, cm(240), 0),
        v3(cm(12), cm(40), cm(12)),
        WHITE,
        0,
    );
    fr.put(
        b,
        Mesh::Octa,
        v3(cm(55), cm(110), cm(80)),
        v3(cm(30), cm(40), cm(30)),
        WHITE,
        ONE,
    );
    b.push_halo(
        Mesh::Sphere,
        &placed(
            fr.pt(v3(cm(55), cm(110), cm(80))),
            0,
            v3(cm(120), cm(120), cm(120)),
        ),
        WHITE,
        ONE / 4,
    );
}
