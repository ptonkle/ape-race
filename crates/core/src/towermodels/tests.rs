//! Smoke tests: every model builds, draws something, and stays within its
//! share of the map's scenery budget (MAPS.md: about 20k shapes on a map,
//! 6–7k drawn at once).

use super::*;
use crate::fx::{cm, int, TURN};
use crate::mesh::Mesh;
use crate::render::Rgb;

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
            shapes(&|b| trampoline_room(b, &id, int(14), int(22), int(5), false, t, 1)),
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
            140,
        ),
        (
            "eldritch banana",
            shapes(&|b| eldritch_banana(b, &id, int(80), t)),
            170,
        ),
        ("summit", shapes(&|b| summit(b, &id, t)), 560),
        (
            "zip 100 m",
            shapes(&|b| zip(b, &id, int(100), int(3), ONE / 2, t)),
            95,
        ),
        (
            "topsy keep, 3 sections",
            shapes(&|b| topsy_keep(b, &id, int(10), 3, deg(15), 1)),
            420,
        ),
        ("debris", shapes(&|b| debris(b, &id, int(40), t, 1)), 18),
        (
            "blob",
            shapes(&|b| {
                blob(b, &id, int(3), Spell::Brine, t, 2);
            }),
            25,
        ),
        (
            "sea monster",
            shapes(&|b| {
                sea_monster(b, &id, Spell::Abyss, t, 1);
            }),
            85,
        ),
        (
            "sea beam 20 m",
            shapes(&|b| sea_beam(b, &id, V3::ZERO, v3(0, int(20), 0), Spell::Brine, t, 1)),
            34,
        ),
        (
            "wreck plank",
            shapes(&|b| wreck_plank(b, &id, int(8), int(4), 1)),
            40,
        ),
        (
            "coral shelf",
            shapes(&|b| coral_shelf(b, &id, int(3), 1)),
            30,
        ),
        (
            "bubble disc",
            shapes(&|b| bubble_disc(b, &id, int(3), t, 1)),
            16,
        ),
        (
            "banana sea",
            shapes(&|b| banana_sea(b, &id, int(400), t)),
            420,
        ),
        // the second design (TM2)
        (
            "hollow section 12 m near, a door and a window, a ledge",
            shapes(&|b| {
                let mut cuts = [Cut::Wall; 8];
                cuts[0] = Cut::Door;
                cuts[4] = Cut::Window;
                hollow_section(
                    b,
                    &id,
                    TOWER1_OUT,
                    TOWER1_IN,
                    SECTION_H,
                    1,
                    cuts,
                    Ledge::Walk,
                    Detail::Near,
                )
            }),
            150,
        ),
        (
            "hollow section 12 m far",
            shapes(&|b| {
                hollow_section(
                    b,
                    &id,
                    TOWER1_OUT,
                    TOWER1_IN,
                    SECTION_H,
                    1,
                    [Cut::Wall; 8],
                    Ledge::Walk,
                    Detail::Far,
                )
            }),
            30,
        ),
        (
            "hollow top (4 turrets)",
            shapes(&|b| hollow_top(b, &id, TOWER1_OUT, TOWER1_IN, 4, 1)),
            170,
        ),
        (
            "tower 1 whole, 274 m",
            shapes(&|b| {
                tower1(
                    b,
                    &id,
                    TOWER1_OUT,
                    TOWER1_IN,
                    TOWER1_HEIGHT,
                    TOWER1_HEIGHT / 2,
                    2,
                    1,
                )
            }),
            3_000,
        ),
        (
            "tower 2 whole, 164 m, 8 balconies, 6 clouds",
            shapes(&|b| {
                tower2(
                    b,
                    &id,
                    TOWER2_RADIUS,
                    TOWER2_HEIGHT,
                    8,
                    TURN * 3 / 2,
                    0,
                    6,
                    1,
                );
            }),
            1_800,
        ),
        (
            "long bridge 250 m",
            shapes(&|b| long_bridge(b, &id, V3::ZERO, v3(BRIDGE_LENGTH, 0, 0), BRIDGE_WIDTH, 1)),
            200,
        ),
        (
            "bridge hangers 250 m",
            shapes(&|b| {
                bridge_hangers(
                    b,
                    &id,
                    V3::ZERO,
                    v3(BRIDGE_LENGTH, 0, 0),
                    BRIDGE_WIDTH,
                    HANGER_SPACING,
                    1,
                );
            }),
            70,
        ),
        (
            "tube 300 m, 10 portholes",
            shapes(&|b| {
                let holes: Vec<Porthole> = (0..10)
                    .map(|i| Porthole {
                        depth: int(20 + 27 * i),
                        bearing: deg(70 * i),
                    })
                    .collect();
                tube(b, &id, TUBE_RADIUS, TUBE_LENGTH, &holes, 1);
            }),
            1_100,
        ),
        (
            "void trampoline",
            shapes(&|b| void_trampoline(b, &id, crate::tower::DISC_RADIUS, ONE / 2, t, 1)),
            12,
        ),
        (
            "tunnel mouth",
            shapes(&|b| {
                tunnel_mouth(b, &id, TUNNEL_RADIUS, t, 1);
            }),
            60,
        ),
        (
            "tunnel 250 m",
            shapes(&|b| tunnel(b, &id, TUNNEL_RADIUS, TUNNEL_LENGTH, 1)),
            600,
        ),
        (
            "portal 56 by 50 m",
            shapes(&|b| portal(b, &id, TEAR_W, TEAR_H, ONE, int(1), t)),
            260,
        ),
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

