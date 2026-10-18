//! The store's props: balloon bunches, the bouncy-ball bin, produce tables
//! and their misters, cardboard display bins, the start's stock cart and
//! the finish's water-and-soda pallet.

use super::{hash, part, Detail, Fr, KRAFT, PARTY, STEEL, WOOD};
use crate::fx::{self, cm, int, mul, sin, ONE, QUARTER, TURN};
use crate::geom::{facing, v3, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{mix, placed, rgb, rod, shade, Batch, Rgb};
use crate::rules::TICKS_PER_SECOND as TPS;

const STRING: Rgb = rgb(236, 236, 232);

pub(super) fn balloons(b: &mut Batch, anchor: V3, tick: u32, k: u32) {
    let n = 3 + (k % 3) as i32;
    let t = tick as i32;
    for i in 0..n {
        let colour = PARTY[((k + i as u32 * 3) % 8) as usize];
        let around = i * TURN / n + (hash(k) % 4096) as i32;
        let spread = if n == 3 {
            int(1) + cm(40)
        } else {
            int(1) + cm(80)
        };
        let bob = mul(sin(t.wrapping_mul(TURN / (TPS * 2)) + i * TURN / 5), cm(35));
        let sway = mul(
            sin(t.wrapping_mul(TURN / (TPS * 3)) + i * TURN * 2 / 7),
            cm(40),
        );
        let knot = anchor.add(v3(
            mul(sin(around), spread) + sway,
            int(5) + (i % 2) * cm(120) + bob,
            mul(fx::cos(around), spread),
        ));
        b.push(Mesh::Cyl, &rod(anchor, knot, cm(6)), STRING, 0);
        b.push(
            Mesh::Cone,
            &placed(knot.add(v3(0, cm(15), 0)), 0, v3(cm(50), cm(45), cm(50))),
            shade(colour, 80),
            0,
        );
        let mid = knot.add(v3(0, cm(40) + cm(190), 0));
        let ball = placed(mid, around, v3(cm(320), cm(380), cm(320)));
        b.push(Mesh::Sphere, &ball, colour, ONE / 8);
        b.push_halo(
            Mesh::Sphere,
            &placed(mid, 0, v3(cm(340), cm(400), cm(340))),
            mix(colour, rgb(255, 255, 255), ONE / 2),
            ONE / 4,
        );
    }
}

pub(super) fn ball_bin(b: &mut Batch, top: V3, radius: i32) {
    let h = top.y;
    let foot = v3(top.x, 0, top.z);
    let fr = Fr::new(foot, 0);
    let frame = rgb(54, 58, 64);
    let r2 = radius * 2;
    // the mesh's shadow inside, a foot ring and a top rim
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, part(h, 47, 100), 0),
        v3(part(r2, 95, 100), part(h, 92, 100), part(r2, 95, 100)),
        rgb(36, 36, 42),
        0,
    );
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, part(h, 4, 100), 0),
        v3(r2, part(h, 8, 100), r2),
        frame,
        0,
    );
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, h - cm(30), 0),
        v3(r2 + cm(10), cm(60), r2 + cm(10)),
        rgb(232, 60, 50),
        0,
    );
    // the wire: upright bars round the side
    for i in 0..8 {
        let a = i * TURN / 8;
        let at = v3(mul(sin(a), radius), 0, mul(fx::cos(a), radius));
        fr.rod(b, Mesh::Cube, at, at.add(v3(0, h, 0)), cm(30), frame, 0);
    }
    // balls behind the wire, two rings, and heaped on top
    let d = (radius * 2 / 5).min(int(3));
    let mut c = 0;
    let mut ball = |b: &mut Batch, at: V3| {
        let colour = PARTY[(hash(c) % 8) as usize];
        c += 1;
        fr.put(b, Mesh::Sphere, at, v3(d, d, d), colour, ONE / 6);
    };
    for ring in 0..2 {
        let y = part(h, 30 + ring * 32, 100);
        for i in 0..9 {
            let a = i * TURN / 9 + ring * TURN / 18 + TURN / 16;
            let r = radius - d / 2 - cm(20);
            ball(b, v3(mul(sin(a), r), y, mul(fx::cos(a), r)));
        }
    }
    let rings = [
        (part(radius, 72, 100), 9, 30),
        (part(radius, 38, 100), 5, 10),
    ];
    for (r, n, lift) in rings {
        for i in 0..n {
            let a = i * TURN / n + r;
            ball(
                b,
                v3(
                    mul(sin(a), r),
                    h - d / 2 + part(d, lift, 100),
                    mul(fx::cos(a), r),
                ),
            );
        }
    }
    ball(b, v3(0, h - d / 2 + part(d, 45, 100), 0));
}

