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

/// Finish times over eight layouts of the jungle (its seed and seven more),
/// three kinds each: for judging AI and layout changes beyond one course.
#[test]
#[ignore]
fn finish_times_over_course_seeds() {
    let mut times = Vec::new();
    for k in 0..8u64 {
        let course = if k == 0 {
            Course::jungle()
        } else {
            Course::jungle_seeded(0x0A9E_2ACE_0001 + k * 7919)
        };
        let mut row = Vec::new();
        for kind in [Kind::Chimpanzee, Kind::Gorilla, Kind::Gibbon] {
            let t = trial(course.clone(), &[kind], kind == Kind::Chimpanzee, LIMIT);
            row.push(t[0].finish.map(secs));
            times.push(t[0].finish.map(secs).unwrap_or(f64::INFINITY));
        }
        println!("layout {k}: {row:.1?}");
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "best {:.1} s, median {:.1} s, worst {:.1} s",
        times[0],
        times[times.len() / 2],
        times[times.len() - 1]
    );
}
