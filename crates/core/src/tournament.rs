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
//! A racer who has not finished when the race is ended (on Standings)
//! places below every finisher: first those still racing, by how far along
//! the course they got, then those out (eaten by the jaguar, or gone), by
//! the same. It scores that place's points and counts the leg's cutoff as
//! its time: the race clock when the race was ended, never less than the
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

/// One racer's race, as it stood when the race was ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    /// its finish (race ticks), or `None`
    pub fin: Option<u32>,
    /// out without finishing: eaten by the jaguar, or its player left
    #[serde(default)]
    pub out: bool,
    /// how far along the course (metres, Q16): the HUD place's measure
    #[serde(default)]
    pub s: i32,
}

impl Run {
    /// Finished at `t`.
    pub fn done(t: u32) -> Run {
        Run {
            fin: Some(t),
            out: false,
            s: 0,
        }
    }

    /// Not finished, `s` along; `out`: eaten or gone.
    pub fn short(s: i32, out: bool) -> Run {
        Run { fin: None, out, s }
    }
}

/// One race of the tournament, once it is over.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Leg {
    pub map: Map,
    /// the race clock when the race was ended (ticks since Go)
    pub cutoff: u32,
    /// each entrant's run, by its place in [`Tour::entrants`]; `None`: not
    /// in this race. Shorter than the entrants when some joined later.
    pub runs: Vec<Option<Run>>,
}

impl Leg {
    /// The cutoff a non-finisher counts: never less than the slowest finish.
    pub fn cutoff(&self) -> u32 {
        self.runs
            .iter()
            .flatten()
            .filter_map(|r| r.fin)
            .fold(self.cutoff, u32::max)
    }

    /// The entrants who raced, in place order: finishers by time, then
    /// those still racing by progress, then those out by progress (a tie
    /// by seat).
    pub fn order(&self) -> Vec<usize> {
        let mut raced: Vec<((u8, i64), usize)> = self
            .runs
            .iter()
            .enumerate()
            .filter_map(|(i, r)| r.map(|r| (r, i)))
            .map(|(r, i)| {
                let key = match (r.fin, r.out) {
                    (Some(t), _) => (0, t as i64),
                    (None, false) => (1, -(r.s as i64)),
                    (None, true) => (2, -(r.s as i64)),
                };
                (key, i)
            })
            .collect();
        raced.sort();
        raced.into_iter().map(|(_, i)| i).collect()
    }

    /// Each entrant's points in this leg (`n` entrants): [`POINTS`] down
    /// the place order from the top; an entrant not in the race 0.
    pub fn points(&self, n: usize) -> Vec<u32> {
        let mut out = vec![0; n.max(self.runs.len())];
        for (rank, i) in self.order().into_iter().enumerate() {
            out[i] = POINTS.get(rank).copied().unwrap_or(0);
        }
        out.truncate(n);
        out
    }

    /// Each entrant's time for the tiebreak: its finish, or the cutoff.
    pub fn ticks(&self, i: usize) -> u32 {
        self.runs
            .get(i)
            .copied()
            .flatten()
            .and_then(|r| r.fin)
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

    /// Records a race: each racer with its run. A racer not yet an entrant
    /// (by key) joins at the end; an entrant not in this race scores
    /// nothing in it. Nothing once the tour is done.
    pub fn record(&mut self, map: Map, cutoff: u32, racers: &[(Entrant, Run)]) {
        if self.done() {
            return;
        }
        for (e, _) in racers {
            if !self.entrants.iter().any(|x| x.key == e.key) {
                self.entrants.push(e.clone());
            }
        }
        let mut runs = vec![None; self.entrants.len()];
        for (e, r) in racers {
            if let Some(i) = self.entrants.iter().position(|x| x.key == e.key) {
                runs[i] = Some(*r);
            }
        }
        self.legs.push(Leg { map, cutoff, runs });
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
    fn places_score_ten_seven_five_three_finishers_first_then_by_progress() {
        let leg = Leg {
            map: Map::Jungle,
            cutoff: 0,
            runs: vec![
                Some(Run::done(300)),
                Some(Run::short(50, false)),
                Some(Run::done(100)),
                Some(Run::done(200)),
            ],
        };
        assert_eq!(leg.points(4), [5, 3, 10, 7]);
        // still racing ranks above eaten or gone, each by progress
        let out = Leg {
            map: Map::Jungle,
            cutoff: 0,
            runs: vec![
                Some(Run::done(50)),
                Some(Run::short(900, true)),
                Some(Run::short(100, false)),
                Some(Run::short(200, true)),
            ],
        };
        assert_eq!(out.points(4), [10, 5, 7, 3]);
        // two racers: the same table from the top
        let two = Leg {
            map: Map::Jungle,
            cutoff: 0,
            runs: vec![Some(Run::done(50)), Some(Run::done(40))],
        };
        assert_eq!(two.points(2), [7, 10]);
    }

    #[test]
    fn a_points_tie_goes_to_the_lower_combined_time_then_the_seat() {
        let mut t = Tour::new(4, false);
        let (a, b, c) = (e("a"), e("b"), e("c"));
        let short = Run::short(0, false);
        // a wins race 1 and b race 2: 17 each; b (the later seat) has the
        // lower combined time
        t.record(
            Map::Jungle,
            0,
            &[
                (a.clone(), Run::done(100)),
                (b.clone(), Run::done(150)),
                (c.clone(), short),
            ],
        );
        t.record(
            Map::City,
            500,
            &[
                (a.clone(), Run::done(260)),
                (b.clone(), Run::done(200)),
                (c.clone(), short),
            ],
        );
        assert_eq!(order(&t), ["b", "a", "c"]);
        let s = t.standings();
        assert_eq!((s[0].points, s[1].points, s[2].points), (17, 17, 10));
        assert_eq!(s[0].ticks, 350);
        // c did not finish either: the cutoffs (the slowest finish, 150,
        // then the race's own 500)
        assert_eq!(s[2].ticks, 650);
        // all equal: seat order
        let mut u = Tour::new(2, false);
        u.record(Map::Jungle, 90, &[(b.clone(), short), (a.clone(), short)]);
        assert_eq!(order(&u), ["b", "a"]);
    }

    #[test]
    fn a_late_entrant_joins_at_the_end_and_a_missing_one_scores_nothing() {
        let mut t = Tour::new(2, true);
        t.record(Map::Jungle, 400, &[(e("a"), Run::done(300))]);
        t.record(
            Map::City,
            400,
            &[(e("b"), Run::done(100)), (e("a"), Run::done(200))],
        );
        assert_eq!(t.entrants.len(), 2);
        assert_eq!(t.legs[0].runs, [Some(Run::done(300))]);
        assert_eq!(order(&t), ["a", "b"]);
        assert!(t.done());
        // a done tour records nothing more
        t.record(Map::City, 1, &[(e("c"), Run::done(1))]);
        assert_eq!(t.legs.len(), 2);
    }

    #[test]
    fn the_race_count_stays_between_two_and_sixteen() {
        assert_eq!(Tour::new(1, false).races, MIN_RACES);
        assert_eq!(Tour::new(99, false).races, MAX_RACES);
        assert_eq!(Tour::new(DEFAULT_RACES, false).races, 4);
    }
}
