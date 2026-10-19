//! Tournament mode's screens (TRIAGE T103): the setup's choice, the solo
//! series (the standings between races, the final standings), and the
//! standings table and podium order both solo and online draw. The rules
//! (points, tiebreak) are `crate::tournament`.

use super::*;
use crate::tournament::{self as tm, Entrant, Tour};

/// The solo player's key in a tournament (the AI apes are `ai1`..).
const YOU: &str = "you";

impl App {
    /// The setup's tournament fields: the toggle and the race count's −/+
    /// (`id`: `setup.tournament`, `setup.races.dec`, `setup.races.inc`, or
    /// the lobby's `lobby.*` twins).
    pub(super) fn tour_setting(&mut self, id: &str, value: &str) {
        let s = &mut self.settings;
        if id.ends_with(".tournament") {
            s.tournament = value == "on";
        } else if id.ends_with(".races.dec") {
            s.tour_races = tm::clamp_races(s.tour_races.saturating_sub(1));
        } else if id.ends_with(".races.inc") {
            s.tour_races = tm::clamp_races(s.tour_races.saturating_add(1));
        } else {
            return;
        }
        self.save = true;
    }

    /// The host's tournament race count from the settings (0: none).
    pub(super) fn tour_wanted(&self) -> u8 {
        if self.settings.tournament {
            self.settings.tour_races
        } else {
            0
        }
    }

    /// The setup's tournament toggle and, when on, the race count (`races`
    /// shown, `can_change`: a joiner's are read-only).
    pub(super) fn tour_items(
        &self,
        prefix: &str,
        on: bool,
        races: u8,
        can_change: bool,
    ) -> Vec<Value> {
        if !can_change {
            let text = if on {
                format!("{races} {}", self.t("tour.races_unit"))
            } else {
                self.t("off")
            };
            return vec![json!({ "type": "list", "rows": [[self.t("tour.label"), text]] })];
        }
        let mut out = vec![
            json!({ "type": "toggle", "id": format!("{prefix}.tournament"),
            "label": self.t("tour.label"), "value": on }),
        ];
        if on {
            out.push(json!({ "type": "stepper", "id": format!("{prefix}.races"),
                "label": self.t("tour.races"), "text": races.to_string(),
                "dec_disabled": races <= tm::MIN_RACES, "inc_disabled": races >= tm::MAX_RACES }));
        }
        out
    }

    /// A new solo tournament from the setup, with its lineup fixed: the
    /// player, then `opponents` AI apes of other kinds, from the dice.
    pub(super) fn new_tour(&self, entries: &[Entry]) -> Tour {
        let mut t = Tour::new(self.settings.tour_races, self.settings.random_map);
        t.entrants = entries
            .iter()
            .enumerate()
            .map(|(i, e)| Entrant {
                key: if i == 0 {
                    YOU.to_string()
                } else {
                    format!("ai{i}")
                },
                name: e.name.clone(),
                kind: e.kind,
                palette: e.palette,
                costume: e.costume,
            })
            .collect();
        t
    }

    /// The race entries of the solo tournament's fixed lineup.
    pub(super) fn tour_entries(t: &Tour) -> Vec<Entry> {
        t.entrants
            .iter()
            .map(|e| Entry {
                name: e.name.clone(),
                kind: e.kind,
                driver: if e.key == YOU {
                    Driver::Local
                } else {
                    Driver::Ai
                },
                palette: e.palette,
                costume: e.costume,
            })
            .collect()
    }

