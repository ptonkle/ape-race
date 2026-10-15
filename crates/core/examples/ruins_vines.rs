//! Every vine of Sunset Ruins: index, metres along, metres right of the
//! line, its anchor and end above the land, and what it hangs from.
//!
//! `cargo run --release -p tool_core --example ruins_vines [-- SEED]`
use tool_core::fx::ONE;
use tool_core::ruins::{ruins, ruins_seeded};

fn main() {
    let k: u64 = std::env::args()
        .nth(1)
        .map(|a| a.parse().unwrap())
        .unwrap_or(0);
    let c = if k == 0 {
        ruins()
    } else {
        ruins_seeded(0x2B1_5E75_0001 + k * 7919)
    };
    let lay = c.ruins.as_ref().unwrap();
    for (i, v) in c.vines.iter().enumerate() {
        let (s, side) = c.progress(v.anchor, c.nearest_anywhere(v.anchor));
        let g = c.terrain(v.anchor.x, v.anchor.z);
        println!(
            "#{i:<4} {:>6.1} m {:>+6.1}  anchor {:>4.1}  end {:>4.1}  {:?}",
            s as f64 / ONE as f64,
            side as f64 / ONE as f64,
            (v.anchor.y - g) as f64 / ONE as f64,
            (v.anchor.y - v.length - g) as f64 / ONE as f64,
            lay.hangers[i]
        );
    }
}
