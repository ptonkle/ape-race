//! Every tuning number of the race, and the six kinds of ape (PLAN P3–P8).
//! Lengths are metres, speeds metres per second, accelerations metres per
//! second squared, all Q16.16; durations are ticks.

use serde::{Deserialize, Serialize};

use crate::fx::{cm, int, ratio, ONE};

/// Physics steps per second (PLAN P3).
pub const TICKS_PER_SECOND: i32 = 120;
/// One tick, in seconds.
pub const DT: i32 = ratio(1, TICKS_PER_SECOND);

pub const fn secs_to_ticks(hundredths: i32) -> i32 {
    hundredths * TICKS_PER_SECOND / 100
}

/// Downward acceleration. Stronger than Earth's so a swing is quick.
pub const GRAVITY: i32 = int(20);

/// From the hands (the body's point) down to the feet.
pub const BODY_DROP: i32 = cm(130);
/// The body's radius, for hits and the walls.
pub const BODY_RADIUS: i32 = cm(60);

/// The shortest rope a grab makes.
pub const MIN_ROPE: i32 = int(3);
/// Share of the speed a grab keeps, redirected along the swing (Grab Keep).
pub const GRAB_KEEP: i32 = ONE * 96 / 100;
/// Extra speed along the velocity when Space is let go on the upswing.
pub const RELEASE_BOOST: i32 = cm(150);
/// Steering on a vine: how fast WASD across the motion turn the swing.
pub const SWING_STEER: i32 = int(16);
/// Below this speed pumping is boosted, up to 1.5 times at rest.
pub const PUMP_BOOST_SPEED: i32 = int(14);
/// Below this speed a swing can be pumped either way, to start from rest.
pub const PUMP_FROM_REST: i32 = int(2);

/// How long a press of Space keeps looking for a vine (Grab Window).
pub const GRAB_WINDOW: i32 = secs_to_ticks(15);
/// After a Missed Grab, how long before Space can grab again (Grab Cooldown).
pub const GRAB_COOLDOWN: i32 = secs_to_ticks(30);

/// A jump from the ground, straight up.
pub const JUMP_SPEED: i32 = int(11);
/// Above Run Speed the ground takes this much speed a second.
pub const GROUND_FRICTION: i32 = int(28);
/// How fast WASD bring the ape to its running velocity on the ground.
pub const RUN_ACCEL: i32 = int(40);

/// The corridor's walls push back this hard (share of the speed into them
/// that bounces back).
pub const WALL_BOUNCE: i32 = ONE * 3 / 10;
/// Nothing flies higher than this above the ground.
pub const CEILING: i32 = int(48);

/// The five stats of a kind of ape, each 1 to 5. Every kind's stats add up
/// to [`STAT_SUM`], so no kind is simply better (PLAN P8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    pub pump: i32,
    pub top_speed: i32,
    pub reach: i32,
    pub air_control: i32,
    pub weight: i32,
}

pub const STAT_SUM: i32 = 15;
pub const STAT_MAX: i32 = 5;
/// The average ape: what Equal Stats gives every kind.
pub const EQUAL: Stats = Stats {
    pump: 3,
    top_speed: 3,
    reach: 3,
    air_control: 3,
    weight: 3,
};

impl Stats {
    pub fn sum(&self) -> i32 {
        self.pump + self.top_speed + self.reach + self.air_control + self.weight
    }

    /// The stats as physics.
    pub fn physics(&self) -> Physics {
        let s = |v: i32| v.clamp(1, STAT_MAX) - 1;
        Physics {
            // 8 to 13 m/s² of pumping
            pump: int(8) + s(self.pump) * cm(125),
            // drag, in hundred-thousandths per metre: 0.0050 to 0.0034
            drag: 500 - s(self.top_speed) * 40,
            // 1.8 to 2.6 m from the hands (TRIAGE row 1: grabs were too tight)
            reach: cm(180) + s(self.reach) * cm(20),
            // 8 to 16 m/s² of steering in the air (TRIAGE row 2: the middle
            // ape could not reliably reach the next vine at 6.5)
            air: int(8) + s(self.air_control) * int(2),
            // a hit takes 130% to 70% of its usual effect
            hit_scale: ONE * 13 / 10 - s(self.weight) * (ONE * 15 / 100),
            // and the heavy run slower: 12 to 10 m/s (TRIAGE row 6)
            run_speed: int(12) - s(self.weight) * cm(50),
        }
    }
}

/// What a kind's stats mean to the physics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Physics {
    /// Acceleration from pumping a swing.
    pub pump: i32,
    /// Quadratic drag, in hundred-thousandths per metre: speed lost per
    /// second is `drag / 100 000 × speed²`.
    pub drag: i32,
    /// How far from the hands a vine can be grabbed.
    pub reach: i32,
    /// Steering in the air.
    pub air: i32,
    /// How strongly items push this ape around.
    pub hit_scale: i32,
    pub run_speed: i32,
}

/// The six kinds of ape (brief section 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Orangutan,
    Gorilla,
    #[default]
    Chimpanzee,
    Gibbon,
    Proboscis,
    Lemur,
}

