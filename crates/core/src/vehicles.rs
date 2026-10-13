//! The City's vehicles (2026-10-12, group Y): parked and moving cars and
//! yellow cabs, vans and buses, hovering helicopters, and boats on the
//! river, in the faceted low-poly style (TRIAGE T3). Each is built in its
//! own metres, +z forward and +y up, about a point on the ground (a
//! helicopter about its rotor's hub), then turned and scaled into place.
//! Nothing here keeps state: the rotors and the bobbing come from the tick.

use crate::city::CarKind;
use crate::fx::{cm, deg, int, mul, sin, ONE};
use crate::geom::{v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{rgb, shade, Batch, Rgb};

const CAB_YELLOW: Rgb = rgb(255, 196, 30);
const SEDANS: [Rgb; 5] = [
    rgb(150, 30, 40),
    rgb(40, 60, 110),
    rgb(200, 204, 210),
    rgb(30, 32, 36),
    rgb(70, 110, 90),
];
const VANS: [Rgb; 3] = [rgb(228, 228, 222), rgb(110, 70, 50), rgb(70, 76, 90)];
const BUS_WHITE: Rgb = rgb(222, 226, 232);
const BUS_BLUE: Rgb = rgb(30, 80, 170);
const GLASS: Rgb = rgb(34, 44, 62);
const TYRE: Rgb = rgb(26, 26, 28);
const RIM: Rgb = rgb(150, 150, 156);
const HEADLIGHT: Rgb = rgb(255, 246, 210);
const TAILLIGHT: Rgb = rgb(255, 40, 40);
const INTERIOR: Rgb = rgb(255, 222, 150);
const SIGN: Rgb = rgb(255, 250, 220);

/// How tall each kind stands, life size (its roof, the top of what bounces).
pub fn roof(kind: CarKind) -> i32 {
    match kind {
        CarKind::Cab => cm(160),
        CarKind::Sedan => cm(146),
        CarKind::Van => cm(212),
        CarKind::Bus => cm(330),
    }
}

/// A shape in a vehicle's own frame `f`: `mesh` of size `size` (metres)
/// with its middle at `at`.
fn part(b: &mut Batch, f: &Affine, mesh: Mesh, at: V3, size: V3, c: Rgb, glow: i32) {
    b.push(
        mesh,
        &f.then(&Affine::translate(at)).then(&Affine::scale(size)),
        c,
        glow,
    );
}

/// A wheel across the car (its axle along x) at `at`, `d` across.
fn wheel(b: &mut Batch, f: &Affine, at: V3, d: i32, w: i32) {
    let axle = f.then(&Affine::translate(at)).then(&Affine::rot_z(deg(90)));
    b.push(Mesh::Hex, &axle.then(&Affine::scale(v3(d, w, d))), TYRE, 0);
    b.push(
        Mesh::Hex,
        &axle.then(&Affine::scale(v3(d / 2, w + cm(4), d / 2))),
        RIM,
        0,
    );
}

/// The frame of a vehicle standing at `at` (on the ground), facing `yaw`,
/// `scale` times life size (Q16).
pub fn frame(at: V3, yaw: i32, scale: i32) -> Affine {
    Affine::translate(at)
        .then(&Affine::rot_y(yaw))
        .then(&Affine::scale(v3(scale, scale, scale)))
}

/// A car, cab, van or bus. `shade_of` picks its paint among the kind's.
pub fn car(b: &mut Batch, kind: CarKind, f: &Affine, shade_of: usize) {
    match kind {
        CarKind::Cab | CarKind::Sedan => {
            let cab = kind == CarKind::Cab;
            let paint = if cab {
                CAB_YELLOW
            } else {
                SEDANS[shade_of % SEDANS.len()]
            };
            let (w, l) = (cm(186), cm(500));
            // the body, the cabin with its windows, and the roof
            part(
                b,
                f,
                Mesh::Bevel,
                v3(0, cm(66), 0),
                v3(w, cm(62), l),
                paint,
                0,
            );
            part(
                b,
                f,
                Mesh::Bevel,
                v3(0, cm(118), -cm(20)),
                v3(w - cm(22), cm(52), cm(250)),
                GLASS,
                ONE / 8,
            );
            part(
                b,
                f,
                Mesh::Cube,
                v3(0, cm(142), -cm(24)),
                v3(w - cm(30), cm(8), cm(200)),
                paint,
                0,
            );
            for z in [cm(150), -cm(150)] {
                for x in [cm(84), -cm(84)] {
                    wheel(b, f, v3(x, cm(36), z), cm(72), cm(26));
                }
            }
            lights(b, f, w, cm(70), l);
            if cab {
                // the roof sign, lit, and the checker band along the side
                part(
                    b,
                    f,
                    Mesh::Bevel,
                    v3(0, cm(156), -cm(30)),
                    v3(cm(90), cm(22), cm(30)),
                    SIGN,
                    ONE * 3 / 4,
                );
                part(
                    b,
                    f,
                    Mesh::Cube,
                    v3(0, cm(80), 0),
                    v3(w + cm(2), cm(10), cm(300)),
                    shade(TYRE, 140),
                    0,
                );
            }
        }
        CarKind::Van => {
            let paint = VANS[shade_of % VANS.len()];
            let (w, l) = (cm(200), cm(540));
            part(
                b,
                f,
                Mesh::Bevel,
                v3(0, cm(120), -cm(10)),
                v3(w, cm(170), cm(500)),
                paint,
                0,
            );
            part(
                b,
                f,
                Mesh::Bevel,
                v3(0, cm(70), cm(230)),
                v3(w - cm(6), cm(70), cm(90)),
                paint,
                0,
            );
            // the windscreen and the cab's side windows
            part(
                b,
                f,
                Mesh::Cube,
                v3(0, cm(160), cm(242)),
                v3(w - cm(24), cm(60), cm(4)),
                GLASS,
                ONE / 8,
            );
            part(
                b,
                f,
                Mesh::Cube,
                v3(0, cm(160), cm(190)),
                v3(w + cm(2), cm(54), cm(80)),
                GLASS,
                ONE / 8,
            );
            for z in [cm(170), -cm(180)] {
                for x in [cm(90), -cm(90)] {
                    wheel(b, f, v3(x, cm(38), z), cm(76), cm(28));
                }
            }
            lights(b, f, w, cm(80), l);
        }
        CarKind::Bus => {
            let (w, l) = (cm(255), cm(1200));
            part(
                b,
                f,
                Mesh::Bevel,
                v3(0, cm(176), 0),
                v3(w, cm(272), l),
                BUS_WHITE,
                0,
            );
            // a lit band of windows down each side, the blue stripe, and the
            // windscreen with its destination sign
            part(
                b,
                f,
                Mesh::Cube,
                v3(0, cm(210), -cm(40)),
                v3(w + cm(4), cm(110), l - cm(200)),
                INTERIOR,
                ONE / 2,
            );
            part(
                b,
                f,
                Mesh::Cube,
                v3(0, cm(110), 0),
                v3(w + cm(4), cm(30), l + cm(2)),
                BUS_BLUE,
                0,
            );
            part(
                b,
                f,
                Mesh::Cube,
                v3(0, cm(200), l / 2),
                v3(w - cm(20), cm(150), cm(6)),
                GLASS,
                ONE / 8,
            );
            part(
                b,
                f,
                Mesh::Cube,
                v3(0, cm(292), l / 2 + cm(2)),
                v3(w - cm(60), cm(30), cm(6)),
                rgb(255, 160, 40),
                ONE,
            );
            part(
                b,
                f,
                Mesh::Bevel,
                v3(0, cm(322), -cm(300)),
                v3(cm(160), cm(30), cm(300)),
                shade(BUS_WHITE, 85),
                0,
            );
            for z in [cm(380), -cm(380)] {
                for x in [cm(110), -cm(110)] {
                    wheel(b, f, v3(x, cm(50), z), cm(100), cm(30));
                }
            }
            lights(b, f, w, cm(70), l);
        }
    }
}

/// Head and tail lights, lit, on a body `w` across and `l` long.
fn lights(b: &mut Batch, f: &Affine, w: i32, y: i32, l: i32) {
    for x in [w * 3 / 10, -w * 3 / 10] {
        part(
            b,
            f,
            Mesh::Cube,
            v3(x, y, l / 2 + cm(1)),
            v3(cm(40), cm(16), cm(6)),
            HEADLIGHT,
            ONE,
        );
        part(
            b,
            f,
            Mesh::Cube,
            v3(x, y + cm(4), -l / 2 - cm(1)),
            v3(cm(36), cm(14), cm(6)),
            TAILLIGHT,
            ONE,
        );
    }
}

/// The helicopters' liveries: police blue and white, news red, tour yellow.
const LIVERIES: [(Rgb, Rgb); 3] = [
    (rgb(30, 50, 120), rgb(230, 232, 236)),
    (rgb(190, 36, 40), rgb(240, 240, 240)),
    (rgb(240, 180, 30), rgb(40, 40, 44)),
];

/// A helicopter hovering with its rotor's hub at `hub`, facing `yaw`,
/// rotor turning with the tick. Its skids are about 3 m under the hub (a
/// vine hangs from under its belly).
pub fn helicopter(b: &mut Batch, hub: V3, yaw: i32, tick: u32, livery: usize, scale: i32) {
    let f = Affine::translate(hub)
        .then(&Affine::rot_y(yaw))
        .then(&Affine::scale(v3(scale, scale, scale)));
    let (body, trim) = LIVERIES[livery % LIVERIES.len()];
    // the cabin, its glass nose, the engine housing and the tail boom
    part(
        b,
        &f,
        Mesh::Geo,
        v3(0, -cm(170), cm(30)),
        v3(cm(230), cm(210), cm(400)),
        body,
        0,
    );
    part(
        b,
        &f,
        Mesh::Geo,
        v3(0, -cm(150), cm(150)),
        v3(cm(190), cm(140), cm(190)),
        GLASS,
        ONE / 6,
    );
    part(
        b,
        &f,
        Mesh::Bevel,
        v3(0, -cm(60), -cm(30)),
        v3(cm(130), cm(70), cm(220)),
        trim,
        0,
    );
    let boom = f.then(&Affine::translate(v3(0, -cm(120), -cm(380))));
    b.push(
        Mesh::Taper,
        &boom
            .then(&Affine::rot_x(deg(90) - deg(5)))
            .then(&Affine::scale(v3(cm(60), cm(520), cm(60)))),
        body,
        0,
    );
    part(
        b,
        &f,
        Mesh::Cube,
        v3(0, -cm(40), -cm(640)),
        v3(cm(12), cm(170), cm(90)),
        trim,
        0,
    );
    // the skids
    for x in [cm(100), -cm(100)] {
        part(
            b,
            &f,
            Mesh::Cube,
            v3(x, -cm(300), cm(20)),
            v3(cm(12), cm(12), cm(360)),
            TYRE,
            0,
        );
        for z in [cm(120), -cm(80)] {
            part(
                b,
                &f,
                Mesh::Cube,
                v3(x * 4 / 5, -cm(270), z),
                v3(cm(8), cm(60), cm(8)),
                TYRE,
                0,
            );
        }
    }
    // the mast and four blades, turning; the tail rotor turning across
    part(
        b,
        &f,
        Mesh::Hex,
        v3(0, -cm(10), 0),
        v3(cm(30), cm(50), cm(30)),
        TYRE,
        0,
    );
    let spin = tick as i32 * 1400;
    for k in 0..4 {
        let blade = f
            .then(&Affine::rot_y(spin + k * deg(90)))
            .then(&Affine::translate(v3(0, cm(14), cm(280))));
        b.push(
            Mesh::Cube,
            &blade.then(&Affine::scale(v3(cm(34), cm(5), cm(560)))),
            TYRE,
            0,
        );
    }
    let tail = f
        .then(&Affine::translate(v3(cm(14), -cm(20), -cm(640))))
        .then(&Affine::rot_x(spin * 2));
    b.push(
        Mesh::Cube,
        &tail.then(&Affine::scale(v3(cm(4), cm(170), cm(16)))),
        TYRE,
        0,
    );
    // navigation lights: red and green either side, a flashing white strobe
    part(
        b,
        &f,
        Mesh::Octa,
        v3(cm(120), -cm(150), 0),
        v3(cm(20), cm(20), cm(20)),
        TAILLIGHT,
        ONE,
    );
    part(
        b,
        &f,
        Mesh::Octa,
        v3(-cm(120), -cm(150), 0),
        v3(cm(20), cm(20), cm(20)),
        rgb(60, 255, 120),
        ONE,
    );
    if (tick / 30).is_multiple_of(3) {
        part(
            b,
            &f,
            Mesh::Octa,
            v3(0, -cm(30), -cm(660)),
            v3(cm(30), cm(30), cm(30)),
            rgb(255, 255, 255),
            ONE,
        );
    }
}

/// How far a hovering helicopter bobs this tick (it drifts up and down
/// half a metre or so, out of step with its neighbours).
pub fn hover(tick: u32, k: usize) -> i32 {
    mul(sin(tick as i32 * 160 + k as i32 * 13_000), cm(30))
}

/// The boats on the river.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatLook {
    Tug,
    Ferry,
    Motorboat,
}

