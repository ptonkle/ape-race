//! The City's vines between two points along it: index, where, across,
//! anchor height above the street, length, bottom.
//!
//! `cargo run --release -p tool_core --example city_vines -- SEED FROM TO`
use tool_core::city::{city, city_seeded};
use tool_core::course::ground;
use tool_core::fx::ONE;

fn main() {
    let a: Vec<i64> = std::env::args().skip(1).map(|a| a.parse().unwrap()).collect();
    let c = if a[0] == 0 {
        city()
    } else {
        city_seeded(0xC17E_5EED_0001 + a[0] as u64 * 7919)
    };
    let m = |v: i32| v as f64 / ONE as f64;
    for (i, v) in c.vines.iter().enumerate() {
        if (v.at as i64) < a[1] * ONE as i64 || (v.at as i64) > a[2] * ONE as i64 {
            continue;
        }
        let k = c.nearest_anywhere(v.anchor);
        let (_, side) = c.progress(v.anchor, k);
        let g = ground(v.anchor.x, v.anchor.z);
        println!(
            "{i:>4} at {:>7.1} side {:>6.1} anchor {:>5.1} len {:>5.1} bottom {:>5.1}",
            m(v.at),
            m(side),
            m(v.anchor.y - g),
            m(v.length),
            m(v.anchor.y - v.length - g)
        );
    }
}