impl Kind {
    pub const ALL: [Kind; 6] = [
        Kind::Orangutan,
        Kind::Gorilla,
        Kind::Chimpanzee,
        Kind::Gibbon,
        Kind::Proboscis,
        Kind::Lemur,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Kind::Orangutan => "orangutan",
            Kind::Gorilla => "gorilla",
            Kind::Chimpanzee => "chimpanzee",
            Kind::Gibbon => "gibbon",
            Kind::Proboscis => "proboscis",
            Kind::Lemur => "lemur",
        }
    }

    pub fn from_key(k: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|c| c.key() == k)
    }

    pub fn index(self) -> usize {
        Kind::ALL.iter().position(|k| *k == self).unwrap_or(0)
    }

    pub fn stats(self) -> Stats {
        let st = |pump, top_speed, reach, air_control, weight| Stats {
            pump,
            top_speed,
            reach,
            air_control,
            weight,
        };
        match self {
            // long arms: the best reach
            Kind::Orangutan => st(3, 2, 5, 2, 3),
            // powerful and heavy
            Kind::Gorilla => st(5, 2, 2, 1, 5),
            // the all-rounder
            Kind::Chimpanzee => st(3, 3, 3, 3, 3),
            // the fastest swinger, and light
            Kind::Gibbon => st(2, 5, 4, 3, 1),
            // steers well in the air
            Kind::Proboscis => st(3, 3, 2, 4, 3),
            // small and nimble
            Kind::Lemur => st(3, 4, 2, 5, 1),
        }
    }

    /// The stats in a race: the kind's own, or the average with Equal Stats.
    pub fn race_stats(self, equal: bool) -> Stats {
        if equal {
            EQUAL
        } else {
            self.stats()
        }
    }
}

// ---- items (PLAN P9) ----

/// A Banana flies this fast, on top of the thrower's speed.
pub const BANANA_SPEED: i32 = int(30);
pub const BANANA_RADIUS: i32 = cm(70);
pub const BANANA_LIFE: i32 = secs_to_ticks(250);
/// A Banana hit: speed kept, and how long the ape is Stunned.
pub const BANANA_KEEP: i32 = ONE * 60 / 100;
pub const BANANA_STUN: i32 = secs_to_ticks(70);
/// A Banana leaves this much above the camera's aim, so it arcs.
pub const BANANA_LOB: i32 = crate::fx::deg(6);

/// A Stick swing reaches this far, takes this share of speed and knocks the
/// ape off its vine.
pub const STICK_RANGE: i32 = cm(350);
pub const STICK_KEEP: i32 = ONE * 70 / 100;
pub const STICK_STUN: i32 = secs_to_ticks(45);
/// A Stick hit shoves the ape away from the swinger this fast.
pub const STICK_PUSH: i32 = int(5);

/// The Hook reaches the nearest ape ahead within this distance.
pub const HOOK_RANGE: i32 = int(60);
/// The hooked ape keeps this share of its speed and is pulled back this fast.
pub const HOOK_KEEP: i32 = ONE * 35 / 100;
pub const HOOK_PULL: i32 = int(12);
pub const HOOK_STUN: i32 = secs_to_ticks(50);
/// The user gains this much speed toward the hooked ape.
pub const HOOK_GAIN: i32 = int(8);
/// The Hook flies this fast, for at most this long.
pub const HOOK_SPEED: i32 = int(80);
pub const HOOK_LIFE: i32 = secs_to_ticks(100);

/// The Speed Boost: this much speed along the current velocity.
pub const BOOST_GAIN: i32 = int(14);

/// An item crate gives an item and comes back this long after.
pub const CRATE_RESPAWN: i32 = secs_to_ticks(400);
pub const CRATE_RADIUS: i32 = cm(150);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_the_same_stat_total() {
        for k in Kind::ALL {
            assert_eq!(k.stats().sum(), STAT_SUM, "{k:?}");
            let s = k.stats();
            for v in [s.pump, s.top_speed, s.reach, s.air_control, s.weight] {
                assert!((1..=STAT_MAX).contains(&v));
            }
        }
        assert_eq!(EQUAL.sum(), STAT_SUM);
    }

    #[test]
    fn equal_stats_make_every_kind_the_same() {
        assert_eq!(
            Kind::Gorilla.race_stats(true).physics(),
            Kind::Lemur.race_stats(true).physics()
        );
        assert_ne!(
            Kind::Gorilla.race_stats(false).physics(),
            Kind::Lemur.race_stats(false).physics()
        );
    }

    #[test]
    fn kinds_round_trip_by_key() {
        for k in Kind::ALL {
            assert_eq!(Kind::from_key(k.key()), Some(k));
        }
    }

    #[test]
    fn better_stats_mean_better_physics() {
        let lo = Stats {
            pump: 1,
            top_speed: 1,
            reach: 1,
            air_control: 1,
            weight: 1,
        }
        .physics();
        let hi = Stats {
            pump: 5,
            top_speed: 5,
            reach: 5,
            air_control: 5,
            weight: 5,
        }
        .physics();
        assert!(hi.pump > lo.pump);
        assert!(hi.drag < lo.drag && hi.drag > 0);
        assert!(hi.reach > lo.reach);
        assert!(hi.air > lo.air);
        assert!(hi.hit_scale < lo.hit_scale);
    }
}
