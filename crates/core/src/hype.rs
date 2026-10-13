//! Hype call-outs (TRIAGE T22): when the local ape's speed crosses a hype
//! level of the speedometer (`speedo::HYPE`), an ape voice calls it out and
//! the screen flashes. The core decides when; the page only plays the sound
//! the app names and draws the flash at the strength the HUD gives.
//!
//! Not spammy: a level is reached on rising past its threshold and left
//! only once the speed falls `MARGIN_KMH` below it, so hovering at a
//! threshold does not flicker; after a call-out, a call at the same level or
//! lower waits `HOLD`. A higher level is always called at once, so the top
//! level's shout is never swallowed by the one before it.

use crate::fx::ONE;
use crate::rules::TICKS_PER_SECOND;
use crate::speedo;

const TPS: u32 = TICKS_PER_SECOND as u32;

/// After a call-out, none at the same level or lower for this many ticks.
pub const HOLD: u32 = TPS * 4;
/// A level is left only this far (km/h) below its threshold.
pub const MARGIN_KMH: i32 = 4;
/// The flash fades out over this many ticks.
pub const FLASH: u32 = TPS * 7 / 10;
/// Whether the screen itself flashes at a call-out (off: the person found
/// it too much, 2026-10-12); the shout still pops.
pub const SCREEN_FLASH: bool = false;
/// The flash's strength (percent) by the level called: the top level is the
/// splashiest.
const FLASH_PCT: [i32; 4] = [0, 35, 60, 100];

/// The call-out state of one player's race.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hype {
    /// the level now, 0 to 3, with the margin on the way down
    level: u8,
    /// the last level called out
    called: u8,
    /// ticks until a call at or below `called` may come again
    hold: u32,
    /// ticks of flash left
    flash: u32,
    /// the level the flash is for
    flash_level: u8,
}

impl Hype {
    /// One race tick at this smoothed speed (km/h): the level to call out,
    /// if one was crossed upward just now.
    pub fn step(&mut self, kmh: i32) -> Option<u8> {
        self.hold = self.hold.saturating_sub(1);
        self.flash = self.flash.saturating_sub(1);
        let up = speedo::hype(kmh);
        let down = speedo::hype(kmh + MARGIN_KMH);
        let was = self.level;
        if up > self.level {
            self.level = up;
        } else if down < self.level {
            self.level = down;
        }
        if self.level > was && (self.level > self.called || self.hold == 0) {
            self.called = self.level;
            self.hold = HOLD;
            self.flash = FLASH;
            self.flash_level = self.level;
            return Some(self.level);
        }
        None
    }

    /// The level now (with the margin on the way down).
    pub fn level(&self) -> u8 {
        self.level
    }

    /// The flash's strength now, Q16 0..=ONE: full at the call, fading
    /// quadratically, scaled by the level it is for.
    pub fn flash(&self) -> i32 {
        let t = self.flash as i64 * ONE as i64 / FLASH as i64;
        let pct = FLASH_PCT[self.flash_level.min(3) as usize] as i64;
        (t * t / ONE as i64 * pct / 100) as i32
    }

    /// The flash as text with two decimals, for the page's opacity.
    pub fn flash_text(&self) -> String {
        let pct = (self.flash() as i64 * 100 / ONE as i64) as i32;
        format!("{}.{:02}", pct / 100, pct % 100)
    }

    /// The screen flash's opacity as text: the flash, or none while
    /// `SCREEN_FLASH` is off.
    pub fn screen_text(&self) -> String {
        if SCREEN_FLASH {
            self.flash_text()
        } else {
            "0.00".to_string()
        }
    }

    /// The level of a call-out whose flash is still showing (the page pops
    /// the shout while it is), or 0.
    pub fn burst(&self) -> u8 {
        if self.flash > 0 {
            self.flash_level
        } else {
            0
        }
    }
}

/// The sound key of a level's call-out (`sound::voice` makes it).
pub fn voice_key(level: u8) -> String {
    format!("hype{}", level.clamp(1, 3))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs `ticks` ticks at `kmh`, collecting the call-outs.
    fn hold_at(h: &mut Hype, kmh: i32, ticks: u32) -> Vec<u8> {
        (0..ticks).filter_map(|_| h.step(kmh)).collect()
    }

    #[test]
    fn crossing_a_level_calls_it_out_once() {
        let mut h = Hype::default();
        assert!(hold_at(&mut h, 30, 100).is_empty());
        assert_eq!(hold_at(&mut h, speedo::HYPE[0], 600), vec![1]);
        assert_eq!(h.level(), 1);
        // straight up to the top: the top is called at once, even inside
        // the hold of the first
        let mut h = Hype::default();
        assert_eq!(hold_at(&mut h, speedo::HYPE[0] + 1, 10), vec![1]);
        assert_eq!(hold_at(&mut h, speedo::HYPE[2] + 1, 10), vec![3]);
    }

    #[test]
    fn hovering_at_a_threshold_does_not_flicker_or_spam() {
        let mut h = Hype::default();
        let t = speedo::HYPE[1];
        let mut calls = Vec::new();
        // a swing's speed rising and falling 3 km/h round the threshold
        // every quarter second, for ten seconds
        for k in 0..40 {
            let kmh = if k % 2 == 0 { t + 1 } else { t - 3 };
            calls.extend(hold_at(&mut h, kmh, TPS / 4));
        }
        assert_eq!(calls, vec![2], "inside the margin: one call");
        // dropping well below and back: called again only after the hold
        let mut calls = Vec::new();
        for k in 0..40 {
            let kmh = if k % 2 == 0 { t + 1 } else { t - 10 };
            calls.extend(hold_at(&mut h, kmh, TPS / 4));
        }
        let most = (10 * TPS / HOLD) as usize + 1;
        assert!(!calls.is_empty() && calls.len() <= most, "{calls:?}");
    }

    #[test]
    fn the_flash_comes_with_a_call_and_fades_strongest_at_the_top() {
        let mut h = Hype::default();
        assert_eq!(h.flash(), 0);
        assert_eq!(h.burst(), 0);
        h.step(speedo::HYPE[0]);
        let one = h.flash();
        assert!(one > 0 && h.burst() == 1);
        h.step(speedo::HYPE[2]);
        let top = h.flash();
        assert!(top > one && h.burst() == 3, "{top} vs {one}");
        assert_eq!(h.flash_text(), "1.00");
        let mut last = top;
        for _ in 0..FLASH {
            h.step(speedo::HYPE[2]);
            assert!(h.flash() <= last);
            last = h.flash();
        }
        assert_eq!((h.flash(), h.burst()), (0, 0));
        assert_eq!(h.flash_text(), "0.00");
    }
}
