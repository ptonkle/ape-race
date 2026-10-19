//! The race's panels: paused, the results, a Time Trial's medals.

use super::*;

impl App {
    pub(super) fn race_panels(&self) -> Vec<Value> {
        let Some(p) = &self.play else {
            return Vec::new();
        };
        if p.online {
            return self.online_race_panels(p);
        }
        if let Some(pb) = &p.replay {
            return vec![self.replay_panel(p, pb)];
        }
        let mut out = Vec::new();
        let tour = self.tour.is_some();
        if p.paused && tour {
            // a tournament race is not restarted; quitting leaves the
            // tournament (TRIAGE T103)
            out.push(
                json!({ "kind": "overlay", "heading": self.t("menu.paused"), "items": [
                    self.button("menu.resume"),
                    self.quiet_button("menu.replay"),
                    self.button("tour.quit"),
                ]}),
            );
        } else if p.paused {
            out.push(
                json!({ "kind": "overlay", "heading": self.t("menu.paused"), "items": [
                    self.button("menu.resume"),
                    self.button("menu.restart"),
                    self.quiet_button("menu.replay"),
                    self.button("menu.quit"),
                ]}),
            );
        } else if results_due(p) {
            let map = Map::of(&p.race.course);
            if p.trial {
                let finish = p.race.racers[p.local].finish.unwrap_or(0);
                let medal = medals::of(map)
                    .earned(finish)
                    .map(|m| self.t(&format!("medal.{}", m.key())))
                    .unwrap_or_else(|| self.t("item.none"));
                out.push(
                    json!({ "kind": "results", "heading": self.t("trial.heading"), "items": [
                        { "type": "list", "rows": [
                            [self.t("setup.map"), self.map_name(map)],
                            [self.t("results.time"), clock_text(finish)],
                            [self.t("trial.medal"), medal],
                        ]},
                        self.best_row(p),
                        self.medal_table(map, Some(finish)),
                        self.button("menu.restart"),
                        self.quiet_button("menu.replay"),
                        self.button("menu.main"),
                    ]}),
                );
            } else {
                // while others race on, a smaller panel at the side, with
                // the racer followed (TRIAGE T34)
                let racing = Focus::leader_racing(&p.race).is_some();
                let mut items = vec![self.results_table(p)];
                if racing {
                    items.push(self.focus_picker(p));
                } else {
                    items.push(self.best_row(p));
                }
                if tour {
                    // the standings next (TRIAGE T103): a racer still out
                    // there then scores nothing
                    items.extend([
                        self.button("tour.standings"),
                        self.quiet_button("menu.replay"),
                        self.button("tour.quit"),
                    ]);
                } else {
                    items.extend([
                        self.button("menu.rematch"),
                        self.quiet_button("menu.replay"),
                        self.button("menu.main"),
                    ]);
                }
                let kind = if racing { "spectate" } else { "results" };
                out.push(
                    json!({ "kind": kind, "heading": self.t("results.heading"), "items": items }),
                );
            }
        }
        out
    }

    /// The personal best under the results, marked when it was just set.
    pub(super) fn best_row(&self, p: &Play) -> Value {
        let map = Map::of(&p.race.course);
        let best = self.settings.best(map, p.trial);
        let best = best.map(clock_text).unwrap_or_else(|| self.t("item.none"));
        let label = if p.new_best {
            self.t("results.new_best")
        } else {
            self.t("results.best")
        };
        let mut rows = vec![
            json!([label, best]),
            json!([self.t("setup.map"), self.map_name(map)]),
        ];
        if let Some(c) = p.new_costume {
            rows.push(json!([
                self.t("results.costume_unlocked"),
                self.t(&format!("costume.{}", c.key()))
            ]));
        }
        json!({ "type": "list", "rows": rows })
    }

    pub(super) fn results_table(&self, p: &Play) -> Value {
        let mut order: Vec<usize> = (0..p.race.racers.len()).collect();
        order.sort_by_key(|i| p.race.racers[*i].place);
        let rows: Vec<Value> = order
            .iter()
            .map(|i| {
                let r = &p.race.racers[*i];
                let shown = match (r.finish, r.eaten) {
                    (Some(t), _) => clock_text(t),
                    (None, Some(_)) => self.t("results.eaten"),
                    (None, None) => self.t("results.racing"),
                };
                json!([
                    ordinal(r.place),
                    r.name.clone(),
                    self.t(&format!("ape.{}", r.ape.kind.key())),
                    shown,
                    *i == p.local
                ])
            })
            .collect();
        json!({ "type": "table", "head": [
            self.t("results.place"), self.t("results.name"), self.t("results.ape"), self.t("results.time")
        ], "rows": rows })
    }

    /// The target times: medal, target (the Author's hidden until Gold is
    /// earned), and a mark on each that `finish` beat.
    pub(super) fn medal_table(&self, map: Map, finish: Option<u32>) -> Value {
        let medals = medals::of(map);
        let shown = medals.author_shown(self.settings.best(map, true));
        let rows: Vec<Value> = trial::Medal::ALL
            .iter()
            .map(|m| {
                let target = medals.target(*m);
                let text = if *m == trial::Medal::Author && !shown {
                    self.t("trial.hidden")
                } else {
                    clock_text(target)
                };
                let beat = finish.map(|t| t <= target).unwrap_or(false);
                json!([
                    self.t(&format!("medal.{}", m.key())),
                    text,
                    if beat {
                        self.t("trial.beaten")
                    } else {
                        String::new()
                    },
                    false
                ])
            })
            .collect();
        json!({ "type": "table", "head": [self.t("trial.medal"), self.t("trial.target"), ""], "rows": rows })
    }
}

/// Whether the results show yet: a second after the local ape finished,
/// or once the jaguar has eaten it (TRIAGE T37).
pub(crate) fn results_due(p: &Play) -> bool {
    let me = &p.race.racers[p.local];
    let wait = if me.finish.is_none() && me.eaten.is_some() {
        // with the camera's pan on
        EATEN_SPECTATE_AFTER
    } else {
        rules::TICKS_PER_SECOND as u32 + 1
    };
    p.since_out.is_some_and(|t| t >= wait)
}
