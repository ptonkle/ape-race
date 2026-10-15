//! Speeds on each map: King Kong and the Sloth alone (mean and top m/s,
//! time on the ground), and on the Ski Slope an ape that only skis
//! (presses W toward the run's way on, never grabs): its speed down the
//! run and how far it gets in 150 s.
//!
//! `cargo run --release -p tool_core --example ski_speeds`
use tool_core::ai::{trial_at, Difficulty};
use tool_core::ape::{Controls, Mode};
use tool_core::fx::ONE;
use tool_core::geom::yaw_of;
use tool_core::map::Map;
use tool_core::race::{Driver, Entry, Phase, Race};
use tool_core::rules::{Kind, TICKS_PER_SECOND};

fn main() {
    let f = |v: i32| v as f64 / ONE as f64;
    let secs = |t: u32| t as f64 / TICKS_PER_SECOND as f64;
    let limit = 150 * TICKS_PER_SECOND as u32;
    for map in Map::ALL {
        for level in [Difficulty::KingKong, Difficulty::Sloth] {
            let t = trial_at(map.course(), &[Kind::Chimpanzee], &[level], true, limit)[0];
            println!(
                "{:<7} {:<10} finish {:>6.1} s  mean {:>5.1}  top {:>5.1} m/s  ground {:>5.1} s",
                map.key(),
                level.key(),
                t.finish.map(secs).unwrap_or(f64::NAN),
                f(t.mean_speed),
                f(t.top_speed),
                secs(t.ground_ticks)
            );
        }
    }
    // skiing only
    let entries = [Entry {
        name: String::new(),
        kind: Kind::Chimpanzee,
        driver: Driver::Ai,
        palette: 0,
        costume: tool_core::costume::Costume::Own,
    }];
    let mut race = Race::new(Map::Ski.course(), &entries, true, 7);
    let mut top = 0;
    let mut sum = 0i64;
    let mut n = 0i64;
    let mut marks = Vec::new();
    while race.clock < limit && race.phase != Phase::Done {
        let a = &race.racers[0].ape;
        let p = race.racers[0].progress;
        let ahead = race.course.point(p + 30 * ONE).sub(a.pos).flat();
        let c = Controls {
            fwd: 1,
            yaw: yaw_of(ahead),
            ..Controls::default()
        };
        race.step(&[c]);
        let a = &race.racers[0].ape;
        if race.running() && a.mode == Mode::Grounded && a.on_snow(&race.course) {
            top = top.max(a.speed());
            sum += a.speed() as i64;
            n += 1;
        }
        if race.running() && race.clock.is_multiple_of(10 * TICKS_PER_SECOND as u32) {
            marks.push(format!(
                "{:.0}s {}m {:.1}",
                secs(race.clock),
                race.racers[0].progress / ONE,
                f(a.speed())
            ));
        }
    }
    println!(
        "ski only: finish {:?} s, at {} m, mean {:.1} m/s, top {:.1} m/s",
        race.racers[0].finish.map(secs),
        race.racers[0].progress / ONE,
        sum as f64 / n.max(1) as f64 / ONE as f64,
        f(top)
    );
    println!("  {}", marks.join(" | "));
}