/// One kind of produce: its mesh, its size across, its colours.
struct Produce {
    mesh: Mesh,
    size: i32,
    colours: [Rgb; 3],
    /// a banana bunch: rods, not balls
    bunch: bool,
}

fn produce(stock: u8) -> Produce {
    let p = |mesh, size, colours| Produce {
        mesh,
        size,
        colours,
        bunch: false,
    };
    match stock % 7 {
        0 => p(
            Mesh::Sphere,
            cm(100),
            [rgb(196, 30, 40), rgb(176, 24, 36), rgb(150, 196, 60)],
        ),
        1 => p(
            Mesh::Geo,
            cm(100),
            [rgb(246, 142, 30), rgb(236, 128, 24), rgb(250, 160, 50)],
        ),
        2 => Produce {
            bunch: true,
            ..p(
                Mesh::Taper,
                cm(120),
                [rgb(244, 212, 64), rgb(232, 200, 56), rgb(220, 210, 80)],
            )
        },
        3 => p(
            Mesh::Sphere,
            cm(180),
            [rgb(214, 180, 120), rgb(200, 168, 110), rgb(206, 222, 150)],
        ),
        4 => p(
            Mesh::Blob,
            cm(180),
            [rgb(110, 170, 70), rgb(132, 190, 84), rgb(96, 150, 60)],
        ),
        5 => p(
            Mesh::Sphere,
            cm(90),
            [rgb(214, 40, 30), rgb(200, 34, 28), rgb(226, 60, 36)],
        ),
        _ => p(
            Mesh::Ico,
            cm(100),
            [rgb(212, 30, 30), rgb(250, 200, 30), rgb(60, 150, 50)],
        ),
    }
}

/// How high a produce pile stands over its table's top.
pub(super) const PILE_MOST: i32 = int(3);

pub(super) fn table(b: &mut Batch, foot: V3, radius: i32, height: i32, stock: u8) {
    let fr = Fr::new(foot, 0);
    let r2 = radius * 2;
    // the table: a top with a raised rim, a skirt, a pedestal, a foot
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, height - cm(30), 0),
        v3(r2, cm(60), r2),
        WOOD,
        0,
    );
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, height, 0),
        v3(r2 + cm(20), cm(100), r2 + cm(20)),
        shade(WOOD, 80),
        0,
    );
    let skirt = part(height, 25, 100);
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, height - cm(60) - skirt / 2, 0),
        v3(part(r2, 92, 100), skirt, part(r2, 92, 100)),
        shade(WOOD, 88),
        0,
    );
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, height / 2, 0),
        v3(part(r2, 28, 100), height, part(r2, 28, 100)),
        shade(WOOD, 70),
        0,
    );
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, cm(30), 0),
        v3(part(r2, 60, 100), cm(60), part(r2, 60, 100)),
        shade(WOOD, 76),
        0,
    );
    // the pile: a mound in the produce's colour, the produce over it
    let pr = produce(stock);
    let d = pr.size;
    let mound = (int(2)).min(PILE_MOST - part(d, 3, 4) - cm(10));
    let mr = part(radius, 90, 100);
    fr.put(
        b,
        Mesh::Sphere,
        v3(0, height, 0),
        v3(mr * 2, mound * 2, mr * 2),
        shade(pr.colours[0], 70),
        0,
    );
    let surface = |r: i32| {
        let q = fx::div(r, mr).min(ONE);
        height + mul(mound, fx::sqrt(ONE - mul(q, q)))
    };
    let mut c: u32 = stock as u32 * 97;
    let mut rings: [(i32, i32); 4] = [(0, 0); 4];
    for (j, f) in [80, 56, 30, 0].into_iter().enumerate() {
        let r = part(radius, f, 100);
        let fit = if r == 0 {
            1
        } else {
            (mul(r, int(6)) / (if pr.bunch { d * 2 } else { d + d / 8 }).max(1))
                .clamp(1, if pr.bunch { 9 } else { 14 })
        };
        rings[j] = (r, fit);
    }
    for (j, (r, n)) in rings.into_iter().enumerate() {
        for i in 0..n {
            let a = i * TURN / n + j as i32 * TURN * 37 / 100;
            let at = v3(mul(sin(a), r), 0, mul(fx::cos(a), r));
            let y = surface(r) + part(d, 22, 100);
            let colour = pr.colours[(hash(c) % 3) as usize];
            c += 1;
            if pr.bunch {
                // a bunch: two bananas fanned along the ring
                let along = facing(a + QUARTER, 0);
                for f in [-1, 1] {
                    let tilt = facing(a + QUARTER + f * TURN / 20, 0);
                    let mid = at.add(v3(0, y, 0)).add(along.scale(f * cm(30)));
                    let half = tilt.scale(cm(110));
                    b.push(
                        Mesh::Taper,
                        &rod(
                            fr.pt(mid.sub(half).add(v3(0, cm(25), 0))),
                            fr.pt(mid.add(half)),
                            cm(42),
                        ),
                        colour,
                        ONE / 10,
                    );
                }
            } else {
                fr.put(
                    b,
                    pr.mesh,
                    at.add(v3(0, y, 0)),
                    v3(d, part(d, 92, 100), d),
                    colour,
                    ONE / 10,
                );
            }
        }
    }
}

