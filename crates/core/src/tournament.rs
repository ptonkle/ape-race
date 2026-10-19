//! Tournament mode (TRIAGE T103): a series of races for points, chosen on
//! Race Setup (racing the AI) or in the host's lobby. Each race's place
//! gives points ([`POINTS`]); after the last race the most points wins.
//!
//! The standings are a pure function of the recorded races ([`Leg`]), so
//! every browser of an online room computes the same order from the same
//! legs (the host records each leg from its own results and sends the legs
//! in the roster).
//!
//! The order: most points, then the lowest combined time, then seat order
//! (the order the racers first appeared: online the lobby's seats, solo
//! the player then the AI apes).
//!
//! A racer who does not finish a race (eaten by the jaguar, still racing
//! when the race was ended, or gone) scores 0 and counts the leg's cutoff
//! as its time: the race clock when the race was ended, never less than the
//! slowest finish.

use serde::{Deserialize, Serialize};

use crate::costume::Costume;
use crate::map::Map;
use crate::rules::Kind;

/// Points by place, 1st first. Fewer racers use the same table from the
/// top; a place past its end scores 0.
pub const POINTS: [u32; 4] = [10, 7, 5, 3];
/// Races in a tournament: the default and the range of the −/+ control.
pub const DEFAULT_RACES: u8 = 4;
pub const MIN_RACES: u8 = 2;
pub const MAX_RACES: u8 = 16;

/// A race count kept in range.
pub fn clamp_races(n: u8) -> u8 {
    n.clamp(MIN_RACES, MAX_RACES)
}

/// One racer of the tournament, as it first appeared.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entrant {
    /// who it is across races: online a human's client id or `ai1`..;
    /// solo `you` or `ai1`..
    pub key: String,
    pub name: String,
    pub kind: Kind,
    #[serde(default)]
    pub palette: u8,
    #[serde(default)]
    pub costume: Costume,
}

/// One race of the tournament, once it is over.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Leg {
    pub map: Map,
    /// the race clock when the race was ended (ticks since Go)
    pub cutoff: u32,
    /// each entrant's finish (race ticks), by its place in
    /// [`Tour::entrants`]; `None`: did not finish. Shorter than the
    /// entrants when some joined later.
    pub fin: Vec<Option<u32>>,
}

impl Leg {
    /// The cutoff a non-finisher counts: never less than the slowest finish.
    pub fn cutoff(&self) -> u32 {
        self.fin
            .iter()
            .flatten()
            .copied()
            .fold(self.cutoff, u32::max)
    }

    /// Each entrant's points in this leg (`n` entrants): finishers by time
    /// (a tie by seat), [`POINTS`] down from the top; non-finishers 0.
    pub fn points(&self, n: usize) -> Vec<u32> {
        let mut done: Vec<(u32, usize)> = self
            .fin
            .iter()
            .enumerate()
            .filter_map(|(i, f)| f.map(|t| (t, i)))
            .collect();
        done.sort();
        let mut out = vec![0; n.max(self.fin.len())];
        for (rank, (_, i)) in done.into_iter().enumerate() {
            out[i] = POINTS.get(rank).copied().unwrap_or(0);
        }
        out.truncate(n);
        out
    }

    /// Each entrant's time for the tiebreak: its finish, or the cutoff.
    pub fn ticks(&self, i: usize) -> u32 {
        self.fin
            .get(i)
            .copied()
            .flatten()
            .unwrap_or_else(|| self.cutoff())
    }
}

/// A tournament in progress.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tour {
    /// how many races
    pub races: u8,
    /// Random map every race (T64); otherwise the player or host picks
    /// the map after each race
    pub random: bool,
    pub entrants: Vec<Entrant>,
    pub legs: Vec<Leg>,
}

/// One line of the standings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Standing {
    /// 1-based, after the tiebreak (never shared)
    pub place: usize,
    /// its place in [`Tour::entrants`]
    pub entrant: usize,
    pub points: u32,
    /// points in the last race recorded
    pub last: u32,
    /// combined race time (ticks), the tiebreak
    pub ticks: u64,
}

impl Tour {
    pub fn new(races: u8, random: bool) -> Tour {
        Tour {
            races: clamp_races(races),
            random,
            entrants: Vec::new(),
            legs: Vec::new(),
        }
    }

    /// Every race has been run.
    pub fn done(&self) -> bool {
        self.legs.len() >= self.races as usize
    }

    /// The number of the race to run next (1-based).
    pub fn next_race(&self) -> usize {
        self.legs.len() + 1
    }

