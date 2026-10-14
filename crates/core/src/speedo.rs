//! The speedometer (TRIAGE row 4): a car-style dial in km/h with a banana
//! for its needle. The core draws the face once as SVG markup and gives the
//! needle's turn each frame; the page only shows them. The faster the ape,
//! the higher the hype level, which the page shows as glow and shake.

use crate::fx::{self, deg, int, mul, ONE};

/// The dial reads 0 to this many km/h.
pub const TOP_KMH: i32 = 140;
/// From here the dial is red.
pub const REDLINE_KMH: i32 = 75;
/// The needle sweeps from -135° (0 km/h) to +135° (the top of the dial).
const SWEEP: i32 = 270;

/// km/h from a speed in Q16 m/s.
pub fn kmh(speed: i32) -> i32 {
    (speed as i64 * 36 / 10 / ONE as i64) as i32
}

/// How hyped the dial is, 0 to 3, from km/h. Set from what a person
/// reaches (TRIAGE row 16: on the fastest ape the person peaks around
/// 90 km/h; the AI is faster, averaging 94 and peaking at 137–155):
/// level 1 is a good swing, level 2 a fast one, level 3 a person's best.
/// Read from the smoothed horizontal speed (TRIAGE row 19): the AI's sits
/// at 94–110 km/h (half to 97% of the time), a person's lower.
pub const HYPE: [i32; 3] = [50, 62, REDLINE_KMH];

/// A speedometer's scale: what the dial reads to, where it turns red, its
/// hype levels, and how far apart its numbers and ticks are. Every map's
/// is `Dial::STANDARD` but Sector Ape's, about twice as fast
/// (`Dial::SPACE`, the person: up to ~300 km/h).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dial {
    pub top: i32,
    pub redline: i32,
    pub hype: [i32; 3],
    /// a number every this many km/h, a tick every `tick`
    pub number: i32,
    pub tick: i32,
}

impl Dial {
    /// The dial on the Jungle, the City and the Ski Slope.
    pub const STANDARD: Dial = Dial {
        top: TOP_KMH,
        redline: REDLINE_KMH,
        hype: HYPE,
        number: 40,
        tick: 10,
    };

    /// Sector Ape's: about twice the pace, the hype levels with it.
    pub const SPACE: Dial = Dial {
        top: 300,
        redline: REDLINE_KMH * 2,
        hype: [HYPE[0] * 2, HYPE[1] * 2, REDLINE_KMH * 2],
        number: 50,
        tick: 25,
    };

    /// The dial for a map.
    pub fn on(map: crate::map::Map) -> Dial {
        match map {
            crate::map::Map::Sector => Dial::SPACE,
            _ => Dial::STANDARD,
        }
    }

    /// How hyped the dial is, 0 to 3, from km/h.
    pub fn hype(&self, kmh: i32) -> u8 {
        self.hype.iter().filter(|h| kmh >= **h).count() as u8
    }

    /// How hyped the dial looks, continuously: 0 at the first hype level,
    /// 1 at the top of the dial, as text with two decimals.
    pub fn amount_text(&self, kmh: i32) -> String {
        let pct = ((kmh - self.hype[0]) * 100 / (self.top - self.hype[0])).clamp(0, 100);
        format!("{}.{:02}", pct / 100, pct % 100)
    }

    /// The needle's angle in degrees for a reading (0 is straight up); it
    /// pins a little past the top.
    pub fn needle_degrees(&self, kmh: i32) -> i32 {
        let k = kmh.clamp(0, self.top + self.top / 28);
        -SWEEP / 2 + k * SWEEP / self.top
    }

    /// The CSS transform that turns the needle.
    pub fn needle(&self, kmh: i32) -> String {
        format!("rotate({}deg)", self.needle_degrees(kmh))
    }

    /// What a speed (Q16 m/s) on this dial is worth on the standard dial:
    /// the same share of the way round. The sky's colours at speed (TRIAGE
    /// T20) and the hype's shout read it.
    pub fn standard(&self, speed: i32) -> i32 {
        (speed as i64 * Dial::STANDARD.top as i64 / self.top as i64) as i32
    }

    fn arc(&self, r: i32, from_kmh: i32, to_kmh: i32, colour: &str, width: i32) -> String {
        let (x0, y0) = at(r, self.needle_degrees(from_kmh));
        let (x1, y1) = at(r, self.needle_degrees(to_kmh));
        let large = if self.needle_degrees(to_kmh) - self.needle_degrees(from_kmh) > 180 {
            1
        } else {
            0
        };
        format!(
            "<path d=\"M {} {} A {} {} 0 {large} 1 {} {}\" fill=\"none\" stroke=\"{colour}\" stroke-width=\"{}\" stroke-linecap=\"round\"/>",
            n(x0),
            n(y0),
            n(r),
            n(r),
            n(x1),
            n(y1),
            n(width)
        )
    }

    /// The dial's face, without the needle: rim, coloured bands, ticks and
    /// numbers, in a 200 × 200 box centred at (100, 100).
    pub fn face(&self) -> String {
        let mut s = String::new();
        s.push_str("<circle cx=\"100\" cy=\"100\" r=\"96\" class=\"speedo-rim\"/>");
        s.push_str("<circle cx=\"100\" cy=\"100\" r=\"88\" class=\"speedo-face\"/>");
        s.push_str(&self.arc(int(78), 0, self.hype[0], "#7cc04a", int(7)));
        s.push_str(&self.arc(int(78), self.hype[0], self.hype[1], "#ffd447", int(7)));
        s.push_str(&self.arc(int(78), self.hype[1], self.redline, "#ff9a3c", int(7)));
        s.push_str(&self.arc(int(78), self.redline, self.top, "#ff4b3a", int(7)));
        let mut k = 0;
        while k <= self.top {
            let d = self.needle_degrees(k);
            let major = k % self.number == 0;
            let (x0, y0) = at(int(if major { 64 } else { 69 }), d);
            let (x1, y1) = at(int(73), d);
            s.push_str(&format!(
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" class=\"{}\"/>",
                n(x0),
                n(y0),
                n(x1),
                n(y1),
                if major { "tick major" } else { "tick" }
            ));
            if major {
                let (x, y) = at(int(51), d);
                s.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" class=\"speedo-num\">{k}</text>",
                    n(x),
                    n(y + int(4))
                ));
            }
            k += self.tick;
        }
        s
    }
}

