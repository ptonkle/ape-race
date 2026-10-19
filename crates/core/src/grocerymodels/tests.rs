use super::gallery::sample;
use super::*;
use crate::fx::{int, ONE};
use crate::geom::{v3, Affine, V3};
use crate::render::Batch;

/// Every shape's transform in `b` (its halos left out).
fn shapes(mut b: Batch) -> Vec<Affine> {
    let _ = b.take_halos();
    let (v, _) = b.finish(0);
    v.chunks(crate::render::INSTANCE_VALUES)
        .map(|s| Affine {
            c0: v3(s[0], s[4], s[8]),
            c1: v3(s[1], s[5], s[9]),
            c2: v3(s[2], s[6], s[10]),
            t: v3(s[3], s[7], s[11]),
        })
        .collect()
}

/// The box round every shape: its unit cube's corners through each
/// transform (every mesh lies inside its unit cube).
fn bounds(all: &[Affine]) -> (V3, V3) {
    let mut lo = v3(i32::MAX, i32::MAX, i32::MAX);
    let mut hi = v3(i32::MIN, i32::MIN, i32::MIN);
    for a in all {
        for k in 0..8 {
            let h = ONE / 2;
            let c = v3(
                if k & 1 == 0 { -h } else { h },
                if k & 2 == 0 { -h } else { h },
                if k & 4 == 0 { -h } else { h },
            );
            let p = a.apply(c);
            lo = v3(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
            hi = v3(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
        }
    }
    (lo, hi)
}

const SLACK: i32 = crate::fx::cm(10);

/// `p`'s shapes at `detail`, standing on the origin facing +z, and whether
/// they all lie in its size box (give or take 10 cm): a message if not.
fn drawn(p: &Product, detail: Detail) -> (usize, Option<String>) {
    let mut b = Batch::new();
    product(&mut b, p, V3::ZERO, 0, detail);
    let all = shapes(b);
    let s = p.size_m();
    let (lo, hi) = bounds(&all);
    let inside = lo.x >= -s.x / 2 - SLACK
        && hi.x <= s.x / 2 + SLACK
        && lo.y >= -SLACK
        && hi.y <= s.y + SLACK
        && lo.z >= -s.z - SLACK
        && hi.z <= SLACK;
    let why = (!inside).then(|| {
        format!(
            "{} ({:?}, {:?}): drawn {:?}..{:?}, its box {:?}",
            p.name, p.shape, p.front, lo, hi, s
        )
    });
    (all.len(), why)
}

#[test]
fn every_product_is_one_block_far_and_at_most_six_shapes_near_inside_its_box() {
    let mut n = 0;
    for a in &AISLES {
        for p in a.products {
            let (block, why) = drawn(p, Detail::Block);
            assert_eq!(block, 1, "{} at Block", p.name);
            assert_eq!(why, None);
            let (full, why) = drawn(p, Detail::Full);
            assert!(
                (2..=FULL_MOST).contains(&full),
                "{} at Full: {full} shapes",
                p.name
            );
            assert_eq!(why, None);
            n += 1;
        }
    }
    assert_eq!(n, 94);
}

#[test]
fn every_shape_draws_with_every_front() {
    for (i, s) in Shape::ALL.iter().enumerate() {
        for f in Front::ALL {
            // a tall carton, and a flat one (a pouch lying down prints on top)
            for size in [[20, 26, 12], [24, 4, 20]] {
                let p = Product {
                    size,
                    ..sample(*s, f, i)
                };
                let (full, why) = drawn(&p, Detail::Full);
                assert!(
                    (2..=FULL_MOST).contains(&full),
                    "{s:?} {f:?} {size:?}: {full}"
                );
                assert_eq!(why, None);
            }
        }
    }
}

#[test]
fn a_shopper_keeps_to_the_budget_in_every_look_and_pose_and_holds_the_string() {
    let feet = v3(int(40), 0, -int(12));
    for look in 0..7u8 {
        for yaw in [0, crate::fx::deg(70)] {
            for stride in [0, ONE / 4, ONE / 2, ONE * 3 / 4] {
                for string in [
                    None,
                    Some(feet.add(v3(-int(3), int(23), int(2)))),
                    Some(feet.add(v3(int(4), int(22), -int(1)))),
                ] {
                    for cart in [false, true] {
                        let mut b = Batch::new();
                        shopper(&mut b, feet, yaw, look, stride, string, cart);
                        let all = shapes(b);
                        assert!(
                            all.len() <= SHOPPER_MOST,
                            "look {look} stride {stride} cart {cart}: {}",
                            all.len()
                        );
                        if let Some(end) = string {
                            assert!(
                                all.iter().any(|a| a.t.dist(end) < crate::fx::cm(5)),
                                "look {look}: no hand at the string's end"
                            );
                        }
                        let (lo, hi) = bounds(&all);
                        assert!(
                            lo.y >= -crate::fx::cm(10),
                            "look {look} stride {stride} cart {cart} yaw {yaw}: below the floor {}",
                            lo.y
                        );
                        if string.is_none() {
                            let h = crate::grocery::SHOPPER_HEIGHT;
                            assert!(
                                (hi.y - h).abs()
                                    < if stride == 0 {
                                        crate::fx::cm(60)
                                    } else {
                                        crate::fx::cm(160)
                                    },
                                "look {look} stride {stride}: {} tall",
                                hi.y
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn every_gallery_key_draws_something_and_is_framed() {
    for (i, key) in GALLERY.iter().enumerate() {
        assert!(!GALLERY[..i].contains(key), "{key} twice");
        let mut b = Batch::new();
        // the mister puffs at tick 0
        gallery(&mut b, key, &Affine::IDENTITY, 0);
        let halos = b.take_halos().count();
        assert!(b.count() + halos > 0, "{key} draws nothing");
        assert!(framing(key).1 > 0, "{key}");
    }
}

/// The box a fill model's shapes span, in its own frame.
fn spans(all: &[Affine]) -> (V3, V3) {
    bounds(all)
}

fn fills(lo: V3, hi: V3, want_lo: V3, want_hi: V3) -> bool {
    [
        (lo.x, want_lo.x),
        (lo.y, want_lo.y),
        (lo.z, want_lo.z),
        (hi.x, want_hi.x),
        (hi.y, want_hi.y),
        (hi.z, want_hi.z),
    ]
    .iter()
    .all(|(a, w)| (a - w).abs() <= SLACK)
}

#[test]
fn the_pallet_and_the_stock_cart_fill_their_boxes_exactly() {
    let size = crate::ravinemodels::SPHINX_SIZE;
    let mut b = Batch::new();
    pallet(&mut b, V3::ZERO, 0);
    let (lo, hi) = spans(&shapes(b));
    assert!(
        fills(
            lo,
            hi,
            v3(-size.x / 2, 0, -size.z),
            v3(size.x / 2, size.y, 0)
        ),
        "pallet spans {lo:?}..{hi:?}, the sphinx box is {size:?} behind its front"
    );
    for cart in [v3(int(10), int(9), int(24)), v3(int(7), int(4), int(14))] {
        let mut b = Batch::new();
        stock_cart(&mut b, V3::ZERO, 0, cart);
        let (lo, hi) = spans(&shapes(b));
        let half = v3(cart.x / 2, 0, cart.z / 2);
        assert!(
            fills(lo, hi, v3(-half.x, 0, -half.z), v3(half.x, cart.y, half.z)),
            "stock cart {cart:?} spans {lo:?}..{hi:?}"
        );
    }
}

#[test]
fn the_mister_puffs_then_stops_and_the_balloons_stay_over_their_knot() {
    let period = 360;
    let puffs = |tick: u32| {
        let mut b = Batch::new();
        mister(&mut b, v3(0, int(15), 0), tick, period, 30);
        b.take_halos().count() + b.count()
    };
    assert!(puffs(period - 30 + 10) > 0, "no puff at its start");
    assert_eq!(puffs(period - 30 + period / 2), 0, "a puff between puffs");
    for k in 0..6 {
        let anchor = v3(0, int(30), 0);
        let mut b = Batch::new();
        balloons(&mut b, anchor, 123 * k, k);
        let all = shapes(b);
        let balls = all.len() / 3;
        assert!((3..=5).contains(&balls), "bunch {k}: {balls} balloons");
        assert!(
            bounds(&all).0.y >= anchor.y - crate::fx::cm(10),
            "bunch {k} below its knot"
        );
    }
}

#[test]
fn a_produce_pile_stays_within_three_metres_of_its_table() {
    for stock in 0..7u8 {
        let (h, r) = (int(9), int(10));
        let mut b = Batch::new();
        table(&mut b, V3::ZERO, r, h, stock);
        let (lo, hi) = bounds(&shapes(b));
        assert!(
            hi.y <= h + props::PILE_MOST,
            "stock {stock}: pile to {}",
            hi.y
        );
        assert!(lo.y >= -crate::fx::cm(10), "stock {stock}: under the floor");
        assert!(
            hi.x <= r + crate::fx::cm(20),
            "stock {stock}: off the table"
        );
    }
}

// ---------------------------------------------------------------------------
// The second pass's models (`outside.rs`).

/// Every shape's glow in `b` (its halos left out).
fn glows(mut b: Batch) -> Vec<i32> {
    let _ = b.take_halos();
    let (v, _) = b.finish(0);
    v.chunks(crate::render::INSTANCE_VALUES)
        .map(|s| s[15])
        .collect()
}

#[test]
fn the_logo_is_at_most_twelve_shapes_flat_and_inside_its_ring() {
    for r in [crate::fx::cm(60), int(5)] {
        let mut b = Batch::new();
        banana_logo(&mut b, &Affine::IDENTITY, r);
        let all = shapes(b);
        assert!(all.len() <= 12, "logo {r}: {} shapes", all.len());
        let (lo, hi) = bounds(&all);
        assert!(
            lo.x >= -r - SLACK && hi.x <= r + SLACK && lo.y >= -r - SLACK && hi.y <= r + SLACK,
            "logo {r} spills out of its ring: {lo:?}..{hi:?}"
        );
        assert!(hi.z - lo.z <= r / 4, "logo {r} {} thick", hi.z - lo.z);
    }
}

#[test]
fn a_list_item_lit_glows_with_a_halo_and_sparkles_and_unlit_is_dim() {
    let at = v3(int(5), int(30), -int(8));
    for zone in 0..4u8 {
        let mut b = Batch::new();
        list_item(&mut b, at, zone, true, 77);
        let halos = b.take_halos().count();
        let n = b.count();
        assert!(n <= 20, "item {zone} lit: {n} shapes");
        assert!(halos >= 1, "item {zone} lit: no halo");
        assert!(
            glows(b).iter().all(|g| *g == ONE),
            "item {zone} lit: a part not glowing"
        );
        let mut b = Batch::new();
        list_item(&mut b, at, zone, false, 77);
        assert_eq!(b.take_halos().count(), 0, "item {zone} unlit: a halo");
        let mut c = Batch::new();
        list_item(&mut c, at, zone, false, 77);
        let (lo, hi) = bounds(&shapes(c));
        assert!(
            glows(b).iter().all(|g| *g == 0),
            "item {zone} unlit: glowing"
        );
        let tall = hi.y - lo.y;
        assert!(
            (int(3) + crate::fx::cm(50)..=int(4) + crate::fx::cm(60)).contains(&tall),
            "item {zone}: {tall} tall"
        );
        assert!(
            (lo.y - (at.y - tall / 2)).abs() < crate::fx::cm(80),
            "item {zone} not round its middle: {lo:?}..{hi:?}"
        );
        // its sparkles twinkle (and it turns)
        let mut a = Batch::new();
        list_item(&mut a, at, zone, true, 0);
        let mut z = Batch::new();
        list_item(&mut z, at, zone, true, 60);
        assert_ne!(shapes(a), shapes(z), "item {zone} still");
    }
}

#[test]
fn the_order_bag_stands_on_its_foot_about_four_metres_tall() {
    let foot = v3(int(3), int(10), int(4));
    let mut b = Batch::new();
    order_bag(&mut b, foot, crate::fx::deg(30));
    let all = shapes(b);
    assert!(all.len() <= 30, "bag: {}", all.len());
    let (lo, hi) = bounds(&all);
    assert!(lo.y >= foot.y - SLACK, "bag under its foot: {}", lo.y);
    assert!(
        (int(4)..=int(6) + crate::fx::cm(50)).contains(&(hi.y - foot.y)),
        "bag {} tall",
        hi.y - foot.y
    );
}

#[test]
fn the_plane_is_sixty_metres_across_its_propeller_turns_and_its_line_reaches_the_hook() {
    let at = v3(0, int(80), 0);
    let hook = at.add(v3(int(2), -int(25), -int(3)));
    let mut b = Batch::new();
    plane(&mut b, at, 0, 0, 0, 10, Some(hook));
    let halos = b.take_halos().count();
    let all = shapes(b);
    assert!(all.len() <= 60, "plane: {}", all.len());
    assert!(halos <= 2);
    let (lo, hi) = bounds(&all);
    let span = hi.x - lo.x;
    assert!(
        (span - int(60)).abs() <= crate::fx::cm(150),
        "plane spans {span}"
    );
    assert!(
        lo.y <= hook.y + SLACK && lo.y >= hook.y - int(2),
        "the hook at {} but the plane's lowest {}",
        hook.y,
        lo.y
    );
    let mut none = Batch::new();
    plane(&mut none, at, 0, 0, 0, 10, None);
    assert!(
        bounds(&shapes(none)).0.y > at.y - int(8),
        "a line without a hook"
    );
    let mut later = Batch::new();
    plane(&mut later, at, 0, 0, 0, 11, Some(hook));
    assert_ne!(shapes(later), all, "the propeller still");
    // nose up: its front (the shape farthest forward) climbs
    let mut up = Batch::new();
    plane(&mut up, at, 0, crate::fx::deg(20), 0, 10, None);
    let up = shapes(up);
    let nose = up.iter().max_by_key(|a| a.t.z).map_or(0, |a| a.t.y);
    assert!(nose > at.y + int(3), "pitch did not lift the nose: {nose}");
}

#[test]
fn every_car_look_fills_the_car_box_exactly_in_at_most_24_shapes() {
    let (len, wid, h) = crate::grocery::CAR_SIZE;
    for look in 0..8u8 {
        let mut b = Batch::new();
        car(&mut b, V3::ZERO, 0, look);
        let all = shapes(b);
        assert!(all.len() <= 24, "car {look}: {}", all.len());
        let (lo, hi) = bounds(&all);
        assert!(
            fills(lo, hi, v3(-wid / 2, 0, -len / 2), v3(wid / 2, h, len / 2)),
            "car {look} spans {lo:?}..{hi:?}"
        );
    }
}

#[test]
fn a_lamp_post_stands_on_its_foot_and_its_head_hangs_over_the_arm() {
    let foot = v3(int(4), 0, int(9));
    let arm = foot.add(v3(int(10), int(60), -int(4)));
    let mut b = Batch::new();
    lamp_post(&mut b, foot, int(60), arm);
    let all = shapes(b);
    assert!(all.len() <= 10, "lamp: {}", all.len());
    let (lo, hi) = bounds(&all);
    assert!(lo.y >= -SLACK && hi.y <= int(62), "lamp {lo:?}..{hi:?}");
    assert!(
        all.iter()
            .any(|a| a.t.flat().dist(arm.flat()) < crate::fx::cm(10)
                && (a.t.y - arm.y).abs() < int(1)),
        "no lamp head at the arm"
    );
}

#[test]
fn corrals_canopies_doors_and_ceilings_keep_to_their_budgets() {
    for len in [12, 30, 44, 80] {
        let mut b = Batch::new();
        corral(&mut b, v3(0, 0, 0), v3(int(len), 0, int(3)));
        assert!(b.count() <= 40, "corral {len} m: {}", b.count());
    }
    for (dx, dz) in [(30, 60), (40, 140)] {
        let (a, c) = (v3(-int(dx), 0, 0), v3(0, 0, int(dz)));
        let mut b = Batch::new();
        canopy(&mut b, a, c, int(30));
        let all = shapes(b);
        assert!(all.len() <= 30, "canopy {dx}×{dz}: {}", all.len());
        let (lo, hi) = bounds(&all);
        assert!(
            lo.x >= a.x - int(1) && hi.x <= c.x + SLACK && lo.z >= -SLACK && hi.z <= c.z + SLACK,
            "canopy spans {lo:?}..{hi:?}"
        );
    }
    let along = v3(0, 0, ONE);
    let draw = |open: i32| {
        let mut b = Batch::new();
        doors(&mut b, V3::ZERO, along, int(36), int(34), open);
        shapes(b)
    };
    let (shut, open) = (draw(0), draw(ONE));
    assert!(shut.len() <= 20 && open.len() <= 20, "doors {}", shut.len());
    // open, the leaves are out of the opening (in the wall)
    let half = int(18);
    let leaves = |all: &[Affine]| {
        all.iter()
            .filter(|a| a.t.y < int(34) && a.t.z.abs() < half)
            .count()
    };
    assert!(leaves(&shut) > leaves(&open) + 3, "the doors did not open");
    for (hw, hl) in [(35, 35), (35, 40), (60, 120)] {
        let mut b = Batch::new();
        let (w, l) = (int(hw), int(hl));
        low_ceiling(&mut b, v3(int(9), 0, 0), v3(0, 0, ONE), w, l, int(40));
        let all = shapes(b);
        assert!(all.len() <= 60, "ceiling {hw}×{hl}: {}", all.len());
        let (lo, hi) = bounds(&all);
        let near = crate::fx::cm(25);
        assert!(
            lo.y >= int(40) - near
                && hi.y <= crate::grocery::CEILING_AT + SLACK
                && lo.z >= -l - near
                && hi.z <= l + near,
            "ceiling {hw}×{hl} spans {lo:?}..{hi:?}"
        );
    }
}

#[test]
fn the_ice_wall_fills_its_box_exactly_in_at_most_60_shapes() {
    for (len, half, top) in [(80, 5, 18), (30, 7, 12), (120, 4, 20)] {
        let (from, to) = (v3(-int(len) / 2, 0, int(6)), v3(int(len) / 2, 0, int(6)));
        let mut b = Batch::new();
        ice_wall(&mut b, from, to, int(half), int(top));
        let all = shapes(b);
        assert!(all.len() <= 60, "ice {len}: {}", all.len());
        let (lo, hi) = bounds(&all);
        assert!(
            fills(
                lo,
                hi,
                v3(from.x, 0, int(6) - int(half)),
                v3(to.x, int(top), int(6) + int(half))
            ),
            "ice {len}×{half}×{top} spans {lo:?}..{hi:?}"
        );
    }
}

#[test]
fn the_sign_arm_hangs_from_the_ceiling_and_the_weight_sits_on_its_foot() {
    let anchor = v3(int(3), int(38), -int(7));
    let mut b = Batch::new();
    sign_arm(&mut b, anchor, v3(ONE, 0, 0));
    let all = shapes(b);
    assert!(all.len() <= 40, "sign arm: {}", all.len());
    let (lo, hi) = bounds(&all);
    assert!(
        (hi.y - crate::grocery::CEILING_AT).abs() <= SLACK,
        "chains to {}",
        hi.y
    );
    assert!(
        hi.x <= anchor.x + crate::fx::cm(50),
        "the sign past the rope's way: {hi:?}"
    );
    assert!(
        lo.x < anchor.x - int(5),
        "the sign is not behind its arm: {lo:?}"
    );
    assert!(
        all.iter().any(|a| a.t.dist(anchor) < crate::fx::cm(5)),
        "nothing at the anchor"
    );
    let at = v3(int(2), int(1), int(3));
    let mut b = Batch::new();
    balloon_weight(&mut b, at);
    let all = shapes(b);
    assert!(all.len() <= 8, "weight: {}", all.len());
    let (lo, hi) = bounds(&all);
    assert!(
        (lo.y - at.y).abs() <= SLACK && hi.y - at.y <= int(3),
        "weight spans {lo:?}..{hi:?}"
    );
}
