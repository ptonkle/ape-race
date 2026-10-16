//! Banana Bay Battle's layout in numbers: its length, the ships by kind
//! and side, the ropes, barrels, volleys and fighters.
//!
//! `cargo run --release -p tool_core --example bay_layout [-- SEED_K]`
use tool_core::bay::{bay, bay_seeded, ShipKind, Side};
use tool_core::fx::ONE;

fn main() {
    let k: u64 = std::env::args()
        .nth(1)
        .map(|a| a.parse().unwrap())
        .unwrap_or(0);
    let c = if k == 0 {
        bay()
    } else {
        bay_seeded(0xBA7_BA77_0001 + k * 7919)
    };
    let b = c.bay.as_ref().unwrap();
    println!("length {} m, {} vines", c.length / ONE, c.vines.len());
    for kind in [
        ShipKind::Galleon,
        ShipKind::Frigate,
        ShipKind::Brig,
        ShipKind::Sloop,
    ] {
        let n = b.ships.iter().filter(|s| s.kind == kind).count();
        println!("{kind:?}: {n}");
    }
    let pirates = b.ships.iter().filter(|s| s.side == Side::Pirate).count();
    println!(
        "pirates {pirates}, navy {}; boundary {}, bouncy {}, volleys {}, fighters {}",
        b.ships.len() - pirates,
        b.boundary.len(),
        b.bouncy.len(),
        b.volleys.len(),
        b.fighters.len()
    );
    for (i, s) in b.ships.iter().enumerate() {
        let (at, _) = c.progress(s.at, c.nearest_anywhere(s.at));
        println!(
            "{i:>2} {:?} {:?} at {} m ({}–{} m), {} m long",
            s.kind,
            s.side,
            at / ONE,
            (at - s.length / 2) / ONE,
            (at + s.length / 2) / ONE,
            s.length / ONE
        );
    }
    // the gaps between ropes along the course, over 20 m
    let mut at: Vec<i32> = c.vines.iter().map(|v| v.at).collect();
    at.sort();
    let gaps: Vec<String> = at
        .windows(2)
        .filter(|w| w[1] - w[0] > 20 * ONE)
        .map(|w| format!("{}→{}", w[0] / ONE, w[1] / ONE))
        .collect();
    println!("rope gaps over 20 m: {}", gaps.join(", "));
}
