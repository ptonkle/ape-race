//! Every map has its picture for the setup's map grid (TRIAGE T54): the
//! file the core names (`Map::picture`) is shipped in `data/`, a webp, and
//! small. `scripts/map_cards.py` makes them from the built game.

use std::fs;
use std::path::Path;

use tool_core::map::Map;

/// The most a card's picture may weigh, in bytes: it loads with the menus.
const MAX_BYTES: usize = 80 * 1024;

#[test]
fn every_map_has_its_picture() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for m in Map::ALL {
        let name = m.picture();
        assert!(name.starts_with("data/"), "{name}: shipped from data/");
        let path = root.join(&name);
        let bytes = fs::read(&path).unwrap_or_else(|e| {
            panic!(
                "{}: no picture for the {} map ({e}); make it with scripts/map_cards.py",
                path.display(),
                m.key()
            )
        });
        assert!(
            bytes.len() > 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
            "{name} is not a webp"
        );
        assert!(
            bytes.len() <= MAX_BYTES,
            "{name} is {} bytes, over {MAX_BYTES}",
            bytes.len()
        );
    }
}
