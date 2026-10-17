//! Sounds: the ape calls (TRIAGE T58), the wind, each cue's wave and the cue a race event makes.

use super::*;

impl App {
    /// The call button (TRIAGE T58): the player's ape gives its kind's
    /// call, in a race (and the others hear it, by distance), in Race Setup
    /// and Time Trial setup, and in the online lobby (the ape chosen as the
    /// character; the other players hear it too). Presses inside the
    /// cooldown do nothing; not while watching a replay, nor paused.
    pub(super) fn call_pressed(&mut self) {
        if self.call_cooldown > 0 {
            return;
        }
        match self.screen {
            Screen::Setup | Screen::Lobby => {
                let kind = self.settings.kind;
                self.calls.push((sound::call_key(kind), 100));
                if let Some(o) = &mut self.online.session {
                    let _ = o.call_in_lobby(kind);
                }
            }
            Screen::Race => {
                let Some(mut p) = self.play.take() else {
                    return;
                };
                if p.replay.is_none() && (!p.paused || p.online) {
                    let who = p.local;
                    self.hear_call(&p, who);
                    if let Some(rec) = &mut p.record {
                        rec.calls.push((p.race.tick, who));
                    }
                    if p.online {
                        if let Some(o) = &mut self.online.session {
                            o.send_call(who);
                        }
                    }
                } else {
                    self.play = Some(p);
                    return;
                }
                self.play = Some(p);
            }
            _ => return,
        }
        self.call_cooldown = sound::CALL_COOLDOWN;
    }

    /// Racer `who` calls: the page plays its kind's call at full loudness
    /// when it is the followed ape, else at a loudness from its distance
    /// to the followed ape (`sound::call_gain`), nothing when too far.
    pub(crate) fn hear_call(&mut self, p: &Play, who: usize) {
        let Some(r) = p.race.racers.get(who) else {
            return;
        };
        let f = &p.race.racers[p.focus.racer];
        let gain = if who == p.focus.racer {
            100
        } else {
            fx::mul(sound::call_gain(r.ape.pos.dist(f.ape.pos)), int(100)) / ONE
        };
        if gain > 0 {
            self.calls.push((sound::call_key(r.ape.kind), gain));
        }
    }

    /// The wind's loudness, 0..=100, from the followed ape's smoothed
    /// horizontal speed, as the speedometer reads it; quiet while a replay
    /// is paused, over or seeking (TRIAGE T30: it read the raw speed, a
    /// swing's fall included, and held on when a replay stopped).
    pub(super) fn wind_gain(&self) -> i32 {
        match &self.play {
            Some(p) if !p.paused && self.screen == Screen::Race => {
                if p.replay.as_ref().is_some_and(|pb| pb.hushed(p.race.tick)) {
                    return 0;
                }
                fx::mul(sound::wind_gain(p.hype_speed), int(100)) / ONE
            }
            _ => 0,
        }
    }

    pub fn sound(&self, key: &str) -> Vec<u8> {
        if key == "wind" {
            return sound::wind(self.settings.volume);
        }
        if key == sound::SCREECH {
            return sound::screech(self.settings.volume);
        }
        if let Some(level) = sound::voice_level(key) {
            return sound::voice(level, self.settings.volume);
        }
        if let Some(kind) = sound::call_kind(key) {
            return sound::call(kind, self.settings.volume);
        }
        match Cue::from_key(key) {
            Some(c) => sound::wav(c, self.settings.volume),
            None => Vec::new(),
        }
    }
}

/// The sound a race event makes, for the player at `local`.
pub(super) fn cue_for(e: &Event, local: usize) -> Option<Cue> {
    match *e {
        Event::Body { who, what } if who == local => Some(match what {
            Happening::Grab => Cue::Grab,
            Happening::Release => Cue::Release,
            Happening::Miss => Cue::Miss,
            Happening::Jump => Cue::Jump,
            Happening::Land => Cue::Land,
            Happening::Knocked => Cue::Hit,
            // its sound comes with the reward
            Happening::Bounce => return None,
            // (Sector Ape; the art may give it its own sound)
            Happening::Rebound => Cue::Bounce,
            // (Sunset Ruins; the art may give it its own sound)
            Happening::Spiked => Cue::Hit,
            // its sound comes with the reward
            Happening::Launched => return None,
            Happening::Wall => return None,
        }),
        Event::Countdown { .. } => Some(Cue::Beep),
        Event::Go => Some(Cue::Go),
        Event::Pickup { who, .. } if who == local => Some(Cue::Pickup),
        Event::Used { who, item } if who == local => Some(match item {
            crate::race::Item::Boost => Cue::Boost,
            _ => Cue::Throw,
        }),
        Event::Hit { who, by, .. } if who == local || by == local => Some(Cue::Hit),
        Event::Finish { who, .. } if who == local => Some(Cue::Finish),
        Event::Pounce { who } if who == local => Some(Cue::Roar),
        Event::Reset { who, .. } if who == local => Some(Cue::Reset),
        // (the door shuts for everyone: every player hears it)
        Event::Door { .. } => Some(Cue::Door),
        Event::Reward { who, what } if who == local => Some(match what {
            crate::race::Reward::Mushroom => Cue::Bounce,
            crate::race::Reward::Catchup => Cue::Catchup,
            crate::race::Reward::Banana => Cue::Banana,
            crate::race::Reward::Shortcut => Cue::Boost,
        }),
        _ => None,
    }
}
