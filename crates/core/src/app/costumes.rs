//! Costumes and palettes (TRIAGE T49, T89): what is unlocked and worn, and their selects.

use super::*;

impl App {
    /// The chosen ape's colours (TRIAGE T49), as a select with id `id`.
    pub(crate) fn palette_select(&self, id: &str) -> Value {
        let kind = self.settings.kind;
        let options: Vec<Value> = (0..models::PALETTES)
            .map(|p| json!({ "value": p.to_string(), "text": self.palette_name(kind, p) }))
            .collect();
        json!({ "type": "select", "id": id, "label": self.t("setup.palette"),
          "value": self.settings.palette().to_string(), "options": options })
    }

    /// The costume to wear (TRIAGE T89), as a select with id `id`: the
    /// default (the map's own), none, and each costume unlocked so far;
    /// locked ones are not listed (the person, 2026-10-15).
    pub(crate) fn costume_select(&self, id: &str) -> Value {
        let options: Vec<Value> = Costume::ALL
            .into_iter()
            .filter(|c| self.settings.wearable(*c))
            .map(|c| json!({ "value": c.key(), "text": self.t(&format!("costume.{}", c.key())) }))
            .collect();
        json!({ "type": "select", "id": id, "label": self.t("setup.costume"),
          "value": self.settings.costume.key(), "options": options })
    }

    /// The lobby's costume select (GR-T11): the costumes this player may
    /// wear, and Grocery Run lent to everyone while the lobby is on Aisle
    /// Ape (not on Random, whose map is picked at the start); the value is
    /// the seat's own, which may be the lent one.
    pub(crate) fn lobby_costume_select(&self, id: &str, view: &crate::online::LobbyView) -> Value {
        let worn = view
            .seats
            .iter()
            .find(|s| s.me)
            .map_or(self.settings.costume, |s| s.costume);
        let options: Vec<Value> = Costume::ALL
            .into_iter()
            .filter(|c| self.settings.wearable(*c) || Self::lent_in(*c, view))
            .map(|c| json!({ "value": c.key(), "text": self.t(&format!("costume.{}", c.key())) }))
            .collect();
        json!({ "type": "select", "id": id, "label": self.t("setup.costume"),
          "value": worn.key(), "options": options })
    }

    /// Whether `c` is lent in the lobby `view` (GR-T11).
    pub(crate) fn lent_in(c: Costume, view: &crate::online::LobbyView) -> bool {
        !view.random_map && c.lent_on(view.map)
    }

    /// Gives a lent costume back when the lobby leaves its map (GR-T11):
    /// the seat wears this player's own choice again.
    pub(crate) fn return_lent_costume(&mut self) {
        let now = self.now_ms;
        let own = self.settings.costume;
        let Some(o) = &mut self.online.session else {
            return;
        };
        let view = o.lobby();
        let Some(me) = view.seats.iter().find(|s| s.me) else {
            return;
        };
        if !self.settings.wearable(me.costume) && !Self::lent_in(me.costume, &view) {
            let _ = o.set_costume(own, now);
        }
    }

    /// The name of `kind`'s palette `p`.
    pub(crate) fn palette_name(&self, kind: Kind, p: u8) -> String {
        if p == 0 {
            self.t("palette.classic")
        } else {
            self.t(&format!("palette.{}.{p}", kind.key()))
        }
    }
}

impl Settings {
    /// Whether this player may wear `c` (TRIAGE T89): the map's own and
    /// none always, the others once unlocked.
    pub fn wearable(&self, c: Costume) -> bool {
        c.free() || self.costumes.contains(c.key())
    }

    /// Unlocks `c`; true when it was locked before.
    pub fn unlock(&mut self, c: Costume) -> bool {
        self.costumes.insert(c.key().to_string())
    }

    /// Chooses the costume to wear; a locked one is refused (false).
    pub fn set_costume(&mut self, c: Costume) -> bool {
        if self.wearable(c) {
            self.costume = c;
        }
        self.costume == c
    }

    /// The colours `kind` wears (TRIAGE T49).
    pub fn palette_of(&self, kind: Kind) -> u8 {
        self.palettes.get(kind.key()).copied().unwrap_or(0)
    }

    /// The colours of the chosen ape.
    pub fn palette(&self) -> u8 {
        self.palette_of(self.kind)
    }

    /// Chooses the colours of the chosen ape (out of range: the classic).
    pub fn set_palette(&mut self, palette: u8) {
        let key = self.kind.key().to_string();
        if palette == 0 || palette >= models::PALETTES {
            self.palettes.remove(&key);
        } else {
            self.palettes.insert(key, palette);
        }
    }
}
