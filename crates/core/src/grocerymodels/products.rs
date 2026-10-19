//! The products: twelve shapes, eight printed fronts. Every product stands
//! in its own size box (across `w`, up `h`, back from its front `d`): the
//! frame's origin is the middle of its base's front edge, the body runs
//! back to z = −d, and print stands at most `PROUD` (3 cm) in front.

use super::{part, Detail, Face, Fr, Front, Product, Shape, FULL_MOST};
use crate::fx::cm;
use crate::geom::{v3, V3};
use crate::mesh::Mesh;
use crate::render::{mix, rgb, shade, Batch, Rgb};

/// A can's rims.
const SILVER: Rgb = rgb(196, 200, 204);

pub(super) fn product(b: &mut Batch, p: &Product, foot: V3, yaw: i32, detail: Detail) {
    let s = p.size_m();
    let fr = Fr::new(foot, yaw);
    let [body, label, accent] = p.colours;
    let (w, h, d) = (s.x, s.y, s.z);
    if detail == Detail::Block {
        fr.put(b, Mesh::Cube, v3(0, h / 2, -d / 2), s, body, 0);
        return;
    }
    let at = |y: i32| v3(0, y, -d / 2);
    // a curved front falls back at its sides: print reaches in to meet it
    let round = part(d, 3, 20);
    // a band round a round body, a little proud of it
    let wrap = |k: i32| v3(w + cm(4), k, d + cm(4));
    match p.shape {
        Shape::Box => {
            fr.put(b, Mesh::Cube, at(h / 2), s, body, 0);
            let mut top = h;
            let used = if p.front == Front::Lid {
                let lid = part(h, 1, 8);
                top = h - lid;
                fr.put(
                    b,
                    Mesh::Cube,
                    at(h - lid / 2 + cm(1)),
                    v3(w + cm(2), lid + cm(2), d + cm(2)),
                    accent,
                    0,
                );
                2
            } else {
                1
            };
            let face = Face::front(&fr, top / 2, 0, w, top, cm(3));
            let n = print_front(b, &face, p.front, (label, accent, body), 3);
            debug_assert!(used + n <= FULL_MOST);
        }
        Shape::Bag => {
            let crimp = part(h, 7, 100);
            fr.put(
                b,
                Mesh::Bevel,
                at(h / 2),
                v3(part(w, 98, 100), h - crimp * 2 + cm(4), d),
                body,
                0,
            );
            let seam = v3(w, crimp + cm(2), part(d, 2, 5));
            let sc = shade(body, 82);
            fr.put(b, Mesh::Cube, at(crimp / 2), seam, sc, 0);
            fr.put(b, Mesh::Cube, at(h - crimp / 2), seam, sc, 0);
            let face = Face::front(&fr, h / 2, 0, part(w, 3, 4), part(h, 3, 5), cm(4));
            print_front(b, &face, p.front, (label, accent, body), 3);
        }
        Shape::Can => {
            let rim = part(h, 7, 100);
            fr.put(
                b,
                Mesh::Cyl,
                at(h / 2),
                v3(w, h - rim * 2 + cm(2), d),
                body,
                0,
            );
            let rims = v3(part(w, 94, 100), rim, part(d, 94, 100));
            fr.put(b, Mesh::Cyl, at(rim / 2), rims, SILVER, 0);
            fr.put(b, Mesh::Cyl, at(h - rim / 2), rims, SILVER, 0);
            fr.put(b, Mesh::Cyl, at(h / 2), wrap(part(h, 11, 20)), label, 0);
            let face = Face::front(&fr, h / 2, cm(2), part(w, 11, 20), part(h, 1, 2), round);
            print_front(b, &face, p.front, (accent, body, label), 2);
        }
        Shape::Bottle => {
            let body_h = part(h, 55, 100);
            fr.put(b, Mesh::Cyl, at(body_h / 2), v3(w, body_h, d), body, 0);
            fr.put(
                b,
                Mesh::Cone,
                at(body_h + part(h, 15, 100)),
                v3(w, part(h, 30, 100), d),
                body,
                0,
            );
            fr.put(
                b,
                Mesh::Cyl,
                at(part(h, 86, 100)),
                v3(part(w, 32, 100), part(h, 12, 100), part(d, 32, 100)),
                body,
                0,
            );
            fr.put(
                b,
                Mesh::Cyl,
                at(part(h, 96, 100)),
                v3(part(w, 40, 100), part(h, 8, 100), part(d, 40, 100)),
                accent,
                0,
            );
            let band_h = part(h, 30, 100);
            fr.put(b, Mesh::Cyl, at(part(h, 28, 100)), wrap(band_h), label, 0);
            let face = Face::front(
                &fr,
                part(h, 28, 100),
                cm(2),
                part(w, 1, 2),
                part(band_h, 3, 4),
                round,
            );
            print_front(b, &face, p.front, (accent, body, label), 1);
        }
        Shape::Jar | Shape::Tub | Shape::Canister => {
            let lid = part(h, if p.shape == Shape::Canister { 10 } else { 14 }, 100);
            let inset = if p.shape == Shape::Canister { 100 } else { 95 };
            let bw = part(w, inset, 100);
            let bd = part(d, inset, 100);
            fr.put(
                b,
                Mesh::Cyl,
                v3(0, (h - lid) / 2, -d / 2),
                v3(bw, h - lid + cm(2), bd),
                body,
                0,
            );
            fr.put(
                b,
                Mesh::Cyl,
                at(h - lid / 2),
                v3(w, lid, d),
                if p.shape == Shape::Jar && p.front != Front::Lid {
                    shade(accent, 90)
                } else {
                    accent
                },
                0,
            );
            let mid = (h - lid) / 2;
            if p.shape == Shape::Tub {
                // a tub is printed all over: print straight on its body
                let face = Face::front(
                    &fr,
                    mid,
                    -(d - bd) / 2,
                    part(bw, 3, 5),
                    part(h - lid, 3, 5),
                    round,
                );
                print_front(b, &face, p.front, (label, accent, body), 3);
            } else {
                let band_h = part(h - lid, 1, 2);
                fr.put(
                    b,
                    Mesh::Cyl,
                    v3(0, mid, -d / 2),
                    v3(bw + cm(4), band_h, bd + cm(4)),
                    label,
                    0,
                );
                let face = Face::front(
                    &fr,
                    mid,
                    cm(2) - (d - bd) / 2,
                    part(bw, 11, 20),
                    part(band_h, 4, 5),
                    round,
                );
                print_front(b, &face, p.front, (accent, body, label), 3);
            }
        }
        Shape::Jug => {
            let body_h = part(h, 72, 100);
            fr.put(b, Mesh::Bevel, at(body_h / 2), v3(w, body_h, d), body, 0);
            fr.put(
                b,
                Mesh::Sphere,
                at(body_h),
                v3(part(w, 96, 100), part(h, 26, 100), part(d, 96, 100)),
                body,
                0,
            );
            let neck = v3(part(w, 22, 100), part(h, 88, 100), -part(d, 40, 100));
            fr.put(
                b,
                Mesh::Cyl,
                neck,
                v3(part(w, 26, 100), part(h, 10, 100), part(d, 26, 100)),
                body,
                0,
            );
            fr.put(
                b,
                Mesh::Cyl,
                v3(neck.x, part(h, 96, 100), neck.z),
                v3(part(w, 30, 100), part(h, 8, 100), part(d, 30, 100)),
                accent,
                0,
            );
            fr.rod(
                b,
                Mesh::Cube,
                v3(-part(w, 26, 100), part(h, 86, 100), -part(d, 55, 100)),
                v3(-part(w, 36, 100), part(h, 52, 100), -part(d, 80, 100)),
                part(w, 10, 100),
                shade(body, 92),
                0,
            );
            let face = Face::front(
                &fr,
                part(h, 38, 100),
                0,
                part(w, 7, 10),
                part(h, 3, 10),
                cm(4),
            );
            print_front(b, &face, p.front, (label, accent, body), 1);
        }
        Shape::Case => {
            let body_h = part(h, 86, 100);
            fr.put(b, Mesh::Cube, at(body_h / 2), v3(w, body_h, d), body, 0);
            for i in -1..=1 {
                fr.put(
                    b,
                    Mesh::Cyl,
                    v3(part(w, i, 3), body_h + part(h, 7, 100), -d / 2),
                    v3(part(w, 1, 4), part(h, 14, 100), part(d, 1, 3)),
                    accent,
                    0,
                );
            }
            let face = Face::front(&fr, body_h / 2, 0, w, body_h, cm(3));
            print_front(b, &face, p.front, (label, accent, body), 2);
        }
        Shape::Pouch => {
            fr.put(b, Mesh::Bevel, at(h / 2), s, body, 0);
            if h * 3 < w {
                // lying flat: the seal along its back edge, print on top
                fr.put(
                    b,
                    Mesh::Cube,
                    v3(0, h / 2, -d + part(d, 4, 100)),
                    v3(w, part(h, 6, 10), part(d, 8, 100)),
                    shade(body, 85),
                    0,
                );
                let face = Face::top(&fr, h, -d / 2, part(w, 4, 5), part(d, 7, 10));
                print_front(b, &face, p.front, (label, accent, body), 3);
            } else {
                let seal = part(h, 8, 100);
                fr.put(
                    b,
                    Mesh::Cube,
                    at(h - seal / 2),
                    v3(w, seal, part(d, 6, 10)),
                    accent,
                    0,
                );
                let face = Face::front(
                    &fr,
                    part(h, 45, 100),
                    0,
                    part(w, 4, 5),
                    part(h, 7, 10),
                    cm(3),
                );
                print_front(b, &face, p.front, (label, accent, body), 3);
            }
        }
        Shape::Tray => {
            let base = part(h, 70, 100);
            fr.put(b, Mesh::Cube, at(base / 2), v3(w, base, d), body, 0);
            fr.put(
                b,
                Mesh::Cube,
                at(base + (h - base) / 2 + cm(1)),
                v3(w + cm(2), h - base + cm(2), d + cm(2)),
                label,
                0,
            );
            let face = Face::top(&fr, h + cm(2), -d / 2, part(w, 4, 5), part(d, 7, 10));
            print_front(b, &face, p.front, (accent, body, label), 3);
        }
        Shape::Sack => {
            let low = part(h, 60, 100);
            fr.put(b, Mesh::Bevel, at(low / 2), v3(w, low, d), body, 0);
            fr.put(
                b,
                Mesh::Bevel,
                at(part(h, 74, 100)),
                v3(part(w, 82, 100), part(h, 32, 100), part(d, 72, 100)),
                shade(body, 95),
                0,
            );
            fr.put(
                b,
                Mesh::Cube,
                at(part(h, 94, 100)),
                v3(part(w, 86, 100), part(h, 12, 100), part(d, 44, 100)),
                shade(body, 80),
                0,
            );
            let face = Face::front(
                &fr,
                part(h, 32, 100),
                0,
                part(w, 8, 10),
                part(low, 8, 10),
                cm(4),
            );
            print_front(b, &face, p.front, (label, accent, body), 3);
        }
    }
}

