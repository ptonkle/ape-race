//! Smoke tests: every model builds, draws something, and stays within its
//! share of the map's scenery budget (MAPS.md: about 20k shapes on a map,
//! 6–7k drawn at once).

use super::*;
use crate::fx::int;

/// Shapes and halos `draw` pushes.
fn shapes(draw: &dyn Fn(&mut Batch)) -> (usize, usize) {
    let mut b = Batch::new();
    draw(&mut b);
    let halos = b.take_halos().count();
    (b.count(), halos)
}

#[test]
fn every_tower_model_draws_something() {
    for key in GALLERY {
        let mut b = Batch::new();
        gallery(&mut b, key, &Affine::IDENTITY, 240);
        assert!(b.count() > 5, "{key}");
    }
}

#[test]
fn the_tower_gallery_keys_are_its_own() {
    for key in GALLERY {
        assert!(!crate::baymodels::GALLERY.contains(&key), "{key}");
        assert!(!crate::ruinsmodels::GALLERY.contains(&key), "{key}");
    }
}

/// Each model under its budget (shapes, halos). The table, with `SHOW`
/// set, is in the last assertion's message (the core may not print).
#[test]
fn the_tower_models_stay_within_budget() {
    let id = Affine::IDENTITY;
    let t = 120;
    let rows: Vec<(&str, (usize, usize), usize)> = vec![
        (
            "keep section 12 m near",
            shapes(&|b| keep_section(b, &id, int(10), SECTION_H, 1, Detail::Near)),
            85,
        ),
        (
            "keep section 12 m far",
            shapes(&|b| keep_section(b, &id, int(10), SECTION_H, 1, Detail::Far)),
            20,
        ),
        (
            "keep base",
            shapes(&|b| keep_base(b, &id, int(10), int(6), 1)),
            20,
        ),
        (
            "keep crown (4 turrets)",
            shapes(&|b| keep_crown(b, &id, int(10), 4, 1)),
            160,
        ),
        (
            "keep, 8 sections",
            shapes(&|b| keep(b, &id, int(10), 8, 1)),
            850,
        ),
        (
            "turret 12 m",
            shapes(&|b| turret(b, &id, int(3), int(12), 1)),
            20,
        ),
        (
            "sub-tower 30 m",
            shapes(&|b| sub_tower(b, &id, int(6), int(30), 1)),
            240,
        ),
        (
            "arch bridge 24 m",
            shapes(&|b| arch_bridge(b, &id, V3::ZERO, v3(int(24), 0, 0), int(4), 1)),
            30,
        ),
        (
            "rope bridge 24 m",
            shapes(&|b| rope_bridge(b, &id, V3::ZERO, v3(int(24), 0, 0), int(2), int(2), 1)),
            80,
        ),
        (
            "magic bridge 24 m",
            shapes(&|b| magic_bridge(b, &id, V3::ZERO, v3(int(24), 0, 0), int(3), Spell::Frost, t)),
            30,
        ),
        (
            "balcony 5 m",
            shapes(&|b| balcony(b, &id, int(5), int(2), 1)),
            26,
        ),
        (
            "mage casting",
            shapes(&|b| mage(b, &id, Spell::Arcane, MagePose::Cast, t, 0)),
            16,
        ),
        (
            "mage riding",
            shapes(&|b| mage(b, &id, Spell::Fire, MagePose::Ride, t, 1)),
            16,
        ),
        (
            "dragon and rider",
            shapes(&|b| {
                dragon(b, &id, Hide::Emerald, Wings::Flap, Spell::Fire, t, 1);
            }),
            75,
        ),
        (
            "spell beam 20 m",
            shapes(&|b| spell_beam(b, &id, V3::ZERO, v3(0, int(20), 0), Spell::Charm, t, 1)),
            30,
        ),
        (
            "spell anchor",
            shapes(&|b| spell_anchor(b, &id, V3::ZERO, Spell::Frost, t, 1)),
            12,
        ),
        (
            "trampoline rune",
            shapes(&|b| trampoline(b, &id, int(2), Pad::Rune, Spell::Arcane, 0, t)),
            24,
        ),
        (
            "trampoline cloth",
            shapes(&|b| trampoline(b, &id, int(2), Pad::Cloth, Spell::Charm, ONE / 2, t)),
            20,
        ),
        (
            "trampoline jelly",
            shapes(&|b| trampoline(b, &id, int(2), Pad::Jelly, Spell::Verdant, ONE / 2, t)),
            8,
        ),
        (
            "trampoline room",
            shapes(&|b| trampoline_room(b, &id, int(14), int(22), false, t, 1)),
            460,
        ),
        (
            "cloud 30 m",
            shapes(&|b| cloud(b, &id, int(30), int(8), 1)),
            16,
        ),
        (
            "cloud anchor",
            shapes(&|b| cloud_anchor(b, &id, Spell::Arcane, t, 1)),
            12,
        ),
        ("frost disc", shapes(&|b| frost_disc(b, &id, int(2), 1)), 14),
        (
            "rune stone",
            shapes(&|b| rune_stone(b, &id, int(3), t, 1)),
            12,
        ),
        (
            "crystal ledge",
            shapes(&|b| crystal_ledge(b, &id, int(4), int(2), 1)),
            10,
        ),
        ("mad wizard", shapes(&|b| mad_wizard(b, &id, t)), 50),
        (
            "rift",
            shapes(&|b| rift(b, &id, int(16), int(30), ONE, t)),
            110,
        ),
        (
            "eldritch banana",
            shapes(&|b| eldritch_banana(b, &id, int(80), t)),
            120,
        ),
        ("summit", shapes(&|b| summit(b, &id, t)), 500),
    ];
    for (name, (n, halos), max) in &rows {
        assert!(*n > 0, "{name} draws nothing");
        assert!(*n <= *max, "{name}: {n} shapes over {max}");
        assert!(*halos <= 40, "{name}: {halos} halos");
    }
    const SHOW: bool = false;
    assert!(!SHOW && !rows.is_empty(), "{rows:#?}");
}

#[test]
fn the_magic_moves_and_the_stone_holds_still() {
    let draw = |tick: u32| {
        let mut b = Batch::new();
        dragon(
            &mut b,
            &Affine::IDENTITY,
            Hide::Bronze,
            Wings::Flap,
            Spell::Verdant,
            tick,
            1,
        );
        b.finish(1).0
    };
    assert_ne!(draw(0), draw(20));
    // the stone takes no tick: two builds are the same
    let stone = || {
        let mut b = Batch::new();
        keep(&mut b, &Affine::IDENTITY, int(10), 3, 4);
        b.finish(0).0
    };
    assert_eq!(stone(), stone());
}

#[test]
fn a_riders_beam_starts_at_the_staff_tip() {
    let mut b = Batch::new();
    let f = Affine::translate(v3(int(40), int(80), 0));
    let tip = dragon(&mut b, &f, Hide::Crimson, Wings::Glide, Spell::Fire, 0, 1);
    // over the saddle, within reach of the rider's raised arm
    let seat = f.apply(SADDLE);
    assert!(
        tip.y > seat.y + int(1) && tip.dist(seat) < int(3),
        "{tip:?} {seat:?}"
    );
}
