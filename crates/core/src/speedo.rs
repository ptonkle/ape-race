//! The speedometer (TRIAGE row 4): a car-style dial in km/h with a banana
//! for its needle. The core draws the face once as SVG markup and gives the
//! needle's turn each frame; the page only shows them. The faster the ape,
//! the higher the hype level, which the page shows as glow and shake.

use crate::fx::{self, deg, int, mul, ONE};

/// The dial reads 0 to this many km/h.
pub const TOP_KMH: i32 = 200;
/// From here the dial is red.
pub const REDLINE_KMH: i32 = 140;
/// The needle sweeps from -135° (0 km/h) to +135° (the top of the dial).
const SWEEP: i32 = 270;

/// km/h from a speed in Q16 m/s.
pub fn kmh(speed: i32) -> i32 {
    (speed as i64 * 36 / 10 / ONE as i64) as i32
}

/// How hyped the dial is, 0 to 3, from km/h. Set from measured speeds
/// (TRIAGE T8: the AI averages about 94 km/h and peaks at 137–155 km/h):
/// level 1 is a good swing, level 2 a fast one, level 3 the very top.
pub const HYPE: [i32; 3] = [85, 115, REDLINE_KMH];

pub fn hype(kmh: i32) -> u8 {
    HYPE.iter().filter(|h| kmh >= **h).count() as u8
}

/// The needle's angle in degrees for a reading (0 is straight up).
pub fn needle_degrees(kmh: i32) -> i32 {
    let k = kmh.clamp(0, TOP_KMH + 8);
    -SWEEP / 2 + k * SWEEP / TOP_KMH
}

/// The CSS transform that turns the needle.
pub fn needle(kmh: i32) -> String {
    format!("rotate({}deg)", needle_degrees(kmh))
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

fn arc(r: i32, from_kmh: i32, to_kmh: i32, colour: &str, width: i32) -> String {
    let (x0, y0) = at(r, needle_degrees(from_kmh));
    let (x1, y1) = at(r, needle_degrees(to_kmh));
    let large = if needle_degrees(to_kmh) - needle_degrees(from_kmh) > 180 {
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
pub fn face() -> String {
    let mut s = String::new();
    s.push_str("<circle cx=\"100\" cy=\"100\" r=\"96\" class=\"speedo-rim\"/>");
    s.push_str("<circle cx=\"100\" cy=\"100\" r=\"88\" class=\"speedo-face\"/>");
    s.push_str(&arc(int(78), 0, HYPE[0], "#7cc04a", int(7)));
    s.push_str(&arc(int(78), HYPE[0], HYPE[1], "#ffd447", int(7)));
    s.push_str(&arc(int(78), HYPE[1], REDLINE_KMH, "#ff9a3c", int(7)));
    s.push_str(&arc(int(78), REDLINE_KMH, TOP_KMH, "#ff4b3a", int(7)));
    let mut k = 0;
    while k <= TOP_KMH {
        let d = needle_degrees(k);
        let major = k % 40 == 0;
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
        k += 10;
    }
    s
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
        assert_eq!(needle(100), "rotate(0deg)");
    }

    #[test]
    fn km_per_hour_from_metres_per_second() {
        assert_eq!(kmh(int(10)), 36);
        assert_eq!(kmh(int(50)), 180);
        assert_eq!(kmh(0), 0);
    }

    #[test]
    fn hype_rises_with_speed() {
        assert_eq!(hype(50), 0);
        assert_eq!(hype(90), 1);
        assert_eq!(hype(120), 2);
        assert_eq!(hype(145), 3);
        // the AI's peaks (137–155 km/h) reach the top level, its average does not
        assert_eq!(hype(94), 1);
        assert_eq!(hype(150), 3);
    }

    #[test]
    fn the_face_has_a_number_every_40() {
        let f = face();
        for k in [0, 40, 80, 120, 160, 200] {
            assert!(f.contains(&format!(">{k}</text>")), "{k}");
        }
        assert_eq!(f.matches("<line").count(), 21);
        // 0 km/h sits lower left of the centre, the top lower right
        assert!(f.contains("class=\"speedo-num\">0<") && n(int(-3) / 2) == "-1.5");
    }
}