pub(super) fn bin(b: &mut Batch, foot: V3, radius: i32, height: i32, stock: u8) {
    let fr = Fr::new(foot, 0);
    let r2 = radius * 2;
    let aisle = &super::AISLES[(stock as usize) % super::AISLES.len()];
    let first = aisle.products[0];
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, height / 2, 0),
        v3(r2, height, r2),
        KRAFT,
        0,
    );
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, part(height, 55, 100), 0),
        v3(r2 + cm(6), part(height, 30, 100), r2 + cm(6)),
        first.colours[1],
        0,
    );
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, part(height, 55, 100) + part(height, 15, 100), 0),
        v3(r2 + cm(8), part(height, 4, 100), r2 + cm(8)),
        first.colours[2],
        0,
    );
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, height - cm(25), 0),
        v3(r2 + cm(10), cm(50), r2 + cm(10)),
        shade(KRAFT, 78),
        0,
    );
    // the fill, then the product heaped in it
    fr.put(
        b,
        Mesh::Cyl,
        v3(0, height - cm(10), 0),
        v3(part(r2, 96, 100), cm(30), part(r2, 96, 100)),
        shade(first.colours[0], 70),
        0,
    );
    let fits: Vec<&super::Product> = aisle
        .products
        .iter()
        .filter(|p| {
            let s = p.size_m();
            s.x.max(s.z) < part(radius, 110, 100)
        })
        .collect();
    if fits.is_empty() {
        fr.put(
            b,
            Mesh::Sphere,
            v3(0, height, 0),
            v3(part(r2, 90, 100), part(radius, 60, 100), part(r2, 90, 100)),
            first.colours[0],
            0,
        );
        return;
    }
    let spots = [
        (0, 0, 0),
        (55, 0, 1),
        (55, TURN / 6, 1),
        (55, TURN / 3, 1),
        (55, TURN / 2, 1),
        (55, TURN * 2 / 3, 1),
        (55, TURN * 5 / 6, 1),
        (20, TURN / 4, 2),
    ];
    for (i, (f, a, layer)) in spots.into_iter().enumerate() {
        let p = fits[i % fits.len()];
        let s = p.size_m();
        let yaw = (hash(stock as u32 * 31 + i as u32) % TURN as u32) as i32;
        let r = part(radius, f, 100);
        let mid = v3(mul(sin(a), r), 0, mul(fx::cos(a), r));
        let sunk = match layer {
            0 => part(s.y, 30, 100),
            1 => part(s.y, 55, 100),
            _ => -part(s.y, 15, 100),
        };
        let base = mid.add(v3(0, height - sunk, 0));
        let front = base.add(facing(yaw, 0).scale(s.z / 2));
        super::products::product(b, p, fr.pt(front), yaw, Detail::Block);
    }
}