#[test]
fn the_descents_creatures_start_their_vines_at_their_tips() {
    let f = Affine::translate(v3(int(10), -int(40), int(5)));
    let mut b = Batch::new();
    let tip = blob(&mut b, &f, int(3), Spell::Abyss, 77, 1);
    assert_eq!(tip, f.apply(blob_beam_start(int(3))));
    let tip = sea_monster(&mut b, &f, Spell::Brine, 77, 1);
    assert_eq!(tip, f.apply(MONSTER_STAFF_TIP));
    // the staff tip is above the monster's head
    assert!(MONSTER_STAFF_TIP.y > int(5), "{MONSTER_STAFF_TIP:?}");
}

#[test]
fn the_upside_down_pieces_turn_sickly() {
    // the same section upright and hung upside down: as many shapes, but
    // recoloured, and the climb's limestone no longer itself
    let count = |topsy_on: bool| {
        let mut b = Batch::new();
        if topsy_on {
            topsy(&mut b, &Affine::IDENTITY, 0, deg(10), |b, g| {
                keep_section(b, g, int(10), SECTION_H, 1, Detail::Near)
            });
        } else {
            keep_section(
                &mut b,
                &Affine::IDENTITY,
                int(10),
                SECTION_H,
                1,
                Detail::Near,
            );
        }
        b.count()
    };
    assert_eq!(count(true), count(false));
    assert_ne!(sickly(STONE, 0), STONE);
    assert_ne!(sickly(WINDOW, ONE), WINDOW);
}

#[test]
fn the_deep_spells_are_not_the_climbs() {
    for d in Spell::DEEP {
        assert!(!Spell::ALL.contains(&d));
    }
    // the five climbing spells still come round in turn
    assert_eq!(Spell::nth(5), Spell::Arcane);
}

// ---------------------------------------------------------------------------
// The second design's models (TM2).

/// Each shape in a batch: its mesh, matrix, colour and glow.
fn all_shapes(own: Batch) -> Vec<(Mesh, Affine, Rgb, i32)> {
    let (values, draws) = own.finish(crate::render::DYNAMIC_BUFFER);
    let mut out = Vec::new();
    for d in draws.chunks(crate::render::DRAW_VALUES) {
        let mesh = Mesh::ALL[d[1] as usize];
        let first = d[2] as usize / crate::render::BYTES_PER_VALUE;
        for i in values[first..first + d[3] as usize * crate::render::INSTANCE_VALUES]
            .chunks(crate::render::INSTANCE_VALUES)
        {
            let a = Affine {
                c0: v3(i[0], i[4], i[8]),
                c1: v3(i[1], i[5], i[9]),
                c2: v3(i[2], i[6], i[10]),
                t: v3(i[3], i[7], i[11]),
            };
            out.push((mesh, a, [i[12], i[13], i[14]], i[15]));
        }
    }
    out
}

