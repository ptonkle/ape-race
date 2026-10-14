//! Every vine of the Ski Slope: index, metres along, metres right of the
//! line, its anchor and end above the snow, and what it hangs from.
//!
//! `cargo run --release -p tool_core --example ski_vines [-- SEED]`
use tool_core::fx::ONE;
use tool_core::ski::{ski, ski_seeded};

fn main() {
    let k: u64 = std::env::args()
        .nth(1)
        .map(|a| a.parse().unwrap())
        .unwrap_or(0);
    let c = if k == 0 {
        ski()
    } else {
        ski_seeded(0x5C1_5EED_0001 + k * 7919)
    };
    let lay = c.ski.as_ref().unwrap();
    for (i, v) in c.vines.iter().enumerate() {
        let (s, side) = c.progress(v.anchor, c.nearest_anywhere(v.anchor));
        let g = c.terrain(v.anchor.x, v.anchor.z);
        println!(
            "#{i:<4} {:>6.1} m {:>+6.1}  anchor {:>4.1}  end {:>4.1}  {:?}",
            s as f64 / ONE as f64,
            side as f64 / ONE as f64,
            (v.anchor.y - g) as f64 / ONE as f64,
            (v.anchor.y - v.length - g) as f64 / ONE as f64,
            lay.hangers[i].kind
        );
    }
}
