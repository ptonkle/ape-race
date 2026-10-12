//! The one door into the core. Every function takes strings and returns a JSON
//! string: `{"ok": ...}` on success or `{"error": "..."}` with a message the
//! page can show as it is. The shim forwards these calls and nothing else.

use serde::Serialize;

use crate::page;

fn ok<T: Serialize>(value: &T) -> String {
    serde_json::json!({ "ok": value }).to_string()
}

fn error(message: String) -> String {
    serde_json::json!({ "error": message }).to_string()
}

/// Checks the strings file and returns the strings the page should draw.
pub fn load(strings_json: &str) -> String {
    match page::parse_strings(strings_json) {
        Ok(strings) => ok(&strings),
        Err(e) => error(e.message()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_wraps_success_in_ok() {
        assert_eq!(
            load(r#"{"title": "T", "intro": "I"}"#),
            r#"{"ok":{"intro":"I","title":"T"}}"#
        );
    }

    #[test]
    fn load_wraps_failure_in_error() {
        assert_eq!(
            load(r#"{"title": "", "intro": "I"}"#),
            r#"{"error":"The strings file has an empty \"title\"."}"#
        );
    }
}
