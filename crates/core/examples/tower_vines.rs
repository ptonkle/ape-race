//! The tower map's vines from FROM to TO on layout K: index, place along the
//! course, anchor and bottom.
//! `cargo run --release -p tool_core --example tower_vines -- K FROM TO`
use tool_core::fx::ONE;
use tool_core::tower::{tower, tower_seeded};

fn main() {
    let a: Vec<u64> = std::env::args()
        .skip(1)
        .map(|a| a.parse().unwrap())
        .collect();
    let c = if a[0] == 0 {
        tower()
    } else {
        tower_seeded(0x70E5_0000_0001 + a[0] * 7919)
    };
    for i in a[1] as usize..=(a[2] as usize).min(c.vines.len() - 1) {
        let v = &c.vines[i];
        let m = |x: i32| x as f64 / ONE as f64;
        println!(
            "{i:4} at {:7.1} anchor ({:6.1},{:6.1},{:6.1}) len {:5.1} bottom {:6.1} slide {}",
            m(v.at),
            m(v.anchor.x),
            m(v.anchor.y),
            m(v.anchor.z),
            m(v.length),
            m(v.anchor.y - v.length),
            v.slide.is_some()
        );
    }
}
