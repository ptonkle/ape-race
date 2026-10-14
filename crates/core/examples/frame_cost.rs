use std::time::Instant;
fn main() {
    let strings = std::fs::read_to_string("data/strings.json").unwrap();
    let mut a = tool_core::app::App::new(&strings, "", 1).unwrap();
    // `-- city`: race the City (the `flag.map` test hook)
    if let Some(map) = std::env::args().nth(1) {
        a.act("flag.map", &map);
    }
    a.act("menu.race", "");
    a.act("menu.start", "");
    let mut now = 0i64;
    a.key("KeyW", true);
    // `-- ski 6000`: this many frames (default 600), then the worst step
    let frames: u32 = std::env::args()
        .nth(2)
        .and_then(|n| n.parse().ok())
        .unwrap_or(600);
    let mut worst = std::time::Duration::ZERO;
    let mut worst_at = 0;
    let mut steps = Vec::new();
    for f in 0..frames {
        if f == 200 {
            a.key("Space", true);
        }
        now += 16_667;
        let t = Instant::now();
        a.advance(now);
        let stepped = t.elapsed();
        if stepped > worst {
            worst = stepped;
            worst_at = f;
        }
        steps.push(stepped);
        let _ = a.ui();
        let _ = a.frame();
        let e = t.elapsed();
        if f % 50 == 0 || e.as_millis() > 5 {
            let p = a.play.as_ref().unwrap();
            let ape = &p.race.racers[0].ape;
            println!(
                "{f}: {:?} (stepping {:?}) pos {:?} mode {:?}",
                e,
                stepped,
                (ape.pos.x >> 16, ape.pos.y >> 16, ape.pos.z >> 16),
                ape.mode
            );
        }
    }
    steps.sort();
    let pct = |p: usize| steps[(steps.len() * p / 100).min(steps.len() - 1)];
    println!(
        "median {:?}, 99th {:?}, frames over 2 ms {}",
        pct(50),
        pct(99),
        steps.iter().filter(|s| s.as_micros() > 2000).count()
    );
    println!("worst step {worst:?} (frame {worst_at}) over {frames} frames");
}
