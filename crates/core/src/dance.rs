//! The finish dance (TRIAGE T53; the person, 2026-10-12: "a unique dance
//! for each of the six apes; no sad slumps for the losers").
//!
//! Every ape that finishes dances, whatever its place, once it has coasted
//! to a stop on the ground past the line: it turns round to face back up
//! the course (where the chase camera and the apes still racing are) and
//! dances its kind's dance (`models`, `Pose::dance`). Nothing here keeps
//! state or changes the race: whether and how an ape dances comes from its
//! finish tick, its mode, speed and place on the course, so every browser,
//! ghost and replay draws the same dance with no message of its own.

use crate::ape::Mode;
use crate::fx::{self, int, HALF, ONE};
use crate::geom::yaw_of;
use crate::race::Race;

/// The ape dances fully once it has slowed below this speed, and starts
/// to turn round and dance as it slows below [`DANCE_FROM`].
pub const DANCE_FULL: i32 = int(5);
pub const DANCE_FROM: i32 = int(12);

/// How an ape dances this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dance {
    /// ticks since the finish: the dance's beat counts from it
    pub beat: u32,
    /// how far into the dance (0..=ONE): it blends in as the ape slows
    pub weight: i32,
}

/// Ape `k`'s dance this frame, and the way it faces while dancing; None
/// while it has not finished, is in the air or on a vine, or was eaten.
pub fn of(race: &Race, k: usize) -> Option<(Dance, i32)> {
    let beat = race.since_finish(k)?;
    let r = &race.racers[k];
    if r.eaten.is_some() || r.ape.mode != Mode::Grounded {
        return None;
    }
    let speed = r.ape.vel.len();
    let weight = fx::div(DANCE_FROM - speed, DANCE_FROM - DANCE_FULL).clamp(0, ONE);
    if weight == 0 {
        return None;
    }
    // back up the course, toward the arch: turning round as it slows
    let c = &race.course;
    let ahead = c.heading[r.ape.hint.min(c.heading.len() - 1)];
    let back = fx::wrap(yaw_of(ahead) + HALF);
    let yaw = fx::wrap(r.ape.yaw + fx::mul(fx::wrap(back - r.ape.yaw), weight));
    Some((Dance { beat, weight }, yaw))
}

/// Whether ape `k` is fully dancing (for the camera, which stays to watch).
pub fn dancing(race: &Race, k: usize) -> bool {
    of(race, k).is_some_and(|(d, _)| d.weight == ONE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ape::Controls;
    use crate::course::Course;
    use crate::fx::deg;
    use crate::geom::{facing, v3, V3};
    use crate::race::{Driver, Entry, COUNTDOWN};
    use crate::rules::Kind;

    /// A race of one ape, 100 ticks after Go, standing still.
    fn race() -> Race {
        let e = Entry {
            name: "P".into(),
            kind: Kind::Gorilla,
            driver: Driver::Local,
            palette: 0,
        };
        let mut r = Race::new(Course::jungle(), &[e], false, 1);
        for _ in 0..COUNTDOWN + 100 {
            r.step(&[Controls::default()]);
        }
        r.racers[0].ape.mode = Mode::Grounded;
        r.racers[0].ape.vel = V3::ZERO;
        r
    }

    #[test]
    fn a_finished_ape_stopped_on_the_ground_dances_facing_back_up_the_course() {
        let mut r = race();
        assert_eq!(of(&r, 0), None, "not before the finish");
        r.racers[0].finish = Some(r.clock - 30);
        assert_eq!(r.since_finish(0), Some(30));
        let (d, yaw) = of(&r, 0).expect("it dances");
        assert_eq!(
            d,
            Dance {
                beat: 30,
                weight: ONE
            }
        );
        assert!(dancing(&r, 0));
        // it faces back the way the course came
        let ahead = r.course.heading[r.racers[0].ape.hint];
        let toward = facing(yaw, 0).dot(ahead);
        assert!(toward < -ONE * 99 / 100, "{toward}");
        // every browser counts the beat from the same finish tick
        r.step(&[Controls::default()]);
        r.racers[0].ape.vel = V3::ZERO;
        r.racers[0].ape.mode = Mode::Grounded;
        assert_eq!(of(&r, 0).unwrap().0.beat, 31);
    }

    #[test]
    fn it_blends_in_as_it_slows_and_never_dances_in_the_air_or_eaten() {
        let mut r = race();
        r.racers[0].finish = Some(r.clock);
        let ape_yaw = r.racers[0].ape.yaw;
        r.racers[0].ape.vel = v3(int(20), 0, 0);
        assert_eq!(of(&r, 0), None, "coasting fast: not yet");
        r.racers[0].ape.vel = v3((DANCE_FROM + DANCE_FULL) / 2, 0, 0);
        let (d, yaw) = of(&r, 0).unwrap();
        assert!(d.weight > ONE / 3 && d.weight < ONE * 2 / 3, "{}", d.weight);
        let turned = fx::wrap(yaw - ape_yaw).abs();
        assert!(
            turned > deg(20) && turned < deg(160),
            "half turned: {turned}"
        );
        assert!(!dancing(&r, 0));
        r.racers[0].ape.vel = V3::ZERO;
        r.racers[0].ape.mode = Mode::Flying;
        assert_eq!(of(&r, 0), None, "in the air");
        r.racers[0].ape.mode = Mode::Grounded;
        let at = r.racers[0].ape.pos;
        r.racers[0].eaten = Some(crate::jaguar::Eaten {
            tick: r.tick,
            at,
            from: at,
            pop: false,
        });
        assert_eq!(of(&r, 0), None, "eaten");
    }
}
