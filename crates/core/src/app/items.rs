//! Items (TRIAGE T42) on the setup, and the hype call-outs (T22).

use super::*;

impl App {
    /// The race setup's item checkboxes (TRIAGE T42): one per item, ticked
    /// when crates may give it, and a note when none is. Online, the host's
    /// go out with the start.
    pub(crate) fn item_toggles(&self) -> Vec<Value> {
        let options: Vec<Value> = crate::items::ItemSet::OFFERED
            .iter()
            .map(|item| {
                json!({ "id": format!("setup.item.{}", item.key()),
                  "text": self.t(&format!("item.{}", item.key())),
                  "value": self.settings.items.has(*item) })
            })
            .collect();
        let note = if self.settings.items.is_empty() {
            self.t("setup.no_items")
        } else {
            String::new()
        };
        vec![
            json!({ "type": "checks", "label": self.t("setup.items"), "options": options, "note": note }),
        ]
    }

    /// An item checkbox changed (`setup.item.<key>`).
    pub(crate) fn tick_item(&mut self, id: &str, value: &str) {
        let key = id.trim_start_matches("setup.item.");
        if let Some(item) = crate::items::ItemSet::OFFERED
            .into_iter()
            .find(|i| i.key() == key)
        {
            self.settings.items = self.settings.items.with(item, value == "on");
            self.save = true;
        }
    }

    /// The hype call-outs (TRIAGE T22): crossing a hype level upward calls
    /// it out in an ape voice (`hype::Hype` keeps it from repeating); not
    /// once the local ape has finished.
    pub(super) fn hype_tick(&mut self, p: &mut Play) {
        if p.race.racers[p.local].out() {
            return;
        }
        // on Sector Ape's faster dial, the same share of the way round
        let dial = crate::speedo::Dial::on(Map::of(&p.race.course));
        let kmh = crate::speedo::kmh(dial.standard(p.hype_speed));
        if let Some(level) = p.hype.step(kmh) {
            self.sounds.push(crate::hype::voice_key(level));
        }
    }
}