const HULL_RED: Rgb = rgb(150, 40, 34);
const HULL_DARK: Rgb = rgb(30, 34, 40);
const FERRY_ORANGE: Rgb = rgb(240, 120, 30);
const DECK_WHITE: Rgb = rgb(230, 230, 222);

/// A boat floating at `at` (on the water), facing `yaw`, `scale` times
/// life size (Q16); it rolls a little with the tick.
pub fn boat(b: &mut Batch, look: BoatLook, at: V3, yaw: i32, scale: i32, tick: u32) {
    let roll = mul(sin(tick as i32 * 120 + at.x), deg(2));
    let f = Affine::translate(at)
        .then(&Affine::rot_y(yaw))
        .then(&Affine::rot_z(roll))
        .then(&Affine::scale(v3(scale, scale, scale)));
    match look {
        BoatLook::Tug => {
            // a black and red hull with its bow, a white wheelhouse, a stack
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(40), 0),
                v3(int(6), cm(220), int(16)),
                HULL_DARK,
                0,
            );
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(160), 0),
                v3(cm(620), cm(40), cm(1620)),
                HULL_RED,
                0,
            );
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(300), -cm(100)),
                v3(int(4), int(3), int(6)),
                DECK_WHITE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(500), cm(30)),
                v3(cm(340), cm(160), int(3)),
                DECK_WHITE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Cube,
                v3(0, cm(500), cm(160)),
                v3(cm(300), cm(70), cm(10)),
                INTERIOR,
                ONE * 3 / 5,
            );
            part(
                b,
                &f,
                Mesh::Hex,
                v3(0, cm(620), -cm(260)),
                v3(cm(120), int(3), cm(120)),
                HULL_RED,
                0,
            );
            part(
                b,
                &f,
                Mesh::Hex,
                v3(0, cm(780), -cm(260)),
                v3(cm(130), cm(40), cm(130)),
                HULL_DARK,
                0,
            );
            // the fenders round the bow
            part(
                b,
                &f,
                Mesh::Ico,
                v3(0, cm(120), cm(820)),
                v3(int(3), cm(120), cm(120)),
                HULL_DARK,
                0,
            );
            part(
                b,
                &f,
                Mesh::Octa,
                v3(0, cm(700), cm(20)),
                v3(cm(24), cm(24), cm(24)),
                TAILLIGHT,
                ONE,
            );
        }
        BoatLook::Ferry => {
            // the orange Staten Island ferry: a long hull, two lit decks
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(60), 0),
                v3(int(16), int(3), int(60)),
                FERRY_ORANGE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(320), 0),
                v3(int(14), cm(260), int(54)),
                FERRY_ORANGE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Cube,
                v3(0, cm(330), 0),
                v3(cm(1410), cm(120), int(50)),
                INTERIOR,
                ONE * 3 / 5,
            );
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(560), 0),
                v3(int(12), cm(220), int(40)),
                DECK_WHITE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Cube,
                v3(0, cm(570), 0),
                v3(cm(1210), cm(100), int(36)),
                INTERIOR,
                ONE * 3 / 5,
            );
            for z in [int(16), -int(16)] {
                part(
                    b,
                    &f,
                    Mesh::Bevel,
                    v3(0, cm(760), z),
                    v3(int(6), int(2), int(6)),
                    DECK_WHITE,
                    0,
                );
                part(
                    b,
                    &f,
                    Mesh::Hex,
                    v3(0, cm(900), z),
                    v3(cm(140), int(2), cm(140)),
                    HULL_DARK,
                    0,
                );
            }
        }
        BoatLook::Motorboat => {
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(50), 0),
                v3(cm(260), cm(90), cm(760)),
                DECK_WHITE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Wedge,
                v3(0, cm(50), cm(470)),
                v3(cm(250), cm(90), cm(200)),
                DECK_WHITE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Cube,
                v3(0, cm(30), 0),
                v3(cm(264), cm(18), cm(764)),
                BUS_BLUE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Cube,
                v3(0, cm(130), cm(120)),
                v3(cm(200), cm(60), cm(8)),
                GLASS,
                ONE / 6,
            );
            part(
                b,
                &f,
                Mesh::Octa,
                v3(0, cm(180), -cm(360)),
                v3(cm(20), cm(20), cm(20)),
                HEADLIGHT,
                ONE,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{DYNAMIC_BUFFER, INSTANCE_VALUES};

    /// The highest point any shape's middle reaches, and how many shapes.
    fn top_and_count(b: Batch) -> (i32, usize) {
        let (v, _) = b.finish(DYNAMIC_BUFFER);
        let top = v
            .chunks(INSTANCE_VALUES)
            .map(|i| i[7] + i[5].abs() / 2)
            .max()
            .unwrap();
        (top, v.len() / INSTANCE_VALUES)
    }

    #[test]
    fn each_car_stands_its_roof_high_and_is_a_few_shapes() {
        for kind in [CarKind::Cab, CarKind::Sedan, CarKind::Van, CarKind::Bus] {
            let mut b = Batch::new();
            car(&mut b, kind, &frame(V3::ZERO, 0, ONE), 1);
            let (top, n) = top_and_count(b);
            assert!(
                (top - roof(kind)).abs() < cm(25),
                "{kind:?}: {} cm against {}",
                top * 100 / ONE,
                roof(kind) * 100 / ONE
            );
            assert!(n <= 24, "{kind:?}: {n} shapes");
        }
    }
}
