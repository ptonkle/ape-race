//! Moments in a Banana Bay Battle tile race for the map shots (step 8):
//! every second, where ape `focus` is (metres along, its mode, the ship
//! under it), the nearest ball in flight and the nearest bouncy barrel.
//!
//!     cargo run --release -p tool_core --example bay_moments -- "map=bay,seed=1" 0
use tool_core::ape::{Controls, Mode};
use tool_core::app::App;
use tool_core::fx::ONE;
use tool_core::replay;
use tool_core::rules::TICKS_PER_SECOND;

fn main() {
    let spec = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "map=bay,seed=1".into());
    let focus: usize = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut a = App::new(include_str!("../../../data/strings.json"), "", 1).unwrap();
    a.act("screen.tile", &format!("{spec},at=0"));
    let p = a.play.as_mut().unwrap();
    let step = TICKS_PER_SECOND as u32 / 2;
    while p.race.tick < TICKS_PER_SECOND as u32 * 120 {
        replay::drive(&mut p.race, &mut p.brains, p.local, Controls::default());
        let t = p.race.tick;
        if !t.is_multiple_of(step) {
            continue;
        }
        let c = &p.race.course;
        let bay = c.bay.as_deref().unwrap();
        let r = &p.race.racers[focus];
        let at = r.ape.pos;
        let m = match r.ape.mode {
            Mode::Swinging { .. } => "S",
            Mode::Flying => "F",
            _ => "G",
        };
        let ball = (0..bay.volleys.len())
            .filter_map(|i| bay.ball_at(i, t).map(|b| (b.dist(at) / ONE, i)))
            .min();
        let barrel = c
            .mushrooms
            .iter()
            .map(|mu| mu.top.dist(at) / ONE)
            .min()
            .unwrap_or(-1);
        println!(
            "{t:>6} {:>5}m {m} y{:>3} ship {:?} ball {:?} barrel {barrel}m",
            r.progress / ONE,
            at.y / ONE,
            bay.ship_under(at.x, at.z),
            ball
        );
        if r.finish.is_some() && t > 600 {
            break;
        }
    }
}