    /// `tour.*`: the results' Standings, the standings' map, Next race,
    /// and leaving the tournament.
    pub(super) fn tour_act(&mut self, id: &str, value: &str) {
        match id {
            "tour.standings" if self.screen == Screen::Race => {
                if let (Some(t), Some(p)) = (&mut self.tour, &self.play) {
                    if p.replay.is_none() {
                        let racers: Vec<(Entrant, Option<u32>)> = t
                            .entrants
                            .iter()
                            .cloned()
                            .zip(p.race.racers.iter().map(|r| r.finish))
                            .collect();
                        t.record(Map::of(&p.race.course), p.race.clock, &racers);
                    }
                }
                if self.tour.is_some() {
                    self.go(Screen::Standings);
                }
            }
            "tour.map" if self.screen == Screen::Standings => {
                if let Some(m) = Map::from_key(value) {
                    self.settings.map = m;
                    self.save = true;
                }
            }
            "tour.next" if self.screen == Screen::Standings => {
                if self.tour.as_ref().is_some_and(|t| !t.done()) {
                    self.start_race();
                }
            }
            "tour.quit" | "tour.done" => self.go(Screen::Setup),
            _ => {}
        }
    }

    /// The solo standings screen: the table, the next map (unless Random),
    /// Next race; after the last race the final standings.
    pub(super) fn standings_panels(&self) -> Vec<Value> {
        let Some(t) = &self.tour else {
            return Vec::new();
        };
        let mut items = vec![self.standings_table(t, YOU)];
        if t.done() {
            items.push(self.button("tour.done"));
        } else {
            if !t.random {
                items.push(self.tour_map_tiles("tour.map", self.settings.map, false));
            }
            items.push(self.button("tour.next"));
            items.push(self.quiet_button("tour.quit"));
        }
        vec![self.standings_panel(t, items)]
    }

    /// The standings' panel: its heading and the race count under it.
    pub(crate) fn standings_panel(&self, t: &Tour, items: Vec<Value>) -> Value {
        let heading = if t.done() {
            "tour.final"
        } else {
            "tour.standings"
        };
        let sub = format!(
            "{} {} / {}",
            self.t("tour.race"),
            t.legs.len().min(t.races as usize),
            t.races
        );
        json!({ "kind": "setup", "heading": self.t(heading), "sub": sub, "items": items })
    }

    /// The map choice between races: the grid without Random (a Random
    /// tournament shows no grid).
    pub(crate) fn tour_map_tiles(&self, id: &str, chosen: Map, read_only: bool) -> Value {
        let mut tiles = self.map_tiles(id, chosen, false, read_only);
        if let Some(o) = tiles["options"].as_array_mut() {
            o.retain(|c| c["value"] != RANDOM_MAP);
        }
        tiles["label"] = json!(self.t("tour.next_map"));
        tiles
    }

    /// The standings as a table: place, name, ape, points in the last
    /// race, total, combined time; `me` marks this player's row.
    pub(crate) fn standings_table(&self, t: &Tour, me: &str) -> Value {
        let rows: Vec<Value> = t
            .standings()
            .iter()
            .map(|s| {
                let e = &t.entrants[s.entrant];
                json!([
                    ordinal(s.place),
                    e.name.clone(),
                    self.t(&format!("ape.{}", e.kind.key())),
                    format!("+{}", s.last),
                    s.points.to_string(),
                    clock_text(s.ticks.min(u32::MAX as u64) as u32),
                    e.key == me
                ])
            })
            .collect();
        json!({ "type": "table", "id": "tour.table", "head": [
            self.t("results.place"), self.t("results.name"), self.t("results.ape"),
            self.t("tour.this_race"), self.t("tour.points"), self.t("tour.time")
        ], "rows": rows })
    }

    /// The final order for the podium (T103 part 2), once a tournament is
    /// over and its final standings show: each ape by overall place.
    pub(super) fn podium(&self) -> Option<Value> {
        let (t, me) = match self.screen {
            Screen::Standings => (self.tour.as_ref()?, YOU.to_string()),
            Screen::Lobby => {
                let o = self.online.session.as_ref()?;
                (o.tour()?, o.client_id().to_string())
            }
            _ => return None,
        };
        if !t.done() {
            return None;
        }
        let order: Vec<Value> = t
            .standings()
            .iter()
            .map(|s| {
                let e = &t.entrants[s.entrant];
                json!({ "place": s.place, "name": e.name, "kind": e.kind.key(),
                    "palette": e.palette, "costume": e.costume.key(),
                    "points": s.points, "me": e.key == me })
            })
            .collect();
        Some(Value::Array(order))
    }
}
