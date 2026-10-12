//! Scratch: a play-by-play of one AI's race.

use tool_core::ai::{think, Brain};
use tool_core::ape::{Happening, Mode};
use tool_core::course::{ground, Course};
use tool_core::fx::{int, ONE};
use tool_core::race::{Driver, Entry, Race};
use tool_core::rules::Kind;

fn m(v: i32) -> f64 {
    v as f64 / ONE as f64
}
fn m64(v: i64) -> f64 {
    v as f64 / ONE as f64
}

fn env(k: &str, d: u32) -> u32 {
    std::env::var(k)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(d)
}

fn line_race() -> Race {
    let gap = env("GAP", 28) as i32;
    let side = env("SIDE", 0) as i32;
    let len = env("LEN", 14) as i32;
    let hi = env("HI", 28) as i32;
    let mut gd = tool_core::fx::Dice(env("SEED", 1) as u64 * 991);
    let mut s = 30;
    let vines: Vec<(i32, i32, i32)> = (0..env("NV", 12) as i32)
        .map(|_| {
            let here = s;
            s += if env("GVAR", 0) == 1 {
                if gd.below(5) == 0 {
                    gd.range(34, 38)
                } else {
                    gd.range(24, 31)
                }
            } else {
                gap
            };
            (int(here), int(hi), int(len))
        })
        .collect();
    let mut course = Course::test_line(&vines);
    for (k, v) in course.vines.iter_mut().enumerate() {
        let wave = env("WAVE", 0) as f64;
        let off = if wave > 0.0 {
            (side as f64
                * (std::f64::consts::TAU * k as f64 * gap as f64 / wave).sin()
                * ONE as f64) as i32
        } else if k % 2 == 0 {
            int(side)
        } else {
            -int(side)
        };
        v.anchor.x += off;
        if env("HVAR", 0) == 1 {
            let mut d = tool_core::fx::Dice(k as u64 * 77 + env("SEED", 1) as u64);
            v.anchor.y = int(d.range(23, 33));
            v.length = int(d.range(10, 17));
        }
    }
    let entries = vec![Entry {
        name: String::new(),
        kind: Kind::Chimpanzee,
        driver: Driver::Ai,
    }];
    let low = env("LOW", 0) as i32;
    if low > 0 {
        // the first vine is a low one, `low` metres before the second
        let v1 = course.vines[1];
        course.vines[0].at = v1.at - int(low);
        course.vines[0].anchor = course.point(v1.at - int(low));
        course.vines[0].anchor.y = int(env("LOWA", 13) as i32);
        course.vines[0].length = int(env("LOWL", 10) as i32);
    }
    let mut race = Race::new(course, &entries, true, 7);
    race.phase = tool_core::race::Phase::Running;
    let a0 = race.course.vines[0].anchor;
    let a = &mut race.racers[0].ape;
    if low > 0 {
        a.pos = tool_core::geom::v3(a0.x, int(3), a0.z);
        a.vel = tool_core::geom::v3(0, 0, int(env("V", 8) as i32));
        a.mode = Mode::Swinging {
            vine: 0,
            rope: a0.y - int(3),
        };
    } else {
        a.pos = tool_core::geom::v3(a0.x, int(hi - len + 2), a0.z - int(3));
        a.vel = tool_core::geom::v3(0, 0, int(env("V", 20) as i32));
        a.mode = Mode::Flying;
    }
    a.hint = race.course.nearest_anywhere(a.pos);
    race
}

fn jungle_race(n: usize) -> Race {
    let entries: Vec<Entry> = (0..n)
        .map(|i| Entry {
            name: String::new(),
            kind: Kind::ALL[(i + env("KIND", 2) as usize) % 6],
            driver: Driver::Ai,
        })
        .collect();
    let seed = env("CSEED", 0) as u64;
    let course = if seed == 0 {
        Course::jungle()
    } else {
        Course::jungle_seeded(0x0A9E_2ACE_0001 + seed * 7919)
    };
    Race::new(course, &entries, env("EQUAL", 1) == 1, 7)
}

fn race() -> Race {
    if env("LINE", 0) == 1 {
        line_race()
    } else {
        jungle_race(env("APES", 1) as usize)
    }
}

#[test]
#[ignore]
fn trace() {
    let mut race = race();
    let n = race.racers.len();
    let mut brains = vec![Brain::new(); n];
    let until = env("UNTIL", 150 * 120);
    while race.clock < until && race.phase != tool_core::race::Phase::Done {
        let c: Vec<_> = (0..n).map(|i| think(&race, i, &mut brains[i])).collect();
        race.step(&c);
        let r = &race.racers[0];
        let a = &r.ape;
        let k = r.ape.hint.min(race.course.samples() - 1);
        let side = race.course.progress(a.pos, k).1;
        let every = env("EVERY", 0);
        if every > 0 && race.clock % every == 0 {
            println!(
                "{:7.2} {:?} pos {:.1} {:.1} {:.1} vel {:.1} {:.1} {:.1} ctl {:?}",
                race.clock as f64 / 120.0,
                a.mode,
                m(a.pos.x),
                m(a.pos.y),
                m(a.pos.z),
                m(a.vel.x),
                m(a.vel.y),
                m(a.vel.z),
                c[0]
            );
        }
        for h in &a.happened {
            let t = race.clock as f64 / 120.0;
            let extra = match (h, a.mode) {
                (Happening::Grab, Mode::Swinging { vine, rope }) => {
                    let v = &race.course.vines[vine];
                    format!(
                        "vine {vine} at {:.0} bottom {:.1} rope {:.1}",
                        m(v.at),
                        m(v.anchor.y - v.length - ground(v.anchor.x, v.anchor.z)),
                        m(rope)
                    )
                }
                _ => format!("target {:?} next {:?}", brains[0].target, brains[0].next),
            };
            println!(
                "{t:7.2} {:<8} prog {:5.0} side {:5.1} y {:4.1} speed {:4.1} {extra}",
                format!("{h:?}"),
                m(r.progress),
                m(side),
                m(a.pos.y - ground(a.pos.x, a.pos.z)),
                m(a.speed())
            );
        }
    }
    println!(
        "finish {:?} replans {} hits {} misses {} give-ups {}",
        race.racers[0].finish.map(|f| f as f64 / 120.0),
        brains[0].replans,
        brains[0].hits,
        brains[0].misses,
        brains[0].give_ups
    );
}

