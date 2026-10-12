//! The tab icon (TRIAGE T4; every game ships with one): a banana on a
//! jungle-green disc, as an SVG data URL the page puts in the tab.

/// The icon's SVG.
pub fn svg() -> &'static str {
    concat!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'>",
        "<circle cx='32' cy='32' r='30' fill='#2f7a3a'/>",
        "<path d='M14 40 C 22 50, 44 50, 52 26 L 55 24 C 52 44, 30 58, 12 44 Z' fill='#ffd93b' stroke='#a8790a' stroke-width='2'/>",
        "<path d='M52 26 L 55 24 L 56 18 L 52 19 Z' fill='#5a3b14'/>",
        "<circle cx='13' cy='42' r='2.5' fill='#5a3b14'/>",
        "</svg>"
    )
}

/// The icon as a data URL: the SVG with the characters a URL may not hold
/// raw percent-encoded.
pub fn data_url() -> String {
    let mut s = String::from("data:image/svg+xml,");
    for c in svg().chars() {
        match c {
            '#' => s.push_str("%23"),
            '<' => s.push_str("%3C"),
            '>' => s.push_str("%3E"),
            ' ' => s.push_str("%20"),
            _ => s.push(c),
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_icon_is_an_svg_data_url_with_nothing_raw() {
        let u = data_url();
        assert!(u.starts_with("data:image/svg+xml,%3Csvg"));
        assert!(!u[19..].contains(['#', '<', '>', ' ']));
        assert!(u.contains("ffd93b"));
    }
}
