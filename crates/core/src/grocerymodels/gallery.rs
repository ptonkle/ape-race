//! The model gallery's grocery keys (`?gallery=<key>`), drawn under the
//! store's strip lights.

use super::{
    aisle_sign, ball_bin, balloons, banner, bin, cart, dept_sign, mister, pallet, product, shopper,
    stock_cart, table, Detail, Front, Product, Shape, AISLES,
};
use crate::fx::{int, ONE};
use crate::geom::{v3, yaw_of, Affine, V3};
use crate::mesh::Mesh;
use crate::render::{rgb, rod, Batch, Rgb};
use crate::rules::TICKS_PER_SECOND as TPS;
use crate::scene::Look;

pub const GALLERY: [&str; 32] = [
    "product",
    "fronts",
    "productgrid",
    "products-0",
    "products-1",
    "products-2",
    "products-3",
    "products-4",
    "products-5",
    "products-6",
    "products-7",
    "products-8",
    "products-9",
    "products-10",
    "products-11",
    "products-12",
    "products-13",
    "shopper",
    "shoppers",
    "shoppercart",
    "balloons",
    "ballbin",
    "table",
    "tables",
    "bin",
    "mister",
    "aislesign",
    "banner",
    "deptsigns",
    "cart",
    "stockcart",
    "pallet",
];

/// The store's light, bright, for the gallery.
pub fn gallery_look() -> Look {
    Look {
        sky: rgb(236, 234, 228),
        ..crate::groceryscape::look()
    }
}

/// One product of each shape (the first in the catalogue).
fn one_of_each() -> Vec<Product> {
    Shape::ALL
        .iter()
        .filter_map(|s| {
            AISLES
                .iter()
                .flat_map(|a| a.products.iter())
                .find(|p| p.shape == *s)
                .copied()
        })
        .collect()
}

/// A plain carton printed with `front`, coloured by its index.
pub(super) fn sample(shape: Shape, front: Front, i: usize) -> Product {
    const C: [Rgb; 6] = [
        rgb(214, 48, 40),
        rgb(36, 96, 186),
        rgb(250, 206, 44),
        rgb(52, 150, 72),
        rgb(240, 236, 224),
        rgb(120, 60, 140),
    ];
    Product {
        name: "sample",
        shape,
        size: [20, 26, 12],
        colours: [C[i % 6], C[(i + 2) % 6], C[(i + 4) % 6]],
        front,
    }
}

/// The products of the key, if it shows a row of products.
fn row(key: &str) -> Option<Vec<Product>> {
    if key == "product" {
        return Some(one_of_each());
    }
    if key == "fronts" {
        return Some(
            Front::ALL
                .iter()
                .enumerate()
                .map(|(i, f)| sample(Shape::Box, *f, i))
                .collect(),
        );
    }
    let n: usize = key.strip_prefix("products-")?.parse().ok()?;
    AISLES.get(n).map(|a| a.products.to_vec())
}

const ROW_GAP: i32 = int(1);

fn row_len(ps: &[Product]) -> i32 {
    ps.iter().map(|p| p.size_m().x + ROW_GAP).sum::<i32>() - ROW_GAP
}

/// The middle and rough radius of a gallery model, metres.
pub fn framing(key: &str) -> (V3, i32) {
    if let Some(ps) = row(key) {
        let tall = ps.iter().map(|p| p.size_m().y).max().unwrap_or(ONE);
        return (v3(0, tall / 2, 0), (row_len(&ps) / 2).max(tall));
    }
    match key {
        "productgrid" => (v3(0, int(20), 0), int(34)),
        "shopper" => (v3(0, int(13), 0), int(14)),
        "shoppers" => (v3(0, int(10), 0), int(34)),
        "shoppercart" => (v3(0, int(10), int(5)), int(15)),
        "balloons" => (v3(0, int(6), 0), int(6)),
        "ballbin" => (v3(0, int(5), 0), int(9)),
        "table" => (v3(0, int(9), 0), int(14)),
        "tables" => (v3(0, int(8), 0), int(50)),
        "bin" => (v3(0, int(5), 0), int(22)),
        "mister" => (v3(0, int(8), 0), int(13)),
        "aislesign" => (v3(0, 0, 0), int(16)),
        "banner" => (v3(0, 0, 0), int(10)),
        "deptsigns" => (v3(0, 0, 0), int(26)),
        "cart" => (v3(0, int(6), 0), int(8)),
        "stockcart" => (v3(0, int(5), 0), int(16)),
        "pallet" => (v3(0, int(9), 0), int(22)),
        _ => (V3::ZERO, int(10)),
    }
}

