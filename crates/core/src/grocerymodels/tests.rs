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
