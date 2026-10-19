//! The weather (the Ski Slope's snow, Banana Bay Battle's squall): what the 3D view shows and the setup's Weather select.

use super::*;

impl App {
    /// The Weather select on a setup (`setup.weather`); the weather is
    /// chosen on a setup, not in Settings (2026-10-17).
    pub(super) fn choose_weather(&mut self, value: &str) {
        if let Some(w) = crate::snow::Weather::from_key(value) {
            self.settings.weather = w;
            self.save = true;
        }
    }

    /// Whether snow falls in the 3D view: the Weather setting is Snowing
    /// or a Blizzard and the view is on the Ski Slope (a race, the title's
    /// flyover, the setup's preview), never on the lineup.
    pub(crate) fn snowing(&self) -> bool {
        self.weather_shown().falls()
    }

    /// The weather the 3D view shows: `weather()` (the host's online) where
    /// the shown map offers it (the Ski Slope's snow, Banana Bay Battle's
    /// squall; not on the lineup), else clear.
    pub(crate) fn weather_shown(&self) -> crate::snow::Weather {
        if !matches!(self.stage, Stage::Lineup | Stage::Podium) {
            self.weather().on(self.shown_map())
        } else {
            crate::snow::Weather::Clear
        }
    }

    /// The weather shown: online (the lobby and an online race) the host's
    /// for everyone (TRIAGE T67), else this player's setting.
    pub(crate) fn weather(&self) -> crate::snow::Weather {
        let online = self.screen == Screen::Lobby || self.play.as_ref().is_some_and(|p| p.online);
        match &self.online.session {
            Some(o) if online => o.weather(),
            _ => self.settings.weather,
        }
    }

    /// A Weather select (`setup.weather` on a setup, `lobby.weather`),
    /// labelled by its id, offering `map`'s weathers (`None`, Random: every one, each shown only on
    /// its map, so each but Clear names that map: "Squall (Banana Bay
    /// Battle)"; SECOND-ORDER BW1).
    pub(crate) fn weather_select(&self, id: &str, map: Option<Map>) -> Value {
        let list: &[crate::snow::Weather] = match map {
            Some(m) => crate::snow::Weather::offered(m),
            None => &crate::snow::Weather::ALL,
        };
        let options: Vec<Value> = list
            .iter()
            .map(|w| {
                let name = self.t(&format!("weather.{}", w.key()));
                let text = match (map, w.home()) {
                    (None, Some(home)) => format!("{name} ({})", self.map_name(home)),
                    _ => name,
                };
                json!({ "value": w.key(), "text": text })
            })
            .collect();
        let value = match map {
            Some(m) => self.weather().on(m),
            None => self.weather(),
        };
        json!({ "type": "select", "id": id, "label": self.t(id),
                "value": value.key(), "options": options })
    }
}
