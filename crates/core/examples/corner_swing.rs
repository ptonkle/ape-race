//! The City's 90° corner swing (2026-10-12): an ape flying level along a
//! street at a speed passes `off` metres right of a corner anchor's cable,
//! `drop` below the anchor, holding grab. How long until its flat velocity
//! has turned 90° left, and how fast and high it is then.
//!
//! `cargo run --release -p tool_core --example corner_swing -- HEIGHT LENGTH [W]`
use tool_core::ape::{Ape, Controls, Mode};
use tool_core::course::Course;
use tool_core::fx::{int, ONE};
use tool_core::geom::{right_of, v3, yaw_of};
use tool_core::rules::Kind;

fn main() {
    let args: Vec<i32> = std::env::args()
        .skip(1)
        .map(|a| a.parse().unwrap())
        .collect();
    let (height, length) = (int(args[0]), int(args[1]));
    let drive = args.get(2).copied().unwrap_or(0);
    for speed in [18, 24, 30, 36] {
        let mut row = format!("{speed:>2} m/s:");
        for off in [2, 3, 4] {
            for drop in [-2, 2, 6, 10] {
                let mut c = Course::test_line(&[(int(80), height, length)]);
                c.half_width = int(300);
                let h = c.heading_at(int(80));
                let r = right_of(h);
                let left = r.neg();
                let a0 = c.vines[0].anchor;
                let start = a0.add(r.scale(int(off))).sub(h.scale(int(20)));
                let mut ape = Ape::new(
                    Kind::Chimpanzee,
                    true,
                    v3(start.x, a0.y - int(drop), start.z),
                    0,
                );
                ape.mode = Mode::Flying;
                ape.vel = h.scale(int(speed)).add(v3(0, int(drop) / 4, 0));
                let mut grabbed = None;
                let mut out = String::from("      -      ");
                for t in 0..600u32 {
                    let ctl = Controls {
                        grab: true,
                        fwd: drive,
                        yaw: yaw_of(left.add(h)),
                        ..Controls::default()
                    };
                    ape.step(&ctl, &c, t);
                    if ape.swinging().is_some() && grabbed.is_none() {
                        grabbed = Some(t);
                    }
                    if let Some(g) = grabbed {
                        if ape.swinging().is_none() {
                            break;
                        }
                        let v = ape.vel.flat();
                        if v.dot(h) <= 0 && v.dot(left) > 0 {
                            out = format!(
                                "{:>3}t{:>3}v{:>3}y",
                                (t - g),
                                ape.speed() / ONE,
                                ape.pos.y / ONE
                            );
                            break;
                        }
                    }
                }
                row += &format!(" [{off} {drop:>3}:{out}]");
            }
        }
        println!("{row}");
    }
}
