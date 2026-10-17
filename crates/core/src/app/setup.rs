//! Race Setup and Time Trial setup: the panel and the map grid (TRIAGE T54).

use super::*;

impl App {
    /// A setup's fields (`setup.*`).
    pub(super) fn setup_act(&mut self, id: &str, value: &str) {
        match id {
            "setup.ghost" => {
                self.settings.ghost = value == "on";
                self.save = true;
            }
            "setup.map" => {
                if value == RANDOM_MAP {
                    self.settings.random_map = true;
                    self.save = true;
                } else if let Some(m) = Map::from_key(value) {
                    self.settings.map = m;
                    self.settings.random_map = false;
                    self.save = true;
                }
            }
            "setup.ape" => {
                if let Some(k) = Kind::from_key(value) {
                    self.settings.kind = k;
                    self.save = true;
                }
            }
            "setup.ape.next" | "setup.ape.prev" => {
                let n = Kind::ALL.len();
                let i = self.settings.kind.index();
                let j = if id.ends_with("next") {
                    (i + 1) % n
                } else {
                    (i + n - 1) % n
                };
                self.settings.kind = Kind::ALL[j];
                self.save = true;
            }
            "setup.palette" => {
                if let Ok(p) = value.parse::<u8>() {
                    self.settings.set_palette(p);
                    self.save = true;
                }
            }
            "setup.costume" => {
                if let Some(c) = Costume::from_key(value) {
                    self.save |= self.settings.set_costume(c);
                }
            }
            "setup.ai_level" => {
                if let Some(d) = crate::ai::Difficulty::from_key(value) {
                    self.settings.ai_level = d;
                    self.save = true;
                }
            }
            "setup.opponents" => {
                if let Ok(n) = value.parse::<u8>() {
                    self.settings.ai_count = n.clamp(1, 3);
                    self.save = true;
                }
            }
            "setup.equal" => {
                self.settings.equal_stats = value == "on";
                self.save = true;
            }
            "setup.lava" => {
                self.settings.lava = value == "on";
                self.save = true;
            }
            _ if id.starts_with("setup.item.") => self.tick_item(id, value),
            "setup.weather" => self.choose_weather(value),
            _ => {}
        }
    }
    pub(super) fn setup_panels(&self) -> Vec<Value> {
        let kinds: Vec<Value> = Kind::ALL
            .iter()
            .map(|k| json!({ "value": k.key(), "text": self.t(&format!("ape.{}", k.key())) }))
            .collect();
        // one to three: Time Trial is the solo run (TRIAGE T26)
        let opponents: Vec<Value> = (1..=3)
            .map(|n| json!({ "value": n.to_string(), "text": n.to_string() }))
            .collect();
        let stats = self.settings.kind.race_stats(self.settings.equal_stats);
        let mut items = vec![
            self.map_tiles(
                "setup.map",
                self.settings.map,
                self.settings.random_map,
                false,
            ),
            json!({ "type": "picker", "id": "setup.ape", "label": self.t("setup.ape"),
              "value": self.settings.kind.key(), "options": kinds,
              "text": self.t(&format!("ape.{}", self.settings.kind.key())) }),
            self.palette_select("setup.palette"),
            self.costume_select("setup.costume"),
            json!({ "type": "stats", "label": self.t("setup.stats"), "rows": self.stats_rows(&stats) }),
        ];
        // the Ski Slope's weather, Banana Bay Battle's (and Random's, which
        // may pick either), after the colours and the costume (TRIAGE T70,
        // T89: those two short selects share a row)
        if self.settings.random_map {
            items.insert(4, self.weather_select("setup.weather", None));
        } else if crate::snow::Weather::offered(self.settings.map).len() > 1 {
            items.insert(
                4,
                self.weather_select("setup.weather", Some(self.settings.map)),
            );
        }
        if self.trial {
            items.push(self.medal_table(self.settings.map, None));
            items.push(
                json!({ "type": "toggle", "id": "setup.ghost", "label": self.t("setup.ghost"),
              "value": self.settings.ghost }),
            );
        } else {
            items.push(json!({ "type": "select", "id": "setup.opponents", "label": self.t("setup.opponents"),
              "value": self.settings.ai_count.max(1).to_string(), "options": opponents }));
            let levels: Vec<Value> = crate::ai::Difficulty::ALL
                .iter()
                .map(|d| json!({ "value": d.key(), "text": self.t(&format!("ai.{}", d.key())) }))
                .collect();
            items.push(
                json!({ "type": "select", "id": "setup.ai_level", "label": self.t("setup.ai_level"),
              "value": self.settings.ai_level.key(), "options": levels }),
            );
        }
        items.push(
            json!({ "type": "toggle", "id": "setup.equal", "label": self.t("setup.equal"),
          "value": self.settings.equal_stats }),
        );
        if !self.trial {
            items.push(self.lava_toggle("setup.lava"));
            items.extend(self.item_toggles());
        }
        items.push(self.button("menu.start"));
        if self.best_replay().is_some() {
            items.push(self.quiet_button("replay.best"));
        }
        items.push(self.button("menu.back"));
        let heading = if self.trial {
            "trial.heading"
        } else {
            "setup.heading"
        };
        vec![json!({ "kind": "setup", "heading": self.t(heading), "items": items })]
    }

    /// The map choice on Race Setup, Time Trial and the lobby (TRIAGE T54):
    /// a grid of cards, one per map with its picture and name, and last
    /// Random (T64, `random`: chosen instead of `chosen`). `read_only` (a
    /// joiner's lobby) shows the host's pick and lets no card be pressed.
    /// A card sends the grid's id with its map's key, or `random`.
    pub(crate) fn map_tiles(&self, id: &str, chosen: Map, random: bool, read_only: bool) -> Value {
        let mut options: Vec<Value> = Map::ALL
            .iter()
            .map(|m| {
                json!({ "id": format!("{id}.{}", m.key()), "value": m.key(),
                  "text": self.map_name(*m), "picture": m.picture(),
                  "chosen": !random && *m == chosen })
            })
            .collect();
        options.push(
            json!({ "id": format!("{id}.{RANDOM_MAP}"), "value": RANDOM_MAP,
          "text": self.t("map.random"), "picture": RANDOM_PICTURE, "chosen": random }),
        );
        let value = if random { RANDOM_MAP } else { chosen.key() };
        json!({ "type": "tiles", "id": id, "label": self.t("setup.map"),
                "value": value, "read_only": read_only, "options": options })
    }

    /// The Floor is Lava checkbox (TRIAGE T37), on the setup (`setup.lava`)
    /// or the host's lobby (`lobby.lava`).
    pub(crate) fn lava_toggle(&self, id: &str) -> Value {
        json!({ "type": "toggle", "id": id, "label": self.t("setup.lava"),
          "value": self.settings.lava })
    }
}