pub fn hype(kmh: i32) -> u8 {
    Dial::STANDARD.hype(kmh)
}

/// How hyped the dial looks, continuously (TRIAGE row 15, a try): 0 at the
/// first hype level, 1 at the top of the dial, as text with two decimals
/// for the page's glow and shake.
pub fn amount_text(kmh: i32) -> String {
    Dial::STANDARD.amount_text(kmh)
}

/// The needle's angle in degrees for a reading (0 is straight up).
pub fn needle_degrees(kmh: i32) -> i32 {
    Dial::STANDARD.needle_degrees(kmh)
}

/// The CSS transform that turns the needle.
pub fn needle(kmh: i32) -> String {
    Dial::STANDARD.needle(kmh)
}

/// A Q16 value as text with one decimal, for SVG coordinates.
fn n(v: i32) -> String {
    let tenths = (v as i64 * 10
        + if v >= 0 {
            ONE as i64 / 2
        } else {
            -(ONE as i64) / 2
        })
        / ONE as i64;
    let sign = if tenths < 0 { "-" } else { "" };
    let t = tenths.abs();
    format!("{sign}{}.{}", t / 10, t % 10)
}

/// The point at `r` from the dial's centre (100, 100), at dial degrees `d`.
fn at(r: i32, d: i32) -> (i32, i32) {
    let a = deg(d);
    (int(100) + mul(r, fx::sin(a)), int(100) - mul(r, fx::cos(a)))
}

/// The dial's face, without the needle: rim, coloured bands, ticks and
/// numbers, in a 200 × 200 box centred at (100, 100).
pub fn face() -> String {
    Dial::STANDARD.face()
}

/// The banana needle, pointing straight up from the centre; the page turns
/// its group by [`needle`].
pub fn banana() -> String {
    concat!(
        "<path d=\"M 94 110 C 84 80 87 50 99 27 L 104 25 C 99 50 100 79 108 108 Q 101 115 94 110 Z\" class=\"banana\"/>",
        "<path d=\"M 99 30 C 94 52 94 80 100 106\" class=\"banana-shine\"/>",
        "<path d=\"M 99 27 L 104 25 L 103 18 L 98.5 20 Z\" class=\"banana-stem\"/>",
        "<circle cx=\"101\" cy=\"110\" r=\"3\" class=\"banana-stem\"/>"
    )
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_needle_sweeps_from_left_to_right() {
        assert_eq!(needle_degrees(0), -135);
        assert_eq!(needle_degrees(TOP_KMH / 2), 0);
        assert_eq!(needle_degrees(TOP_KMH), 135);
        assert!(needle_degrees(9999) < 150);
        assert_eq!(needle(70), "rotate(0deg)");
    }

    #[test]
    fn km_per_hour_from_metres_per_second() {
        assert_eq!(kmh(int(10)), 36);
        assert_eq!(kmh(int(50)), 180);
        assert_eq!(kmh(0), 0);
    }

    #[test]
    fn the_hype_amount_follows_the_exact_speed() {
        assert_eq!(amount_text(0), "0.00");
        assert_eq!(amount_text(HYPE[0]), "0.00");
        assert_eq!(amount_text(TOP_KMH), "1.00");
        assert_ne!(amount_text(120), amount_text(121 + 2));
    }

    #[test]
    fn hype_rises_with_speed() {
        assert_eq!(hype(45), 0);
        assert_eq!(hype(55), 1);
        assert_eq!(hype(65), 2);
        // a good human pace across the ground reaches the top
        assert_eq!(hype(78), 3);
    }

    #[test]
    fn sector_apes_dial_reads_to_300_with_the_hype_levels_doubled() {
        // the person (2026-10-13): about double the other maps, ~300 km/h
        let d = Dial::on(crate::map::Map::Sector);
        assert_eq!(d.top, 300);
        assert_eq!(d.needle_degrees(150), 0);
        assert_eq!(d.hype(99), 0);
        assert_eq!(d.hype(2 * 78), 3);
        for k in [0, 50, 100, 150, 200, 250, 300] {
            assert!(d.face().contains(&format!(">{k}</text>")), "{k}");
        }
        assert_eq!(d.standard(int(84)), int(84) * 140 / 300);
        // every other map keeps the standard dial, unchanged
        for m in [
            crate::map::Map::Jungle,
            crate::map::Map::City,
            crate::map::Map::Ski,
        ] {
            assert_eq!(Dial::on(m), Dial::STANDARD);
        }
        assert_eq!(Dial::STANDARD.standard(int(30)), int(30));
    }

    #[test]
    fn the_face_has_a_number_every_40() {
        let f = face();
        for k in [0, 40, 80, 120] {
            assert!(f.contains(&format!(">{k}</text>")), "{k}");
        }
        assert_eq!(f.matches("<line").count(), 15);
        // 0 km/h sits lower left of the centre, the top lower right
        assert!(f.contains("class=\"speedo-num\">0<") && n(int(-3) / 2) == "-1.5");
    }
}
