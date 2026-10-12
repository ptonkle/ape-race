//! Sounds (PLAN P14), made by the core as WAV files. Placeholder until the
//! sound work lands: every cue is silent.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cue {
    Grab,
    Release,
    Miss,
    Jump,
    Land,
    Pickup,
    Throw,
    Hit,
    Boost,
    Beep,
    Go,
    Finish,
    Click,
}

impl Cue {
    pub const ALL: [Cue; 13] = [
        Cue::Grab,
        Cue::Release,
        Cue::Miss,
        Cue::Jump,
        Cue::Land,
        Cue::Pickup,
        Cue::Throw,
        Cue::Hit,
        Cue::Boost,
        Cue::Beep,
        Cue::Go,
        Cue::Finish,
        Cue::Click,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Cue::Grab => "grab",
            Cue::Release => "release",
            Cue::Miss => "miss",
            Cue::Jump => "jump",
            Cue::Land => "land",
            Cue::Pickup => "pickup",
            Cue::Throw => "throw",
            Cue::Hit => "hit",
            Cue::Boost => "boost",
            Cue::Beep => "beep",
            Cue::Go => "go",
            Cue::Finish => "finish",
            Cue::Click => "click",
        }
    }

    pub fn from_key(k: &str) -> Option<Cue> {
        Cue::ALL.into_iter().find(|c| c.key() == k)
    }
}

/// A cue as a complete WAV file at `volume` percent.
pub fn wav(cue: Cue, volume: u8) -> Vec<u8> {
    let _ = (cue, volume);
    Vec::new()
}

/// The wind loop at `volume` percent: one second that loops cleanly. The
/// page plays it with a gain the core sets from the speed.
pub fn wind(volume: u8) -> Vec<u8> {
    let _ = volume;
    Vec::new()
}