/// The horizontal distance of `p` from the y axis.
fn off_axis(p: V3) -> i32 {
    v3(p.x, 0, p.z).len()
}

#[test]
fn tower_1_is_hollow_inside_and_open_at_the_top() {
    let mut b = Batch::new();
    let mut cuts = [Cut::Wall; 8];
    cuts[2] = Cut::Window;
    hollow_section(
        &mut b,
        &Affine::IDENTITY,
        TOWER1_OUT,
        TOWER1_IN,
        SECTION_H,
        1,
        cuts,
        Ledge::Walk,
        Detail::Near,
    );
    hollow_top(
        &mut b,
        &Affine::translate(v3(0, SECTION_H, 0)),
        TOWER1_OUT,
        TOWER1_IN,
        4,
        1,
    );
    let inner = TOWER1_IN - LEDGE_DEPTH - int(1);
    for (mesh, a, _, _) in all_shapes(b) {
        assert!(
            off_axis(a.t) >= inner,
            "{mesh:?} at {:?} is {} m from the axis, inside the wall's {} m",
            a.t,
            off_axis(a.t) / ONE,
            inner / ONE
        );
    }
    // the inside is about 90 m across, the outside 1.25 keeps
    assert!(2 * TOWER1_IN >= int(88) && 2 * TOWER1_IN <= int(92));
    assert_eq!(TOWER1_OUT, int(51) + ONE / 4);
}

/// Whether any shape's middle lies in the box `lo..hi` (both corners).
fn any_in(shapes: &[(Mesh, Affine, Rgb, i32)], lo: V3, hi: V3) -> bool {
    shapes.iter().any(|(_, a, _, _)| {
        let p = a.t;
        p.x > lo.x && p.x < hi.x && p.y > lo.y && p.y < hi.y && p.z > lo.z && p.z < hi.z
    })
}

#[test]
fn tower_1s_doorway_and_windows_are_open_through_the_wall() {
    let section = |cut: Cut| {
        let mut b = Batch::new();
        let mut cuts = [Cut::Wall; 8];
        cuts[0] = cut;
        hollow_section(
            &mut b,
            &Affine::IDENTITY,
            TOWER1_OUT,
            TOWER1_IN,
            SECTION_H,
            1,
            cuts,
            Ledge::None,
            Detail::Near,
        );
        all_shapes(b)
    };
    let (z0, z1) = (TOWER1_IN - int(1), TOWER1_OUT + int(1));
    // the doorway: nothing in the opening from the floor to under the lintel
    let door = section(Cut::Door);
    assert!(
        !any_in(
            &door,
            v3(-DOOR_W / 2 + int(1), int(1), z0),
            v3(DOOR_W / 2 - int(1), SECTION_H - int(3), z1)
        ),
        "the doorway is blocked"
    );
    // the window: nothing in its opening either
    let window = section(Cut::Window);
    let (lo, hi) = (SECTION_H / 2 - WINDOW_H / 2, SECTION_H / 2 + WINDOW_H / 2);
    assert!(
        !any_in(
            &window,
            v3(-WINDOW_W / 2 + int(1), lo + int(1), z0),
            v3(WINDOW_W / 2 - int(1), hi - int(1), z1)
        ),
        "the window is blocked"
    );
    // a plain wall fills that same space
    let wall = section(Cut::Wall);
    assert!(any_in(
        &wall,
        v3(-WINDOW_W / 2 + int(1), lo + int(1), z0),
        v3(WINDOW_W / 2 - int(1), hi - int(1), z1)
    ));
}

