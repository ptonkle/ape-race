//! What's New: the patch notes screen and its badge.

use super::*;

impl App {
    /// The patch notes list minor versions too (`notes.minor`).
    pub(super) fn notes_minor(&mut self, value: &str) {
        self.settings.notes_minor = value == "on";
        self.save = true;
    }
    /// What's New on the main menu, marked "New" while this version's notes
    /// are unopened (a minor version too).
    pub(super) fn notes_button(&self) -> Value {
        let mut b = self.button("menu.notes");
        if crate::patch_notes::newer(VERSION, &self.settings.notes_seen) {
            b["badge"] = json!(self.t("notes.new"));
        }
        b
    }

    pub(super) fn version_opens_notes(&self) -> bool {
        self.gallery.is_none()
            && matches!(
                self.screen,
                Screen::Title | Screen::Settings | Screen::Replays | Screen::Notes
            )
    }

    pub(super) fn open_notes(&mut self) {
        if self.settings.notes_seen != VERSION {
            self.settings.notes_seen = VERSION.to_string();
            self.save = true;
        }
        self.go(Screen::Notes);
    }

    /// The patch notes, newest first: the major versions only, unless the
    /// player ticks minor ones in.
    pub(super) fn notes_panels(&self) -> Vec<Value> {
        let entries: Vec<Value> = crate::patch_notes::all()
            .into_iter()
            .filter(|e| self.settings.notes_minor || !e.minor())
            .map(|e| {
                json!({ "version": format!("v{}", e.version), "date": e.date,
                        "minor": e.minor(), "lines": e.notes })
            })
            .collect();
        vec![
            json!({ "kind": "notes", "heading": self.t("notes.heading"), "items": [
                { "type": "toggle", "id": "notes.minor", "label": self.t("notes.minor"),
                  "value": self.settings.notes_minor },
                { "type": "notes", "id": "notes.list", "entries": entries },
                self.button("menu.back"),
            ]}),
        ]
    }
}
