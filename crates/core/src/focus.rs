//! Which racer the camera follows (TRIAGE T24, T34).
//!
//! One switch serves every place the view leaves the local ape: the replay
//! viewer (next / previous racer), spectating the rest of a race after the
//! local ape finishes, and any later way out of a race (T37's jaguar). Who
//! counts as still in the race is [`racing`], in one place, so a new way
//! out only changes that function.

use crate::race::{Race, Racer};

/// Whether a racer is still in the race: not finished, and not eaten by
/// the jaguar (TRIAGE T37).
pub fn racing(r: &Racer) -> bool {
    !r.out()
}

/// The racer the camera follows, by seat (an index into `race.racers`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Focus {
    pub racer: usize,
}

impl Focus {
    pub fn on(racer: usize) -> Focus {
        Focus { racer }
    }

    /// The next racer in seat order, wrapping. With `only_racing` it skips
    /// those out of the race, unless every other one is. Returns whether
    /// the focus moved.
    pub fn next(&mut self, race: &Race, only_racing: bool) -> bool {
        self.step(race, only_racing, 1)
    }

    /// The previous racer in seat order, wrapping; as [`Focus::next`].
    pub fn prev(&mut self, race: &Race, only_racing: bool) -> bool {
        self.step(race, only_racing, race.racers.len().saturating_sub(1))
    }

    fn step(&mut self, race: &Race, only_racing: bool, by: usize) -> bool {
        let n = race.racers.len();
        if n == 0 {
            return false;
        }
        let from = self.racer.min(n - 1);
        let seats = (1..n).map(|k| (from + k * by) % n);
        let pick = seats
            .clone()
            .find(|i| !only_racing || racing(&race.racers[*i]))
            .or_else(|| seats.clone().next());
        match pick {
            Some(i) if i != self.racer => {
                self.racer = i;
                true
            }
            _ => false,
        }
    }

    /// The leading racer still in the race (the best place), if any.
    pub fn leader_racing(race: &Race) -> Option<usize> {
        (0..race.racers.len())
            .filter(|i| racing(&race.racers[*i]))
            .min_by_key(|i| race.racers[*i].place)
    }

    /// Follow the next racer still racing: when the followed one is out of
    /// the race, move to the leading racer still in it. Stays when the
    /// followed one is still racing or no one is. Returns whether it moved.
    pub fn follow_racing(&mut self, race: &Race) -> bool {
        if race.racers.get(self.racer).is_some_and(racing) {
            return false;
        }
        match Focus::leader_racing(race) {
            Some(i) if i != self.racer => {
                self.racer = i;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course::Course;
    use crate::race::{Driver, Entry};
    use crate::rules::Kind;

    fn race(n: usize) -> Race {
        let entries: Vec<Entry> = (0..n)
            .map(|i| Entry {
                name: format!("P{i}"),
                kind: Kind::ALL[i],
                driver: if i == 0 { Driver::Local } else { Driver::Ai },
            })
            .collect();
        Race::new(Course::test_line(&[]), &entries, false, 1)
    }

    #[test]
    fn next_and_previous_cycle_the_seats() {
        let r = race(3);
        let mut f = Focus::on(0);
        assert!(f.next(&r, false));
        assert_eq!(f.racer, 1);
        f.next(&r, false);
        f.next(&r, false);
        assert_eq!(f.racer, 0, "wraps");
        assert!(f.prev(&r, false));
        assert_eq!(f.racer, 2);
        // alone, nothing to switch to
        let mut solo = Focus::on(0);
        assert!(!solo.next(&race(1), false));
    }

    #[test]
    fn only_racing_skips_the_finished() {
        let mut r = race(4);
        r.racers[1].finish = Some(100);
        r.racers[2].finish = Some(120);
        let mut f = Focus::on(0);
        f.next(&r, true);
        assert_eq!(f.racer, 3);
        f.prev(&r, true);
        assert_eq!(f.racer, 0);
        // everyone out: cycles them all
        for i in 0..4 {
            r.racers[i].finish = Some(200);
        }
        f.next(&r, true);
        assert_eq!(f.racer, 1);
    }

    #[test]
    fn follow_racing_moves_to_the_leader_still_racing() {
        let mut r = race(4);
        let mut f = Focus::on(0);
        assert!(!f.follow_racing(&r), "still racing: stays");
        r.racers[0].finish = Some(100);
        r.racers[0].place = 1;
        r.racers[1].place = 4;
        r.racers[2].place = 2;
        r.racers[3].place = 3;
        assert!(f.follow_racing(&r));
        assert_eq!(f.racer, 2, "the best place still racing");
        // and on when that one finishes too
        r.racers[2].finish = Some(110);
        assert!(f.follow_racing(&r));
        assert_eq!(f.racer, 3);
        for i in 0..4 {
            r.racers[i].finish = Some(200);
        }
        assert!(!f.follow_racing(&r), "no one left: stays");
        assert_eq!(Focus::leader_racing(&r), None);
    }
}
