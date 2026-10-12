//! How long a typical swing lasts: an ape flying in level at `speed` grabs a
//! vine `drop` below its anchor, holds W, and is timed until its velocity
//! points 40° up (a natural moment to let go).
use tool_core::ape::{Ape, Controls, Mode};
use tool_core::course::Course;
use tool_core::fx::{int, ONE};
use tool_core::geom::v3;
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let args: Vec<i32> = std::env::args()
        .skip(1)
        .map(|a| a.parse().unwrap())
        .collect();
    let (anchor, length) = (int(args[0]), int(args[1]));
    for speed in [15, 22, 30] {
        let _ = 0;
        for drop in [10, 18, 26, 32] {
            let c = Course::test_line(&[(int(60), anchor, length)]);
            let a = c.vines[0].anchor;
            let mut ape = Ape::new(
                Kind::Chimpanzee,
                true,
                v3(a.x, a.y - int(drop), a.z - int(3)),
                0,
            );
            ape.mode = Mode::Flying;
            ape.vel = v3(0, 0, int(speed));
            let mut held_from = None;
            let mut out = String::from("-");
            for t in 0..TICKS_PER_SECOND as u32 * 6 {
                let ctl = Controls {
                    fwd: 1,
                    grab: true,
                    ..Controls::default()
                };
                ape.step(&ctl, &c, t);
                if ape.swinging().is_some() && held_from.is_none() {
                    held_from = Some(t);
                }
                if let Some(f) = held_from {
                    let v = ape.vel;
                    // 40° up: vy > 0.84 * horizontal
                    if v.y > 0 && v.y as i64 * 100 > 119 * v.len_xz() as i64 {
                        out = format!(
                            "{:.2}s at {} m/s",
                            (t - f) as f32 / 120.0,
                            ape.speed() / ONE
                        );
                        break;
                    }
                }
            }
            println!("speed {speed:>2} drop {drop:>2}: {out}");
        }
    }
}