pub(super) fn mister(b: &mut Batch, at: V3, tick: u32, period: u32, phase: u32) {
    if period == 0 {
        return;
    }
    let t = tick.wrapping_add(phase) % period;
    let dur = (period / 3).clamp(1, TPS as u32 * 3 / 2);
    if t >= dur {
        return;
    }
    let k = (t as i32 * ONE) / dur as i32;
    let out = mul(k, 2 * ONE - k);
    let fade = ONE - k;
    let mist = rgb(240, 248, 255);
    b.push_halo(
        Mesh::Sphere,
        &placed(at, 0, v3(cm(150), cm(150), cm(150))),
        mist,
        mul(fade, ONE * 9 / 10),
    );
    for i in 0..5 {
        let a = i * TURN / 5 + (hash(phase) % 4096) as i32;
        let c = at.add(v3(
            mul(mul(sin(a), int(3)), out),
            -mul(int(2) + cm(50), out),
            mul(mul(fx::cos(a), int(3)), out),
        ));
        let s = cm(100) + mul(int(4), out);
        b.push_halo(
            Mesh::Sphere,
            &placed(c, 0, v3(s, part(s, 80, 100), s)),
            mist,
            mul(fade, ONE * 3 / 5),
        );
    }
}

/// A shipping case's printed bands.
const CASE_PRINT: [Rgb; 4] = [
    rgb(200, 40, 40),
    rgb(30, 80, 160),
    rgb(40, 130, 60),
    rgb(240, 238, 230),
];

pub(super) fn stock_cart(b: &mut Batch, centre: V3, yaw: i32, size: V3) {
    let fr = Fr::new(centre, yaw);
    let (hx, hz) = (size.x / 2, size.z / 2);
    let wheel = (size.y / 5).min(crate::grocery::real_cm(10));
    let deck = cm(50).min(size.y / 8);
    let deck_top = wheel + deck;
    // casters, the deck
    for (sx, sz) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
        fr.turned(
            b,
            Mesh::Cyl,
            v3(sx * (hx - cm(60)), wheel / 2, sz * (hz - wheel)),
            Affine::rot_z(QUARTER),
            v3(wheel, cm(40), wheel),
            rgb(40, 40, 44),
            0,
        );
    }
    fr.put(
        b,
        Mesh::Cube,
        v3(0, wheel + deck / 2, 0),
        v3(size.x, deck, size.z),
        STEEL,
        0,
    );
    // the U at each end: two posts and a bar, up to the load's top
    let post = cm(30);
    for sz in [-1, 1] {
        let z = sz * (hz - post / 2);
        for sx in [-1, 1] {
            fr.put(
                b,
                Mesh::Cube,
                v3(sx * (hx - post / 2), (deck_top + size.y) / 2, z),
                v3(post, size.y - deck_top, post),
                STEEL,
                0,
            );
        }
        fr.put(
            b,
            Mesh::Cube,
            v3(0, size.y - post / 2, z),
            v3(size.x - post * 2, post, post),
            shade(STEEL, 90),
            0,
        );
    }
    // the load: cases in a grid, their tops flush with the load's top
    let lz = size.z - post * 2 - cm(10);
    let ly = size.y - deck_top;
    let count = |len: i32, case: i32, most: i32| ((len + case / 2) / case).clamp(1, most);
    let nx = count(size.x, crate::grocery::real_cm(45), 2);
    let ny = count(ly, crate::grocery::real_cm(30), 3);
    let nz = count(lz, crate::grocery::real_cm(35), 5);
    let (cx, cy, cz) = (size.x / nx, ly / ny, lz / nz);
    let gap = cm(10);
    fr.put(
        b,
        Mesh::Cube,
        v3(0, deck_top + (ly - gap) / 2, 0),
        v3(size.x - gap * 2, ly - gap * 2, lz - gap * 2),
        shade(KRAFT, 50),
        0,
    );
    for ix in 0..nx {
        for iy in 0..ny {
            for iz in 0..nz {
                let n = (ix * 31 + iy * 7 + iz * 13) as u32;
                let x = -hx + cx * ix + cx / 2;
                let y = deck_top + cy * iy + cy / 2 + gap / 2;
                let z = -lz / 2 + cz * iz + cz / 2;
                let tone = 88 + (hash(n) % 18) as i32;
                fr.put(
                    b,
                    Mesh::Cube,
                    v3(x, y, z),
                    v3(cx - gap, cy - gap, cz - gap),
                    shade(KRAFT, tone),
                    0,
                );
                // a printed band on the cases at the sides, every other one
                let outer = if ix == 0 {
                    -1
                } else if ix == nx - 1 {
                    1
                } else {
                    0
                };
                if outer != 0 && hash(n + 5).is_multiple_of(2) {
                    fr.put(
                        b,
                        Mesh::Cube,
                        v3(outer * (hx - gap / 2), y, z),
                        v3(cm(6), part(cy, 28, 100), part(cz, 78, 100)),
                        CASE_PRINT[(hash(n) % 4) as usize],
                        0,
                    );
                }
            }
        }
    }
}