/// Mean and worst finish over course seeds and kinds.
#[test]
#[ignore]
fn seeds() {
    use tool_core::ai::trial;
    let n = env("SEEDS", 8) as u64;
    let mut times = Vec::new();
    let mut dnf = 0;
    let mut line = String::new();
    for seed in 0..n {
        let course = if seed == 0 {
            Course::jungle()
        } else {
            Course::jungle_seeded(0x0A9E_2ACE_0001 + seed * 7919)
        };
        for kind in [Kind::Chimpanzee, Kind::Gorilla, Kind::Gibbon] {
            let t = trial(course.clone(), &[kind], kind == Kind::Chimpanzee, 150 * 120);
            match t[0].finish {
                Some(f) => {
                    times.push(f as f64 / 120.0);
                    line += &format!("{:.0} ", f as f64 / 120.0);
                }
                None => {
                    dnf += 1;
                    line += &format!("[{:.0}m] ", m(t[0].progress));
                }
            }
        }
        line += "| ";
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean = times.iter().sum::<f64>() / times.len().max(1) as f64;
    println!(
        "SEEDS mean {mean:.1} median {:.1} worst {:.1} dnf {dnf}",
        times.get(times.len() / 2).copied().unwrap_or(0.0),
        times.last().copied().unwrap_or(0.0)
    );
    println!("{line}");
}

#[test]
#[ignore]
fn dump() {
    let seed = env("CSEED", 0) as u64;
    let c = if seed == 0 {
        Course::jungle()
    } else {
        Course::jungle_seeded(0x0A9E_2ACE_0001 + seed * 7919)
    };
    let (lo, hi) = (env("FROM", 0) as i32, env("TO", 1800) as i32);
    for (i, v) in c.vines.iter().enumerate() {
        if v.at < int(lo) || v.at > int(hi) {
            continue;
        }
        let k = c.nearest_anywhere(v.anchor);
        let side = c.progress(v.anchor, k).1;
        let g = ground(v.anchor.x, v.anchor.z);
        println!(
            "vine {i:3} at {:5.0} side {:5.1} anchor {:4.1} bottom {:4.1} len {:4.1}{}",
            m(v.at),
            m(side),
            m(v.anchor.y - g),
            m(v.anchor.y - v.length - g),
            m(v.length),
            if v.slide.is_some() { " sliding" } else { "" }
        );
    }
}

#[test]
#[ignore]
fn hover() {
    let mut race = race();
    let mut brain = Brain::new();
    let at = env("AT", 200);
    while race.clock < at {
        let c = think(&race, 0, &mut brain);
        race.step(&[c]);
    }
    let mut a = race.racers[0].ape.clone();
    let an = race.course.vines[0].anchor;
    for t in 0..env("N", 20) {
        let c = think(&race, 0, &mut brain);
        let before = a.vel;
        a.step(&c, &race.course, race.tick + 1 + t);
        let d = a.pos.sub(an);
        println!(
            "rel z {:.4} y {:.4} len {:.4} | vel {:.4} {:.4} -> {:.4} {:.4} | fwd {} yaw {}",
            m(d.z),
            m(d.y),
            m(d.len()),
            m(before.z),
            m(before.y),
            m(a.vel.z),
            m(a.vel.y),
            c.fwd,
            c.yaw
        );
        race.racers[0].ape = a.clone();
    }
}

#[test]
#[ignore]
fn release_scan() {
    let at = env("AT", 32 * 120);
    let target = env("TGT", 14) as usize;
    let mut race = race();
    let mut brain = Brain::new();
    while race.clock < at {
        let c = think(&race, 0, &mut brain);
        race.step(&[c]);
    }
    let a = &race.racers[0].ape;
    println!(
        "mode {:?} pos y {:.1} speed {:.1} prog {:.0}",
        a.mode,
        m(a.pos.y),
        m(a.speed()),
        m(race.racers[0].progress)
    );
    let v = &race.course.vines[target];
    println!(
        "target at {:.0} anchor y {:.1} len {:.1}",
        m(v.at),
        m(v.anchor.y),
        m(v.length)
    );
    let (cands, next, rows) = tool_core::ai::debug_release(&race, 0, target, env("N", 240));
    println!("cands {cands:?} next {next:?}");
    for r in rows {
        println!(
            "{} y {:5.1} side {:5.1} prog {:5.1} | v along {:5.1} side {:5.1} up {:5.1} | hit {} after {} speed {:.1} rope {:.1} score {:.1} | close {:.1} above-bottom {:.1} flat {:.1}",
            r[0], m64(r[1]), m64(r[2]), m64(r[3]), m64(r[4]), m64(r[5]), m64(r[6]),
            r[7], r[8], m64(r[9]), m64(r[10]), m64(r[11]), m64(r[12]), m64(r[13]), m64(r[14])
        );
    }
}