    /// Records a race: each racer with its finish (`None`: did not finish).
    /// A racer not yet an entrant (by key) joins at the end; an entrant not
    /// in this race did not finish it. Nothing once the tour is done.
    pub fn record(&mut self, map: Map, cutoff: u32, racers: &[(Entrant, Option<u32>)]) {
        if self.done() {
            return;
        }
        for (e, _) in racers {
            if !self.entrants.iter().any(|x| x.key == e.key) {
                self.entrants.push(e.clone());
            }
        }
        let mut fin = vec![None; self.entrants.len()];
        for (e, f) in racers {
            if let Some(i) = self.entrants.iter().position(|x| x.key == e.key) {
                fin[i] = *f;
            }
        }
        self.legs.push(Leg { map, cutoff, fin });
    }

    /// The standings after the legs so far: most points, then the lowest
    /// combined time, then seat order.
    pub fn standings(&self) -> Vec<Standing> {
        let n = self.entrants.len();
        let mut rows: Vec<Standing> = (0..n)
            .map(|i| Standing {
                place: 0,
                entrant: i,
                points: 0,
                last: 0,
                ticks: 0,
            })
            .collect();
        for leg in &self.legs {
            let pts = leg.points(n);
            for (i, row) in rows.iter_mut().enumerate() {
                row.points += pts[i];
                row.last = pts[i];
                row.ticks += leg.ticks(i) as u64;
            }
        }
        rows.sort_by(|a, b| {
            b.points
                .cmp(&a.points)
                .then(a.ticks.cmp(&b.ticks))
                .then(a.entrant.cmp(&b.entrant))
        });
        for (k, r) in rows.iter_mut().enumerate() {
            r.place = k + 1;
        }
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(key: &str) -> Entrant {
        Entrant {
            key: key.to_string(),
            name: key.to_string(),
            kind: Kind::Chimpanzee,
            palette: 0,
            costume: Costume::Own,
        }
    }

    fn order(t: &Tour) -> Vec<String> {
        t.standings()
            .iter()
            .map(|s| t.entrants[s.entrant].key.clone())
            .collect()
    }

    #[test]
    fn places_score_ten_seven_five_three_and_a_non_finisher_nothing() {
        let leg = Leg {
            map: Map::Jungle,
            cutoff: 0,
            fin: vec![Some(300), None, Some(100), Some(200)],
        };
        assert_eq!(leg.points(4), [5, 0, 10, 7]);
        // two racers: the same table from the top
        let two = Leg {
            map: Map::Jungle,
            cutoff: 0,
            fin: vec![Some(50), Some(40)],
        };
        assert_eq!(two.points(2), [7, 10]);
    }

    #[test]
    fn a_points_tie_goes_to_the_lower_combined_time_then_the_seat() {
        let mut t = Tour::new(4, false);
        let (a, b, c) = (e("a"), e("b"), e("c"));
        // a wins race 1 and b race 2: 17 each; b (the later seat) has the
        // lower combined time
        t.record(
            Map::Jungle,
            0,
            &[
                (a.clone(), Some(100)),
                (b.clone(), Some(150)),
                (c.clone(), None),
            ],
        );
        t.record(
            Map::City,
            500,
            &[
                (a.clone(), Some(260)),
                (b.clone(), Some(200)),
                (c.clone(), None),
            ],
        );
        assert_eq!(order(&t), ["b", "a", "c"]);
        let s = t.standings();
        assert_eq!((s[0].points, s[1].points, s[2].points), (17, 17, 0));
        assert_eq!(s[0].ticks, 350);
        // c did not finish either: the cutoffs (the slowest finish, 150,
        // then the race's own 500)
        assert_eq!(s[2].ticks, 650);
        // all equal: seat order
        let mut u = Tour::new(2, false);
        u.record(Map::Jungle, 90, &[(b.clone(), None), (a.clone(), None)]);
        assert_eq!(order(&u), ["b", "a"]);
    }

    #[test]
    fn a_late_entrant_joins_at_the_end_and_a_missing_one_did_not_finish() {
        let mut t = Tour::new(2, true);
        t.record(Map::Jungle, 400, &[(e("a"), Some(300))]);
        t.record(Map::City, 400, &[(e("b"), Some(100)), (e("a"), Some(200))]);
        assert_eq!(t.entrants.len(), 2);
        assert_eq!(t.legs[0].fin, [Some(300)]);
        assert_eq!(order(&t), ["a", "b"]);
        assert!(t.done());
        // a done tour records nothing more
        t.record(Map::City, 1, &[(e("c"), Some(1))]);
        assert_eq!(t.legs.len(), 2);
    }

    #[test]
    fn the_race_count_stays_between_two_and_sixteen() {
        assert_eq!(Tour::new(1, false).races, MIN_RACES);
        assert_eq!(Tour::new(99, false).races, MAX_RACES);
        assert_eq!(Tour::new(DEFAULT_RACES, false).races, 4);
    }
}