/// Draws gallery model `key` in frame `f` (turning about up) at `tick`.
pub fn gallery(b: &mut Batch, key: &str, f: &Affine, tick: u32) {
    let yaw = yaw_of(f.c2);
    let at = |x: i32, y: i32, z: i32| f.apply(v3(x, y, z));
    let walk = ((tick as i32) * ONE / TPS) % ONE;
    if let Some(ps) = row(key) {
        let mut x = -row_len(&ps) / 2;
        for p in &ps {
            let w = p.size_m().x;
            product(b, p, at(x + w / 2, 0, 0), yaw, Detail::Full);
            x += w + ROW_GAP;
        }
        return;
    }
    match key {
        "productgrid" => {
            // every shape (rows, bottom up) with every front (across)
            for (r, s) in Shape::ALL.iter().enumerate() {
                for (c, fr) in Front::ALL.iter().enumerate() {
                    let p = sample(*s, *fr, r + c);
                    let x = int(8) * c as i32 - int(28);
                    product(b, &p, at(x, int(4) * r as i32, 0), yaw, Detail::Full);
                }
            }
        }
        "shopper" => {
            let feet = at(0, 0, 0);
            let end = at(-int(3), int(23), int(2));
            let anchor = at(-int(3), int(31), int(2));
            shopper(b, feet, yaw, 1, walk, Some(end), false);
            b.push(Mesh::Cyl, &rod(end, anchor, crate::fx::cm(20)), VINE, 0);
            balloons(b, anchor, tick, 1);
        }
        "shoppers" => {
            for look in 0..7u8 {
                let x = int(10) * look as i32 - int(30);
                let stride = if look == 0 { 0 } else { walk };
                shopper(b, at(x, 0, 0), yaw, look, stride, None, false);
            }
        }
        "shoppercart" => {
            shopper(b, at(0, 0, -int(6)), yaw, 2, walk, None, true);
        }
        "balloons" => {
            let anchor = at(0, int(2), 0);
            b.push(
                Mesh::Cyl,
                &rod(at(0, -int(6), 0), anchor, crate::fx::cm(30)),
                VINE,
                0,
            );
            balloons(b, anchor, tick, 4);
        }
        "ballbin" => ball_bin(b, at(0, int(8), 0), int(7)),
        "table" => table(b, at(0, 0, 0), int(10), int(9), 0),
        "tables" => {
            for s in 0..7u8 {
                let x = int(24) * (s as i32 % 4) - int(36);
                let z = if s < 4 { int(12) } else { -int(12) };
                table(b, at(x, 0, z), int(10), int(9), s);
            }
        }
        "bin" => {
            for (i, s) in [3u8, 6, 9, 12].into_iter().enumerate() {
                bin(b, at(int(14) * i as i32 - int(21), 0, 0), int(6), int(9), s);
            }
        }
        "mister" => {
            table(b, at(0, 0, 0), int(10), int(9), 4);
            let period = TPS as u32 * 3;
            mister(b, at(0, int(15), 0), tick, period, 0);
            mister(b, at(int(5), int(15), int(3)), tick, period, period / 2);
        }
        "aislesign" => {
            aisle_sign(b, at(-int(8), 0, 0), yaw, 3);
            aisle_sign(b, at(int(8), 0, 0), yaw, 12);
        }
        "banner" => {
            banner(b, at(-int(4), 0, 0), yaw, 0);
            banner(b, at(int(4), 0, 0), yaw, 16);
        }
        "deptsigns" => {
            for d in 0..6u8 {
                let x = int(15) * (d as i32 % 3) - int(15);
                let y = if d < 3 { int(6) } else { -int(6) };
                dept_sign(b, at(x, y, 0), yaw, d, int(13));
            }
        }
        "cart" => cart(b, at(0, 0, 0), yaw),
        "stockcart" => stock_cart(b, at(0, 0, 0), yaw, v3(int(10), int(9), int(24))),
        "pallet" => {
            let size = crate::ravinemodels::SPHINX_SIZE;
            pallet(b, at(0, 0, size.z / 2), yaw);
        }
        _ => {}
    }
}

/// A stand-in for the race's vine under the balloons (drawn by the scene).
const VINE: Rgb = crate::groceryscape::VINE;
