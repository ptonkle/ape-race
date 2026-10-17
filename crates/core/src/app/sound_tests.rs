//! A race's events and their sounds (moved from app.rs's tests).

use super::*;

#[test]
fn a_reset_and_the_door_have_their_own_sounds() {
    // (Sector Ape, the course agent's SC1: a warp back onto a deck for
    // the ape put back, and the door's slam for everyone)
    let reset = Event::Reset { who: 0, pad: 3 };
    assert_eq!(cue_for(&reset, 0), Some(Cue::Reset));
    assert_eq!(cue_for(&reset, 1), None);
    let door = Event::Door { who: 2 };
    assert_eq!(cue_for(&door, 0), Some(Cue::Door));
    assert_eq!(cue_for(&door, 2), Some(Cue::Door));
    assert_ne!(sound::wav(Cue::Reset, 50), sound::wav(Cue::Door, 50));
}