/// The front `front` printed on `face` in (label, accent, body) colours,
/// in at most `most` shapes (the first ones of its pattern). `most` 0 is
/// read as 1. Returns the shapes drawn.
fn print_front(
    b: &mut Batch,
    face: &Face,
    front: Front,
    cols: (Rgb, Rgb, Rgb),
    most: usize,
) -> usize {
    let (label, accent, body) = cols;
    let most = most.max(1);
    let (w, h) = (face.w, face.h);
    let mut n = 0;
    let mut go = |b: &mut Batch, f: &dyn Fn(&mut Batch)| {
        if n < most {
            f(b);
            n += 1;
        }
    };
    match front {
        Front::Band => {
            go(b, &|b| face.slab(b, (0, 0), w, part(h, 38, 100), label, 0));
            go(b, &|b| {
                face.slab(b, (0, part(h, 25, 100)), w, part(h, 6, 100), accent, 0)
            });
            go(b, &|b| {
                face.slab(
                    b,
                    (-part(w, 22, 100), 0),
                    part(w, 24, 100),
                    part(h, 24, 100),
                    accent,
                    1,
                )
            });
        }
        Front::Stripe => {
            go(b, &|b| {
                face.slab(b, (0, part(h, 26, 100)), w, part(h, 14, 100), label, 0)
            });
            go(b, &|b| {
                face.slab(b, (0, -part(h, 26, 100)), w, part(h, 14, 100), label, 0)
            });
            go(b, &|b| face.slab(b, (0, 0), w, part(h, 8, 100), accent, 0));
        }
        Front::Split => {
            go(b, &|b| {
                face.slab(b, (0, -h / 4), w, part(h, 50, 100), label, 0)
            });
            go(b, &|b| face.slab(b, (0, 0), w, part(h, 6, 100), accent, 1));
            go(b, &|b| {
                face.slab(
                    b,
                    (part(w, 22, 100), h / 4),
                    part(w, 30, 100),
                    part(h, 20, 100),
                    accent,
                    0,
                )
            });
        }
        Front::Dots => {
            let dia = part(w.min(h), 30, 100);
            go(b, &|b| {
                face.dot(b, (-part(w, 24, 100), part(h, 14, 100)), dia, label, 0)
            });
            go(b, &|b| {
                face.dot(b, (part(w, 24, 100), part(h, 14, 100)), dia, accent, 0)
            });
            go(b, &|b| face.dot(b, (0, -part(h, 22, 100)), dia, label, 0));
        }
        Front::Window => {
            go(b, &|b| {
                face.slab(b, (0, 0), part(w, 80, 100), part(h, 64, 100), label, 0)
            });
            let clear = mix(body, rgb(255, 255, 255), crate::fx::ONE / 3);
            go(b, &|b| {
                face.slab(
                    b,
                    (0, part(h, 5, 100)),
                    part(w, 50, 100),
                    part(h, 32, 100),
                    clear,
                    1,
                )
            });
            go(b, &|b| {
                face.slab(b, (0, -part(h, 42, 100)), w, part(h, 10, 100), accent, 0)
            });
        }
        Front::Chevron => {
            let arm = part(w, 46, 100).min(part(h, 80, 100));
            let t = part(h, 12, 100).min(part(w, 14, 100));
            let ang = crate::fx::deg(32);
            go(b, &|b| {
                face.mark(
                    b,
                    Mesh::Cube,
                    (-part(arm, 42, 100), 0),
                    arm,
                    t,
                    -ang,
                    label,
                    0,
                )
            });
            go(b, &|b| {
                face.mark(
                    b,
                    Mesh::Cube,
                    (part(arm, 42, 100), 0),
                    arm,
                    t,
                    ang,
                    label,
                    0,
                )
            });
            go(b, &|b| {
                face.slab(b, (0, part(h, 38, 100)), w, part(h, 12, 100), accent, 0)
            });
        }
        Front::Lid => {
            go(b, &|b| {
                face.slab(
                    b,
                    (0, -part(h, 10, 100)),
                    part(w, 90, 100),
                    part(h, 50, 100),
                    label,
                    0,
                )
            });
            go(b, &|b| {
                face.slab(
                    b,
                    (0, -part(h, 10, 100)),
                    part(w, 90, 100),
                    part(h, 8, 100),
                    accent,
                    1,
                )
            });
        }
        Front::Plain => {
            go(b, &|b| {
                face.slab(
                    b,
                    (0, part(h, 10, 100)),
                    part(w, 40, 100),
                    part(h, 25, 100),
                    label,
                    0,
                )
            });
        }
    }
    n
}