#[test]
fn the_bridge_hangers_hang_one_every_22_m_under_the_deck() {
    let (a, z) = (
        v3(int(100), int(55), int(10)),
        v3(int(350), int(55), int(10)),
    );
    let hooks = hanger_points(a, z, HANGER_SPACING);
    assert_eq!(hooks.len(), 11, "{hooks:?}");
    for (i, h) in hooks.iter().enumerate() {
        assert_eq!(h.y, int(55) - HANGER_DROP, "hook {i} is not under the deck");
        if i > 0 {
            let gap = h.dist(hooks[i - 1]);
            assert!(
                (gap - HANGER_SPACING).abs() < cm(5),
                "hook {i}: {} m from the last",
                gap / ONE
            );
        }
    }
    // drawn, the hangers return the same points
    let mut b = Batch::new();
    let drawn = bridge_hangers(
        &mut b,
        &Affine::IDENTITY,
        a,
        z,
        BRIDGE_WIDTH,
        HANGER_SPACING,
        1,
    );
    assert_eq!(drawn, hooks);
    assert!(b.count() >= 5 * hooks.len());
}

/// A void trampoline stays within its radius (a little over, for its
/// rim's padding), close to its face, and its bullseye glows (T100).
#[test]
fn a_void_trampoline_stays_on_its_disc_and_glows() {
    let r = crate::tower::DISC_RADIUS;
    let mut b = Batch::new();
    void_trampoline(&mut b, &Affine::IDENTITY, r, 0, 30, 1);
    let shapes = all_shapes(b);
    assert!(shapes
        .iter()
        .all(|(_, a, _, _)| off_axis(a.t) <= r + cm(10) && a.t.y.abs() <= int(1)));
    assert!(shapes.iter().any(|(_, _, _, glow)| *glow >= ONE / 2));
}

#[test]
fn the_portal_turns_and_breathes_and_shut_draws_nothing() {
    let draw = |tick: u32, open: i32| {
        let mut b = Batch::new();
        portal(
            &mut b,
            &Affine::IDENTITY,
            TEAR_W,
            TEAR_H,
            open,
            int(1),
            tick,
        );
        b
    };
    // the bands' streaks (the shards 20 cm deep) have turned after 20
    // ticks: their bearings round the middle differ (breathing alone would
    // only move them in and out)
    let bearings = |tick: u32| -> Vec<i32> {
        all_shapes(draw(tick, ONE))
            .into_iter()
            .filter(|(m, a, _, _)| *m == Mesh::Shard && a.c2.z == cm(20))
            .map(|(_, a, _, _)| crate::fx::atan2(a.t.x, a.t.y - TEAR_H / 2))
            .collect()
    };
    let (s0, s20) = (bearings(0), bearings(20));
    assert!(s0.len() >= 60, "{} streaks", s0.len());
    assert_ne!(s0, s20, "the bands do not turn");
    assert_eq!(draw(0, 0).count(), 0);
    assert!(draw(0, ONE).count() > 150);
    // the opening is clear through its middle, within the tear's size class
    let mid = portal_clear(TEAR_W, TEAR_H, TEAR_H / 2);
    assert!(mid > TEAR_W / 3 && mid < TEAR_W / 2, "{}", mid / ONE);
    assert_eq!(portal_clear(TEAR_W, TEAR_H, 0), 0);
    assert_eq!(portal_clear(TEAR_W, TEAR_H, TEAR_H), 0);
    assert!(portal_clear(TEAR_W, TEAR_H, TEAR_H / 10) < mid);
}

#[test]
fn the_tunnel_runs_level_from_its_mouth_under_the_anchor_bar() {
    let r = TUNNEL_RADIUS;
    let mut b = Batch::new();
    tunnel(&mut b, &Affine::IDENTITY, r, TUNNEL_LENGTH, 1);
    for (mesh, a, _, _) in all_shapes(b) {
        let p = a.t;
        assert!(
            p.z >= -int(1)
                && p.z <= TUNNEL_LENGTH + int(1)
                && p.y.abs() <= r + int(2)
                && p.x.abs() <= r + int(2),
            "{mesh:?} at {p:?} is outside the tunnel"
        );
    }
    let mut b = Batch::new();
    let (a, z) = tunnel_mouth(&mut b, &Affine::IDENTITY, r, 7, 1);
    assert!(
        a.y > r / 2 && a.y == z.y && a.x < 0 && z.x > 0 && a.z == 0,
        "{a:?} {z:?}"
    );
}
