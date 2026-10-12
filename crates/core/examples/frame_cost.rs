use std::time::Instant;
fn main() {
    let strings = std::fs::read_to_string("data/strings.json").unwrap();
    let mut a = tool_core::app::App::new(&strings, "", 1).unwrap();
    a.act("menu.race", "");
    a.act("menu.start", "");
    let mut now = 0i64;
    a.key("KeyW", true);
    for f in 0..600 {
        if f == 200 {
            a.key("Space", true);
        }
        now += 16_667;
        let t = Instant::now();
        a.advance(now);
        let _ = a.ui();
        let _ = a.frame();
        let e = t.elapsed();
        if f % 50 == 0 || e.as_millis() > 5 {
            let p = a.play.as_ref().unwrap();
            let ape = &p.race.racers[0].ape;
            println!(
                "{f}: {:?} pos {:?} mode {:?}",
                e,
                (ape.pos.x >> 16, ape.pos.y >> 16, ape.pos.z >> 16),
                ape.mode
            );
        }
    }
}
