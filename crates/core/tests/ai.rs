//! The AI on the jungle course (PLAN P12, and P7's race length).

use tool_core::ai::{trial, Trial};
use tool_core::course::Course;
use tool_core::fx::ONE;
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn secs(ticks: u32) -> f64 {
    ticks as f64 / TICKS_PER_SECOND as f64
}

fn report(label: &str, trials: &[Trial]) {
    println!("{label}");
    for t in trials {
        println!(
            "  {:<11} finish {:>9}  at {:>5.0} m  mean {:>5.1} m/s  top {:>5.1} m/s  grabs {:>3}  landings {:>2}  ground {:>5.1} s",
            format!("{:?}", t.kind),
            t.finish
                .map(|f| format!("{:.2} s", secs(f)))
                .unwrap_or("-".into()),
            t.progress as f64 / ONE as f64,
            t.mean_speed as f64 / ONE as f64,
            t.top_speed as f64 / ONE as f64,
            t.grabs,
            t.landings,
            secs(t.ground_ticks),
        );
    }
}

const LIMIT: u32 = 150 * TICKS_PER_SECOND as u32;

/// Found by the AI: an ape near the top of its swing, almost still, could
/// hang there for ever, holding W or not (SECOND-ORDER A3). Gravity must
/// bring it back down.
#[test]
fn a_slow_swing_never_hangs_at_its_turning_point() {
    use tool_core::ape::{Ape, Controls, Mode};
    use tool_core::fx::{deg, int, mul};
    use tool_core::geom::v3;
    let c = Course::test_line(&[(int(50), int(30), int(12))]);
    let a = c.vines[0].anchor;
    let rope = int(10);
    let angle = deg(66);
    for fwd in [0, 1] {
        for creep in [ONE / 8, ONE / 6, ONE / 4] {
            let (s, co) = (tool_core::fx::sin(angle), tool_core::fx::cos(angle));
            let mut ape = Ape::new(
                Kind::Chimpanzee,
                true,
                a.add(v3(0, -mul(rope, co), mul(rope, s))),
                0,
            );
            ape.mode = Mode::Swinging { vine: 0, rope };
            // creeping up the arc, slower than a tick of gravity takes away
            ape.vel = v3(0, mul(creep, s), mul(creep, co));
            let start = ape.pos.y;
            for t in 0..TICKS_PER_SECOND as u32 {
                let ctl = Controls {
                    fwd,
                    grab: true,
                    ..Controls::default()
                };
                ape.step(&ctl, &c, t);
            }
            assert!(
                ape.pos.y < start - int(1),
                "fwd {fwd} creep {creep}: {} -> {}",
                start / 655,
                ape.pos.y / 655
            );
        }
    }
}

#[test]
fn one_ai_finishes_the_jungle_in_one_to_two_minutes() {
    let t = trial(Course::jungle(), &[Kind::Chimpanzee], true, LIMIT);
    report("one AI, equal stats", &t);
    let f = t[0].finish.expect("the AI finishes");
    assert!(
        (60 * TICKS_PER_SECOND as u32..=120 * TICKS_PER_SECOND as u32).contains(&f),
        "{:.2} s",
        secs(f)
    );
}

#[test]
#[ignore]
fn speed_report_every_kind_alone_and_four_together() {
    for k in Kind::ALL {
        let t = trial(Course::jungle(), &[k], false, LIMIT);
        report("alone", &t);
    }
    let t = trial(
        Course::jungle(),
        &[Kind::Gibbon, Kind::Gorilla, Kind::Lemur, Kind::Orangutan],
        false,
        LIMIT,
    );
    report("four together", &t);
}