const WATER: Rgb = rgb(110, 172, 226);
const WATER_BAND: Rgb = rgb(28, 78, 168);
const SODA: Rgb = rgb(200, 30, 40);
const SODA_BAND: Rgb = rgb(242, 242, 238);
const PALLET_WOOD: Rgb = rgb(178, 142, 98);

pub(super) fn pallet(b: &mut Batch, front: V3, yaw: i32) {
    let size = crate::ravinemodels::SPHINX_SIZE;
    let fr = Fr::new(front, yaw);
    let hx = size.x / 2;
    let gap = cm(40);
    let long = (size.z - gap) / 2;
    let base = crate::grocery::real_cm(15);
    let cap = cm(50);
    let layers = 5;
    let lh = (size.y - base - cap) / layers;
    for k in 0..2 {
        let z0 = -(long + gap) * k;
        let mid = z0 - long / 2;
        // the pallet: three stringers under a deck
        for sx in [-1, 0, 1] {
            fr.put(
                b,
                Mesh::Cube,
                v3(sx * (hx - cm(80)), (base - cm(50)) / 2, mid),
                v3(cm(160), base - cm(50), long),
                shade(PALLET_WOOD, 80),
                0,
            );
        }
        fr.put(
            b,
            Mesh::Cube,
            v3(0, base - cm(25), mid),
            v3(size.x, cm(50), long),
            PALLET_WOOD,
            0,
        );
        // the layers of cases: water low on the front pallet, soda over it
        for l in 0..layers {
            let water = if k == 0 { l < 3 } else { l >= 2 };
            let (body, band) = if water {
                (WATER, WATER_BAND)
            } else {
                (SODA, SODA_BAND)
            };
            let y = base + lh * l + lh / 2;
            let (w, d) = (size.x - cm(20), long - cm(20));
            fr.put(b, Mesh::Cube, v3(0, y, mid), v3(w, lh - cm(12), d), body, 0);
            fr.put(
                b,
                Mesh::Cube,
                v3(0, y, mid),
                v3(w + cm(8), part(lh, 34, 100), d + cm(8)),
                band,
                0,
            );
            let seam = shade(body, 62);
            for sx in [-1, 1] {
                fr.put(
                    b,
                    Mesh::Cube,
                    v3(sx * size.x / 6, y, mid),
                    v3(cm(14), lh - cm(20), d + cm(4)),
                    seam,
                    0,
                );
            }
            fr.put(
                b,
                Mesh::Cube,
                v3(0, y, mid),
                v3(w + cm(4), lh - cm(20), cm(14)),
                seam,
                0,
            );
        }
        // the bottle caps on top
        let top = base + lh * layers;
        let water_top = k == 1;
        for ix in 0..4 {
            for iz in 0..3 {
                let x = -hx + size.x * (2 * ix + 1) / 8;
                let z = z0 - long * (2 * iz + 1) / 6;
                fr.put(
                    b,
                    Mesh::Cyl,
                    v3(x, top + cap / 2, z),
                    v3(cm(100), cap, cm(100)),
                    if water_top {
                        rgb(236, 240, 246)
                    } else {
                        rgb(150, 20, 30)
                    },
                    0,
                );
            }
        }
        // the shrink-wrap's sheen
        b.push_halo(
            Mesh::Cube,
            &fr.aff(
                v3(0, (base + size.y) / 2, mid),
                &Affine::IDENTITY,
                v3(size.x, size.y - base, long),
            ),
            rgb(230, 240, 250),
            ONE / 8,
        );
    }
}
