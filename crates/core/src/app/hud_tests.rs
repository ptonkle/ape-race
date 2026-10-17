//! The HUD: the clock's text, places and the minimap (moved from app.rs's
//! tests).

use super::*;

fn app() -> App {
    App::new(include_str!("../../../../data/strings.json"), "", 7).unwrap()
}

#[test]
fn clock_text_is_minutes_seconds_hundredths() {
    assert_eq!(clock_text(0), "0:00.00");
    assert_eq!(clock_text(120 * 75 + 60), "1:15.50");
}

#[test]
fn ordinals() {
    assert_eq!(ordinal(1), "1st");
    assert_eq!(ordinal(2), "2nd");
    assert_eq!(ordinal(3), "3rd");
    assert_eq!(ordinal(4), "4th");
}

/// The minimap is the course seen from above, not from below (TRIAGE
/// T93): a point to the right of the ape's heading in the world lies
/// to the right of that heading on the map too.
#[test]
fn the_minimap_is_seen_from_above_not_mirrored() {
    for key in ["jungle", "city", "sector"] {
        let mut a = app();
        a.act("menu.race", "");
        a.act("setup.map", key);
        a.act("menu.start", "");
        let me = a.play.as_ref().unwrap().local;
        let (pos, fwd) = {
            let p = a.play.as_ref().unwrap();
            let r = &p.race.racers[me];
            (r.ape.pos, p.race.course.heading_at(r.progress))
        };
        let right = crate::geom::right_of(fwd);
        let mut map_at = |q: V3| -> (i64, i64) {
            a.play.as_mut().unwrap().race.racers[me].ape.pos = q;
            let ui: Value = serde_json::from_str(&a.ui()).unwrap();
            let d = &ui["hud"]["minimap"]["apes"];
            let d = d
                .as_array()
                .unwrap()
                .iter()
                .find(|d| d["me"] == true)
                .unwrap();
            (d["x"].as_i64().unwrap(), d["y"].as_i64().unwrap())
        };
        let o = map_at(pos);
        let f = map_at(pos.add(fwd.scale(int(40))));
        let r = map_at(pos.add(right.scale(int(40))));
        // on the page (y down) the right of a heading (fx, fy) is
        // (-fy, fx): a quarter turn clockwise
        let (fx_, fy) = (f.0 - o.0, f.1 - o.1);
        let (rx, ry) = (r.0 - o.0, r.1 - o.1);
        let side = rx * -fy + ry * fx_;
        assert!(
            side > 0,
            "{key}: right of the heading is drawn on its left (fwd {fx_},{fy}, right {rx},{ry})"
        );
    }
}

#[test]
fn the_ski_slopes_minimap_shows_both_branches_of_each_fork() {
    let mut a = app();
    a.act("menu.race", "");
    a.act("setup.map", "ski");
    a.act("menu.start", "");
    let ui: Value = serde_json::from_str(&a.ui()).unwrap();
    let m = &ui["hud"]["minimap"];
    let branches = m["branches"].as_array().unwrap();
    assert_eq!(branches.len(), 2, "one more branch per fork");
    let line: Vec<i64> = m["line"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_i64())
        .collect();
    for b in branches {
        let b: Vec<i64> = b
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_i64())
            .collect();
        assert!(b.len() >= 8);
        // the branch's middle is apart from every point of the run's line
        let (x, y) = (b[(b.len() / 2) & !1], b[((b.len() / 2) & !1) + 1]);
        let near = line
            .chunks(2)
            .map(|p| (p[0] - x).abs() + (p[1] - y).abs())
            .min()
            .unwrap();
        assert!(near > 4, "the branch's middle is {near} from the line");
    }
    // the other maps have none
    let mut j = app();
    j.act("menu.race", "");
    j.act("menu.start", "");
    let ui: Value = serde_json::from_str(&j.ui()).unwrap();
    assert!(ui["hud"]["minimap"]["branches"]
        .as_array()
        .unwrap()
        .is_empty());
}
