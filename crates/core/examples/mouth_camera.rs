//! Where the chase camera is as an ape of a Sunset Ruins tile race passes
//! the temple mouth: the ape's and the eye's metres along the course,
//! across it and over the temple floor, every ten ticks (SECOND-ORDER
//! RS22).
//!
//!     cargo run --release -p tool_core --example mouth_camera -- "map=ruins,seed=1,focus=3" 2250 2450
use tool_core::app::App;
use tool_core::fx::ONE;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let spec = args.get(1).cloned().unwrap_or_default();
    let from: u32 = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(2250);
    let to: u32 = args.get(3).and_then(|a| a.parse().ok()).unwrap_or(2450);
    let mut a = App::new(include_str!("../../../data/strings.json"), "", 1).unwrap();
    let mut t = from;
    while t <= to {
        a.act("screen.tile", &format!("{spec},at={t}"));
        let p = a.play.as_ref().unwrap();
        let c = &p.race.course;
        let r = c.ruins.as_ref().unwrap();
        let floor = r.chamber.centre.y;
        let f = &p.race.racers[p.focus.racer];
        let ape = f.ape.pos;
        let eye = p.chase.eye;
        let (es, ea) = c.progress(eye, c.nearest_anywhere(eye));
        let (s, aa) = c.progress(ape, c.nearest_anywhere(ape));
        let m = |v: i32| v as f64 / ONE as f64;
        println!(
            "{t:>5} ape s {:7.1} a {:6.1} y+{:6.1} | eye s {:7.1} a {:6.1} y+{:6.1} land+{:6.1} | mouth {}",
            m(s),
            m(aa),
            m(ape.y - floor),
            m(es),
            m(ea),
            m(eye.y - floor),
            m(c.terrain(eye.x, eye.z) - floor),
            r.marks.mouth / ONE
        );
        t += 10;
    }
}
