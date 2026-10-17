//! The title, Settings and the shared menu pieces (buttons, the stats rows).

use super::*;

impl App {
    /// The Settings screen's fields (`settings.*`).
    pub(super) fn settings_act(&mut self, id: &str, value: &str) {
        match id {
            "settings.volume" => {
                if let Ok(v) = value.parse::<u8>() {
                    self.settings.volume = v.min(100);
                    self.save = true;
                }
            }
            "settings.sensitivity" => {
                if let Ok(v) = value.parse::<i32>() {
                    self.settings.sensitivity = v.clamp(1, 10);
                    self.save = true;
                }
            }
            "settings.invert" => {
                self.settings.invert_y = value == "on";
                self.save = true;
            }
            "settings.speed_colours" => {
                self.settings.speed_colours = value == "on";
                self.save = true;
            }
            "settings.colourblind" => {
                self.settings.colourblind = value == "on";
                self.save = true;
            }
            "settings.auto_camera" => {
                self.settings.auto_camera = value == "on";
                self.save = true;
            }
            _ => {}
        }
    }

    /// A click on the title text: ten unlock Speed Colours (TRIAGE row 26).
    pub(super) fn title_click(&mut self) {
        self.title_clicks += 1;
        if self.title_clicks == TITLE_CLICKS && !self.settings.colours_unlocked {
            self.settings.colours_unlocked = true;
            self.settings.speed_colours = true;
            self.celebrate = true;
            self.save = true;
            self.sounds.push(Cue::Finish.key().to_string());
        }
    }
    pub(crate) fn button(&self, id: &str) -> Value {
        json!({ "type": "button", "id": id, "text": self.t(id) })
    }

    pub(crate) fn quiet_button(&self, id: &str) -> Value {
        json!({ "type": "button", "id": id, "text": self.t(id), "quiet": true })
    }

    pub(super) fn title_panels(&self) -> Vec<Value> {
        vec![
            json!({ "kind": "title", "heading": self.t("title"), "sub": self.t("tagline"),
                    "click": "title.click", "items": [] }),
            json!({ "kind": "menu", "items": [
                self.button("menu.race"),
                self.button("menu.trial"),
                self.button("menu.online"),
                self.button("menu.replays"),
                self.button("menu.settings"),
                self.notes_button(),
            ]}),
            self.controls_panel(),
        ]
    }

    pub(super) fn controls_panel(&self) -> Value {
        let rows: Vec<Value> = ["swing", "grab", "jump", "item", "look", "pause", "retry"]
            .iter()
            .map(|k| {
                json!([
                    self.t(&format!("controls.{k}")),
                    self.t(&format!("controls.{k}_keys"))
                ])
            })
            .collect();
        json!({ "kind": "controls", "heading": self.t("controls.heading"), "items": [
            { "type": "list", "rows": rows }
        ]})
    }

    pub(super) fn settings_panels(&self) -> Vec<Value> {
        let mut panels = vec![
            json!({ "kind": "settings", "heading": self.t("settings.heading"), "items": [
                { "type": "slider", "id": "settings.volume", "label": self.t("settings.volume"),
                  "min": 0, "max": 100, "step": 5, "value": self.settings.volume,
                  "text": format!("{}%", self.settings.volume) },
                { "type": "slider", "id": "settings.sensitivity", "label": self.t("settings.sensitivity"),
                  "min": 1, "max": 10, "step": 1, "value": self.settings.sensitivity,
                  "text": self.settings.sensitivity.to_string() },
                { "type": "toggle", "id": "settings.invert", "label": self.t("settings.invert"),
                  "value": self.settings.invert_y },
                { "type": "toggle", "id": "settings.auto_camera", "label": self.t("settings.auto_camera"),
                  "value": self.settings.auto_camera },
                { "type": "toggle", "id": "settings.colourblind", "label": self.t("settings.colourblind"),
                  "value": self.settings.colourblind },
            ]}),
        ];
        // Speed Colours shows once unlocked (TRIAGE row 26)
        if let Some(items) = panels[0]["items"].as_array_mut() {
            if self.settings.colours_unlocked {
                items.push(json!({ "type": "toggle", "id": "settings.speed_colours",
                  "label": self.t("settings.speed_colours"), "value": self.settings.speed_colours }));
            }
            items.push(self.button("menu.back"));
        }
        panels
    }

    pub(crate) fn stats_rows(&self, stats: &Stats) -> Value {
        let row = |key: &str, v: i32| json!({ "label": self.t(&format!("stat.{key}")), "value": v, "max": rules::STAT_MAX });
        json!([
            row("pump", stats.pump),
            row("top_speed", stats.top_speed),
            row("reach", stats.reach),
            row("air_control", stats.air_control),
            row("weight", stats.weight),
        ])
    }
}
