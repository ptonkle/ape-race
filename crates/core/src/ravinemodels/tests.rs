//! Smoke tests: every model draws something and stays within its shape
//! budget (the scene's 5,500 shapes on screen), and the skull's teeth leave
//! exactly the gap asked for.

use super::*;
use crate::fx::cm;

/// Shapes and halos `draw` pushes.
fn shapes(draw: &dyn Fn(&mut Batch)) -> (usize, usize) {
    let mut b = Batch::new();
    draw(&mut b);
    let halos = b.take_halos().count();
    (b.count(), halos)
}

#[test]
fn every_ravine_model_draws_something() {
    for key in GALLERY {
        let mut b = Batch::new();
        gallery(&mut b, key, &Affine::IDENTITY, 240);
        assert!(b.count() > 3, "{key}");
    }
}

#[test]
fn the_ravine_gallery_keys_are_its_own() {
    for key in GALLERY {
        assert!(!crate::baymodels::GALLERY.contains(&key), "{key}");
        assert!(!crate::ruinsmodels::GALLERY.contains(&key), "{key}");
        assert!(!crate::towermodels::GALLERY.contains(&key), "{key}");
    }
}

#[test]
fn the_ravine_models_stay_within_budget() {
    let id = Affine::IDENTITY;
    let t = 300;
    let rows: Vec<(&str, (usize, usize), usize)> = vec![
        (
            "snake vine 10 m",
            shapes(&|b| snake_vine(b, &id, int(10), t, 1)),
            48,
        ),
        (
            "snake snapped",
            shapes(&|b| snake_snapped(b, &id, int(10), int(3), t, 1)),
            56,
        ),
        (
            "lizard",
            shapes(&|b| lizard(b, &id, 0, LizardHue::Blue)),
            60,
        ),
        ("mine cart", shapes(&|b| mine_cart(b, &id, t, 1, true)), 34),
        (
            "track 20 m raised",
            shapes(&|b| mine_track(b, &id, v3(0, int(4), 0), v3(0, int(4), int(20)), Some(0), 1)),
            60,
        ),
        (
            "wall hole",
            shapes(&|b| wall_hole(b, &id, int(5), int(5), 1)),
            18,
        ),
        (
            "rex skull",
            shapes(&|b| rex_skull(b, &id, REX_GAP, &rex_teeth(REX_GAP))),
            60,
        ),
        ("sphinx", shapes(&|b| sphinx(b, &id)), 60),
        ("campfire", shapes(&|b| campfire(b, &id, t, 1)), 40),
        ("tent", shapes(&|b| tent(b, &id, TENT_CANVAS, 1)), 16),
        ("pickaxe", shapes(&|b| pickaxe(b, &id, 0)), 5),
        ("lantern", shapes(&|b| lantern(b, &id, t, 1)), 10),
        ("ore bucket", shapes(&|b| ore_bucket(b, &id, 1)), 11),
        (
            "mine support",
            shapes(&|b| mine_support(b, &id, int(6), int(5), 1)),
            9,
        ),
        (
            "dig marks",
            shapes(&|b| dig_marks(b, &id, int(4), int(3), t, 1)),
            19,
        ),
        (
            "rope bridge 16 m",
            shapes(&|b| rope_bridge_span(b, &id, int(16), int(3), t, 1)),
            80,
        ),
        (
            "log span 16 m",
            shapes(&|b| log_span(b, &id, int(16), cm(60), 1)),
            14,
        ),
        (
            "ribs",
            shapes(&|b| fossil_inlay(b, &id, Fossil::Ribs, 1)),
            13,
        ),
        (
            "spine",
            shapes(&|b| fossil_inlay(b, &id, Fossil::Spine, 1)),
            16,
        ),
        (
            "ammonite",
            shapes(&|b| fossil_inlay(b, &id, Fossil::Ammonite, 1)),
            10,
        ),
        ("mushrooms", shapes(&|b| mushrooms(b, &id, t, 1)), 11),
        ("torch", shapes(&|b| torch(b, &id, t, 1)), 6),
        ("fireflies", shapes(&|b| fireflies(b, &id, int(3), t, 1)), 9),
    ];
    let table: Vec<String> = rows
        .iter()
        .map(|(name, (n, h), most)| format!("{name}: {n} shapes + {h} halos (budget {most})"))
        .collect();
    for (name, (n, _), most) in &rows {
        assert!(
            *n > 0 && n <= most,
            "{name} over budget\n{}",
            table.join("\n")
        );
    }
}

#[test]
fn the_rex_teeth_leave_exactly_the_gap_asked_for() {
    for gap in [REX_GAP, int(3) + 7, int(5) + ONE / 3] {
        let teeth = rex_teeth(gap);
        assert_eq!(teeth.len(), 28);
        let lowest_upper = teeth
            .iter()
            .filter(|t| t.upper())
            .map(|t| t.tip.y)
            .min()
            .unwrap();
        let highest_lower = teeth
            .iter()
            .filter(|t| !t.upper())
            .map(|t| t.tip.y)
            .max()
            .unwrap();
        assert_eq!(lowest_upper - highest_lower, gap, "gap {gap}");
        // the bone stays out of the mouth
        for (mid, size) in rex_bone_boxes(gap) {
            let (lo, hi) = (mid.y - size.y / 2, mid.y + size.y / 2);
            assert!(
                lo >= lowest_upper || hi <= highest_lower,
                "bone in the gap {gap}"
            );
        }
    }
}

#[test]
fn a_snapped_snake_drops_its_head_half() {
    let whole = shapes(&|b| snake_snapped(b, &Affine::IDENTITY, int(10), 0, 0, 1));
    let fallen = shapes(&|b| snake_snapped(b, &Affine::IDENTITY, int(10), int(4), 0, 1));
    // the same pieces, whatever the fall
    assert_eq!(whole, fallen);
    let pts = snake_points(int(10), 0, 1);
    // the hook lifts the head: the last point is above the one before
    let n = pts.len();
    assert!(pts[n - 1].y > pts[n - 2].y);
}

#[test]
fn a_lizard_on_a_wall_lies_flat_on_it() {
    let g = on_surface(v3(0, int(5), 0), v3(0, 0, ONE), V3::UP);
    // its belly's normal (+y) is the wall's, its head (+z) points up
    assert!(g.c1.z > ONE * 99 / 100);
    assert!(g.c2.y > ONE * 99 / 100);
}
