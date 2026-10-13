//! The City's vehicles (2026-10-12, group Y): parked and moving cars and
//! yellow cabs, vans and buses, hovering helicopters, and boats on the
//! river, in the faceted low-poly style (TRIAGE T3). Each is built in its
//! own metres, +z forward and +y up, about a point on the ground (a
//! helicopter about its rotor's hub), then turned and scaled into place.
//! Nothing here keeps state: the rotors and the bobbing come from the tick.

use crate::city::{BoatKind, CarKind};
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

const HULL_RED: Rgb = rgb(150, 40, 34);
const HULL_DARK: Rgb = rgb(30, 34, 40);
const FERRY_ORANGE: Rgb = rgb(240, 120, 30);
const DECK_WHITE: Rgb = rgb(230, 230, 222);
const RUST: Rgb = rgb(110, 60, 40);
const CONTAINERS: [Rgb; 3] = [rgb(40, 90, 150), rgb(170, 50, 40), rgb(60, 120, 70)];

/// How high each boat's top stands over the water, life size (the roof
/// of a tug's wheelhouse or a ferry's cabin, a barge's cargo): what a
/// swimmer bounces off.
pub fn boat_top(kind: BoatKind) -> i32 {
    match kind {
        BoatKind::Tug => cm(400),
        BoatKind::Ferry => cm(390),
        BoatKind::Barge => cm(260),
    }
}

/// A boat floating at `at` (its middle at the waterline), facing `yaw`,
/// `scale` times life size (Q16): a red and black tugboat, an orange
/// ferry with its cabin lit, or a rusty barge stacked with containers.
pub fn boat(b: &mut Batch, kind: BoatKind, at: V3, yaw: i32, scale: i32) {
    let f = frame(at, yaw, scale);
    match kind {
        BoatKind::Tug => {
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(20), 0),
                v3(int(5), cm(180), int(14)),
                HULL_DARK,
                0,
            );
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(120), 0),
                v3(cm(520), cm(40), cm(1420)),
                HULL_RED,
                0,
            );
            part(
                b,
                &f,
                Mesh::Wedge,
                v3(0, cm(70), cm(780)),
                v3(cm(480), cm(120), cm(160)),
                HULL_DARK,
                0,
            );
            // the wheelhouse, its lit windows, the stack and the fenders
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(290), -cm(40)),
                v3(int(4), cm(220), int(6)),
                DECK_WHITE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Cube,
                v3(0, cm(320), cm(262)),
                v3(cm(340), cm(70), cm(8)),
                INTERIOR,
                ONE * 3 / 5,
            );
            part(
                b,
                &f,
                Mesh::Hex,
                v3(0, cm(460), -cm(260)),
                v3(cm(110), int(2), cm(110)),
                HULL_RED,
                0,
            );
            part(
                b,
                &f,
                Mesh::Hex,
                v3(0, cm(570), -cm(260)),
                v3(cm(120), cm(30), cm(120)),
                HULL_DARK,
                0,
            );
            for z in [int(4), -int(4)] {
                for x in [cm(270), -cm(270)] {
                    part(
                        b,
                        &f,
                        Mesh::Ico,
                        v3(x, cm(90), z),
                        v3(cm(60), cm(90), cm(90)),
                        TYRE,
                        0,
                    );
                }
            }
            part(
                b,
                &f,
                Mesh::Octa,
                v3(0, cm(420), 0),
                v3(cm(24), cm(24), cm(24)),
                TAILLIGHT,
                ONE,
            );
        }
        BoatKind::Ferry => {
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(30), 0),
                v3(int(6), cm(180), int(17)),
                FERRY_ORANGE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Wedge,
                v3(0, cm(40), cm(920)),
                v3(cm(580), cm(160), cm(160)),
                FERRY_ORANGE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(270), -cm(60)),
                v3(cm(520), cm(230), int(12)),
                DECK_WHITE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Cube,
                v3(0, cm(280), -cm(60)),
                v3(cm(530), cm(90), cm(1140)),
                INTERIOR,
                ONE * 3 / 5,
            );
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(385), cm(400)),
                v3(cm(300), cm(20), cm(300)),
                DECK_WHITE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Cube,
                v3(0, cm(120), 0),
                v3(cm(608), cm(30), cm(1710)),
                BUS_WHITE,
                0,
            );
            part(
                b,
                &f,
                Mesh::Octa,
                v3(0, cm(420), cm(400)),
                v3(cm(24), cm(24), cm(24)),
                rgb(60, 255, 120),
                ONE,
            );
        }
        BoatKind::Barge => {
            part(
                b,
                &f,
                Mesh::Bevel,
                v3(0, cm(20), 0),
                v3(int(6), cm(200), int(14)),
                RUST,
                0,
            );
            part(
                b,
                &f,
                Mesh::Cube,
                v3(0, cm(122), 0),
                v3(cm(580), cm(6), cm(1380)),
                shade(RUST, 70),
                0,
            );
            for (k, z) in [-cm(420), 0, cm(420)].into_iter().enumerate() {
                part(
                    b,
                    &f,
                    Mesh::Cube,
                    v3(0, cm(190), z),
                    v3(cm(480), cm(140), cm(400)),
                    CONTAINERS[k % CONTAINERS.len()],
                    0,
                );
            }
            part(
                b,
                &f,
                Mesh::Octa,
                v3(0, cm(290), cm(660)),
                v3(cm(24), cm(24), cm(24)),
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
