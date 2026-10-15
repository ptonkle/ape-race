//! The patch notes (`data/patch-notes.json`, newest first): what each
//! version changed, in the players' words. The same file feeds the in-game
//! "What's new" panel and the site's `patch-notes.html` (`scripts/pages.sh`).
//!
//! A version whose last number is not 0 (1.10.2) is a minor patch; x.y.0
//! is a major one. The JSON carries no flag for it, so it cannot disagree
//! with the number.

use serde::Deserialize;

/// The notes file, built into the core.
pub const JSON: &str = include_str!("../../../data/patch-notes.json");

/// One version's notes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Entry {
    pub version: String,
    pub date: String,
    pub notes: Vec<String>,
}

impl Entry {
    /// A minor patch: the last number is not 0.
    pub fn minor(&self) -> bool {
        parse(&self.version).is_some_and(|v| v.2 != 0)
    }
}

/// Every version's notes, newest first (empty if the file does not read).
pub fn all() -> Vec<Entry> {
    serde_json::from_str(JSON).unwrap_or_default()
}

/// "1.10.2" as (1, 10, 2).
pub fn parse(version: &str) -> Option<(u32, u32, u32)> {
    let mut it = version.trim_start_matches('v').split('.');
    let a = it.next()?.parse().ok()?;
    let b = it.next()?.parse().ok()?;
    let c = it.next()?.parse().ok()?;
    if it.next().is_some() {
        return None;
    }
    Some((a, b, c))
}

/// `version` is newer than `seen` (a player who has seen none, or whose
/// saved one does not read, has not seen it).
pub fn newer(version: &str, seen: &str) -> bool {
    match (parse(version), parse(seen)) {
        (Some(v), Some(s)) => v > s,
        (Some(_), None) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deploy cannot go out without its notes: the newest entry is the
    /// game's version (HANDOFF, Deploy).
    #[test]
    fn newest_entry_is_this_version() {
        let all = all();
        assert!(!all.is_empty(), "data/patch-notes.json does not read");
        assert_eq!(
            all[0].version,
            crate::app::VERSION,
            "add this version's entry to the top of data/patch-notes.json"
        );
    }

    #[test]
    fn entries_are_newest_first_and_short() {
        let all = all();
        for e in &all {
            assert!(parse(&e.version).is_some(), "{} is not x.y.z", e.version);
            assert!(
                (1..=6).contains(&e.notes.len()),
                "{} has {} notes",
                e.version,
                e.notes.len()
            );
            assert_eq!(e.date.len(), 10, "{}: date as YYYY-MM-DD", e.version);
        }
        for w in all.windows(2) {
            assert!(
                newer(&w[0].version, &w[1].version),
                "{} is listed above {}",
                w[0].version,
                w[1].version
            );
        }
    }

    #[test]
    fn minor_and_newer() {
        let e = |v: &str| Entry {
            version: v.into(),
            date: String::new(),
            notes: Vec::new(),
        };
        assert!(e("1.10.2").minor());
        assert!(!e("1.10.0").minor());
        assert!(newer("1.10.0", "1.9.2"));
        assert!(newer("1.10.2", "1.10.1"));
        assert!(!newer("1.10.2", "1.10.2"));
        assert!(newer("1.10.2", ""));
    }
}
